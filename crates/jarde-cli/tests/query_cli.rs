//! P1 query protocol acceptance for `jarde-cli`: one JSON request in, one JSON document out,
//! and a report that is field for field the report `Engine::query` returns for the same request.
//!
//! The fixture archive is assembled in this file because the CLI package has no ZIP writer
//! dependency: stored entries are written as local file headers plus a central directory and an
//! end-of-central-directory record, so every assertion still runs against a real snapshot, a
//! real enumeration and the public query entry point.

use jarde::{
    ArtifactInput, ArtifactSnapshot, Budget, CancellationToken, ConsumerKind, ConsumerSchema,
    CoverageState, Engine, ExecutionReport, JvmBytes, Limits, Location, PhysicalScope,
    PhysicalView, QueryAnalysis, QueryRelation, QueryReport, QueryRequest, QueryTarget, SymbolRef,
    TerminationReason, UsageSnapshot, XrefOperation, XrefTarget,
};
use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const BIN: &str = env!("CARGO_BIN_EXE_jarde-cli");
const MAX_REQUEST_BYTES: usize = 1024 * 1024;
/// Archive entries of [`fixture`]: each enumerated entry costs one `result_items`, which is what
/// the tight budget in `result_item_budget_keeps_the_reliable_prefix_and_names_the_dimension`
/// is derived from.
const FIXTURE_ENTRIES: u64 = 4;
/// Archive-relative names of [`fixture`], in enumeration order.
const MANIFEST_ENTRY: &[u8] = b"META-INF/MANIFEST.MF";
const SERVICES_ENTRY: &[u8] = b"META-INF/services/com.example.Service";
const CLASS_ENTRY: &[u8] = b"p/A.class";
const TEXT_ENTRY: &[u8] = b"docs/readme.txt";

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-cli-query-test-{}-{nonce}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("create test directory");
        Self { path }
    }

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.path.join(name);
        fs::write(&path, bytes).expect("write test fixture");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

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

// --- fixture ---------------------------------------------------------------------------

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
/// Agent attributes of [`MANIFEST_LINES`] in document order, which is item order.
const AGENT_ATTRIBUTES: [&str; 3] = ["Launcher-Agent-Class", "Premain-Class", "Agent-Class"];

/// Main section attributes terminated with CRLF, without a named section.
fn manifest() -> Vec<u8> {
    let mut out = Vec::new();
    for line in MANIFEST_LINES {
        out.extend_from_slice(line.as_bytes());
        out.extend_from_slice(b"\r\n");
    }
    out.extend_from_slice(b"\r\n");
    out
}

/// Service registrations: a comment, a blank line, a plain provider, a provider with a
/// trailing comment and a provider that ends its name with the continuation marker.
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

/// Minimal, complete class file so unrelated entries stay realistic.
fn class() -> Vec<u8> {
    let mut out = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut out, 0);
    u16b(&mut out, 52);
    u16b(&mut out, 5);
    utf8(&mut out, b"p/A");
    out.push(7);
    u16b(&mut out, 1);
    utf8(&mut out, b"java/lang/Object");
    out.push(7);
    u16b(&mut out, 3);
    u16b(&mut out, 0x21);
    u16b(&mut out, 2);
    u16b(&mut out, 4);
    u16b(&mut out, 0);
    u16b(&mut out, 0);
    u16b(&mut out, 0);
    u16b(&mut out, 0);
    out
}

fn fixture() -> Vec<u8> {
    zip(&[
        (MANIFEST_ENTRY, &manifest()),
        (SERVICES_ENTRY, &services()),
        (CLASS_ENTRY, &class()),
        (TEXT_ENTRY, b"not a resource"),
    ])
}

fn u16b(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn utf8(output: &mut Vec<u8>, value: &[u8]) {
    output.push(1);
    u16b(
        output,
        u16::try_from(value.len()).expect("fixture UTF-8 length fits u16"),
    );
    output.extend_from_slice(value);
}

/// ZIP32 archive with stored entries: local file header, central directory record and EOCD.
///
/// The artifact reader verifies the entry CRC, so the checksum is computed here rather than
/// assumed; every other field is the constant of a stored, UTF-8-named entry.
fn zip(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    const LOCAL_SIGNATURE: u32 = 0x0403_4b50;
    const CENTRAL_SIGNATURE: u32 = 0x0201_4b50;
    const EOCD_SIGNATURE: u32 = 0x0605_4b50;
    const VERSION: u16 = 20;
    /// "name is UTF-8" plus the reserved bits a stored entry needs cleared.
    const UTF8_FLAG: u16 = 0x0800;
    /// 1980-01-01 00:00: DOS date of the fixed timestamp these fixtures use.
    const DOS_DATE: u16 = 0x0021;

    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in entries {
        let offset = u32::try_from(out.len()).expect("fixture offset fits u32");
        let length = u32::try_from(data.len()).expect("fixture length fits u32");
        let name_length = u16::try_from(name.len()).expect("fixture name length fits u16");
        let crc = crc32(data);

        out.extend_from_slice(&LOCAL_SIGNATURE.to_le_bytes());
        out.extend_from_slice(&VERSION.to_le_bytes());
        out.extend_from_slice(&UTF8_FLAG.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&DOS_DATE.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&length.to_le_bytes());
        out.extend_from_slice(&length.to_le_bytes());
        out.extend_from_slice(&name_length.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(name);
        out.extend_from_slice(data);

        central.extend_from_slice(&CENTRAL_SIGNATURE.to_le_bytes());
        central.extend_from_slice(&VERSION.to_le_bytes());
        central.extend_from_slice(&VERSION.to_le_bytes());
        central.extend_from_slice(&UTF8_FLAG.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&DOS_DATE.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&length.to_le_bytes());
        central.extend_from_slice(&length.to_le_bytes());
        central.extend_from_slice(&name_length.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&0_u32.to_le_bytes());
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name);
    }
    let central_offset = u32::try_from(out.len()).expect("central directory offset fits u32");
    let central_size = u32::try_from(central.len()).expect("central directory size fits u32");
    let count = u16::try_from(entries.len()).expect("fixture entry count fits u16");
    out.extend_from_slice(&central);
    out.extend_from_slice(&EOCD_SIGNATURE.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&central_size.to_le_bytes());
    out.extend_from_slice(&central_offset.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes());
    out
}

/// IEEE CRC-32, the checksum a stored ZIP entry must carry.
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

// --- process driving -------------------------------------------------------------------

/// One query request as sent on the wire, plus the library call it must agree with.
#[derive(Clone, Debug)]
struct Spec {
    relation: QueryRelation,
    target: QueryTarget,
    scope: PhysicalScope,
    consumers: ConsumerSchema,
    max_items: u64,
    cursor: Option<Value>,
}

impl Spec {
    fn symbol(
        relation: QueryRelation,
        owner: &str,
        kinds: &[ConsumerKind],
        max_items: u64,
    ) -> Self {
        Self {
            relation,
            target: QueryTarget::Symbol {
                value: SymbolRef::Class {
                    owner: JvmBytes(owner.as_bytes().to_vec()),
                },
            },
            scope: PhysicalScope::SnapshotAll,
            consumers: ConsumerSchema::new(1, kinds.to_vec()),
            max_items,
            cursor: None,
        }
    }

    fn literal(
        relation: QueryRelation,
        value: &str,
        kinds: &[ConsumerKind],
        max_items: u64,
    ) -> Self {
        Self {
            relation,
            target: QueryTarget::Literal {
                value: jarde::LiteralValue::String {
                    value: JvmBytes(value.as_bytes().to_vec()),
                },
            },
            scope: PhysicalScope::SnapshotAll,
            consumers: ConsumerSchema::new(1, kinds.to_vec()),
            max_items,
            cursor: None,
        }
    }

    fn with_scope(mut self, scope: PhysicalScope) -> Self {
        self.scope = scope;
        self
    }

    fn with_cursor(mut self, cursor: Value) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// Request body exactly as the CLI reads it.
    fn operation_json(&self) -> Value {
        json!({
            "kind": "query",
            "relation": self.relation,
            "target": self.target,
            "physical": self.scope,
            "consumers": self.consumers,
            "max_items": self.max_items,
            "cursor": self.cursor,
        })
    }

    fn library_request(&self, snapshot: &ArtifactSnapshot) -> QueryRequest {
        QueryRequest {
            relation: self.relation,
            target: self.target.clone(),
            // The adapter derives the snapshot from `input_path`; the caller declares the scope.
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: self.scope.clone(),
            },
            consumers: self.consumers.clone(),
            max_items: self.max_items,
            cursor: None,
        }
    }
}

fn request(input_path: &Path, limits: &Limits, operation: Value) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "input_path": input_path,
        "limits": limits,
        "operation": operation,
    }))
    .expect("serialize request")
}

fn run_stdin(request: &[u8]) -> Output {
    let mut child = Command::new(BIN)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn jarde-cli");
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(request)
        .expect("write request JSON");
    child.wait_with_output().expect("wait for jarde-cli")
}

/// Parses stdout and checks the output contract: one JSON document and one trailing newline.
fn response(output: &Output) -> Value {
    assert_eq!(
        output.stdout.last(),
        Some(&b'\n'),
        "stdout must end with one newline: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(
        output.stdout.iter().filter(|&&byte| byte == b'\n').count(),
        1,
        "stdout must contain one JSON document"
    );
    assert_eq!(
        output.stderr, b"",
        "a JSON protocol answer never writes diagnostics to stderr"
    );
    serde_json::from_slice(&output.stdout).expect("stdout is valid JSON")
}

fn assert_ok(output: &Output, kind: &str) -> Value {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let value = response(output);
    assert_eq!(value["status"], "ok");
    assert_eq!(value["result"]["kind"], kind);
    assert_eq!(
        value["transport"]["response_bytes"],
        u64::try_from(output.stdout.len()).expect("stdout length fits u64")
    );
    value
}

fn assert_error_code(output: &Output, code: &str) -> Value {
    assert_eq!(output.status.code(), Some(1));
    let value = response(output);
    assert_eq!(value["status"], "error");
    assert_eq!(value["error"]["code"], code);
    value
}

fn query_via_cli(path: &Path, request_limits: &Limits, spec: &Spec) -> (Value, QueryReport) {
    let output = run_stdin(&request(path, request_limits, spec.operation_json()));
    let value = assert_ok(&output, "query");
    let report: QueryReport = serde_json::from_value(value["result"]["report"].clone())
        .expect("the CLI report deserializes as the library report type");
    (value, report)
}

fn query_direct(path: &Path, request_limits: &Limits, spec: &Spec) -> QueryReport {
    let mut budget = Budget::new(request_limits.clone());
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::Path(path.to_path_buf()), &mut budget)
        .expect("open the fixture directly");
    let request = spec.library_request(&snapshot);
    engine
        .query(&snapshot, &request, &mut budget)
        .expect("query the fixture directly")
}

/// Wall-clock usage is the only field the two entry paths cannot share.
fn normalized_execution(report: &ExecutionReport) -> ExecutionReport {
    let mut report = report.clone();
    match &mut report {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage.elapsed_millis = 0,
    }
    report
}

fn normalized_usage(usage: &UsageSnapshot) -> UsageSnapshot {
    let mut usage = usage.clone();
    usage.elapsed_millis = 0;
    usage
}

fn normalize_report_json(value: &Value) -> Value {
    let mut value = value.clone();
    if let Some(usage) = value.pointer_mut("/execution/usage/elapsed_millis") {
        *usage = json!(0);
    }
    value
}

/// The full field-by-field agreement between one CLI answer and one direct library call.
fn assert_same_report(cli: &QueryReport, direct: &QueryReport) {
    assert_eq!(cli.physical, direct.physical);
    assert_eq!(cli.relation, direct.relation);
    assert_eq!(cli.consumers, direct.consumers);
    assert_eq!(cli.analysis, direct.analysis);
    assert_eq!(cli.items, direct.items);
    assert_eq!(cli.page, direct.page);
    assert_eq!(cli.coverage, direct.coverage);
    assert_eq!(cli.diagnostics, direct.diagnostics);
    assert_eq!(
        normalized_execution(&cli.execution),
        normalized_execution(&direct.execution)
    );
}

fn resource_location(item: &jarde::XrefItem) -> (u64, &[u8], u64, u64) {
    match &item.source.location {
        Location::Resource { entry, span } => {
            (entry.ordinal, &entry.raw_name.0, span.start, span.length)
        }
        other => panic!("expected a resource location, got {other:?}"),
    }
}

fn offset_of(haystack: &[u8], needle: &[u8]) -> u64 {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
        .expect("fixture text must be present") as u64
}

// --- acceptance ------------------------------------------------------------------------

#[test]
fn query_report_matches_the_library_field_by_field() {
    let temp = TempDir::new();
    let manifest = manifest();
    let services = services();
    let archive = temp.write("app.zip", &fixture());
    let request_limits = limits(u64::MAX);

    // One target that matches a single manifest attribute, and one that matches three agent
    // attributes plus two service registrations across two entries.
    for (owner, expected_items, expected_operations) in [
        (
            "com/example/Main",
            1_usize,
            vec![XrefOperation::ManifestMainClass],
        ),
        (
            "com/example/Agent",
            5,
            vec![
                XrefOperation::ManifestAgent,
                XrefOperation::ManifestAgent,
                XrefOperation::ManifestAgent,
                XrefOperation::ServiceProvider,
                XrefOperation::ServiceProvider,
            ],
        ),
    ] {
        let spec = Spec::symbol(
            QueryRelation::MentionsSymbol,
            owner,
            &[ConsumerKind::Resource],
            0,
        );
        let (value, cli) = query_via_cli(&archive, &request_limits, &spec);
        let direct = query_direct(&archive, &request_limits, &spec);

        assert_same_report(&cli, &direct);
        assert_eq!(
            normalize_report_json(&value["result"]["report"]),
            normalize_report_json(&serde_json::to_value(&direct).expect("serialize direct report")),
            "the CLI document must carry exactly the library fields"
        );

        // `result.kind` names the operation, `result.report` the `QueryReport` itself.
        assert_eq!(value["result"]["report"]["analysis"]["kind"], "performed");
        assert_eq!(value["result"]["report"]["relation"], "mentions_symbol");
        assert_eq!(
            value["result"]["report"]["physical"]["scope"]["kind"],
            "snapshot_all"
        );
        assert_eq!(value["result"]["report"]["execution"]["status"], "complete");

        assert_eq!(cli.items.len(), expected_items);
        assert_eq!(
            cli.items
                .iter()
                .map(|item| item.operation)
                .collect::<Vec<_>>(),
            expected_operations
        );
        assert_eq!(
            cli.items
                .iter()
                .map(|item| item.consumer)
                .collect::<Vec<_>>(),
            vec![Some(ConsumerKind::Resource); expected_items]
        );
        assert_eq!(cli.coverage.scanned_items, expected_items as u64);
        assert_eq!(cli.coverage.unknown_candidates, 0);
        assert_eq!(
            cli.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );
        assert_eq!(cli.page.returned_items, expected_items as u64);
        assert!(!cli.page.has_more);
        assert!(cli.page.cursor.is_none());
    }

    // The single-attribute case, checked on the wire document: provenance, evidence and target
    // of every item are the ones the resource consumer produced.
    let spec = Spec::symbol(
        QueryRelation::MentionsSymbol,
        "com/example/Main",
        &[ConsumerKind::Resource],
        0,
    );
    let (value, cli) = query_via_cli(&archive, &request_limits, &spec);
    let item = &cli.items[0];
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
        Some(jarde::ArchiveNameBytes(b"Main-Class".to_vec()))
    );
    let (ordinal, raw_name, start, length) = resource_location(item);
    assert_eq!(ordinal, 0);
    assert_eq!(raw_name, MANIFEST_ENTRY);
    assert_eq!(start, offset_of(&manifest, b"com.example.Main"));
    assert_eq!(length, "com.example.Main".len() as u64);
    assert_eq!(
        item.evidence.span,
        Some(jarde::ByteSpan::new(start, length))
    );
    let wire = &value["result"]["report"]["items"][0];
    assert_eq!(wire["consumer"], "resource");
    assert_eq!(wire["derivation"], "structural_consumer");
    assert_eq!(wire["certainty"], "exact");
    assert_eq!(wire["resolution"], "not_requested");
    assert_eq!(wire["operation"], "manifest_main_class");
    assert_eq!(wire["target"]["kind"], "symbol");
    assert_eq!(wire["target"]["value"]["kind"], "class");
    assert_eq!(wire["target"]["value"]["owner"], json!(b"com/example/Main"));
    assert_eq!(wire["source"]["location"]["kind"], "resource");
    assert_eq!(wire["source"]["location"]["entry"]["ordinal"], 0);
    assert_eq!(
        wire["source"]["location"]["entry"]["raw_name"],
        json!(MANIFEST_ENTRY)
    );
    assert_eq!(wire["evidence"]["attribute"], json!(b"Main-Class"));
    assert_eq!(wire["evidence"]["span"]["start"], start);
    assert_eq!(wire["evidence"]["span"]["length"], length);

    // Manifest attributes keep document order and service items follow them per entry, so
    // paging has a defined order.
    let spec = Spec::symbol(
        QueryRelation::MentionsSymbol,
        "com/example/Agent",
        &[ConsumerKind::Resource],
        0,
    );
    let (_, cli) = query_via_cli(&archive, &request_limits, &spec);
    let attributes = cli
        .items
        .iter()
        .take(3)
        .map(|item| {
            String::from_utf8(item.evidence.attribute.clone().unwrap().0)
                .expect("attribute names are ASCII")
        })
        .collect::<Vec<_>>();
    assert_eq!(attributes, AGENT_ATTRIBUTES);
    assert_eq!(
        cli.items
            .iter()
            .map(|item| resource_location(item).0)
            .collect::<Vec<_>>(),
        vec![0, 0, 0, 1, 1]
    );
    let (ordinal, raw_name, start, _) = resource_location(&cli.items[3]);
    assert_eq!(ordinal, 1);
    assert_eq!(raw_name, SERVICES_ENTRY);
    assert_eq!(
        start,
        offset_of(&services, b"com.example.Agent # trailing comment")
    );

    // Only the resource entries are materialized: the class and the text entry stay unread, and
    // a query never bills caller output bytes in the library.
    let ExecutionReport::Complete { usage } = &cli.execution else {
        panic!("the scan must complete");
    };
    assert_eq!(
        normalized_usage(usage).entry_bytes,
        u64::try_from(manifest.len() + services.len()).expect("fixture length fits u64")
    );
    assert_eq!(usage.output_bytes, 0);
    // `archive_entries` counts central-directory records processed, which includes the replay
    // locator of each read, so the provider range is what pins the fixture size.
    let central_directory = cli
        .coverage
        .dimensions
        .artifact_structural
        .scanned
        .iter()
        .filter(|range| range.label == "central_directory_entries")
        .map(|range| (range.start, range.end))
        .collect::<Vec<_>>();
    assert_eq!(central_directory, vec![(0, FIXTURE_ENTRIES)]);

    // A literal target goes through the same path and answers the `Class-Path` token.
    let spec = Spec::literal(
        QueryRelation::LiteralValue,
        "lib/two.jar",
        &[ConsumerKind::Resource],
        0,
    );
    let (_, cli) = query_via_cli(&archive, &request_limits, &spec);
    let direct = query_direct(&archive, &request_limits, &spec);
    assert_same_report(&cli, &direct);
    assert_eq!(cli.items.len(), 1);
    assert_eq!(cli.items[0].operation, XrefOperation::ManifestClassPath);
    let (_, _, start, length) = resource_location(&cli.items[0]);
    assert_eq!(start, offset_of(&manifest, b"lib/two.jar"));
    assert_eq!(length, "lib/two.jar".len() as u64);
}

#[test]
fn unsupported_relation_is_an_analysis_state_not_an_error_or_a_no_match() {
    let temp = TempDir::new();
    let archive = temp.write("app.zip", &fixture());
    let request_limits = limits(u64::MAX);

    for relation in [
        QueryRelation::ReferencesDefinition,
        QueryRelation::MayDispatchTo,
    ] {
        let spec = Spec::symbol(relation, "com/example/Agent", &[ConsumerKind::Resource], 0);
        let (value, cli) = query_via_cli(&archive, &request_limits, &spec);
        let direct = query_direct(&archive, &request_limits, &spec);
        assert_same_report(&cli, &direct);

        assert_eq!(
            cli.analysis,
            QueryAnalysis::UnsupportedAnalysis { relation },
            "the analysis state is reported instead of a resolution"
        );
        assert_eq!(
            value["result"]["report"]["analysis"]["kind"],
            "unsupported_analysis"
        );
        assert_eq!(
            value["result"]["report"]["analysis"]["relation"],
            serde_json::to_value(relation).expect("relation tag")
        );
        // A successful response, not an error and not a silent empty result.
        assert_eq!(value["status"], "ok");
        assert_eq!(value["result"]["report"]["execution"]["status"], "complete");
        // P1 keeps the raw constant-pool candidates for these relations (task 2.4), so the
        // scan really runs and reports what it examined. The analysis state above still says
        // that no definition or dispatch resolution was performed.
        assert_eq!(
            value["result"]["report"]["coverage"]["dimensions"]["artifact_structural"]["state"],
            "complete_within_schema"
        );
        let diagnostic = value["result"]["report"]["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .iter()
            .find(|diagnostic| diagnostic["code"] == "query_relation_unsupported")
            .expect("the unsupported relation is explained by a diagnostic");
        assert_eq!(diagnostic["severity"], "warning");
        assert!(
            diagnostic["message"]
                .as_str()
                .expect("diagnostic message")
                .contains("does not analyze"),
            "the diagnostic names the missing analysis: {diagnostic}"
        );
        assert!(cli.page.cursor.is_none());
    }

    // The same target under a performed relation stays distinguishable: analysis says it ran,
    // and the item list is a real answer.
    let performed = Spec::symbol(
        QueryRelation::MentionsSymbol,
        "com/example/Agent",
        &[ConsumerKind::Resource],
        0,
    );
    let (_, cli) = query_via_cli(&archive, &request_limits, &performed);
    assert_eq!(cli.analysis, QueryAnalysis::Performed);
    assert_eq!(cli.items.len(), 5);

    // A performed search with no match is a third, separate state: empty items, analysis says
    // the scan ran and coverage says the schema was scanned.
    let missing = Spec::symbol(
        QueryRelation::MentionsSymbol,
        "com/example/Missing",
        &[ConsumerKind::Resource],
        0,
    );
    let (_, cli) = query_via_cli(&archive, &request_limits, &missing);
    assert_eq!(cli.analysis, QueryAnalysis::Performed);
    assert!(cli.items.is_empty());
    assert_eq!(
        cli.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
}

#[test]
fn unimplemented_consumer_categories_stay_visible_in_the_cli_report() {
    let temp = TempDir::new();
    let archive = temp.write("app.zip", &fixture());
    let request_limits = limits(u64::MAX);
    let spec = Spec::symbol(
        QueryRelation::MentionsSymbol,
        "com/example/Agent",
        &[
            ConsumerKind::Resource,
            ConsumerKind::Verification,
            ConsumerKind::Debug,
        ],
        0,
    );
    let (value, cli) = query_via_cli(&archive, &request_limits, &spec);
    let direct = query_direct(&archive, &request_limits, &spec);
    assert_same_report(&cli, &direct);

    // Requested categories P1 declares but does not scan are a schema fact of the shared
    // consumer set: the request is answered, and neither completion nor a negative conclusion is
    // claimed for them.
    assert_eq!(
        cli.coverage.unsupported_categories,
        vec![ConsumerKind::Verification, ConsumerKind::Debug]
    );
    assert_eq!(
        value["result"]["report"]["coverage"]["unsupported_categories"],
        json!(["verification", "debug"])
    );
    assert_ne!(
        cli.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(matches!(cli.execution, ExecutionReport::Complete { .. }));
    // The scanned evidence is unchanged; only what may be claimed is bounded.
    assert_eq!(cli.items.len(), 5);
}

#[test]
fn physical_scope_declares_snapshot_all_or_the_artifact_tree() {
    let temp = TempDir::new();
    let archive = temp.write("app.zip", &fixture());
    let request_limits = limits(u64::MAX);
    let target = |scope: PhysicalScope| {
        Spec::symbol(
            QueryRelation::MentionsSymbol,
            "com/example/Agent",
            &[ConsumerKind::Resource],
            0,
        )
        .with_scope(scope)
    };

    let snapshot_all = target(PhysicalScope::SnapshotAll);
    let (_, over_all) = query_via_cli(&archive, &request_limits, &snapshot_all);
    assert_eq!(over_all.physical.scope, PhysicalScope::SnapshotAll);

    let tree = target(PhysicalScope::ArtifactTree {
        root_container: jarde::ContainerId("root".into()),
    });
    let (value, over_tree) = query_via_cli(&archive, &request_limits, &tree);
    let direct = query_direct(&archive, &request_limits, &tree);
    assert_same_report(&over_tree, &direct);
    assert_eq!(
        value["result"]["report"]["physical"]["scope"]["kind"],
        "artifact_tree"
    );
    assert_eq!(
        over_tree.items, over_all.items,
        "the same fixture is the same root tree"
    );
    assert_eq!(over_tree.physical.snapshot, over_all.physical.snapshot);
    assert_eq!(
        over_tree.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );

    // The library still owns the scope validation: a root the snapshot never established is
    // rejected instead of being scanned as if it existed.
    let unknown_root = target(PhysicalScope::ArtifactTree {
        root_container: jarde::ContainerId("not-the-root".into()),
    });
    let output = run_stdin(&request(
        &archive,
        &request_limits,
        unknown_root.operation_json(),
    ));
    let value = assert_error_code(&output, "query_artifact_tree_root_mismatch");
    assert_eq!(value["error"]["kind"], "invalid_input");

    // The snapshot identity is the one this input resolves to, and a cursor can be echoed back
    // verbatim, which is what a client does on the second page.
    let mut budget = Budget::new(request_limits);
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::Path(archive.clone()), &mut budget)
        .expect("open the fixture directly");
    assert_eq!(&over_all.physical.snapshot, snapshot.id());
    assert_eq!(snapshot.id().0.len(), 64, "blake3 content digest");
}

#[test]
fn pagination_continuation_reproduces_one_full_scan() {
    let temp = TempDir::new();
    let archive = temp.write("app.zip", &fixture());
    let request_limits = limits(u64::MAX);
    let spec = |cursor: Option<Value>, max_items: u64| {
        let spec = Spec::symbol(
            QueryRelation::MentionsSymbol,
            "com/example/Agent",
            &[ConsumerKind::Resource],
            max_items,
        );
        match cursor {
            Some(cursor) => spec.with_cursor(cursor),
            None => spec,
        }
    };

    let (_, full) = query_via_cli(&archive, &request_limits, &spec(None, 0));
    assert_eq!(full.items.len(), 5);
    assert!(!full.page.has_more);
    assert!(full.page.cursor.is_none());

    // Page 1 is requested without a cursor field at all, to pin the optional field.
    let mut first = spec(Some(Value::Null), 2).operation_json();
    first
        .as_object_mut()
        .expect("operation object")
        .remove("cursor");
    let output = run_stdin(&request(&archive, &request_limits, first));
    let value = assert_ok(&output, "query");
    let first: QueryReport =
        serde_json::from_value(value["result"]["report"].clone()).expect("query report");

    let mut collected = first.items.clone();
    // The continuation value stays on the wire: a client echoes it back verbatim.
    let mut cursor = wire_cursor(&value);
    assert!(first.page.has_more);
    assert_eq!(first.page.returned_items, 2);
    assert!(cursor.is_some(), "a truncated page carries a cursor");
    assert!(first.page.cursor.is_some());
    let mut pages = 1_usize;
    while let Some(resume) = cursor {
        let (value, report) = query_via_cli(&archive, &request_limits, &spec(Some(resume), 2));
        pages += 1;
        assert!(
            matches!(report.execution, ExecutionReport::Complete { .. }),
            "a page limit is not a degraded execution"
        );
        assert_eq!(
            report.page.returned_items,
            u64::try_from(report.items.len()).expect("page length fits u64")
        );
        assert!(report.page.returned_items <= 2);
        assert_eq!(report.page.has_more, report.page.cursor.is_some());
        collected.extend(report.items.iter().cloned());
        cursor = wire_cursor(&value);
        assert_eq!(cursor.is_some(), report.page.cursor.is_some());
        assert!(pages < 8, "pagination must terminate");
    }
    assert!(pages > 1, "the fixture must span more than one page");
    assert_eq!(collected, full.items, "pages must not repeat or skip items");
    assert_eq!(collected.len(), 5);
}

/// The `page.cursor` value of a CLI answer, as the client receives it.
fn wire_cursor(value: &Value) -> Option<Value> {
    match &value["result"]["report"]["page"]["cursor"] {
        Value::Object(_) => Some(value["result"]["report"]["page"]["cursor"].clone()),
        _ => None,
    }
}

#[test]
fn cursor_mismatch_uses_the_stable_error_code() {
    let temp = TempDir::new();
    let archive = temp.write("app.zip", &fixture());
    let other = temp.write("other.zip", &zip(&[(MANIFEST_ENTRY, &manifest())]));
    let request_limits = limits(u64::MAX);
    let spec = Spec::symbol(
        QueryRelation::MentionsSymbol,
        "com/example/Agent",
        &[ConsumerKind::Resource],
        2,
    );

    let (value, report) = query_via_cli(&archive, &request_limits, &spec);
    let cursor = report
        .page
        .cursor
        .clone()
        .expect("a truncated page carries a cursor");
    assert_eq!(cursor.engine_schema, jarde::QUERY_ENGINE_SCHEMA);
    assert_eq!(
        value["result"]["report"]["page"]["cursor"]["engine_schema"],
        jarde::QUERY_ENGINE_SCHEMA
    );
    let wire = value["result"]["report"]["page"]["cursor"].clone();

    let mismatches: [(&str, PathBuf, Value); 5] = [
        ("snapshot", other.clone(), wire.clone()),
        ("relation", archive.clone(), {
            let mut tampered = wire.clone();
            tampered["relation"] = json!("literal_value");
            tampered
        }),
        ("consumer schema", archive.clone(), {
            let mut tampered = wire.clone();
            tampered["consumers"]["kinds"] = json!(["type"]);
            tampered
        }),
        ("consumer schema", archive.clone(), {
            let mut tampered = wire.clone();
            tampered["consumers"]["version"] = json!(2);
            tampered
        }),
        ("digest", archive.clone(), {
            let mut tampered = wire.clone();
            tampered["boundary"]["item_index"] = json!(3);
            tampered
        }),
    ];
    for (bound_field, input, cursor) in mismatches {
        let output = run_stdin(&request(
            &input,
            &request_limits,
            spec.clone().with_cursor(cursor).operation_json(),
        ));
        let value = assert_error_code(&output, "query_cursor_mismatch");
        assert_eq!(value["error"]["kind"], "invalid_input");
        assert!(
            value["error"]["message"]
                .as_str()
                .expect("error message")
                .contains(bound_field),
            "the error names the mismatched binding {bound_field}: {value}"
        );
    }
}

/// R1 through the CLI: a cursor the library issued for target A must not resume as target
/// B, while the same identity still continues page by page.
#[test]
fn cursor_for_another_target_is_rejected_by_the_cli() {
    let temp = TempDir::new();
    let archive = temp.write("app.zip", &fixture());
    let request_limits = limits(u64::MAX);
    let target_a = Spec::symbol(
        QueryRelation::MentionsSymbol,
        "com/example/Agent",
        &[ConsumerKind::Resource],
        2,
    );

    // The cursor is the library's own value for target A; the CLI only transports it.
    let (_, first) = query_via_cli(&archive, &request_limits, &target_a);
    let cursor = first
        .page
        .cursor
        .clone()
        .expect("a truncated page carries a cursor");
    assert_eq!(cursor.engine_schema, jarde::QUERY_ENGINE_SCHEMA);
    assert_eq!(
        cursor.target,
        QueryTarget::Symbol {
            value: SymbolRef::Class {
                owner: JvmBytes(b"com/example/Agent".to_vec()),
            },
        }
    );
    let wire = serde_json::to_value(&cursor).expect("serialize the library cursor");

    // Target B is another query identity: A's published prefix does not describe B's pages.
    let changed = Spec::symbol(
        QueryRelation::MentionsSymbol,
        "com/example/Main",
        &[ConsumerKind::Resource],
        0,
    )
    .with_cursor(wire.clone());
    let output = run_stdin(&request(
        &archive,
        &request_limits,
        changed.operation_json(),
    ));
    let value = assert_error_code(&output, "query_cursor_mismatch");
    assert_eq!(value["error"]["kind"], "invalid_input");
    assert!(
        value["error"]["message"]
            .as_str()
            .expect("error message")
            .contains("cursor target"),
        "the CLI answer names the target binding: {value}"
    );
    let answer = value.to_string();
    assert!(
        value["result"].is_null() && !answer.contains("\"items\"") && !answer.contains("complete"),
        "a rejected continuation publishes no items and no completion claim: {value}"
    );

    // The unchanged identity still continues from the same cursor, and the page carries
    // exactly the items that follow the first page of the full scan.
    let (_, full) = query_via_cli(
        &archive,
        &request_limits,
        &Spec::symbol(
            QueryRelation::MentionsSymbol,
            "com/example/Agent",
            &[ConsumerKind::Resource],
            0,
        ),
    );
    assert_eq!(full.items.len(), 5);
    assert_eq!(first.items.len(), 2);
    let (value, continued) = query_via_cli(
        &archive,
        &request_limits,
        &target_a.clone().with_cursor(wire),
    );
    assert!(
        matches!(continued.execution, ExecutionReport::Complete { .. }),
        "a page limit is not a degraded execution"
    );
    assert_eq!(continued.items, full.items[2..4].to_vec());
    assert_eq!(
        continued.page.has_more,
        continued.page.cursor.is_some(),
        "the CLI page keeps `has_more` and its cursor in agreement"
    );
    assert!(wire_cursor(&value).is_some());
}

#[test]
fn result_item_budget_keeps_the_reliable_prefix_and_names_the_dimension() {
    let temp = TempDir::new();
    let archive = temp.write("app.zip", &fixture());
    let request_limits = limits(u64::MAX);
    let spec = Spec::symbol(
        QueryRelation::MentionsSymbol,
        "com/example/Agent",
        &[ConsumerKind::Resource],
        0,
    );

    // Every enumerated entry costs one result item, so exactly one item is still publishable.
    let tight = Limits {
        result_items: FIXTURE_ENTRIES + 1,
        ..request_limits
    };
    let (value, report) = query_via_cli(&archive, &tight, &spec);
    assert_eq!(report.items.len(), 1);
    assert_eq!(report.coverage.scanned_items, 1);
    assert!(report.page.has_more);
    assert!(report.page.cursor.is_some());
    match &report.execution {
        ExecutionReport::Partial { reason, usage } => {
            assert_eq!(
                reason,
                &TerminationReason::BudgetExceeded {
                    dimension: jarde::BudgetDimension::ResultItems,
                }
            );
            assert_eq!(usage.result_items, tight.result_items);
        }
        other => panic!("a budget stop keeps the reliable prefix as partial, got {other:?}"),
    }
    let wire = &value["result"]["report"];
    assert_eq!(wire["execution"]["status"], "partial");
    assert_eq!(wire["execution"]["reason"]["kind"], "budget_exceeded");
    assert_eq!(wire["execution"]["reason"]["dimension"], "result_items");
    assert_eq!(
        wire["execution"]["usage"]["result_items"],
        tight.result_items
    );
    assert_eq!(wire["page"]["has_more"], true);
    assert!(
        wire["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .iter()
            .any(|diagnostic| diagnostic["code"] == "budget_exceeded_result_items"),
        "the terminal diagnostic explains the stop: {wire}"
    );
    let direct = query_direct(&archive, &tight, &spec);
    assert_same_report(&report, &direct);
}

#[test]
fn cancellation_is_library_state_because_the_cli_request_has_no_token() {
    let temp = TempDir::new();
    let archive = temp.write("app.zip", &fixture());
    let request_limits = limits(u64::MAX);
    let spec = Spec::symbol(
        QueryRelation::MentionsSymbol,
        "com/example/Agent",
        &[ConsumerKind::Resource],
        0,
    );

    // The one-shot protocol has no field that could carry a cancellation token, and the
    // adapter rejects an invented one instead of ignoring it.
    let mut invented = spec.operation_json();
    invented["cancel"] = json!(true);
    let output = run_stdin(&request(&archive, &request_limits, invented));
    let value = assert_error_code(&output, "cli_request_json");
    assert!(
        value["error"]["message"]
            .as_str()
            .expect("error message")
            .contains("unknown field"),
        "{value}"
    );

    // Cancellation stays a library state, so the CLI cannot express it; the state itself is
    // reached through the same `Engine::query` call and is never reported as complete.
    let mut budget = Budget::new(request_limits.clone());
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::Path(archive), &mut budget)
        .expect("open the fixture directly");
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(request_limits.clone(), token);
    let report = engine
        .query(&snapshot, &spec.library_request(&snapshot), &mut budget)
        .expect("a cancelled query still returns a report");
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
    assert_ne!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
}

#[test]
fn stdout_contract_and_output_budget() {
    let temp = TempDir::new();
    let archive = temp.write("app.zip", &fixture());
    let spec = Spec::symbol(
        QueryRelation::MentionsSymbol,
        "com/example/Agent",
        &[ConsumerKind::Resource],
        0,
    );

    let baseline = run_stdin(&request(&archive, &limits(u64::MAX), spec.operation_json()));
    let baseline = assert_ok(&baseline, "query");
    let core_output = baseline["result"]["report"]["execution"]["usage"]["output_bytes"]
        .as_u64()
        .expect("core output usage is u64");
    let response_bytes = baseline["transport"]["response_bytes"]
        .as_u64()
        .expect("transport response bytes is u64");

    // The needed budget is measured rather than guessed: the report carries the wall-clock
    // milliseconds, so two runs of the same request may differ by a byte. Each short budget is
    // still rejected as a whole and reports the exact size it needed, which converges here.
    let mut needed = response_bytes;
    let mut exact = None;
    for _ in 0..4 {
        let mut exact_limits = limits(u64::MAX);
        exact_limits.output_bytes = needed;
        let output = run_stdin(&request(&archive, &exact_limits, spec.operation_json()));
        let value = response(&output);
        if output.status.code() == Some(0) {
            assert_eq!(value["status"], "ok");
            assert_eq!(
                value["transport"]["response_bytes"], needed,
                "response_bytes is the whole document plus its one newline"
            );
            assert_eq!(
                u64::try_from(output.stdout.len()).expect("stdout length fits u64"),
                needed
            );
            assert_eq!(
                value["result"]["report"]["execution"]["usage"]["output_bytes"],
                core_output
            );
            exact = Some(needed);
            break;
        }
        assert_eq!(value["status"], "error");
        assert_eq!(value["error"]["kind"], "budget_exceeded");
        assert_eq!(value["error"]["dimension"], "output_bytes");
        assert_eq!(value["error"]["limit"], needed);
        assert_eq!(value["error"]["consumed"], core_output);
        assert!(
            value.get("transport").is_none(),
            "a rejected response carries no transport accounting: {value}"
        );
        assert!(
            !output
                .stdout
                .windows(b"\"status\":\"ok\"".len())
                .any(|window| { window == b"\"status\":\"ok\"" }),
            "a rejected response is never partially published"
        );
        let requested = value["error"]["requested"]
            .as_u64()
            .expect("requested bytes are u64");
        assert!(
            requested > needed,
            "the error reports the exact document size: {value}"
        );
        needed = requested;
    }
    assert!(
        exact.is_some(),
        "the measured document size must be reachable within the retry budget"
    );
}

#[test]
fn protocol_errors_use_the_existing_error_contract() {
    let temp = TempDir::new();
    let archive = temp.write("app.zip", &fixture());
    let spec = Spec::symbol(
        QueryRelation::MentionsSymbol,
        "com/example/Agent",
        &[ConsumerKind::Resource],
        0,
    );

    let broken = |mutate: &dyn Fn(&mut Value)| {
        let mut operation = spec.operation_json();
        mutate(&mut operation);
        run_stdin(&request(&archive, &limits(u64::MAX), operation))
    };

    // Unknown operation, unknown field, unknown enum names and an ill-formed view are all
    // request-decoding failures, exactly as for the P0 operations. Nested request payloads keep
    // the strictness of the shared library schema: `target` and `consumers` reject siblings,
    // while the scope value follows `PhysicalScope`, which the library does not declare strict,
    // so this adapter does not invent a stricter rule of its own.
    let unknown_operation = run_stdin(&request(
        &archive,
        &limits(u64::MAX),
        json!({"kind": "query_relations"}),
    ));
    assert_error_code(&unknown_operation, "cli_request_json");

    let unknown_field = broken(&|operation| operation["unexpected"] = json!(true));
    let value = assert_error_code(&unknown_field, "cli_request_json");
    assert!(
        value["error"]["message"]
            .as_str()
            .expect("error message")
            .contains("unknown field"),
        "{value}"
    );

    let unknown_relation = broken(&|operation| operation["relation"] = json!("mentions_symbols"));
    assert_error_code(&unknown_relation, "cli_request_json");

    let unknown_consumer = broken(&|operation| {
        operation["consumers"] = json!({"version": 1, "kinds": ["resource", "nope"]});
    });
    assert_error_code(&unknown_consumer, "cli_request_json");

    let unknown_target = broken(&|operation| {
        operation["target"] = json!({"kind": "constant", "value": 1});
    });
    assert_error_code(&unknown_target, "cli_request_json");

    let unknown_physical = broken(&|operation| {
        operation["physical"] = json!({"kind": "runtime_view"});
    });
    assert_error_code(&unknown_physical, "cli_request_json");

    // Requests that decode but are rejected by the shared request compiler keep their library
    // error codes and exit with failure, so the CLI never invents a second error contract.
    let unsupported_schema = broken(&|operation| {
        operation["consumers"] = json!({"version": 2, "kinds": ["resource"]});
    });
    let value = assert_error_code(&unsupported_schema, "query_consumer_schema_version");
    assert_eq!(value["error"]["kind"], "unsupported");

    let mismatched_target = broken(&|operation| {
        operation["relation"] = json!("literal_value");
    });
    let value = assert_error_code(&mismatched_target, "query_target_relation_mismatch");
    assert_eq!(value["error"]["kind"], "invalid_input");

    // The 1 MiB control-plane limit is checked before JSON parsing, as for every operation.
    let oversized = run_stdin(&vec![b' '; MAX_REQUEST_BYTES + 1]);
    let value = assert_error_code(&oversized, "cli_request_too_large");
    assert_eq!(value["error"]["kind"], "invalid_input");
    assert_eq!(
        value["usage"],
        serde_json::to_value(UsageSnapshot::default()).expect("usage snapshot")
    );
}
