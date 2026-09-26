//! XRef scan orchestration: the physical walk, the units it yields, result billing,
//! page and coverage.
//!
//! The scan is a *pull*, not a collection. Two walks run together, and neither of them
//! builds the range it has not reached:
//!
//! 1. [`UnitStream`] yields one unit at a time — the standalone CLASS root, the entries a
//!    request's schema needs in the scope's own physical order. Which stream a request gets
//!    is the request's own decision, and the entries are the reason there are two of them:
//!    `resource` is the only consumer that reads an entry which is not a class, so a request
//!    that names it walks *every* entry of every container through the reader's incremental
//!    entry cursor, while every other schema can only produce items from class candidates
//!    and walks those through the reader's incremental scope cursor. Both descend into a
//!    nested container only when the scan pulls through the entry that holds it, so nothing
//!    behind the page's stop is read, expanded or charged — and a whole-snapshot class-only
//!    request keeps the provider's own enumeration of the one container that scope *is*,
//!    which is the same single directory product either walk would have to build.
//! 2. [`UnitScan`] runs one unit as a program of steps — `resource`, then the code producer
//!    (the raw pool probe, or the instruction stream of one method after another), then
//!    `metadata`, then `bootstrap` — and stops *between* steps and *between* the items of
//!    one step. Decoding that has not started is neither billed nor materialized, and the
//!    items a step produced after the page filled are dropped, not published.
//!
//! The stop is recorded as a [`QueryBoundary`]: the unit and the step the next item would
//! come from, plus the number of that step's items already published. A continuation
//! re-derives the same program, verifies the position against the unit's own class file
//! (`QueryPosition::Method` carries the member's raw name and descriptor), skips the
//! published prefix of that one step and continues — so it never re-reads the units before
//! the boundary, never re-runs an earlier step of the boundary unit, and re-decodes the
//! boundary method alone when the page was cut inside one.
//!
//! This module is the single billing point for query results: every item and every
//! domain diagnostic that enters the report is charged one `ResultItems` before it
//! is published, and a failed charge keeps the reliable prefix and stops without
//! publishing an unbilled item. Sub-scan modules must not charge `ResultItems`
//! themselves, must not add or change public schema types, and must fill only
//! their own file.
//!
//! The scan reports what it really did: `execution` mirrors the walk and scan
//! outcome (`Complete`/`Partial`/`Cancelled`/`Failed`), coverage carries the ranges
//! the invocation examined, and a page limit or an interruption never turns into
//! `Complete`. A range the walk did not reach stays *unknown*: it is not named as
//! examined, and `Partial` says the scope was not covered — never the empty answer.
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
// The resource consumer owns what a `META-INF/services` entry is for this engine. The plugin
// plane (P4 3.1) reads the same configuration through the same matcher rather than restating the
// format, so the two planes cannot disagree about which entries are configurations.
pub(crate) mod resource;

use crate::query::{
    ConsumerKind, ConsumerSchema, LiteralValue, QUERY_ENGINE_SCHEMA, QueryBoundary, QueryCoverage,
    QueryCursor, QueryPage, QueryPosition, QueryRelation, QueryRequest, QueryTarget, XrefCertainty,
    XrefItem, XrefTarget, cursor_digest, not_requested_coverage, unsupported_categories,
};
use code::CodeOutcome;
use jarde_reader::artifact::{
    ArtifactKind, ArtifactSnapshot, PhysicalEntry, budget_dimension_code,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension, UsageSnapshot};
use jarde_reader::entry_cursor::EntryCursor;
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{
    ByteSpan, ClassBytesId, ContainerId, ContainerOrigin, Coverage, CoverageDimension,
    CoverageRange, CoverageState, Diagnostic, DiagnosticSeverity, Digest, ExecutionReport,
    JvmBytes, Location, PhysicalClassLocation, PhysicalDefinitionId, PhysicalEntryId,
    PhysicalVariant, Provenance, SnapshotId, SymbolRef, TerminationReason,
    physical_variant_for_path,
};
use jarde_reader::scope_cursor::ScopeCursor;
use jarde_reader::view::{PhysicalScope, PhysicalView};

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

/// The candidate shapes a caller outside this crate may ask [`scan_candidates`] for.
///
/// Candidate scans support member-shape matching, signature-polymorphic method matching and
/// exact raw owner matching. [`CandidateRule::Exact`] remains query-internal semantics: it
/// names one complete target to compare against rather than a declaration or owner to look for.
#[derive(Clone, Debug)]
pub enum CandidateFilter {
    /// Any class, method or field symbol whose raw class owner bytes equal this value.
    ///
    /// A class symbol carries the class name itself; member symbols carry their declaring
    /// owner. This is an exact byte comparison and performs no hierarchy resolution.
    Owner { owner: JvmBytes },
    /// The raw shape of one member reference: a `SymbolRef::Method`/`SymbolRef::Field` whose
    /// name and descriptor bytes are equal.
    ///
    /// The owner deliberately does not take part. `Sub.foo` may resolve to the declaration
    /// `Base.foo`, so filtering by the declaration's own owner would drop exactly the use
    /// sites a declaration-reference query exists to find; the owner of each candidate is
    /// read from the item instead, and comparing the two is the caller's resolution step.
    MemberShape {
        name: JvmBytes,
        descriptor: JvmBytes,
    },
    /// The shape of one signature-polymorphic method: a `SymbolRef::Method` whose owner and
    /// name bytes are equal, whatever descriptor the site spells.
    ///
    /// JVMS 2.9 makes the call site's descriptor the site's own choice — `MethodHandle.invoke`
    /// and `invokeExact` are matched by name at resolution — so comparing descriptors would
    /// turn a real, resolvable use site into "not even a candidate": a silent false negative
    /// under a report that claims complete coverage. The owner *is* part of the identity here,
    /// which is not in tension with `MemberShape` ignoring it: signature polymorphism is
    /// defined for exactly one owner, so a site on any other owner does not answer this shape
    /// at all.
    SignaturePolymorphic { owner: JvmBytes, name: JvmBytes },
}

impl From<CandidateFilter> for CandidateRule {
    /// The scanner's own spelling of a caller's rule.
    ///
    /// The two variants carry the same dimensions on both sides, so this is a move rather than
    /// a translation: there is no second matching rule that could drift from the first, and a
    /// rule the caller cannot state ([`CandidateRule::Exact`]) has no public spelling at all.
    fn from(filter: CandidateFilter) -> Self {
        match filter {
            CandidateFilter::Owner { owner } => Self::Owner { owner },
            CandidateFilter::MemberShape { name, descriptor } => {
                Self::MemberShape { name, descriptor }
            }
            CandidateFilter::SignaturePolymorphic { owner, name } => {
                Self::SignaturePolymorphic { owner, name }
            }
        }
    }
}

/// How a sub-scan decides whether one candidate it found answers the scan.
///
/// This is the one place that decision lives, so every consumer sub-scan applies the same
/// rule and none of them compares targets on its own: [`CandidateRule::Exact`] is the
/// behaviour `Engine::query` has always had (exact equality on the raw bytes the request
/// names), while the three candidate rules match member shape, signature-polymorphic methods,
/// or one exact raw owner. [`CandidateFilter`] is the caller's half of those rules; `Exact`
/// has no public spelling.
#[derive(Clone, Debug)]
pub(crate) enum CandidateRule {
    /// The request's own target: the candidate must carry the same raw bytes.
    Exact(QueryTarget),
    /// A class symbol named by these raw bytes, or a method/field symbol with these raw
    /// owner bytes.
    Owner { owner: JvmBytes },
    /// The raw shape of one member reference: a `SymbolRef::Method`/`SymbolRef::Field` whose
    /// name and descriptor bytes are equal.
    ///
    /// The owner deliberately does not take part. `Sub.foo` may resolve to the declaration
    /// `Base.foo`, so filtering by the declaration's own owner would drop exactly the use
    /// sites a declaration-reference query exists to find; the owner of each candidate is
    /// read from the item instead, and comparing the two is the caller's resolution step.
    MemberShape {
        name: JvmBytes,
        descriptor: JvmBytes,
    },
    /// The shape of one signature-polymorphic method: a `SymbolRef::Method` whose owner and
    /// name bytes are equal, whatever descriptor the site spells.
    ///
    /// JVMS 2.9 makes the call site's descriptor the site's own choice — `MethodHandle.invoke`
    /// and `invokeExact` are matched by name at resolution — so comparing descriptors would
    /// turn a real, resolvable use site into "not even a candidate": a silent false negative
    /// under a report that claims complete coverage. The owner *is* part of the identity here,
    /// which is not in tension with `MemberShape` ignoring it: signature polymorphism is
    /// defined for exactly one owner, so a site on any other owner does not answer this shape
    /// at all.
    SignaturePolymorphic { owner: JvmBytes, name: JvmBytes },
}

impl CandidateRule {
    /// The target the scan's own request carries.
    ///
    /// `Exact` is the caller's target. A shape filter names no target: the dimensions it
    /// leaves out (an owner, a descriptor) are exactly the ones it does not compare, so there
    /// is no complete reference symbol to state. The scan's request still carries one because
    /// `QueryRequest` states a target, and the resource sub-scan is the only consumer that
    /// still compares against it — a resource fact is a class symbol or a literal, and neither
    /// can equal the member symbol stated here — so this value is never published and never
    /// matched.
    fn request_target(&self) -> QueryTarget {
        match self {
            Self::Exact(target) => target.clone(),
            Self::Owner { owner } => QueryTarget::Symbol {
                value: SymbolRef::Class {
                    owner: owner.clone(),
                },
            },
            Self::MemberShape { name, descriptor } => QueryTarget::Symbol {
                value: SymbolRef::Method {
                    owner: JvmBytes(Vec::new()),
                    name: name.clone(),
                    descriptor: descriptor.clone(),
                },
            },
            Self::SignaturePolymorphic { owner, name } => QueryTarget::Symbol {
                value: SymbolRef::Method {
                    owner: owner.clone(),
                    name: name.clone(),
                    descriptor: JvmBytes(Vec::new()),
                },
            },
        }
    }
}

/// Raw name and descriptor of one member symbol, or `None` for a symbol that has neither.
///
/// A class symbol names one type, so it carries no member shape and never answers a
/// member-shaped candidate.
fn member_shape(symbol: &SymbolRef) -> Option<(&JvmBytes, &JvmBytes)> {
    match symbol {
        SymbolRef::Field {
            name, descriptor, ..
        }
        | SymbolRef::Method {
            name, descriptor, ..
        } => Some((name, descriptor)),
        SymbolRef::Class { .. } => None,
    }
}

/// Raw owner and name of one *method* symbol, or `None` for any other symbol.
///
/// A field symbol does not answer a method-shaped candidate, and a class symbol carries no
/// member at all.
fn method_owner_and_name(symbol: &SymbolRef) -> Option<(&JvmBytes, &JvmBytes)> {
    match symbol {
        SymbolRef::Method { owner, name, .. } => Some((owner, name)),
        SymbolRef::Field { .. } | SymbolRef::Class { .. } => None,
    }
}

/// Candidate scan output: the items one filter found, and what the scan really did.
///
/// This is a query scan without the page view: the caller states its own item limit and
/// continuation, so the scan publishes every candidate it found and reports whether it
/// stopped before the end of the range instead of issuing a cursor.
pub struct CandidateScan {
    pub items: Vec<XrefItem>,
    /// Whether the scan stopped before the end of the range (an item limit, a budget or
    /// cancellation stop, or a provider that did not finish), so `items` must not be
    /// presented as the whole result.
    pub has_more: bool,
    pub coverage: QueryCoverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
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

    let mut stream = UnitStream::open(snapshot, request, budget)?;
    let unsupported = unsupported_categories(&request.consumers);
    // The sub-scans see the evidence probe; this `scan` reports everything under the
    // caller's request.
    let probe = evidence_probe_request(request);
    let mut ctx = ScanContext::new(
        snapshot,
        &probe,
        CandidateRule::Exact(request.target.clone()),
        budget,
    );
    let pass = run_pass(
        &mut ctx,
        &mut stream,
        request,
        request
            .cursor
            .as_ref()
            .map(|cursor| cursor.boundary.clone()),
    )?;

    // A boundary that no unit of a walk which reached the end of the scope matched cannot
    // describe this snapshot/view/relation/schema binding. A walk that stopped may simply
    // not have reached it, and reports a partial status instead.
    if pass.resume_pending && stream.exhausted() {
        return Err(Error::invalid_input(
            "query_cursor_mismatch",
            "cursor boundary does not exist in this snapshot/view/relation/schema binding",
        ));
    }

    let has_more = pass.stopped_early || pass.issue.is_some();
    let cursor = if has_more {
        pass.boundary
            .map(|boundary| build_cursor(snapshot.id(), request, boundary))
            .transpose()?
    } else {
        None
    };
    let (scanned_ranges, skipped_ranges) =
        stream.coverage_parts(&pass.examined, pass.standalone_examined);
    // `complete_within_schema` needs all of it: no interruption or page limit, no known
    // range left unexamined (a skip can also come from a continued page or from a schema
    // whose producers never read a standalone CLASS root), and a walk that really reached
    // the end of the scope instead of stopping inside it.
    let complete = pass.issue.is_none()
        && !pass.stopped_early
        && unsupported.is_empty()
        && skipped_ranges.is_empty()
        && stream.exhausted();
    let execution = jarde_reader::accounting::with_usage(
        pass.issue.unwrap_or(ExecutionReport::Complete {
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
        scanned_items: pass.scanned_items,
        unknown_candidates: pass.unknown_candidates,
    };
    let mut diagnostics = stream.diagnostics().to_vec();
    diagnostics.extend(pass.diagnostics);
    Ok(ScanResult {
        page: QueryPage {
            has_more,
            returned_items: u64::try_from(pass.items.len()).map_err(|_| {
                Error::invalid_input("query_size_overflow", "page item count does not fit u64")
            })?,
            cursor,
        },
        items: pass.items,
        coverage,
        execution,
        diagnostics,
    })
}

/// Scans one snapshot for every candidate that answers `filter`.
///
/// This is the scan `Engine::query` runs — the same provider enumeration, the same consumer
/// sub-scan order, the same item coordinates, the same billing — read through a candidate
/// filter instead of one exact target, and without the page and the cursor: the caller states
/// its own item limit and continuation, so this entry publishes what it found and reports
/// whether it stopped before the end of the range.
///
/// `max_items` is the caller's own limit and works exactly like `Engine::query`'s page limit:
/// the scan stops before publishing more items, its artifact coverage turns `Partial`, and the
/// execution does not change because of it. The caller publishes the truncation as its own
/// `has_more`; no cursor is issued for it.
pub fn scan_candidates(
    snapshot: &ArtifactSnapshot,
    scope: &PhysicalScope,
    consumers: &ConsumerSchema,
    filter: CandidateFilter,
    max_items: u64,
    budget: &mut Budget,
) -> Result<CandidateScan> {
    let filter = CandidateRule::from(filter);
    let request = QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target: filter.request_target(),
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: scope.clone(),
        },
        consumers: consumers.clone(),
        max_items,
        // The caller owns the continuation too, and a declaration-reference query has none.
        cursor: None,
    };
    if request.consumers.kinds.is_empty() {
        return Ok(CandidateScan {
            items: Vec::new(),
            has_more: false,
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
    let mut stream = UnitStream::open(snapshot, &request, budget)?;
    let unsupported = unsupported_categories(&request.consumers);
    let mut ctx = ScanContext::new(snapshot, &request, filter, budget);
    let pass = run_pass(&mut ctx, &mut stream, &request, None)?;
    let (scanned_ranges, skipped_ranges) =
        stream.coverage_parts(&pass.examined, pass.standalone_examined);
    let complete = pass.issue.is_none()
        && !pass.stopped_early
        && unsupported.is_empty()
        && skipped_ranges.is_empty()
        && stream.exhausted();
    let has_more = pass.stopped_early || pass.issue.is_some();
    let execution = jarde_reader::accounting::with_usage(
        pass.issue.unwrap_or(ExecutionReport::Complete {
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
        scanned_items: pass.scanned_items,
        unknown_candidates: pass.unknown_candidates,
    };
    let mut diagnostics = stream.diagnostics().to_vec();
    diagnostics.extend(pass.diagnostics);
    Ok(CandidateScan {
        has_more,
        items: pass.items,
        coverage,
        execution,
        diagnostics,
    })
}

/// Outcome of one pass over the scope's units.
///
/// The pass reports what the scan really did, in the terms both entries need: the items it
/// published in scan order, whether a limit or a stop kept it from reaching the end of the
/// range, the step position it last published, the containers and bytes it examined, and the
/// stops and diagnostics it produced. The caller decides what those facts mean for its own
/// report (a page and a cursor, or a candidate list).
struct UnitPass {
    items: Vec<XrefItem>,
    /// Position of the next published item: the step it comes from and how many of that
    /// step's items this page already published.
    boundary: Option<QueryBoundary>,
    /// The item limit stopped the pass.
    stopped_early: bool,
    /// A continuation boundary no unit of this stream matched.
    resume_pending: bool,
    /// The stop that ended the pass early, if any.
    issue: Option<ExecutionReport>,
    diagnostics: Vec<Diagnostic>,
    examined: Vec<ExaminedContainer>,
    standalone_examined: u64,
    scanned_items: u64,
    unknown_candidates: u64,
}

/// Why a unit's scan ended before the unit's program did.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UnitEnd {
    /// The page filled: the boundary names the next item.
    Page,
    /// A refused charge or a producer's error ended the pass.
    Terminal,
}

/// Pulls units from the scope's walk and runs each one as a program of steps.
///
/// This is the whole scan both entries share, so the order (the walk's own physical order,
/// then the fixed producer order inside one unit, then each producer's own position order),
/// the billing (one `ResultItems` per published item and per domain diagnostic, charged by
/// this pass alone) and the stop semantics (the prefix published before a refused charge, a
/// limit or an interruption stays) cannot drift between them. `request` carries the item
/// limit and the relation; `resume` is the boundary a continuation resumes from.
fn run_pass(
    ctx: &mut ScanContext<'_>,
    stream: &mut UnitStream<'_>,
    request: &QueryRequest,
    resume: Option<QueryBoundary>,
) -> Result<UnitPass> {
    let mut pass = UnitPass {
        items: Vec::new(),
        boundary: None,
        stopped_early: false,
        resume_pending: false,
        // No stream seeds a terminal state: every unit comes from a walk whose own failures
        // arrive as errors at its pull, and every subtree it could not read is folded in once
        // the pass is over (see `UnitStream::subtree_issue`).
        issue: None,
        diagnostics: Vec::new(),
        examined: Vec::new(),
        standalone_examined: 0,
        scanned_items: 0,
        unknown_candidates: 0,
    };
    let keep_relation = keeps_pool_evidence(request.relation);
    let mut pending = resume;
    'units: loop {
        // A full page stops the scan instead of looking for the next item. It is checked
        // before the walk is asked for another unit, so a page limit leaves the units
        // behind it neither read nor materialized, and `has_more` stays a conservative
        // "stopped before the end of the range".
        if request.max_items != 0 && to_u64(pass.items.len())? >= request.max_items {
            pass.stopped_early = true;
            break 'units;
        }
        // The next unit — and, for the incremental walk, the directory it is read from.
        // A cancelled or exhausted request is observed by the walk *before* it examines
        // an entry, and by this check before an already-enumerated unit is scanned, so no
        // unit work starts after a stop.
        let unit = match stream.next_unit(ctx.budget()) {
            Ok(Some(unit)) => unit,
            Ok(None) => break 'units,
            Err(error) => {
                pass.issue = merge_issue(pass.issue, terminal_execution(&error, ctx.usage()));
                pass.diagnostics.push(terminal_diagnostic(&error, None));
                break 'units;
            }
        };
        // Units before the cursor boundary were published by an earlier page: they
        // are neither read nor re-billed here.
        if let Some(resume) = &pending
            && (resume.container != *unit.origin() || resume.ordinal != unit.ordinal())
        {
            continue;
        }
        // Cooperative interruption is checked before any unit work, so a cancelled
        // or exhausted request never reports more than the published prefix.
        if let Err(error) = ctx.budget().poll() {
            pass.issue = merge_issue(pass.issue, terminal_execution(&error, ctx.usage()));
            pass.diagnostics.push(terminal_diagnostic(&error, None));
            break 'units;
        }
        let (position, skip) = match pending.take() {
            Some(boundary) => (boundary.position, boundary.item_index),
            None => (QueryPosition::Resource, 0),
        };
        let mut scan = UnitScan::open(position, skip);
        ctx.begin_unit();
        record_examined(&mut pass.examined, &unit);
        let mut end: Option<UnitEnd> = None;
        loop {
            let mut items = Vec::new();
            let step = scan.step(ctx, &unit, &mut items);
            for item in items {
                if scan.skip_pending() {
                    // Replayed prefix of a continued step: an earlier page already
                    // published and billed it, so it counts as scanned but is neither
                    // published nor charged again.
                    scan.skip_one();
                    pass.scanned_items += 1;
                    pass.unknown_candidates += u64::from(item.certainty == XrefCertainty::Unknown);
                    continue;
                }
                if request.max_items != 0 && to_u64(pass.items.len())? >= request.max_items {
                    end = Some(UnitEnd::Page);
                    break;
                }
                if let Err(error) = ctx.charge_result_item() {
                    pass.issue = merge_issue(pass.issue, terminal_execution(&error, ctx.usage()));
                    pass.diagnostics
                        .push(terminal_diagnostic(&error, Some(&unit)));
                    end = Some(UnitEnd::Terminal);
                    break;
                }
                let mut item = item;
                // The probe answered as a raw constant-pool probe, but the caller asked a
                // relation P1 does not resolve. Each item keeps the caller's relation and
                // stays an unanalysed candidate: derivation, consumer, operation,
                // certainty and evidence are the facts the probe found, and nothing here
                // expands or resolves them.
                if keep_relation {
                    item.relation = request.relation;
                }
                pass.scanned_items += 1;
                pass.unknown_candidates += u64::from(item.certainty == XrefCertainty::Unknown);
                scan.publish_one();
                pass.boundary = Some(boundary_of(
                    &unit,
                    scan.position().clone(),
                    scan.published(),
                ));
                pass.items.push(item);
            }
            if end.is_some() {
                break;
            }
            // A step that answers fewer items than a resumed position already published
            // does not describe this unit: the cursor is refused instead of skipping an
            // item it never published.
            if scan.skip_pending() {
                return Err(cursor_position_mismatch());
            }
            // The step's own domain diagnostics are charged after its items, exactly as a
            // unit's were; a refused charge keeps the prefix and stops.
            for diagnostic in ctx.take_diagnostics() {
                if let Err(error) = ctx.charge_result_item() {
                    pass.issue = merge_issue(pass.issue, terminal_execution(&error, ctx.usage()));
                    pass.diagnostics
                        .push(terminal_diagnostic(&error, Some(&unit)));
                    end = Some(UnitEnd::Terminal);
                    break;
                }
                pass.diagnostics.push(diagnostic);
            }
            if end.is_some() {
                break;
            }
            match step {
                Err(error) if is_cursor_position_mismatch(&error) => return Err(error),
                Err(error) => {
                    pass.issue = merge_issue(pass.issue, terminal_execution(&error, ctx.usage()));
                    pass.diagnostics
                        .push(terminal_diagnostic(&error, Some(&unit)));
                    end = Some(UnitEnd::Terminal);
                    break;
                }
                Ok(outcome) => {
                    // Every item and diagnostic of the step was published: the walk stands
                    // at the step that follows, so the position the next page resumes at is
                    // that step — never the step that just finished.
                    scan.finish_step();
                    pass.boundary = Some(boundary_of(
                        &unit,
                        scan.position().clone(),
                        scan.published(),
                    ));
                    if let Some(length) = ctx.materialized_length(&unit) {
                        pass.standalone_examined = length;
                    }
                    match outcome {
                        StepOutcome::Finished => break,
                        StepOutcome::Stepped => {
                            if request.max_items != 0
                                && to_u64(pass.items.len())? >= request.max_items
                            {
                                end = Some(UnitEnd::Page);
                                break;
                            }
                        }
                    }
                }
            }
        }
        if let Some(end) = end {
            if end == UnitEnd::Page {
                pass.stopped_early = true;
            }
            break 'units;
        }
    }

    pass.examined = merge_examined(pass.examined, stream.examined());
    // A subtree the walk could not read bounds the report without ending the scan: the entries
    // behind it are unknown, so the pass is not complete, and the state names why. It is folded
    // here rather than seeded up front because a walk discovers the subtree as it pulls.
    if let Some(subtree) = stream.subtree_issue() {
        pass.issue = merge_issue(pass.issue, subtree);
    }
    pass.resume_pending = pending.is_some();
    Ok(pass)
}

/// One unit's scan: the producer program, resumed where the previous page stopped.
///
/// The program is fixed for every unit — [`Producer::Resource`], then [`Producer::Code`]
/// (the raw pool probe, or the instruction stream of one method after another), then
/// [`Producer::Metadata`], then [`Producer::Bootstrap`] — so a position names the same step
/// in every page of one request. A step never starts after the page filled, and the items a
/// started step produced beyond the page are dropped instead of published.
struct UnitScan {
    /// The producer whose step is running.
    producer: Producer,
    /// The producer the step after it belongs to.
    next: Producer,
    /// The step the items of the running step belong to.
    position: QueryPosition,
    /// The step that follows it.
    next_position: QueryPosition,
    /// The code producer's own state: opened by its step, kept across the steps after it.
    code: code::CodeWalk,
    /// Items of the running step this page published.
    published: u64,
    /// Items of the running step an earlier page published: skipped, never published twice.
    skip: u64,
}

/// The producers of one unit, in the order their items come out.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Producer {
    Resource,
    Code,
    Metadata,
    Bootstrap,
    /// Every producer of the unit ran.
    Done,
}

/// What one step of a unit's scan did.
enum StepOutcome {
    /// The step is over; the unit's next step runs next.
    Stepped,
    /// The unit's whole program ran.
    Finished,
}

impl UnitScan {
    /// Opens one unit's scan at the position a fresh page or a continuation starts from.
    fn open(position: QueryPosition, skip: u64) -> Self {
        let producer = producer_of(&position);
        Self {
            producer,
            next: producer,
            position: position.clone(),
            next_position: position,
            code: code::CodeWalk::unopened(),
            published: 0,
            skip,
        }
    }

    /// The step the items of the running step belong to.
    fn position(&self) -> &QueryPosition {
        &self.position
    }

    /// Items of the running step this page already published.
    fn published(&self) -> u64 {
        self.published
    }

    /// Whether the resumed position still owes items to an earlier page.
    fn skip_pending(&self) -> bool {
        self.skip > 0
    }

    /// Consumes one replayed item of the resumed position.
    fn skip_one(&mut self) {
        self.skip -= 1;
        self.published += 1;
    }

    /// Counts one item of the running step as published by this request.
    ///
    /// The count covers the items an earlier page published (the replayed prefix) and the
    /// items this page just published, so it is exactly the item count the boundary carries.
    fn publish_one(&mut self) {
        self.published += 1;
    }

    /// Runs the next step of the unit's program into `out`.
    ///
    /// The step's own position does not move here: the run records the boundary from it while
    /// it publishes the step's items, and only [`UnitScan::finish_step`] stands the walk at
    /// the step that follows. A page that fills inside a step therefore keeps the position of
    /// that step, and a page that fills exactly at its last item keeps the step after it.
    fn step(
        &mut self,
        ctx: &mut ScanContext<'_>,
        unit: &ScanUnit,
        out: &mut Vec<XrefItem>,
    ) -> Result<StepOutcome> {
        match self.producer {
            Producer::Resource => {
                let outcome = resource::scan(ctx, unit, out);
                self.next = Producer::Code;
                self.next_position = QueryPosition::Code;
                outcome.map(|()| StepOutcome::Stepped)
            }
            Producer::Code => match self.code.step(ctx, unit, self.position.clone(), out)? {
                CodeOutcome::Stepped(position) => {
                    self.next = Producer::Code;
                    self.next_position = position;
                    Ok(StepOutcome::Stepped)
                }
                CodeOutcome::Finished => {
                    self.next = Producer::Metadata;
                    self.next_position = QueryPosition::Metadata;
                    Ok(StepOutcome::Stepped)
                }
            },
            Producer::Metadata => {
                let outcome = metadata::scan(ctx, unit, out);
                self.next = Producer::Bootstrap;
                self.next_position = QueryPosition::Bootstrap;
                outcome.map(|()| StepOutcome::Stepped)
            }
            Producer::Bootstrap => {
                let outcome = bootstrap::scan(ctx, unit, out);
                self.next = Producer::Done;
                self.next_position = QueryPosition::UnitComplete;
                outcome.map(|()| StepOutcome::Stepped)
            }
            Producer::Done => Ok(StepOutcome::Finished),
        }
    }

    /// Stands at the step that follows the one whose items were all published.
    fn finish_step(&mut self) {
        self.producer = self.next;
        self.position = self.next_position.clone();
        self.published = 0;
    }
}

/// The producer a position belongs to.
fn producer_of(position: &QueryPosition) -> Producer {
    match position {
        QueryPosition::Resource => Producer::Resource,
        QueryPosition::Code | QueryPosition::Method { .. } => Producer::Code,
        QueryPosition::Metadata => Producer::Metadata,
        QueryPosition::Bootstrap => Producer::Bootstrap,
        QueryPosition::UnitComplete => Producer::Done,
    }
}

/// One unit's position in the report's own terms.
fn boundary_of(unit: &ScanUnit, position: QueryPosition, item_index: u64) -> QueryBoundary {
    QueryBoundary {
        container: unit.origin().clone(),
        ordinal: unit.ordinal(),
        position,
        item_index,
    }
}

/// The error a continuation carries when its position does not describe the unit it names.
fn cursor_position_mismatch() -> Error {
    Error::invalid_input(
        "query_cursor_mismatch",
        "cursor position does not exist in the unit the boundary names",
    )
}

/// Whether one producer's failure is a refused cursor position rather than a scan result.
///
/// A position that cannot be honoured is a binding error, not a partial scan: it is returned
/// as an error instead of being reported as damage of the unit.
fn is_cursor_position_mismatch(error: &Error) -> bool {
    matches!(error, Error::InvalidInput { code, .. } if code == "query_cursor_mismatch")
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
                variant: physical_variant_for_path(&entry.id.raw_name.0),
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
pub(crate) fn escaped_raw_name(raw: &[u8]) -> String {
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
    /// How a candidate answers this scan; every sub-scan asks this context instead of
    /// comparing targets itself.
    filter: CandidateRule,
    budget: &'a mut Budget,
    diagnostics: Vec<Diagnostic>,
    materialized: Vec<MaterializedUnit>,
}

impl<'a> ScanContext<'a> {
    fn new(
        snapshot: &'a ArtifactSnapshot,
        request: &'a QueryRequest,
        filter: CandidateRule,
        budget: &'a mut Budget,
    ) -> Self {
        Self {
            snapshot,
            request,
            filter,
            budget,
            diagnostics: Vec::new(),
            materialized: Vec::new(),
        }
    }

    pub(super) fn request(&self) -> &QueryRequest {
        self.request
    }

    /// Whether one candidate a sub-scan found answers the active filter.
    ///
    /// A candidate is stated in the two dimensions a query target has — a symbol and a
    /// literal — and a sub-scan passes the dimensions its own product carries, so one entry
    /// of a class file can answer a symbol request and a descriptor-type request without the
    /// two being confused. `Exact` compares the raw bytes of the dimension the request names;
    /// `MemberShape` answers a member symbol whose raw name and descriptor bytes are equal,
    /// whatever owner it spells; `SignaturePolymorphic` answers a method symbol whose owner and
    /// name bytes are equal, whatever descriptor the site spells.
    pub(super) fn candidate_matches(
        &self,
        symbol: Option<&SymbolRef>,
        literal: Option<&LiteralValue>,
    ) -> bool {
        self.published_target(symbol, literal).is_some()
    }

    /// The target one published item carries for a candidate that answered the filter.
    ///
    /// Under `Exact` the candidate's own value *is* the request's target (matching is
    /// equality), so this publishes exactly the item P1 always published. Under a shape filter
    /// the candidate's own symbol is the one fact the item must keep: the filter compares only
    /// some of the symbol's dimensions on purpose, so the dimensions it does not compare (the
    /// owner under `MemberShape`, the descriptor under `SignaturePolymorphic`) can only come
    /// back through the candidate. `None` means the candidate does not answer the filter at
    /// all, which is the same decision [`ScanContext::candidate_matches`] reports.
    pub(super) fn published_target(
        &self,
        symbol: Option<&SymbolRef>,
        literal: Option<&LiteralValue>,
    ) -> Option<XrefTarget> {
        match &self.filter {
            CandidateRule::Exact(QueryTarget::Symbol { value }) => {
                (symbol == Some(value)).then(|| XrefTarget::Symbol {
                    value: value.clone(),
                })
            }
            CandidateRule::Exact(QueryTarget::Literal { value }) => {
                (literal == Some(value)).then(|| XrefTarget::Literal {
                    value: value.clone(),
                })
            }
            CandidateRule::Owner { owner } => {
                let symbol = symbol?;
                let found_owner = match symbol {
                    SymbolRef::Class { owner } => owner,
                    SymbolRef::Method { owner, .. } | SymbolRef::Field { owner, .. } => owner,
                };
                (found_owner.0 == owner.0).then(|| XrefTarget::Symbol {
                    value: symbol.clone(),
                })
            }
            CandidateRule::MemberShape { name, descriptor } => {
                let symbol = symbol?;
                let (found_name, found_descriptor) = member_shape(symbol)?;
                (found_name.0 == name.0 && found_descriptor.0 == descriptor.0).then(|| {
                    XrefTarget::Symbol {
                        value: symbol.clone(),
                    }
                })
            }
            CandidateRule::SignaturePolymorphic { owner, name } => {
                let symbol = symbol?;
                let (found_owner, found_name) = method_owner_and_name(symbol)?;
                (found_owner.0 == owner.0 && found_name.0 == name.0).then(|| XrefTarget::Symbol {
                    value: symbol.clone(),
                })
            }
        }
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
                let materialized = self.snapshot.read_entry_for_analysis(entry, self.budget)?;
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

#[derive(Clone)]
struct ExaminedContainer {
    container: ContainerId,
    first: u64,
    last_exclusive: u64,
}

/// The units one request's scope and consumer schema need, pulled one at a time.
///
/// Which stream a request gets is the request's own decision, and that is the first half of
/// "scan by demand": `resource` is the only consumer that reads an entry which is not a
/// class, so a request that names it asks about every entry of every container — its
/// denominator *is* the whole range — and is answered by the reader's incremental *entry*
/// cursor, which reaches one container at a time and descends into a nested container
/// exactly when the scan pulls through the entry that holds it. Every other schema can only
/// produce items from class candidates, so it walks the same range through the reader's
/// incremental *scope* cursor, which is that same walk with the class filter applied.
///
/// No stream collects a provider report: the query has no eager range to apply a page size
/// to, and the only directory products it pays for are the ones a walk opened on its way to
/// the units it handed over.
enum UnitStream<'a> {
    /// A standalone CLASS snapshot: its root is the one unit, and no range is enumerated.
    Standalone { unit: Option<ScanUnit>, bytes: u64 },
    /// The scope's own entries, walked one container at a time.
    Entries(EntryScope),
    /// The scope's class candidates, walked one container at a time.
    Walked(WalkedScope<'a>),
}

/// Every entry of a scope, walked by the reader's incremental entry cursor.
///
/// The walk validates one container's directory at a time and hands one entry over per pull, so a
/// nested container is materialized only when the walk *descends* into the entry that holds it, and
/// an entry's own bytes are read only when a consumer asks for them. That validation is this
/// invocation's own work — the ranges below state it, container by container — so the walk opens
/// each container itself rather than being answered from a store or from a product another consumer
/// happens to hold (the reader's `walked_container_facts` is that access). The containers it reached
/// are the ones it may state a denominator for; a container behind the page's stop was never opened
/// and stays unknown.
struct EntryScope {
    cursor: EntryCursor,
    exhausted: bool,
}

/// The scope's class candidates, walked by the reader's incremental scope cursor.
///
/// The walk reads a container's directory once and hands one class candidate over per pull,
/// so the methods and the nested containers behind the page's stop are never reached: a
/// nested container is materialized when the walk *descends* into the entry that holds it,
/// which happens only while the scan keeps pulling. What the walk examined is its own
/// product too: an entry ordinal it passed, and the ordinal of a descent, are the ranges
/// this invocation really examined — a container it never entered is left unknown instead
/// of being counted as examined or as empty.
struct WalkedScope<'a> {
    snapshot: &'a ArtifactSnapshot,
    cursor: ScopeCursor,
    examined: Vec<ExaminedContainer>,
    exhausted: bool,
}

impl<'a> UnitStream<'a> {
    /// Opens the unit stream this request's scope and consumer schema need.
    ///
    /// Opening reads nothing at all: a standalone snapshot's unit is its own root, a walk's
    /// first container is opened by its first pull, and no stream collects a range it has not
    /// been asked for yet.
    fn open(
        snapshot: &'a ArtifactSnapshot,
        request: &QueryRequest,
        _budget: &mut Budget,
    ) -> Result<Self> {
        match (snapshot.kind(), &request.physical.scope) {
            (ArtifactKind::StandaloneClass, PhysicalScope::SnapshotAll) => Ok(Self::Standalone {
                unit: Some(ScanUnit {
                    origin: root_origin(snapshot.id()),
                    ordinal: 0,
                    kind: UnitKind::StandaloneRoot,
                }),
                bytes: snapshot.len(),
            }),
            // A request that does not name the resource consumer can only produce items from
            // class candidates, so over a tree it walks them through the reader's class-only
            // cursor — the same walk this file has always used for that scope, which reads one
            // container at a time and descends only while the scan keeps pulling.
            _ if !wants_resource(request)
                && matches!(request.physical.scope, PhysicalScope::ArtifactTree { .. }) =>
            {
                Ok(Self::Walked(WalkedScope {
                    snapshot,
                    cursor: snapshot.scope_cursor(&request.physical.scope)?,
                    examined: Vec::new(),
                    exhausted: false,
                }))
            }
            // Every other request asks about entries no class candidate carries — the
            // resource consumer is the one that reads an entry which is not a class — so its
            // range is the scope's entries, pulled one at a time. Nothing behind the page's
            // stop is opened: a nested container is expanded only when the walk descends into
            // the entry that holds it, and no entry's bytes are read before a consumer asks.
            //
            // This is also the whole-snapshot class-only request: the one container that scope
            // *is* is the directory product the walk has to build before its first unit either
            // way, and pulling its entries from the walk costs exactly what enumerating them
            // into a report did — without building the report. A non-ZIP snapshot is refused by
            // the walk itself, whose refusal is propagated instead of being rewritten here.
            _ => Ok(Self::Entries(EntryScope {
                cursor: snapshot.entry_cursor(&request.physical.scope)?,
                exhausted: false,
            })),
        }
    }

    /// The next unit of the scope's own physical order, or `None` at the end of the scope.
    fn next_unit(&mut self, budget: &mut Budget) -> Result<Option<ScanUnit>> {
        match self {
            Self::Standalone { unit, .. } => Ok(unit.take()),
            Self::Entries(scope) => scope.next_unit(budget),
            Self::Walked(walk) => walk.next_unit(budget),
        }
    }

    /// Whether this walk reached the end of the scope with nothing left unknown.
    ///
    /// An enumerated range is part of the stream by construction; an incremental walk is
    /// only exhausted once it really returned the end of the scope *and* left no subtree
    /// unread — a container the walk could not open is unknown, so the walk has not
    /// established the scope's contents even when it walked past it.
    fn exhausted(&self) -> bool {
        match self {
            Self::Standalone { unit, .. } => unit.is_none(),
            Self::Entries(scope) => scope.exhausted && scope.cursor.issue().is_none(),
            Self::Walked(walk) => walk.exhausted,
        }
    }

    /// The walk's own diagnostics, named where each failing subtree hangs from.
    fn diagnostics(&self) -> &[Diagnostic] {
        match self {
            Self::Standalone { .. } => &[],
            Self::Entries(scope) => scope.cursor.diagnostics(),
            Self::Walked(walk) => walk.cursor.diagnostics(),
        }
    }

    /// The stop one walk recorded for a subtree it could not read, if any.
    ///
    /// A subtree that stays unknown bounds the *report* without ending the scan, so this is
    /// merged into the pass's own issue once the pass is over; a stream that reported it as
    /// its terminal state would claim more than the scan really stopped on.
    fn subtree_issue(&self) -> Option<ExecutionReport> {
        match self {
            Self::Entries(scope) => scope.cursor.issue().cloned(),
            Self::Standalone { .. } | Self::Walked(_) => None,
        }
    }

    /// The ordinal prefixes this walk examined, by container.
    fn examined(&self) -> &[ExaminedContainer] {
        match self {
            Self::Standalone { .. } | Self::Entries(_) => &[],
            Self::Walked(walk) => &walk.examined,
        }
    }

    /// The stream's own ranges plus this pass's own ordinal or byte coverage.
    ///
    /// The provider's own ranges are carried verbatim (they were already billed by the
    /// provider); the `xref_scan_entries` / `xref_scan_bytes` ranges describe what this
    /// invocation examined, so a page limit, a continuation or a schema that never reads a
    /// unit's bytes shows up instead of a claim.
    ///
    /// The walks describe a container's remainder differently, and the difference is the
    /// honest one: an enumerated container has a known entry count, so the ordinals no unit
    /// reached are named as skipped; a walked container only establishes the prefix this
    /// invocation looked at, so the range behind it stays *unknown* — it is not named as
    /// examined and not claimed as empty, and the coverage state is `Partial` unless the
    /// walk itself reached the end of the scope.
    ///
    /// The entry walk sits between the two, and for the same reason: every container it
    /// *reached* states its own denominator (its directory was validated in full, so the
    /// ordinals no unit of this invocation examined are named as skipped), while a container
    /// it never opened states nothing at all — no range, no denominator, no empty claim.
    fn coverage_parts(
        &self,
        examined: &[ExaminedContainer],
        standalone_examined: u64,
    ) -> (Vec<CoverageRange>, Vec<CoverageRange>) {
        match self {
            Self::Standalone { bytes, .. } => {
                let label = "standalone_class:xref_scan_bytes".to_string();
                let mut scanned = Vec::new();
                let mut skipped = Vec::new();
                let examined = standalone_examined.min(*bytes);
                if examined > 0 {
                    scanned.push(CoverageRange {
                        label: label.clone(),
                        start: 0,
                        end: examined,
                    });
                }
                if examined < *bytes {
                    skipped.push(CoverageRange {
                        label,
                        start: examined,
                        end: *bytes,
                    });
                }
                (scanned, skipped)
            }
            Self::Entries(scope) => scope.coverage_parts(examined),
            Self::Walked(_) => {
                let scanned = examined
                    .iter()
                    .filter(|entry| entry.first < entry.last_exclusive)
                    .map(|entry| CoverageRange {
                        label: format!("container:{}:xref_scan_entries", entry.container.0),
                        start: entry.first,
                        end: entry.last_exclusive,
                    })
                    .collect();
                (scanned, Vec::new())
            }
        }
    }
}

/// Whether one request names the consumer that reads an entry which is not a class.
fn wants_resource(request: &QueryRequest) -> bool {
    request.consumers.kinds.contains(&ConsumerKind::Resource)
}

impl EntryScope {
    /// The next entry of the scope, or `None` at the end of the walk.
    ///
    /// The record the cursor hands over is the container's own verified record, so the unit is
    /// built from it directly: the walk already checked the ordinal and the raw name against the
    /// directory it parsed, and no entry behind the caller's stop is opened on the way.
    fn next_unit(&mut self, budget: &mut Budget) -> Result<Option<ScanUnit>> {
        let Some(entry) = self.cursor.next_entry(budget)? else {
            self.exhausted = true;
            return Ok(None);
        };
        Ok(Some(ScanUnit {
            origin: entry.entry.id.origin.clone(),
            ordinal: entry.entry.id.ordinal,
            kind: UnitKind::Entry(entry.entry),
        }))
    }

    /// The ranges this walk established, plus the pass's own examined prefixes.
    ///
    /// Every container the walk *opened* published its whole directory as validated work — the
    /// same per-record charges the provider's own enumeration makes, and the one statement a
    /// validated directory can make about itself — and the entries of it that no unit of this
    /// invocation examined stay named as skipped. A container the walk stopped before opening
    /// appears with the count its own directory declares and without one validated record: its
    /// range is named as skipped, never as examined. A container the walk never reached at all
    /// appears in neither side: its entry count was never established, so no range may claim it as
    /// examined or as empty. The scope's own root container keeps the provider's label, because it
    /// *is* the range a whole-snapshot request would have enumerated.
    fn coverage_parts(
        &self,
        examined: &[ExaminedContainer],
    ) -> (Vec<CoverageRange>, Vec<CoverageRange>) {
        let mut scanned = Vec::new();
        let mut skipped = Vec::new();
        for (index, container) in self.cursor.containers().iter().enumerate() {
            let id = &container.origin.current_container().0;
            let known = container.entries;
            let directory_label = if index == 0 {
                "central_directory_entries".to_string()
            } else {
                format!("container:{id}:central_directory_entries")
            };
            let directory = (known > 0).then_some(CoverageRange {
                label: directory_label,
                start: 0,
                end: known,
            });
            let label = format!("container:{id}:xref_scan_entries");
            if !container.walked {
                // The walk never opened this container: the count is the directory's own
                // declaration, and not one of its records was validated or examined.
                if let Some(range) = directory {
                    skipped.push(range);
                }
                if known > 0 {
                    skipped.push(CoverageRange {
                        label,
                        start: 0,
                        end: known,
                    });
                }
                continue;
            }
            if let Some(range) = directory {
                scanned.push(range);
            }
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
        (scanned, skipped)
    }
}

impl WalkedScope<'_> {
    /// The next class candidate of the scope, or `None` at the end of the walk.
    ///
    /// The candidate's own record is looked up in the container the walk is inside, which
    /// is the authority for the entry's identity: the ordinal and the raw name the walk
    /// handed over are checked against the record at that coordinate before the unit is
    /// built, and a container that cannot be read completely is refused instead of being
    /// answered with the entries it managed to read.
    fn next_unit(&mut self, budget: &mut Budget) -> Result<Option<ScanUnit>> {
        let Some(class) = self.cursor.next_class(budget)? else {
            self.exhausted = true;
            return Ok(None);
        };
        match class.location {
            PhysicalClassLocation::StandaloneRoot { snapshot } => Ok(Some(ScanUnit {
                origin: root_origin(&snapshot),
                ordinal: 0,
                kind: UnitKind::StandaloneRoot,
            })),
            PhysicalClassLocation::ArchiveEntry { entry } => {
                let record = self
                    .snapshot
                    .container_record(&entry, budget)?
                    .ok_or_else(|| {
                        Error::invalid_input(
                            "entry_not_found",
                            "the walked entry is not a record of the container it names",
                        )
                    })?;
                self.observe(&record.id);
                Ok(Some(ScanUnit {
                    origin: record.id.origin.clone(),
                    ordinal: record.id.ordinal,
                    kind: UnitKind::Entry(record),
                }))
            }
        }
    }

    /// Records the ordinal prefix this invocation examined for one entry's container and for
    /// every container above it.
    ///
    /// The walk examines a container's entries in ordinal order before it descends, so an
    /// entry at ordinal `n` means the container's own directory was examined up to and
    /// including `n`. An ancestor's prefix is the descent that reached the child: the child
    /// entry's own ordinal in that ancestor. Both are lower bounds of what the walk really
    /// looked at (a container it left behind was examined in full, which the walk does not
    /// report), and a lower bound never claims work that did not happen.
    fn observe(&mut self, entry: &PhysicalEntryId) {
        self.observe_prefix(&entry.origin, entry.ordinal);
        for depth in (0..entry.origin.steps.len()).rev() {
            let step = &entry.origin.steps[depth];
            let parent = ContainerOrigin {
                snapshot: entry.origin.snapshot.clone(),
                root_container: entry.origin.root_container.clone(),
                steps: entry.origin.steps[..depth].to_vec(),
            };
            self.observe_prefix(&parent, step.via_ordinal);
        }
    }

    /// Merges one examined ordinal into the prefix of its container.
    fn observe_prefix(&mut self, origin: &ContainerOrigin, ordinal: u64) {
        let container = origin.current_container().clone();
        let end = ordinal.saturating_add(1);
        match self
            .examined
            .iter_mut()
            .find(|entry| entry.container == container)
        {
            Some(entry) => {
                entry.first = entry.first.min(ordinal);
                entry.last_exclusive = entry.last_exclusive.max(end);
            }
            None => self.examined.push(ExaminedContainer {
                container,
                first: ordinal,
                last_exclusive: end,
            }),
        }
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

/// The union of two examined-container lists, per container.
///
/// The pass records the units it started and the walk records the ordinal prefixes it looked
/// at; the walk's prefixes are the wider statement, and a range they do not cover is not
/// examined by this invocation at all.
fn merge_examined(
    mut examined: Vec<ExaminedContainer>,
    other: &[ExaminedContainer],
) -> Vec<ExaminedContainer> {
    for entry in other {
        match examined
            .iter_mut()
            .find(|held| held.container == entry.container)
        {
            Some(held) => {
                held.first = held.first.min(entry.first);
                held.last_exclusive = held.last_exclusive.max(entry.last_exclusive);
            }
            None => examined.push(entry.clone()),
        }
    }
    examined
}

fn root_origin(snapshot: &SnapshotId) -> ContainerOrigin {
    ContainerOrigin {
        snapshot: snapshot.clone(),
        root_container: ContainerId("root".into()),
        steps: Vec::new(),
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
pub(crate) fn to_u64(value: usize) -> Result<u64> {
    u64::try_from(value).map_err(|_| {
        Error::invalid_input(
            "query_size_overflow",
            "byte or entry count does not fit u64",
        )
    })
}
