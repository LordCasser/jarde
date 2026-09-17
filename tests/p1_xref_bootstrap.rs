//! P1 bootstrap-consumer acceptance: the deferred bootstrap/condy graph, its `via` paths,
//! its derived bounds and its diagnostics — all through the public `Engine::query` entry
//! point over hand-built class files.
//!
//! Hand-built fixtures are used because the acceptance pins exact constant-pool indexes,
//! argument positions, BCI and class-file spans that a compiled sample cannot control.
//!
//! * `real_lambda_fixture` is the **A04 shape**: a real
//!   `LambdaMetafactory.metafactory` bootstrap site whose declared parameter count is the
//!   three JVM-supplied leading arguments plus its three static arguments, so the invocation
//!   it describes is linkable (JVMS §5.4.3.6). Its test asserts the implementation handle at
//!   static `argument_index` 1, the SAM name and invoked type on the site's own `NameAndType`
//!   (reported by the code consumer), and that no fact claims the implementation is called.
//! * `fixture` is a **structural sample**, deliberately not a linkable bootstrap: its
//!   `p/Sample.lambdaLike` and `p/Custom.bsm` members declare parameter counts that are not
//!   tuned to their entries' static argument counts, because its job is to pin mixed argument
//!   kinds, graph sharing, nested condy and unused entries — positions and paths, not
//!   linkage. Its lambda-like site (SAM name `apply`) and its custom site share a
//!   nested-condy subgraph (A05), including the same `Dynamic` entry as two different static
//!   arguments; two `ldc` sites reach the nested nodes; one `BootstrapMethods` entry, its
//!   handle and its `Dynamic` site are reached by no use-site; one member has no body.
//! * The smaller fixtures isolate one property each: the `ldc` family over dynamic entries,
//!   a condy chain that fits the derived bound, a chain that outgrows it, a cycle, a table
//!   that cannot be used as declared, a member body that does not decode, a class whose
//!   table nothing consumes, the raw pool probe's guard, and the negative control for an
//!   entry only an unused table entry can reach.
//!
//! Every request that expects bootstrap edges names `Bootstrap` as its only consumer
//! category, so every item in those reports comes from the bootstrap stream and the class
//! consumer sub-scans cannot be mistaken for it.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

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

/// STORED ZIP whose entries keep the order they are listed in.
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

/// The bootstrap consumer alone: the request that makes this stream the only producer.
const BOOTSTRAP: [ConsumerKind; 1] = [ConsumerKind::Bootstrap];

fn method_symbol(owner: &str, name: &str, descriptor: &str) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Method {
            owner: JvmBytes(owner.as_bytes().to_vec()),
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
    }
}

/// A dynamic site's symbol carries no owner, so its name and descriptor alone identify it.
fn dynamic_site_symbol(name: &str, descriptor: &str) -> QueryTarget {
    method_symbol("", name, descriptor)
}

fn class_symbol(owner: &str) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Class {
            owner: JvmBytes(owner.as_bytes().to_vec()),
        },
    }
}

/// Runs one positive query, which must be a completed scan that explains nothing.
fn run_complete(snapshot: &ArtifactSnapshot, request: &QueryRequest) -> QueryReport {
    let report = run(snapshot, request);
    assert!(
        is_complete(&report),
        "a positive case must complete, got {:?} with diagnostics {:?}",
        report.execution,
        report.diagnostics
    );
    assert!(
        report.diagnostics.is_empty(),
        "a completed scan explains nothing, got {:?}",
        report.diagnostics
    );
    report
}

fn literal_string(value: &str) -> QueryTarget {
    QueryTarget::Literal {
        value: LiteralValue::String {
            value: JvmBytes(value.as_bytes().to_vec()),
        },
    }
}

/// One `via` hop: the node reached, and the edge position that reached it.
fn hop(index: u16, bootstrap_index: Option<u16>, argument_index: Option<u16>) -> BootstrapVia {
    BootstrapVia {
        constant_pool_index: index,
        bootstrap_index,
        argument_index,
    }
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

/// One reported item's coordinates: operation, described entry, BCI, opcode.
type EvidenceRow = (XrefOperation, Option<u16>, Option<u32>, Option<u8>);

/// `(operation, described entry, BCI, opcode)` of every reported item, in report order.
fn rows(report: &QueryReport) -> Vec<EvidenceRow> {
    report
        .items
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

/// `via` paths of every reported item, in report order.
fn vias(report: &QueryReport) -> Vec<Vec<BootstrapVia>> {
    report
        .items
        .iter()
        .map(|item| item.evidence.via.clone())
        .collect()
}

fn diagnostic_codes(report: &QueryReport) -> Vec<&str> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

fn is_complete(report: &QueryReport) -> bool {
    matches!(report.execution, ExecutionReport::Complete { .. })
}

fn is_cancelled(report: &QueryReport) -> bool {
    matches!(report.execution, ExecutionReport::Cancelled { .. })
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

fn code_location(item: &XrefItem) -> (&PhysicalMethodId, u32) {
    match &item.source.location {
        Location::Code { method, bci } => (method, *bci),
        other => panic!("expected a code location, got {other:?}"),
    }
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

    fn method_handle(&mut self, kind: u8, reference: u16) -> u16 {
        let mut entry = vec![15, kind];
        u16b(&mut entry, reference);
        self.push(entry)
    }

    fn method_type(&mut self, descriptor: u16) -> u16 {
        let mut entry = vec![16];
        u16b(&mut entry, descriptor);
        self.push(entry)
    }

    /// `CONSTANT_Dynamic` (17).
    fn dynamic(&mut self, bootstrap: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![17];
        u16b(&mut entry, bootstrap);
        u16b(&mut entry, name_and_type);
        self.push(entry)
    }

    /// `CONSTANT_InvokeDynamic` (18).
    fn invoke_dynamic(&mut self, bootstrap: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![18];
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

/// One member declaration.
struct MethodSpec {
    access: u16,
    name: u16,
    descriptor: u16,
    /// Attribute name index and content of the member's single attribute, when it has one.
    attribute: Option<(u16, Vec<u8>)>,
}

/// One class-level attribute declaration.
struct AttributeSpec {
    name: u16,
    content: Vec<u8>,
}

/// Class-file offsets of the attribute content a fixture writes.
///
/// Kept so a test can assert an item's span or an attribute charge against the bytes that
/// were really written instead of recomputing the frame sizes by hand.
#[derive(Default)]
struct Layout {
    /// Content start of each member attribute, in method declaration order.
    method_attributes: Vec<u64>,
    /// Content start of each class-level attribute, in declaration order.
    class_attributes: Vec<u64>,
}

/// Assembles a class file around a pool and the declarations the caller makes.
///
/// The version is Java 8 for every fixture: the reader facts never gate attributes or
/// constant-pool tags on a version, and these fixtures pin structure rather than a version
/// contract.
fn class_bytes(
    pool: &Pool,
    this_class: u16,
    super_class: u16,
    methods: &[MethodSpec],
    attributes: &[AttributeSpec],
) -> (Vec<u8>, Layout) {
    let mut layout = Layout::default();
    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0); // minor
    u16b(&mut bytes, 52); // major
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
    for method in methods {
        u16b(&mut bytes, method.access);
        u16b(&mut bytes, method.name);
        u16b(&mut bytes, method.descriptor);
        u16b(&mut bytes, u16::from(method.attribute.is_some()));
        if let Some((name, content)) = &method.attribute {
            u16b(&mut bytes, *name);
            u32b(
                &mut bytes,
                u32::try_from(content.len()).expect("attribute content fits u32"),
            );
            layout
                .method_attributes
                .push(u64::try_from(bytes.len()).expect("offset fits u64"));
            bytes.extend_from_slice(content);
        }
    }
    u16b(
        &mut bytes,
        u16::try_from(attributes.len()).expect("fixture attributes fit u16"),
    );
    for attribute in attributes {
        u16b(&mut bytes, attribute.name);
        u32b(
            &mut bytes,
            u32::try_from(attribute.content.len()).expect("attribute content fits u32"),
        );
        layout
            .class_attributes
            .push(u64::try_from(bytes.len()).expect("offset fits u64"));
        bytes.extend_from_slice(&attribute.content);
    }
    (bytes, layout)
}

/// One `Code` attribute body with no exception table and no nested attribute.
fn code_body(instructions: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    u16b(&mut body, 4); // max_stack
    u16b(&mut body, 2); // max_locals
    u32b(
        &mut body,
        u32::try_from(instructions.len()).expect("fixture code fits u32"),
    );
    body.extend_from_slice(instructions);
    u16b(&mut body, 0); // exception table
    u16b(&mut body, 0); // attributes
    body
}

/// One `BootstrapMethods` attribute body: `(handle, arguments)` per entry.
fn bootstrap_body(entries: &[(u16, Vec<u16>)]) -> Vec<u8> {
    let mut body = Vec::new();
    u16b(
        &mut body,
        u16::try_from(entries.len()).expect("fixture bootstrap entries fit u16"),
    );
    for (handle, arguments) in entries {
        u16b(&mut body, *handle);
        u16b(
            &mut body,
            u16::try_from(arguments.len()).expect("fixture arguments fit u16"),
        );
        for argument in arguments {
            u16b(&mut body, *argument);
        }
    }
    body
}

/// `invokedynamic` over one constant-pool entry.
fn indy(entry: u16) -> Vec<u8> {
    let mut bytes = vec![0xba];
    u16b(&mut bytes, entry);
    bytes.push(0);
    bytes.push(0);
    bytes
}

/// `ldc` over one constant-pool entry; every fixture pool fits the one-byte operand.
fn ldc(entry: u16) -> Vec<u8> {
    let mut bytes = vec![0x12];
    bytes.push(u8::try_from(entry).expect("fixture pool fits the one-byte ldc operand"));
    bytes
}

/// Attribute entry charge of a content: the shell plus its content, as the reader bills it.
fn shell(content: &[u8]) -> u64 {
    u64::try_from(content.len()).expect("content fits u64") + 6
}

// ---------------------------------------------------------------------------
// The structural sample fixture
// ---------------------------------------------------------------------------

const SAME_USE_METHOD: (&str, &str) = ("use", "()V");
const CUSTOM_DESCRIPTOR: &str = "(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;";
/// The sample's lambda-like bootstrap: five parameters, which is **not** the real
/// `LambdaMetafactory.metafactory` (six parameters, three of them static). This member is a
/// fictitious `p/Sample` method on purpose: naming a real JDK member with a signature it does
/// not have would be a false claim about that member, while this sample only exists to pin
/// how a table's declared argument positions and kinds are reported.
/// `the_structural_sample_is_not_a_linkable_bootstrap` checks that the label is earned.
const SAMPLE_BSM_DESCRIPTOR: &str = "(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/CallSite;";
const CONDY_DESCRIPTOR: &str = "Ljava/lang/Object;";

/// Constant-pool indexes and class-file facts the sample fixture's tests assert against.
struct Indexes {
    lambda_site: u16,
    custom_site: u16,
    nested_condy: u16,
    nested_condy2: u16,
    unused_condy: u16,
    lambda_bsm: u16,
    custom_bsm: u16,
    unused_bsm: u16,
    lookup_handle: u16,
    sam_name: u16,
    sam_mt: u16,
    inst_mt: u16,
    impl_handle: u16,
    impl_ref: u16,
    leaf_string: u16,
    /// A `String` only the unused table entry references.
    orphan_string: u16,
    integer: u16,
}

/// Hand-built class with one lambda-like site, one custom site, nested condy, sharing and two
/// bootstrap-table entries no use-site reaches.
struct Fixture {
    bytes: Vec<u8>,
    index: Indexes,
    code_shell: u64,
    bootstrap_shell: u64,
    /// Start offset and length of the instruction array of `use()V`.
    code_span: ByteSpan,
}

/// BCI of every use-site of the main fixture, in the order the scan must report them.
const LAMBDA_BCI: u32 = 0;
const CUSTOM_BCI: u32 = 5;
const NESTED_LDC_BCI: u32 = 10;
const DEEPER_LDC_BCI: u32 = 12;

fn fixture() -> Fixture {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/Boot");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let code_name = pool.utf8(b"Code");
    let bootstrap_name = pool.utf8(b"BootstrapMethods");
    let use_name = pool.utf8(b"use");
    let void_descriptor = pool.utf8(b"()V");
    let todo_name = pool.utf8(b"todo");

    // The sample's lambda-like bootstrap: `p/Sample.lambdaLike`, not a JDK member.
    let sample_owner = pool.utf8(b"p/Sample");
    let sample_class = pool.class(sample_owner);
    let sample_name = pool.utf8(b"lambdaLike");
    let sample_descriptor = pool.utf8(SAMPLE_BSM_DESCRIPTOR.as_bytes());
    let sample_nat = pool.name_and_type(sample_name, sample_descriptor);
    let sample_ref = pool.member(10, sample_class, sample_nat);
    let lambda_bsm = pool.method_handle(6, sample_ref);
    let lookup_owner = pool.utf8(b"p/Lookup");
    let lookup_class = pool.class(lookup_owner);
    let lookup_name = pool.utf8(b"lookup");
    let lookup_descriptor = pool.utf8(b"()Lp/Lookup;");
    let lookup_nat = pool.name_and_type(lookup_name, lookup_descriptor);
    let lookup_ref = pool.member(10, lookup_class, lookup_nat);
    let lookup_handle = pool.method_handle(6, lookup_ref);
    let sam_name_text = pool.utf8(b"apply");
    let sam_name = pool.string(sam_name_text);
    let sam_mt = pool.method_type(void_descriptor);
    let inst_mt = pool.method_type(void_descriptor);
    let impl_owner = pool.utf8(b"p/Impl");
    let impl_class = pool.class(impl_owner);
    let impl_name = pool.utf8(b"lambda$use$0");
    let impl_nat = pool.name_and_type(impl_name, void_descriptor);
    let impl_ref = pool.member(10, impl_class, impl_nat);
    let impl_handle = pool.method_handle(6, impl_ref);
    // A custom bootstrap that reappears as every condy's bootstrap in this fixture.
    let custom_owner = pool.utf8(b"p/Custom");
    let custom_class = pool.class(custom_owner);
    let custom_name = pool.utf8(b"bsm");
    let custom_descriptor = pool.utf8(CUSTOM_DESCRIPTOR.as_bytes());
    let custom_nat = pool.name_and_type(custom_name, custom_descriptor);
    let custom_ref = pool.member(10, custom_class, custom_nat);
    let custom_bsm = pool.method_handle(6, custom_ref);

    // The nested condy nodes: bootstrap 2 holds node 2, bootstrap 3 holds the leaf.
    let condy_name = pool.utf8(b"condy");
    let condy_descriptor = pool.utf8(CONDY_DESCRIPTOR.as_bytes());
    let condy_nat = pool.name_and_type(condy_name, condy_descriptor);
    let nested_condy = pool.dynamic(2, condy_nat);
    let nested_condy2 = pool.dynamic(3, condy_nat);
    let leaf_text = pool.utf8(b"leaf");
    let leaf_string = pool.string(leaf_text);
    let integer = pool.integer(7);

    // An entry and a dynamic site no use-site reaches. The fifth table entry references
    // `orphan_string`, which no used entry references either: the negative control for
    // "an entry only an unused table entry can reach".
    let unused_owner = pool.utf8(b"p/Unused");
    let unused_class = pool.class(unused_owner);
    let unused_name = pool.utf8(b"member");
    let unused_nat = pool.name_and_type(unused_name, void_descriptor);
    let unused_ref = pool.member(10, unused_class, unused_nat);
    let unused_bsm = pool.method_handle(6, unused_ref);
    let unused_condy_name = pool.utf8(b"unused_condy");
    let unused_condy_nat = pool.name_and_type(unused_condy_name, condy_descriptor);
    let unused_condy = pool.dynamic(4, unused_condy_nat);
    let orphan_text = pool.utf8(b"orphan");
    let orphan_string = pool.string(orphan_text);

    // The two `invokedynamic` sites: a lambda site and a custom site.
    let fn_name = pool.utf8(b"p/Fn");
    let _fn_class = pool.class(fn_name);
    let fn_descriptor = pool.utf8(b"()Lp/Fn;");
    let lambda_nat = pool.name_and_type(sam_name_text, fn_descriptor);
    let lambda_site = pool.invoke_dynamic(0, lambda_nat);
    let custom_site_name = pool.utf8(b"custom");
    let custom_site_nat = pool.name_and_type(custom_site_name, condy_descriptor);
    let custom_site = pool.invoke_dynamic(1, custom_site_nat);

    let mut code = Vec::new();
    code.extend_from_slice(&indy(lambda_site));
    code.extend_from_slice(&indy(custom_site));
    code.extend_from_slice(&ldc(nested_condy));
    code.extend_from_slice(&ldc(nested_condy2));
    code.push(0xb1); // return
    let body = code_body(&code);

    let bootstrap = bootstrap_body(&[
        (
            lambda_bsm,
            vec![lookup_handle, sam_name, sam_mt, impl_handle, inst_mt],
        ),
        // The same nested node twice: one shared subgraph, two edges.
        (custom_bsm, vec![nested_condy, nested_condy]),
        (custom_bsm, vec![nested_condy2, integer]),
        (custom_bsm, vec![leaf_string]),
        (unused_bsm, vec![orphan_string]),
    ]);

    let code_method = MethodSpec {
        access: 0x0002,
        name: use_name,
        descriptor: void_descriptor,
        attribute: Some((code_name, body.clone())),
    };
    // A member without a body proves an abstract declaration is skipped.
    let abstract_method = MethodSpec {
        access: 0x0401,
        name: todo_name,
        descriptor: void_descriptor,
        attribute: None,
    };
    let attributes = vec![AttributeSpec {
        name: bootstrap_name,
        content: bootstrap.clone(),
    }];
    let (bytes, layout) = class_bytes(
        &pool,
        this_class,
        object_class,
        &[code_method, abstract_method],
        &attributes,
    );

    // The instruction array follows `max_stack`, `max_locals` and `code_length` inside the
    // `Code` attribute content, and the fixtures below slice the entries out of the class
    // bytes through these offsets.
    let code_content_offset = layout.method_attributes[0];
    let code_span = ByteSpan::new(
        code_content_offset + 8,
        u64::try_from(code.len()).expect("fixture code fits u64"),
    );
    let bootstrap_content_offset = layout.class_attributes[0];
    assert_eq!(
        &bytes[bootstrap_content_offset as usize..bootstrap_content_offset as usize + 2],
        &5_u16.to_be_bytes(),
        "the bootstrap table has five entries"
    );
    // `BootstrapMethods` entry 4 — the one no dynamic site names — really holds the unused
    // handle at the offset its declared argument counts put it: 2 bytes of entry count, then
    // 14 + 8 + 8 + 6 bytes of the four entries before it.
    let unused_entry = bootstrap_content_offset as usize + 2 + 14 + 8 + 8 + 6;
    assert_eq!(
        &bytes[unused_entry..unused_entry + 2],
        &unused_bsm.to_be_bytes(),
        "the fixture's fifth bootstrap entry is the unused handle"
    );
    // ... and its single declared static argument really is `orphan_string`, so the "only an
    // unused entry reaches it" control is wired and not vacuous.
    assert_eq!(
        &bytes[unused_entry + 4..unused_entry + 6],
        &orphan_string.to_be_bytes(),
        "the unused entry's argument is the orphan string"
    );
    assert_eq!(
        bytes[code_span.start as usize], 0xba,
        "the instruction array starts with the lambda-like site"
    );

    Fixture {
        bytes,
        index: Indexes {
            lambda_site,
            custom_site,
            nested_condy,
            nested_condy2,
            unused_condy,
            lambda_bsm,
            custom_bsm,
            unused_bsm,
            lookup_handle,
            sam_name,
            sam_mt,
            inst_mt,
            impl_handle,
            impl_ref,
            leaf_string,
            orphan_string,
            integer,
        },
        code_shell: shell(&body),
        bootstrap_shell: shell(&bootstrap),
        code_span,
    }
}

/// Attribute bytes one query that reads the class and both attribute contents must charge.
fn expected_attribute_bytes(fixture: &Fixture) -> u64 {
    // `class_facts` bills every shell once, the code read bills `Code` again, and the
    // bootstrap read bills `BootstrapMethods` again.
    fixture.bootstrap_shell + fixture.code_shell + fixture.code_shell + fixture.bootstrap_shell
}

/// Attribute bytes one query that reads the class and the `Code` content only must charge:
/// the `BootstrapMethods` content is never read without the category.
fn attribute_bytes_without_bootstrap(fixture: &Fixture) -> u64 {
    fixture.bootstrap_shell + fixture.code_shell + fixture.code_shell
}

// ---------------------------------------------------------------------------
// Shared assertion helpers
// ---------------------------------------------------------------------------

/// Every operation this consumer can produce.
const BOOTSTRAP_OPERATIONS: [XrefOperation; 2] = [
    XrefOperation::BootstrapMethod,
    XrefOperation::BootstrapArgument,
];

/// The class-file bytes of the entry one item describes.
fn described_entry<'a>(bytes: &'a [u8], item: &XrefItem) -> &'a [u8] {
    let span = item
        .evidence
        .span
        .clone()
        .expect("every bootstrap item describes one constant-pool entry with a span");
    &bytes[span.start as usize..(span.start + span.length) as usize]
}

/// Invariants every bootstrap item of a report must satisfy, checked against the bytes.
///
/// They are the shape the module claims: a bootstrap edge with a real path, a described
/// constant-pool entry, and no resolution or execution claim.
fn check_item_shape(bytes: &[u8], code_span: &ByteSpan, report: &QueryReport) {
    for item in &report.items {
        assert_eq!(item.consumer, Some(ConsumerKind::Bootstrap), "{item:?}");
        assert_eq!(item.derivation, XrefDerivation::BootstrapEdge, "{item:?}");
        assert_eq!(item.certainty, XrefCertainty::Exact, "{item:?}");
        assert_eq!(item.resolution, QueryResolution::NotRequested, "{item:?}");
        assert!(
            BOOTSTRAP_OPERATIONS.contains(&item.operation),
            "no consumer operation other than a bootstrap edge may be produced: {item:?}"
        );
        assert!(
            matches!(item.source.location, Location::Code { .. }),
            "a graph node is located at the use-site that reached it: {item:?}"
        );
        assert_eq!(
            item.evidence.attribute,
            Some(ArchiveNameBytes(b"BootstrapMethods".to_vec())),
            "{item:?}"
        );
        let via = &item.evidence.via;
        assert!(!via.is_empty(), "every path starts at a use-site: {item:?}");
        assert_eq!(
            (via[0].bootstrap_index, via[0].argument_index),
            (None, None),
            "the first hop is the use-site itself: {item:?}"
        );
        assert_eq!(
            Some(via[via.len() - 1].constant_pool_index),
            item.evidence.constant_pool_index,
            "the last hop is the described entry: {item:?}"
        );
        assert!(
            item.evidence.bci.is_some() && item.evidence.opcode.is_some(),
            "the use-site instruction is kept: {item:?}"
        );
        // The use-site coordinates are the class bytes, not a restatement: the opcode is
        // really at that BCI inside the method's instruction array.
        let bci = u64::from(item.evidence.bci.expect("a use-site BCI"));
        assert_eq!(
            Some(bytes[(code_span.start + bci) as usize]),
            item.evidence.opcode,
            "{item:?}"
        );
        let entry = described_entry(bytes, item);
        if matches!(item.target, XrefTarget::Literal { .. }) {
            assert_ne!(
                entry[0], 17,
                "a constant-dynamic site is never reported as a value: {item:?}"
            );
            assert_ne!(
                entry[0], 18,
                "an invokedynamic site is never reported as a value: {item:?}"
            );
        }
    }
}

/// The sample fixture's own invariants on top of [`check_item_shape`]: the entries no
/// use-site reaches may never appear.
fn check_item_invariants(fixture: &Fixture, report: &QueryReport) {
    check_item_shape(&fixture.bytes, &fixture.code_span, report);
    for item in &report.items {
        assert_ne!(
            item.evidence.constant_pool_index,
            Some(fixture.index.unused_condy),
            "a dynamic entry no use-site consumes is never a graph node: {item:?}"
        );
        assert!(
            !item
                .evidence
                .via
                .iter()
                .any(|element| element.constant_pool_index == fixture.index.unused_bsm),
            "a bootstrap handle no dynamic site names is never on a path: {item:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// A04: the real LambdaMetafactory shape
// ---------------------------------------------------------------------------

/// The real `LambdaMetafactory.metafactory` bootstrap method: six parameters, of which the
/// JVM supplies the first three (`MethodHandles.Lookup`, the name, the invoked `MethodType`)
/// and the table supplies the last three.
const METAFACTORY_DESCRIPTOR: &str = "(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/CallSite;";
const METAFACTORY_OWNER: &str = "java/lang/invoke/LambdaMetafactory";
const METAFACTORY_NAME: &str = "metafactory";

/// Arguments the JVM itself supplies when it invokes a bootstrap method (JVMS §5.4.3.6).
const JVM_SUPPLIED_ARGUMENTS: usize = 3;

/// Static arguments the real fixture's single bootstrap entry declares.
const REAL_STATIC_ARGUMENTS: usize = 3;

/// `BootstrapMethods` index the real fixture's site declares.
const REAL_SITE_BOOTSTRAP_INDEX: u16 = 0;

/// SAM method name of the call site. `metafactory` receives it as its second JVM-supplied
/// argument, so it lives in the site's `NameAndType` and never in the bootstrap table.
const SAM_NAME: &str = "apply";
/// Invoked (factory) type of the call site: the third JVM-supplied argument, and likewise the
/// site's `NameAndType` descriptor.
const INVOKED_DESCRIPTOR: &str = "()Lp/Fn;";
const IMPL_OWNER: &str = "p/Impl";
const IMPL_NAME: &str = "lambda$use$0";
/// Descriptor of the implementation method. The fixture uses one `Utf8` for it and for the
/// instantiated method type, which is what a non-capturing static implementation implies.
const IMPL_DESCRIPTOR: &str = "()Ljava/lang/Integer;";
/// Erased SAM method descriptor: different from the instantiated type, the way a generic
/// interface method differs from its lambda body.
const SAM_DESCRIPTOR: &str = "()Ljava/lang/Object;";
const LAMBDA_CONST_DESCRIPTOR: &str = "()V";

/// One real `invokedynamic` lambda site: `LambdaMetafactory.metafactory` with exactly the
/// three static arguments JVMS and `java.lang.invoke` describe.
struct RealLambda {
    bytes: Vec<u8>,
    /// `CONSTANT_InvokeDynamic` of the site.
    site: u16,
    /// `CONSTANT_MethodHandle` referencing `LambdaMetafactory.metafactory`.
    metafactory_handle: u16,
    metafactory_ref: u16,
    /// Static argument 0: the SAM method type.
    sam_type: u16,
    /// Static argument 1: the implementation `MethodHandle`.
    impl_handle: u16,
    impl_ref: u16,
    /// Static argument 2: the instantiated method type.
    instantiated_type: u16,
    code_span: ByteSpan,
}

/// The single BCI of the real fixture's use-site.
const REAL_BCI: u32 = 0;

fn real_lambda_fixture() -> RealLambda {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/RealLambda");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let code_name = pool.utf8(b"Code");
    let bootstrap_name = pool.utf8(b"BootstrapMethods");
    let use_name = pool.utf8(b"use");
    let void_descriptor = pool.utf8(LAMBDA_CONST_DESCRIPTOR.as_bytes());

    // The bootstrap method itself: a real JDK member with its real 6-parameter descriptor.
    let metafactory_owner = pool.utf8(METAFACTORY_OWNER.as_bytes());
    let metafactory_class = pool.class(metafactory_owner);
    let metafactory_name = pool.utf8(b"metafactory");
    let metafactory_descriptor = pool.utf8(METAFACTORY_DESCRIPTOR.as_bytes());
    let metafactory_nat = pool.name_and_type(metafactory_name, metafactory_descriptor);
    let metafactory_ref = pool.member(10, metafactory_class, metafactory_nat);
    let metafactory_handle = pool.method_handle(6, metafactory_ref);

    // Static argument 0 and 2: two method types. The instantiated one shares its `Utf8` with
    // the implementation method's descriptor.
    let sam_descriptor = pool.utf8(SAM_DESCRIPTOR.as_bytes());
    let sam_type = pool.method_type(sam_descriptor);
    let impl_descriptor = pool.utf8(IMPL_DESCRIPTOR.as_bytes());
    let instantiated_type = pool.method_type(impl_descriptor);

    // Static argument 1: the implementation handle, a static method reference.
    let impl_owner = pool.utf8(IMPL_OWNER.as_bytes());
    let impl_class = pool.class(impl_owner);
    let impl_method_name = pool.utf8(IMPL_NAME.as_bytes());
    let impl_nat = pool.name_and_type(impl_method_name, impl_descriptor);
    let impl_ref = pool.member(10, impl_class, impl_nat);
    let impl_handle = pool.method_handle(6, impl_ref);

    // The site: SAM name and invoked type live here, per JVMS.
    let sam_name_text = pool.utf8(SAM_NAME.as_bytes());
    let invoked_descriptor = pool.utf8(INVOKED_DESCRIPTOR.as_bytes());
    let site_nat = pool.name_and_type(sam_name_text, invoked_descriptor);
    let site = pool.invoke_dynamic(REAL_SITE_BOOTSTRAP_INDEX, site_nat);

    let mut code = indy(site);
    code.push(0xb1); // return
    let method = MethodSpec {
        access: 0x0002,
        name: use_name,
        descriptor: void_descriptor,
        attribute: Some((code_name, code_body(&code))),
    };
    let attributes = vec![AttributeSpec {
        name: bootstrap_name,
        content: bootstrap_body(&[(
            metafactory_handle,
            vec![sam_type, impl_handle, instantiated_type],
        )]),
    }];
    let (bytes, layout) = class_bytes(&pool, this_class, object_class, &[method], &attributes);
    let code_span = ByteSpan::new(
        layout.method_attributes[0] + 8,
        u64::try_from(code.len()).expect("fixture code fits u64"),
    );

    // The linkability this fixture claims is checked on the attribute bytes, not assumed from
    // the arguments passed above: entry count, bootstrap handle, declared static argument count
    // and the three argument indexes in the order the bootstrap contract gives them.
    let entry = layout.class_attributes[0] as usize;
    assert_eq!(&bytes[entry..entry + 2], &1_u16.to_be_bytes());
    assert_eq!(
        &bytes[entry + 2..entry + 4],
        &metafactory_handle.to_be_bytes()
    );
    assert_eq!(
        &bytes[entry + 4..entry + 6],
        &u16::try_from(REAL_STATIC_ARGUMENTS)
            .expect("static argument count fits u16")
            .to_be_bytes(),
        "the entry declares three static arguments"
    );
    assert_eq!(
        [
            u16::from_be_bytes([bytes[entry + 6], bytes[entry + 7]]),
            u16::from_be_bytes([bytes[entry + 8], bytes[entry + 9]]),
            u16::from_be_bytes([bytes[entry + 10], bytes[entry + 11]]),
        ],
        [sam_type, impl_handle, instantiated_type],
        "the static arguments are the SAM method type, the implementation handle and the \
         instantiated method type, in that order"
    );

    RealLambda {
        bytes,
        site,
        metafactory_handle,
        metafactory_ref,
        sam_type,
        impl_handle,
        impl_ref,
        instantiated_type,
        code_span,
    }
}

/// Parameter count of a method descriptor, per JVMS §4.3.3.
///
/// Used to state the linkability of a fixture from the descriptor it declares rather than
/// from a comment: a bootstrap method may be invoked only when the arguments it will receive
/// — the three the JVM supplies plus the entry's static arguments — match its parameters.
fn descriptor_parameters(descriptor: &str) -> usize {
    let bytes = descriptor.as_bytes();
    let open = bytes
        .iter()
        .position(|byte| *byte == b'(')
        .expect("a method descriptor has a parameter list");
    let mut index = open + 1;
    let mut count = 0;
    while index < bytes.len() && bytes[index] != b')' {
        while bytes[index] == b'[' {
            index += 1;
        }
        match bytes[index] {
            b'L' => {
                index += 1;
                while bytes[index] != b';' {
                    index += 1;
                }
                index += 1;
            }
            b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z' => index += 1,
            other => panic!("descriptor holds an unknown type: {}", other as char),
        }
        count += 1;
    }
    count
}

#[test]
fn a_real_metafactory_site_is_linkable_and_traces_the_implementation_handle() {
    let fixture = real_lambda_fixture();

    // Linkability, stated from the declared shapes: `metafactory` declares six parameters, the
    // entry declares three static arguments, and the JVM adds three leading ones — so the
    // invocation this fixture describes is one the JVM could really perform, unlike the
    // structural sample (see `the_structural_sample_is_not_a_linkable_bootstrap`).
    assert_eq!(descriptor_parameters(METAFACTORY_DESCRIPTOR), 6);
    assert_eq!(
        descriptor_parameters(METAFACTORY_DESCRIPTOR),
        JVM_SUPPLIED_ARGUMENTS + REAL_STATIC_ARGUMENTS,
        "three static arguments plus the three the JVM supplies"
    );

    let snapshot = open(fixture.bytes.clone());
    let target = method_symbol(IMPL_OWNER, IMPL_NAME, IMPL_DESCRIPTOR);
    let (report, _) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target.clone(),
            &BOOTSTRAP,
        ),
        limits(),
    );

    // A04: the implementation handle is traceable through the member it names, at the static
    // argument position the real bootstrap declares for it.
    assert_eq!(report.items.len(), 1, "{:?}", rows(&report));
    let item = &report.items[0];
    assert_eq!(item.operation, XrefOperation::BootstrapArgument);
    assert_eq!(item.target, result_target(&target));
    assert_eq!(item.evidence.constant_pool_index, Some(fixture.impl_handle));
    assert_eq!(item.evidence.bci, Some(REAL_BCI));
    assert_eq!(item.evidence.opcode, Some(0xba));
    assert_eq!(
        item.evidence.via,
        vec![
            hop(fixture.site, None, None),
            hop(
                fixture.impl_handle,
                Some(REAL_SITE_BOOTSTRAP_INDEX),
                Some(1)
            ),
        ],
        "the implementation handle is static argument 1, the position metafactory declares"
    );
    // The described entry is the handle, so `reference_kind` and the member it points at stay
    // readable in the class bytes instead of being restated as a call.
    let entry = described_entry(&fixture.bytes, item);
    assert_eq!(entry[0], 15, "CONSTANT_MethodHandle");
    assert_eq!(entry[1], 6, "REF_invokeStatic");
    assert_eq!(u16::from_be_bytes([entry[2], entry[3]]), fixture.impl_ref);

    // Creating the lambda form is not calling the implementation: the report holds bootstrap
    // edges only, and no invocation, load or allocation operation.
    check_item_shape(&fixture.bytes, &fixture.code_span, &report);
    assert_eq!(item.consumer, Some(ConsumerKind::Bootstrap));
    assert_eq!(item.derivation, XrefDerivation::BootstrapEdge);
    assert_eq!(item.resolution, QueryResolution::NotRequested);
    assert!(is_complete(&report));
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
}

#[test]
fn a_real_metafactory_site_keeps_every_static_argument_in_its_own_position() {
    let fixture = real_lambda_fixture();
    let snapshot = open(fixture.bytes.clone());

    // Static argument 0 is the SAM method type and static argument 2 the instantiated method
    // type: the descriptors differ, so the two positions are told apart by the bytes they
    // hold, not by their order alone.
    let sam = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            literal_string(SAM_DESCRIPTOR),
            &BOOTSTRAP,
        ),
    );
    assert_eq!(
        rows(&sam),
        vec![(
            XrefOperation::BootstrapArgument,
            Some(fixture.sam_type),
            Some(REAL_BCI),
            Some(0xba)
        )]
    );
    assert_eq!(
        vias(&sam),
        vec![vec![
            hop(fixture.site, None, None),
            hop(fixture.sam_type, Some(REAL_SITE_BOOTSTRAP_INDEX), Some(0)),
        ]]
    );
    for item in &sam.items {
        assert_eq!(
            described_entry(&fixture.bytes, item)[0],
            16,
            "a method type argument describes a CONSTANT_MethodType, never a CONSTANT_String"
        );
    }

    let instantiated = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            literal_string(IMPL_DESCRIPTOR),
            &BOOTSTRAP,
        ),
    );
    assert_eq!(
        rows(&instantiated),
        vec![(
            XrefOperation::BootstrapArgument,
            Some(fixture.instantiated_type),
            Some(REAL_BCI),
            Some(0xba)
        )]
    );
    assert_eq!(
        vias(&instantiated),
        vec![vec![
            hop(fixture.site, None, None),
            hop(
                fixture.instantiated_type,
                Some(REAL_SITE_BOOTSTRAP_INDEX),
                Some(2)
            ),
        ]],
        "the instantiated method type is static argument 2"
    );

    // The bootstrap method handle itself, reached by the `None` argument position.
    let metafactory = method_symbol(METAFACTORY_OWNER, METAFACTORY_NAME, METAFACTORY_DESCRIPTOR);
    let handle = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            metafactory,
            &BOOTSTRAP,
        ),
    );
    assert_eq!(
        rows(&handle),
        vec![(
            XrefOperation::BootstrapMethod,
            Some(fixture.metafactory_handle),
            Some(REAL_BCI),
            Some(0xba)
        )]
    );
    assert_eq!(
        vias(&handle),
        vec![vec![
            hop(fixture.site, None, None),
            hop(
                fixture.metafactory_handle,
                Some(REAL_SITE_BOOTSTRAP_INDEX),
                None
            ),
        ]]
    );
    let entry = described_entry(&fixture.bytes, &handle.items[0]);
    assert_eq!(
        u16::from_be_bytes([entry[2], entry[3]]),
        fixture.metafactory_ref
    );
    check_item_shape(&fixture.bytes, &fixture.code_span, &handle);

    // The SAM name and the invoked type are the site's own `NameAndType`, so the code
    // consumer reports them and the bootstrap consumer reports nothing about them: the two
    // products divide the site without either inventing the other's fact.
    let site_symbol = dynamic_site_symbol(SAM_NAME, INVOKED_DESCRIPTOR);
    let site_by_code = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            site_symbol.clone(),
            &[ConsumerKind::Invocation],
        ),
    );
    assert_eq!(
        rows(&site_by_code),
        vec![(
            XrefOperation::InvokeDynamic,
            Some(fixture.site),
            Some(REAL_BCI),
            Some(0xba)
        )]
    );
    assert_eq!(
        site_by_code.items[0].consumer,
        Some(ConsumerKind::Invocation)
    );
    assert_eq!(
        site_by_code.items[0].derivation,
        XrefDerivation::StructuralConsumer
    );
    let site_by_bootstrap = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            site_symbol,
            &BOOTSTRAP,
        ),
    );
    assert!(
        site_by_bootstrap.items.is_empty(),
        "the bootstrap consumer answers no fact about the site's own name and type: {:?}",
        rows(&site_by_bootstrap)
    );
}

#[test]
fn the_structural_sample_is_not_a_linkable_bootstrap() {
    // The sample's members are deliberately not real shapes. This test earns the label the
    // sample's documentation uses, so a later edit cannot quietly turn the "structural
    // sample" wording into a claim about a linkable bootstrap.
    assert_eq!(descriptor_parameters(SAMPLE_BSM_DESCRIPTOR), 5);
    assert_ne!(
        descriptor_parameters(SAMPLE_BSM_DESCRIPTOR),
        JVM_SUPPLIED_ARGUMENTS + 5,
        "five static arguments cannot be passed to a five-parameter bootstrap method"
    );
    assert_eq!(
        descriptor_parameters(CUSTOM_DESCRIPTOR),
        JVM_SUPPLIED_ARGUMENTS + 1,
        "the custom descriptor matches the single static argument the condy fixtures declare, \
         so those traversals describe a bootstrap the JVM could really invoke"
    );
}

// ---------------------------------------------------------------------------
// A04: the structural sample is traced for positions and paths, and nothing more is claimed
// ---------------------------------------------------------------------------

#[test]
fn the_sample_site_traces_its_declared_implementation_argument() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let target = method_symbol("p/Impl", "lambda$use$0", "()V");
    let (report, _) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target.clone(),
            &BOOTSTRAP,
        ),
        limits(),
    );

    assert_eq!(
        report.items.len(),
        1,
        "only static argument 3 of the sample's bootstrap names this member: {:?}",
        rows(&report)
    );
    let item = &report.items[0];
    assert_eq!(item.operation, XrefOperation::BootstrapArgument);
    assert_eq!(item.target, result_target(&target));
    assert_eq!(
        item.evidence.constant_pool_index,
        Some(fixture.index.impl_handle)
    );
    assert_eq!(item.evidence.bci, Some(LAMBDA_BCI));
    assert_eq!(item.evidence.opcode, Some(0xba));
    assert_eq!(
        item.evidence.via,
        vec![
            hop(fixture.index.lambda_site, None, None),
            hop(fixture.index.impl_handle, Some(0), Some(3)),
        ],
        "the path is the use-site, then static argument 3 of bootstrap 0"
    );
    // The described entry is the `MethodHandle` itself, so its reference kind stays
    // readable in the class bytes instead of being restated as a call.
    let entry = described_entry(&fixture.bytes, item);
    assert_eq!(entry[0], 15, "CONSTANT_MethodHandle");
    assert_eq!(entry[1], 6, "REF_invokeStatic");
    assert_eq!(
        u16::from_be_bytes([entry[2], entry[3]]),
        fixture.index.impl_ref
    );

    let (method, bci) = code_location(item);
    let (name, descriptor) = SAME_USE_METHOD;
    assert_eq!(
        (
            method.name.0.as_slice(),
            method.descriptor.0.as_slice(),
            bci
        ),
        (name.as_bytes(), descriptor.as_bytes(), LAMBDA_BCI)
    );
    check_item_invariants(&fixture, &report);
    assert!(is_complete(&report));
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
}

#[test]
fn every_declared_static_argument_of_the_sample_keeps_its_position_and_value() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());

    // The bootstrap method handle itself.
    let sample = method_symbol("p/Sample", "lambdaLike", SAMPLE_BSM_DESCRIPTOR);
    let report = run(
        &snapshot,
        &request(&snapshot, QueryRelation::MentionsSymbol, sample, &BOOTSTRAP),
    );
    assert_eq!(report.items.len(), 1, "{:?}", rows(&report));
    assert_eq!(report.items[0].operation, XrefOperation::BootstrapMethod);
    assert_eq!(
        report.items[0].evidence.constant_pool_index,
        Some(fixture.index.lambda_bsm)
    );
    assert_eq!(
        report.items[0].evidence.via,
        vec![
            hop(fixture.index.lambda_site, None, None),
            hop(fixture.index.lambda_bsm, Some(0), None),
        ],
        "the bootstrap method handle is the `None` argument position of its bootstrap"
    );

    // Every declared static argument keeps its own slot under the same `via` prefix, whatever
    // kind of node it is: the sample's mixed list is what makes those positions observable.
    let name_report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            literal_string("apply"),
            &BOOTSTRAP,
        ),
    );
    assert_eq!(
        rows(&name_report),
        vec![(
            XrefOperation::BootstrapArgument,
            Some(fixture.index.sam_name),
            Some(LAMBDA_BCI),
            Some(0xba)
        )]
    );
    assert_eq!(
        vias(&name_report),
        vec![vec![
            hop(fixture.index.lambda_site, None, None),
            hop(fixture.index.sam_name, Some(0), Some(1)),
        ]]
    );

    let descriptor_report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            literal_string("()V"),
            &BOOTSTRAP,
        ),
    );
    assert_eq!(
        rows(&descriptor_report),
        vec![
            (
                XrefOperation::BootstrapArgument,
                Some(fixture.index.sam_mt),
                Some(LAMBDA_BCI),
                Some(0xba)
            ),
            (
                XrefOperation::BootstrapArgument,
                Some(fixture.index.inst_mt),
                Some(LAMBDA_BCI),
                Some(0xba)
            ),
        ],
        "a method type argument is reported as its raw descriptor, entry by entry"
    );
    for item in &descriptor_report.items {
        assert_eq!(
            described_entry(&fixture.bytes, item)[0],
            16,
            "the described entry is a CONSTANT_MethodType, never a CONSTANT_String"
        );
    }
    check_item_invariants(&fixture, &descriptor_report);
    // A `MethodHandle` argument is reported as the member it names, whichever position it
    // holds: static argument 0 of the sample's list is a handle.
    let lookup = method_symbol("p/Lookup", "lookup", "()Lp/Lookup;");
    let lookup_report = run(
        &snapshot,
        &request(&snapshot, QueryRelation::MentionsSymbol, lookup, &BOOTSTRAP),
    );
    assert_eq!(
        rows(&lookup_report),
        vec![(
            XrefOperation::BootstrapArgument,
            Some(fixture.index.lookup_handle),
            Some(LAMBDA_BCI),
            Some(0xba)
        )]
    );
    assert_eq!(
        vias(&lookup_report),
        vec![vec![
            hop(fixture.index.lambda_site, None, None),
            hop(fixture.index.lookup_handle, Some(0), Some(0)),
        ]]
    );
    check_item_invariants(&fixture, &lookup_report);

    // A primitive argument keeps its JVM value and its own position: argument 1 of the
    // level-1 node's bootstrap.
    let integer_report = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            QueryTarget::Literal {
                value: LiteralValue::Integer { value: 7 },
            },
            &BOOTSTRAP,
        ),
    );
    assert_eq!(
        rows(&integer_report),
        vec![
            (
                XrefOperation::BootstrapArgument,
                Some(fixture.index.integer),
                Some(CUSTOM_BCI),
                Some(0xba)
            ),
            (
                XrefOperation::BootstrapArgument,
                Some(fixture.index.integer),
                Some(NESTED_LDC_BCI),
                Some(0x12)
            ),
        ],
        "the level-1 node uses the same argument"
    );
    assert_eq!(
        vias(&integer_report),
        vec![
            vec![
                hop(fixture.index.custom_site, None, None),
                hop(fixture.index.nested_condy, Some(1), Some(0)),
                hop(fixture.index.integer, Some(2), Some(1)),
            ],
            vec![
                hop(fixture.index.nested_condy, None, None),
                hop(fixture.index.integer, Some(2), Some(1)),
            ],
        ],
        "a primitive argument belongs to the node's own bootstrap, not to the site's"
    );
    check_item_invariants(&fixture, &integer_report);
}

#[test]
fn an_implementation_handle_is_never_reported_as_a_call() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());

    // The scan reports the member the handle names, and invents no sibling: `javac` often
    // synthesizes `lambda$...` names, but their existence is not a structural fact.
    let invented = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Impl", "lambda$use$1", "()V"),
            &BOOTSTRAP,
        ),
    );
    assert!(invented.items.is_empty(), "{:?}", rows(&invented));

    // Creating a lambda form is not calling the implementation, so an invocation-only
    // request over the same member finds nothing.
    let invocations_only = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Impl", "lambda$use$0", "()V"),
            &[ConsumerKind::Invocation],
        ),
    );
    assert!(
        invocations_only.items.is_empty(),
        "{:?}",
        rows(&invocations_only)
    );

    // The implementation handle is an argument, and the whole report stays inside the
    // bootstrap edge vocabulary.
    let bootstrap_only = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Impl", "lambda$use$0", "()V"),
            &BOOTSTRAP,
        ),
    );
    assert_eq!(
        rows(&bootstrap_only),
        vec![(
            XrefOperation::BootstrapArgument,
            Some(fixture.index.impl_handle),
            Some(LAMBDA_BCI),
            Some(0xba)
        )]
    );
    assert!(
        !bootstrap_only.items.iter().any(|item| matches!(
            item.operation,
            XrefOperation::InvokeVirtual
                | XrefOperation::InvokeSpecial
                | XrefOperation::InvokeStatic
                | XrefOperation::InvokeInterface
                | XrefOperation::InvokeDynamic
                | XrefOperation::Ldc
                | XrefOperation::New
        )),
        "a created lambda form is not an invocation: {:?}",
        rows(&bootstrap_only)
    );
    check_item_invariants(&fixture, &bootstrap_only);
}

#[test]
fn a_custom_bootstrap_claims_no_final_target() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let handle = method_symbol("p/Custom", "bsm", CUSTOM_DESCRIPTOR);

    // The custom bootstrap's own member is what a "resolved runtime target" claim would have
    // to be about, so the negative case is stated on it and not on an unrelated symbol: every
    // fact is a bootstrap method edge, none of them resolves anything, and no member of the
    // bootstrap or its arguments is reported as an invocation.
    let (report, _) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            handle.clone(),
            &BOOTSTRAP,
        ),
        limits(),
    );
    assert!(!report.items.is_empty(), "the fixture must reach the node");
    for item in &report.items {
        assert_eq!(
            item.operation,
            XrefOperation::BootstrapMethod,
            "only the bootstrap method edge names this member: {item:?}"
        );
        assert_eq!(item.resolution, QueryResolution::NotRequested, "{item:?}");
        assert_eq!(item.derivation, XrefDerivation::BootstrapEdge, "{item:?}");
        assert_eq!(item.consumer, Some(ConsumerKind::Bootstrap), "{item:?}");
    }
    // The four edges the sample's custom site and its condy nodes open to this bootstrap:
    // node 0 (the site), node 1, node 2 and the `ldc` of node 2.
    assert_eq!(report.items.len(), 6, "{:?}", rows(&report));
    check_item_invariants(&fixture, &report);

    // The same member with both consumers asked for: still no invocation, because no
    // instruction in this class consumes it.
    let with_code_consumer = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            handle,
            &[ConsumerKind::Invocation, ConsumerKind::Bootstrap],
        ),
    );
    assert_eq!(
        with_code_consumer.items, report.items,
        "the code consumer adds nothing for a bootstrap that no instruction calls"
    );
}

// ---------------------------------------------------------------------------
// A05: nested condy, sharing and per-use-site paths
// ---------------------------------------------------------------------------

#[test]
fn nested_condy_reports_both_levels_and_one_fact_per_shared_subgraph() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let target = dynamic_site_symbol("condy", CONDY_DESCRIPTOR);
    let (report, _) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target.clone(),
            &BOOTSTRAP,
        ),
        limits(),
    );

    // Both nesting levels are reachable: the level-1 node is the argument of bootstrap 1
    // of the custom site, and the level-2 node is the argument of the level-1 node's own
    // bootstrap. The repeated argument is a second edge, not a second subgraph.
    assert_eq!(
        rows(&report),
        vec![
            (
                XrefOperation::BootstrapArgument,
                Some(fixture.index.nested_condy),
                Some(CUSTOM_BCI),
                Some(0xba)
            ),
            (
                XrefOperation::BootstrapArgument,
                Some(fixture.index.nested_condy2),
                Some(CUSTOM_BCI),
                Some(0xba)
            ),
            (
                XrefOperation::BootstrapArgument,
                Some(fixture.index.nested_condy),
                Some(CUSTOM_BCI),
                Some(0xba)
            ),
            (
                XrefOperation::BootstrapArgument,
                Some(fixture.index.nested_condy2),
                Some(NESTED_LDC_BCI),
                Some(0x12)
            ),
        ],
        "the use-site at BCI 12 consumes the level-2 node, whose only argument is a string"
    );
    let custom_root = hop(fixture.index.custom_site, None, None);
    let nested_first = hop(fixture.index.nested_condy, Some(1), Some(0));
    let nested_second = hop(fixture.index.nested_condy, Some(1), Some(1));
    assert_eq!(
        vias(&report),
        vec![
            vec![custom_root, nested_first],
            vec![
                custom_root,
                nested_first,
                hop(fixture.index.nested_condy2, Some(2), Some(0))
            ],
            vec![custom_root, nested_second],
            vec![
                hop(fixture.index.nested_condy, None, None),
                hop(fixture.index.nested_condy2, Some(2), Some(0)),
            ],
        ]
    );
    // The shared subgraph is published once per use-site: the second edge names the node
    // but does not walk its interior again, while the edge itself stays locatable.
    assert!(
        report
            .items
            .iter()
            .all(|item| item.evidence.via.len() == 2 || item.evidence.via[1] == nested_first),
        "only the first edge to the shared node carries its interior facts: {:?}",
        vias(&report)
    );
    check_item_invariants(&fixture, &report);
    assert!(is_complete(&report));
}

#[test]
fn every_use_site_keeps_its_own_path_to_a_shared_deeper_node() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let target = literal_string("leaf");
    let (report, _) = run_with_limits(
        &snapshot,
        &request(&snapshot, QueryRelation::LiteralValue, target, &BOOTSTRAP),
        limits(),
    );

    // The leaf is the deepest fact of the fixture, and three different use-sites reach it:
    // one through the custom site, one through a `ldc` of the level-1 node, and one through
    // a `ldc` of the level-2 node.
    assert_eq!(
        rows(&report),
        vec![
            (
                XrefOperation::BootstrapArgument,
                Some(fixture.index.leaf_string),
                Some(CUSTOM_BCI),
                Some(0xba)
            ),
            (
                XrefOperation::BootstrapArgument,
                Some(fixture.index.leaf_string),
                Some(NESTED_LDC_BCI),
                Some(0x12)
            ),
            (
                XrefOperation::BootstrapArgument,
                Some(fixture.index.leaf_string),
                Some(DEEPER_LDC_BCI),
                Some(0x12)
            ),
        ],
        "one fact per use-site, deepest first"
    );
    assert_eq!(
        vias(&report),
        vec![
            vec![
                hop(fixture.index.custom_site, None, None),
                hop(fixture.index.nested_condy, Some(1), Some(0)),
                hop(fixture.index.nested_condy2, Some(2), Some(0)),
                hop(fixture.index.leaf_string, Some(3), Some(0)),
            ],
            vec![
                hop(fixture.index.nested_condy, None, None),
                hop(fixture.index.nested_condy2, Some(2), Some(0)),
                hop(fixture.index.leaf_string, Some(3), Some(0)),
            ],
            vec![
                hop(fixture.index.nested_condy2, None, None),
                hop(fixture.index.leaf_string, Some(3), Some(0)),
            ],
        ],
        "every use-site carries its own complete path to the shared node"
    );
    for item in &report.items {
        let entry = described_entry(&fixture.bytes, item);
        assert_eq!(entry[0], 8, "CONSTANT_String");
    }
    check_item_invariants(&fixture, &report);
}

#[test]
fn bootstrap_table_entries_no_use_site_reaches_produce_nothing() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());

    // The table declares five entries, two of which no dynamic site reaches: neither the
    // unused handle nor the unused dynamic site may appear.
    let unused_handle = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Unused", "member", "()V"),
            &BOOTSTRAP,
        ),
    );
    assert!(unused_handle.items.is_empty(), "{:?}", rows(&unused_handle));
    let unused_site = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            dynamic_site_symbol("unused_condy", CONDY_DESCRIPTOR),
            &BOOTSTRAP,
        ),
    );
    assert!(unused_site.items.is_empty(), "{:?}", rows(&unused_site));

    // Control: the same query shapes find the entries a use-site really reaches, so the
    // empty results above are about unusedness and not about the query shape.
    let used_handle = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Custom", "bsm", CUSTOM_DESCRIPTOR),
            &BOOTSTRAP,
        ),
    );
    assert_eq!(
        rows(&used_handle),
        vec![
            (
                XrefOperation::BootstrapMethod,
                Some(fixture.index.custom_bsm),
                Some(CUSTOM_BCI),
                Some(0xba)
            ),
            (
                XrefOperation::BootstrapMethod,
                Some(fixture.index.custom_bsm),
                Some(CUSTOM_BCI),
                Some(0xba)
            ),
            (
                XrefOperation::BootstrapMethod,
                Some(fixture.index.custom_bsm),
                Some(CUSTOM_BCI),
                Some(0xba)
            ),
            (
                XrefOperation::BootstrapMethod,
                Some(fixture.index.custom_bsm),
                Some(NESTED_LDC_BCI),
                Some(0x12)
            ),
            (
                XrefOperation::BootstrapMethod,
                Some(fixture.index.custom_bsm),
                Some(NESTED_LDC_BCI),
                Some(0x12)
            ),
            (
                XrefOperation::BootstrapMethod,
                Some(fixture.index.custom_bsm),
                Some(DEEPER_LDC_BCI),
                Some(0x12)
            ),
        ],
        "every node a use-site reaches keeps its bootstrap method fact"
    );
    assert_eq!(
        vias(&used_handle)
            .iter()
            .map(|via| via[0].constant_pool_index)
            .collect::<Vec<_>>(),
        vec![
            fixture.index.custom_site,
            fixture.index.custom_site,
            fixture.index.custom_site,
            fixture.index.nested_condy,
            fixture.index.nested_condy,
            fixture.index.nested_condy2,
        ]
    );
    check_item_invariants(&fixture, &used_handle);
}

// ---------------------------------------------------------------------------
// Filtering and cost
// ---------------------------------------------------------------------------

#[test]
fn the_bootstrap_table_is_read_only_when_its_category_is_requested() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let target = method_symbol("p/Impl", "lambda$use$0", "()V");
    let class_length = u64::try_from(fixture.bytes.len()).expect("fixture length fits u64");

    // Without the category the table is never read, even though the class is scanned by the
    // consumer that was requested.
    let (without, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target.clone(),
            &[ConsumerKind::Invocation],
        ),
        limits(),
    );
    assert!(without.items.is_empty(), "{:?}", rows(&without));
    assert_eq!(budget.usage().class_bytes, class_length);
    assert_eq!(
        budget.usage().attribute_bytes,
        attribute_bytes_without_bootstrap(&fixture),
        "the BootstrapMethods content is never read without its category"
    );

    // With the category, the same scan reads the class once, each `Code` content once and
    // the bootstrap table once.
    let (with, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target.clone(),
            &BOOTSTRAP,
        ),
        limits(),
    );
    assert_eq!(with.items.len(), 1);
    assert_eq!(budget.usage().class_bytes, class_length);
    assert_eq!(budget.usage().read_bytes, class_length);
    assert_eq!(
        budget.usage().attribute_bytes,
        expected_attribute_bytes(&fixture)
    );
    assert_eq!(
        budget.usage().result_items,
        u64::try_from(with.items.len()).expect("item count fits u64"),
        "the consumer charges no result item of its own"
    );
    check_item_invariants(&fixture, &with);

    // A request that matches nothing still reads the table, because the deferral decision
    // is about use-sites and not about the target.
    let (nothing, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Nothing", "nope", "()V"),
            &BOOTSTRAP,
        ),
        limits(),
    );
    assert!(nothing.items.is_empty());
    assert!(is_complete(&nothing));
    assert_eq!(
        budget.usage().attribute_bytes,
        expected_attribute_bytes(&fixture)
    );

    // A code consumer request keeps its own items: the graph facts belong to the dynamic
    // site's symbol, not to the use-site symbol the code consumer answers.
    let code_only_site = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            dynamic_site_symbol("custom", CONDY_DESCRIPTOR),
            &[ConsumerKind::Invocation, ConsumerKind::Bootstrap],
        ),
    );
    assert_eq!(
        rows(&code_only_site),
        vec![(
            XrefOperation::InvokeDynamic,
            Some(fixture.index.custom_site),
            Some(CUSTOM_BCI),
            Some(0xba)
        )],
        "the graph produces no fact about the use-site's own symbol"
    );
}

// ---------------------------------------------------------------------------
// Budget, cancellation, determinism and pagination
// ---------------------------------------------------------------------------

#[test]
fn result_items_and_code_bytes_stops_keep_the_reliable_prefix() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let target = literal_string("leaf");

    let mut tight = limits();
    tight.result_items = 1;
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            target.clone(),
            &BOOTSTRAP,
        ),
        tight,
    );
    assert_eq!(report.items.len(), 1, "exactly the item it could bill");
    assert_eq!(
        partial_dimension(&report),
        Some(BudgetDimension::ResultItems)
    );
    assert_eq!(budget.usage().result_items, 1);
    assert!(diagnostic_codes(&report).contains(&"budget_exceeded_result_items"));
    assert!(report.page.has_more);
    assert!(report.page.cursor.is_some());
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );

    // A `CodeBytes` stop happens before any instruction of the only body was charged, so no
    // use-site exists and no bootstrap byte is read either.
    let mut tight = limits();
    tight.code_bytes = 4;
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            target.clone(),
            &BOOTSTRAP,
        ),
        tight,
    );
    assert!(report.items.is_empty());
    assert_eq!(
        partial_dimension(&report),
        Some(BudgetDimension::CodeBytes),
        "execution: {:?}, diagnostics: {:?}",
        report.execution,
        report.diagnostics
    );
    assert_eq!(budget.usage().code_bytes, 0);
    assert_eq!(
        budget.usage().attribute_bytes,
        attribute_bytes_without_bootstrap(&fixture),
        "the bootstrap table is not read when no use-site was reached"
    );
    assert!(diagnostic_codes(&report).contains(&"budget_exceeded_code_bytes"));

    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let report = Engine::new()
        .query(
            &snapshot,
            &request(&snapshot, QueryRelation::LiteralValue, target, &BOOTSTRAP),
            &mut budget,
        )
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

#[test]
fn the_same_request_twice_reports_the_same_items_and_paths() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let target = dynamic_site_symbol("condy", CONDY_DESCRIPTOR);
    let request = request(&snapshot, QueryRelation::MentionsSymbol, target, &BOOTSTRAP);

    let first = run(&snapshot, &request);
    let second = run(&snapshot, &request);
    assert_eq!(first.items, second.items, "the item sequence is stable");
    assert_eq!(vias(&first), vias(&second));

    // Use-sites are visited in ascending BCI, and arguments in ascending position.
    let bcis = first
        .items
        .iter()
        .map(|item| item.evidence.bci.expect("a use-site BCI"))
        .collect::<Vec<_>>();
    assert_eq!(
        bcis,
        vec![CUSTOM_BCI, CUSTOM_BCI, CUSTOM_BCI, NESTED_LDC_BCI]
    );
    let mut paths: Vec<Vec<BootstrapVia>> = Vec::new();
    for path in vias(&first) {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    assert_eq!(
        paths.len(),
        4,
        "the three use-sites and the two edge slots of the shared node stay distinct"
    );
}

#[test]
fn a_page_continuation_replays_the_bootstrap_items_without_repetition() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());
    let mut request = request(
        &snapshot,
        QueryRelation::LiteralValue,
        literal_string("leaf"),
        &BOOTSTRAP,
    );
    let complete = run(&snapshot, &request);
    assert_eq!(complete.items.len(), 3);

    request.max_items = 2;
    let (first, _) = run_with_limits(&snapshot, &request, limits());
    assert_eq!(first.items.len(), 2);
    assert!(first.page.has_more);
    let mut continuation = request.clone();
    continuation.cursor = Some(first.page.cursor.clone().expect("a continuation cursor"));

    let (second, _) = run_with_limits(&snapshot, &continuation, limits());
    assert_eq!(second.items.len(), 1, "the last item of the unit");
    assert!(!second.page.has_more);
    assert!(second.page.cursor.is_none());

    let mut replayed = first.items.clone();
    replayed.extend(second.items.clone());
    assert_eq!(
        replayed, complete.items,
        "consecutive pages neither repeat nor skip a bootstrap item"
    );
}

// ---------------------------------------------------------------------------
// Cycles and the derived bound
// ---------------------------------------------------------------------------

/// A nested-condy fixture: one `ldc` over `nodes[0]`, one bootstrap entry per node.
struct CondyFixture {
    bytes: Vec<u8>,
    nodes: Vec<u16>,
    leaf: u16,
}

/// A chain of `levels` nested nodes, or a cycle that closes the chain back to its first
/// node.
fn condy_fixture(levels: usize, cycle: bool) -> CondyFixture {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/Condy");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let code_name = pool.utf8(b"Code");
    let bootstrap_name = pool.utf8(b"BootstrapMethods");
    let use_name = pool.utf8(b"use");
    let void_descriptor = pool.utf8(b"()V");
    let owner_name = pool.utf8(b"p/Custom");
    let owner_class = pool.class(owner_name);
    let bsm_name = pool.utf8(b"bsm");
    let bsm_descriptor = pool.utf8(CUSTOM_DESCRIPTOR.as_bytes());
    let bsm_nat = pool.name_and_type(bsm_name, bsm_descriptor);
    let bsm_ref = pool.member(10, owner_class, bsm_nat);
    let handle = pool.method_handle(6, bsm_ref);
    let leaf_text = pool.utf8(b"leaf");
    let leaf = pool.string(leaf_text);
    let condy_name = pool.utf8(b"condy");
    let condy_descriptor = pool.utf8(CONDY_DESCRIPTOR.as_bytes());
    let condy_nat = pool.name_and_type(condy_name, condy_descriptor);

    let nodes = (0..levels)
        .map(|level| {
            pool.dynamic(
                u16::try_from(level).expect("fixture levels fit u16"),
                condy_nat,
            )
        })
        .collect::<Vec<_>>();
    let entries = (0..levels)
        .map(|level| {
            let arguments = if level + 1 < levels {
                vec![nodes[level + 1]]
            } else if cycle {
                vec![nodes[0]]
            } else {
                vec![leaf]
            };
            (handle, arguments)
        })
        .collect::<Vec<_>>();
    let bootstrap = bootstrap_body(&entries);

    let mut code = ldc(nodes[0]);
    code.push(0xb1);
    let method = MethodSpec {
        access: 0x0002,
        name: use_name,
        descriptor: void_descriptor,
        attribute: Some((code_name, code_body(&code))),
    };
    let (bytes, _) = class_bytes(
        &pool,
        this_class,
        object_class,
        &[method],
        &[AttributeSpec {
            name: bootstrap_name,
            content: bootstrap,
        }],
    );
    CondyFixture { bytes, nodes, leaf }
}

/// Constant-pool entry count a class file declares.
fn pool_count(bytes: &[u8]) -> u16 {
    u16::from_be_bytes([bytes[8], bytes[9]])
}

/// A short chain is walked to its end, and the deepest fact keeps its whole path.
#[test]
fn a_condy_chain_within_the_derived_bound_completes() {
    let fixture = condy_fixture(3, false);
    let snapshot = open(fixture.bytes.clone());
    let (report, _) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            literal_string("leaf"),
            &BOOTSTRAP,
        ),
        limits(),
    );

    assert!(is_complete(&report), "{:?}", report.execution);
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(report.items.len(), 1, "{:?}", rows(&report));
    assert_eq!(
        report.items[0].evidence.via,
        vec![
            hop(fixture.nodes[0], None, None),
            hop(fixture.nodes[1], Some(0), Some(0)),
            hop(fixture.nodes[2], Some(1), Some(0)),
            hop(fixture.leaf, Some(2), Some(0)),
        ],
        "three nested nodes, then the terminal argument"
    );
}

#[test]
fn a_condy_chain_past_the_derived_bound_stops_with_a_diagnostic() {
    let levels = 24;
    let fixture = condy_fixture(levels, false);
    // The bound is derived from the class's own constant pool: the traversal may count as
    // many entries as the pool holds (this fixture has no `Long`/`Double`, so its entry count
    // is the declared count minus the unused zero slot). Each chain level charges exactly two
    // edges — its bootstrap method handle and its single static argument — so a chain of
    // `levels` nodes needs `2 * levels` edges, more edges than the class has entries.
    let entries = usize::from(pool_count(&fixture.bytes) - 1);
    assert!(
        2 * levels > entries,
        "the fixture must really exceed the derived bound: {} edges over {entries} entries",
        2 * levels
    );
    // Nodes the walk can enter before the edge bound refuses the next edge.
    let expanded = entries / 2;
    assert!(expanded < levels, "the chain must be truncated");

    let snapshot = open(fixture.bytes.clone());
    let (report, _) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            dynamic_site_symbol("condy", CONDY_DESCRIPTOR),
            &BOOTSTRAP,
        ),
        limits(),
    );

    assert_eq!(failed_code(&report), Some("query_bootstrap_graph_limit"));
    assert!(diagnostic_codes(&report).contains(&"query_bootstrap_graph_limit"));
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
    assert!(report.page.has_more);

    // The published prefix is pinned, not merely bounded. A query over the chain's own dynamic
    // site symbol answers one fact per followed argument edge — the nodes 1 through `expanded`
    // — and no bootstrap method fact, because that fact names the bootstrap member instead.
    assert_eq!(
        report.items.len(),
        expanded,
        "one dynamic-site fact per followed edge: {:?}",
        rows(&report)
    );
    assert!(
        report
            .items
            .iter()
            .all(|item| item.operation == XrefOperation::BootstrapArgument)
    );
    assert_eq!(
        report
            .items
            .iter()
            .map(|item| item.evidence.constant_pool_index.expect("an entry"))
            .collect::<Vec<_>>(),
        fixture.nodes[1..=expanded].to_vec(),
        "the facts run along the chain in order and stop at the first node that did not enter"
    );

    // The complementary product: the same walk truncated after exactly the same nodes, seen as
    // the bootstrap method facts of the nodes that were entered (0 through `expanded - 1`).
    let (by_handle, _) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("p/Custom", "bsm", CUSTOM_DESCRIPTOR),
            &BOOTSTRAP,
        ),
        limits(),
    );
    assert_eq!(failed_code(&by_handle), Some("query_bootstrap_graph_limit"));
    assert_eq!(
        by_handle
            .items
            .iter()
            .filter(|item| item.operation == XrefOperation::BootstrapMethod)
            .count(),
        expanded,
        "one bootstrap method fact per node whose entry was charged"
    );
    assert_eq!(by_handle.items.len(), expanded, "{:?}", rows(&by_handle));

    // Where it stopped: the deepest fact is the edge to the first node that could not be
    // entered, and no fact mentions a node beyond it.
    let deepest = &report.items[report.items.len() - 1];
    let via = &deepest.evidence.via;
    assert_eq!(via.len(), expanded + 1);
    assert_eq!(via[0], hop(fixture.nodes[0], None, None));
    assert_eq!(
        via[via.len() - 1].constant_pool_index,
        fixture.nodes[expanded],
        "the last published edge reaches the node whose entry stopped the traversal"
    );
    for (depth, element) in via.iter().enumerate().skip(1) {
        assert_eq!(
            element.constant_pool_index, fixture.nodes[depth],
            "hop {depth} is the {depth}-th node of the chain"
        );
        assert_eq!(
            element.bootstrap_index,
            Some(u16::try_from(depth - 1).expect("depth fits u16"))
        );
        assert_eq!(element.argument_index, Some(0));
    }
    assert!(
        report.items.iter().all(|item| {
            item.evidence.via.iter().all(|element| {
                element.constant_pool_index != fixture.leaf
                    && fixture.nodes[..=expanded].contains(&element.constant_pool_index)
            })
        }),
        "no fact may describe a node past the stop, and the truncated chain never reaches the \
         terminal string: {:?}",
        vias(&report)
    );
}

#[test]
fn a_cycle_is_reported_instead_of_followed() {
    // A node whose bootstrap argument is the node itself, and a two-node cycle: neither may
    // recurse, and both must name the repeated entry.
    for levels in [1_usize, 2] {
        let fixture = condy_fixture(levels, true);
        let snapshot = open(fixture.bytes.clone());
        let (report, _) = run_with_limits(
            &snapshot,
            &request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                dynamic_site_symbol("condy", CONDY_DESCRIPTOR),
                &BOOTSTRAP,
            ),
            limits(),
        );

        assert_eq!(
            failed_code(&report),
            Some("query_bootstrap_cycle"),
            "a cycle is a structural stop for {levels} nodes"
        );
        assert!(
            diagnostic_codes(&report).contains(&"query_bootstrap_cycle"),
            "{:?}",
            report.diagnostics
        );
        assert_eq!(
            report.items.len(),
            levels,
            "every edge up to the repeated node stays a fact: {:?}",
            vias(&report)
        );
        // The last fact is the edge that closes the cycle.
        let closing = &report.items[levels - 1];
        let via = &closing.evidence.via;
        assert_eq!(via[via.len() - 1].constant_pool_index, fixture.nodes[0]);
        assert_eq!(via[0].constant_pool_index, fixture.nodes[0]);
        assert_eq!(via.len(), levels + 1);
        assert_eq!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::Partial
        );
    }
}

// ---------------------------------------------------------------------------
// The `ldc` family, the raw pool probe and entries only unused tables reach
// ---------------------------------------------------------------------------

/// `ldc_w` over one constant-pool entry.
fn ldc_w(entry: u16) -> Vec<u8> {
    let mut bytes = vec![0x13];
    u16b(&mut bytes, entry);
    bytes
}

/// `ldc2_w` over one constant-pool entry.
fn ldc2_w(entry: u16) -> Vec<u8> {
    let mut bytes = vec![0x14];
    u16b(&mut bytes, entry);
    bytes
}

/// One dynamic node of the `ldc` family fixture.
struct LdcNode {
    site: u16,
    opcode: u8,
    bci: u32,
}

/// A class whose single member consumes three dynamic entries through the three `ldc` forms.
struct LdcFamily {
    bytes: Vec<u8>,
    nodes: Vec<LdcNode>,
    /// The one `String` every entry's bootstrap declares as its static argument.
    leaf: u16,
    /// Length of the member's instruction array.
    code_length: usize,
}
/// Builds one `ldc`-family fixture: `ldc` over an object-typed dynamic entry, `ldc2_w` over a
/// `J`-typed one — the entry kind `ldc2_w` exists for (JVMS §6.5) — and `ldc_w` over a third.
///
/// All three entries share one bootstrap whose single static argument is a string, so a
/// query for that string reports one fact per instruction.
fn ldc_family_fixture() -> LdcFamily {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/LdcFamily");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let code_name = pool.utf8(b"Code");
    let bootstrap_name = pool.utf8(b"BootstrapMethods");
    let use_name = pool.utf8(b"use");
    let void_descriptor = pool.utf8(b"()V");
    let owner_name = pool.utf8(b"p/Custom");
    let owner_class = pool.class(owner_name);
    let bsm_name = pool.utf8(b"bsm");
    let bsm_descriptor = pool.utf8(CUSTOM_DESCRIPTOR.as_bytes());
    let bsm_nat = pool.name_and_type(bsm_name, bsm_descriptor);
    let bsm_ref = pool.member(10, owner_class, bsm_nat);
    let handle = pool.method_handle(6, bsm_ref);
    let leaf_text = pool.utf8(b"leaf");
    let leaf = pool.string(leaf_text);
    let object_descriptor = pool.utf8(CONDY_DESCRIPTOR.as_bytes());
    let long_descriptor = pool.utf8(b"J");
    let first_name = pool.utf8(b"first");
    let second_name = pool.utf8(b"second");
    let third_name = pool.utf8(b"third");
    let first_nat = pool.name_and_type(first_name, object_descriptor);
    let second_nat = pool.name_and_type(second_name, long_descriptor);
    let third_nat = pool.name_and_type(third_name, object_descriptor);
    let first = pool.dynamic(0, first_nat);
    let second = pool.dynamic(0, second_nat);
    let third = pool.dynamic(0, third_nat);

    let mut code = Vec::new();
    let nodes = vec![
        LdcNode {
            site: first,
            opcode: 0x12,
            bci: 0,
        },
        LdcNode {
            site: second,
            opcode: 0x14,
            bci: 2,
        },
        LdcNode {
            site: third,
            opcode: 0x13,
            bci: 5,
        },
    ];
    code.extend_from_slice(&ldc(first));
    code.extend_from_slice(&ldc2_w(second));
    code.extend_from_slice(&ldc_w(third));
    code.push(0xb1); // return

    let method = MethodSpec {
        access: 0x0002,
        name: use_name,
        descriptor: void_descriptor,
        attribute: Some((code_name, code_body(&code))),
    };
    let (bytes, layout) = class_bytes(
        &pool,
        this_class,
        object_class,
        &[method],
        &[AttributeSpec {
            name: bootstrap_name,
            content: bootstrap_body(&[(handle, vec![leaf])]),
        }],
    );
    // Each declared node really is the instruction the test reads: its form sits at its BCI.
    let code_start = layout.method_attributes[0] + 8;
    for node in &nodes {
        assert_eq!(
            bytes[(code_start + u64::from(node.bci)) as usize],
            node.opcode,
            "node at BCI {} is opcode {:#04x}",
            node.bci,
            node.opcode
        );
    }
    LdcFamily {
        bytes,
        nodes,
        leaf,
        code_length: code.len(),
    }
}

#[test]
fn the_whole_ldc_family_opens_a_deferred_graph() {
    let fixture = ldc_family_fixture();
    let snapshot = open(fixture.bytes.clone());
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            literal_string("leaf"),
            &BOOTSTRAP,
        ),
        limits(),
    );

    // One fact per instruction, each keeping the form and the position it was read at. The
    // described entry is the string all three entries share, so the use-site coordinates are
    // what tells the three facts apart.
    assert_eq!(
        rows(&report),
        vec![
            (
                XrefOperation::BootstrapArgument,
                Some(fixture.leaf),
                Some(0),
                Some(0x12)
            ),
            (
                XrefOperation::BootstrapArgument,
                Some(fixture.leaf),
                Some(2),
                Some(0x14)
            ),
            (
                XrefOperation::BootstrapArgument,
                Some(fixture.leaf),
                Some(5),
                Some(0x13)
            ),
        ],
        "every `ldc` form reaches its own graph: {:?}",
        report.diagnostics
    );
    for (item, node) in report.items.iter().zip(&fixture.nodes) {
        assert_eq!(
            item.evidence.via,
            vec![
                hop(node.site, None, None),
                hop(fixture.leaf, Some(0), Some(0))
            ],
            "each use-site is the first hop of its own path"
        );
    }
    assert!(is_complete(&report), "{:?}", report.execution);
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    // Every byte of this body belongs to exactly one instruction (`ldc` is two bytes, the wide
    // forms three, `return` one, and no instruction needs alignment padding), so a complete
    // decode charges the code array's length exactly once.
    assert_eq!(
        budget.usage().code_bytes,
        u64::try_from(fixture.code_length).expect("code length fits u64"),
        "one charge per decoded instruction width"
    );
}

#[test]
fn the_raw_pool_probe_never_produces_a_bootstrap_edge() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());

    // `constant_pool_contains` asks about pool entries, so the X0 producer answers it and the
    // bootstrap consumer must stay out of the way even when its category is requested.
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::ConstantPoolContains,
            dynamic_site_symbol("condy", CONDY_DESCRIPTOR),
            &BOOTSTRAP,
        ),
        limits(),
    );
    assert_eq!(
        report.items.len(),
        3,
        "the three `condy` pool entries are candidates: {:?}",
        rows(&report)
    );
    for item in &report.items {
        assert_eq!(item.operation, XrefOperation::ConstantPoolEntry);
        assert_eq!(item.consumer, None);
        assert_eq!(item.derivation, XrefDerivation::ConstantPoolCandidate);
        assert!(item.evidence.via.is_empty());
    }
    assert_eq!(
        budget.usage().attribute_bytes,
        fixture.bootstrap_shell + fixture.code_shell,
        "a pool probe reads the class, its shells and no attribute content at all"
    );

    // The same target under a consumer relation is a bootstrap edge, which is what makes the
    // probe's silence a guard and not a no-match.
    let edges = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            dynamic_site_symbol("condy", CONDY_DESCRIPTOR),
            &BOOTSTRAP,
        ),
    );
    assert!(
        edges
            .items
            .iter()
            .all(|item| item.derivation == XrefDerivation::BootstrapEdge),
        "{:?}",
        rows(&edges)
    );
    assert!(!edges.items.is_empty());
}

#[test]
fn an_entry_only_an_unused_table_entry_reaches_produces_no_fact() {
    let fixture = fixture();
    let snapshot = open(fixture.bytes.clone());

    // `orphan` is a static argument of the fifth table entry, which no dynamic site names, so
    // no path from any use-site can reach it.
    let orphan = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            literal_string("orphan"),
            &BOOTSTRAP,
        ),
    );
    assert!(orphan.items.is_empty(), "{:?}", rows(&orphan));
    assert!(is_complete(&orphan), "{:?}", orphan.execution);

    // Control: the same query shape and the same kind of literal, on a string a used entry
    // really reaches, is reported. The difference is reachability, not the query.
    let reached = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            literal_string("leaf"),
            &BOOTSTRAP,
        ),
    );
    assert_eq!(reached.items.len(), 3, "{:?}", rows(&reached));
    assert!(
        reached
            .items
            .iter()
            .all(|item| item.evidence.constant_pool_index == Some(fixture.index.leaf_string))
    );

    // The orphan entry really is in the class file: the raw pool probe finds it (as the
    // `String` and as the `Utf8` it points at), so the empty bootstrap result is about
    // reachability and not about a fixture entry that was never written.
    let candidate = run(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::ConstantPoolContains,
            literal_string("orphan"),
            &BOOTSTRAP,
        ),
    );
    assert_eq!(candidate.items.len(), 2, "{:?}", rows(&candidate));
    assert!(
        candidate
            .items
            .iter()
            .any(|item| item.evidence.constant_pool_index == Some(fixture.index.orphan_string)),
        "the pool probe describes the `String` entry: {:?}",
        rows(&candidate)
    );
    assert!(
        candidate
            .items
            .iter()
            .all(|item| item.evidence.constant_pool_index != Some(fixture.index.leaf_string))
    );
}

// ---------------------------------------------------------------------------
// A table that cannot be used as declared
// ---------------------------------------------------------------------------

/// The shared pool of the small stop fixtures: a valid bootstrap handle, one `Dynamic` over
/// `name` that names `bootstrap_index`, and one `ldc` use-site over it.
struct SmallCondy {
    pool: Pool,
    this_class: u16,
    object_class: u16,
    code_name: u16,
    bootstrap_name: u16,
    use_name: u16,
    void_descriptor: u16,
    handle: u16,
    condy: u16,
}

impl SmallCondy {
    /// Assembles the class with one class-level attribute per element of `attributes`.
    ///
    /// `uses_condy` decides whether the single member consumes the dynamic entry at all:
    /// with no use-site the consumer must not read the table it declares.
    fn build(&self, attributes: &[(u16, Vec<u8>)], uses_condy: bool) -> Vec<u8> {
        let mut code = if uses_condy {
            ldc(self.condy)
        } else {
            Vec::new()
        };
        code.push(0xb1);
        let method = MethodSpec {
            access: 0x0002,
            name: self.use_name,
            descriptor: self.void_descriptor,
            attribute: Some((self.code_name, code_body(&code))),
        };
        let attributes = attributes
            .iter()
            .map(|(name, content)| AttributeSpec {
                name: *name,
                content: content.clone(),
            })
            .collect::<Vec<_>>();
        let (bytes, _) = class_bytes(
            &self.pool,
            self.this_class,
            self.object_class,
            &[method],
            &attributes,
        );
        bytes
    }
}

fn small_condy(bootstrap_index: u16, name: &[u8]) -> SmallCondy {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/Small");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let code_name = pool.utf8(b"Code");
    let bootstrap_name = pool.utf8(b"BootstrapMethods");
    let use_name = pool.utf8(b"use");
    let void_descriptor = pool.utf8(b"()V");
    let owner_name = pool.utf8(b"p/Custom");
    let owner_class = pool.class(owner_name);
    let bsm_name = pool.utf8(b"bsm");
    let bsm_descriptor = pool.utf8(CUSTOM_DESCRIPTOR.as_bytes());
    let bsm_nat = pool.name_and_type(bsm_name, bsm_descriptor);
    let bsm_ref = pool.member(10, owner_class, bsm_nat);
    let handle = pool.method_handle(6, bsm_ref);
    let condy_name = pool.utf8(name);
    let condy_descriptor = pool.utf8(CONDY_DESCRIPTOR.as_bytes());
    let condy_nat = pool.name_and_type(condy_name, condy_descriptor);
    let condy = pool.dynamic(bootstrap_index, condy_nat);
    SmallCondy {
        pool,
        this_class,
        object_class,
        code_name,
        bootstrap_name,
        use_name,
        void_descriptor,
        handle,
        condy,
    }
}

#[test]
fn a_class_without_a_dynamic_use_site_never_reads_the_table() {
    let small = small_condy(0, b"condy");
    let table = bootstrap_body(&[(small.handle, vec![])]);
    let bytes = small.build(&[(small.bootstrap_name, table.clone())], false);
    let snapshot = open(bytes);
    // The handle's descriptor is the only place this type is named, so any category that
    // read the table into facts would have to report it.
    let target = class_symbol("java/lang/invoke/MethodHandles$Lookup");
    let body = code_body(&[0xb1]);
    let shells = shell(&table) + shell(&body);

    // `Bootstrap` alone: one pass over the class's shells, plus the body it reads for its
    // use-sites.
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            dynamic_site_symbol("condy", CONDY_DESCRIPTOR),
            &BOOTSTRAP,
        ),
        limits(),
    );
    assert!(report.items.is_empty(), "{:?}", rows(&report));
    assert!(is_complete(&report));
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(
        budget.usage().attribute_bytes,
        shells + shell(&body),
        "the table's shell is enumerated once and its content is never read"
    );

    // The categories that own facts in a table behave the same way when no instruction
    // consumes a dynamic entry: `Type` opens the graph only where a real use-site opens it,
    // so neither the `Type` category nor `Type` with `Bootstrap` reads the table's content.
    // Each class pass bills every shell once (the code, metadata and bootstrap streams each
    // read the class) and each body read bills the `Code` entry once — the table appears in
    // the shell passes alone, and a content read would add another `table` charge.
    for (kinds, passes, body_reads) in [
        (&[ConsumerKind::Type][..], 3_u64, 2_u64),
        (&[ConsumerKind::Bootstrap, ConsumerKind::Type][..], 3, 2),
    ] {
        let (report, budget) = run_with_limits(
            &snapshot,
            &request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                target.clone(),
                kinds,
            ),
            limits(),
        );
        assert!(
            report.items.is_empty(),
            "{kinds:?}: an unreached table entry produces nothing: {:?}",
            report.items
        );
        assert!(is_complete(&report), "{kinds:?}");
        assert_eq!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema,
            "{kinds:?}"
        );
        assert_eq!(
            budget.usage().attribute_bytes,
            passes * shells + body_reads * shell(&body),
            "{kinds:?}: the table's content is never read"
        );
    }
}

#[test]
fn a_member_with_broken_code_does_not_hide_another_member_site() {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/Broken");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let code_name = pool.utf8(b"Code");
    let bootstrap_name = pool.utf8(b"BootstrapMethods");
    let broken_name = pool.utf8(b"broken");
    let use_name = pool.utf8(b"use");
    let void_descriptor = pool.utf8(b"()V");
    let owner_name = pool.utf8(b"p/Custom");
    let owner_class = pool.class(owner_name);
    let bsm_name = pool.utf8(b"bsm");
    let bsm_descriptor = pool.utf8(CUSTOM_DESCRIPTOR.as_bytes());
    let bsm_nat = pool.name_and_type(bsm_name, bsm_descriptor);
    let bsm_ref = pool.member(10, owner_class, bsm_nat);
    let handle = pool.method_handle(6, bsm_ref);
    let leaf_text = pool.utf8(b"leaf");
    let leaf = pool.string(leaf_text);
    let condy_name = pool.utf8(b"condy");
    let condy_descriptor = pool.utf8(CONDY_DESCRIPTOR.as_bytes());
    let condy_nat = pool.name_and_type(condy_name, condy_descriptor);
    let condy = pool.dynamic(0, condy_nat);

    // `invokevirtual` with its operand byte missing: the first member cannot be decoded to
    // its end, and the second member's dynamic site is still a fact.
    let broken_code = vec![0xb6, 0x00];
    let mut use_code = ldc(condy);
    use_code.push(0xb1);
    let methods = [
        MethodSpec {
            access: 0x0002,
            name: broken_name,
            descriptor: void_descriptor,
            attribute: Some((code_name, code_body(&broken_code))),
        },
        MethodSpec {
            access: 0x0002,
            name: use_name,
            descriptor: void_descriptor,
            attribute: Some((code_name, code_body(&use_code))),
        },
    ];
    let (bytes, _) = class_bytes(
        &pool,
        this_class,
        object_class,
        &methods,
        &[AttributeSpec {
            name: bootstrap_name,
            content: bootstrap_body(&[(handle, vec![leaf])]),
        }],
    );
    let snapshot = open(bytes);
    let (report, _) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::LiteralValue,
            literal_string("leaf"),
            &BOOTSTRAP,
        ),
        limits(),
    );

    assert_eq!(report.items.len(), 1, "{:?}", rows(&report));
    let (method, bci) = code_location(&report.items[0]);
    assert_eq!(method.name.0.as_slice(), &b"use"[..]);
    assert_eq!(bci, 0);
    assert_eq!(
        report.items[0].evidence.via,
        vec![hop(condy, None, None), hop(leaf, Some(0), Some(0))]
    );
    // A use-site list that is missing the stopped member's sites must not read as a
    // complete graph.
    assert!(!is_complete(&report));
    assert!(failed_code(&report).is_some(), "{:?}", report.execution);
    assert!(
        diagnostic_codes(&report).contains(&"query_bootstrap_code_stopped"),
        "{:?}",
        report.diagnostics
    );
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
}

#[test]
fn a_missing_duplicated_or_broken_table_is_a_structured_stop() {
    let target = dynamic_site_symbol("condy", CONDY_DESCRIPTOR);

    // A dynamic use-site without the attribute a dynamic site requires (JVMS 4.7.23).
    let small = small_condy(0, b"condy");
    let bytes = small.build(&[], true);
    let snapshot = open(bytes);
    let (report, _) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target.clone(),
            &BOOTSTRAP,
        ),
        limits(),
    );
    assert_eq!(
        failed_code(&report),
        Some("query_bootstrap_attribute_missing")
    );
    assert!(diagnostic_codes(&report).contains(&"query_bootstrap_attribute_missing"));
    assert!(report.items.is_empty());

    // Two bootstrap tables: no single table a dynamic entry could mean.
    let content = bootstrap_body(&[(small.handle, vec![])]);
    let bytes = small.build(
        &[
            (small.bootstrap_name, content.clone()),
            (small.bootstrap_name, content),
        ],
        true,
    );
    let snapshot = open(bytes);
    let (report, _) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target.clone(),
            &BOOTSTRAP,
        ),
        limits(),
    );
    assert_eq!(
        failed_code(&report),
        Some("query_bootstrap_attribute_duplicate")
    );
    assert!(diagnostic_codes(&report).contains(&"query_bootstrap_attribute_duplicate"));

    // A bootstrap handle that is not a `MethodHandle`: the reader owns that contract, and
    // this consumer explains which use-site made it read the attribute.
    let mut small = small_condy(0, b"condy");
    let method_type = small.pool.method_type(small.void_descriptor);
    let bytes = small.build(
        &[(
            small.bootstrap_name,
            bootstrap_body(&[(method_type, vec![])]),
        )],
        true,
    );
    let snapshot = open(bytes);
    let (report, _) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target.clone(),
            &BOOTSTRAP,
        ),
        limits(),
    );
    assert_eq!(
        failed_code(&report),
        Some("classfile_constant_pool_tag_mismatch")
    );
    assert!(diagnostic_codes(&report).contains(&"query_bootstrap_malformed"));

    // A dynamic site that names a bootstrap entry the table does not declare.
    let small = small_condy(3, b"condy");
    let bytes = small.build(
        &[(
            small.bootstrap_name,
            bootstrap_body(&[(small.handle, vec![])]),
        )],
        true,
    );
    let snapshot = open(bytes);
    let (report, _) = run_with_limits(
        &snapshot,
        &request(&snapshot, QueryRelation::MentionsSymbol, target, &BOOTSTRAP),
        limits(),
    );
    assert_eq!(failed_code(&report), Some("query_bootstrap_malformed"));
    assert!(diagnostic_codes(&report).contains(&"query_bootstrap_malformed"));
    assert!(report.items.is_empty());
}

// ---------------------------------------------------------------------------
// Class candidates inside an archive
// ---------------------------------------------------------------------------

/// `container:root:xref_scan_entries` skipped ranges of the structural dimension.
fn xref_skipped_ranges(report: &QueryReport) -> Vec<(u64, u64)> {
    report
        .coverage
        .dimensions
        .artifact_structural
        .skipped
        .iter()
        .filter(|range| range.label == "container:root:xref_scan_entries")
        .map(|range| (range.start, range.end))
        .collect()
}

#[test]
fn a_truncated_class_candidate_fails_with_its_entry_origin() {
    let fixture = fixture();
    let archive = zip(&[
        (b"p/Boot.class", fixture.bytes.as_slice()),
        (b"p/Broken.class", &[0xca, 0xfe, 0xba]),
        (b"p/Other.class", fixture.bytes.as_slice()),
    ]);
    let snapshot = open(archive);
    let (report, _) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            dynamic_site_symbol("condy", CONDY_DESCRIPTOR),
            &BOOTSTRAP,
        ),
        limits(),
    );

    // Every use-site of the candidate that scanned before the damaged one stays
    // published, in the scan's own order.
    let bcis = report
        .items
        .iter()
        .map(|item| item.evidence.bci.expect("a use-site BCI"))
        .collect::<Vec<_>>();
    assert_eq!(
        bcis,
        vec![CUSTOM_BCI, CUSTOM_BCI, CUSTOM_BCI, NESTED_LDC_BCI]
    );
    for item in &report.items {
        let (method, _) = code_location(item);
        match &method.owner.location {
            PhysicalClassLocation::ArchiveEntry { entry } => {
                assert_eq!(entry.raw_name.0, b"p/Boot.class");
            }
            other => panic!("expected the archive entry as the physical source, got {other:?}"),
        }
    }
    // The damaged candidate is located by entry, and neither the execution nor the
    // structural coverage claims this range complete.
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
    assert!(
        diagnostic.message.contains("Broken.class"),
        "{diagnostic:?}"
    );
    assert!(diagnostic.message.contains("CA FE BA BE"), "{diagnostic:?}");
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
    assert_eq!(xref_skipped_ranges(&report), vec![(2, 3)]);
    assert!(report.page.has_more);
}

#[test]
fn the_class_candidate_rule_is_case_sensitive_and_shared() {
    let fixture = fixture();
    let garbage = b"this entry is not a class file";
    // The upper-case name holds the *same legal class bytes* as the lower-case one: a
    // case-folding rule would report its items a second time and materialize it, so the
    // assertions below are falsified by that mistake instead of agreeing with it.
    let archive = zip(&[
        (b"p/Boot.class", fixture.bytes.as_slice()),
        (b".class", fixture.bytes.as_slice()),
        (b"p/Boot.CLASS", fixture.bytes.as_slice()),
        (b"docs/readme.txt", garbage),
    ]);
    let snapshot = open(archive);
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            dynamic_site_symbol("condy", CONDY_DESCRIPTOR),
            &BOOTSTRAP,
        ),
        limits(),
    );

    // The lower-case name and the bare `.class` are both candidates: the bootstrap
    // stream uses the same rule as the code and metadata streams.
    assert_eq!(report.items.len(), 8, "{:?}", report.items);
    let mut entry_names = report
        .items
        .iter()
        .map(|item| {
            let (method, _) = code_location(item);
            match &method.owner.location {
                PhysicalClassLocation::ArchiveEntry { entry } => entry.raw_name.0.clone(),
                other => panic!("expected an archive entry, got {other:?}"),
            }
        })
        .collect::<Vec<_>>();
    entry_names.dedup();
    assert_eq!(
        entry_names,
        vec![b"p/Boot.class".to_vec(), b".class".to_vec()]
    );
    // The upper-case name and the plain resource stay outside the range: even though the
    // upper-case entry is a legal class file, it produces no items and no damage
    // diagnostic, and it is never materialized.
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

// ---------------------------------------------------------------------------
// A04/A05: the types a bootstrap descriptor really names
// ---------------------------------------------------------------------------

/// One bootstrap table whose descriptors are the only place their types appear.
///
/// Entry 0 is reached by the site, and its `MethodType` argument is declared twice, so one
/// shared entry is reached through two argument slots; entry 1 is reached by nothing. Both
/// bootstrap method handles and both `MethodType` descriptors name types that exist nowhere
/// else in the pool, so a type item can only come from the descriptor a real edge reached.
struct DescriptorBootstrap {
    bytes: Vec<u8>,
    /// `CONSTANT_InvokeDynamic` of the site.
    site: u16,
    /// `CONSTANT_MethodHandle` of the reached bootstrap method.
    bsm_handle: u16,
    /// `CONSTANT_MethodType` static argument declared in both argument slots.
    shared_argument: u16,
    code_span: ByteSpan,
    code_shell: u64,
    bootstrap_shell: u64,
}

fn descriptor_bootstrap_fixture() -> DescriptorBootstrap {
    let mut pool = Pool::default();
    let class_name = pool.utf8(b"p/DescriptorBoot");
    let this_class = pool.class(class_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let code_name = pool.utf8(b"Code");
    let bootstrap_name = pool.utf8(b"BootstrapMethods");
    let use_name = pool.utf8(b"use");
    let void_descriptor = pool.utf8(b"()V");

    // Entry 0: a bootstrap method whose descriptor names two types of its own.
    let bsm_owner = pool.utf8(b"p/Bsm");
    let bsm_class = pool.class(bsm_owner);
    let bsm_name = pool.utf8(b"prepare");
    let bsm_descriptor = pool.utf8(b"(Lp/BsmParam;)Lp/BsmResult;");
    let bsm_nat = pool.name_and_type(bsm_name, bsm_descriptor);
    let bsm_ref = pool.member(10, bsm_class, bsm_nat);
    let bsm_handle = pool.method_handle(6, bsm_ref);
    let shared_descriptor = pool.utf8(b"(Lp/SharedParam;)Lp/SharedResult;");
    let shared_argument = pool.method_type(shared_descriptor);

    // Entry 1: a handle and an argument no dynamic site reaches.
    let unused_owner = pool.utf8(b"p/UnusedBsm");
    let unused_class = pool.class(unused_owner);
    let unused_name = pool.utf8(b"run");
    let unused_descriptor = pool.utf8(b"(Lp/UnusedParam;)Lp/UnusedResult;");
    let unused_nat = pool.name_and_type(unused_name, unused_descriptor);
    let unused_ref = pool.member(10, unused_class, unused_nat);
    let unused_handle = pool.method_handle(6, unused_ref);
    let unused_type_descriptor = pool.utf8(b"(Lp/UnusedParam2;)Lp/UnusedResult2;");
    let unused_type = pool.method_type(unused_type_descriptor);

    // The site: its own descriptor names no type, so no item can come from the code
    // consumer's product for the same instruction.
    let site_name = pool.utf8(b"site");
    let site_nat = pool.name_and_type(site_name, void_descriptor);
    let site = pool.invoke_dynamic(0, site_nat);

    for name in [
        b"p/BsmParam".as_slice(),
        b"p/BsmResult",
        b"p/SharedParam",
        b"p/SharedResult",
        b"p/UnusedParam",
        b"p/UnusedResult",
        b"p/UnusedParam2",
        b"p/UnusedResult2",
    ] {
        assert!(
            !pool.entries.iter().any(|entry| {
                entry.first() == Some(&1) && entry.len() == 3 + name.len() && &entry[3..] == name
            }),
            "{} must exist only inside a bootstrap descriptor",
            String::from_utf8_lossy(name)
        );
    }

    let instructions = indy(site);
    let body = code_body(&instructions);
    let bootstrap = bootstrap_body(&[
        (bsm_handle, vec![shared_argument, shared_argument]),
        (unused_handle, vec![unused_type]),
    ]);
    let (bytes, layout) = class_bytes(
        &pool,
        this_class,
        object_class,
        &[MethodSpec {
            access: 0x0002,
            name: use_name,
            descriptor: void_descriptor,
            attribute: Some((code_name, body.clone())),
        }],
        &[AttributeSpec {
            name: bootstrap_name,
            content: bootstrap.clone(),
        }],
    );
    let code_span = ByteSpan::new(
        layout.method_attributes[0] + 8,
        u64::try_from(instructions.len()).expect("fixture code fits u64"),
    );
    assert_eq!(bytes[code_span.start as usize], 0xba);
    DescriptorBootstrap {
        bytes,
        site,
        bsm_handle,
        shared_argument,
        code_span,
        code_shell: shell(&body),
        bootstrap_shell: shell(&bootstrap),
    }
}

#[test]
fn bootstrap_descriptors_report_their_types_with_the_edge_that_reached_them() {
    let fixture = descriptor_bootstrap_fixture();
    let snapshot = open(fixture.bytes.clone());

    // The bootstrap method handle: its descriptor's types are the types the handle
    // declares, at the node the handle is, with the path that reached it.
    let report = run_complete(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/BsmResult"),
            &[ConsumerKind::Bootstrap, ConsumerKind::Type],
        ),
    );
    assert_eq!(report.items.len(), 1, "{:?}", report.items);
    let item = &report.items[0];
    assert_eq!(item.consumer, Some(ConsumerKind::Type));
    assert_eq!(item.operation, XrefOperation::BootstrapMethod);
    assert_eq!(item.derivation, XrefDerivation::BootstrapEdge);
    assert_eq!(item.certainty, XrefCertainty::Exact);
    assert_eq!(item.resolution, QueryResolution::NotRequested);
    assert_eq!(item.evidence.constant_pool_index, Some(fixture.bsm_handle));
    assert_eq!(
        described_entry(&fixture.bytes, item)[0],
        15,
        "the described entry is the CONSTANT_MethodHandle itself"
    );
    let (method, bci) = code_location(item);
    assert_eq!(bci, 0);
    assert_eq!(method.name.0, b"use");
    assert_eq!(
        fixture.bytes[(fixture.code_span.start + u64::from(bci)) as usize],
        0xba,
        "the recorded BCI is inside the instruction array the class really holds"
    );
    assert_eq!(item.evidence.opcode, Some(0xba));
    assert_eq!(item.evidence.bci, Some(0));
    assert_eq!(
        item.evidence.attribute,
        Some(ArchiveNameBytes(b"BootstrapMethods".to_vec()))
    );
    assert_eq!(
        item.evidence.via,
        vec![
            BootstrapVia {
                constant_pool_index: fixture.site,
                bootstrap_index: None,
                argument_index: None,
            },
            BootstrapVia {
                constant_pool_index: fixture.bsm_handle,
                bootstrap_index: Some(0),
                argument_index: None,
            },
        ]
    );

    // A `MethodType` argument declared in two slots: each edge keeps its own path, exactly
    // like the node items of the same table.
    let report = run_complete(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/SharedResult"),
            &[ConsumerKind::Bootstrap, ConsumerKind::Type],
        ),
    );
    assert_eq!(report.items.len(), 2, "{:?}", report.items);
    let slots: Vec<Option<u16>> = report
        .items
        .iter()
        .map(|item| {
            assert_eq!(item.consumer, Some(ConsumerKind::Type));
            assert_eq!(item.operation, XrefOperation::BootstrapArgument);
            assert_eq!(
                item.evidence.constant_pool_index,
                Some(fixture.shared_argument)
            );
            assert_eq!(
                item.evidence.attribute,
                Some(ArchiveNameBytes(b"BootstrapMethods".to_vec()))
            );
            let via = &item.evidence.via;
            assert_eq!(via.len(), 2, "{item:?}");
            assert_eq!(via[0].constant_pool_index, fixture.site);
            assert_eq!(via[1].constant_pool_index, fixture.shared_argument);
            assert_eq!(via[1].bootstrap_index, Some(0));
            assert_eq!(described_entry(&fixture.bytes, item)[0], 16);
            via[1].argument_index
        })
        .collect();
    assert_eq!(
        slots,
        vec![Some(0), Some(1)],
        "one item per argument edge, in declaration order"
    );

    // The parameter position of the same descriptor is the same claim.
    let report = run_complete(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/BsmParam"),
            &[ConsumerKind::Bootstrap, ConsumerKind::Type],
        ),
    );
    assert_eq!(report.items.len(), 1);
    assert_eq!(report.items[0].operation, XrefOperation::BootstrapMethod);
}

#[test]
fn unused_bootstrap_entries_and_type_only_requests_keep_the_table_deferred() {
    let fixture = descriptor_bootstrap_fixture();
    let snapshot = open(fixture.bytes.clone());

    // Types only the unused table entry names produce nothing, and the scan still covers
    // its schema completely.
    for owner in [
        "p/UnusedResult",
        "p/UnusedParam",
        "p/UnusedResult2",
        "p/UnusedParam2",
    ] {
        let report = run_complete(
            &snapshot,
            &request(
                &snapshot,
                QueryRelation::MentionsSymbol,
                class_symbol(owner),
                &[ConsumerKind::Bootstrap, ConsumerKind::Type],
            ),
        );
        assert!(report.items.is_empty(), "{owner}: {:?}", report.items);
        assert_eq!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );
    }

    // The `Type` category is the gate for a type item: `Bootstrap` alone produces the node
    // facts of this table and none of the types its descriptors name.
    let bootstrap_only = run_complete(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/SharedResult"),
            &[ConsumerKind::Bootstrap],
        ),
    );
    assert!(
        bootstrap_only.items.is_empty(),
        "a descriptor type needs the Type category: {:?}",
        bootstrap_only.items
    );

    // `Type` alone opens the graph of a class that really consumes a dynamic entry, and
    // produces the types that only these descriptors name: the `Type` category owns them,
    // so it triggers the reads they need instead of depending on `Bootstrap` being
    // requested too (decision 26). The shared `MethodType` argument is declared in two
    // slots, so each edge carries its own path.
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/SharedResult"),
            &[ConsumerKind::Type],
        ),
        limits(),
    );
    assert!(is_complete(&report));
    assert_eq!(report.items.len(), 2, "{:?}", report.items);
    let slots: Vec<Option<u16>> = report
        .items
        .iter()
        .map(|item| {
            assert_eq!(item.consumer, Some(ConsumerKind::Type), "{item:?}");
            assert_eq!(item.operation, XrefOperation::BootstrapArgument);
            assert_eq!(item.derivation, XrefDerivation::BootstrapEdge);
            assert_eq!(
                item.evidence.constant_pool_index,
                Some(fixture.shared_argument)
            );
            assert_eq!(item.evidence.bci, Some(0));
            assert_eq!(item.evidence.opcode, Some(0xba));
            assert_eq!(
                item.evidence.attribute,
                Some(ArchiveNameBytes(b"BootstrapMethods".to_vec()))
            );
            let via = &item.evidence.via;
            assert_eq!(via.len(), 2, "{item:?}");
            assert_eq!(
                via[0],
                BootstrapVia {
                    constant_pool_index: fixture.site,
                    bootstrap_index: None,
                    argument_index: None,
                }
            );
            assert_eq!(via[1].constant_pool_index, fixture.shared_argument);
            assert_eq!(via[1].bootstrap_index, Some(0));
            via[1].argument_index
        })
        .collect();
    assert_eq!(
        slots,
        vec![Some(0), Some(1)],
        "each edge of a shared argument keeps its own path"
    );
    let type_only = budget.usage().attribute_bytes;
    // The graph is a pass of its own, and it is the reason a `Type`-only request costs
    // more than it did before decision 26 was honoured here: the code stream reads the
    // class and the body for its instructions, the metadata stream reads the class for the
    // hierarchy and the member descriptors, and this stream reads the class, the body for
    // its use-sites and the `BootstrapMethods` content. Each class pass bills every
    // attribute shell, each body read bills the `Code` entry, and the table's content is
    // billed once — the entry charge the reader facts own.
    assert_eq!(
        type_only,
        3 * (fixture.bootstrap_shell + fixture.code_shell)
            + 2 * fixture.code_shell
            + fixture.bootstrap_shell,
        "a Type-only request reads the body for its use-sites and the table's content once"
    );

    // A `Type`-only request states no node fact: the bootstrap method handle and the
    // arguments are `Bootstrap`'s own claim.
    let node_symbol = method_symbol("p/Bsm", "prepare", "(Lp/BsmParam;)Lp/BsmResult;");
    let type_only_node = run_complete(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            node_symbol,
            &[ConsumerKind::Type],
        ),
    );
    assert!(
        type_only_node.items.is_empty(),
        "a node fact belongs to the Bootstrap category: {:?}",
        type_only_node.items
    );

    // With both categories the same type is reported with the node fact of the entry that
    // names it, so the negative control above is about the gate and not about an
    // unreachable fixture.
    let (report, budget) = run_with_limits(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("p/SharedResult"),
            &[ConsumerKind::Bootstrap, ConsumerKind::Type],
        ),
        limits(),
    );
    assert_eq!(report.items.len(), 2, "{:?}", report.items);
    assert_eq!(
        budget.usage().attribute_bytes,
        type_only,
        "the two categories share one graph pass, so asking for both costs what asking \
         for Type alone costs"
    );
    assert_eq!(
        described_entry(&fixture.bytes, &report.items[0])[0],
        16,
        "the described entry is the shared CONSTANT_MethodType"
    );
    assert_eq!(report.items[0].evidence.bci, Some(0));
    assert_eq!(report.items[0].evidence.opcode, Some(0xba));
}

/// The design's B2 counterexample: a real `javac 23.0.1 --release 8` lambda whose
/// `java/lang/invoke/MethodType` occurs *only* inside the bootstrap method handle's
/// descriptor. See `tests/fixtures/b2-bootstrap-descriptor/README.md` for source, command
/// and digest.
const LAMBDA_SAMPLE: &[u8] =
    include_bytes!("fixtures/b2-bootstrap-descriptor/v8/LambdaSample.class");

/// One constant-pool entry of the compiled lambda sample.
///
/// The pool is walked here instead of the numbers being copied from a tool, so every
/// assertion below names entries these bytes really hold.
struct SampleEntry {
    /// 1-based index.
    index: u16,
    tag: u8,
    /// Raw bytes of a `CONSTANT_Utf8`.
    utf8: Option<Vec<u8>>,
    /// `CONSTANT_MethodHandle`: reference kind and referenced entry.
    method_handle: Option<(u8, u16)>,
    /// `CONSTANT_Fieldref`/`Methodref`/`InterfaceMethodref`: class and `NameAndType`.
    member: Option<(u16, u16)>,
    /// `CONSTANT_NameAndType`: name and descriptor.
    name_and_type: Option<(u16, u16)>,
    /// `CONSTANT_InvokeDynamic`: bootstrap entry and `NameAndType`.
    invoke_dynamic: Option<(u16, u16)>,
}

/// Every constant-pool entry of the compiled sample, in index order.
fn sample_pool(bytes: &[u8]) -> Vec<SampleEntry> {
    let count = u16::from_be_bytes([bytes[8], bytes[9]]);
    let mut entries = Vec::new();
    let mut at = 10usize;
    let mut slot = 1u16;
    while slot < count {
        let tag = bytes[at];
        at += 1;
        let mut entry = SampleEntry {
            index: slot,
            tag,
            utf8: None,
            method_handle: None,
            member: None,
            name_and_type: None,
            invoke_dynamic: None,
        };
        let pair = |at: usize| u16::from_be_bytes([bytes[at], bytes[at + 1]]);
        match tag {
            1 => {
                let length = usize::from(pair(at));
                entry.utf8 = Some(bytes[at + 2..at + 2 + length].to_vec());
                at += 2 + length;
            }
            // A `Long`/`Double` occupies two slots, and its second slot has no entry.
            5 | 6 => {
                at += 8;
                slot += 1;
            }
            7 | 8 | 16 | 19 | 20 => at += 2,
            9..=11 => {
                entry.member = Some((pair(at), pair(at + 2)));
                at += 4;
            }
            12 => {
                entry.name_and_type = Some((pair(at), pair(at + 2)));
                at += 4;
            }
            15 => {
                entry.method_handle = Some((bytes[at], pair(at + 1)));
                at += 3;
            }
            17 => at += 4,
            18 => {
                entry.invoke_dynamic = Some((pair(at), pair(at + 2)));
                at += 4;
            }
            3 | 4 => at += 4,
            other => panic!("unexpected constant-pool tag {other} at index {slot}"),
        }
        entries.push(entry);
        slot += 1;
    }
    entries
}

/// One entry of a walked pool by index.
fn sample_entry(pool: &[SampleEntry], index: u16) -> &SampleEntry {
    pool.iter()
        .find(|entry| entry.index == index)
        .expect("the index names an entry")
}

/// Raw bytes of one `CONSTANT_Utf8` entry of a walked pool.
fn sample_utf8(pool: &[SampleEntry], index: u16) -> Vec<u8> {
    sample_entry(pool, index)
        .utf8
        .clone()
        .expect("the index names a Utf8 entry")
}

/// The `CONSTANT_InvokeDynamic` entry of the sample: one class, one dynamic site.
fn sample_site(pool: &[SampleEntry]) -> &SampleEntry {
    let sites: Vec<&SampleEntry> = pool
        .iter()
        .filter(|entry| entry.invoke_dynamic.is_some())
        .collect();
    assert_eq!(sites.len(), 1, "the sample declares one dynamic site");
    sites[0]
}

/// The one bootstrap method handle of the sample whose referenced member's descriptor names
/// `java/lang/invoke/MethodType`.
fn sample_metafactory_handle(pool: &[SampleEntry]) -> u16 {
    let mut found = Vec::new();
    for entry in pool {
        let Some((_, reference)) = entry.method_handle else {
            continue;
        };
        let Some((_, name_and_type)) = sample_entry(pool, reference).member else {
            continue;
        };
        let name_and_type = sample_entry(pool, name_and_type);
        let Some((_, descriptor)) = name_and_type.name_and_type else {
            continue;
        };
        let descriptor = sample_utf8(pool, descriptor);
        if descriptor
            .windows(b"java/lang/invoke/MethodType".len())
            .any(|window| window == b"java/lang/invoke/MethodType")
        {
            found.push(entry.index);
        }
    }
    assert_eq!(
        found.len(),
        1,
        "exactly one handle names a descriptor that mentions the type"
    );
    found[0]
}

#[test]
fn a_type_only_request_reaches_a_type_a_bootstrap_descriptor_names() {
    let snapshot = open(LAMBDA_SAMPLE.to_vec());
    let pool = sample_pool(LAMBDA_SAMPLE);
    let site = sample_site(&pool).index;
    let metafactory_handle = sample_metafactory_handle(&pool);

    // `java/lang/invoke/MethodType` occurs only inside the metafactory descriptor: the
    // sample holds no `Utf8` entry for it on its own (and a `CONSTANT_Class` always names
    // one), so a `Type` hit for it can only come from the descriptor the bootstrap method
    // handle names — no other consumer in the schema can report it.
    assert!(
        !pool
            .iter()
            .any(|entry| entry.utf8.as_deref() == Some(b"java/lang/invoke/MethodType".as_slice())),
        "the sample names the type only inside descriptors"
    );

    // `Type` alone: the request triggers the reads it needs itself, so the graph is opened
    // without `Bootstrap` being requested too.
    let target = class_symbol("java/lang/invoke/MethodType");
    let report = run_complete(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target.clone(),
            &[ConsumerKind::Type],
        ),
    );
    assert_eq!(report.items.len(), 1, "{:?}", report.items);
    let item = &report.items[0];
    assert_eq!(item.consumer, Some(ConsumerKind::Type));
    assert_eq!(item.operation, XrefOperation::BootstrapMethod);
    assert_eq!(item.derivation, XrefDerivation::BootstrapEdge);
    assert_eq!(item.certainty, XrefCertainty::Exact);
    assert_eq!(item.resolution, QueryResolution::NotRequested);
    assert_eq!(
        item.evidence.constant_pool_index,
        Some(metafactory_handle),
        "the described entry is the bootstrap method handle"
    );
    assert_eq!(sample_entry(&pool, metafactory_handle).tag, 15);
    assert_eq!(item.evidence.opcode, Some(0xba));
    assert_eq!(item.evidence.bci, Some(0));
    assert_eq!(
        item.evidence.attribute,
        Some(ArchiveNameBytes(b"BootstrapMethods".to_vec()))
    );
    assert_eq!(
        item.evidence.via,
        vec![
            BootstrapVia {
                constant_pool_index: site,
                bootstrap_index: None,
                argument_index: None,
            },
            BootstrapVia {
                constant_pool_index: metafactory_handle,
                bootstrap_index: Some(0),
                argument_index: None,
            },
        ],
        "a bootstrap fact keeps the path that reached it"
    );
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );

    // Both categories together answer the same type item, and the node's own symbol is the
    // node fact of the same request pair: the two products of one graph node coexist.
    let both = run_complete(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target,
            &[ConsumerKind::Bootstrap, ConsumerKind::Type],
        ),
    );
    assert_eq!(both.items.len(), 1, "{:?}", both.items);
    assert_eq!(both.items[0].evidence, item.evidence);
    let node = run_complete(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            method_symbol("LambdaSample", "lambda$run$0", "()V"),
            &[ConsumerKind::Bootstrap, ConsumerKind::Type],
        ),
    );
    assert_eq!(node.items.len(), 1, "{:?}", node.items);
    assert_eq!(node.items[0].consumer, Some(ConsumerKind::Bootstrap));
    assert_eq!(node.items[0].operation, XrefOperation::BootstrapArgument);
    assert_eq!(
        node.items[0]
            .evidence
            .via
            .last()
            .map(|hop| hop.argument_index),
        Some(Some(1)),
        "the implementation handle is the second static argument"
    );

    // The gate in the other direction: `Bootstrap` alone reports the node facts and not the
    // type a reached descriptor names.
    let bootstrap_only = run_complete(
        &snapshot,
        &request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            class_symbol("java/lang/invoke/MethodType"),
            &[ConsumerKind::Bootstrap],
        ),
    );
    assert!(
        bootstrap_only.items.is_empty(),
        "a descriptor type is the Type category's claim: {:?}",
        bootstrap_only.items
    );
}
