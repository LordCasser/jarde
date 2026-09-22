//! Bounded materialization and inspection of one class inside a snapshot.
//!
//! One inspection request is one class: the target names either the root of a standalone
//! `CLASS` snapshot or one physical entry of a `ZIP` snapshot, and it is materialized through
//! the snapshot's own bounded read — the entry is validated against that snapshot, the read is
//! charged in the category it belongs to, and the identity of the bytes is derived from what was
//! returned. The inspection that follows reads only those bytes.
//!
//! This is the reader's inspection entry, not the facade's: it takes a snapshot and a budget and
//! returns the report of one read. Nothing above the reader decides what a target is.

use crate::artifact::{ArtifactKind, ArtifactSnapshot, PhysicalEntry};
use crate::budget::Budget;
use crate::classfile::{BytecodeInspection, HeaderInspection, InspectionMode, MethodSelector};
use crate::error::{Error, Result};
use crate::model::{
    ClassBytesId, Coverage, CoverageDimension, CoverageRange, CoverageState, Digest,
    ExecutionReport, PhysicalClassLocation, PhysicalDefinitionId, SnapshotId,
    physical_variant_for_path,
};
use serde::{Deserialize, Serialize};

/// Which class of a snapshot an inspection request names.
#[derive(Clone, Copy, Debug)]
pub enum ClassTarget<'a> {
    /// The class the standalone `CLASS` snapshot *is*.
    Root,
    /// One physical entry of a `ZIP` snapshot, as the snapshot itself enumerated it.
    Entry(&'a PhysicalEntry),
}

/// Where one materialized class came from and which bytes it is.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ClassSource {
    pub location: PhysicalClassLocation,
    pub class_bytes: ClassBytesId,
}

impl ClassSource {
    pub fn snapshot(&self) -> &crate::model::SnapshotId {
        self.location.snapshot()
    }

    pub fn entry(&self) -> Option<&crate::model::PhysicalEntryId> {
        self.location.entry()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EngineHeaderReport {
    pub source: ClassSource,
    pub inspection: HeaderInspection,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EngineBytecodeReport {
    pub source: ClassSource,
    pub inspection: BytecodeInspection,
    pub coverage: Coverage,
}

/// Materializes the target class and inspects its header under `mode`.
pub fn inspect_header(
    snapshot: &ArtifactSnapshot,
    target: ClassTarget<'_>,
    budget: &mut Budget,
    mode: InspectionMode,
) -> Result<EngineHeaderReport> {
    let (bytes, source) = materialize(snapshot, target, budget)?;
    let inspection = crate::classfile::inspect_header(&bytes, budget, mode)?;
    let coverage = header_coverage(source.class_bytes.length);
    Ok(EngineHeaderReport {
        source,
        inspection,
        coverage,
        execution: ExecutionReport::Complete {
            usage: budget.usage(),
        },
    })
}

/// Materializes the target class and inspects the body of the selected method.
pub fn inspect_method_bytecode(
    snapshot: &ArtifactSnapshot,
    target: ClassTarget<'_>,
    selector: MethodSelector,
    budget: &mut Budget,
) -> Result<EngineBytecodeReport> {
    let (bytes, source) = materialize(snapshot, target, budget)?;
    let inspection = crate::classfile::inspect_method_bytecode(&bytes, selector, budget)?;
    let coverage = bytecode_coverage(&inspection)?;
    Ok(EngineBytecodeReport {
        source,
        inspection,
        coverage,
    })
}

/// The class bytes and their identity, read under the snapshot's own bounded accounting.
///
/// The target's kind must match the snapshot's kind: asking a `ZIP` snapshot for its root class,
/// or a standalone `CLASS` snapshot for one of its entries, is an input error rather than an
/// empty read. An entry belongs to exactly one snapshot and one identity — the read itself
/// refuses an entry another snapshot or another locator owns.
fn materialize(
    snapshot: &ArtifactSnapshot,
    target: ClassTarget<'_>,
    budget: &mut Budget,
) -> Result<(Vec<u8>, ClassSource)> {
    match target {
        ClassTarget::Root => {
            if snapshot.kind() != ArtifactKind::StandaloneClass {
                return Err(Error::invalid_input(
                    "class_target_root_on_zip",
                    "root class target requires a standalone CLASS snapshot",
                ));
            }
            let bytes = snapshot.root_bytes(budget)?;
            let class_bytes = ClassBytesId {
                digest: Digest(blake3::hash(&bytes).to_hex().to_string()),
                length: u64::try_from(bytes.len()).map_err(|_| {
                    Error::invalid_input("class_size_overflow", "class length does not fit u64")
                })?,
            };
            Ok((
                bytes,
                ClassSource {
                    location: PhysicalClassLocation::StandaloneRoot {
                        snapshot: snapshot.id().clone(),
                    },
                    class_bytes,
                },
            ))
        }
        ClassTarget::Entry(entry) => {
            if snapshot.kind() != ArtifactKind::Zip {
                return Err(Error::invalid_input(
                    "class_target_entry_on_class",
                    "entry class target requires a ZIP snapshot",
                ));
            }
            let materialized = snapshot.read_entry(entry, budget)?;
            let source = ClassSource {
                location: PhysicalClassLocation::ArchiveEntry {
                    entry: materialized.entry,
                },
                class_bytes: ClassBytesId {
                    digest: materialized.content_digest,
                    length: u64::try_from(materialized.bytes.len()).map_err(|_| {
                        Error::invalid_input("class_size_overflow", "class length does not fit u64")
                    })?,
                },
            };
            Ok((materialized.bytes, source))
        }
    }
}

/// Reads the bytes of the standalone class a `CLASS` snapshot **is**, and states the source they are.
///
/// The read a listing performs for a standalone candidate, and the one read an identity-addressed
/// request performs for a standalone definition: the snapshot is the class, so there is no entry to
/// locate and no chain to verify — the bytes and the identity derived from them are the whole answer.
/// The bytes are charged as the caller's own output, exactly as [`inspect_header`]'s root read charges
/// them.
///
/// What it does not do: it does not parse the bytes, and it claims nothing about them beyond their
/// position and length. `Err` is reserved for a snapshot that is not a standalone `CLASS` and for a
/// refused charge; bytes that are not a class file are returned as they are.
pub fn materialize_root(
    snapshot: &ArtifactSnapshot,
    budget: &mut Budget,
) -> Result<(Vec<u8>, ClassSource)> {
    if snapshot.kind() != ArtifactKind::StandaloneClass {
        return Err(Error::invalid_input(
            "class_target_root_on_zip",
            "root class target requires a standalone CLASS snapshot",
        ));
    }
    let bytes = snapshot.root_bytes(budget)?;
    let class_bytes = ClassBytesId {
        digest: Digest(blake3::hash(&bytes).to_hex().to_string()),
        length: u64::try_from(bytes.len()).map_err(|_| class_size_overflow())?,
    };
    Ok((
        bytes,
        ClassSource {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes,
        },
    ))
}

/// Reads the class one physical definition names, and states the source those bytes came from.
///
/// This is the read an **identity-addressed** request performs: the caller already holds a
/// [`PhysicalDefinitionId`] — a location, the class bytes' digest and the syntactic variant — and
/// asks for exactly that class. The location decides which bounded read is used (the standalone
/// root's own bytes, or the entry the location names, located through the snapshot's directed
/// container access) and the result is verified against the definition **before** it is returned:
///
/// * the location's snapshot is this snapshot, and its shape matches the snapshot's kind;
/// * an entry location names an entry this snapshot really holds at those coordinates;
/// * the entry's raw name derives exactly the variant the definition names;
/// * the bytes' own digest and length are the ones the definition names.
///
/// Any disagreement is a structured `Error` and never a read of "something near" the identity: an
/// identity that could quietly denote other bytes would not be an identity, and a caller that
/// checks its result against the definition afterwards would be doing this read's job.
///
/// The bytes of a container entry are charged as an intermediate read, because they are consumed
/// inside the request that asked for them rather than returned as the request's own answer.
///
/// What it does **not** do: it does not parse the bytes, does not verify the entry is a class file
/// at all, and does not resolve, load or analyse anything. The digest is a check that the bytes are
/// the ones the *identity* was derived from, never a claim that those bytes are legal.
///
/// # The read the request's store may answer
///
/// An entry location's bytes are read out of the snapshot's own directed container access, and that
/// read is the one the request's store may answer when an earlier request of the same snapshot
/// already performed exactly it (change `reuse-selected-class-read`). The third element of the
/// answer says which of the two happened: `true` means the store handed back the bytes and the
/// identity a previous read established, so **this** request performed no entry read — a caller that
/// counts the reads a request performed must not count this one — and `false` means the read above
/// ran. Everything else is the same on both paths: the location, the variant and the declared class
/// bytes are still checked here, against the very identity the answer carries.
///
/// A standalone root is never answered from retention: its bytes are the snapshot's own, already
/// resident, and the read of them is a copy this request needs anyway.
pub fn materialize_definition(
    snapshot: &ArtifactSnapshot,
    definition: &PhysicalDefinitionId,
    budget: &mut Budget,
) -> Result<(Vec<u8>, ClassSource, bool)> {
    match &definition.location {
        PhysicalClassLocation::StandaloneRoot { snapshot: named } => {
            require_snapshot(snapshot, named)?;
            let (bytes, source) = materialize_root(snapshot, budget)?;
            require_class_bytes(definition, &source.class_bytes)?;
            Ok((bytes, source, false))
        }
        PhysicalClassLocation::ArchiveEntry { entry } => {
            require_snapshot(snapshot, entry.snapshot())?;
            if snapshot.kind() != ArtifactKind::Zip {
                return Err(definition_location_mismatch(
                    "an archive-entry definition requires a ZIP snapshot",
                ));
            }
            let record = snapshot.container_record(entry, budget)?.ok_or_else(|| {
                Error::invalid_input(
                    "definition_entry_not_found",
                    "the definition names an entry this snapshot does not hold at those coordinates",
                )
            })?;
            if physical_variant_for_path(&record.id.raw_name.0) != definition.variant {
                return Err(Error::invalid_input(
                    "definition_variant_mismatch",
                    "the definition's physical variant is not the one its entry's raw name derives",
                ));
            }
            // The read of the definition this caller selected: the request's store answers it when
            // an earlier request of the same snapshot already performed exactly this read. A hit
            // hands back the bytes and the identity *that* read established, so the checks below run
            // against them exactly as they run against a fresh read's answer.
            let (bytes, class_bytes, retained) =
                match snapshot.retained_definition_read(definition, budget)? {
                    Some((bytes, class_bytes)) => ((*bytes).clone(), class_bytes, true),
                    None => {
                        let materialized = snapshot.read_entry_for_analysis(&record, budget)?;
                        snapshot.remember_definition_read(
                            definition,
                            &materialized.bytes,
                            &materialized.content_digest,
                            budget,
                        );
                        let class_bytes = ClassBytesId {
                            digest: materialized.content_digest,
                            length: u64::try_from(materialized.bytes.len())
                                .map_err(|_| class_size_overflow())?,
                        };
                        (materialized.bytes, class_bytes, false)
                    }
                };
            require_class_bytes(definition, &class_bytes)?;
            Ok((
                bytes,
                ClassSource {
                    location: definition.location.clone(),
                    class_bytes,
                },
                retained,
            ))
        }
    }
}

fn require_snapshot(snapshot: &ArtifactSnapshot, named: &SnapshotId) -> Result<()> {
    if named != snapshot.id() {
        return Err(Error::invalid_input(
            "definition_snapshot_mismatch",
            "the definition belongs to another snapshot",
        ));
    }
    Ok(())
}

fn require_class_bytes(definition: &PhysicalDefinitionId, read: &ClassBytesId) -> Result<()> {
    if &definition.class_bytes != read {
        return Err(Error::invalid_input(
            "definition_class_bytes_mismatch",
            "the bytes at the definition's location are not the class bytes the definition names",
        ));
    }
    Ok(())
}

fn definition_location_mismatch(message: &str) -> Error {
    Error::invalid_input("definition_location_mismatch", message)
}

fn class_size_overflow() -> Error {
    Error::invalid_input("class_size_overflow", "class length does not fit u64")
}

fn header_coverage(class_length: u64) -> Coverage {
    Coverage {
        artifact_structural: CoverageDimension {
            state: CoverageState::CompleteWithinSchema,
            scanned: vec![CoverageRange {
                label: "class_header_schema".into(),
                start: 0,
                end: class_length,
            }],
            skipped: Vec::new(),
            uninterpreted_extensions: Vec::new(),
        },
        runtime_resolution: CoverageDimension::not_requested(),
        dynamic_analysis: CoverageDimension::not_requested(),
    }
}

/// Coverage of the bytecode inspection of one method body.
///
/// The mapping is the reader's own ([`crate::classfile::method_code_coverage`]), shared with
/// the method facts so one body has one coverage plane whichever path read it.
fn bytecode_coverage(inspection: &BytecodeInspection) -> Result<Coverage> {
    crate::classfile::method_code_coverage(
        inspection.code_span.length,
        &inspection.instructions,
        inspection.exception_handlers.len(),
        inspection.exception_handler_count,
        &inspection.execution,
        inspection.stopped_at.as_ref(),
    )
}
