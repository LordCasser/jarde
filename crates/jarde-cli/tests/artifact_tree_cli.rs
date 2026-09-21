//! `bind-prefixed-load-roots` 2.1/2.2: the JSON `enumerate_artifact_tree` operation and the loop
//! it closes.
//!
//! The operation is deliberately thin: it opens `input_path` and hands *that* snapshot to the
//! library's existing artifact-tree entry, so the response is the library's own report. What these
//! tests hold, one by one, is
//!
//! * that "thin" is true — the CLI document and the library document are one document, field by
//!   field, for a whole tree, for a bounded tree and for a tree with a broken child;
//! * that ordinary `enumerate` still does not recurse, while the explicit tree operation does;
//! * that the operation declares nothing of its own: it has no request field at all, and the
//!   snapshot it echoes is the one it opened;
//! * that a caller can use the identities it returns to *declare* a prefixed load position and
//!   recover a method — for a WAR's `WEB-INF/classes/` and, to show the mechanism generalises to
//!   the same prefix shape, for a controlled `BOOT-INF/classes/` sample. The latter is a prefix
//!   sample and **not** a claim of Spring Boot loader support: nothing here reads
//!   `classpath.idx`, orders `BOOT-INF/lib`, or executes a launcher.
//!
//! The fixtures are built in memory by this file (a small STORED-only ZIP writer, so no new
//! dependency joins the CLI package) and written to a temp file, because the adapter's own entry
//! takes a path.

use jarde::{
    AnalysisStage, ArchiveNameBytes, ArtifactInput, ArtifactTreeReport, Budget, BudgetDimension,
    ClassTarget, ContainerId, ContainerOrigin, DelegationPolicy, Engine, ExecutionReport,
    HeaderProvider, InspectionMode, JvmBytes, LayoutMode, Limits, LoadDomain, LoadRoot, LoaderId,
    MethodAnalysisRequest, ModuleMode, MultiReleasePolicy, PhysicalDefinitionId, PhysicalEntryId,
    PhysicalMethodId, PhysicalScope, PhysicalVariant, PhysicalView, RecoveryEvidenceRequest,
    ResolutionEnvironment, RuntimeProfile, RuntimeUncertainty, RuntimeView, TerminationReason,
};
use serde_json::{Map, Value, json};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const BIN: &str = env!("CARGO_BIN_EXE_jarde-cli");
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

/// The prefix a WAR's class layer is declared under, spelled by the caller.
const WAR_CLASSES: &[u8] = b"WEB-INF/classes/";
/// The same for the controlled Boot-shaped sample.
const BOOT_CLASSES: &[u8] = b"BOOT-INF/classes/";

// ---------------------------------------------------------------------------------------------
// Temp files
// ---------------------------------------------------------------------------------------------

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
            "jarde-cli-tree-{}-{nonce}-{}",
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

// ---------------------------------------------------------------------------------------------
// Fixtures: a stored-only ZIP, one class with a body the presentation can write
// ---------------------------------------------------------------------------------------------

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

/// One stored entry of the in-memory fixtures.
struct Stored<'a> {
    name: &'a [u8],
    data: &'a [u8],
}

/// A minimal STORED-only ZIP writer, enough for the reader's own central-directory path.
///
/// The CLI test package declares no archive dependency on purpose (it is a dev-dependency of the
/// root package, and adding one here would grow the CLI's own manifest and lock entry), so the
/// few dozen bytes of the format the fixtures need are written here.
fn zip(entries: &[Stored<'_>]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut directory = Vec::new();
    for entry in entries {
        let offset = u32::try_from(output.len()).expect("fixture archive fits u32");
        let crc = crc32(entry.data);
        let size = u32::try_from(entry.data.len()).expect("fixture entry fits u32");
        let name_len = u16::try_from(entry.name.len()).expect("fixture name fits u16");
        // Local file header.
        output.extend_from_slice(&0x0403_4b50_u32.to_le_bytes());
        output.extend_from_slice(&20_u16.to_le_bytes()); // version needed
        output.extend_from_slice(&0_u16.to_le_bytes()); // flags
        output.extend_from_slice(&0_u16.to_le_bytes()); // method: stored
        output.extend_from_slice(&0_u16.to_le_bytes()); // mod time
        output.extend_from_slice(&0_u16.to_le_bytes()); // mod date
        output.extend_from_slice(&crc.to_le_bytes());
        output.extend_from_slice(&size.to_le_bytes());
        output.extend_from_slice(&size.to_le_bytes());
        output.extend_from_slice(&name_len.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes()); // extra length
        output.extend_from_slice(entry.name);
        output.extend_from_slice(entry.data);
        // Central directory record.
        directory.extend_from_slice(&0x0201_4b50_u32.to_le_bytes());
        directory.extend_from_slice(&20_u16.to_le_bytes()); // version made by
        directory.extend_from_slice(&20_u16.to_le_bytes()); // version needed
        directory.extend_from_slice(&0_u16.to_le_bytes()); // flags
        directory.extend_from_slice(&0_u16.to_le_bytes()); // method
        directory.extend_from_slice(&0_u16.to_le_bytes()); // mod time
        directory.extend_from_slice(&0_u16.to_le_bytes()); // mod date
        directory.extend_from_slice(&crc.to_le_bytes());
        directory.extend_from_slice(&size.to_le_bytes());
        directory.extend_from_slice(&size.to_le_bytes());
        directory.extend_from_slice(&name_len.to_le_bytes());
        directory.extend_from_slice(&0_u16.to_le_bytes()); // extra length
        directory.extend_from_slice(&0_u16.to_le_bytes()); // comment length
        directory.extend_from_slice(&0_u16.to_le_bytes()); // disk number
        directory.extend_from_slice(&0_u16.to_le_bytes()); // internal attributes
        directory.extend_from_slice(&0_u32.to_le_bytes()); // external attributes
        directory.extend_from_slice(&offset.to_le_bytes());
        directory.extend_from_slice(entry.name);
    }
    let directory_offset = u32::try_from(output.len()).expect("fixture archive fits u32");
    let directory_size = u32::try_from(directory.len()).expect("fixture directory fits u32");
    let count = u16::try_from(entries.len()).expect("fixture entry count fits u16");
    output.extend_from_slice(&directory);
    // End of central directory.
    output.extend_from_slice(&0x0605_4b50_u32.to_le_bytes());
    output.extend_from_slice(&0_u16.to_le_bytes());
    output.extend_from_slice(&0_u16.to_le_bytes());
    output.extend_from_slice(&count.to_le_bytes());
    output.extend_from_slice(&count.to_le_bytes());
    output.extend_from_slice(&directory_size.to_le_bytes());
    output.extend_from_slice(&directory_offset.to_le_bytes());
    output.extend_from_slice(&0_u16.to_le_bytes()); // comment length
    output
}

fn u16b(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn u32b(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes());
}

/// The body both recovery cases present: `if (local1 != 0) { return; } else { return; }`, the
/// shape `crates/jarde-cli/tests/json_cli.rs` uses for the same purpose, written the way a
/// compiler writes it.
const RECOVERY_BODY: &[u8] = &[
    0x03, // 0: iconst_0
    0x3c, // 1: istore_1
    0x1b, // 2: iload_1
    0x99, 0x00, 0x04, // 3: ifeq 7
    0xb1, // 6: return
    0xb1, // 7: return
];

/// One class with a `run()V` method holding [`RECOVERY_BODY`], declaring `this_class` as given.
fn class_bytes(this_class: &[u8]) -> Vec<u8> {
    let mut output = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0);
    u16b(&mut output, 52);
    u16b(&mut output, 8); // #1..#7
    for name in [
        this_class,
        b"java/lang/Object".as_slice(),
        b"run".as_slice(),
        b"()V".as_slice(),
        b"Code".as_slice(),
    ] {
        output.push(1); // CONSTANT_Utf8
        u16b(
            &mut output,
            u16::try_from(name.len()).expect("fixture name fits u16"),
        );
        output.extend_from_slice(name);
    }
    output.push(7); // #6 the class entry of this_class
    u16b(&mut output, 1);
    output.push(7); // #7 the class entry of java/lang/Object
    u16b(&mut output, 2);
    u16b(&mut output, 0x0021);
    u16b(&mut output, 6); // this_class
    u16b(&mut output, 7); // super_class
    u16b(&mut output, 0); // interfaces
    u16b(&mut output, 0); // fields
    u16b(&mut output, 1); // methods
    u16b(&mut output, 0x0001);
    u16b(&mut output, 3); // "run"
    u16b(&mut output, 4); // "()V"
    u16b(&mut output, 1);
    u16b(&mut output, 5); // "Code"
    let mut code = Vec::new();
    u16b(&mut code, 1); // max_stack
    u16b(&mut code, 2); // max_locals
    u32b(
        &mut code,
        u32::try_from(RECOVERY_BODY.len()).expect("fixture code fits u32"),
    );
    code.extend_from_slice(RECOVERY_BODY);
    u16b(&mut code, 0); // exception handlers
    u16b(&mut code, 0); // code attributes
    u32b(
        &mut output,
        u32::try_from(code.len()).expect("fixture attribute fits u32"),
    );
    output.extend_from_slice(&code);
    u16b(&mut output, 0); // class attributes
    output
}

/// The prefix fixture: one class entry under `prefix`, the application directory of the same
/// shape and one nested library beside it.
fn war_with(prefix: &[u8], class: &[u8]) -> Vec<u8> {
    let mut class_entry = prefix.to_vec();
    class_entry.extend_from_slice(b"com/demo/App.class");
    let library = zip(&[Stored {
        name: b"com/demo/Lib.class",
        data: class,
    }]);
    let library_entry: &[u8] = if prefix == BOOT_CLASSES {
        b"BOOT-INF/lib/L.jar"
    } else {
        b"WEB-INF/lib/L.jar"
    };
    zip(&[
        Stored {
            name: &class_entry,
            data: class,
        },
        Stored {
            name: library_entry,
            data: &library,
        },
    ])
}

// ---------------------------------------------------------------------------------------------
// Requests and responses
// ---------------------------------------------------------------------------------------------

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
        class_headers: value,
        method_bodies: value,
        ir_items: value,
        ir_edges: value,
        analysis_steps: value,
        normalization_clones: value,
        nested_depth: value,
        dependency_depth: value,
        elapsed_millis: u64::MAX,
    }
}

/// Limits with every P2 dimension funded, for the recovery loop.
fn analysis_limits() -> Limits {
    Limits {
        class_headers: 4,
        method_bodies: 4,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        dependency_depth: 4,
        ..limits(u64::MAX)
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

fn run_stdin(bytes: &[u8]) -> Output {
    let mut command = Command::new(BIN);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("spawn jarde-cli");
    child
        .stdin
        .take()
        .expect("stdin is piped")
        .write_all(bytes)
        .expect("write the request");
    child.wait_with_output().expect("wait for jarde-cli")
}

/// One successful CLI answer, with its transport length checked against the document.
fn assert_ok(output: &Output, operation: &str) -> Value {
    assert!(
        output.status.success(),
        "{operation}: the CLI failed: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value =
        serde_json::from_slice(&output.stdout).expect("the CLI writes one JSON document");
    assert_eq!(value["status"], json!("ok"), "{operation}: {value}");
    let bytes = u64::try_from(output.stdout.len()).expect("response length fits u64");
    assert_eq!(
        value["transport"]["response_bytes"],
        json!(bytes),
        "{operation}: the transport length is the document's own"
    );
    value
}

/// The same document with every `elapsed_millis` deleted: the one field two runs cannot agree on.
fn strip_elapsed(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(key, _)| key.as_str() != "elapsed_millis")
                .map(|(key, child)| (key.clone(), strip_elapsed(child)))
                .collect::<Map<String, Value>>(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(strip_elapsed).collect()),
        other => other.clone(),
    }
}

fn tree_report(value: &Value) -> ArtifactTreeReport {
    serde_json::from_value(value["result"]["report"].clone())
        .expect("the operation answers with the library's own report document")
}

/// The environment one caller declares from a tree report: the root position of the tree with an
/// explicit raw prefix, and nothing else.
fn environment_with_prefix(report: &ArtifactTreeReport, prefix: &[u8]) -> ResolutionEnvironment {
    let root = report
        .containers
        .iter()
        .find(|container| container.origin.steps.is_empty())
        .expect("the tree report names its root container")
        .origin
        .clone();
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Container {
            origin: root,
            prefix: ArchiveNameBytes(prefix.to_vec()),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: report.view.snapshot.clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: domain.clone(),
        },
        domains: vec![domain],
        providers: Vec::<HeaderProvider>::new(),
    }
}

fn container_of(report: &ArtifactTreeReport, leaf: &[u8]) -> ContainerOrigin {
    report
        .containers
        .iter()
        .filter(|container| {
            container
                .origin
                .steps
                .last()
                .is_some_and(|step| step.via_raw_name.0 == leaf)
        })
        .map(|container| container.origin.clone())
        .next()
        .unwrap_or_else(|| {
            panic!(
                "the report names a container reached by \"{}\"",
                String::from_utf8_lossy(leaf)
            )
        })
}

/// The physical entry one container of a report holds at one raw name.
fn entry_of(
    report: &ArtifactTreeReport,
    origin: &ContainerOrigin,
    raw_name: &[u8],
) -> PhysicalEntryId {
    report
        .containers
        .iter()
        .find(|container| &container.origin == origin)
        .expect("the report holds the container")
        .entries
        .iter()
        .find(|entry| entry.id.raw_name.0 == raw_name)
        .unwrap_or_else(|| {
            panic!(
                "the container holds \"{}\"",
                String::from_utf8_lossy(raw_name)
            )
        })
        .id
        .clone()
}

fn recovery_operation(request: &MethodAnalysisRequest) -> Value {
    let mut operation =
        serde_json::to_value(request).expect("a method-analysis request serializes");
    let fields = operation
        .as_object_mut()
        .expect("the request serializes as an object");
    fields.insert("kind".to_string(), json!("recover_method"));
    // The evidence selection this loop asks for is stated on **both** sides of the wire: this test
    // reads the segment table's physical paths, and an unrequested category is a category the
    // library does not materialize (`add-demand-driven-core-results`, D1).
    fields.insert(
        "evidence".to_string(),
        serde_json::to_value(RecoveryEvidenceRequest::all()).expect("a selection serializes"),
    );
    operation
}

/// The whole CLI loop for one prefix fixture: enumerate, declare the position, recover, and
/// compare the answer with the library's own run of the same request.
///
/// The position — container origin and prefix — comes from the enumeration and from the caller's
/// own declaration; the class-bytes identity is read through the library's public inspection entry
/// on the entry the enumeration named, exactly as the method-analysis operation's other callers
/// derive it, so nothing here forges a child identity or rewrites a raw name.
fn cli_enumeration_then_recovery(name: &str, prefix: &[u8]) -> Value {
    let temp = TempDir::new();
    let class = class_bytes(b"com/demo/App");
    let war = war_with(prefix, &class);
    let path = temp.write(name, &war);

    let enumeration = run_stdin(&request(
        &path,
        &limits(1 << 20),
        json!({"kind": "enumerate_artifact_tree"}),
    ));
    let enumerated = assert_ok(&enumeration, "enumerate_artifact_tree");
    let report = tree_report(&enumerated);

    let origin = report
        .containers
        .iter()
        .find(|container| container.origin.steps.is_empty())
        .expect("the WAR's root container")
        .origin
        .clone();
    let mut class_entry = prefix.to_vec();
    class_entry.extend_from_slice(b"com/demo/App.class");
    let entry_id = entry_of(&report, &origin, &class_entry);

    // The class bytes' own identity, read through the public inspection entry of the *entry* the
    // enumeration published.
    let engine = Engine::new();
    let mut budget = Budget::new(analysis_limits());
    let snapshot = engine
        .open(ArtifactInput::Path(path.clone()), &mut budget)
        .expect("open the fixture the CLI enumerated");
    assert_eq!(
        snapshot.id(),
        &report.view.snapshot,
        "the CLI enumerated the snapshot it echoes"
    );
    let listed = engine
        .enumerate(&snapshot, &mut budget)
        .expect("the library lists the fixture's own entries");
    let entry = listed
        .entries
        .iter()
        .find(|entry| entry.id == entry_id)
        .expect("the entry the CLI published is the library's own record")
        .clone();
    let header = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Entry(&entry),
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the entry's header is readable");
    let definition = PhysicalDefinitionId {
        location: header.source.location.clone(),
        class_bytes: header.source.class_bytes.clone(),
        variant: PhysicalVariant::Base,
    };
    assert_eq!(
        definition.entry().map(|entry| entry.raw_name.0.clone()),
        Some(class_entry.clone()),
        "the definition keeps the physical prefixed path"
    );

    let analysis = MethodAnalysisRequest {
        environment: environment_with_prefix(&report, prefix),
        method: PhysicalMethodId {
            owner: definition,
            name: JvmBytes(b"run".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };

    let recovered = run_stdin(&request(
        &path,
        &analysis_limits(),
        recovery_operation(&analysis),
    ));
    let value = assert_ok(&recovered, "recover_method");

    // The library's own run of the same request over the same input, under the same shape of
    // budget the adapter spends: the CLI opens `input_path` and runs the request on one budget, so
    // this opens a second snapshot and runs the request on the budget that paid for that open.
    let mut library_budget = Budget::new(analysis_limits());
    let library_snapshot = engine
        .open(ArtifactInput::Path(path.clone()), &mut library_budget)
        .expect("open the fixture for the comparison");
    let direct = engine
        .recover_method_with_evidence(
            std::slice::from_ref(&library_snapshot),
            &analysis,
            &RecoveryEvidenceRequest::all(),
            &mut library_budget,
        )
        .expect("the same request through the library");
    assert_eq!(
        strip_elapsed(&value["result"]["report"]),
        strip_elapsed(&serde_json::to_value(direct.recovery()).expect("the report serializes")),
        "{name}: the CLI's recovery document is the library's own report"
    );
    assert_eq!(
        strip_elapsed(&value["result"]["analysis"]),
        strip_elapsed(&serde_json::to_value(direct.analysis()).expect("the run serializes")),
        "{name}: and the run beside it is the same run's report"
    );
    value
}

// ---------------------------------------------------------------------------------------------
// The operation
// ---------------------------------------------------------------------------------------------

/// The tree operation's answer is the library's own report, field by field.
#[test]
fn the_tree_operation_returns_the_library_report_unchanged() {
    let temp = TempDir::new();
    let class = class_bytes(b"com/demo/App");
    let path = temp.write("app.war", &war_with(WAR_CLASSES, &class));
    let request_limits = limits(1 << 20);

    let output = run_stdin(&request(
        &path,
        &request_limits,
        json!({"kind": "enumerate_artifact_tree"}),
    ));
    let value = assert_ok(&output, "enumerate_artifact_tree");
    let report = tree_report(&value);

    let engine = Engine::new();
    let mut budget = Budget::new(request_limits);
    let snapshot = engine
        .open(ArtifactInput::Path(path), &mut budget)
        .expect("open the fixture directly");
    let direct = engine
        .enumerate_artifact_tree(&snapshot, &mut budget)
        .expect("the same enumeration through the library");

    assert_eq!(
        strip_elapsed(&value["result"]["report"]),
        strip_elapsed(&serde_json::to_value(&direct).expect("the tree report serializes")),
        "the adapter adds, drops and rewrites nothing"
    );
    assert_eq!(report.view.snapshot, direct.view.snapshot);
    assert!(
        report.containers.len() > 1,
        "the fixture really is a tree: {:?}",
        report.containers
    );
}

/// Ordinary enumeration still stops at the top-level container; the explicit tree operation
/// descends.
///
/// The same WAR goes through both operations: `enumerate` reports the root's own entries — the
/// nested library is a `candidate_not_scanned` record and nothing inside it is read — while
/// `enumerate_artifact_tree` publishes the nested container and the class inside it. Neither
/// operation declares a load position: that stays the caller's declaration.
#[test]
fn ordinary_enumeration_still_does_not_recurse_and_the_tree_operation_does() {
    let temp = TempDir::new();
    let class = class_bytes(b"com/demo/App");
    let war = war_with(WAR_CLASSES, &class);
    let path = temp.write("app.war", &war);
    let request_limits = limits(1 << 20);

    let ordinary = assert_ok(
        &run_stdin(&request(
            &path,
            &request_limits,
            json!({"kind": "enumerate"}),
        )),
        "enumerate",
    );
    let report: jarde::EnumerationReport =
        serde_json::from_value(ordinary["result"]["report"].clone())
            .expect("the ordinary enumeration is the library's report");
    assert_eq!(
        report
            .entries
            .iter()
            .filter(|entry| !entry.id.origin.steps.is_empty())
            .count(),
        0,
        "ordinary enumeration reports no nested entry"
    );
    let nested = report
        .entries
        .iter()
        .find(|entry| entry.id.raw_name.0 == b"WEB-INF/lib/L.jar")
        .expect("the nested library is one top-level record");
    assert_eq!(
        nested.nested_archive,
        jarde::NestedArchiveState::CandidateNotScanned,
        "the candidate is marked, not followed"
    );

    let tree = tree_report(&assert_ok(
        &run_stdin(&request(
            &path,
            &request_limits,
            json!({"kind": "enumerate_artifact_tree"}),
        )),
        "enumerate_artifact_tree",
    ));
    let library = container_of(&tree, b"WEB-INF/lib/L.jar");
    assert!(
        tree.containers
            .iter()
            .any(|container| container.origin == library
                && container
                    .entries
                    .iter()
                    .any(|entry| entry.id.raw_name.0 == b"com/demo/Lib.class")),
        "the tree operation publishes the nested container and its class"
    );
}

/// A bounded and a broken tree keep the library's own prefix, and the CLI reports exactly it.
///
/// The budget-limited run stops the same way the library's does — `Partial` with the dimension
/// that refused, the containers it published, and the diagnostics that explain the stop — and the
/// broken nested child is a diagnostic on the child's own provenance in both. Cancellation is not
/// reachable through this adapter (it injects no token, and the operation takes no cancellation
/// field), so nothing here claims a `Cancelled` parity the CLI cannot express; the library's own
/// cancellation semantics are covered by the P1 tests.
#[test]
fn a_partial_tree_is_the_libraries_own_prefix() {
    let temp = TempDir::new();
    let class = class_bytes(b"com/demo/App");
    let path = temp.write("app.war", &war_with(WAR_CLASSES, &class));

    let tight = Limits {
        archive_entries: 3,
        ..limits(1 << 20)
    };
    let value = assert_ok(
        &run_stdin(&request(
            &path,
            &tight,
            json!({"kind": "enumerate_artifact_tree"}),
        )),
        "enumerate_artifact_tree",
    );
    let report = tree_report(&value);
    assert!(
        matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::ArchiveEntries
                },
                ..
            }
        ),
        "the CLI publishes the library's own stop: {:?}",
        report.execution
    );
    assert!(
        !report.coverage.artifact_structural.skipped.is_empty() || !report.diagnostics.is_empty(),
        "a stopped tree keeps the range it did not read: {report:?}"
    );

    let engine = Engine::new();
    let mut budget = Budget::new(tight);
    let snapshot = engine
        .open(ArtifactInput::Path(path.clone()), &mut budget)
        .expect("open the fixture directly");
    let direct = engine
        .enumerate_artifact_tree(&snapshot, &mut budget)
        .expect("the same enumeration through the library");
    assert_eq!(
        strip_elapsed(&value["result"]["report"]),
        strip_elapsed(&serde_json::to_value(&direct).expect("the tree report serializes")),
        "a partial answer is the library's own document too"
    );

    // A broken nested child: the report keeps the reliable prefix and names the child.
    let broken = zip(&[
        Stored {
            name: b"WEB-INF/classes/com/demo/App.class",
            data: &class,
        },
        Stored {
            name: b"WEB-INF/lib/Broken.jar",
            data: b"not an archive",
        },
    ]);
    let broken_path = temp.write("broken.war", &broken);
    let value = assert_ok(
        &run_stdin(&request(
            &broken_path,
            &limits(1 << 20),
            json!({"kind": "enumerate_artifact_tree"}),
        )),
        "enumerate_artifact_tree",
    );
    let report = tree_report(&value);
    assert!(
        matches!(report.execution, ExecutionReport::Partial { .. }),
        "a broken child is a partial tree, not an empty one: {:?}",
        report.execution
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.provenance.is_some()),
        "the diagnostic names the child it came from: {:?}",
        report.diagnostics
    );
    assert!(
        report
            .containers
            .iter()
            .any(|container| container.origin.steps.is_empty()),
        "the root container stays published"
    );
}

/// The operation declares no root of its own and echoes the snapshot it opened.
///
/// `enumerate_artifact_tree` carries no field that could name a prefix, a root, a loader order or
/// a classpath — its request object is the operation tag alone, like ordinary `enumerate` — and a
/// request that *does* carry a declaration of its own is answered with exactly the same report:
/// the adapter reads no declaration here, it only opens the path and hands the snapshot to the
/// library entry. What the answer echoes is the snapshot the adapter really opened, under the
/// tree scope of the content it addressed.
#[test]
fn the_operation_declares_no_roots_and_echoes_the_opened_snapshot() {
    let temp = TempDir::new();
    let class = class_bytes(b"com/demo/App");
    let path = temp.write("app.war", &war_with(WAR_CLASSES, &class));

    let plain = assert_ok(
        &run_stdin(&request(
            &path,
            &limits(1 << 20),
            json!({"kind": "enumerate_artifact_tree"}),
        )),
        "enumerate_artifact_tree",
    );
    let declared = assert_ok(
        &run_stdin(&request(
            &path,
            &limits(1 << 20),
            json!({
                "kind": "enumerate_artifact_tree",
                "roots": [{"kind": "container", "prefix": []}],
                "environment": {"domains": []},
            }),
        )),
        "enumerate_artifact_tree",
    );
    assert_eq!(
        strip_elapsed(&declared["result"]),
        strip_elapsed(&plain["result"]),
        "a declaration of the caller's own changes no answer: this operation reads none"
    );

    let report = tree_report(&plain);
    let engine = Engine::new();
    let mut budget = Budget::new(limits(1 << 20));
    let snapshot = engine
        .open(ArtifactInput::Path(path), &mut budget)
        .expect("open the fixture directly");
    assert_eq!(
        report.view.snapshot,
        *snapshot.id(),
        "the view names the snapshot the CLI opened, not one it rewrote"
    );
    assert_eq!(
        report.view.scope,
        PhysicalScope::ArtifactTree {
            root_container: ContainerId("root".into()),
        }
    );
}

// ---------------------------------------------------------------------------------------------
// 2.2 — the end-to-end loop
// ---------------------------------------------------------------------------------------------

/// Enumerate a WAR through the CLI, declare the position it names, and recover the method.
///
/// The identity is the CLI's: the report publishes the container origin and the entry, and the
/// request names exactly those. The prefix is the *caller's* declaration — the report publishes
/// layout evidence, and the contract is that a caller declares the position it wants rather than
/// having one inferred. The recovered text's source map still names the real physical path, and the
/// whole answer is compared with the library's own run of the same request and the same input.
#[test]
fn cli_enumeration_then_recovery_binds_a_war_class_and_names_its_physical_path() {
    let value = cli_enumeration_then_recovery("app.war", WAR_CLASSES);

    let report = &value["result"]["report"];
    let text = report["text"]
        .as_str()
        .expect("the report carries its text");
    assert!(!text.is_empty(), "the loop recovered a method: {report}");
    let segments = report["source_map"]["segments"]
        .as_array()
        .expect("the report carries its segment table");
    let mut physical = prefix_of_origin_paths(segments).into_iter();
    let named = physical
        .find(|name| name == &String::from_utf8_lossy(b"WEB-INF/classes/com/demo/App.class"));
    assert!(
        named.is_some(),
        "the source map names the physical path the class was read from"
    );
}

/// The same loop over a controlled `BOOT-INF/classes/` sample: the mechanism is a byte prefix,
/// and a prefix shaped like a Boot layout is no different from a WAR one.
///
/// This sample says nothing about Spring Boot: it is a prefix fixture, and the support boundary
/// stays where the support matrix states it — no `classpath.idx`, no `BOOT-INF/lib` ordering, no
/// launcher, no claim that a Boot executable's load order is resolved.
#[test]
fn the_same_mechanism_binds_a_boot_inf_classes_sample() {
    let value = cli_enumeration_then_recovery("app.boot", BOOT_CLASSES);
    let report = &value["result"]["report"];
    assert!(
        !report["text"]
            .as_str()
            .expect("the report carries text")
            .is_empty(),
        "the controlled Boot-shaped prefix sample recovered a method: {report}"
    );
    let segments = report["source_map"]["segments"]
        .as_array()
        .expect("the report carries its segment table");
    assert!(
        prefix_of_origin_paths(segments)
            .iter()
            .any(|name| name == &String::from_utf8_lossy(b"BOOT-INF/classes/com/demo/App.class")),
        "the source map keeps the Boot-shaped physical path"
    );
}

/// Every physical path the anchors of one segment table name, as the raw entry names they are.
fn prefix_of_origin_paths(segments: &[Value]) -> Vec<String> {
    let mut names = Vec::new();
    let anchors = segments.iter().flat_map(|segment| {
        let origin = &segment["origin"];
        std::iter::once(&origin["primary"]).chain(
            origin["derived"]
                .as_array()
                .expect("a segment states the anchors it presents")
                .iter(),
        )
    });
    for anchor in anchors {
        let Some(owner) = anchor["method"]["owner"].as_object() else {
            continue;
        };
        let Some(location) = owner["location"]["entry"]["raw_name"].as_array() else {
            continue;
        };
        let bytes: Vec<u8> = location
            .iter()
            .map(|octet| {
                u8::try_from(octet.as_u64().expect("an octet is a number"))
                    .expect("an octet is a byte")
            })
            .collect();
        names.push(String::from_utf8_lossy(&bytes).into_owned());
    }
    names
}
