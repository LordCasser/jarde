//! Immutable artifact snapshots and bounded physical ZIP access.
//!
//! `ReadBytes` counts compressed bytes selected from the fixed snapshot for an
//! entry read. `EntryBytes` counts logical bytes produced by decompression, and
//! `OutputBytes` independently accounts for the owned result buffer returned to
//! the caller. The latter two intentionally describe different resources.

use crate::budget::{Budget, BudgetDimension, CountedBudgetDimension, UsageSnapshot};
use crate::error::{Error, Result};
use crate::model::{
    ArchiveNameBytes, ByteSpan, ContainerId, ContainerOrigin, ContainerOriginStep, Coverage,
    CoverageDimension, CoverageRange, CoverageState, Diagnostic, DiagnosticSeverity,
    ExecutionReport, Location, PhysicalEntryId, Provenance, SnapshotId, TerminationReason,
};
use crate::view::{PhysicalScope, PhysicalView};
use rawzip::ZipArchive;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs::{File, Metadata};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

const CLASS_MAGIC: &[u8; 4] = b"\xca\xfe\xba\xbe";
const STORE: u16 = 0;
const DEFLATE: u16 = 8;
const READ_CHUNK: usize = 16 * 1024;

#[derive(Clone, Debug)]
pub enum ArtifactInput {
    Path(PathBuf),
    Bytes(Arc<[u8]>),
}

impl ArtifactInput {
    pub fn bytes(bytes: impl Into<Arc<[u8]>>) -> Self {
        Self::Bytes(bytes.into())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    StandaloneClass,
    Zip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryCompression {
    Stored,
    Deflated,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntryFlags {
    pub raw_bits: u16,
    pub encrypted: bool,
    pub strong_encryption: bool,
    pub data_descriptor: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntryLayout {
    pub local_header_offset: u64,
    pub central_header_offset: u64,
    pub compressed_data: ByteSpan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NestedArchiveState {
    NotCandidate,
    CandidateNotScanned,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureMetadataKind {
    Manifest,
    SignatureFile,
    SignatureBlock,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureVerificationState {
    NotVerified,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignatureMetadata {
    pub kind: SignatureMetadataKind,
    pub verification: SignatureVerificationState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PhysicalEntry {
    pub id: PhysicalEntryId,
    pub compression: EntryCompression,
    pub compression_method: u16,
    pub flags: EntryFlags,
    pub crc32: u32,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub layout: EntryLayout,
    pub nested_archive: NestedArchiveState,
    pub signature_metadata: Option<SignatureMetadata>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EnumerationReport {
    pub snapshot: SnapshotId,
    pub entries: Vec<PhysicalEntry>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutNodeKind {
    NestedArchive,
    BootClasses,
    BootLibrary,
    WarClasses,
    WarLibrary,
}

impl LayoutNodeKind {
    /// Whether this layer publishes class-path roots rather than a nested library.
    ///
    /// The distinction is what a layout selection turns on: a class layer's prefix decides which
    /// entries of its container are on the path, a library layer's container is on the path whole.
    pub fn is_class_layer(self) -> bool {
        matches!(self, Self::BootClasses | Self::WarClasses)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LayoutNodeSource {
    Prefix {
        container: ContainerOrigin,
        prefix: ArchiveNameBytes,
        evidence_entry: PhysicalEntryId,
    },
    Archive {
        entry: PhysicalEntryId,
        child_container: Option<ContainerOrigin>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LayoutNode {
    pub kind: LayoutNodeKind,
    pub source: LayoutNodeSource,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContainerReport {
    pub origin: ContainerOrigin,
    pub depth: u64,
    pub entries: Vec<PhysicalEntry>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtifactTreeReport {
    pub view: PhysicalView,
    pub containers: Vec<ContainerReport>,
    pub layout_nodes: Vec<LayoutNode>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MaterializedEntry {
    pub entry: PhysicalEntryId,
    pub bytes: Vec<u8>,
    pub content_digest: crate::model::Digest,
    pub usage: UsageSnapshot,
}

#[derive(Clone, Debug)]
pub struct ArtifactSnapshot {
    id: SnapshotId,
    kind: ArtifactKind,
    bytes: Arc<[u8]>,
}

impl ArtifactSnapshot {
    pub fn open(input: ArtifactInput, budget: &mut Budget) -> Result<Self> {
        budget.poll()?;
        let bytes = match input {
            ArtifactInput::Bytes(bytes) => {
                budget.charge(CountedBudgetDimension::InputBytes, as_u64(bytes.len())?)?;
                bytes
            }
            ArtifactInput::Path(path) => Arc::from(read_stable_path(&path, budget)?),
        };
        let kind = classify(&bytes)?;
        let id = SnapshotId(blake3::hash(&bytes).to_hex().to_string());
        Ok(Self { id, kind, bytes })
    }

    pub fn id(&self) -> &SnapshotId {
        &self.id
    }

    pub fn kind(&self) -> ArtifactKind {
        self.kind
    }

    pub fn len(&self) -> u64 {
        self.bytes.len() as u64
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn root_bytes(&self, budget: &mut Budget) -> Result<Vec<u8>> {
        if self.kind != ArtifactKind::StandaloneClass {
            return Err(Error::invalid_input(
                "not_standalone_class",
                "root bytes are only available for a standalone CLASS snapshot",
            ));
        }
        budget.check(CountedBudgetDimension::ReadBytes, self.len())?;
        budget.check(CountedBudgetDimension::OutputBytes, self.len())?;
        budget.charge(CountedBudgetDimension::ReadBytes, self.len())?;
        budget.charge(CountedBudgetDimension::OutputBytes, self.len())?;
        Ok(self.bytes.to_vec())
    }

    pub fn enumerate(&self, budget: &mut Budget) -> Result<EnumerationReport> {
        self.enumerate_with_hook(budget, |_| {})
    }

    fn enumerate_with_hook<F>(
        &self,
        budget: &mut Budget,
        mut completed_hook: F,
    ) -> Result<EnumerationReport>
    where
        F: FnMut(u64),
    {
        if self.kind != ArtifactKind::Zip {
            return Err(Error::invalid_input(
                "not_zip",
                "physical entry enumeration requires a ZIP snapshot",
            ));
        }
        let archive = ZipArchive::from_slice(&self.bytes).map_err(zip_invalid("zip_open"))?;
        let expected = archive.entries_hint();
        let directory_offset = archive.directory_offset();
        let mut iterator = archive.entries();
        let mut entries = Vec::new();
        let mut diagnostics = Vec::new();
        let mut names: HashMap<Vec<u8>, u64> = HashMap::new();
        let mut ranges: BTreeMap<u64, (u64, u64)> = BTreeMap::new();
        let mut ordinal = 0_u64;

        loop {
            if let Err(error) = budget.poll() {
                return Ok(terminated_enumeration(
                    &self.id,
                    entries,
                    diagnostics,
                    EnumerationProgress {
                        completed: ordinal,
                        expected,
                        known_end: None,
                    },
                    error,
                    budget,
                ));
            }
            let header = match iterator.next_entry() {
                Ok(Some(header)) => header,
                Ok(None) => break,
                Err(error) => {
                    let known_end = ordinal.checked_add(1).ok_or_else(|| {
                        Error::invalid_input(
                            "entry_count_overflow",
                            "central entry evidence exceeds u64",
                        )
                    })?;
                    return Ok(terminated_enumeration(
                        &self.id,
                        entries,
                        diagnostics,
                        EnumerationProgress {
                            completed: ordinal,
                            expected,
                            known_end: Some(known_end),
                        },
                        zip_invalid("central_directory")(error),
                        budget,
                    ));
                }
            };
            let known_end = ordinal.checked_add(1).ok_or_else(|| {
                Error::invalid_input("entry_count_overflow", "central entry evidence exceeds u64")
            })?;
            if let Err(error) = budget.charge(CountedBudgetDimension::ArchiveEntries, 1) {
                return Ok(terminated_enumeration(
                    &self.id,
                    entries,
                    diagnostics,
                    EnumerationProgress {
                        completed: ordinal,
                        expected,
                        known_end: Some(known_end),
                    },
                    error,
                    budget,
                ));
            }

            let parsed = (|| -> Result<(PhysicalEntry, Option<Diagnostic>, u64, u64)> {
                budget.poll()?;
                let local = archive
                    .get_entry(header.wayfinder())
                    .map_err(zip_invalid("local_entry"))?;
                budget.poll()?;
                validate_headers(&header, &local)?;
                let (data_start, data_end) = local.compressed_data_range();
                let range_start = header.local_header_offset();
                if range_start > data_start || data_start > data_end || data_end > directory_offset
                {
                    return Err(Error::invalid_input(
                        "invalid_entry_span",
                        format!(
                            "entry {ordinal} range {range_start}..{data_end} is invalid for the file area ending at {directory_offset}"
                        ),
                    ));
                }
                ensure_disjoint_range(&ranges, ordinal, range_start, data_end)?;

                budget.poll()?;
                if local
                    .data_descriptor()
                    .map_err(zip_invalid("data_descriptor"))?
                    .is_some_and(|descriptor| {
                        descriptor.crc32() != header.crc32()
                            || descriptor.compressed_size() != header.compressed_size_hint()
                            || descriptor.uncompressed_size() != header.uncompressed_size_hint()
                    })
                {
                    return Err(Error::invalid_input(
                        "descriptor_central_mismatch",
                        format!("entry {ordinal} data descriptor conflicts with central directory"),
                    ));
                }
                budget.poll()?;

                let raw_name = header.file_path().as_bytes().to_vec();
                let pending_diagnostic = names.get(&raw_name).map(|first| Diagnostic {
                    code: "duplicate_raw_name".into(),
                    severity: DiagnosticSeverity::Warning,
                    message: format!(
                        "entry {ordinal} repeats raw name first seen at ordinal {first}"
                    ),
                    provenance: None,
                });
                budget.check(CountedBudgetDimension::ResultItems, 1)?;
                if pending_diagnostic.is_some() {
                    budget.check(CountedBudgetDimension::ResultItems, 2)?;
                }
                budget.charge(CountedBudgetDimension::ResultItems, 1)?;
                if pending_diagnostic.is_some() {
                    budget.charge(CountedBudgetDimension::ResultItems, 1)?;
                }

                let flags = header.flags();
                let method = header.compression_method().as_u16();
                Ok((
                    PhysicalEntry {
                        id: PhysicalEntryId {
                            origin: ContainerOrigin {
                                snapshot: self.id.clone(),
                                root_container: ContainerId("root".into()),
                                steps: Vec::new(),
                            },
                            ordinal,
                            raw_name: ArchiveNameBytes(raw_name),
                        },
                        compression: compression(method),
                        compression_method: method,
                        flags: EntryFlags {
                            raw_bits: flags.bits(),
                            encrypted: flags.is_encrypted(),
                            strong_encryption: flags.has_strong_encryption(),
                            data_descriptor: flags.has_data_descriptor(),
                        },
                        crc32: header.crc32(),
                        compressed_size: header.compressed_size_hint(),
                        uncompressed_size: header.uncompressed_size_hint(),
                        layout: EntryLayout {
                            local_header_offset: range_start,
                            central_header_offset: header.central_directory_offset(),
                            compressed_data: ByteSpan::new(data_start, data_end - data_start),
                        },
                        nested_archive: nested_state(header.file_path().as_bytes()),
                        signature_metadata: signature_metadata(header.file_path().as_bytes()),
                    },
                    pending_diagnostic,
                    range_start,
                    data_end,
                ))
            })();

            let (entry, pending_diagnostic, range_start, range_end) = match parsed {
                Ok(parsed) => parsed,
                Err(error) => {
                    return Ok(terminated_enumeration(
                        &self.id,
                        entries,
                        diagnostics,
                        EnumerationProgress {
                            completed: ordinal,
                            expected,
                            known_end: Some(known_end),
                        },
                        error,
                        budget,
                    ));
                }
            };
            if let Some(diagnostic) = pending_diagnostic {
                diagnostics.push(diagnostic);
            } else {
                names.insert(entry.id.raw_name.0.clone(), ordinal);
            }
            ranges.insert(range_start, (range_end, ordinal));
            entries.push(entry);
            ordinal = ordinal.checked_add(1).ok_or_else(|| {
                Error::invalid_input("entry_count_overflow", "central entry ordinal overflow")
            })?;
            completed_hook(ordinal);
        }

        if ordinal != expected {
            return Ok(terminated_enumeration(
                &self.id,
                entries,
                diagnostics,
                EnumerationProgress {
                    completed: ordinal,
                    expected,
                    known_end: None,
                },
                Error::invalid_input(
                    "entry_count_mismatch",
                    format!(
                        "EOCD declares {expected} entries but central directory yielded {ordinal}"
                    ),
                ),
                budget,
            ));
        }
        Ok(EnumerationReport {
            snapshot: self.id.clone(),
            entries,
            coverage: enumeration_coverage(
                CoverageState::CompleteWithinSchema,
                ordinal,
                expected,
                None,
            ),
            execution: ExecutionReport::Complete {
                usage: budget.usage(),
            },
            diagnostics,
        })
    }

    pub fn enumerate_artifact_tree(&self, budget: &mut Budget) -> Result<ArtifactTreeReport> {
        self.enumerate_artifact_tree_with_hook(budget, |_| {})
    }

    fn enumerate_artifact_tree_with_hook<F>(
        &self,
        budget: &mut Budget,
        mut candidate_hook: F,
    ) -> Result<ArtifactTreeReport>
    where
        F: FnMut(u64),
    {
        if self.kind != ArtifactKind::Zip {
            return Err(Error::invalid_input(
                "not_zip",
                "artifact-tree enumeration requires a ZIP snapshot",
            ));
        }
        let root = root_origin(&self.id);
        let view = PhysicalView {
            snapshot: self.id.clone(),
            scope: PhysicalScope::ArtifactTree {
                root_container: root.root_container.clone(),
            },
        };
        let root_expected_entries = ZipArchive::from_slice(&self.bytes)
            .map_err(zip_invalid("zip_open"))?
            .entries_hint();
        let mut stack = vec![TreeStackItem {
            bytes: self.bytes.clone(),
            origin: root,
            depth: 0,
            parent_entry: None,
            expected_entries: root_expected_entries,
        }];
        let mut containers = Vec::new();
        let mut layout_nodes = Vec::new();
        let mut diagnostics = Vec::new();
        let mut first_issue: Option<ExecutionReport> = None;
        let mut scanned_candidates = Vec::new();
        let mut skipped_candidates = Vec::new();

        while let Some(current) = stack.pop() {
            if let Err(error) = budget.charge(CountedBudgetDimension::ResultItems, 1) {
                merge_tree_error(&mut first_issue, &error, budget);
                diagnostics.push(tree_diagnostic(&error, current.parent_entry.as_ref()));
                stack.push(current);
                break;
            }
            let TreeStackItem {
                bytes,
                origin,
                depth,
                parent_entry,
                expected_entries: _,
            } = current;
            let temporary = ArtifactSnapshot {
                id: self.id.clone(),
                kind: ArtifactKind::Zip,
                bytes,
            };
            let mut report = temporary.enumerate(budget)?;
            for entry in &mut report.entries {
                entry.id.origin = origin.clone();
            }
            let container_complete = matches!(report.execution, ExecutionReport::Complete { .. });
            for mut diagnostic in report.diagnostics.drain(..) {
                if !container_complete
                    && diagnostic.provenance.is_none()
                    && let Some(parent) = parent_entry.as_ref()
                {
                    diagnostic.provenance = tree_parent_provenance(parent);
                }
                diagnostics.push(diagnostic);
            }
            if !container_complete {
                let issue = if depth == 0 {
                    report.execution.clone()
                } else {
                    nested_container_execution(report.execution.clone())
                };
                merge_tree_execution(&mut first_issue, issue);
            }
            let entries = report.entries;
            let container_execution = report.execution;
            let can_expand = matches!(container_execution, ExecutionReport::Complete { .. });
            let container_coverage = relabel_coverage(report.coverage, &origin);
            let mut boot_classes = false;
            let mut war_classes = false;
            let mut children = Vec::new();
            let mut stop_after_container = false;

            for (entry_index, entry) in entries.iter().enumerate() {
                let name = entry.id.raw_name.0.as_slice();
                if !boot_classes && name.starts_with(b"BOOT-INF/classes/") {
                    if let Err(error) = budget.charge(CountedBudgetDimension::ResultItems, 1) {
                        append_skipped_candidates(
                            &entries,
                            entry_index,
                            &origin,
                            &mut skipped_candidates,
                        )?;
                        merge_tree_error(&mut first_issue, &error, budget);
                        diagnostics.push(tree_diagnostic(&error, Some(entry)));
                        stop_after_container = true;
                        break;
                    }
                    layout_nodes.push(prefix_layout(
                        LayoutNodeKind::BootClasses,
                        &origin,
                        b"BOOT-INF/classes/",
                        entry,
                    ));
                    boot_classes = true;
                }
                if !war_classes && name.starts_with(b"WEB-INF/classes/") {
                    if let Err(error) = budget.charge(CountedBudgetDimension::ResultItems, 1) {
                        append_skipped_candidates(
                            &entries,
                            entry_index,
                            &origin,
                            &mut skipped_candidates,
                        )?;
                        merge_tree_error(&mut first_issue, &error, budget);
                        diagnostics.push(tree_diagnostic(&error, Some(entry)));
                        stop_after_container = true;
                        break;
                    }
                    layout_nodes.push(prefix_layout(
                        LayoutNodeKind::WarClasses,
                        &origin,
                        b"WEB-INF/classes/",
                        entry,
                    ));
                    war_classes = true;
                }
                if entry.nested_archive != NestedArchiveState::CandidateNotScanned {
                    continue;
                }
                let candidate_range = candidate_coverage_range(entry, &origin)?;
                if !can_expand {
                    skipped_candidates.push(candidate_range);
                    continue;
                }
                let kind = if is_direct_library(name, b"BOOT-INF/lib/") {
                    LayoutNodeKind::BootLibrary
                } else if is_direct_library(name, b"WEB-INF/lib/") {
                    LayoutNodeKind::WarLibrary
                } else {
                    LayoutNodeKind::NestedArchive
                };
                if let Err(error) = budget.charge(CountedBudgetDimension::ResultItems, 1) {
                    skipped_candidates.push(candidate_range);
                    append_skipped_candidates(
                        &entries,
                        entry_index + 1,
                        &origin,
                        &mut skipped_candidates,
                    )?;
                    merge_tree_error(&mut first_issue, &error, budget);
                    diagnostics.push(tree_diagnostic(&error, Some(entry)));
                    stop_after_container = true;
                    break;
                }
                let child_depth = depth.saturating_add(1);
                let mut child_origin = origin.clone();
                let child_id =
                    derive_child_container(&origin, entry.id.ordinal, &entry.id.raw_name);
                child_origin.steps.push(ContainerOriginStep {
                    via_ordinal: entry.id.ordinal,
                    via_raw_name: entry.id.raw_name.clone(),
                    child_container: child_id,
                });
                let mut child_bytes = None;
                match budget.check_nested_depth(child_depth) {
                    Ok(()) => {
                        let mut local_entry = entry.clone();
                        local_entry.id.origin = root_origin(&self.id);
                        match temporary.read_entry_with_hooks(
                            &local_entry,
                            budget,
                            MaterializationAccounting::Intermediate,
                            |_| {},
                            |_| {},
                        ) {
                            Ok(materialized) => {
                                let bytes: Arc<[u8]> = Arc::from(materialized.bytes);
                                match ZipArchive::from_slice(&bytes) {
                                    Ok(archive) => {
                                        let expected_entries = archive.entries_hint();
                                        child_bytes = Some(bytes.clone());
                                        children.push(TreeStackItem {
                                            bytes,
                                            origin: child_origin.clone(),
                                            depth: child_depth,
                                            parent_entry: Some(entry.clone()),
                                            expected_entries,
                                        });
                                    }
                                    Err(error) => record_tree_issue(
                                        zip_invalid("nested_zip_open")(error),
                                        entry,
                                        budget,
                                        &mut first_issue,
                                        &mut diagnostics,
                                    ),
                                }
                            }
                            Err(error) => record_tree_issue(
                                error,
                                entry,
                                budget,
                                &mut first_issue,
                                &mut diagnostics,
                            ),
                        }
                    }
                    Err(error) => {
                        record_tree_issue(error, entry, budget, &mut first_issue, &mut diagnostics)
                    }
                }
                if child_bytes.is_some() {
                    scanned_candidates.push(candidate_range);
                } else {
                    skipped_candidates.push(candidate_range);
                }
                layout_nodes.push(LayoutNode {
                    kind,
                    source: LayoutNodeSource::Archive {
                        entry: entry.id.clone(),
                        child_container: child_bytes.map(|_| child_origin),
                    },
                });
                let candidate_count = scanned_candidates
                    .len()
                    .checked_add(skipped_candidates.len())
                    .and_then(|count| u64::try_from(count).ok())
                    .ok_or_else(|| {
                        Error::invalid_input(
                            "candidate_count_overflow",
                            "nested archive candidate count overflow",
                        )
                    })?;
                candidate_hook(candidate_count);
                if tree_must_stop(&first_issue) {
                    append_skipped_candidates(
                        &entries,
                        entry_index + 1,
                        &origin,
                        &mut skipped_candidates,
                    )?;
                    break;
                }
            }
            containers.push(ContainerReport {
                origin,
                depth,
                entries,
                coverage: container_coverage,
                execution: container_execution,
            });
            for child in children.into_iter().rev() {
                stack.push(child);
            }
            if stop_after_container || tree_must_stop(&first_issue) {
                break;
            }
            if depth == 0
                && !matches!(
                    containers.last().unwrap().execution,
                    ExecutionReport::Complete { .. }
                )
            {
                break;
            }
        }

        let execution = first_issue
            .map(|execution| crate::accounting::with_usage(execution, budget.usage()))
            .unwrap_or_else(|| ExecutionReport::Complete {
                usage: budget.usage(),
            });
        let coverage = tree_coverage(
            &containers,
            &stack,
            scanned_candidates,
            skipped_candidates,
            matches!(execution, ExecutionReport::Complete { .. }),
        );
        Ok(ArtifactTreeReport {
            view,
            containers,
            layout_nodes,
            coverage,
            execution,
            diagnostics,
        })
    }

    /// Materializes one entry for analysis, keeping the contract of [`Self::read_entry`].
    ///
    /// The three rules `read_entry` is held to are exactly the ones this entry keeps:
    ///
    /// - the snapshot and the entry are validated before anything is read (a locator that
    ///   another snapshot or another entry owns is an input error, not a read);
    /// - the read is accounted in the category it belongs to — compressed `ReadBytes` for a
    ///   stored entry, logical `EntryBytes` for a deflated one — so a caller cannot read
    ///   outside the budget it was given;
    /// - the returned buffer is charged as `OutputBytes`, because it is owned data handed to
    ///   the caller and not a borrowed view of the archive.
    ///
    /// It differs from `read_entry` in one thing only, and that difference is accounting: the
    /// result is charged as an `Intermediate` read, because the bytes are consumed inside the
    /// request that asked for them rather than returned as the request's own answer.
    pub fn read_entry_for_analysis(
        &self,
        entry: &PhysicalEntry,
        budget: &mut Budget,
    ) -> Result<MaterializedEntry> {
        if entry.id.origin.steps.is_empty() {
            self.read_entry_with_hooks(
                entry,
                budget,
                MaterializationAccounting::Intermediate,
                |_| {},
                |_| {},
            )
        } else {
            self.read_nested_entry_with_accounting(
                entry,
                budget,
                MaterializationAccounting::Intermediate,
            )
        }
    }

    pub fn read_entry(
        &self,
        entry: &PhysicalEntry,
        budget: &mut Budget,
    ) -> Result<MaterializedEntry> {
        if entry.id.origin.steps.is_empty() {
            self.read_entry_with_hooks(
                entry,
                budget,
                MaterializationAccounting::CallerOutput,
                |_| {},
                |_| {},
            )
        } else {
            self.read_nested_entry(entry, budget)
        }
    }

    #[cfg(test)]
    fn read_entry_with_hook<F>(
        &self,
        entry: &PhysicalEntry,
        budget: &mut Budget,
        hook: F,
    ) -> Result<MaterializedEntry>
    where
        F: FnMut(usize),
    {
        self.read_entry_with_hooks(
            entry,
            budget,
            MaterializationAccounting::CallerOutput,
            |_| {},
            hook,
        )
    }

    fn read_entry_with_hooks<L, F>(
        &self,
        entry: &PhysicalEntry,
        budget: &mut Budget,
        accounting: MaterializationAccounting,
        mut locator_hook: L,
        mut hook: F,
    ) -> Result<MaterializedEntry>
    where
        L: FnMut(u64),
        F: FnMut(usize),
    {
        if self.kind != ArtifactKind::Zip || entry.id.snapshot() != &self.id {
            return Err(Error::invalid_input(
                "entry_snapshot_mismatch",
                "entry does not belong to this ZIP snapshot",
            ));
        }
        budget.poll()?;
        let archive = ZipArchive::from_slice(&self.bytes).map_err(zip_invalid("zip_open"))?;
        let mut iterator = archive.entries();
        let mut current = 0_u64;
        let header = loop {
            budget.poll()?;
            let candidate = iterator
                .next_entry()
                .map_err(zip_invalid("central_directory"))?
                .ok_or_else(|| {
                    Error::invalid_input("entry_not_found", "entry ordinal is absent")
                })?;
            budget.charge(CountedBudgetDimension::ArchiveEntries, 1)?;
            locator_hook(current + 1);
            if current == entry.id.ordinal {
                break candidate;
            }
            current = current.checked_add(1).ok_or_else(|| {
                Error::invalid_input("entry_count_overflow", "central entry ordinal overflow")
            })?;
        };
        if header.file_path().as_bytes() != entry.id.raw_name.0
            || header.local_header_offset() != entry.layout.local_header_offset
            || header.central_directory_offset() != entry.layout.central_header_offset
        {
            return Err(Error::invalid_input(
                "entry_locator_mismatch",
                "entry identity no longer matches the fixed central-directory locator",
            ));
        }
        budget.poll()?;
        let local = archive
            .get_entry(header.wayfinder())
            .map_err(zip_invalid("local_entry"))?;
        budget.poll()?;
        validate_headers(&header, &local)?;
        budget.poll()?;
        if local
            .data_descriptor()
            .map_err(zip_invalid("data_descriptor"))?
            .is_some_and(|descriptor| {
                descriptor.crc32() != header.crc32()
                    || descriptor.compressed_size() != header.compressed_size_hint()
                    || descriptor.uncompressed_size() != header.uncompressed_size_hint()
            })
        {
            return Err(Error::invalid_input(
                "descriptor_central_mismatch",
                format!("entry {current} data descriptor conflicts with central directory"),
            ));
        }
        budget.poll()?;
        let flags = header.flags();
        let method = header.compression_method().as_u16();
        let (data_start, data_end) = local.compressed_data_range();
        let authoritative = PhysicalEntry {
            id: PhysicalEntryId {
                origin: ContainerOrigin {
                    snapshot: self.id.clone(),
                    root_container: ContainerId("root".into()),
                    steps: Vec::new(),
                },
                ordinal: current,
                raw_name: ArchiveNameBytes(header.file_path().as_bytes().to_vec()),
            },
            compression: compression(method),
            compression_method: method,
            flags: EntryFlags {
                raw_bits: flags.bits(),
                encrypted: flags.is_encrypted(),
                strong_encryption: flags.has_strong_encryption(),
                data_descriptor: flags.has_data_descriptor(),
            },
            crc32: header.crc32(),
            compressed_size: header.compressed_size_hint(),
            uncompressed_size: header.uncompressed_size_hint(),
            layout: EntryLayout {
                local_header_offset: header.local_header_offset(),
                central_header_offset: header.central_directory_offset(),
                compressed_data: ByteSpan::new(data_start, data_end - data_start),
            },
            nested_archive: nested_state(header.file_path().as_bytes()),
            signature_metadata: signature_metadata(header.file_path().as_bytes()),
        };
        if entry != &authoritative {
            return Err(Error::invalid_input(
                "entry_metadata_mismatch",
                "caller-supplied entry metadata differs from the fixed snapshot",
            ));
        }
        if authoritative.flags.encrypted || authoritative.flags.strong_encryption {
            return Err(Error::unsupported(
                "encrypted_zip_entry",
                "encrypted, strong-encryption, and AES entries are not supported",
            ));
        }
        if !matches!(
            authoritative.compression,
            EntryCompression::Stored | EntryCompression::Deflated
        ) {
            return Err(Error::unsupported(
                "zip_compression_method",
                format!(
                    "compression method {} is not supported",
                    authoritative.compression_method
                ),
            ));
        }
        budget.check(
            CountedBudgetDimension::EntryBytes,
            authoritative.uncompressed_size,
        )?;
        if accounting == MaterializationAccounting::CallerOutput {
            budget.check(
                CountedBudgetDimension::OutputBytes,
                authoritative.uncompressed_size,
            )?;
        }
        budget.check(
            CountedBudgetDimension::ReadBytes,
            authoritative.compressed_size,
        )?;
        let compressed_len = as_u64(local.data().len())?;
        if compressed_len != authoritative.compressed_size {
            return Err(Error::invalid_input(
                "compressed_size_mismatch",
                format!(
                    "central size {} differs from local data span {compressed_len}",
                    authoritative.compressed_size
                ),
            ));
        }
        budget.charge(CountedBudgetDimension::ReadBytes, compressed_len)?;

        let mut output = Vec::new();
        let read_result =
            match authoritative.compression {
                EntryCompression::Stored => {
                    let reader = std::io::Cursor::new(local.data());
                    read_verified(&local, reader, &mut output, budget, accounting, &mut hook)
                        .and_then(|reader| {
                            if reader.position() == local.data().len() as u64 {
                                Ok(())
                            } else {
                                Err(std::io::Error::other(
                                    "stored entry has trailing compressed bytes",
                                ))
                            }
                        })
                }
                EntryCompression::Deflated => {
                    let decoder = flate2::bufread::DeflateDecoder::new(local.data());
                    read_verified(&local, decoder, &mut output, budget, accounting, &mut hook)
                        .and_then(|decoder| {
                            if decoder.total_in() == local.data().len() as u64 {
                                Ok(())
                            } else {
                                Err(std::io::Error::other(
                                    "deflate stream has trailing compressed bytes",
                                ))
                            }
                        })
                }
                EntryCompression::Unsupported => unreachable!("unsupported method rejected above"),
            };
        if let Err(error) = read_result {
            output.clear();
            return Err(map_verification_error(error));
        }
        let actual = as_u64(output.len())?;
        if actual != authoritative.uncompressed_size {
            return Err(Error::invalid_input(
                "uncompressed_size_mismatch",
                format!(
                    "central size {} differs from actual output {actual}",
                    authoritative.uncompressed_size
                ),
            ));
        }
        let content_digest = crate::model::Digest(blake3::hash(&output).to_hex().to_string());
        Ok(MaterializedEntry {
            entry: entry.id.clone(),
            bytes: output,
            content_digest,
            usage: budget.usage(),
        })
    }

    fn locate_entry_for_replay(
        &self,
        origin: &ContainerOrigin,
        ordinal: u64,
        raw_name: &ArchiveNameBytes,
        budget: &mut Budget,
    ) -> Result<PhysicalEntry> {
        budget.poll()?;
        let archive = ZipArchive::from_slice(&self.bytes).map_err(zip_invalid("zip_open"))?;
        let directory_offset = archive.directory_offset();
        let mut iterator = archive.entries();
        let mut current = 0_u64;
        loop {
            budget.poll()?;
            let header = iterator
                .next_entry()
                .map_err(zip_invalid("central_directory"))?
                .ok_or_else(|| {
                    Error::invalid_input("entry_not_found", "entry ordinal is absent")
                })?;
            budget.charge(CountedBudgetDimension::ArchiveEntries, 1)?;
            if current != ordinal {
                current = current.checked_add(1).ok_or_else(|| {
                    Error::invalid_input("entry_count_overflow", "central entry ordinal overflow")
                })?;
                continue;
            }
            if header.file_path().as_bytes() != raw_name.0 {
                return Err(Error::invalid_input(
                    "entry_locator_mismatch",
                    "entry ordinal does not match the requested raw name",
                ));
            }
            budget.poll()?;
            let local = archive
                .get_entry(header.wayfinder())
                .map_err(zip_invalid("local_entry"))?;
            validate_headers(&header, &local)?;
            let (data_start, data_end) = local.compressed_data_range();
            let range_start = header.local_header_offset();
            if range_start > data_start || data_start > data_end || data_end > directory_offset {
                return Err(Error::invalid_input(
                    "invalid_entry_span",
                    format!(
                        "entry {ordinal} range {range_start}..{data_end} is invalid for the file area ending at {directory_offset}"
                    ),
                ));
            }
            budget.poll()?;
            if local
                .data_descriptor()
                .map_err(zip_invalid("data_descriptor"))?
                .is_some_and(|descriptor| {
                    descriptor.crc32() != header.crc32()
                        || descriptor.compressed_size() != header.compressed_size_hint()
                        || descriptor.uncompressed_size() != header.uncompressed_size_hint()
                })
            {
                return Err(Error::invalid_input(
                    "descriptor_central_mismatch",
                    format!("entry {ordinal} data descriptor conflicts with central directory"),
                ));
            }
            let flags = header.flags();
            let method = header.compression_method().as_u16();
            return Ok(PhysicalEntry {
                id: PhysicalEntryId {
                    origin: origin.clone(),
                    ordinal,
                    raw_name: raw_name.clone(),
                },
                compression: compression(method),
                compression_method: method,
                flags: EntryFlags {
                    raw_bits: flags.bits(),
                    encrypted: flags.is_encrypted(),
                    strong_encryption: flags.has_strong_encryption(),
                    data_descriptor: flags.has_data_descriptor(),
                },
                crc32: header.crc32(),
                compressed_size: header.compressed_size_hint(),
                uncompressed_size: header.uncompressed_size_hint(),
                layout: EntryLayout {
                    local_header_offset: range_start,
                    central_header_offset: header.central_directory_offset(),
                    compressed_data: ByteSpan::new(data_start, data_end - data_start),
                },
                nested_archive: nested_state(header.file_path().as_bytes()),
                signature_metadata: signature_metadata(header.file_path().as_bytes()),
            });
        }
    }

    fn read_nested_entry(
        &self,
        entry: &PhysicalEntry,
        budget: &mut Budget,
    ) -> Result<MaterializedEntry> {
        self.read_nested_entry_with_accounting(
            entry,
            budget,
            MaterializationAccounting::CallerOutput,
        )
    }

    fn read_nested_entry_with_accounting(
        &self,
        entry: &PhysicalEntry,
        budget: &mut Budget,
        final_accounting: MaterializationAccounting,
    ) -> Result<MaterializedEntry> {
        if self.kind != ArtifactKind::Zip || entry.id.snapshot() != &self.id {
            return Err(Error::invalid_input(
                "entry_snapshot_mismatch",
                "entry does not belong to this ZIP snapshot",
            ));
        }
        let root = root_origin(&self.id);
        if entry.id.origin.root_container != root.root_container {
            return Err(Error::invalid_input(
                "entry_origin_mismatch",
                "nested entry root container does not match this snapshot",
            ));
        }
        let mut bytes = self.bytes.clone();
        let mut current_origin = root;
        for (index, step) in entry.id.origin.steps.iter().enumerate() {
            let depth = u64::try_from(index + 1).map_err(|_| {
                Error::invalid_input("nested_depth_overflow", "nested origin is too deep")
            })?;
            budget.check_nested_depth(depth)?;
            if step.child_container
                != derive_child_container(&current_origin, step.via_ordinal, &step.via_raw_name)
            {
                return Err(Error::invalid_input(
                    "child_container_mismatch",
                    "nested origin child container is not derived from its verified parent entry",
                ));
            }
            let temporary = ArtifactSnapshot {
                id: self.id.clone(),
                kind: ArtifactKind::Zip,
                bytes: bytes.clone(),
            };
            let parent = temporary.locate_entry_for_replay(
                &current_origin,
                step.via_ordinal,
                &step.via_raw_name,
                budget,
            )?;
            if parent.nested_archive != NestedArchiveState::CandidateNotScanned {
                return Err(Error::invalid_input(
                    "nested_parent_not_candidate",
                    "nested origin parent entry is not an archive candidate",
                ));
            }
            let mut local_parent = parent;
            local_parent.id.origin = root_origin(&self.id);
            let materialized = temporary.read_entry_with_hooks(
                &local_parent,
                budget,
                MaterializationAccounting::Intermediate,
                |_| {},
                |_| {},
            )?;
            bytes = Arc::from(materialized.bytes);
            ZipArchive::from_slice(&bytes).map_err(zip_invalid("nested_zip_open"))?;
            current_origin.steps.push(step.clone());
        }
        if current_origin != entry.id.origin {
            return Err(Error::invalid_input(
                "entry_origin_mismatch",
                "nested entry origin does not match the replayed container",
            ));
        }
        let temporary = ArtifactSnapshot {
            id: self.id.clone(),
            kind: ArtifactKind::Zip,
            bytes,
        };
        let authoritative = temporary.locate_entry_for_replay(
            &current_origin,
            entry.id.ordinal,
            &entry.id.raw_name,
            budget,
        )?;
        if &authoritative != entry {
            return Err(Error::invalid_input(
                "entry_metadata_mismatch",
                "caller-supplied nested entry metadata differs from the fixed snapshot",
            ));
        }
        let mut local_entry = authoritative.clone();
        local_entry.id.origin = root_origin(&self.id);
        let mut materialized = temporary.read_entry_with_hooks(
            &local_entry,
            budget,
            final_accounting,
            |_| {},
            |_| {},
        )?;
        materialized.entry = entry.id.clone();
        materialized.usage = budget.usage();
        Ok(materialized)
    }
}

fn root_origin(snapshot: &SnapshotId) -> ContainerOrigin {
    ContainerOrigin {
        snapshot: snapshot.clone(),
        root_container: ContainerId("root".into()),
        steps: Vec::new(),
    }
}

fn derive_child_container(
    parent: &ContainerOrigin,
    ordinal: u64,
    raw_name: &ArchiveNameBytes,
) -> ContainerId {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"jarde.child-container.v1\0");
    hash_length_prefixed(&mut hasher, parent.snapshot.0.as_bytes());
    hash_length_prefixed(&mut hasher, parent.root_container.0.as_bytes());
    hasher.update(&(parent.steps.len() as u64).to_le_bytes());
    for step in &parent.steps {
        hasher.update(&step.via_ordinal.to_le_bytes());
        hash_length_prefixed(&mut hasher, &step.via_raw_name.0);
        hash_length_prefixed(&mut hasher, step.child_container.0.as_bytes());
    }
    hasher.update(&ordinal.to_le_bytes());
    hash_length_prefixed(&mut hasher, &raw_name.0);
    ContainerId(hasher.finalize().to_hex().to_string())
}

fn hash_length_prefixed(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn prefix_layout(
    kind: LayoutNodeKind,
    origin: &ContainerOrigin,
    prefix: &[u8],
    entry: &PhysicalEntry,
) -> LayoutNode {
    LayoutNode {
        kind,
        source: LayoutNodeSource::Prefix {
            container: origin.clone(),
            prefix: ArchiveNameBytes(prefix.to_vec()),
            evidence_entry: entry.id.clone(),
        },
    }
}

fn is_direct_library(name: &[u8], prefix: &[u8]) -> bool {
    name.strip_prefix(prefix)
        .is_some_and(|tail| !tail.is_empty() && !tail.contains(&b'/') && tail.ends_with(b".jar"))
}

fn candidate_coverage_range(
    entry: &PhysicalEntry,
    origin: &ContainerOrigin,
) -> Result<CoverageRange> {
    Ok(CoverageRange {
        label: format!(
            "container:{}:nested_archive_candidates",
            origin.current_container().0
        ),
        start: entry.id.ordinal,
        end: entry.id.ordinal.checked_add(1).ok_or_else(|| {
            Error::invalid_input(
                "entry_ordinal_overflow",
                "nested archive candidate ordinal overflow",
            )
        })?,
    })
}

fn append_skipped_candidates(
    entries: &[PhysicalEntry],
    start: usize,
    origin: &ContainerOrigin,
    skipped: &mut Vec<CoverageRange>,
) -> Result<()> {
    for entry in &entries[start..] {
        if entry.nested_archive == NestedArchiveState::CandidateNotScanned {
            skipped.push(candidate_coverage_range(entry, origin)?);
        }
    }
    Ok(())
}

fn relabel_coverage(mut coverage: Coverage, origin: &ContainerOrigin) -> Coverage {
    let identity = origin.current_container().0.as_str();
    for range in coverage
        .artifact_structural
        .scanned
        .iter_mut()
        .chain(coverage.artifact_structural.skipped.iter_mut())
    {
        range.label = format!("container:{identity}:{}", range.label);
    }
    coverage
}

struct TreeStackItem {
    bytes: Arc<[u8]>,
    origin: ContainerOrigin,
    depth: u64,
    parent_entry: Option<PhysicalEntry>,
    expected_entries: u64,
}

fn tree_coverage(
    containers: &[ContainerReport],
    pending: &[TreeStackItem],
    mut scanned: Vec<CoverageRange>,
    mut skipped: Vec<CoverageRange>,
    complete: bool,
) -> Coverage {
    for container in containers {
        scanned.extend(container.coverage.artifact_structural.scanned.clone());
        skipped.extend(container.coverage.artifact_structural.skipped.clone());
    }
    for container in pending {
        skipped.push(CoverageRange {
            label: format!(
                "container:{}:central_directory_entries",
                container.origin.current_container().0
            ),
            start: 0,
            end: container.expected_entries,
        });
    }
    Coverage {
        artifact_structural: CoverageDimension {
            state: if complete {
                CoverageState::CompleteWithinSchema
            } else {
                CoverageState::Partial
            },
            scanned,
            skipped,
            uninterpreted_extensions: Vec::new(),
        },
        runtime_resolution: CoverageDimension::not_requested(),
        dynamic_analysis: CoverageDimension::not_requested(),
    }
}

fn record_tree_issue(
    error: Error,
    parent_entry: &PhysicalEntry,
    budget: &Budget,
    first_issue: &mut Option<ExecutionReport>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    merge_tree_error(first_issue, &error, budget);
    diagnostics.push(tree_diagnostic(&error, Some(parent_entry)));
}

fn merge_tree_error(first_issue: &mut Option<ExecutionReport>, error: &Error, budget: &Budget) {
    let execution = match error {
        Error::Cancelled { .. } => ExecutionReport::Cancelled {
            usage: budget.usage(),
        },
        Error::BudgetExceeded { dimension, .. } => ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: *dimension,
            },
            usage: budget.usage(),
        },
        Error::Unsupported { code, .. } => ExecutionReport::Partial {
            reason: TerminationReason::Unsupported { code: code.clone() },
            usage: budget.usage(),
        },
        Error::InvalidInput { code, .. }
        | Error::Io {
            operation: code, ..
        } => ExecutionReport::Partial {
            reason: TerminationReason::Error { code: code.clone() },
            usage: budget.usage(),
        },
    };
    merge_tree_execution(first_issue, execution);
}

fn nested_container_execution(execution: ExecutionReport) -> ExecutionReport {
    match execution {
        ExecutionReport::Failed { reason, usage } => ExecutionReport::Partial { reason, usage },
        other => other,
    }
}

fn merge_tree_execution(first_issue: &mut Option<ExecutionReport>, execution: ExecutionReport) {
    let incoming_priority = tree_issue_priority(&execution);
    let current_priority = first_issue.as_ref().map(tree_issue_priority).unwrap_or(0);
    if first_issue.is_none() || incoming_priority > current_priority {
        *first_issue = Some(execution);
    }
}

fn tree_issue_priority(execution: &ExecutionReport) -> u8 {
    match execution {
        ExecutionReport::Cancelled { .. } => 3,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded { dimension },
            ..
        }
        | ExecutionReport::Failed {
            reason: TerminationReason::BudgetExceeded { dimension },
            ..
        } if *dimension != BudgetDimension::NestedDepth => 2,
        ExecutionReport::Complete { .. } => 0,
        _ => 1,
    }
}

fn tree_must_stop(issue: &Option<ExecutionReport>) -> bool {
    match issue {
        Some(ExecutionReport::Cancelled { .. }) => true,
        Some(ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded { dimension },
            ..
        }) => *dimension != BudgetDimension::NestedDepth,
        _ => false,
    }
}

fn tree_parent_provenance(parent_entry: &PhysicalEntry) -> Option<Provenance> {
    Some(Provenance {
        location: Location::Entry {
            id: parent_entry.id.clone(),
            span: parent_entry.layout.compressed_data.clone(),
        },
    })
}

fn tree_diagnostic(error: &Error, parent_entry: Option<&PhysicalEntry>) -> Diagnostic {
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
        provenance: parent_entry.and_then(tree_parent_provenance),
    }
}

fn ensure_disjoint_range(
    ranges: &BTreeMap<u64, (u64, u64)>,
    ordinal: u64,
    start: u64,
    end: u64,
) -> Result<()> {
    if let Some((&other_start, &(other_end, other_ordinal))) = ranges.range(..=start).next_back()
        && start < other_end
    {
        return Err(Error::invalid_input(
            "overlapping_entry_spans",
            format!(
                "entry {ordinal} range {start}..{end} overlaps predecessor entry {other_ordinal} range {other_start}..{other_end}"
            ),
        ));
    }
    if let Some((&other_start, &(other_end, other_ordinal))) = ranges.range(start..).next()
        && other_start < end
    {
        return Err(Error::invalid_input(
            "overlapping_entry_spans",
            format!(
                "entry {ordinal} range {start}..{end} overlaps successor entry {other_ordinal} range {other_start}..{other_end}"
            ),
        ));
    }
    Ok(())
}

fn enumeration_coverage(
    state: CoverageState,
    completed: u64,
    expected: u64,
    known_end: Option<u64>,
) -> Coverage {
    let covered_end = known_end.unwrap_or(completed).max(expected).max(completed);
    Coverage {
        artifact_structural: CoverageDimension {
            state,
            scanned: vec![CoverageRange {
                label: "central_directory_entries".into(),
                start: 0,
                end: completed,
            }],
            skipped: (completed < covered_end)
                .then(|| CoverageRange {
                    label: "central_directory_entries".into(),
                    start: completed,
                    end: covered_end,
                })
                .into_iter()
                .collect(),
            uninterpreted_extensions: Vec::new(),
        },
        runtime_resolution: CoverageDimension::not_requested(),
        dynamic_analysis: CoverageDimension::not_requested(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EnumerationProgress {
    completed: u64,
    expected: u64,
    known_end: Option<u64>,
}

pub const fn budget_dimension_code(dimension: BudgetDimension) -> &'static str {
    match dimension {
        BudgetDimension::InputBytes => "input_bytes",
        BudgetDimension::ArchiveEntries => "archive_entries",
        BudgetDimension::EntryBytes => "entry_bytes",
        BudgetDimension::ReadBytes => "read_bytes",
        BudgetDimension::ClassBytes => "class_bytes",
        BudgetDimension::AttributeBytes => "attribute_bytes",
        BudgetDimension::CodeBytes => "code_bytes",
        BudgetDimension::ResultItems => "result_items",
        BudgetDimension::OutputBytes => "output_bytes",
        BudgetDimension::ClassHeaders => "class_headers",
        BudgetDimension::MethodBodies => "method_bodies",
        BudgetDimension::IrItems => "ir_items",
        BudgetDimension::IrEdges => "ir_edges",
        BudgetDimension::AnalysisSteps => "analysis_steps",
        BudgetDimension::NormalizationClones => "normalization_clones",
        BudgetDimension::NestedDepth => "nested_depth",
        BudgetDimension::DependencyDepth => "dependency_depth",
        BudgetDimension::ElapsedMillis => "elapsed_millis",
    }
}

fn terminated_enumeration(
    snapshot: &SnapshotId,
    entries: Vec<PhysicalEntry>,
    mut diagnostics: Vec<Diagnostic>,
    progress: EnumerationProgress,
    error: Error,
    budget: &Budget,
) -> EnumerationReport {
    let (execution, severity) = match &error {
        Error::Cancelled { .. } => (
            ExecutionReport::Cancelled {
                usage: budget.usage(),
            },
            DiagnosticSeverity::Warning,
        ),
        Error::BudgetExceeded { dimension, .. } => (
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: *dimension,
                },
                usage: budget.usage(),
            },
            DiagnosticSeverity::Warning,
        ),
        Error::InvalidInput { code, .. }
        | Error::Io {
            operation: code, ..
        } => (
            ExecutionReport::Failed {
                reason: TerminationReason::Error { code: code.clone() },
                usage: budget.usage(),
            },
            DiagnosticSeverity::Error,
        ),
        Error::Unsupported { code, .. } => (
            ExecutionReport::Failed {
                reason: TerminationReason::Unsupported { code: code.clone() },
                usage: budget.usage(),
            },
            DiagnosticSeverity::Error,
        ),
    };
    diagnostics.push(Diagnostic {
        code: match &error {
            Error::InvalidInput { code, .. } | Error::Unsupported { code, .. } => code.clone(),
            Error::BudgetExceeded { dimension, .. } => {
                format!("budget_exceeded_{}", budget_dimension_code(*dimension))
            }
            Error::Cancelled { .. } => "cancelled".into(),
            Error::Io { operation, .. } => operation.clone(),
        },
        severity,
        message: error.to_string(),
        provenance: None,
    });
    EnumerationReport {
        snapshot: snapshot.clone(),
        entries,
        coverage: enumeration_coverage(
            CoverageState::Partial,
            progress.completed,
            progress.expected,
            progress.known_end,
        ),
        execution,
        diagnostics,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MaterializationAccounting {
    Intermediate,
    CallerOutput,
}

fn read_verified<R: Read, F: FnMut(usize)>(
    entry: &rawzip::ZipSliceEntry<'_>,
    reader: R,
    output: &mut Vec<u8>,
    budget: &mut Budget,
    accounting: MaterializationAccounting,
    hook: &mut F,
) -> std::io::Result<R> {
    let reader = BudgetedEntryReader {
        reader,
        budget,
        accounting,
    };
    let mut verifier = entry.verifying_reader(reader);
    let mut chunk = [0_u8; READ_CHUNK];
    loop {
        let count = verifier.read(&mut chunk)?;
        if count == 0 {
            break;
        }
        output.extend_from_slice(&chunk[..count]);
        hook(output.len());
    }
    Ok(verifier.into_inner().reader)
}

struct BudgetedEntryReader<'a, R> {
    reader: R,
    budget: &'a mut Budget,
    accounting: MaterializationAccounting,
}

impl<R: Read> Read for BudgetedEntryReader<'_, R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.budget.poll().map_err(budget_io)?;
        let count = self.reader.read(buffer)?;
        let count =
            u64::try_from(count).map_err(|_| std::io::Error::other("chunk length overflow"))?;
        self.budget
            .charge(CountedBudgetDimension::EntryBytes, count)
            .map_err(budget_io)?;
        if self.accounting == MaterializationAccounting::CallerOutput {
            self.budget
                .charge(CountedBudgetDimension::OutputBytes, count)
                .map_err(budget_io)?;
        }
        usize::try_from(count).map_err(|_| std::io::Error::other("chunk length overflow"))
    }
}

fn budget_io(error: Error) -> std::io::Error {
    std::io::Error::other(BudgetReadError(error))
}

#[derive(Debug)]
struct BudgetReadError(Error);

impl std::fmt::Display for BudgetReadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for BudgetReadError {}

fn map_verification_error(error: std::io::Error) -> Error {
    if let Some(error) = error
        .get_ref()
        .and_then(|inner| inner.downcast_ref::<BudgetReadError>())
    {
        return error.0.clone();
    }
    Error::invalid_input("entry_integrity", error.to_string())
}

fn validate_headers(
    central: &rawzip::ZipFileHeaderRecord<'_>,
    local: &rawzip::ZipSliceEntry<'_>,
) -> Result<()> {
    let local_header = local.local_header();
    if central.file_path().as_bytes() != local_header.file_path().as_bytes() {
        return Err(Error::invalid_input(
            "central_local_name_mismatch",
            "central and local raw entry names differ",
        ));
    }
    if central.compression_method() != local_header.compression_method() {
        return Err(Error::invalid_input(
            "central_local_method_mismatch",
            "central and local compression methods differ",
        ));
    }
    if central.flags().bits() != local_header.flags().bits() {
        return Err(Error::invalid_input(
            "central_local_flags_mismatch",
            "central and local flags differ",
        ));
    }
    if !central.flags().has_data_descriptor()
        && (central.crc32() != local_header.crc32()
            || central.compressed_size_hint() != local_header.compressed_size_hint()
            || central.uncompressed_size_hint() != local_header.uncompressed_size_hint())
    {
        return Err(Error::invalid_input(
            "central_local_integrity_mismatch",
            "central and local CRC or size fields differ",
        ));
    }
    Ok(())
}

fn compression(method: u16) -> EntryCompression {
    match method {
        STORE => EntryCompression::Stored,
        DEFLATE => EntryCompression::Deflated,
        _ => EntryCompression::Unsupported,
    }
}

fn nested_state(name: &[u8]) -> NestedArchiveState {
    let lower = name.iter().map(u8::to_ascii_lowercase).collect::<Vec<_>>();
    if (lower.ends_with(b".jar") || lower.ends_with(b".war"))
        && lower.as_slice() != b"meta-inf/manifest.mf"
    {
        NestedArchiveState::CandidateNotScanned
    } else {
        NestedArchiveState::NotCandidate
    }
}

fn signature_metadata(name: &[u8]) -> Option<SignatureMetadata> {
    let upper = name.iter().map(u8::to_ascii_uppercase).collect::<Vec<_>>();
    let kind = if upper.as_slice() == b"META-INF/MANIFEST.MF" {
        SignatureMetadataKind::Manifest
    } else if upper.starts_with(b"META-INF/") && upper.ends_with(b".SF") {
        SignatureMetadataKind::SignatureFile
    } else if upper.starts_with(b"META-INF/")
        && (upper.ends_with(b".RSA") || upper.ends_with(b".DSA") || upper.ends_with(b".EC"))
    {
        SignatureMetadataKind::SignatureBlock
    } else {
        return None;
    };
    Some(SignatureMetadata {
        kind,
        verification: SignatureVerificationState::NotVerified,
    })
}

fn classify(bytes: &[u8]) -> Result<ArtifactKind> {
    if bytes.starts_with(CLASS_MAGIC) {
        return Ok(ArtifactKind::StandaloneClass);
    }
    if ZipArchive::from_slice(bytes).is_ok() {
        return Ok(ArtifactKind::Zip);
    }
    Err(Error::invalid_input(
        "unknown_artifact_kind",
        "input is neither a standalone CLASS nor a valid ZIP container",
    ))
}

fn as_u64(value: usize) -> Result<u64> {
    u64::try_from(value)
        .map_err(|_| Error::invalid_input("length_overflow", "byte length does not fit in u64"))
}

fn zip_invalid(operation: &'static str) -> impl FnOnce(rawzip::Error) -> Error {
    move |error| Error::invalid_input(operation, error.to_string())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceIdentity {
    len: u64,
    modified: Option<SystemTime>,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}

impl SourceIdentity {
    fn from_metadata(metadata: &Metadata) -> Self {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        Self {
            len: metadata.len(),
            modified: metadata.modified().ok(),
            #[cfg(unix)]
            device: metadata.dev(),
            #[cfg(unix)]
            inode: metadata.ino(),
        }
    }
}

trait SnapshotSource {
    fn before(&mut self) -> std::io::Result<SourceIdentity>;
    fn read_chunk(&mut self, buffer: &mut [u8]) -> std::io::Result<usize>;
    fn after(&mut self) -> std::io::Result<(SourceIdentity, SourceIdentity)>;
}

struct FileSnapshotSource {
    path: PathBuf,
    file: File,
}

impl SnapshotSource for FileSnapshotSource {
    fn before(&mut self) -> std::io::Result<SourceIdentity> {
        Ok(SourceIdentity::from_metadata(&self.file.metadata()?))
    }

    fn read_chunk(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buffer)
    }

    fn after(&mut self) -> std::io::Result<(SourceIdentity, SourceIdentity)> {
        Ok((
            SourceIdentity::from_metadata(&self.file.metadata()?),
            SourceIdentity::from_metadata(&std::fs::metadata(&self.path)?),
        ))
    }
}

fn read_stable_path(path: &Path, budget: &mut Budget) -> Result<Vec<u8>> {
    let mut file = File::open(path).map_err(|error| Error::Io {
        operation: "open_artifact".into(),
        message: error.to_string(),
    })?;
    file.seek(SeekFrom::Start(0)).map_err(|error| Error::Io {
        operation: "seek_artifact".into(),
        message: error.to_string(),
    })?;
    let mut source = FileSnapshotSource {
        path: path.to_path_buf(),
        file,
    };
    read_stable_source(&mut source, budget)
}

fn read_stable_source(source: &mut impl SnapshotSource, budget: &mut Budget) -> Result<Vec<u8>> {
    let before = source.before().map_err(|error| Error::Io {
        operation: "metadata_before_read".into(),
        message: error.to_string(),
    })?;
    budget.check(CountedBudgetDimension::InputBytes, before.len)?;
    let capacity = usize::try_from(before.len).map_err(|_| {
        Error::invalid_input(
            "input_too_large",
            "input length does not fit in memory address space",
        )
    })?;
    let mut bytes = Vec::with_capacity(capacity);
    let mut chunk = [0_u8; READ_CHUNK];
    loop {
        budget.poll()?;
        let count = source.read_chunk(&mut chunk).map_err(|error| Error::Io {
            operation: "read_artifact".into(),
            message: error.to_string(),
        })?;
        if count == 0 {
            break;
        }
        budget.charge(CountedBudgetDimension::InputBytes, as_u64(count)?)?;
        bytes.extend_from_slice(&chunk[..count]);
    }
    let (handle_after, path_after) = source.after().map_err(|error| Error::Io {
        operation: "metadata_after_read".into(),
        message: error.to_string(),
    })?;
    if before != handle_after || before != path_after || before.len != bytes.len() as u64 {
        return Err(Error::invalid_input(
            "source_changed_during_open",
            "detectable source identity or length changed while creating the snapshot",
        ));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::{BudgetDimension, Limits};
    use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
    use std::io::{Cursor, Write};
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_ID: AtomicU64 = AtomicU64::new(0);

    fn limits(value: u64) -> Limits {
        Limits {
            input_bytes: value,
            archive_entries: value,
            entry_bytes: value,
            read_bytes: value,
            class_bytes: value,
            attribute_bytes: value,
            code_bytes: value,
            result_items: value,
            output_bytes: value,
            nested_depth: value,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        }
    }

    fn budget() -> Budget {
        Budget::new(limits(10 * 1024 * 1024))
    }

    fn class_bytes(marker: u8) -> Vec<u8> {
        vec![0xca, 0xfe, 0xba, 0xbe, marker]
    }

    fn zip(entries: &[(&[u8], &[u8], u16)]) -> Vec<u8> {
        let mut output = Cursor::new(Vec::new());
        {
            let mut archive = ZipArchiveWriter::new(&mut output);
            for (name, data, method) in entries {
                let (mut entry, config) = archive
                    .new_file(EntryPath::verbatim(name.to_vec()))
                    .compression_method(CompressionMethod::new(*method))
                    .start()
                    .unwrap();
                if *method == DEFLATE {
                    let encoder = flate2::write::DeflateEncoder::new(
                        &mut entry,
                        flate2::Compression::default(),
                    );
                    let mut writer = config.wrap(encoder);
                    writer.write_all(data).unwrap();
                    let (encoder, descriptor) = writer.finish().unwrap();
                    encoder.finish().unwrap();
                    entry.finish(descriptor).unwrap();
                } else {
                    let mut writer = config.wrap(&mut entry);
                    writer.write_all(data).unwrap();
                    let (_, descriptor) = writer.finish().unwrap();
                    entry.finish(descriptor).unwrap();
                }
            }
            archive.finish().unwrap();
        }
        output.into_inner()
    }

    const CLASSIC_EOCD_SIGNATURE: u32 = 0x0605_4b50;
    const ZIP64_EOCD_SIGNATURE: u32 = 0x0606_4b50;
    const ZIP64_LOCATOR_SIGNATURE: u32 = 0x0706_4b50;
    const CLASSIC_EOCD_LEN: usize = 22;
    const ZIP64_EOCD_LEN: usize = 56;
    const ZIP64_LOCATOR_LEN: usize = 20;
    const CLASSIC_ENTRIES_ON_DISK: usize = 8;
    const CLASSIC_TOTAL_ENTRIES: usize = 10;
    const CLASSIC_CENTRAL_SIZE: usize = 12;
    const CLASSIC_CENTRAL_OFFSET: usize = 16;
    const CLASSIC_COMMENT_LEN: usize = 20;

    struct Zip64Fixture {
        bytes: Vec<u8>,
        zip64_eocd_offset: usize,
        locator_offset: usize,
        classic_eocd_offset: usize,
    }

    fn checked_offset(base: usize, relative: usize) -> usize {
        base.checked_add(relative)
            .expect("ZIP fixture offset must not overflow")
    }

    fn checked_range(offset: usize, len: usize) -> std::ops::Range<usize> {
        offset..checked_offset(offset, len)
    }

    fn read_array_checked<const N: usize>(bytes: &[u8], offset: usize) -> [u8; N] {
        bytes
            .get(checked_range(offset, N))
            .expect("ZIP fixture field must be in bounds")
            .try_into()
            .expect("ZIP fixture field width must match")
    }

    fn read_u16_checked(bytes: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes(read_array_checked(bytes, offset))
    }

    fn read_u32_checked(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(read_array_checked(bytes, offset))
    }

    fn write_checked(bytes: &mut [u8], offset: usize, value: &[u8]) {
        bytes
            .get_mut(checked_range(offset, value.len()))
            .expect("ZIP fixture field must be in bounds")
            .copy_from_slice(value);
    }

    fn small_zip64_fixture(name: &[u8], payload: &[u8]) -> Zip64Fixture {
        let mut bytes = zip(&[(name, payload, STORE)]);
        let classic_eocd_offset = bytes
            .len()
            .checked_sub(CLASSIC_EOCD_LEN)
            .expect("standard ZIP must contain a classic EOCD");
        assert_eq!(
            checked_offset(classic_eocd_offset, CLASSIC_EOCD_LEN),
            bytes.len()
        );
        assert_eq!(
            read_u32_checked(&bytes, classic_eocd_offset),
            CLASSIC_EOCD_SIGNATURE
        );
        assert_eq!(
            read_u16_checked(
                &bytes,
                checked_offset(classic_eocd_offset, CLASSIC_COMMENT_LEN)
            ),
            0,
            "the standard helper must emit an uncommented EOCD at the archive end"
        );
        assert_eq!(
            read_u16_checked(
                &bytes,
                checked_offset(classic_eocd_offset, CLASSIC_TOTAL_ENTRIES)
            ),
            1
        );

        let central_size = read_u32_checked(
            &bytes,
            checked_offset(classic_eocd_offset, CLASSIC_CENTRAL_SIZE),
        );
        let central_offset = read_u32_checked(
            &bytes,
            checked_offset(classic_eocd_offset, CLASSIC_CENTRAL_OFFSET),
        );
        let central_end = usize::try_from(central_offset)
            .expect("central offset must fit usize")
            .checked_add(usize::try_from(central_size).expect("central size must fit usize"))
            .expect("central directory span must not overflow");
        assert_eq!(central_end, classic_eocd_offset);

        let mut classic_eocd = bytes
            .get(checked_range(classic_eocd_offset, CLASSIC_EOCD_LEN))
            .expect("classic EOCD must be in bounds")
            .to_vec();
        bytes.truncate(classic_eocd_offset);

        let zip64_eocd_offset = bytes.len();
        let zip64_eocd_offset_u64 =
            u64::try_from(zip64_eocd_offset).expect("ZIP64 EOCD offset must fit u64");
        bytes.extend_from_slice(&ZIP64_EOCD_SIGNATURE.to_le_bytes());
        bytes.extend_from_slice(&44_u64.to_le_bytes());
        bytes.extend_from_slice(&45_u16.to_le_bytes());
        bytes.extend_from_slice(&45_u16.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u64.to_le_bytes());
        bytes.extend_from_slice(&1_u64.to_le_bytes());
        bytes.extend_from_slice(&u64::from(central_size).to_le_bytes());
        bytes.extend_from_slice(&u64::from(central_offset).to_le_bytes());

        let locator_offset = bytes.len();
        assert_eq!(
            checked_offset(zip64_eocd_offset, ZIP64_EOCD_LEN),
            locator_offset
        );
        bytes.extend_from_slice(&ZIP64_LOCATOR_SIGNATURE.to_le_bytes());
        bytes.extend_from_slice(&0_u32.to_le_bytes());
        bytes.extend_from_slice(&zip64_eocd_offset_u64.to_le_bytes());
        bytes.extend_from_slice(&1_u32.to_le_bytes());

        let classic_eocd_offset = bytes.len();
        assert_eq!(
            checked_offset(locator_offset, ZIP64_LOCATOR_LEN),
            classic_eocd_offset
        );
        write_checked(
            &mut classic_eocd,
            CLASSIC_ENTRIES_ON_DISK,
            &u16::MAX.to_le_bytes(),
        );
        write_checked(
            &mut classic_eocd,
            CLASSIC_TOTAL_ENTRIES,
            &u16::MAX.to_le_bytes(),
        );
        write_checked(
            &mut classic_eocd,
            CLASSIC_CENTRAL_SIZE,
            &u32::MAX.to_le_bytes(),
        );
        write_checked(
            &mut classic_eocd,
            CLASSIC_CENTRAL_OFFSET,
            &u32::MAX.to_le_bytes(),
        );
        bytes.extend_from_slice(&classic_eocd);

        Zip64Fixture {
            bytes,
            zip64_eocd_offset,
            locator_offset,
            classic_eocd_offset,
        }
    }

    fn assert_zip64_usage(usage: &UsageSnapshot, fixture_len: u64, payload_len: u64) {
        assert_eq!(usage.input_bytes, fixture_len);
        assert_eq!(usage.archive_entries, 2);
        assert_eq!(usage.read_bytes, payload_len);
        assert_eq!(usage.entry_bytes, payload_len);
        assert_eq!(usage.output_bytes, payload_len);
    }

    fn open_snapshot(bytes: Vec<u8>) -> (ArtifactSnapshot, Budget) {
        let mut budget = budget();
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes), &mut budget).unwrap();
        (snapshot, budget)
    }

    fn central_offsets(bytes: &[u8]) -> Vec<usize> {
        let archive = ZipArchive::from_slice(bytes).unwrap();
        let mut offsets = Vec::new();
        let mut entries = archive.entries();
        while let Some(entry) = entries.next_entry().unwrap() {
            offsets.push(usize::try_from(entry.central_directory_offset()).unwrap());
        }
        offsets
    }

    fn local_offsets(bytes: &[u8]) -> Vec<usize> {
        let archive = ZipArchive::from_slice(bytes).unwrap();
        let mut offsets = Vec::new();
        let mut entries = archive.entries();
        while let Some(entry) = entries.next_entry().unwrap() {
            offsets.push(usize::try_from(entry.local_header_offset()).unwrap());
        }
        offsets
    }

    fn get_u16(bytes: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
    }

    fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn standalone_bytes_and_path_are_fixed_snapshots() {
        let original = class_bytes(1);
        let replacement = class_bytes(2);
        let (bytes_snapshot, mut bytes_budget) = open_snapshot(original.clone());
        assert_eq!(bytes_snapshot.kind(), ArtifactKind::StandaloneClass);
        assert_eq!(
            bytes_snapshot.root_bytes(&mut bytes_budget).unwrap(),
            original
        );

        let path = std::env::temp_dir().join(format!(
            "jarde-snapshot-{}-{}",
            std::process::id(),
            TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, &original).unwrap();
        let mut path_budget = budget();
        let path_snapshot =
            ArtifactSnapshot::open(ArtifactInput::Path(path.clone()), &mut path_budget).unwrap();
        std::fs::write(&path, replacement).unwrap();
        assert_eq!(
            path_snapshot.root_bytes(&mut path_budget).unwrap(),
            original
        );
        assert_eq!(path_snapshot.id(), bytes_snapshot.id());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn empty_and_minimal_jar_war_enumeration_are_complete() {
        for archive_bytes in [zip(&[]), zip(&[(b"WEB-INF/classes/A.class", b"a", STORE)])] {
            let (snapshot, mut budget) = open_snapshot(archive_bytes);
            let report = snapshot.enumerate(&mut budget).unwrap();
            assert_eq!(&report.snapshot, snapshot.id());
            assert_eq!(
                report.coverage.artifact_structural.state,
                CoverageState::CompleteWithinSchema
            );
            assert_eq!(
                report.coverage.runtime_resolution.state,
                CoverageState::NotRequested
            );
            assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        }
    }

    #[test]
    fn small_zip64_fixture_round_trips_from_bytes_and_jar_path() {
        let entry_name = b"classes/Zip64.class";
        let payload = b"zip64-stored-payload";
        let fixture = small_zip64_fixture(entry_name, payload);
        let fixture_len =
            u64::try_from(fixture.bytes.len()).expect("small fixture length must fit u64");
        let payload_len = u64::try_from(payload.len()).expect("payload length must fit u64");
        let zip64_eocd_offset =
            u64::try_from(fixture.zip64_eocd_offset).expect("ZIP64 EOCD offset must fit u64");

        assert!(fixture.bytes.len() < 1024, "ZIP64 fixture must stay small");
        assert_eq!(
            checked_offset(fixture.zip64_eocd_offset, ZIP64_EOCD_LEN),
            fixture.locator_offset
        );
        assert_eq!(
            checked_offset(fixture.locator_offset, ZIP64_LOCATOR_LEN),
            fixture.classic_eocd_offset
        );
        assert_eq!(
            read_u32_checked(&fixture.bytes, fixture.zip64_eocd_offset),
            ZIP64_EOCD_SIGNATURE
        );
        assert_eq!(
            read_u32_checked(&fixture.bytes, fixture.locator_offset),
            ZIP64_LOCATOR_SIGNATURE
        );
        assert_eq!(
            read_u32_checked(&fixture.bytes, fixture.classic_eocd_offset),
            CLASSIC_EOCD_SIGNATURE
        );
        assert_eq!(
            read_u16_checked(
                &fixture.bytes,
                checked_offset(fixture.classic_eocd_offset, CLASSIC_ENTRIES_ON_DISK)
            ),
            u16::MAX
        );
        assert_eq!(
            read_u16_checked(
                &fixture.bytes,
                checked_offset(fixture.classic_eocd_offset, CLASSIC_TOTAL_ENTRIES)
            ),
            u16::MAX
        );
        assert_eq!(
            read_u32_checked(
                &fixture.bytes,
                checked_offset(fixture.classic_eocd_offset, CLASSIC_CENTRAL_SIZE)
            ),
            u32::MAX
        );
        assert_eq!(
            read_u32_checked(
                &fixture.bytes,
                checked_offset(fixture.classic_eocd_offset, CLASSIC_CENTRAL_OFFSET)
            ),
            u32::MAX
        );

        let (entries_hint, directory_offset) = {
            let raw_archive = ZipArchive::from_slice(&fixture.bytes).unwrap();
            assert_eq!(
                raw_archive.eocd_offset(),
                u64::try_from(fixture.classic_eocd_offset)
                    .expect("classic EOCD offset must fit u64")
            );
            (raw_archive.entries_hint(), raw_archive.directory_offset())
        };
        assert_eq!(entries_hint, 1);
        println!(
            "small ZIP64 fixture: {} bytes; rawzip entries_hint={entries_hint}",
            fixture.bytes.len()
        );

        let (snapshot, mut bytes_budget) = open_snapshot(fixture.bytes.clone());
        assert_eq!(snapshot.kind(), ArtifactKind::Zip);
        let report = snapshot.enumerate(&mut bytes_budget).unwrap();
        assert_eq!(&report.snapshot, snapshot.id());
        assert_eq!(
            report.coverage.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );
        let enumeration_usage = match &report.execution {
            ExecutionReport::Complete { usage } => usage,
            other => panic!("ZIP64 enumeration must complete, got {other:?}"),
        };
        assert_eq!(enumeration_usage.input_bytes, fixture_len);
        assert_eq!(enumeration_usage.archive_entries, 1);
        assert!(report.diagnostics.is_empty());
        assert_eq!(report.entries.len(), 1);

        let entry = &report.entries[0];
        assert_eq!(entry.id.snapshot(), snapshot.id());
        assert_eq!(entry.id.ordinal, 0);
        assert_eq!(entry.id.raw_name.0.as_slice(), entry_name);
        assert_eq!(entry.compression, EntryCompression::Stored);
        assert_eq!(entry.compressed_size, payload_len);
        assert_eq!(entry.uncompressed_size, payload_len);
        assert!(entry.layout.local_header_offset < entry.layout.compressed_data.start);
        assert_eq!(entry.layout.central_header_offset, directory_offset);
        let data_end = entry
            .layout
            .compressed_data
            .start
            .checked_add(entry.layout.compressed_data.length)
            .expect("compressed data span must not overflow");
        assert!(data_end <= entry.layout.central_header_offset);
        assert!(entry.layout.central_header_offset < zip64_eocd_offset);
        let data_start = usize::try_from(entry.layout.compressed_data.start)
            .expect("compressed data offset must fit usize");
        let data_len = usize::try_from(entry.layout.compressed_data.length)
            .expect("compressed data length must fit usize");
        assert_eq!(
            fixture.bytes.get(checked_range(data_start, data_len)),
            Some(payload.as_slice())
        );

        let materialized = snapshot.read_entry(entry, &mut bytes_budget).unwrap();
        assert_eq!(materialized.entry, entry.id);
        assert_eq!(materialized.bytes.as_slice(), payload);
        assert_eq!(
            materialized.content_digest,
            crate::model::Digest(blake3::hash(payload).to_hex().to_string())
        );
        assert_zip64_usage(&materialized.usage, fixture_len, payload_len);

        let path = std::env::temp_dir().join(format!(
            "jarde-zip64-{}-{}.jar",
            std::process::id(),
            TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, &fixture.bytes).unwrap();
        let mut path_budget = budget();
        let path_snapshot =
            ArtifactSnapshot::open(ArtifactInput::Path(path.clone()), &mut path_budget).unwrap();
        std::fs::remove_file(&path).unwrap();

        assert_eq!(path_snapshot.kind(), ArtifactKind::Zip);
        assert_eq!(path_snapshot.id(), snapshot.id());
        let path_report = path_snapshot.enumerate(&mut path_budget).unwrap();
        assert!(matches!(
            &path_report.execution,
            ExecutionReport::Complete { .. }
        ));
        assert_eq!(&path_report.snapshot, path_snapshot.id());
        assert_eq!(path_report.entries.as_slice(), report.entries.as_slice());
        let path_materialized = path_snapshot
            .read_entry(&path_report.entries[0], &mut path_budget)
            .unwrap();
        assert_eq!(path_materialized.entry, materialized.entry);
        assert_eq!(path_materialized.bytes.as_slice(), payload);
        assert_eq!(
            path_materialized.content_digest,
            materialized.content_digest
        );
        assert_zip64_usage(&path_materialized.usage, fixture_len, payload_len);
    }

    #[test]
    fn duplicate_non_utf8_and_nul_names_are_lossless_and_ordered() {
        let name = b"bad/\xff\0.class";
        let (snapshot, mut budget) =
            open_snapshot(zip(&[(name, b"1", STORE), (name, b"2", STORE)]));
        let report = snapshot.enumerate(&mut budget).unwrap();
        assert_eq!(report.entries.len(), 2);
        assert_eq!(report.entries[0].id.raw_name.0, name);
        assert_eq!(report.entries[1].id.raw_name.0, name);
        assert_ne!(report.entries[0].id.ordinal, report.entries[1].id.ordinal);
        assert_eq!(report.diagnostics[0].code, "duplicate_raw_name");
    }

    #[test]
    fn physical_view_keeps_all_mr_variants_nested_candidates_and_unverified_signatures() {
        let archive = zip(&[
            (b"A.class", b"root", STORE),
            (b"META-INF/versions/11/A.class", b"11", STORE),
            (b"META-INF/versions/17/A.class", b"17", STORE),
            (b"WEB-INF/lib/library.jar", b"nested", STORE),
            (b"META-INF/MANIFEST.MF", b"manifest", STORE),
            (b"META-INF/TEST.SF", b"sf", STORE),
            (b"META-INF/TEST.RSA", b"rsa", STORE),
        ]);
        let (snapshot, mut budget) = open_snapshot(archive);
        let report = snapshot.enumerate(&mut budget).unwrap();
        assert_eq!(report.entries.len(), 7);
        assert_eq!(
            report.entries[3].nested_archive,
            NestedArchiveState::CandidateNotScanned
        );
        for entry in &report.entries[4..] {
            assert_eq!(
                entry.signature_metadata.as_ref().unwrap().verification,
                SignatureVerificationState::NotVerified
            );
        }
    }

    #[test]
    fn stored_and_deflated_reads_are_verified_and_accounted() {
        let archive = zip(&[
            (b"stored", b"stored-data", STORE),
            (b"deflated", b"deflated-data", DEFLATE),
        ]);
        let (snapshot, mut budget) = open_snapshot(archive);
        let report = snapshot.enumerate(&mut budget).unwrap();
        let before = budget.usage();
        let stored = snapshot
            .read_entry(&report.entries[0], &mut budget)
            .unwrap();
        let deflated = snapshot
            .read_entry(&report.entries[1], &mut budget)
            .unwrap();
        assert_eq!(stored.bytes, b"stored-data");
        assert_eq!(deflated.bytes, b"deflated-data");
        assert!(budget.usage().read_bytes > before.read_bytes);
        assert_eq!(budget.usage().entry_bytes, 24);
        assert_eq!(budget.usage().output_bytes, 24);
    }

    #[test]
    fn crc_and_size_mismatch_never_return_bytes() {
        let base = zip(&[(b"entry", b"payload", STORE)]);
        let archive = ZipArchive::from_slice(&base).unwrap();
        let header = archive.entries().next_entry().unwrap().unwrap();
        let local = archive.get_entry(header.wayfinder()).unwrap();
        let (data_start, _) = local.compressed_data_range();
        let mut crc_bad = base.clone();
        crc_bad[usize::try_from(data_start).unwrap()] ^= 1;
        let (snapshot, mut budget) = open_snapshot(crc_bad);
        let report = snapshot.enumerate(&mut budget).unwrap();
        assert!(
            matches!(snapshot.read_entry(&report.entries[0], &mut budget), Err(Error::InvalidInput { ref code, .. }) if code == "entry_integrity")
        );

        let mut size_bad = base;
        let central = central_offsets(&size_bad)[0];
        let local = local_offsets(&size_bad)[0];
        put_u32(&mut size_bad, central + 24, 8);
        put_u32(&mut size_bad, local + 22, 8);
        let (snapshot, mut budget) = open_snapshot(size_bad);
        let report = snapshot.enumerate(&mut budget).unwrap();
        assert!(matches!(
            report.execution,
            ExecutionReport::Failed {
                reason: TerminationReason::Error { ref code },
                ..
            } if code == "descriptor_central_mismatch" || code == "central_local_integrity_mismatch"
        ));
    }

    #[test]
    fn encrypted_and_unknown_compression_are_explicitly_unsupported() {
        let mut encrypted = zip(&[(b"entry", b"payload", STORE)]);
        let central = central_offsets(&encrypted)[0];
        let local = local_offsets(&encrypted)[0];
        let central_flags = get_u16(&encrypted, central + 8) | 1;
        let local_flags = get_u16(&encrypted, local + 6) | 1;
        put_u16(&mut encrypted, central + 8, central_flags);
        put_u16(&mut encrypted, local + 6, local_flags);
        let (snapshot, mut budget) = open_snapshot(encrypted);
        let report = snapshot.enumerate(&mut budget).unwrap();
        assert!(
            matches!(snapshot.read_entry(&report.entries[0], &mut budget), Err(Error::Unsupported { ref code, .. }) if code == "encrypted_zip_entry")
        );

        let mut unknown = zip(&[(b"entry", b"payload", STORE)]);
        let central = central_offsets(&unknown)[0];
        let local = local_offsets(&unknown)[0];
        put_u16(&mut unknown, central + 10, 99);
        put_u16(&mut unknown, local + 8, 99);
        let (snapshot, mut budget) = open_snapshot(unknown);
        let report = snapshot.enumerate(&mut budget).unwrap();
        assert!(
            matches!(snapshot.read_entry(&report.entries[0], &mut budget), Err(Error::Unsupported { ref code, .. }) if code == "zip_compression_method")
        );
    }

    #[test]
    fn caller_cannot_forge_entry_metadata_to_change_security_or_budget_semantics() {
        let archive = zip(&[(b"entry", b"payload", STORE)]);
        let (snapshot, mut setup_budget) = open_snapshot(archive);
        let report = snapshot.enumerate(&mut setup_budget).unwrap();
        let original = &report.entries[0];

        let mut variants = Vec::new();
        let mut origin = original.clone();
        origin.id.origin.root_container = ContainerId("forged-root".into());
        variants.push(origin);
        let mut flags = original.clone();
        flags.flags.encrypted = true;
        variants.push(flags);
        let mut method = original.clone();
        method.compression = EntryCompression::Deflated;
        method.compression_method = DEFLATE;
        variants.push(method);
        let mut sizes = original.clone();
        sizes.uncompressed_size = 0;
        sizes.compressed_size = 0;
        variants.push(sizes);
        let mut layout = original.clone();
        layout.layout.compressed_data.start += 1;
        variants.push(layout);

        for forged in variants {
            let mut read_budget = budget();
            assert!(
                matches!(snapshot.read_entry(&forged, &mut read_budget), Err(Error::InvalidInput { ref code, .. }) if code == "entry_metadata_mismatch")
            );
            assert_eq!(read_budget.usage().read_bytes, 0);
            assert_eq!(read_budget.usage().entry_bytes, 0);
        }
    }

    #[test]
    fn malformed_local_span_and_header_conflict_fail_without_panicking() {
        let mut span_bad = zip(&[(b"entry", b"payload", STORE)]);
        let central = central_offsets(&span_bad)[0];
        put_u32(&mut span_bad, central + 42, u32::MAX - 4);
        let (snapshot, mut budget) = open_snapshot(span_bad);
        assert!(matches!(
            snapshot.enumerate(&mut budget).unwrap().execution,
            ExecutionReport::Failed { .. }
        ));

        let mut method_bad = zip(&[(b"entry", b"payload", STORE)]);
        let central = central_offsets(&method_bad)[0];
        put_u16(&mut method_bad, central + 10, DEFLATE);
        let (snapshot, mut budget) = open_snapshot(method_bad);
        assert!(matches!(
            snapshot.enumerate(&mut budget).unwrap().execution,
            ExecutionReport::Failed {
                reason: TerminationReason::Error { ref code },
                ..
            } if code == "central_local_method_mismatch"
        ));
    }

    #[test]
    fn named_budgets_cover_declared_actual_read_and_entry_count() {
        let archive = zip(&[(b"entry", &vec![7; READ_CHUNK + 1], DEFLATE)]);
        let mut open_budget = budget();
        let snapshot =
            ArtifactSnapshot::open(ArtifactInput::bytes(archive), &mut open_budget).unwrap();
        let report = snapshot.enumerate(&mut open_budget).unwrap();

        let mut declared = Budget::new(Limits {
            entry_bytes: 1,
            ..limits(10 * 1024 * 1024)
        });
        assert!(matches!(
            snapshot.read_entry(&report.entries[0], &mut declared),
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::EntryBytes,
                ..
            })
        ));

        let mut understated_archive = zip(&[(b"entry", &vec![7; READ_CHUNK + 1], DEFLATE)]);
        let central = central_offsets(&understated_archive)[0];
        let raw_archive = ZipArchive::from_slice(&understated_archive).unwrap();
        let header = raw_archive.entries().next_entry().unwrap().unwrap();
        let local_entry = raw_archive.get_entry(header.wayfinder()).unwrap();
        let (_, data_end) = local_entry.compressed_data_range();
        put_u32(&mut understated_archive, central + 24, 1);
        put_u32(
            &mut understated_archive,
            usize::try_from(data_end).unwrap() + 12,
            1,
        );
        let mut understated_open = budget();
        let understated = ArtifactSnapshot::open(
            ArtifactInput::bytes(understated_archive),
            &mut understated_open,
        )
        .unwrap();
        let understated_report = understated.enumerate(&mut understated_open).unwrap();
        let mut actual = Budget::new(Limits {
            entry_bytes: 100,
            ..limits(10 * 1024 * 1024)
        });
        let error = understated
            .read_entry(&understated_report.entries[0], &mut actual)
            .unwrap_err();
        assert!(
            matches!(
                error,
                Error::BudgetExceeded {
                    dimension: BudgetDimension::EntryBytes,
                    ..
                }
            ),
            "{error:?}"
        );

        let mut read = Budget::new(Limits {
            read_bytes: 0,
            ..limits(10 * 1024 * 1024)
        });
        assert!(matches!(
            snapshot.read_entry(&report.entries[0], &mut read),
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::ReadBytes,
                ..
            })
        ));

        let mut entries = Budget::new(Limits {
            archive_entries: 0,
            ..limits(10 * 1024 * 1024)
        });
        assert!(matches!(
            snapshot.enumerate(&mut entries).unwrap().execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::ArchiveEntries
                },
                ..
            }
        ));
    }

    #[test]
    fn cancellation_before_and_during_read_is_cooperative() {
        let archive = zip(&[(b"entry", &vec![3; READ_CHUNK * 3], STORE)]);
        let (snapshot, mut setup_budget) = open_snapshot(archive);
        let report = snapshot.enumerate(&mut setup_budget).unwrap();

        let mut before = budget();
        before.cancellation_token().cancel();
        assert!(matches!(
            snapshot.read_entry(&report.entries[0], &mut before),
            Err(Error::Cancelled { .. })
        ));

        let mut during = budget();
        let token = during.cancellation_token();
        let result = snapshot.read_entry_with_hook(&report.entries[0], &mut during, |count| {
            if count >= READ_CHUNK {
                token.cancel();
            }
        });
        assert!(matches!(result, Err(Error::Cancelled { .. })));
        assert_eq!(during.usage().entry_bytes, READ_CHUNK as u64);
    }

    #[test]
    fn eocd_count_mismatch_and_overlapping_spans_are_rejected() {
        let mut count_bad = zip(&[(b"entry", b"x", STORE)]);
        let eocd = count_bad.len() - 22;
        put_u16(&mut count_bad, eocd + 8, 2);
        put_u16(&mut count_bad, eocd + 10, 2);
        let (snapshot, mut budget) = open_snapshot(count_bad);
        assert!(matches!(
            snapshot.enumerate(&mut budget).unwrap().execution,
            ExecutionReport::Failed {
                reason: TerminationReason::Error { ref code },
                ..
            } if code == "entry_count_mismatch"
        ));

        let mut overlap = zip(&[(b"same", b"abc", STORE), (b"same", b"abc", STORE)]);
        let central = central_offsets(&overlap);
        let local = local_offsets(&overlap);
        put_u32(
            &mut overlap,
            central[1] + 42,
            u32::try_from(local[0]).unwrap(),
        );
        let (snapshot, mut budget) = open_snapshot(overlap);
        assert!(matches!(
            snapshot.enumerate(&mut budget).unwrap().execution,
            ExecutionReport::Failed {
                reason: TerminationReason::Error { ref code },
                ..
            } if code == "overlapping_entry_spans"
        ));
    }

    #[test]
    fn eocd_undercount_uses_observed_next_record_for_partial_coverage() {
        let mut archive = zip(&[
            (b"a", b"a", STORE),
            (b"b", b"b", STORE),
            (b"c", b"c", STORE),
        ]);
        let eocd = archive.len() - 22;
        put_u16(&mut archive, eocd + 8, 2);
        put_u16(&mut archive, eocd + 10, 2);
        let (snapshot, _) = open_snapshot(archive);
        let mut limited = Budget::new(Limits {
            archive_entries: 2,
            ..limits(10 * 1024 * 1024)
        });
        let report = snapshot.enumerate(&mut limited).unwrap();
        assert_eq!(report.entries.len(), 2);
        assert!(matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::ArchiveEntries
                },
                ..
            }
        ));
        assert_eq!(
            report.coverage.artifact_structural.scanned,
            vec![CoverageRange {
                label: "central_directory_entries".into(),
                start: 0,
                end: 2,
            }]
        );
        assert_eq!(
            report.coverage.artifact_structural.skipped,
            vec![CoverageRange {
                label: "central_directory_entries".into(),
                start: 2,
                end: 3,
            }]
        );

        let mut cancelled = budget();
        let token = cancelled.cancellation_token();
        let report = snapshot
            .enumerate_with_hook(&mut cancelled, |completed| {
                if completed == 2 {
                    token.cancel();
                }
            })
            .unwrap();
        assert_eq!(report.entries.len(), 2);
        assert!(matches!(
            report.execution,
            ExecutionReport::Cancelled { .. }
        ));
        assert!(report.coverage.artifact_structural.skipped.is_empty());

        let mut damaged = zip(&[
            (b"a", b"a", STORE),
            (b"b", b"b", STORE),
            (b"c", b"c", STORE),
        ]);
        let central = central_offsets(&damaged);
        put_u16(&mut damaged, central[2] + 10, DEFLATE);
        let eocd = damaged.len() - 22;
        put_u16(&mut damaged, eocd + 8, 2);
        put_u16(&mut damaged, eocd + 10, 2);
        let (snapshot, mut budget) = open_snapshot(damaged);
        let report = snapshot.enumerate(&mut budget).unwrap();
        assert_eq!(report.entries.len(), 2);
        assert!(matches!(
            report.execution,
            ExecutionReport::Failed {
                reason: TerminationReason::Error { ref code },
                ..
            } if code == "central_local_method_mismatch"
        ));
        assert_eq!(
            report.coverage.artifact_structural.skipped,
            vec![CoverageRange {
                label: "central_directory_entries".into(),
                start: 2,
                end: 3,
            }]
        );
    }

    #[test]
    fn detectable_source_change_seam_rejects_mixed_open_snapshot() {
        struct ChangingSource {
            bytes: Cursor<Vec<u8>>,
            before: SourceIdentity,
            after: SourceIdentity,
        }
        impl SnapshotSource for ChangingSource {
            fn before(&mut self) -> std::io::Result<SourceIdentity> {
                Ok(self.before.clone())
            }
            fn read_chunk(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
                self.bytes.read(buffer)
            }
            fn after(&mut self) -> std::io::Result<(SourceIdentity, SourceIdentity)> {
                Ok((self.after.clone(), self.after.clone()))
            }
        }
        let identity = SourceIdentity {
            len: 5,
            modified: None,
            #[cfg(unix)]
            device: 1,
            #[cfg(unix)]
            inode: 1,
        };
        let mut changed = identity.clone();
        changed.len = 6;
        let mut source = ChangingSource {
            bytes: Cursor::new(class_bytes(1)),
            before: identity,
            after: changed,
        };
        assert!(
            matches!(read_stable_source(&mut source, &mut budget()), Err(Error::InvalidInput { ref code, .. }) if code == "source_changed_during_open")
        );
    }

    #[test]
    fn rawzip_path_supports_descriptor_and_prefixed_archives_and_keeps_physical_ids() {
        let base = zip(&[
            (b"same.class", b"identical", DEFLATE),
            (b"same.class", b"identical", DEFLATE),
        ]);
        let mut prefixed = b"#!/bin/sh\n".to_vec();
        prefixed.extend_from_slice(&base);
        let archive = ZipArchive::from_slice(&base).unwrap();
        let mut entries = archive.entries();
        let first = entries.next_entry().unwrap().unwrap();
        assert!(first.flags().has_data_descriptor());

        let (snapshot, mut budget) = open_snapshot(base);
        let report = snapshot.enumerate(&mut budget).unwrap();
        let left = snapshot
            .read_entry(&report.entries[0], &mut budget)
            .unwrap();
        let right = snapshot
            .read_entry(&report.entries[1], &mut budget)
            .unwrap();
        assert_eq!(left.content_digest, right.content_digest);
        assert_ne!(left.entry, right.entry);

        // A raw prefix is accepted by rawzip and remains represented by absolute spans.
        let (prefixed_snapshot, mut prefixed_budget) = open_snapshot(prefixed);
        assert_eq!(
            prefixed_snapshot
                .enumerate(&mut prefixed_budget)
                .unwrap()
                .entries
                .len(),
            2
        );
    }

    #[test]
    fn enumeration_cancellation_returns_reliable_prefix() {
        let archive = zip(&[(b"a", b"a", STORE), (b"b", b"b", STORE)]);
        let (snapshot, _) = open_snapshot(archive);

        let mut before = budget();
        before.cancellation_token().cancel();
        let report = snapshot.enumerate(&mut before).unwrap();
        assert_eq!(&report.snapshot, snapshot.id());
        assert!(report.entries.is_empty());
        assert!(matches!(
            report.execution,
            ExecutionReport::Cancelled { .. }
        ));
        assert_eq!(
            report.coverage.artifact_structural.state,
            CoverageState::Partial
        );
        assert_eq!(report.coverage.artifact_structural.scanned[0].end, 0);
        assert_eq!(report.coverage.artifact_structural.skipped[0].end, 2);
        assert_eq!(
            report.diagnostics.last().unwrap().severity,
            DiagnosticSeverity::Warning
        );
        assert_eq!(
            serde_json::to_value(report.diagnostics.last().unwrap()).unwrap()["severity"],
            "warning"
        );

        let mut during = budget();
        let token = during.cancellation_token();
        let report = snapshot
            .enumerate_with_hook(&mut during, |completed| {
                if completed == 1 {
                    token.cancel();
                }
            })
            .unwrap();
        assert_eq!(&report.snapshot, snapshot.id());
        assert_eq!(report.entries.len(), 1);
        assert!(matches!(
            report.execution,
            ExecutionReport::Cancelled { .. }
        ));
        assert_eq!(report.coverage.artifact_structural.scanned[0].end, 1);
        assert_eq!(report.coverage.artifact_structural.skipped[0].start, 1);
    }

    #[test]
    fn enumeration_budget_exhaustion_preserves_prefix_and_dimension() {
        let archive = zip(&[(b"a", b"a", STORE), (b"b", b"b", STORE)]);
        let (snapshot, _) = open_snapshot(archive);
        for (dimension, diagnostic_code, configured) in [
            (
                BudgetDimension::ArchiveEntries,
                "budget_exceeded_archive_entries",
                Limits {
                    archive_entries: 1,
                    ..limits(10 * 1024 * 1024)
                },
            ),
            (
                BudgetDimension::ResultItems,
                "budget_exceeded_result_items",
                Limits {
                    result_items: 1,
                    ..limits(10 * 1024 * 1024)
                },
            ),
        ] {
            let mut limited = Budget::new(configured);
            let report = snapshot.enumerate(&mut limited).unwrap();
            assert_eq!(&report.snapshot, snapshot.id());
            assert_eq!(report.entries.len(), 1);
            assert!(matches!(
                report.execution,
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded { dimension: actual },
                    ..
                } if actual == dimension
            ));
            assert_eq!(
                report.coverage.artifact_structural.state,
                CoverageState::Partial
            );
            let diagnostic = report.diagnostics.last().unwrap();
            assert_eq!(diagnostic.code, diagnostic_code);
            assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
            let json = serde_json::to_value(diagnostic).unwrap();
            assert_eq!(json["code"], diagnostic_code);
            assert_eq!(json["severity"], "warning");
        }
    }

    #[test]
    fn later_structural_damage_preserves_prefix_and_fails() {
        let mut archive = zip(&[(b"a", b"a", STORE), (b"b", b"b", STORE)]);
        let central = central_offsets(&archive);
        put_u16(&mut archive, central[1] + 10, DEFLATE);
        let (snapshot, mut budget) = open_snapshot(archive);
        let report = snapshot.enumerate(&mut budget).unwrap();
        assert_eq!(&report.snapshot, snapshot.id());
        assert_eq!(report.entries.len(), 1);
        assert!(matches!(
            report.execution,
            ExecutionReport::Failed {
                reason: TerminationReason::Error { ref code },
                ..
            } if code == "central_local_method_mismatch"
        ));
        let diagnostic = report.diagnostics.last().unwrap();
        assert_eq!(diagnostic.code, "central_local_method_mismatch");
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(
            serde_json::to_value(diagnostic).unwrap()["severity"],
            "error"
        );
        assert_ne!(
            report.coverage.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );
    }

    #[test]
    fn locator_charges_records_and_can_be_cancelled_mid_search() {
        let archive = zip(&[
            (b"a", b"a", STORE),
            (b"b", b"b", STORE),
            (b"c", b"c", STORE),
        ]);
        let (snapshot, mut setup) = open_snapshot(archive);
        let report = snapshot.enumerate(&mut setup).unwrap();

        let mut limited = Budget::new(Limits {
            archive_entries: 2,
            ..limits(10 * 1024 * 1024)
        });
        assert!(matches!(
            snapshot.read_entry(&report.entries[2], &mut limited),
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::ArchiveEntries,
                consumed: 2,
                ..
            })
        ));
        assert_eq!(limited.usage().archive_entries, 2);

        let mut cancelled = budget();
        let token = cancelled.cancellation_token();
        let result = snapshot.read_entry_with_hooks(
            &report.entries[2],
            &mut cancelled,
            MaterializationAccounting::CallerOutput,
            |records| {
                if records == 1 {
                    token.cancel();
                }
            },
            |_| {},
        );
        assert!(matches!(result, Err(Error::Cancelled { .. })));
        assert_eq!(cancelled.usage().archive_entries, 1);
    }

    #[test]
    fn overlap_index_rejects_predecessor_successor_aliases_and_zero_length_entries() {
        let mut ranges = BTreeMap::new();
        ranges.insert(20, (30, 0));
        assert!(ensure_disjoint_range(&ranges, 1, 25, 35).is_err());
        assert!(ensure_disjoint_range(&ranges, 1, 10, 25).is_err());
        assert!(ensure_disjoint_range(&ranges, 1, 20, 30).is_err());

        let mut alias = zip(&[(b"same", b"", STORE), (b"same", b"", STORE)]);
        let central = central_offsets(&alias);
        let local = local_offsets(&alias);
        put_u32(
            &mut alias,
            central[1] + 42,
            u32::try_from(local[0]).unwrap(),
        );
        let (snapshot, mut budget) = open_snapshot(alias);
        let report = snapshot.enumerate(&mut budget).unwrap();
        assert!(matches!(
            report.execution,
            ExecutionReport::Failed {
                reason: TerminationReason::Error { ref code },
                ..
            } if code == "overlapping_entry_spans"
        ));
    }

    #[test]
    fn central_directory_order_may_differ_from_physical_offset_order() {
        let mut archive = zip(&[(b"aa", b"first", STORE), (b"bb", b"second", STORE)]);
        let central = central_offsets(&archive);
        let record_len = central[1] - central[0];
        let first = archive[central[0]..central[0] + record_len].to_vec();
        let second = archive[central[1]..central[1] + record_len].to_vec();
        archive[central[0]..central[0] + record_len].copy_from_slice(&second);
        archive[central[1]..central[1] + record_len].copy_from_slice(&first);

        let (snapshot, mut budget) = open_snapshot(archive);
        let report = snapshot.enumerate(&mut budget).unwrap();
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        assert_eq!(report.entries[0].id.raw_name.0, b"bb");
        assert_eq!(report.entries[1].id.raw_name.0, b"aa");
        assert!(
            report.entries[0].layout.local_header_offset
                > report.entries[1].layout.local_header_offset
        );
    }

    #[test]
    fn many_disjoint_entries_enumerate_without_order_assumptions() {
        let owned = (0..256)
            .map(|index| (format!("entry-{index}"), vec![index as u8]))
            .collect::<Vec<_>>();
        let borrowed = owned
            .iter()
            .map(|(name, data)| (name.as_bytes(), data.as_slice(), STORE))
            .collect::<Vec<_>>();
        let (snapshot, mut budget) = open_snapshot(zip(&borrowed));
        assert_eq!(snapshot.enumerate(&mut budget).unwrap().entries.len(), 256);
    }

    #[test]
    fn legal_zip_payload_cannot_forge_a_nested_origin_when_parent_is_not_candidate() {
        let inner = zip(&[(b"Inner.class", b"inner", STORE)]);
        let outer = zip(&[(b"payload.bin", &inner, STORE)]);
        let (snapshot, mut tree_budget) = open_snapshot(outer);
        let root_report = snapshot.enumerate(&mut tree_budget).unwrap();
        let parent = root_report.entries[0].clone();
        assert_eq!(parent.nested_archive, NestedArchiveState::NotCandidate);

        let (inner_snapshot, mut inner_budget) = open_snapshot(inner);
        let mut forged = inner_snapshot.enumerate(&mut inner_budget).unwrap().entries[0].clone();
        let root = root_origin(&snapshot.id);
        let child_container = derive_child_container(&root, parent.id.ordinal, &parent.id.raw_name);
        forged.id.origin = root;
        forged.id.origin.steps.push(ContainerOriginStep {
            via_ordinal: parent.id.ordinal,
            via_raw_name: parent.id.raw_name,
            child_container,
        });

        let error = snapshot.read_entry(&forged, &mut budget()).unwrap_err();
        assert!(matches!(
            error,
            Error::InvalidInput { ref code, .. } if code == "nested_parent_not_candidate"
        ));
    }

    #[test]
    fn artifact_tree_cancellation_after_a_candidate_keeps_root_prefix() {
        let inner = zip(&[(b"Inner.class", b"inner", STORE)]);
        let outer = zip(&[
            (b"first.jar", b"malformed", STORE),
            (b"second.jar", &inner, STORE),
        ]);
        let (snapshot, mut budget) = open_snapshot(outer);
        let token = budget.cancellation_token();
        let report = snapshot
            .enumerate_artifact_tree_with_hook(&mut budget, |completed| {
                if completed == 1 {
                    token.cancel();
                }
            })
            .unwrap();
        assert!(matches!(
            report.execution,
            ExecutionReport::Cancelled { .. }
        ));
        assert_eq!(report.containers.len(), 1);
        assert_eq!(report.containers[0].entries.len(), 2);
        assert_eq!(report.layout_nodes.len(), 1);
        assert!(report.coverage.artifact_structural.state == CoverageState::Partial);
        let label = format!(
            "container:{}:nested_archive_candidates",
            report.containers[0].origin.current_container().0
        );
        for ordinal in 0..2 {
            assert!(
                report
                    .coverage
                    .artifact_structural
                    .skipped
                    .iter()
                    .any(|range| range.label == label
                        && range.start == ordinal
                        && range.end == ordinal + 1)
            );
        }
        assert!(
            report
                .coverage
                .artifact_structural
                .scanned
                .iter()
                .all(|range| range.label != label)
        );
    }

    #[test]
    fn deflate_trailing_compressed_bytes_are_rejected_after_verification() {
        let payload = vec![b'x'; 32 * 1024];
        let mut output = Cursor::new(Vec::new());
        {
            let mut archive = ZipArchiveWriter::new(&mut output);
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(b"entry".to_vec()))
                .compression_method(CompressionMethod::new(DEFLATE))
                .start()
                .unwrap();
            let encoder =
                flate2::write::DeflateEncoder::new(&mut entry, flate2::Compression::none());
            let mut writer = config.wrap(encoder);
            writer.write_all(&payload).unwrap();
            let (encoder, descriptor) = writer.finish().unwrap();
            encoder.finish().unwrap();
            entry.finish(descriptor).unwrap();
            archive.finish().unwrap();
        }
        let mut archive = output.into_inner();
        let raw = ZipArchive::from_slice(&archive).unwrap();
        let header = raw.entries().next_entry().unwrap().unwrap();
        let local = raw.get_entry(header.wayfinder()).unwrap();
        let (start, end) = local.compressed_data_range();
        let mut compact = Vec::new();
        let mut encoder =
            flate2::write::DeflateEncoder::new(&mut compact, flate2::Compression::best());
        encoder.write_all(&payload).unwrap();
        encoder.finish().unwrap();
        let span = usize::try_from(end - start).unwrap();
        assert!(compact.len() < span);
        let start = usize::try_from(start).unwrap();
        archive[start..start + compact.len()].copy_from_slice(&compact);
        archive[start + compact.len()..start + span].fill(0xa5);

        let (snapshot, mut budget) = open_snapshot(archive);
        let report = snapshot.enumerate(&mut budget).unwrap();
        assert!(matches!(
            snapshot.read_entry(&report.entries[0], &mut budget),
            Err(Error::InvalidInput { ref code, .. }) if code == "entry_integrity"
        ));
    }
}
