mod task;

use clap::Parser;
use jarde::{
    AnalysisStage, ArtifactInput, ArtifactTreeReport, Budget, CalleeReadReport, ClassTarget,
    ConsumerSchema, CountedBudgetDimension, Engine, EngineBytecodeReport, EngineHeaderReport,
    EnumerationReport, Error, InspectionMode, JvmBytes, Limits, MethodAnalysisReport,
    MethodAnalysisRequest, MethodSelector, PhysicalEntry, PhysicalMethodId, PhysicalScope,
    PhysicalView, QueryCursor, QueryRelation, QueryReport, QueryRequest, QueryTarget,
    RecoveryReport, ResolutionEnvironment, UsageSnapshot,
};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::slice;

const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_RESPONSE_LENGTH_ITERATIONS: usize = 20;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Read one JSON request from FILE; omit or use '-' for standard input.
    #[arg(long, value_name = "FILE")]
    request: Option<PathBuf>,
    /// One task-oriented command: friendly parameters over the library's own entries.
    #[command(subcommand)]
    command: Option<task::Command>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    input_path: PathBuf,
    limits: RequestLimits,
    operation: Operation,
}

/// Request limits, one required field per [`Limits`] dimension.
///
/// Deliberately hand-written instead of derived from `Limits`: `serde` can deserialize a
/// struct with `#[serde(default)]`, but that would silently substitute zero (or any other
/// fallback) for a limit the caller forgot, and this schema has no omit-means-default case.
/// Every dimension stays required, including the six P2 dimensions the method-analysis
/// operation really spends, so a request that means to run P2 work cannot accidentally ask
/// for "no limit" or "zero limit" by leaving a field out.
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
    class_headers: u64,
    method_bodies: u64,
    ir_items: u64,
    ir_edges: u64,
    analysis_steps: u64,
    normalization_clones: u64,
    nested_depth: u64,
    dependency_depth: u64,
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
            class_headers: value.class_headers,
            method_bodies: value.method_bodies,
            ir_items: value.ir_items,
            ir_edges: value.ir_edges,
            analysis_steps: value.analysis_steps,
            normalization_clones: value.normalization_clones,
            nested_depth: value.nested_depth,
            dependency_depth: value.dependency_depth,
            elapsed_millis: value.elapsed_millis,
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Operation {
    Enumerate,
    /// One explicit artifact-tree enumeration: every container, entry, layout node, coverage and
    /// diagnostic the library's own entry reports, under this request's limits.
    ///
    /// The adapter adds nothing here — it opens `input_path` and hands *that* snapshot to the
    /// library's existing `enumerate_artifact_tree` entry, so the answer is the library's own
    /// report and the snapshot it echoes is the one it really opened. What the operation does
    /// **not** do is as much of the contract as what it does: it declares no root, invents no
    /// delegation order and derives no classpath from the layout evidence it publishes. A caller
    /// uses the identities it returns to declare its own load positions, which is why the
    /// operation has no request field at all.
    EnumerateArtifactTree,
    InspectHeader {
        target: Target,
        mode: InspectionMode,
    },
    InspectMethodBytecode {
        target: Target,
        selector: Selector,
    },
    /// One P1 query request.
    ///
    /// The request carries the same fields as the library [`QueryRequest`] except for the
    /// snapshot identity: this adapter opens `input_path` itself, so the caller declares the
    /// physical scope it wants scanned and the effective `PhysicalView`, including the snapshot
    /// it resolved to, is echoed back in the report. A continuation replays the `cursor` value
    /// from an earlier response verbatim, which is how a cursor that belongs to another input
    /// stays detectable instead of being rewritten.
    Query {
        relation: QueryRelation,
        target: QueryTarget,
        physical: PhysicalScope,
        consumers: ConsumerSchema,
        /// `0` means "no page limit"; the scan still obeys `limits`.
        max_items: u64,
        /// Boxed so the request enum stays small: a cursor binds the whole query identity,
        /// and this adapter holds exactly one request at a time.
        #[serde(default)]
        cursor: Option<Box<QueryCursor>>,
    },
    /// One P2 method-analysis request, carrying the library [`MethodAnalysisRequest`] as it
    /// stands: the environment, the method's physical identity and the requested stages.
    ///
    /// The adapter owns exactly one thing here, and it is not a field of the payload: it opens
    /// `input_path` and hands *that* artifact to the library as the request's one content. The
    /// snapshot identities inside the request — the runtime view's own and the `snapshot` roots
    /// of the domains and providers — are therefore not rewritten, and a request that names
    /// another artifact is answered by the library's own content check
    /// (`resolution_snapshot_mismatch`) rather than silently re-pointed at this input. Every
    /// field stays where the library put it, which is what makes the answer the library's own
    /// report instead of a re-description of it.
    AnalyzeMethod {
        /// Boxed for the same reason the query cursor is: the environment is the largest field
        /// any operation carries, and this adapter holds exactly one request at a time. `serde`
        /// treats the box as the value it holds, so the JSON operation is the library's own
        /// environment shape and nothing here is a second schema of it.
        environment: Box<ResolutionEnvironment>,
        method: PhysicalMethodId,
        stages: Vec<AnalysisStage>,
    },
    /// One P3 recovery request (1.3): the same three fields the method-analysis operation
    /// carries, because it is the *same run* — the library performs the analysis once and hands
    /// its own payload to the presentation.
    ///
    /// Why there is no separate recovery-profile field: the profile the gate reads is the
    /// environment's own runtime profile, and a second field naming a release would be a second
    /// source of truth for one fact (and a way for a request to present a Java 8 artifact under a
    /// profile its environment never declared). Why `stages` is required and not defaulted: a
    /// schedule is a request's own statement, and a recovery request whose stages omit the tables
    /// the presentation reads is answered with a *stop* inside the response (`jre_ir_table_missing`)
    /// rather than with an adapter-chosen schedule.
    RecoverMethod {
        environment: Box<ResolutionEnvironment>,
        method: PhysicalMethodId,
        stages: Vec<AnalysisStage>,
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
    Enumeration {
        report: EnumerationReport,
    },
    /// The library's own artifact-tree report, unchanged.
    ArtifactTree {
        report: ArtifactTreeReport,
    },
    Header {
        report: EngineHeaderReport,
    },
    Bytecode {
        report: EngineBytecodeReport,
    },
    Query {
        report: QueryReport,
    },
    MethodAnalysis {
        report: MethodAnalysisReport,
    },
    /// The two reports of one recovery request, and both are of the same run: `analysis` is the
    /// method-analysis report of the run whose tables were presented, `report` is the presentation
    /// itself. A caller that wants to know what the request read and which stages completed reads
    /// `analysis`; a caller that wants the Java text, its segment table and the planes that describe
    /// it reads `report`.
    ///
    /// `callees` is what the request read **beyond** that run, when the body it presented named a
    /// call site the `accessor@1` rule would decide from (P3 3.2): the one class the members were
    /// read from, the members it declares among the candidates, the candidates it refused with the
    /// reason, the read's own reason and what it charged. It is `null` for every request whose body
    /// named no such call site, because nothing was read and nothing was charged.
    RecoverMethod {
        /// Both halves are boxed for the size reason the environments above are boxed for: this is
        /// the largest result any operation carries, and the adapter holds exactly one request at a
        /// time. `serde` treats a box as the value it holds, so the wire document is the library's
        /// own and nothing here is a second schema of it.
        analysis: Box<MethodAnalysisReport>,
        report: Box<RecoveryReport>,
        /// Boxed for the same reason, and `None` exactly when no callee was read.
        callees: Option<Box<CalleeReadReport>>,
    },
}

/// The one failure document both request surfaces write.
///
/// A protocol error (the legacy `--request` path) writes it to standard output, because that path's
/// standard output is the response stream; a task command writes the same document to standard error
/// and classifies the failure with its exit status, because a task command's standard output carries
/// a *report* and a failure is not one. The shape is stated once for both.
#[derive(Serialize)]
pub(crate) struct ErrorResponse<'a> {
    pub(crate) status: &'static str,
    pub(crate) error: &'a Error,
    pub(crate) usage: &'a UsageSnapshot,
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
    // Two request surfaces, one invocation: `--request` is the JSON control plane and a subcommand
    // is a task command. Absent subcommand keeps the legacy path exactly as it was — including its
    // stdin default — and naming both is a usage error rather than a silently elected one.
    if let Some(command) = cli.command {
        if cli.request.is_some() {
            eprintln!(
                "jarde-cli: `--request` and a task command are two different requests; name one of \
                 them"
            );
            return ExitCode::from(task::EXIT_USAGE);
        }
        return task::run(command);
    }
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
        Operation::EnumerateArtifactTree => engine
            .enumerate_artifact_tree(&snapshot, budget)
            .map(|report| OperationResult::ArtifactTree { report }),
        Operation::InspectHeader { target, mode } => engine
            .inspect_header(&snapshot, target.as_core(), budget, mode)
            .map(|report| OperationResult::Header { report }),
        Operation::InspectMethodBytecode { target, selector } => engine
            .inspect_method_bytecode(&snapshot, target.as_core(), selector.into(), budget)
            .map(|report| OperationResult::Bytecode { report }),
        Operation::Query {
            relation,
            target,
            physical,
            consumers,
            max_items,
            cursor,
        } => {
            let request = QueryRequest {
                relation,
                target,
                // The snapshot is the one this invocation opened; the caller only declares the
                // scope. Validation, relation dispatch, paging and coverage all stay in the
                // library, so the adapter never filters or re-scans a result.
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
                    scope: physical,
                },
                consumers,
                max_items,
                cursor: cursor.map(|cursor| *cursor),
            };
            engine
                .query(&snapshot, &request, budget)
                .map(|report| OperationResult::Query { report })
        }
        Operation::AnalyzeMethod {
            environment,
            method,
            stages,
        } => {
            // The one thing this adapter binds is the content, and it binds it by handing the
            // snapshot it opened to the library: validation, scheduling, every phase of the
            // pass table, the stop and the report are the library's, so the payload is the
            // library's own request and the answer is the library's own report.
            let request = MethodAnalysisRequest {
                environment: *environment,
                method,
                stages,
            };
            engine
                .analyze_method(slice::from_ref(&snapshot), &request, budget)
                .map(|report| OperationResult::MethodAnalysis { report })
        }
        Operation::RecoverMethod {
            environment,
            method,
            stages,
        } => {
            // The adapter binds the content once and calls **one** library entry: the entry runs the
            // analysis and presents the payload of that very run, so no second analysis happens
            // here, and no table is reconstructed from the analysis report.
            let request = MethodAnalysisRequest {
                environment: *environment,
                method,
                stages,
            };
            engine
                .recover_method(slice::from_ref(&snapshot), &request, budget)
                .map(|recovered| {
                    let (analysis, report, callees) = recovered.into_parts();
                    OperationResult::RecoverMethod {
                        analysis: Box::new(analysis),
                        report: Box::new(report),
                        callees: callees.map(Box::new),
                    }
                })
        }
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
