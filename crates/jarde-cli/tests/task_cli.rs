//! `add-task-oriented-cli`: the task commands, their two renderings, their exit statuses and the
//! chain they close.
//!
//! Every archive here is built in memory by this file, so a case can pin the exact identity, read
//! count, stream and status it is about. The CLI is always driven as a process, because the exit
//! status and the two standard streams are part of what is under test; the library is called in
//! process for the *same* request wherever a document is compared, so "the CLI's document is the
//! library's own" is a field-by-field equality rather than a claim about it.
//!
//! The fixture is deliberately shaped like the chain it must serve, and every shape is used:
//!
//! * `p/Base.class` declares one field and three methods — `foo()V` (the recovered body),
//!   `foo(I)V` (the same name at another descriptor) and `bar()V` — so a body read can be told
//!   apart from a class-wide scan and an overload from an elected one.
//! * `WEB-INF/classes/p/Base.class` holds the same bytes at a second origin, which is what makes
//!   the friendly name ambiguous and the identity necessary.
//! * `p/Caller.class` mentions `p/Base.foo()V` from a real call site, which is what the reference
//!   scan and its grouping are read from.
//! * `WEB-INF/lib/L.jar` holds the same two classes *inside a nested container*, so the ordinary
//!   scope and the caller-declared tree scope must answer differently; `WEB-INF/lib/Broken.jar`
//!   beside it is a child that cannot be established, which the library answers with a prefix and
//!   a diagnostic — and this adapter must publish exactly that.

use jarde::{
    ArtifactInput, ArtifactSnapshot, BodyRef, Budget, BudgetOverride, ClassNameQuery, ClassRef,
    ClassViewRequest, ConsumerKind, ConsumerSchema, Engine, EnvironmentPolicy, EnvironmentRequest,
    LayoutMode, Limits, LoadRoot, LoaderId, MethodOperationRequest, MethodRef, MultiReleasePolicy,
    PhysicalDefinitionId, PhysicalMethodId, PhysicalScope, PhysicalView, QueryRelation,
    QueryRequest, QueryTarget, ReferenceGrouping, RuntimeProfile, SymbolRef, task_budget,
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
const MAX_REQUEST_BYTES: usize = 1024 * 1024;

/// The exit statuses the task commands promise, spelled here so a renamed constant in the crate is
/// a compile error in this file rather than a silently unchecked number.
const EXIT_COMPLETE: i32 = 0;
const EXIT_DELIVERY: i32 = 1;
const EXIT_USAGE: i32 = 2;
const EXIT_AMBIGUOUS: i32 = 3;
const EXIT_INCOMPLETE: i32 = 4;

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
            "jarde-cli-task-{}-{nonce}-{}",
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
// Fixtures: a stored-only ZIP writer and a class-file writer
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

/// One stored entry of an in-memory archive.
struct Stored<'a> {
    name: &'a [u8],
    data: &'a [u8],
}

/// A minimal STORED-only ZIP writer: the CLI test package declares no archive dependency, so the
/// few dozen bytes of the format these fixtures need are written here.
fn zip(entries: &[Stored<'_>]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut directory = Vec::new();
    for entry in entries {
        let offset = u32::try_from(output.len()).expect("fixture archive fits u32");
        let crc = crc32(entry.data);
        let size = u32::try_from(entry.data.len()).expect("fixture entry fits u32");
        let name_len = u16::try_from(entry.name.len()).expect("fixture name fits u16");
        output.extend_from_slice(&0x0403_4b50_u32.to_le_bytes());
        output.extend_from_slice(&20_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes());
        output.extend_from_slice(&0_u16.to_le_bytes()); // method: stored
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
    let directory_offset = u32::try_from(output.len()).expect("fixture archive fits u32");
    let directory_size = u32::try_from(directory.len()).expect("fixture directory fits u32");
    let count = u16::try_from(entries.len()).expect("fixture entry count fits u16");
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
            return u16::try_from(index + 1).expect("pool index fits u16");
        }
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("pool index fits u16")
    }

    fn utf8(&mut self, text: &[u8]) -> u16 {
        let mut entry = vec![1_u8];
        u16b(
            &mut entry,
            u16::try_from(text.len()).expect("fixture name fits u16"),
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

    fn name_and_type(&mut self, name: &[u8], descriptor: &[u8]) -> u16 {
        let name = self.utf8(name);
        let descriptor = self.utf8(descriptor);
        let mut entry = vec![12_u8];
        u16b(&mut entry, name);
        u16b(&mut entry, descriptor);
        self.intern(entry)
    }

    /// A `CONSTANT_Methodref` to a *static* method, so a call site needs no receiver.
    fn method_ref(&mut self, owner: &[u8], name: &[u8], descriptor: &[u8]) -> u16 {
        let class = self.class(owner);
        let name_and_type = self.name_and_type(name, descriptor);
        let mut entry = vec![10_u8];
        u16b(&mut entry, class);
        u16b(&mut entry, name_and_type);
        self.intern(entry)
    }
}

/// The body one fixture method declares.
enum Body<'a> {
    /// The instruction bytes, with the slots they need.
    Instructions {
        bytes: &'a [u8],
        max_stack: u16,
        max_locals: u16,
    },
    /// A call site `invokestatic owner.name:descriptor` followed by `return`: the shape a
    /// reference scan finds as an invocation.
    StaticCall {
        owner: &'a [u8],
        name: &'a [u8],
        descriptor: &'a [u8],
    },
    /// No `Code` attribute at all, which is how `abstract` and `native` are stated.
    NoCode,
}

struct MethodSpec<'a> {
    name: &'a [u8],
    descriptor: &'a [u8],
    flags: u16,
    body: Body<'a>,
}

/// One class file, written the way the reader reads it.
///
/// Every constant-pool entry is interned *before* the pool is written — the field's own names, the
/// members' names and descriptors, the `Code` attribute's name and any call site's reference — so
/// no index can point past the pool that was serialized.
fn class_file(
    this_class: &[u8],
    super_class: &[u8],
    field: Option<(&[u8], &[u8], u16)>,
    methods: &[MethodSpec<'_>],
) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_class_index = pool.class(this_class);
    let super_class_index = pool.class(super_class);
    let field_indices = field.map(|(name, descriptor, _)| (pool.utf8(name), pool.utf8(descriptor)));
    let method_indices: Vec<(u16, u16)> = methods
        .iter()
        .map(|method| (pool.utf8(method.name), pool.utf8(method.descriptor)))
        .collect();
    let code_attribute = pool.utf8(b"Code");
    let codes: Vec<Option<(Vec<u8>, u16, u16)>> = methods
        .iter()
        .map(|method| match &method.body {
            Body::Instructions {
                bytes,
                max_stack,
                max_locals,
            } => Some((bytes.to_vec(), *max_stack, *max_locals)),
            Body::StaticCall {
                owner,
                name,
                descriptor,
            } => {
                let reference = pool.method_ref(owner, name, descriptor);
                let mut code = Vec::new();
                code.push(0xb8);
                u16b(&mut code, reference);
                code.push(0xb1);
                Some((code, 0, 0))
            }
            Body::NoCode => None,
        })
        .collect();

    let mut output = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0);
    u16b(&mut output, 52);
    u16b(
        &mut output,
        u16::try_from(pool.entries.len() + 1).expect("fixture pool fits u16"),
    );
    for entry in &pool.entries {
        output.extend_from_slice(entry);
    }
    u16b(&mut output, 0x0021);
    u16b(&mut output, this_class_index);
    u16b(&mut output, super_class_index);
    u16b(&mut output, 0); // interfaces
    match (field, field_indices) {
        (Some((_, _, flags)), Some((name, descriptor))) => {
            u16b(&mut output, 1);
            u16b(&mut output, flags);
            u16b(&mut output, name);
            u16b(&mut output, descriptor);
            u16b(&mut output, 0);
        }
        _ => u16b(&mut output, 0),
    }
    u16b(
        &mut output,
        u16::try_from(methods.len()).expect("fixture method count fits u16"),
    );
    for (index, method) in methods.iter().enumerate() {
        u16b(&mut output, method.flags);
        u16b(&mut output, method_indices[index].0);
        u16b(&mut output, method_indices[index].1);
        match &codes[index] {
            Some((code, max_stack, max_locals)) => {
                u16b(&mut output, 1);
                u16b(&mut output, code_attribute);
                let mut attribute = Vec::new();
                u16b(&mut attribute, *max_stack);
                u16b(&mut attribute, *max_locals);
                u32b(
                    &mut attribute,
                    u32::try_from(code.len()).expect("fixture body fits u32"),
                );
                attribute.extend_from_slice(code);
                u16b(&mut attribute, 0); // exception table
                u16b(&mut attribute, 0); // code attributes
                u32b(
                    &mut output,
                    u32::try_from(attribute.len()).expect("fixture attribute fits u32"),
                );
                output.extend_from_slice(&attribute);
            }
            None => u16b(&mut output, 0),
        }
    }
    u16b(&mut output, 0); // class attributes
    output
}

/// The recovered body: `if (local1 != 0) { return; } else { return; }`, written the way a compiler
/// writes it. Eight bytes, two slots — the same shape the other CLI suites present.
const RECOVERY_BODY: &[u8] = &[
    0x03, // 0: iconst_0
    0x3c, // 1: istore_1
    0x1b, // 2: iload_1
    0x99, 0x00, 0x04, // 3: ifeq 7
    0xb1, // 6: return
    0xb1, // 7: return
];

/// The body the *unrequested* method declares: a different length, so a case can tell which body
/// was charged.
const OTHER_BODY: &[u8] = &[
    0x04, // 0: iconst_1
    0x57, // 1: pop
    0xb1, // 2: return
];

const CLASS_FLAGS: u16 = 0x0009; // public static

/// `p/Base`: one field and four methods — the recovered `foo()V`, its `foo(I)V` overload, the
/// `bar()V` nobody asks for, and the `nativeCall()V` that declares no body at all.
fn base_class() -> Vec<u8> {
    class_file(
        b"p/Base",
        b"java/lang/Object",
        Some((b"value", b"I", 0x0001)),
        &[
            MethodSpec {
                name: b"foo",
                descriptor: b"()V",
                flags: CLASS_FLAGS,
                body: Body::Instructions {
                    bytes: RECOVERY_BODY,
                    max_stack: 1,
                    max_locals: 2,
                },
            },
            MethodSpec {
                name: b"foo",
                descriptor: b"(I)V",
                flags: CLASS_FLAGS,
                body: Body::Instructions {
                    bytes: &[0xb1],
                    max_stack: 0,
                    max_locals: 1,
                },
            },
            MethodSpec {
                name: b"bar",
                descriptor: b"()V",
                flags: CLASS_FLAGS,
                body: Body::Instructions {
                    bytes: OTHER_BODY,
                    max_stack: 1,
                    max_locals: 0,
                },
            },
            MethodSpec {
                name: b"nativeCall",
                descriptor: b"()V",
                flags: 0x0109, // public static native
                body: Body::NoCode,
            },
        ],
    )
}

/// `p/Caller`: one static method whose body really calls `p/Base.foo()V`.
fn caller_class() -> Vec<u8> {
    class_file(
        b"p/Caller",
        b"java/lang/Object",
        None,
        &[MethodSpec {
            name: b"run",
            descriptor: b"()V",
            flags: CLASS_FLAGS,
            body: Body::StaticCall {
                owner: b"p/Base",
                name: b"foo",
                descriptor: b"()V",
            },
        }],
    )
}

/// The controlled archive: two top-level origins of `p/Base`, the caller beside them, a nested
/// library holding the same pair, a child that cannot be established and one resource.
fn app_archive() -> Vec<u8> {
    let base = base_class();
    let caller = caller_class();
    let nested = zip(&[
        Stored {
            name: b"p/Base.class",
            data: &base,
        },
        Stored {
            name: b"p/Caller.class",
            data: &caller,
        },
    ]);
    zip(&[
        Stored {
            name: b"p/Base.class",
            data: &base,
        },
        Stored {
            name: b"p/Caller.class",
            data: &caller,
        },
        Stored {
            name: b"WEB-INF/classes/p/Base.class",
            data: &base,
        },
        Stored {
            name: b"WEB-INF/lib/L.jar",
            data: &nested,
        },
        Stored {
            name: b"WEB-INF/lib/Broken.jar",
            data: b"not an archive",
        },
        Stored {
            name: b"META-INF/MANIFEST.MF",
            data: b"Manifest-Version: 1.0\n",
        },
    ])
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

/// One command that is expected to succeed, with its document.
fn run_complete(args: &[&str]) -> Value {
    let output = run(args);
    assert_eq!(
        status(&output),
        EXIT_COMPLETE,
        "{} failed: {}",
        args[0],
        stderr_text(&output)
    );
    stdout_json(&output)
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

/// The assertion this whole file is about: the CLI's document is the library's own, field by field.
fn assert_same_document<T: serde::Serialize>(label: &str, cli: &Value, library: &T) {
    let expected =
        strip_elapsed(&serde_json::to_value(library).expect("the library value serializes"));
    assert_eq!(
        strip_elapsed(cli),
        expected,
        "{label}: the CLI's document is not the library's own"
    );
}

/// One opened fixture under the overrides the CLI's own parameters name.
fn open(path: &Path, overrides: &[BudgetOverride]) -> (Engine, ArtifactSnapshot, Budget) {
    let mut budget = task_budget(overrides).expect("the task defaults are a bounded budget");
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::Path(path.to_path_buf()), &mut budget)
        .expect("the fixture opens");
    (engine, snapshot, budget)
}

fn path_of(path: &Path) -> &str {
    path.to_str().expect("the fixture path is UTF-8")
}

/// The physical definition identity one entry's own raw name was confirmed under.
fn definition_of(listing: &Value, raw_name: &[u8]) -> Value {
    listing["items"]
        .as_array()
        .expect("a confirmed listing publishes items")
        .iter()
        .find(|item| item["definition"]["location"]["entry"]["raw_name"] == json!(raw_name))
        .unwrap_or_else(|| {
            panic!(
                "no confirmed declaration for `{}`: {listing}",
                String::from_utf8_lossy(raw_name)
            )
        })["definition"]
        .clone()
}

/// The physical method identity one member listing printed for a raw name and descriptor.
fn method_identity(listing: &Value, name: &[u8], descriptor: &[u8]) -> Value {
    listing["items"]
        .as_array()
        .expect("a member listing publishes items")
        .iter()
        .find(|item| {
            item["kind"] == json!("method")
                && item["name"]["raw"] == json!(name)
                && item["descriptor"]["raw"] == json!(descriptor)
        })
        .unwrap_or_else(|| {
            panic!(
                "no method `{}:{}` in {listing}",
                String::from_utf8_lossy(name),
                String::from_utf8_lossy(descriptor)
            )
        })["identity"]
        .clone()
}

/// The JSON string one parameter takes: the identity exactly as the previous command printed it.
fn argument(identity: &Value) -> String {
    serde_json::to_string(identity).expect("a printed identity serializes")
}

/// Every `path = value` line of one text rendering, checked against the document it came from.
///
/// This is the text/JSON contract as a check rather than a promise: a line the projection could not
/// trace back to a field of the very document the JSON mode writes fails here.
fn assert_lines_correspond(document: &Value, stream: &str, label: &str) {
    for line in stream.lines() {
        let (path, value) = line
            .split_once(" = ")
            .unwrap_or_else(|| panic!("{label}: `{line}` is not a `path = value` line"));
        let found = lookup(document, path)
            .unwrap_or_else(|| panic!("{label}: `{path}` is not a field of the report"));
        if path.ends_with("elapsed_millis") {
            // The one field two runs of one request may differ in: the text run and the JSON run
            // are two runs, so the wall clock they measured is not compared, only the field's own
            // presence and shape.
            assert!(
                found.is_u64() && value.parse::<u64>().is_ok(),
                "{label}: `{path}` is a count in both renderings"
            );
            continue;
        }
        assert_eq!(
            found.to_string(),
            value,
            "{label}: the text rendering of `{path}` differs from the JSON field"
        );
    }
}

fn lookup<'a>(document: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = document;
    for segment in path.split('.') {
        current = match current {
            Value::Object(fields) => fields.get(segment)?,
            Value::Array(items) => items.get(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(current)
}

// ---------------------------------------------------------------------------------------------
// The legacy JSON control plane, used beside the task commands
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

fn request_json(input_path: &Path, limits: &Limits, operation: Value) -> Vec<u8> {
    let bytes = serde_json::to_vec(&json!({
        "input_path": input_path,
        "limits": limits,
        "operation": operation,
    }))
    .expect("the legacy request document serializes");
    assert!(bytes.len() < MAX_REQUEST_BYTES);
    bytes
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
        .expect("write the request JSON");
    child.wait_with_output().expect("wait for jarde-cli")
}

/// The containers and entries one explicit tree enumeration published, as the library's own type.
fn tree_report(path: &Path) -> jarde::ArtifactTreeReport {
    let output = run_stdin(&request_json(
        path,
        &limits(1 << 20),
        json!({"kind": "enumerate_artifact_tree"}),
    ));
    assert_eq!(
        status(&output),
        EXIT_COMPLETE,
        "the tree enumeration failed: {}",
        stderr_text(&output)
    );
    let document = stdout_json(&output);
    assert_eq!(document["result"]["kind"], json!("artifact_tree"));
    serde_json::from_value(document["result"]["report"].clone())
        .expect("the tree report is the library's own document")
}

/// The physical definition one of these JSON values describes: a candidate's own, or the value
/// itself when it already is a definition.
fn definition_value(value: &Value) -> &Value {
    value.get("definition").unwrap_or(value)
}

/// Whether that definition lives inside a nested container.
fn nested(value: &Value) -> bool {
    definition_value(value)["location"]["entry"]["origin"]["steps"]
        .as_array()
        .is_some_and(|steps| !steps.is_empty())
}

/// The definitions one listing published for an entry raw name, in listing order.
fn definitions_named(listing: &Value, raw_name: &[u8]) -> Vec<Value> {
    listing["items"]
        .as_array()
        .expect("a confirmed listing publishes items")
        .iter()
        .filter(|item| item["definition"]["location"]["entry"]["raw_name"] == json!(raw_name))
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------------------------
// 1.1 / A13, A16: every command's JSON document is the library's own
// ---------------------------------------------------------------------------------------------

#[test]
fn every_task_document_is_the_librarys_own() {
    let temp = TempDir::new();
    let path = temp.write("app.jar", &app_archive());
    let input = path_of(&path);

    // The candidate listing reads no header at all. Those two counts are the library's, and an
    // adapter that had read the artifact for itself would move them: this is the A13/A16 "no read
    // of its own" evidence, on top of the field-by-field equality below.
    let candidates = run_complete(&[
        "list-classes",
        "--input",
        input,
        "--evidence",
        "candidates",
        "--format",
        "json",
    ]);
    assert_eq!(candidates["execution"]["usage"]["class_headers"], json!(0));
    assert_eq!(candidates["execution"]["usage"]["class_bytes"], json!(0));
    let (engine, snapshot, mut budget) = open(&path, &[]);
    let report = engine
        .list_class_candidates(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the library lists the candidates");
    assert_same_document("list-classes --evidence candidates", &candidates, &report);

    // The confirmed listing, and the member listing of one identity it printed: both are the
    // library's reports, unchanged.
    let declarations = run_complete(&[
        "list-classes",
        "--input",
        input,
        "--evidence",
        "declarations",
        "--format",
        "json",
    ]);
    let (engine, snapshot, mut budget) = open(&path, &[]);
    let report = engine
        .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the library lists the declarations");
    assert_same_document(
        "list-classes --evidence declarations",
        &declarations,
        &report,
    );
    assert_eq!(
        declarations["execution"]["usage"]["class_headers"],
        json!(report.items.len()),
        "one header attempt per confirmed declaration and no other header read"
    );

    let definition = definition_of(&declarations, b"p/Base.class");
    let members = run_complete(&[
        "list-members",
        "--input",
        input,
        "--definition",
        &argument(&definition),
        "--format",
        "json",
    ]);
    let identity: PhysicalDefinitionId =
        serde_json::from_value(definition.clone()).expect("the printed identity is the library's");
    let (engine, snapshot, mut budget) = open(&path, &[]);
    let report = engine
        .list_members(&snapshot, &identity, &mut budget)
        .expect("the library lists the members");
    assert_same_document("list-members", &members, &report);
    assert_eq!(
        members["execution"]["usage"]["method_bodies"],
        json!(0),
        "a member listing reads no body: {}",
        members["execution"]["usage"]
    );
}

// ---------------------------------------------------------------------------------------------
// 1.2 / A13: text and JSON are one report
// ---------------------------------------------------------------------------------------------

#[test]
fn text_and_json_are_one_report() {
    let temp = TempDir::new();
    let path = temp.write("app.jar", &app_archive());
    let input = path_of(&path);

    let declarations = run_complete(&[
        "list-classes",
        "--input",
        input,
        "--evidence",
        "declarations",
        "--format",
        "json",
    ]);
    let definition = argument(&definition_of(&declarations, b"p/Base.class"));
    let members = run_complete(&[
        "list-members",
        "--input",
        input,
        "--definition",
        &definition,
        "--format",
        "json",
    ]);
    let identity = argument(&method_identity(&members, b"foo", b"()V"));

    let class_view = run_complete(&[
        "class-view",
        "--input",
        input,
        "--definition",
        &definition,
        "--body",
        "bar()V",
        "--format",
        "json",
    ]);
    let references = run_complete(&[
        "references",
        "--input",
        input,
        "--class-name",
        "p/Base",
        "--member",
        "method",
        "--member-name",
        "foo",
        "--descriptor",
        "()V",
        "--consumer",
        "invocation",
        "--format",
        "json",
    ]);
    let recovered = run_complete(&[
        "recover",
        "--input",
        input,
        "--method",
        &identity,
        "--policy",
        "plain-jar",
        "--format",
        "json",
    ]);

    // Every `path = value` line either stream carries names a field of the very document this
    // command writes for the same request, so the two renderings are one report rather than two
    // that agree today.
    let cases: Vec<(Vec<&str>, &Value)> = vec![
        (
            vec![
                "list-classes",
                "--input",
                input,
                "--evidence",
                "declarations",
            ],
            &declarations,
        ),
        (
            vec![
                "list-members",
                "--input",
                input,
                "--definition",
                &definition,
            ],
            &members,
        ),
        (
            vec![
                "class-view",
                "--input",
                input,
                "--definition",
                &definition,
                "--body",
                "bar()V",
            ],
            &class_view,
        ),
        (
            vec![
                "references",
                "--input",
                input,
                "--class-name",
                "p/Base",
                "--member",
                "method",
                "--member-name",
                "foo",
                "--descriptor",
                "()V",
                "--consumer",
                "invocation",
            ],
            &references,
        ),
        (
            vec![
                "recover",
                "--input",
                input,
                "--method",
                &identity,
                "--policy",
                "plain-jar",
            ],
            &recovered,
        ),
    ];
    for (args, document) in cases {
        let text_args: Vec<String> = args
            .iter()
            .map(|argument| (*argument).to_owned())
            .chain(["--format".to_owned(), "text".to_owned()])
            .collect();
        let borrowed: Vec<&str> = text_args.iter().map(String::as_str).collect();
        let output = run(&borrowed);
        assert_eq!(
            status(&output),
            EXIT_COMPLETE,
            "{} failed: {}",
            args[0],
            stderr_text(&output)
        );
        match document["recovered"]["recovery"]["text"].as_str() {
            // The one body a report carries is the field's own text, verbatim: standard output is
            // the document's `recovered.recovery.text` and nothing else, and the whole report
            // beside it is bookkeeping on standard error.
            Some(body) => {
                assert!(!body.is_empty(), "{}: the fixture recovers a body", args[0]);
                assert_eq!(
                    stdout_text(&output),
                    body,
                    "{}: standard output is the report's own text, byte for byte",
                    args[0]
                );
                assert_lines_correspond(document, &stderr_text(&output), "bookkeeping");
            }
            None => {
                assert_lines_correspond(document, &stdout_text(&output), "content");
                assert_lines_correspond(document, &stderr_text(&output), "bookkeeping");
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// 1.3 / A13: diagnostics are separated from the body
// ---------------------------------------------------------------------------------------------

/// The committed ECJ 4.6.1 sample whose `finallyPath(I)I` run really states a diagnostic — the same
/// sample the legacy suites use, so the case is a real one rather than a hand-made report.
const DIAGNOSTIC_SAMPLE: &[u8] =
    include_bytes!("../../../tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

#[test]
fn diagnostics_stay_out_of_the_recovered_body() {
    let temp = TempDir::new();
    let path = temp.write("HistoricalControlFlow.class", DIAGNOSTIC_SAMPLE);
    let input = path_of(&path);
    let target = [
        "--input",
        input,
        "--class-name",
        "HistoricalControlFlow",
        "--method-name",
        "finallyPath",
        "--descriptor",
        "(I)I",
        "--policy",
        "single-class",
    ];

    let mut json_args: Vec<&str> = vec!["recover"];
    json_args.extend_from_slice(&target);
    json_args.extend_from_slice(&["--format", "json"]);
    let document = run_complete(&json_args);
    let diagnostics = document["recovered"]["recovery"]["diagnostics"]
        .as_array()
        .expect("the recovery report publishes its diagnostics");
    assert!(
        !diagnostics.is_empty(),
        "the fixture is the one whose run states a diagnostic: {document}"
    );
    let code = diagnostics[0]["code"]
        .as_str()
        .expect("a diagnostic states its code");
    let text = document["recovered"]["recovery"]["text"]
        .as_str()
        .expect("a performed recovery publishes its text");
    assert!(!text.is_empty());

    let mut text_args: Vec<&str> = vec!["recover"];
    text_args.extend_from_slice(&target);
    text_args.extend_from_slice(&["--format", "text"]);
    let output = run(&text_args);
    assert_eq!(
        status(&output),
        EXIT_COMPLETE,
        "the recovery failed: {}",
        stderr_text(&output)
    );

    // The body is the report's own `text` field byte for byte, so nothing the adapter could print
    // was appended to it — no header, no diagnostic line, no newline. (The recovery layer's own
    // text may quote a reason as a comment, which is why the equality with the field, and not a
    // substring search for a message, is what proves the separation.)
    assert_eq!(
        stdout_text(&output),
        text,
        "standard output is the report's `recovered.recovery.text` and nothing else"
    );
    assert!(
        !stdout_text(&output).contains(code),
        "the diagnostic's own code is not in the body"
    );
    assert!(
        !stdout_text(&output)
            .lines()
            .any(|line| line.starts_with("recovered.recovery.diagnostics")),
        "no diagnostic line reaches standard output"
    );

    // The diagnostic is locatable in both renderings, under the same field path: a field of the
    // document in JSON mode, a line of the bookkeeping stream in text mode.
    let stderr = stderr_text(&output);
    assert!(
        stderr.contains(&format!(
            "recovered.recovery.diagnostics.0.code = \"{code}\""
        )),
        "the diagnostic is on standard error under its own JSON path: {stderr}"
    );
    assert_lines_correspond(&document, &stderr, "bookkeeping");
}

// ---------------------------------------------------------------------------------------------
// 2.1 / A14: the output file is the document standard output would carry
// ---------------------------------------------------------------------------------------------

#[test]
fn the_output_file_carries_exactly_what_standard_output_would() {
    let temp = TempDir::new();
    let path = temp.write("app.jar", &app_archive());
    let input = path_of(&path);
    let listing = run_complete(&[
        "list-classes",
        "--input",
        input,
        "--evidence",
        "declarations",
        "--format",
        "json",
    ]);
    let definition = argument(&definition_of(&listing, b"p/Base.class"));

    // JSON mode: the file carries the document standard output would have carried. The two runs
    // are two runs, so the comparison is field by field with the one field they may differ in
    // removed — the same rule the CLI-versus-library comparison uses.
    let stdout_run = run(&[
        "list-members",
        "--input",
        input,
        "--definition",
        &definition,
        "--format",
        "json",
    ]);
    assert_eq!(
        status(&stdout_run),
        EXIT_COMPLETE,
        "{}",
        stderr_text(&stdout_run)
    );
    let file = temp.join("members.json");
    let file_run = run(&[
        "list-members",
        "--input",
        input,
        "--definition",
        &definition,
        "--format",
        "json",
        "--output",
        path_of(&file),
    ]);
    assert_eq!(
        status(&file_run),
        EXIT_COMPLETE,
        "{}",
        stderr_text(&file_run)
    );
    let written: Value =
        serde_json::from_slice(&fs::read(&file).expect("the output file is written"))
            .expect("the output file is one JSON document");
    assert_eq!(
        strip_elapsed(&written),
        strip_elapsed(&stdout_json(&stdout_run)),
        "the file is the document standard output would have carried"
    );
    assert!(
        file_run.stdout.is_empty(),
        "the document went to the file, not to both"
    );

    // Text mode: the file carries the content — the body — byte for byte as standard output would,
    // and the bookkeeping planes stay on standard error exactly as they do without `--output`.
    let members = run_complete(&[
        "list-members",
        "--input",
        input,
        "--definition",
        &definition,
        "--format",
        "json",
    ]);
    let method = argument(&method_identity(&members, b"foo", b"()V"));
    let to_stdout = run(&[
        "recover",
        "--input",
        input,
        "--method",
        &method,
        "--policy",
        "plain-jar",
        "--format",
        "text",
    ]);
    assert_eq!(
        status(&to_stdout),
        EXIT_COMPLETE,
        "{}",
        stderr_text(&to_stdout)
    );
    let text_file = temp.join("Base.java");
    let to_file = run(&[
        "recover",
        "--input",
        input,
        "--method",
        &method,
        "--policy",
        "plain-jar",
        "--format",
        "text",
        "--output",
        path_of(&text_file),
    ]);
    assert_eq!(status(&to_file), EXIT_COMPLETE, "{}", stderr_text(&to_file));
    assert_eq!(
        fs::read(&text_file).expect("the recovered text is written"),
        to_stdout.stdout,
        "the file is the body standard output would have carried, byte for byte"
    );
    assert!(
        to_file.stdout.is_empty(),
        "the body went to the file, not to both"
    );
    assert!(
        !stderr_text(&to_file).is_empty(),
        "the bookkeeping planes still go to standard error"
    );

    // A refused charge writes nothing at all: the limit covers the library's own charges of that
    // dimension — measured by the run above — and cannot fund the document on top of them, so the
    // refusal is the delivery check's and not the run's.
    let usage = stdout_json(&stdout_run)["execution"]["usage"]["output_bytes"]
        .as_u64()
        .expect("the listing publishes its usage");
    let document_length = u64::try_from(stdout_run.stdout.len()).expect("the document fits u64");
    assert!(
        usage + 1 < document_length,
        "the fixture's document ({document_length}) must exceed the run's own charges ({usage})"
    );
    let refused_file = temp.join("refused.json");
    let refused = run(&[
        "list-members",
        "--input",
        input,
        "--definition",
        &definition,
        "--format",
        "json",
        "--budget",
        &format!("output_bytes={}", usage + 1),
        "--output",
        path_of(&refused_file),
    ]);
    assert_eq!(status(&refused), EXIT_USAGE, "{}", stderr_text(&refused));
    assert!(
        !refused_file.exists(),
        "a refused document leaves no file behind"
    );
    assert!(
        stderr_text(&refused).contains("\"dimension\":\"output_bytes\""),
        "the refusal names the dimension it could not pay: {}",
        stderr_text(&refused)
    );
    let failure: Value = serde_json::from_str(stderr_text(&refused).trim())
        .expect("the failure document is one JSON document");
    assert_eq!(failure["status"], json!("error"));
    assert_eq!(
        failure["usage"]["output_bytes"],
        json!(usage),
        "the refusal states what the request had already cost, beside its code"
    );
}

// ---------------------------------------------------------------------------------------------
// 2.2 / A14: the exit statuses
// ---------------------------------------------------------------------------------------------

#[test]
fn exit_statuses_are_explicit_and_a_stopped_run_is_never_success() {
    let temp = TempDir::new();
    let path = temp.write("app.jar", &app_archive());
    let input = path_of(&path);

    // Success.
    let complete = run(&["list-classes", "--input", input, "--format", "json"]);
    assert_eq!(
        status(&complete),
        EXIT_COMPLETE,
        "{}",
        stderr_text(&complete)
    );
    assert!(
        stderr_text(&complete).is_empty(),
        "JSON mode says it in the document"
    );

    // Usage: a parameter the command itself refuses before anything is opened.
    let usage_error = run(&[
        "references",
        "--input",
        input,
        "--class-name",
        "p/Base",
        "--member",
        "method",
        "--member-name",
        "foo",
        "--descriptor",
        "()V",
    ]);
    assert_eq!(
        status(&usage_error),
        EXIT_USAGE,
        "{}",
        stderr_text(&usage_error)
    );
    assert!(
        stderr_text(&usage_error).contains("cli_consumer_kinds_missing"),
        "the failure document names the refusal: {}",
        stderr_text(&usage_error)
    );
    assert!(
        usage_error.stdout.is_empty(),
        "a failure is not a report: standard output stays empty"
    );

    // Usage: a request-level refusal from the library, with the library's own code.
    let not_found = run(&[
        "recover",
        "--input",
        input,
        "--class-name",
        "p/Absent",
        "--method-name",
        "run",
        "--descriptor",
        "()V",
        "--policy",
        "plain-jar",
        "--format",
        "json",
    ]);
    assert_eq!(
        status(&not_found),
        EXIT_USAGE,
        "{}",
        stderr_text(&not_found)
    );
    assert!(
        stderr_text(&not_found).contains("operation_target_not_found"),
        "{}",
        stderr_text(&not_found)
    );

    // Ambiguity: the name holds two physical definitions and the library ran nothing. The
    // candidates are the report, so they are on standard output, and the status says a choice is
    // the caller's.
    let ambiguous = run(&[
        "class-view",
        "--input",
        input,
        "--class-name",
        "p/Base",
        "--format",
        "json",
    ]);
    assert_eq!(
        status(&ambiguous),
        EXIT_AMBIGUOUS,
        "{}",
        stderr_text(&ambiguous)
    );
    let document = stdout_json(&ambiguous);
    assert_eq!(document["outcome"], json!("ambiguous"));
    let candidates = document["candidates"]
        .as_array()
        .expect("the candidates are published");
    assert_eq!(
        candidates.len(),
        2,
        "both origins of the name are candidates: {document}"
    );
    assert!(
        candidates.iter().all(is_class_candidate),
        "each candidate is the declaration it was read as: {document}"
    );
    let origins: Vec<&Value> = candidates
        .iter()
        .map(|candidate| &candidate["definition"]["location"]["entry"]["raw_name"])
        .collect();
    assert!(
        origins.contains(&&json!(b"p/Base.class".as_slice()))
            && origins.contains(&&json!(b"WEB-INF/classes/p/Base.class".as_slice())),
        "the two origins of the name are both candidates and neither is elected: {document}"
    );

    // Incomplete: one header is funded, the scan stops, and the reliable prefix stays usable —
    // with a status that says so rather than a zero the caller would read as a whole answer.
    let stopped = run(&[
        "list-classes",
        "--input",
        input,
        "--evidence",
        "declarations",
        "--budget",
        "class_headers=1",
        "--format",
        "json",
    ]);
    assert_eq!(
        status(&stopped),
        EXIT_INCOMPLETE,
        "{}",
        stderr_text(&stopped)
    );
    let document = stdout_json(&stopped);
    assert_eq!(document["execution"]["status"], json!("partial"));
    assert_eq!(
        document["execution"]["reason"]["kind"],
        json!("budget_exceeded")
    );
    assert_eq!(
        document["execution"]["reason"]["dimension"],
        json!("class_headers"),
        "the stop names the dimension that ran out: {document}"
    );
    assert_eq!(
        document["items"]
            .as_array()
            .expect("the prefix is published")
            .len(),
        1,
        "the reliable prefix is delivered beside the stop: {document}"
    );
    assert!(
        !document["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .is_empty(),
        "the stop is locatable: {document}"
    );

    // Delivery: a document that cannot be written is its own status, distinct from every class
    // above, and it is not a usage error of the request.
    let unwritable = temp.join("absent-directory/members.json");
    let delivery = run(&[
        "list-members",
        "--input",
        input,
        "--definition",
        &argument(&definition_of(&complete_report(input), b"p/Base.class")),
        "--format",
        "json",
        "--output",
        path_of(&unwritable),
    ]);
    assert_eq!(
        status(&delivery),
        EXIT_DELIVERY,
        "{}",
        stderr_text(&delivery)
    );
    assert!(
        stderr_text(&delivery).contains("cli_create_output"),
        "{}",
        stderr_text(&delivery)
    );

    // The library's own document for that same request, so the CLI's stop is not a re-description
    // of it.
    let (engine, snapshot, mut budget) = open(&path, &[BudgetOverride::ClassHeaders { limit: 1 }]);
    let report = engine
        .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("a stopped listing is a report, not an error");
    assert_same_document(
        "list-classes under a tight class_headers",
        &document,
        &report,
    );
}

/// The confirmed listing of one fixture, for the cases that need a definition identity.
fn complete_report(input: &str) -> Value {
    run_complete(&[
        "list-classes",
        "--input",
        input,
        "--evidence",
        "declarations",
        "--format",
        "json",
    ])
}

/// Whether one candidate of an ambiguous outcome is a class declaration.
fn is_class_candidate(candidate: &Value) -> bool {
    candidate["kind"] == json!("class_declaration") && candidate["definition"].is_object()
}

// ---------------------------------------------------------------------------------------------
// 3.2 / A16, A17: the class view reads exactly the bodies it was asked for
// ---------------------------------------------------------------------------------------------

#[test]
fn the_class_view_charges_one_body_per_requested_body() {
    let temp = TempDir::new();
    let path = temp.write("app.jar", &app_archive());
    let input = path_of(&path);
    let listing = run_complete(&[
        "list-classes",
        "--input",
        input,
        "--evidence",
        "declarations",
        "--format",
        "json",
    ]);
    let definition = argument(&definition_of(&listing, b"p/Base.class"));

    // `foo()V` and `bar()V` declare bodies of different lengths, so the body that was charged is
    // the body that was asked for — and not a class-wide read of every body.
    let foo = run_complete(&[
        "class-view",
        "--input",
        input,
        "--definition",
        &definition,
        "--body",
        "foo()V",
        "--format",
        "json",
    ]);
    let bar = run_complete(&[
        "class-view",
        "--input",
        input,
        "--definition",
        &definition,
        "--body",
        "bar()V",
        "--format",
        "json",
    ]);
    for (label, document, name, descriptor, other_length) in [
        ("foo", &foo, "foo", "()V", &bar),
        ("bar", &bar, "bar", "()V", &foo),
    ] {
        assert_eq!(
            document["usage"]["method_bodies"],
            json!(1),
            "{label}: one requested body is one `method_bodies` attempt"
        );
        let bodies = document["bodies"]
            .as_array()
            .expect("one result per requested body");
        assert_eq!(bodies.len(), 1, "{label}: exactly the requested body");
        assert_eq!(bodies[0]["kind"], json!("read"));
        assert_eq!(bodies[0]["method"]["name"], json!(name.as_bytes()));
        assert_eq!(
            bodies[0]["method"]["descriptor"],
            json!(descriptor.as_bytes())
        );
        assert_ne!(
            document["usage"]["code_bytes"], other_length["usage"]["code_bytes"],
            "{label}: the unrequested body was not charged: {}",
            document["usage"]
        );
        assert!(
            document["usage"]["code_bytes"].as_u64().unwrap_or(0) > 0,
            "{label}: the requested body really was read: {}",
            document["usage"]
        );
    }

    // The library's own view of the same request: the numbers above are its numbers.
    let identity: PhysicalDefinitionId =
        serde_json::from_str(&definition).expect("the printed identity is the library's");
    let (engine, snapshot, mut budget) = open(&path, &[]);
    let outcome = engine
        .class_view(
            &snapshot,
            &PhysicalScope::SnapshotAll,
            &ClassViewRequest {
                class: ClassRef::Definition {
                    definition: identity,
                },
                bodies: vec![BodyRef::Name {
                    name: jarde::JvmBytes(b"foo".to_vec()),
                    descriptor: Some(jarde::JvmBytes(b"()V".to_vec())),
                }],
            },
            &mut budget,
        )
        .expect("the library views the class");
    assert_same_document("class-view --body foo()V", &foo, &outcome);

    // A body name with two declared descriptors reads nothing and answers with the candidates: the
    // overload is the caller's to choose, and no body is charged while it is unchosen.
    let overload = run(&[
        "class-view",
        "--input",
        input,
        "--definition",
        &definition,
        "--body",
        "foo",
        "--format",
        "json",
    ]);
    assert_eq!(
        status(&overload),
        EXIT_AMBIGUOUS,
        "{}",
        stderr_text(&overload)
    );
    let document = stdout_json(&overload);
    assert_eq!(document["outcome"], json!("ambiguous"));
    assert!(
        document["candidates"]
            .as_array()
            .expect("the overloads are published")
            .len()
            >= 2,
        "{document}"
    );
    assert!(
        document.get("bodies").is_none(),
        "an unchosen overload is no body result: {document}"
    );
}

// ---------------------------------------------------------------------------------------------
// 3.2 / A16, A17: the chain, driven by the identities the previous command printed
// ---------------------------------------------------------------------------------------------

#[test]
fn the_task_chain_runs_over_the_identities_the_listings_printed() {
    let temp = TempDir::new();
    let path = temp.write("app.jar", &app_archive());
    let input = path_of(&path);

    // 1. List the classes. The listing's own `definition` is the identity every later command is
    //    addressed by; nothing here reassembles one from a name, an ordinal and a digest.
    let classes = run_complete(&[
        "list-classes",
        "--input",
        input,
        "--evidence",
        "declarations",
        "--format",
        "json",
    ]);
    let definition = argument(&definition_of(&classes, b"p/Base.class"));
    assert!(
        !nested(&definition_of(&classes, b"p/Base.class")),
        "the identity names the definition's own physical place at the archive's top level"
    );

    // 2. List that definition's members, and take the identity of one method it printed.
    let members = run_complete(&[
        "list-members",
        "--input",
        input,
        "--definition",
        &definition,
        "--format",
        "json",
    ]);
    let method = argument(&method_identity(&members, b"foo", b"()V"));
    assert_eq!(members["execution"]["usage"]["method_bodies"], json!(0));

    // 3. Look at the references to the symbol the listing named.
    let references = run_complete(&[
        "references",
        "--input",
        input,
        "--class-name",
        "p/Base",
        "--member",
        "method",
        "--member-name",
        "foo",
        "--descriptor",
        "()V",
        "--consumer",
        "invocation",
        "--format",
        "json",
    ]);
    let (engine, snapshot, mut budget) = open(&path, &[]);
    let report = engine
        .query(
            &snapshot,
            &QueryRequest {
                relation: QueryRelation::MentionsSymbol,
                target: QueryTarget::Symbol {
                    value: SymbolRef::Method {
                        owner: jarde::JvmBytes(b"p/Base".to_vec()),
                        name: jarde::JvmBytes(b"foo".to_vec()),
                        descriptor: jarde::JvmBytes(b"()V".to_vec()),
                    },
                },
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
                max_items: 0,
                cursor: None,
            },
            &mut budget,
        )
        .expect("the library scans the references");
    let grouping = ReferenceGrouping::from_query(report);
    assert_same_document("references", &references, &grouping);
    assert!(
        !grouping.methods.is_empty(),
        "the call site is grouped under the method that owns it: {references}"
    );

    // 4. Open the code of the method the member listing printed.
    let recovered = run_complete(&[
        "recover",
        "--input",
        input,
        "--method",
        &method,
        "--policy",
        "plain-jar",
        "--format",
        "json",
    ]);
    let identity: PhysicalMethodId =
        serde_json::from_str(&method).expect("the printed identity is the library's");
    let (engine, snapshot, mut budget) = open(&path, &[]);
    let outcome = engine
        .recover_target(
            slice::from_ref(&snapshot),
            &MethodOperationRequest {
                method: MethodRef::Method {
                    method: identity.clone(),
                },
                environment: EnvironmentRequest {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                    policy: EnvironmentPolicy::PlainJar,
                    profile: RuntimeProfile {
                        java_release: 8,
                        multi_release: MultiReleasePolicy::Disabled,
                        layout: LayoutMode::Generic,
                    },
                    loader: LoaderId("app".to_string()),
                },
            },
            &mut budget,
        )
        .expect("the library recovers the method");
    assert_same_document("recover", &recovered, &outcome);

    // The chain's last step really ran over the identity the member listing printed, and its read
    // range is the library operation's own: one header, one body — the requested one — and nothing
    // of the class's other two methods.
    let usage = &recovered["recovered"]["analysis"]["execution"]["usage"];
    assert_eq!(recovered["outcome"], json!("performed"));
    assert_eq!(
        recovered["method"],
        method_identity(&members, b"foo", b"()V")
    );
    assert_eq!(usage["class_headers"], json!(1));
    assert_eq!(usage["method_bodies"], json!(1));
    let text = recovered["recovered"]["recovery"]["text"]
        .as_str()
        .expect("the recovery publishes its text");
    assert!(!text.is_empty(), "{recovered}");
    assert_eq!(
        recovered["presentation"]["execution"]["status"],
        json!("complete")
    );

    // The method identity the chain consumed is also what a class view decodes: one body attempt,
    // for the same member, through the other identity-taking command.
    let view = run_complete(&[
        "class-view",
        "--input",
        input,
        "--definition",
        &definition,
        "--body-method",
        &method,
        "--format",
        "json",
    ]);
    assert_eq!(view["usage"]["method_bodies"], json!(1));
    assert_eq!(
        view["bodies"][0]["method"],
        method_identity(&members, b"foo", b"()V")
    );
}

// ---------------------------------------------------------------------------------------------
// 3.1 / A07, A08, A14: navigation uses the library listings and the caller's scope
// ---------------------------------------------------------------------------------------------

#[test]
fn navigation_uses_the_library_listings_and_the_caller_declared_scope() {
    let temp = TempDir::new();
    let path = temp.write("app.jar", &app_archive());
    let input = path_of(&path);

    // The discovery surface is the explicit tree enumeration: it is where a real container identity
    // comes from, and the caller declares what to scan from it.
    let tree = tree_report(&path);
    let root = tree
        .containers
        .iter()
        .find(|container| container.origin.steps.is_empty())
        .expect("the root container is published")
        .origin
        .root_container
        .clone();
    let nested_container = tree
        .containers
        .iter()
        .find(|container| {
            container
                .origin
                .steps
                .last()
                .is_some_and(|step| step.via_raw_name.0 == b"WEB-INF/lib/L.jar")
        })
        .map(|container| container.origin.clone())
        .expect("the nested library is a container of the tree");

    // The ordinary scope does not recurse: the nested library is one entry, and the classes inside
    // it are not candidates of this scope. The declared tree scope reaches them.
    let ordinary = run_complete(&[
        "list-classes",
        "--input",
        input,
        "--evidence",
        "candidates",
        "--format",
        "json",
    ]);
    assert!(
        ordinary["items"]
            .as_array()
            .expect("the candidates are published")
            .iter()
            .all(|item| {
                item["kind"] != json!("class_candidate")
                    || item["location"]["entry"]["origin"]["steps"] == json!([])
            }),
        "ordinary enumeration reports top-level entries only: {ordinary}"
    );
    assert!(
        ordinary["items"]
            .as_array()
            .expect("the candidates are published")
            .iter()
            .any(|item| item["kind"] == json!("resource")
                && item["entry"]["raw_name"] == json!(b"WEB-INF/lib/L.jar".as_slice())),
        "the nested library is published as the resource it is: {ordinary}"
    );

    let scope = json!({"kind": "artifact_tree", "root_container": root}).to_string();
    let declared = run(&[
        "list-classes",
        "--input",
        input,
        "--evidence",
        "declarations",
        "--scope",
        &scope,
        "--format",
        "json",
    ]);
    let document = stdout_json(&declared);
    assert_eq!(
        document["view"]["scope"]["kind"],
        json!("artifact_tree"),
        "the report echoes the scope the caller declared: {document}"
    );
    let inside = definitions_named(&document, b"p/Base.class")
        .into_iter()
        .find(nested)
        .unwrap_or_else(|| {
            panic!(
                "the declared tree scope reaches the class inside the nested library: {document}"
            )
        });
    assert_eq!(
        definition_value(&inside)["location"]["entry"]["origin"],
        serde_json::to_value(&nested_container).expect("the origin serializes"),
        "the listing keeps the container identity the tree enumeration published"
    );

    // A child that cannot be established is the library's own partial answer: the prefix, the
    // diagnostic that names the child and a non-Complete execution, never a repaired tree or a
    // silent empty one. The status says the answer stopped.
    assert_eq!(
        status(&declared),
        EXIT_INCOMPLETE,
        "a broken child is a stopped answer: {}",
        stderr_text(&declared)
    );
    assert_eq!(document["execution"]["status"], json!("partial"));
    assert!(
        document["diagnostics"]
            .as_array()
            .expect("the stop is stated")
            .iter()
            .any(|diagnostic| diagnostic["provenance"].is_object()),
        "the diagnostic names the child it came from: {document}"
    );
    let (engine, snapshot, mut budget) = open(&path, &[]);
    let report = engine
        .list_class_declarations(
            &snapshot,
            &PhysicalScope::ArtifactTree {
                root_container: root.clone(),
            },
            &mut budget,
        )
        .expect("a stopped tree listing is a report, not an error");
    assert_same_document(
        "list-classes over the declared tree scope",
        &document,
        &report,
    );

    // Two origins of one name are two candidates, and the caller chooses: a name-bound recovery
    // over this archive is answered with them rather than with an elected one.
    let ambiguous = run(&[
        "recover",
        "--input",
        input,
        "--class-name",
        "p/Base",
        "--method-name",
        "foo",
        "--descriptor",
        "()V",
        "--policy",
        "plain-jar",
        "--format",
        "json",
    ]);
    assert_eq!(
        status(&ambiguous),
        EXIT_AMBIGUOUS,
        "{}",
        stderr_text(&ambiguous)
    );

    // The declared positions are the caller's, and the adapter generates none: the report echoes
    // the environment's own identity — a root this adapter had added would change it — and the
    // library's answer for exactly this declaration is the document that comes back.
    let caller = definition_of(&document, b"p/Caller.class");
    let members = run_complete(&[
        "list-members",
        "--input",
        input,
        "--definition",
        &argument(&caller),
        "--format",
        "json",
    ]);
    let method = argument(&method_identity(&members, b"run", b"()V"));
    let declared = json!({"kind": "external", "id": "host-jdk"}).to_string();
    let ran = run(&[
        "recover",
        "--input",
        input,
        "--method",
        &method,
        "--policy",
        "explicit-classpath",
        "--root",
        &declared,
        "--format",
        "json",
    ]);
    assert_eq!(
        status(&ran),
        EXIT_INCOMPLETE,
        "a run that cannot resolve what its declaration names stops: {}",
        stderr_text(&ran)
    );
    let recovered = stdout_json(&ran);
    let identity: PhysicalMethodId =
        serde_json::from_str(&method).expect("the printed identity is the library's");
    let (engine, snapshot, mut budget) = open(&path, &[]);
    let outcome = engine
        .recover_target(
            slice::from_ref(&snapshot),
            &MethodOperationRequest {
                method: MethodRef::Method { method: identity },
                environment: EnvironmentRequest {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                    policy: EnvironmentPolicy::ExplicitClasspath {
                        roots: vec![LoadRoot::External {
                            id: "host-jdk".to_string(),
                        }],
                    },
                    profile: RuntimeProfile {
                        java_release: 8,
                        multi_release: MultiReleasePolicy::Disabled,
                        layout: LayoutMode::Generic,
                    },
                    loader: LoaderId("app".to_string()),
                },
            },
            &mut budget,
        )
        .expect("the library recovers the method under the declared environment");
    assert_same_document(
        "recover under a declared external root",
        &recovered,
        &outcome,
    );
    assert!(
        recovered["recovered"]["analysis"]["environment_identity"].is_object(),
        "the report echoes the environment it ran under: {recovered}"
    );
    assert!(
        !recovered["recovered"]["analysis"]["environment_problems"]
            .as_array()
            .expect("the analysis publishes its environment problems")
            .is_empty(),
        "a declaration that names content this request did not provide stays the library's own \
         unavailable position: {recovered}"
    );
}

// ---------------------------------------------------------------------------------------------
// preserve-task-operation-stops: an unfinished selection is an incomplete command, not a choice
// or a missing target
// ---------------------------------------------------------------------------------------------

/// The independent review's archive: one class whose name search is answerable and one more
/// candidate of the same name whose bytes are not a class, in the order the entries are stored.
fn unfinished_search_archive(damaged_first: bool) -> Vec<u8> {
    let base = base_class();
    let broken: &[u8] = b"broken";
    if damaged_first {
        zip(&[
            Stored {
                name: b"p/Base.class",
                data: broken,
            },
            Stored {
                name: b"x/p/Base.class",
                data: &base,
            },
        ])
    } else {
        zip(&[
            Stored {
                name: b"p/Base.class",
                data: &base,
            },
            Stored {
                name: b"x/p/Base.class",
                data: broken,
            },
        ])
    }
}

#[test]
fn an_unfinished_name_selection_exits_four_and_is_the_librarys_own_document() {
    let temp = TempDir::new();
    let path = temp.write("unfinished.jar", &unfinished_search_archive(false));
    let input = path_of(&path);

    // One candidate confirmed and then a same-named entry that does not read: the search did not
    // finish. The command exits 4 — not 0 (an unfinished search is not a success), not 3 (one
    // candidate has not been shown to be ambiguous) and not 2 (the name has not been shown to be
    // missing) — and the document is the library's own outcome.
    let view = run(&[
        "class-view",
        "--input",
        input,
        "--class-name",
        "p/Base",
        "--format",
        "json",
    ]);
    assert_eq!(status(&view), EXIT_INCOMPLETE, "{}", stderr_text(&view));
    let document = stdout_json(&view);
    assert_eq!(document["outcome"], json!("incomplete"));
    assert_eq!(document["execution"]["status"], json!("failed"));
    assert_eq!(
        document["execution"]["reason"]["code"],
        json!("classfile_decode")
    );
    assert_eq!(
        document["candidates"]
            .as_array()
            .expect("the confirmed prefix is published")
            .len(),
        1
    );
    assert_eq!(
        document["execution"]["usage"]["method_bodies"],
        json!(0),
        "an unfinished selection decodes no body: {document}"
    );
    assert_eq!(document["execution"]["usage"]["ir_items"], json!(0));
    assert!(
        document.get("bodies").is_none(),
        "no view is published for a target the search never bound: {document}"
    );

    // The library's own value for the same request: the JSON above is it, field for field.
    let (engine, snapshot, mut budget) = open(&path, &[]);
    let outcome = engine
        .class_view(
            &snapshot,
            &PhysicalScope::SnapshotAll,
            &ClassViewRequest {
                class: ClassRef::Name {
                    class: ClassNameQuery::internal("p/Base"),
                },
                bodies: Vec::new(),
            },
            &mut budget,
        )
        .expect("a stopped search is a report, not an error");
    assert_same_document("class-view over an unfinished search", &document, &outcome);

    // The same unfinished search on the recovery path: the review's other defect — a confirmed
    // method beside a damaged candidate was recovered as if it were the unique target.
    let recover = run(&[
        "recover",
        "--input",
        input,
        "--class-name",
        "p/Base",
        "--method-name",
        "foo",
        "--descriptor",
        "()V",
        "--policy",
        "plain-jar",
        "--format",
        "json",
    ]);
    assert_eq!(
        status(&recover),
        EXIT_INCOMPLETE,
        "{}",
        stderr_text(&recover)
    );
    let recovered = stdout_json(&recover);
    assert_eq!(recovered["outcome"], json!("incomplete"));
    assert_eq!(
        recovered["candidates"]
            .as_array()
            .expect("candidates")
            .len(),
        1
    );
    assert_eq!(recovered["execution"]["usage"]["method_bodies"], json!(0));
    let (engine, snapshot, mut budget) = open(&path, &[]);
    let outcome = engine
        .recover_target(
            slice::from_ref(&snapshot),
            &MethodOperationRequest {
                method: MethodRef::Name {
                    class: ClassNameQuery::internal("p/Base"),
                    name: jarde::JvmBytes(b"foo".to_vec()),
                    descriptor: Some(jarde::JvmBytes(b"()V".to_vec())),
                },
                environment: EnvironmentRequest {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                    policy: EnvironmentPolicy::PlainJar,
                    profile: RuntimeProfile {
                        java_release: 8,
                        multi_release: MultiReleasePolicy::Disabled,
                        layout: LayoutMode::Generic,
                    },
                    loader: LoaderId("app".to_string()),
                },
            },
            &mut budget,
        )
        .expect("a stopped search is a report, not an error");
    assert_same_document("recover over an unfinished search", &recovered, &outcome);

    // The damaged entry first: zero confirmed candidates, and still the same incomplete command
    // rather than the missing-target error the count alone would suggest.
    let before = temp.write("unfinished-before.jar", &unfinished_search_archive(true));
    let view = run(&[
        "class-view",
        "--input",
        path_of(&before),
        "--class-name",
        "p/Base",
        "--format",
        "json",
    ]);
    assert_eq!(status(&view), EXIT_INCOMPLETE, "{}", stderr_text(&view));
    let before_document = stdout_json(&view);
    assert_eq!(before_document["outcome"], json!("incomplete"));
    assert!(
        before_document["candidates"]
            .as_array()
            .expect("candidates")
            .is_empty(),
        "{before_document}"
    );

    // Text is the same report: every line it prints names a field of the very document above, and
    // the stop facts the JSON states are among the lines text prints.
    let text = run(&[
        "class-view",
        "--input",
        input,
        "--class-name",
        "p/Base",
        "--format",
        "text",
    ]);
    assert_eq!(status(&text), EXIT_INCOMPLETE);
    let content = stdout_text(&text);
    let bookkeeping = stderr_text(&text);
    assert_lines_correspond(&document, &content, "content");
    assert_lines_correspond(&document, &bookkeeping, "bookkeeping");
    assert!(content.contains("outcome = \"incomplete\""), "{content}");
    assert!(
        content.contains("query.class.spelling = \"p/Base\""),
        "{content}"
    );
    assert!(
        content.contains("candidates.0."),
        "the confirmed candidate is content: {content}"
    );
    for fact in [
        "execution.status = \"failed\"",
        "execution.reason.code = \"classfile_decode\"",
        "diagnostics.0.code = \"classfile_decode\"",
    ] {
        assert!(
            bookkeeping.contains(fact),
            "the text rendering states `{fact}`: {bookkeeping}"
        );
    }

    // The other two statuses stay what they are on the same kind of request: two readable origins
    // of one name are a choice (3), and a name the whole scope searched is an input error (2).
    let base = base_class();
    let two_origins = temp.write(
        "two-origins.jar",
        &zip(&[
            Stored {
                name: b"p/Base.class",
                data: &base,
            },
            Stored {
                name: b"WEB-INF/classes/p/Base.class",
                data: &base,
            },
        ]),
    );
    let ambiguous = run(&[
        "class-view",
        "--input",
        path_of(&two_origins),
        "--class-name",
        "p/Base",
        "--format",
        "json",
    ]);
    assert_eq!(
        status(&ambiguous),
        EXIT_AMBIGUOUS,
        "{}",
        stderr_text(&ambiguous)
    );
    assert_eq!(stdout_json(&ambiguous)["outcome"], json!("ambiguous"));

    let missing = run(&[
        "class-view",
        "--input",
        input,
        "--class-name",
        "p/Absent",
        "--format",
        "json",
    ]);
    assert_eq!(status(&missing), EXIT_USAGE, "{}", stderr_text(&missing));
    assert!(
        stderr_text(&missing).contains("operation_target_not_found"),
        "{}",
        stderr_text(&missing)
    );
}

/// The library merges a stopped body into the view's own top-level execution, so the adapter reads
/// the depth of the answer from the report instead of walking the bodies again.
#[test]
fn a_stopped_body_is_an_incomplete_view_from_the_librarys_own_plane() {
    let temp = TempDir::new();
    // One illegal first opcode at BCI 0 (`0xff` is no instruction of the JVM's table) beside one
    // healthy body: the member table is intact and only one body's decode stopped.
    const ILLEGAL: &[u8] = &[0xff];
    const HEALTHY: &[u8] = &[0xb1];
    let class = class_file(
        b"p/Body",
        b"java/lang/Object",
        None,
        &[
            MethodSpec {
                name: b"broken",
                descriptor: b"()V",
                flags: CLASS_FLAGS,
                body: Body::Instructions {
                    bytes: ILLEGAL,
                    max_stack: 0,
                    max_locals: 0,
                },
            },
            MethodSpec {
                name: b"healthy",
                descriptor: b"()V",
                flags: CLASS_FLAGS,
                body: Body::Instructions {
                    bytes: HEALTHY,
                    max_stack: 0,
                    max_locals: 0,
                },
            },
        ],
    );
    let path = temp.write(
        "body.jar",
        &zip(&[Stored {
            name: b"p/Body.class",
            data: &class,
        }]),
    );
    let input = path_of(&path);
    let view = run(&[
        "class-view",
        "--input",
        input,
        "--class-name",
        "p/Body",
        "--body",
        "broken()V",
        "--body",
        "healthy()V",
        "--format",
        "json",
    ]);
    assert_eq!(status(&view), EXIT_INCOMPLETE, "{}", stderr_text(&view));
    let document = stdout_json(&view);
    assert_eq!(
        document["execution"]["status"],
        json!("partial"),
        "the library's own top level carries the body's stop: {document}"
    );
    assert_eq!(
        document["execution"]["reason"]["code"],
        json!("classfile_instruction_decode")
    );
    let bodies = document["bodies"]
        .as_array()
        .expect("both requested bodies are results");
    assert_eq!(bodies.len(), 2);
    assert_eq!(bodies[0]["stopped_at"]["phase"], json!("instructions"));
    assert_eq!(bodies[0]["stopped_at"]["bci"], json!(0));
    assert_eq!(bodies[0]["execution"]["status"], json!("partial"));
    assert_eq!(bodies[1]["stopped_at"], Value::Null);
    assert_eq!(bodies[1]["execution"]["status"], json!("complete"));
    // The class and member planes are the read's own evidence, not the body's.
    assert_eq!(
        document["coverage"]["artifact_structural"]["state"],
        json!("complete_within_schema")
    );
    assert_eq!(document["items"][0]["member_table"], Value::Null);

    // And the library's own outcome for the same request is that document.
    let (engine, snapshot, mut budget) = open(&path, &[]);
    let outcome = engine
        .class_view(
            &snapshot,
            &PhysicalScope::SnapshotAll,
            &ClassViewRequest {
                class: ClassRef::Name {
                    class: ClassNameQuery::internal("p/Body"),
                },
                bodies: vec![
                    BodyRef::Name {
                        name: jarde::JvmBytes(b"broken".to_vec()),
                        descriptor: Some(jarde::JvmBytes(b"()V".to_vec())),
                    },
                    BodyRef::Name {
                        name: jarde::JvmBytes(b"healthy".to_vec()),
                        descriptor: Some(jarde::JvmBytes(b"()V".to_vec())),
                    },
                ],
            },
            &mut budget,
        )
        .expect("the view runs");
    assert_same_document("class-view with one stopped body", &document, &outcome);
}

// ---------------------------------------------------------------------------------------------
// Design decision 6: the legacy JSON control plane stays the other surface
// ---------------------------------------------------------------------------------------------

#[test]
fn the_legacy_operations_remain_their_own_surface() {
    let temp = TempDir::new();
    let path = temp.write("app.jar", &app_archive());

    // The control plane still answers a request that names no task command.
    let legacy = run_stdin(&request_json(
        &path,
        &limits(1 << 20),
        json!({"kind": "enumerate"}),
    ));
    assert_eq!(status(&legacy), EXIT_COMPLETE, "{}", stderr_text(&legacy));
    let document = stdout_json(&legacy);
    assert_eq!(document["status"], json!("ok"));
    assert_eq!(document["result"]["kind"], json!("enumeration"));

    // And naming both surfaces at once is a usage error rather than a silently elected one.
    let both = run(&[
        "--request",
        path_of(&path),
        "list-classes",
        "--input",
        path_of(&path),
    ]);
    assert_eq!(status(&both), EXIT_USAGE, "{}", stderr_text(&both));
    assert!(
        stderr_text(&both).contains("two different requests"),
        "{}",
        stderr_text(&both)
    );
    assert!(both.stdout.is_empty());
}

// ---------------------------------------------------------------------------------------------
// `bound-recovery-recursion`: a request that used to abort the process answers with a report
// ---------------------------------------------------------------------------------------------

/// One hand-assembled class whose only method drives the region walk's own cycle.
///
/// This is the process-level half of `bound-recovery-recursion`'s controlled fixture: the same
/// bytes `tests/p3_eval_context.rs`'s `recursion_fixture` writes (the two writers produce the same
/// 170 bytes; the SHA-256 is recorded in the change's verification), so the library entry and the
/// process entry answer for one input rather than for two lookalikes.
///
/// The shape is the one the change's localization found on the real repro — an inner loop whose
/// `goto` ends the outer loop's header block, with the outer latch in a block of its own. Before
/// the guard existed, this member's walk re-entered its own header without a bound and the process
/// died with `fatal runtime error: stack overflow, aborting`; the point of this case is that the
/// entry now answers, so it asserts the *report*, never the absence of a crash message.
///
/// The body is the bytecode `javac 23.0.1 --release 8 -g:none` emits for
/// `int x = 0; int i; do { for (i = 0; i < 3; i = i + 1) { x = x + 1; } } while (x < 5);`, with the
/// same `StackMapTable` (an `append_frame` at BCI 2, one at BCI 4, a `same_frame` at BCI 20) — a
/// version-52 body with a back edge has to declare the frames its verifier starts blocks with.
fn recursion_fixture() -> Vec<u8> {
    /// `int x = 0; int i; do { for (i = 0; i < 3; i = i + 1) { x = x + 1; } } while (x < 5);`
    const BODY: &[u8] = &[
        0x03, // 0: iconst_0
        0x3b, // 1: istore_0
        0x03, // 2: iconst_0      ← H: the outer loop's header and its back edge's target
        0x3c, // 3: istore_1
        0x1b, // 4: iload_1       ← the inner loop's header (the `goto` target):
        0x06, // 5: iconst_3      its own block is what ends H's
        0xa2, 0x00, 0x0e, // 6: if_icmpge 20
        0x1a, // 9: iload_0
        0x04, // 10: iconst_1
        0x60, // 11: iadd
        0x3b, // 12: istore_0
        0x1b, // 13: iload_1
        0x04, // 14: iconst_1
        0x60, // 15: iadd
        0x3c, // 16: istore_1
        0xa7, 0xff, 0xf3, // 17: goto 4
        0x1a, // 20: iload_0      ← L: the latch, a block of its own
        0x08, // 21: iconst_5
        0xa1, 0xff, 0xec, // 22: if_icmplt 2
        0xb1, // 25: return
    ];
    /// The frames javac declares for `BODY`: two `append_frame`s and a `same_frame`.
    const FRAMES: &[u8] = &[
        0x00, 0x03, // number_of_entries
        0xfc, 0x00, 0x02, 0x01, // append_frame: BCI 2, +[int]
        0xfc, 0x00, 0x01, 0x01, // append_frame: BCI 4, +[int]
        0x0f, // same_frame: BCI 20 (delta 15)
    ];

    fn u16b(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    fn u32b(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    fn utf8(pool: &mut Vec<u8>, text: &[u8]) {
        pool.push(1);
        u16b(
            pool,
            u16::try_from(text.len()).expect("fixture name fits u16"),
        );
        pool.extend_from_slice(text);
    }

    let mut pool: Vec<u8> = Vec::new();
    utf8(&mut pool, b"p/Recursive"); // 1
    pool.push(7); // 2: Class 1
    u16b(&mut pool, 1);
    utf8(&mut pool, b"java/lang/Object"); // 3
    pool.push(7); // 4: Class 3
    u16b(&mut pool, 3);
    utf8(&mut pool, b"method"); // 5
    utf8(&mut pool, b"()V"); // 6
    utf8(&mut pool, b"Code"); // 7
    utf8(&mut pool, b"StackMapTable"); // 8

    let mut output = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0); // minor
    u16b(&mut output, 52); // major: Java 8
    u16b(&mut output, 9); // constant_pool_count
    output.extend_from_slice(&pool);
    u16b(&mut output, 0x0021); // public super
    u16b(&mut output, 2); // this_class
    u16b(&mut output, 4); // super_class
    u16b(&mut output, 0); // interfaces
    u16b(&mut output, 0); // fields
    u16b(&mut output, 1); // methods
    u16b(&mut output, 0x0009); // public static
    u16b(&mut output, 5); // name → "method"
    u16b(&mut output, 6); // descriptor → "()V"
    u16b(&mut output, 1); // attributes
    u16b(&mut output, 7); // "Code"
    let mut code = Vec::new();
    u16b(&mut code, 2); // max_stack
    u16b(&mut code, 2); // max_locals
    u32b(
        &mut code,
        u32::try_from(BODY.len()).expect("fixture body fits u32"),
    );
    code.extend_from_slice(BODY);
    u16b(&mut code, 0); // exception table
    u16b(&mut code, 1); // code attributes
    u16b(&mut code, 8); // "StackMapTable"
    u32b(
        &mut code,
        u32::try_from(FRAMES.len()).expect("fixture frames fit u32"),
    );
    code.extend_from_slice(FRAMES);
    u32b(
        &mut output,
        u32::try_from(code.len()).expect("fixture attribute fits u32"),
    );
    output.extend_from_slice(&code);
    u16b(&mut output, 0); // class attributes
    output
}

/// The process entry answers for the input that used to abort it: exit 4, a JSON report on standard
/// output, and a stop whose plane and diagnostic say what happened.
///
/// `status` is the signal assertion as well as the status one: it panics when the process ended on a
/// signal instead of with a status, so a regression that aborts again fails here rather than passing
/// on a non-zero code — and a report is required on standard output, which a dying process cannot
/// write.
#[test]
fn a_recovery_that_used_to_abort_the_process_exits_with_a_report() {
    let temp = TempDir::new();
    let fixture = recursion_fixture();
    assert_eq!(
        fixture.len(),
        170,
        "the fixture is the same class `tests/p3_eval_context.rs` builds"
    );
    let path = temp.write("Recursive.class", &fixture);

    let output = run(&[
        "recover",
        "--input",
        path_of(&path),
        "--policy",
        "single-class",
        "--class-name",
        "p/Recursive",
        "--method-name",
        "method",
        "--descriptor",
        "()V",
        "--format",
        "json",
    ]);
    assert_eq!(
        status(&output),
        EXIT_INCOMPLETE,
        "a stopped recovery is never success: {}",
        stderr_text(&output)
    );
    let document = stdout_json(&output);
    let recovery = &document["recovered"]["recovery"];
    assert_eq!(
        recovery["outcome"]["stopped"]["interrupted"]["code"],
        json!("jre_recursion_reentry"),
        "the walk re-entered its own loop's header: {document}"
    );
    assert_eq!(
        recovery["outcome"]["stopped"]["interrupted"]["at"],
        json!(2),
        "and the stop names the block it was about to re-enter: {document}"
    );
    assert_eq!(
        recovery["execution"]["status"],
        json!("partial"),
        "a stop is a non-Complete execution plane: {document}"
    );
    assert_eq!(recovery["content"], json!("not_produced"));
    assert_eq!(recovery["text"], json!(""), "a stop hands out no artifact");
    let diagnostic = recovery["diagnostics"]
        .as_array()
        .expect("the stop carries diagnostics")
        .iter()
        .find(|diagnostic| diagnostic["code"] == json!("jre_recursion_reentry"))
        .unwrap_or_else(|| panic!("the stop states its reason: {document}"));
    assert_eq!(diagnostic["severity"], json!("error"));
    let message = diagnostic["message"]
        .as_str()
        .expect("a diagnostic states a message");
    assert!(
        message.contains("BCI 2") && message.contains("re-entered"),
        "the diagnosis names the block and the fact that it re-entered it: {message}"
    );
}

// ---------------------------------------------------------------------------------------------
// `re-express-string-concatenation`: the deep chain that used to abort the process
// ---------------------------------------------------------------------------------------------

/// The chain length the generated deep fixture is pinned at.
///
/// The length is the check's **parameter** (the review's interval is 1536/2048 and 1024 completed
/// before the fix). This file has no digest crate to hash with — the library entry's copy of the same
/// writer (`tests/p3_concat_conversion.rs`) pins the blake3 digest of the bytes, and the shape is
/// asserted from the bytes themselves here; the change's verification records the SHA-256 of the
/// bytes both writers produce at N = 2048
/// (`00ef5b955567f1ec1af860a30acf68603eb515525495c109a610da427622450e`), which is also the input the
/// pre-fix abort was measured on.
const DEEP_APPENDS: usize = 2048;

/// One hand-assembled class whose only member is a straight line of `append` calls.
///
/// The bytes of this writer and `tests/p3_concat_conversion.rs`'s `deep_concat_fixture` are
/// identical (`DEEP_DIGEST` is asserted in both files). The shape mirrors the review's
/// `DeepConcat2048` reproduction — `new StringBuilder().append(s)` repeated N times, then
/// `toString()`, on the same class, with the same overload — and it is **generated** rather than
/// compiled because the chain length is the parameter of the check and `javac` needs an enlarged
/// compiler stack (`-J-Xss64m`) for these inputs: a way to produce bytes, not a dependency a
/// regression may have. There is no branch, so the version-52 body needs no `StackMapTable`.
fn deep_concat_fixture(appends: usize) -> Vec<u8> {
    fn u16b(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    fn u32b(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    fn utf8(pool: &mut Vec<u8>, text: &[u8]) {
        pool.push(1);
        u16b(
            pool,
            u16::try_from(text.len()).expect("fixture name fits u16"),
        );
        pool.extend_from_slice(text);
    }

    let mut pool: Vec<u8> = Vec::new();
    utf8(&mut pool, b"p/DeepConcat"); // 1
    pool.push(7); // 2: Class 1
    u16b(&mut pool, 1);
    utf8(&mut pool, b"java/lang/Object"); // 3
    pool.push(7); // 4: Class 3
    u16b(&mut pool, 3);
    utf8(&mut pool, b"method"); // 5
    utf8(&mut pool, b"(Ljava/lang/String;)Ljava/lang/String;"); // 6
    utf8(&mut pool, b"Code"); // 7
    utf8(&mut pool, b"java/lang/StringBuilder"); // 8
    pool.push(7); // 9: Class 8
    u16b(&mut pool, 8);
    utf8(&mut pool, b"<init>"); // 10
    utf8(&mut pool, b"()V"); // 11
    pool.push(12); // 12: NameAndType 10, 11
    u16b(&mut pool, 10);
    u16b(&mut pool, 11);
    pool.push(10); // 13: Methodref 9, 12
    u16b(&mut pool, 9);
    u16b(&mut pool, 12);
    utf8(&mut pool, b"append"); // 14
    utf8(&mut pool, b"(Ljava/lang/String;)Ljava/lang/StringBuilder;"); // 15
    pool.push(12); // 16: NameAndType 14, 15
    u16b(&mut pool, 14);
    u16b(&mut pool, 15);
    pool.push(10); // 17: Methodref 9, 16
    u16b(&mut pool, 9);
    u16b(&mut pool, 16);
    utf8(&mut pool, b"toString"); // 18
    utf8(&mut pool, b"()Ljava/lang/String;"); // 19
    pool.push(12); // 20: NameAndType 18, 19
    u16b(&mut pool, 18);
    u16b(&mut pool, 19);
    pool.push(10); // 21: Methodref 9, 20
    u16b(&mut pool, 9);
    u16b(&mut pool, 20);

    let mut code: Vec<u8> = Vec::with_capacity(11 + 4 * appends);
    code.push(0xbb); // 0: new
    code.extend_from_slice(&9_u16.to_be_bytes());
    code.push(0x59); // 3: dup
    code.push(0xb7); // 4: invokespecial <init>()V
    code.extend_from_slice(&13_u16.to_be_bytes());
    for _ in 0..appends {
        code.push(0x2a); // aload_0
        code.push(0xb6); // invokevirtual append(Ljava/lang/String;)
        code.extend_from_slice(&17_u16.to_be_bytes());
    }
    code.push(0xb6); // invokevirtual toString()Ljava/lang/String;
    code.extend_from_slice(&21_u16.to_be_bytes());
    code.push(0xb0); // areturn

    let mut output = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0); // minor
    u16b(&mut output, 52); // major: Java 8
    u16b(&mut output, 22); // constant_pool_count: the 21 entries above
    output.extend_from_slice(&pool);
    u16b(&mut output, 0x0021); // public super
    u16b(&mut output, 2); // this_class
    u16b(&mut output, 4); // super_class
    u16b(&mut output, 0); // interfaces
    u16b(&mut output, 0); // fields
    u16b(&mut output, 1); // methods
    u16b(&mut output, 0x0009); // public static
    u16b(&mut output, 5); // name → "method"
    u16b(&mut output, 6); // descriptor
    u16b(&mut output, 1); // attributes
    u16b(&mut output, 7); // "Code"
    let mut attribute = Vec::new();
    u16b(&mut attribute, 2); // max_stack
    u16b(&mut attribute, 1); // max_locals: the `String` parameter
    u32b(
        &mut attribute,
        u32::try_from(code.len()).expect("the fixture body fits u32"),
    );
    attribute.extend_from_slice(&code);
    u16b(&mut attribute, 0); // exception table
    u16b(&mut attribute, 0); // code attributes
    u32b(
        &mut output,
        u32::try_from(attribute.len()).expect("the fixture attribute fits u32"),
    );
    output.extend_from_slice(&attribute);
    u16b(&mut output, 0); // class attributes
    output
}

/// The one `recover` invocation every deep-chain case makes, against a named binary.
///
/// The binary is a parameter because the debug and the optimized entry are two *build boundaries*
/// of the same code, and the acceptance names both: the explicit release gate runs the same
/// assertions against `target/release/jarde-cli`. No case sets `RUST_MIN_STACK` or moves the work
/// to a thread of its own: the check has to hold on the default stack of the process the CLI is.
fn recover_deep_chain(binary: &Path, class: &Path, extra: &[&str]) -> Output {
    let mut arguments: Vec<&str> = vec![
        "recover",
        "--input",
        path_of(class),
        "--policy",
        "single-class",
        "--class-name",
        "p/DeepConcat",
        "--method-name",
        "method",
        "--descriptor",
        "(Ljava/lang/String;)Ljava/lang/String;",
        "--format",
        "json",
    ];
    arguments.extend_from_slice(extra);
    Command::new(binary)
        .args(&arguments)
        .env_remove("RUST_MIN_STACK")
        .output()
        .unwrap_or_else(|error| panic!("run {}: {error}", binary.display()))
}

/// The deep chain answers through the CLI: a report on standard output with the chain's own text
/// under the default budget, and the existing stop shape under a bound that stops the run — in both
/// cases a status rather than a signal, and never an empty standard output.
///
/// The budget case is `output_bytes=17000`, which leaves the run without the payload it needs: the
/// report is the existing `not_produced` stop (exit 4) rather than a crash. A bound that stops the
/// **emission itself** cannot be delivered as a document by this CLI for a reason that is older than
/// this change: the request's `output_bytes` funds the rendered document too, so a run that consumed
/// its bound mid-emission leaves nothing for the document, and the adapter answers with its
/// `budget_exceeded` refusal instead (exit 2, nothing on standard output). That mid-emission stop —
/// with every part of the chain built and released — is asserted where it can be observed as a
/// report: `tests/p3_concat_conversion.rs`, through `Engine::recover_method`.
fn the_deep_chain_answers(binary: &Path) {
    let temp = TempDir::new();
    let fixture = deep_concat_fixture(DEEP_APPENDS);
    // The bytes this writer produces, read back out of the class file: the fixed pool and member
    // shell, the `new`/`dup`/`<init>` prologue, one `aload_0; invokevirtual append` per part and the
    // `toString`/`areturn` tail. `tests/p3_concat_conversion.rs` writes the same bytes (its
    // `deep_concat_fixture` is this function again) and pins their digest.
    let code_len = 11 + 4 * DEEP_APPENDS;
    assert_eq!(
        fixture.len(),
        312 + code_len,
        "the class is the fixed pool and member shell plus the body"
    );
    let code = &fixture[fixture.len() - 6 - code_len..fixture.len() - 6];
    assert_eq!(
        &code[..7],
        &[0xbb, 0x00, 0x09, 0x59, 0xb7, 0x00, 0x0d],
        "the allocation, its copy and the constructor"
    );
    for part in 0..DEEP_APPENDS {
        assert_eq!(
            &code[7 + 4 * part..11 + 4 * part],
            &[0x2a, 0xb6, 0x00, 0x11],
            "part {part} is one `append(Ljava/lang/String;)` on the parameter"
        );
    }
    assert_eq!(
        &code[code_len - 4..],
        &[0xb6, 0x00, 0x15, 0xb0],
        "the `toString` the chain ends in and the `areturn` that consumes it"
    );
    let class = temp.write("DeepConcat.class", &fixture);

    // Normal completion: the chain is presentable, so the run presents it.
    let output = recover_deep_chain(binary, &class, &[]);
    assert_eq!(
        status(&output),
        EXIT_COMPLETE,
        "{}: {}",
        binary.display(),
        stderr_text(&output)
    );
    let document = stdout_json(&output);
    let recovery = &document["recovered"]["recovery"];
    assert_eq!(recovery["content"], json!("contains_statements"));
    assert_eq!(recovery["representation"], json!("java"));
    let text = recovery["text"]
        .as_str()
        .expect("the artifact is a string, not a stop");
    assert_eq!(
        text.matches(" + ").count(),
        DEEP_APPENDS - 1,
        "one part per `append`, joined by the `+`s the chain performs"
    );
    assert_eq!(text.matches("arg0").count(), DEEP_APPENDS);

    // A `--budget` stop: the existing stop shape, the report on standard output and no artifact.
    // A `--budget` stop: the run never reports success and never hands out an artifact it did not
    // produce. The *kind* of stop a bound this tight lands on is deliberately not pinned here any
    // more: the redundant class read this arm was calibrated against is gone (the read path now
    // materializes a class once), and with it the bound that used to stop inside the recovery of
    // this body — a sweep of 2,000–60,000 bytes lands on either an unfinished search or a run the
    // document cannot fit, both of which exit non-zero. The precise shape of a short-bound stop is
    // asserted where it is still reachable: `tests/p3_concat_conversion.rs`'s
    // `a_deep_chain_under_a_short_output_bound_stops_without_an_artifact`.
    let output = recover_deep_chain(binary, &class, &["--budget", "output_bytes=17000"]);
    assert_ne!(
        status(&output),
        EXIT_COMPLETE,
        "a run under a bound it cannot afford is never success: {}",
        stderr_text(&output)
    );
    if let Ok(document) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
        let recovery = &document["recovered"]["recovery"];
        assert_ne!(
            recovery["content"],
            json!("contains_statements"),
            "and it never states recovered statements it could not write: {document}"
        );
        assert_eq!(
            recovery["text"],
            json!(""),
            "a stop hands out no artifact: {document}"
        );
        assert!(
            recovery["execution"]["status"] != json!("complete"),
            "a stopped run is a non-Complete execution plane: {document}"
        );
    } else {
        assert_eq!(
            status(&output),
            EXIT_USAGE,
            "a refusal that writes no document is a usage failure: {}",
            stderr_text(&output)
        );
    }
}

/// The debug entry — the default gate — on the input the pre-fix printer aborted on.
#[test]
fn a_deep_concatenation_chain_answers_in_a_subprocess() {
    the_deep_chain_answers(Path::new(BIN));
}

/// The optimized entry of the same code: the explicit release gate, run by hand.
///
/// ```text
/// cargo build --release -p jarde-cli --locked
/// cargo test -p jarde-cli --test task_cli --locked -- --ignored the_deep_chain_answers_in_the_optimized_build
/// ```
///
/// The review measured two *different* build boundaries of this input (debug stopped at 1536/2048
/// while the optimized build completed 2048–15000 on the main thread), so neither build's answer
/// stands for the other's: this is the second one, and it is `#[ignore]`d because a release build is
/// an explicit gate and not part of `cargo test`.
#[test]
#[ignore = "explicit gate: build the optimized CLI with `cargo build --release -p jarde-cli --locked`"]
fn the_deep_chain_answers_in_the_optimized_build() {
    the_deep_chain_answers(&optimized_cli());
}

/// The optimized CLI the two explicit release gates run: the same binary, resolved the same way.
fn optimized_cli() -> std::path::PathBuf {
    let mut binary = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    binary.pop();
    binary.pop();
    binary.push("target");
    binary.push("release");
    binary.push("jarde-cli");
    assert!(
        binary.is_file(),
        "the optimized CLI is not built: run `cargo build --release -p jarde-cli --locked` first \
         (expected {})",
        binary.display()
    );
    binary
}

// ---------------------------------------------------------------------------------------------
// The same input through the bulk entry: one operation, two workers, the ordinary worker stack
// ---------------------------------------------------------------------------------------------

/// The one `export` invocation the bulk deep-chain cases make, against a named binary.
///
/// `--jobs 2` is what this gate is about. The bulk operation runs a class task on the **calling**
/// thread only when its effective worker count is one, so the same input under `--jobs 1` would be
/// presented on the process's own (larger) stack and would say nothing about the workers this change
/// added. Two workers put this class's task on a worker thread the library created with
/// `std::thread::Builder::new()` and no `stack_size`, and the stream's own `header` record is
/// asserted below to have really run with two.
///
/// No case sets `RUST_MIN_STACK` — the variable `Builder::new()` reads for that default — so what the
/// chain has to fit in is the ordinary stack of an ordinary worker, not one a case enlarged.
fn export_deep_chain(binary: &Path, class: &Path, output: &Path) -> Output {
    Command::new(binary)
        .args([
            "export",
            "--input",
            path_of(class),
            "--policy",
            "single-class",
            "--jobs",
            "2",
            "--output",
            path_of(output),
            "--format",
            "jsonl",
        ])
        .env_remove("RUST_MIN_STACK")
        .output()
        .unwrap_or_else(|error| panic!("run {}: {error}", binary.display()))
}

/// Every record of one `export` stream, in file order.
///
/// The framing is checked before the lines are: a stream that does not end at a record boundary is
/// not read as a list of records that happen to be followed by a fragment.
fn jsonl_records(path: &Path) -> Vec<Value> {
    let bytes = fs::read(path).expect("the stream file is readable");
    assert!(
        bytes.ends_with(b"\n"),
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
                    "a stream line is not one JSON document ({error}): {}",
                    String::from_utf8_lossy(line)
                )
            })
        })
        .collect()
}

/// The bulk entry presents the deep chain too: one `export` process, two workers, one whole method
/// record carrying the chain the single-method entry presents.
///
/// The assertion this case exists for is that the run **ends**: the chain is presented on a worker
/// stack and the process answers with a status and a stream instead of aborting. The stream is then
/// read for the same text the `recover` gate reads, so "the bulk entry's chain" is the chain and not
/// a shorter one that happens to fit.
fn the_deep_chain_reaches_a_bulk_worker(binary: &Path) {
    let temp = TempDir::new();
    let class = temp.write("DeepConcat.class", &deep_concat_fixture(DEEP_APPENDS));
    let stream = temp.join("deep.jsonl");
    let ran = export_deep_chain(binary, &class, &stream);
    assert_eq!(
        status(&ran),
        EXIT_COMPLETE,
        "{}: a deep chain has to be presented and delivered, not aborted: {}",
        binary.display(),
        stderr_text(&ran)
    );

    let records = jsonl_records(&stream);
    let header = records.first().expect("the stream has a header");
    assert_eq!(header["kind"], json!("header"), "{header}");
    assert_eq!(header["limits"]["workers_requested"], json!(2), "{header}");
    assert_eq!(
        header["limits"]["workers_effective"],
        json!(2),
        "the class task ran on a worker thread of the operation rather than on this process's own \
         stack, which is the stack this gate is about: {header}"
    );

    let last = records.last().expect("the stream has a final record");
    assert_eq!(last["kind"], json!("final"), "{last}");
    let summary = &last["summary"];
    assert_eq!(
        summary["execution"]["status"],
        json!("complete"),
        "{summary}"
    );
    assert_eq!(summary["traversal_complete"], json!(true), "{summary}");
    assert_eq!(summary["classes_prepared"], json!(1), "{summary}");
    assert_eq!(summary["methods_declared"], json!(1), "{summary}");
    assert_eq!(summary["methods_delivered"], json!(1), "{summary}");

    let method = records
        .iter()
        .find(|record| record["kind"] == json!("method"))
        .unwrap_or_else(|| panic!("the stream states the class's one method: {records:?}"));
    assert_eq!(method["delivery"]["state"], json!("recovered"), "{method}");
    let name: Vec<u8> = serde_json::from_value(method["method"]["name"].clone())
        .expect("the record's raw name is a byte array");
    let descriptor: Vec<u8> = serde_json::from_value(method["method"]["descriptor"].clone())
        .expect("the record's raw descriptor is a byte array");
    assert_eq!(name, b"method".as_slice(), "{method}");
    assert_eq!(
        descriptor,
        b"(Ljava/lang/String;)Ljava/lang/String;".as_slice(),
        "{method}"
    );
    let recovery = &method["delivery"]["recovery"];
    assert_eq!(
        recovery["content"],
        json!("contains_statements"),
        "{method}"
    );
    let text = recovery["text"]
        .as_str()
        .unwrap_or_else(|| panic!("a presented chain is text: {method}"));
    assert_eq!(
        text.matches(" + ").count(),
        DEEP_APPENDS - 1,
        "one part per `append`, joined by the `+`s the chain performs"
    );
    assert_eq!(
        text.matches("arg0").count(),
        DEEP_APPENDS,
        "each part is the value its own `append` read"
    );
}

/// The debug entry of the bulk command — the default gate — on the input the pre-fix printer aborted
/// on.
#[test]
fn a_deep_concatenation_chain_is_presented_by_a_bulk_worker() {
    the_deep_chain_reaches_a_bulk_worker(Path::new(BIN));
}

/// The optimized entry of the same case: the second explicit release gate, run by hand.
///
/// ```text
/// cargo build --release -p jarde-cli --locked
/// cargo test -p jarde-cli --test task_cli --locked -- --ignored the_deep_chain_reaches_a_bulk_worker_in_the_optimized_build
/// ```
///
/// The two builds are two boundaries of the same code — the review measured *different* answers from
/// them for this input on the main thread — so the worker stack is stated in both.
#[test]
#[ignore = "explicit gate: build the optimized CLI with `cargo build --release -p jarde-cli --locked`"]
fn the_deep_chain_reaches_a_bulk_worker_in_the_optimized_build() {
    the_deep_chain_reaches_a_bulk_worker(&optimized_cli());
}
