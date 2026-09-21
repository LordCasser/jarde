//! `class-source`: the shortcut that presents one class as Java source, driven as a process.
//!
//! The command's contract has three halves, and each is checked against the library's own values
//! rather than against the text alone:
//!
//! * **the text mode writes the library's own text.** Standard output is byte-for-byte the `text`
//!   field of the report the library answers the same request with, so the shortcut's main path is a
//!   projection of the document and not a second rendering of the bytes.
//! * **the JSON mode writes the library's own document.** The whole `ClassSourceReport` — the
//!   declaration, every member with the report of its own run, the planes — is compared field by
//!   field with the value the library returns in process.
//! * **the exit status is the report's own.** A presentation whose members were all recovered exits
//!   0; a class with a member whose run stopped exits 4 with that member marked in the text; a name
//!   that several definitions answer to presents nothing and exits 3.
//!
//! Every archive here is built in memory by this file, so a case can pin the exact member shape it
//! is about, and one case reads the committed compiled sample so the text is checked against bytes
//! no builder wrote.

use jarde::{
    ArtifactInput, ClassRef, ClassSourceReport, ClassSourceRequest, Engine, EnvironmentPolicy,
    EnvironmentRequest, LayoutMode, LoaderId, MultiReleasePolicy, PhysicalScope, RuntimeProfile,
    task_budget,
};
use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const BIN: &str = env!("CARGO_BIN_EXE_jarde-cli");

/// The exit statuses the task commands promise, spelled here so a renamed constant in the crate is
/// a compile error in this file rather than a silently unchecked number.
const EXIT_COMPLETE: i32 = 0;
const EXIT_USAGE: i32 = 2;
const EXIT_AMBIGUOUS: i32 = 3;
const EXIT_INCOMPLETE: i32 = 4;

/// The committed compiled sample the command's main path is checked against.
const HISTORICAL: &str =
    "../../tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class";

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
            "jarde-cli-class-source-{}-{nonce}-{}",
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
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

// ---------------------------------------------------------------------------------------------
// Fixtures: a stored-only ZIP writer and a class-file writer
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
        output.extend_from_slice(&0_u16.to_le_bytes());
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

fn u16b(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn u32b(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes());
}

/// The constant pool of a fixture class, interning each entry once.
#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    fn intern(&mut self, entry: Vec<u8>) -> u16 {
        if let Some(index) = self.entries.iter().position(|existing| *existing == entry) {
            return u16::try_from(index + 1).expect("the pool index fits u16");
        }
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("the pool index fits u16")
    }

    fn utf8(&mut self, text: &[u8]) -> u16 {
        let mut entry = vec![1_u8];
        u16b(
            &mut entry,
            u16::try_from(text.len()).expect("the fixture text fits u16"),
        );
        entry.extend_from_slice(text);
        self.intern(entry)
    }

    fn class(&mut self, name: &[u8]) -> u16 {
        let name = self.utf8(name);
        let mut entry = vec![7_u8];
        u16b(&mut entry, name);
        self.intern(entry)
    }
}

struct MethodSpec<'a> {
    flags: u16,
    name: &'a [u8],
    descriptor: &'a [u8],
    code: Option<&'a [u8]>,
}

/// One class file, written the way the reader reads it.
fn class_file(
    this_class: &[u8],
    super_class: &[u8],
    flags: u16,
    methods: &[MethodSpec<'_>],
) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_index = pool.class(this_class);
    let super_index = pool.class(super_class);
    let code_name = pool.utf8(b"Code");
    let method_indices: Vec<(u16, u16)> = methods
        .iter()
        .map(|method| (pool.utf8(method.name), pool.utf8(method.descriptor)))
        .collect();

    let mut output = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0);
    u16b(&mut output, 52);
    u16b(
        &mut output,
        u16::try_from(pool.entries.len() + 1).expect("the fixture pool fits u16"),
    );
    for entry in &pool.entries {
        output.extend_from_slice(entry);
    }
    u16b(&mut output, flags);
    u16b(&mut output, this_index);
    u16b(&mut output, super_index);
    u16b(&mut output, 0);
    u16b(&mut output, 0);
    u16b(
        &mut output,
        u16::try_from(methods.len()).expect("the fixture methods fit u16"),
    );
    for (index, method) in methods.iter().enumerate() {
        u16b(&mut output, method.flags);
        u16b(&mut output, method_indices[index].0);
        u16b(&mut output, method_indices[index].1);
        let Some(code) = method.code else {
            u16b(&mut output, 0);
            continue;
        };
        let mut body = Vec::new();
        u16b(&mut body, 2);
        u16b(&mut body, 4);
        u32b(
            &mut body,
            u32::try_from(code.len()).expect("the fixture body fits u32"),
        );
        body.extend_from_slice(code);
        u16b(&mut body, 0);
        u16b(&mut body, 0);
        u16b(&mut output, 1);
        u16b(&mut output, code_name);
        u32b(
            &mut output,
            u32::try_from(body.len()).expect("the fixture attribute fits u32"),
        );
        output.extend_from_slice(&body);
    }
    u16b(&mut output, 0);
    output
}

/// `iconst_1; pop; return`: a body whose recovery produces a statement.
const PLAIN_BODY: &[u8] = &[0x04, 0x57, 0xb1];

/// `new` with an incomplete index: a body whose own decode stops inside it, which is what makes that
/// member's run stop while the members beside it are presented as usual.
const DAMAGED_BODY: &[u8] = &[0xbb, 0x00];

const CLASS_FLAGS: u16 = 0x0021;
const PUBLIC_STATIC_METHOD: u16 = 0x0009;
const PUBLIC_METHOD: u16 = 0x0001;
const PUBLIC_ABSTRACT_METHOD: u16 = 0x0401;
const PUBLIC_NATIVE_METHOD: u16 = 0x0101;

/// `p/Probe`: a class with every shape the text has to state — a body it recovers, a body whose run
/// stops, a member with no `Code` for each of the three ways that can be said.
fn probe_class() -> Vec<u8> {
    class_file(
        b"p/Probe",
        b"java/lang/Object",
        CLASS_FLAGS,
        &[
            MethodSpec {
                flags: PUBLIC_STATIC_METHOD,
                name: b"good",
                descriptor: b"()V",
                code: Some(PLAIN_BODY),
            },
            MethodSpec {
                flags: PUBLIC_STATIC_METHOD,
                name: b"broken",
                descriptor: b"()V",
                code: Some(DAMAGED_BODY),
            },
            MethodSpec {
                flags: PUBLIC_ABSTRACT_METHOD,
                name: b"abstractOne",
                descriptor: b"()V",
                code: None,
            },
            MethodSpec {
                flags: PUBLIC_NATIVE_METHOD,
                name: b"nativeOne",
                descriptor: b"()V",
                code: None,
            },
            MethodSpec {
                flags: PUBLIC_METHOD,
                name: b"contradictory",
                descriptor: b"()V",
                code: None,
            },
        ],
    )
}

// ---------------------------------------------------------------------------------------------
// The two sides of every comparison: the CLI as a process, the library in process
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

fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout_json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "standard output is not one JSON document ({error}): {}",
            stdout_text(output)
        )
    })
}

fn repository_fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn path_of(path: &Path) -> &str {
    path.to_str().expect("the fixture path is UTF-8")
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

/// The library's own report for the request the CLI is expected to build: one snapshot, one scope,
/// the same policy — called in process, so the comparison is a value rather than a claim.
fn library_report(path: &Path, class: ClassRef, policy: EnvironmentPolicy) -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("the task defaults are a bounded budget");
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::Path(path.to_path_buf()), &mut budget)
        .expect("the fixture opens");
    let request = ClassSourceRequest {
        class,
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    // The full-evidence selection, stated on both sides of the comparison: the document cases in
    // this file read the member reports' own records, and a category nobody selected is a category
    // the library does not materialize (`add-demand-driven-core-results`, D1).
    match engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &jarde::RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("a legal request is answered")
    {
        jarde::OperationOutcome::Performed(report) => report,
        other => panic!("expected one bound class, got {other:?}"),
    }
}

/// Every marker one text carries, without the leading indentation.
///
/// The presentation's own two-line prologue is a comment block like any other, and it is skipped
/// here: what a case is about is the markers *inside* the class.
fn markers(text: &str) -> Vec<&str> {
    text.lines()
        .skip_while(|line| line.trim_start().starts_with("//"))
        .map(str::trim_start)
        .filter(|line| line.starts_with("// jarde:"))
        .collect()
}

/// The brace balance of a text, ignoring comment lines (a fallback's reason may quote anything).
fn braces_balance(text: &str) -> i64 {
    let mut balance = 0_i64;
    for line in text.lines() {
        let line = line.trim_start();
        if line.starts_with("//") {
            continue;
        }
        for character in line.chars() {
            match character {
                '{' => balance += 1,
                '}' => balance -= 1,
                _ => {}
            }
        }
        assert!(
            balance >= 0,
            "a closing brace with nothing open in:\n{text}"
        );
    }
    balance
}

/// Whether every line of a text starts at a multiple of four columns.
fn indentation_is_four_spaces(text: &str) -> bool {
    text.lines().all(|line| {
        let leading = line.len() - line.trim_start_matches(' ').len();
        leading % 4 == 0
    })
}

// ---------------------------------------------------------------------------------------------
// The text mode is the library's own text
// ---------------------------------------------------------------------------------------------

/// The shortcut's main path: standard output is the assembled Java source, byte for byte, and the
/// report's bookkeeping planes go to standard error.
#[test]
fn the_text_mode_writes_the_librarys_own_source() {
    let fixture = repository_fixture(HISTORICAL);
    let output = run(&[
        "class-source",
        "--evidence",
        "all",
        "--input",
        path_of(&fixture),
        "--policy",
        "single-class",
        "--class",
        "HistoricalControlFlow",
    ]);
    assert_eq!(status(&output), EXIT_COMPLETE, "{}", stderr_text(&output));
    let text = stdout_text(&output);
    assert_eq!(
        text,
        library_report(
            &fixture,
            ClassRef::Name {
                class: jarde::ClassNameQuery::dotted("HistoricalControlFlow"),
            },
            EnvironmentPolicy::SingleClass,
        )
        .text,
        "standard output is the report's own text field"
    );

    // A reader's first glance: the declaration, one member's body, and nothing that is not Java
    // structure at four-column indentation.
    assert!(text.contains("public class HistoricalControlFlow extends java.lang.Object {\n"));
    assert!(text.contains("    public int add(int arg1, int arg2) {\n"));
    assert!(text.contains("        return arg1 + arg2;\n"));
    assert!(braces_balance(&text) == 0, "{text}");
    assert!(indentation_is_four_spaces(&text), "{text}");
    // This sample's third member produced an explanation-only artifact: its marker is the one place
    // the text says so, and the artifact's own comment lines are kept under it.
    assert_eq!(
        markers(&text),
        [
            "// jarde: not recovered: the recovery run for `finallyPath(I)I` produced no statement \
             (explanation only); the artifact's own comment lines are below"
        ]
    );

    // Everything the document holds beside the text is on standard error, as `path = value` lines:
    // the planes as their own fields, and every other field of the report — a member's own text
    // among them — as the JSON value the document holds, so a member's text cannot end up in the
    // class text twice.
    let bookkeeping = stderr_text(&output);
    // **One** class read whatever the member count — the binding read, over which the one
    // preparation the member bodies are decoded against is built (`add-demand-driven-core-results`
    // task 3.2: the selected definition is materialized once, and a presentation is not a reason for
    // a second read) — and one body attempt per member that declares one. The three members of this
    // sample are three bodies, so a per-member class read would say 4 here, and a second read for
    // the preparation would say 2.
    assert!(
        bookkeeping.contains("usage.class_headers = 1"),
        "one materialization, one preparation: {bookkeeping}"
    );
    assert!(
        bookkeeping.contains("usage.method_bodies = 3"),
        "one body attempt per member: {bookkeeping}"
    );
    // The class's bytes are *parsed* twice — the binding's own member walk and the one preparation
    // built over that same read — which is two parses of one read, not two reads.
    assert!(
        bookkeeping.contains("usage.class_bytes = 606"),
        "one read, parsed for its members and once more for the preparation: {bookkeeping}"
    );
    assert!(bookkeeping.contains("execution.status = \"complete\""));
    assert!(
        bookkeeping
            .contains("coverage.artifact_structural.scanned.2.label = \"class_source_bodies\""),
        "the plane this operation adds is in the bookkeeping: {bookkeeping}"
    );
    assert!(
        bookkeeping.contains("declaration.declaration = \"public class HistoricalControlFlow"),
        "{bookkeeping}"
    );
    assert!(
        bookkeeping.contains("methods.1.text = \"    public int add(int arg1, int arg2) {\\n"),
        "a member's own text is projected as the JSON string the document holds: {bookkeeping}"
    );
}

/// The same command twice writes the same bytes, so the shortcut is a pure function of the artifact,
/// the parameters and the limits.
#[test]
fn the_same_command_twice_writes_the_same_bytes() {
    let fixture = repository_fixture(HISTORICAL);
    let args = [
        "class-source",
        "--evidence",
        "all",
        "--input",
        path_of(&fixture),
        "--policy",
        "single-class",
        "--class",
        "HistoricalControlFlow",
    ];
    let first = run(&args);
    let second = run(&args);
    assert_eq!(status(&first), EXIT_COMPLETE);
    assert_eq!(first.stdout, second.stdout);
    // The bookkeeping may differ in the elapsed clock alone, and that clock is one line of the
    // `path = value` rendering: every other line has to be the same one.
    let without_elapsed = |output: &Output| -> Vec<String> {
        stderr_text(output)
            .lines()
            .filter(|line| !line.contains("elapsed_millis"))
            .map(str::to_owned)
            .collect()
    };
    assert_eq!(without_elapsed(&first), without_elapsed(&second));
}

/// `--output` writes exactly the bytes standard output would have received.
#[test]
fn the_output_file_receives_the_same_document_as_standard_output() {
    let fixture = repository_fixture(HISTORICAL);
    let directory = TempDir::new();
    let target = directory.path.join("HistoricalControlFlow.java");
    let output = run(&[
        "class-source",
        "--evidence",
        "all",
        "--input",
        path_of(&fixture),
        "--policy",
        "single-class",
        "--class",
        "HistoricalControlFlow",
        "--output",
        path_of(&target),
    ]);
    assert_eq!(status(&output), EXIT_COMPLETE, "{}", stderr_text(&output));
    assert!(output.stdout.is_empty(), "the document went to the file");
    let written = fs::read(&target).expect("the output file is readable");
    assert_eq!(
        String::from_utf8_lossy(&written),
        library_report(
            &fixture,
            ClassRef::Name {
                class: jarde::ClassNameQuery::dotted("HistoricalControlFlow"),
            },
            EnvironmentPolicy::SingleClass,
        )
        .text
    );
}

// ---------------------------------------------------------------------------------------------
// The JSON mode is the library's own document
// ---------------------------------------------------------------------------------------------

/// `--format json` writes the library's own serialization, field by field, including every member's
/// own recovery report.
#[test]
fn the_json_mode_is_the_librarys_own_document() {
    let bytes = probe_class();
    let directory = TempDir::new();
    let fixture = directory.write("Probe.class", &bytes);
    let output = run(&[
        "class-source",
        "--evidence",
        "all",
        "--input",
        path_of(&fixture),
        "--policy",
        "single-class",
        "--class",
        "p/Probe",
        "--format",
        "json",
    ]);
    assert_eq!(status(&output), EXIT_INCOMPLETE, "{}", stderr_text(&output));
    let document = stdout_json(&output);
    let expected = library_report(
        &fixture,
        ClassRef::Name {
            class: jarde::ClassNameQuery::internal("p/Probe"),
        },
        EnvironmentPolicy::SingleClass,
    );
    let library =
        strip_elapsed(&serde_json::to_value(&expected).expect("the library value serializes"));
    // An internally tagged outcome flattens a performed report into the document itself, so the
    // document is the report *plus* its one `outcome` tag.
    let mut report = document.clone();
    let tag = report
        .as_object_mut()
        .expect("the document is an object")
        .remove("outcome");
    assert_eq!(tag, Some(json!("performed")));
    assert_eq!(strip_elapsed(&report), library);
    assert_eq!(
        strip_elapsed(&serde_json::to_value(&expected).expect("serializes"))["text"],
        document["text"]
    );

    // The document really holds what the text says, member by member.
    assert_eq!(document["declaration"]["name"], json!("p.Probe"));
    let names: Vec<&str> = document["methods"]
        .as_array()
        .expect("the members are an array")
        .iter()
        .map(|method| {
            method["item"]["name"]["escaped"]
                .as_str()
                .expect("a name is a string")
        })
        .collect();
    assert_eq!(
        names,
        [
            "good",
            "broken",
            "abstractOne",
            "nativeOne",
            "contradictory"
        ]
    );
    assert_eq!(
        document["methods"][1]["outcome"]["report"]["content"],
        json!("not_produced")
    );
    assert_eq!(document["methods"][2]["no_body_kind"], json!("abstract"));
    assert_eq!(document["methods"][2]["outcome"]["kind"], json!("no_body"));
    // The member's own recovery report is in the document, segment table and all: the text of the
    // body is a field of the very run that produced it.
    assert_eq!(
        document["methods"][0]["outcome"]["report"]["outcome"],
        json!("produced")
    );
    assert!(
        document["methods"][0]["outcome"]["report"]["source_map"]["segments"]
            .as_array()
            .is_some_and(|segments| !segments.is_empty()),
        "{document}"
    );
}

// ---------------------------------------------------------------------------------------------
// Nothing is disguised, and a member's failure is that member's
// ---------------------------------------------------------------------------------------------

/// Every member appears in the text, and the ones that are not full recoveries say so: a member with
/// no `Code` is a declaration, a member whose run stopped keeps its declaration and a marker, and the
/// class report is not `Complete`.
#[test]
fn a_member_without_a_body_and_a_stopped_member_are_marked_in_the_text() {
    let bytes = probe_class();
    let directory = TempDir::new();
    let fixture = directory.write("Probe.class", &bytes);
    let output = run(&[
        "class-source",
        "--evidence",
        "all",
        "--input",
        path_of(&fixture),
        "--policy",
        "single-class",
        "--class",
        "p/Probe",
    ]);
    assert_eq!(
        status(&output),
        EXIT_INCOMPLETE,
        "a member stopped, so the class is not a complete presentation: {}",
        stderr_text(&output)
    );
    let text = stdout_text(&output);

    // No member of the class's own table is dropped.
    for name in [
        "good",
        "broken",
        "abstractOne",
        "nativeOne",
        "contradictory",
    ] {
        assert!(text.contains(name), "`{name}` is not in:\n{text}");
    }
    let carried = markers(&text);
    assert!(
        carried.iter().any(|marker| marker.contains("no body")
            && marker.contains("abstractOne()V")
            && marker.contains("abstract")),
        "{carried:?}"
    );
    assert!(
        carried.iter().any(|marker| marker.contains("no body")
            && marker.contains("nativeOne()V")
            && marker.contains("native")),
        "{carried:?}"
    );
    assert!(
        carried.iter().any(|marker| marker.contains("no body")
            && marker.contains("contradictory()V")
            && marker.contains("neither abstract nor native")),
        "{carried:?}"
    );
    assert!(
        carried.iter().any(|marker| marker.contains("not recovered")
            && marker.contains("broken()V")
            && marker.contains("stopped")),
        "{carried:?}"
    );
    // The member with no body is a declaration Java writes without one; the stopped member keeps a
    // block whose whole content is the marker, so neither is an empty body.
    assert!(
        text.contains("    public abstract void abstractOne();\n"),
        "{text}"
    );
    assert!(
        text.contains(
            "    public static void broken() {\n        // jarde: not recovered: the recovery run \
             for `broken()V` stopped"
        ),
        "{text}"
    );
    // And the member whose run really completed keeps its body.
    assert!(
        text.contains(
            "        // recovered from bytecode; presentation is not claimed to compile\n"
        ),
        "{text}"
    );
    assert!(braces_balance(&text) == 0, "{text}");
    assert!(indentation_is_four_spaces(&text), "{text}");
    assert!(
        stderr_text(&output).contains("execution.status = \"partial\""),
        "{}",
        stderr_text(&output)
    );
}

/// A member whose descriptor is not a method descriptor is stated as such in the text and in the
/// document — and the member beside it is presented as usual.
#[test]
fn a_member_that_cannot_be_spelled_is_stated_in_the_text() {
    let bytes = class_file(
        b"p/Odd",
        b"java/lang/Object",
        CLASS_FLAGS,
        &[
            MethodSpec {
                flags: PUBLIC_METHOD,
                name: b"fine",
                descriptor: b"()V",
                code: Some(PLAIN_BODY),
            },
            MethodSpec {
                flags: PUBLIC_METHOD,
                name: b"odd",
                descriptor: b"not-a-descriptor",
                code: Some(PLAIN_BODY),
            },
        ],
    );
    let directory = TempDir::new();
    let fixture = directory.write("Odd.class", &bytes);
    let output = run(&[
        "class-source",
        "--evidence",
        "all",
        "--input",
        path_of(&fixture),
        "--policy",
        "single-class",
        "--class",
        "p/Odd",
    ]);
    assert_eq!(status(&output), EXIT_COMPLETE, "{}", stderr_text(&output));
    let text = stdout_text(&output);
    assert!(
        text.contains("// jarde: not spelled: the descriptor `not-a-descriptor`"),
        "{text}"
    );
    assert!(text.contains("    public void fine() {\n"), "{text}");
    // The member that cannot be spelled was not run: only the other one's body was read.
    let document = stdout_json(&run(&[
        "class-source",
        "--evidence",
        "all",
        "--input",
        path_of(&fixture),
        "--policy",
        "single-class",
        "--class",
        "p/Odd",
        "--format",
        "json",
    ]));
    assert_eq!(document["usage"]["method_bodies"], json!(1));
    assert_eq!(
        document["methods"][1]["outcome"]["kind"],
        json!("unspelled")
    );
}

// ---------------------------------------------------------------------------------------------
// The exit status is the report's own
// ---------------------------------------------------------------------------------------------

/// A name that two definitions answer to is answered with both candidates and presents nothing.
#[test]
fn a_name_two_definitions_answer_to_exits_three() {
    let bytes = probe_class();
    let archive = zip(&[
        Stored {
            name: b"p/Probe.class",
            data: &bytes,
        },
        Stored {
            name: b"WEB-INF/classes/p/Probe.class",
            data: &bytes,
        },
        Stored {
            name: b"META-INF/MANIFEST.MF",
            data: b"Manifest-Version: 1.0\n",
        },
    ]);
    let directory = TempDir::new();
    let fixture = directory.write("app.jar", &archive);
    let output = run(&[
        "class-source",
        "--evidence",
        "all",
        "--input",
        path_of(&fixture),
        "--policy",
        "plain-jar",
        "--class",
        "p/Probe",
        "--format",
        "json",
    ]);
    assert_eq!(status(&output), EXIT_AMBIGUOUS, "{}", stderr_text(&output));
    let document = stdout_json(&output);
    assert_eq!(document["outcome"], json!("ambiguous"));
    let candidates = document["candidates"]
        .as_array()
        .expect("the candidates are an array");
    assert_eq!(candidates.len(), 2);
    assert!(
        candidates
            .iter()
            .all(|candidate| candidate["kind"] == json!("class_declaration")),
        "{document}"
    );
    assert!(
        document.get("text").is_none(),
        "nothing was presented: {document}"
    );

    // One candidate's own identity is directly usable, and exits for its own reasons.
    let definition = candidates[0]["definition"].clone();
    let argument = serde_json::to_string(&definition).expect("the identity serializes");
    let output = run(&[
        "class-source",
        "--evidence",
        "all",
        "--input",
        path_of(&fixture),
        "--policy",
        "plain-jar",
        "--class",
        &argument,
    ]);
    assert_eq!(status(&output), EXIT_INCOMPLETE, "{}", stderr_text(&output));
    assert_eq!(
        stdout_text(&output),
        library_report(
            &fixture,
            ClassRef::Definition {
                definition: serde_json::from_value(definition).expect("the identity reads back"),
            },
            EnvironmentPolicy::PlainJar,
        )
        .text
    );
}

/// A definition of another artifact is refused as the usage error it is, never replaced by a
/// same-named class of the input at hand.
#[test]
fn a_definition_of_another_artifact_is_a_usage_error() {
    let directory = TempDir::new();
    let fixture = directory.write("Probe.class", &probe_class());
    // An identity of the same class read from another snapshot: the digest of a different file is
    // what makes it foreign.
    let other = directory.write(
        "Other.class",
        &class_file(b"p/Other", b"java/lang/Object", CLASS_FLAGS, &[]),
    );
    let foreign = library_report(
        &other,
        ClassRef::Name {
            class: jarde::ClassNameQuery::internal("p/Other"),
        },
        EnvironmentPolicy::SingleClass,
    )
    .class;
    let argument = serde_json::to_string(&foreign).expect("the identity serializes");
    let output = run(&[
        "class-source",
        "--evidence",
        "all",
        "--input",
        path_of(&fixture),
        "--policy",
        "single-class",
        "--class",
        &argument,
        "--format",
        "json",
    ]);
    assert_eq!(status(&output), EXIT_USAGE, "{}", stderr_text(&output));
    assert!(
        stderr_text(&output).contains("operation_target_snapshot_mismatch"),
        "{}",
        stderr_text(&output)
    );
}

/// The environment is the caller's declaration: the default `plain-jar` policy refuses a standalone
/// class file instead of inventing a root for it.
#[test]
fn a_policy_that_does_not_fit_the_input_is_a_usage_error() {
    let directory = TempDir::new();
    let fixture = directory.write("Probe.class", &probe_class());
    let output = run(&[
        "class-source",
        "--evidence",
        "all",
        "--input",
        path_of(&fixture),
        "--class",
        "p/Probe",
    ]);
    assert_eq!(status(&output), EXIT_USAGE, "{}", stderr_text(&output));
    assert!(
        stderr_text(&output).contains("environment_policy_snapshot_kind_mismatch"),
        "{}",
        stderr_text(&output)
    );
}

/// No `--class` at all is a usage error the parser refuses.
#[test]
fn a_missing_class_parameter_is_a_usage_error() {
    let fixture = repository_fixture(HISTORICAL);
    let output = run(&[
        "class-source",
        "--evidence",
        "all",
        "--input",
        path_of(&fixture),
        "--policy",
        "single-class",
    ]);
    assert_eq!(status(&output), EXIT_USAGE);
}

/// Standard input is never read: every parameter of this command is a value or a document.
#[test]
fn the_command_reads_no_standard_input() {
    let fixture = repository_fixture(HISTORICAL);
    let mut child = Command::new(BIN)
        .args([
            "class-source",
            "--input",
            path_of(&fixture),
            "--policy",
            "single-class",
            "--class",
            "HistoricalControlFlow",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn jarde-cli");
    child
        .stdin
        .take()
        .expect("the child has a standard input")
        .write_all(b"{\"unread\": true}\n")
        .expect("write to the child's standard input");
    let output = child.wait_with_output().expect("the child exits");
    assert_eq!(status(&output), EXIT_COMPLETE, "{}", stderr_text(&output));
    assert!(stdout_text(&output).contains("public class HistoricalControlFlow"));
}

/// The budget parameters are the library's own: an override this command cannot parse, or a
/// dimension the task requests do not serve, is a usage error.
#[test]
fn a_budget_override_this_command_cannot_read_is_a_usage_error() {
    let fixture = repository_fixture(HISTORICAL);
    for override_value in ["result_items", "result_items=0", "nonsense=1"] {
        let output = run(&[
            "class-source",
            "--input",
            path_of(&fixture),
            "--policy",
            "single-class",
            "--class",
            "HistoricalControlFlow",
            "--budget",
            override_value,
        ]);
        assert_eq!(
            status(&output),
            EXIT_USAGE,
            "`{override_value}` was accepted: {}",
            stderr_text(&output)
        );
    }
}

/// The `--class` parameter tells a name from an identity document by its first byte, so a JSON
/// document that is not a definition is refused rather than read as a name.
#[test]
fn a_class_parameter_that_is_not_a_definition_is_a_usage_error() {
    let fixture = repository_fixture(HISTORICAL);
    let output = run(&[
        "class-source",
        "--evidence",
        "all",
        "--input",
        path_of(&fixture),
        "--policy",
        "single-class",
        "--class",
        "{\"not\": \"a definition\"}",
    ]);
    assert_eq!(status(&output), EXIT_USAGE, "{}", stderr_text(&output));
    assert!(
        stderr_text(&output).contains("cli_definition_json"),
        "{}",
        stderr_text(&output)
    );
}

/// An invocation that states no evidence asks for the ordinary recovery: the necessary results and
/// no optional record, on both sides of the adapter.
///
/// This is the adapter's half of "an ordinary recovery defaults to Essential": the CLI invents no
/// default of its own — the selection it passes is the one it read — so the document it writes and
/// the library report of the same request agree about every category being `NotRequested` and
/// about the segment table being absent (change `add-demand-driven-core-results`, D1).
#[test]
fn an_invocation_that_states_no_evidence_asks_for_none() {
    let bytes = probe_class();
    let directory = TempDir::new();
    let fixture = directory.write("Probe.class", &bytes);
    let output = run(&[
        "class-source",
        "--input",
        path_of(&fixture),
        "--policy",
        "single-class",
        "--class",
        "p/Probe",
        "--format",
        "json",
    ]);
    assert_eq!(status(&output), EXIT_INCOMPLETE, "{}", stderr_text(&output));
    let document = stdout_json(&output);
    let methods = document["methods"]
        .as_array()
        .expect("the class has members");
    let mut recovered = 0usize;
    for method in methods {
        let Some(report) = method["outcome"].get("report") else {
            continue;
        };
        if report.is_null() {
            continue;
        }
        recovered += 1;
        assert!(
            report["source_map"]["segments"]
                .as_array()
                .is_some_and(|segments| segments.is_empty()),
            "no segment table was asked for: {report}"
        );
        let categories = report["evidence"]["categories"]
            .as_array()
            .expect("the status list is fixed-size");
        assert_eq!(categories.len(), 5, "one entry per category: {report}");
        for category in categories {
            assert_eq!(
                category["state"]["state"], "not_requested",
                "an unstated category is not requested: {report}"
            );
        }
        assert_eq!(
            report["evidence"]["requested"]["kinds"],
            json!([]),
            "and the report echoes the empty selection"
        );
    }
    assert!(
        recovered > 0,
        "the fixture's members really ran: {document}"
    );
}

/// A spelling this vocabulary does not hold is a usage error, never an ignored word.
#[test]
fn an_unknown_evidence_category_is_a_usage_error() {
    let fixture = repository_fixture(HISTORICAL);
    let output = run(&[
        "class-source",
        "--evidence",
        "region_detail",
        "--input",
        path_of(&fixture),
        "--policy",
        "single-class",
        "--class",
        "HistoricalControlFlow",
    ]);
    assert_eq!(status(&output), EXIT_USAGE, "{}", stderr_text(&output));
    assert!(
        stderr_text(&output).contains("cli_evidence_kind"),
        "{}",
        stderr_text(&output)
    );
}

/// A category this layer's vocabulary holds but this entry does not materialize is refused by the
/// library, with the library's own code — not answered with an empty delivery.
#[test]
fn a_category_the_entry_does_not_materialize_is_refused_by_the_library() {
    let bytes = probe_class();
    let directory = TempDir::new();
    let fixture = directory.write("Probe.class", &bytes);
    let output = run(&[
        "class-source",
        "--evidence",
        "read_details",
        "--input",
        path_of(&fixture),
        "--policy",
        "single-class",
        "--class",
        "p/Probe",
        "--format",
        "json",
    ]);
    assert_eq!(status(&output), EXIT_INCOMPLETE, "{}", stderr_text(&output));
    let document = stdout_json(&output);
    let methods = document["methods"]
        .as_array()
        .expect("the class has members");
    let mut refusals = 0usize;
    for method in methods {
        let Some(report) = method["outcome"].get("report") else {
            continue;
        };
        if report.is_null() {
            continue;
        }
        // Every member states a stop: the refusal of the selection where the run's own analysis
        // reached the presentation, and the analysis's own missing table where it did not (the
        // refused selection is stated before anything is presented, so a member whose analysis
        // stopped first states that instead — both are stops, and neither is an empty delivery).
        let stopped = report["outcome"]
            .get("stopped")
            .unwrap_or_else(|| panic!("{report}"));
        let code = stopped
            .get("evidence_refused")
            .map(|refusal| refusal["code"].clone())
            .or_else(|| {
                stopped
                    .get("ir_table_missing")
                    .map(|_| json!("jre_ir_table_missing"))
            })
            .unwrap_or_else(|| panic!("a stated stop: {report}"));
        if code == json!("jre_evidence_kind_unsupported") {
            refusals += 1;
        }
        assert_eq!(report["text"], json!(""));
        for category in report["evidence"]["categories"]
            .as_array()
            .expect("the status list is fixed-size")
        {
            if category["kind"] == "read_details" {
                assert_eq!(category["state"]["state"], "not_performed", "{report}");
            }
        }
    }
    assert!(
        refusals > 0,
        "the members whose analysis reached the presentation state the refusal: {document}"
    );
}
