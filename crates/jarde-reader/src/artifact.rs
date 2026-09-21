//! Immutable artifact snapshots and bounded physical ZIP access.
//!
//! `ReadBytes` counts compressed bytes selected from the fixed snapshot for an
//! entry read. `EntryBytes` counts logical bytes produced by decompression, and
//! `OutputBytes` independently accounts for the owned result buffer returned to
//! the caller. The latter two intentionally describe different resources.
//!
//! ## Two ways to read a container
//!
//! `enumerate` reads one container's whole directory and publishes it as the request's own
//! result; `enumerate_artifact_tree` walks every container the tree holds and reports each one,
//! damage included. Both keep their coverage.
//!
//! `container_candidates` and `container_record` are the **directed** access: they address one
//! container by its origin, walk only the ancestors that reach it, parse that container's own
//! complete central directory and look the requested raw name up in the multi-value locator built
//! from it. A container the caller did not name is never enumerated, so a local answer is never a
//! whole-tree claim — and a directory that stopped on the budget, on a cancellation or on damage
//! is a refusal, never "this name is missing".
//!
//! Both ways share one parser ([`parse_container_directory`]) and one local-header check
//! ([`verify_local_against_record`]), so a record is validated the same way whichever path read
//! it. A container's facts can be handed to the request's [`crate::facts_cache::FactsCache`] and
//! retained across requests; the direct path is the same code with no cache attached, and nothing
//! here requires one to exist.
//!
//! ## What a read is answered from, in order
//!
//! A container access asks the request's store first (a hit or nothing, as it always did), then the
//! products **some live handle is still holding** — a [`crate::scope_cursor::ScopeCursor`]'s current
//! container, a [`crate::prepared::PreparedClassRead`]'s, or any handle a consumer kept — and only
//! then reads: ancestors walked, directory parsed, all of it charged. That order is what makes a
//! store's admission decision one thing and a fact's lifetime another: a request that is already
//! consuming a container keeps consuming it after the store is cleared, filled up or never attached
//! at all, and a product nothing holds any more is not answered from the memory of it. See
//! [`HeldContainerFacts`] for what the record is, and [`ContainerFactsHandle`] for the handle a
//! consumer holds.
//!
//! ## The class-bytes ceiling
//!
//! [`ArtifactSnapshot::prepared_class_within`] and [`ArtifactSnapshot::prepared_root_class_within`]
//! admit one class read under a caller's ceiling. It is applied before anything of the class is
//! materialized — decided from the container directory's own record — and again while the bytes are
//! produced, so the produced bytes are bounded by the ceiling rather than by the entry's declared or
//! real size. [`ArtifactSnapshot::prepared_class`] is the same read with no ceiling.

use crate::budget::{Budget, BudgetDimension, CountedBudgetDimension, UsageSnapshot};
use crate::error::{Error, Result};
use crate::model::{
    ArchiveNameBytes, ByteSpan, ClassBytesId, ContainerId, ContainerOrigin, ContainerOriginStep,
    Coverage, CoverageDimension, CoverageRange, CoverageState, Diagnostic, DiagnosticSeverity,
    Digest, ExecutionReport, Location, PhysicalClassLocation, PhysicalEntryId, Provenance,
    SnapshotId, TerminationReason,
};
use crate::view::{PhysicalScope, PhysicalView};
use rawzip::{ZipArchive, ZipArchiveEntryWayfinder};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{File, Metadata};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
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
    /// The container products some **live handle** of this snapshot still holds, by the physical
    /// origin each was verified for.
    ///
    /// The index holds weak references and is consulted only after the request's facts store has
    /// answered nothing: it is how a read reaches facts the caller is already consuming — a
    /// [`crate::scope_cursor::ScopeCursor`]'s current container, a
    /// [`crate::prepared::PreparedClassRead`]'s container, or a product retained under another
    /// handle — instead of parsing the same directory again. It retains nothing, has no capacity and
    /// no counters, and a dead entry is dropped the next time its origin is asked for; see
    /// [`HeldContainerFacts`].
    held_facts: Arc<Mutex<HeldContainerFacts>>,
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
        Ok(Self {
            id,
            kind,
            bytes,
            held_facts: Arc::new(Mutex::new(HeldContainerFacts::default())),
        })
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
        let origin = root_origin(&self.id);
        let parsed = parse_container_directory(
            &self.bytes,
            &origin,
            DirectoryIntent::Publish,
            budget,
            &mut completed_hook,
        )?;
        Ok(EnumerationReport {
            snapshot: self.id.clone(),
            entries: parsed.entries,
            coverage: parsed.coverage,
            execution: parsed.execution,
            diagnostics: parsed.diagnostics,
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
                held_facts: Arc::new(Mutex::new(HeldContainerFacts::default())),
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
                let child_origin = derive_child_origin(&origin, &entry.id);
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
                                if let Some(cache) = budget.facts_cache() {
                                    cache.note_nested_materialization(bytes.len() as u64);
                                }
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
        // The local header is checked against the **record this snapshot's own central directory
        // produced**, never against the caller's report: the report is compared afterwards, and
        // the two checks answer different questions.
        verify_local_against_record(&authoritative, &local)?;
        if entry != &authoritative {
            return Err(Error::invalid_input(
                "entry_metadata_mismatch",
                "caller-supplied entry metadata differs from the fixed snapshot",
            ));
        }
        let mut materialized =
            materialize_verified(&authoritative, &local, budget, accounting, None, &mut hook)?;
        materialized.entry = entry.id.clone();
        Ok(materialized)
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

    // -------------------------------------------------------------------------------------------
    // Directed container access
    // -------------------------------------------------------------------------------------------

    /// The raw-name candidates of one container, reached along its own ancestor chain.
    ///
    /// This is the one entry point a lookup uses to ask "which physical entries of *this* declared
    /// position carry this raw name". It validates the origin — the snapshot, the root container
    /// and every derivation in the chain — walks the ancestors the container really has, parses
    /// the container's own complete central directory and looks the name up in the multi-value
    /// locator built from it. It never enumerates a container the caller did not name, so an
    /// unrelated sibling is neither read nor reported, and the candidates it returns are the
    /// directory's own records, ordered by central-directory ordinal and never merged by name.
    ///
    /// The shape is deliberately source-agnostic: a container origin plus the raw bytes of the
    /// name. A later change that addresses a container together with a prefix (an entry of a
    /// container selected by a path prefix rather than by the name being looked up) reaches this
    /// same entry point with the container's origin; the name it passes stays the requested raw
    /// name, and the prefix rule is applied to the candidates this returns.
    ///
    /// A lookup that cannot prove the container's directory complete is refused, never answered
    /// with the entries it managed to read: `Missing` and `Ambiguous` mean "this container holds
    /// no/these candidates for the name", and an incomplete directory cannot say that.
    pub fn container_candidates(
        &self,
        origin: &ContainerOrigin,
        raw_name: &[u8],
        budget: &mut Budget,
    ) -> Result<Vec<PhysicalEntry>> {
        let facts = self.container_facts(origin, budget)?;
        Ok(facts.candidates(raw_name))
    }

    /// The authoritative record one entry identity names, looked up at its own container.
    ///
    /// The identity carries its container, its ordinal and its raw name, so this asks that
    /// container's directory for the record at that ordinal and requires the record to carry that
    /// raw name. `Ok(None)` is "this snapshot has no such entry at that coordinate", which is the
    /// same verdict a whole-snapshot listing would give, reached without enumerating a container
    /// the caller did not name.
    pub fn container_record(
        &self,
        entry: &PhysicalEntryId,
        budget: &mut Budget,
    ) -> Result<Option<PhysicalEntry>> {
        let facts = self.container_facts(&entry.origin, budget)?;
        let Ok(position) = usize::try_from(entry.ordinal) else {
            return Ok(None);
        };
        Ok(facts
            .entries
            .get(position)
            .filter(|record| record.id.raw_name == entry.raw_name)
            .cloned())
    }

    // -------------------------------------------------------------------------------------------
    // Incremental physical traversal (bulk stream A)
    // -------------------------------------------------------------------------------------------

    /// Opens one physical scope for incremental traversal: one class candidate at a time, in the
    /// scope's own order, without building a package-wide class list first.
    ///
    /// The scope's shape is checked here — a standalone `CLASS` snapshot holds no containers, so an
    /// artifact-tree scope on one is an input error, and an artifact-tree scope has to name this
    /// snapshot's own root container — and the walk itself happens in
    /// [`crate::scope_cursor::ScopeCursor::next_class`], which is where the charging, the stop
    /// semantics and the cancellation checks live.
    pub fn scope_cursor(&self, scope: &PhysicalScope) -> Result<crate::scope_cursor::ScopeCursor> {
        crate::scope_cursor::ScopeCursor::new(self.clone(), scope)
    }

    // -------------------------------------------------------------------------------------------
    // Prepared class reads (bulk stream A)
    // -------------------------------------------------------------------------------------------

    /// The bytes of one container of this snapshot, without copying them out of the read that
    /// materialized them.
    ///
    /// The root container's backing is the snapshot's own bytes — the same `Arc` every other read
    /// of this snapshot shares — and a nested container's is the backing the verified read that
    /// materialized it produced, reached along the container's real ancestor chain (so a forged
    /// origin is refused at the step that does not derive, and no sibling entry is touched). What
    /// the caller gets is a strong reference: it keeps the bytes alive on its own terms, whatever
    /// happens to any cache in the meantime.
    ///
    /// This is not a second container reader: it is the same directed access every lookup uses, and
    /// it hands out the backing that access already holds rather than re-reading anything.
    pub fn container_backing(
        &self,
        origin: &ContainerOrigin,
        budget: &mut Budget,
    ) -> Result<Arc<[u8]>> {
        budget.poll()?;
        if origin.snapshot != self.id {
            return Err(Error::invalid_input(
                "entry_snapshot_mismatch",
                "container origin does not belong to this snapshot",
            ));
        }
        if origin.steps.is_empty() {
            if origin.root_container != root_origin(&self.id).root_container {
                return Err(Error::invalid_input(
                    "entry_origin_mismatch",
                    "container origin does not name this snapshot's root container",
                ));
            }
            // A snapshot's root container is the snapshot itself, whichever artifact kind it is:
            // one ZIP's own bytes, or one standalone CLASS file's.
            return Ok(self.bytes.clone());
        }
        let facts = self.container_facts(origin, budget)?;
        Ok(facts.backing().clone())
    }

    /// One class entry's verified read, for a class task that will decode many of its methods.
    ///
    /// The entry is located in its own container's **complete** central directory (a container whose
    /// directory could not be read completely is refused, never answered), the record there is the
    /// authority, and the entry's local header is checked against that record — raw name, local and
    /// central header offsets, CRC, compressed and uncompressed sizes, the data descriptor when it
    /// declares one, and the compressed span — before any byte is handed out. The entry's own bytes
    /// are then read through the same verifying reader every other read uses. A mismatch on any of
    /// those is the existing error code of that check, never a new one, and never a silent read.
    ///
    /// The read is charged once, as an intermediate read of that entry. A class task that decodes
    /// `N` methods therefore pays for one class read, not `N`.
    ///
    /// The container's facts are the ones some live handle already holds when there are any (the
    /// cursor that yielded this class, a read of the same container, or the request's store), so a
    /// scope walk reads each container's directory once however many classes it prepares out of it;
    /// see [`Self::container_facts`]. The returned read carries that product as an active handle
    /// ([`crate::prepared::PreparedClassRead::container_facts`]) for as long as its consumer keeps it.
    ///
    /// What this deliberately does **not** do is apply a loader, profile or version gate: a
    /// prepared read is bytes and the entry they came from, and the layers above keep applying the
    /// same environment and binding checks they apply to the same bytes read any other way.
    pub fn prepared_class(
        &self,
        entry: &PhysicalEntryId,
        budget: &mut Budget,
    ) -> Result<crate::prepared::PreparedClassRead> {
        self.prepared_class_within(entry, u64::MAX, budget)
    }

    /// The same read, admitted under a **class-bytes ceiling** before it is materialized.
    ///
    /// `max_class_bytes` is the largest class this read may produce. It is applied twice, and both
    /// applications matter:
    ///
    /// 1. **Before any byte of the class is read**, from the container directory's own record: the
    ///    entry's declared uncompressed size is compared with the ceiling, and a larger class is
    ///    refused under `class_bytes_ceiling` with nothing materialized, nothing decompressed and no
    ///    `EntryBytes`/`ReadBytes`/`OutputBytes` charged for it. This is the application a caller uses
    ///    to keep a whole operation inside a memory budget: the refusal costs the directory it was
    ///    decided from and no allocation proportional to the class.
    /// 2. **While the bytes are produced**, on every chunk the read produces, so a record that
    ///    understates the entry's real size cannot smuggle the class past the ceiling: the read stops
    ///    as soon as the produced length would cross it, instead of materializing the whole entry and
    ///    comparing afterwards.
    ///
    /// The ceiling is a read admission and nothing else: it is not a class-file validation, not a
    /// verdict about the bytes and not a substitute for the budget's own dimensions, which keep
    /// applying exactly as they did. A ceiling of [`u64::MAX`] admits everything, which is what
    /// [`Self::prepared_class`] states.
    pub fn prepared_class_within(
        &self,
        entry: &PhysicalEntryId,
        max_class_bytes: u64,
        budget: &mut Budget,
    ) -> Result<crate::prepared::PreparedClassRead> {
        budget.poll()?;
        if self.kind != ArtifactKind::Zip {
            return Err(Error::invalid_input(
                "not_zip",
                "class entry reads require a ZIP snapshot",
            ));
        }
        let facts = self.container_facts(&entry.origin, budget)?;
        let position = usize::try_from(entry.ordinal).map_err(|_| {
            Error::invalid_input("entry_count_overflow", "entry ordinal does not fit usize")
        })?;
        let record = facts.record(position)?;
        if record.id.raw_name != entry.raw_name {
            return Err(Error::invalid_input(
                "entry_locator_mismatch",
                "entry ordinal does not match the requested raw name",
            ));
        }
        let read = facts.read_prepared_class(position, Some(max_class_bytes), budget)?;
        let depth = u64::try_from(entry.origin.steps.len()).map_err(|_| {
            Error::invalid_input("nested_depth_overflow", "nested origin is too deep")
        })?;
        // The read keeps this container for as long as its consumer keeps the read, so the product
        // is recorded as held now: every later read of the same container — a method request's
        // loader binding query, chiefly — reaches it instead of parsing the directory again.
        self.hold_container_facts(&facts);
        Ok(crate::prepared::PreparedClassRead::new(
            PhysicalClassLocation::ArchiveEntry {
                entry: record.id.clone(),
            },
            ClassBytesId {
                digest: read.digest.clone(),
                length: read.span.length,
            },
            entry.origin.current_container().clone(),
            depth,
            read.backing,
            read.span,
            facts.backing_digest().clone(),
            Some(ContainerFactsHandle { facts }),
        ))
    }

    /// The standalone-root equivalent of [`Self::prepared_class`]: this CLASS file as one read.
    ///
    /// A standalone snapshot has no container directory to verify an address against, and it does
    /// not need one: the open established the bytes this value holds, the snapshot identity is the
    /// digest of exactly those bytes, and there is no entry to locate. What the read states is
    /// therefore the root position, the whole file as the class bytes, and the snapshot's own digest
    /// as the backing's — all of it already established, so nothing is hashed again.
    pub fn prepared_root_class(
        &self,
        budget: &mut Budget,
    ) -> Result<crate::prepared::PreparedClassRead> {
        self.prepared_root_class_within(u64::MAX, budget)
    }

    /// The same read, refused when this snapshot's own bytes cross `max_class_bytes`.
    ///
    /// A standalone `CLASS` snapshot is materialized by [`Self::open`], so the ceiling cannot stop the
    /// file from being read — what it stops is *this read*: the file's length is compared with the
    /// ceiling before any of the read's charges or any preparation, and a larger class is refused
    /// under `class_bytes_ceiling` exactly as an archive entry of that size is. Nothing of the root
    /// is copied for a refusal: the read hands out the snapshot's own bytes, and a refused read hands
    /// out nothing.
    pub fn prepared_root_class_within(
        &self,
        max_class_bytes: u64,
        budget: &mut Budget,
    ) -> Result<crate::prepared::PreparedClassRead> {
        budget.poll()?;
        if self.kind != ArtifactKind::StandaloneClass {
            return Err(Error::invalid_input(
                "not_standalone_class",
                "root class reads are only available for a standalone CLASS snapshot",
            ));
        }
        let length = self.len();
        if length > max_class_bytes {
            return Err(class_bytes_ceiling(&self.id, length, max_class_bytes));
        }
        budget.check(CountedBudgetDimension::EntryBytes, length)?;
        budget.check(CountedBudgetDimension::ReadBytes, length)?;
        budget.charge(CountedBudgetDimension::ReadBytes, length)?;
        budget.charge(CountedBudgetDimension::EntryBytes, length)?;
        let digest = Digest(self.id.0.clone());
        Ok(crate::prepared::PreparedClassRead::new(
            PhysicalClassLocation::StandaloneRoot {
                snapshot: self.id.clone(),
            },
            ClassBytesId {
                digest: digest.clone(),
                length,
            },
            root_origin(&self.id).root_container,
            0,
            self.bytes.clone(),
            ByteSpan::new(0, length),
            digest,
            // A standalone root is not read out of a container directory: there is none, and the
            // snapshot's own bytes are the whole class.
            None,
        ))
    }

    /// One class read a caller **already performed**, in the shape a class task consumes.
    ///
    /// [`Self::prepared_class`] and [`Self::prepared_root_class`] are the reads a class task
    /// performs; this is the same value for a read that has already happened elsewhere. A caller
    /// that binds a target (a class view's identity, a search's elected candidate, a method
    /// request's own definition) holds the bytes a read of this snapshot verified, the digest and
    /// length that read established, and the location they came from; handing them over here lets
    /// the class be **prepared** from that read instead of being read a second time
    /// (`add-demand-driven-core-results` task 3.1: one operation, one materialization of the class
    /// it selected).
    ///
    /// Nothing is read, hashed, verified or charged here: the bytes are the caller's own evidence,
    /// already accounted as the read that produced them, and `class_bytes` is that read's identity —
    /// the same two values [`Self::prepared_class`] states for the same entry. What this adds is the
    /// one thing a preparation cannot build for itself: the container the class was read out of, as
    /// the active handle [`PreparedClassRead::container_facts`] hands out. `container` says whether
    /// that handle is wanted at all — see [`ContainerHandover`] — because reaching a container
    /// charges its directory, and a consumer that only decodes bodies out of the read asks the
    /// container nothing.
    ///
    /// A `location` that is not an address of this snapshot is an input error, exactly as it is for
    /// the read that would have produced the bytes.
    pub fn prepared_read_of(
        &self,
        location: PhysicalClassLocation,
        class_bytes: ClassBytesId,
        bytes: Vec<u8>,
        container: crate::prepared::ContainerHandover,
        budget: &mut Budget,
    ) -> Result<crate::prepared::PreparedClassRead> {
        budget.poll()?;
        let (container_id, depth, container_facts) = match &location {
            PhysicalClassLocation::ArchiveEntry { entry } => {
                if entry.snapshot() != &self.id {
                    return Err(Error::invalid_input(
                        "definition_snapshot_mismatch",
                        "the class read belongs to another snapshot",
                    ));
                }
                let origin = &entry.origin;
                let depth = u64::try_from(origin.steps.len()).map_err(|_| {
                    Error::invalid_input("nested_depth_overflow", "nested origin is too deep")
                })?;
                let facts = match container {
                    crate::prepared::ContainerHandover::Keep => {
                        let facts = self.container_facts(origin, budget)?;
                        // A hold is this read's own statement that it is what keeps the product
                        // alive, and it is recorded only when that is true: with a store attached
                        // the store is the one keeping it, and recording a hold here would answer a
                        // *later* request — one whose budget carries no store at all — from a
                        // product that outlives this read for the store's reason. That is exactly
                        // what [`HeldContainerFacts`]'s own note about a store's retention rules
                        // out. A request with no store reaches the product by its own read, so the
                        // hold is the read's and dies with it.
                        if budget.facts_cache().is_none() {
                            self.hold_container_facts(&facts);
                        }
                        Some(ContainerFactsHandle { facts })
                    }
                    crate::prepared::ContainerHandover::NotNeeded => None,
                };
                (origin.current_container().clone(), depth, facts)
            }
            PhysicalClassLocation::StandaloneRoot { snapshot } => {
                if snapshot != &self.id {
                    return Err(Error::invalid_input(
                        "definition_snapshot_mismatch",
                        "the class read belongs to another snapshot",
                    ));
                }
                (root_origin(&self.id).root_container, 0, None)
            }
        };
        let length = u64::try_from(bytes.len()).map_err(|_| {
            Error::invalid_input("class_size_overflow", "class length does not fit u64")
        })?;
        // The class bytes are this read's whole backing, so the backing's trusted identity is the
        // class bytes' own identity: the digest the read that produced them established.
        let backing_digest = class_bytes.digest.clone();
        Ok(crate::prepared::PreparedClassRead::new(
            location,
            class_bytes,
            container_id,
            depth,
            Arc::from(bytes),
            ByteSpan::new(0, length),
            backing_digest,
            container_facts,
        ))
    }

    /// The verified facts of one container, from the facts some live handle already holds, from
    /// retention when a store answers, and from this request's own read otherwise.
    ///
    /// The direct path is this function with no store attached: the same validation, the same
    /// ancestor walk and the same directory parse, with the product dropped when the request ends.
    /// Nothing about the access depends on a cache existing.
    ///
    /// The order is the one the two retentions have: a store that holds the product answers first
    /// (unchanged, counters included), then a product a **live handle** is still holding
    /// ([`HeldContainerFacts`]) is reused, and only a container that neither answers is read — its
    /// ancestors walked, its directory parsed, all of it charged. A store that is absent, full,
    /// refused or cleared therefore never makes a consumer rebuild facts it is already holding.
    ///
    /// The held index is written by the consumers that keep a product — a walk
    /// ([`crate::scope_cursor::ScopeCursor`]) and a prepared class read
    /// ([`Self::prepared_class_within`]) — and never by this access itself: a product answered from
    /// here and dropped by the caller is not "in use" by anybody, and a product a *store* retains is
    /// that store's own decision, not a hold (the store answers for it first, and its counters stay
    /// its own).
    ///
    /// It is crate-visible because the scope cursor opens the root container through this access
    /// rather than through a reader of its own, and because [`child_container_facts`] reuses its
    /// products for the containers a walk descends into: one container, one verification, one
    /// directory parse, however many consumers ask for it.
    pub(crate) fn container_facts(
        &self,
        origin: &ContainerOrigin,
        budget: &mut Budget,
    ) -> Result<Arc<ContainerFacts>> {
        if let Some(facts) = self.retained_container_facts(origin, budget)? {
            return Ok(facts);
        }
        if let Some(facts) = self.held_container_facts(origin, budget)? {
            return Ok(facts);
        }
        let facts = self.build_container_facts(origin, budget)?;
        if let Some(cache) = budget.facts_cache() {
            cache.remember_container(&facts);
        }
        Ok(facts)
    }

    /// The verified facts of one container, built for **this** request rather than answered from a
    /// retention or from a product some other consumer is holding.
    ///
    /// This is the access a walk that *reports its own directory validation* uses: the entry cursor
    /// states, per container it reached, that this invocation walked that container's directory to
    /// its end — a range, a charged record count and a per-entry validation, all of them this
    /// invocation's own work — so it opens the container directly. The listing it publishes is
    /// therefore paid for exactly as the provider's own enumeration pays for it, whether or not the
    /// request attached a store or another consumer of this snapshot happens to hold the same
    /// product: an answer taken from either would make the range this walk reports describe work
    /// this invocation did not do. The product is still offered to the request's store afterwards,
    /// so a later read that states no such range — a nested entry read, a prepared class — is
    /// answered from it as usual.
    pub(crate) fn walked_container_facts(
        &self,
        origin: &ContainerOrigin,
        budget: &mut Budget,
    ) -> Result<Arc<ContainerFacts>> {
        let facts = self.build_container_facts(origin, budget)?;
        if let Some(cache) = budget.facts_cache() {
            cache.remember_container(&facts);
        }
        Ok(facts)
    }

    /// The entry count the snapshot's own root container declares, read without opening it.
    ///
    /// This is the one denominator a request can state for a container it never got to examine: the
    /// end-of-central-directory record of the snapshot's bytes declares how many entries the root
    /// container holds, and that declaration is a property of those bytes. Nothing is materialized
    /// and nothing is charged — the answer states a range, it does not read entries — and a
    /// snapshot whose bytes do not locate their own record declares nothing at all.
    pub(crate) fn declared_root_entries(&self) -> Option<u64> {
        if self.kind != ArtifactKind::Zip {
            return None;
        }
        ZipArchive::from_slice(&self.bytes)
            .ok()
            .map(|archive| archive.entries_hint())
    }

    /// Records one container product as held by a live handle of this snapshot.
    ///
    /// Called by the consumers that **keep** a product for their own lifetime: the scope cursor as
    /// it opens a container and descends into one, and every prepared class read that carries the
    /// container it was read out of. Recording is idempotent and cheap — one map slot under the
    /// product's origin and a weak reference — and it is what makes the product answerable to
    /// *other* consumers of the same origin for as long as it lives. It is deliberately not called
    /// by [`Self::container_facts`] itself: a product the caller dropped, or one a store retains,
    /// is not a hold, and letting either register one would make a store's own retention decision
    /// change what an unrelated request is answered from.
    pub(crate) fn hold_container_facts(&self, facts: &Arc<ContainerFacts>) {
        self.held_facts().hold(facts);
    }

    /// The verified facts of one container when some live handle still holds them.
    ///
    /// The current request decides first, exactly as a store hit does: the budget is polled, so a
    /// cancelled or expired request is refused before anything is served, and the container's own
    /// depth is checked against this request's `NestedDepth` limit, so facts reached under a wider
    /// allowance are never handed to a request that may not reach that depth.
    fn held_container_facts(
        &self,
        origin: &ContainerOrigin,
        budget: &mut Budget,
    ) -> Result<Option<Arc<ContainerFacts>>> {
        budget.poll()?;
        let depth = u64::try_from(origin.steps.len()).map_err(|_| {
            Error::invalid_input(
                "nested_depth_overflow",
                "container origin is too deep to address",
            )
        })?;
        budget.check_nested_depth(depth)?;
        Ok(self.held_facts().held(origin))
    }

    /// The held-facts index, behind the one lock this snapshot keeps.
    ///
    /// The lock is not a concurrency feature of the reads themselves — nothing here runs work in
    /// parallel — it is what keeps [`ArtifactSnapshot`] `Send + Sync` while one index is shared by
    /// every clone of it, exactly like the budget's facts store. Like that store, a poisoned lock is
    /// read as it stands: an index is not a reason to fail every later request.
    fn held_facts(&self) -> MutexGuard<'_, HeldContainerFacts> {
        self.held_facts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The facts of one container when the request's cache already holds them.
    ///
    /// This is the *retention* half only: it never builds anything, so a caller that must not pay
    /// for a directory parse (the selected-entry read on a warm path) can ask without risking one.
    fn retained_container_facts(
        &self,
        origin: &ContainerOrigin,
        budget: &mut Budget,
    ) -> Result<Option<Arc<ContainerFacts>>> {
        let Some(cache) = budget.facts_cache().cloned() else {
            return Ok(None);
        };
        cache.container(origin, budget)
    }

    /// Reads one container's facts from the fixed snapshot, walking its real ancestor chain.
    ///
    /// Every ancestor is reached through its own parent's directory — the parent entry is located
    /// by ordinal and raw name, its derivation is recomputed and compared, and its bytes are
    /// materialized under the budget through the same verified path a class read uses — so a
    /// forged origin chain is refused at the step that does not derive and no sibling entry is
    /// ever touched.
    fn build_container_facts(
        &self,
        origin: &ContainerOrigin,
        budget: &mut Budget,
    ) -> Result<Arc<ContainerFacts>> {
        self.check_container_origin(origin)?;
        // The backing's trusted identity travels with it: the snapshot's own digest for the root
        // container, and the digest the materializing read produced for a nested one. Neither is
        // recomputed here, so handing a backing to a class read costs no hash of the container.
        let (backing, backing_digest) = match origin.steps.split_last() {
            None => (self.bytes.clone(), Digest(self.id.0.clone())),
            Some((step, parents)) => {
                let parent_origin = ContainerOrigin {
                    snapshot: origin.snapshot.clone(),
                    root_container: origin.root_container.clone(),
                    steps: parents.to_vec(),
                };
                let parent = self.container_facts(&parent_origin, budget)?;
                if step.child_container
                    != derive_child_container(&parent_origin, step.via_ordinal, &step.via_raw_name)
                {
                    return Err(Error::invalid_input(
                        "child_container_mismatch",
                        "nested origin child container is not derived from its verified parent entry",
                    ));
                }
                let depth = u64::try_from(origin.steps.len()).map_err(|_| {
                    Error::invalid_input("nested_depth_overflow", "nested origin is too deep")
                })?;
                budget.check_nested_depth(depth)?;
                let position = usize::try_from(step.via_ordinal).map_err(|_| {
                    Error::invalid_input("entry_count_overflow", "entry ordinal does not fit usize")
                })?;
                let Some(parent_entry) = parent.entries.get(position) else {
                    return Err(Error::invalid_input(
                        "entry_not_found",
                        "entry ordinal is absent",
                    ));
                };
                if parent_entry.id.raw_name.0 != step.via_raw_name.0 {
                    return Err(Error::invalid_input(
                        "entry_locator_mismatch",
                        "entry ordinal does not match the requested raw name",
                    ));
                }
                if parent_entry.nested_archive != NestedArchiveState::CandidateNotScanned {
                    return Err(Error::invalid_input(
                        "nested_parent_not_candidate",
                        "nested origin parent entry is not an archive candidate",
                    ));
                }
                let materialized =
                    parent.read_entry(position, budget, MaterializationAccounting::Intermediate)?;
                if let Some(cache) = budget.facts_cache() {
                    cache.note_nested_materialization(materialized.bytes.len() as u64);
                }
                (Arc::from(materialized.bytes), materialized.content_digest)
            }
        };
        let parsed = parse_container_directory(
            &backing,
            origin,
            DirectoryIntent::Locate,
            budget,
            &mut |_| {},
        )?;
        if !matches!(parsed.execution, ExecutionReport::Complete { .. }) {
            return Err(container_refusal(budget, origin, &parsed));
        }
        Ok(Arc::new(ContainerFacts::new(
            origin.clone(),
            backing,
            backing_digest,
            parsed,
        )))
    }

    /// Checks that `origin` addresses a container of **this** snapshot and that every step of the
    /// chain derives from the one before it.
    ///
    /// The derivation is the one [`derive_child_origin`] performs and the tree walk uses, so a chain
    /// reached through this function and a child reached from a held parent
    /// ([`child_container_facts`]) name the same container. Because the rule is applied at every
    /// step, an origin that was not produced by reading this snapshot's entries cannot be addressed
    /// at all — a caller cannot name a container that does not exist, or move a whole chain under
    /// another parent, by handing in a fabricated `ContainerOrigin`.
    fn check_container_origin(&self, origin: &ContainerOrigin) -> Result<()> {
        if self.kind != ArtifactKind::Zip {
            return Err(Error::invalid_input(
                "not_zip",
                "container access requires a ZIP snapshot",
            ));
        }
        if origin.snapshot != self.id {
            return Err(Error::invalid_input(
                "entry_snapshot_mismatch",
                "container origin does not belong to this snapshot",
            ));
        }
        let root = root_origin(&self.id);
        if origin.root_container != root.root_container {
            return Err(Error::invalid_input(
                "entry_origin_mismatch",
                "container origin root container does not match this snapshot",
            ));
        }
        let mut current = root;
        for step in &origin.steps {
            if step.child_container
                != derive_child_container(&current, step.via_ordinal, &step.via_raw_name)
            {
                return Err(Error::invalid_input(
                    "child_container_mismatch",
                    "nested origin child container is not derived from its verified parent entry",
                ));
            }
            current.steps.push(step.clone());
        }
        Ok(())
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
        // A retained container answers the read without a second directory pass and without
        // re-materializing the parents it already holds. The record it serves is checked against
        // the caller's metadata and the selected entry's own bytes are still verified in full, so
        // a hit changes what the request pays for, never what it proves.
        if let Some(facts) = self.retained_container_facts(&entry.id.origin, budget)? {
            return facts.read_caller_entry(entry, budget, final_accounting);
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
                held_facts: Arc::new(Mutex::new(HeldContainerFacts::default())),
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
            if let Some(cache) = budget.facts_cache() {
                cache.note_nested_materialization(materialized.bytes.len() as u64);
            }
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
            held_facts: Arc::new(Mutex::new(HeldContainerFacts::default())),
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

// -----------------------------------------------------------------------------------------------
// Directed container access: one directory parser, one locator, one verified backing
// -----------------------------------------------------------------------------------------------

/// What one directory parse is for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DirectoryIntent {
    /// The entries are the request's own result: each one is charged as a published item and a
    /// repeated raw name also publishes the duplicate diagnostic.
    Publish,
    /// The directory is the internal locator of one container: its records are charged as derived
    /// storage the request holds while it locates a name, but nothing is published, so the
    /// duplicate diagnostic has no report to go into. Dropping the report is the point of the
    /// local path; dropping the charge would be a hidden subsidy, so the entries are still billed.
    Locate,
}

/// One parse of one container's central directory, complete or stopped.
///
/// A stopped parse still carries the records it accepted and the execution that says why it
/// stopped; the caller decides whether an incomplete directory may be used at all. The directed
/// access never uses one.
struct ParsedDirectory {
    entries: Vec<PhysicalEntry>,
    /// The wayfinder that reaches each entry's local header and data without a second
    /// central-directory pass, parallel to `entries`.
    wayfinders: Vec<ZipArchiveEntryWayfinder>,
    /// Raw name -> positions in `entries`, ascending. This is the multi-value locator: two
    /// physical entries with one raw name are two positions and are never merged.
    names: BTreeMap<Vec<u8>, Vec<u64>>,
    diagnostics: Vec<Diagnostic>,
    coverage: Coverage,
    execution: ExecutionReport,
}

/// Parses one container's central directory out of `bytes`.
///
/// `bytes` is the container's own backing — the snapshot's bytes for the root container, the
/// materialized nested container otherwise — and `origin` is where those bytes live, so every
/// record this yields is identified by the container it really belongs to. The record validation,
/// the charging and the stop semantics are the ones the whole-snapshot enumeration always had; the
/// intent is the only thing that differs between the two callers.
///
/// The returned `execution` is `Complete` only when the whole declared directory was read and
/// matched the EOCD's own count. Every other ending returns the prefix it read together with the
/// reason, exactly as `enumerate` publishes it.
fn parse_container_directory<F>(
    bytes: &Arc<[u8]>,
    origin: &ContainerOrigin,
    intent: DirectoryIntent,
    budget: &mut Budget,
    completed_hook: &mut F,
) -> Result<ParsedDirectory>
where
    F: FnMut(u64),
{
    let archive = ZipArchive::from_slice(bytes).map_err(zip_invalid("zip_open"))?;
    // The parse is counted where it starts, not where it succeeds: a directory that stopped on the
    // budget or on damage was still parsed, and a measurement that only counted complete ones would
    // report less work than the request really did.
    if let Some(cache) = budget.facts_cache() {
        cache.note_directory_parse();
    }
    let expected = archive.entries_hint();
    let directory_offset = archive.directory_offset();
    let mut iterator = archive.entries();
    let mut entries = Vec::new();
    let mut wayfinders = Vec::new();
    let mut diagnostics = Vec::new();
    let mut names: BTreeMap<Vec<u8>, Vec<u64>> = BTreeMap::new();
    let mut ranges: BTreeMap<u64, (u64, u64)> = BTreeMap::new();
    let mut ordinal = 0_u64;

    loop {
        if let Err(error) = budget.poll() {
            return Ok(terminated_directory(
                entries,
                wayfinders,
                names,
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
                return Ok(terminated_directory(
                    entries,
                    wayfinders,
                    names,
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
            return Ok(terminated_directory(
                entries,
                wayfinders,
                names,
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
            let wayfinder = header.wayfinder();
            let local = archive
                .get_entry(wayfinder)
                .map_err(zip_invalid("local_entry"))?;
            budget.poll()?;
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
            let pending_diagnostic = (intent == DirectoryIntent::Publish)
                .then(|| {
                    names
                        .get(&raw_name)
                        .and_then(|seen| seen.first())
                        .map(|first| Diagnostic {
                            code: "duplicate_raw_name".into(),
                            severity: DiagnosticSeverity::Warning,
                            message: format!(
                                "entry {ordinal} repeats raw name first seen at ordinal {first}"
                            ),
                            provenance: None,
                        })
                })
                .flatten();
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
                        origin: origin.clone(),
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
                return Ok(terminated_directory(
                    entries,
                    wayfinders,
                    names,
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
        }
        names
            .entry(entry.id.raw_name.0.clone())
            .or_default()
            .push(ordinal);
        ranges.insert(range_start, (range_end, ordinal));
        entries.push(entry);
        wayfinders.push(header.wayfinder());
        ordinal = ordinal.checked_add(1).ok_or_else(|| {
            Error::invalid_input("entry_count_overflow", "central entry ordinal overflow")
        })?;
        completed_hook(ordinal);
    }

    if ordinal != expected {
        return Ok(terminated_directory(
            entries,
            wayfinders,
            names,
            diagnostics,
            EnumerationProgress {
                completed: ordinal,
                expected,
                known_end: None,
            },
            Error::invalid_input(
                "entry_count_mismatch",
                format!("EOCD declares {expected} entries but central directory yielded {ordinal}"),
            ),
            budget,
        ));
    }
    Ok(ParsedDirectory {
        entries,
        wayfinders,
        names,
        diagnostics,
        coverage: enumeration_coverage(
            CoverageState::CompleteWithinSchema,
            ordinal,
            expected,
            None,
        ),
        execution: ExecutionReport::Complete {
            usage: budget.usage(),
        },
    })
}

/// A per-record residency weight for one retained directory. A **proxy**, like every weight this
/// change adds: it stands for the map slot, the identity and the layout a `PhysicalEntry` keeps,
/// and no allocator measured it.
const DIRECTORY_ENTRY_WEIGHT: u64 = 192;

/// A per-name residency weight for one locator slot, on top of the name's own bytes.
const NAME_TABLE_SLOT_WEIGHT: u64 = 48;

/// One container's verified facts: the immutable backing its entries live in, its complete central
/// directory, the raw-name locator over that directory and the wayfinder that reaches each entry's
/// data without a second central-directory pass.
///
/// A `ContainerFacts` is only ever built for a container whose directory parsed completely — the
/// builder refuses a stopped parse instead of keeping a prefix — so "this container holds no such
/// name" is never read out of an incomplete directory. It is immutable once built, which is what
/// lets one be retained across requests and shared by every handle of one store.
///
/// The value holds the container's **own** backing: the bytes its central directory and its
/// entries live in. A nested container's backing is the materialized output of its parent entry,
/// which was verified in full before it was opened as an archive; a locator from this directory is
/// therefore only ever used against this backing, and the recorded spans are re-checked against it
/// on every read.
///
/// `backing_digest` is the trusted content identity of that backing, and it is **carried** rather
/// than computed: the root container's backing is the snapshot's own bytes, whose digest is the
/// snapshot identity the open established, and a nested container's backing is the output of the
/// verified read that materialized it, which produced a digest of exactly those bytes. Neither has
/// to be hashed again when a class read hands the backing out.
#[derive(Debug)]
pub(crate) struct ContainerFacts {
    origin: ContainerOrigin,
    backing: Arc<[u8]>,
    backing_digest: Digest,
    entries: Vec<PhysicalEntry>,
    wayfinders: Vec<ZipArchiveEntryWayfinder>,
    names: BTreeMap<Vec<u8>, Vec<u64>>,
    weight: u64,
}

/// One container's verified facts, held as an **active strong reference**.
///
/// This is the reader's handover of a product the running operation already has. The facts are
/// immutable, they were verified for one physical origin of one snapshot, and a caller that holds
/// this handle keeps them alive on its own terms — so a class read, a loader binding query or any
/// other read of that container is answered from the same product instead of parsing the directory
/// again. Nothing about a read is weakened by holding one: the entry's local header is still checked
/// against this directory's record, its CRC and sizes are still re-established from the bytes, and
/// the loader/profile/version checks of the layers above are untouched.
///
/// What it is **not**: a store, a cache or a second lifetime for the reader's own facts. It retains
/// nothing by itself, has no capacity and no key beyond the physical origin, and dropping the last
/// handle simply lets the product die (the request's [`crate::facts_cache::FactsCache`], if any,
/// stays the only thing that retains facts nothing is using any more). Handles are handed out by
/// [`ArtifactSnapshot::prepared_class`] — the read carries the container it was prepared from — and
/// by [`crate::scope_cursor::ScopeCursor::container_facts`], which hands over the container the walk
/// is currently inside.
#[derive(Clone, Debug)]
pub struct ContainerFactsHandle {
    /// The shared product; `pub(crate)` because only this module builds handles.
    pub(crate) facts: Arc<ContainerFacts>,
}

impl ContainerFactsHandle {
    /// The physical origin of the container this product was verified for.
    pub fn origin(&self) -> &ContainerOrigin {
        self.facts.origin()
    }
}

/// How many entries the held-facts index keeps before it sweeps the dead ones out of itself.
///
/// Below this figure the index is a handful of origins and a sweep would cost more than it saves;
/// above it a sweep runs whenever the index has doubled since the last one, which keeps the total
/// work of sweeping proportional to the number of holds taken.
const HELD_FACTS_SWEEP_FLOOR: usize = 64;

/// The container products some live handle of one snapshot still holds, by the origin they were read
/// at.
///
/// A read consults this **after** the request's facts store has answered nothing, and it is the
/// reason a store that is absent, full, refused or cleared does not make a request re-parse a
/// directory whose product some consumer is still holding. The index stores weak references only: an
/// entry is answerable exactly while the `Arc<ContainerFacts>` its holder recorded is alive — a
/// walk's own stack, a prepared class read carrying the container it was read from — and a dead entry
/// is dropped the next time its origin is asked for.
///
/// What is deliberately **not** a hold is a store's own retention: a product only a
/// [`crate::facts_cache::FactsCache`] keeps is not one a consumer is using, that store answers for it
/// first, and letting its retention register here would make one store's capacity decision change
/// what an unrelated request is answered from.
///
/// This is deliberately **not** a store either: it retains no memory (a weak reference keeps nothing
/// alive), it has no capacity, no declaration, no report and no counters, and it can never answer
/// with a product this snapshot did not verify at that origin. What it decides is only *where* an
/// answer comes from: a product that is already in use, or a fresh parse.
#[derive(Debug, Default)]
struct HeldContainerFacts {
    by_origin: BTreeMap<ContainerOrigin, Weak<ContainerFacts>>,
    /// The index's length when it was last swept of dead entries.
    swept_at: usize,
}

impl HeldContainerFacts {
    /// Records one live product under its origin.
    ///
    /// A sweep of dead entries may run first; it only ever removes entries nothing holds any more,
    /// so every live product stays answerable across it.
    fn hold(&mut self, facts: &Arc<ContainerFacts>) {
        let origin = facts.origin().clone();
        self.by_origin.insert(origin, Arc::downgrade(facts));
        let sweep_above = self.swept_at.saturating_mul(2).max(HELD_FACTS_SWEEP_FLOOR);
        if self.by_origin.len() > sweep_above {
            self.by_origin.retain(|_, held| held.strong_count() > 0);
            self.swept_at = self.by_origin.len();
        }
    }

    /// The live product for one origin, or `None` when nothing holds it any more.
    ///
    /// A dead entry is removed here rather than kept: the product it remembered was released, and no
    /// later lookup of that origin may be answered from the memory of it.
    fn held(&mut self, origin: &ContainerOrigin) -> Option<Arc<ContainerFacts>> {
        match self.by_origin.get(origin).and_then(Weak::upgrade) {
            Some(facts) => Some(facts),
            None => {
                self.by_origin.remove(origin);
                None
            }
        }
    }
}

impl ContainerFacts {
    fn new(
        origin: ContainerOrigin,
        backing: Arc<[u8]>,
        backing_digest: Digest,
        parsed: ParsedDirectory,
    ) -> Self {
        let weight = container_weight(backing.len() as u64, &parsed.entries, &parsed.names);
        Self {
            origin,
            backing,
            backing_digest,
            entries: parsed.entries,
            wayfinders: parsed.wayfinders,
            names: parsed.names,
            weight,
        }
    }

    pub(crate) fn origin(&self) -> &ContainerOrigin {
        &self.origin
    }

    /// The bytes this container's own directory and its entries live in.
    pub(crate) fn backing(&self) -> &Arc<[u8]> {
        &self.backing
    }

    /// This container's complete central directory, in physical order.
    ///
    /// The records are the directory's own, so the ordinal of a record is the position it holds in
    /// the archive: a walk that iterates this slice visits entries in exactly the order the archive
    /// declares them.
    pub(crate) fn entries(&self) -> &[PhysicalEntry] {
        &self.entries
    }

    /// The trusted content identity of [`Self::backing`].
    pub(crate) fn backing_digest(&self) -> &Digest {
        &self.backing_digest
    }

    /// The residency weight this product contributes to a store that retains it.
    pub(crate) fn weight(&self) -> u64 {
        self.weight
    }

    /// The physical entries whose raw name is exactly `raw_name`, in central-directory order.
    ///
    /// A hit copies the matched records only: the locator is keyed by name, so nothing walks or
    /// clones the directory, and two records under one name stay two records.
    pub(crate) fn candidates(&self, raw_name: &[u8]) -> Vec<PhysicalEntry> {
        match self.names.get(raw_name) {
            Some(positions) => positions
                .iter()
                .filter_map(|position| self.entries.get(*position as usize).cloned())
                .collect(),
            None => Vec::new(),
        }
    }

    /// The records of this directory that repeat a raw name, as `(ordinal, first ordinal)`.
    ///
    /// The Publish intent of the one directory parser reports a repeat for every record whose name
    /// it had already seen, in record order. A walk that yields this directory's records instead of
    /// a report states the same fact about the same product, so the pairs are derived from the
    /// locator rather than re-detected record by record by the caller.
    pub(crate) fn repeated_names(&self) -> Vec<(u64, u64)> {
        let mut repeated = Vec::new();
        for positions in self.names.values() {
            let Some((&first, rest)) = positions.split_first() else {
                continue;
            };
            for position in rest {
                repeated.push((*position, first));
            }
        }
        repeated.sort_unstable();
        repeated
    }

    /// The one record of this directory at `position`, or the refusal that says the address does
    /// not exist in it.
    fn record(&self, position: usize) -> Result<&PhysicalEntry> {
        self.entries.get(position).ok_or_else(|| {
            Error::invalid_input(
                "entry_not_found",
                "entry ordinal is absent from the container",
            )
        })
    }

    /// Reads the caller's entry out of this retained directory and its backing.
    ///
    /// The rules of a nested read are unchanged; only the way to the record is. The record at the
    /// caller's ordinal has to carry the caller's raw name, the caller's whole metadata has to
    /// equal the **authoritative** record this directory parsed — a caller-deserialized report is
    /// never trusted over it — and the selected entry's own local header, CRC and sizes are then
    /// verified against that record before any byte is returned.
    fn read_caller_entry(
        &self,
        entry: &PhysicalEntry,
        budget: &mut Budget,
        accounting: MaterializationAccounting,
    ) -> Result<MaterializedEntry> {
        if self.origin != entry.id.origin {
            return Err(Error::invalid_input(
                "entry_origin_mismatch",
                "entry origin does not match the retained container",
            ));
        }
        let position = usize::try_from(entry.id.ordinal).map_err(|_| {
            Error::invalid_input("entry_count_overflow", "entry ordinal does not fit usize")
        })?;
        let authoritative = self.record(position)?;
        if authoritative.id.raw_name != entry.id.raw_name {
            return Err(Error::invalid_input(
                "entry_locator_mismatch",
                "entry ordinal does not match the requested raw name",
            ));
        }
        if authoritative != entry {
            return Err(Error::invalid_input(
                "entry_metadata_mismatch",
                "caller-supplied nested entry metadata differs from the fixed snapshot",
            ));
        }
        let mut materialized = self.read_entry(position, budget, accounting)?;
        materialized.entry = entry.id.clone();
        materialized.usage = budget.usage();
        Ok(materialized)
    }

    /// Reads one class entry of this directory for a prepared class, without copying its bytes when
    /// the entry is stored.
    ///
    /// The verification is the one every read of this container performs and in the same order: the
    /// address must exist in this directory and its local header must match the **record this
    /// directory parsed** (raw name, offset, CRC, compressed and uncompressed sizes, the data
    /// descriptor when it declares one, and the compressed span the record describes), and the
    /// entry's own bytes are then read through the same verifying reader, so the CRC and the sizes
    /// are re-established from the content itself. What differs is only where the verified bytes
    /// end up: a stored entry stays where it is, so the caller gets the container's own backing and
    /// the span inside it, and a deflated entry is produced by the shared materialization and handed
    /// back as a backing of its own. The digest is computed from the chunks the verifying reader
    /// checked, once, and it is the digest of the bytes the caller is about to read either way.
    ///
    /// The read is charged as an intermediate one — `ReadBytes` for the compressed selection,
    /// `EntryBytes` for the logical bytes the read produced — exactly like the analysis read of the
    /// same entry: the bytes are consumed inside the operation that asked for them, not returned as
    /// its answer.
    fn read_prepared_class(
        &self,
        position: usize,
        ceiling: Option<u64>,
        budget: &mut Budget,
    ) -> Result<PreparedEntryRead> {
        let record = self.record(position)?;
        check_entry_readable(record)?;
        // The ceiling is decided from this directory's own record **before** anything of the entry is
        // selected: a class the caller does not admit is refused with nothing materialized, and the
        // read of it is not even started.
        admit_class_bytes(record, ceiling)?;
        let archive = ZipArchive::from_slice(&self.backing).map_err(zip_invalid("zip_open"))?;
        budget.poll()?;
        let local = archive
            .get_entry(self.wayfinders[position])
            .map_err(zip_invalid("local_entry"))?;
        budget.poll()?;
        verify_local_against_record(record, &local)?;
        if record.compression == EntryCompression::Stored {
            budget.check(CountedBudgetDimension::EntryBytes, record.uncompressed_size)?;
            budget.check(CountedBudgetDimension::ReadBytes, record.compressed_size)?;
            let compressed_len = as_u64(local.data().len())?;
            if compressed_len != record.compressed_size {
                return Err(Error::invalid_input(
                    "compressed_size_mismatch",
                    format!(
                        "central size {} differs from local data span {compressed_len}",
                        record.compressed_size
                    ),
                ));
            }
            budget.charge(CountedBudgetDimension::ReadBytes, compressed_len)?;
            let digest = verify_stored_entry(&local, record, ceiling, budget)?;
            return Ok(PreparedEntryRead {
                backing: self.backing.clone(),
                span: record.layout.compressed_data.clone(),
                digest,
            });
        }
        let materialized = materialize_verified(
            record,
            &local,
            budget,
            MaterializationAccounting::Intermediate,
            ceiling,
            &mut |_| {},
        )?;
        let length = as_u64(materialized.bytes.len())?;
        Ok(PreparedEntryRead {
            backing: Arc::from(materialized.bytes),
            span: ByteSpan::new(0, length),
            digest: materialized.content_digest,
        })
    }

    /// Materializes one entry of this directory from its verified backing.
    ///
    /// The record's own local header is read at the offset it names and checked against the
    /// record, the compressed span is checked to be the one the record describes, and the bytes
    /// are then read through the same verifying reader every other read uses, so CRC and size are
    /// re-established for the selected entry itself. No central-directory pass happens here: the
    /// directory this product holds is the one that was charged when it was parsed.
    fn read_entry(
        &self,
        position: usize,
        budget: &mut Budget,
        accounting: MaterializationAccounting,
    ) -> Result<MaterializedEntry> {
        let record = self.record(position)?;
        let archive = ZipArchive::from_slice(&self.backing).map_err(zip_invalid("zip_open"))?;
        budget.poll()?;
        let local = archive
            .get_entry(self.wayfinders[position])
            .map_err(zip_invalid("local_entry"))?;
        budget.poll()?;
        verify_local_against_record(record, &local)?;
        materialize_verified(record, &local, budget, accounting, None, &mut |_| {})
    }
}

/// The residency weight of one container product: the backing bytes, the parsed records and the
/// name table that locates them.
fn container_weight(
    backing_len: u64,
    entries: &[PhysicalEntry],
    names: &BTreeMap<Vec<u8>, Vec<u64>>,
) -> u64 {
    let mut weight = backing_len.saturating_add(
        u64::try_from(entries.len())
            .unwrap_or(u64::MAX)
            .saturating_mul(DIRECTORY_ENTRY_WEIGHT),
    );
    for (name, positions) in names {
        weight = weight
            .saturating_add(u64::try_from(name.len()).unwrap_or(u64::MAX))
            .saturating_add(NAME_TABLE_SLOT_WEIGHT)
            .saturating_add(
                u64::try_from(positions.len())
                    .unwrap_or(u64::MAX)
                    .saturating_mul(8),
            );
    }
    weight
}

/// The refusal one stopped directory parse stands for.
///
/// The parse already charged the dimension it stopped on to the limit, so the numbers the budget
/// holds are the evidence it stopped with. The message names the container the request was
/// addressing, because a local lookup's failure has to say which position it could not decide.
fn container_refusal(budget: &Budget, origin: &ContainerOrigin, parsed: &ParsedDirectory) -> Error {
    let label = format!("container `{}`", origin.current_container().0);
    let message = match parsed.diagnostics.last() {
        Some(diagnostic) => format!(
            "{label} could not be read completely ({}: {}); the position stays undecided and the \
             lookup does not continue",
            diagnostic.code, diagnostic.message
        ),
        None => format!(
            "{label} could not be read completely; the position stays undecided and the lookup \
             does not continue"
        ),
    };
    match &parsed.execution {
        ExecutionReport::Complete { .. } => panic!("a complete directory is not a refusal"),
        ExecutionReport::Cancelled { .. } => Error::Cancelled { reason: message },
        ExecutionReport::Partial { reason, .. } | ExecutionReport::Failed { reason, .. } => {
            match reason {
                TerminationReason::BudgetExceeded { dimension } => {
                    let (limit, consumed) = dimension_evidence(budget, *dimension);
                    Error::BudgetExceeded {
                        dimension: *dimension,
                        limit,
                        consumed,
                        requested: 1,
                    }
                }
                TerminationReason::Error { code } => Error::invalid_input(code.clone(), message),
                TerminationReason::Unsupported { code } => {
                    Error::unsupported(code.clone(), message)
                }
            }
        }
    }
}

/// Limit and usage of one dimension, read from its own slots.
fn dimension_evidence(budget: &Budget, dimension: BudgetDimension) -> (u64, u64) {
    let limits = budget.limits();
    let usage = budget.usage();
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

/// Checks one entry's local header and data descriptor against the record the container's central
/// directory produced.
///
/// This is the one local-vs-record validation: the whole-snapshot read builds its record from the
/// central record it just scanned and calls this, and a retained directory hands the record it
/// parsed and calls the same function, so both paths accept and refuse exactly the same bytes. The
/// central side is the **record**, never a caller's report, which is why a deserialized `PhysicalEntry`
/// cannot weaken the check that follows.
fn verify_local_against_record(
    record: &PhysicalEntry,
    local: &rawzip::ZipSliceEntry<'_>,
) -> Result<()> {
    let local_header = local.local_header();
    if record.id.raw_name.0 != local_header.file_path().as_bytes() {
        return Err(Error::invalid_input(
            "central_local_name_mismatch",
            "central and local raw entry names differ",
        ));
    }
    if record.compression_method != local_header.compression_method().as_u16() {
        return Err(Error::invalid_input(
            "central_local_method_mismatch",
            "central and local compression methods differ",
        ));
    }
    if record.flags.raw_bits != local_header.flags().bits() {
        return Err(Error::invalid_input(
            "central_local_flags_mismatch",
            "central and local flags differ",
        ));
    }
    if !record.flags.data_descriptor
        && (record.crc32 != local_header.crc32()
            || record.compressed_size != local_header.compressed_size_hint()
            || record.uncompressed_size != local_header.uncompressed_size_hint())
    {
        return Err(Error::invalid_input(
            "central_local_integrity_mismatch",
            "central and local CRC or size fields differ",
        ));
    }
    if local
        .data_descriptor()
        .map_err(zip_invalid("data_descriptor"))?
        .is_some_and(|descriptor| {
            descriptor.crc32() != record.crc32
                || descriptor.compressed_size() != record.compressed_size
                || descriptor.uncompressed_size() != record.uncompressed_size
        })
    {
        return Err(Error::invalid_input(
            "descriptor_central_mismatch",
            format!(
                "entry {} data descriptor conflicts with central directory",
                record.id.ordinal
            ),
        ));
    }
    if local.compressed_data_range()
        != (
            record.layout.compressed_data.start,
            record
                .layout
                .compressed_data
                .start
                .saturating_add(record.layout.compressed_data.length),
        )
    {
        return Err(Error::invalid_input(
            "entry_locator_mismatch",
            "entry identity no longer matches the fixed central-directory locator",
        ));
    }
    Ok(())
}

/// Reads and verifies the bytes of one entry whose local header already matches its record.
///
/// The checks before the read are the contract every materialization keeps: an encrypted or
/// unsupported method is refused instead of read, the budget is asked *before* the bytes are
/// selected (`ReadBytes`, plus `EntryBytes` and, for a caller-owned result, `OutputBytes`), and
/// the compressed length the record claims has to be the length really present. The read itself
/// runs through rawzip's verifying reader, so the CRC and the uncompressed size are re-established
/// from the bytes, and a decode that stops or ends early clears the buffer instead of returning it.
fn materialize_verified<F>(
    record: &PhysicalEntry,
    local: &rawzip::ZipSliceEntry<'_>,
    budget: &mut Budget,
    accounting: MaterializationAccounting,
    ceiling: Option<u64>,
    hook: &mut F,
) -> Result<MaterializedEntry>
where
    F: FnMut(usize),
{
    check_entry_readable(record)?;
    // The ceiling's pre-materialization half, applied wherever one is stated: the record's declared
    // size decides it before a byte of the entry is produced.
    admit_class_bytes(record, ceiling)?;
    budget.check(CountedBudgetDimension::EntryBytes, record.uncompressed_size)?;
    if accounting == MaterializationAccounting::CallerOutput {
        budget.check(
            CountedBudgetDimension::OutputBytes,
            record.uncompressed_size,
        )?;
    }
    budget.check(CountedBudgetDimension::ReadBytes, record.compressed_size)?;
    let compressed_len = as_u64(local.data().len())?;
    if compressed_len != record.compressed_size {
        return Err(Error::invalid_input(
            "compressed_size_mismatch",
            format!(
                "central size {} differs from local data span {compressed_len}",
                record.compressed_size
            ),
        ));
    }
    budget.charge(CountedBudgetDimension::ReadBytes, compressed_len)?;

    let mut output = Vec::new();
    let read_result = match record.compression {
        EntryCompression::Stored => {
            let reader = std::io::Cursor::new(local.data());
            read_verified(
                local,
                reader,
                &mut output,
                budget,
                accounting,
                ceiling,
                hook,
            )
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
            read_verified(
                local,
                decoder,
                &mut output,
                budget,
                accounting,
                ceiling,
                hook,
            )
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
    if actual != record.uncompressed_size {
        return Err(Error::invalid_input(
            "uncompressed_size_mismatch",
            format!(
                "central size {} differs from actual output {actual}",
                record.uncompressed_size
            ),
        ));
    }
    let content_digest = crate::model::Digest(blake3::hash(&output).to_hex().to_string());
    Ok(MaterializedEntry {
        entry: record.id.clone(),
        bytes: output,
        content_digest,
        usage: budget.usage(),
    })
}

/// The two refusals every read of one entry shares, before any byte is selected.
///
/// An encrypted entry and an entry whose compression method this reader does not implement are
/// refused wherever they are met — the materializing read, the stored in-place read a prepared class
/// uses, and the nested-container read — so "which entry can be read at all" is one rule rather than
/// one per path.
fn check_entry_readable(record: &PhysicalEntry) -> Result<()> {
    if record.flags.encrypted || record.flags.strong_encryption {
        return Err(Error::unsupported(
            "encrypted_zip_entry",
            "encrypted, strong-encryption, and AES entries are not supported",
        ));
    }
    if !matches!(
        record.compression,
        EntryCompression::Stored | EntryCompression::Deflated
    ) {
        return Err(Error::unsupported(
            "zip_compression_method",
            format!(
                "compression method {} is not supported",
                record.compression_method
            ),
        ));
    }
    Ok(())
}

/// One entry's verified bytes, handed back without a copy when the entry is stored.
///
/// `span` indexes `backing` and yields the entry's bytes; `digest` is the trusted content identity of
/// the bytes at that span — the container's own digest for a stored entry, and the digest of the
/// produced bytes for one that had to be materialized.
pub(crate) struct PreparedEntryRead {
    pub(crate) backing: Arc<[u8]>,
    pub(crate) span: ByteSpan,
    pub(crate) digest: Digest,
}

/// Reads one stored entry through the same verifying reader every other read uses, without copying
/// its bytes out of the container backing.
///
/// The bytes are where they are, so the read cannot hand back a buffer it filled; what it *can*
/// establish is the same thing the copying read establishes, and it does: the CRC and the
/// uncompressed size are re-established from the content by rawzip's verifying reader, the stream has
/// to end exactly at the data the record names, the length has to be the recorded one, and the
/// `EntryBytes` charges are made per chunk while the bytes pass. The digest is computed from those
/// same verified chunks, so the identity handed to the caller describes bytes the read really
/// checked rather than bytes it merely located.
fn verify_stored_entry(
    local: &rawzip::ZipSliceEntry<'_>,
    record: &PhysicalEntry,
    ceiling: Option<u64>,
    budget: &mut Budget,
) -> Result<Digest> {
    let reader = BudgetedEntryReader {
        reader: std::io::Cursor::new(local.data()),
        budget,
        accounting: MaterializationAccounting::Intermediate,
    };
    let mut verifier = local.verifying_reader(reader);
    let mut hasher = blake3::Hasher::new();
    let mut chunk = [0_u8; READ_CHUNK];
    let mut total = 0_u64;
    loop {
        let count = verifier.read(&mut chunk).map_err(map_verification_error)?;
        if count == 0 {
            break;
        }
        hasher.update(&chunk[..count]);
        total = total.checked_add(as_u64(count)?).ok_or_else(|| {
            Error::invalid_input("classfile_span_overflow", "entry size overflow")
        })?;
        // A stored entry hands out the container's own backing, so this read copies nothing: there
        // is no buffer for the ceiling to bound, but the length the read establishes is still the
        // admission's own, and a stream longer than the ceiling is refused here rather than after it
        // was verified in full.
        if let Some(ceiling) = ceiling
            && total > ceiling
        {
            return Err(class_bytes_ceiling_reached(total, ceiling));
        }
    }
    let reader = verifier.into_inner().reader;
    if reader.position() != local.data().len() as u64 {
        return Err(Error::invalid_input(
            "entry_integrity",
            "stored entry has trailing compressed bytes",
        ));
    }
    if total != record.uncompressed_size {
        return Err(Error::invalid_input(
            "uncompressed_size_mismatch",
            format!(
                "central size {} differs from the bytes the entry holds ({total})",
                record.uncompressed_size
            ),
        ));
    }
    Ok(Digest(hasher.finalize().to_hex().to_string()))
}

/// The verified facts of one child container, reached from a parent whose facts the request already
/// holds.
///
/// [`ArtifactSnapshot::container_facts`] can build any container's facts from its origin alone, by
/// walking the ancestors that reach it — which is what a lookup that only knows an origin must do.
/// A walk that already holds the parent's facts does not need that second walk: the parent's own
/// complete directory is the authority for the child entry, so the child is reached from the record
/// at `entry_position`, and one container is parsed once per request instead of once per path that
/// reaches it. Nothing about the verification is skipped on the way: the record has to be an archive
/// candidate, the nested-depth rule is applied to the child's depth, the entry is materialized
/// through the parent's own verified read (record cross-check, local header, CRC and sizes), and the
/// child's directory is parsed by the one parser and refused unless it is complete.
pub(crate) fn child_container_facts(
    parent: &ContainerFacts,
    entry_position: usize,
    budget: &mut Budget,
) -> Result<Arc<ContainerFacts>> {
    let parent_entry = parent.record(entry_position)?;
    if parent_entry.nested_archive != NestedArchiveState::CandidateNotScanned {
        return Err(Error::invalid_input(
            "nested_parent_not_candidate",
            "nested origin parent entry is not an archive candidate",
        ));
    }
    let child_origin = derive_child_origin(&parent.origin, &parent_entry.id);
    let depth = u64::try_from(child_origin.steps.len())
        .map_err(|_| Error::invalid_input("nested_depth_overflow", "nested origin is too deep"))?;
    budget.check_nested_depth(depth)?;
    let materialized = parent.read_entry(
        entry_position,
        budget,
        MaterializationAccounting::Intermediate,
    )?;
    if let Some(cache) = budget.facts_cache() {
        cache.note_nested_materialization(materialized.bytes.len() as u64);
    }
    let backing: Arc<[u8]> = Arc::from(materialized.bytes);
    let parsed = parse_container_directory(
        &backing,
        &child_origin,
        DirectoryIntent::Locate,
        budget,
        &mut |_| {},
    )?;
    if !matches!(parsed.execution, ExecutionReport::Complete { .. }) {
        return Err(container_refusal(budget, &child_origin, &parsed));
    }
    let facts = Arc::new(ContainerFacts::new(
        child_origin,
        backing,
        materialized.content_digest,
        parsed,
    ));
    if let Some(cache) = budget.facts_cache() {
        cache.remember_container(&facts);
    }
    Ok(facts)
}

/// The origin of one container entry's child container, derived from the parent's origin and the
/// entry's own identity.
///
/// One derivation, used by the artifact-tree walk and the scope cursor alike: a child container's id
/// is a function of the parent's whole chain plus the entry's ordinal and raw name, so two paths that
/// derived it differently would address different containers for the same physical entry — and the
/// origin chain is exactly what the directed container access validates a step at a time.
pub(crate) fn derive_child_origin(
    parent: &ContainerOrigin,
    entry: &PhysicalEntryId,
) -> ContainerOrigin {
    let mut child = parent.clone();
    child.steps.push(ContainerOriginStep {
        via_ordinal: entry.ordinal,
        via_raw_name: entry.raw_name.clone(),
        child_container: derive_child_container(parent, entry.ordinal, &entry.raw_name),
    });
    child
}

/// The origin of the one root container a snapshot establishes.
pub(crate) fn root_origin(snapshot: &SnapshotId) -> ContainerOrigin {
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

/// Folds one subtree failure into the walk's own issue, in the tree walk's vocabulary.
///
/// A subtree that could not be read is never a stop of the walk itself: it bounds the report with
/// the state that failure means — a cancellation, an exhausted dimension, an unsupported structure
/// or an error code — while the walk keeps going. The entry walk
/// ([`crate::entry_cursor::EntryCursor`]) reports through this same function, so one failure means
/// one state wherever a walk carries it.
pub(crate) fn merge_tree_error(
    first_issue: &mut Option<ExecutionReport>,
    error: &Error,
    budget: &Budget,
) {
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

/// The diagnostic one failure publishes, located at the entry the failed read hangs from.
///
/// The severity, the code and the provenance are the whole-tree walk's own, and the scope cursor
/// publishes through the same function so a subtree that could not be read has one shape wherever a
/// request reports it.
pub(crate) fn tree_diagnostic(error: &Error, parent_entry: Option<&PhysicalEntry>) -> Diagnostic {
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

/// The directory parse that stopped, as a value that carries exactly the evidence
/// `enumerate` publishes for the same stop: the accepted prefix, the coverage that says how much
/// of the declared directory was read, the execution and the diagnostic naming the cause.
fn terminated_directory(
    entries: Vec<PhysicalEntry>,
    wayfinders: Vec<ZipArchiveEntryWayfinder>,
    names: BTreeMap<Vec<u8>, Vec<u64>>,
    mut diagnostics: Vec<Diagnostic>,
    progress: EnumerationProgress,
    error: Error,
    budget: &Budget,
) -> ParsedDirectory {
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
    ParsedDirectory {
        entries,
        wayfinders,
        names,
        diagnostics,
        coverage: enumeration_coverage(
            CoverageState::Partial,
            progress.completed,
            progress.expected,
            progress.known_end,
        ),
        execution,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MaterializationAccounting {
    Intermediate,
    CallerOutput,
}

/// Reads one entry's data through the verifying reader, charging every chunk, and enforces
/// `ceiling` **while the bytes are produced**.
///
/// The ceiling is not a comparison made after the read: each chunk is requested with the bytes the
/// ceiling still allows (plus the one that would cross it), so an entry whose bytes are longer than
/// it was admitted to be cannot be materialized past the ceiling at all — the produced buffer is
/// bounded by `ceiling + 1` bytes whatever the entry's own declared or real size is, and the read
/// that crosses the ceiling is refused with [`class_bytes_ceiling_reached`] instead of continuing.
/// Without a ceiling the read is the plain chunked one it always was.
fn read_verified<R: Read, F: FnMut(usize)>(
    entry: &rawzip::ZipSliceEntry<'_>,
    reader: R,
    output: &mut Vec<u8>,
    budget: &mut Budget,
    accounting: MaterializationAccounting,
    ceiling: Option<u64>,
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
        let request = match ceiling {
            Some(ceiling) => {
                let produced = u64::try_from(output.len())
                    .map_err(|_| std::io::Error::other("output length overflow"))?;
                if produced > ceiling {
                    return Err(std::io::Error::other(BudgetReadError(
                        class_bytes_ceiling_reached(produced, ceiling),
                    )));
                }
                usize::try_from(ceiling.saturating_sub(produced).saturating_add(1))
                    .unwrap_or(usize::MAX)
                    .min(READ_CHUNK)
            }
            None => READ_CHUNK,
        };
        let count = verifier.read(&mut chunk[..request])?;
        if count == 0 {
            break;
        }
        output.extend_from_slice(&chunk[..count]);
        hook(output.len());
    }
    Ok(verifier.into_inner().reader)
}

/// The refusal of a class read whose entry's declared size crosses the ceiling it was admitted
/// under, decided from the directory record and before any of the entry is materialized.
fn admit_class_bytes(record: &PhysicalEntry, ceiling: Option<u64>) -> Result<()> {
    let Some(ceiling) = ceiling else {
        return Ok(());
    };
    if record.uncompressed_size > ceiling {
        return Err(Error::invalid_input(
            "class_bytes_ceiling",
            format!(
                "the class entry declares {} bytes and this read admits at most {ceiling}: the class \
                 is refused before it is materialized, and nothing of it is read",
                record.uncompressed_size
            ),
        ));
    }
    Ok(())
}

/// The refusal of a root class read whose own bytes cross the ceiling, decided before the read
/// charges anything.
fn class_bytes_ceiling(snapshot: &SnapshotId, length: u64, ceiling: u64) -> Error {
    Error::invalid_input(
        "class_bytes_ceiling",
        format!(
            "the class is {length} bytes and this read admits at most {ceiling}: {snapshot:?} is \
             refused, and nothing of it is read"
        ),
    )
}

/// The refusal of a class read that crossed its ceiling while the bytes were being produced.
///
/// It is the same code as [`admit_class_bytes`]'s, because it is the same admission failing at the
/// second point it is enforced: a directory record may understate an entry's real size, and the read
/// then stops at the ceiling rather than materializing the rest of it.
fn class_bytes_ceiling_reached(produced: u64, ceiling: u64) -> Error {
    Error::invalid_input(
        "class_bytes_ceiling",
        format!(
            "the class entry produced at least {produced} bytes and this read admits at most \
             {ceiling}: the read stopped at the ceiling instead of materializing the whole entry"
        ),
    )
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

    #[test]
    fn the_ceiling_bounds_the_bytes_a_read_produces_not_the_bytes_it_hoped_for() {
        // The ceiling is enforced **at the point of production**: each chunk is requested with only
        // the bytes the ceiling still allows (plus the one that would cross it), so a read admitted
        // for `ceiling` bytes can never produce more than `ceiling + 1` of them — and the read that
        // crosses the ceiling is refused by this admission rather than left to the entry's own size
        // check, which only knows what the entry declared.
        let payload = vec![0x5a_u8; 4 * 1024];
        let bytes = zip(&[(b"entry", &payload, STORE)]);
        let archive = ZipArchive::from_slice(&bytes).unwrap();
        let header = archive.entries().next_entry().unwrap().unwrap();
        let local = archive.get_entry(header.wayfinder()).unwrap();

        let mut ceiled = budget();
        let mut output = Vec::new();
        let error = read_verified(
            &local,
            Cursor::new(local.data()),
            &mut output,
            &mut ceiled,
            MaterializationAccounting::Intermediate,
            Some(64),
            &mut |_| {},
        )
        .expect_err("the entry is longer than the ceiling admits");
        assert!(
            error
                .get_ref()
                .and_then(|inner| inner.downcast_ref::<BudgetReadError>())
                .is_some_and(|error| matches!(&error.0, Error::InvalidInput { code, .. } if code == "class_bytes_ceiling")),
            "the crossing is this admission's refusal, not the entry's own: {error}"
        );
        assert_eq!(
            output.len(),
            65,
            "the read produced exactly one byte past the ceiling it may not cross"
        );
        assert!(
            ceiled.usage().entry_bytes <= 65,
            "and it charged no more than it produced: {:?}",
            ceiled.usage()
        );

        // Without a ceiling the same read is the plain chunked one: the whole entry, and nothing
        // about it changes for the reads that state no admission.
        let mut unceiled = budget();
        let mut output = Vec::new();
        read_verified(
            &local,
            Cursor::new(local.data()),
            &mut output,
            &mut unceiled,
            MaterializationAccounting::Intermediate,
            None,
            &mut |_| {},
        )
        .expect("a read without a ceiling admits the entry");
        assert_eq!(output, payload);
    }
}
