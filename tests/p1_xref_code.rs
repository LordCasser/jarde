//! P1 code-consumer acceptance: raw constant-pool candidates (X0), instruction and
//! exception-table consumers (X1), category filtering, exact owner matching, budget
//! stops and read locality — all through the public `Engine::query` entry point.
//!
//! The main fixture is a hand-built class file, because the acceptance criteria pin
//! exact BCIs, constant-pool indexes, opcodes and class-file spans that a compiled
//! sample cannot control: it holds one `Methodref` and one `Class` entry that no code
//! consumer uses (A01), one instruction per consumer category this stream owns, an
//! `invokedynamic` site, an exception table with a named catch type and a catch-all, and
//! an abstract method with no `Code` attribute. The historical ecj fixture supplies the
//! independent A02 sample — `<init>`'s `invokespecial` at BCI 1 on constant-pool index 8
//! — and doubles as the cross-check that the query's instruction facts equal the public
//! `inspect_method_bytecode` facts for the same bytes.
//!
//! Positive assertions look at this stream's operations only (`code_items`), because
//! other consumer sub-scans share the `Type`, `Constant` and `Exception` categories.
//! Assertions that a symbol produces *nothing* use the full item list where the symbol
//! cannot be reached by any other consumer.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const HISTORICAL: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

/// Operations only this stream produces.
const CODE_OPERATIONS: [XrefOperation; 17] = [
    XrefOperation::ConstantPoolEntry,
    XrefOperation::InvokeVirtual,
    XrefOperation::InvokeSpecial,
    XrefOperation::InvokeStatic,
    XrefOperation::InvokeInterface,
    XrefOperation::InvokeDynamic,
    XrefOperation::GetField,
    XrefOperation::GetStatic,
    XrefOperation::PutField,
    XrefOperation::PutStatic,
    XrefOperation::New,
    XrefOperation::NewArray,
    XrefOperation::MultiNewArray,
    XrefOperation::CheckCast,
    XrefOperation::InstanceOf,
    XrefOperation::Ldc,
    XrefOperation::ExceptionHandler,
];

/// Categories the code consumer implements.
const CODE_CATEGORIES: [ConsumerKind; 5] = [
    ConsumerKind::Invocation,
    ConsumerKind::Field,
    ConsumerKind::Type,
    ConsumerKind::Constant,
    ConsumerKind::Exception,
];

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
        nested_depth: 8,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

fn u16b(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn u32b(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    let mut budget = Budget::new(limits());
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .expect("fixture must open")
}

fn physical(snapshot: &ArtifactSnapshot) -> PhysicalView {
    PhysicalView {
        snapshot: snapshot.id().clone(),
        scope: PhysicalScope::SnapshotAll,
    }
}

fn request(
    snapshot: &ArtifactSnapshot,
    relation: QueryRelation,
    target: QueryTarget,
    kinds: &[ConsumerKind],
) -> QueryRequest {
    QueryRequest {
        relation,
        target,
        physical: physical(snapshot),
        consumers: ConsumerSchema::new(1, kinds.to_vec()),
        max_items: 0,
        cursor: None,
    }
}

fn run_with_limits(
    snapshot: &ArtifactSnapshot,
    request: &QueryRequest,
    limits: Limits,
) -> (QueryReport, Budget) {
    let mut budget = Budget::new(limits);
    let report = Engine::new()
        .query(snapshot, request, &mut budget)
        .expect("query must return a report");
    (report, budget)
}

fn run(snapshot: &ArtifactSnapshot, request: &QueryRequest) -> QueryReport {
    run_with_limits(snapshot, request, limits()).0
}

fn class_symbol(owner: &str) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Class {
            owner: JvmBytes(owner.as_bytes().to_vec()),
        },
    }
}

fn method_symbol(owner: &str, name: &str, descriptor: &str) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Method {
            owner: JvmBytes(owner.as_bytes().to_vec()),
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
    }
}

fn field_symbol(owner: &str, name: &str, descriptor: &str) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Field {
            owner: JvmBytes(owner.as_bytes().to_vec()),
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
    }
}

/// Dynamic call site: its symbol carries no owner, so the name and descriptor alone
/// identify it.
fn dynamic_site_symbol(name: &str, descriptor: &str) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Method {
            owner: JvmBytes(Vec::new()),
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
    }
}

fn literal_string(value: &str) -> QueryTarget {
    QueryTarget::Literal {
        value: LiteralValue::String {
            value: JvmBytes(value.as_bytes().to_vec()),
        },
    }
}

fn literal_class(value: &str) -> QueryTarget {
    QueryTarget::Literal {
        value: LiteralValue::Class {
            value: JvmBytes(value.as_bytes().to_vec()),
        },
    }
}

fn literal_integer(value: i32) -> QueryTarget {
    QueryTarget::Literal {
        value: LiteralValue::Integer { value },
    }
}

fn literal_long(value: i64) -> QueryTarget {
    QueryTarget::Literal {
        value: LiteralValue::Long { value },
    }
}

fn literal_float(bits: u32) -> QueryTarget {
    QueryTarget::Literal {
        value: LiteralValue::Float { value: bits },
    }
}

fn literal_double(bits: u64) -> QueryTarget {
    QueryTarget::Literal {
        value: LiteralValue::Double { value: bits },
    }
}

/// Items of this stream's operations, in report order.
fn code_items(report: &QueryReport) -> Vec<&XrefItem> {
    report
        .items
        .iter()
        .filter(|item| CODE_OPERATIONS.contains(&item.operation))
        .collect()
}

/// One reported item's coordinates: operation, constant-pool index, BCI, opcode.
type EvidenceRow = (XrefOperation, Option<u16>, Option<u32>, Option<u8>);

/// One exception table record: start BCI, end BCI, handler BCI, catch type index.
type HandlerRecord = (u16, u16, u16, u16);

/// `(operation, evidence)` of this stream's items, in report order.
fn evidence(report: &QueryReport) -> Vec<EvidenceRow> {
    code_items(report)
        .iter()
        .map(|item| {
            (
                item.operation,
                item.evidence.constant_pool_index,
                item.evidence.bci,
                item.evidence.opcode,
            )
        })
        .collect()
}

/// Result form of a request target, for item assertions.
fn result_target(target: &QueryTarget) -> XrefTarget {
    match target {
        QueryTarget::Symbol { value } => XrefTarget::Symbol {
            value: value.clone(),
        },
        QueryTarget::Literal { value } => XrefTarget::Literal {
            value: value.clone(),
        },
    }
}

/// Recorded constant-pool indexes of a report's items, in report order.
fn pool_indexes(report: &QueryReport) -> Vec<u16> {
    report
        .items
        .iter()
        .map(|item| {
            item.evidence
                .constant_pool_index
                .expect("every pool item records its index")
        })
        .collect()
}

fn code_location(item: &XrefItem) -> (&PhysicalMethodId, u32) {
    match &item.source.location {
        Location::Code { method, bci } => (method, *bci),
        other => panic!("expected a code location, got {other:?}"),
    }
}

fn class_offset(item: &XrefItem) -> (&PhysicalDefinitionId, u64) {
    match &item.source.location {
        Location::ClassOffset { definition, offset } => (definition, *offset),
        other => panic!("expected a class-offset location, got {other:?}"),
    }
}

fn attribute_location(item: &XrefItem) -> (&PhysicalDefinitionId, &str, &ByteSpan) {
    match &item.source.location {
        Location::Attribute { owner, path, span } => (owner, path, span),
        other => panic!("expected an attribute location, got {other:?}"),
    }
}

fn zip(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for &(name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(data).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        archive.finish().unwrap();
    }
    output.into_inner()
}

// ---------------------------------------------------------------------------
// Hand-built class fixture
// ---------------------------------------------------------------------------

/// Constant-pool builder: appends entries in order and reports their 1-based indexes.
#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    fn push(&mut self, entry: Vec<u8>) -> u16 {
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("fixture pool fits u16")
    }

    /// Appends a two-slot entry (`Long`/`Double`) together with its reserved slot.
    fn push_wide(&mut self, entry: Vec<u8>) -> u16 {
        let index = self.entries.len() + 1;
        self.entries.push(entry);
        self.entries.push(Vec::new());
        u16::try_from(index).expect("fixture pool fits u16")
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

    fn float(&mut self, bits: u32) -> u16 {
        let mut entry = vec![4];
        entry.extend_from_slice(&bits.to_be_bytes());
        self.push(entry)
    }

    fn long(&mut self, value: i64) -> u16 {
        let mut entry = vec![5];
        entry.extend_from_slice(&value.to_be_bytes());
        self.push_wide(entry)
    }

    fn double(&mut self, bits: u64) -> u16 {
        let mut entry = vec![6];
        entry.extend_from_slice(&bits.to_be_bytes());
        self.push_wide(entry)
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

    fn method_handle(&mut self, kind: u8, reference: u16) -> u16 {
        let mut entry = vec![15, kind];
        u16b(&mut entry, reference);
        self.push(entry)
    }

    fn invoke_dynamic(&mut self, bootstrap: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![18];
        u16b(&mut entry, bootstrap);
        u16b(&mut entry, name_and_type);
        self.push(entry)
    }

    /// `CONSTANT_MethodType` (16): the entry holds a method descriptor.
    fn method_type(&mut self, descriptor: u16) -> u16 {
        let mut entry = vec![16];
        u16b(&mut entry, descriptor);
        self.push(entry)
    }

    /// `CONSTANT_Dynamic` (17): the entry holds a field descriptor (JVMS 4.4.10).
    fn dynamic(&mut self, bootstrap: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![17];
        u16b(&mut entry, bootstrap);
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

/// Constant-pool indexes the fixture's code and the tests refer to.
struct Indexes {
    /// `Methodref java/lang/Runtime.exec` that no code consumer uses.
    runtime_exec: u16,
    target_run: u16,
    sub_run: u16,
    util_run: u16,
    api_run: u16,
    object_init: u16,
    target_value: u16,
    shape: u16,
    text: u16,
    string_hello: u16,
    /// `Utf8` entry holding `java/lang/String`.
    java_string_name: u16,
    java_string: u16,
    throwable: u16,
    unused_type: u16,
    dynamic_site: u16,
    integer: u16,
    long: u16,
    float: u16,
    double: u16,
}

/// Hand-built class file plus the coordinates the tests assert against.
struct Fixture {
    bytes: Vec<u8>,
    index: Indexes,
    /// Class-file offset of each method's code array, in declaration order.
    code_offsets: [u64; 3],
    /// `Code` attribute entry charges, `<init>` first.
    code_shells: [u64; 2],
    /// Instruction array length of `exercises()`.
    exercises_code_length: u64,
    /// `CodeBytes` one complete scan of both bodies charges: every instruction width.
    code_byte_charge: u64,
    /// `BootstrapMethods` attribute entry charge.
    bootstrap_shell: u64,
}

/// One `Code` attribute body.
fn code_body(max_stack: u16, max_locals: u16, code: &[u8], handlers: &[HandlerRecord]) -> Vec<u8> {
    let mut body = Vec::new();
    u16b(&mut body, max_stack);
    u16b(&mut body, max_locals);
    u32b(
        &mut body,
        u32::try_from(code.len()).expect("fixture code fits u32"),
    );
    body.extend_from_slice(code);
    u16b(
        &mut body,
        u16::try_from(handlers.len()).expect("handler count fits u16"),
    );
    for &(start, end, handler, catch) in handlers {
        u16b(&mut body, start);
        u16b(&mut body, end);
        u16b(&mut body, handler);
        u16b(&mut body, catch);
    }
    u16b(&mut body, 0);
    body
}

/// Appends one method with `attributes` and returns the class-file offset of its code
/// array, when it has one.
fn push_method(
    bytes: &mut Vec<u8>,
    access: u16,
    name: u16,
    descriptor: u16,
    code: Option<(&[u8], &[HandlerRecord])>,
    code_name: u16,
) -> (u64, u64) {
    u16b(bytes, access);
    u16b(bytes, name);
    u16b(bytes, descriptor);
    u16b(bytes, u16::from(code.is_some()));
    let Some((instructions, handlers)) = code else {
        return (0, 0);
    };
    let body = code_body(2, 2, instructions, handlers);
    u16b(bytes, code_name);
    u32b(
        bytes,
        u32::try_from(body.len()).expect("code body fits u32"),
    );
    let body_start = bytes.len();
    bytes.extend_from_slice(&body);
    let shell = u64::try_from(body.len()).expect("code body fits u64") + 6;
    (body_start as u64 + 8, shell)
}

/// A structurally well-formed Java 8 class file with one instruction per consumer
/// category this stream owns.
///
/// It is deliberately not verifier-valid: the scan is structural, so it pins the
/// byte-level facts (BCI, opcode, index, span) rather than stack shapes.
fn fixture() -> Fixture {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/CodeFixture");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let runtime_name = pool.utf8(b"java/lang/Runtime");
    let runtime_class = pool.class(runtime_name);
    let exec_name = pool.utf8(b"exec");
    let exec_descriptor = pool.utf8(b"([Ljava/lang/String;)Ljava/lang/Process;");
    let exec_nat = pool.name_and_type(exec_name, exec_descriptor);
    let runtime_exec = pool.member(10, runtime_class, exec_nat);
    let run_name = pool.utf8(b"run");
    let void_descriptor = pool.utf8(b"()V");
    let run_nat = pool.name_and_type(run_name, void_descriptor);
    let target_name = pool.utf8(b"p/Target");
    let target_class = pool.class(target_name);
    let target_run = pool.member(10, target_class, run_nat);
    let sub_name = pool.utf8(b"p/Sub");
    let sub_class = pool.class(sub_name);
    let sub_run = pool.member(10, sub_class, run_nat);
    let util_name = pool.utf8(b"p/Util");
    let util_class = pool.class(util_name);
    let util_run = pool.member(10, util_class, run_nat);
    let api_name = pool.utf8(b"p/Api");
    let api_class = pool.class(api_name);
    let api_run = pool.member(11, api_class, run_nat);
    let init_name = pool.utf8(b"<init>");
    let init_nat = pool.name_and_type(init_name, void_descriptor);
    let object_init = pool.member(10, object_class, init_nat);
    let value_name = pool.utf8(b"value");
    let int_descriptor = pool.utf8(b"I");
    let value_nat = pool.name_and_type(value_name, int_descriptor);
    let target_value = pool.member(9, target_class, value_nat);
    let shape_name = pool.utf8(b"p/Shape");
    let shape = pool.class(shape_name);
    let text = pool.utf8(b"hello");
    let string_hello = pool.string(text);
    let java_string_name = pool.utf8(b"java/lang/String");
    let java_string = pool.class(java_string_name);
    let throwable_name = pool.utf8(b"java/lang/Throwable");
    let throwable = pool.class(throwable_name);
    let unused_name = pool.utf8(b"p/UnusedType");
    let unused_type = pool.class(unused_name);
    let accept_name = pool.utf8(b"accept");
    let accept_descriptor = pool.utf8(b"(Lp/Shape;)V");
    let accept_nat = pool.name_and_type(accept_name, accept_descriptor);
    let dynamic_site = pool.invoke_dynamic(0, accept_nat);
    let handle = pool.method_handle(6, target_run);
    let integer = pool.integer(7);
    let long = pool.long(9);
    let float = pool.float(1.5_f32.to_bits());
    let double = pool.double(2.5_f64.to_bits());
    let todo_name = pool.utf8(b"todo");
    let exercises_name = pool.utf8(b"exercises");
    let bootstrap_name = pool.utf8(b"BootstrapMethods");
    let code_name = pool.utf8(b"Code");

    // `<init>()V`: aload_0; invokespecial java/lang/Object.<init>()V; return.
    let mut init_code = vec![0x2a, 0xb7];
    u16b(&mut init_code, object_init);
    init_code.push(0xb1);

    // `exercises()V`: new; invokespecial; astore_1; aload_1; invokevirtual;
    // checkcast; instanceof; anewarray; multianewarray; the five `ldc` forms;
    // invokedynamic; the four field accesses; invokevirtual; invokestatic;
    // invokeinterface; return.
    let mut code = Vec::new();
    code.push(0xbb);
    u16b(&mut code, shape); // 0: new p/Shape
    code.push(0xb7);
    u16b(&mut code, object_init); // 3: invokespecial
    code.push(0x4c); // 6: astore_1
    code.push(0x2b); // 7: aload_1
    code.push(0xb6);
    u16b(&mut code, target_run); // 8: invokevirtual p/Target.run
    code.push(0xc0);
    u16b(&mut code, shape); // 11: checkcast p/Shape
    code.push(0xc1);
    u16b(&mut code, shape); // 14: instanceof p/Shape
    code.push(0xbd);
    u16b(&mut code, java_string); // 17: anewarray java/lang/String
    code.push(0xc5);
    u16b(&mut code, shape);
    code.push(2); // 20: multianewarray p/Shape 2
    code.push(0x12);
    code.push(u8::try_from(integer).unwrap()); // 24: ldc Integer
    code.push(0x12);
    code.push(u8::try_from(string_hello).unwrap()); // 26: ldc "hello"
    code.push(0x12);
    code.push(u8::try_from(java_string).unwrap()); // 28: ldc Class java/lang/String
    code.push(0x13);
    u16b(&mut code, float); // 30: ldc_w Float
    code.push(0x14);
    u16b(&mut code, long); // 33: ldc2_w Long
    code.push(0x14);
    u16b(&mut code, double); // 36: ldc2_w Double
    code.push(0xba);
    u16b(&mut code, dynamic_site);
    code.push(0);
    code.push(0); // 39: invokedynamic
    code.push(0xb2);
    u16b(&mut code, target_value); // 44: getstatic
    code.push(0xb3);
    u16b(&mut code, target_value); // 47: putstatic
    code.push(0xb4);
    u16b(&mut code, target_value); // 50: getfield
    code.push(0xb5);
    u16b(&mut code, target_value); // 53: putfield
    code.push(0xb6);
    u16b(&mut code, sub_run); // 56: invokevirtual p/Sub.run
    code.push(0xb8);
    u16b(&mut code, util_run); // 59: invokestatic p/Util.run
    code.push(0xb9);
    u16b(&mut code, api_run);
    code.push(1);
    code.push(0); // 62: invokeinterface p/Api.run
    code.push(0xb1); // 67: return
    let exercises_code_length = u64::try_from(code.len()).expect("code length fits u64");

    // A named catch type at ordinal 0 and a catch-all at ordinal 1.
    let handlers: [HandlerRecord; 2] = [
        (0_u16, 24_u16, 44_u16, throwable),
        (0_u16, 68_u16, 67_u16, 0_u16),
    ];

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0); // minor
    u16b(&mut bytes, 52); // major: Java 8
    u16b(&mut bytes, pool.declared());
    bytes.extend_from_slice(&pool.bytes());
    u16b(&mut bytes, 0x0021); // ACC_PUBLIC | ACC_SUPER
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, object_class);
    u16b(&mut bytes, 0); // interfaces
    u16b(&mut bytes, 0); // fields
    u16b(&mut bytes, 3); // methods
    let (init_code_offset, init_shell) = push_method(
        &mut bytes,
        0x0001,
        init_name,
        void_descriptor,
        Some((&init_code, &[])),
        code_name,
    );
    let (exercises_code_offset, exercises_shell) = push_method(
        &mut bytes,
        0x0002,
        exercises_name,
        void_descriptor,
        Some((&code, &handlers)),
        code_name,
    );
    // An abstract method has no `Code` attribute: there is no instruction stream to
    // scan, and the query must still scan the rest of the class.
    u16b(&mut bytes, 0x0401);
    u16b(&mut bytes, todo_name);
    u16b(&mut bytes, void_descriptor);
    u16b(&mut bytes, 0);
    // One class-level `BootstrapMethods` attribute for the `invokedynamic` site.
    u16b(&mut bytes, 1);
    u16b(&mut bytes, bootstrap_name);
    let mut bootstrap = Vec::new();
    u16b(&mut bootstrap, 1);
    u16b(&mut bootstrap, handle);
    u16b(&mut bootstrap, 0);
    u32b(
        &mut bytes,
        u32::try_from(bootstrap.len()).expect("bootstrap body fits u32"),
    );
    bytes.extend_from_slice(&bootstrap);

    Fixture {
        bytes,
        index: Indexes {
            runtime_exec,
            target_run,
            sub_run,
            util_run,
            api_run,
            object_init,
            target_value,
            shape,
            text,
            string_hello,
            java_string_name,
            java_string,
            throwable,
            unused_type,
            dynamic_site,
            integer,
            long,
            float,
            double,
        },
        code_offsets: [init_code_offset, exercises_code_offset, 0],
        code_shells: [init_shell, exercises_shell],
        exercises_code_length,
        code_byte_charge: u64::try_from(init_code.len() + code.len()).expect("code fits u64"),
        bootstrap_shell: u64::try_from(bootstrap.len()).expect("bootstrap fits u64") + 6,
    }
}

/// Attribute bytes one query over `fixture()` must charge: `class_facts` bills every
/// enumerated shell, and the code consumer bills each `Code` attribute again when it
/// reads that content. The `BootstrapMethods` content is never read, so its bytes appear
/// exactly once.
fn expected_attribute_bytes(fixture: &Fixture) -> u64 {
    fixture.bootstrap_shell
        + fixture.code_shells.iter().sum::<u64>()
        + fixture.code_shells.iter().sum::<u64>()
}

// ---------------------------------------------------------------------------
// Descriptor-consumer fixtures: types a real use site consumes
// ---------------------------------------------------------------------------

/// Constant-pool indexes of the `Utf8` entries holding exactly this text.
///
/// The fixture writes the pool itself, so a test can state that a type name has no pool
/// entry of its own — which is what makes a `CONSTANT_Class` for it impossible.
fn utf8_indexes(pool: &Pool, text: &[u8]) -> Vec<u16> {
    pool.entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            entry.first() == Some(&1) && entry.len() == 3 + text.len() && &entry[3..] == text
        })
        .map(|(index, _)| u16::try_from(index + 1).expect("pool index fits u16"))
        .collect()
}

/// One concrete method of a hand-built class: access flags, name, descriptor and body.
struct Body {
    access: u16,
    name: u16,
    descriptor: u16,
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
}

/// Assembles a Java 8 class file around a pool and concrete methods, each with exactly one
/// `Code` attribute and no exception table.
///
/// Returns the class bytes and the class-file offset of each method's code array, so a
/// test can slice an instruction out of the bytes it wrote.
fn assemble(
    pool: &Pool,
    code_name: u16,
    this_class: u16,
    super_class: u16,
    methods: &[Body],
    class_attributes: &[(u16, Vec<u8>)],
) -> (Vec<u8>, Vec<u64>) {
    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0); // minor
    u16b(&mut bytes, 52); // major: Java 8
    u16b(&mut bytes, pool.declared());
    bytes.extend_from_slice(&pool.bytes());
    u16b(&mut bytes, 0x0021); // ACC_PUBLIC | ACC_SUPER
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, super_class);
    u16b(&mut bytes, 0); // interfaces
    u16b(&mut bytes, 0); // fields
    u16b(
        &mut bytes,
        u16::try_from(methods.len()).expect("fixture methods fit u16"),
    );
    let mut code_offsets = Vec::new();
    for method in methods {
        u16b(&mut bytes, method.access);
        u16b(&mut bytes, method.name);
        u16b(&mut bytes, method.descriptor);
        u16b(&mut bytes, 1); // one attribute: Code
        u16b(&mut bytes, code_name);
        let body = code_body(
            method.max_stack,
            method.max_locals,
            &method.code,
            &[] as &[HandlerRecord],
        );
        u32b(
            &mut bytes,
            u32::try_from(body.len()).expect("code body fits u32"),
        );
        let body_start = bytes.len() as u64;
        bytes.extend_from_slice(&body);
        code_offsets.push(body_start + 8);
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
    (bytes, code_offsets)
}

/// `BootstrapMethods` content with one entry: a method handle and no static arguments.
fn bootstrap_content(handle: u16) -> Vec<u8> {
    let mut out = Vec::new();
    u16b(&mut out, 1); // num_bootstrap_methods
    u16b(&mut out, handle);
    u16b(&mut out, 0); // num_bootstrap_arguments
    out
}

// ---------------------------------------------------------------------------
// Execution and diagnostic helpers
// ---------------------------------------------------------------------------

fn is_complete(report: &QueryReport) -> bool {
    matches!(report.execution, ExecutionReport::Complete { .. })
}

fn is_cancelled(report: &QueryReport) -> bool {
    matches!(report.execution, ExecutionReport::Cancelled { .. })
}

/// Budget dimension of a `Partial { BudgetExceeded }` execution.
fn partial_dimension(report: &QueryReport) -> Option<BudgetDimension> {
    match &report.execution {
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded { dimension },
            ..
        } => Some(*dimension),
        _ => None,
    }
}

/// Error code of a `Failed { Error }` execution.
fn failed_code(report: &QueryReport) -> Option<&str> {
    match &report.execution {
        ExecutionReport::Failed {
            reason: TerminationReason::Error { code },
            ..
        } => Some(code.as_str()),
        _ => None,
    }
}

fn diagnostic_codes(report: &QueryReport) -> Vec<&str> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

fn record_at(bytes: &[u8], start: u64) -> (u16, u16, u16, u16) {
    let start = usize::try_from(start).expect("fixture offset fits usize");
    let read =
        |offset: usize| u16::from_be_bytes([bytes[start + offset], bytes[start + offset + 1]]);
    (read(0), read(2), read(4), read(6))
}

/// Instruction opcode at a BCI of the `exercises()` body.
fn opcode_at(bytes: &[u8], code_offset: u64, bci: u64) -> u8 {
    bytes[usize::try_from(code_offset + bci).expect("offset fits usize")]
}

// ---------------------------------------------------------------------------
// A01: a used pool entry is not a call, and an unused one is not either
// ---------------------------------------------------------------------------

#[test]
fn unused_constant_pool_entries_are_candidates_but_never_calls() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());

    // X0: the pool contains the `Methodref`, and the item says exactly that.
    let target = method_symbol(
        "java/lang/Runtime",
        "exec",
        "([Ljava/lang/String;)Ljava/lang/Process;",
    );
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::ConstantPoolContains,
            target.clone(),
            &[ConsumerKind::Invocation],
        ),
    );
    assert_eq!(report.items.len(), 1);
    let item = &report.items[0];
    assert_eq!(item.consumer, None);
    assert_eq!(item.operation, XrefOperation::ConstantPoolEntry);
    assert_eq!(item.derivation, XrefDerivation::ConstantPoolCandidate);
    assert_eq!(item.certainty, XrefCertainty::Exact);
    assert_eq!(item.target, result_target(&target));
    assert_eq!(
        item.evidence.constant_pool_index,
        Some(fixture.index.runtime_exec)
    );
    assert_eq!(item.evidence.bci, None, "a pool entry has no BCI");
    assert_eq!(item.evidence.opcode, None, "a pool entry has no opcode");
    assert!(item.evidence.via.is_empty());
    let span = item
        .evidence
        .span
        .clone()
        .expect("the entry span is recorded");
    assert_eq!(
        fixture.bytes[usize::try_from(span.start).unwrap()],
        10,
        "the recorded span starts at the CONSTANT_Methodref tag"
    );
    assert_eq!(span.length, 5, "tag byte plus two u16 indexes");
    let (definition, offset) = class_offset(item);
    assert_eq!(offset, span.start);
    assert_eq!(
        definition.class_bytes.length,
        u64::try_from(fixture.bytes.len()).unwrap()
    );
    assert!(is_complete(&report));

    // X1: no consumer uses that entry, so no call, no BCI and no consumer appears.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target,
            &CODE_CATEGORIES,
        ),
    );
    assert!(
        report.items.is_empty(),
        "an unused Methodref must not become a call: {:?}",
        report.items
    );
    assert_eq!(report.coverage.scanned_items, 0);
    assert_eq!(report.coverage.unknown_candidates, 0);
    assert!(is_complete(&report));

    // The same contrast for a type that only exists in the pool.
    let target = class_symbol("p/UnusedType");
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::ConstantPoolContains,
            target.clone(),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(report.items.len(), 1);
    assert_eq!(
        report.items[0].evidence.constant_pool_index,
        Some(fixture.index.unused_type)
    );
    assert_eq!(report.items[0].consumer, None);
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target,
            &CODE_CATEGORIES,
        ),
    );
    assert!(
        report.items.is_empty(),
        "a pool-only type is not a reference"
    );
}

// ---------------------------------------------------------------------------
// A02: the recorded invocation keeps symbol, BCI, opcode and pool index
// ---------------------------------------------------------------------------

#[test]
fn invocation_item_keeps_the_complete_symbol_and_its_byte_coordinates() {
    let snapshot = open(HISTORICAL.to_vec());
    let target = method_symbol("java/lang/Object", "<init>", "()V");
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target.clone(),
            &[ConsumerKind::Invocation],
        ),
    );
    let items = code_items(&report);
    assert_eq!(items.len(), 1);
    let item = items[0];
    assert_eq!(item.consumer, Some(ConsumerKind::Invocation));
    assert_eq!(item.operation, XrefOperation::InvokeSpecial);
    assert_eq!(item.derivation, XrefDerivation::StructuralConsumer);
    assert_eq!(item.certainty, XrefCertainty::Exact);
    assert_eq!(
        item.target,
        result_target(&target),
        "the whole symbol is reported back"
    );
    assert_eq!(item.evidence.constant_pool_index, Some(8));
    assert_eq!(item.evidence.bci, Some(1));
    assert_eq!(item.evidence.opcode, Some(0xb7));
    assert_eq!(
        item.evidence.attribute.as_ref().map(|name| name.0.clone()),
        Some(b"Code".to_vec())
    );
    let span = item.evidence.span.clone().expect("instruction span");
    assert_eq!(span.length, 3);
    assert_eq!(
        HISTORICAL[usize::try_from(span.start).unwrap()],
        0xb7,
        "the recorded span starts at the invokespecial opcode"
    );
    let operand = usize::try_from(span.start).unwrap() + 1;
    assert_eq!(&HISTORICAL[operand..operand + 2], &8_u16.to_be_bytes());
    let (method, bci) = code_location(item);
    assert_eq!(bci, 1);
    assert_eq!(method.name.0, b"<init>");
    assert_eq!(method.descriptor.0, b"()V");

    // The same bytes through the public bytecode path report the same instruction
    // facts: the query scan does not have a decode path of its own.
    let mut budget = Budget::new(limits());
    let inspection = inspect_method_bytecode(
        HISTORICAL,
        MethodSelector {
            name: JvmBytes(b"<init>".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        },
        &mut budget,
    )
    .expect("the historical method decodes");
    assert_eq!(inspection.instructions[1].bci, 1);
    assert_eq!(inspection.instructions[1].opcode, 0xb7);
    assert_eq!(inspection.instructions[1].constant_pool_index, Some(8));
    assert_eq!(inspection.instructions[1].span, span);
    assert_eq!(
        span.start,
        inspection.code_span.start + 1,
        "the query span is the instruction at BCI 1 of the same code array"
    );
}

// ---------------------------------------------------------------------------
// X0: the raw probe covers every representation, and only the probe
// ---------------------------------------------------------------------------

#[test]
fn raw_probe_matches_the_bytes_and_values_the_pool_really_stores() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());

    // The same bytes are spelled by a `Utf8` entry and a `String` entry; the raw probe
    // answers for both, in ascending pool order.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::ConstantPoolContains,
            literal_string("hello"),
            &[ConsumerKind::Constant],
        ),
    );
    assert_eq!(
        pool_indexes(&report),
        vec![fixture.index.text, fixture.index.string_hello]
    );
    assert_eq!(report.items[0].evidence.span.as_ref().unwrap().length, 8);
    assert_eq!(report.items[1].evidence.span.as_ref().unwrap().length, 3);

    // A class literal name is held by a `Utf8` and a `Class` entry.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::ConstantPoolContains,
            literal_class("java/lang/String"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(
        pool_indexes(&report),
        vec![fixture.index.java_string_name, fixture.index.java_string]
    );
    assert_eq!(report.items[0].evidence.span.as_ref().unwrap().length, 19);
    assert_eq!(report.items[1].evidence.span.as_ref().unwrap().length, 3);
}

#[test]
fn raw_probe_distinguishes_values_and_bit_patterns() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());

    let cases = [
        (literal_integer(7), fixture.index.integer, 5),
        (literal_long(9), fixture.index.long, 9),
        (literal_float(1.5_f32.to_bits()), fixture.index.float, 5),
        (literal_double(2.5_f64.to_bits()), fixture.index.double, 9),
    ];
    for (target, index, entry_length) in cases {
        let report = run(
            &snapshot,
            &request(
                &snapshot,
                QueryRelation::ConstantPoolContains,
                target.clone(),
                &[ConsumerKind::Constant],
            ),
        );
        assert_eq!(
            pool_indexes(&report),
            vec![index],
            "expected exactly the {target:?} entry"
        );
        assert_eq!(
            report.items[0].evidence.span.as_ref().unwrap().length,
            entry_length
        );
        assert_eq!(
            report.items[0].derivation,
            XrefDerivation::ConstantPoolCandidate
        );
    }

    // A different value does not match, and `-1.5` is not the bit pattern of `1.5`.
    for target in [
        literal_integer(8),
        literal_long(-9),
        literal_float((-1.5_f32).to_bits()),
        literal_double(2.5_f64.to_bits() + 1),
    ] {
        let report = run(
            &snapshot,
            &request(
                &snapshot,
                QueryRelation::ConstantPoolContains,
                target,
                &[ConsumerKind::Constant],
            ),
        );
        assert!(report.items.is_empty());
    }
}

#[test]
fn a_value_relation_reports_consumers_and_never_pool_candidates() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            literal_string("hello"),
            &[ConsumerKind::Constant],
        ),
    );
    // The `Utf8` and `String` entries exist, but `literal_value` is about a consumer
    // that really read the value: the `ldc` instruction.
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::Ldc,
            Some(fixture.index.string_hello),
            Some(26),
            Some(0x12)
        )]
    );
    assert!(
        report
            .items
            .iter()
            .all(|item| item.derivation == XrefDerivation::StructuralConsumer)
    );
    let (method, _) = code_location(code_items(&report)[0]);
    assert_eq!(method.name.0, b"exercises");
}

// ---------------------------------------------------------------------------
// X1: one item per consumer category, with the raw coordinates
// ---------------------------------------------------------------------------

#[test]
fn invocation_kinds_and_dynamic_sites_keep_operation_and_position() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let cases = [
        (
            method_symbol("p/Target", "run", "()V"),
            (
                XrefOperation::InvokeVirtual,
                fixture.index.target_run,
                8,
                0xb6,
            ),
        ),
        (
            method_symbol("p/Sub", "run", "()V"),
            (
                XrefOperation::InvokeVirtual,
                fixture.index.sub_run,
                56,
                0xb6,
            ),
        ),
        (
            method_symbol("p/Util", "run", "()V"),
            (
                XrefOperation::InvokeStatic,
                fixture.index.util_run,
                59,
                0xb8,
            ),
        ),
        (
            // An interface method reference answers a method symbol like a class one.
            method_symbol("p/Api", "run", "()V"),
            (
                XrefOperation::InvokeInterface,
                fixture.index.api_run,
                62,
                0xb9,
            ),
        ),
        (
            dynamic_site_symbol("accept", "(Lp/Shape;)V"),
            (
                XrefOperation::InvokeDynamic,
                fixture.index.dynamic_site,
                39,
                0xba,
            ),
        ),
    ];
    for (target, (operation, index, bci, opcode)) in cases {
        let report = run(
            &snapshot,
            &request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target.clone(),
                &[ConsumerKind::Invocation],
            ),
        );
        assert_eq!(
            evidence(&report),
            vec![(operation, Some(index), Some(bci), Some(opcode))],
            "expected one {operation:?} item for {target:?}"
        );
        let item = code_items(&report)[0];
        assert_eq!(item.consumer, Some(ConsumerKind::Invocation));
        assert_eq!(item.certainty, XrefCertainty::Exact);
        assert_eq!(item.resolution, QueryResolution::NotRequested);
        assert_eq!(item.target, result_target(&target));
        let (method, item_bci) = code_location(item);
        assert_eq!(item_bci, bci);
        assert_eq!(method.name.0, b"exercises");
        assert_eq!(method.descriptor.0, b"()V");
        // The recorded span covers exactly the instruction bytes at that BCI.
        let span = item.evidence.span.as_ref().unwrap();
        assert_eq!(span.start, fixture.code_offsets[1] + u64::from(bci));
        assert_eq!(
            opcode_at(&fixture.bytes, fixture.code_offsets[1], u64::from(bci)),
            opcode
        );
    }

    // The same symbol appears in two methods: both positions are reported in method
    // order, and a dynamic site is not reachable with an owner.
    let target = method_symbol("java/lang/Object", "<init>", "()V");
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target,
            &[ConsumerKind::Invocation],
        ),
    );
    assert_eq!(
        evidence(&report),
        vec![
            (
                XrefOperation::InvokeSpecial,
                Some(fixture.index.object_init),
                Some(1),
                Some(0xb7)
            ),
            (
                XrefOperation::InvokeSpecial,
                Some(fixture.index.object_init),
                Some(3),
                Some(0xb7)
            ),
        ]
    );
    let names: Vec<&[u8]> = code_items(&report)
        .iter()
        .map(|item| code_location(item).0.name.0.as_slice())
        .collect();
    assert_eq!(names, vec![b"<init>".as_slice(), b"exercises".as_slice()]);

    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Shape", "accept", "(Lp/Shape;)V"),
            &[ConsumerKind::Invocation],
        ),
    );
    assert!(
        report.items.is_empty(),
        "an invokedynamic site names no owner, so an owned symbol must not match it"
    );
}

#[test]
fn field_accesses_keep_read_write_and_static_instance_distinct() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            field_symbol("p/Target", "value", "I"),
            &[ConsumerKind::Field],
        ),
    );
    assert_eq!(
        evidence(&report),
        vec![
            (
                XrefOperation::GetStatic,
                Some(fixture.index.target_value),
                Some(44),
                Some(0xb2)
            ),
            (
                XrefOperation::PutStatic,
                Some(fixture.index.target_value),
                Some(47),
                Some(0xb3)
            ),
            (
                XrefOperation::GetField,
                Some(fixture.index.target_value),
                Some(50),
                Some(0xb4)
            ),
            (
                XrefOperation::PutField,
                Some(fixture.index.target_value),
                Some(53),
                Some(0xb5)
            ),
        ]
    );
    assert!(
        code_items(&report)
            .iter()
            .all(|item| item.consumer == Some(ConsumerKind::Field))
    );
    // A member symbol is not answered by a member reference of the other kind.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            field_symbol("p/Target", "run", "()V"),
            &[ConsumerKind::Field],
        ),
    );
    assert!(report.items.is_empty());
}

#[test]
fn type_operations_keep_the_operation_that_consumed_the_type() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/Shape"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(
        evidence(&report),
        vec![
            (
                XrefOperation::New,
                Some(fixture.index.shape),
                Some(0),
                Some(0xbb)
            ),
            (
                XrefOperation::CheckCast,
                Some(fixture.index.shape),
                Some(11),
                Some(0xc0)
            ),
            (
                XrefOperation::InstanceOf,
                Some(fixture.index.shape),
                Some(14),
                Some(0xc1)
            ),
            (
                XrefOperation::MultiNewArray,
                Some(fixture.index.shape),
                Some(20),
                Some(0xc5)
            ),
            (
                // The `invokedynamic` site's own descriptor names the same type, so the
                // instruction that consumed that entry is a second source for it under
                // the `Type` category (decision 26).
                XrefOperation::InvokeDynamic,
                Some(fixture.index.dynamic_site),
                Some(39),
                Some(0xba)
            ),
        ]
    );
    assert!(
        code_items(&report)
            .iter()
            .all(|item| item.consumer == Some(ConsumerKind::Type))
    );

    // An array component type is a `CONSTANT_Class` consumed by `anewarray`.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("java/lang/String"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::NewArray,
            Some(fixture.index.java_string),
            Some(17),
            Some(0xbd)
        )]
    );
}

#[test]
fn loads_report_the_value_and_the_symbol_they_consume() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let cases = [
        (
            literal_string("hello"),
            (fixture.index.string_hello, 26, 0x12),
        ),
        (literal_integer(7), (fixture.index.integer, 24, 0x12)),
        (
            literal_class("java/lang/String"),
            (fixture.index.java_string, 28, 0x12),
        ),
        (
            literal_float(1.5_f32.to_bits()),
            (fixture.index.float, 30, 0x13),
        ),
        (literal_long(9), (fixture.index.long, 33, 0x14)),
        (
            literal_double(2.5_f64.to_bits()),
            (fixture.index.double, 36, 0x14),
        ),
    ];
    for (target, (index, bci, opcode)) in cases {
        let report = run(
            &snapshot,
            &request(
                &snapshot,
                QueryRelation::LiteralValue,
                target.clone(),
                &[ConsumerKind::Constant],
            ),
        );
        assert_eq!(
            evidence(&report),
            vec![(XrefOperation::Ldc, Some(index), Some(bci), Some(opcode))],
            "expected one ldc item for {target:?}"
        );
        assert_eq!(
            code_items(&report)[0].consumer,
            Some(ConsumerKind::Constant)
        );
    }

    // The same `Class` entry is a type symbol for `mentions_symbol` and a value for
    // `literal_value`: the relation decides which claim is made.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("java/lang/String"),
            &[ConsumerKind::Constant],
        ),
    );
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::Ldc,
            Some(fixture.index.java_string),
            Some(28),
            Some(0x12)
        )]
    );

    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            literal_string("bye"),
            &[ConsumerKind::Constant],
        ),
    );
    assert!(report.items.is_empty());
}

#[test]
fn exception_handlers_keep_their_ordinal_protected_range_and_handler_bci() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("java/lang/Throwable"),
            &[ConsumerKind::Exception],
        ),
    );
    let items = code_items(&report);
    assert_eq!(items.len(), 1, "the catch-all handler names no type");
    let item = items[0];
    assert_eq!(item.consumer, Some(ConsumerKind::Exception));
    assert_eq!(item.operation, XrefOperation::ExceptionHandler);
    assert_eq!(
        item.evidence.constant_pool_index,
        Some(fixture.index.throwable)
    );
    assert_eq!(item.evidence.bci, Some(44), "the handler entry BCI");
    assert_eq!(
        item.evidence.opcode, None,
        "a handler is not an instruction"
    );
    // The evidence span is the protected range [start_bci, end_bci) of ordinal 0.
    let protected = item.evidence.span.as_ref().unwrap();
    assert_eq!(protected.start, fixture.code_offsets[1]);
    assert_eq!(protected.length, 24);
    // The location names the ordinal and addresses that exception table record, so the
    // ordinal can be re-checked against the class bytes.
    let (owner, path, record) = attribute_location(item);
    assert_eq!(path, "methods[1].Code.exception_handlers[0]");
    assert_eq!(
        record.start,
        fixture.code_offsets[1] + fixture.exercises_code_length + 2
    );
    assert_eq!(record.length, 8);
    assert_eq!(
        record_at(&fixture.bytes, record.start),
        (0, 24, 44, fixture.index.throwable)
    );
    assert_eq!(
        owner.class_bytes.length,
        u64::try_from(fixture.bytes.len()).unwrap()
    );
    // Ordinal 1 is the catch-all: it is a real record with no catch type, so it produces
    // no type reference.
    assert_eq!(record_at(&fixture.bytes, record.start + 8), (0, 68, 67, 0));

    // A literal target never matches a handler.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            literal_string("java/lang/Throwable"),
            &[ConsumerKind::Exception],
        ),
    );
    assert!(report.items.is_empty());
}

// ---------------------------------------------------------------------------
// Category filtering and owner matching
// ---------------------------------------------------------------------------

#[test]
fn unrequested_categories_produce_no_items() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let invocation = method_symbol("p/Target", "run", "()V");
    for kinds in [
        [ConsumerKind::Field],
        [ConsumerKind::Type],
        [ConsumerKind::Constant],
        [ConsumerKind::Exception],
    ] {
        let report = run(
            &snapshot,
            &request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                invocation.clone(),
                &kinds,
            ),
        );
        assert!(
            report.items.is_empty(),
            "only Invocation consumes this symbol, but {kinds:?} returned items"
        );
        assert!(is_complete(&report));
    }

    // One `CONSTANT_Class` entry is consumed by two categories with two operations, and
    // each request sees only the category it asked for.
    let string = class_symbol("java/lang/String");
    let counts: [(&[ConsumerKind], usize); 4] = [
        (&[ConsumerKind::Invocation], 0_usize),
        (&[ConsumerKind::Type], 1),
        (&[ConsumerKind::Constant], 1),
        (&[ConsumerKind::Type, ConsumerKind::Constant], 2),
    ];
    for (kinds, expected) in counts {
        let report = run(
            &snapshot,
            &request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                string.clone(),
                kinds,
            ),
        );
        assert_eq!(
            code_items(&report).len(),
            expected,
            "category set {kinds:?} returned the wrong number of items"
        );
    }

    // A request that names no code category at all never even reads class bytes.
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/Shape"),
            &[ConsumerKind::Resource],
        ),
        limits(),
    );
    assert!(report.items.is_empty());
    assert_eq!(
        budget.usage().class_bytes,
        0,
        "no code category was requested"
    );
}

#[test]
fn symbol_matching_stays_exact_and_does_not_expand_the_owner() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    // `p/Target.run` and `p/Sub.run` share name and descriptor and differ only in owner.
    let target = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Target", "run", "()V"),
            &[ConsumerKind::Invocation],
        ),
    );
    assert_eq!(
        evidence(&target),
        vec![(
            XrefOperation::InvokeVirtual,
            Some(fixture.index.target_run),
            Some(8),
            Some(0xb6)
        )]
    );
    let sub = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Sub", "run", "()V"),
            &[ConsumerKind::Invocation],
        ),
    );
    assert_eq!(
        evidence(&sub),
        vec![(
            XrefOperation::InvokeVirtual,
            Some(fixture.index.sub_run),
            Some(56),
            Some(0xb6)
        )]
    );
    // The owner dimension is the raw pool owner: a call site whose pool owner differs
    // from the declared owner is a miss here. Expanding `Sub` to the method inherited
    // from `Base` is the P2 definition resolver's job behind `references_definition`.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Base", "run", "()V"),
            &[ConsumerKind::Invocation],
        ),
    );
    assert!(
        report.items.is_empty(),
        "P1 must not resolve an inherited declaration"
    );
}

// ---------------------------------------------------------------------------
// Cost and locality
// ---------------------------------------------------------------------------

#[test]
fn one_class_read_and_no_result_items_per_instruction() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    // `Invocation` is a code-only category, so this request is scanned by this stream
    // alone and every charge below belongs to it.
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Target", "run", "()V"),
            &[ConsumerKind::Invocation],
        ),
        limits(),
    );
    assert_eq!(code_items(&report).len(), 1);
    let usage = budget.usage();
    assert_eq!(
        usage.class_bytes,
        u64::try_from(fixture.bytes.len()).unwrap(),
        "one class read for a class with three methods, one of them abstract, and no \
         second read per decoded method"
    );
    assert_eq!(
        usage.code_bytes, fixture.code_byte_charge,
        "every instruction width once, and nothing for the abstract member"
    );
    assert_eq!(
        usage.attribute_bytes,
        expected_attribute_bytes(&fixture),
        "every attribute shell once, plus each `Code` content the consumer read: the \
         `BootstrapMethods` content is never read"
    );
    assert_eq!(
        usage.result_items, 1,
        "one result item per returned item, none per scanned instruction"
    );
    assert_eq!(
        usage.read_bytes,
        u64::try_from(fixture.bytes.len()).unwrap(),
        "the standalone root is read once"
    );
    assert!(
        report.diagnostics.is_empty(),
        "a complete scan reports no diagnostics: {:?}",
        report.diagnostics
    );
}

#[test]
fn zip_scan_reads_the_class_entry_and_never_other_entries() {
    let fixture = fixture();
    let large = vec![0x5a_u8; 8192];
    let archive = zip(&[
        (b"p/CodeFixture.class", &fixture.bytes),
        (b"docs/large.bin", &large),
        (b"data/notes.txt", b"not a class file"),
    ]);
    let snapshot = open(archive);
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Target", "run", "()V"),
            &[ConsumerKind::Invocation],
        ),
        limits(),
    );
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::InvokeVirtual,
            Some(fixture.index.target_run),
            Some(8),
            Some(0xb6)
        )]
    );
    let (method, _) = code_location(code_items(&report)[0]);
    match &method.owner.location {
        PhysicalClassLocation::ArchiveEntry { entry } => {
            assert_eq!(entry.raw_name.0, b"p/CodeFixture.class");
            assert_eq!(entry.ordinal, 0);
        }
        other => panic!("expected the archive entry as the physical source, got {other:?}"),
    }
    let usage = budget.usage();
    let class_length = u64::try_from(fixture.bytes.len()).unwrap();
    assert_eq!(usage.read_bytes, class_length);
    assert_eq!(usage.entry_bytes, class_length);
    assert_eq!(usage.class_bytes, class_length);
    assert_eq!(usage.code_bytes, fixture.code_byte_charge);
    assert!(
        usage.read_bytes < u64::try_from(large.len()).unwrap(),
        "the unrelated 8 KiB entry is never materialized"
    );

    // The raw pool probe reads the same single class entry and answers from its pool.
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::ConstantPoolContains,
            method_symbol(
                "java/lang/Runtime",
                "exec",
                "([Ljava/lang/String;)Ljava/lang/Process;",
            ),
            &[ConsumerKind::Invocation],
        ),
        limits(),
    );
    assert_eq!(pool_indexes(&report), vec![fixture.index.runtime_exec]);
    assert_eq!(
        report.items[0].derivation,
        XrefDerivation::ConstantPoolCandidate
    );
    assert_eq!(budget.usage().read_bytes, class_length);

    // A request for other categories leaves the class bytes untouched.
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/Shape"),
            &[ConsumerKind::Resource],
        ),
        limits(),
    );
    assert!(report.items.is_empty());
    let usage = budget.usage();
    assert_eq!(
        (
            usage.read_bytes,
            usage.class_bytes,
            usage.attribute_bytes,
            usage.code_bytes
        ),
        (0, 0, 0, 0),
        "no class entry is read for a request that names no code category"
    );

    // An entry outside the candidate range is never materialized: a plain resource
    // that only looks like a class neither fails the query whose real class entries
    // are readable nor costs a read. A `.class`-named entry with these bytes is a
    // damaged candidate instead, which
    // `a_truncated_class_candidate_fails_with_its_entry_origin` covers.
    let mislabeled = zip(&[
        (b"p/CodeFixture.class", &fixture.bytes),
        (b"p/notes.txt", b"this file is not a class file"),
    ]);
    let snapshot = open(mislabeled);
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Target", "run", "()V"),
            &[ConsumerKind::Invocation],
        ),
        limits(),
    );
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::InvokeVirtual,
            Some(fixture.index.target_run),
            Some(8),
            Some(0xb6)
        )]
    );
    assert!(is_complete(&report));
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(
        budget.usage().entry_bytes,
        class_length,
        "only the class candidate is materialized"
    );
}

/// `container:root:xref_scan_entries` ranges of one structural dimension side.
fn xref_ranges(ranges: &[CoverageRange]) -> Vec<(u64, u64)> {
    ranges
        .iter()
        .filter(|range| range.label == "container:root:xref_scan_entries")
        .map(|range| (range.start, range.end))
        .collect()
}

#[test]
fn a_truncated_class_candidate_fails_with_its_entry_origin() {
    let fixture = fixture();
    let archive = zip(&[
        (b"p/CodeFixture.class", &fixture.bytes),
        (b"p/Broken.class", &[0xca, 0xfe, 0xba]),
        (b"p/Other.class", &fixture.bytes),
    ]);
    let snapshot = open(archive);
    let (report, _) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Target", "run", "()V"),
            &[ConsumerKind::Invocation],
        ),
        limits(),
    );

    // The candidate that scanned before the damaged one keeps its facts: the result is
    // a reliable prefix, not an empty miss.
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::InvokeVirtual,
            Some(fixture.index.target_run),
            Some(8),
            Some(0xb6)
        )]
    );
    // The damaged candidate ends the unit with its own physical origin, so the caller
    // can locate the entry that could not be parsed.
    assert_eq!(
        failed_code(&report),
        Some("query_class_candidate_malformed")
    );
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "query_class_candidate_malformed")
        .expect("the damaged candidate is reported");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    match diagnostic
        .provenance
        .as_ref()
        .map(|provenance| &provenance.location)
    {
        Some(Location::Entry { id, span }) => {
            assert_eq!(id.raw_name.0, b"p/Broken.class");
            assert_eq!(id.ordinal, 1);
            assert_eq!(*span, ByteSpan::new(0, 3));
        }
        other => panic!("expected the damaged entry as the diagnostic origin, got {other:?}"),
    }
    // The message names the entry, the bytes it really holds and the magic it lacks,
    // so a three-byte truncation is distinguishable from four wrong bytes.
    assert!(
        diagnostic.message.contains("Broken.class"),
        "{diagnostic:?}"
    );
    assert!(diagnostic.message.contains('3'), "{diagnostic:?}");
    assert!(diagnostic.message.contains("CA FE BA BE"), "{diagnostic:?}");
    // Neither the execution nor the structural coverage may claim this range complete,
    // and the sibling the fail-stop never reached is declared skipped.
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
    assert_eq!(
        xref_ranges(&report.coverage.dimensions.artifact_structural.skipped),
        vec![(2, 3)]
    );
    assert!(report.page.has_more);
}

#[test]
fn a_class_candidate_names_the_bytes_that_make_it_damaged() {
    /// Message and raw entry name of the damaged-candidate diagnostic.
    fn damaged(report: &QueryReport) -> (&str, String) {
        let diagnostic = report
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "query_class_candidate_malformed")
            .expect("the damaged candidate is reported");
        let name = match diagnostic
            .provenance
            .as_ref()
            .map(|provenance| &provenance.location)
        {
            Some(Location::Entry { id, .. }) => id.raw_name.0.clone(),
            other => panic!("expected the damaged entry as the diagnostic origin, got {other:?}"),
        };
        (
            diagnostic.message.as_str(),
            String::from_utf8_lossy(&name).into_owned(),
        )
    }

    let fixture = fixture();
    let probe = |snapshot: &ArtifactSnapshot| {
        request(
            snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Target", "run", "()V"),
            &[ConsumerKind::Invocation],
        )
    };

    // Nothing at all cannot be a class file either, and the message says so.
    let archive = zip(&[
        (b"p/CodeFixture.class", &fixture.bytes),
        (b"p/Empty.class", b""),
        (b"p/Other.class", &fixture.bytes),
    ]);
    let snapshot = open(archive);
    let (report, _) = run_with_limits(&snapshot, &probe(&snapshot), limits());
    let (message, name) = damaged(&report);
    assert_eq!(name, "p/Empty.class");
    assert!(message.contains("0 byte(s)"), "{message}");
    assert!(message.contains("shorter than"), "{message}");
    // The reliable prefix of the candidate that scanned first is still published.
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::InvokeVirtual,
            Some(fixture.index.target_run),
            Some(8),
            Some(0xb6)
        )]
    );

    // Four bytes that are something else are the other branch: the message names the
    // bytes it read, and the entry name keeps its non-printable byte escaped instead of
    // turning it into lossy text.
    let archive = zip(&[
        (b"p/CodeFixture.class", &fixture.bytes),
        (b"p/W\xffrong.class", &[0x00, 0x01, 0x02, 0x03]),
        (b"p/Other.class", &fixture.bytes),
    ]);
    let snapshot = open(archive);
    let (report, _) = run_with_limits(&snapshot, &probe(&snapshot), limits());
    assert_eq!(
        failed_code(&report),
        Some("query_class_candidate_malformed")
    );
    let (message, name) = damaged(&report);
    assert_eq!(name, "p/W\u{fffd}rong.class");
    assert!(message.contains("W\\xFFrong.class"), "{message}");
    assert!(message.contains("00 01 02 03"), "{message}");
    assert!(message.contains("4 byte(s)"), "{message}");
    assert!(!message.contains("shorter than"), "{message}");
}

#[test]
fn the_class_candidate_rule_is_case_sensitive_and_shared() {
    let fixture = fixture();
    let garbage = b"this entry is not a class file";
    // The upper-case name holds the *same legal class bytes* as the lower-case one: a
    // case-folding rule would report its items a second time and materialize it, so the
    // assertions below are falsified by that mistake instead of agreeing with it.
    let archive = zip(&[
        (b"p/Good.class", &fixture.bytes),
        (b".class", &fixture.bytes),
        (b"p/Good.CLASS", &fixture.bytes),
        (b"docs/readme.txt", garbage),
    ]);
    let snapshot = open(archive);
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Target", "run", "()V"),
            &[ConsumerKind::Invocation],
        ),
        limits(),
    );

    // The lower-case suffix and the bare `.class` are both candidates: the name is a
    // physical scan rule and claims nothing about loadability.
    assert_eq!(
        evidence(&report),
        vec![
            (
                XrefOperation::InvokeVirtual,
                Some(fixture.index.target_run),
                Some(8),
                Some(0xb6)
            ),
            (
                XrefOperation::InvokeVirtual,
                Some(fixture.index.target_run),
                Some(8),
                Some(0xb6)
            ),
        ]
    );
    let entry_names = report
        .items
        .iter()
        .map(|item| match &item.source.location {
            Location::Code { method, .. } => match &method.owner.location {
                PhysicalClassLocation::ArchiveEntry { entry } => entry.raw_name.0.clone(),
                other => panic!("expected an archive entry, got {other:?}"),
            },
            other => panic!("expected a code location, got {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        entry_names,
        vec![b"p/Good.class".to_vec(), b".class".to_vec()]
    );
    // A name that differs in case and a plain resource are outside the candidate range:
    // even though the upper-case entry is a legal class file, it produces neither items
    // nor a damage diagnostic.
    assert!(is_complete(&report));
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert_eq!(
        budget.usage().entry_bytes,
        2 * u64::try_from(fixture.bytes.len()).unwrap(),
        "only the two candidates are materialized"
    );
}

#[test]
fn a_class_candidate_outside_the_requested_categories_is_not_scanned() {
    let archive = zip(&[
        (b"p/Broken.class", &[0xca, 0xfe, 0xba]),
        (b"docs/readme.txt", b"this entry is not a class file"),
    ]);
    let snapshot = open(archive);
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/Target"),
            &[ConsumerKind::Resource],
        ),
        limits(),
    );

    // The request names no class category, so the damaged candidate is outside its
    // range: it is neither read nor reported, and the run really is complete.
    assert!(report.items.is_empty());
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert!(is_complete(&report));
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(!report.page.has_more);
    let usage = budget.usage();
    assert_eq!(
        (usage.read_bytes, usage.class_bytes, usage.entry_bytes),
        (0, 0, 0),
        "an out-of-scope candidate is never materialized"
    );
}

// ---------------------------------------------------------------------------
// A08/A14: one damaged candidate inside a nested tree bounds the whole request
// ---------------------------------------------------------------------------

/// The same writing helper as `zip`, but with a real DEFLATE child for the nested
/// STORED/DEFLATED combination.
fn zip_with_deflate(entries: &[(&[u8], Vec<u8>, u16)]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data, method) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(*method))
                .start()
                .unwrap();
            if *method == 8 {
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

/// Minimal Java 8 class whose `call` method invokes `p/Target.run()V` `calls` times.
///
/// Returns the class bytes and the constant-pool index of the `Methodref`, so a test can
/// name the entry the reported items really carry.
fn calling_class(calls: usize) -> (Vec<u8>, u16) {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/Caller");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let run_name = pool.utf8(b"run");
    let void_descriptor = pool.utf8(b"()V");
    let run_nat = pool.name_and_type(run_name, void_descriptor);
    let target_name = pool.utf8(b"p/Target");
    let target_class = pool.class(target_name);
    let target_run = pool.member(10, target_class, run_nat);
    let call_name = pool.utf8(b"call");
    let code_name = pool.utf8(b"Code");

    let mut code = Vec::new();
    for _ in 0..calls {
        code.push(0xb6);
        u16b(&mut code, target_run);
    }
    code.push(0xb1);

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 52);
    u16b(&mut bytes, pool.declared());
    bytes.extend_from_slice(&pool.bytes());
    u16b(&mut bytes, 0x0021);
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, object_class);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 1);
    push_method(
        &mut bytes,
        0x0002,
        call_name,
        void_descriptor,
        Some((&code, &[])),
        code_name,
    );
    u16b(&mut bytes, 0);
    (bytes, target_run)
}

/// A query request over the whole artifact tree of a snapshot.
fn tree_request(snapshot: &ArtifactSnapshot, target: QueryTarget, max_items: u64) -> QueryRequest {
    QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target,
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::ArtifactTree {
                root_container: ContainerId("root".into()),
            },
        },
        consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
        max_items,
        cursor: None,
    }
}

/// `xref_scan_entries` ranges of one structural dimension side, sorted as `(start, end)`.
fn scan_ranges(ranges: &[CoverageRange]) -> Vec<(u64, u64)> {
    let mut found: Vec<(u64, u64)> = ranges
        .iter()
        .filter(|range| range.label.ends_with(":xref_scan_entries"))
        .map(|range| (range.start, range.end))
        .collect();
    found.sort_unstable();
    found
}

/// A08/A14: a truncated class candidate inside a nested child bounds every continuation
/// of the same request, and the readable sibling stays declared as skipped.
///
/// The root archive holds two child jars: `lib/a.jar` (STORED) with a call, then a
/// truncated `Broken.class`, then a readable sibling; `lib/b.jar` (DEFLATED) with its own
/// readable class. The scan is fail-stop, so only the prefix before the damaged entry is
/// published — but pages, result-item budgets and cancellation must all report the same
/// terminal dimension, the same reliable prefix and the same skipped intervals, and none
/// of them may turn into a complete scan.

#[test]
fn a_nested_truncated_candidate_bounds_pages_budgets_and_cancellation() {
    let (first_class, target_run) = calling_class(2);
    let (sibling_class, _) = calling_class(1);
    let (other_class, _) = calling_class(1);
    let child_a = zip_with_deflate(&[
        (b"p/First.class", first_class, 0),
        (b"Broken.class", vec![0xca, 0xfe, 0xba], 0),
        (b"p/Sibling.class", sibling_class, 0),
    ]);
    let child_b = zip_with_deflate(&[(b"p/Other.class", other_class, 0)]);
    let root = zip_with_deflate(&[(b"lib/a.jar", child_a, 0), (b"lib/b.jar", child_b, 8)]);
    let snapshot = open(root);
    let target = method_symbol("p/Target", "run", "()V");

    // The full run publishes the reliable prefix, ends in the damaged candidate's own
    // diagnostic and declares the unexamined sibling and the whole second child skipped.
    let (report, _) = run_with_limits(
        &snapshot,
        &tree_request(&snapshot, target.clone(), 0),
        limits(),
    );
    assert_eq!(
        evidence(&report),
        vec![
            (
                XrefOperation::InvokeVirtual,
                Some(target_run),
                Some(0),
                Some(0xb6)
            ),
            (
                XrefOperation::InvokeVirtual,
                Some(target_run),
                Some(3),
                Some(0xb6)
            ),
        ],
        "the prefix before the damaged entry is published"
    );
    assert_eq!(
        failed_code(&report),
        Some("query_class_candidate_malformed")
    );
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "query_class_candidate_malformed")
        .expect("the damaged candidate is reported");
    match diagnostic
        .provenance
        .as_ref()
        .map(|provenance| &provenance.location)
    {
        Some(Location::Entry { id, span }) => {
            assert_eq!(id.raw_name.0, b"Broken.class");
            assert_eq!(id.ordinal, 1, "the sibling sits in its child container");
            assert_eq!(
                id.origin.steps.len(),
                1,
                "the diagnostic keeps the nested origin"
            );
            assert_eq!(*span, ByteSpan::new(0, 3));
        }
        other => panic!("expected the damaged entry as the diagnostic origin, got {other:?}"),
    }
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
    assert_eq!(
        scan_ranges(&report.coverage.dimensions.artifact_structural.scanned),
        vec![(0, 1), (0, 2)],
        "the root entry the walk descended through and the child entries up to the \
         damaged one are examined"
    );
    assert_eq!(
        scan_ranges(&report.coverage.dimensions.artifact_structural.skipped),
        Vec::<(u64, u64)>::new(),
        "the sibling behind the fail-stop and the second child are unknown: the walk \
         stopped pulling before it examined them, so no range claims them"
    );
    assert!(
        report.page.has_more,
        "a failed scan stopped before the end of the range"
    );

    // A page boundary stops the scan before the damaged entry; its continuation cannot
    // become complete either, and keeps the same skipped intervals.
    let (first_page, _) = run_with_limits(
        &snapshot,
        &tree_request(&snapshot, target.clone(), 1),
        limits(),
    );
    assert_eq!(first_page.items.len(), 1);
    assert!(first_page.page.has_more);
    let cursor = first_page
        .page
        .cursor
        .clone()
        .expect("a truncated page hands out a cursor");
    let mut continuation = tree_request(&snapshot, target.clone(), 0);
    continuation.cursor = Some(cursor);
    let (second_page, _) = run_with_limits(&snapshot, &continuation, limits());
    assert_eq!(
        evidence(&second_page),
        vec![(
            XrefOperation::InvokeVirtual,
            Some(target_run),
            Some(3),
            Some(0xb6)
        )],
        "the continuation publishes the remaining prefix item without repeating the first"
    );
    assert_eq!(
        failed_code(&second_page),
        Some("query_class_candidate_malformed")
    );
    assert_eq!(
        second_page.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
    assert_eq!(
        scan_ranges(&second_page.coverage.dimensions.artifact_structural.scanned),
        vec![(0, 1), (0, 2)],
        "the continuation resumes in the entry the boundary names and walks through the \
         root entry above it, so both are examined by this invocation"
    );
    assert_eq!(
        scan_ranges(&second_page.coverage.dimensions.artifact_structural.skipped),
        Vec::<(u64, u64)>::new(),
        "the entries behind the fail-stop stay unknown instead of being named"
    );
    assert!(
        second_page.page.has_more,
        "the fail-stop cannot be read as the end of the range"
    );

    // An exhausted result-item budget names its own dimension and keeps the one published
    // item as the reliable prefix. The walk bills the container directories it opens and
    // the items it publishes from the same `ResultItems` budget, so the tight budget is
    // measured on a run that stops after the first published item — instead of being
    // derived from a second, eager enumeration of the whole tree, which is exactly the
    // work a small page must not pay for.
    let (first_only, first_budget) = run_with_limits(
        &snapshot,
        &tree_request(&snapshot, target.clone(), 1),
        limits(),
    );
    assert_eq!(first_only.items.len(), 1);
    let provider_items = first_budget.usage().result_items;
    assert!(
        provider_items > 0,
        "the walk bills the directories it opens"
    );
    let mut tight = limits();
    tight.result_items = provider_items;
    let (budgeted, budget) = run_with_limits(
        &snapshot,
        &tree_request(&snapshot, target.clone(), 0),
        tight,
    );
    assert_eq!(budgeted.items.len(), 1);
    assert_eq!(
        partial_dimension(&budgeted),
        Some(BudgetDimension::ResultItems)
    );
    assert_eq!(budget.usage().result_items, provider_items);
    assert!(
        diagnostic_codes(&budgeted).contains(&"budget_exceeded_result_items"),
        "{:?}",
        budgeted.diagnostics
    );
    assert_eq!(
        scan_ranges(&budgeted.coverage.dimensions.artifact_structural.scanned),
        vec![(0, 1), (0, 1)],
        "the budget stopped the scan after the first child entry was examined"
    );
    assert_eq!(
        scan_ranges(&budgeted.coverage.dimensions.artifact_structural.skipped),
        Vec::<(u64, u64)>::new(),
        "nothing behind the stop was examined, so nothing is named for it"
    );
    assert!(budgeted.page.has_more);

    // Cancellation before the scan keeps nothing published and is never reported as
    // complete. The scope's own walk is cancelled before its first pull, so this
    // invocation examined no range at all: `Partial` states the uncovered scope.
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled_budget = Budget::with_cancellation_token(limits(), token);
    let cancelled = Engine::new()
        .query(
            &snapshot,
            &tree_request(&snapshot, target, 0),
            &mut cancelled_budget,
        )
        .expect("a cancelled query returns a report");
    assert!(is_cancelled(&cancelled));
    assert!(cancelled.items.is_empty());
    assert!(cancelled.page.cursor.is_none());
    assert!(
        cancelled.page.has_more,
        "the scan never reached the end of the range"
    );
    assert!(diagnostic_codes(&cancelled).contains(&"cancelled"));
    assert_ne!(
        cancelled.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(
        cancelled
            .coverage
            .dimensions
            .artifact_structural
            .scanned
            .is_empty()
            && cancelled
                .coverage
                .dimensions
                .artifact_structural
                .skipped
                .is_empty(),
        "a cancelled walk names no range it never examined: {:?}",
        cancelled.coverage.dimensions.artifact_structural
    );
}

// ---------------------------------------------------------------------------
// Deterministic order: the page cursor depends on it
// ---------------------------------------------------------------------------

#[test]
fn page_continuation_replays_the_code_items_in_position_order() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let mut request = request(
        &snapshot,
        QueryRelation::MentionsSymbol,
        class_symbol("p/Shape"),
        &[ConsumerKind::Type],
    );
    request.max_items = 2;

    let (first, _) = run_with_limits(&snapshot, &request, limits());
    assert_eq!(
        evidence(&first),
        vec![
            (
                XrefOperation::New,
                Some(fixture.index.shape),
                Some(0),
                Some(0xbb)
            ),
            (
                XrefOperation::CheckCast,
                Some(fixture.index.shape),
                Some(11),
                Some(0xc0)
            ),
        ]
    );
    assert!(first.page.has_more);
    let mut continuation = request.clone();
    continuation.cursor = Some(first.page.cursor.clone().expect("a continuation cursor"));

    let (second, _) = run_with_limits(&snapshot, &continuation, limits());
    assert_eq!(
        evidence(&second),
        vec![
            (
                XrefOperation::InstanceOf,
                Some(fixture.index.shape),
                Some(14),
                Some(0xc1)
            ),
            (
                XrefOperation::MultiNewArray,
                Some(fixture.index.shape),
                Some(20),
                Some(0xc5)
            ),
        ],
        "the continuation neither repeats nor skips a position"
    );
    // The page limit bounds every page, including a continued one, and the descriptor
    // type of the `invokedynamic` site is the position that did not fit here.
    assert!(second.page.has_more);
    let mut third = request.clone();
    third.cursor = Some(second.page.cursor.clone().expect("a continuation cursor"));

    let (third, _) = run_with_limits(&snapshot, &third, limits());
    assert_eq!(
        evidence(&third),
        vec![(
            XrefOperation::InvokeDynamic,
            Some(fixture.index.dynamic_site),
            Some(39),
            Some(0xba)
        )],
        "the last position of the scan is the descriptor type of the dynamic site"
    );
    assert!(!third.page.has_more);
    assert!(third.page.cursor.is_none());
}

// ---------------------------------------------------------------------------
// Budget, cancellation and undecodable members
// ---------------------------------------------------------------------------

#[test]
fn a_code_bytes_stop_keeps_the_prefix_it_already_billed() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let mut tight = limits();
    // `<init>` needs five bytes, then `exercises` decodes its first instruction.
    tight.code_bytes = 8;

    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("java/lang/Object", "<init>", "()V"),
            &[ConsumerKind::Invocation],
        ),
        tight.clone(),
    );
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::InvokeSpecial,
            Some(fixture.index.object_init),
            Some(1),
            Some(0xb7)
        )],
        "the published prefix stops before the instruction that did not fit"
    );
    assert_eq!(partial_dimension(&report), Some(BudgetDimension::CodeBytes));
    assert_eq!(
        budget.usage().code_bytes,
        8,
        "the refused instruction is not charged"
    );
    assert_eq!(
        budget.usage().result_items,
        u64::try_from(report.items.len()).unwrap()
    );
    assert!(report.page.has_more);
    assert!(report.page.cursor.is_some());
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
    assert!(diagnostic_codes(&report).contains(&"budget_exceeded_code_bytes"));

    // The member that stopped keeps the instructions it did decode: `new` at BCI 0 was
    // charged and published, `checkcast` at BCI 11 never ran.
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/Shape"),
            &[ConsumerKind::Type],
        ),
        tight,
    );
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::New,
            Some(fixture.index.shape),
            Some(0),
            Some(0xbb)
        )]
    );
    assert_eq!(budget.usage().code_bytes, 8);
    assert_eq!(partial_dimension(&report), Some(BudgetDimension::CodeBytes));
    assert_eq!(budget.usage().result_items, 1);
}

#[test]
fn result_items_and_cancellation_stop_without_unbilled_items() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let request = request(
        &snapshot,
        QueryRelation::MentionsSymbol,
        class_symbol("p/Shape"),
        &[ConsumerKind::Type],
    );

    let mut tight = limits();
    tight.result_items = 1;
    let (report, budget) = run_with_limits(&snapshot, &request, tight);
    assert_eq!(
        code_items(&report).len(),
        1,
        "the page keeps exactly the items it could bill"
    );
    assert_eq!(
        partial_dimension(&report),
        Some(BudgetDimension::ResultItems)
    );
    assert_eq!(budget.usage().result_items, 1);
    assert!(diagnostic_codes(&report).contains(&"budget_exceeded_result_items"));

    let mut none = limits();
    none.result_items = 0;
    let (report, budget) = run_with_limits(&snapshot, &request, none);
    assert!(report.items.is_empty());
    assert_eq!(
        partial_dimension(&report),
        Some(BudgetDimension::ResultItems)
    );
    assert_eq!(budget.usage().result_items, 0);
    assert!(diagnostic_codes(&report).contains(&"budget_exceeded_result_items"));

    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let report = Engine::new()
        .query(&snapshot, &request, &mut budget)
        .expect("a cancelled query still reports");
    assert!(report.items.is_empty());
    assert!(is_cancelled(&report));
    assert!(diagnostic_codes(&report).contains(&"cancelled"));
    let usage = budget.usage();
    assert_eq!(
        (usage.class_bytes, usage.code_bytes, usage.result_items),
        (0, 0, 0),
        "a cancelled request reads and bills nothing"
    );
}

/// Class file with a truncated method body followed by a decodable caller.
///
/// Declaration order matters: the undecodable member comes first, so the test proves the
/// scan still reports the members that do decode.
fn broken_member_fixture() -> (Vec<u8>, u16) {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/Broken");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let run_name = pool.utf8(b"run");
    let void_descriptor = pool.utf8(b"()V");
    let run_nat = pool.name_and_type(run_name, void_descriptor);
    let target_name = pool.utf8(b"p/Target");
    let target_class = pool.class(target_name);
    let target_run = pool.member(10, target_class, run_nat);
    let broken_name = pool.utf8(b"broken");
    let call_name = pool.utf8(b"call");
    let code_name = pool.utf8(b"Code");

    // `invokevirtual` with one operand byte missing: the instruction stream cannot be
    // decoded to its end.
    let broken_code = vec![0xb6, 0x00];
    let mut call_code = vec![0x2a, 0xb6];
    u16b(&mut call_code, target_run);
    call_code.push(0xb1);

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0); // minor
    u16b(&mut bytes, 52); // major
    u16b(&mut bytes, pool.declared());
    bytes.extend_from_slice(&pool.bytes());
    u16b(&mut bytes, 0x0021);
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, object_class);
    u16b(&mut bytes, 0); // interfaces
    u16b(&mut bytes, 0); // fields
    u16b(&mut bytes, 2); // methods
    push_method(
        &mut bytes,
        0x0002,
        broken_name,
        void_descriptor,
        Some((&broken_code, &[])),
        code_name,
    );
    push_method(
        &mut bytes,
        0x0002,
        call_name,
        void_descriptor,
        Some((&call_code, &[])),
        code_name,
    );
    u16b(&mut bytes, 0); // class attributes
    (bytes, target_run)
}

#[test]
fn an_undecodable_member_is_reported_without_hiding_the_other_members() {
    let (bytes, target_run) = broken_member_fixture();
    let snapshot = open(bytes);
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Target", "run", "()V"),
            &[ConsumerKind::Invocation],
        ),
        limits(),
    );

    // The member that decodes is still scanned and its reference reported.
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::InvokeVirtual,
            Some(target_run),
            Some(1),
            Some(0xb6)
        )]
    );
    // The stopped member is named with its own coordinates, and the report does not
    // claim a complete scan.
    let member = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "query_code_stopped")
        .expect("a member-level diagnostic for the stopped member");
    assert_eq!(member.severity, DiagnosticSeverity::Error);
    match member
        .provenance
        .as_ref()
        .map(|provenance| &provenance.location)
    {
        Some(Location::Code { method, bci }) => {
            assert_eq!(*bci, 0);
            assert_eq!(method.name.0, b"broken");
        }
        other => panic!("expected the stopped member's code location, got {other:?}"),
    }
    assert_eq!(failed_code(&report), Some("classfile_instruction_decode"));
    assert!(diagnostic_codes(&report).contains(&"classfile_instruction_decode"));
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
    // One result item plus the member diagnostic; the terminal diagnostic is control
    // metadata and is not charged.
    assert_eq!(
        budget.usage().result_items,
        u64::try_from(report.items.len()).unwrap() + 1
    );
    assert!(report.page.has_more);
}

/// Class file whose only body reaches `return` by jumping over an invocation.
fn dead_code_fixture() -> (Vec<u8>, u16) {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/Dead");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let run_name = pool.utf8(b"run");
    let void_descriptor = pool.utf8(b"()V");
    let run_nat = pool.name_and_type(run_name, void_descriptor);
    let target_name = pool.utf8(b"p/Target");
    let target_class = pool.class(target_name);
    let target_run = pool.member(10, target_class, run_nat);
    let dead_name = pool.utf8(b"dead");
    let code_name = pool.utf8(b"Code");

    // 0: goto +6; 3: invokevirtual p/Target.run; 6: return — the call is never executed.
    let mut code = vec![0xa7, 0x00, 0x06, 0xb6];
    u16b(&mut code, target_run);
    code.push(0xb1);

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0); // minor
    u16b(&mut bytes, 52); // major
    u16b(&mut bytes, pool.declared());
    bytes.extend_from_slice(&pool.bytes());
    u16b(&mut bytes, 0x0021);
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, object_class);
    u16b(&mut bytes, 0); // interfaces
    u16b(&mut bytes, 0); // fields
    u16b(&mut bytes, 1); // methods
    push_method(
        &mut bytes,
        0x0002,
        dead_name,
        void_descriptor,
        Some((&code, &[])),
        code_name,
    );
    u16b(&mut bytes, 0); // class attributes
    (bytes, target_run)
}

#[test]
fn references_in_unreachable_code_are_still_reported() {
    let (bytes, target_run) = dead_code_fixture();
    let snapshot = open(bytes);
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Target", "run", "()V"),
            &[ConsumerKind::Invocation],
        ),
        limits(),
    );
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::InvokeVirtual,
            Some(target_run),
            Some(3),
            Some(0xb6)
        )],
        "the scan is linear: it reports the reference the bytes contain, not the one \
         execution would reach"
    );
    assert!(is_complete(&report));
    assert_eq!(
        budget.usage().code_bytes,
        7,
        "every instruction is decoded, including the unreachable ones"
    );
    assert!(
        report.diagnostics.is_empty(),
        "no control-flow analysis is needed to read these bytes"
    );
}

/// Class file whose only body `return`s before the invocation: the trailing bytes are
/// unreachable, and no branch target points at them.
fn trailing_dead_code_fixture() -> (Vec<u8>, u16) {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/Trailing");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let run_name = pool.utf8(b"run");
    let void_descriptor = pool.utf8(b"()V");
    let run_nat = pool.name_and_type(run_name, void_descriptor);
    let target_name = pool.utf8(b"p/Target");
    let target_class = pool.class(target_name);
    let target_run = pool.member(10, target_class, run_nat);
    let call_name = pool.utf8(b"call");
    let code_name = pool.utf8(b"Code");

    // 0: return; 1: invokevirtual p/Target.run; 4: return.
    let mut code = vec![0xb1, 0xb6];
    u16b(&mut code, target_run);
    code.push(0xb1);

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 52);
    u16b(&mut bytes, pool.declared());
    bytes.extend_from_slice(&pool.bytes());
    u16b(&mut bytes, 0x0021);
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, object_class);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 1);
    push_method(
        &mut bytes,
        0x0002,
        call_name,
        void_descriptor,
        Some((&code, &[])),
        code_name,
    );
    u16b(&mut bytes, 0);
    (bytes, target_run)
}

/// A17: the scan reads the instruction bytes the class holds in one linear pass, without
/// building a CFG, SSA form or an AST.
///
/// A reachability analysis would never visit the invocation behind the leading `return`
/// (no branch or handler names it), and a graph-based pass would either skip those bytes
/// or visit them while also charging more than the instruction widths. The assertions
/// stay on the public result: the reference is reported at its real BCI, the `CodeBytes`
/// charge is exactly the decoded instruction widths, and the run completes with no
/// diagnostics. Production modules and dependencies are held to the same boundary by the
/// CI dependency-tree check, not by textual inspection here.
#[test]
fn a_linear_scan_reads_trailing_dead_bytes_without_reachability_analysis() {
    let (bytes, target_run) = trailing_dead_code_fixture();
    let snapshot = open(bytes);
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Target", "run", "()V"),
            &[ConsumerKind::Invocation],
        ),
        limits(),
    );
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::InvokeVirtual,
            Some(target_run),
            Some(1),
            Some(0xb6)
        )],
        "the reference after the unconditional return is still a byte fact"
    );
    assert_eq!(report.coverage.scanned_items, 1);
    assert_eq!(
        budget.usage().code_bytes,
        5,
        "every instruction width is decoded exactly once: return, invocation, return"
    );
    assert!(is_complete(&report));
    assert!(
        report.diagnostics.is_empty(),
        "reading unreachable bytes needs no control-flow analysis: {:?}",
        report.diagnostics
    );
}

// ---------------------------------------------------------------------------
// A03/A17: the descriptor types a real use site consumes (R3)
// ---------------------------------------------------------------------------

/// The R3 sample: the design's structure sample, plus a `MethodType` no instruction uses.
///
/// The first nine constant-pool entries are exactly the sample the design records
/// (`MethodTypeOnly`, its class, `java/lang/Object`, its class, `probe`, `()V`, `Code`,
/// `(Lp/OnlyParam;)Lp/OnlyResult;`, the `MethodType`), so the site is entry 9 and the body
/// is `12 09 57 B1`; the last two entries carry a descriptor-bearing entry nothing
/// consumes.
struct MethodTypeFixture {
    bytes: Vec<u8>,
    /// `CONSTANT_MethodType` the `ldc` consumes.
    method_type: u16,
    /// `Utf8` entry holding the descriptor the `ldc` consumes.
    descriptor_index: u16,
    /// `CONSTANT_MethodType` no instruction consumes.
    unused_method_type: u16,
    unused_descriptor_index: u16,
    /// Class-file offset of the consumed instruction.
    code_offset: u64,
}

impl MethodTypeFixture {
    /// Class-file range of the `ldc`: the opcode and its one-byte index.
    fn instruction_span(&self) -> ByteSpan {
        ByteSpan::new(self.code_offset, 2)
    }
}

fn method_type_fixture() -> MethodTypeFixture {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"MethodTypeOnly");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let super_class = pool.class(object_name);
    let probe = pool.utf8(b"probe");
    let void_descriptor = pool.utf8(b"()V");
    let code_name = pool.utf8(b"Code");
    let descriptor_index = pool.utf8(b"(Lp/OnlyParam;)Lp/OnlyResult;");
    let method_type = pool.method_type(descriptor_index);
    let unused_descriptor_index = pool.utf8(b"(Lp/UnusedParam;)Lp/UnusedResult;");
    let unused_method_type = pool.method_type(unused_descriptor_index);
    assert_eq!(
        method_type, 9,
        "the design's sample puts the consumed entry at index 9"
    );
    // The unused entry really is a `MethodType` over the descriptor the X0 probe below
    // reports, so that negative control is wired and not vacuous.
    let mut unused_entry = vec![16];
    unused_entry.extend_from_slice(&unused_descriptor_index.to_be_bytes());
    assert_eq!(
        pool.entries[usize::from(unused_method_type) - 1],
        unused_entry
    );

    // Every type either descriptor names exists nowhere else: no `Utf8` entry spells one
    // of these names, so no `CONSTANT_Class` can name one of them.
    for name in [
        b"p/OnlyParam".as_slice(),
        b"p/OnlyResult",
        b"p/UnusedParam",
        b"p/UnusedResult",
    ] {
        assert!(
            utf8_indexes(&pool, name).is_empty(),
            "{} must exist only inside a descriptor",
            String::from_utf8_lossy(name)
        );
    }

    let mut code = vec![0x12];
    code.push(u8::try_from(method_type).expect("the sample entry fits one byte"));
    code.push(0x57); // pop
    code.push(0xb1); // return
    let (bytes, offsets) = assemble(
        &pool,
        code_name,
        this_class,
        super_class,
        &[Body {
            access: 0x0009, // ACC_PUBLIC | ACC_STATIC
            name: probe,
            descriptor: void_descriptor,
            max_stack: 1,
            max_locals: 0,
            code,
        }],
        &[],
    );
    MethodTypeFixture {
        bytes,
        method_type,
        descriptor_index,
        unused_method_type,
        unused_descriptor_index,
        code_offset: offsets[0],
    }
}

/// One `ldc_w` per descriptor-bearing constant a load or dynamic instruction consumes.
struct ConsumersFixture {
    bytes: Vec<u8>,
    field_handle: u16,
    method_handle: u16,
    condy: u16,
    site: u16,
}

fn descriptor_consumers_fixture() -> ConsumersFixture {
    let mut pool = Pool::default();
    let code_name = pool.utf8(b"Code");
    let class_name = pool.utf8(b"p/Handles");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let super_class = pool.class(object_name);
    let use_method = pool.utf8(b"use");
    let void_descriptor = pool.utf8(b"()V");
    let bootstrap_name = pool.utf8(b"BootstrapMethods");

    // A bootstrap method whose own descriptor names no object type, so this fixture's
    // bootstrap table adds no type item of its own.
    let bsm_owner = pool.utf8(b"p/Bsm");
    let bsm_class = pool.class(bsm_owner);
    let bsm_name = pool.utf8(b"make");
    let bsm_nat = pool.name_and_type(bsm_name, void_descriptor);
    let bsm_ref = pool.member(10, bsm_class, bsm_nat);
    let bsm_handle = pool.method_handle(6, bsm_ref);

    // `p/Holder.value:Lp/FieldOnly;` behind a field handle.
    let holder_name = pool.utf8(b"p/Holder");
    let holder_class = pool.class(holder_name);
    let value_name = pool.utf8(b"value");
    let field_descriptor = pool.utf8(b"Lp/FieldOnly;");
    let value_nat = pool.name_and_type(value_name, field_descriptor);
    let value_ref = pool.member(9, holder_class, value_nat);
    let field_handle = pool.method_handle(1, value_ref);

    // `p/Holder.run(Lp/Shared;)Lp/MethodOnly;` behind a method handle.
    let run_name = pool.utf8(b"run");
    let method_descriptor = pool.utf8(b"(Lp/Shared;)Lp/MethodOnly;");
    let run_nat = pool.name_and_type(run_name, method_descriptor);
    let run_ref = pool.member(10, holder_class, run_nat);
    let method_handle = pool.method_handle(6, run_ref);

    // A condy whose own field descriptor names a type of its own.
    let condy_name = pool.utf8(b"condy");
    let condy_descriptor = pool.utf8(b"Lp/CondyOnly;");
    let condy_nat = pool.name_and_type(condy_name, condy_descriptor);
    let condy = pool.dynamic(0, condy_nat);

    // A call site whose descriptor repeats a type and shares one with the method handle.
    let site_name = pool.utf8(b"site");
    let site_descriptor = pool.utf8(b"(Lp/SiteOnly;Lp/Dup;Lp/Dup;)Lp/Shared;");
    let site_nat = pool.name_and_type(site_name, site_descriptor);
    let site = pool.invoke_dynamic(0, site_nat);

    // Every type the descriptors name exists nowhere else in the pool.
    for name in [
        b"p/FieldOnly".as_slice(),
        b"p/MethodOnly",
        b"p/CondyOnly",
        b"p/SiteOnly",
        b"p/Dup",
        b"p/Shared",
    ] {
        assert!(
            utf8_indexes(&pool, name).is_empty(),
            "{} must exist only inside a descriptor",
            String::from_utf8_lossy(name)
        );
    }

    // 0: ldc_w field handle, 4: ldc_w method handle, 8: ldc_w condy,
    // 12: invokedynamic, 17: return.
    let mut code = Vec::new();
    code.push(0x13);
    u16b(&mut code, field_handle);
    code.push(0x57);
    code.push(0x13);
    u16b(&mut code, method_handle);
    code.push(0x57);
    code.push(0x13);
    u16b(&mut code, condy);
    code.push(0x57);
    code.push(0xba);
    u16b(&mut code, site);
    code.push(0);
    code.push(0);
    code.push(0xb1);
    let (bytes, offsets) = assemble(
        &pool,
        code_name,
        this_class,
        super_class,
        &[Body {
            access: 0x0002,
            name: use_method,
            descriptor: void_descriptor,
            max_stack: 2,
            max_locals: 0,
            code,
        }],
        &[(bootstrap_name, bootstrap_content(bsm_handle))],
    );
    assert_eq!(offsets.len(), 1);
    ConsumersFixture {
        bytes,
        field_handle,
        method_handle,
        condy,
        site,
    }
}

/// A class whose only descriptor-bearing entry is consumed with the descriptor `text`, which
/// `ldc` then loads.
fn method_type_fixture_with(descriptor: &[u8]) -> Vec<u8> {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/Malformed");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let super_class = pool.class(object_name);
    let probe = pool.utf8(b"probe");
    let void_descriptor = pool.utf8(b"()V");
    let code_name = pool.utf8(b"Code");
    let descriptor_index = pool.utf8(descriptor);
    let method_type = pool.method_type(descriptor_index);
    let mut code = vec![0x12];
    code.push(u8::try_from(method_type).expect("fits one byte"));
    code.push(0x57); // pop
    code.push(0xb1); // return
    let (bytes, _) = assemble(
        &pool,
        code_name,
        this_class,
        super_class,
        &[Body {
            access: 0x0002,
            name: probe,
            descriptor: void_descriptor,
            max_stack: 1,
            max_locals: 0,
            code,
        }],
        &[],
    );
    bytes
}

/// A class whose only descriptor-bearing entry is consumed with a descriptor that cannot
/// be a method descriptor.
fn malformed_descriptor_fixture() -> Vec<u8> {
    // `(` opens a parameter list that never closes.
    method_type_fixture_with(b"(Lp/Unterminated;")
}

#[test]
fn a_used_method_type_descriptor_reports_its_types() {
    let fixture = method_type_fixture();
    let snapshot = open(fixture.bytes.clone());

    // Type-only: the request triggers the read itself, without Constant, Invocation or
    // Bootstrap, and the item carries the instruction's own coordinates.
    for owner in ["p/OnlyResult", "p/OnlyParam"] {
        let report = run(
            &snapshot,
            &request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                class_symbol(owner),
                &[ConsumerKind::Type],
            ),
        );
        assert_eq!(
            evidence(&report),
            vec![(
                XrefOperation::Ldc,
                Some(fixture.method_type),
                Some(0),
                Some(0x12)
            )],
            "{owner}"
        );
        let item = code_items(&report)[0];
        assert_eq!(item.consumer, Some(ConsumerKind::Type));
        assert_eq!(item.derivation, XrefDerivation::StructuralConsumer);
        assert_eq!(item.certainty, XrefCertainty::Exact);
        assert_eq!(item.resolution, QueryResolution::NotRequested);
        assert!(item.evidence.via.is_empty());
        let (method, bci) = code_location(item);
        assert_eq!(bci, 0);
        assert_eq!(method.name.0, b"probe");
        assert_eq!(method.descriptor.0, b"()V");
        assert_eq!(
            item.evidence.attribute.as_ref().map(|name| name.0.clone()),
            Some(b"Code".to_vec())
        );
        let span = item
            .evidence
            .span
            .clone()
            .expect("the instruction span is recorded");
        assert_eq!(span, fixture.instruction_span());
        assert_eq!(
            &fixture.bytes[usize::try_from(span.start).unwrap()
                ..usize::try_from(span.start + span.length).unwrap()],
            &[0x12, u8::try_from(fixture.method_type).unwrap()],
            "the span is the instruction's own bytes"
        );
        assert!(is_complete(&report));
        assert_eq!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    }

    // The descriptor bytes themselves are not a type: a literal request for them answers
    // nothing under `Type`, and the raw candidate belongs to the X0 producer.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            literal_string("(Lp/OnlyParam;)Lp/OnlyResult;"),
            &[ConsumerKind::Type],
        ),
    );
    assert!(report.items.is_empty(), "{:?}", report.items);

    // A combined request answers the same product.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/OnlyResult"),
            &[
                ConsumerKind::Type,
                ConsumerKind::Signature,
                ConsumerKind::Constant,
                ConsumerKind::Bootstrap,
            ],
        ),
    );
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::Ldc,
            Some(fixture.method_type),
            Some(0),
            Some(0x12)
        )]
    );

    // The category is the gate: a `Constant`-only request never parses the descriptor.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/OnlyResult"),
            &[ConsumerKind::Constant],
        ),
    );
    assert!(report.items.is_empty());
    assert!(is_complete(&report));

    // An unused `MethodType` of the same class stays a raw candidate: it produces no type
    // item, and its descriptor bytes are still reportable by the X0 probe.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/UnusedResult"),
            &[ConsumerKind::Type],
        ),
    );
    assert!(
        report.items.is_empty(),
        "nothing consumes the unused entry: {:?}",
        report.items
    );
    assert!(is_complete(&report));
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::ConstantPoolContains,
            literal_string("(Lp/UnusedParam;)Lp/UnusedResult;"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(pool_indexes(&report), vec![fixture.unused_descriptor_index]);
    assert_eq!(
        report.items[0].derivation,
        XrefDerivation::ConstantPoolCandidate
    );
    assert_eq!(report.items[0].consumer, None);
    assert_ne!(
        fixture.descriptor_index, fixture.unused_descriptor_index,
        "the consumed and the unused descriptor are different entries"
    );
    assert_ne!(
        fixture.method_type, fixture.unused_method_type,
        "the consumed and the unused entry are different entries"
    );
}

#[test]
fn descriptor_types_of_handles_sites_and_condy_come_from_the_use_site() {
    let fixture = descriptor_consumers_fixture();
    let snapshot = open(fixture.bytes.clone());

    let cases: [(&str, u16, u32, u8); 3] = [
        ("p/FieldOnly", fixture.field_handle, 0, 0x13),
        ("p/MethodOnly", fixture.method_handle, 4, 0x13),
        ("p/CondyOnly", fixture.condy, 8, 0x13),
    ];
    for (owner, index, bci, opcode) in cases {
        let report = run(
            &snapshot,
            &request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                class_symbol(owner),
                &[ConsumerKind::Type],
            ),
        );
        assert_eq!(
            evidence(&report),
            vec![(XrefOperation::Ldc, Some(index), Some(bci), Some(opcode))],
            "{owner}"
        );
        assert_eq!(code_items(&report)[0].consumer, Some(ConsumerKind::Type));
        assert!(is_complete(&report));
    }

    // A type the site's descriptor spells twice is reported once, and the site's own
    // operation and position are the ones recorded.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/Dup"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::InvokeDynamic,
            Some(fixture.site),
            Some(12),
            Some(0xba)
        )],
        "a type the descriptor repeats is one item"
    );
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/SiteOnly"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(
        evidence(&report),
        vec![(
            XrefOperation::InvokeDynamic,
            Some(fixture.site),
            Some(12),
            Some(0xba)
        )]
    );

    // One type named by two different instructions is one item per instruction, in
    // class-file position order.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/Shared"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(
        evidence(&report),
        vec![
            (
                XrefOperation::Ldc,
                Some(fixture.method_handle),
                Some(4),
                Some(0x13)
            ),
            (
                XrefOperation::InvokeDynamic,
                Some(fixture.site),
                Some(12),
                Some(0xba)
            ),
        ]
    );

    // One instruction, two products: the member a `ldc` consumed and the types its
    // descriptor names. They are addressed by different targets (a member symbol and a
    // class symbol), so each query shows its own product with the same coordinates.
    let kinds = [ConsumerKind::Type, ConsumerKind::Constant];
    let member = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            field_symbol("p/Holder", "value", "Lp/FieldOnly;"),
            &kinds,
        ),
    );
    assert_eq!(
        evidence(&member),
        vec![(
            XrefOperation::Ldc,
            Some(fixture.field_handle),
            Some(0),
            Some(0x13)
        )]
    );
    assert_eq!(
        code_items(&member)[0].consumer,
        Some(ConsumerKind::Constant)
    );
    let types = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/FieldOnly"),
            &kinds,
        ),
    );
    assert_eq!(
        evidence(&types),
        vec![(
            XrefOperation::Ldc,
            Some(fixture.field_handle),
            Some(0),
            Some(0x13)
        )]
    );
    assert_eq!(code_items(&types)[0].consumer, Some(ConsumerKind::Type));
    assert_eq!(
        member.items[0].evidence, types.items[0].evidence,
        "both products describe the same instruction"
    );

    // The gate: without the `Type` category the descriptor is never parsed, so an entry
    // whose type exists nowhere else is not reported.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/CondyOnly"),
            &[ConsumerKind::Constant],
        ),
    );
    assert!(report.items.is_empty());
    assert!(is_complete(&report));
}

#[test]
fn a_malformed_descriptor_of_a_used_entry_is_a_structured_stop() {
    let bytes = malformed_descriptor_fixture();
    let snapshot = open(bytes);

    // The `Type` category parses the descriptor the use site consumed, so a descriptor
    // that cannot be a method descriptor is a structured stop instead of a silent miss.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/Anything"),
            &[ConsumerKind::Type],
        ),
    );
    assert!(report.items.is_empty());
    assert_eq!(failed_code(&report), Some("query_descriptor_malformed"));
    assert_ne!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );

    // A request that never asks for those types never parses the descriptor.
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/Anything"),
            &[ConsumerKind::Constant],
        ),
    );
    assert!(report.items.is_empty());
    assert!(is_complete(&report));
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);

    // The descriptor grammar stays the descriptor grammar: a *signature* written into a
    // `MethodType` is not a method descriptor, so the descriptor path still refuses it. This
    // is the guard that the signature fix did not loosen the descriptor productions — the
    // two grammars are read by different code (JVMS 4.3 versus 4.7.9.1).
    let signature_shaped = b"()Ljava/util/List<Ljava/lang/String;>;";
    let bytes = method_type_fixture_with(signature_shaped);
    let snapshot = open(bytes);
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("java/util/List"),
            &[ConsumerKind::Type],
        ),
    );
    assert!(report.items.is_empty(), "{:?}", report.items);
    assert_eq!(failed_code(&report), Some("query_descriptor_malformed"));
    let report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("java/lang/String"),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(failed_code(&report), Some("query_descriptor_malformed"));
}
