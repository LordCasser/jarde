//! Stateless synchronous composition of artifact and classfile contracts.

use crate::artifact::{
    ArtifactInput, ArtifactKind, ArtifactSnapshot, EnumerationReport, PhysicalEntry,
};
use crate::budget::Budget;
use crate::classfile::{
    BytecodeInspection, BytecodeStop, BytecodeStopPhase, HeaderInspection, InspectionMode,
    MethodSelector,
};
use crate::error::{Error, Result};
use crate::model::{
    ClassBytesId, Coverage, CoverageDimension, CoverageRange, CoverageState, Digest,
    ExecutionReport, PhysicalEntryId, SnapshotId,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default)]
pub struct Engine;

#[derive(Clone, Copy, Debug)]
pub enum ClassTarget<'a> {
    Root,
    Entry(&'a PhysicalEntry),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClassSource {
    pub snapshot: SnapshotId,
    pub entry: Option<PhysicalEntryId>,
    pub class_bytes: ClassBytesId,
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

impl Engine {
    pub const fn new() -> Self {
        Self
    }

    pub fn open(&self, input: ArtifactInput, budget: &mut Budget) -> Result<ArtifactSnapshot> {
        ArtifactSnapshot::open(input, budget)
    }

    pub fn enumerate(
        &self,
        snapshot: &ArtifactSnapshot,
        budget: &mut Budget,
    ) -> Result<EnumerationReport> {
        snapshot.enumerate(budget)
    }

    pub fn inspect_header(
        &self,
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

    pub fn inspect_method_bytecode(
        &self,
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

fn bytecode_coverage(inspection: &BytecodeInspection) -> Result<Coverage> {
    let code_length = inspection.code_span.length;
    let prefix_end = inspection.instructions.last().map_or(Ok(0), |fact| {
        u64::from(fact.bci)
            .checked_add(u64::from(fact.width))
            .ok_or_else(|| {
                Error::invalid_input("classfile_coverage_overflow", "instruction prefix overflow")
            })
    })?;
    if prefix_end > code_length {
        return Err(Error::invalid_input(
            "classfile_coverage_out_of_bounds",
            "instruction prefix exceeds code length",
        ));
    }
    let handlers_returned = u64::try_from(inspection.exception_handlers.len()).map_err(|_| {
        Error::invalid_input(
            "classfile_coverage_overflow",
            "handler count does not fit u64",
        )
    })?;
    let handlers_total = u64::from(inspection.exception_handler_count);
    if handlers_returned > handlers_total {
        return Err(Error::invalid_input(
            "classfile_coverage_out_of_bounds",
            "returned handler count exceeds declared count",
        ));
    }
    let complete = matches!(inspection.execution, ExecutionReport::Complete { .. });
    let handlers_complete = complete
        || !matches!(
            inspection.stopped_at.as_ref().map(BytecodeStop::phase),
            Some(BytecodeStopPhase::ExceptionHandlers)
        );
    let mut scanned = vec![CoverageRange {
        label: "method_code_bci".into(),
        start: 0,
        end: prefix_end,
    }];
    scanned.push(CoverageRange {
        label: "exception_handler_ordinal".into(),
        start: 0,
        end: handlers_returned,
    });
    let mut skipped = Vec::new();
    if prefix_end < code_length {
        skipped.push(CoverageRange {
            label: "method_code_bci".into(),
            start: prefix_end,
            end: code_length,
        });
    }
    if !handlers_complete && handlers_returned < handlers_total {
        skipped.push(CoverageRange {
            label: "exception_handler_ordinal".into(),
            start: handlers_returned,
            end: handlers_total,
        });
    }
    Ok(Coverage {
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
    })
}

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
                    snapshot: snapshot.id().clone(),
                    entry: None,
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
                snapshot: snapshot.id().clone(),
                entry: Some(materialized.entry),
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
