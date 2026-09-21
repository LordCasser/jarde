//! `export`: the streaming adapter, driven as a process.
//!
//! The command's contract has five halves, and each is checked against a process's own behaviour
//! rather than against a claim about it:
//!
//! * **one process, one library operation.** A run writes one `header` first and one `final` last,
//!   with every class's records between its own `class_prepared` and `class_end` in physical order; the
//!   whole stream is then compared field by field with the events **one** in-process
//!   `Engine::recover_all` hands a sink for the same request, which is what "no per-method process and
//!   no second analysis" looks like from a consumer's side.
//! * **the parameters are the request's.** `--jobs auto` reports the machine's own count as the
//!   *requested* value and the library's window reduction as the *effective* one, `--jobs 1` reports
//!   one and one, and `0` or a value that is not a number is refused with exit 2 before any file
//!   exists.
//! * **the budget is the command's own.** A whole package — three hundred classes under one root and
//!   a nested library beside them — reaches its `final` with `traversal_complete` under **no**
//!   `--budget` at all, the header states the finite ceiling it ran under, and the counted dimensions
//!   the task defaults do not fund can be stated by the caller (`--budget ir_items=…`), while a
//!   dimension that is not a count and a limit of zero stay input errors.
//! * **the destination is created exclusively.** An existing path is refused with exit 2 and left
//!   byte for byte as it was.
//! * **delivery is a whole line, funded before it is written.** A tiny `output_bytes` allowance leaves
//!   a readable prefix, no partial last line, no `final` claiming more than the run confirmed, and a
//!   non-zero exit that states the allowance's own numbers; a scope whose traversal is damaged reaches
//!   a `final` with honest counts and exits 4.
//!
//! The archives here are built in memory by the stored-only ZIP writer below, and the classes inside
//! them are the repository's own committed fixtures: `Scope.class` and `Holder.class` declare real
//! bodies, so the records' recovered text is checked against the library's values rather than against
//! the empty strings a synthetic class would produce. The whole-package fixture is the one exception,
//! and it says why: a package needs classes of its own, so it generates them (and puts the committed
//! fixtures in its nested library).

use jarde::{
    ArchiveNameBytes, ArtifactInput, BulkDiagnosticEvent, BulkFinalEvent, BulkHeaderEvent,
    BulkRecoveryRequest, ClassEndEvent, ClassPreparedEvent, Engine, EnvironmentPolicy,
    EnvironmentRequest, LayoutMode, Limits, LoadRoot, LoaderId, MethodDelivery, MethodResultEvent,
    MultiReleasePolicy, PhysicalScope, RecoverySink, RuntimeProfile, SinkControl, task_budget,
};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const BIN: &str = env!("CARGO_BIN_EXE_jarde-cli");

/// The exit statuses the streaming command promises, spelled here so a renamed constant in the crate is
/// a compile error in this file rather than a silently unchecked number.
const EXIT_COMPLETE: i32 = 0;
const EXIT_USAGE: i32 = 2;
const EXIT_INCOMPLETE: i32 = 4;

/// The root container id every snapshot's own root carries.
///
/// A tree scope names it and the library refuses any other, so it is a fact of the identity model
/// rather than of this fixture — the tree enumeration publishes it beside every container it walks.
const ROOT_CONTAINER: &str = "root";

/// One class with a constructor, a static initializer and a branch, so its records carry text.
const SCOPE: &[u8] = include_bytes!("../../../tests/fixtures/p3-scope/v8/Scope.class");
/// A class with four ordinary members, used for the multi-class scope.
const HOLDER: &[u8] = include_bytes!("../../../tests/fixtures/p3-declaration/v8/Holder.class");
/// An interface: one of its members is a declaration without a body at all.
const SHAPE: &[u8] = include_bytes!("../../../tests/fixtures/p3-declaration/v8/Shape.class");
/// The compiled sample one case compares the whole stream against, in process.
const HISTORICAL: &[u8] =
    include_bytes!("../../../tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-cli-export-{}-{nonce}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("create the test directory");
        Self { path }
    }

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.path.join(name);
        fs::write(&path, bytes).expect("write the fixture");
        path
    }

    fn join(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

// ---------------------------------------------------------------------------------------------
// Fixtures: a stored-only ZIP writer, and the scopes the cases are about
// ---------------------------------------------------------------------------------------------

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc >>= 1;
            crc ^= 0xedb8_8320 & mask;
        }
    }
    !crc
}

/// One stored entry of an in-memory archive.
struct Stored<'a> {
    name: &'a [u8],
    data: &'a [u8],
}

/// A minimal STORED-only ZIP writer: this test package declares no archive dependency, so the few
/// dozen bytes of the format these fixtures need are written here.
fn zip(entries: &[Stored<'_>]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut directory = Vec::new();
    for entry in entries {
        let offset = u32::try_from(output.len()).expect("the fixture archive fits u32");
        let crc = crc32(entry.data);
        let size = u32::try_from(entry.data.len()).expect("the fixture entry fits u32");
        let name_len = u16::try_from(entry.name.len()).expect("the fixture name fits u16");
        output.extend_from_slice(&0x0403_4b50_u32.to_le_bytes());
        output.extend_from_slice(&20_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes()); // stored
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&crc.to_le_bytes());
        output.extend_from_slice(&size.to_le_bytes());
        output.extend_from_slice(&size.to_le_bytes());
        output.extend_from_slice(&name_len.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(entry.name);
        output.extend_from_slice(entry.data);
        directory.extend_from_slice(&0x0201_4b50_u32.to_le_bytes());
        directory.extend_from_slice(&20_u16.to_le_bytes());
        directory.extend_from_slice(&20_u16.to_le_bytes());
        directory.extend_from_slice(&0_u16.to_le_bytes());
        directory.extend_from_slice(&0_u16.to_le_bytes());
        directory.extend_from_slice(&0_u16.to_le_bytes());
        directory.extend_from_slice(&0_u16.to_le_bytes());
        directory.extend_from_slice(&crc.to_le_bytes());
        directory.extend_from_slice(&size.to_le_bytes());
        directory.extend_from_slice(&size.to_le_bytes());
        directory.extend_from_slice(&name_len.to_le_bytes());
        directory.extend_from_slice(&0_u16.to_le_bytes());
        directory.extend_from_slice(&0_u16.to_le_bytes());
        directory.extend_from_slice(&0_u16.to_le_bytes());
        directory.extend_from_slice(&0_u16.to_le_bytes());
        directory.extend_from_slice(&0_u32.to_le_bytes());
        directory.extend_from_slice(&offset.to_le_bytes());
        directory.extend_from_slice(entry.name);
    }
    let directory_offset = u32::try_from(output.len()).expect("the fixture archive fits u32");
    let directory_size = u32::try_from(directory.len()).expect("the fixture directory fits u32");
    let count = u16::try_from(entries.len()).expect("the fixture entry count fits u16");
    output.extend_from_slice(&directory);
    output.extend_from_slice(&0x0605_4b50_u32.to_le_bytes());
    output.extend_from_slice(&0_u16.to_le_bytes());
    output.extend_from_slice(&0_u16.to_le_bytes());
    output.extend_from_slice(&count.to_le_bytes());
    output.extend_from_slice(&count.to_le_bytes());
    output.extend_from_slice(&directory_size.to_le_bytes());
    output.extend_from_slice(&directory_offset.to_le_bytes());
    output.extend_from_slice(&0_u16.to_le_bytes());
    output
}

/// Three classes at one archive's root, in the order the scope walks them.
///
/// Each entry is named by the internal name its own bytes declare, which is what the `plain-jar`
/// policy's prefix rule looks a class up by.
fn three_class_archive() -> Vec<u8> {
    zip(&[
        Stored {
            name: b"Scope.class",
            data: SCOPE,
        },
        Stored {
            name: b"Holder.class",
            data: HOLDER,
        },
        Stored {
            name: b"Shape.class",
            data: SHAPE,
        },
    ])
}

/// One class at the root beside a nested container that is not an archive at all.
///
/// The damaged entry is last, so the traversal walks the class before it: a run over this scope really
/// prepares something *and* cannot reach the end of the scope it was given.
fn damaged_archive() -> Vec<u8> {
    zip(&[
        Stored {
            name: b"Holder.class",
            data: HOLDER,
        },
        Stored {
            name: b"lib/broken.jar",
            data: b"not an archive at all",
        },
    ])
}

/// One class at the root beside one nested archive that holds a second class.
fn nested_archive() -> Vec<u8> {
    let nested = zip(&[Stored {
        name: b"Holder.class",
        data: HOLDER,
    }]);
    zip(&[
        Stored {
            name: b"Scope.class",
            data: SCOPE,
        },
        Stored {
            name: b"lib/nested.jar",
            data: &nested,
        },
    ])
}

// ---------------------------------------------------------------------------------------------
// Driving the binary, and reading what it wrote
// ---------------------------------------------------------------------------------------------

fn run(args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .output()
        .expect("run jarde-cli")
}

fn status(output: &Output) -> i32 {
    output
        .status
        .code()
        .expect("the command exited with a status rather than a signal")
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn path_of(path: &Path) -> &str {
    path.to_str().expect("the fixture path is UTF-8")
}

/// The error document one failed command wrote to standard error.
fn error_document(output: &Output) -> Value {
    serde_json::from_slice(&output.stderr).unwrap_or_else(|error| {
        panic!(
            "standard error is not one error document ({error}): {}",
            stderr_text(output)
        )
    })
}

/// Every record of one stream, in file order.
///
/// The read is deliberately strict: every line must be a complete JSON object, and the file must end
/// at a line boundary. A prefix that ended in the middle of a record would fail here rather than being
/// read as a slack tail.
fn stream(path: &Path) -> Vec<Value> {
    let bytes = fs::read(path).expect("the stream file is readable");
    assert!(
        bytes.is_empty() || bytes.ends_with(b"\n"),
        "the stream does not end at a record boundary: {} byte(s) ending {:?}",
        bytes.len(),
        &bytes[bytes.len().saturating_sub(16)..]
    );
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            serde_json::from_slice(line).unwrap_or_else(|error| {
                panic!(
                    "a stream line is not one JSON object ({error}): {}",
                    String::from_utf8_lossy(line)
                )
            })
        })
        .collect()
}

/// The raw lines of one stream, newline included, so a case can state the byte allowance it needs.
fn raw_lines(path: &Path) -> Vec<Vec<u8>> {
    fs::read(path)
        .expect("the stream file is readable")
        .split_inclusive(|byte| *byte == b'\n')
        .map(<[u8]>::to_vec)
        .collect()
}

fn kinds(records: &[Value]) -> Vec<String> {
    records
        .iter()
        .map(|record| {
            record["kind"]
                .as_str()
                .unwrap_or_else(|| panic!("a record has no kind: {record}"))
                .to_owned()
        })
        .collect()
}

fn first_of_kind<'a>(records: &'a [Value], kind: &str) -> &'a Value {
    records
        .iter()
        .find(|record| record["kind"] == json!(kind))
        .unwrap_or_else(|| panic!("the stream has no `{kind}` record: {:?}", kinds(records)))
}

/// The document without the one field two runs of the same request may differ in.
fn strip_elapsed(value: &Value) -> Value {
    match value {
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .filter(|(key, _)| key.as_str() != "elapsed_millis")
                .map(|(key, child)| (key.clone(), strip_elapsed(child)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(strip_elapsed).collect()),
        leaf => leaf.clone(),
    }
}

/// Whether one record's own frame order is the operation's: every class's records sit between its
/// `class_prepared` and its `class_end`, in increasing class ordinal, and `final` is last.
fn assert_frame_order(records: &[Value]) {
    let names = kinds(records);
    assert_eq!(
        names.first().map(String::as_str),
        Some("header"),
        "the stream does not start with its header: {names:?}"
    );
    assert_eq!(
        names.last().map(String::as_str),
        Some("final"),
        "the stream does not end with its final record: {names:?}"
    );
    assert_eq!(
        names.iter().filter(|name| *name == "header").count(),
        1,
        "one operation publishes one header: {names:?}"
    );
    let mut open: Option<u64> = None;
    let mut classes: Vec<u64> = Vec::new();
    for record in records {
        let ordinal = record["class_ordinal"].as_u64();
        match record["kind"].as_str() {
            Some("class_prepared") => {
                assert_eq!(open, None, "a class is prepared inside another: {names:?}");
                open = ordinal;
                classes.push(ordinal.expect("a prepared class states its ordinal"));
            }
            Some("class_end") => {
                assert_eq!(
                    open, ordinal,
                    "a class ends outside its own frame: {names:?}"
                );
                open = None;
            }
            Some("method") => assert_eq!(
                open, ordinal,
                "a method record sits outside its class's frame: {names:?}"
            ),
            _ => {}
        }
    }
    assert_eq!(open, None, "a class is left open: {names:?}");
    assert_eq!(
        classes,
        (0..u64::try_from(classes.len()).expect("the class count fits u64")).collect::<Vec<_>>(),
        "classes are delivered in physical order, numbered from zero: {names:?}"
    );
}

/// The request one `export` invocation over one fixture states, as the command's own declarations are
/// read: the same view, policy, profile, loader and worker count the cases above pass as parameters,
/// under the limits the run itself published.
///
/// The limits are the *published* ones, not a second copy of the command's defaults: the point of the
/// comparison below is that one configuration produced both documents, and the header is where that
/// configuration is stated.
fn request_of(
    snapshot: &jarde::ArtifactSnapshot,
    workers: usize,
    scope: PhysicalScope,
    limits: Limits,
) -> BulkRecoveryRequest {
    BulkRecoveryRequest::for_scope(
        EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
        workers,
        limits,
    )
}

/// The effective limits one stream's own `header` published, as the library's own value.
fn published_limits(records: &[Value]) -> Limits {
    serde_json::from_value(first_of_kind(records, "header")["limits"]["method"].clone())
        .expect("the header publishes the library's own `Limits` document")
}

// ---------------------------------------------------------------------------------------------
// One process, one operation: the configuration the header publishes
// ---------------------------------------------------------------------------------------------

#[test]
fn auto_and_one_worker_finish_in_one_process_and_the_header_reports_the_configuration() {
    let temp = TempDir::new();
    let input = temp.write("app.jar", &three_class_archive());
    let automatic = temp.join("auto.jsonl");
    let serial = temp.join("one.jsonl");

    for (label, jobs, output) in [("auto", "auto", &automatic), ("one", "1", &serial)] {
        let ran = run(&[
            "export",
            "--input",
            path_of(&input),
            "--policy",
            "plain-jar",
            "--jobs",
            jobs,
            "--output",
            path_of(output),
        ]);
        assert_eq!(
            status(&ran),
            EXIT_COMPLETE,
            "`--jobs {label}`: {}",
            stderr_text(&ran)
        );
    }

    let automatic_records = stream(&automatic);
    let serial_records = stream(&serial);
    // One process wrote one operation's frames: one header, every class's records inside its own frame,
    // one final. A per-method process or a second analysis would have no single stream to state this.
    assert_frame_order(&automatic_records);
    assert_frame_order(&serial_records);

    for records in [&automatic_records, &serial_records] {
        let summary = &first_of_kind(records, "final")["summary"];
        assert_eq!(summary["classes_seen"], json!(3), "{summary}");
        assert_eq!(summary["classes_prepared"], json!(3), "{summary}");
        assert_eq!(summary["traversal_complete"], json!(true), "{summary}");
        assert_eq!(
            summary["methods_delivered"], summary["methods_executed"],
            "every executed method was delivered: {summary}"
        );
        assert_eq!(
            summary["methods_declared"], summary["methods_executed"],
            "the whole scope ran: {summary}"
        );
        assert!(
            summary["methods_declared"].as_u64().unwrap_or(0) > 3,
            "the fixture really has methods: {summary}"
        );
        assert_eq!(
            summary["execution"]["status"],
            json!("complete"),
            "{summary}"
        );
    }

    // The header is the effective configuration: both worker values, the library's own capacities, and
    // the reduction the window states — the effective count is what the window can hold.
    let automatic_limits = &first_of_kind(&automatic_records, "header")["limits"];
    let requested = automatic_limits["workers_requested"]
        .as_u64()
        .expect("the header states the requested count");
    let effective = automatic_limits["workers_effective"]
        .as_u64()
        .expect("the header states the effective count");
    let window = automatic_limits["max_buffered_result_weight"]
        .as_u64()
        .expect("the header states the window")
        / automatic_limits["max_result_weight"]
            .as_u64()
            .expect("the header states the per-result ceiling");
    assert!(
        requested >= 1,
        "`auto` is at least one worker: {automatic_limits}"
    );
    assert!(
        requested
            <= u64::try_from(
                std::thread::available_parallelism().map_or(usize::MAX, |count| count.get())
            )
            .expect("this machine's own count fits u64"),
        "`auto` is this machine's own count, not more: {automatic_limits}"
    );
    assert_eq!(
        effective,
        requested.min(window).max(1),
        "the reduction is the window's, and it is published: {automatic_limits}"
    );
    assert_eq!(
        automatic_limits["max_class_bytes"],
        json!(jarde::DEFAULT_MAX_CLASS_BYTES)
    );
    assert_eq!(
        automatic_limits["max_result_weight"],
        json!(jarde::DEFAULT_MAX_RESULT_WEIGHT)
    );
    assert_eq!(
        automatic_limits["max_buffered_result_weight"],
        json!(jarde::DEFAULT_MAX_BUFFERED_RESULT_WEIGHT)
    );
    assert_eq!(
        automatic_limits["facts_capacity"],
        json!({"entries": 16384, "retained_bytes": 134217728}),
        "the command attaches its own bounded store for this one operation, and publishes it"
    );

    // `--jobs 1` really is the serial configuration, in the record and in the run.
    let serial_limits = &first_of_kind(&serial_records, "header")["limits"];
    assert_eq!(serial_limits["workers_requested"], json!(1));
    assert_eq!(serial_limits["workers_effective"], json!(1));
}

// ---------------------------------------------------------------------------------------------
// The parameters: a worker count, and a destination that must not exist
// ---------------------------------------------------------------------------------------------

#[test]
fn zero_and_nonnumeric_jobs_are_usage_errors_that_create_no_file() {
    let temp = TempDir::new();
    let input = temp.write("app.jar", &three_class_archive());
    for (value, code) in [
        ("0", "cli_jobs_zero"),
        ("abc", "cli_jobs_value"),
        ("1.5", "cli_jobs_value"),
    ] {
        let output = temp.join(&format!("jobs-{value}.jsonl"));
        let refused = run(&[
            "export",
            "--input",
            path_of(&input),
            "--policy",
            "plain-jar",
            "--jobs",
            value,
            "--output",
            path_of(&output),
        ]);
        assert_eq!(
            status(&refused),
            EXIT_USAGE,
            "`--jobs {value}` is not a worker count: {}",
            stderr_text(&refused)
        );
        assert!(
            !output.exists(),
            "a refused worker count created an output file: `--jobs {value}`"
        );
        let document = error_document(&refused);
        assert_eq!(document["status"], json!("error"));
        assert_eq!(document["error"]["kind"], json!("invalid_input"));
        assert_eq!(document["error"]["code"], json!(code));
    }
}

#[test]
fn an_existing_output_is_refused_and_left_exactly_as_it_was() {
    let temp = TempDir::new();
    let input = temp.write("app.jar", &three_class_archive());
    let before = b"a record from an earlier run\n";
    let output = temp.write("records.jsonl", before);

    let refused = run(&[
        "export",
        "--input",
        path_of(&input),
        "--policy",
        "plain-jar",
        "--jobs",
        "1",
        "--output",
        path_of(&output),
    ]);
    assert_eq!(status(&refused), EXIT_USAGE, "{}", stderr_text(&refused));
    assert_eq!(
        fs::read(&output).expect("the file is still readable"),
        before.to_vec(),
        "the refused run changed the file it was refused"
    );
    let document = error_document(&refused);
    assert_eq!(document["error"]["kind"], json!("io"));
    assert_eq!(document["error"]["operation"], json!("cli_create_output"));
}

// ---------------------------------------------------------------------------------------------
// The delivery rule: a whole line, funded before it is written
// ---------------------------------------------------------------------------------------------

#[test]
fn a_small_allowance_keeps_a_readable_prefix_and_never_confirms_completeness() {
    let temp = TempDir::new();
    let input = temp.write("HistoricalControlFlow.class", HISTORICAL);
    let whole = temp.join("whole.jsonl");
    let complete = run(&[
        "export",
        "--input",
        path_of(&input),
        "--policy",
        "single-class",
        "--jobs",
        "1",
        "--output",
        path_of(&whole),
    ]);
    assert_eq!(
        status(&complete),
        EXIT_COMPLETE,
        "{}",
        stderr_text(&complete)
    );
    let lines = raw_lines(&whole);
    assert!(
        lines.len() >= 5,
        "the fixture's stream is a real one: {} record(s)",
        lines.len()
    );

    // (1) An allowance that holds the header and little else: the next record cannot be afforded, and
    //     nothing of it is written — not even its first bytes.
    let header_only = temp.join("header-only.jsonl");
    let budget = u64::try_from(lines[0].len()).expect("the header line fits u64") + 64;
    let stopped = run(&[
        "export",
        "--input",
        path_of(&input),
        "--policy",
        "single-class",
        "--jobs",
        "1",
        "--budget",
        &format!("output_bytes={budget}"),
        "--output",
        path_of(&header_only),
    ]);
    assert_eq!(
        status(&stopped),
        EXIT_INCOMPLETE,
        "{}",
        stderr_text(&stopped)
    );
    let records = stream(&header_only);
    assert_eq!(
        kinds(&records),
        vec!["header".to_owned()],
        "the allowance delivered the header and no part of the record after it"
    );
    let written = fs::metadata(&header_only)
        .expect("the stream file is readable")
        .len();
    assert!(written <= budget, "the stream never exceeds its allowance");
    let document = error_document(&stopped);
    assert_eq!(document["error"]["kind"], json!("budget_exceeded"));
    assert_eq!(document["error"]["dimension"], json!("output_bytes"));
    assert_eq!(document["error"]["limit"], json!(budget));
    assert_eq!(document["error"]["consumed"], json!(written));
    assert!(
        written
            + document["error"]["requested"]
                .as_u64()
                .expect("the refusal states what the record needed")
            > budget,
        "the refused record really did not fit: {document}"
    );

    // (2) A prefix of several records has the same shape: every line complete, no `final` claiming
    //     completeness, fewer bytes than the allowance.
    let partial = temp.join("partial.jsonl");
    let half = fs::metadata(&whole).expect("the stream is readable").len() / 2;
    let stopped = run(&[
        "export",
        "--input",
        path_of(&input),
        "--policy",
        "single-class",
        "--jobs",
        "1",
        "--budget",
        &format!("output_bytes={half}"),
        "--output",
        path_of(&partial),
    ]);
    assert_eq!(
        status(&stopped),
        EXIT_INCOMPLETE,
        "{}",
        stderr_text(&stopped)
    );
    let records = stream(&partial);
    assert!(
        records.len() >= 2,
        "the prefix holds the records the allowance really funded: {:?}",
        kinds(&records)
    );
    assert!(
        records
            .iter()
            .all(|record| record["kind"] != json!("final")),
        "an unfinished stream has no final record: {:?}",
        kinds(&records)
    );
    assert!(
        fs::metadata(&partial)
            .expect("the stream is readable")
            .len()
            <= half,
        "the stream never exceeds its allowance"
    );

    // (3) An allowance that funds every record but the last: the `final` record is the one the run
    //     cannot afford, and completion is not a reason to spend past the allowance — the file keeps
    //     *all* the other records and states no `final` at all. The allowance is the prefix's own
    //     length plus one byte, so it is the last record that cannot fit whatever the allowance's own
    //     digits do to the earlier records' lengths.
    let last = u64::try_from(lines[lines.len() - 1].len()).expect("the final line fits u64");
    let prefix = fs::metadata(&whole)
        .expect("the stream is readable")
        .len()
        .checked_sub(last)
        .expect("the fixture's stream holds its last record")
        .checked_add(1)
        .expect("the prefix length fits u64");
    let without_final = temp.join("without-final.jsonl");
    let stopped = run(&[
        "export",
        "--input",
        path_of(&input),
        "--policy",
        "single-class",
        "--jobs",
        "1",
        "--budget",
        &format!("output_bytes={prefix}"),
        "--output",
        path_of(&without_final),
    ]);
    assert_eq!(
        status(&stopped),
        EXIT_INCOMPLETE,
        "{}",
        stderr_text(&stopped)
    );
    let records = stream(&without_final);
    let mut expected = kinds(&stream(&whole));
    expected.pop();
    assert_eq!(
        kinds(&records),
        expected,
        "the run delivered every record it could afford — and no `final` it could not"
    );
    let document = error_document(&stopped);
    assert_eq!(document["error"]["kind"], json!("budget_exceeded"));
    assert_eq!(document["error"]["limit"], json!(prefix));
    assert!(
        document["error"]["consumed"]
            .as_u64()
            .expect("the refusal states what the stream delivered")
            + document["error"]["requested"]
                .as_u64()
                .expect("the refusal states what the record needed")
            > prefix,
        "the record the run stopped on really did not fit: {document}"
    );
}

// ---------------------------------------------------------------------------------------------
// A scope whose traversal is damaged
// ---------------------------------------------------------------------------------------------

#[test]
fn an_incomplete_traversal_reaches_a_final_and_exits_four() {
    let temp = TempDir::new();
    let input = temp.write("damaged.jar", &damaged_archive());
    let output = temp.join("damaged.jsonl");
    let scope = json!({"kind": "artifact_tree", "root_container": ROOT_CONTAINER}).to_string();

    let incomplete = run(&[
        "export",
        "--input",
        path_of(&input),
        "--policy",
        "plain-jar",
        "--scope",
        &scope,
        "--jobs",
        "1",
        "--output",
        path_of(&output),
    ]);
    assert_eq!(
        status(&incomplete),
        EXIT_INCOMPLETE,
        "{}",
        stderr_text(&incomplete)
    );
    let records = stream(&output);
    assert_frame_order(&records);
    let summary = &first_of_kind(&records, "final")["summary"];
    assert_eq!(
        summary["traversal_complete"],
        json!(false),
        "the damaged subtree is not walked: {summary}"
    );
    assert_eq!(summary["classes_seen"], json!(1), "{summary}");
    assert_eq!(summary["classes_prepared"], json!(1), "{summary}");
    assert_eq!(summary["classes_refused"], json!(0), "{summary}");
    assert_eq!(
        summary["methods_delivered"], summary["methods_executed"],
        "the class the traversal did reach was delivered whole: {summary}"
    );
    assert_eq!(
        summary["methods_declared"], summary["methods_executed"],
        "and its methods really ran: {summary}"
    );
    assert!(
        summary["methods_declared"].as_u64().unwrap_or(0) > 0,
        "the fixture really has methods: {summary}"
    );
    assert_eq!(
        summary["execution"]["status"],
        json!("partial"),
        "{summary}"
    );

    // Where the traversal stopped is a located record of the stream, not only a number in a summary.
    let diagnostic = first_of_kind(&records, "diagnostic");
    assert_eq!(diagnostic["diagnostic"]["severity"], json!("error"));
    assert!(
        !diagnostic["diagnostic"]["code"]
            .as_str()
            .unwrap_or_default()
            .is_empty(),
        "the diagnostic names what happened: {diagnostic}"
    );
    assert_eq!(
        diagnostic["diagnostic"]["provenance"]["location"]["id"]["raw_name"],
        json!(b"lib/broken.jar".as_slice()),
        "and it is located at the entry that could not be walked: {diagnostic}"
    );

    // The exit status states the same aggregate the `final` record carries.
    let document = error_document(&incomplete);
    assert_eq!(document["error"]["code"], json!("cli_export_unfinished"));
    assert!(
        document["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("partial"),
        "the failure document names the aggregate: {document}"
    );
}

// ---------------------------------------------------------------------------------------------
// The records are the library's own values
// ---------------------------------------------------------------------------------------------

/// One in-process sink: every event as the library's own document, beside the typed method events the
/// identity and text comparison below needs.
#[derive(Default)]
struct Recorder {
    records: Vec<Value>,
    methods: Vec<MethodResultEvent>,
}

impl Recorder {
    fn keep<T: serde::Serialize>(&mut self, event: &T) {
        self.records
            .push(serde_json::to_value(event).expect("the library's own event serializes"));
    }
}

impl RecoverySink for Recorder {
    fn header(&mut self, event: &BulkHeaderEvent) -> Result<SinkControl, jarde::Error> {
        self.keep(event);
        Ok(SinkControl::Continue)
    }

    fn class_prepared(&mut self, event: &ClassPreparedEvent) -> Result<SinkControl, jarde::Error> {
        self.keep(event);
        Ok(SinkControl::Continue)
    }

    fn method(&mut self, event: &MethodResultEvent) -> Result<SinkControl, jarde::Error> {
        self.records
            .push(serde_json::to_value(event).expect("the library's own event serializes"));
        self.methods.push(event.clone());
        Ok(SinkControl::Continue)
    }

    fn class_end(&mut self, event: &ClassEndEvent) -> Result<SinkControl, jarde::Error> {
        self.keep(event);
        Ok(SinkControl::Continue)
    }

    fn diagnostic(&mut self, event: &BulkDiagnosticEvent) -> Result<SinkControl, jarde::Error> {
        self.keep(event);
        Ok(SinkControl::Continue)
    }

    fn final_event(&mut self, event: &BulkFinalEvent) -> Result<SinkControl, jarde::Error> {
        self.keep(event);
        Ok(SinkControl::Continue)
    }
}

#[test]
fn every_record_of_the_stream_is_the_library_value_of_the_same_run() {
    let temp = TempDir::new();
    let path = temp.write("HistoricalControlFlow.class", HISTORICAL);
    let output = temp.join("stream.jsonl");
    let complete = run(&[
        "export",
        "--input",
        path_of(&path),
        "--policy",
        "single-class",
        "--jobs",
        "1",
        "--output",
        path_of(&output),
    ]);
    assert_eq!(
        status(&complete),
        EXIT_COMPLETE,
        "{}",
        stderr_text(&complete)
    );
    let records = stream(&output);

    // The same request through the library's own entry, in this process: one operation, one sink,
    // under the very limits the run published in its header.
    // The same request through the library's own entry, in this process: one operation, one sink,
    // under the very limits the run published in its header — including the store the header
    // published, so both documents are produced by one configuration rather than two.
    let mut budget = task_budget(&[]).expect("the task defaults are a bounded budget");
    let published = published_limits(&records);
    let capacity: jarde::FactsCapacity = serde_json::from_value(
        first_of_kind(&records, "header")["limits"]["facts_capacity"].clone(),
    )
    .expect("the header publishes the reader's own capacity document");
    budget = budget.with_facts_cache(jarde::FactsCache::current(capacity));
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::Path(path.clone()), &mut budget)
        .expect("the fixture opens");
    let request = request_of(&snapshot, 1, PhysicalScope::SnapshotAll, published);
    let mut recorder = Recorder::default();
    let report = engine
        .recover_all(
            slice::from_ref(&snapshot),
            &request,
            &mut budget,
            &mut recorder,
        )
        .expect("the fixture's scope is recoverable");
    assert!(
        report.final_delivered,
        "the in-process run confirmed its own final record"
    );

    // The stream is that one operation's stream: the same frames, in the same order, each one the
    // library's own value with nothing but the `kind` tag added to it.
    assert_eq!(
        records.len(),
        recorder.records.len(),
        "the stream has exactly the frames one operation published: {:?}",
        kinds(&records)
    );
    for (index, (record, event)) in records.iter().zip(&recorder.records).enumerate() {
        let mut record = record.clone();
        let kind = record
            .as_object_mut()
            .expect("a record is an object")
            .remove("kind")
            .expect("every record carries its kind");
        assert!(
            kind.as_str().is_some_and(|kind| !kind.is_empty()),
            "record {index} carries a stable kind"
        );
        assert_eq!(
            strip_elapsed(&record),
            strip_elapsed(event),
            "record {index} ({kind}) is not the library's own value for the same run"
        );
    }

    // The identity and the text of a method record, against the values the library published for the
    // very same run: the raw name and descriptor bytes, the two ordinals, and the recovered text.
    assert!(!recorder.methods.is_empty(), "the fixture declares methods");
    let event = &recorder.methods[0];
    let record = first_of_kind(&records, "method");
    assert_eq!(record["class_ordinal"], json!(event.class_ordinal));
    assert_eq!(record["member_ordinal"], json!(event.member_ordinal));
    assert_eq!(record["method"]["name"], json!(event.method.name.0));
    assert_eq!(
        record["method"]["descriptor"],
        json!(event.method.descriptor.0)
    );
    let MethodDelivery::Recovered(recovered) = &event.delivery else {
        panic!(
            "the fixture's first member declares a body, so it recovered: {:?}",
            event.delivery
        );
    };
    assert_eq!(record["delivery"]["state"], json!("recovered"));
    assert_eq!(
        record["delivery"]["recovery"]["text"],
        json!(recovered.recovery().text)
    );
    assert!(
        !recovered.recovery().text.is_empty(),
        "the fixture's first member really recovered text"
    );
    assert!(
        record["delivery"]["recovery"]["source_map"]["segments"].is_array(),
        "the source map travels with the text: {record}"
    );
}

// ---------------------------------------------------------------------------------------------
// The roots declaration: one array document, or repeated single documents
// ---------------------------------------------------------------------------------------------

/// The load positions a fixture's own two containers are declared by, in walk order.
///
/// They come from the library's own tree enumeration, because a root is a *container identity* the
/// enumeration establishes and not a path this file could invent: the root container of the snapshot,
/// and the container the nested archive establishes at its entry.
fn container_roots(path: &Path) -> Vec<LoadRoot> {
    let mut budget = task_budget(&[]).expect("the task defaults are a bounded budget");
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::Path(path.to_path_buf()), &mut budget)
        .expect("the fixture opens");
    let tree = snapshot
        .enumerate_artifact_tree(&mut budget)
        .expect("the fixture's tree is readable");
    let root = tree
        .containers
        .iter()
        .find(|container| container.origin.steps.is_empty())
        .expect("the root container is published")
        .origin
        .clone();
    let nested = tree
        .containers
        .iter()
        .find(|container| {
            container
                .origin
                .steps
                .last()
                .is_some_and(|step| step.via_raw_name.0 == b"lib/nested.jar")
        })
        .expect("the nested library is a container of the tree")
        .origin
        .clone();
    vec![
        LoadRoot::Container {
            origin: root,
            prefix: ArchiveNameBytes(Vec::new()),
        },
        LoadRoot::Container {
            origin: nested,
            prefix: ArchiveNameBytes(Vec::new()),
        },
    ]
}

#[test]
fn a_roots_document_declares_the_same_roots_as_repeated_root_arguments() {
    let temp = TempDir::new();
    let input = temp.write("nested.jar", &nested_archive());
    let roots = container_roots(&input);
    assert_eq!(roots.len(), 2, "the fixture declares two containers");
    let documents: Vec<String> = roots
        .iter()
        .map(|root| serde_json::to_string(root).expect("a load position serializes"))
        .collect();
    let array = temp.write(
        "roots.json",
        serde_json::to_vec(&roots)
            .expect("the roots serialize")
            .as_slice(),
    );
    let reversed = roots.iter().rev().cloned().collect::<Vec<_>>();
    let reversed_array = temp.write(
        "reversed-roots.json",
        serde_json::to_vec(&reversed)
            .expect("the roots serialize")
            .as_slice(),
    );
    let scope = json!({"kind": "artifact_tree", "root_container": ROOT_CONTAINER}).to_string();

    // The same declaration twice: repeated `--root` documents, then one `--roots` array.
    let repeated = temp.join("repeated.jsonl");
    let mut arguments = vec![
        "export".to_owned(),
        "--input".to_owned(),
        path_of(&input).to_owned(),
        "--policy".to_owned(),
        "explicit-classpath".to_owned(),
        "--scope".to_owned(),
        scope.clone(),
        "--jobs".to_owned(),
        "1".to_owned(),
    ];
    for document in &documents {
        arguments.push("--root".to_owned());
        arguments.push(document.clone());
    }
    arguments.push("--output".to_owned());
    arguments.push(path_of(&repeated).to_owned());
    let mut borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let ran = run(&borrowed);
    assert_eq!(status(&ran), EXIT_COMPLETE, "{}", stderr_text(&ran));
    borrowed.clear();

    let declared = temp.join("declared.jsonl");
    let ran = run(&[
        "export",
        "--input",
        path_of(&input),
        "--policy",
        "explicit-classpath",
        "--scope",
        &scope,
        "--jobs",
        "1",
        "--roots",
        &format!("@{}", array.display()),
        "--output",
        path_of(&declared),
    ]);
    assert_eq!(status(&ran), EXIT_COMPLETE, "{}", stderr_text(&ran));

    // Both spellings declared the same environment, so both runs published the same stream: the
    // records are compared field by field, with the one field two runs may differ in removed.
    let repeated_records = stream(&repeated);
    let declared_records = stream(&declared);
    assert_eq!(kinds(&repeated_records), kinds(&declared_records));
    for (index, (repeated, declared)) in repeated_records.iter().zip(&declared_records).enumerate()
    {
        assert_eq!(
            strip_elapsed(repeated),
            strip_elapsed(declared),
            "record {index} differs between the two spellings of one declaration"
        );
    }
    // And the run's own environment identity is that declaration, in that order.
    let declared_roots =
        first_of_kind(&declared_records, "method")["delivery"]["analysis"]["environment_identity"]
            ["runtime"]["load_domain"]["roots"]
            .clone();
    assert_eq!(
        declared_roots,
        serde_json::to_value(&roots).expect("the roots serialize")
    );

    // The array's order is the declaration's order: the reversed array is a different declaration, and
    // the run reports it as written rather than sorting it.
    let reversed_output = temp.join("reversed.jsonl");
    let ran = run(&[
        "export",
        "--input",
        path_of(&input),
        "--policy",
        "explicit-classpath",
        "--scope",
        &scope,
        "--jobs",
        "1",
        "--roots",
        &format!("@{}", reversed_array.display()),
        "--output",
        path_of(&reversed_output),
    ]);
    assert_eq!(status(&ran), EXIT_COMPLETE, "{}", stderr_text(&ran));
    let reversed_records = stream(&reversed_output);
    assert_eq!(
        first_of_kind(&reversed_records, "method")["delivery"]["analysis"]["environment_identity"]
            ["runtime"]["load_domain"]["roots"],
        serde_json::to_value(&reversed).expect("the roots serialize")
    );

    // The two spellings are one declaration, so naming both is a usage error rather than an order this
    // adapter would have to invent.
    let both = temp.join("both.jsonl");
    let refused = run(&[
        "export",
        "--input",
        path_of(&input),
        "--policy",
        "explicit-classpath",
        "--scope",
        &scope,
        "--jobs",
        "1",
        "--root",
        &documents[0],
        "--roots",
        &format!("@{}", array.display()),
        "--output",
        path_of(&both),
    ]);
    assert_eq!(status(&refused), EXIT_USAGE, "{}", stderr_text(&refused));
    assert!(!both.exists(), "the refused run created a file");

    // And a document that is not an array is refused with the document this parameter takes, rather
    // than being read as a single load position.
    let single = temp.join("single.jsonl");
    let refused = run(&[
        "export",
        "--input",
        path_of(&input),
        "--policy",
        "explicit-classpath",
        "--scope",
        &scope,
        "--jobs",
        "1",
        "--roots",
        &documents[0],
        "--output",
        path_of(&single),
    ]);
    assert_eq!(status(&refused), EXIT_USAGE, "{}", stderr_text(&refused));
    assert!(!single.exists(), "the refused run created a file");
    assert_eq!(
        error_document(&refused)["error"]["code"],
        json!("cli_roots_json")
    );
}

// ---------------------------------------------------------------------------------------------
// A whole package: many classes under one root, a nested library beside them
// ---------------------------------------------------------------------------------------------

/// How many generated classes the whole-package fixture stores at its root's own root.
///
/// The number is what makes the fixture a *package* rather than one request's view, and it is read
/// off the dimension the command's own default set exists for: preparing one class of the fixture
/// costs the container's own directory records again, so the run's `archive_entries` grows with the
/// class count and a few hundred classes are already past the sixty-five thousand records a task
/// default funds — the shape a real jar showed, where two dozen classes used a whole default's
/// archive records up.
const PACKAGE_CLASSES: usize = 256;

/// How many ordinary methods each generated class declares beside its constructor.
const PACKAGE_METHODS: usize = 4;

/// The classes the fixture's nested library contributes, and the members each one declares as its
/// own bytes were compiled: `Scope` (a constructor, `<clinit>` and members with real local shapes),
/// `Holder` (a constructor, `<clinit>` and two ordinary members) and `Shape` (an interface, so one
/// of its members is a declaration with no body at all).
const PACKAGE_NESTED_METHODS: [u64; 3] = [8, 4, 3];

fn u16b(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn u32b(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

/// One constant-pool `Utf8` entry, answering its index.
fn utf8(pool: &mut Vec<Vec<u8>>, text: &[u8]) -> u16 {
    let mut entry = vec![1];
    u16b(
        &mut entry,
        u16::try_from(text.len()).expect("the fixture name fits u16"),
    );
    entry.extend_from_slice(text);
    pool.push(entry);
    u16::try_from(pool.len()).expect("the fixture pool fits u16")
}

/// One constant-pool `Class` entry over an already-pooled name.
fn class_entry(pool: &mut Vec<Vec<u8>>, name: u16) -> u16 {
    let mut entry = vec![7];
    u16b(&mut entry, name);
    pool.push(entry);
    u16::try_from(pool.len()).expect("the fixture pool fits u16")
}

/// One constant-pool `Methodref` over a class, a name and a descriptor.
fn method_ref(pool: &mut Vec<Vec<u8>>, class: u16, name: u16, descriptor: u16) -> u16 {
    let mut name_and_type = vec![12];
    u16b(&mut name_and_type, name);
    u16b(&mut name_and_type, descriptor);
    pool.push(name_and_type);
    let name_and_type_index = u16::try_from(pool.len()).expect("the fixture pool fits u16");
    let mut entry = vec![10];
    u16b(&mut entry, class);
    u16b(&mut entry, name_and_type_index);
    pool.push(entry);
    u16::try_from(pool.len()).expect("the fixture pool fits u16")
}

/// One generated class: the class it declares carries `name` itself — the entry it is stored at is
/// that name plus `.class` — and its members are a constructor that calls `java/lang/Object.<init>`
/// and `methods` ordinary members whose bodies really branch (`if (local == 0) local = 2`).
///
/// A generated class is what lets the fixture be a *package*: every class has to be a class of its
/// own, because a physical read binds the name a class declares to the entry it lives at, and the
/// repository's committed fixtures are only a handful of distinct names.
fn generated_class(name: &[u8], methods: usize) -> Vec<u8> {
    let mut pool: Vec<Vec<u8>> = Vec::new();
    let this_utf8 = utf8(&mut pool, name);
    let this_class = class_entry(&mut pool, this_utf8);
    let object_utf8 = utf8(&mut pool, b"java/lang/Object");
    let object_class = class_entry(&mut pool, object_utf8);
    let init_utf8 = utf8(&mut pool, b"<init>");
    let void_utf8 = utf8(&mut pool, b"()V");
    let super_init = method_ref(&mut pool, object_class, init_utf8, void_utf8);
    let code_utf8 = utf8(&mut pool, b"Code");
    let mut members: Vec<(u16, u16)> = Vec::new();
    for index in 0..methods {
        let name = format!("m{index}");
        let name_index = utf8(&mut pool, name.as_bytes());
        members.push((name_index, void_utf8));
    }

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 52);
    u16b(
        &mut bytes,
        u16::try_from(pool.len() + 1).expect("the fixture pool fits u16"),
    );
    for entry in &pool {
        bytes.extend_from_slice(entry);
    }
    u16b(&mut bytes, 0x0021);
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, object_class);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 0);
    u16b(
        &mut bytes,
        u16::try_from(members.len() + 1).expect("the fixture fits u16"),
    );
    // `aload_0; invokespecial java/lang/Object.<init>; return`.
    let mut constructor = vec![0x2a, 0xb7];
    constructor.push(u8::try_from(super_init >> 8).expect("the pool index fits u8"));
    constructor.push(u8::try_from(super_init & 0xff).expect("the pool index fits u8"));
    constructor.push(0xb1);
    write_method(
        &mut bytes,
        code_utf8,
        &GeneratedBody {
            name: init_utf8,
            descriptor: void_utf8,
            access_flags: 0x0001,
            max_stack: 1,
            max_locals: 1,
            code: &constructor,
        },
    );
    // `iconst_1; istore_1; iload_1; ifeq +5; iconst_2; istore_1; return`: one real branch whose two
    // paths meet at the return, so the body decodes, analyses and recovers as a body rather than as
    // an empty one.
    let body = [0x03, 0x3c, 0x1b, 0x99, 0x00, 0x05, 0x05, 0x3c, 0xb1];
    for (name, descriptor) in &members {
        write_method(
            &mut bytes,
            code_utf8,
            &GeneratedBody {
                name: *name,
                descriptor: *descriptor,
                access_flags: 0x0001,
                max_stack: 1,
                max_locals: 2,
                code: &body,
            },
        );
    }
    u16b(&mut bytes, 0);
    bytes
}

/// One generated member: the pool indices of its name and descriptor, and the body it declares.
struct GeneratedBody<'a> {
    name: u16,
    descriptor: u16,
    access_flags: u16,
    max_stack: u16,
    max_locals: u16,
    code: &'a [u8],
}

/// One method record with its `Code` attribute, so the member walk sees a complete body.
fn write_method(bytes: &mut Vec<u8>, code_name: u16, body: &GeneratedBody<'_>) {
    u16b(bytes, body.access_flags);
    u16b(bytes, body.name);
    u16b(bytes, body.descriptor);
    u16b(bytes, 1);
    u16b(bytes, code_name);
    let content_length = 2 + 2 + 4 + body.code.len() + 2 + 2;
    u32b(
        bytes,
        u32::try_from(content_length).expect("fixture code fits u32"),
    );
    u16b(bytes, body.max_stack);
    u16b(bytes, body.max_locals);
    u32b(
        bytes,
        u32::try_from(body.code.len()).expect("fixture code fits u32"),
    );
    bytes.extend_from_slice(body.code);
    u16b(bytes, 0);
    u16b(bytes, 0);
}

/// A whole package: [`PACKAGE_CLASSES`] generated classes at one archive's root, one nested library
/// holding this repository's own committed classes, and nothing else.
///
/// Every class of it declares a name the environment can really bind — the generated ones under
/// `p/`, the nested ones at the nested container's own root — so the run is a whole package of
/// classes that resolve, decode, analyse and recover rather than a fixture whose results are all
/// refusals.
fn bulk_package() -> Vec<u8> {
    let mut names: Vec<Vec<u8>> = Vec::with_capacity(PACKAGE_CLASSES);
    let mut data: Vec<Vec<u8>> = Vec::with_capacity(PACKAGE_CLASSES);
    for index in 0..PACKAGE_CLASSES {
        let internal = format!("p/C{index:04}").into_bytes();
        let mut entry = internal.clone();
        entry.extend_from_slice(b".class");
        names.push(entry);
        data.push(generated_class(&internal, PACKAGE_METHODS));
    }
    let nested = zip(&[
        Stored {
            name: b"Scope.class",
            data: SCOPE,
        },
        Stored {
            name: b"Holder.class",
            data: HOLDER,
        },
        Stored {
            name: b"Shape.class",
            data: SHAPE,
        },
    ]);
    let mut stored: Vec<Stored<'_>> = names
        .iter()
        .zip(data.iter())
        .map(|(name, data)| Stored { name, data })
        .collect();
    stored.push(Stored {
        name: b"lib/nested.jar",
        data: &nested,
    });
    zip(&stored)
}

/// The whole-package run one case drives: the fixture above, both containers declared as roots, and
/// the budget declarations the caller added — nothing else. The worker count is the command's own
/// default, because the configuration under test is the one a caller who states no parameters gets.
fn run_package(input: &Path, roots: &Path, budgets: &[&str], output: &Path) -> Output {
    let scope = json!({"kind": "artifact_tree", "root_container": ROOT_CONTAINER}).to_string();
    let mut arguments = vec![
        "export".to_owned(),
        "--input".to_owned(),
        path_of(input).to_owned(),
        "--policy".to_owned(),
        "explicit-classpath".to_owned(),
        "--scope".to_owned(),
        scope,
        "--roots".to_owned(),
        format!("@{}", roots.display()),
    ];
    for budget in budgets {
        arguments.push("--budget".to_owned());
        arguments.push((*budget).to_owned());
    }
    arguments.push("--output".to_owned());
    arguments.push(path_of(output).to_owned());
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();
    run(&borrowed)
}

/// The `final` record's summary of one stream, as the JSON the command wrote.
fn summary_of(records: &[Value]) -> Value {
    first_of_kind(records, "final")["summary"].clone()
}

#[test]
fn a_whole_package_reaches_its_final_under_the_commands_own_defaults() {
    let temp = TempDir::new();
    let input = temp.write("package.jar", &bulk_package());
    let roots = temp.write(
        "roots.json",
        &serde_json::to_vec(&container_roots(&input)).expect("the roots serialize"),
    );
    let output = temp.join("package.jsonl");

    // No `--budget` at all: the command's own defaults are the configuration, which is exactly the
    // case that used to stop a real package a few dozen classes in.
    let complete = run_package(&input, &roots, &[], &output);
    assert_eq!(
        status(&complete),
        EXIT_COMPLETE,
        "{}",
        stderr_text(&complete)
    );
    let records = stream(&output);
    assert_frame_order(&records);
    let summary = summary_of(&records);
    assert_eq!(summary["traversal_complete"], json!(true), "{summary}");
    assert_eq!(summary["classes_refused"], json!(0), "{summary}");
    assert_eq!(
        summary["classes_seen"],
        json!(u64::try_from(PACKAGE_CLASSES).expect("the count fits") + 3),
        "{summary}"
    );
    assert_eq!(
        summary["classes_prepared"], summary["classes_seen"],
        "{summary}"
    );
    let nested_methods: u64 = PACKAGE_NESTED_METHODS.iter().sum();
    let expected = u64::try_from(PACKAGE_CLASSES).expect("the count fits")
        * (1 + PACKAGE_METHODS as u64)
        + nested_methods;
    assert_eq!(summary["methods_declared"], json!(expected), "{summary}");
    assert_eq!(
        summary["methods_executed"], summary["methods_declared"],
        "every declared method of the whole package ran: {summary}"
    );
    assert_eq!(
        summary["methods_delivered"], summary["methods_executed"],
        "and every one of them was delivered: {summary}"
    );
    assert_eq!(
        summary["execution"]["status"],
        json!("complete"),
        "{summary}"
    );

    // The header publishes the configuration the run really used: this command's own finite ceiling
    // for every counted dimension, its own wall clock, and the task defaults for the two depths no
    // budget raises.
    let limits = &first_of_kind(&records, "header")["limits"]["method"];
    for dimension in [
        "input_bytes",
        "archive_entries",
        "entry_bytes",
        "read_bytes",
        "class_bytes",
        "attribute_bytes",
        "code_bytes",
        "result_items",
        "output_bytes",
        "class_headers",
        "method_bodies",
        "ir_items",
        "ir_edges",
        "analysis_steps",
        "normalization_clones",
    ] {
        assert_eq!(
            limits[dimension],
            json!(1_u64 << 40),
            "the header states the command's own ceiling for {dimension}: {limits}"
        );
    }
    // The wall clock is the command's own too, bounded and stated rather than unbounded: a whole
    // package takes hours in the worst case this command is sized for, and never `u64::MAX`.
    assert_eq!(
        limits["elapsed_millis"],
        json!(2 * 60 * 60 * 1000),
        "{limits}"
    );
    assert_eq!(limits["nested_depth"], json!(4), "{limits}");
    assert_eq!(limits["dependency_depth"], json!(8), "{limits}");

    // The members of the package were really decoded and delivered: not one class or member was
    // refused, none was stopped or oversized, and the only member without a body is the nested
    // interface's abstract one. Every other member is a delivered record — a recovered body where
    // the recovery layer could write statements, and its own explanation where this fixture's
    // hand-written branch falls outside the statements it can prove.
    assert_eq!(summary["outcomes"]["refused"], json!(0), "{summary}");
    assert_eq!(summary["outcomes"]["not_produced"], json!(0), "{summary}");
    assert_eq!(summary["outcomes"]["oversized"], json!(0), "{summary}");
    assert_eq!(summary["outcomes"]["no_body"], json!(1), "{summary}");
    let delivered_members = summary["outcomes"]["produced"].as_u64().unwrap_or(0)
        + summary["outcomes"]["explanation_only"]
            .as_u64()
            .unwrap_or(0);
    assert_eq!(
        delivered_members,
        expected - 1,
        "every member of the package with a body was delivered: {summary}"
    );
    assert!(
        summary["outcomes"]["produced"].as_u64().unwrap_or(0) >= 256,
        "the constructors of the package really recovered statements: {summary}"
    );
}

#[test]
fn a_counted_dimension_override_is_accepted_and_stops_the_run_at_that_dimension() {
    let temp = TempDir::new();
    let input = temp.write("package.jar", &bulk_package());
    let roots = temp.write(
        "roots.json",
        &serde_json::to_vec(&container_roots(&input)).expect("the roots serialize"),
    );
    let output = temp.join("tight.jsonl");

    // One derived item is not a budget for any class: the first charge of `ir_items` is refused, and
    // the operation stops with that dimension as its first stop — a stop that used to be unreachable,
    // because the dimension could not be named at all (`budget_override_dimension_unknown`).
    let stopped = run_package(&input, &roots, &["ir_items=1"], &output);
    assert_eq!(
        status(&stopped),
        EXIT_INCOMPLETE,
        "{}",
        stderr_text(&stopped)
    );
    let document = error_document(&stopped);
    assert_eq!(document["error"]["code"], json!("cli_export_unfinished"));
    let message = document["error"]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("the failure document states a message: {document}"));
    assert!(
        message.contains("\"dimension\":\"ir_items\""),
        "the first stop the report states is the dimension the caller named: {message}"
    );
    assert!(
        message.contains("aggregate `partial`") || message.contains("aggregate `cancelled`"),
        "the run is not complete: {message}"
    );
    // A run the operation's own budget stopped does not spend the allowance on a `final` it would
    // have to claim completeness with: the file is the confirmed prefix, and the account of the run
    // is the failure document's own usage.
    let records = stream(&output);
    assert!(
        records
            .iter()
            .all(|record| record["kind"] != json!("final")),
        "a stopped run delivers no final record: {:?}",
        kinds(&records)
    );
    assert!(
        document["usage"]["ir_items"].as_u64().unwrap_or(u64::MAX) <= 1,
        "the stopped run states the dimension it spent: {document}"
    );

    // The same dimension, stated wide, is a run of the same size that finishes: the override is a
    // number the run obeys rather than a refusal.
    let wide = temp.join("wide.jsonl");
    let complete = run_package(&input, &roots, &["ir_items=1099511627776"], &wide);
    assert_eq!(
        status(&complete),
        EXIT_COMPLETE,
        "{}",
        stderr_text(&complete)
    );
    let summary = summary_of(&stream(&wide));
    assert_eq!(
        summary["execution"]["status"],
        json!("complete"),
        "{summary}"
    );
    assert_eq!(
        summary["methods_delivered"], summary["methods_declared"],
        "{summary}"
    );
}

#[test]
fn a_zero_or_unknown_dimension_is_refused_before_anything_is_read() {
    let temp = TempDir::new();
    let input = temp.write("package.jar", &bulk_package());
    let roots = temp.write(
        "roots.json",
        &serde_json::to_vec(&container_roots(&input)).expect("the roots serialize"),
    );
    for (declaration, code) in [
        ("ir_items=0", "budget_override_invalid"),
        ("input_bytes=0", "budget_override_invalid"),
        ("nested_depth=8", "budget_override_dimension_unknown"),
        ("dependency_depth=8", "budget_override_dimension_unknown"),
        ("nonsense=5", "budget_override_dimension_unknown"),
    ] {
        let output = temp.join("refused.jsonl");
        let refused = run_package(&input, &roots, &[declaration], &output);
        assert_eq!(
            status(&refused),
            EXIT_USAGE,
            "`--budget {declaration}`: {}",
            stderr_text(&refused)
        );
        assert!(
            !output.exists(),
            "a refused budget created the output file: `--budget {declaration}`"
        );
        let document = error_document(&refused);
        assert_eq!(
            document["error"]["code"],
            json!(code),
            "`--budget {declaration}`: {document}"
        );
        assert_eq!(document["usage"]["input_bytes"], json!(0), "{document}");
    }
}
