//! Evidence-backed, per-container multi-release JAR selection.

use crate::artifact::{
    ArtifactKind, ArtifactSnapshot, ArtifactTreeReport, EnumerationReport, PhysicalEntry,
    budget_dimension_code,
};
use crate::budget::{Budget, BudgetDimension, CountedBudgetDimension};
use crate::classfile::{VerificationStatus, probe_minimal_header};
use crate::error::{Error, Result};
use crate::model::{
    ArchiveNameBytes, ContainerOrigin, Coverage, CoverageDimension, CoverageRange, CoverageState,
    Diagnostic, DiagnosticSeverity, ExecutionReport, JvmString, Location, PhysicalEntryId,
    Provenance, TerminationReason,
};
use crate::view::{MultiReleasePolicy, PhysicalScope, RuntimeView};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, HashMap};

const MANIFEST: &[u8] = b"META-INF/MANIFEST.MF";
const VERSION_PREFIX: &[u8] = b"META-INF/versions/";
const ACC_PUBLIC: u16 = 0x0001;
const SELECTION_METRIC: &str = "multi_release_selection_entries";
const COMPLIANCE_METRIC: &str = "multi_release_compliance_entries";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiReleaseViewReport {
    pub view: RuntimeView,
    pub physical: MultiReleasePhysicalEvidence,
    pub containers: Vec<MultiReleaseContainerReport>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<MultiReleaseReportDiagnostic>,
    pub verification: VerificationStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MultiReleasePhysicalEvidence {
    Snapshot { report: EnumerationReport },
    ArtifactTree { report: ArtifactTreeReport },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiReleaseContainerReport {
    pub origin: ContainerOrigin,
    pub manifest: ManifestEvidence,
    pub entries: Vec<MultiReleaseEntryEvidence>,
    pub selections: Vec<MultiReleaseSelection>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestEvidence {
    pub entries: Vec<PhysicalEntryId>,
    pub state: ManifestState,
    pub attribute_name: Option<ArchiveNameBytes>,
    pub attribute_value: Option<ArchiveNameBytes>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestState {
    Missing,
    Active,
    Inactive,
    Ambiguous,
    Malformed,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiReleaseEntryEvidence {
    pub entry: PhysicalEntryId,
    pub logical_path: Option<ArchiveNameBytes>,
    pub variant: MultiReleaseEntryVariant,
    pub decision: MultiReleaseSelectionDecision,
    pub compliance: MultiReleaseCompliance,
    pub class_evidence: Option<MultiReleaseClassEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiReleaseClassEvidence {
    pub major_version: u16,
    pub minor_version: u16,
    pub access_flags: u16,
    pub this_class: JvmString,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MultiReleaseEntryVariant {
    Base,
    Versioned { release: u64 },
    InvalidVersioned { issue: MultiReleaseVersionPathIssue },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultiReleaseVersionPathIssue {
    EmptyRelease,
    NonDecimalRelease,
    LeadingZeroRelease,
    ReleaseBelowNine,
    ReleaseOverflow,
    EmptyLogicalPath,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiReleaseSelection {
    pub logical_path: ArchiveNameBytes,
    pub outcome: MultiReleaseSelectionOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MultiReleaseSelectionOutcome {
    Selected {
        entry: PhysicalEntryId,
    },
    Ambiguous {
        entries: Vec<PhysicalEntryId>,
    },
    NoSelection {
        reason: MultiReleaseNoSelectionReason,
    },
    Unknown {
        reason: MultiReleaseUnknownReason,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MultiReleaseSelectionDecision {
    Selected,
    Shadowed {
        winners: Vec<PhysicalEntryId>,
    },
    Inactive {
        reason: MultiReleaseInactiveReason,
    },
    Ambiguous,
    Unknown {
        reason: MultiReleaseUnknownReason,
    },
    NotApplicable {
        reason: MultiReleaseNotApplicableReason,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultiReleaseInactiveReason {
    PolicyDisabled,
    ManifestMissing,
    ManifestInactive,
    TargetBelowNine,
    ReleaseAboveTarget,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultiReleaseNoSelectionReason {
    NoBaseWhileInactive,
    NoApplicableRelease,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultiReleaseUnknownReason {
    PhysicalEvidenceIncomplete,
    ManifestEvidenceAmbiguous,
    ManifestEvidenceMalformed,
    ManifestEvidenceUnreadable,
    CustomPolicy,
    UnknownPolicy,
    ProcessingInterrupted,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultiReleaseNotApplicableReason {
    InvalidVersionPath,
    VersionedMetaInfResource,
    DirectoryEntry,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MultiReleaseCompliance {
    NotApplicable,
    ConformantWithinChecks,
    NonConformant,
    Unknown {
        reason: MultiReleaseComplianceUnknownReason,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultiReleaseComplianceUnknownReason {
    PhysicalEvidenceIncomplete,
    ProbeInterrupted,
    PredecessorAmbiguous,
    ModuleExportsNotInspected,
    PolicyUnsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MultiReleaseReportDiagnostic {
    Domain { diagnostic: MultiReleaseDiagnostic },
    Terminal { diagnostic: Diagnostic },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultiReleaseDiagnosticCode {
    MultiReleaseManifestNoncanonicalPath,
    MultiReleaseManifestDuplicate,
    MultiReleaseManifestMalformed,
    MultiReleaseVersionPathInvalid,
    MultiReleaseVersionBelowNine,
    MultiReleaseVersionOverflow,
    MultiReleaseMetaInfResource,
    MultiReleaseDuplicateCandidate,
    MultiReleaseClassMalformed,
    MultiReleaseClassVersionTooNew,
    MultiReleasePublicPredecessorMissing,
    MultiReleasePublicPredecessorMismatch,
}

impl MultiReleaseDiagnosticCode {
    fn severity(self) -> DiagnosticSeverity {
        if self == Self::MultiReleaseManifestNoncanonicalPath {
            DiagnosticSeverity::Warning
        } else {
            DiagnosticSeverity::Error
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MultiReleaseDiagnostic {
    pub code: MultiReleaseDiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub provenance: Option<Provenance>,
}

impl MultiReleaseDiagnostic {
    pub fn new(
        code: MultiReleaseDiagnosticCode,
        message: impl Into<String>,
        provenance: Option<Provenance>,
    ) -> Self {
        Self {
            code,
            severity: code.severity(),
            message: message.into(),
            provenance,
        }
    }
}

impl<'de> Deserialize<'de> for MultiReleaseDiagnostic {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            code: MultiReleaseDiagnosticCode,
            severity: DiagnosticSeverity,
            message: String,
            provenance: Option<Provenance>,
        }
        let raw = Raw::deserialize(deserializer)?;
        if raw.severity != raw.code.severity() {
            return Err(D::Error::custom(
                "multi-release diagnostic code/severity mismatch",
            ));
        }
        Ok(Self {
            code: raw.code,
            severity: raw.severity,
            message: raw.message,
            provenance: raw.provenance,
        })
    }
}

#[derive(Clone)]
struct Classified<'a> {
    physical: &'a PhysicalEntry,
    logical: Option<Vec<u8>>,
    variant: MultiReleaseEntryVariant,
    meta_inf: bool,
    directory: bool,
}

pub fn select(
    snapshot: &ArtifactSnapshot,
    view: &RuntimeView,
    budget: &mut Budget,
) -> Result<MultiReleaseViewReport> {
    if snapshot.kind() != ArtifactKind::Zip {
        return Err(Error::invalid_input(
            "multi_release_not_zip",
            "multi-release selection requires a ZIP snapshot",
        ));
    }
    if &view.physical.snapshot != snapshot.id() {
        return Err(Error::invalid_input(
            "multi_release_snapshot_mismatch",
            "RuntimeView snapshot does not match the artifact snapshot",
        ));
    }
    let physical = match &view.physical.scope {
        PhysicalScope::SnapshotAll => MultiReleasePhysicalEvidence::Snapshot {
            report: snapshot.enumerate(budget)?,
        },
        PhysicalScope::ArtifactTree { root_container } => {
            if root_container.0 != "root" {
                return Err(Error::invalid_input(
                    "multi_release_root_container_mismatch",
                    "RuntimeView tree root does not match the snapshot root container",
                ));
            }
            let report = snapshot.enumerate_artifact_tree(budget)?;
            if report.view.scope != view.physical.scope {
                return Err(Error::invalid_input(
                    "multi_release_root_container_mismatch",
                    "RuntimeView tree root does not match the snapshot root container",
                ));
            }
            if report
                .containers
                .first()
                .is_some_and(|c| c.origin.current_container() != root_container)
            {
                return Err(Error::invalid_input(
                    "multi_release_root_container_mismatch",
                    "RuntimeView tree root does not match the snapshot root container",
                ));
            }
            MultiReleasePhysicalEvidence::ArtifactTree { report }
        }
    };
    let mut reports = Vec::new();
    let mut diagnostics = Vec::new();
    let mut runtime_scanned = Vec::new();
    let mut runtime_skipped = Vec::new();
    // The aggregate only reports the outcome; it is preset with the physical provider's own
    // execution and must never decide whether this run may keep inspecting containers.
    let mut aggregate = Issues::new(physical_execution(&physical));
    let mut pending = container_inputs(snapshot, &physical).into_iter();
    for input in pending.by_ref() {
        if !produces_report(&input.coverage, &input.execution) {
            // An established container that enumerated no ordinal and did not complete
            // cannot support a Manifest/selection claim; only its physical evidence stands.
            declare_unprocessed(&input, &mut runtime_skipped);
            if blocks(priority_of(&input.execution)) {
                break;
            }
            continue;
        }
        if let Err(error) = budget.charge(CountedBudgetDimension::ResultItems, 2) {
            push_terminal(&mut diagnostics, &error, None);
            aggregate.record(&error, budget);
            declare_unprocessed(&input, &mut runtime_skipped);
            break;
        }
        let processed = process_container(
            snapshot,
            view,
            input.origin,
            input.entries,
            input.coverage,
            input.execution,
            budget,
        );
        diagnostics.extend(processed.diagnostics);
        runtime_scanned.extend(processed.scanned);
        runtime_skipped.extend(processed.skipped);
        aggregate.merge(&processed.report.execution);
        // Only this container's own interruption may stop the container loop: a complete
        // container is worth inspecting even when the physical aggregate already stopped.
        let stop = blocks(priority_of(&processed.report.execution));
        reports.push(processed.report);
        if stop {
            break;
        }
    }
    // Every container this run never processed — rejected by the report rule above, refused
    // its reservation, or never reached — still declares its known ordinals as skipped, on
    // top of the physical suffix its provider did not enumerate.
    for input in pending {
        declare_unprocessed(&input, &mut runtime_skipped);
    }
    let execution = aggregate.finish(budget);
    let structural = physical_coverage(&physical).artifact_structural.clone();
    let complete = matches!(execution, ExecutionReport::Complete { .. });
    Ok(MultiReleaseViewReport {
        view: view.clone(),
        physical,
        containers: reports,
        coverage: Coverage {
            artifact_structural: structural,
            runtime_resolution: CoverageDimension {
                state: if complete {
                    CoverageState::CompleteWithinSchema
                } else {
                    CoverageState::Partial
                },
                scanned: runtime_scanned,
                skipped: runtime_skipped,
                uninterpreted_extensions: Vec::new(),
            },
            dynamic_analysis: CoverageDimension::not_requested(),
        },
        execution,
        diagnostics,
        verification: VerificationStatus::NotPerformed,
    })
}

struct ContainerInput {
    origin: ContainerOrigin,
    entries: Vec<PhysicalEntry>,
    coverage: Coverage,
    execution: ExecutionReport,
}

fn container_inputs(
    snapshot: &ArtifactSnapshot,
    physical: &MultiReleasePhysicalEvidence,
) -> Vec<ContainerInput> {
    match physical {
        MultiReleasePhysicalEvidence::Snapshot { report } => vec![ContainerInput {
            origin: root_origin(snapshot),
            entries: report.entries.clone(),
            coverage: report.coverage.clone(),
            execution: report.execution.clone(),
        }],
        MultiReleasePhysicalEvidence::ArtifactTree { report } => report
            .containers
            .iter()
            .map(|container| ContainerInput {
                origin: container.origin.clone(),
                entries: container.entries.clone(),
                coverage: container.coverage.clone(),
                execution: container.execution.clone(),
            })
            .collect(),
    }
}

struct Processed {
    report: MultiReleaseContainerReport,
    diagnostics: Vec<MultiReleaseReportDiagnostic>,
    scanned: Vec<CoverageRange>,
    skipped: Vec<CoverageRange>,
}

fn process_container(
    snapshot: &ArtifactSnapshot,
    view: &RuntimeView,
    origin: ContainerOrigin,
    entries: Vec<PhysicalEntry>,
    physical_coverage: Coverage,
    physical_execution: ExecutionReport,
    budget: &mut Budget,
) -> Processed {
    let physical_complete = matches!(physical_execution, ExecutionReport::Complete { .. });
    let unsupported = unsupported_policy(view);
    let classified: Vec<_> = entries.iter().map(classify).collect();
    let manifest_indexes: Vec<usize> = classified
        .iter()
        .enumerate()
        .filter(|(_, entry)| ascii_eq(&entry.physical.id.raw_name.0, MANIFEST))
        .map(|(index, _)| index)
        .collect();
    let mut diagnostics = Vec::new();
    let mut issues = Issues::new(&physical_execution);
    let manifest_outcome = match unsupported {
        Some(_) => ManifestOutcome::preserved(ManifestEvidence {
            entries: manifest_indexes
                .iter()
                .map(|index| classified[*index].physical.id.clone())
                .collect(),
            state: ManifestState::Unknown,
            attribute_name: None,
            attribute_value: None,
        }),
        None => read_manifest(
            snapshot,
            &classified,
            &manifest_indexes,
            physical_complete,
            budget,
            &mut diagnostics,
            &mut issues,
        ),
    };
    let manifest = manifest_outcome.evidence.clone();
    let mut evidence: Vec<MultiReleaseEntryEvidence> = classified
        .iter()
        .map(|entry| {
            initial_evidence(
                entry,
                unsupported.map(|(reason, _)| reason),
                physical_complete,
                &manifest,
                view,
            )
        })
        .collect();
    let (group_of, groups) = collect_groups(&classified);
    if !issues.items_exhausted() {
        mark_duplicates(
            &classified,
            &mut evidence,
            &mut diagnostics,
            budget,
            &mut issues,
        );
    }
    let outcomes: Vec<MultiReleaseSelectionOutcome> = groups
        .iter()
        .map(|group| {
            choose(
                &group.indexes,
                &classified,
                &mut evidence,
                view,
                physical_complete,
                &manifest,
                unsupported.map(|(reason, _)| reason),
            )
        })
        .collect();
    let mut returned: Vec<MultiReleaseEntryEvidence> = Vec::new();
    if !issues.items_exhausted() {
        let mut index = 0;
        for item in evidence {
            match budget.charge(CountedBudgetDimension::ResultItems, 1) {
                Ok(()) => {
                    returned.push(item);
                    index += 1;
                }
                Err(error) => {
                    push_terminal(&mut diagnostics, &error, Some(classified[index].physical));
                    issues.record(&error, budget);
                    break;
                }
            }
        }
    }
    let mut selections = Vec::new();
    let mut selection_halt = None;
    if !issues.items_exhausted() {
        for (position, group) in groups.iter().enumerate() {
            match budget.charge(CountedBudgetDimension::ResultItems, 1) {
                Ok(()) => selections.push(MultiReleaseSelection {
                    logical_path: ArchiveNameBytes(group.logical.clone()),
                    outcome: outcomes[position].clone(),
                }),
                Err(error) => {
                    push_terminal(
                        &mut diagnostics,
                        &error,
                        Some(classified[group.indexes[0]].physical),
                    );
                    issues.record(&error, budget);
                    selection_halt = Some(group.min_ordinal);
                    break;
                }
            }
        }
    }
    let mut probe_states = vec![ProbeState::Untouched; classified.len()];
    if !issues.items_exhausted() && !issues.blocked() && physical_complete && unsupported.is_none()
    {
        probe_compliance(
            snapshot,
            &classified,
            &mut returned,
            &mut probe_states,
            budget,
            &mut diagnostics,
            &mut issues,
        );
    }
    if let Some((_, code)) = unsupported {
        issues.override_with(ExecutionReport::Failed {
            reason: TerminationReason::Unsupported { code: code.into() },
            usage: budget.usage(),
        });
    }
    let execution = issues.finish(budget);
    let (scanned, mut skipped) = coverage_ranges(
        &origin,
        &classified,
        &returned,
        &probe_states,
        selection_halt,
        &groups,
        &group_of,
        manifest_outcome.read_interrupted,
    );
    skipped.extend(physical_coverage.artifact_structural.skipped.clone());
    let coverage = Coverage {
        artifact_structural: physical_coverage.artifact_structural,
        runtime_resolution: CoverageDimension {
            state: if matches!(execution, ExecutionReport::Complete { .. }) {
                CoverageState::CompleteWithinSchema
            } else {
                CoverageState::Partial
            },
            scanned: scanned.clone(),
            skipped: skipped.clone(),
            uninterpreted_extensions: Vec::new(),
        },
        dynamic_analysis: CoverageDimension::not_requested(),
    };
    Processed {
        report: MultiReleaseContainerReport {
            origin,
            manifest,
            entries: returned,
            selections,
            coverage,
            execution,
        },
        diagnostics,
        scanned,
        skipped,
    }
}

/// A logical-path group of physical candidates, ordered by its minimum ordinal.
struct Group {
    min_ordinal: u64,
    logical: Vec<u8>,
    indexes: Vec<usize>,
}

fn collect_groups(classified: &[Classified<'_>]) -> (Vec<Option<usize>>, Vec<Group>) {
    let mut lookup: HashMap<Vec<u8>, usize> = HashMap::new();
    let mut group_of = vec![None; classified.len()];
    let mut groups: Vec<Group> = Vec::new();
    for (index, entry) in classified.iter().enumerate() {
        if entry.directory
            || entry.meta_inf
            || matches!(
                entry.variant,
                MultiReleaseEntryVariant::InvalidVersioned { .. }
            )
        {
            continue;
        }
        let Some(path) = entry.logical.clone() else {
            continue;
        };
        match lookup.get(&path) {
            Some(position) => {
                groups[*position].indexes.push(index);
                group_of[index] = Some(*position);
            }
            None => {
                let position = groups.len();
                groups.push(Group {
                    min_ordinal: entry.physical.id.ordinal,
                    logical: path.clone(),
                    indexes: vec![index],
                });
                lookup.insert(path, position);
                group_of[index] = Some(position);
            }
        }
    }
    (group_of, groups)
}

#[allow(clippy::too_many_arguments)]
fn coverage_ranges(
    origin: &ContainerOrigin,
    classified: &[Classified<'_>],
    returned: &[MultiReleaseEntryEvidence],
    probe_states: &[ProbeState],
    selection_halt: Option<u64>,
    groups: &[Group],
    group_of: &[Option<usize>],
    manifest_read_interrupted: bool,
) -> (Vec<CoverageRange>, Vec<CoverageRange>) {
    let mut scanned = Vec::new();
    let mut skipped = Vec::new();
    for (index, entry) in classified.iter().enumerate() {
        let ordinal = entry.physical.id.ordinal;
        let decision_returned = match group_of[index] {
            None => true,
            Some(position) => selection_halt.is_none_or(|halt| groups[position].min_ordinal < halt),
        };
        // A Manifest candidate whose read was interrupted has no Manifest decision
        // evidence, so its ordinal is not claimed as scanned path evidence.
        let manifest_interrupted =
            manifest_read_interrupted && ascii_eq(&entry.physical.id.raw_name.0, MANIFEST);
        if index < returned.len() && decision_returned && !manifest_interrupted {
            scanned.push(range(origin, SELECTION_METRIC, ordinal));
        } else {
            skipped.push(range(origin, SELECTION_METRIC, ordinal));
        }
        // A probe target is declared even before it was touched, and an ordinal that was
        // probed in another role — a Base serving as a public predecessor — is declared
        // too, so an attempted check never disappears from the compliance dimension.
        let probe_state = probe_states[index];
        if is_probe_applicable(entry) || probe_state != ProbeState::Untouched {
            if probe_state == ProbeState::Completed {
                scanned.push(range(origin, COMPLIANCE_METRIC, ordinal));
            } else {
                skipped.push(range(origin, COMPLIANCE_METRIC, ordinal));
            }
        }
    }
    (scanned, skipped)
}

/// How far the bounded Header probe got for one ordinal, which decides the compliance
/// dimension of the runtime coverage: a completed check is scanned, an attempted but
/// unfinished one is skipped, and an ordinal the probe never touched is only declared when
/// it is a probe target by itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProbeState {
    /// The probe never looked at this ordinal.
    Untouched,
    /// The probe was attempted but was interrupted before it could produce a verdict.
    Interrupted,
    /// The probe ran to a verdict, which is recorded on the entry evidence.
    Completed,
}

fn unsupported_policy(view: &RuntimeView) -> Option<(MultiReleaseUnknownReason, &'static str)> {
    match view.profile.multi_release {
        MultiReleasePolicy::Custom { .. } => Some((
            MultiReleaseUnknownReason::CustomPolicy,
            "multi_release_custom_policy",
        )),
        MultiReleasePolicy::Unknown => Some((
            MultiReleaseUnknownReason::UnknownPolicy,
            "multi_release_unknown_policy",
        )),
        MultiReleasePolicy::Disabled | MultiReleasePolicy::Enabled => None,
    }
}

fn is_probe_applicable(entry: &Classified<'_>) -> bool {
    matches!(entry.variant, MultiReleaseEntryVariant::Versioned { .. })
        && !entry.meta_inf
        && entry
            .logical
            .as_deref()
            .is_some_and(|path| path.ends_with(b".class"))
}

fn produces_report(coverage: &Coverage, execution: &ExecutionReport) -> bool {
    coverage
        .artifact_structural
        .scanned
        .iter()
        .any(|range| range.end > range.start)
        || matches!(execution, ExecutionReport::Complete { .. })
}

fn classify(entry: &PhysicalEntry) -> Classified<'_> {
    let name = &entry.id.raw_name.0;
    if !name.is_empty() && name.ends_with(b"/") {
        return Classified {
            physical: entry,
            logical: None,
            variant: MultiReleaseEntryVariant::Base,
            meta_inf: false,
            directory: true,
        };
    }
    if let Some(rest) = name.strip_prefix(VERSION_PREFIX) {
        let (number, logical) = rest
            .iter()
            .position(|b| *b == b'/')
            .map_or((rest, &b""[..]), |position| {
                (&rest[..position], &rest[position + 1..])
            });
        let issue = if number.is_empty() {
            Some(MultiReleaseVersionPathIssue::EmptyRelease)
        } else if number.iter().any(|b| !b.is_ascii_digit()) {
            Some(MultiReleaseVersionPathIssue::NonDecimalRelease)
        } else if number.len() > 1 && number[0] == b'0' {
            Some(MultiReleaseVersionPathIssue::LeadingZeroRelease)
        } else {
            match std::str::from_utf8(number)
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
            {
                None => Some(MultiReleaseVersionPathIssue::ReleaseOverflow),
                Some(n) if n < 9 => Some(MultiReleaseVersionPathIssue::ReleaseBelowNine),
                Some(_) if logical.is_empty() => {
                    Some(MultiReleaseVersionPathIssue::EmptyLogicalPath)
                }
                Some(n) => {
                    return Classified {
                        physical: entry,
                        logical: Some(logical.to_vec()),
                        variant: MultiReleaseEntryVariant::Versioned { release: n },
                        meta_inf: logical.starts_with(b"META-INF/"),
                        directory: false,
                    };
                }
            }
        };
        return Classified {
            physical: entry,
            logical: None,
            variant: MultiReleaseEntryVariant::InvalidVersioned {
                issue: issue.unwrap(),
            },
            meta_inf: false,
            directory: false,
        };
    }
    Classified {
        physical: entry,
        logical: Some(name.clone()),
        variant: MultiReleaseEntryVariant::Base,
        meta_inf: false,
        directory: false,
    }
}

fn initial_evidence(
    c: &Classified<'_>,
    unsupported: Option<MultiReleaseUnknownReason>,
    physical_complete: bool,
    manifest: &ManifestEvidence,
    view: &RuntimeView,
) -> MultiReleaseEntryEvidence {
    let (decision, compliance) = if c.directory {
        (
            MultiReleaseSelectionDecision::NotApplicable {
                reason: MultiReleaseNotApplicableReason::DirectoryEntry,
            },
            MultiReleaseCompliance::NotApplicable,
        )
    } else if matches!(c.variant, MultiReleaseEntryVariant::InvalidVersioned { .. }) {
        (
            MultiReleaseSelectionDecision::NotApplicable {
                reason: MultiReleaseNotApplicableReason::InvalidVersionPath,
            },
            MultiReleaseCompliance::NonConformant,
        )
    } else if c.meta_inf {
        (
            MultiReleaseSelectionDecision::NotApplicable {
                reason: MultiReleaseNotApplicableReason::VersionedMetaInfResource,
            },
            MultiReleaseCompliance::NonConformant,
        )
    } else if let Some(reason) = unsupported {
        (
            MultiReleaseSelectionDecision::Unknown { reason },
            MultiReleaseCompliance::Unknown {
                reason: MultiReleaseComplianceUnknownReason::PolicyUnsupported,
            },
        )
    } else if !physical_complete {
        (
            MultiReleaseSelectionDecision::Unknown {
                reason: MultiReleaseUnknownReason::PhysicalEvidenceIncomplete,
            },
            MultiReleaseCompliance::Unknown {
                reason: MultiReleaseComplianceUnknownReason::PhysicalEvidenceIncomplete,
            },
        )
    } else if manifest.state == ManifestState::Unknown {
        (
            MultiReleaseSelectionDecision::Unknown {
                reason: MultiReleaseUnknownReason::ManifestEvidenceUnreadable,
            },
            MultiReleaseCompliance::Unknown {
                reason: MultiReleaseComplianceUnknownReason::ProbeInterrupted,
            },
        )
    } else {
        // A versioned class is only ever Conformant once a bounded Header probe really
        // proved it; until then the compliance evidence stays unknown.
        let compliance = if matches!(c.variant, MultiReleaseEntryVariant::Versioned { .. }) {
            if c.logical
                .as_deref()
                .is_some_and(|path| path.ends_with(b".class"))
            {
                MultiReleaseCompliance::Unknown {
                    reason: MultiReleaseComplianceUnknownReason::ProbeInterrupted,
                }
            } else {
                MultiReleaseCompliance::ConformantWithinChecks
            }
        } else {
            MultiReleaseCompliance::NotApplicable
        };
        (default_decision(c, manifest, view), compliance)
    };
    MultiReleaseEntryEvidence {
        entry: c.physical.id.clone(),
        logical_path: c.logical.clone().map(ArchiveNameBytes),
        variant: c.variant.clone(),
        decision,
        compliance,
        class_evidence: None,
    }
}

fn default_decision(
    c: &Classified<'_>,
    manifest: &ManifestEvidence,
    view: &RuntimeView,
) -> MultiReleaseSelectionDecision {
    match &c.variant {
        MultiReleaseEntryVariant::Base => MultiReleaseSelectionDecision::Selected,
        MultiReleaseEntryVariant::Versioned { release } => {
            let reason = if matches!(view.profile.multi_release, MultiReleasePolicy::Disabled) {
                MultiReleaseInactiveReason::PolicyDisabled
            } else if manifest.state == ManifestState::Missing {
                MultiReleaseInactiveReason::ManifestMissing
            } else if manifest.state == ManifestState::Inactive {
                MultiReleaseInactiveReason::ManifestInactive
            } else if view.profile.java_release < 9 {
                MultiReleaseInactiveReason::TargetBelowNine
            } else if *release > u64::from(view.profile.java_release) {
                MultiReleaseInactiveReason::ReleaseAboveTarget
            } else {
                return MultiReleaseSelectionDecision::Selected;
            };
            MultiReleaseSelectionDecision::Inactive { reason }
        }
        MultiReleaseEntryVariant::InvalidVersioned { .. } => unreachable!(),
    }
}

fn choose(
    indexes: &[usize],
    c: &[Classified<'_>],
    evidence: &mut [MultiReleaseEntryEvidence],
    view: &RuntimeView,
    physical_complete: bool,
    manifest: &ManifestEvidence,
    unsupported: Option<MultiReleaseUnknownReason>,
) -> MultiReleaseSelectionOutcome {
    let unknown = unsupported
        .or((!physical_complete).then_some(MultiReleaseUnknownReason::PhysicalEvidenceIncomplete))
        .or(match manifest.state {
            ManifestState::Ambiguous => Some(MultiReleaseUnknownReason::ManifestEvidenceAmbiguous),
            ManifestState::Malformed => Some(MultiReleaseUnknownReason::ManifestEvidenceMalformed),
            ManifestState::Unknown => Some(MultiReleaseUnknownReason::ManifestEvidenceUnreadable),
            _ => None,
        });
    if let Some(reason) = unknown {
        for &i in indexes {
            evidence[i].decision = MultiReleaseSelectionDecision::Unknown { reason };
        }
        return MultiReleaseSelectionOutcome::Unknown { reason };
    }
    let active = matches!(view.profile.multi_release, MultiReleasePolicy::Enabled)
        && manifest.state == ManifestState::Active
        && view.profile.java_release >= 9;
    let mut levels: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
    for &i in indexes {
        let level = match c[i].variant {
            MultiReleaseEntryVariant::Base => 0,
            MultiReleaseEntryVariant::Versioned { release } => release,
            _ => continue,
        };
        levels.entry(level).or_default().push(i);
    }
    let winner_level = if active {
        levels
            .keys()
            .copied()
            .filter(|n| *n == 0 || *n <= u64::from(view.profile.java_release))
            .max()
    } else {
        levels.contains_key(&0).then_some(0)
    };
    let Some(level) = winner_level else {
        return MultiReleaseSelectionOutcome::NoSelection {
            reason: if active {
                MultiReleaseNoSelectionReason::NoApplicableRelease
            } else {
                MultiReleaseNoSelectionReason::NoBaseWhileInactive
            },
        };
    };
    let winners = levels[&level]
        .iter()
        .map(|i| c[*i].physical.id.clone())
        .collect::<Vec<_>>();
    for &i in indexes {
        if levels[&level].contains(&i) {
            evidence[i].decision = if winners.len() == 1 {
                MultiReleaseSelectionDecision::Selected
            } else {
                MultiReleaseSelectionDecision::Ambiguous
            };
        } else if matches!(c[i].variant, MultiReleaseEntryVariant::Versioned { release } if !active || release > u64::from(view.profile.java_release))
        {
            evidence[i].decision = default_decision(&c[i], manifest, view);
        } else {
            evidence[i].decision = MultiReleaseSelectionDecision::Shadowed {
                winners: winners.clone(),
            };
        }
    }
    if winners.len() == 1 {
        MultiReleaseSelectionOutcome::Selected {
            entry: winners[0].clone(),
        }
    } else {
        MultiReleaseSelectionOutcome::Ambiguous { entries: winners }
    }
}

/// Manifest evidence plus whether the candidate read itself was interrupted, which decides
/// whether the Manifest ordinal can be claimed as scanned path evidence.
struct ManifestOutcome {
    evidence: ManifestEvidence,
    read_interrupted: bool,
}

impl ManifestOutcome {
    fn preserved(evidence: ManifestEvidence) -> Self {
        Self {
            evidence,
            read_interrupted: false,
        }
    }

    fn interrupted(evidence: ManifestEvidence) -> Self {
        Self {
            evidence,
            read_interrupted: true,
        }
    }
}

fn read_manifest(
    snapshot: &ArtifactSnapshot,
    classified: &[Classified<'_>],
    manifest_indexes: &[usize],
    physical_complete: bool,
    budget: &mut Budget,
    diagnostics: &mut Vec<MultiReleaseReportDiagnostic>,
    issues: &mut Issues,
) -> ManifestOutcome {
    let entries = manifest_indexes
        .iter()
        .map(|index| classified[*index].physical.id.clone())
        .collect::<Vec<_>>();
    let unknown = |state: ManifestState| ManifestEvidence {
        entries: entries.clone(),
        state,
        attribute_name: None,
        attribute_value: None,
    };
    for index in manifest_indexes {
        if issues.items_exhausted() {
            break;
        }
        let manifest = &classified[*index];
        if manifest.physical.id.raw_name.0 != MANIFEST
            && let Err(error) = push_domain(
                diagnostics,
                MultiReleaseDiagnosticCode::MultiReleaseManifestNoncanonicalPath,
                "Manifest path uses non-canonical ASCII case",
                Some(manifest.physical),
                budget,
            )
        {
            push_terminal(diagnostics, &error, Some(manifest.physical));
            issues.record(&error, budget);
            break;
        }
    }
    if !physical_complete {
        return ManifestOutcome::preserved(unknown(ManifestState::Unknown));
    }
    if manifest_indexes.is_empty() {
        return ManifestOutcome::preserved(ManifestEvidence {
            entries,
            state: ManifestState::Missing,
            attribute_name: None,
            attribute_value: None,
        });
    }
    if manifest_indexes.len() > 1 {
        for index in manifest_indexes {
            if issues.items_exhausted() {
                break;
            }
            let manifest = &classified[*index];
            if let Err(error) = push_domain(
                diagnostics,
                MultiReleaseDiagnosticCode::MultiReleaseManifestDuplicate,
                "multiple case-equivalent Manifest entries",
                Some(manifest.physical),
                budget,
            ) {
                push_terminal(diagnostics, &error, Some(manifest.physical));
                issues.record(&error, budget);
                break;
            }
        }
        return ManifestOutcome::preserved(unknown(ManifestState::Ambiguous));
    }
    let manifest = &classified[manifest_indexes[0]];
    let materialized = match snapshot.read_entry_for_analysis(manifest.physical, budget) {
        Ok(value) => value,
        Err(error) => {
            push_terminal(diagnostics, &error, Some(manifest.physical));
            issues.record(&error, budget);
            return ManifestOutcome::interrupted(unknown(ManifestState::Unknown));
        }
    };
    let parsed = match parse_manifest(&materialized.bytes, budget) {
        ManifestParse::Interrupted(error) => {
            push_terminal(diagnostics, &error, Some(manifest.physical));
            issues.record(&error, budget);
            return ManifestOutcome::interrupted(unknown(ManifestState::Unknown));
        }
        ManifestParse::Malformed => {
            if !issues.items_exhausted()
                && let Err(error) = push_domain(
                    diagnostics,
                    MultiReleaseDiagnosticCode::MultiReleaseManifestMalformed,
                    "Manifest main section is malformed or has duplicate Multi-Release attributes",
                    Some(manifest.physical),
                    budget,
                )
            {
                push_terminal(diagnostics, &error, Some(manifest.physical));
                issues.record(&error, budget);
            }
            // The malformed main section is a proven domain fact; the interrupted
            // diagnostic accounting only makes the surrounding execution non-Complete.
            return ManifestOutcome::preserved(unknown(ManifestState::Malformed));
        }
        ManifestParse::Parsed(attribute) => attribute,
    };
    ManifestOutcome::preserved(match parsed {
        Some((name, value)) => ManifestEvidence {
            entries,
            state: if ascii_eq(&value, b"true") {
                ManifestState::Active
            } else {
                ManifestState::Inactive
            },
            attribute_name: Some(ArchiveNameBytes(name)),
            attribute_value: Some(ArchiveNameBytes(value)),
        },
        None => ManifestEvidence {
            entries,
            state: ManifestState::Inactive,
            attribute_name: None,
            attribute_value: None,
        },
    })
}

type ManifestAttribute = Option<(Vec<u8>, Vec<u8>)>;

enum ManifestParse {
    Parsed(ManifestAttribute),
    Malformed,
    Interrupted(Error),
}

/// Single pass over the raw main section: no per-line copy is materialized, and only the
/// matching `Multi-Release` `(name, value)` bytes are retained.
fn parse_manifest(bytes: &[u8], budget: &Budget) -> ManifestParse {
    if bytes.contains(&0) {
        return ManifestParse::Malformed;
    }
    let mut name_bytes: Option<Vec<u8>> = None;
    let mut value_bytes: Option<Vec<u8>> = None;
    let mut current_is_attribute = false;
    let mut has_logical_line = false;
    let mut start = 0;
    let mut main_terminated = false;
    while start < bytes.len() {
        if let Err(error) = budget.poll() {
            return ManifestParse::Interrupted(error);
        }
        let mut end = start;
        while end < bytes.len() && bytes[end] != b'\r' && bytes[end] != b'\n' {
            end += 1;
        }
        if end == bytes.len() {
            return ManifestParse::Malformed;
        }
        let line = &bytes[start..end];
        if bytes[end] == b'\r' && bytes.get(end + 1) == Some(&b'\n') {
            end += 1;
        }
        start = end + 1;
        if line.is_empty() {
            main_terminated = true;
            break;
        }
        if line[0] == b' ' {
            // Single-space continuation: an orphan continuation has no logical line yet.
            if !has_logical_line {
                return ManifestParse::Malformed;
            }
            if current_is_attribute && let Some(value) = value_bytes.as_mut() {
                value.extend_from_slice(&line[1..]);
            }
            continue;
        }
        has_logical_line = true;
        current_is_attribute = false;
        let Some(colon) = line.iter().position(|byte| *byte == b':') else {
            return ManifestParse::Malformed;
        };
        if line.get(colon + 1) != Some(&b' ') {
            return ManifestParse::Malformed;
        }
        let name = &line[..colon];
        if !ascii_eq(name, b"Multi-Release") {
            continue;
        }
        if name_bytes.is_some() {
            return ManifestParse::Malformed;
        }
        name_bytes = Some(name.to_vec());
        value_bytes = Some(line[colon + 2..].to_vec());
        current_is_attribute = true;
    }
    if !main_terminated {
        return ManifestParse::Malformed;
    }
    match (name_bytes, value_bytes) {
        (None, None) => ManifestParse::Parsed(None),
        (Some(name), Some(value)) => ManifestParse::Parsed(Some((name, value))),
        _ => ManifestParse::Malformed,
    }
}

fn mark_duplicates(
    c: &[Classified<'_>],
    evidence: &mut [MultiReleaseEntryEvidence],
    diagnostics: &mut Vec<MultiReleaseReportDiagnostic>,
    budget: &mut Budget,
    issues: &mut Issues,
) {
    let mut map: HashMap<(Vec<u8>, u64), Vec<usize>> = HashMap::new();
    for (i, item) in c.iter().enumerate() {
        let Some(path) = &item.logical else {
            continue;
        };
        if item.meta_inf {
            continue;
        }
        let level = match item.variant {
            MultiReleaseEntryVariant::Base => 0,
            MultiReleaseEntryVariant::Versioned { release } => release,
            _ => continue,
        };
        map.entry((path.clone(), level)).or_default().push(i);
    }
    let mut duplicates: Vec<Vec<usize>> = map
        .into_values()
        .filter(|indexes| indexes.len() > 1)
        .collect();
    duplicates.sort_by_key(|indexes| indexes[0]);
    for indexes in duplicates {
        for &i in &indexes {
            merge_compliance(
                &mut evidence[i].compliance,
                MultiReleaseCompliance::NonConformant,
            );
        }
        for &i in &indexes {
            if issues.items_exhausted() {
                return;
            }
            if let Err(error) = push_domain(
                diagnostics,
                MultiReleaseDiagnosticCode::MultiReleaseDuplicateCandidate,
                "duplicate candidate at the same logical path and release level",
                Some(c[i].physical),
                budget,
            ) {
                push_terminal(diagnostics, &error, Some(c[i].physical));
                issues.record(&error, budget);
                return;
            }
        }
    }
    for (i, item) in c.iter().enumerate() {
        if let MultiReleaseEntryVariant::InvalidVersioned { issue } = item.variant {
            if issues.items_exhausted() {
                return;
            }
            let code = match issue {
                MultiReleaseVersionPathIssue::ReleaseBelowNine => {
                    MultiReleaseDiagnosticCode::MultiReleaseVersionBelowNine
                }
                MultiReleaseVersionPathIssue::ReleaseOverflow => {
                    MultiReleaseDiagnosticCode::MultiReleaseVersionOverflow
                }
                _ => MultiReleaseDiagnosticCode::MultiReleaseVersionPathInvalid,
            };
            if let Err(error) = push_domain(
                diagnostics,
                code,
                "invalid multi-release version path",
                Some(item.physical),
                budget,
            ) {
                push_terminal(diagnostics, &error, Some(item.physical));
                issues.record(&error, budget);
                return;
            }
        } else if item.meta_inf {
            merge_compliance(
                &mut evidence[i].compliance,
                MultiReleaseCompliance::NonConformant,
            );
            if issues.items_exhausted() {
                return;
            }
            if let Err(error) = push_domain(
                diagnostics,
                MultiReleaseDiagnosticCode::MultiReleaseMetaInfResource,
                "versioned META-INF resources are not selectable",
                Some(item.physical),
                budget,
            ) {
                push_terminal(diagnostics, &error, Some(item.physical));
                issues.record(&error, budget);
                return;
            }
        }
    }
}

fn probe_compliance(
    snapshot: &ArtifactSnapshot,
    c: &[Classified<'_>],
    returned: &mut [MultiReleaseEntryEvidence],
    probe_states: &mut [ProbeState],
    budget: &mut Budget,
    diagnostics: &mut Vec<MultiReleaseReportDiagnostic>,
    issues: &mut Issues,
) {
    let module = c.iter().any(|x| {
        x.logical.as_deref() == Some(b"module-info.class")
            && matches!(
                x.variant,
                MultiReleaseEntryVariant::Base | MultiReleaseEntryVariant::Versioned { .. }
            )
    });
    let mut base_probes: HashMap<
        usize,
        std::result::Result<crate::classfile::MinimalHeaderFacts, Error>,
    > = HashMap::new();
    for i in 0..returned.len() {
        if issues.items_exhausted() {
            return;
        }
        if !is_probe_applicable(&c[i]) {
            continue;
        }
        let MultiReleaseEntryVariant::Versioned { release } = c[i].variant else {
            continue;
        };
        let materialized = match snapshot.read_entry_for_analysis(c[i].physical, budget) {
            Ok(value) => value,
            Err(error) => {
                probe_states[i] = ProbeState::Interrupted;
                merge_compliance(
                    &mut returned[i].compliance,
                    MultiReleaseCompliance::Unknown {
                        reason: MultiReleaseComplianceUnknownReason::ProbeInterrupted,
                    },
                );
                push_terminal(diagnostics, &error, Some(c[i].physical));
                issues.record(&error, budget);
                mark_remaining_probe_unknown(c, returned, i + 1);
                return;
            }
        };
        let facts = match probe_minimal_header(&materialized.bytes, budget) {
            Ok(facts) => facts,
            Err(Error::InvalidInput { .. }) => {
                probe_states[i] = ProbeState::Completed;
                merge_compliance(
                    &mut returned[i].compliance,
                    MultiReleaseCompliance::NonConformant,
                );
                if let Err(charge_error) = push_domain(
                    diagnostics,
                    MultiReleaseDiagnosticCode::MultiReleaseClassMalformed,
                    "versioned class has a malformed minimal Header",
                    Some(c[i].physical),
                    budget,
                ) {
                    push_terminal(diagnostics, &charge_error, Some(c[i].physical));
                    issues.record(&charge_error, budget);
                    mark_remaining_probe_unknown(c, returned, i + 1);
                    return;
                }
                continue;
            }
            Err(error) => {
                probe_states[i] = ProbeState::Interrupted;
                merge_compliance(
                    &mut returned[i].compliance,
                    MultiReleaseCompliance::Unknown {
                        reason: MultiReleaseComplianceUnknownReason::ProbeInterrupted,
                    },
                );
                push_terminal(diagnostics, &error, Some(c[i].physical));
                issues.record(&error, budget);
                mark_remaining_probe_unknown(c, returned, i + 1);
                return;
            }
        };
        probe_states[i] = ProbeState::Completed;
        returned[i].class_evidence = Some(MultiReleaseClassEvidence {
            major_version: facts.major_version,
            minor_version: facts.minor_version,
            access_flags: facts.access_flags,
            this_class: facts.this_class.clone(),
        });
        if u128::from(facts.major_version) > u128::from(release) + 44 {
            merge_compliance(
                &mut returned[i].compliance,
                MultiReleaseCompliance::NonConformant,
            );
            if let Err(error) = push_domain(
                diagnostics,
                MultiReleaseDiagnosticCode::MultiReleaseClassVersionTooNew,
                "classfile major exceeds release + 44",
                Some(c[i].physical),
                budget,
            ) {
                push_terminal(diagnostics, &error, Some(c[i].physical));
                issues.record(&error, budget);
                mark_remaining_probe_unknown(c, returned, i + 1);
                return;
            }
        }
        if facts.access_flags & ACC_PUBLIC == 0 {
            merge_compliance(
                &mut returned[i].compliance,
                MultiReleaseCompliance::ConformantWithinChecks,
            );
            continue;
        }
        let Some(path) = c[i].logical.as_ref() else {
            continue;
        };
        let roots = c
            .iter()
            .enumerate()
            .filter(|(_, x)| {
                x.logical.as_ref() == Some(path)
                    && matches!(x.variant, MultiReleaseEntryVariant::Base)
            })
            .map(|(n, _)| n)
            .collect::<Vec<_>>();
        if roots.len() > 1 {
            merge_compliance(
                &mut returned[i].compliance,
                MultiReleaseCompliance::Unknown {
                    reason: MultiReleaseComplianceUnknownReason::PredecessorAmbiguous,
                },
            );
            continue;
        }
        let Some(root) = roots.first().copied() else {
            if module {
                merge_compliance(
                    &mut returned[i].compliance,
                    MultiReleaseCompliance::Unknown {
                        reason: MultiReleaseComplianceUnknownReason::ModuleExportsNotInspected,
                    },
                );
            } else {
                merge_compliance(
                    &mut returned[i].compliance,
                    MultiReleaseCompliance::NonConformant,
                );
                if let Err(error) = push_domain(
                    diagnostics,
                    MultiReleaseDiagnosticCode::MultiReleasePublicPredecessorMissing,
                    "public versioned class has no root predecessor",
                    Some(c[i].physical),
                    budget,
                ) {
                    push_terminal(diagnostics, &error, Some(c[i].physical));
                    issues.record(&error, budget);
                    mark_remaining_probe_unknown(c, returned, i + 1);
                    return;
                }
            }
            continue;
        };
        if let std::collections::hash_map::Entry::Vacant(slot) = base_probes.entry(root) {
            // The probe only runs over a complete evidence prefix, so a probed base is
            // always present in `returned`; the index guard keeps that invariant explicit.
            let result = snapshot
                .read_entry_for_analysis(c[root].physical, budget)
                .and_then(|bytes| probe_minimal_header(&bytes.bytes, budget));
            match &result {
                Ok(base) => {
                    probe_states[root] = ProbeState::Completed;
                    if root < returned.len() {
                        returned[root].class_evidence = Some(MultiReleaseClassEvidence {
                            major_version: base.major_version,
                            minor_version: base.minor_version,
                            access_flags: base.access_flags,
                            this_class: base.this_class.clone(),
                        });
                    }
                }
                Err(Error::InvalidInput { .. }) => {
                    // A malformed predecessor is a non-conformance of the versioned candidate
                    // that needed it, not of the plain Base entry, whose closed-table state
                    // stays `NotApplicable`. The Base ordinal still counts as checked.
                    probe_states[root] = ProbeState::Completed;
                }
                Err(_) => {
                    // The predecessor read or Header probe was attempted and interrupted, so
                    // the Base ordinal is a probe target and is declared as skipped.
                    probe_states[root] = ProbeState::Interrupted;
                }
            }
            slot.insert(result);
        }
        match base_probes.get(&root).unwrap() {
            Ok(base)
                if base.access_flags & ACC_PUBLIC != 0 && base.this_class == facts.this_class =>
            {
                merge_compliance(
                    &mut returned[i].compliance,
                    MultiReleaseCompliance::ConformantWithinChecks,
                );
            }
            Err(Error::InvalidInput { .. }) => {
                if root < returned.len()
                    && !has_domain_diagnostic(
                        diagnostics,
                        MultiReleaseDiagnosticCode::MultiReleaseClassMalformed,
                        &c[root].physical.id,
                    )
                    && let Err(charge_error) = push_domain(
                        diagnostics,
                        MultiReleaseDiagnosticCode::MultiReleaseClassMalformed,
                        "root predecessor has a malformed minimal Header",
                        Some(c[root].physical),
                        budget,
                    )
                {
                    push_terminal(diagnostics, &charge_error, Some(c[root].physical));
                    issues.record(&charge_error, budget);
                    mark_remaining_probe_unknown(c, returned, i + 1);
                    return;
                }
                if module {
                    merge_compliance(
                        &mut returned[i].compliance,
                        MultiReleaseCompliance::Unknown {
                            reason: MultiReleaseComplianceUnknownReason::ModuleExportsNotInspected,
                        },
                    );
                } else {
                    merge_compliance(
                        &mut returned[i].compliance,
                        MultiReleaseCompliance::NonConformant,
                    );
                    if let Err(charge_error) = push_domain(
                        diagnostics,
                        MultiReleaseDiagnosticCode::MultiReleasePublicPredecessorMismatch,
                        "root predecessor is malformed",
                        Some(c[i].physical),
                        budget,
                    ) {
                        push_terminal(diagnostics, &charge_error, Some(c[i].physical));
                        issues.record(&charge_error, budget);
                        mark_remaining_probe_unknown(c, returned, i + 1);
                        return;
                    }
                }
            }
            Ok(_) if module => merge_compliance(
                &mut returned[i].compliance,
                MultiReleaseCompliance::Unknown {
                    reason: MultiReleaseComplianceUnknownReason::ModuleExportsNotInspected,
                },
            ),
            Ok(_) => {
                merge_compliance(
                    &mut returned[i].compliance,
                    MultiReleaseCompliance::NonConformant,
                );
                if let Err(error) = push_domain(
                    diagnostics,
                    MultiReleaseDiagnosticCode::MultiReleasePublicPredecessorMismatch,
                    "root predecessor is not public with the same this_class",
                    Some(c[i].physical),
                    budget,
                ) {
                    push_terminal(diagnostics, &error, Some(c[i].physical));
                    issues.record(&error, budget);
                    mark_remaining_probe_unknown(c, returned, i + 1);
                    return;
                }
            }
            Err(error) => {
                let error = error.clone();
                // The candidate's own Header facts are known, but its compliance verdict is
                // not: the interrupted predecessor check leaves this ordinal unfinished.
                probe_states[i] = ProbeState::Interrupted;
                merge_compliance(
                    &mut returned[i].compliance,
                    MultiReleaseCompliance::Unknown {
                        reason: MultiReleaseComplianceUnknownReason::ProbeInterrupted,
                    },
                );
                push_terminal(diagnostics, &error, Some(c[root].physical));
                issues.record(&error, budget);
                mark_remaining_probe_unknown(c, returned, i + 1);
                return;
            }
        }
    }
}

/// Every remaining applicable candidate of an interrupted probe keeps the unknown
/// compliance it had; a proven nonconformance is never downgraded.
fn mark_remaining_probe_unknown(
    c: &[Classified<'_>],
    evidence: &mut [MultiReleaseEntryEvidence],
    from_index: usize,
) {
    for (index, item) in c.iter().enumerate().skip(from_index) {
        if index >= evidence.len() {
            break;
        }
        if !is_probe_applicable(item) {
            continue;
        }
        merge_compliance(
            &mut evidence[index].compliance,
            MultiReleaseCompliance::Unknown {
                reason: MultiReleaseComplianceUnknownReason::ProbeInterrupted,
            },
        );
    }
}

/// `NonConformant` absorbs every later observation, and no other state can turn a proof
/// back into a weaker one. `Unknown(ProbeInterrupted)` only ever states "not probed yet",
/// so any concrete observation replaces it, while every other `Unknown` absorbs
/// `ConformantWithinChecks` and `NotApplicable`.
fn merge_compliance(current: &mut MultiReleaseCompliance, incoming: MultiReleaseCompliance) {
    fn rank(compliance: &MultiReleaseCompliance) -> u8 {
        match compliance {
            MultiReleaseCompliance::NonConformant => 4,
            MultiReleaseCompliance::Unknown {
                reason: MultiReleaseComplianceUnknownReason::ProbeInterrupted,
            } => 1,
            MultiReleaseCompliance::Unknown { .. } => 3,
            MultiReleaseCompliance::ConformantWithinChecks => 2,
            MultiReleaseCompliance::NotApplicable => 0,
        }
    }
    if rank(&incoming) > rank(current) {
        *current = incoming;
    }
}

fn push_domain(
    out: &mut Vec<MultiReleaseReportDiagnostic>,
    code: MultiReleaseDiagnosticCode,
    message: &str,
    entry: Option<&PhysicalEntry>,
    budget: &mut Budget,
) -> Result<()> {
    budget.charge(CountedBudgetDimension::ResultItems, 1)?;
    out.push(MultiReleaseReportDiagnostic::Domain {
        diagnostic: MultiReleaseDiagnostic::new(code, message, entry.map(provenance)),
    });
    Ok(())
}

fn push_terminal(
    out: &mut Vec<MultiReleaseReportDiagnostic>,
    error: &Error,
    entry: Option<&PhysicalEntry>,
) {
    out.push(MultiReleaseReportDiagnostic::Terminal {
        diagnostic: terminal(error, entry),
    });
}

fn has_domain_diagnostic(
    diagnostics: &[MultiReleaseReportDiagnostic],
    code: MultiReleaseDiagnosticCode,
    entry: &PhysicalEntryId,
) -> bool {
    diagnostics.iter().any(|item| match item {
        MultiReleaseReportDiagnostic::Domain { diagnostic } => {
            diagnostic.code == code
                && diagnostic.provenance.as_ref().is_some_and(|provenance| {
                    matches!(
                        &provenance.location,
                        Location::Entry { id, .. } if id == entry
                    )
                })
        }
        MultiReleaseReportDiagnostic::Terminal { .. } => false,
    })
}

fn provenance(entry: &PhysicalEntry) -> Provenance {
    Provenance {
        location: Location::Entry {
            id: entry.id.clone(),
            span: entry.layout.compressed_data.clone(),
        },
    }
}
fn terminal(error: &Error, entry: Option<&PhysicalEntry>) -> Diagnostic {
    Diagnostic {
        code: match error {
            Error::InvalidInput { code, .. } | Error::Unsupported { code, .. } => code.clone(),
            Error::BudgetExceeded { dimension, .. } => {
                format!("budget_exceeded_{}", budget_dimension_code(*dimension))
            }
            Error::Cancelled { .. } => "cancelled".into(),
            Error::Io { operation, .. } => operation.clone(),
        },
        severity: if matches!(
            error,
            Error::InvalidInput { .. } | Error::Unsupported { .. }
        ) {
            DiagnosticSeverity::Error
        } else {
            DiagnosticSeverity::Warning
        },
        message: error.to_string(),
        provenance: entry.map(provenance),
    }
}
fn ascii_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.eq_ignore_ascii_case(y))
}
fn range(origin: &ContainerOrigin, suffix: &str, ordinal: u64) -> CoverageRange {
    CoverageRange {
        label: format!("container:{}:{suffix}", origin.current_container().0),
        start: ordinal,
        end: ordinal.saturating_add(1),
    }
}
fn unprocessed_ranges(origin: &ContainerOrigin, entries: &[PhysicalEntry]) -> Vec<CoverageRange> {
    let mut ranges = Vec::new();
    for entry in entries {
        ranges.push(range(origin, SELECTION_METRIC, entry.id.ordinal));
        if is_probe_applicable(&classify(entry)) {
            ranges.push(range(origin, COMPLIANCE_METRIC, entry.id.ordinal));
        }
    }
    ranges
}

/// A container this run never produced a report for: every ordinal it did enumerate is
/// declared skipped in both MR dimensions, followed by the physical suffix its provider
/// itself did not enumerate (label and bounds kept verbatim).
fn declare_unprocessed(input: &ContainerInput, out: &mut Vec<CoverageRange>) {
    out.extend(unprocessed_ranges(&input.origin, &input.entries));
    out.extend(input.coverage.artifact_structural.skipped.clone());
}
fn root_origin(snapshot: &ArtifactSnapshot) -> ContainerOrigin {
    ContainerOrigin {
        snapshot: snapshot.id().clone(),
        root_container: crate::model::ContainerId("root".into()),
        steps: Vec::new(),
    }
}
fn physical_execution(p: &MultiReleasePhysicalEvidence) -> &ExecutionReport {
    match p {
        MultiReleasePhysicalEvidence::Snapshot { report } => &report.execution,
        MultiReleasePhysicalEvidence::ArtifactTree { report } => &report.execution,
    }
}
fn physical_coverage(p: &MultiReleasePhysicalEvidence) -> &Coverage {
    match p {
        MultiReleasePhysicalEvidence::Snapshot { report } => &report.coverage,
        MultiReleasePhysicalEvidence::ArtifactTree { report } => &report.coverage,
    }
}
fn execution_for(error: &Error, budget: &Budget, failed: bool) -> ExecutionReport {
    match error {
        Error::Cancelled { .. } => ExecutionReport::Cancelled {
            usage: budget.usage(),
        },
        Error::BudgetExceeded { dimension, .. } => ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: *dimension,
            },
            usage: budget.usage(),
        },
        Error::Unsupported { code, .. } => {
            if failed {
                ExecutionReport::Failed {
                    reason: TerminationReason::Unsupported { code: code.clone() },
                    usage: budget.usage(),
                }
            } else {
                ExecutionReport::Partial {
                    reason: TerminationReason::Unsupported { code: code.clone() },
                    usage: budget.usage(),
                }
            }
        }
        Error::InvalidInput { code, .. }
        | Error::Io {
            operation: code, ..
        } => {
            if failed {
                ExecutionReport::Failed {
                    reason: TerminationReason::Error { code: code.clone() },
                    usage: budget.usage(),
                }
            } else {
                ExecutionReport::Partial {
                    reason: TerminationReason::Error { code: code.clone() },
                    usage: budget.usage(),
                }
            }
        }
    }
}
/// A blocking issue is a budget or cancellation error that really prevented progress.
fn blocks(priority: u8) -> bool {
    priority >= 3
}

fn priority_of(execution: &ExecutionReport) -> u8 {
    match execution {
        ExecutionReport::Cancelled { .. } => 4,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded { .. },
            ..
        }
        | ExecutionReport::Failed {
            reason: TerminationReason::BudgetExceeded { .. },
            ..
        } => 3,
        ExecutionReport::Failed {
            reason: TerminationReason::Unsupported { .. },
            ..
        } => 2,
        ExecutionReport::Complete { .. } => 0,
        _ => 1,
    }
}

fn merge_issue(slot: &mut Option<ExecutionReport>, incoming: ExecutionReport) {
    match slot {
        Some(current) if priority_of(current) >= priority_of(&incoming) => {}
        _ => *slot = Some(incoming),
    }
}

/// Highest-priority issue of one container or of the aggregate report. The first issue of
/// the winning priority is kept, `usage` is refreshed when the report is finalized, a
/// blocking budget or cancellation error stops later work, and an exhausted result-item
/// budget stops later non-terminal items (terminal diagnostics stay control metadata).
struct Issues {
    best: Option<ExecutionReport>,
    items_exhausted: bool,
}

impl Issues {
    fn new(initial: &ExecutionReport) -> Self {
        Self {
            best: Some(initial.clone()),
            items_exhausted: false,
        }
    }

    /// No further entry evidence, selection, or domain diagnostic can be charged.
    fn items_exhausted(&self) -> bool {
        self.items_exhausted
    }

    /// Work was stopped by a budget or cancellation error that really prevented progress.
    fn blocked(&self) -> bool {
        self.best
            .as_ref()
            .is_some_and(|report| blocks(priority_of(report)))
    }

    fn record(&mut self, error: &Error, budget: &Budget) {
        if matches!(
            error,
            Error::BudgetExceeded {
                dimension: BudgetDimension::ResultItems,
                ..
            }
        ) {
            self.items_exhausted = true;
        }
        merge_issue(&mut self.best, execution_for(error, budget, false));
    }

    /// Merges an issue the caller already fully evaluated. Reporting only: the caller decides
    /// whether this issue stops its own work.
    fn merge(&mut self, report: &ExecutionReport) {
        merge_issue(&mut self.best, report.clone());
    }

    /// The policy verdict is the reason this probe never ran, so it also wins over an equally
    /// ranked issue; a higher-ranked issue (cancellation or a blocking budget) still stands.
    fn override_with(&mut self, report: ExecutionReport) {
        match &self.best {
            Some(current) if priority_of(current) > priority_of(&report) => {}
            _ => self.best = Some(report),
        }
    }

    fn finish(self, budget: &Budget) -> ExecutionReport {
        let report = self.best.unwrap_or(ExecutionReport::Complete {
            usage: budget.usage(),
        });
        crate::accounting::with_usage(report, budget.usage())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::UsageSnapshot;

    fn unsupported(code: &str) -> ExecutionReport {
        ExecutionReport::Failed {
            reason: TerminationReason::Unsupported { code: code.into() },
            usage: UsageSnapshot::default(),
        }
    }

    fn partial(dimension: BudgetDimension) -> ExecutionReport {
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded { dimension },
            usage: UsageSnapshot::default(),
        }
    }

    fn issue_of(report: ExecutionReport) -> Issues {
        Issues::new(&report)
    }

    fn code_of(issues: &Issues) -> String {
        match issues.best.as_ref().unwrap() {
            ExecutionReport::Failed {
                reason: TerminationReason::Unsupported { code },
                ..
            }
            | ExecutionReport::Partial {
                reason: TerminationReason::Unsupported { code },
                ..
            } => code.clone(),
            other => format!("{other:?}"),
        }
    }

    /// The policy verdict explains why no standard probe ran, so it also replaces an equally
    /// ranked unsupported issue instead of being outranked by it.
    #[test]
    fn policy_verdict_replaces_an_equally_ranked_unsupported_issue() {
        let mut issues = issue_of(unsupported("physical_unsupported_probe"));
        issues.override_with(unsupported("multi_release_custom_policy"));
        assert_eq!(code_of(&issues), "multi_release_custom_policy");
    }

    /// Cancellation and a blocking budget really prevented the work, so they stay.
    #[test]
    fn a_blocking_issue_outranks_the_policy_verdict() {
        for blocking in [
            ExecutionReport::Cancelled {
                usage: UsageSnapshot::default(),
            },
            partial(BudgetDimension::ReadBytes),
        ] {
            let mut issues = issue_of(blocking.clone());
            issues.override_with(unsupported("multi_release_unknown_policy"));
            assert_eq!(issues.best, Some(blocking));
        }
    }

    /// Equal priorities keep the issue that appeared first, for merged and recorded issues.
    #[test]
    fn the_first_issue_of_a_priority_wins() {
        let mut issues = issue_of(unsupported("first_unsupported"));
        issues.merge(&unsupported("second_unsupported"));
        assert_eq!(code_of(&issues), "first_unsupported");
        let mut issues = issue_of(partial(BudgetDimension::ReadBytes));
        issues.merge(&partial(BudgetDimension::ClassBytes));
        assert_eq!(issues.best, Some(partial(BudgetDimension::ReadBytes)));
        // A strictly higher priority still wins over the earlier issue.
        issues.merge(&ExecutionReport::Cancelled {
            usage: UsageSnapshot::default(),
        });
        assert_eq!(
            issues.best,
            Some(ExecutionReport::Cancelled {
                usage: UsageSnapshot::default()
            })
        );
    }

    /// Every budget dimension blocks further work, but only the result-item dimension also
    /// stops item charges: terminal diagnostics stay control metadata.
    #[test]
    fn result_item_exhaustion_stops_items_and_blocks_further_work() {
        let mut issues = issue_of(ExecutionReport::Complete {
            usage: UsageSnapshot::default(),
        });
        assert!(!issues.items_exhausted() && !issues.blocked());
        let budget = Budget::new(test_limits());
        issues.record(
            &Error::BudgetExceeded {
                dimension: BudgetDimension::ResultItems,
                limit: 0,
                consumed: 0,
                requested: 1,
            },
            &budget,
        );
        assert!(issues.items_exhausted());
        assert!(issues.blocked());
        assert_eq!(issues.best, Some(partial(BudgetDimension::ResultItems)));
    }

    fn test_limits() -> crate::budget::Limits {
        crate::budget::Limits {
            input_bytes: 1 << 24,
            archive_entries: 1_000,
            entry_bytes: 1 << 24,
            read_bytes: 1 << 24,
            class_bytes: 1 << 24,
            attribute_bytes: 1 << 24,
            code_bytes: 1 << 24,
            result_items: 1_000,
            output_bytes: 1 << 24,
            nested_depth: 8,
            elapsed_millis: u64::MAX,
            ..crate::budget::Limits::default()
        }
    }
}
