//! XRef scan orchestration: unit iteration, result billing, page and coverage.
//!
//! This module owns the deterministic order of the P1 scan:
//!
//! 1. containers in provider order, then entry ordinal ascending,
//! 2. inside one entry, the consumer sub-scan order declared by [`scan_unit`]
//!    (`resource`, `code`, `metadata`, `bootstrap`) and each sub-scan's own
//!    position order.
//!
//! That order is the base of the cursor semantics: a continuation replays one
//! unit and skips the items an earlier page already published, so consecutive
//! pages neither repeat nor skip results.
//!
//! This module is the single billing point for query results: every item and every
//! domain diagnostic that enters the report is charged one `ResultItems` before it
//! is published, and a failed charge keeps the reliable prefix and stops without
//! publishing an unbilled item. Sub-scan modules must not charge `ResultItems`
//! themselves, must not add or change public schema types, and must fill only
//! their own file.
//!
//! The scan reports what it really did: `execution` mirrors the provider and scan
//! outcome (`Complete`/`Partial`/`Cancelled`/`Failed`), coverage carries the
//! provider ranges and this pass's own ordinal/byte ranges, and a page limit or an
//! interruption never turns into `Complete`.
//!
//! Two relations have no resolver in P1: `references_definition` and
//! `may_dispatch_to` run the raw constant-pool probe and publish its candidates under
//! the caller's relation (see [`keeps_pool_evidence`] and [`evidence_probe_request`]).
//! The analysis state and its diagnostic say that no definition or dispatch resolution
//! was performed, while items, page, coverage and billing are the ones this scan really
//! produced — the candidates ride the ordinary publishing path and are never a special
//! case of it.

mod bootstrap;
mod code;
mod metadata;
mod resource;

use crate::artifact::{ArtifactKind, ArtifactSnapshot, PhysicalEntry, budget_dimension_code};
use crate::budget::{Budget, CountedBudgetDimension, UsageSnapshot};
use crate::error::{Error, Result};
use crate::model::{
    ByteSpan, ClassBytesId, ContainerId, ContainerOrigin, Coverage, CoverageDimension,
    CoverageRange, CoverageState, Diagnostic, DiagnosticSeverity, Digest, ExecutionReport,
    Location, PhysicalClassLocation, PhysicalDefinitionId, PhysicalVariant, Provenance, SnapshotId,
    TerminationReason,
};
use crate::query::{
    ConsumerKind, QUERY_ENGINE_SCHEMA, QueryBoundary, QueryCoverage, QueryCursor, QueryPage,
    QueryRelation, QueryRequest, XrefCertainty, XrefItem, cursor_digest, not_requested_coverage,
    unsupported_categories,
};
use crate::view::PhysicalScope;

/// Relations whose definition/dispatch resolution P1 does not perform but whose raw
/// constant-pool candidates are still answerable facts.
///
/// The candidate probe compares the raw bytes the class file stores, so a pool entry
/// whose owner differs from the queried owner is not a candidate at all. Expanding an
/// owner through a hierarchy and turning a candidate into a reference is the P2
/// resolver's job behind these relations, and that boundary is exactly why the items
/// stay `XrefDerivation::ConstantPoolCandidate` with no consumer category.
fn keeps_pool_evidence(relation: QueryRelation) -> bool {
    matches!(
        relation,
        QueryRelation::ReferencesDefinition | QueryRelation::MayDispatchTo
    )
}

/// The request the consumer sub-scans see.
///
/// For a relation that keeps raw pool evidence the scan *is* the raw constant-pool
/// probe: `code::scan` answers it through the same X0 path with the same target, and
/// the producers that answer only structural consumers (`resource`, `metadata`,
/// `bootstrap`) return early on it, so the whole scan reduces to pool candidates.
/// Every other field, the caller's `cursor` included, is untouched: identity, order,
/// billing and the cursor binding stay the caller's request.
fn evidence_probe_request(request: &QueryRequest) -> QueryRequest {
    let mut probe = request.clone();
    if keeps_pool_evidence(request.relation) {
        probe.relation = QueryRelation::ConstantPoolContains;
    }
    probe
}

/// Query-scan output consumed by [`crate::query::execute`].
pub(crate) struct ScanResult {
    pub(crate) items: Vec<XrefItem>,
    pub(crate) page: QueryPage,
    pub(crate) coverage: QueryCoverage,
    pub(crate) execution: ExecutionReport,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

/// Runs the XRef scan for one validated request.
pub(crate) fn scan(
    snapshot: &ArtifactSnapshot,
    request: &QueryRequest,
    budget: &mut Budget,
) -> Result<ScanResult> {
    // A request that names no consumer category asks for no scan at all: nothing is
    // enumerated, read or billed, and coverage stays `not_requested`.
    if request.consumers.kinds.is_empty() {
        return Ok(ScanResult {
            items: Vec::new(),
            page: QueryPage {
                has_more: false,
                returned_items: 0,
                cursor: None,
            },
            coverage: QueryCoverage {
                dimensions: not_requested_coverage(),
                consumer_schema: request.consumers.clone(),
                unsupported_categories: Vec::new(),
                scanned_items: 0,
                unknown_candidates: 0,
            },
            execution: ExecutionReport::Complete {
                usage: budget.usage(),
            },
            diagnostics: Vec::new(),
        });
    }

    let provider = ProviderScan::collect(snapshot, request, budget)?;
    let unsupported = unsupported_categories(&request.consumers);
    // The sub-scans see the evidence probe; this `scan` reports everything under the
    // caller's request.
    let probe = evidence_probe_request(request);
    let mut ctx = ScanContext::new(snapshot, &probe, budget);

    // The provider's own terminal state, if any, already limits this run.
    let mut issue: Option<ExecutionReport> = match &provider.execution {
        ExecutionReport::Complete { .. } => None,
        other => Some(other.clone()),
    };
    // Provider diagnostics were billed by the provider itself (P0 discipline: each
    // result counts once), so they are carried over without a second charge.
    let mut diagnostics: Vec<Diagnostic> = provider.diagnostics.clone();
    let mut items: Vec<XrefItem> = Vec::new();
    let mut examined: Vec<ExaminedContainer> = Vec::new();
    let mut boundary: Option<QueryBoundary> = None;
    let mut pending: Option<QueryBoundary> = request
        .cursor
        .as_ref()
        .map(|cursor| cursor.boundary.clone());
    let mut skip_items = 0_u64;
    let mut stopped_early = false;
    let mut published = 0_u64;
    let mut scanned_items = 0_u64;
    let mut unknown_candidates = 0_u64;
    let mut standalone_examined = 0_u64;

    'units: for unit in &provider.units {
        // Units before the cursor boundary were published by an earlier page: they
        // are neither re-read nor re-billed here.
        if let Some(resume) = &pending {
            if resume.container != *unit.origin() || resume.ordinal != unit.ordinal() {
                continue;
            }
            skip_items = resume.item_index;
            pending = None;
        }
        // Cooperative interruption is checked before any unit work, so a cancelled
        // or exhausted request never reports more than the published prefix.
        if let Err(error) = ctx.budget().poll() {
            issue = merge_issue(issue, terminal_execution(&error, ctx.usage()));
            diagnostics.push(terminal_diagnostic(&error, Some(unit)));
            break 'units;
        }
        // A full page stops the scan instead of looking for the next item: the page
        // limit must bound the work, so `has_more` stays a conservative "stopped
        // before the end of the range" and a continuation may still be empty.
        if request.max_items != 0 && published >= request.max_items {
            stopped_early = true;
            break 'units;
        }

        ctx.begin_unit();
        let mut unit_items = Vec::new();
        let scan_error = scan_unit(&mut ctx, unit, &mut unit_items).err();
        // The probe answered as a raw constant-pool probe, but the caller asked a relation
        // P1 does not resolve. Each item keeps the caller's relation and stays an
        // unanalysed candidate: derivation, consumer, operation, certainty and evidence
        // are the facts the probe found, and nothing here expands or resolves them.
        if keeps_pool_evidence(request.relation) {
            for item in &mut unit_items {
                item.relation = request.relation;
            }
        }
        record_examined(&mut examined, unit);
        if let Some(length) = ctx.materialized_length(unit) {
            standalone_examined = length;
        }

        for (index, item) in unit_items.into_iter().enumerate() {
            let unknown = item.certainty == XrefCertainty::Unknown;
            if (index as u64) < skip_items {
                // Replayed prefix of a continuation: already published and billed, so
                // it counts as scanned but is neither published nor charged again.
                scanned_items += 1;
                unknown_candidates += u64::from(unknown);
                continue;
            }
            if request.max_items != 0 && published >= request.max_items {
                stopped_early = true;
                break 'units;
            }
            if let Err(error) = ctx.charge_result_item() {
                issue = merge_issue(issue, terminal_execution(&error, ctx.usage()));
                diagnostics.push(terminal_diagnostic(&error, Some(unit)));
                break 'units;
            }
            published += 1;
            scanned_items += 1;
            unknown_candidates += u64::from(unknown);
            boundary = Some(QueryBoundary {
                container: unit.origin().clone(),
                ordinal: unit.ordinal(),
                item_index: index as u64 + 1,
            });
            items.push(item);
        }
        skip_items = 0;

        let unit_diagnostics = ctx.take_diagnostics();
        for diagnostic in unit_diagnostics {
            if let Err(error) = ctx.charge_result_item() {
                issue = merge_issue(issue, terminal_execution(&error, ctx.usage()));
                diagnostics.push(terminal_diagnostic(&error, Some(unit)));
                break 'units;
            }
            diagnostics.push(diagnostic);
        }

        if let Some(error) = scan_error {
            issue = merge_issue(issue, terminal_execution(&error, ctx.usage()));
            diagnostics.push(terminal_diagnostic(&error, Some(unit)));
            break 'units;
        }
    }

    // A boundary that does not exist in a complete unit stream cannot describe this
    // snapshot/view/relation/schema binding. An incomplete provider may simply not
    // have reached it yet, and reports a partial status instead.
    if pending.is_some() && matches!(provider.execution, ExecutionReport::Complete { .. }) {
        return Err(Error::invalid_input(
            "query_cursor_mismatch",
            "cursor boundary does not exist in this snapshot/view/relation/schema binding",
        ));
    }

    let has_more = stopped_early || issue.is_some();
    let cursor = if has_more {
        boundary
            .map(|boundary| build_cursor(snapshot.id(), request, boundary))
            .transpose()?
    } else {
        None
    };
    let (scanned_ranges, skipped_ranges) = provider.coverage_parts(&examined, standalone_examined);
    // `complete_within_schema` needs both: no interruption or page limit, and no
    // known range left unexamined (a skip can also come from a continued page or
    // from a schema whose producers never read a standalone CLASS root).
    let complete =
        issue.is_none() && !stopped_early && unsupported.is_empty() && skipped_ranges.is_empty();
    let execution = with_usage(
        issue.unwrap_or(ExecutionReport::Complete {
            usage: budget.usage(),
        }),
        budget.usage(),
    );
    let coverage = QueryCoverage {
        dimensions: Coverage {
            artifact_structural: CoverageDimension {
                state: if complete {
                    CoverageState::CompleteWithinSchema
                } else {
                    CoverageState::Partial
                },
                scanned: scanned_ranges,
                skipped: skipped_ranges,
                uninterpreted_extensions: Vec::new(),
            },
            runtime_resolution: CoverageDimension::not_requested(),
            dynamic_analysis: CoverageDimension::not_requested(),
        },
        consumer_schema: request.consumers.clone(),
        unsupported_categories: unsupported,
        scanned_items,
        unknown_candidates,
    };
    Ok(ScanResult {
        page: QueryPage {
            has_more,
            returned_items: u64::try_from(items.len()).map_err(|_| {
                Error::invalid_input("query_size_overflow", "page item count does not fit u64")
            })?,
            cursor,
        },
        items,
        coverage,
        execution,
        diagnostics,
    })
}

/// Fixed consumer sub-scan order inside one unit.
///
/// The order defines the item order of a unit, so a continuation replays the same
/// sequence. Each sub-scan owns its own file and its own budget dimensions; this
/// function only fixes the order in which they answer one unit.
fn scan_unit(ctx: &mut ScanContext<'_>, unit: &ScanUnit, out: &mut Vec<XrefItem>) -> Result<()> {
    resource::scan(ctx, unit, out)?;
    code::scan(ctx, unit, out)?;
    metadata::scan(ctx, unit, out)?;
    bootstrap::scan(ctx, unit, out)
}

/// One scan unit: a container entry, or the standalone CLASS snapshot root.
///
/// A standalone CLASS is one unit with no synthetic entry: its bytes are the
/// snapshot root, and its ordinal is `0` for boundary bookkeeping only.
pub(super) struct ScanUnit {
    origin: ContainerOrigin,
    ordinal: u64,
    kind: UnitKind,
}

pub(super) enum UnitKind {
    Entry(PhysicalEntry),
    StandaloneRoot,
}

impl ScanUnit {
    pub(super) fn origin(&self) -> &ContainerOrigin {
        &self.origin
    }

    pub(super) fn ordinal(&self) -> u64 {
        self.ordinal
    }

    pub(super) fn entry(&self) -> Option<&PhysicalEntry> {
        match &self.kind {
            UnitKind::Entry(entry) => Some(entry),
            UnitKind::StandaloneRoot => None,
        }
    }

    /// Physical definition identity of this unit's class bytes.
    ///
    /// The variant is a syntactic physical label of the container-relative raw
    /// path; it is not the multi-release selection contract and claims nothing
    /// about activation or validity.
    pub(super) fn definition(&self, class_bytes: ClassBytesId) -> PhysicalDefinitionId {
        match &self.kind {
            UnitKind::Entry(entry) => PhysicalDefinitionId {
                location: PhysicalClassLocation::ArchiveEntry {
                    entry: entry.id.clone(),
                },
                class_bytes,
                variant: entry_variant(&entry.id.raw_name.0),
            },
            UnitKind::StandaloneRoot => PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: self.origin.snapshot.clone(),
                },
                class_bytes,
                variant: PhysicalVariant::Base,
            },
        }
    }

    /// Resource provenance inside this unit's entry content.
    ///
    /// `None` for the standalone CLASS root, which has no resource entry.
    pub(super) fn resource_provenance(&self, span: ByteSpan) -> Option<Provenance> {
        match &self.kind {
            UnitKind::Entry(entry) => Some(Provenance {
                location: Location::Resource {
                    entry: entry.id.clone(),
                    span,
                },
            }),
            UnitKind::StandaloneRoot => None,
        }
    }
}

/// Bytes of one unit plus the digest the materialization path computed.
pub(super) struct UnitContent {
    pub(super) bytes: Vec<u8>,
    /// Content digest of `bytes`; a classfile sub-scan turns it into a `ClassBytesId`.
    pub(super) digest: Digest,
}

/// Magic every JVM class file starts with (JVMS 4.1).
pub(super) const CLASS_MAGIC: [u8; 4] = [0xca, 0xfe, 0xba, 0xbe];

/// Whether an archive entry's raw name puts it in the physical class scan range.
///
/// This is the one archive candidate rule the `code`, `metadata` and `bootstrap`
/// consumers share: a name that ends in a case-sensitive `.class`, the bare name
/// `.class` included. A directory name ends in `/`, so it never matches. The name only
/// decides which entries this scan physically opens — it proves nothing about JVM
/// loadability, and normalizing its case here would put entries into the scan range
/// that the archive really spells differently.
pub(super) fn is_class_candidate(raw_name: &[u8]) -> bool {
    raw_name.ends_with(b".class")
}

/// Materializes a unit's bytes when the unit can carry a class file.
///
/// * An archive entry outside the candidate range is never read and never billed: a
///   plain resource cannot fail a query whose class entries are readable.
/// * An archive candidate is read and must start with the class magic. Bytes that do
///   not are a damaged candidate, so the unit ends with a
///   `query_class_candidate_malformed` error naming this entry; `scan` turns that into
///   the located diagnostic and the non-complete execution, and the items published
///   before it stay. Reporting "no fact" here would silently turn a truncated class
///   into a complete empty miss.
/// * The standalone CLASS root is a class file by construction: P0 only classifies a
///   snapshot as `StandaloneClass` when its bytes start with the magic, and the P0
///   open/root contract owns every other root failure. This path therefore neither
///   applies the candidate rule nor reports damage of its own.
pub(super) fn class_content(
    ctx: &mut ScanContext<'_>,
    unit: &ScanUnit,
) -> Result<Option<UnitContent>> {
    let Some(entry) = unit.entry() else {
        return ctx.read_unit(unit).map(Some);
    };
    if !is_class_candidate(&entry.id.raw_name.0) {
        return Ok(None);
    }
    let content = ctx.read_unit(unit)?;
    if !content.bytes.starts_with(&CLASS_MAGIC) {
        return Err(Error::invalid_input(
            MALFORMED_CLASS_CANDIDATE_CODE,
            class_candidate_message(entry, &content.bytes),
        ));
    }
    Ok(Some(content))
}

/// Diagnostic code of a candidate whose bytes cannot be a class file.
const MALFORMED_CLASS_CANDIDATE_CODE: &str = "query_class_candidate_malformed";

/// Damage message of a candidate whose bytes cannot be a class file.
///
/// The message names the entry, the byte length this scan really observed and the
/// magic it expected, so a candidate shorter than the magic is distinguishable from one
/// whose four bytes are something else.
fn class_candidate_message(entry: &PhysicalEntry, bytes: &[u8]) -> String {
    let name = escaped_raw_name(&entry.id.raw_name.0);
    let expected = hex_bytes(&CLASS_MAGIC);
    if bytes.len() < CLASS_MAGIC.len() {
        format!(
            "class candidate entry \"{name}\" holds {} byte(s), shorter than the {expected} class magic",
            bytes.len()
        )
    } else {
        format!(
            "class candidate entry \"{name}\" holds {} byte(s) starting with {}, not the {expected} class magic",
            bytes.len(),
            hex_bytes(&bytes[..CLASS_MAGIC.len()])
        )
    }
}

/// Readable, unambiguous rendering of a raw archive name for a diagnostic message.
///
/// A name is raw bytes, so anything outside printable ASCII is escaped rather than
/// replaced by a lossy character.
fn escaped_raw_name(raw: &[u8]) -> String {
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

/// Space-separated upper-case hex, for the magic a message compares against.
fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// State shared by the consumer sub-scans during one request.
pub(super) struct ScanContext<'a> {
    snapshot: &'a ArtifactSnapshot,
    request: &'a QueryRequest,
    budget: &'a mut Budget,
    diagnostics: Vec<Diagnostic>,
    materialized: Vec<MaterializedUnit>,
}

impl<'a> ScanContext<'a> {
    fn new(
        snapshot: &'a ArtifactSnapshot,
        request: &'a QueryRequest,
        budget: &'a mut Budget,
    ) -> Self {
        Self {
            snapshot,
            request,
            budget,
            diagnostics: Vec::new(),
            materialized: Vec::new(),
        }
    }

    pub(super) fn request(&self) -> &QueryRequest {
        self.request
    }

    /// Whether the request asks for this consumer category.
    pub(super) fn wants(&self, kind: ConsumerKind) -> bool {
        self.request.consumers.kinds.contains(&kind)
    }

    /// Budget for the decode dimensions a sub-scan really consumes
    /// (`ClassBytes`, `AttributeBytes`, `CodeBytes`, reads). Sub-scans never charge
    /// `ResultItems`.
    pub(super) fn budget(&mut self) -> &mut Budget {
        self.budget
    }

    /// Materializes one unit's bytes.
    ///
    /// ZIP entries are read through the crate-private P0 path that charges
    /// `ReadBytes`/`EntryBytes` and no `OutputBytes`. The standalone CLASS root is
    /// only exposed through P0 `root_bytes`, which charges read and output bytes;
    /// no `OutputBytes` is charged for query items themselves.
    pub(super) fn read_unit(&mut self, unit: &ScanUnit) -> Result<UnitContent> {
        let content = match &unit.kind {
            UnitKind::Entry(entry) => {
                let materialized = self.snapshot.read_entry_internal(entry, self.budget)?;
                UnitContent {
                    bytes: materialized.bytes,
                    digest: materialized.content_digest,
                }
            }
            UnitKind::StandaloneRoot => {
                let bytes = self.snapshot.root_bytes(self.budget)?;
                let digest = Digest(blake3::hash(&bytes).to_hex().to_string());
                UnitContent { bytes, digest }
            }
        };
        self.materialized.push(MaterializedUnit {
            container: unit.origin().current_container().clone(),
            ordinal: unit.ordinal(),
            length: to_u64(content.bytes.len())?,
        });
        Ok(content)
    }

    /// Queues a domain diagnostic for this request; the scan bills it once.
    pub(super) fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    fn take_diagnostics(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.diagnostics)
    }

    fn begin_unit(&mut self) {
        self.materialized.clear();
    }

    fn materialized_length(&self, unit: &ScanUnit) -> Option<u64> {
        self.materialized
            .iter()
            .find(|item| {
                item.ordinal == unit.ordinal()
                    && item.container == *unit.origin().current_container()
            })
            .map(|item| item.length)
    }

    fn usage(&self) -> UsageSnapshot {
        self.budget.usage()
    }

    fn charge_result_item(&mut self) -> Result<()> {
        self.budget.charge(CountedBudgetDimension::ResultItems, 1)
    }
}

struct MaterializedUnit {
    container: ContainerId,
    ordinal: u64,
    length: u64,
}

struct ExaminedContainer {
    container: ContainerId,
    first: u64,
    last_exclusive: u64,
}

/// Provider-derived unit stream plus the physical facts of this request.
struct ProviderScan {
    units: Vec<ScanUnit>,
    containers: Vec<ProviderContainer>,
    scanned: Vec<CoverageRange>,
    skipped: Vec<CoverageRange>,
    diagnostics: Vec<Diagnostic>,
    execution: ExecutionReport,
    standalone_class_bytes: Option<u64>,
}

struct ProviderContainer {
    origin: ContainerOrigin,
    known_entries: u64,
}

impl ProviderScan {
    fn collect(
        snapshot: &ArtifactSnapshot,
        request: &QueryRequest,
        budget: &mut Budget,
    ) -> Result<Self> {
        match (snapshot.kind(), &request.physical.scope) {
            (ArtifactKind::StandaloneClass, PhysicalScope::SnapshotAll) => Ok(Self {
                units: vec![ScanUnit {
                    origin: root_origin(snapshot.id()),
                    ordinal: 0,
                    kind: UnitKind::StandaloneRoot,
                }],
                containers: Vec::new(),
                scanned: Vec::new(),
                skipped: Vec::new(),
                diagnostics: Vec::new(),
                execution: ExecutionReport::Complete {
                    usage: budget.usage(),
                },
                standalone_class_bytes: Some(snapshot.len()),
            }),
            (ArtifactKind::Zip, PhysicalScope::SnapshotAll) => {
                let report = snapshot.enumerate(budget)?;
                let origin = root_origin(&report.snapshot);
                let known_entries = match covered_end(&report.coverage.artifact_structural) {
                    Some(end) => end,
                    None => to_u64(report.entries.len())?,
                };
                let units = report
                    .entries
                    .iter()
                    .map(|entry| ScanUnit {
                        origin: origin.clone(),
                        ordinal: entry.id.ordinal,
                        kind: UnitKind::Entry(entry.clone()),
                    })
                    .collect();
                Ok(Self {
                    units,
                    containers: vec![ProviderContainer {
                        origin,
                        known_entries,
                    }],
                    scanned: report.coverage.artifact_structural.scanned,
                    skipped: report.coverage.artifact_structural.skipped,
                    diagnostics: report.diagnostics,
                    execution: report.execution,
                    standalone_class_bytes: None,
                })
            }
            // The tree provider rejects a non-ZIP snapshot itself; its error is
            // propagated instead of being rewritten here.
            (_, PhysicalScope::ArtifactTree { .. }) => {
                let report = snapshot.enumerate_artifact_tree(budget)?;
                let mut units = Vec::new();
                let mut containers = Vec::new();
                for container in &report.containers {
                    for entry in &container.entries {
                        units.push(ScanUnit {
                            origin: container.origin.clone(),
                            ordinal: entry.id.ordinal,
                            kind: UnitKind::Entry(entry.clone()),
                        });
                    }
                    let known_entries = match covered_end(&container.coverage.artifact_structural) {
                        Some(end) => end,
                        None => to_u64(container.entries.len())?,
                    };
                    containers.push(ProviderContainer {
                        origin: container.origin.clone(),
                        known_entries,
                    });
                }
                Ok(Self {
                    units,
                    containers,
                    scanned: report.coverage.artifact_structural.scanned,
                    skipped: report.coverage.artifact_structural.skipped,
                    diagnostics: report.diagnostics,
                    execution: report.execution,
                    standalone_class_bytes: None,
                })
            }
        }
    }

    /// Provider ranges plus this pass's own ordinal or byte coverage.
    ///
    /// Provider ranges are carried verbatim (they were already billed by the
    /// provider); the `xref_scan_entries` / `xref_scan_bytes` ranges describe what
    /// this invocation examined, so a page limit, a continuation or a schema that
    /// never reads a unit's bytes shows up as a skipped range instead of a claim.
    fn coverage_parts(
        &self,
        examined: &[ExaminedContainer],
        standalone_examined: u64,
    ) -> (Vec<CoverageRange>, Vec<CoverageRange>) {
        let mut scanned = self.scanned.clone();
        let mut skipped = self.skipped.clone();
        for container in &self.containers {
            let id = &container.origin.current_container().0;
            let label = format!("container:{id}:xref_scan_entries");
            let known = container.known_entries;
            match examined
                .iter()
                .find(|entry| entry.container.0 == *id)
                .map(|entry| (entry.first.min(known), entry.last_exclusive.min(known)))
            {
                Some((first, last_exclusive)) => {
                    if first > 0 {
                        skipped.push(CoverageRange {
                            label: label.clone(),
                            start: 0,
                            end: first,
                        });
                    }
                    if first < last_exclusive {
                        scanned.push(CoverageRange {
                            label: label.clone(),
                            start: first,
                            end: last_exclusive,
                        });
                    }
                    if last_exclusive < known {
                        skipped.push(CoverageRange {
                            label,
                            start: last_exclusive,
                            end: known,
                        });
                    }
                }
                None => {
                    if known > 0 {
                        skipped.push(CoverageRange {
                            label,
                            start: 0,
                            end: known,
                        });
                    }
                }
            }
        }
        if let Some(total) = self.standalone_class_bytes {
            let label = "standalone_class:xref_scan_bytes".to_string();
            let examined = standalone_examined.min(total);
            if examined > 0 {
                scanned.push(CoverageRange {
                    label: label.clone(),
                    start: 0,
                    end: examined,
                });
            }
            if examined < total {
                skipped.push(CoverageRange {
                    label,
                    start: examined,
                    end: total,
                });
            }
        }
        (scanned, skipped)
    }
}

fn build_cursor(
    snapshot: &SnapshotId,
    request: &QueryRequest,
    boundary: QueryBoundary,
) -> Result<QueryCursor> {
    let digest = cursor_digest(
        QUERY_ENGINE_SCHEMA,
        snapshot,
        &request.physical,
        request.relation,
        &request.target,
        &request.consumers,
        &boundary,
    )?;
    Ok(QueryCursor {
        engine_schema: QUERY_ENGINE_SCHEMA,
        snapshot: snapshot.clone(),
        physical: request.physical.clone(),
        relation: request.relation,
        target: request.target.clone(),
        consumers: request.consumers.clone(),
        boundary,
        digest,
    })
}

fn record_examined(examined: &mut Vec<ExaminedContainer>, unit: &ScanUnit) {
    let container = unit.origin().current_container().clone();
    let end = unit.ordinal().saturating_add(1);
    match examined
        .iter_mut()
        .find(|entry| entry.container == container)
    {
        Some(entry) => {
            entry.first = entry.first.min(unit.ordinal());
            entry.last_exclusive = entry.last_exclusive.max(end);
        }
        None => examined.push(ExaminedContainer {
            container,
            first: unit.ordinal(),
            last_exclusive: end,
        }),
    }
}

fn covered_end(dimension: &CoverageDimension) -> Option<u64> {
    dimension
        .scanned
        .iter()
        .chain(dimension.skipped.iter())
        .map(|range| range.end)
        .max()
}

fn root_origin(snapshot: &SnapshotId) -> ContainerOrigin {
    ContainerOrigin {
        snapshot: snapshot.clone(),
        root_container: ContainerId("root".into()),
        steps: Vec::new(),
    }
}

/// Syntactic physical-variant label of a container-relative raw path.
fn entry_variant(raw_name: &[u8]) -> PhysicalVariant {
    const PREFIX: &[u8] = b"META-INF/versions/";
    let Some(rest) = raw_name.strip_prefix(PREFIX) else {
        return PhysicalVariant::Base;
    };
    let Some(slash) = rest.iter().position(|byte| *byte == b'/') else {
        return PhysicalVariant::Base;
    };
    let (release, logical) = rest.split_at(slash);
    let release_unlabelled = PhysicalVariant::Other {
        label: "multi_release_version_unlabelled".into(),
    };
    if release.is_empty() || logical.len() <= 1 || (release.len() > 1 && release[0] == b'0') {
        return release_unlabelled;
    }
    if !release.iter().all(u8::is_ascii_digit) {
        return release_unlabelled;
    }
    let mut version = 0_u64;
    for digit in release {
        version = match version
            .checked_mul(10)
            .and_then(|value| value.checked_add(u64::from(digit - b'0')))
        {
            Some(value) => value,
            None => return release_unlabelled,
        };
    }
    match u16::try_from(version) {
        Ok(version) => PhysicalVariant::MultiRelease { version },
        Err(_) => release_unlabelled,
    }
}

fn merge_issue(
    current: Option<ExecutionReport>,
    incoming: ExecutionReport,
) -> Option<ExecutionReport> {
    match current {
        Some(current) if priority_of(&current) >= priority_of(&incoming) => Some(current),
        _ => Some(incoming),
    }
}

/// Terminal-state priority, mirroring the P0 provider order.
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

fn terminal_execution(error: &Error, usage: UsageSnapshot) -> ExecutionReport {
    match error {
        Error::Cancelled { .. } => ExecutionReport::Cancelled { usage },
        Error::BudgetExceeded { dimension, .. } => ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: *dimension,
            },
            usage,
        },
        Error::Unsupported { code, .. } => ExecutionReport::Partial {
            reason: TerminationReason::Unsupported { code: code.clone() },
            usage,
        },
        Error::InvalidInput { code, .. }
        | Error::Io {
            operation: code, ..
        } => ExecutionReport::Failed {
            reason: TerminationReason::Error { code: code.clone() },
            usage,
        },
    }
}

fn with_usage(execution: ExecutionReport, usage: UsageSnapshot) -> ExecutionReport {
    match execution {
        ExecutionReport::Complete { .. } => ExecutionReport::Complete { usage },
        ExecutionReport::Partial { reason, .. } => ExecutionReport::Partial { reason, usage },
        ExecutionReport::Cancelled { .. } => ExecutionReport::Cancelled { usage },
        ExecutionReport::Failed { reason, .. } => ExecutionReport::Failed { reason, usage },
    }
}

/// Terminal diagnostics are control metadata: they explain a stop, so they are
/// returned even when `ResultItems` is exhausted and are never charged.
fn terminal_diagnostic(error: &Error, unit: Option<&ScanUnit>) -> Diagnostic {
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
        provenance: unit.and_then(|unit| {
            unit.entry().map(|entry| Provenance {
                location: Location::Entry {
                    id: entry.id.clone(),
                    span: ByteSpan::new(0, entry.uncompressed_size),
                },
            })
        }),
    }
}

/// Shared size conversion for the scan modules.
pub(super) fn to_u64(value: usize) -> Result<u64> {
    u64::try_from(value).map_err(|_| {
        Error::invalid_input(
            "query_size_overflow",
            "byte or entry count does not fit u64",
        )
    })
}
