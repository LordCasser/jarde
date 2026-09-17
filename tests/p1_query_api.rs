//! P1 query API acceptance: request validation, relation dispatch, the resource
//! consumer, the raw constant-pool evidence kept for the relations P1 cannot resolve,
//! paging, budget/cancellation semantics and JSON stability.
//!
//! Fixtures are built in memory with the `rawzip` writer, so every assertion runs
//! against a real snapshot, a real enumeration and the public `Engine::query`
//! entry point.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const QUERY_LIMITS: u64 = 10_000;

const MANIFEST_LINES: [&str; 8] = [
    "Manifest-Version: 1.0",
    "Main-Class: com.example.Main",
    "Class-Path: lib/one.jar lib/two.jar",
    "Automatic-Module-Name: com.example.app",
    "Launcher-Agent-Class: com.example.Agent",
    "Premain-Class: com.example.Agent",
    "Agent-Class: com.example.Agent",
    "Multi-Release: true",
];
const NAMED_SECTION: [&str; 2] = ["Name: com/example/Agent", "Main-Class: should.not.appear"];
const SERVICE_KEY: &str = "com.example.Service";

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 24,
        archive_entries: 10_000,
        entry_bytes: 1 << 24,
        read_bytes: 1 << 24,
        class_bytes: 1 << 24,
        attribute_bytes: 1 << 24,
        code_bytes: 1 << 24,
        result_items: QUERY_LIMITS,
        output_bytes: 1 << 24,
        nested_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn u16b(v: &mut Vec<u8>, n: u16) {
    v.extend_from_slice(&n.to_be_bytes());
}

fn utf8(v: &mut Vec<u8>, s: &[u8]) {
    v.push(1);
    u16b(v, s.len() as u16);
    v.extend_from_slice(s);
}

/// Minimal, complete class file, so unrelated entries stay realistic.
fn class() -> Vec<u8> {
    let mut v = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut v, 0);
    u16b(&mut v, 52);
    u16b(&mut v, 5);
    utf8(&mut v, b"p/A");
    v.push(7);
    u16b(&mut v, 1);
    utf8(&mut v, b"java/lang/Object");
    v.push(7);
    u16b(&mut v, 3);
    u16b(&mut v, 0x21);
    u16b(&mut v, 2);
    u16b(&mut v, 4);
    u16b(&mut v, 0);
    u16b(&mut v, 0);
    u16b(&mut v, 0);
    u16b(&mut v, 0);
    v
}

/// Class-file builder for the raw constant-pool candidate fixtures.
///
/// Entries are appended in index order, so a test can name the exact index of a pool
/// entry and slice it out of the finished bytes. The finished class declares
/// `java/lang/Object` as its superclass and one method per [`MethodFixture`].
struct ClassFile {
    pool: Vec<u8>,
    next: u16,
}

/// One declared method with a single `Code` attribute.
struct MethodFixture {
    name: &'static str,
    descriptor: &'static str,
    code: Vec<u8>,
}

impl ClassFile {
    fn new() -> Self {
        Self {
            pool: Vec::new(),
            next: 1,
        }
    }

    fn entry(&mut self, bytes: &[u8]) -> u16 {
        let index = self.next;
        self.pool.extend_from_slice(bytes);
        self.next += 1;
        index
    }

    /// `Long` or `Double` entry, which reserves the pool slot after it.
    fn wide_entry(&mut self, bytes: &[u8]) -> u16 {
        let index = self.next;
        self.pool.extend_from_slice(bytes);
        self.next += 2;
        index
    }

    fn utf8(&mut self, text: &str) -> u16 {
        let mut entry = vec![1];
        u16b(&mut entry, text.len() as u16);
        entry.extend_from_slice(text.as_bytes());
        self.entry(&entry)
    }

    fn class(&mut self, name: &str) -> u16 {
        let name = self.utf8(name);
        let mut entry = vec![7];
        u16b(&mut entry, name);
        self.entry(&entry)
    }

    fn name_and_type(&mut self, name: &str, descriptor: &str) -> u16 {
        let name = self.utf8(name);
        let descriptor = self.utf8(descriptor);
        let mut entry = vec![12];
        u16b(&mut entry, name);
        u16b(&mut entry, descriptor);
        self.entry(&entry)
    }

    /// Member reference with the given tag: 10 for `Methodref`, 11 for
    /// `InterfaceMethodref`. Both answer the same `SymbolRef::Method`, because
    /// class-versus-interface is a resolution detail, not a raw query dimension.
    fn member_ref(&mut self, tag: u8, owner: &str, name: &str, descriptor: &str) -> u16 {
        let class = self.class(owner);
        let name_and_type = self.name_and_type(name, descriptor);
        let mut entry = vec![tag];
        u16b(&mut entry, class);
        u16b(&mut entry, name_and_type);
        self.entry(&entry)
    }

    fn method_ref(&mut self, owner: &str, name: &str, descriptor: &str) -> u16 {
        self.member_ref(10, owner, name, descriptor)
    }

    fn interface_method_ref(&mut self, owner: &str, name: &str, descriptor: &str) -> u16 {
        self.member_ref(11, owner, name, descriptor)
    }

    /// Completes the class around the pooled `this_class` entry.
    fn finish(mut self, this: u16, methods: &[MethodFixture]) -> Vec<u8> {
        let super_class = self.class("java/lang/Object");
        let code_name = self.utf8("Code");
        let declared = methods
            .iter()
            .map(|method| {
                (
                    self.utf8(method.name),
                    self.utf8(method.descriptor),
                    method.code.as_slice(),
                )
            })
            .collect::<Vec<_>>();
        let mut out = 0xcafebabe_u32.to_be_bytes().to_vec();
        u16b(&mut out, 0);
        u16b(&mut out, 52);
        u16b(&mut out, self.next);
        out.extend_from_slice(&self.pool);
        u16b(&mut out, 0x21);
        u16b(&mut out, this);
        u16b(&mut out, super_class);
        u16b(&mut out, 0);
        u16b(&mut out, 0);
        u16b(&mut out, methods.len() as u16);
        for (name, descriptor, code) in declared {
            u16b(&mut out, 0x0009);
            u16b(&mut out, name);
            u16b(&mut out, descriptor);
            u16b(&mut out, 1);
            u16b(&mut out, code_name);
            let mut attribute = Vec::new();
            u16b(&mut attribute, 2);
            u16b(&mut attribute, 1);
            attribute.extend_from_slice(&(code.len() as u32).to_be_bytes());
            attribute.extend_from_slice(code);
            u16b(&mut attribute, 0);
            u16b(&mut attribute, 0);
            out.extend_from_slice(&(attribute.len() as u32).to_be_bytes());
            out.extend_from_slice(&attribute);
        }
        u16b(&mut out, 0);
        out
    }
}

/// `invokestatic <index>` followed by `return`.
fn invokestatic(index: u16) -> Vec<u8> {
    let mut code = vec![0xb8];
    u16b(&mut code, index);
    code.push(0xb1);
    code
}

/// Target for a member symbol.
fn method_target(owner: &str, name: &str, descriptor: &str) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Method {
            owner: JvmBytes(owner.as_bytes().to_vec()),
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
    }
}

/// Target for a field symbol.
fn field_target(owner: &str, name: &str, descriptor: &str) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Field {
            owner: JvmBytes(owner.as_bytes().to_vec()),
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
    }
}

/// Target for a literal value.
fn literal_target(value: LiteralValue) -> QueryTarget {
    QueryTarget::Literal { value }
}

/// Manifest text with the given line ending and an optional named section.
fn manifest(ending: &str, with_named_section: bool) -> Vec<u8> {
    let mut out = Vec::new();
    for line in MANIFEST_LINES {
        out.extend_from_slice(line.as_bytes());
        out.extend_from_slice(ending.as_bytes());
    }
    out.extend_from_slice(ending.as_bytes());
    if with_named_section {
        for line in NAMED_SECTION {
            out.extend_from_slice(line.as_bytes());
            out.extend_from_slice(ending.as_bytes());
        }
    }
    out
}

/// Service registrations: a comment, a blank line, a plain provider, a provider
/// with a trailing comment and a provider split over two lines.
fn services() -> Vec<u8> {
    let mut out = Vec::new();
    for line in [
        "# provider registrations",
        "",
        "com.example.ProviderA",
        "com.example.Agent # trailing comment",
        "com.example.Agent",
        "com.example.Continued.",
        "Provider",
    ] {
        out.extend_from_slice(line.as_bytes());
        out.extend_from_slice(b"\r\n");
    }
    out
}

fn zip(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for &(name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(data).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        archive.finish().unwrap();
    }
    output.into_inner()
}

/// Archive holding only the given `.class` entries.
fn class_archive(entries: &[(&[u8], Vec<u8>)]) -> Vec<u8> {
    let borrowed: Vec<(&[u8], &[u8])> = entries
        .iter()
        .map(|(name, bytes)| (*name, bytes.as_slice()))
        .collect();
    zip(&borrowed)
}

fn fixture(ending: &str, with_named_section: bool) -> Vec<u8> {
    let manifest = manifest(ending, with_named_section);
    let services = services();
    let class = class();
    zip(&[
        (b"META-INF/MANIFEST.MF", &manifest),
        (b"META-INF/services/com.example.Service", &services),
        (b"p/A.class", &class),
        (b"docs/readme.txt", b"not a resource"),
    ])
}

/// Archive whose only class carries one entry of every literal kind, plus a second entry
/// so a page limit stops the scan and hands out a cursor.
///
/// `Long` and `Double` reserve the pool slot after them; `wide_entry` counts that slot, so
/// the fixture stays a well-formed class file whose class marker, `Integer` and floating
/// point entries keep their right indices.
fn literal_archive() -> Vec<u8> {
    let mut class_file = ClassFile::new();
    let this = class_file.class("p/L");
    class_file.entry(&[3, 0, 0, 0, 7]); // Integer 7
    class_file.wide_entry(&[5, 0, 0, 0, 0, 0, 0, 0, 8]); // Long 8
    class_file.entry(&[4, 0x00, 0x00, 0x00, 0x00]); // Float 0.0
    class_file.wide_entry(&[6, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // Double 0.0
    class_file.entry(&[4, 0x7f, 0xc0, 0x00, 0x00]); // Float NaN payload
    class_file.wide_entry(&[6, 0x7f, 0xf8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]); // Double NaN payload
    class_archive(&[
        (b"p/L.class", class_file.finish(this, &[])),
        (b"docs/readme.txt", b"not a resource".to_vec()),
    ])
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    let mut budget = Budget::new(limits());
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .unwrap()
}

fn physical(snapshot: &ArtifactSnapshot) -> PhysicalView {
    PhysicalView {
        snapshot: snapshot.id().clone(),
        scope: PhysicalScope::SnapshotAll,
    }
}

fn symbol(owner: &str) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Class {
            owner: JvmBytes(owner.as_bytes().to_vec()),
        },
    }
}

fn literal(value: &str) -> QueryTarget {
    QueryTarget::Literal {
        value: LiteralValue::String {
            value: JvmBytes(value.as_bytes().to_vec()),
        },
    }
}

fn request_with(
    snapshot: &ArtifactSnapshot,
    relation: QueryRelation,
    target: QueryTarget,
    kinds: &[ConsumerKind],
    max_items: u64,
) -> QueryRequest {
    QueryRequest {
        relation,
        target,
        physical: physical(snapshot),
        consumers: ConsumerSchema::new(1, kinds.to_vec()),
        max_items,
        cursor: None,
    }
}

fn run(snapshot: &ArtifactSnapshot, request: &QueryRequest) -> QueryReport {
    let mut budget = Budget::new(limits());
    Engine::new()
        .query(snapshot, request, &mut budget)
        .expect("query must succeed")
}

fn error_code(error: &Error) -> String {
    match error {
        Error::InvalidInput { code, .. } | Error::Unsupported { code, .. } => code.clone(),
        other => panic!("unexpected error {other:?}"),
    }
}

fn entry_ids(snapshot: &ArtifactSnapshot) -> Vec<PhysicalEntryId> {
    let mut budget = Budget::new(limits());
    snapshot
        .enumerate(&mut budget)
        .unwrap()
        .entries
        .iter()
        .map(|entry| entry.id.clone())
        .collect()
}

fn offset_of(haystack: &[u8], needle: &[u8]) -> u64 {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
        .expect("fixture text must be present") as u64
}

fn resource_location(item: &XrefItem) -> (&PhysicalEntryId, ByteSpan) {
    match &item.source.location {
        Location::Resource { entry, span } => (entry, span.clone()),
        other => panic!("expected a resource location, got {other:?}"),
    }
}

fn operations(report: &QueryReport) -> Vec<XrefOperation> {
    report.items.iter().map(|item| item.operation).collect()
}

/// Entry of a fixture archive by raw name.
fn entry_named(snapshot: &ArtifactSnapshot, name: &[u8]) -> PhysicalEntry {
    let mut budget = Budget::new(limits());
    snapshot
        .enumerate(&mut budget)
        .unwrap()
        .entries
        .into_iter()
        .find(|entry| entry.id.raw_name.0 == name)
        .expect("fixture entry is present")
}

/// Asserts one item is a raw X0 candidate and returns its pool index and span.
///
/// The candidate answers the caller's relation, names no consumer category, carries no
/// BCI and no opcode, and its span addresses the constant-pool entry inside the class
/// file the caller can slice themselves.
fn assert_pool_candidate(
    item: &XrefItem,
    relation: QueryRelation,
    class_bytes: &[u8],
) -> (u16, ByteSpan) {
    assert_eq!(item.relation, relation);
    assert_eq!(item.derivation, XrefDerivation::ConstantPoolCandidate);
    assert_eq!(
        item.consumer, None,
        "a raw pool candidate is not produced by any consumer"
    );
    assert_eq!(item.operation, XrefOperation::ConstantPoolEntry);
    assert_eq!(item.resolution, QueryResolution::NotRequested);
    assert_eq!(item.certainty, XrefCertainty::Exact);
    assert_eq!(item.evidence.bci, None);
    assert_eq!(item.evidence.opcode, None);
    assert_eq!(item.evidence.attribute, None);
    assert!(item.evidence.via.is_empty());
    let index = item
        .evidence
        .constant_pool_index
        .expect("a pool candidate names its entry");
    let span = item
        .evidence
        .span
        .clone()
        .expect("a pool candidate has a span");
    assert!(
        span.start + span.length <= class_bytes.len() as u64,
        "the span addresses the class file the caller holds"
    );
    (index, span)
}

#[test]
fn unsupported_relations_report_the_analysis_state_without_resolving() {
    // The fixture class pool carries `p/A` and `java/lang/Object` only, so the target has
    // no pool entry at all: the raw probe runs, finds nothing, and the report says why
    // that is an analysis state rather than a resolved "no reference" answer.
    let snapshot = open(fixture("\r\n", true));
    for relation in [
        QueryRelation::ReferencesDefinition,
        QueryRelation::MayDispatchTo,
    ] {
        let request = request_with(
            &snapshot,
            relation,
            symbol("com/example/Agent"),
            &[ConsumerKind::Resource],
            0,
        );
        let report = run(&snapshot, &request);
        assert_eq!(
            report.analysis,
            QueryAnalysis::UnsupportedAnalysis { relation }
        );
        assert_eq!(report.relation, relation);
        assert!(
            report.items.is_empty(),
            "the class pool holds no entry for this target"
        );
        assert_eq!(
            report
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "query_relation_unsupported")
                .count(),
            1
        );
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        assert!(!report.page.has_more);
        assert!(report.page.cursor.is_none());
        assert_eq!(report.page.returned_items, 0);
        // The raw probe really ran, so structural coverage describes the entries it
        // examined instead of staying not-requested; resolution stays out of scope.
        assert_eq!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );
        assert_eq!(
            report.coverage.dimensions.runtime_resolution.state,
            CoverageState::NotRequested
        );
        assert_eq!(
            report.coverage.dimensions.dynamic_analysis.state,
            CoverageState::NotRequested
        );
        assert_eq!(report.coverage.scanned_items, 0);
        assert_eq!(report.coverage.unknown_candidates, 0);
        assert_eq!(report.consumers, request.consumers);
    }
}

#[test]
fn request_validation_uses_stable_codes() {
    let snapshot = open(fixture("\r\n", true));
    let other = open(fixture("\r\n", false));

    // Snapshot identity.
    let mut request = request_with(
        &snapshot,
        QueryRelation::MentionsSymbol,
        symbol("com/example/Agent"),
        &[ConsumerKind::Resource],
        0,
    );
    request.physical.snapshot = other.id().clone();
    let mut budget = Budget::new(limits());
    let error = Engine::new()
        .query(&snapshot, &request, &mut budget)
        .unwrap_err();
    assert_eq!(error_code(&error), "query_snapshot_mismatch");

    // Artifact-tree root.
    let mut request = request_with(
        &snapshot,
        QueryRelation::MentionsSymbol,
        symbol("com/example/Agent"),
        &[ConsumerKind::Resource],
        0,
    );
    request.physical.scope = PhysicalScope::ArtifactTree {
        root_container: ContainerId("not-the-root".into()),
    };
    let error = Engine::new()
        .query(&snapshot, &request, &mut Budget::new(limits()))
        .unwrap_err();
    assert_eq!(error_code(&error), "query_artifact_tree_root_mismatch");

    // Target/relation shape.
    for (relation, target) in [
        (QueryRelation::LiteralValue, symbol("com/example/Main")),
        (QueryRelation::MentionsSymbol, literal("lib/one.jar")),
        (QueryRelation::ReferencesDefinition, literal("lib/one.jar")),
    ] {
        let request = request_with(&snapshot, relation, target, &[ConsumerKind::Resource], 0);
        let error = Engine::new()
            .query(&snapshot, &request, &mut Budget::new(limits()))
            .unwrap_err();
        assert_eq!(error_code(&error), "query_target_relation_mismatch");
    }

    // Consumer schema version.
    let mut request = request_with(
        &snapshot,
        QueryRelation::MentionsSymbol,
        symbol("com/example/Agent"),
        &[ConsumerKind::Resource],
        0,
    );
    request.consumers = ConsumerSchema::new(2, [ConsumerKind::Resource]);
    let error = Engine::new()
        .query(&snapshot, &request, &mut Budget::new(limits()))
        .unwrap_err();
    assert!(matches!(error, Error::Unsupported { .. }));
    assert_eq!(error_code(&error), "query_consumer_schema_version");
}

// ---------------------------------------------------------------------------
// A18: an open snapshot is byte-stable, a reopened one is a new query identity
// ---------------------------------------------------------------------------

/// Minimal unique temp directory; the snapshot contract is about a real file path.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static UNIQUE: AtomicU64 = AtomicU64::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos();
        let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-p1-query-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&path).expect("create temp directory");
        Self(path)
    }

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, bytes).expect("write fixture");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a18_open_snapshot_stays_stable_and_a_reopened_one_rejects_old_cursors() {
    let temp = TempDir::new();
    let path = temp.write("app.zip", &fixture("\r\n", true));
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::Path(path.clone()),
            &mut Budget::new(limits()),
        )
        .expect("the path opens");
    let request = request_with(
        &snapshot,
        QueryRelation::MentionsSymbol,
        symbol("com/example/Agent"),
        &[ConsumerKind::Resource],
        1,
    );
    let first = run(&snapshot, &request);
    let cursor = first
        .page
        .cursor
        .clone()
        .expect("a truncated page carries a cursor");
    let first_items = first.items.clone();
    assert!(!first_items.is_empty());
    // 3.1 onward: the cursor is public structured state, bound to the full query identity
    // (target included) under the current engine schema.
    assert_eq!(cursor.engine_schema, QUERY_ENGINE_SCHEMA);
    assert_eq!(QUERY_ENGINE_SCHEMA, 2, "the target-bound cursor generation");
    assert_eq!(cursor.target, request.target);
    assert!(
        serde_json::to_value(&cursor)
            .unwrap()
            .get("target")
            .is_some(),
        "the published cursor carries its target"
    );

    // The path changes on disk after the snapshot was opened.
    std::fs::write(&path, fixture("\n", false)).expect("replace the fixture file");

    // The open snapshot owns its bytes: the same request answers the same facts, and the
    // old cursor still continues the old query instead of failing or drifting.
    let again = run(&snapshot, &request);
    assert_eq!(again.items, first_items);
    assert_eq!(&again.physical.snapshot, snapshot.id());
    let mut continuation = request.clone();
    continuation.max_items = 0;
    continuation.cursor = Some(cursor.clone());
    let next = run(&snapshot, &continuation);
    assert!(
        matches!(next.execution, ExecutionReport::Complete { .. }),
        "{:?}",
        next.execution
    );
    assert_eq!(
        next.items.len() + first_items.len(),
        5,
        "the continuation publishes the remaining resources of the same snapshot"
    );

    // A reopened snapshot is the new bytes, so it is a different query identity.
    let reopened = engine
        .open(ArtifactInput::Path(path), &mut Budget::new(limits()))
        .expect("the changed path reopens");
    assert_ne!(reopened.id(), snapshot.id());
    assert_ne!(reopened.id().0, snapshot.id().0);

    // A request still bound to the old snapshot is rejected before any scan.
    let stale_request = request_with(
        &snapshot,
        QueryRelation::MentionsSymbol,
        symbol("com/example/Agent"),
        &[ConsumerKind::Resource],
        0,
    );
    let mut probe = Budget::new(limits());
    let error = engine
        .query(&reopened, &stale_request, &mut probe)
        .unwrap_err();
    assert_eq!(error_code(&error), "query_snapshot_mismatch");
    assert_eq!(probe.usage().archive_entries, 0);
    assert_eq!(probe.usage().result_items, 0);

    // A request bound to the new snapshot but carrying the old snapshot's cursor is
    // rejected by the cursor binding, before the scan reads or bills anything.
    let mut stale_cursor = request_with(
        &reopened,
        QueryRelation::MentionsSymbol,
        symbol("com/example/Agent"),
        &[ConsumerKind::Resource],
        0,
    );
    stale_cursor.cursor = Some(cursor);
    let mut probe = Budget::new(limits());
    let error = engine
        .query(&reopened, &stale_cursor, &mut probe)
        .unwrap_err();
    assert_eq!(error_code(&error), "query_cursor_mismatch");
    assert!(
        error.to_string().contains("cursor snapshot"),
        "the error names the cross-snapshot binding: {error}"
    );
    assert_eq!(probe.usage().archive_entries, 0);
    assert_eq!(probe.usage().result_items, 0);

    // The fresh identity is still queryable without a cursor.
    let fresh = run(
        &reopened,
        &request_with(
            &reopened,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Agent"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    assert_eq!(fresh.items.len(), 5);
    assert_eq!(&fresh.physical.snapshot, reopened.id());
}

#[test]
fn cursor_mismatches_bind_snapshot_view_relation_and_schema() {
    let snapshot = open(fixture("\r\n", true));
    let other = open(fixture("\n", false));
    let request = request_with(
        &snapshot,
        QueryRelation::MentionsSymbol,
        symbol("com/example/Agent"),
        &[ConsumerKind::Resource],
        1,
    );
    let first = run(&snapshot, &request);
    let cursor = first
        .page
        .cursor
        .clone()
        .expect("a truncated page carries a cursor");
    assert_eq!(cursor.engine_schema, QUERY_ENGINE_SCHEMA);
    assert_eq!(&cursor.snapshot, snapshot.id());
    assert_eq!(cursor.physical, request.physical);
    assert_eq!(cursor.relation, request.relation);
    assert_eq!(cursor.consumers, request.consumers);

    let bound = |cursor: QueryCursor| {
        let mut request = request.clone();
        request.cursor = Some(cursor);
        request
    };
    let cases: Vec<(&str, QueryCursor)> = vec![
        ("engine schema", {
            let mut cursor = cursor.clone();
            cursor.engine_schema = QUERY_ENGINE_SCHEMA + 1;
            cursor
        }),
        ("snapshot", {
            let mut cursor = cursor.clone();
            cursor.snapshot = other.id().clone();
            cursor
        }),
        ("physical view", {
            let mut cursor = cursor.clone();
            cursor.physical.scope = PhysicalScope::ArtifactTree {
                root_container: ContainerId("root".into()),
            };
            cursor
        }),
        ("relation", {
            let mut cursor = cursor.clone();
            cursor.relation = QueryRelation::LiteralValue;
            cursor
        }),
        ("consumer schema", {
            let mut cursor = cursor.clone();
            cursor.consumers = ConsumerSchema::new(1, [ConsumerKind::Type]);
            cursor
        }),
        ("digest", {
            let mut cursor = cursor.clone();
            cursor.boundary.item_index += 1;
            cursor
        }),
        // The target is part of the cursor identity too. A client holds the cursor as the
        // public JSON value, so the binding is changed through that form: the request keeps
        // its own target while the cursor claims another one.
        ("target", {
            let mut value = serde_json::to_value(&cursor).unwrap();
            value["target"] = serde_json::to_value(symbol("com/example/Main")).unwrap();
            serde_json::from_value(value).unwrap()
        }),
    ];
    for (bound_field, mismatched) in cases {
        let error = Engine::new()
            .query(&snapshot, &bound(mismatched), &mut Budget::new(limits()))
            .unwrap_err();
        assert_eq!(error_code(&error), "query_cursor_mismatch", "{bound_field}");
        assert!(
            error.to_string().contains(&format!("cursor {bound_field}")),
            "error must name the mismatched binding: {error}"
        );
    }

    // The cursor keeps its target in the public JSON form, and a value without the field
    // is refused instead of being replayed with an unverified target binding.
    let mut value = serde_json::to_value(&cursor).unwrap();
    assert!(
        value.get("target").is_some(),
        "the cursor must publish its target: {value}"
    );
    assert_eq!(
        serde_json::from_value::<QueryCursor>(value.clone()).unwrap(),
        cursor
    );
    value.as_object_mut().unwrap().remove("target");
    assert!(
        serde_json::from_value::<QueryCursor>(value).is_err(),
        "a cursor without a target must not deserialize"
    );
}

/// R1: the cursor is bound to the query target, so a continuation for another target is
/// rejected before any unit is read.
///
/// Target A publishes one item on the manifest unit and stops at the page limit. Target B
/// answers exactly one manifest fact of its own, which lives in that same unit. Reusing A's
/// cursor for B replays the unit and skips B's published prefix, so the old engine returned
/// the B page as if it were complete.
#[test]
fn cursor_for_another_target_is_rejected_before_the_scan() {
    let snapshot = open(fixture("\r\n", true));
    let first = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Agent"),
            &[ConsumerKind::Resource],
            1,
        ),
    );
    assert_eq!(first.items.len(), 1);
    assert!(first.page.has_more);
    let cursor = first
        .page
        .cursor
        .clone()
        .expect("a truncated page carries a cursor");

    // A fresh scan for target B finds the one Main-Class fact of the clear-text section.
    let full_b = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Main"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    assert_eq!(full_b.items.len(), 1);

    let mut continuation = request_with(
        &snapshot,
        QueryRelation::MentionsSymbol,
        symbol("com/example/Main"),
        &[ConsumerKind::Resource],
        0,
    );
    continuation.cursor = Some(cursor);
    let outcome = Engine::new().query(&snapshot, &continuation, &mut Budget::new(limits()));
    match outcome {
        Err(error) => {
            assert_eq!(error_code(&error), "query_cursor_mismatch");
            assert!(
                error.to_string().contains("cursor target does not match"),
                "the error must name the target binding: {error}"
            );
        }
        Ok(report) => panic!(
            "a continuation that changes the target must be rejected before the scan, but the \
             scan ran and claimed the new target's items: items={} (the fresh scan returns {}), \
             execution={:?}, artifact_structural={:?}, diagnostics={:?}",
            report.items.len(),
            full_b.items.len(),
            report.execution,
            report.coverage.dimensions.artifact_structural.state,
            report.diagnostics,
        ),
    }
}

/// First page cursor of one symbol query against the manifest fixture.
fn first_page_cursor(snapshot: &ArtifactSnapshot, target: QueryTarget) -> QueryCursor {
    let report = run(
        snapshot,
        &request_with(
            snapshot,
            QueryRelation::MentionsSymbol,
            target,
            &[ConsumerKind::Resource],
            1,
        ),
    );
    assert_eq!(report.items.len(), 1, "the fixture must answer this target");
    assert!(report.page.has_more);
    report
        .page
        .cursor
        .clone()
        .expect("a truncated page carries a cursor")
}

/// First page cursor of one literal raw-pool query against [`literal_archive`].
fn literal_cursor(snapshot: &ArtifactSnapshot, target: QueryTarget) -> QueryCursor {
    let report = run(
        snapshot,
        &request_with(
            snapshot,
            QueryRelation::ConstantPoolContains,
            target,
            &[ConsumerKind::Constant],
            1,
        ),
    );
    assert_eq!(
        report.items.len(),
        1,
        "the fixture must answer this literal"
    );
    assert!(report.page.has_more);
    report
        .page
        .cursor
        .clone()
        .expect("a truncated page carries a cursor")
}

/// Continues `cursor` with another target and expects the engine to refuse it.
///
/// The request keeps every other bound field — snapshot, view, relation, consumers — so
/// the only binding it changes is the target, and the error has to name it.
fn assert_target_change_is_rejected(
    snapshot: &ArtifactSnapshot,
    relation: QueryRelation,
    kinds: &[ConsumerKind],
    cursor: QueryCursor,
    changed: QueryTarget,
    label: &str,
) {
    let mut request = request_with(snapshot, relation, changed, kinds, 0);
    request.cursor = Some(cursor);
    match Engine::new().query(snapshot, &request, &mut Budget::new(limits())) {
        Err(error) => {
            assert_eq!(error_code(&error), "query_cursor_mismatch", "{label}");
            assert!(
                error.to_string().contains("cursor target does not match"),
                "{label}: the error must name the target binding: {error}"
            );
        }
        Ok(report) => panic!(
            "{label}: a continuation that changes the target must be rejected before the scan, \
             but the scan ran: items={}, execution={:?}, artifact_structural={:?}, \
             diagnostics={:?}",
            report.items.len(),
            report.execution,
            report.coverage.dimensions.artifact_structural.state,
            report.diagnostics,
        ),
    }
}

/// R1 matrix: every dimension of the target is part of the cursor identity.
#[test]
fn cursor_rejects_every_target_change() {
    let snapshot = open(fixture("\r\n", true));
    let symbol_cases: Vec<(&str, QueryTarget)> = vec![
        ("symbol owner", symbol("com/example/Other")),
        ("symbol variant", field_target("com/example/Agent", "", "")),
        (
            "symbol name",
            method_target("com/example/Agent", "call", "()V"),
        ),
        (
            "symbol descriptor",
            method_target("com/example/Agent", "main", "([Ljava/lang/String;)V"),
        ),
    ];
    for (label, changed) in symbol_cases {
        let cursor = first_page_cursor(&snapshot, symbol("com/example/Agent"));
        assert_target_change_is_rejected(
            &snapshot,
            QueryRelation::MentionsSymbol,
            &[ConsumerKind::Resource],
            cursor,
            changed,
            label,
        );
    }

    let literals = open(literal_archive());
    let literal_cases: Vec<(&str, QueryTarget, QueryTarget)> = vec![
        (
            "literal kind",
            literal_target(LiteralValue::Integer { value: 7 }),
            literal_target(LiteralValue::Long { value: 7 }),
        ),
        (
            "literal raw value",
            literal_target(LiteralValue::Integer { value: 7 }),
            literal_target(LiteralValue::Integer { value: 8 }),
        ),
        (
            "literal kind over equal bytes",
            literal_target(LiteralValue::String {
                value: JvmBytes(b"p/L".to_vec()),
            }),
            literal_target(LiteralValue::Class {
                value: JvmBytes(b"p/L".to_vec()),
            }),
        ),
        (
            "float zero sign",
            literal_target(LiteralValue::Float { value: 0x0000_0000 }),
            literal_target(LiteralValue::Float { value: 0x8000_0000 }),
        ),
        (
            "float NaN payload",
            literal_target(LiteralValue::Float { value: 0x7fc0_0000 }),
            literal_target(LiteralValue::Float { value: 0x7fc0_0001 }),
        ),
        (
            "double zero sign",
            literal_target(LiteralValue::Double {
                value: 0x0000_0000_0000_0000,
            }),
            literal_target(LiteralValue::Double {
                value: 0x8000_0000_0000_0000,
            }),
        ),
        (
            "double NaN payload",
            literal_target(LiteralValue::Double {
                value: 0x7ff8_0000_0000_0001,
            }),
            literal_target(LiteralValue::Double {
                value: 0x7ff8_0000_0000_0002,
            }),
        ),
        (
            "float to double",
            literal_target(LiteralValue::Float { value: 0x0000_0000 }),
            literal_target(LiteralValue::Double { value: 0 }),
        ),
    ];
    for (label, original, changed) in literal_cases {
        let cursor = literal_cursor(&literals, original);
        assert_target_change_is_rejected(
            &literals,
            QueryRelation::ConstantPoolContains,
            &[ConsumerKind::Constant],
            cursor,
            changed,
            label,
        );
    }
}

/// Pages one request to the end with the page sizes of `sizes`, in order.
///
/// The sequence must describe exactly one scan: the last page may not hand out a cursor,
/// and the sizes may not end while one is still open. Every page is checked for the
/// `has_more`/cursor agreement the paging contract requires.
fn paged_items(
    snapshot: &ArtifactSnapshot,
    request: &QueryRequest,
    sizes: &[u64],
) -> Vec<XrefItem> {
    let mut cursor = None;
    let mut collected = Vec::new();
    for (index, size) in sizes.iter().copied().enumerate() {
        let mut page = request.clone();
        page.max_items = size;
        page.cursor = cursor.clone();
        let report = run(snapshot, &page);
        assert!(
            matches!(report.execution, ExecutionReport::Complete { .. }),
            "a page limit is not a degraded execution"
        );
        assert_eq!(report.page.has_more, report.page.cursor.is_some());
        assert_eq!(
            report.page.returned_items,
            u64::try_from(report.items.len()).expect("page length fits u64")
        );
        assert!(
            size == 0 || report.page.returned_items <= size,
            "a page never exceeds its item limit"
        );
        collected.extend(report.items.iter().cloned());
        cursor = report.page.cursor.clone();
        if cursor.is_none() {
            assert_eq!(
                index + 1,
                sizes.len(),
                "the page sizes must stop exactly at the end of the scan"
            );
            return collected;
        }
    }
    panic!("the page sizes must cover the whole scan: a page still carries a cursor");
}

/// Page size is not part of the query identity: only `max_items` changes between pages.
#[test]
fn page_size_does_not_change_query_identity() {
    let snapshot = open(fixture("\r\n", true));
    let identity = |max_items: u64| {
        request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Agent"),
            &[ConsumerKind::Resource],
            max_items,
        )
    };
    let full = run(&snapshot, &identity(0));
    assert_eq!(full.items.len(), 5);
    assert!(!full.page.has_more);
    assert!(full.page.cursor.is_none());

    // A fixed page size reads the same scan, one unchanged limit per page.
    for (size, sizes) in [
        (1_u64, vec![1_u64; 6]),
        (2, vec![2; 3]),
        (3, vec![3; 2]),
        (0, vec![0]),
    ] {
        assert_eq!(
            paged_items(&snapshot, &identity(size), &sizes),
            full.items,
            "page size {size} must reproduce the unpaged scan"
        );
    }

    // The limit may also change between pages of one continuation, including 0, which
    // means "no page limit" for that page only.
    assert_eq!(
        paged_items(&snapshot, &identity(1), &[1, 2, 0]),
        full.items,
        "changing max_items between pages must not repeat or skip items"
    );
}

#[test]
fn resource_consumer_reports_manifest_and_service_facts() {
    let snapshot = open(fixture("\r\n", true));
    let manifest = manifest("\r\n", true);
    let services = services();
    let ids = entry_ids(&snapshot);
    assert_eq!(
        ids[0].raw_name,
        ArchiveNameBytes(b"META-INF/MANIFEST.MF".to_vec())
    );
    assert_eq!(
        ids[1].raw_name,
        ArchiveNameBytes(b"META-INF/services/com.example.Service".to_vec())
    );

    // Main-Class: a structural class reference with its raw value span.
    let report = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Main"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    assert_eq!(report.items.len(), 1);
    let item = &report.items[0];
    assert_eq!(item.consumer, Some(ConsumerKind::Resource));
    assert_eq!(item.operation, XrefOperation::ManifestMainClass);
    assert_eq!(item.derivation, XrefDerivation::StructuralConsumer);
    assert_eq!(item.certainty, XrefCertainty::Exact);
    assert_eq!(item.resolution, QueryResolution::NotRequested);
    assert_eq!(item.relation, QueryRelation::MentionsSymbol);
    assert_eq!(
        item.target,
        XrefTarget::Symbol {
            value: SymbolRef::Class {
                owner: JvmBytes(b"com/example/Main".to_vec()),
            },
        }
    );
    assert_eq!(
        item.evidence.attribute,
        Some(ArchiveNameBytes(b"Main-Class".to_vec()))
    );
    assert_eq!(item.evidence.constant_pool_index, None);
    assert_eq!(item.evidence.bci, None);
    let (entry, span) = resource_location(item);
    assert_eq!(entry, &ids[0]);
    assert_eq!(span.start, offset_of(&manifest, b"com.example.Main"));
    assert_eq!(span.length, "com.example.Main".len() as u64);
    assert_eq!(item.evidence.span, Some(span));

    // Class-Path tokens are relative literal references, one item per token.
    let report = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::LiteralValue,
            literal("lib/one.jar"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::ManifestClassPath]);
    assert_eq!(
        report.items[0].target,
        XrefTarget::Literal {
            value: LiteralValue::String {
                value: JvmBytes(b"lib/one.jar".to_vec()),
            },
        }
    );
    let (entry, span) = resource_location(&report.items[0]);
    assert_eq!(entry, &ids[0]);
    assert_eq!(span.start, offset_of(&manifest, b"lib/one.jar"));
    assert_eq!(span.length, "lib/one.jar".len() as u64);

    // Automatic-Module-Name and Multi-Release stay raw literal values.
    let report = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::LiteralValue,
            literal("com.example.app"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    assert_eq!(
        operations(&report),
        vec![XrefOperation::ManifestAutomaticModuleName]
    );
    assert_eq!(
        report.items[0].evidence.attribute,
        Some(ArchiveNameBytes(b"Automatic-Module-Name".to_vec()))
    );
    let report = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::LiteralValue,
            literal("true"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    assert_eq!(
        operations(&report),
        vec![XrefOperation::ManifestMultiRelease]
    );
    assert_eq!(
        report.items[0].evidence.attribute,
        Some(ArchiveNameBytes(b"Multi-Release".to_vec()))
    );

    // Agent attributes and the service registration both name the same class, so a
    // single request returns four structural items across two entries.
    let report = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Agent"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    assert_eq!(report.items.len(), 5);
    assert_eq!(
        operations(&report),
        vec![
            XrefOperation::ManifestAgent,
            XrefOperation::ManifestAgent,
            XrefOperation::ManifestAgent,
            XrefOperation::ServiceProvider,
            XrefOperation::ServiceProvider,
        ]
    );
    assert_eq!(
        report
            .items
            .iter()
            .map(|item| resource_location(item).0.ordinal)
            .collect::<Vec<_>>(),
        vec![0, 0, 0, 1, 1]
    );
    assert_eq!(report.coverage.scanned_items, 5);
    assert_eq!(report.coverage.unknown_candidates, 0);

    // A named section after the main section is not an application attribute.
    let report = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("should/not/appear"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    assert!(report.items.is_empty());

    // The registration key comes from the entry path, so its span is the resource.
    let report = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Service"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::ServiceProvider]);
    assert_eq!(
        report.items[0].evidence.attribute,
        Some(ArchiveNameBytes(SERVICE_KEY.as_bytes().to_vec()))
    );
    let (entry, span) = resource_location(&report.items[0]);
    assert_eq!(entry, &ids[1]);
    assert_eq!(span.start, 0);
    assert_eq!(span.length, services.len() as u64);

    // Providers keep their raw name bytes, including a name split over two lines.
    let report = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/ProviderA"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::ServiceProvider]);
    let (entry, span) = resource_location(&report.items[0]);
    assert_eq!(entry, &ids[1]);
    assert_eq!(span.start, offset_of(&services, b"com.example.ProviderA"));
    let report = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Continued/Provider"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    assert_eq!(operations(&report), vec![XrefOperation::ServiceProvider]);
    let (_, span) = resource_location(&report.items[0]);
    assert_eq!(span.start, offset_of(&services, b"com.example.Continued."));
    assert!(
        span.length > "com.example.Continued.".len() as u64,
        "a continued name covers both lines"
    );
}

#[test]
fn resource_scan_respects_relation_and_consumer_schema() {
    let snapshot = open(fixture("\r\n", true));

    // A raw constant-pool probe is never answered by a resource fact.
    let report = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::ConstantPoolContains,
            symbol("com/example/Main"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    assert!(report.items.is_empty());

    // Without the resource category the manifest is not scanned at all.
    let report = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Main"),
            &[ConsumerKind::Type],
            0,
        ),
    );
    assert!(report.items.is_empty());

    // An empty consumer schema requests no scan: nothing is enumerated or billed.
    let mut request = request_with(
        &snapshot,
        QueryRelation::MentionsSymbol,
        symbol("com/example/Main"),
        &[],
        0,
    );
    request.consumers = ConsumerSchema::new(1, []);
    let mut budget = Budget::new(limits());
    let report = Engine::new()
        .query(&snapshot, &request, &mut budget)
        .unwrap();
    assert!(report.items.is_empty());
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::NotRequested
    );
    assert_eq!(budget.usage().archive_entries, 0);
    assert_eq!(budget.usage().read_bytes, 0);
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));

    // Only the resource entries are materialized: the class and the text entry stay
    // untouched, so an unrequested entry costs no read and no entry bytes.
    let request = request_with(
        &snapshot,
        QueryRelation::MentionsSymbol,
        symbol("com/example/Main"),
        &[ConsumerKind::Resource],
        0,
    );
    let mut budget = Budget::new(limits());
    let report = Engine::new()
        .query(&snapshot, &request, &mut budget)
        .unwrap();
    assert_eq!(report.items.len(), 1);
    assert_eq!(
        budget.usage().entry_bytes,
        (manifest("\r\n", true).len() + services().len()) as u64,
        "only the manifest and the service resource are decompressed"
    );
    assert_eq!(
        budget.usage().output_bytes,
        0,
        "query results never bill caller output bytes in the library"
    );
}

#[test]
fn manifest_line_endings_are_handled_without_losing_the_continuation_rule() {
    for ending in ["\r\n", "\n", "\r"] {
        let snapshot = open(fixture(ending, false));
        let report = run(
            &snapshot,
            &request_with(
                &snapshot,
                QueryRelation::MentionsSymbol,
                symbol("com/example/Main"),
                &[ConsumerKind::Resource],
                0,
            ),
        );
        assert_eq!(report.items.len(), 1, "line ending {ending:?}");
        assert_eq!(report.items[0].operation, XrefOperation::ManifestMainClass);
        assert_eq!(
            report.items[0].evidence.attribute,
            Some(ArchiveNameBytes(b"Main-Class".to_vec())),
            "attribute names keep their raw bytes"
        );
    }
}

#[test]
fn pagination_continuation_reproduces_one_full_scan() {
    let snapshot = open(fixture("\r\n", true));
    let full = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Agent"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    assert_eq!(full.items.len(), 5);
    assert!(!full.page.has_more);
    assert!(full.page.cursor.is_none());
    assert!(matches!(
        full.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    ));
    assert!(
        full.coverage
            .dimensions
            .artifact_structural
            .skipped
            .is_empty(),
        "a complete scan leaves no range unexamined"
    );

    let mut collected = Vec::new();
    let mut cursor = None;
    let mut pages: Vec<QueryReport> = Vec::new();
    loop {
        let mut request = request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Agent"),
            &[ConsumerKind::Resource],
            1,
        );
        request.cursor = cursor.clone();
        let report = run(&snapshot, &request);
        assert!(
            matches!(report.execution, ExecutionReport::Complete { .. }),
            "a page limit is not a degraded execution"
        );
        assert!(report.page.returned_items <= 1);
        collected.extend(report.items.iter().cloned());
        match report.page.cursor.clone() {
            Some(next) => {
                assert!(report.page.has_more);
                cursor = Some(next);
            }
            None => {
                assert!(!report.page.has_more);
                pages.push(report);
                break;
            }
        }
        pages.push(report);
        assert!(pages.len() < 8, "pagination must terminate");
    }
    assert_eq!(pages.len(), 6);
    assert_eq!(collected, full.items, "pages must not repeat or skip items");

    let xref_ranges = |ranges: &[CoverageRange]| {
        ranges
            .iter()
            .filter(|range| range.label == "container:root:xref_scan_entries")
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>()
    };
    let provider_ranges = |ranges: &[CoverageRange]| {
        ranges
            .iter()
            .filter(|range| range.label == "central_directory_entries")
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>()
    };

    // Pages 1..=5 each publish one item and keep scanning, because the page limit
    // bounds work: the provider ranges stay verbatim while the scan ranges describe
    // exactly what this invocation examined.
    for page in &pages[..5] {
        assert_eq!(page.page.returned_items, 1);
        assert!(page.page.has_more);
    }
    assert_eq!(
        xref_ranges(&pages[0].coverage.dimensions.artifact_structural.scanned),
        vec![(0, 1)]
    );
    assert_eq!(
        xref_ranges(&pages[0].coverage.dimensions.artifact_structural.skipped),
        vec![(1, 4)]
    );
    assert_eq!(
        provider_ranges(&pages[0].coverage.dimensions.artifact_structural.scanned),
        vec![(0, 4)],
        "provider enumeration ranges are carried verbatim"
    );
    assert_eq!(
        xref_ranges(&pages[1].coverage.dimensions.artifact_structural.scanned),
        vec![(0, 1)],
        "a continuation replays the boundary unit and skips its published prefix"
    );

    // The final page starts at the second container entry: the first entry is
    // reported as not examined by this invocation, and `scanned_items` counts the
    // replayed prefix once more (read again, neither re-published nor re-billed).
    // `has_more` is conservative, so the page after the last result is empty.
    let last = pages.last().unwrap();
    assert_eq!(last.page.returned_items, 0);
    assert!(!last.page.has_more);
    assert!(last.items.is_empty());
    assert_eq!(last.coverage.scanned_items, 2);
    assert_eq!(
        xref_ranges(&last.coverage.dimensions.artifact_structural.scanned),
        vec![(1, 4)]
    );
    assert_eq!(
        xref_ranges(&last.coverage.dimensions.artifact_structural.skipped),
        vec![(0, 1)]
    );
    assert_eq!(
        last.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial,
        "a continuation reports the ranges it did not examine as skipped"
    );
}

#[test]
fn result_item_budget_keeps_the_reliable_prefix() {
    let snapshot = open(fixture("\r\n", true));
    let request = request_with(
        &snapshot,
        QueryRelation::MentionsSymbol,
        symbol("com/example/Agent"),
        &[ConsumerKind::Resource],
        0,
    );
    // Four enumeration entries plus exactly one published item.
    let tight = Limits {
        result_items: 5,
        ..limits()
    };
    let mut budget = Budget::new(tight);
    let report = Engine::new()
        .query(&snapshot, &request, &mut budget)
        .unwrap();
    assert_eq!(report.items.len(), 1);
    assert_eq!(report.items[0].operation, XrefOperation::ManifestAgent);
    assert_eq!(report.coverage.scanned_items, 1);
    assert!(report.page.has_more);
    assert!(report.page.cursor.is_some());
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert_eq!(budget.usage().result_items, 5);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "budget_exceeded_result_items"),
        "the terminal diagnostic explains the stop"
    );
    assert!(!matches!(
        report.execution,
        ExecutionReport::Complete { .. }
    ));
}

/// A budget stop publishes a prefix of one scan, and the continuation resumes exactly after
/// that prefix whatever page size the caller chooses next.
///
/// The first invocation can pay for the four archive entries plus one published item, so it
/// stops on the `ResultItems` dimension and keeps a cursor. The following pages then use a
/// page limit twice and no limit at all, and they must tile the unpaged scan of the same
/// identity: nothing repeated, nothing skipped, and only the page that actually reached the
/// end may claim the complete state.
#[test]
fn budget_interruption_resumes_at_the_published_boundary() {
    let snapshot = open(fixture("\r\n", true));
    let identity = |max_items: u64| {
        request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Agent"),
            &[ConsumerKind::Resource],
            max_items,
        )
    };
    let full = run(&snapshot, &identity(0));
    assert_eq!(full.items.len(), 5);
    assert!(!full.page.has_more);
    assert!(full.page.cursor.is_none());
    assert_eq!(
        full.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );

    // Four enumeration entries plus exactly one published item.
    let result_items = 5;
    let mut budget = Budget::new(Limits {
        result_items,
        ..limits()
    });
    let first = Engine::new()
        .query(&snapshot, &identity(0), &mut budget)
        .unwrap();
    match &first.execution {
        ExecutionReport::Partial { reason, usage } => {
            assert_eq!(
                reason,
                &TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::ResultItems
                },
                "the stop names the exhausted dimension"
            );
            assert_eq!(usage.result_items, result_items);
        }
        other => panic!("an exhausted item budget is a partial execution, got {other:?}"),
    }
    assert_eq!(
        first.items,
        full.items[..1],
        "a stopped page is the reliable prefix of the same scan"
    );
    assert_eq!(first.page.returned_items, 1);
    assert!(first.page.has_more);
    assert_eq!(
        first.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial,
        "a stopped scan never claims completeness"
    );
    assert!(
        first
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "budget_exceeded_result_items"),
        "the terminal diagnostic explains the stop: {:?}",
        first.diagnostics
    );
    let cursor = first
        .page
        .cursor
        .clone()
        .expect("a budget stop keeps a cursor");
    assert_eq!(
        cursor.target,
        symbol("com/example/Agent"),
        "the cursor continues the same target"
    );
    assert_eq!(cursor.consumers, identity(0).consumers);

    let mut collected = first.items.clone();
    let mut last = first;
    let mut sizes = [1_u64, 1, 0].into_iter();
    while let Some(resume) = last.page.cursor.clone() {
        let max_items = sizes
            .next()
            .expect("the continuation must reach the end within the planned pages");
        let mut request = identity(max_items);
        request.cursor = Some(resume);
        let page = run(&snapshot, &request);
        let start = collected.len();
        let page_items = page.items.len();
        assert_eq!(
            page.page.returned_items,
            u64::try_from(page_items).expect("page length fits u64")
        );
        assert!(
            max_items == 0 || page.page.returned_items <= max_items,
            "a page never exceeds its item limit"
        );
        collected.extend(page.items.iter().cloned());
        assert_eq!(
            collected[start..],
            full.items[start..start + page_items],
            "each continuation page carries the next items of the unpaged scan"
        );
        last = page;
        assert_eq!(
            last.page.has_more,
            last.page.cursor.is_some(),
            "a page keeps `has_more` and its cursor in agreement"
        );
        assert!(
            matches!(last.execution, ExecutionReport::Complete { .. }),
            "a page limit and a sufficient budget are not a degraded execution: {:?}",
            last.execution
        );
        if last.page.cursor.is_some() {
            assert_eq!(
                last.coverage.dimensions.artifact_structural.state,
                CoverageState::Partial,
                "a page that stopped at its limit does not claim completeness"
            );
        }
    }
    assert!(
        sizes.next().is_none(),
        "the continuation must resume until the scan ends, not over the same prefix again"
    );
    assert_eq!(
        collected, full.items,
        "the pages must not repeat or skip items"
    );
    assert!(
        !last.page.has_more && last.page.cursor.is_none(),
        "the last page ends the scan"
    );
    assert_eq!(
        last.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema,
        "the page that examined every unit and ran out of work claims completeness"
    );
}

#[test]
fn cancellation_is_never_reported_as_complete() {
    let snapshot = open(fixture("\r\n", true));
    let request = request_with(
        &snapshot,
        QueryRelation::MentionsSymbol,
        symbol("com/example/Agent"),
        &[ConsumerKind::Resource],
        0,
    );

    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let report = Engine::new()
        .query(&snapshot, &request, &mut budget)
        .unwrap();
    assert!(matches!(
        report.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert!(report.items.is_empty());
    assert!(report.page.cursor.is_none());
    assert!(report.page.has_more, "the scan never reached the end");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "cancelled")
    );

    // Cancelling between two pages keeps the prefix of the first page and reports
    // the second page as cancelled instead of complete.
    let mut request = request;
    let first = run(&snapshot, &request);
    assert_eq!(first.page.returned_items, 5);
    request.max_items = 1;
    let first = run(&snapshot, &request);
    let cursor = first.page.cursor.clone().expect("truncated page");
    let token = CancellationToken::new();
    token.cancel();
    request.cursor = Some(cursor);
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let second = Engine::new()
        .query(&snapshot, &request, &mut budget)
        .unwrap();
    assert!(matches!(
        second.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert!(second.items.is_empty());
}

#[test]
fn unsupported_categories_do_not_change_scanned_evidence() {
    let snapshot = open(fixture("\r\n", true));
    let resource_only = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Agent"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    let with_unsupported = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Agent"),
            &[
                ConsumerKind::Resource,
                ConsumerKind::Verification,
                ConsumerKind::Debug,
            ],
            0,
        ),
    );
    assert_eq!(
        with_unsupported.coverage.unsupported_categories,
        vec![ConsumerKind::Verification, ConsumerKind::Debug]
    );
    assert!(
        with_unsupported
            .coverage
            .unsupported_categories
            .contains(&ConsumerKind::Debug)
    );
    assert!(resource_only.coverage.unsupported_categories.is_empty());
    assert_eq!(with_unsupported.items, resource_only.items);
    assert_eq!(
        with_unsupported.coverage.scanned_items,
        resource_only.coverage.scanned_items
    );
    assert_eq!(
        with_unsupported
            .coverage
            .dimensions
            .artifact_structural
            .scanned,
        resource_only
            .coverage
            .dimensions
            .artifact_structural
            .scanned
    );
    assert_eq!(
        with_unsupported
            .coverage
            .dimensions
            .artifact_structural
            .skipped,
        resource_only
            .coverage
            .dimensions
            .artifact_structural
            .skipped
    );
    assert_eq!(
        resource_only.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert_ne!(
        with_unsupported
            .coverage
            .dimensions
            .artifact_structural
            .state,
        CoverageState::CompleteWithinSchema
    );
    // The run itself still completed: unsupported categories are a schema fact, not
    // an execution failure.
    assert!(matches!(
        with_unsupported.execution,
        ExecutionReport::Complete { .. }
    ));
}

#[test]
fn standalone_class_snapshot_uses_class_offset_coverage() {
    let class = class();
    let snapshot = open(class.clone());
    let mut budget = Budget::new(limits());
    let request = request_with(
        &snapshot,
        QueryRelation::MentionsSymbol,
        symbol("p/A"),
        &[ConsumerKind::Resource],
        0,
    );
    let report = Engine::new()
        .query(&snapshot, &request, &mut budget)
        .unwrap();
    assert!(report.items.is_empty());
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        budget.usage().read_bytes,
        0,
        "a resource-only schema never materializes the standalone root"
    );
    let structural = &report.coverage.dimensions.artifact_structural;
    assert!(
        structural
            .skipped
            .iter()
            .any(|range| range.label == "standalone_class:xref_scan_bytes"
                && range.start == 0
                && range.end == class.len() as u64),
        "no resource producer reads a bare class, so its bytes stay unexamined"
    );
    assert_eq!(structural.state, CoverageState::Partial);
}

#[test]
fn reports_round_trip_as_tagged_json_without_unknown_fields() {
    let snapshot = open(fixture("\r\n", true));
    let report = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            symbol("com/example/Agent"),
            &[ConsumerKind::Resource],
            0,
        ),
    );
    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("\"kind\":\"symbol\""));
    assert!(json.contains("\"kind\":\"resource\""));
    assert!(json.contains("\"operation\":\"service_provider\""));
    assert!(json.contains("\"resolution\":\"not_requested\""));
    let round_trip: QueryReport = serde_json::from_str(&json).unwrap();
    assert_eq!(round_trip, report);

    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value["unexpected"] = serde_json::json!(1);
    assert!(
        serde_json::from_value::<QueryReport>(value).is_err(),
        "deny_unknown_fields must reject a sibling field"
    );
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value["items"][0]["evidence"]["unexpected"] = serde_json::json!(1);
    assert!(
        serde_json::from_value::<QueryReport>(value).is_err(),
        "nested evidence must reject a sibling field"
    );

    // Internal tags with snake_case variant names, for unit and payload variants.
    assert_eq!(
        serde_json::to_string(&QueryAnalysis::Performed).unwrap(),
        "{\"kind\":\"performed\"}"
    );
    assert_eq!(
        serde_json::to_string(&QueryAnalysis::UnsupportedAnalysis {
            relation: QueryRelation::MayDispatchTo
        })
        .unwrap(),
        "{\"kind\":\"unsupported_analysis\",\"relation\":\"may_dispatch_to\"}"
    );
    assert_eq!(
        serde_json::to_string(&LiteralValue::Integer { value: 7 }).unwrap(),
        "{\"kind\":\"integer\",\"value\":7}"
    );
    assert_eq!(
        serde_json::to_string(&QueryTarget::Literal {
            value: LiteralValue::Integer { value: 7 }
        })
        .unwrap(),
        "{\"kind\":\"literal\",\"value\":{\"kind\":\"integer\",\"value\":7}}"
    );
    assert!(
        serde_json::from_str::<LiteralValue>("{\"kind\":\"integer\",\"value\":7,\"extra\":1}")
            .is_err()
    );
    assert!(
        serde_json::from_str::<QueryRequest>("{\"relation\":\"mentions_symbol\"}").is_err(),
        "a partial request must not deserialize"
    );
}

#[test]
fn unsupported_relations_keep_raw_constant_pool_candidates() {
    let mut class_file = ClassFile::new();
    let this = class_file.class("p/A");
    let agent = class_file.class("com/example/Agent");
    let class_bytes = class_file.finish(this, &[]);
    // The class bytes never leave the test: the entry read and the candidate span are
    // both checked against this exact buffer.
    let snapshot = open(class_archive(&[(b"p/A.class", class_bytes.clone())]));
    let target = symbol("com/example/Agent");

    for relation in [
        QueryRelation::ReferencesDefinition,
        QueryRelation::MayDispatchTo,
    ] {
        let report = run(
            &snapshot,
            &request_with(
                &snapshot,
                relation,
                target.clone(),
                &[ConsumerKind::Type],
                0,
            ),
        );
        assert_eq!(
            report.analysis,
            QueryAnalysis::UnsupportedAnalysis { relation },
            "the analysis state is reported, never a resolution"
        );
        assert_eq!(report.items.len(), 1);
        let item = &report.items[0];
        let (index, span) = assert_pool_candidate(item, relation, &class_bytes);
        assert_eq!(index, agent);
        assert_eq!(
            item.target,
            XrefTarget::Symbol {
                value: SymbolRef::Class {
                    owner: JvmBytes(b"com/example/Agent".to_vec()),
                },
            }
        );
        // A `Class` entry is its tag plus the name index, so the span can be sliced out
        // of the bytes and checked against what the file really stores.
        assert_eq!(span.length, 3);
        assert_eq!(
            &class_bytes[span.start as usize..(span.start + span.length) as usize],
            &[7, 0, 3]
        );
        match &item.source.location {
            Location::ClassOffset { definition, offset } => {
                assert_eq!(*offset, span.start);
                let entry = definition.entry().expect("the class came from an entry");
                assert_eq!(entry.raw_name, ArchiveNameBytes(b"p/A.class".to_vec()));
                assert_eq!(definition.class_bytes.length, class_bytes.len() as u64);
                // The physical identity is the materialized entry's own identity, so the
                // candidate addresses bytes the caller can re-read through the public API.
                let mut budget = Budget::new(limits());
                let materialized = snapshot
                    .read_entry(&entry_named(&snapshot, b"p/A.class"), &mut budget)
                    .unwrap();
                assert_eq!(definition.class_bytes.digest, materialized.content_digest);
            }
            other => panic!("expected a class-offset location, got {other:?}"),
        }
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "query_relation_unsupported")
        );
        assert_eq!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );
        assert_eq!(report.coverage.scanned_items, 1);
        assert_eq!(report.coverage.unknown_candidates, 0);
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        assert!(!report.page.has_more);

        // The same target under the raw probe is the same item: the evidence is the probe
        // path's own output, not something this relation invents.
        let probe = run(
            &snapshot,
            &request_with(
                &snapshot,
                QueryRelation::ConstantPoolContains,
                target.clone(),
                &[ConsumerKind::Type],
                0,
            ),
        );
        assert_eq!(probe.items.len(), 1);
        assert_eq!(probe.items[0].evidence, item.evidence);
        assert_eq!(probe.items[0].source, item.source);
        assert_eq!(probe.items[0].operation, item.operation);
        assert_eq!(probe.items[0].derivation, item.derivation);
        assert_eq!(probe.items[0].certainty, item.certainty);

        // Nobody consumes the entry, so a performed relation reports no reference at all
        // (acceptance A01): the candidate must not become a use site on either path.
        let performed = run(
            &snapshot,
            &request_with(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target.clone(),
                &[ConsumerKind::Type],
                0,
            ),
        );
        assert!(performed.items.is_empty());
        assert_eq!(performed.analysis, QueryAnalysis::Performed);
    }
}

#[test]
fn unsupported_relations_report_candidates_even_when_a_call_consumes_one() {
    let mut class_file = ClassFile::new();
    let this = class_file.class("p/A");
    let consumed = class_file.method_ref("com/example/Agent", "run", "()V");
    let unused = class_file.interface_method_ref("com/example/Agent", "run", "()V");
    let class_bytes = class_file.finish(
        this,
        &[MethodFixture {
            name: "main",
            descriptor: "()V",
            code: invokestatic(consumed),
        }],
    );
    let snapshot = open(class_archive(&[(b"p/A.class", class_bytes.clone())]));
    let target = method_target("com/example/Agent", "run", "()V");

    // The fixture really consumes the symbol: the performed relation reports the call
    // site, so the candidate-only answer below is a boundary and not an empty fixture.
    let performed = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target.clone(),
            &[ConsumerKind::Invocation],
            0,
        ),
    );
    assert_eq!(performed.items.len(), 1);
    let call = &performed.items[0];
    assert_eq!(call.consumer, Some(ConsumerKind::Invocation));
    assert_eq!(call.operation, XrefOperation::InvokeStatic);
    assert_eq!(call.derivation, XrefDerivation::StructuralConsumer);
    assert_eq!(call.evidence.bci, Some(0));
    assert_eq!(call.evidence.opcode, Some(0xb8));
    assert_eq!(call.evidence.constant_pool_index, Some(consumed));

    for relation in [
        QueryRelation::ReferencesDefinition,
        QueryRelation::MayDispatchTo,
    ] {
        let report = run(
            &snapshot,
            &request_with(
                &snapshot,
                relation,
                target.clone(),
                &[ConsumerKind::Invocation],
                0,
            ),
        );
        assert_eq!(
            report.analysis,
            QueryAnalysis::UnsupportedAnalysis { relation }
        );
        assert_eq!(
            report.items.len(),
            2,
            "both pool entries carry the raw target"
        );
        let mut indices = Vec::new();
        for item in &report.items {
            let (index, _) = assert_pool_candidate(item, relation, &class_bytes);
            indices.push(index);
        }
        indices.sort_unstable();
        assert_eq!(indices, vec![consumed, unused]);
        // No candidate may become a consumer claim: no category, no BCI, no opcode, and
        // no `invoke*` operation appears in this answer.
        assert!(report.items.iter().all(|item| item.consumer.is_none()
            && item.evidence.bci.is_none()
            && item.evidence.opcode.is_none()
            && item.operation == XrefOperation::ConstantPoolEntry));
        assert_eq!(report.coverage.scanned_items, 2);
        assert_eq!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );
    }
}

#[test]
fn unsupported_relations_do_not_expand_a_pool_owner() {
    let mut class_file = ClassFile::new();
    let this = class_file.class("p/A");
    let sub = class_file.method_ref("p/Sub", "run", "()V");
    let class_bytes = class_file.finish(
        this,
        &[MethodFixture {
            name: "main",
            descriptor: "()V",
            code: invokestatic(sub),
        }],
    );
    let snapshot = open(class_archive(&[(b"p/A.class", class_bytes.clone())]));

    // The pool really carries the subclass owner, so the miss below is about the owner
    // dimension and not about an empty pool.
    let sub_report = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::ReferencesDefinition,
            method_target("p/Sub", "run", "()V"),
            &[ConsumerKind::Invocation],
            0,
        ),
    );
    assert_eq!(sub_report.items.len(), 1);
    let (index, _) = assert_pool_candidate(
        &sub_report.items[0],
        QueryRelation::ReferencesDefinition,
        &class_bytes,
    );
    assert_eq!(index, sub);

    // A call site whose pool owner is `p/Sub` is not a reference to the method `p/Base`
    // declares: expanding the owner is the P2 resolver's job.
    for relation in [
        QueryRelation::ReferencesDefinition,
        QueryRelation::MayDispatchTo,
    ] {
        let report = run(
            &snapshot,
            &request_with(
                &snapshot,
                relation,
                method_target("p/Base", "run", "()V"),
                &[ConsumerKind::Invocation],
                0,
            ),
        );
        assert_eq!(
            report.analysis,
            QueryAnalysis::UnsupportedAnalysis { relation }
        );
        assert!(
            report.items.is_empty(),
            "P1 must not expand an owner or resolve a definition"
        );
        assert_eq!(report.coverage.scanned_items, 0);
        assert_eq!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        assert_eq!(
            report
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "query_relation_unsupported")
                .count(),
            1
        );
    }
}

#[test]
fn unsupported_relation_candidates_page_without_repeats_or_gaps() {
    let entries = ["a/One.class", "b/Two.class", "c/Three.class"]
        .iter()
        .map(|name| {
            // One class entry per archive entry, each with exactly one pool entry for the
            // target, so the candidate stream has three items in unit order.
            let mut class_file = ClassFile::new();
            let this = class_file.class("p/A");
            let _target_entry = class_file.class("com/example/Agent");
            let bytes = class_file.finish(this, &[]);
            (name.as_bytes(), bytes)
        })
        .collect::<Vec<_>>();
    let snapshot = open(class_archive(&entries));
    let target = symbol("com/example/Agent");

    let full = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::ReferencesDefinition,
            target.clone(),
            &[ConsumerKind::Type],
            0,
        ),
    );
    assert_eq!(full.items.len(), 3);
    assert!(!full.page.has_more);
    assert!(full.page.cursor.is_none());
    // Container order, then entry ordinal ascending: the page boundary depends on it.
    let ordinals = full
        .items
        .iter()
        .map(|item| match &item.source.location {
            Location::ClassOffset { definition, .. } => {
                definition.entry().expect("archive entry").ordinal
            }
            other => panic!("expected a class-offset location, got {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(ordinals, vec![0, 1, 2]);

    let mut cursor = None;
    let mut collected = Vec::new();
    let mut pages = Vec::new();
    loop {
        let mut request = request_with(
            &snapshot,
            QueryRelation::ReferencesDefinition,
            target.clone(),
            &[ConsumerKind::Type],
            1,
        );
        request.cursor = cursor.clone();
        let report = run(&snapshot, &request);
        assert_eq!(
            report.analysis,
            QueryAnalysis::UnsupportedAnalysis {
                relation: QueryRelation::ReferencesDefinition
            }
        );
        assert!(report.items.len() <= 1);
        assert_eq!(report.page.returned_items, report.items.len() as u64);
        let next = report.page.cursor.clone();
        collected.extend(report.items.iter().cloned());
        pages.push(report);
        match next {
            Some(next) => cursor = Some(next),
            None => break,
        }
        assert!(pages.len() < 8, "pagination must terminate");
    }
    assert_eq!(pages.len(), 3);
    assert_eq!(
        collected, full.items,
        "pages must not repeat or skip candidates"
    );
    for page in &pages[..2] {
        assert!(page.page.has_more);
    }
    assert!(!pages[2].page.has_more);
    assert_eq!(
        pages[0]
            .page
            .cursor
            .as_ref()
            .expect("truncated page")
            .boundary
            .ordinal,
        0
    );
    assert_eq!(
        pages[0]
            .page
            .cursor
            .as_ref()
            .expect("truncated page")
            .boundary
            .item_index,
        1
    );
    assert_eq!(
        pages[1]
            .page
            .cursor
            .as_ref()
            .expect("truncated page")
            .boundary
            .ordinal,
        1
    );
    // A continuation replays the resumed entry: its already published candidate is read
    // again but neither re-published nor billed again, so the page reports one replay and
    // one new candidate as scanned.
    assert_eq!(pages[1].coverage.scanned_items, 2);
    assert_eq!(pages[2].coverage.scanned_items, 2);

    // The cursor binds the caller's relation, not the substituted probe: a cursor from
    // `references_definition` cannot be replayed for `may_dispatch_to`.
    let mut reused = request_with(
        &snapshot,
        QueryRelation::MayDispatchTo,
        target.clone(),
        &[ConsumerKind::Type],
        1,
    );
    reused.cursor = pages[0].page.cursor.clone();
    let error = Engine::new()
        .query(&snapshot, &reused, &mut Budget::new(limits()))
        .unwrap_err();
    assert_eq!(error_code(&error), "query_cursor_mismatch");
}

#[test]
fn unsupported_relation_candidates_use_the_ordinary_item_budget() {
    let entries = ["a/One.class", "b/Two.class", "c/Three.class"]
        .iter()
        .map(|name| {
            let mut class_file = ClassFile::new();
            let this = class_file.class("p/A");
            let _target_entry = class_file.class("com/example/Agent");
            (name.as_bytes(), class_file.finish(this, &[]))
        })
        .collect::<Vec<_>>();
    let snapshot = open(class_archive(&entries));
    let request = request_with(
        &snapshot,
        QueryRelation::ReferencesDefinition,
        symbol("com/example/Agent"),
        &[ConsumerKind::Type],
        0,
    );

    // Three result items are billed by the enumeration, so this limit publishes exactly
    // one candidate before the page stops.
    let tight = Limits {
        result_items: 4,
        ..limits()
    };
    let mut budget = Budget::new(tight);
    let report = Engine::new()
        .query(&snapshot, &request, &mut budget)
        .unwrap();
    assert_eq!(report.items.len(), 1);
    assert_eq!(report.coverage.scanned_items, 1);
    assert_eq!(
        report.analysis,
        QueryAnalysis::UnsupportedAnalysis {
            relation: QueryRelation::ReferencesDefinition
        }
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert_eq!(
        report
            .page
            .cursor
            .as_ref()
            .expect("a truncated candidate page keeps a cursor")
            .relation,
        QueryRelation::ReferencesDefinition
    );
    assert_eq!(budget.usage().result_items, 4);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "budget_exceeded_result_items")
    );
}

#[test]
fn unsupported_relation_keeps_candidates_for_a_standalone_class() {
    let mut class_file = ClassFile::new();
    let this = class_file.class("p/A");
    let _agent = class_file.class("com/example/Agent");
    let class_bytes = class_file.finish(this, &[]);
    let snapshot = open(class_bytes.clone());

    let report = run(
        &snapshot,
        &request_with(
            &snapshot,
            QueryRelation::ReferencesDefinition,
            symbol("com/example/Agent"),
            &[ConsumerKind::Type],
            0,
        ),
    );
    assert_eq!(
        report.analysis,
        QueryAnalysis::UnsupportedAnalysis {
            relation: QueryRelation::ReferencesDefinition
        }
    );
    assert_eq!(report.items.len(), 1);
    let (_, span) = assert_pool_candidate(
        &report.items[0],
        QueryRelation::ReferencesDefinition,
        &class_bytes,
    );
    // The standalone root is a real physical location, not a synthetic entry.
    match &report.items[0].source.location {
        Location::ClassOffset { definition, offset } => {
            assert_eq!(*offset, span.start);
            assert_eq!(
                definition.location,
                PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone()
                }
            );
            assert_eq!(definition.entry(), None);
        }
        other => panic!("expected a class-offset location, got {other:?}"),
    }
    // Coverage names the class-offset range of the bytes the probe read.
    let structural = &report.coverage.dimensions.artifact_structural;
    assert_eq!(structural.state, CoverageState::CompleteWithinSchema);
    assert!(
        structural.scanned.iter().any(|range| {
            range.label == "standalone_class:xref_scan_bytes"
                && range.start == 0
                && range.end == class_bytes.len() as u64
        }),
        "the probe read the whole root: {structural:?}"
    );
}
