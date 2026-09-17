//! Stateless synchronous composition of artifact and classfile contracts.

use crate::artifact::{
    ArtifactInput, ArtifactKind, ArtifactSnapshot, ArtifactTreeReport, EnumerationReport,
    PhysicalEntry,
};
use crate::budget::Budget;
use crate::classfile::{
    BytecodeInspection, BytecodeStop, BytecodeStopPhase, HeaderInspection, InspectionMode,
    MethodSelector,
};
use crate::error::{Error, Result};
use crate::model::{
    ClassBytesId, Coverage, CoverageDimension, CoverageRange, CoverageState, Digest,
    ExecutionReport, PhysicalClassLocation,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default)]
pub struct Engine;

#[derive(Clone, Copy, Debug)]
pub enum ClassTarget<'a> {
    Root,
    Entry(&'a PhysicalEntry),
}

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

    pub fn enumerate_artifact_tree(
        &self,
        snapshot: &ArtifactSnapshot,
        budget: &mut Budget,
    ) -> Result<ArtifactTreeReport> {
        snapshot.enumerate_artifact_tree(budget)
    }

    pub fn select_multi_release(
        &self,
        snapshot: &ArtifactSnapshot,
        view: &crate::view::RuntimeView,
        budget: &mut Budget,
    ) -> Result<crate::multi_release::MultiReleaseViewReport> {
        crate::multi_release::select(snapshot, view, budget)
    }

    pub fn query(
        &self,
        snapshot: &ArtifactSnapshot,
        request: &crate::query::QueryRequest,
        budget: &mut Budget,
    ) -> Result<crate::query::QueryReport> {
        crate::query::execute(snapshot, request, budget)
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

    /// Demand-bound symbol resolution under an explicit environment (P2 entry point).
    ///
    /// The request shape is checked first: a snapshot the content does not provide, a target
    /// whose kind contradicts the reference use, or a dispatch range whose tree root cannot
    /// describe this snapshot is an input error (`resolution_snapshot_mismatch`,
    /// `resolution_target_use_mismatch`, `query_artifact_tree_root_mismatch`). Environment
    /// problems are not an error; they are part of the report.
    ///
    /// A class symbol is looked up by name in the declared search order and the selected
    /// definition is reported as `Resolved` / `Missing` / `Ambiguous` (2.1); a member symbol is
    /// resolved by the JVMS 5.4.3 member rules under the invocation-kind and access rules (2.3);
    /// a request that also names a dispatch range (`request.dispatch`) enumerates the known
    /// candidates of that range with their open-world evidence once its member declaration
    /// resolved (2.5) — never a unique runtime target. A request whose environment the validator
    /// rejected keeps the honest unavailable state, because a rejected environment never yields
    /// a definition.
    pub fn resolve_symbol(
        &self,
        content: &[ArtifactSnapshot],
        request: &crate::resolver::ResolutionRequest,
        budget: &mut Budget,
    ) -> Result<crate::resolver::ResolutionReport> {
        crate::resolver::validate_request(content, request)?;
        Ok(crate::resolver::resolution_report(content, request, budget))
    }

    /// Declaration-reference scan under an explicit environment (P2 entry point).
    ///
    /// Same request-level check as [`Engine::resolve_symbol`]. The query scans the explicit
    /// scope for candidate use sites with the structure consumers, resolves every candidate's
    /// owner, and publishes only the candidates that resolve to the requested declaration;
    /// candidates no search could decide are reported as unresolved instead of excluded, and a
    /// rejected environment keeps the honest unavailable state.
    pub fn declaration_references(
        &self,
        content: &[ArtifactSnapshot],
        query: &crate::resolver::DeclarationRefQuery,
        budget: &mut Budget,
    ) -> Result<crate::resolver::DeclarationRefReport> {
        crate::resolver::validate_declaration_reference_query(content, query)?;
        crate::resolver::declaration_reference_report(content, query, budget)
    }

    /// Method IR analysis under an explicit environment (P2 entry point).
    ///
    /// An empty stage set is an input error (`analysis_no_stages`); every other mismatch
    /// is checked like [`Engine::resolve_symbol`]. The requested stages are then validated
    /// against the fixed pass table *before* anything runs: a schedule the table cannot
    /// serve is an input error (`ir_pass_prerequisite_missing`, `ir_pass_order_invalid`,
    /// `ir_pass_graph_cycle`, `ir_stale_fact`) rather than a half-initialized pipeline.
    /// This slice performs no phase and reads no artifact byte, so a valid request is
    /// answered with the report that lists its scheduled phases as `NotPerformed`.
    pub fn analyze_method(
        &self,
        content: &[ArtifactSnapshot],
        request: &crate::ir::MethodAnalysisRequest,
        budget: &mut Budget,
    ) -> Result<crate::ir::MethodAnalysisReport> {
        crate::ir::validate_request(content, request)?;
        crate::passes::validate_requested_stages(&request.stages)?;
        Ok(crate::ir::analysis_report(content, request, budget))
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
