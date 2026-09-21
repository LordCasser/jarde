//! The trusted prepared class view (bulk task 2.1, reader half).
//!
//! What these tests pin, in the order the change states it:
//!
//! * one class read and one member-table walk prepare a class however many of its methods are
//!   decoded, and a body decoded through the prepared view is field-for-field the body the
//!   single-method entry point publishes, charged in the same dimensions and amounts;
//! * the multi-valued locator keeps every ordinal declaring one raw name+descriptor and never elects
//!   a first match;
//! * an absent, a duplicated and an unreadable `Code` entry stay three different facts, with the
//!   error codes the single-method path already answers;
//! * a damaged member table publishes its stop instead of turning an incomplete read into a
//!   per-method verdict, and a damaged declaration stays an error;
//! * the bytes a prepared read hands out are the verified bytes of that entry: a content that
//!   disagrees with its central-directory record is refused before anything is read, and the shared
//!   backing outlives a cleared or zero-capacity facts cache.

use flate2::write::DeflateEncoder;
use jarde::{
    ArtifactInput, ArtifactSnapshot, Budget, ContainerId, ContainerOrigin, Error, FactsCache,
    FactsCapacity, FactsIdentity, Limits, MemberTablePhase, MethodCodeFacts, PhysicalClassLocation,
    PhysicalEntryId, UsageSnapshot, class_facts, method_code_facts,
};
use jarde_reader::classfile::{LocalDebugTable, class_member_facts};
use jarde_reader::prepared::{MethodCodeAttribute, MethodOrdinal, PreparedClass};
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const STORE: u16 = 0;
const DEFLATE: u16 = 8;

/// The class-file fixture with eight methods, three of them with bodies and every one of them a
/// real `javac` body: a class whose methods are all worth decoding.
const SCOPE_FIXTURE: &[u8] = include_bytes!("fixtures/p3-scope/v8/Scope.class");

fn limits() -> Limits {
    Limits {
        input_bytes: 16 * 1024 * 1024,
        archive_entries: 10_000,
        entry_bytes: 16 * 1024 * 1024,
        read_bytes: 16 * 1024 * 1024,
        class_bytes: 16 * 1024 * 1024,
        attribute_bytes: 16 * 1024 * 1024,
        code_bytes: 16 * 1024 * 1024,
        result_items: 10_000,
        output_bytes: 16 * 1024 * 1024,
        nested_depth: 8,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

fn zip(entries: &[(&[u8], &[u8], u16)]) -> Vec<u8> {
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
                let encoder = DeflateEncoder::new(&mut entry, flate2::Compression::default());
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

fn open(bytes: Vec<u8>) -> (ArtifactSnapshot, Budget) {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes), &mut budget).unwrap();
    (snapshot, budget)
}

fn root_origin(snapshot: &ArtifactSnapshot) -> ContainerOrigin {
    ContainerOrigin {
        snapshot: snapshot.id().clone(),
        root_container: ContainerId("root".into()),
        steps: Vec::new(),
    }
}

/// What one read charged, in the four dimensions a prepared decode may move.
///
/// `AttributeBytes` and `CodeBytes` are the two a body decode bills; `ClassBytes` and `ResultItems`
/// are the two it must not, and they are in the tuple so that "the same as the single-method path"
/// and "not one class read or one published item per body" are one assertion rather than two.
fn charged(before: &UsageSnapshot, after: &UsageSnapshot) -> (u64, u64, u64, u64) {
    (
        after.attribute_bytes - before.attribute_bytes,
        after.code_bytes - before.code_bytes,
        after.class_bytes - before.class_bytes,
        after.result_items - before.result_items,
    )
}

/// One body read's evidence, with the running usage taken out of its `execution`.
///
/// Two reads of one body under two budgets cannot hold the same `UsageSnapshot` — the usage is the
/// total of the request each read ran in — so "the prepared path and the single-method path publish
/// the same facts" is every field of [`MethodCodeFacts`] *and* the shape of its `execution`, which
/// is what this value is. The execution's usage is compared by the charge assertion instead, where
/// it means something: the two reads must move the same dimensions by the same amounts.
#[derive(Debug, Eq, PartialEq)]
struct BodyEvidence<'a> {
    max_stack: u16,
    max_locals: u16,
    code_span: jarde::ByteSpan,
    instructions: &'a [jarde::InstructionFact],
    operands: &'a [jarde::InstructionOperands],
    exception_handlers: &'a [jarde::ExceptionHandlerFact],
    exception_handler_count: u32,
    stopped_at: Option<&'a jarde::BytecodeStop>,
    debug: &'a LocalDebugTable,
    execution: ExecutionShape,
}

#[derive(Debug, Eq, PartialEq)]
enum ExecutionShape {
    Complete,
    Partial(jarde::TerminationReason),
    Cancelled,
    Failed(jarde::TerminationReason),
}

fn body_evidence(facts: &MethodCodeFacts) -> BodyEvidence<'_> {
    BodyEvidence {
        max_stack: facts.max_stack,
        max_locals: facts.max_locals,
        code_span: facts.code_span.clone(),
        instructions: &facts.instructions,
        operands: facts.operands(),
        exception_handlers: &facts.exception_handlers,
        exception_handler_count: facts.exception_handler_count,
        stopped_at: facts.stopped_at.as_ref(),
        debug: facts.debug(),
        execution: match &facts.execution {
            jarde::ExecutionReport::Complete { .. } => ExecutionShape::Complete,
            jarde::ExecutionReport::Partial { reason, .. } => {
                ExecutionShape::Partial(reason.clone())
            }
            jarde::ExecutionReport::Cancelled { .. } => ExecutionShape::Cancelled,
            jarde::ExecutionReport::Failed { reason, .. } => ExecutionShape::Failed(reason.clone()),
        },
    }
}

/// The stable code of one structured refusal.
fn refusal_code<T: std::fmt::Debug>(result: jarde::Result<T>) -> String {
    match result {
        Ok(value) => panic!("expected a refusal, got {value:?}"),
        Err(Error::InvalidInput { code, .. }) | Err(Error::Unsupported { code, .. }) => code,
        Err(other) => panic!("expected a structured refusal, got {other}"),
    }
}

// -----------------------------------------------------------------------------------------------
// A minimal class-file assembler for the fixtures this file needs
// -----------------------------------------------------------------------------------------------

/// One constant-pool entry, already encoded with its tag.
struct ClassFixture {
    entries: Vec<Vec<u8>>,
}

impl ClassFixture {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// The 1-based index of one entry just appended.
    fn push(&mut self, entry: Vec<u8>) -> u16 {
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("fixture constant pool fits u16")
    }

    fn utf8(&mut self, value: &[u8]) -> u16 {
        let mut entry = vec![1];
        entry.extend_from_slice(
            &u16::try_from(value.len())
                .expect("fixture name fits u16")
                .to_be_bytes(),
        );
        entry.extend_from_slice(value);
        self.push(entry)
    }

    fn class(&mut self, name: u16) -> u16 {
        let mut entry = vec![7];
        entry.extend_from_slice(&name.to_be_bytes());
        self.push(entry)
    }
}

/// One method record of an assembled class.
struct MethodFixture {
    access_flags: u16,
    name: u16,
    descriptor: u16,
    attributes: Vec<AttributeFixture>,
}

/// One attribute entry of an assembled class.
enum AttributeFixture {
    /// A well-formed `Code` entry holding these instruction bytes.
    Code {
        code: Vec<u8>,
        max_stack: u16,
        max_locals: u16,
    },
}

/// The class file one fixture describes: version 52, no interfaces, no fields, no class attributes.
fn assemble(
    fixture: &ClassFixture,
    code_name: u16,
    this_class: u16,
    super_class: u16,
    methods: &[MethodFixture],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes());
    out.extend_from_slice(&52_u16.to_be_bytes());
    out.extend_from_slice(
        &(u16::try_from(fixture.entries.len()).expect("fixture pool fits u16") + 1).to_be_bytes(),
    );
    for entry in &fixture.entries {
        out.extend_from_slice(entry);
    }
    out.extend_from_slice(&0x0021_u16.to_be_bytes());
    out.extend_from_slice(&this_class.to_be_bytes());
    out.extend_from_slice(&super_class.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes()); // interfaces
    out.extend_from_slice(&0_u16.to_be_bytes()); // fields
    out.extend_from_slice(
        &u16::try_from(methods.len())
            .expect("fixture methods fit u16")
            .to_be_bytes(),
    );
    for method in methods {
        out.extend_from_slice(&method.access_flags.to_be_bytes());
        out.extend_from_slice(&method.name.to_be_bytes());
        out.extend_from_slice(&method.descriptor.to_be_bytes());
        out.extend_from_slice(
            &u16::try_from(method.attributes.len())
                .expect("fixture attributes fit u16")
                .to_be_bytes(),
        );
        for attribute in &method.attributes {
            let AttributeFixture::Code {
                code,
                max_stack,
                max_locals,
            } = attribute;
            let mut content = Vec::new();
            content.extend_from_slice(&max_stack.to_be_bytes());
            content.extend_from_slice(&max_locals.to_be_bytes());
            content.extend_from_slice(
                &u32::try_from(code.len())
                    .expect("fixture code fits u32")
                    .to_be_bytes(),
            );
            content.extend_from_slice(code);
            content.extend_from_slice(&0_u16.to_be_bytes()); // exception table
            content.extend_from_slice(&0_u16.to_be_bytes()); // nested attributes
            out.extend_from_slice(&code_name.to_be_bytes());
            out.extend_from_slice(
                &u32::try_from(content.len())
                    .expect("fixture attribute fits u32")
                    .to_be_bytes(),
            );
            out.extend_from_slice(&content);
        }
    }
    out.extend_from_slice(&0_u16.to_be_bytes()); // class attributes
    out
}

/// One method per name, each with a one-instruction body (`return`), plus one abstract member.
fn class_with_bodies(count: usize) -> Vec<u8> {
    let mut fixture = ClassFixture::new();
    let this_name = fixture.utf8(b"Fixture");
    let this = fixture.class(this_name);
    let object = fixture.utf8(b"java/lang/Object");
    let super_class = fixture.class(object);
    let code_name = fixture.utf8(b"Code");
    let mut methods = Vec::new();
    for index in 0..count {
        let name = fixture.utf8(format!("m{index}").as_bytes());
        let descriptor = fixture.utf8(b"()V");
        methods.push(MethodFixture {
            access_flags: 0x0001,
            name,
            descriptor,
            attributes: vec![AttributeFixture::Code {
                code: vec![0xb1],
                max_stack: 1,
                max_locals: 1,
            }],
        });
    }
    assemble(&fixture, code_name, this, super_class, &methods)
}

// -----------------------------------------------------------------------------------------------
// One class read, one decoder
// -----------------------------------------------------------------------------------------------

#[test]
fn a_real_fixture_class_decodes_every_method_like_the_single_method_path() {
    let bytes = SCOPE_FIXTURE;
    let mut class_budget = Budget::new(limits());
    let facts = class_facts(bytes, &mut class_budget).unwrap();
    assert_eq!(
        facts.methods.len(),
        8,
        "the fixture really has eight methods"
    );

    let (snapshot, mut budget) = open(bytes.to_vec());
    let read = snapshot.prepared_root_class(&mut budget).unwrap();
    assert_eq!(
        read.location,
        PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone()
        }
    );
    assert_eq!(read.bytes(), bytes);
    assert_eq!(read.span.length, bytes.len() as u64);
    assert_eq!(read.class_bytes.length, bytes.len() as u64);
    assert_eq!(read.depth, 0);
    // The snapshot identity is the digest of exactly these bytes, so the standing digest is the
    // class's own and nothing is hashed again.
    assert_eq!(read.backing_digest().0, snapshot.id().0);

    let prepared = PreparedClass::prepare(&read, &mut budget).unwrap();
    assert_eq!(prepared.member_table_stop(), None);
    assert_eq!(prepared.method_count(), 8);
    assert_eq!(prepared.method_slots().len(), 8);
    assert_eq!(prepared.slot(MethodOrdinal(8)), None);
    assert_eq!(
        prepared.parse_policy(),
        jarde_reader::prepared::ParsePolicy::Structure
    );
    let after_prepare = budget.usage();
    // One class read: the whole class was billed once, by the preparation.
    assert_eq!(after_prepare.class_bytes, bytes.len() as u64);

    for (index, header) in facts.methods.iter().enumerate() {
        let ordinal = MethodOrdinal(u32::try_from(index).unwrap());
        let slot = prepared.slot(ordinal).unwrap();
        assert_eq!(&slot.header, header);
        assert_eq!(&slot.name, &header.name);
        assert_eq!(slot.access_flags, header.access_flags);
        assert_eq!(
            prepared.locate_method(
                header.name.raw().0.as_slice(),
                header.descriptor.raw().0.as_slice()
            ),
            std::slice::from_ref(&ordinal),
            "ordinal {index} is the only record with that raw name and descriptor"
        );

        let before = budget.usage();
        let through_prepared = prepared.method_code(ordinal, &mut budget).unwrap();
        let after = budget.usage();
        let mut single = Budget::new(limits());
        let single_before = single.usage();
        let through_single = method_code_facts(bytes, header, &mut single).unwrap();
        assert_eq!(
            body_evidence(&through_prepared),
            body_evidence(&through_single),
            "body {index} differs"
        );
        assert!(through_prepared.operands().len() == through_prepared.instructions.len());
        assert_eq!(
            charged(&before, &after),
            charged(&single_before, &single.usage()),
            "body {index} is charged differently"
        );
        assert_eq!(
            charged(&before, &after).2,
            0,
            "a body decode never bills ClassBytes"
        );
        assert_eq!(
            charged(&before, &after).3,
            0,
            "a body decode never bills ResultItems"
        );
    }
    // Every method was decoded, and the class was still read exactly once: N bodies cost one class
    // read, not N of them.
    assert_eq!(budget.usage().class_bytes, bytes.len() as u64);
    assert_eq!(budget.usage().archive_entries, 0);
}

#[test]
fn one_class_read_is_enough_for_every_method_of_the_class() {
    let bytes = SCOPE_FIXTURE;
    // A budget that can pay for exactly one class read and for no published item at all. A second
    // class read, or one `ResultItems` charge per member, would make every assertion below fail.
    let tight = Limits {
        class_bytes: bytes.len() as u64,
        result_items: 0,
        ..limits()
    };
    let (snapshot, mut snapshot_budget) = open(bytes.to_vec());
    let read = snapshot.prepared_root_class(&mut snapshot_budget).unwrap();
    let mut budget = Budget::new(tight);
    let prepared = PreparedClass::prepare(&read, &mut budget).unwrap();
    for index in 0..prepared.method_slots().len() {
        let ordinal = MethodOrdinal(u32::try_from(index).unwrap());
        prepared.method_code(ordinal, &mut budget).unwrap();
    }
    assert_eq!(budget.usage().class_bytes, bytes.len() as u64);
    assert_eq!(budget.usage().result_items, 0);
}

#[test]
fn stored_and_deflated_entries_both_decode_like_the_single_method_path() {
    let hand_written = class_with_bodies(3);
    let archive = zip(&[
        (b"p/Stored.class", &hand_written, STORE),
        (b"p/Deflated.class", SCOPE_FIXTURE, DEFLATE),
        (b"p/resource.txt", b"not a class", STORE),
    ]);
    let (snapshot, mut budget) = open(archive.clone());
    let listing = snapshot.enumerate(&mut budget).unwrap();
    assert_eq!(listing.entries.len(), 3);

    for record in &listing.entries {
        if !record.id.raw_name.0.ends_with(b".class") {
            continue;
        }
        let bytes = snapshot
            .read_entry_for_analysis(record, &mut budget)
            .unwrap()
            .bytes;
        let mut class_budget = Budget::new(limits());
        let facts = class_facts(&bytes, &mut class_budget).unwrap();
        let read = snapshot.prepared_class(&record.id, &mut budget).unwrap();
        assert_eq!(
            read.location,
            PhysicalClassLocation::ArchiveEntry {
                entry: record.id.clone()
            }
        );
        assert_eq!(read.container, ContainerId("root".into()));
        assert_eq!(read.depth, 0);
        assert_eq!(read.bytes(), bytes.as_slice());
        assert_eq!(read.class_bytes.length, bytes.len() as u64);
        // Whether the entry was stored in the container or had to be produced, the digest the read
        // publishes is the trusted identity of exactly these bytes.
        assert_eq!(
            read.class_bytes.digest,
            jarde::Digest(blake3::hash(&bytes).to_hex().to_string())
        );
        assert_eq!(read.backing_digest().0, snapshot.id().0);

        let prepared = PreparedClass::prepare(&read, &mut budget).unwrap();
        assert_eq!(prepared.method_slots().len(), facts.methods.len());
        for (index, header) in facts.methods.iter().enumerate() {
            let ordinal = MethodOrdinal(u32::try_from(index).unwrap());
            assert_eq!(
                body_evidence(&prepared.method_code(ordinal, &mut budget).unwrap()),
                body_evidence(
                    &method_code_facts(&bytes, header, &mut Budget::new(limits())).unwrap()
                )
            );
        }
        if record.id.raw_name.0 == b"p/Stored.class" {
            // A stored entry lives in the container the read already holds: the backing is the
            // whole ZIP and the class bytes are a range inside it, so nothing was copied.
            assert_eq!(read.backing.len(), archive.len());
            assert_eq!(read.span.start + read.span.length, record_span_end(record));
        } else {
            // A deflated entry had to be produced, so the read hands back exactly those bytes as
            // its own backing.
            assert_eq!(read.backing.len(), bytes.len());
            assert_eq!(read.span.start, 0);
        }
    }
}

/// The end of a stored entry's compressed range, as the record states it.
fn record_span_end(record: &jarde::PhysicalEntry) -> u64 {
    record.layout.compressed_data.start + record.layout.compressed_data.length
}

#[test]
fn the_locator_keeps_every_duplicate_declaration_and_decodes_none_of_them() {
    // Two methods with one raw name and descriptor, one with another descriptor: the locator keys
    // on both fields, and two declarations of one signature stay two.
    let mut fixture = ClassFixture::new();
    let this_name = fixture.utf8(b"Fixture");
    let this = fixture.class(this_name);
    let object = fixture.utf8(b"java/lang/Object");
    let super_class = fixture.class(object);
    let code_name = fixture.utf8(b"Code");
    let name = fixture.utf8(b"x");
    let void = fixture.utf8(b"()V");
    let ints = fixture.utf8(b"()I");
    let method = |name: u16, descriptor: u16, code: Vec<u8>| MethodFixture {
        access_flags: 0x0001,
        name,
        descriptor,
        attributes: vec![AttributeFixture::Code {
            code,
            max_stack: 1,
            max_locals: 1,
        }],
    };
    let methods = vec![
        method(name, void, vec![0xb1]),
        method(name, void, vec![0xb1]),
        method(name, ints, vec![0x10, 0x2a, 0xac]),
    ];
    let bytes = assemble(&fixture, code_name, this, super_class, &methods);

    let mut class_budget = Budget::new(limits());
    let facts = class_facts(&bytes, &mut class_budget).unwrap();
    let (snapshot, mut budget) = open(bytes.clone());
    let read = snapshot.prepared_root_class(&mut budget).unwrap();
    let prepared = PreparedClass::prepare(&read, &mut budget).unwrap();

    assert_eq!(
        prepared.locate_method(b"x", b"()V"),
        &[MethodOrdinal(0), MethodOrdinal(1)]
    );
    assert_eq!(prepared.locate_method(b"x", b"()I"), &[MethodOrdinal(2)]);
    assert!(prepared.locate_method(b"y", b"()V").is_empty());
    // A duplicate declaration is never decoded as one of the two: the ordinals are all reported and
    // neither gets a body, which is the same answer the single-method path gives for that class.
    assert_eq!(
        refusal_code(prepared.method_code(MethodOrdinal(0), &mut budget)),
        "classfile_method_ambiguous"
    );
    assert_eq!(
        refusal_code(prepared.method_code(MethodOrdinal(1), &mut budget)),
        "classfile_method_ambiguous"
    );
    assert_eq!(
        refusal_code(method_code_facts(
            &bytes,
            &facts.methods[0],
            &mut Budget::new(limits())
        )),
        "classfile_method_ambiguous"
    );
    // The third record is unique, so it decodes exactly as the single-method path does.
    assert_eq!(
        body_evidence(&prepared.method_code(MethodOrdinal(2), &mut budget).unwrap()),
        body_evidence(
            &method_code_facts(&bytes, &facts.methods[2], &mut Budget::new(limits())).unwrap()
        )
    );
    assert_eq!(
        refusal_code(prepared.method_code(MethodOrdinal(9), &mut budget)),
        "classfile_method_not_found"
    );
}

#[test]
fn absent_duplicate_and_single_code_entries_keep_the_codes_of_the_single_method_path() {
    let mut fixture = ClassFixture::new();
    let this_name = fixture.utf8(b"Fixture");
    let this = fixture.class(this_name);
    let object = fixture.utf8(b"java/lang/Object");
    let super_class = fixture.class(object);
    let code_name = fixture.utf8(b"Code");
    let body = fixture.utf8(b"body");
    let abstract_name = fixture.utf8(b"abstractMember");
    let twice = fixture.utf8(b"twice");
    let descriptor = fixture.utf8(b"()V");
    let one_code = AttributeFixture::Code {
        code: vec![0xb1],
        max_stack: 1,
        max_locals: 1,
    };
    let methods = vec![
        MethodFixture {
            access_flags: 0x0001,
            name: body,
            descriptor,
            attributes: vec![AttributeFixture::Code {
                code: vec![0xb1],
                max_stack: 1,
                max_locals: 1,
            }],
        },
        // An abstract member: no `Code` entry at all, which is a declaration shape and not damage.
        MethodFixture {
            access_flags: 0x0401,
            name: abstract_name,
            descriptor,
            attributes: Vec::new(),
        },
        MethodFixture {
            access_flags: 0x0001,
            name: twice,
            descriptor,
            attributes: vec![
                AttributeFixture::Code {
                    code: vec![0xb1],
                    max_stack: 1,
                    max_locals: 1,
                },
                one_code,
            ],
        },
    ];
    let bytes = assemble(&fixture, code_name, this, super_class, &methods);

    let mut class_budget = Budget::new(limits());
    let facts = class_facts(&bytes, &mut class_budget).unwrap();
    assert_eq!(facts.methods.len(), 3);
    let (snapshot, mut budget) = open(bytes.clone());
    let read = snapshot.prepared_root_class(&mut budget).unwrap();
    let prepared = PreparedClass::prepare(&read, &mut budget).unwrap();

    match &prepared.slot(MethodOrdinal(0)).unwrap().code {
        MethodCodeAttribute::Single(shell) => {
            assert_eq!(shell.name.raw().0.as_slice(), b"Code");
        }
        other => panic!("the first record declares one Code entry, got {other:?}"),
    }
    assert_eq!(
        prepared.slot(MethodOrdinal(1)).unwrap().code,
        MethodCodeAttribute::Absent
    );
    assert_eq!(
        prepared.slot(MethodOrdinal(2)).unwrap().code,
        MethodCodeAttribute::Duplicate
    );
    assert_eq!(
        refusal_code(prepared.method_code(MethodOrdinal(1), &mut budget)),
        "classfile_method_has_no_code"
    );
    assert_eq!(
        refusal_code(prepared.method_code(MethodOrdinal(2), &mut budget)),
        "classfile_duplicate_code_attribute"
    );
    // And the same two codes are what the single-method path answers for those very records.
    assert_eq!(
        refusal_code(method_code_facts(
            &bytes,
            &facts.methods[1],
            &mut Budget::new(limits())
        )),
        "classfile_method_has_no_code"
    );
    assert_eq!(
        refusal_code(method_code_facts(
            &bytes,
            &facts.methods[2],
            &mut Budget::new(limits())
        )),
        "classfile_duplicate_code_attribute"
    );
    assert_eq!(
        body_evidence(&prepared.method_code(MethodOrdinal(0), &mut budget).unwrap()),
        body_evidence(
            &method_code_facts(&bytes, &facts.methods[0], &mut Budget::new(limits())).unwrap()
        )
    );
}

// -----------------------------------------------------------------------------------------------
// Stops, refusals and lifetimes
// -----------------------------------------------------------------------------------------------

#[test]
fn a_damaged_member_table_publishes_the_stop_and_no_body_verdict() {
    let full = class_with_bodies(2);
    // The last bytes of the file belong to the second method's `Code` entry, so cutting them leaves
    // that record's own name and descriptor readable while its attribute list cannot be read.
    let truncated = full[..full.len() - 4].to_vec();

    let mut budget = Budget::new(limits());
    let mut reference = Budget::new(limits());
    let member_stop = class_member_facts(&truncated, &mut reference)
        .unwrap()
        .stopped_at
        .expect("the member walk stops inside the second record");

    let (snapshot, mut read_budget) = open(truncated);
    let read = snapshot
        .prepared_root_class(&mut read_budget)
        .expect("a damaged member table is not a damaged declaration");
    let prepared = match PreparedClass::prepare(&read, &mut budget) {
        Ok(prepared) => prepared,
        Err(error) => panic!("a damaged member table must publish a stop, got {error}"),
    };

    let stop = prepared
        .member_table_stop()
        .expect("the stop is published on the class, not silently truncated");
    assert_eq!(stop.phase, MemberTablePhase::Methods);
    assert_eq!(stop.index, 1);
    assert_eq!(stop.code, member_stop.code);
    assert!(stop.class_offset > 0);
    assert_eq!(prepared.method_count(), 2, "the class declares two methods");
    assert_eq!(prepared.method_slots().len(), 1, "one record could be read");
    // An incomplete table is never turned into a body verdict: the record it did read states the
    // stop rather than `Absent` or a decodable `Code` entry.
    assert_eq!(
        prepared.slot(MethodOrdinal(0)).unwrap().code,
        MethodCodeAttribute::Unreadable(stop.clone())
    );
    assert_eq!(
        refusal_code(prepared.method_code(MethodOrdinal(0), &mut budget)),
        stop.code
    );
    // The record before the stop is still locatable; the record the walk never reached is not, and
    // the class's declared count is what says the table is incomplete.
    assert_eq!(prepared.locate_method(b"m0", b"()V"), &[MethodOrdinal(0)]);
    assert!(prepared.locate_method(b"m1", b"()V").is_empty());
    assert!(prepared.method_count() > prepared.method_slots().len() as u64);
}

#[test]
fn a_damaged_class_declaration_is_an_error() {
    let full = class_with_bodies(2);
    // Cutting inside the constant pool leaves no declaration to publish.
    let truncated = full[..12].to_vec();
    let (snapshot, mut budget) = open(truncated);
    let read = snapshot.prepared_root_class(&mut budget).unwrap();
    assert!(
        PreparedClass::prepare(&read, &mut budget).is_err(),
        "a class whose own declaration does not decode has no prefix to publish"
    );
}

#[test]
fn a_content_that_disagrees_with_its_record_is_refused_before_any_byte() {
    let hand_written = class_with_bodies(2);
    let mut archive = zip(&[(b"p/Stored.class", &hand_written, STORE)]);
    // Flip one byte of the stored class content: the central directory still records the CRC of the
    // bytes that were written, so the entry no longer agrees with its own record.
    let position = archive
        .windows(hand_written.len())
        .position(|window| window == hand_written.as_slice())
        .expect("the stored class is in the archive verbatim");
    archive[position + 12] ^= 0xff;

    let (snapshot, mut budget) = open(archive);
    let listing = snapshot.enumerate(&mut budget).unwrap();
    let record = listing.entries[0].clone();
    assert_eq!(
        refusal_code(snapshot.prepared_class(&record.id, &mut budget)),
        "entry_integrity"
    );
    // The materializing read refuses the same bytes for the same reason: the two paths keep one
    // record cross-check and one verified content read.
    assert_eq!(
        refusal_code(snapshot.read_entry_for_analysis(&record, &mut budget)),
        "entry_integrity"
    );

    // An ordinal whose raw name is not the one this directory records is refused as a locator
    // mismatch, and an ordinal the container does not have is not found at all.
    let mut wrong_name = record.id.clone();
    wrong_name.raw_name = jarde::ArchiveNameBytes(b"p/Other.class".to_vec());
    assert_eq!(
        refusal_code(snapshot.prepared_class(&wrong_name, &mut budget)),
        "entry_locator_mismatch"
    );
    let mut absent = record.id.clone();
    absent.ordinal = 9;
    assert_eq!(
        refusal_code(snapshot.prepared_class(&absent, &mut budget)),
        "entry_not_found"
    );
}

#[test]
fn a_prepared_read_survives_a_cleared_or_zero_capacity_cache() {
    let archive = zip(&[(b"p/Stored.class", &class_with_bodies(2), STORE)]);
    let (snapshot, mut budget) = open(archive);
    let listing = snapshot.enumerate(&mut budget).unwrap();
    let entry: PhysicalEntryId = listing.entries[0].id.clone();

    // A store with room, then cleared under the class task's feet.
    let cache = FactsCache::new(
        FactsIdentity::current(),
        FactsCapacity::new(4, 4 * 1024 * 1024),
    );
    let mut budget = Budget::new(limits()).with_facts_cache(cache.clone());
    let read = snapshot.prepared_class(&entry, &mut budget).unwrap();
    let expected = read.bytes().to_vec();
    cache.clear();
    assert_eq!(read.bytes(), expected.as_slice());
    let prepared = PreparedClass::prepare(&read, &mut budget).unwrap();
    assert_eq!(
        body_evidence(&prepared.method_code(MethodOrdinal(0), &mut budget).unwrap()),
        body_evidence(
            &method_code_facts(
                &expected,
                &class_facts(&expected, &mut Budget::new(limits()))
                    .unwrap()
                    .methods[0],
                &mut Budget::new(limits())
            )
            .unwrap()
        )
    );

    // And a store that can retain nothing at all: the read still hands out the same bytes.
    let zero = FactsCache::new(FactsIdentity::current(), FactsCapacity::new(0, 0));
    let mut budget = Budget::new(limits()).with_facts_cache(zero);
    let read = snapshot.prepared_class(&entry, &mut budget).unwrap();
    assert_eq!(read.bytes(), expected.as_slice());
    assert!(PreparedClass::prepare(&read, &mut budget).is_ok());
}

#[test]
fn container_backing_hands_out_the_container_that_was_read() {
    let inner = zip(&[(b"p/Inner.class", &class_with_bodies(1), STORE)]);
    let outer = zip(&[
        (b"BOOT-INF/classes/App.class", &class_with_bodies(1), STORE),
        (b"BOOT-INF/lib/inner.jar", &inner, STORE),
    ]);
    let (snapshot, mut budget) = open(outer.clone());

    // The root container's backing is the snapshot's own bytes, handed over as the same value.
    let root = snapshot
        .container_backing(&root_origin(&snapshot), &mut budget)
        .unwrap();
    assert_eq!(root.as_ref(), outer.as_slice());

    // A nested container's backing is the bytes its parent entry verified, so reading that entry
    // and asking for the container's backing agree. The child container's own origin is the one the
    // artifact-tree walk derived for it, which is what a caller holds after a tree walk.
    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();
    assert_eq!(tree.containers.len(), 2);
    let listing = snapshot.enumerate(&mut budget).unwrap();
    let nested = listing.entries[1].clone();
    let inner_bytes = snapshot
        .read_entry_for_analysis(&nested, &mut budget)
        .unwrap()
        .bytes;
    let child_origin = tree.containers[1].origin.clone();
    assert_eq!(child_origin.steps.len(), 1);
    let child = snapshot
        .container_backing(&child_origin, &mut budget)
        .unwrap();
    assert_eq!(child.as_ref(), inner_bytes.as_slice());

    // An origin of another snapshot is refused rather than read.
    let mut foreign = nested.id.origin.clone();
    foreign.snapshot = jarde::SnapshotId("0".repeat(64));
    assert_eq!(
        refusal_code(snapshot.container_backing(&foreign, &mut budget)),
        "entry_snapshot_mismatch"
    );
}

#[test]
fn a_nested_class_is_read_through_its_own_container_chain() {
    let inner = zip(&[(b"p/Inner.class", SCOPE_FIXTURE, STORE)]);
    let outer = zip(&[(b"BOOT-INF/lib/inner.jar", &inner, STORE)]);
    let (snapshot, mut budget) = open(outer);
    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();
    assert_eq!(tree.containers.len(), 2);
    let nested = tree.containers[1].entries[0].id.clone();
    assert_eq!(nested.origin.steps.len(), 1);

    let read = snapshot.prepared_class(&nested, &mut budget).unwrap();
    assert_eq!(read.depth, 1);
    assert_eq!(read.bytes(), SCOPE_FIXTURE);
    // The nested container's backing is the archive its parent entry was verified to hold, and the
    // class bytes are a range inside it — the digest is the one that materializing read produced.
    assert_eq!(read.backing.len(), inner.len());
    assert_eq!(
        read.backing_digest().0,
        blake3::hash(&inner).to_hex().to_string()
    );
    assert_eq!(read.class_bytes.length, SCOPE_FIXTURE.len() as u64);

    let prepared = PreparedClass::prepare(&read, &mut budget).unwrap();
    assert_eq!(prepared.method_slots().len(), 8);
    let facts = class_facts(SCOPE_FIXTURE, &mut Budget::new(limits())).unwrap();
    assert_eq!(
        body_evidence(&prepared.method_code(MethodOrdinal(0), &mut budget).unwrap()),
        body_evidence(
            &method_code_facts(SCOPE_FIXTURE, &facts.methods[0], &mut Budget::new(limits()))
                .unwrap()
        )
    );
}

/// The identity a prepared class publishes is the one a caller may bind facts to.
#[test]
fn a_prepared_class_publishes_the_reads_identity() {
    let bytes = class_with_bodies(2);
    let (snapshot, mut budget) = open(bytes.clone());
    let read = snapshot.prepared_root_class(&mut budget).unwrap();
    let prepared = PreparedClass::prepare(&read, &mut budget).unwrap();
    assert_eq!(prepared.location(), &read.location);
    assert_eq!(prepared.class_bytes(), &read.class_bytes);
    assert_eq!(prepared.class_bytes().length, bytes.len() as u64);
    assert_eq!(
        prepared.class_bytes().digest,
        jarde::Digest(blake3::hash(&bytes).to_hex().to_string())
    );
    assert_eq!(prepared.class_facts().methods.len(), 2);
    assert_eq!(prepared.backing_digest().0, read.backing_digest().0);
}
