//! `export`: the streaming adapter — one bulk operation, one JSONL file.
//!
//! The command is one library call and one destination. `Engine::recover_all` walks a physical scope,
//! prepares each class once and hands one record per class prepared, method result, class end and
//! located diagnostic to a sink; this module is that sink, and the file it writes is the whole
//! product. Nothing here renders a document, starts a second analysis or invents a second schema: a
//! record is the library's own event with the `kind` that names it, written as one line.
//!
//! Five rules the code below is built around, in the order a record meets them:
//!
//! * **a record is the library's own value.** [`Record`] is a tagged view of an event reference, so
//!   every field of a method record — its physical identity, its raw name and descriptor, its member
//!   ordinal, its class, the `RecoveryReport` with the text and the source map, the analysis planes —
//!   is the library's serialization of that value, not a copy of it. The `kind` tag is the one thing
//!   this adapter adds, and it adds it as a tag rather than as a field of its own.
//! * **the stream is the library's order.** The sink writes each record inside the callback that
//!   delivered it, so the frames of the file are the frames of the operation — header, class prepared,
//!   that class's methods, its class end, and the final summary last.
//! * **encoding is bounded by the request's own allowance.** A record is encoded into one reused
//!   buffer that never holds more than the stream's remaining `output_bytes` reading, and it is
//!   debited *before* it is written; a record that does not fit stops the run without being written at
//!   all, and the encoder states that record's real size by counting the bytes it cannot hold.
//! * **a record is delivered when the whole line is on the file.** Each line is handed over in one
//!   `write_all`, framing included, and only then is its length added to the delivered prefix. A write
//!   that fails part way leaves a partial line, which is not a record: the file is truncated back to
//!   the last confirmed boundary so that a consumer reading the file never meets half a line.
//! * **the `final` record is confirmed only after the file is flushed and closed.** A run that could
//!   not do that does not confirm it, whatever the library's own planes say, and the exit status is 2
//!   even though the `final` line may already be visible in the file.
//!
//! The exit statuses are the four classes design decision 7 fixes: 0 only for a confirmed `final` whose
//! aggregate is `Complete`, 2 for input, infrastructure and output failures, and 4 for a run that
//! stopped — an exhausted budget, a stopped method, a refused class, a damaged traversal, or a stream
//! whose `final` was never delivered.
//!
//! **The budget and the retention this command runs under are its own.** A whole scope is not one
//! view, so the counted dimensions and the wall clock are the command's own finite ceilings, and
//! the facts store ([`EXPORT_FACTS`]) is attached here so a sweep parses each container's
//! directory once for the whole scope rather than once per class. The library invents neither
//! number; every effective value is published in the stream's `header` record. The library requires explicit numbers and this
//! adapter is the caller that states them: the task defaults bound *one request's view* — a 2 MiB
//! output, sixty-five thousand archive records, four million derived items — and a whole package is
//! not one view, so this command replaces every counted dimension with its own finite ceiling and the
//! wall clock with its own bound ([`EXPORT_COUNTED_TOTAL`], [`EXPORT_ELAPSED_MILLIS`]) before the
//! caller's own `--budget` declarations are applied on top. Nothing here is unbounded and nothing here
//! is implicit: the header record publishes the effective configuration as the library reported it,
//! so what a run really ran under is read from the run.

use crate::task::{EXIT_COMPLETE, EXIT_INCOMPLETE, EXIT_USAGE, EnvironmentArgs, Failure, Opened};
use clap::{ArgAction, Args, ValueEnum};
use jarde::artifact::budget_dimension_code;
use jarde::{
    BudgetDimension, BulkDiagnosticEvent, BulkFinalEvent, BulkHeaderEvent, BulkRecoveryReport,
    BulkRecoveryRequest, BulkStop, ClassEndEvent, ClassPreparedEvent, CountedBudgetDimension,
    Error, ExecutionReport, FactsCache, FactsCapacity, MethodResultEvent, RecoverySink,
    SinkControl,
};
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::slice;

/// One `export` request: the artifact, the scope, the environment, the destination and the workers.
#[derive(Debug, Args)]
pub(crate) struct Export {
    /// The artifact to open: an archive or a standalone class file.
    #[arg(long, value_name = "PATH")]
    input: PathBuf,
    /// The physical scope to export, as a `PhysicalScope` document (or `@FILE`); defaults to
    /// `{"kind":"snapshot_all"}`, the whole opened snapshot.
    ///
    /// The scope decides which container subtrees are walked, so a nested library's classes are part
    /// of the run only when the scope (or the environment's own roots) names them.
    #[arg(long, value_name = "JSON")]
    scope: Option<String>,
    /// One budget override, `dimension=limit`, repeatable — the counted dimensions and the wall
    /// clock, spelled as every task command spells them.
    ///
    /// This command runs under its own finite defaults — a bulk ceiling for every counted dimension
    /// and a whole-package wall clock — and a declaration here **replaces** the dimension it names,
    /// so `ir_items=…` is the way to state a tighter or a wider one. The effective configuration is
    /// published in the stream's `header`.
    ///
    /// `output_bytes` is also this stream's own allowance: the JSONL bytes this command writes are
    /// debited from the same reading the request declares, and a record that does not fit ends the run
    /// rather than being written anyway.
    #[arg(long = "budget", value_name = "DIMENSION=LIMIT", action = ArgAction::Append)]
    budget: Vec<String>,
    /// The file the JSONL stream is written to.
    ///
    /// Required. The file is created with `create_new`, so an existing path is refused — exit 2, with
    /// the file left exactly as it was — rather than truncated: a second run never overwrites a first
    /// run's records.
    #[arg(long, value_name = "FILE")]
    output: PathBuf,
    /// The one format this command writes: `jsonl`.
    ///
    /// Every line is one JSON object carrying the library's own record under a stable `kind`. The value
    /// is accepted explicitly and is also the default; the spellings the report commands write (`text`,
    /// `json`) are not part of this command's vocabulary.
    #[arg(long, value_enum, default_value = "jsonl")]
    format: Jsonl,
    /// How many class tasks run at once: `auto` (this machine's own count, at least one) or a positive
    /// decimal count.
    ///
    /// The value is the *requested* count. The operation reduces it to the number of class tasks its
    /// result window can hold, and the stream's `header` record publishes both numbers, so what a run
    /// really used is read from the run rather than inferred from this parameter. `0` and every value
    /// that is not a count are usage errors: neither means "as many as there are".
    #[arg(long, value_name = "auto|N", default_value = "auto")]
    jobs: String,
    /// The environment the methods are read and analysed under, declared exactly as `recover` declares
    /// it.
    #[command(flatten)]
    environment: EnvironmentArgs,
}

/// The one line-oriented encoding this command writes.
///
/// A single-variant enum rather than a string: the flag has a value vocabulary, and a second
/// line-oriented format would be a variant here with its own framing beside the first rather than
/// falling out of a default. `text` and `json` are not in it — this command renders no document and
/// writes no text projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum Jsonl {
    /// One JSON object per line, each line ended by a newline.
    Jsonl,
}

impl Jsonl {
    /// The byte that ends one record of this format.
    fn framing(self) -> u8 {
        match self {
            Self::Jsonl => b'\n',
        }
    }
}

/// The finite ceiling every counted dimension of a whole-package export runs under.
///
/// A bulk export is one operation over a whole physical scope, so the task defaults — which bound
/// *one request's view*: `1 << 16` archive records, `1 << 22` derived items and worklist steps,
/// 16 MiB of output — are not the numbers a whole package needs. Measured on this machine against
/// the corpora this command is built for, one whole-package run needed:
///
/// | corpus | `ir_items` | `analysis_steps` |
/// | --- | --- | --- |
/// | `bcprov-jdk15on-152.jar` | 11,025,007 | 3,156,885 |
/// | `S2-009.war` (54 roots) | 32,966,465 | 8,646,330 |
///
/// Both are past the task defaults by an order of magnitude, and one bulk stream that did not raise
/// them stopped a real export a few dozen classes in — the defect this default set exists to remove.
///
/// `1 << 40` (about 1.1e12 units) is that ceiling: more than thirty thousand times the largest
/// measured requirement, so a real package's real work fits with room for a bigger one, and still a
/// number a run cannot reach by accident — every counted dimension stays bounded, so a run that did
/// reach it stops with its own dimension named rather than running on without a bound. A caller with
/// a different shape states its own numbers through `--budget`, which replaces this ceiling for the
/// dimension it names.
const EXPORT_COUNTED_TOTAL: u64 = 1 << 40;

/// The retention this command's one operation reads through: two independent bounds, both finite,
/// both published in the stream's `header` record as `limits.facts_capacity`.
///
/// A whole-scope export walks a container's **directory** once per class it holds unless that
/// container's facts are retained. Without a store the charge grows the way the artifact is wide:
/// `bcprov-jdk15on-152.jar` holds 2,569 directory records and 2,430 classes, so every class
/// re-parsed the whole directory and the default totals (`archive_entries` and `result_items`:
/// 65,536 each) were exhausted after a few dozen classes — a sweep that could not reach its own
/// scope. Measured with these bounds over the two corpora the change uses: bcprov retains 3.7 MB
/// for a whole 2,430-class, 15,003-method sweep against a single container product, and
/// `S2-009.war` retains 27 MB — 53 container products, 52 of them materialized nested libraries —
/// for a whole 7,200-class, 57,180-method sweep whose 53 directories were parsed once each. This
/// capacity covers both with room for a taller, wider tree: 16,384 answers is far more than the
/// container products such trees hold beside the class payloads reused across a run, and 128 MiB is
/// nearly five times what the larger corpus retains.
///
/// The bounds are this command's own effective default, not a number the engine invented: a request
/// states its facts capacity through the budget it already carries, and the header publishes the one
/// this constant states. Retention is also not a correctness knob — an answer that does not fit is
/// not retained, the run re-reads what it needs, and its records, their order and its exit status
/// stay exactly what they would be with room to spare.
const EXPORT_FACTS: FactsCapacity = FactsCapacity::new(1 << 14, 1 << 27);

/// The wall clock one whole-package export runs under, in milliseconds.
///
/// The task default (30 s) bounds an interactive view, and a whole package is not one. Measured on
/// this machine, the largest corpus of the benchmark set — `S2-009.war`, 12 MB, 54 containers,
/// 7,200 classes and 57,180 methods — needed the whole thirty minutes this constant first stated
/// just to reach 46,980 of its methods: a bound that stops the run before its scope ends is the same
/// defect as a counted default the work does not fit, and it is why this number is stated in whole
/// hours rather than in seconds. Two hours is that bound with room for a slower machine, a larger
/// package, or the re-reading this worktree still does per class (task 5.4's store is not attached
/// here): it is finite, a run that is wedged rather than working stops at it, and the stop names the
/// clock like any other dimension. `--jobs N` shortens the wall clock the work needs without touching
/// this ceiling, and a caller that wants a tighter deadline states one with `--budget elapsed_millis=…`.
const EXPORT_ELAPSED_MILLIS: u64 = 2 * 60 * 60 * 1000;

/// The `dimension=limit` declarations one `export` runs under, in the order they apply.
///
/// This command's own finite defaults come first — every counted dimension at
/// [`EXPORT_COUNTED_TOTAL`] and the clock at [`EXPORT_ELAPSED_MILLIS`] — and the caller's own
/// `--budget` declarations after them, so a caller's number always replaces what the command would
/// otherwise have run with. Both halves are spelled in the one vocabulary the `--budget` parameter
/// takes and parsed by the one place that parses it, so an unknown dimension or a zero limit is
/// refused by the library's own check wherever it came from, and the effective result is what the
/// stream's `header` publishes.
///
/// The counted dimensions are taken from the reader's own [`CountedBudgetDimension::ALL`] and each
/// name from the reader's own dimension code, so a counted dimension added there is covered by this
/// ceiling at the same time rather than being left at a task default nobody raised. The two
/// high-water depths are deliberately not declared: see `OVERRIDABLE_BUDGET_DIMENSIONS` in the
/// facade for why a deeper container or dependency walk is a scope declaration, not a budget.
fn budget_declarations(caller: &[String]) -> Vec<String> {
    let mut declared: Vec<String> = CountedBudgetDimension::ALL
        .into_iter()
        .map(|dimension| declaration(dimension.into(), EXPORT_COUNTED_TOTAL))
        .collect();
    declared.push(declaration(
        BudgetDimension::ElapsedMillis,
        EXPORT_ELAPSED_MILLIS,
    ));
    declared.extend(caller.iter().cloned());
    declared
}

/// One `dimension=limit` declaration, spelled with the reader's own name for the dimension.
fn declaration(dimension: BudgetDimension, limit: u64) -> String {
    format!("{}={limit}", budget_dimension_code(dimension))
}

/// Runs one `export` command and returns the process exit status.
///
/// The one thing this function does that the report commands do not is deliver: the records leave as
/// the library produces them, so the exit status is decided after the operation is over rather than
/// from a document that was rendered beside it.
pub(crate) fn run(args: Export) -> Result<ExitCode, Failure> {
    let Export {
        input,
        scope,
        budget,
        output,
        format,
        jobs,
        environment,
    } = args;
    // The parameters are read before anything is opened, as in every other command: a value this
    // adapter cannot read never becomes a read of anything.
    let declaration = environment.declaration()?;
    let workers = jobs_requested(&jobs)?;
    let framing = format.framing();
    // The effective budget: this command's own finite defaults, then the caller's declarations after
    // them. The one place that opens the request reads and checks all of them, so a dimension nobody
    // could fund is refused before the input is opened.
    let declared = budget_declarations(&budget);
    let mut opened = Opened::open(&input, scope.as_deref(), &declared)?;
    // The operation's own retention, attached to the budget this request already carries and before
    // the operation is called: one handle, shared with every class task and method budget the
    // operation builds from it, so a sweep consults one store for the whole of its scope — and the
    // totals above stay totals of the *work* rather than of re-parsing each container once per class.
    // The store is this command's — nothing below the CLI attaches one, and the ordinary report
    // commands pass through this constructor with no capacity at all.
    opened.budget = opened
        .budget
        .with_facts_cache(FactsCache::current(EXPORT_FACTS));
    // The destination is created after the artifact was opened: a run that cannot have its input
    // leaves no file behind, and a run that cannot have its file reads no further input. `create_new`
    // is what makes both halves of that statement true.
    let file = match create_output(&output) {
        Ok(file) => file,
        Err(error) => return Err(Failure::export_failed(error, opened.budget.usage())),
    };
    let request = BulkRecoveryRequest::for_scope(
        declaration.bind(&opened.snapshot, &opened.scope),
        workers,
        opened.budget.limits().clone(),
    );
    let mut stream = Stream::new(file, output, framing, stream_allowance(&opened.budget));
    // One process, one library operation: the stream is this call's sink, and there is no second run
    // behind it — not a serial retry, not a per-method process.
    let outcome = opened.engine.recover_all(
        slice::from_ref(&opened.snapshot),
        &request,
        &mut opened.budget,
        &mut stream,
    );
    let stopped = stream.finish();
    let report = match outcome {
        Ok(report) => report,
        // A refused request: the operation never started, so the class is the input's — 2 — beside
        // whatever opening and validating it cost.
        Err(error) => return Err(Failure::export_failed(error, opened.budget.usage())),
    };
    let status = exit_status(stopped.as_ref(), &report);
    if status == EXIT_COMPLETE {
        return Ok(ExitCode::from(EXIT_COMPLETE));
    }
    // The reason the failure document states is the adapter's own stop when there was one, and the
    // library's own account of the run when there was not.
    let error = match stopped.as_ref() {
        Some(stopped) => stopped.error().clone(),
        None => unfinished_error(&report),
    };
    let usage = report.usage.clone();
    Err(if status == EXIT_USAGE {
        Failure::export_failed(error, usage)
    } else {
        Failure::export_stopped(error, usage)
    })
}

/// The worker count `--jobs` asks for.
///
/// `auto` is this machine's own count — `std::thread::available_parallelism`, and 1 when that reading
/// fails — and it is the *requested* value: the reduction to what the result window can hold is the
/// operation's own, and the header record publishes both numbers. A literal count must be a positive
/// decimal number. `0` is not "as many as there are" and neither is a value that is not a number, so
/// both are usage errors rather than a silent fallback to one worker.
fn jobs_requested(value: &str) -> Result<usize, Error> {
    if value == "auto" {
        return Ok(std::thread::available_parallelism().map_or(1, |count| count.get()));
    }
    match value.parse::<usize>() {
        Ok(0) => Err(Error::invalid_input(
            "cli_jobs_zero",
            "`--jobs 0` is not a worker count: state a positive number, or `auto` for this machine's \
             own count",
        )),
        Ok(count) => Ok(count),
        Err(error) => Err(Error::invalid_input(
            "cli_jobs_value",
            format!(
                "`{value}` is not a worker count ({error}): state a positive number, or `auto`"
            ),
        )),
    }
}

/// The byte allowance this stream's own encoding runs under.
///
/// It is the request's remaining `output_bytes` reading, taken once before the operation starts: the
/// adapter may hold and write no more than what the request declared for output. The reading is a
/// *reading* and not a charge because the operation holds the caller's one budget mutably for the whole
/// call — a sink callback borrows only itself — so the adapter carries the number it is bounded by
/// instead of taking a permit the operation would have to release. The two accounts stay separate and
/// are both readable: the request's totals are the report's `usage`, and the stream's own bytes are
/// what the output file holds.
fn stream_allowance(budget: &jarde::Budget) -> u64 {
    budget
        .limits()
        .output_bytes
        .saturating_sub(budget.usage().output_bytes)
}

/// Creates the destination exclusively.
///
/// `create_new` is the whole statement: this command never truncates, never appends and never replaces
/// — a path that already exists is refused and left exactly as it was.
fn create_output(path: &Path) -> Result<File, Error> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| Error::Io {
            operation: "cli_create_output".to_owned(),
            message: format!("`{}`: {error}", path.display()),
        })
}

/// The exit status one finished run states.
///
/// The rule is design decision 7's, and it reads the run's own planes rather than re-deriving them:
/// only a `final` record that was confirmed beside an aggregate `Complete` is a success; a failure of
/// the input, the infrastructure or the output is 2 — the same status a usage refusal exits with; and
/// every other run that did not complete is 4, which is what a budget stop, a stopped method, a refused
/// class, a damaged traversal and a missing `final` all classify as. An output failure outranks
/// everything: an I/O failure observed after a record was written is the run's outcome, and a visible
/// line does not replace it.
fn exit_status(stopped: Option<&Stopped>, report: &BulkRecoveryReport) -> u8 {
    if matches!(stopped, Some(Stopped::Output(_))) {
        return EXIT_USAGE;
    }
    if matches!(report.summary.execution, ExecutionReport::Complete { .. })
        && report.final_delivered
    {
        return EXIT_COMPLETE;
    }
    if matches!(report.summary.execution, ExecutionReport::Failed { .. }) {
        return EXIT_USAGE;
    }
    EXIT_INCOMPLETE
}

/// The reason an unfinished run states, when the adapter is not the one that stopped the stream.
///
/// The library's report names the reason already — its aggregate state, its first stop and the
/// diagnostics that located what went wrong — and this adapter does not translate that vocabulary into
/// a second one. What it states is *what happened to this command's stream*, with the library's own
/// words for why, because the error variants are the library's failure kinds and none of them means
/// "the operation stopped short of its scope".
fn unfinished_error(report: &BulkRecoveryReport) -> Error {
    let end = if report.final_delivered {
        "a `final` record was delivered, and it states that aggregate"
    } else {
        "no `final` record was delivered, so the stream is a prefix"
    };
    Error::invalid_input(
        "cli_export_unfinished",
        format!(
            "the operation did not reach the end of its scope (aggregate `{}`, first stop {}, {} \
             diagnostic(s) in the report); {end}",
            report.summary.status(),
            stop_document(report.stop.as_ref()),
            report.diagnostics.len(),
        ),
    )
}

/// The library's own stop record, as its own document.
///
/// Quoted rather than re-spelled: the vocabulary of a failure message is then the vocabulary of a
/// report. Rendering it cannot fail — three unit enums and an optional fourth, all of the crate's own —
/// and the one place that could says so instead of failing a run over a diagnosis.
fn stop_document(stop: Option<&BulkStop>) -> String {
    match stop {
        Some(stop) => serde_json::to_string(stop)
            .unwrap_or_else(|_| "a stop this adapter cannot quote".into()),
        None => "none recorded".to_owned(),
    }
}

/// One record of the stream: the library's own event beside the `kind` that names it.
///
/// The tag is the only thing this adapter adds to a record, and it is added as a tag rather than as a
/// field of its own: everything beside `kind` is the event's own serialization, field for field, so a
/// consumer that ignores the tag is reading the library's document.
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Record<'a> {
    /// The view the operation covers and the configuration it runs under, published before anything is
    /// discovered.
    Header {
        #[serde(flatten)]
        event: &'a BulkHeaderEvent,
    },
    /// One class prepared: its identity, the evidence of its one read and the count it declares.
    ClassPrepared {
        #[serde(flatten)]
        event: &'a ClassPreparedEvent,
    },
    /// One method's record, in the physical and declaration order of its class.
    Method {
        #[serde(flatten)]
        event: &'a MethodResultEvent,
    },
    /// One located diagnostic.
    Diagnostic {
        #[serde(flatten)]
        event: &'a BulkDiagnosticEvent,
    },
    /// The end of one class: what it declared and what its execution reached.
    ClassEnd {
        #[serde(flatten)]
        event: &'a ClassEndEvent,
    },
    /// The last event of a stream that reached its end: the operation's bounded counts and boundaries.
    Final {
        #[serde(flatten)]
        event: &'a BulkFinalEvent,
    },
}

/// What encoding one record produced.
enum Encoded {
    /// The record is in the buffer, framing included, and is this many bytes long.
    Line(usize),
    /// The record did not fit the remaining allowance. It needed this many bytes in total, and no byte
    /// of it was held beyond the allowance.
    TooLarge { needed: u64 },
}

/// Why the adapter ended the stream itself, when it did.
enum Stopped {
    /// A record did not fit the stream's remaining allowance. Nothing was written for it, the prefix
    /// before it stands, and the run is incomplete rather than failed.
    Allowance(Error),
    /// The output failed: a record could not be written, or the file could not be flushed, closed and
    /// read back. The confirmed prefix stands — the file holds whole lines up to the last boundary — and
    /// the run is a failure whatever the library's own planes state.
    Output(Error),
}

impl Stopped {
    /// The error document this stop states.
    fn error(&self) -> &Error {
        match self {
            Self::Allowance(error) | Self::Output(error) => error,
        }
    }
}

/// The one destination of an `export` stream: the file, the bounded encoder and the stream's own
/// remaining allowance.
struct Stream {
    /// The destination, created exclusively before the operation started. It is taken when the stream
    /// is closed, so "one closing per stream" is a property of the type.
    file: Option<File>,
    /// The destination's path, for the messages of a failure.
    path: PathBuf,
    /// The encoding of one record, reused from record to record: the only buffer this adapter owns, and
    /// it never holds more than the stream's remaining allowance.
    buffer: Vec<u8>,
    /// The byte that ends one record, as the command's `--format` states it.
    framing: u8,
    /// The request's remaining `output_bytes` reading, taken before the operation started.
    allowance: u64,
    /// How many bytes of confirmed records the file holds: delivery is the prefix up to this offset.
    confirmed: u64,
    /// Why the adapter stopped the stream, if it did.
    stopped: Option<Stopped>,
}

impl Stream {
    fn new(file: File, path: PathBuf, framing: u8, allowance: u64) -> Self {
        Self {
            file: Some(file),
            path,
            buffer: Vec::new(),
            framing,
            allowance,
            confirmed: 0,
            stopped: None,
        }
    }

    /// How much of the allowance is left after the records already confirmed.
    fn remaining(&self) -> u64 {
        self.allowance.saturating_sub(self.confirmed)
    }

    /// Records why the adapter stopped, keeping the observation that states the run's class.
    ///
    /// Both observations can happen — a record the allowance refused, then a file that could not be
    /// closed — and the output failure is the one the exit status is read off, so it replaces an
    /// allowance refusal and is never itself replaced.
    fn note(&mut self, stopped: Stopped) {
        let replace = matches!(
            (&self.stopped, &stopped),
            (None, _) | (Some(Stopped::Allowance(_)), Stopped::Output(_))
        );
        if replace {
            self.stopped = Some(stopped);
        }
    }

    /// Encodes one record into the buffer, framing included, or answers that it does not fit.
    fn encode(&mut self, record: &Record<'_>) -> Result<Encoded, Error> {
        let remaining = self.remaining();
        self.buffer.clear();
        let mut encoder = Bounded::new(&mut self.buffer, remaining);
        let outcome = serde_json::to_writer(&mut encoder, record);
        encoder.byte(self.framing);
        // The encoder is not used again: its two answers are what the rest of this function reads, and
        // the borrow it held on the buffer ends with the fields below.
        let refused = encoder.refused;
        let needed = encoder.needed;
        if refused {
            return Ok(Encoded::TooLarge { needed });
        }
        if let Err(error) = outcome {
            return Err(Error::invalid_input(
                "cli_export_json",
                format!("a record could not be encoded: {error}"),
            ));
        }
        Ok(Encoded::Line(self.buffer.len()))
    }

    /// Encodes, debits and writes one record, and answers whether the stream goes on.
    fn deliver(&mut self, record: Record<'_>) -> jarde::Result<SinkControl> {
        let line = match self.encode(&record)? {
            Encoded::Line(line) => line,
            Encoded::TooLarge { needed } => {
                // The record is not written — not even partially — so the prefix is exactly what was
                // confirmed before it, and the stop states the numbers: the allowance this stream ran
                // under, what it delivered, and what the record would have needed.
                self.note(Stopped::Allowance(Error::BudgetExceeded {
                    dimension: BudgetDimension::OutputBytes,
                    limit: self.allowance,
                    consumed: self.confirmed,
                    requested: needed,
                }));
                return Ok(SinkControl::Stop);
            }
        };
        self.write(line)?;
        Ok(SinkControl::Continue)
    }

    /// Writes one encoded line and confirms it.
    ///
    /// One `write_all` of the whole line, so a line reaches the file as a unit, and the confirmed
    /// offset moves only after that call returned `Ok`. A write that fails part way is rolled back to
    /// the last confirmed boundary — best effort, because the write has already failed once — since a
    /// partial line is not a delivered record and a consumer must not meet one.
    fn write(&mut self, line: usize) -> jarde::Result<()> {
        let Some(mut file) = self.file.take() else {
            // The stream has one file and one closing, and `final` is the library's last record: a
            // record after the closing is not something this operation produces, and it is answered
            // rather than panicked on.
            return Err(self.output_failure(
                "cli_write_output",
                "the stream is already closed".to_owned(),
            ));
        };
        let outcome = file.write_all(&self.buffer[..line]);
        if outcome.is_err() {
            let _ = file.set_len(self.confirmed);
        }
        self.file = Some(file);
        match outcome {
            Ok(()) => {
                self.confirmed = self
                    .confirmed
                    .saturating_add(u64::try_from(line).unwrap_or(u64::MAX));
                Ok(())
            }
            Err(error) => Err(self.output_failure(
                "cli_write_output",
                format!(
                    "the output file `{}` could not be written: {error}",
                    self.path.display()
                ),
            )),
        }
    }

    /// Flushes and closes the destination, and only then lets the `final` record be confirmed.
    ///
    /// `std::fs::File` reports no error when its descriptor is closed and its `flush` is the one the
    /// stream owes — this stream holds nothing back, so there is nothing left to flush — which leaves
    /// the close's own outcome unobservable through the type. It is therefore checked by its
    /// consequence: the file now holds exactly the bytes this stream confirmed. A file that is shorter
    /// lost bytes, and one that is longer was written by something else; either way the run states a
    /// failure rather than confirming a record it cannot vouch for.
    fn close(&mut self) -> jarde::Result<()> {
        let Some(file) = self.file.take() else {
            return Err(self.output_failure(
                "cli_close_output",
                "the stream is already closed".to_owned(),
            ));
        };
        if let Err(error) = flush_and_close(file) {
            return Err(self.output_failure(
                "cli_close_output",
                format!(
                    "the output file `{}` could not be flushed: {error}",
                    self.path.display()
                ),
            ));
        }
        match std::fs::metadata(&self.path) {
            Ok(metadata) if metadata.len() == self.confirmed => Ok(()),
            Ok(metadata) => Err(self.output_failure(
                "cli_close_output",
                format!(
                    "the output file `{}` holds {} byte(s) after it was closed, while this stream \
                     confirmed {}",
                    self.path.display(),
                    metadata.len(),
                    self.confirmed
                ),
            )),
            Err(error) => Err(self.output_failure(
                "cli_close_output",
                format!(
                    "the output file `{}` could not be read back after it was closed: {error}",
                    self.path.display()
                ),
            )),
        }
    }

    /// Ends the stream: closes whatever the run left open and answers why the adapter stopped it.
    fn finish(mut self) -> Option<Stopped> {
        if self.file.is_some() {
            let _ = self.close();
        }
        self.stopped
    }

    /// Records an output failure and states it in the library's error type.
    ///
    /// The operation names the step that failed — writing a record, or closing the file the `final`
    /// record waits for — so a failure document says where the stream ended as well as why.
    fn output_failure(&mut self, operation: &str, message: String) -> Error {
        let error = Error::Io {
            operation: operation.to_owned(),
            message,
        };
        self.note(Stopped::Output(error.clone()));
        error
    }
}

/// Flushes a destination and closes it, in that order.
///
/// The two steps are one function because they are one confirmation: the `final` record waits for both.
/// Closing is `drop(file)` — `std::fs::File` has no fallible close — so this answers the flush's own
/// outcome, and [`Stream::close`] checks the close by reading the file back.
fn flush_and_close(mut file: File) -> io::Result<()> {
    file.flush()?;
    drop(file);
    Ok(())
}

impl RecoverySink for Stream {
    fn header(&mut self, event: &BulkHeaderEvent) -> jarde::Result<SinkControl> {
        self.deliver(Record::Header { event })
    }

    fn class_prepared(&mut self, event: &ClassPreparedEvent) -> jarde::Result<SinkControl> {
        self.deliver(Record::ClassPrepared { event })
    }

    fn method(&mut self, event: &MethodResultEvent) -> jarde::Result<SinkControl> {
        self.deliver(Record::Method { event })
    }

    fn class_end(&mut self, event: &ClassEndEvent) -> jarde::Result<SinkControl> {
        self.deliver(Record::ClassEnd { event })
    }

    fn diagnostic(&mut self, event: &BulkDiagnosticEvent) -> jarde::Result<SinkControl> {
        self.deliver(Record::Diagnostic { event })
    }

    fn final_event(&mut self, event: &BulkFinalEvent) -> jarde::Result<SinkControl> {
        if self.deliver(Record::Final { event })? == SinkControl::Stop {
            // The `final` record did not fit the allowance. It is not written and it must not be:
            // completion is not a reason to spend an allowance the request already used up, and the
            // stream keeps the prefix it confirmed.
            return Ok(SinkControl::Stop);
        }
        // The record is written; the event is confirmed only now, after the file was flushed and
        // closed. A failure here leaves the library's `final_delivered` false however visible the line
        // already is in the file.
        self.close()?;
        Ok(SinkControl::Continue)
    }
}

/// The bounded encoder of one record.
///
/// It holds the record's bytes until they exceed the remaining allowance and *counts* the rest without
/// holding it, which is what makes both halves of the encoding rule true at once: the buffer this
/// adapter owns never holds more than the allowance — so a record that cannot be afforded does not
/// become an allocation either — and the record that ended the run is still stated with its real size
/// rather than with a lower bound. A write is never refused *to the serializer* (a refusal would stop
/// it mid-record): the flag reports that bytes went uncounted for, and the count is what states them.
struct Bounded<'a> {
    buffer: &'a mut Vec<u8>,
    /// What is left of the stream's allowance.
    remaining: u64,
    /// Whether any part of this record exceeded the allowance.
    refused: bool,
    /// How many bytes the record needs in total, once it exceeded the allowance.
    needed: u64,
}

impl<'a> Bounded<'a> {
    fn new(buffer: &'a mut Vec<u8>, remaining: u64) -> Self {
        Self {
            buffer,
            remaining,
            refused: false,
            needed: 0,
        }
    }

    /// Takes these bytes, or counts them once the record has already gone over.
    fn put(&mut self, bytes: &[u8]) -> bool {
        let amount = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if self.refused {
            self.needed = self.needed.saturating_add(amount);
            return true;
        }
        if amount > self.remaining {
            self.refused = true;
            // What the buffer already holds plus this piece is where the record first went over, so
            // the running total starts here and every later piece is added by the branch above.
            self.needed = u64::try_from(self.buffer.len())
                .unwrap_or(u64::MAX)
                .saturating_add(amount);
            return false;
        }
        self.remaining -= amount;
        self.buffer.extend_from_slice(bytes);
        true
    }

    /// Takes the framing byte of one record.
    fn byte(&mut self, byte: u8) -> bool {
        self.put(&[byte])
    }
}

impl Write for Bounded<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.put(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
