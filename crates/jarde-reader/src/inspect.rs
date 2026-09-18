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
    ExecutionReport, PhysicalClassLocation,
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
