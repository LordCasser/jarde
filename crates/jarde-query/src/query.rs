//! Query request/result schema, request compiler and cursor binding.
//!
//! The public types below are the P1 query contract: a request declares the
//! relation, target, physical view and consumer schema, and the report returns
//! items, page, coverage, execution and diagnostics without ever mixing the
//! analysis state with the execution state.

use jarde_reader::artifact::ArtifactSnapshot;
use jarde_reader::budget::Budget;
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{
    ArchiveNameBytes, ByteSpan, ContainerOrigin, Coverage, CoverageDimension, Diagnostic,
    DiagnosticSeverity, Digest, ExecutionReport, JvmBytes, Provenance, SnapshotId, SymbolRef,
};
use jarde_reader::view::{PhysicalScope, PhysicalView};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryRelation {
    ConstantPoolContains,
    MentionsSymbol,
    LiteralValue,
    ReferencesDefinition,
    MayDispatchTo,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsumerKind {
    Invocation,
    Field,
    Type,
    Constant,
    Exception,
    Signature,
    Annotation,
    InnerNest,
    Module,
    Bootstrap,
    Resource,
    Verification,
    Debug,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsumerSchema {
    pub version: u16,
    pub kinds: BTreeSet<ConsumerKind>,
}

impl ConsumerSchema {
    pub fn new(version: u16, kinds: impl IntoIterator<Item = ConsumerKind>) -> Self {
        Self {
            version,
            kinds: kinds.into_iter().collect(),
        }
    }
}

/// Engine schema version bound into every query cursor.
///
/// It covers the query request/result schema itself, so a cursor issued under a
/// different schema generation cannot be replayed against this engine.
///
/// Schema 2 binds the complete [`QueryTarget`] into the cursor. A schema 1 cursor does
/// not carry a target, and a continuation that changed the target used to be replayed
/// with the old page's published prefix, so the engine rejects this generation
/// explicitly instead of keeping a compatibility path it cannot verify.
///
/// Schema 3 adds [`QueryBoundary::position`]: a page that stops inside a unit now names
/// the step it stopped in, so a continuation resumes at that step instead of replaying
/// the unit. A schema 2 boundary counts items per unit and cannot be interpreted as a
/// step position, so this generation is rejected as a whole rather than replayed at a
/// boundary it was not issued for.
pub const QUERY_ENGINE_SCHEMA: u16 = 3;

/// Consumer schema version this engine executes.
const CONSUMER_SCHEMA_VERSION: u16 = 1;

/// Consumer categories P1 declares but does not implement.
///
/// `Verification` and `Debug` would need `StackMapTable`/`LVT` decoding that P0
/// does not provide, so a request that names them can never claim
/// `complete_within_schema` for the artifact-structural dimension.
const UNIMPLEMENTED_CONSUMER_KINDS: [ConsumerKind; 2] =
    [ConsumerKind::Verification, ConsumerKind::Debug];

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryRequest {
    pub relation: QueryRelation,
    pub target: QueryTarget,
    pub physical: PhysicalView,
    pub consumers: ConsumerSchema,
    /// `0` means "no page limit"; the scan still obeys the budget.
    pub max_items: u64,
    pub cursor: Option<QueryCursor>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum QueryTarget {
    Symbol { value: SymbolRef },
    Literal { value: LiteralValue },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LiteralValue {
    String {
        value: JvmBytes,
    },
    Class {
        value: JvmBytes,
    },
    Integer {
        value: i32,
    },
    Long {
        value: i64,
    },
    /// IEEE 754 single-precision bit pattern.
    Float {
        value: u32,
    },
    /// IEEE 754 double-precision bit pattern.
    Double {
        value: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryReport {
    pub physical: PhysicalView,
    pub relation: QueryRelation,
    pub consumers: ConsumerSchema,
    pub analysis: QueryAnalysis,
    pub items: Vec<XrefItem>,
    pub page: QueryPage,
    pub coverage: QueryCoverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum QueryAnalysis {
    Performed,
    UnsupportedAnalysis { relation: QueryRelation },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryPage {
    pub has_more: bool,
    pub returned_items: u64,
    pub cursor: Option<QueryCursor>,
}

/// Continuation value of one page.
///
/// The cursor is a public structured value that binds the complete query identity:
/// engine schema, snapshot, physical view, relation, [`QueryCursor::target`], consumer
/// schema and the published [`QueryBoundary`]. A continuation whose request disagrees
/// with any of them is rejected with `query_cursor_mismatch`. `max_items` and the budget
/// are execution knobs rather than identity, so a continuation may change them.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryCursor {
    pub engine_schema: u16,
    pub snapshot: SnapshotId,
    pub physical: PhysicalView,
    pub relation: QueryRelation,
    pub target: QueryTarget,
    pub consumers: ConsumerSchema,
    pub boundary: QueryBoundary,
    pub digest: Digest,
}

/// Position immediately after the last published item.
///
/// The position is a pair: [`QueryBoundary::position`] names the step of one unit's scan
/// the page stopped in — the step the next item would come from — and
/// [`QueryBoundary::item_index`] counts the items that step already published for this
/// request's target. The target is part of the cursor identity, and the count is the
/// number of items that target answered in that step, not a byte offset.
///
/// A continuation re-derives the unit's scan program from the bound request and the
/// unit's own class file, verifies that `position` names a step of that program (the
/// [`QueryPosition::Method`] anchor against the class file's own member, and the item
/// count against the items that step answers), and then re-runs that one step, skipping
/// exactly `item_index` of its items. Every earlier step of the unit, every earlier unit
/// and every earlier container is therefore neither re-read nor re-billed, and
/// consecutive pages of one query neither repeat nor skip published items.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryBoundary {
    pub container: ContainerOrigin,
    pub ordinal: u64,
    /// The step of this unit's scan the next published item comes from.
    pub position: QueryPosition,
    /// Items already published from [`QueryBoundary::position`]'s step.
    pub item_index: u64,
}

/// Where the scan stands inside one unit: the step the next published item comes from.
///
/// One unit's scan is a program of steps, all of them derived from the bound request and,
/// for a class candidate, from the class file's own member table: the resource consumer,
/// then the code producer — the raw constant-pool probe, or the instruction stream of one
/// method after another — then the metadata consumer, then the bootstrap consumer. The
/// scan stops between steps, and inside the items of one step, as soon as the page is full
/// or a stop is observed; the position it publishes says where it stopped.
///
/// This is a *verifiable* position, not merely a count: a continuation re-derives the same
/// program and refuses a position that does not name a step of it — [`QueryPosition::Method`]
/// is checked against the class file member it names, and `item_index` against the items the
/// step answers — with `query_cursor_mismatch` instead of skipping items on a guess.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum QueryPosition {
    /// The resource consumer's step of this unit.
    Resource,
    /// The code producer's opening step of this unit: the raw constant-pool probe, or the
    /// instruction stream's first member. Which one it is depends on the bound relation and
    /// on the class file, so the position cannot name that member before the producer read
    /// the unit; the continuation opens the producer at the same point and verifies the
    /// stream it derives.
    Code,
    /// The instruction stream of one member of this unit's class file.
    Method {
        /// Position of the member in the class file's own method list.
        index: u32,
        /// Raw name of the member, as the class file spells it.
        name: JvmBytes,
        /// Raw descriptor of the member, as the class file spells it.
        descriptor: JvmBytes,
    },
    /// The metadata consumer's step of this unit.
    Metadata,
    /// The bootstrap consumer's step of this unit.
    Bootstrap,
    /// Every step of this unit ran: the next item comes from the unit after it.
    UnitComplete,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryCoverage {
    /// P0 three-dimensional coverage; P1 keeps `runtime_resolution` not requested.
    pub dimensions: Coverage,
    pub consumer_schema: ConsumerSchema,
    pub unsupported_categories: Vec<ConsumerKind>,
    pub scanned_items: u64,
    pub unknown_candidates: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XrefItem {
    pub relation: QueryRelation,
    pub source: Provenance,
    pub target: XrefTarget,
    /// Consumer category that produced the item. Raw constant-pool candidates are not
    /// produced by a consumer, so they carry `None` and `XrefDerivation::ConstantPoolCandidate`.
    pub consumer: Option<ConsumerKind>,
    pub operation: XrefOperation,
    pub derivation: XrefDerivation,
    pub certainty: XrefCertainty,
    pub resolution: QueryResolution,
    pub evidence: XrefEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum XrefTarget {
    Symbol { value: SymbolRef },
    Literal { value: LiteralValue },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XrefDerivation {
    StructuralConsumer,
    ConstantPoolCandidate,
    BootstrapEdge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XrefCertainty {
    Exact,
    Unknown,
}

/// P1 fixes this to `not_requested`; P2 adds variants behind a spec change.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryResolution {
    NotRequested,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XrefEvidence {
    pub constant_pool_index: Option<u16>,
    pub bci: Option<u32>,
    pub opcode: Option<u8>,
    /// Raw attribute name bytes: a classfile attribute name, a manifest attribute
    /// name, or the raw service registration key of a `META-INF/services` entry.
    pub attribute: Option<ArchiveNameBytes>,
    /// Class-file coordinates for classfile items; resource items use the
    /// decompressed entry bytes of the resource they were read from.
    pub span: Option<ByteSpan>,
    /// Only populated for bootstrap/condy edges.
    pub via: Vec<BootstrapVia>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapVia {
    pub constant_pool_index: u16,
    pub bootstrap_index: Option<u16>,
    pub argument_index: Option<u16>,
}

/// Closed P1 operation set; new variants require a design/spec change first.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XrefOperation {
    /// X0 raw constant-pool candidate; not produced by any consumer.
    ConstantPoolEntry,
    InvokeVirtual,
    InvokeSpecial,
    InvokeStatic,
    InvokeInterface,
    InvokeDynamic,
    GetField,
    GetStatic,
    PutField,
    PutStatic,
    New,
    NewArray,
    MultiNewArray,
    CheckCast,
    InstanceOf,
    Ldc,
    ConstantValue,
    ExceptionHandler,
    ExceptionsAttribute,
    SuperClass,
    Interface,
    FieldDescriptor,
    MethodDescriptor,
    GenericSignature,
    RecordComponent,
    Annotation,
    TypeAnnotation,
    AnnotationDefault,
    InnerClass,
    EnclosingMethod,
    NestHost,
    NestMembers,
    PermittedSubclasses,
    ModuleUses,
    ModuleProvides,
    BootstrapMethod,
    BootstrapArgument,
    ManifestMainClass,
    ManifestAgent,
    ManifestClassPath,
    ManifestAutomaticModuleName,
    ManifestMultiRelease,
    ServiceProvider,
}

/// Stable snake_case code of a relation; it must match the serde renaming of
/// [`QueryRelation`] (pinned by a unit test, and bound into the cursor digest).
pub(crate) fn relation_code(relation: QueryRelation) -> &'static str {
    match relation {
        QueryRelation::ConstantPoolContains => "constant_pool_contains",
        QueryRelation::MentionsSymbol => "mentions_symbol",
        QueryRelation::LiteralValue => "literal_value",
        QueryRelation::ReferencesDefinition => "references_definition",
        QueryRelation::MayDispatchTo => "may_dispatch_to",
    }
}

/// Stable snake_case code of a consumer category; it must match the serde
/// renaming of [`ConsumerKind`] (pinned by a unit test).
fn consumer_kind_code(kind: ConsumerKind) -> &'static str {
    match kind {
        ConsumerKind::Invocation => "invocation",
        ConsumerKind::Field => "field",
        ConsumerKind::Type => "type",
        ConsumerKind::Constant => "constant",
        ConsumerKind::Exception => "exception",
        ConsumerKind::Signature => "signature",
        ConsumerKind::Annotation => "annotation",
        ConsumerKind::InnerNest => "inner_nest",
        ConsumerKind::Module => "module",
        ConsumerKind::Bootstrap => "bootstrap",
        ConsumerKind::Resource => "resource",
        ConsumerKind::Verification => "verification",
        ConsumerKind::Debug => "debug",
    }
}

/// Categories named by this request that P1 cannot scan.
pub(crate) fn unsupported_categories(consumers: &ConsumerSchema) -> Vec<ConsumerKind> {
    UNIMPLEMENTED_CONSUMER_KINDS
        .iter()
        .copied()
        .filter(|kind| consumers.kinds.contains(kind))
        .collect()
}

/// Coverage of a request that performed no scan at all.
pub(crate) fn not_requested_coverage() -> Coverage {
    Coverage {
        artifact_structural: CoverageDimension::not_requested(),
        runtime_resolution: CoverageDimension::not_requested(),
        dynamic_analysis: CoverageDimension::not_requested(),
    }
}

/// Runs one query request and returns a self-describing report.
///
/// Validation and relation dispatch happen here; the scan itself is owned by
/// [`crate::xref::scan`], which bills items and diagnostics. Every relation scans:
/// `references_definition` and `may_dispatch_to` have no resolver in P1, so they report
/// the `UnsupportedAnalysis` state and keep the raw constant-pool candidates the scan
/// still answers instead of returning an empty, unexplained result.
pub fn execute(
    snapshot: &ArtifactSnapshot,
    request: &QueryRequest,
    budget: &mut Budget,
) -> Result<QueryReport> {
    validate_request(snapshot, request)?;
    let analysis = analysis_of(request.relation);
    let scan = crate::xref::scan(snapshot, request, budget)?;
    let mut diagnostics = Vec::with_capacity(scan.diagnostics.len() + 1);
    if let QueryAnalysis::UnsupportedAnalysis { relation } = analysis {
        // The explanation of the analysis state comes first: it describes the request
        // itself, while every later diagnostic describes the scan that still ran.
        diagnostics.push(relation_unsupported_diagnostic(relation));
    }
    diagnostics.extend(scan.diagnostics);
    Ok(QueryReport {
        physical: request.physical.clone(),
        relation: request.relation,
        consumers: request.consumers.clone(),
        analysis,
        items: scan.items,
        page: scan.page,
        coverage: scan.coverage,
        execution: scan.execution,
        diagnostics,
    })
}

/// Analysis state of one relation in P1.
///
/// `references_definition` and `may_dispatch_to` need the P2 definition resolver, so P1
/// reports the analysis state instead of a resolution and never claims one. The scan
/// still runs, because the raw constant-pool candidates for the requested target are
/// answerable X0 facts: they are reported as unanalysed candidates with no consumer
/// category, no BCI and no owner expansion.
fn analysis_of(relation: QueryRelation) -> QueryAnalysis {
    if matches!(
        relation,
        QueryRelation::ReferencesDefinition | QueryRelation::MayDispatchTo
    ) {
        QueryAnalysis::UnsupportedAnalysis { relation }
    } else {
        QueryAnalysis::Performed
    }
}

/// Validates the request against the snapshot and the engine schema.
///
/// Stable codes: `query_snapshot_mismatch`, `query_artifact_tree_root_mismatch`,
/// `query_target_relation_mismatch` (invalid input),
/// `query_consumer_schema_version` (unsupported) and `query_cursor_mismatch`.
pub(crate) fn validate_request(snapshot: &ArtifactSnapshot, request: &QueryRequest) -> Result<()> {
    if &request.physical.snapshot != snapshot.id() {
        return Err(Error::invalid_input(
            "query_snapshot_mismatch",
            "QueryRequest physical view snapshot does not match the artifact snapshot",
        ));
    }
    if let PhysicalScope::ArtifactTree { root_container } = &request.physical.scope {
        // The only root a fresh snapshot establishes is its own root container, so
        // a caller cannot name an arbitrary container as the tree root.
        if root_container.0 != "root" {
            return Err(Error::invalid_input(
                "query_artifact_tree_root_mismatch",
                "QueryRequest tree root does not match the snapshot root container",
            ));
        }
    }
    if request.consumers.version != CONSUMER_SCHEMA_VERSION {
        return Err(Error::unsupported(
            "query_consumer_schema_version",
            format!(
                "consumer schema version {} is not supported; this engine executes version {}",
                request.consumers.version, CONSUMER_SCHEMA_VERSION
            ),
        ));
    }
    if !relation_admits_target(request.relation, &request.target) {
        return Err(Error::invalid_input(
            "query_target_relation_mismatch",
            format!(
                "target does not match the {} relation",
                relation_code(request.relation)
            ),
        ));
    }
    if let Some(cursor) = &request.cursor {
        validate_cursor(snapshot, request, cursor)?;
    }
    Ok(())
}

/// `constant_pool_contains` is the raw constant-pool probe, so it accepts either
/// a symbol or a literal target; the symbolic relations require a symbol target
/// and `literal_value` requires a literal target.
fn relation_admits_target(relation: QueryRelation, target: &QueryTarget) -> bool {
    match relation {
        QueryRelation::LiteralValue => matches!(target, QueryTarget::Literal { .. }),
        QueryRelation::ConstantPoolContains => true,
        QueryRelation::MentionsSymbol
        | QueryRelation::ReferencesDefinition
        | QueryRelation::MayDispatchTo => matches!(target, QueryTarget::Symbol { .. }),
    }
}

/// Checks a continuation against the binding that issued it.
///
/// The checks run in binding order — schema, snapshot, physical view, relation, target,
/// consumer schema, digest — so an error names the first field that disagrees. The target
/// check happens before the scan: a cursor only describes the published prefix of the
/// target it was issued for, so replaying it for another target would skip that target's
/// items instead of continuing its pages.
fn validate_cursor(
    snapshot: &ArtifactSnapshot,
    request: &QueryRequest,
    cursor: &QueryCursor,
) -> Result<()> {
    if cursor.engine_schema != QUERY_ENGINE_SCHEMA {
        return Err(cursor_mismatch("engine schema version"));
    }
    if &cursor.snapshot != snapshot.id() {
        return Err(cursor_mismatch("snapshot"));
    }
    if cursor.physical != request.physical {
        return Err(cursor_mismatch("physical view"));
    }
    if cursor.relation != request.relation {
        return Err(cursor_mismatch("relation"));
    }
    if cursor.target != request.target {
        return Err(cursor_mismatch("target"));
    }
    if cursor.consumers != request.consumers {
        return Err(cursor_mismatch("consumer schema"));
    }
    let digest = cursor_digest(
        cursor.engine_schema,
        &cursor.snapshot,
        &cursor.physical,
        cursor.relation,
        &cursor.target,
        &cursor.consumers,
        &cursor.boundary,
    )?;
    if digest != cursor.digest {
        return Err(cursor_mismatch("digest"));
    }
    Ok(())
}

fn cursor_mismatch(bound: &str) -> Error {
    Error::invalid_input(
        "query_cursor_mismatch",
        format!("cursor {bound} does not match this snapshot/view/relation/target/schema binding"),
    )
}

/// Canonical cursor binding digest: blake3 over `engine_schema`, `snapshot`, `physical`,
/// `relation`, `target`, `consumers` and `boundary` (the boundary's step position included).
///
/// The encoding is length-prefixed and written by hand: the crate has no JSON
/// dependency outside its tests, and the digest must not depend on a serializer
/// implementation. This is the entry point used both to issue and to check a
/// cursor, so a cursor can never be replayed against a different binding.
///
/// The target is encoded by [`hash_target`] as type tags plus the raw bytes and bit
/// patterns of the value, so textual spellings that would collide stay distinct.
pub(crate) fn cursor_digest(
    engine_schema: u16,
    snapshot: &SnapshotId,
    physical: &PhysicalView,
    relation: QueryRelation,
    target: &QueryTarget,
    consumers: &ConsumerSchema,
    boundary: &QueryBoundary,
) -> Result<Digest> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&engine_schema.to_be_bytes());
    hash_field(&mut hasher, snapshot.0.as_bytes());
    match &physical.scope {
        PhysicalScope::SnapshotAll => hash_field(&mut hasher, b"snapshot_all"),
        PhysicalScope::ArtifactTree { root_container } => {
            hash_field(&mut hasher, b"artifact_tree");
            hash_field(&mut hasher, root_container.0.as_bytes());
        }
    }
    hash_field(&mut hasher, physical.snapshot.0.as_bytes());
    hash_field(&mut hasher, relation_code(relation).as_bytes());
    hash_target(&mut hasher, target);
    // Sorted codes keep the digest independent of the `ConsumerKind` declaration
    // order while the consumer set itself stays canonical in `ConsumerSchema`.
    let mut kinds = consumers
        .kinds
        .iter()
        .map(|kind| consumer_kind_code(*kind))
        .collect::<Vec<_>>();
    kinds.sort_unstable();
    hasher.update(&consumers.version.to_be_bytes());
    hash_count(&mut hasher, kinds.len())?;
    for kind in kinds {
        hash_field(&mut hasher, kind.as_bytes());
    }
    hash_field(&mut hasher, boundary.container.snapshot.0.as_bytes());
    hash_field(&mut hasher, boundary.container.root_container.0.as_bytes());
    hash_count(&mut hasher, boundary.container.steps.len())?;
    for step in &boundary.container.steps {
        hasher.update(&step.via_ordinal.to_be_bytes());
        hash_field(&mut hasher, &step.via_raw_name.0);
        hash_field(&mut hasher, step.child_container.0.as_bytes());
    }
    hasher.update(&boundary.ordinal.to_be_bytes());
    hash_position(&mut hasher, &boundary.position);
    hasher.update(&boundary.item_index.to_be_bytes());
    Ok(Digest(hasher.finalize().to_hex().to_string()))
}

/// Encodes the step a page stopped in into the cursor digest.
///
/// Every variant is tagged, so a position can never be read as another: a method step is
/// not its own index repeated as a pool step, and the member's raw name and descriptor take
/// part with the index, so a cursor issued for one member cannot be replayed against a
/// class file whose member list moved under the same number.
fn hash_position(hasher: &mut blake3::Hasher, position: &QueryPosition) {
    match position {
        QueryPosition::Resource => hash_field(hasher, b"position_resource"),
        QueryPosition::Code => hash_field(hasher, b"position_code"),
        QueryPosition::Method {
            index,
            name,
            descriptor,
        } => {
            hash_field(hasher, b"position_method");
            hasher.update(&index.to_be_bytes());
            hash_field(hasher, &name.0);
            hash_field(hasher, &descriptor.0);
        }
        QueryPosition::Metadata => hash_field(hasher, b"position_metadata"),
        QueryPosition::Bootstrap => hash_field(hasher, b"position_bootstrap"),
        QueryPosition::UnitComplete => hash_field(hasher, b"position_unit_complete"),
    }
}

/// Encodes one query target into the cursor digest.
///
/// Every level is tagged, so two values that would share their raw bytes stay distinct
/// bindings: a `Class` symbol is not a `Field` whose empty name and descriptor pad it to
/// the same bytes, `LiteralValue::String` is not `LiteralValue::Class` over the same bytes,
/// and no target hashes its display spelling. Bytes go through [`hash_field`], integers and
/// floating point values as their big-endian bit pattern, so `-0.0` and each `NaN` payload
/// keep their exact spelling.
fn hash_target(hasher: &mut blake3::Hasher, target: &QueryTarget) {
    match target {
        QueryTarget::Symbol { value } => {
            hash_field(hasher, b"symbol");
            match value {
                SymbolRef::Class { owner } => {
                    hash_field(hasher, b"class");
                    hash_field(hasher, &owner.0);
                }
                SymbolRef::Field {
                    owner,
                    name,
                    descriptor,
                } => {
                    hash_field(hasher, b"field");
                    hash_field(hasher, &owner.0);
                    hash_field(hasher, &name.0);
                    hash_field(hasher, &descriptor.0);
                }
                SymbolRef::Method {
                    owner,
                    name,
                    descriptor,
                } => {
                    hash_field(hasher, b"method");
                    hash_field(hasher, &owner.0);
                    hash_field(hasher, &name.0);
                    hash_field(hasher, &descriptor.0);
                }
            }
        }
        QueryTarget::Literal { value } => {
            hash_field(hasher, b"literal");
            match value {
                LiteralValue::String { value } => {
                    hash_field(hasher, b"string");
                    hash_field(hasher, &value.0);
                }
                LiteralValue::Class { value } => {
                    hash_field(hasher, b"class");
                    hash_field(hasher, &value.0);
                }
                LiteralValue::Integer { value } => {
                    hash_field(hasher, b"integer");
                    hasher.update(&value.to_be_bytes());
                }
                LiteralValue::Long { value } => {
                    hash_field(hasher, b"long");
                    hasher.update(&value.to_be_bytes());
                }
                LiteralValue::Float { value } => {
                    hash_field(hasher, b"float");
                    hasher.update(&value.to_be_bytes());
                }
                LiteralValue::Double { value } => {
                    hash_field(hasher, b"double");
                    hasher.update(&value.to_be_bytes());
                }
            }
        }
    }
}

/// Length-prefixed field: distinct bindings cannot collide by concatenation.
fn hash_field(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    // `usize` never exceeds `u64` on a supported target.
    hasher.update(&(bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

fn hash_count(hasher: &mut blake3::Hasher, count: usize) -> Result<()> {
    let count = u64::try_from(count).map_err(|_| {
        Error::invalid_input(
            "query_cursor_encoding",
            "cursor binding count does not fit u64",
        )
    })?;
    hasher.update(&count.to_be_bytes());
    Ok(())
}

/// Explains a relation P1 does not analyze.
///
/// This is request-level control metadata rather than a scan result: it explains the
/// analysis state itself, so it costs no `ResultItems` and is reported whether or not the
/// scan found a candidate. It says what was not analyzed (no candidate expansion, no
/// definition or dispatch resolution) and does not deny what the scan did observe.
fn relation_unsupported_diagnostic(relation: QueryRelation) -> Diagnostic {
    Diagnostic {
        code: "query_relation_unsupported".into(),
        severity: DiagnosticSeverity::Warning,
        message: format!(
            "P1 does not analyze the {} relation: no candidate expansion or definition resolution is claimed. Raw constant-pool candidates for the requested target are reported as unanalysed evidence.",
            relation_code(relation)
        ),
        provenance: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_RELATIONS: [QueryRelation; 5] = [
        QueryRelation::ConstantPoolContains,
        QueryRelation::MentionsSymbol,
        QueryRelation::LiteralValue,
        QueryRelation::ReferencesDefinition,
        QueryRelation::MayDispatchTo,
    ];

    const ALL_KINDS: [ConsumerKind; 13] = [
        ConsumerKind::Invocation,
        ConsumerKind::Field,
        ConsumerKind::Type,
        ConsumerKind::Constant,
        ConsumerKind::Exception,
        ConsumerKind::Signature,
        ConsumerKind::Annotation,
        ConsumerKind::InnerNest,
        ConsumerKind::Module,
        ConsumerKind::Bootstrap,
        ConsumerKind::Resource,
        ConsumerKind::Verification,
        ConsumerKind::Debug,
    ];

    fn physical() -> PhysicalView {
        PhysicalView {
            snapshot: SnapshotId("snap-1".into()),
            scope: PhysicalScope::SnapshotAll,
        }
    }

    fn boundary(ordinal: u64, item_index: u64) -> QueryBoundary {
        QueryBoundary {
            container: ContainerOrigin {
                snapshot: SnapshotId("snap-1".into()),
                root_container: container(),
                steps: Vec::new(),
            },
            ordinal,
            position: QueryPosition::Resource,
            item_index,
        }
    }

    fn container() -> jarde_reader::model::ContainerId {
        jarde_reader::model::ContainerId("root".into())
    }

    fn symbol_class(owner: &[u8]) -> QueryTarget {
        QueryTarget::Symbol {
            value: SymbolRef::Class {
                owner: JvmBytes(owner.to_vec()),
            },
        }
    }

    fn symbol_field(owner: &[u8], name: &[u8], descriptor: &[u8]) -> QueryTarget {
        QueryTarget::Symbol {
            value: SymbolRef::Field {
                owner: JvmBytes(owner.to_vec()),
                name: JvmBytes(name.to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            },
        }
    }

    fn symbol_method(owner: &[u8], name: &[u8], descriptor: &[u8]) -> QueryTarget {
        QueryTarget::Symbol {
            value: SymbolRef::Method {
                owner: JvmBytes(owner.to_vec()),
                name: JvmBytes(name.to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            },
        }
    }

    fn literal(value: LiteralValue) -> QueryTarget {
        QueryTarget::Literal { value }
    }

    fn digest(
        engine_schema: u16,
        snapshot: &str,
        physical: &PhysicalView,
        relation: QueryRelation,
        target: &QueryTarget,
        consumers: &ConsumerSchema,
        boundary: &QueryBoundary,
    ) -> Digest {
        cursor_digest(
            engine_schema,
            &SnapshotId(snapshot.into()),
            physical,
            relation,
            target,
            consumers,
            boundary,
        )
        .unwrap()
    }

    #[test]
    fn stable_codes_match_the_serde_tags() {
        for relation in ALL_RELATIONS {
            assert_eq!(
                serde_json::to_value(relation).unwrap(),
                serde_json::Value::String(relation_code(relation).into())
            );
        }
        for kind in ALL_KINDS {
            assert_eq!(
                serde_json::to_value(kind).unwrap(),
                serde_json::Value::String(consumer_kind_code(kind).into())
            );
        }
    }

    #[test]
    fn cursor_digest_binds_every_bound_field() {
        let physical = physical();
        let consumers = ConsumerSchema::new(1, [ConsumerKind::Resource]);
        let cursor_boundary = boundary(3, 2);
        let target = symbol_class(b"com/example/Agent");
        let base = digest(
            QUERY_ENGINE_SCHEMA,
            "snap-1",
            &physical,
            QueryRelation::MentionsSymbol,
            &target,
            &consumers,
            &cursor_boundary,
        );

        assert_ne!(
            base,
            digest(
                QUERY_ENGINE_SCHEMA + 1,
                "snap-1",
                &physical,
                QueryRelation::MentionsSymbol,
                &target,
                &consumers,
                &cursor_boundary
            )
        );
        assert_ne!(
            base,
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-2",
                &physical,
                QueryRelation::MentionsSymbol,
                &target,
                &consumers,
                &cursor_boundary
            )
        );
        let tree = PhysicalView {
            snapshot: SnapshotId("snap-1".into()),
            scope: PhysicalScope::ArtifactTree {
                root_container: container(),
            },
        };
        assert_ne!(
            base,
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &tree,
                QueryRelation::MentionsSymbol,
                &target,
                &consumers,
                &cursor_boundary
            )
        );
        assert_ne!(
            base,
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &physical,
                QueryRelation::LiteralValue,
                &target,
                &consumers,
                &cursor_boundary
            )
        );
        assert_ne!(
            base,
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &physical,
                QueryRelation::MentionsSymbol,
                &target,
                &ConsumerSchema::new(1, [ConsumerKind::Resource, ConsumerKind::Type]),
                &cursor_boundary
            )
        );
        assert_ne!(
            base,
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &physical,
                QueryRelation::MentionsSymbol,
                &target,
                &ConsumerSchema::new(2, [ConsumerKind::Resource]),
                &cursor_boundary
            )
        );
        assert_ne!(
            base,
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &physical,
                QueryRelation::MentionsSymbol,
                &target,
                &consumers,
                &boundary(4, 2)
            )
        );
        assert_ne!(
            base,
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &physical,
                QueryRelation::MentionsSymbol,
                &target,
                &consumers,
                &boundary(3, 3)
            )
        );
        // The step position is part of the boundary binding: the same unit and the same
        // published count under another step is another continuation.
        let mut elsewhere = boundary(3, 2);
        elsewhere.position = QueryPosition::Metadata;
        assert_ne!(
            base,
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &physical,
                QueryRelation::MentionsSymbol,
                &target,
                &consumers,
                &elsewhere
            )
        );
        let mut other_member = boundary(3, 2);
        other_member.position = QueryPosition::Method {
            index: 0,
            name: JvmBytes(b"run".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        };
        let mut same_index_other_name = other_member.clone();
        same_index_other_name.position = QueryPosition::Method {
            index: 0,
            name: JvmBytes(b"walk".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        };
        let mut same_name_other_descriptor = other_member.clone();
        same_name_other_descriptor.position = QueryPosition::Method {
            index: 0,
            name: JvmBytes(b"run".to_vec()),
            descriptor: JvmBytes(b"(I)V".to_vec()),
        };
        assert_ne!(
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &physical,
                QueryRelation::MentionsSymbol,
                &target,
                &consumers,
                &other_member
            ),
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &physical,
                QueryRelation::MentionsSymbol,
                &target,
                &consumers,
                &same_index_other_name
            )
        );
        assert_ne!(
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &physical,
                QueryRelation::MentionsSymbol,
                &target,
                &consumers,
                &other_member
            ),
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &physical,
                QueryRelation::MentionsSymbol,
                &target,
                &consumers,
                &same_name_other_descriptor
            )
        );
        // Absent and empty consumer sets are distinct bindings.
        assert_ne!(
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &physical,
                QueryRelation::MentionsSymbol,
                &target,
                &ConsumerSchema::new(1, []),
                &cursor_boundary
            ),
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &physical,
                QueryRelation::MentionsSymbol,
                &target,
                &ConsumerSchema::new(1, [ConsumerKind::Resource]),
                &cursor_boundary
            )
        );
        assert_eq!(
            base,
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &physical,
                QueryRelation::MentionsSymbol,
                &target,
                &consumers,
                &cursor_boundary
            )
        );

        // The target is bound by raw bytes and bit patterns, not by a display spelling.
        let with_target = |target: &QueryTarget| {
            digest(
                QUERY_ENGINE_SCHEMA,
                "snap-1",
                &physical,
                QueryRelation::MentionsSymbol,
                target,
                &consumers,
                &cursor_boundary,
            )
        };
        // Every symbol dimension is part of the identity.
        assert_ne!(base, with_target(&symbol_class(b"com/example/Other")));
        assert_ne!(
            base,
            with_target(&symbol_method(b"com/example/Agent", b"call", b"()V"))
        );
        assert_ne!(
            with_target(&symbol_field(b"e", b"n", b"()V")),
            with_target(&symbol_field(b"e", b"other", b"()V"))
        );
        assert_ne!(
            with_target(&symbol_field(b"e", b"n", b"()V")),
            with_target(&symbol_field(b"e", b"n", b"(I)V"))
        );
        assert_ne!(
            with_target(&symbol_field(b"e", b"n", b"()V")),
            with_target(&symbol_field(b"other", b"n", b"()V"))
        );
        // A variant tag separates symbol shapes that share their raw bytes, so a class
        // never collides with a field or method whose empty dimensions pad it to the
        // same bytes, and a field never collides with a method.
        assert_ne!(
            with_target(&symbol_class(b"e")),
            with_target(&symbol_field(b"e", b"", b""))
        );
        assert_ne!(
            with_target(&symbol_field(b"e", b"n", b"()V")),
            with_target(&symbol_method(b"e", b"n", b"()V"))
        );
        // The same bytes under a different literal kind stay distinct.
        assert_ne!(
            with_target(&literal(LiteralValue::String {
                value: JvmBytes(b"e".to_vec())
            })),
            with_target(&literal(LiteralValue::Class {
                value: JvmBytes(b"e".to_vec())
            }))
        );
        assert_ne!(
            with_target(&symbol_class(b"e")),
            with_target(&literal(LiteralValue::String {
                value: JvmBytes(b"e".to_vec())
            }))
        );
        assert_ne!(
            with_target(&literal(LiteralValue::String {
                value: JvmBytes(b"e".to_vec())
            })),
            with_target(&literal(LiteralValue::String {
                value: JvmBytes(b"f".to_vec())
            }))
        );
        // Integer and long compare their two's-complement value, float and double their
        // exact bit pattern, so the kind, `-0.0` and every NaN payload stay distinct.
        assert_ne!(
            with_target(&literal(LiteralValue::Integer { value: 7 })),
            with_target(&literal(LiteralValue::Integer { value: 8 }))
        );
        assert_ne!(
            with_target(&literal(LiteralValue::Integer { value: 7 })),
            with_target(&literal(LiteralValue::Long { value: 7 }))
        );
        assert_ne!(
            with_target(&literal(LiteralValue::Float { value: 0x0000_0000 })),
            with_target(&literal(LiteralValue::Float { value: 0x8000_0000 }))
        );
        assert_ne!(
            with_target(&literal(LiteralValue::Float { value: 0x7fc0_0000 })),
            with_target(&literal(LiteralValue::Float { value: 0x7fc0_0001 }))
        );
        assert_ne!(
            with_target(&literal(LiteralValue::Double {
                value: 0x0000_0000_0000_0000
            })),
            with_target(&literal(LiteralValue::Double {
                value: 0x8000_0000_0000_0000
            }))
        );
        assert_ne!(
            with_target(&literal(LiteralValue::Double {
                value: 0x7ff8_0000_0000_0000
            })),
            with_target(&literal(LiteralValue::Double {
                value: 0x7ff8_0000_0000_0001
            }))
        );
        assert_ne!(
            with_target(&literal(LiteralValue::Float { value: 0 })),
            with_target(&literal(LiteralValue::Double { value: 0 }))
        );
        // One target always encodes to one digest.
        assert_eq!(
            with_target(&symbol_class(b"com/example/Agent")),
            with_target(&symbol_class(b"com/example/Agent"))
        );
    }

    #[test]
    fn payload_enums_round_trip_and_reject_unknown_fields() {
        let target = QueryTarget::Symbol {
            value: SymbolRef::Method {
                owner: JvmBytes(b"com/example/Main".to_vec()),
                name: JvmBytes(b"main".to_vec()),
                descriptor: JvmBytes(b"([Ljava/lang/String;)V".to_vec()),
            },
        };
        let json = serde_json::to_string(&target).unwrap();
        assert!(json.starts_with("{\"kind\":\"symbol\",\"value\":{\"kind\":\"method\""));
        assert_eq!(serde_json::from_str::<QueryTarget>(&json).unwrap(), target);

        let literal = LiteralValue::Float {
            value: 0x7fc0_0000_u32,
        };
        let json = serde_json::to_string(&literal).unwrap();
        assert_eq!(json, "{\"kind\":\"float\",\"value\":2143289344}");
        assert_eq!(
            serde_json::from_str::<LiteralValue>(&json).unwrap(),
            literal
        );
        assert!(
            serde_json::from_str::<LiteralValue>("{\"kind\":\"float\",\"value\":1,\"extra\":1}")
                .is_err()
        );
        assert!(serde_json::from_str::<LiteralValue>("{\"kind\":\"short\",\"value\":1}").is_err());

        let analysis = QueryAnalysis::UnsupportedAnalysis {
            relation: QueryRelation::MayDispatchTo,
        };
        let json = serde_json::to_string(&analysis).unwrap();
        assert_eq!(
            json,
            "{\"kind\":\"unsupported_analysis\",\"relation\":\"may_dispatch_to\"}"
        );
        assert_eq!(
            serde_json::from_str::<QueryAnalysis>(&json).unwrap(),
            analysis
        );
        assert!(
            serde_json::from_str::<QueryAnalysis>(
                "{\"kind\":\"unsupported_analysis\",\"relation\":\"may_dispatch_to\",\"resolved\":true}"
            )
            .is_err()
        );

        let page = serde_json::json!({
            "has_more": true,
            "returned_items": 1,
            "unexpected": false
        });
        assert!(serde_json::from_value::<QueryPage>(page).is_err());
    }
}
