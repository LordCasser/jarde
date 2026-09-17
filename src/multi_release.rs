//! Evidence-backed, per-container multi-release JAR selection.

use crate::artifact::{
    ArtifactKind, ArtifactSnapshot, ArtifactTreeReport, EnumerationReport, PhysicalEntry,
    budget_dimension_code,
};
use crate::budget::{Budget, CountedBudgetDimension};
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

pub(crate) fn select(
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
    let mut aggregate_issue = physical_execution(&physical).clone();
    let containers: Vec<(
        ContainerOrigin,
        Vec<PhysicalEntry>,
        Coverage,
        ExecutionReport,
    )> = match &physical {
        MultiReleasePhysicalEvidence::Snapshot { report } => vec![(
            root_origin(snapshot),
            report.entries.clone(),
            report.coverage.clone(),
            report.execution.clone(),
        )],
        MultiReleasePhysicalEvidence::ArtifactTree { report } => report
            .containers
            .iter()
            .map(|c| {
                (
                    c.origin.clone(),
                    c.entries.clone(),
                    c.coverage.clone(),
                    c.execution.clone(),
                )
            })
            .collect(),
    };
    for (origin, entries, physical_coverage, physical_execution) in containers {
        if let Err(error) = budget.charge(CountedBudgetDimension::ResultItems, 2) {
            set_issue(&mut aggregate_issue, &error, budget, false);
            skip_entries(&origin, &entries, &mut runtime_skipped)?;
            break;
        }
        let processed = process_container(
            snapshot,
            view,
            origin,
            entries,
            physical_coverage,
            physical_execution,
            budget,
        );
        diagnostics.extend(processed.diagnostics);
        runtime_scanned.extend(processed.scanned);
        runtime_skipped.extend(processed.skipped);
        merge_execution(&mut aggregate_issue, &processed.report.execution);
        reports.push(processed.report);
        if must_stop(&aggregate_issue) {
            break;
        }
    }
    if matches!(
        view.profile.multi_release,
        MultiReleasePolicy::Custom { .. }
    ) {
        aggregate_issue = ExecutionReport::Failed {
            reason: TerminationReason::Unsupported {
                code: "multi_release_custom_policy".into(),
            },
            usage: budget.usage(),
        };
    } else if matches!(view.profile.multi_release, MultiReleasePolicy::Unknown) {
        aggregate_issue = ExecutionReport::Failed {
            reason: TerminationReason::Unsupported {
                code: "multi_release_unknown_policy".into(),
            },
            usage: budget.usage(),
        };
    } else {
        aggregate_issue = with_usage(aggregate_issue, budget);
    }
    let structural = physical_coverage(&physical).artifact_structural.clone();
    let complete = matches!(aggregate_issue, ExecutionReport::Complete { .. });
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
        execution: aggregate_issue,
        diagnostics,
        verification: VerificationStatus::NotPerformed,
    })
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
    let unsupported = match view.profile.multi_release {
        MultiReleasePolicy::Custom { .. } => Some(MultiReleaseUnknownReason::CustomPolicy),
        MultiReleasePolicy::Unknown => Some(MultiReleaseUnknownReason::UnknownPolicy),
        _ => None,
    };
    let classified: Vec<_> = entries.iter().map(classify).collect();
    let mut diagnostics = Vec::new();
    let manifests: Vec<_> = classified
        .iter()
        .filter(|e| ascii_eq(&e.physical.id.raw_name.0, MANIFEST))
        .collect();
    let (manifest, manifest_issue) = if unsupported.is_some() {
        (ManifestEvidence {
            entries: manifests.iter().map(|e| e.physical.id.clone()).collect(),
            state: ManifestState::Unknown,
            attribute_name: None,
            attribute_value: None,
        }, None)
    } else {
        read_manifest(
            snapshot,
            &manifests,
            physical_complete,
            budget,
            &mut diagnostics,
        )
    };
    let mut evidence: Vec<MultiReleaseEntryEvidence> = classified
        .iter()
        .map(|c| initial_evidence(c, unsupported, physical_complete, &manifest, view))
        .collect();
    let mut groups: BTreeMap<Vec<u8>, Vec<usize>> = BTreeMap::new();
    for (index, c) in classified.iter().enumerate() {
        if c.directory
            || c.meta_inf
            || matches!(c.variant, MultiReleaseEntryVariant::InvalidVersioned { .. })
        {
            continue;
        }
        if let Some(path) = &c.logical {
            groups.entry(path.clone()).or_default().push(index);
        }
    }
    mark_duplicates(&classified, &mut evidence, &mut diagnostics, budget);
    let mut selections = Vec::new();
    for (path, indexes) in &groups {
        let outcome = choose(
            indexes,
            &classified,
            &mut evidence,
            view,
            physical_complete,
            &manifest,
            unsupported,
        );
        if budget
            .charge(CountedBudgetDimension::ResultItems, 1)
            .is_ok()
        {
            selections.push(MultiReleaseSelection {
                logical_path: ArchiveNameBytes(path.clone()),
                outcome,
            });
        }
    }
    if unsupported.is_none() {
        probe_compliance(
            snapshot,
            &classified,
            &mut evidence,
            physical_complete,
            budget,
            &mut diagnostics,
        );
    }
    let mut returned = Vec::new();
    let mut interrupted = None;
    for item in evidence {
        match budget.charge(CountedBudgetDimension::ResultItems, 1) {
            Ok(()) => returned.push(item),
            Err(e) => {
                interrupted = Some(e);
                break;
            }
        }
    }
    let mut execution = if let Some(error) = interrupted.as_ref() {
        execution_for(error, budget, false)
    } else if let Some(error) = manifest_issue.as_ref() {
        execution_for(error, budget, false)
    } else {
        physical_execution.clone()
    };
    if let Some(reason) = unsupported {
        execution = ExecutionReport::Failed {
            reason: TerminationReason::Unsupported {
                code: if reason == MultiReleaseUnknownReason::CustomPolicy {
                    "multi_release_custom_policy"
                } else {
                    "multi_release_unknown_policy"
                }
                .into(),
            },
            usage: budget.usage(),
        };
    }
    let scanned: Vec<_> = returned
        .iter()
        .flat_map(|e| {
            [
                range(&origin, "multi_release_selection_entries", e.entry.ordinal),
                range(&origin, "multi_release_compliance_entries", e.entry.ordinal),
            ]
        })
        .collect();
    let skipped: Vec<CoverageRange> = entries
        .iter()
        .filter(|entry| !returned.iter().any(|e| e.entry == entry.id))
        .flat_map(|e| {
            [
                range(&origin, "multi_release_selection_entries", e.id.ordinal),
                range(&origin, "multi_release_compliance_entries", e.id.ordinal),
            ]
        })
        .collect();
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
            execution: with_usage(execution, budget),
        },
        diagnostics,
        scanned,
        skipped,
    }
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
        let compliance = if matches!(c.variant, MultiReleaseEntryVariant::Versioned { .. }) {
            MultiReleaseCompliance::ConformantWithinChecks
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

fn read_manifest(
    snapshot: &ArtifactSnapshot,
    manifests: &[&Classified<'_>],
    physical_complete: bool,
    budget: &mut Budget,
    diagnostics: &mut Vec<MultiReleaseReportDiagnostic>,
) -> (ManifestEvidence, Option<Error>) {
    let ids = manifests
        .iter()
        .map(|e| e.physical.id.clone())
        .collect::<Vec<_>>();
    for manifest in manifests {
        if manifest.physical.id.raw_name.0 != MANIFEST {
            push_domain(
                diagnostics,
                MultiReleaseDiagnosticCode::MultiReleaseManifestNoncanonicalPath,
                "Manifest path uses non-canonical ASCII case",
                Some(manifest.physical),
                budget,
            );
        }
    }
    if !physical_complete {
        return (ManifestEvidence {
            entries: ids,
            state: ManifestState::Unknown,
            attribute_name: None,
            attribute_value: None,
        }, None);
    }
    if manifests.is_empty() {
        return (ManifestEvidence {
            entries: ids,
            state: ManifestState::Missing,
            attribute_name: None,
            attribute_value: None,
        }, None);
    }
    if manifests.len() > 1 {
        for e in manifests {
            push_domain(
                diagnostics,
                MultiReleaseDiagnosticCode::MultiReleaseManifestDuplicate,
                "multiple case-equivalent Manifest entries",
                Some(e.physical),
                budget,
            );
        }
        return (ManifestEvidence {
            entries: ids,
            state: ManifestState::Ambiguous,
            attribute_name: None,
            attribute_value: None,
        }, None);
    }
    let materialized = match snapshot.read_entry_internal(manifests[0].physical, budget) {
        Ok(v) => v,
        Err(error) => {
            diagnostics.push(MultiReleaseReportDiagnostic::Terminal {
                diagnostic: terminal(&error, Some(manifests[0].physical)),
            });
            return (ManifestEvidence {
                entries: ids,
                state: ManifestState::Unknown,
                attribute_name: None,
                attribute_value: None,
            }, Some(error));
        }
    };
    match parse_manifest(&materialized.bytes, budget) {
        ManifestParse::Parsed(Some((name, value))) => (ManifestEvidence {
            entries: ids,
            state: if ascii_eq(&value, b"true") {
                ManifestState::Active
            } else {
                ManifestState::Inactive
            },
            attribute_name: Some(ArchiveNameBytes(name)),
            attribute_value: Some(ArchiveNameBytes(value)),
        }, None),
        ManifestParse::Parsed(None) => (ManifestEvidence {
            entries: ids,
            state: ManifestState::Inactive,
            attribute_name: None,
            attribute_value: None,
        }, None),
        ManifestParse::Interrupted(error) => {
            diagnostics.push(MultiReleaseReportDiagnostic::Terminal {
                diagnostic: terminal(&error, Some(manifests[0].physical)),
            });
            (ManifestEvidence {
                entries: ids,
                state: ManifestState::Unknown,
                attribute_name: None,
                attribute_value: None,
            }, Some(error))
        }
        ManifestParse::Malformed => {
            push_domain(
                diagnostics,
                MultiReleaseDiagnosticCode::MultiReleaseManifestMalformed,
                "Manifest main section is malformed or has duplicate Multi-Release attributes",
                Some(manifests[0].physical),
                budget,
            );
            (ManifestEvidence {
                entries: ids,
                state: ManifestState::Malformed,
                attribute_name: None,
                attribute_value: None,
            }, None)
        }
    }
}

type ManifestAttribute = Option<(Vec<u8>, Vec<u8>)>;

enum ManifestParse {
    Parsed(ManifestAttribute),
    Malformed,
    Interrupted(Error),
}

fn parse_manifest(bytes: &[u8], budget: &Budget) -> ManifestParse {
    if bytes.contains(&0) {
        return ManifestParse::Malformed;
    }
    let mut logical: Vec<Vec<u8>> = Vec::new();
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
        let line = bytes[start..end].to_vec();
        if bytes[end] == b'\r' && bytes.get(end + 1) == Some(&b'\n') {
            end += 1;
        }
        start = end + 1;
        if line.is_empty() {
            main_terminated = true;
            break;
        }
        if line.first() == Some(&b' ') {
            let Some(last) = logical.last_mut() else {
                return ManifestParse::Malformed;
            };
            last.extend_from_slice(&line[1..]);
        } else {
            logical.push(line);
        }
    }
    if !main_terminated {
        return ManifestParse::Malformed;
    }
    let mut found = None;
    for line in logical {
        let Some(colon) = line.iter().position(|b| *b == b':') else {
            return ManifestParse::Malformed;
        };
        if line.get(colon + 1) != Some(&b' ') {
            return ManifestParse::Malformed;
        }
        let name = &line[..colon];
        let value = &line[colon + 2..];
        if ascii_eq(name, b"Multi-Release") {
            if found.is_some() {
                return ManifestParse::Malformed;
            }
            found = Some((name.to_vec(), value.to_vec()));
        }
    }
    ManifestParse::Parsed(found)
}

fn mark_duplicates(
    c: &[Classified<'_>],
    evidence: &mut [MultiReleaseEntryEvidence],
    diagnostics: &mut Vec<MultiReleaseReportDiagnostic>,
    budget: &mut Budget,
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
    for indexes in map.values().filter(|v| v.len() > 1) {
        for &i in indexes {
            evidence[i].compliance = MultiReleaseCompliance::NonConformant;
            push_domain(
                diagnostics,
                MultiReleaseDiagnosticCode::MultiReleaseDuplicateCandidate,
                "duplicate candidate at the same logical path and release level",
                Some(c[i].physical),
                budget,
            );
        }
    }
    for (i, item) in c.iter().enumerate() {
        if let MultiReleaseEntryVariant::InvalidVersioned { issue } = item.variant {
            let code = match issue {
                MultiReleaseVersionPathIssue::ReleaseBelowNine => {
                    MultiReleaseDiagnosticCode::MultiReleaseVersionBelowNine
                }
                MultiReleaseVersionPathIssue::ReleaseOverflow => {
                    MultiReleaseDiagnosticCode::MultiReleaseVersionOverflow
                }
                _ => MultiReleaseDiagnosticCode::MultiReleaseVersionPathInvalid,
            };
            push_domain(
                diagnostics,
                code,
                "invalid multi-release version path",
                Some(item.physical),
                budget,
            );
        } else if item.meta_inf {
            push_domain(
                diagnostics,
                MultiReleaseDiagnosticCode::MultiReleaseMetaInfResource,
                "versioned META-INF resources are not selectable",
                Some(item.physical),
                budget,
            );
            evidence[i].compliance = MultiReleaseCompliance::NonConformant;
        }
    }
}

fn probe_compliance(
    snapshot: &ArtifactSnapshot,
    c: &[Classified<'_>],
    evidence: &mut [MultiReleaseEntryEvidence],
    physical_complete: bool,
    budget: &mut Budget,
    diagnostics: &mut Vec<MultiReleaseReportDiagnostic>,
) -> Option<Error> {
    let module = c.iter().any(|x| {
        x.logical.as_deref() == Some(b"module-info.class")
            && matches!(
                x.variant,
                MultiReleaseEntryVariant::Base | MultiReleaseEntryVariant::Versioned { .. }
            )
    });
    let mut base_probes: HashMap<usize, std::result::Result<crate::classfile::MinimalHeaderFacts, Error>> = HashMap::new();
    for i in 0..c.len() {
        let release = match c[i].variant {
            MultiReleaseEntryVariant::Versioned { release }
                if c[i].logical.as_deref().is_some_and(|p| p.ends_with(b".class")) && !c[i].meta_inf => release,
            _ => continue,
        };
        let materialized = match snapshot.read_entry_internal(c[i].physical, budget) {
            Ok(v) => v,
            Err(error) => {
                merge_compliance(&mut evidence[i].compliance, MultiReleaseCompliance::Unknown {
                    reason: MultiReleaseComplianceUnknownReason::ProbeInterrupted,
                });
                diagnostics.push(MultiReleaseReportDiagnostic::Terminal {
                    diagnostic: terminal(&error, Some(c[i].physical)),
                });
                if stops_probes(&error) { mark_remaining_probe_unknown(c, evidence, i + 1); return Some(error); }
                return Some(error);
            }
        };
        let facts = match probe_minimal_header(&materialized.bytes, budget) {
            Ok(v) => v,
            Err(Error::InvalidInput { .. }) => {
                merge_compliance(&mut evidence[i].compliance, MultiReleaseCompliance::NonConformant);
                push_domain(diagnostics, MultiReleaseDiagnosticCode::MultiReleaseClassMalformed,
                    "versioned class has a malformed minimal Header", Some(c[i].physical), budget);
                continue;
            }
            Err(error) => {
                merge_compliance(&mut evidence[i].compliance, MultiReleaseCompliance::Unknown {
                    reason: MultiReleaseComplianceUnknownReason::ProbeInterrupted,
                });
                diagnostics.push(MultiReleaseReportDiagnostic::Terminal { diagnostic: terminal(&error, Some(c[i].physical)) });
                if stops_probes(&error) { mark_remaining_probe_unknown(c, evidence, i + 1); }
                return Some(error);
            }
        };
        evidence[i].class_evidence = Some(MultiReleaseClassEvidence {
            major_version: facts.major_version,
            minor_version: facts.minor_version,
            access_flags: facts.access_flags,
            this_class: facts.this_class.clone(),
        });
        if u128::from(facts.major_version) > u128::from(release) + 44 {
            merge_compliance(&mut evidence[i].compliance, MultiReleaseCompliance::NonConformant);
            if let Err(error) = push_domain(diagnostics, MultiReleaseDiagnosticCode::MultiReleaseClassVersionTooNew,
                "classfile major exceeds release + 44", Some(c[i].physical), budget) {
                diagnostics.push(MultiReleaseReportDiagnostic::Terminal { diagnostic: terminal(&error, Some(c[i].physical)) });
                mark_remaining_probe_unknown(c, evidence, i + 1);
                return Some(error);
            }
        }
        if facts.access_flags & ACC_PUBLIC == 0 { continue; }
        let path = c[i].logical.as_ref().unwrap();
        let roots = c.iter().enumerate().filter(|(_, x)| x.logical.as_ref() == Some(path)
            && matches!(x.variant, MultiReleaseEntryVariant::Base)).map(|(n, _)| n).collect::<Vec<_>>();
        if roots.len() > 1 {
            merge_compliance(&mut evidence[i].compliance, MultiReleaseCompliance::Unknown {
                reason: MultiReleaseComplianceUnknownReason::PredecessorAmbiguous,
            });
            continue;
        }
        if roots.is_empty() {
            if module {
                merge_compliance(&mut evidence[i].compliance, MultiReleaseCompliance::Unknown {
                    reason: MultiReleaseComplianceUnknownReason::ModuleExportsNotInspected,
                });
            } else if physical_complete {
                merge_compliance(&mut evidence[i].compliance, MultiReleaseCompliance::NonConformant);
                if let Err(error) = push_domain(diagnostics, MultiReleaseDiagnosticCode::MultiReleasePublicPredecessorMissing,
                    "public versioned class has no root predecessor", Some(c[i].physical), budget) {
                    diagnostics.push(MultiReleaseReportDiagnostic::Terminal { diagnostic: terminal(&error, Some(c[i].physical)) });
                    mark_remaining_probe_unknown(c, evidence, i + 1);
                    return Some(error);
                }
            }
            continue;
        }
        let root = roots[0];
        if !base_probes.contains_key(&root) {
            let result = snapshot.read_entry_internal(c[root].physical, budget)
                .and_then(|bytes| probe_minimal_header(&bytes.bytes, budget));
            if let Ok(base) = &result {
                evidence[root].class_evidence = Some(MultiReleaseClassEvidence {
                    major_version: base.major_version, minor_version: base.minor_version,
                    access_flags: base.access_flags, this_class: base.this_class.clone(),
                });
            }
            base_probes.insert(root, result);
        }
        match base_probes.get(&root).unwrap() {
            Ok(base) if base.access_flags & ACC_PUBLIC != 0 && base.this_class == facts.this_class => {}
            Err(Error::InvalidInput { .. }) => {
                merge_compliance(&mut evidence[root].compliance, MultiReleaseCompliance::NonConformant);
                if !diagnostics.iter().any(|d| diagnostic_for_entry(d, MultiReleaseDiagnosticCode::MultiReleaseClassMalformed, &c[root].physical.id)) {
                    if let Err(error) = push_domain(diagnostics, MultiReleaseDiagnosticCode::MultiReleaseClassMalformed,
                        "root predecessor has a malformed minimal Header", Some(c[root].physical), budget) {
                        diagnostics.push(MultiReleaseReportDiagnostic::Terminal { diagnostic: terminal(&error, Some(c[root].physical)) });
                        mark_remaining_probe_unknown(c, evidence, i + 1); return Some(error);
                    }
                }
                if module {
                    merge_compliance(&mut evidence[i].compliance, MultiReleaseCompliance::Unknown { reason: MultiReleaseComplianceUnknownReason::ModuleExportsNotInspected });
                } else {
                    merge_compliance(&mut evidence[i].compliance, MultiReleaseCompliance::NonConformant);
                    if let Err(error) = push_domain(diagnostics, MultiReleaseDiagnosticCode::MultiReleasePublicPredecessorMismatch,
                        "root predecessor is malformed", Some(c[i].physical), budget) {
                        diagnostics.push(MultiReleaseReportDiagnostic::Terminal { diagnostic: terminal(&error, Some(c[i].physical)) });
                        mark_remaining_probe_unknown(c, evidence, i + 1); return Some(error);
                    }
                }
            }
            Ok(_) if module => merge_compliance(&mut evidence[i].compliance, MultiReleaseCompliance::Unknown { reason: MultiReleaseComplianceUnknownReason::ModuleExportsNotInspected }),
            Ok(_) => {
                merge_compliance(&mut evidence[i].compliance, MultiReleaseCompliance::NonConformant);
                if let Err(error) = push_domain(diagnostics, MultiReleaseDiagnosticCode::MultiReleasePublicPredecessorMismatch,
                    "root predecessor is not public with the same this_class", Some(c[i].physical), budget) {
                    diagnostics.push(MultiReleaseReportDiagnostic::Terminal { diagnostic: terminal(&error, Some(c[i].physical)) });
                    mark_remaining_probe_unknown(c, evidence, i + 1); return Some(error);
                }
            }
            Err(error) => {
                let error = error.clone();
                merge_compliance(&mut evidence[i].compliance, MultiReleaseCompliance::Unknown { reason: MultiReleaseComplianceUnknownReason::ProbeInterrupted });
                diagnostics.push(MultiReleaseReportDiagnostic::Terminal { diagnostic: terminal(&error, Some(c[root].physical)) });
                if stops_probes(&error) { mark_remaining_probe_unknown(c, evidence, i + 1); }
                return Some(error);
            }
        }
    }
    None
}

fn push_domain(
    out: &mut Vec<MultiReleaseReportDiagnostic>,
    code: MultiReleaseDiagnosticCode,
    message: &str,
    entry: Option<&PhysicalEntry>,
    budget: &mut Budget,
) {
    if budget
        .charge(CountedBudgetDimension::ResultItems, 1)
        .is_ok()
    {
        out.push(MultiReleaseReportDiagnostic::Domain {
            diagnostic: MultiReleaseDiagnostic {
                code,
                severity: code.severity(),
                message: message.into(),
                provenance: entry.map(provenance),
            },
        });
    }
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
                format!("budget_exceeded_{dimension:?}").to_ascii_lowercase()
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
fn skip_entries(
    origin: &ContainerOrigin,
    entries: &[PhysicalEntry],
    out: &mut Vec<CoverageRange>,
) -> Result<()> {
    for e in entries {
        out.push(range(
            origin,
            "multi_release_selection_entries",
            e.id.ordinal,
        ));
        out.push(range(
            origin,
            "multi_release_compliance_entries",
            e.id.ordinal,
        ));
    }
    Ok(())
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
fn with_usage(e: ExecutionReport, budget: &Budget) -> ExecutionReport {
    match e {
        ExecutionReport::Complete { .. } => ExecutionReport::Complete {
            usage: budget.usage(),
        },
        ExecutionReport::Partial { reason, .. } => ExecutionReport::Partial {
            reason,
            usage: budget.usage(),
        },
        ExecutionReport::Cancelled { .. } => ExecutionReport::Cancelled {
            usage: budget.usage(),
        },
        ExecutionReport::Failed { reason, .. } => ExecutionReport::Failed {
            reason,
            usage: budget.usage(),
        },
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
fn priority(e: &ExecutionReport) -> u8 {
    match e {
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
fn merge_execution(current: &mut ExecutionReport, incoming: &ExecutionReport) {
    if priority(incoming) > priority(current) {
        *current = incoming.clone();
    }
}
fn set_issue(current: &mut ExecutionReport, error: &Error, budget: &Budget, failed: bool) {
    merge_execution(current, &execution_for(error, budget, failed));
}
fn must_stop(e: &ExecutionReport) -> bool {
    matches!(
        e,
        ExecutionReport::Cancelled { .. }
            | ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded { .. },
                ..
            }
            | ExecutionReport::Failed {
                reason: TerminationReason::BudgetExceeded { .. },
                ..
            }
    )
}
