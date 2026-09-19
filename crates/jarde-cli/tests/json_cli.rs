use jarde::{
    AnalysisStage, ArtifactInput, ArtifactSnapshot, Budget, BudgetDimension, ClassTarget,
    CompileStatus, DelegationPolicy, Engine, EngineBytecodeReport, EngineHeaderReport,
    EnumerationReport, ExecutionReport, InspectionMode, JvmBytes, LayoutMode, Limits, LoadDomain,
    LoadRoot, LoaderId, MethodAnalysisReport, MethodAnalysisRequest, MethodBodyState,
    MethodSelector, ModuleMode, MultiReleasePolicy, PhysicalClassLocation, PhysicalDefinitionId,
    PhysicalMethodId, PhysicalScope, PhysicalVariant, PhysicalView, Quality, Representation,
    ResolutionEnvironment, RuntimeProfile, RuntimeUncertainty, RuntimeView, SemanticValidation,
    StageState, SyntaxStatus, TerminationReason, UsageSnapshot, VerificationStatus,
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
            "jarde-cli-test-{}-{nonce}-{}",
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

fn u16b(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn u32b(output: &mut Vec<u8>, value: u32) {
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

fn class(code: &[u8]) -> Vec<u8> {
    class_with_handlers(code, &[])
}

fn class_with_handlers(code: &[u8], handlers: &[(u16, u16, u16, u16)]) -> Vec<u8> {
    class_with_locals(code, handlers, 1, 0)
}

/// The same fixture class with the `Code` attribute's own `max_stack`/`max_locals` stated by the
/// caller. The default fixture above declares one stack slot and no locals, which is the smallest
/// body the pipeline accepts; a case that wants a body with a local (a branch on a stored value, a
/// loop counter) has to declare the slots it uses, exactly as a compiler would.
fn class_with_locals(
    code: &[u8],
    handlers: &[(u16, u16, u16, u16)],
    max_stack: u16,
    max_locals: u16,
) -> Vec<u8> {
    let mut output = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0);
    u16b(&mut output, 52);
    u16b(&mut output, 8);
    utf8(&mut output, b"Test");
    output.push(7);
    u16b(&mut output, 1);
    utf8(&mut output, b"java/lang/Object");
    output.push(7);
    u16b(&mut output, 3);
    utf8(&mut output, b"run");
    utf8(&mut output, b"()V");
    utf8(&mut output, b"Code");
    u16b(&mut output, 0x21);
    u16b(&mut output, 2);
    u16b(&mut output, 4);
    u16b(&mut output, 0);
    u16b(&mut output, 0);
    u16b(&mut output, 1);
    u16b(&mut output, 9);
    u16b(&mut output, 5);
    u16b(&mut output, 6);
    u16b(&mut output, 1);

    let mut code_attribute = Vec::new();
    u16b(&mut code_attribute, max_stack);
    u16b(&mut code_attribute, max_locals);
    u32b(
        &mut code_attribute,
        u32::try_from(code.len()).expect("fixture code length fits u32"),
    );
    code_attribute.extend_from_slice(code);
    u16b(
        &mut code_attribute,
        u16::try_from(handlers.len()).expect("fixture handler count fits u16"),
    );
    for &(start, end, handler, catch_type) in handlers {
        u16b(&mut code_attribute, start);
        u16b(&mut code_attribute, end);
        u16b(&mut code_attribute, handler);
        u16b(&mut code_attribute, catch_type);
    }
    u16b(&mut code_attribute, 0);
    u16b(&mut output, 7);
    u32b(
        &mut output,
        u32::try_from(code_attribute.len()).expect("fixture attribute length fits u32"),
    );
    output.extend_from_slice(&code_attribute);
    u16b(&mut output, 0);
    output
}

fn empty_zip() -> Vec<u8> {
    let mut bytes = vec![0x50, 0x4b, 0x05, 0x06];
    bytes.resize(22, 0);
    bytes
}

fn selector() -> MethodSelector {
    MethodSelector {
        name: JvmBytes(b"run".to_vec()),
        descriptor: JvmBytes(b"()V".to_vec()),
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

fn run_stdin(request: &[u8], explicit_dash: bool, without_java: bool) -> Output {
    let mut command = Command::new(BIN);
    if explicit_dash {
        command.args(["--request", "-"]);
    }
    if without_java {
        command.env("PATH", "");
        command.env_remove("JAVA_HOME");
    }
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("spawn jarde-cli");
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(request)
        .expect("write request JSON");
    child.wait_with_output().expect("wait for jarde-cli")
}

fn run_request_file(request_path: &Path) -> Output {
    Command::new(BIN)
        .arg("--request")
        .arg(request_path)
        .output()
        .expect("run jarde-cli with request file")
}

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
    serde_json::from_slice(&output.stdout).expect("stdout is valid JSON")
}

fn assert_ok(output: &Output, kind: &str) -> Value {
    assert!(
        output.status.success(),
        "stderr: {}\nstdout: {}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stderr.is_empty());
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

fn normalized_execution(report: &ExecutionReport) -> Value {
    let mut value = serde_json::to_value(report).expect("serialize execution report");
    value["usage"]["elapsed_millis"] = json!(0);
    value
}

fn normalized_usage(usage: &UsageSnapshot) -> UsageSnapshot {
    let mut usage = usage.clone();
    usage.elapsed_millis = 0;
    usage
}

// --- the method-analysis operation -----------------------------------------------------

/// The request limits the P2 method-analysis operation really spends.
///
/// The P1 operations of this file are answered from the byte dimensions, so their
/// `limits(u64::MAX)` leaves every P2 counter at its fail-closed zero — which is exactly what a
/// request that means to run the pipeline must not do, so the six dimensions `analyze_method`
/// charges are funded here on top of the P1 ones.
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

/// One caller domain rooted at the fixture's own snapshot, and nothing else: the simplest
/// environment the library's validator accepts without a problem.
fn environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Snapshot {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
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
        providers: Vec::new(),
    }
}

/// One method-analysis request for the fixture's `run()V`, with the physical identity the
/// library's own reading of the artifact publishes.
///
/// The adapter under test binds the content by opening `input_path` and handing *that* snapshot
/// to the library, so the request has to name the snapshot and the class-bytes digest of those
/// very bytes. Both are asked of the library here (`open` and `inspect_header`, the public reads
/// that derive them) rather than recomputed from the fixture, which is what keeps the two answers
/// comparable and keeps this test free of a hashing dependency the CLI package does not declare.
fn method_request(path: &Path, stages: Vec<AnalysisStage>) -> MethodAnalysisRequest {
    member_request(path, b"run", b"()V", stages)
}

/// The same request for one named member of the fixture.
fn member_request(
    path: &Path,
    name: &[u8],
    descriptor: &[u8],
    stages: Vec<AnalysisStage>,
) -> MethodAnalysisRequest {
    let mut budget = Budget::new(analysis_limits());
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::Path(path.to_path_buf()), &mut budget)
        .expect("open the fixture for its identity");
    let header = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("read the fixture's own class bytes identity");
    MethodAnalysisRequest {
        environment: environment(&snapshot),
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: header.source.class_bytes,
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        },
        stages,
    }
}

/// The `analyze_method` operation of one library request: the payload is the library's own
/// serialization plus the operation tag, so this adapter is not a second schema of it.
fn method_operation(request: &MethodAnalysisRequest) -> Value {
    let mut operation =
        serde_json::to_value(request).expect("a method-analysis request serializes");
    operation
        .as_object_mut()
        .expect("the request serializes as an object")
        .insert("kind".to_string(), json!("analyze_method"));
    operation
}

/// The `recover_method` operation of one library request: the same three fields the analysis
/// operation carries (the payload is the library's own serialization plus the operation tag, so
/// this adapter is not a second schema of it), under the recovery kind.
fn recover_operation(request: &MethodAnalysisRequest) -> Value {
    let mut operation = method_operation(request);
    operation
        .as_object_mut()
        .expect("the request serializes as an object")
        .insert("kind".to_string(), json!("recover_method"));
    operation
}

/// The body both recovery cases present: `if (local1 != 0) { return; } else { return; }` written the
/// way a compiler writes it — `iconst_0; istore_1; iload_1; ifeq +4; return; return`, where the
/// branch transfers past the first `return` to the second. It is presented with
/// `max_stack 1, max_locals 2`, which is what the slots it uses need.
const RECOVERY_BODY: &[u8] = &[
    0x03, // 0: iconst_0
    0x3c, // 1: istore_1
    0x1b, // 2: iload_1
    0x99, 0x00, 0x04, // 3: ifeq 7
    0xb1, // 6: return
    0xb1, // 7: return
];

/// The recovery cases' fixture class: [`RECOVERY_BODY`] with the slots it uses declared.
fn recovery_class() -> Vec<u8> {
    class_with_locals(RECOVERY_BODY, &[], 1, 2)
}

/// `Test` with the field a compiler generated an accessor for, the accessor itself, and the member
/// that calls it (P3 3.2's shape: the call site names a member of *this* class, whose body the
/// operation has to read to present the call as the field access it forwards):
///
/// ```text
/// f:I                 private
/// access$100(LTest;)I public static synthetic: aload_0; getfield Test.f:I; ireturn
/// method()I           public: iconst_0; istore_1; aload_0;
///                             invokestatic Test.access$100(LTest;)I; ireturn
/// ```
fn accessor_class() -> Vec<u8> {
    let mut output = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0);
    u16b(&mut output, 52);
    u16b(&mut output, 16);
    utf8(&mut output, b"Test"); // 1
    output.push(7);
    u16b(&mut output, 1); // 2: class Test
    utf8(&mut output, b"java/lang/Object"); // 3
    output.push(7);
    u16b(&mut output, 3); // 4: class Object
    utf8(&mut output, b"f"); // 5
    utf8(&mut output, b"I"); // 6
    output.push(12);
    u16b(&mut output, 5);
    u16b(&mut output, 6); // 7: NameAndType f:I
    output.push(9);
    u16b(&mut output, 2);
    u16b(&mut output, 7); // 8: Fieldref Test.f:I
    utf8(&mut output, b"access$100"); // 9
    utf8(&mut output, b"(LTest;)I"); // 10
    output.push(12);
    u16b(&mut output, 9);
    u16b(&mut output, 10); // 11: NameAndType access$100(LTest;)I
    output.push(10);
    u16b(&mut output, 2);
    u16b(&mut output, 11); // 12: Methodref Test.access$100
    utf8(&mut output, b"method"); // 13
    utf8(&mut output, b"()I"); // 14
    utf8(&mut output, b"Code"); // 15

    u16b(&mut output, 0x21);
    u16b(&mut output, 2);
    u16b(&mut output, 4);
    u16b(&mut output, 0); // no interfaces
    u16b(&mut output, 1); // one field
    u16b(&mut output, 0x0002);
    u16b(&mut output, 5);
    u16b(&mut output, 6);
    u16b(&mut output, 0);
    u16b(&mut output, 2); // two members
    /// One member of the fixture: its body, its flags, its name and descriptor indexes, and the two
    /// slots its `Code` attribute declares.
    struct FixtureMember {
        code: &'static [u8],
        flags: u16,
        name: u16,
        descriptor: u16,
        max_stack: u16,
        max_locals: u16,
    }
    let members = [
        // access$100(LTest;)I: aload_0; getfield Test.f:I; ireturn
        FixtureMember {
            code: &[0x2a, 0xb4, 0x00, 0x08, 0xac],
            flags: 0x1008,
            name: 9,
            descriptor: 10,
            max_stack: 1,
            max_locals: 1,
        },
        // method()I: iconst_0; istore_1; aload_0; invokestatic access$100; ireturn
        FixtureMember {
            code: &[0x03, 0x3c, 0x2a, 0xb8, 0x00, 0x0c, 0xac],
            flags: 0x0001,
            name: 13,
            descriptor: 14,
            max_stack: 1,
            max_locals: 2,
        },
    ];
    for member in members {
        u16b(&mut output, member.flags);
        u16b(&mut output, member.name);
        u16b(&mut output, member.descriptor);
        u16b(&mut output, 1); // one attribute
        u16b(&mut output, 15); // "Code"
        let mut attribute = Vec::new();
        u16b(&mut attribute, member.max_stack);
        u16b(&mut attribute, member.max_locals);
        u32b(
            &mut attribute,
            u32::try_from(member.code.len()).expect("fixture code length fits u32"),
        );
        attribute.extend_from_slice(member.code);
        u16b(&mut attribute, 0); // no exception handlers
        u16b(&mut attribute, 0); // no nested attributes
        u32b(
            &mut output,
            u32::try_from(attribute.len()).expect("fixture attribute length fits u32"),
        );
        output.extend_from_slice(&attribute);
    }
    u16b(&mut output, 0);
    output
}

/// One report as JSON with every `elapsed_millis` removed: the one measurement two entry paths
/// cannot share, and the only field the 5.1 acceptance lets them differ in.
fn strip_elapsed(report: &MethodAnalysisReport) -> Value {
    strip_elapsed_document(
        &serde_json::to_value(report).expect("a method-analysis report serializes"),
    )
}

/// The same removal, over a document that is already JSON.
///
/// The recovery report arrives as the adapter's own document (the CLI writes the library's type),
/// and comparing two documents field by field does not require the CLI's to be deserialized first —
/// which is just as well, because the recovery report's code fields are borrowed strings the wire
/// document spells as text.
fn strip_elapsed_document(document: &Value) -> Value {
    fn walk(value: &mut Value) {
        match value {
            Value::Object(fields) => {
                fields.remove("elapsed_millis");
                for child in fields.values_mut() {
                    walk(child);
                }
            }
            Value::Array(items) => {
                for item in items {
                    walk(item);
                }
            }
            _ => {}
        }
    }
    let mut value = document.clone();
    walk(&mut value);
    value
}

fn diagnostic_codes(report: &MethodAnalysisReport) -> Vec<&str> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

#[test]
fn header_matches_direct_engine_without_java_runtime() {
    let temp = TempDir::new();
    let bytes = class(&[0xb1]);
    let class_path = temp.write("Test.class", &bytes);
    let request_limits = limits(u64::MAX);
    let request = request(
        &class_path,
        &request_limits,
        json!({
            "kind": "inspect_header",
            "target": {"kind": "root"},
            "mode": "strict"
        }),
    );

    let output = run_stdin(&request, false, true);
    let value = assert_ok(&output, "header");
    let cli_report: EngineHeaderReport =
        serde_json::from_value(value["result"]["report"].clone()).expect("header report");

    let engine = Engine::new();
    let mut budget = Budget::new(request_limits);
    let snapshot = engine
        .open(ArtifactInput::Path(class_path), &mut budget)
        .expect("open direct snapshot");
    let direct_report = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("inspect direct header");

    assert_eq!(cli_report.source, direct_report.source);
    assert_eq!(cli_report.inspection, direct_report.inspection);
    assert_eq!(cli_report.coverage, direct_report.coverage);
    assert_eq!(
        cli_report.inspection.verification,
        VerificationStatus::NotPerformed
    );
    assert_eq!(
        normalized_execution(&cli_report.execution),
        normalized_execution(&direct_report.execution)
    );
    let ExecutionReport::Complete { usage: cli_usage } = &cli_report.execution else {
        panic!("header execution must be complete");
    };
    let ExecutionReport::Complete {
        usage: direct_usage,
    } = &direct_report.execution
    else {
        panic!("direct header execution must be complete");
    };
    assert_eq!(normalized_usage(cli_usage), normalized_usage(direct_usage));
}

#[test]
fn invalid_bytecode_remains_an_ok_partial_core_report() {
    let temp = TempDir::new();
    let class_path = temp.write("Partial.class", &class(&[0x00, 0xcb]));
    let request_limits = limits(u64::MAX);
    let request = request(
        &class_path,
        &request_limits,
        json!({
            "kind": "inspect_method_bytecode",
            "target": {"kind": "root"},
            "selector": {"name": [114, 117, 110], "descriptor": [40, 41, 86]}
        }),
    );

    let output = run_stdin(&request, true, false);
    let value = assert_ok(&output, "bytecode");
    let cli_report: EngineBytecodeReport =
        serde_json::from_value(value["result"]["report"].clone()).expect("bytecode report");
    assert!(matches!(
        cli_report.inspection.execution,
        ExecutionReport::Partial { .. }
    ));
    assert!(!cli_report.inspection.diagnostics.is_empty());
    assert_eq!(
        value["result"]["report"]["inspection"]["stopped_at"]["phase"],
        "instructions"
    );
    assert_eq!(
        value["result"]["report"]["inspection"]["stopped_at"]["bci"],
        1
    );
    assert!(
        value["result"]["report"]["inspection"]["stopped_at"]
            .get("ordinal")
            .is_none()
    );

    let engine = Engine::new();
    let mut budget = Budget::new(request_limits);
    let snapshot = engine
        .open(ArtifactInput::Path(class_path), &mut budget)
        .expect("open direct snapshot");
    let direct_report = engine
        .inspect_method_bytecode(&snapshot, ClassTarget::Root, selector(), &mut budget)
        .expect("inspect direct bytecode");

    assert_eq!(
        normalized_execution(&cli_report.inspection.execution),
        normalized_execution(&direct_report.inspection.execution)
    );
    assert_eq!(
        cli_report.inspection.diagnostics,
        direct_report.inspection.diagnostics
    );
    assert_eq!(
        cli_report.inspection.stopped_at,
        direct_report.inspection.stopped_at
    );
    assert_eq!(cli_report.coverage, direct_report.coverage);
}

#[test]
fn handler_budget_stop_serializes_ordinal_and_record_offset() {
    let temp = TempDir::new();
    let class_path = temp.write(
        "Handlers.class",
        &class_with_handlers(&[0xb1], &[(0, 1, 0, 0), (0, 1, 0, 2)]),
    );
    let mut request_limits = limits(u64::MAX);
    request_limits.result_items = 2;
    let request = request(
        &class_path,
        &request_limits,
        json!({
            "kind": "inspect_method_bytecode",
            "target": {"kind": "root"},
            "selector": {"name": [114, 117, 110], "descriptor": [40, 41, 86]}
        }),
    );

    let output = run_stdin(&request, false, false);
    let value = assert_ok(&output, "bytecode");
    let report: EngineBytecodeReport =
        serde_json::from_value(value["result"]["report"].clone()).expect("bytecode report");
    assert!(matches!(
        report.inspection.execution,
        ExecutionReport::Partial { .. }
    ));
    let stop = &value["result"]["report"]["inspection"]["stopped_at"];
    assert_eq!(stop["phase"], "exception_handlers");
    assert_eq!(stop["ordinal"], 1);
    assert!(stop.get("bci").is_none());
    let expected_offset = report
        .inspection
        .code_span
        .start
        .checked_add(report.inspection.code_span.length)
        .and_then(|offset| offset.checked_add(2))
        .and_then(|offset| offset.checked_add(8))
        .unwrap();
    assert_eq!(stop["class_offset"], expected_offset);
}

#[test]
fn empty_zip_enumeration_uses_the_open_snapshot() {
    let temp = TempDir::new();
    let zip_path = temp.write("empty.zip", &empty_zip());
    let request_path = temp.write(
        "request.json",
        &request(&zip_path, &limits(u64::MAX), json!({"kind": "enumerate"})),
    );

    let output = run_request_file(&request_path);
    let value = assert_ok(&output, "enumeration");
    let report: EnumerationReport =
        serde_json::from_value(value["result"]["report"].clone()).expect("enumeration report");
    assert!(report.entries.is_empty());
    assert_eq!(report.snapshot.0.len(), 64);
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));

    let engine = Engine::new();
    let mut budget = Budget::new(limits(u64::MAX));
    let snapshot = engine
        .open(ArtifactInput::Path(zip_path), &mut budget)
        .expect("open direct ZIP snapshot");
    assert_eq!(report.snapshot, *snapshot.id());
}

#[test]
fn malformed_and_unknown_request_json_are_protocol_errors() {
    let malformed = run_stdin(b"{", false, false);
    let malformed = assert_error_code(&malformed, "cli_request_json");
    assert_eq!(malformed["error"]["kind"], "invalid_input");
    assert_eq!(
        malformed["usage"],
        serde_json::to_value(UsageSnapshot::default()).unwrap()
    );

    let temp = TempDir::new();
    let class_path = temp.write("Test.class", &class(&[0xb1]));
    let unknown = serde_json::to_vec(&json!({
        "input_path": class_path,
        "limits": limits(u64::MAX),
        "operation": {"kind": "enumerate"},
        "unexpected": true
    }))
    .unwrap();
    let unknown = run_stdin(&unknown, false, false);
    let unknown = assert_error_code(&unknown, "cli_request_json");
    assert!(
        unknown["error"]["message"]
            .as_str()
            .expect("JSON error message")
            .contains("unknown field")
    );
}

#[test]
fn output_budget_exact_boundary_and_failure_are_atomic() {
    let temp = TempDir::new();
    let bytes = class(&[0xb1]);
    let class_path = temp.write("Test.class", &bytes);
    let operation = json!({
        "kind": "inspect_header",
        "target": {"kind": "root"},
        "mode": "strict"
    });

    let baseline = run_stdin(
        &request(&class_path, &limits(u64::MAX), operation.clone()),
        false,
        false,
    );
    let baseline = assert_ok(&baseline, "header");
    let core_output = baseline["result"]["report"]["execution"]["usage"]["output_bytes"]
        .as_u64()
        .expect("core output usage is u64");
    let response_bytes = baseline["transport"]["response_bytes"]
        .as_u64()
        .expect("transport response bytes is u64");
    let exact_limit = core_output.checked_add(response_bytes).unwrap();

    let mut exact_limits = limits(u64::MAX);
    exact_limits.output_bytes = exact_limit;
    let exact = run_stdin(
        &request(&class_path, &exact_limits, operation.clone()),
        false,
        false,
    );
    let exact = assert_ok(&exact, "header");
    assert_eq!(exact["transport"]["response_bytes"], response_bytes);
    assert_eq!(
        exact["result"]["report"]["execution"]["usage"]["output_bytes"],
        core_output
    );

    let mut short_limits = exact_limits;
    short_limits.output_bytes = exact_limit.checked_sub(1).unwrap();
    let short = run_stdin(
        &request(&class_path, &short_limits, operation),
        false,
        false,
    );
    let value = response(&short);
    assert_eq!(short.status.code(), Some(1));
    assert_eq!(value["status"], "error");
    assert_eq!(value["error"]["kind"], "budget_exceeded");
    assert_eq!(value["error"]["dimension"], "output_bytes");
    assert_eq!(value["usage"]["output_bytes"], core_output);
    assert!(
        !short
            .stdout
            .windows(b"\"status\":\"ok\"".len())
            .any(|window| { window == b"\"status\":\"ok\"" })
    );
}

#[test]
fn entry_target_rejects_ordinal_shortcuts_and_incomplete_metadata() {
    let temp = TempDir::new();
    let class_path = temp.write("Test.class", &class(&[0xb1]));
    for target in [
        json!({"kind": "entry", "ordinal": 0}),
        json!({"kind": "entry", "entry": {"id": {"ordinal": 0}}}),
    ] {
        let request = request(
            &class_path,
            &limits(u64::MAX),
            json!({
                "kind": "inspect_header",
                "target": target,
                "mode": "strict"
            }),
        );
        let output = run_stdin(&request, false, false);
        assert_error_code(&output, "cli_request_json");
    }
}

#[test]
fn request_larger_than_one_mib_is_rejected_before_json_parsing() {
    let request = vec![b' '; MAX_REQUEST_BYTES + 1];
    let output = run_stdin(&request, false, false);
    let value = assert_error_code(&output, "cli_request_too_large");
    assert_eq!(value["error"]["kind"], "invalid_input");
    assert_eq!(
        value["usage"],
        serde_json::to_value(UsageSnapshot::default()).unwrap()
    );
}

#[test]
fn method_analysis_matches_direct_engine_field_by_field() {
    let temp = TempDir::new();
    let class_path = temp.write("Test.class", &class(&[0xb1]));
    let request_limits = analysis_limits();
    let analysis = method_request(&class_path, vec![AnalysisStage::Ssa]);

    let output = run_stdin(
        &request(&class_path, &request_limits, method_operation(&analysis)),
        false,
        false,
    );
    let value = assert_ok(&output, "method_analysis");
    let cli_report: MethodAnalysisReport =
        serde_json::from_value(value["result"]["report"].clone())
            .expect("the CLI report deserializes as the library report type");

    let engine = Engine::new();
    let mut budget = Budget::new(request_limits);
    let snapshot = engine
        .open(ArtifactInput::Path(class_path), &mut budget)
        .expect("open the fixture directly");
    let direct_report = engine
        .analyze_method(std::slice::from_ref(&snapshot), &analysis, &mut budget)
        .expect("analyze the fixture directly");

    // The whole document, not a chosen subset of it: a field this adapter dropped, reordered or
    // rewrote would show up here even when no named plane below looks at that field. Only the
    // wall clock is removed, because the two runs are two runs.
    assert_eq!(strip_elapsed(&cli_report), strip_elapsed(&direct_report));
    assert_eq!(cli_report.method, direct_report.method);
    assert_eq!(cli_report.reads, direct_report.reads);

    // The planes the 5.1 acceptance names, stated on the wire document of a request that ran the
    // whole pipeline this build declares: bytecode, conservative, not Java, not compiled, local
    // invariants as this run's own evidence, and the verifier still not performed.
    assert_eq!(cli_report.representation, Representation::Bytecode);
    assert_eq!(cli_report.quality, Quality::Conservative);
    assert_eq!(cli_report.syntax_status, SyntaxStatus::NotJava);
    assert_eq!(cli_report.compile_status, CompileStatus::NotAttempted);
    assert_eq!(
        cli_report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
    assert_eq!(cli_report.verification, VerificationStatus::NotPerformed);
    assert_eq!(cli_report.body, MethodBodyState::Present);
    assert_eq!(cli_report.requested_stages, vec![AnalysisStage::Ssa]);
    assert_eq!(
        cli_report
            .stages
            .iter()
            .map(|stage| stage.state.clone())
            .collect::<Vec<_>>(),
        vec![StageState::Completed; 6]
    );
    assert!(matches!(
        cli_report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(value["result"]["report"]["execution"]["status"], "complete");
}

#[test]
fn method_protocol_errors_are_transport_errors() {
    let temp = TempDir::new();
    let class_path = temp.write("Test.class", &class(&[0xb1]));
    let request_limits = analysis_limits();
    let well_formed = method_request(&class_path, vec![AnalysisStage::Ssa]);

    // A stage name the schema does not know: the operation's own shape, refused like every other
    // operation's protocol error, with the same code and the same empty usage.
    let mut unknown_stage = method_operation(&well_formed);
    unknown_stage["stages"] = json!(["ssa", "ssa_typo"]);
    let output = run_stdin(
        &request(&class_path, &request_limits, unknown_stage),
        false,
        false,
    );
    let value = assert_error_code(&output, "cli_request_json");
    assert_eq!(value["error"]["kind"], "invalid_input");
    assert_eq!(
        value["usage"],
        serde_json::to_value(UsageSnapshot::default()).unwrap()
    );

    // A required field left out, and a field this operation does not have: both are the same
    // protocol error, because the operation is a closed schema.
    let mut missing = method_operation(&well_formed);
    missing
        .as_object_mut()
        .expect("the operation is an object")
        .remove("stages");
    let output = run_stdin(
        &request(&class_path, &request_limits, missing),
        false,
        false,
    );
    assert_error_code(&output, "cli_request_json");

    let mut unexpected = method_operation(&well_formed);
    unexpected
        .as_object_mut()
        .expect("the operation is an object")
        .insert(
            "selector".to_string(),
            json!({"name": [114, 117, 110], "descriptor": [40, 41, 86]}),
        );
    let output = run_stdin(
        &request(&class_path, &request_limits, unexpected),
        false,
        false,
    );
    assert_error_code(&output, "cli_request_json");

    // An input error of the *library* is a transport error too, not a report: the request names a
    // snapshot this invocation did not open, and the adapter answers the library's own code
    // instead of silently re-pointing the environment at its input.
    let mut foreign_snapshot = method_operation(&well_formed);
    foreign_snapshot["environment"]["runtime"]["physical"]["snapshot"] = json!("0".repeat(64));
    let output = run_stdin(
        &request(&class_path, &request_limits, foreign_snapshot),
        false,
        false,
    );
    let value = assert_error_code(&output, "resolution_snapshot_mismatch");
    assert_eq!(value["error"]["kind"], "invalid_input");
    assert!(
        value["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("snapshot")),
        "the message names what the request and the input disagree about: {}",
        value["error"]["message"]
    );
}

#[test]
fn a_method_analysis_stop_is_the_payload_of_a_successful_response() {
    let temp = TempDir::new();
    let class_path = temp.write("Test.class", &class(&[0xb1]));
    let analysis = method_request(&class_path, vec![AnalysisStage::Ssa]);

    // A budget that cannot fund the pipeline's work: the charge the run cannot make ends it, and
    // that stop belongs to the *report* — an `ok` response carrying a `Partial` execution and the
    // budget layer's own diagnostic, never an error of the control plane.
    let mut stopped = analysis_limits();
    stopped.analysis_steps = 0;
    let output = run_stdin(
        &request(&class_path, &stopped, method_operation(&analysis)),
        false,
        false,
    );
    let value = assert_ok(&output, "method_analysis");
    let report: MethodAnalysisReport = serde_json::from_value(value["result"]["report"].clone())
        .expect("the CLI report deserializes as the library report type");
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps
            },
            ..
        }
    ));
    assert_eq!(value["result"]["report"]["execution"]["status"], "partial");
    assert_eq!(
        diagnostic_codes(&report),
        vec!["budget_exceeded_analysis_steps"]
    );
    assert_eq!(
        report.quality,
        Quality::Fallback,
        "no canonical artifact was produced by a run that stopped inside the raw graph"
    );
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::Unproven,
        "a stopped run carries no semantic evidence"
    );
    // The stop is a phase's own state inside the report: the phase the refused charge ended is
    // `Partial`, and every phase behind it never ran. The body the run had already read stays a
    // fact of the report, because the stop did not unread it.
    let stopped_at = report
        .stages
        .iter()
        .position(|stage| stage.state != StageState::Completed)
        .expect("a budget stop leaves a phase that did not complete");
    assert_eq!(
        report.stages[stopped_at].state,
        StageState::Partial,
        "the stopped phase is partial: {:?}",
        report.stages
    );
    assert!(
        report.stages[stopped_at + 1..]
            .iter()
            .all(|stage| stage.state == StageState::NotPerformed),
        "no phase behind the stopped one ran: {:?}",
        report.stages
    );
    assert_eq!(report.body, MethodBodyState::Present);
}

#[test]
fn recovery_matches_the_library_entry_field_by_field() {
    // The library/CLI obligation of P3 1.3: one request, two entry paths, one document. The body is
    // an `if`/`else` so the presentation really has a structure to write, and the comparison is the
    // whole document of both halves the operation answers with — a field this adapter dropped,
    // reordered or rewrote would show up here even when no named plane below looks at that field.
    // Only the wall clock is removed, because the two runs are two runs.
    let temp = TempDir::new();
    let class_path = temp.write("Test.class", &recovery_class());
    let request_limits = analysis_limits();
    let analysis = method_request(&class_path, vec![AnalysisStage::Ssa]);

    let output = run_stdin(
        &request(&class_path, &request_limits, recover_operation(&analysis)),
        false,
        false,
    );
    let value = assert_ok(&output, "recover_method");

    let engine = Engine::new();
    let mut budget = Budget::new(request_limits);
    let snapshot = engine
        .open(ArtifactInput::Path(class_path), &mut budget)
        .expect("open the fixture directly");
    let direct = engine
        .recover_method(std::slice::from_ref(&snapshot), &analysis, &mut budget)
        .expect("the same request through the library");

    assert_eq!(
        strip_elapsed_document(&value["result"]["report"]),
        strip_elapsed_document(
            &serde_json::to_value(direct.recovery()).expect("the recovery report serializes")
        ),
        "the adapter's document is the library's own report, field by field"
    );
    assert_eq!(
        strip_elapsed_document(&value["result"]["analysis"]),
        strip_elapsed_document(
            &serde_json::to_value(direct.analysis()).expect("the analysis report serializes")
        ),
        "and the run it answers beside it is that same run's report"
    );

    // The planes the P3 acceptance names, stated on the wire document: Java text, structured,
    // nothing checked or compiled, no semantic evidence of this run's own, never verified — and the
    // profile the request declared, with the rules the text came from.
    let report = &value["result"]["report"];
    assert_eq!(report["representation"], "java");
    assert_eq!(report["quality"], "structured");
    assert_eq!(report["syntax_status"], "unchecked");
    assert_eq!(report["compile_status"], "not_attempted");
    assert_eq!(report["semantic_validation"], "unproven");
    assert_eq!(report["verification"], "not_performed");
    assert_eq!(report["outcome"], "produced");
    assert_eq!(report["method"], "run()V");
    assert_eq!(report["profile"]["java_release"], 8);
    let rules: Vec<&str> = report["rules"]
        .as_array()
        .expect("the report names the rules it used")
        .iter()
        .map(|rule| rule["rule"].as_str().expect("a rule name"))
        .collect();
    assert!(
        rules.contains(&"if"),
        "the branch was presented by the `if` rule: {rules:?}"
    );
    assert!(
        report["text"].as_str().expect("text").contains("if ("),
        "{}",
        report["text"]
    );
    assert!(
        report["regions"]
            .as_array()
            .expect("regions")
            .iter()
            .any(|region| region["structured"] == true && region["rule"]["rule"] == "if"),
        "each region states which rule produced it: {}",
        report["regions"]
    );

    // A16 on the wire, in the same answer: one header read and one body attempted for a request
    // that names one member, and the recovery added no read of its own.
    assert_eq!(
        value["result"]["analysis"]["execution"]["usage"]["class_headers"],
        1
    );
    assert_eq!(
        value["result"]["analysis"]["execution"]["usage"]["method_bodies"],
        1
    );

    // P3 3.2's own read, on the same wire: this body names no call site the accessor rule would read
    // a callee for, so the operation answers `callees: null` — nothing was read and nothing was
    // charged beyond the run — which is what the library's own answer says too.
    assert_eq!(value["result"]["callees"], Value::Null);
    assert!(
        direct.callees().is_none(),
        "the library read no member for a body that names none"
    );
}

#[test]
fn a_presented_accessor_and_the_members_it_reads_cross_the_wire() {
    // P3 3.2's success half through the CLI: the request names a body that calls a member of its own
    // class, the operation reads that member (and only it), presents the call as the field access it
    // forwards, and states on the wire which class and which members it read, why, and what it
    // charged. The library's own answer is the reference, as in the case above.
    let temp = TempDir::new();
    let class_path = temp.write("Test.class", &accessor_class());
    let analysis = member_request(&class_path, b"method", b"()I", AnalysisStage::ALL.to_vec());

    let output = run_stdin(
        &request(
            &class_path,
            &analysis_limits(),
            recover_operation(&analysis),
        ),
        false,
        false,
    );
    let value = assert_ok(&output, "recover_method");

    let engine = Engine::new();
    let mut budget = Budget::new(analysis_limits());
    let snapshot = engine
        .open(ArtifactInput::Path(class_path), &mut budget)
        .expect("open the fixture directly");
    let direct = engine
        .recover_method(std::slice::from_ref(&snapshot), &analysis, &mut budget)
        .expect("the same request through the library");
    let callees = direct.callees().expect("the call site names a member");

    assert_eq!(
        strip_elapsed_document(&value["result"]["callees"]),
        strip_elapsed_document(&serde_json::to_value(callees).expect("the callee read serializes")),
        "the adapter's document is the library's own read, field by field"
    );

    // The text is the direct field expression the source had, and the call the compiler made is not
    // in it.
    let text = value["result"]["report"]["text"].as_str().expect("text");
    assert!(text.contains("return arg0.f;"), "{text}");
    assert!(!text.contains("access$100("), "{text}");

    // The evidence the read states: one class, one member of it — with the reason the read happened
    // under — and the one member body it attempted beyond the run's own.
    let read = &value["result"]["callees"];
    assert_eq!(read["class"], "Test");
    assert_eq!(read["members"].as_array().expect("members").len(), 1);
    assert_eq!(
        read["members"][0]["identity"]["name"],
        json!([97, 99, 99, 101, 115, 115, 36, 49, 48, 48])
    );
    assert!(read["members"][0]["body"].is_object(), "{read}");
    assert_eq!(read["refusals"].as_array().expect("refusals").len(), 0);
    assert_eq!(
        read["reads"][0]["reason"],
        json!("callee_member_body"),
        "{read}"
    );
    assert_eq!(read["usage"]["class_headers"], 2, "{read}");
    assert_eq!(read["usage"]["method_bodies"], 2, "{read}");

    // And the presented call is recorded as presented, with the member it read.
    let accessors = value["result"]["report"]["accessors"]
        .as_array()
        .expect("accessors");
    assert_eq!(accessors.len(), 1);
    assert_eq!(accessors[0]["presented"], json!(true));
    assert_eq!(
        accessors[0]["callee"]["name"],
        json!([97, 99, 99, 101, 115, 115, 36, 49, 48, 48])
    );
}

#[test]
fn a_recovery_request_that_asks_for_no_ssa_stops_inside_a_successful_response() {
    // A schedule is the request's own statement, and a payload without the SSA table is answered
    // with a *stop in the payload* — status `ok`, kind `recover_method`, no text, no segments, an
    // execution plane that says the run was partial — rather than with a transport error. That
    // distinction is the adapter's contract: only request-level problems (unreadable JSON, a
    // refused environment, a budget the response itself cannot pay) leave the success envelope.
    let temp = TempDir::new();
    let class_path = temp.write("Test.class", &recovery_class());
    let analysis = method_request(&class_path, vec![AnalysisStage::Frame]);

    let output = run_stdin(
        &request(
            &class_path,
            &analysis_limits(),
            recover_operation(&analysis),
        ),
        false,
        false,
    );
    let value = assert_ok(&output, "recover_method");
    let report = &value["result"]["report"];
    assert_eq!(
        report["outcome"]["stopped"]["ir_table_missing"]["table"],
        "ssa"
    );
    assert_eq!(report["text"], "");
    assert_eq!(report["source_map"]["segments"], json!([]));
    assert_eq!(report["regions"], json!([]));
    assert_eq!(report["representation"], "bytecode");
    assert_eq!(report["quality"], "fallback");
    assert_eq!(report["syntax_status"], "not_java");
    assert_eq!(
        report["diagnostics"][0]["code"], "jre_ir_table_missing",
        "the stop is stated with its code and its message: {}",
        report["diagnostics"]
    );
    // And the analysis half says the truth about its own run: the schedule it was given completed
    // (frame and its prerequisites ran). It is the *presentation* that cannot be written from a
    // payload without SSA, which is why the stop is in the recovery report and not in the run.
    assert_eq!(
        value["result"]["analysis"]["execution"]["status"],
        "complete"
    );
    assert_eq!(
        value["result"]["analysis"]["execution"]["usage"]["method_bodies"],
        1
    );
}
