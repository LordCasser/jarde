use jarde::{
    ArtifactInput, Budget, ClassTarget, Engine, EngineBytecodeReport, EngineHeaderReport,
    EnumerationReport, ExecutionReport, InspectionMode, JvmBytes, Limits, MethodSelector,
    UsageSnapshot, VerificationStatus,
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
    u16b(&mut code_attribute, 1);
    u16b(&mut code_attribute, 0);
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
