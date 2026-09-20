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
    ArtifactInput, ArtifactSnapshot, BodyRef, Budget, BudgetOverride, ClassRef, ClassViewRequest,
    ConsumerKind, ConsumerSchema, Engine, EnvironmentPolicy, EnvironmentRequest, LayoutMode,
    Limits, LoadRoot, LoaderId, MethodOperationRequest, MethodRef, MultiReleasePolicy,
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
