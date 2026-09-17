use clap::Parser;
use jarde::{
    ArtifactInput, Budget, ClassTarget, CountedBudgetDimension, Engine, EngineBytecodeReport,
    EngineHeaderReport, EnumerationReport, Error, InspectionMode, JvmBytes, Limits, MethodSelector,
    PhysicalEntry, UsageSnapshot,
};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_RESPONSE_LENGTH_ITERATIONS: usize = 20;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Read one JSON request from FILE; omit or use '-' for standard input.
    #[arg(long, value_name = "FILE")]
    request: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    input_path: PathBuf,
    limits: RequestLimits,
    operation: Operation,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestLimits {
    input_bytes: u64,
    archive_entries: u64,
    entry_bytes: u64,
    read_bytes: u64,
    class_bytes: u64,
    attribute_bytes: u64,
    code_bytes: u64,
    result_items: u64,
    output_bytes: u64,
    elapsed_millis: u64,
}

impl From<RequestLimits> for Limits {
    fn from(value: RequestLimits) -> Self {
        Self {
            input_bytes: value.input_bytes,
            archive_entries: value.archive_entries,
            entry_bytes: value.entry_bytes,
            read_bytes: value.read_bytes,
            class_bytes: value.class_bytes,
            attribute_bytes: value.attribute_bytes,
            code_bytes: value.code_bytes,
            result_items: value.result_items,
            output_bytes: value.output_bytes,
            elapsed_millis: value.elapsed_millis,
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Operation {
    Enumerate,
    InspectHeader {
        target: Target,
        mode: InspectionMode,
    },
    InspectMethodBytecode {
        target: Target,
        selector: Selector,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Target {
    Root,
    Entry { entry: PhysicalEntry },
}

impl Target {
    fn as_core(&self) -> ClassTarget<'_> {
        match self {
            Self::Root => ClassTarget::Root,
            Self::Entry { entry } => ClassTarget::Entry(entry),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Selector {
    name: Vec<u8>,
    descriptor: Vec<u8>,
}

impl From<Selector> for MethodSelector {
    fn from(value: Selector) -> Self {
        Self {
            name: JvmBytes(value.name),
            descriptor: JvmBytes(value.descriptor),
        }
    }
}

#[derive(Serialize)]
struct SuccessResponse {
    status: &'static str,
    result: OperationResult,
    transport: SuccessTransport,
}

#[derive(Serialize)]
struct SuccessTransport {
    response_bytes: u64,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum OperationResult {
    Enumeration { report: EnumerationReport },
    Header { report: EngineHeaderReport },
    Bytecode { report: EngineBytecodeReport },
}

#[derive(Serialize)]
struct ErrorResponse<'a> {
    status: &'static str,
    error: &'a Error,
    usage: &'a UsageSnapshot,
}

#[derive(Debug)]
enum RequestReadError {
    Io(io::Error),
    TooLarge,
}

#[derive(Debug)]
enum SuccessWriteError {
    Adapter(Error),
    Stdout,
}

#[derive(Default)]
struct CountingWriter {
    bytes: u64,
}

impl Write for CountingWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let amount = u64::try_from(buffer.len())
            .map_err(|_| io::Error::other("JSON response length does not fit u64"))?;
        self.bytes = self
            .bytes
            .checked_add(amount)
            .ok_or_else(|| io::Error::other("JSON response length overflow"))?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let request_bytes = match read_request(cli.request.as_deref()) {
        Ok(bytes) => bytes,
        Err(error) => {
            let error = match error {
                RequestReadError::Io(error) => Error::Io {
                    operation: "cli_read_request".into(),
                    message: error.to_string(),
                },
                RequestReadError::TooLarge => Error::invalid_input(
                    "cli_request_too_large",
                    "request JSON exceeds the 1 MiB control-plane limit",
                ),
            };
            return fail(error, UsageSnapshot::default());
        }
    };
    let request: Request = match serde_json::from_slice(&request_bytes) {
        Ok(request) => request,
        Err(error) => {
            return fail(
                Error::invalid_input("cli_request_json", error.to_string()),
                UsageSnapshot::default(),
            );
        }
    };

    let Request {
        input_path,
        limits,
        operation,
    } = request;
    let mut budget = Budget::new(limits.into());
    let result = match execute(input_path, operation, &mut budget) {
        Ok(result) => result,
        Err(error) => return fail(error, budget.usage()),
    };
    let response = SuccessResponse {
        status: "ok",
        result,
        transport: SuccessTransport { response_bytes: 0 },
    };
    match write_success(response, &mut budget) {
        Ok(()) => ExitCode::SUCCESS,
        Err(SuccessWriteError::Adapter(error)) => fail(error, budget.usage()),
        Err(SuccessWriteError::Stdout) => {
            report_stdout_failure();
            ExitCode::FAILURE
        }
    }
}

fn read_request(path: Option<&Path>) -> Result<Vec<u8>, RequestReadError> {
    match path {
        Some(path) if path.as_os_str() != OsStr::new("-") => {
            let file = File::open(path).map_err(RequestReadError::Io)?;
            read_bounded(file)
        }
        _ => read_bounded(io::stdin().lock()),
    }
}

fn read_bounded(reader: impl Read) -> Result<Vec<u8>, RequestReadError> {
    let limit = u64::try_from(MAX_REQUEST_BYTES)
        .expect("the fixed request limit fits u64")
        .checked_add(1)
        .expect("the fixed request limit plus one fits u64");
    let mut bytes = Vec::new();
    reader
        .take(limit)
        .read_to_end(&mut bytes)
        .map_err(RequestReadError::Io)?;
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err(RequestReadError::TooLarge);
    }
    Ok(bytes)
}

fn execute(
    input_path: PathBuf,
    operation: Operation,
    budget: &mut Budget,
) -> Result<OperationResult, Error> {
    let engine = Engine::new();
    let snapshot = engine.open(ArtifactInput::Path(input_path), budget)?;
    match operation {
        Operation::Enumerate => engine
            .enumerate(&snapshot, budget)
            .map(|report| OperationResult::Enumeration { report }),
        Operation::InspectHeader { target, mode } => engine
            .inspect_header(&snapshot, target.as_core(), budget, mode)
            .map(|report| OperationResult::Header { report }),
        Operation::InspectMethodBytecode { target, selector } => engine
            .inspect_method_bytecode(&snapshot, target.as_core(), selector.into(), budget)
            .map(|report| OperationResult::Bytecode { report }),
    }
}

fn write_success(
    mut response: SuccessResponse,
    budget: &mut Budget,
) -> Result<(), SuccessWriteError> {
    let mut total = None;
    for _ in 0..MAX_RESPONSE_LENGTH_ITERATIONS {
        let measured = count_response_bytes(&response).map_err(SuccessWriteError::Adapter)?;
        if measured == response.transport.response_bytes {
            total = Some(measured);
            break;
        }
        response.transport.response_bytes = measured;
    }
    let total = total.ok_or_else(|| SuccessWriteError::Adapter(response_json_error()))?;
    let capacity =
        usize::try_from(total).map_err(|_| SuccessWriteError::Adapter(response_json_error()))?;

    budget
        .check(CountedBudgetDimension::OutputBytes, total)
        .map_err(SuccessWriteError::Adapter)?;
    let mut output = Vec::with_capacity(capacity);
    serde_json::to_writer(&mut output, &response)
        .map_err(|_| SuccessWriteError::Adapter(response_json_error()))?;
    output.push(b'\n');
    let actual = u64::try_from(output.len())
        .map_err(|_| SuccessWriteError::Adapter(response_json_error()))?;
    if actual != total || output.len() != capacity {
        return Err(SuccessWriteError::Adapter(response_json_error()));
    }
    budget
        .charge(CountedBudgetDimension::OutputBytes, total)
        .map_err(SuccessWriteError::Adapter)?;
    write_stdout(&output).map_err(|_| SuccessWriteError::Stdout)
}

fn count_response_bytes(response: &SuccessResponse) -> Result<u64, Error> {
    let mut counter = CountingWriter::default();
    serde_json::to_writer(&mut counter, response).map_err(|_| response_json_error())?;
    counter.bytes.checked_add(1).ok_or_else(response_json_error)
}

fn response_json_error() -> Error {
    Error::invalid_input("cli_response_json", "failed to serialize the JSON response")
}

fn fail(error: Error, usage: UsageSnapshot) -> ExitCode {
    if write_error(&error, &usage).is_err() {
        report_stdout_failure();
    }
    ExitCode::FAILURE
}

fn write_error(error: &Error, usage: &UsageSnapshot) -> io::Result<()> {
    // As with core termination metadata, this small envelope explains why the request failed;
    // it is not charged to the request budget and never mutates the reliable core report.
    let response = ErrorResponse {
        status: "error",
        error,
        usage,
    };
    let output = match serialize_error_response(&response) {
        Ok(output) => output,
        Err(()) => {
            let fallback = response_json_error();
            let response = ErrorResponse {
                status: "error",
                error: &fallback,
                usage,
            };
            serialize_error_response(&response)
                .map_err(|()| io::Error::other("failed to serialize the JSON error response"))?
        }
    };
    write_stdout(&output)
}

fn serialize_error_response(response: &ErrorResponse<'_>) -> Result<Vec<u8>, ()> {
    let mut output = Vec::new();
    serde_json::to_writer(&mut output, response).map_err(|_| ())?;
    output.push(b'\n');
    Ok(output)
}

fn write_stdout(document: &[u8]) -> io::Result<()> {
    io::stdout().lock().write_all(document)
}

fn report_stdout_failure() {
    eprintln!("jarde-cli: failed to write JSON response to stdout");
}
