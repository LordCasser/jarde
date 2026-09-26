//! P1 structural-XRef golden replays.
//!
//! Each file under `tests/fixtures/p1-golden/` records one representative input — the
//! fixture name, its provenance and the blake3 digest of the bytes the builder produces —
//! and, for every replay, the complete request or runtime view plus the complete report
//! the public entry point returned for it. These tests rebuild the input through the same
//! builder, replay the recorded request through `Engine::query` (or
//! `Engine::select_multi_release`), and compare the serialized report field by field.
//!
//! The only normalization is deleting `elapsed_millis`: it is the wall-clock number every
//! usage snapshot carries and not part of the query semantics. Origins, coverage ranges,
//! diagnostics, `via` paths, complete symbols (owner/name/descriptor), BCI/opcode/CP index
//! and spans, and item order are compared exactly as published. The files never
//! regenerate themselves: a missing, changed or unreadable golden is a test failure, not
//! an update.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use serde_json::Value;
use std::io::{Cursor, Write};
use std::path::PathBuf;

const STORE: u16 = 0;
const DEFLATE: u16 = 8;

/// Query limits generous enough that no golden replay can stop for a budget reason.
///
/// Every dimension is stated: a `..Limits::default()` tail would quietly grant zero to the
/// derived-work dimensions (`class_headers`, `method_bodies`, `ir_items`, `ir_edges`,
/// `analysis_steps`, `normalization_clones`, `dependency_depth`), and a structural replay
/// that legitimately consumes one of them — a `Signature` grammar parse charges
/// `analysis_steps` per node — would then stop partial instead of replaying complete.
fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 24,
        archive_entries: 10_000,
        entry_bytes: 1 << 24,
        read_bytes: 1 << 24,
        class_bytes: 1 << 24,
        attribute_bytes: 1 << 24,
        code_bytes: 1 << 24,
        result_items: 10_000,
        output_bytes: 1 << 24,
        class_headers: 10_000,
        method_bodies: 10_000,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn u16b(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn u32b(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

// ---------------------------------------------------------------------------
// Hand-built class fixture builder
// ---------------------------------------------------------------------------

/// Constant-pool builder: entries are appended in order and keep their 1-based indexes,
/// so the golden records the exact entries the bytes really hold.
#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    fn push(&mut self, entry: Vec<u8>) -> u16 {
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("fixture pool fits u16")
    }

    fn utf8(&mut self, text: &[u8]) -> u16 {
        let mut entry = vec![1];
        u16b(
            &mut entry,
            u16::try_from(text.len()).expect("text fits u16"),
        );
        entry.extend_from_slice(text);
        self.push(entry)
    }

    fn class(&mut self, name: u16) -> u16 {
        let mut entry = vec![7];
        u16b(&mut entry, name);
        self.push(entry)
    }

    fn string(&mut self, value: u16) -> u16 {
        let mut entry = vec![8];
        u16b(&mut entry, value);
        self.push(entry)
    }

    fn integer(&mut self, value: i32) -> u16 {
        let mut entry = vec![3];
        entry.extend_from_slice(&value.to_be_bytes());
        self.push(entry)
    }

    fn name_and_type(&mut self, name: u16, descriptor: u16) -> u16 {
        let mut entry = vec![12];
        u16b(&mut entry, name);
        u16b(&mut entry, descriptor);
        self.push(entry)
    }

    /// `Fieldref` (9), `Methodref` (10) or `InterfaceMethodref` (11).
    fn member(&mut self, tag: u8, class: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![tag];
        u16b(&mut entry, class);
        u16b(&mut entry, name_and_type);
        self.push(entry)
    }

    fn declared(&self) -> u16 {
        u16::try_from(self.entries.len() + 1).expect("fixture pool fits u16")
    }

    fn bytes(&self) -> Vec<u8> {
        self.entries.iter().flatten().copied().collect()
    }
}

struct FieldSpec {
    access: u16,
    name: u16,
    descriptor: u16,
}

struct MethodSpec {
    access: u16,
    name: u16,
    descriptor: u16,
    attributes: Vec<(u16, Vec<u8>)>,
}

/// One `Code` attribute body with no exception table and no nested attributes.
fn code_body(max_stack: u16, max_locals: u16, code: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    u16b(&mut body, max_stack);
    u16b(&mut body, max_locals);
    u32b(
        &mut body,
        u32::try_from(code.len()).expect("fixture code fits u32"),
    );
    body.extend_from_slice(code);
    u16b(&mut body, 0);
    u16b(&mut body, 0);
    body
}

/// Assembles a class file around a finished constant pool.
#[allow(clippy::too_many_arguments)]
fn assemble_class(
    pool: &Pool,
    major: u16,
    access: u16,
    this_class: u16,
    super_class: u16,
    interfaces: &[u16],
    fields: &[FieldSpec],
    methods: &[MethodSpec],
    class_attributes: &[(u16, Vec<u8>)],
) -> Vec<u8> {
    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0);
    u16b(&mut bytes, major);
    u16b(&mut bytes, pool.declared());
    bytes.extend_from_slice(&pool.bytes());
    u16b(&mut bytes, access);
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, super_class);
    u16b(
        &mut bytes,
        u16::try_from(interfaces.len()).expect("interfaces fit u16"),
    );
    for interface in interfaces {
        u16b(&mut bytes, *interface);
    }
    u16b(
        &mut bytes,
        u16::try_from(fields.len()).expect("fields fit u16"),
    );
    for field in fields {
        u16b(&mut bytes, field.access);
        u16b(&mut bytes, field.name);
        u16b(&mut bytes, field.descriptor);
        u16b(&mut bytes, 0);
    }
    u16b(
        &mut bytes,
        u16::try_from(methods.len()).expect("methods fit u16"),
    );
    for method in methods {
        u16b(&mut bytes, method.access);
        u16b(&mut bytes, method.name);
        u16b(&mut bytes, method.descriptor);
        u16b(
            &mut bytes,
            u16::try_from(method.attributes.len()).expect("method attributes fit u16"),
        );
        for (name, content) in &method.attributes {
            u16b(&mut bytes, *name);
            u32b(
                &mut bytes,
                u32::try_from(content.len()).expect("attribute content fits u32"),
            );
            bytes.extend_from_slice(content);
        }
    }
    u16b(
        &mut bytes,
        u16::try_from(class_attributes.len()).expect("class attributes fit u16"),
    );
    for (name, content) in class_attributes {
        u16b(&mut bytes, *name);
        u32b(
            &mut bytes,
            u32::try_from(content.len()).expect("attribute content fits u32"),
        );
        bytes.extend_from_slice(content);
    }
    bytes
}

fn zip_bytes(entries: &[(&[u8], &[u8], u16)]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data, method) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(*method))
                .start()
                .unwrap();
            if *method == DEFLATE {
                let encoder =
                    flate2::write::DeflateEncoder::new(&mut entry, flate2::Compression::default());
                let mut writer = config.wrap(encoder);
                writer.write_all(data).unwrap();
                let (encoder, descriptor) = writer.finish().unwrap();
                encoder.finish().unwrap();
                entry.finish(descriptor).unwrap();
            } else {
                let mut writer = config.wrap(&mut entry);
                writer.write_all(data).unwrap();
                let (_, descriptor) = writer.finish().unwrap();
                entry.finish(descriptor).unwrap();
            }
        }
        archive.finish().unwrap();
    }
    output.into_inner()
}

/// The code golden input: one used invocation, one `Methodref` no consumer uses, one
/// `ldc` of a string and one of an integer, in one method body.
fn code_fixture() -> Vec<u8> {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/GoldenCode");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let target_name = pool.utf8(b"p/Target");
    let target_class = pool.class(target_name);
    let unused_name = pool.utf8(b"p/Unused");
    let unused_class = pool.class(unused_name);
    let run_name = pool.utf8(b"run");
    let void_descriptor = pool.utf8(b"()V");
    let run_nat = pool.name_and_type(run_name, void_descriptor);
    let target_run = pool.member(10, target_class, run_nat);
    let never_name = pool.utf8(b"never");
    let never_nat = pool.name_and_type(never_name, void_descriptor);
    pool.member(10, unused_class, never_nat);
    let hello = pool.utf8(b"hello");
    let hello_string = pool.string(hello);
    let seven = pool.integer(7);
    let main_name = pool.utf8(b"main");
    let code_name = pool.utf8(b"Code");

    // 0: invokevirtual p/Target.run; 3: ldc "hello"; 5: ldc 7; 7: return.
    let mut code = vec![0xb6];
    u16b(&mut code, target_run);
    code.push(0x12);
    code.push(u8::try_from(hello_string).expect("pool index fits one byte"));
    code.push(0x12);
    code.push(u8::try_from(seven).expect("pool index fits one byte"));
    code.push(0xb1);

    assemble_class(
        &pool,
        52,
        0x0021,
        this_class,
        object_class,
        &[],
        &[],
        &[MethodSpec {
            access: 0x0009,
            name: main_name,
            descriptor: void_descriptor,
            attributes: vec![(code_name, code_body(2, 1, &code))],
        }],
        &[],
    )
}

/// The metadata golden input: superclass, interface, field descriptor, a method
/// `Signature` and one class-level annotation whose type exists nowhere else.
fn metadata_fixture() -> Vec<u8> {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/GoldenMeta");
    let this_class = pool.class(class_name);
    let sup_name = pool.utf8(b"p/Sup");
    let sup_class = pool.class(sup_name);
    let iface_name = pool.utf8(b"p/Iface");
    let iface_class = pool.class(iface_name);
    let field_name = pool.utf8(b"f");
    let field_descriptor = pool.utf8(b"Lp/Sup;");
    let method_name = pool.utf8(b"m");
    let method_descriptor = pool.utf8(b"(Lp/Arg;)Lp/Ret;");
    let signature_text = pool.utf8(b"()Ljava/util/List<Ljava/lang/String;>;");
    let signature_name = pool.utf8(b"Signature");
    let annotations_name = pool.utf8(b"RuntimeVisibleAnnotations");
    let annotation_descriptor = pool.utf8(b"Lp/Ann;");

    let mut signature = Vec::new();
    u16b(&mut signature, signature_text);
    let mut annotations = Vec::new();
    u16b(&mut annotations, 1);
    u16b(&mut annotations, annotation_descriptor);
    u16b(&mut annotations, 0);

    assemble_class(
        &pool,
        52,
        0x0021,
        this_class,
        sup_class,
        &[iface_class],
        &[FieldSpec {
            access: 0x0001,
            name: field_name,
            descriptor: field_descriptor,
        }],
        &[MethodSpec {
            access: 0x0001,
            name: method_name,
            descriptor: method_descriptor,
            attributes: vec![(signature_name, signature)],
        }],
        &[(annotations_name, annotations)],
    )
}

/// One `p/Join` class of the multi-release golden input: the given version and superclass,
/// with a static `main` that calls `p/All.run()V`.
fn mr_class(major: u16, super_name: &[u8]) -> Vec<u8> {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/Join");
    let this_class = pool.class(class_name);
    let sup_name = pool.utf8(super_name);
    let sup_class = pool.class(sup_name);
    let all_name = pool.utf8(b"p/All");
    let all_class = pool.class(all_name);
    let run_name = pool.utf8(b"run");
    let void_descriptor = pool.utf8(b"()V");
    let run_nat = pool.name_and_type(run_name, void_descriptor);
    let all_run = pool.member(10, all_class, run_nat);
    let main_name = pool.utf8(b"main");
    let code_name = pool.utf8(b"Code");

    let mut code = vec![0xb8];
    u16b(&mut code, all_run);
    code.push(0xb1);

    assemble_class(
        &pool,
        major,
        0x0021,
        this_class,
        sup_class,
        &[],
        &[],
        &[MethodSpec {
            access: 0x0009,
            name: main_name,
            descriptor: void_descriptor,
            attributes: vec![(code_name, code_body(2, 1, &code))],
        }],
        &[],
    )
}

/// Same class bytes for every variant; only the version and superclass differ, so the
/// three physical entries are told apart by their origin and variant, never merged.
fn mr_fixture() -> Vec<u8> {
    let base = mr_class(52, b"p/BaseSup");
    let v11 = mr_class(55, b"p/V11Sup");
    let v17 = mr_class(61, b"p/V17Sup");
    zip_bytes(&[
        (
            b"META-INF/MANIFEST.MF",
            b"Manifest-Version: 1.0\r\nMulti-Release: true\r\n\r\n",
            STORE,
        ),
        (b"p/Join.class", &base, STORE),
        (b"META-INF/versions/11/p/Join.class", &v11, STORE),
        (b"META-INF/versions/17/p/Join.class", &v17, STORE),
    ])
}

/// The nested golden input: a WAR layout in which the same jar bytes sit at two
/// `WEB-INF/lib` entries, one STORED and one DEFLATED, each holding the same
/// `p/Dup.class` bytes under the same raw name.
fn nested_fixture() -> Vec<u8> {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/Dup");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let target_name = pool.utf8(b"p/Target");
    let target_class = pool.class(target_name);
    let run_name = pool.utf8(b"run");
    let void_descriptor = pool.utf8(b"()V");
    let run_nat = pool.name_and_type(run_name, void_descriptor);
    let target_run = pool.member(10, target_class, run_nat);
    let main_name = pool.utf8(b"main");
    let code_name = pool.utf8(b"Code");
    let mut code = vec![0xb8];
    u16b(&mut code, target_run);
    code.push(0xb1);
    let class = assemble_class(
        &pool,
        52,
        0x0021,
        this_class,
        object_class,
        &[],
        &[],
        &[MethodSpec {
            access: 0x0009,
            name: main_name,
            descriptor: void_descriptor,
            attributes: vec![(code_name, code_body(2, 1, &code))],
        }],
        &[],
    );
    let inner = zip_bytes(&[(b"p/Dup.class", &class, STORE)]);
    zip_bytes(&[
        (b"WEB-INF/lib/a.jar", &inner, STORE),
        (b"WEB-INF/lib/b.jar", &inner, DEFLATE),
    ])
}

/// Bytes of the one fixture a golden names.
fn fixture_bytes(name: &str) -> Vec<u8> {
    match name {
        "code" => code_fixture(),
        "metadata" => metadata_fixture(),
        "bootstrap" => {
            include_bytes!("fixtures/b2-bootstrap-descriptor/v8/LambdaSample.class").to_vec()
        }
        "multi-release" => mr_fixture(),
        "nested" => nested_fixture(),
        other => panic!("golden fixture {other:?} has no builder"),
    }
}

// ---------------------------------------------------------------------------
// Golden loading, replay and normalization
// ---------------------------------------------------------------------------

fn golden_path(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/p1-golden")
        .join(file)
}

fn golden(file: &str) -> Value {
    let path = golden_path(file);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "golden {} is missing or unreadable: {error}; goldens are checked-in expectations, \
             they are never generated by a test run",
            path.display()
        )
    });
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("golden {} is not valid JSON: {error}", path.display()))
}

fn replay_named<'a>(golden: &'a Value, name: &str) -> &'a Value {
    golden["replays"]
        .as_array()
        .expect("golden replays are an array")
        .iter()
        .find(|replay| replay["name"] == name)
        .unwrap_or_else(|| panic!("golden has no replay named {name:?}"))
}

/// Typed report of one named query replay.
fn replay_query(snapshot: &ArtifactSnapshot, golden: &Value, name: &str) -> QueryReport {
    match run_replayed(snapshot, replay_named(golden, name)) {
        Replayed::Query(report) => *report,
        other => panic!(
            "golden replay {name:?} is {}, not a query",
            replay_kind(&other)
        ),
    }
}

/// Typed report of one named multi-release selection replay.
fn replay_selection(
    snapshot: &ArtifactSnapshot,
    golden: &Value,
    name: &str,
) -> MultiReleaseViewReport {
    match run_replayed(snapshot, replay_named(golden, name)) {
        Replayed::MultiRelease(report) => *report,
        other => panic!(
            "golden replay {name:?} is {}, not a multi-release selection",
            replay_kind(&other)
        ),
    }
}

/// Bytes of the one constant-pool entry an item describes.
///
/// A bootstrap/condy fact records the entry it was read from as its class-file span, so the
/// entry's own tag and payload can be read back out of the fixture instead of being
/// restated by the test.
fn described_pool_entry<'a>(bytes: &'a [u8], item: &XrefItem) -> &'a [u8] {
    let span = item
        .evidence
        .span
        .clone()
        .expect("every bootstrap item describes one constant-pool entry with a span");
    let start = usize::try_from(span.start).expect("offset fits usize");
    let end = usize::try_from(span.start + span.length).expect("end offset fits usize");
    &bytes[start..end]
}

/// Every replay a golden must record, in file order.
///
/// This list is part of the expectation, not a summary of the file: `assert_golden_replays`
/// compares it with what the JSON really holds, so deleting, renaming or adding a replay
/// fails even though the remaining replays would still replay correctly. Each entry also
/// names the acceptance point that replay carries, and the per-golden tests below look the
/// core entries up by name, so the recorded evidence cannot quietly shrink.
fn expected_replays(file: &str) -> &'static [&'static str] {
    match file {
        // A01/A02: the X0/X1 unused-`Methodref` contrast, the call coordinates and the
        // literal value of the same fixture.
        "code.json" => &[
            "x0-unused-methodref-candidate",
            "x1-unused-methodref-is-not-a-call",
            "x1-invocation-coordinates",
            "literal-value-string",
        ],
        // A03: single-category requests next to combined ones on the same positions.
        "metadata.json" => &[
            "type-single-sup",
            "type-and-annotation-combined-sup",
            "signature-single-string",
            "annotation-single-ann",
            "annotation-and-signature-combined-ann",
        ],
        // A04/A05: the implementation handle with its `via` path, the descriptor type of the
        // bootstrap handle, the bootstrap-only control and the dynamic use-site.
        "bootstrap.json" => &[
            "bootstrap-implementation-handle",
            "type-methodtype-from-the-bootstrap-descriptor",
            "bootstrap-only-methodtype-is-not-a-type-fact",
            "invocation-dynamic-site",
        ],
        // A06: three runtime views next to one physical query over the same snapshot.
        "multi-release.json" => &[
            "select-java-8",
            "select-java-11",
            "select-java-17",
            "query-all-physical-variants",
        ],
        // A07/A08: the two identical nested copies and their distinct origins.
        "nested.json" => &["query-tree-two-origins"],
        other => panic!("golden {other:?} has no expected replay list"),
    }
}

/// Opens the fixture a golden documents, after checking the bytes still hash to the
/// recorded digest. A changed builder must fail here instead of silently rewriting the
/// expectation.
fn open_fixture(golden: &Value) -> ArtifactSnapshot {
    let fixture = &golden["fixture"];
    let name = fixture["name"].as_str().expect("fixture name");
    let bytes = fixture_bytes(name);
    let digest = blake3::hash(&bytes).to_hex().to_string();
    assert_eq!(
        digest,
        fixture["blake3"].as_str().expect("fixture digest"),
        "the {name} builder produces different bytes than the golden records"
    );
    assert_eq!(
        bytes.len() as u64,
        fixture["bytes"].as_u64().expect("fixture byte count"),
        "the {name} builder produces a different length than the golden records"
    );
    let mut budget = Budget::new(limits());
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .expect("golden fixture must open")
}

/// A replayed golden request and its typed report.
enum Replayed {
    Query(Box<QueryReport>),
    MultiRelease(Box<MultiReleaseViewReport>),
}

/// Replays one recorded request or runtime view through the public entry point.
fn run_replayed(snapshot: &ArtifactSnapshot, replay: &Value) -> Replayed {
    match replay["kind"].as_str().expect("replay kind") {
        "query" => {
            let request: QueryRequest = serde_json::from_value(replay["request"].clone())
                .expect("the golden records a replayable query request");
            let mut budget = Budget::new(limits());
            let report = Engine::new()
                .query(snapshot, &request, &mut budget)
                .expect("a golden query replay must return a report");
            Replayed::Query(Box::new(report))
        }
        "multi_release" => {
            let view: RuntimeView = serde_json::from_value(replay["view"].clone())
                .expect("the golden records a replayable runtime view");
            let mut budget = Budget::new(limits());
            let report = Engine::new()
                .select_multi_release(snapshot, &view, &mut budget)
                .expect("a golden selection replay must return a report");
            Replayed::MultiRelease(Box::new(report))
        }
        other => panic!("golden replay kind {other:?} is not replayable"),
    }
}

/// Removes non-semantic run values from a serialized report.
///
/// `elapsed_millis` is the one field whose value depends on wall-clock time; every other
/// field in these reports is a pure function of the fixture bytes and the request. The
/// function deletes exactly that key, at any depth, and nothing else.
fn normalize(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("elapsed_millis");
            for child in map.values_mut() {
                normalize(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                normalize(item);
            }
        }
        _ => {}
    }
}

fn replay_json(snapshot: &ArtifactSnapshot, replay: &Value) -> Value {
    let mut value = match run_replayed(snapshot, replay) {
        Replayed::Query(report) => serde_json::to_value(report).expect("report serializes"),
        Replayed::MultiRelease(report) => serde_json::to_value(report).expect("report serializes"),
    };
    normalize(&mut value);
    value
}

/// Fails when a value still carries a run-dependent field, so the checked-in golden cannot
/// silently become a moving target.
fn assert_normalized(value: &Value) {
    let mut normalized = value.clone();
    normalize(&mut normalized);
    assert_eq!(
        &normalized, value,
        "golden values must be recorded without run-dependent fields"
    );
}

fn assert_golden_replays(file: &str) {
    let golden = golden(file);
    // The recorded replay set is itself an expectation: a golden that lost, renamed or
    // gained a replay is no longer the file the acceptance index refers to.
    let replays = golden["replays"]
        .as_array()
        .expect("golden replays are an array");
    let names: Vec<&str> = replays
        .iter()
        .map(|replay| replay["name"].as_str().expect("replay name"))
        .collect();
    assert_eq!(
        names,
        expected_replays(file).to_vec(),
        "golden {file} must record exactly its expected replays, in order"
    );
    let snapshot = open_fixture(&golden);
    for replay in replays {
        let name = replay["name"].as_str().expect("replay name");
        assert_normalized(replay);
        let actual = replay_json(&snapshot, replay);
        assert_eq!(
            actual, replay["report"],
            "golden {file} replay {name:?} no longer matches the public entry point"
        );
    }
}

// ---------------------------------------------------------------------------
// Golden tests
// ---------------------------------------------------------------------------

#[test]
fn code_golden_replays_the_unused_pool_contrast_and_the_call_coordinates() {
    assert_golden_replays("code.json");
    let golden = golden("code.json");
    let snapshot = open_fixture(&golden);
    let bytes = fixture_bytes("code");

    // A01: the unused `Methodref` is a raw pool candidate in X0 and no call at all in X1.
    // Both replays are looked up by name, so the contrast the acceptance index claims
    // cannot be dropped by deleting or renaming one of the two.
    let x0 = replay_query(&snapshot, &golden, "x0-unused-methodref-candidate");
    assert_eq!(x0.items.len(), 1, "{:?}", x0.items);
    let candidate = &x0.items[0];
    assert_eq!(candidate.consumer, None);
    assert_eq!(candidate.operation, XrefOperation::ConstantPoolEntry);
    assert_eq!(candidate.derivation, XrefDerivation::ConstantPoolCandidate);
    assert_eq!(candidate.evidence.bci, None, "a pool entry has no BCI");
    assert_eq!(
        candidate.evidence.opcode, None,
        "a pool entry has no opcode"
    );
    assert!(candidate.evidence.via.is_empty());
    let span = candidate
        .evidence
        .span
        .clone()
        .expect("a raw candidate records the pool entry span");
    assert_eq!(
        bytes[usize::try_from(span.start).expect("offset fits usize")],
        10,
        "the recorded span starts at the CONSTANT_Methodref tag"
    );
    assert_eq!(span.length, 5, "tag byte plus two u16 indexes");
    assert!(matches!(x0.execution, ExecutionReport::Complete { .. }));

    let x1 = replay_query(&snapshot, &golden, "x1-unused-methodref-is-not-a-call");
    assert!(
        x1.items.is_empty(),
        "an unused Methodref must not become a call: {:?}",
        x1.items
    );
    assert_eq!(x1.coverage.scanned_items, 0);
    assert!(matches!(x1.execution, ExecutionReport::Complete { .. }));

    // A02: the call the bytes really contain keeps its own coordinates, and the recorded
    // span is the instruction at that class-file offset.
    let coordinates = replay_query(&snapshot, &golden, "x1-invocation-coordinates");
    assert_eq!(coordinates.items.len(), 1, "{:?}", coordinates.items);
    let call = &coordinates.items[0];
    assert_eq!(call.consumer, Some(ConsumerKind::Invocation));
    assert_eq!(call.operation, XrefOperation::InvokeVirtual);
    assert_eq!(call.certainty, XrefCertainty::Exact);
    let index = call
        .evidence
        .constant_pool_index
        .expect("an invocation names its constant-pool entry");
    assert_eq!(call.evidence.bci, Some(0));
    assert_eq!(call.evidence.opcode, Some(0xb6));
    let span = call.evidence.span.clone().expect("the instruction span");
    assert_eq!(span.length, 3);
    let start = usize::try_from(span.start).expect("offset fits usize");
    assert_eq!(
        bytes[start], 0xb6,
        "the span starts at the invokevirtual opcode"
    );
    assert_eq!(
        &bytes[start + 1..start + 3],
        &index.to_be_bytes(),
        "the operand is the constant-pool index the item names"
    );

    // The literal of the same fixture is a consumer fact of the `ldc` instruction, not a
    // pool candidate.
    let literal = replay_query(&snapshot, &golden, "literal-value-string");
    assert_eq!(literal.items.len(), 1, "{:?}", literal.items);
    assert_eq!(literal.items[0].consumer, Some(ConsumerKind::Constant));
    assert_eq!(literal.items[0].operation, XrefOperation::Ldc);
    assert_eq!(literal.items[0].evidence.opcode, Some(0x12));
}

#[test]
fn metadata_golden_replays_single_and_combined_categories() {
    assert_golden_replays("metadata.json");
    let golden = golden("metadata.json");
    let snapshot = open_fixture(&golden);

    // The same category answers the same facts alone and in a combination: combining the
    // schema must neither lose items nor add ones the single category did not produce.
    let single = replay_query(&snapshot, &golden, "type-single-sup");
    let combined = replay_query(&snapshot, &golden, "type-and-annotation-combined-sup");
    assert_eq!(single.items, combined.items);
    assert_eq!(
        single.coverage.scanned_items,
        combined.coverage.scanned_items
    );
    assert_eq!(
        combined.coverage.unsupported_categories,
        Vec::<ConsumerKind>::new()
    );
}

#[test]
fn bootstrap_golden_replays_the_compiled_lambda() {
    assert_golden_replays("bootstrap.json");
    let golden = golden("bootstrap.json");
    let snapshot = open_fixture(&golden);
    let bytes = fixture_bytes("bootstrap");

    // A04: the implementation handle is traced through the static argument position the
    // real `metafactory` declares, along the path from the use-site that reached it.
    let handle = replay_query(&snapshot, &golden, "bootstrap-implementation-handle");
    assert_eq!(handle.items.len(), 1, "{:?}", handle.items);
    let item = &handle.items[0];
    assert_eq!(item.consumer, Some(ConsumerKind::Bootstrap));
    assert_eq!(item.operation, XrefOperation::BootstrapArgument);
    assert_eq!(item.derivation, XrefDerivation::BootstrapEdge);
    assert_eq!(item.certainty, XrefCertainty::Exact);
    assert_eq!(item.resolution, QueryResolution::NotRequested);
    assert_eq!(item.evidence.bci, Some(0));
    assert_eq!(item.evidence.opcode, Some(0xba));
    assert_eq!(
        item.evidence.attribute,
        Some(ArchiveNameBytes(b"BootstrapMethods".to_vec()))
    );
    assert_eq!(item.evidence.via.len(), 2, "{item:?}");
    assert_eq!(
        (
            item.evidence.via[0].bootstrap_index,
            item.evidence.via[0].argument_index
        ),
        (None, None),
        "the first hop is the invokedynamic use-site itself: {item:?}"
    );
    assert_eq!(
        item.evidence.via[1].argument_index,
        Some(1),
        "the handle sits at the static argument position metafactory declares: {item:?}"
    );
    let described = described_pool_entry(&bytes, item);
    assert_eq!(
        described[0], 15,
        "the described entry is a CONSTANT_MethodHandle"
    );
    assert_eq!(described[1], 6, "REF_invokeStatic");

    // Decision 26: the type inside the bootstrap handle's descriptor is a `Type` fact with
    // the same edge, and the node's own symbol facts stay with `Bootstrap` alone.
    let typed = replay_query(
        &snapshot,
        &golden,
        "type-methodtype-from-the-bootstrap-descriptor",
    );
    assert_eq!(typed.items.len(), 1, "{:?}", typed.items);
    assert_eq!(typed.items[0].consumer, Some(ConsumerKind::Type));
    assert_eq!(typed.items[0].derivation, XrefDerivation::BootstrapEdge);
    assert_eq!(typed.items[0].evidence.via.len(), 2);
    assert_eq!(
        described_pool_entry(&bytes, &typed.items[0])[0],
        15,
        "the path ends at the bootstrap handle whose descriptor names the type"
    );

    let bootstrap_only = replay_query(
        &snapshot,
        &golden,
        "bootstrap-only-methodtype-is-not-a-type-fact",
    );
    assert!(
        bootstrap_only.items.is_empty(),
        "a descriptor type is not a bootstrap fact: {:?}",
        bootstrap_only.items
    );
    assert!(matches!(
        bootstrap_only.execution,
        ExecutionReport::Complete { .. }
    ));

    // The dynamic use-site is the invocation the class really contains, not a bootstrap edge.
    let site = replay_query(&snapshot, &golden, "invocation-dynamic-site");
    assert_eq!(site.items.len(), 1, "{:?}", site.items);
    assert_eq!(site.items[0].consumer, Some(ConsumerKind::Invocation));
    assert_eq!(site.items[0].operation, XrefOperation::InvokeDynamic);
    assert_eq!(site.items[0].evidence.opcode, Some(0xba));
    assert!(site.items[0].evidence.via.is_empty());
}

#[test]
fn multi_release_golden_replays_three_views_and_one_physical_query() {
    assert_golden_replays("multi-release.json");
    let golden = golden("multi-release.json");
    let snapshot = open_fixture(&golden);

    // A06: each view selects its own physical entry from the same snapshot.
    for (replay, expected) in [
        ("select-java-8", b"p/Join.class".as_slice()),
        (
            "select-java-11",
            b"META-INF/versions/11/p/Join.class".as_slice(),
        ),
        (
            "select-java-17",
            b"META-INF/versions/17/p/Join.class".as_slice(),
        ),
    ] {
        let report = replay_selection(&snapshot, &golden, replay);
        assert_eq!(
            selected_raw_name(&report, b"p/Join.class"),
            expected,
            "{replay}"
        );
        assert_eq!(report.verification, VerificationStatus::NotPerformed);
    }

    // The physical query on the same snapshot keeps all three variants side by side: it
    // answers the physical view, so the runtime selection of a view does not filter it.
    let query = replay_query(&snapshot, &golden, "query-all-physical-variants");
    assert_eq!(query.items.len(), 3, "{:?}", query.items);
    let variants: Vec<PhysicalVariant> = query
        .items
        .iter()
        .map(|item| {
            let (definition, _) = code_location(item);
            definition.variant.clone()
        })
        .collect();
    assert_eq!(
        variants,
        vec![
            PhysicalVariant::Base,
            PhysicalVariant::MultiRelease { version: 11 },
            PhysicalVariant::MultiRelease { version: 17 },
        ]
    );
    let entries: Vec<PhysicalEntryId> = query
        .items
        .iter()
        .map(|item| {
            let (definition, _) = code_location(item);
            definition
                .entry()
                .expect("the golden calls an archive entry")
                .clone()
        })
        .collect();
    assert_eq!(
        entries
            .iter()
            .map(|entry| entry.raw_name.0.clone())
            .collect::<Vec<_>>(),
        vec![
            b"p/Join.class".to_vec(),
            b"META-INF/versions/11/p/Join.class".to_vec(),
            b"META-INF/versions/17/p/Join.class".to_vec(),
        ],
        "the same logical class is three physical origins, in entry order"
    );
    for (index, left) in entries.iter().enumerate() {
        for right in entries.iter().skip(index + 1) {
            assert_ne!(left, right, "physical origins must not collapse");
        }
    }
}

#[test]
fn nested_golden_replays_stored_and_deflated_origins() {
    assert_golden_replays("nested.json");
    let golden = golden("nested.json");
    let snapshot = open_fixture(&golden);

    let query = replay_query(&snapshot, &golden, "query-tree-two-origins");
    assert_eq!(query.items.len(), 2, "{:?}", query.items);
    let definitions: Vec<PhysicalDefinitionId> = query
        .items
        .iter()
        .map(|item| code_location(item).0.clone())
        .collect();
    assert_ne!(definitions[0], definitions[1]);
    assert_eq!(
        definitions[0].class_bytes, definitions[1].class_bytes,
        "the two copies really are the same bytes"
    );
    for definition in &definitions {
        let Some(entry) = definition.entry() else {
            panic!("expected an archive entry origin, got {definition:?}");
        };
        assert_eq!(entry.raw_name.0, b"p/Dup.class");
        assert_eq!(entry.origin.steps.len(), 1);
    }

    // A08: the two copies sit in a STORED and a DEFLATED `WEB-INF/lib` entry of the same
    // root, so the distinct origins above are also the distinct nested compression methods
    // of a real WAR layout.
    let mut budget = Budget::new(limits());
    let tree = Engine::new()
        .enumerate_artifact_tree(&snapshot, &mut budget)
        .expect("the tree enumerates");
    let mut methods: Vec<u16> = tree
        .containers
        .iter()
        .filter(|container| container.origin.steps.is_empty())
        .flat_map(|container| container.entries.iter())
        .map(|entry| entry.compression_method)
        .collect();
    methods.sort_unstable();
    assert_eq!(methods, vec![STORE, DEFLATE]);
    let war_libraries: Vec<&LayoutNode> = tree
        .layout_nodes
        .iter()
        .filter(|node| node.kind == LayoutNodeKind::WarLibrary)
        .collect();
    assert_eq!(
        war_libraries.len(),
        2,
        "both WEB-INF/lib entries are physical WAR library evidence: {:?}",
        tree.layout_nodes
    );
    for node in &war_libraries {
        match &node.source {
            LayoutNodeSource::Archive {
                entry,
                child_container: Some(child),
            } => {
                assert!(entry.raw_name.0.starts_with(b"WEB-INF/lib/"), "{entry:?}");
                assert_eq!(child.steps.len(), 1, "the library has an established child");
            }
            other => panic!("expected an archive-sourced WAR library node, got {other:?}"),
        }
    }
    let root_entry = |ordinal: u64| {
        tree.containers
            .iter()
            .find(|container| container.origin.steps.is_empty())
            .and_then(|container| {
                container
                    .entries
                    .iter()
                    .find(|entry| entry.id.ordinal == ordinal)
            })
            .expect("the root entry is enumerated")
    };
    for definition in &definitions {
        let step = &definition.entry().expect("entry origin").origin.steps[0];
        let parent = root_entry(step.via_ordinal);
        assert_eq!(parent.id.raw_name.0, step.via_raw_name.0);
        assert!(
            parent.compression_method == STORE || parent.compression_method == DEFLATE,
            "the nested copy comes from a real STORED or DEFLATED entry"
        );
    }
}

fn replay_kind(replayed: &Replayed) -> &'static str {
    match replayed {
        Replayed::Query(_) => "query",
        Replayed::MultiRelease(_) => "multi_release",
    }
}

fn code_location(item: &XrefItem) -> (&PhysicalDefinitionId, u32) {
    match &item.source.location {
        Location::Code { method, bci } => (&method.owner, *bci),
        other => panic!("expected a code location, got {other:?}"),
    }
}

fn selected_raw_name(report: &MultiReleaseViewReport, logical: &[u8]) -> Vec<u8> {
    let container = report
        .containers
        .first()
        .expect("the fixture has one container");
    let selection = container
        .selections
        .iter()
        .find(|item| item.logical_path.0 == logical)
        .unwrap_or_else(|| panic!("no selection for {:?}", String::from_utf8_lossy(logical)));
    match &selection.outcome {
        MultiReleaseSelectionOutcome::Selected { entry } => entry.raw_name.0.clone(),
        other => panic!(
            "expected a selected entry for {:?}, got {other:?}",
            String::from_utf8_lossy(logical)
        ),
    }
}
