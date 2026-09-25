//! The class-source presentation: one class file assembled into Java text, and the facts that text
//! was spelled from.
//!
//! The contract this file holds has three halves, and each is checked against the library's own
//! values rather than against the text alone:
//!
//! * **one class, one read, one run per member.** The declaration and the member tables are the
//!   class view's own read, and every member body is one single-method recovery — so the report
//!   publishes the very `RecoveryReport` of that run, and a member that declares no body charges no
//!   run at all.
//! * **nothing is disguised.** A member with no `Code`, a member whose run stopped, a member whose
//!   artifact holds no statement and a member whose descriptor cannot be read are each marked in the
//!   text with the same `// jarde:` prefix the report publishes per member, and no empty body is ever
//!   written for a body that was not recovered.
//! * **a member's failure is that member's.** The members beside a stopped one are presented, the
//!   class report's execution plane is non-`Complete`, and a name that several definitions answer to
//!   presents nothing at all.
//!
//! The committed real compiled sample is read as it is, and crafted probes are built here so a
//! case can pin the exact member shapes it is about.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const STORE: u16 = 0;

/// The real compiled sample: three members, one of them explanation-only.
const HISTORICAL: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");
const BRIDGE_API: &[u8] =
    include_bytes!("fixtures/p3-bridge-projection/positive/v8/BridgeApi.class");
const BRIDGE_PROBE: &[u8] =
    include_bytes!("fixtures/p3-bridge-projection/positive/v8/BridgeProbe.class");
const BRIDGE_API_SOURCE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-23/bridge-source-projection/positive/BridgeApi.java"
);
const BRIDGE_RUNNER_SOURCE: &str =
    include_str!("fixtures/p3-bridge-projection/positive/BridgeRunner.java");
const FAKE_BRIDGE: &[u8] =
    include_bytes!("fixtures/p3-bridge-projection/negative/v8/FakeBridge.class");
const ORPHAN_BRIDGE: &[u8] =
    include_bytes!("fixtures/p3-bridge-projection/orphan/v8/OrphanBridge.class");
const CLASS_RETENTION_TARGET: &[u8] =
    include_bytes!("fixtures/class-annotation-uses/v8/HiddenTarget.class");
const EMPTY_ANNOTATION_TARGET: &[u8] =
    include_bytes!("fixtures/class-annotation-uses/v8/EmptyTarget.class");
const RUNTIME_VISIBLE_TARGET: &[u8] =
    include_bytes!("fixtures/class-annotation-uses/v8/VisibleTarget.class");
const ANNOTATION_TYPE: &[u8] = include_bytes!("fixtures/class-annotation-uses/v8/HiddenTag.class");
const NESTED_ARRAY_TARGET: &[u8] =
    include_bytes!("fixtures/class-annotation-uses/v8/DuplicateTarget.class");
const MIXED_RETENTION_TARGET: &[u8] =
    include_bytes!("fixtures/class-annotation-uses/v8/MixedTarget.class");
const MEMBER_PLACEMENT_TARGET: &[u8] =
    include_bytes!("fixtures/class-annotation-uses/v8/MemberPlacementTarget.class");
const MEMBER_ANNOTATION_TARGET: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/member-annotation-uses/generated/original/MemberTagged.class"
);
const MEMBER_BOUNDARY_TARGET: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/member-annotation-uses/boundaries/generated/legal/BoundaryTagged.class"
);
const MEMBER_BOUNDARY_DUPLICATE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/member-annotation-uses/boundaries/generated/patched/duplicate-same-position/BoundaryTagged.class"
);
const MEMBER_BOUNDARY_COUNT_MISMATCH: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/member-annotation-uses/boundaries/generated/patched/parameter-count-mismatch/BoundaryTagged.class"
);
const TYPE_USE_TARGET: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/type-use-annotations/generated/original/TypeUseSubject.class"
);
const PRIMITIVE_TYPE_USE_TARGET: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/type-use-annotations/primitive-boundaries/build/patched/ScalarCases.class"
);
const INVISIBLE_TYPE_USE_TARGET: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/type-use-annotations/generated/invisible/HiddenTypeUse.class"
);
const POSITIONED_TYPE_USE_TARGET: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/type-use-annotations/generated/positioned/PositionedTypeUse.class"
);
const DUAL_TARGET_TYPE_USE_TARGET: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/type-use-annotations/placement-boundaries/generated/classes/PlacementSubject.class"
);
const UNSUPPORTED_TYPE_USE_TARGET: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/type-use-annotations/generated/unsupported/UnsupportedTypeUse.class"
);
const ENUM_SWITCH_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/enum-switch-labels/enum-switch.jar"
);
const ENUM_SWITCH_SWAPPED_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/enum-switch-labels/enum-switch-swapped.jar"
);
const ENUM_SWITCH_ALIASED_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/enum-switch-labels/negative/aliased-enum/aliased-enum.jar"
);
const ENUM_SWITCH_FACTORY_NULL_ELEMENT_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/enum-switch-labels/negative/enum-values-array/factory-null-element.jar"
);
const ENUM_SWITCH_VALUES_RETURNS_NULL_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/enum-switch-labels/negative/enum-values-array/values-returns-null.jar"
);

// ---------------------------------------------------------------------------------------------
// Fixtures: one class-file builder and one stored-only archive writer
// ---------------------------------------------------------------------------------------------

/// One constant pool that interns each entry once, so a fixture's indices are stable.
#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    fn intern(&mut self, entry: Vec<u8>) -> u16 {
        if let Some(index) = self.entries.iter().position(|existing| *existing == entry) {
            return u16::try_from(index + 1).expect("the fixture pool index fits u16");
        }
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("the fixture pool index fits u16")
    }

    fn utf8(&mut self, text: &[u8]) -> u16 {
        let mut entry = vec![1];
        u16b(
            &mut entry,
            u16::try_from(text.len()).expect("the fixture text fits u16"),
        );
        entry.extend_from_slice(text);
        self.intern(entry)
    }

    fn class(&mut self, name: &[u8]) -> u16 {
        let name = self.utf8(name);
        let mut entry = vec![7];
        u16b(&mut entry, name);
        self.intern(entry)
    }
}

#[test]
fn enum_switch_projection_follows_proved_mapping_and_refuses_aliased_enum_fields() {
    let normal = enum_switch_class_source(ENUM_SWITCH_JAR);
    assert_eq!(normal.enum_switch_proofs.len(), 1);
    assert!(
        normal.enum_switch_proofs[0].projected,
        "{:?}",
        normal.enum_switch_proofs[0]
    );
    assert!(normal.text.contains("switch (arg0)"));
    assert!(normal.text.contains("case RED:"));
    assert!(normal.text.contains("case BLUE:"));

    let swapped = enum_switch_class_source(ENUM_SWITCH_SWAPPED_JAR);
    assert_eq!(swapped.enum_switch_proofs.len(), 1);
    assert!(swapped.enum_switch_proofs[0].projected);
    assert!(
        swapped
            .text
            .contains("case BLUE:\n                return mark(1);")
    );
    assert!(
        swapped
            .text
            .contains("case RED:\n                return mark(2);")
    );

    let aliased = enum_switch_class_source(ENUM_SWITCH_ALIASED_JAR);
    assert_eq!(aliased.enum_switch_proofs.len(), 1);
    assert!(!aliased.enum_switch_proofs[0].projected);
    assert!(
        aliased.enum_switch_proofs[0]
            .refusal
            .as_deref()
            .is_some_and(|reason| reason.contains("enum <clinit> contains instructions outside")),
        "{:?}",
        aliased.enum_switch_proofs[0]
    );
    assert!(aliased.text.contains("$SwitchMap$Hue[arg0.ordinal()]"));
}

#[test]
fn enum_switch_projection_refuses_irregular_factory_and_public_values() {
    for (jar, expected) in [
        (ENUM_SWITCH_FACTORY_NULL_ELEMENT_JAR, "factory"),
        (ENUM_SWITCH_VALUES_RETURNS_NULL_JAR, "public values()"),
    ] {
        let report = enum_switch_class_source(jar);
        assert_eq!(report.enum_switch_proofs.len(), 1);
        assert!(!report.enum_switch_proofs[0].projected);
        assert!(
            report.enum_switch_proofs[0]
                .refusal
                .as_deref()
                .is_some_and(|reason| reason.contains(expected)),
            "{report:?}"
        );
        assert!(report.text.contains("$SwitchMap$Hue[arg0.ordinal()]"));
    }
}

fn u16b(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn u32b(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes());
}

/// One field of a fixture class.
struct FieldSpec<'a> {
    flags: u16,
    name: &'a [u8],
    descriptor: &'a [u8],
}

/// One method of a fixture class: its flags, its name and descriptor, and the instructions of its
/// `Code` attribute — or `None` for a member that declares no `Code` at all.
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
    interfaces: &[&[u8]],
    flags: u16,
    fields: &[FieldSpec<'_>],
    methods: &[MethodSpec<'_>],
) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_index = pool.class(this_class);
    let super_index = pool.class(super_class);
    let interface_indices: Vec<u16> = interfaces.iter().map(|name| pool.class(name)).collect();
    let code_name = pool.utf8(b"Code");
    let field_indices: Vec<(u16, u16)> = fields
        .iter()
        .map(|field| (pool.utf8(field.name), pool.utf8(field.descriptor)))
        .collect();
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
    u16b(
        &mut output,
        u16::try_from(interface_indices.len()).expect("the fixture interfaces fit u16"),
    );
    for index in interface_indices {
        u16b(&mut output, index);
    }
    u16b(
        &mut output,
        u16::try_from(fields.len()).expect("the fixture fields fit u16"),
    );
    for (index, field) in fields.iter().enumerate() {
        u16b(&mut output, field.flags);
        u16b(&mut output, field_indices[index].0);
        u16b(&mut output, field_indices[index].1);
        u16b(&mut output, 0);
    }
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

/// `iconst_1; dup; iadd; ireturn`: a valid value flow whose duplicate is not a verified chained
/// assignment. It remains explanation-only while still allowing the class and method executions to
/// complete.
const DUPLICATE_RESULT_BODY: &[u8] = &[0x04, 0x59, 0x60, 0xac];

fn duplicate_result_class() -> Vec<u8> {
    class_file(
        b"p/DuplicateResult",
        b"java/lang/Object",
        &[],
        CLASS_FLAGS,
        &[],
        &[MethodSpec {
            flags: PUBLIC_STATIC_METHOD,
            name: b"duplicate",
            descriptor: b"()I",
            code: Some(DUPLICATE_RESULT_BODY),
        }],
    )
}

/// `new` with an incomplete index: a body whose own decode stops inside it, which is what makes the
/// analysis of that member non-`Complete`.
const DAMAGED_BODY: &[u8] = &[0xbb, 0x00];

const CLASS_FLAGS: u16 = 0x0021;
const PUBLIC_METHOD: u16 = 0x0001;
const PUBLIC_STATIC_METHOD: u16 = 0x0009;
const PUBLIC_ABSTRACT_METHOD: u16 = 0x0401;
const PUBLIC_NATIVE_METHOD: u16 = 0x0101;
const PUBLIC_STATIC_FINAL_FIELD: u16 = 0x0019;

/// One class with every member shape this presentation has to state: bodies it recovers, a body
/// whose analysis stops, a third body behind them (so a request that ends early leaves a member it
/// never began), a member with no `Code` for each of the three ways that can be said, a field, and
/// two interfaces.
fn probe_class() -> Vec<u8> {
    class_file(
        b"p/Probe",
        b"java/lang/Object",
        &[b"p/Marker", b"java/io/Serializable"],
        CLASS_FLAGS,
        &[FieldSpec {
            flags: PUBLIC_STATIC_FINAL_FIELD,
            name: b"value",
            descriptor: b"I",
        }],
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
                flags: PUBLIC_STATIC_METHOD,
                name: b"third",
                descriptor: b"()V",
                code: Some(PLAIN_BODY),
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

/// One class file whose field table declares a field and then stops inside its record: the class
/// itself is readable, the members behind the stop are not, and the report has to state that rather
/// than present a shorter class as a whole one.
fn truncated_member_table_class() -> Vec<u8> {
    let mut pool = Pool::default();
    let this_index = pool.class(b"p/Truncated");
    let super_index = pool.class(b"java/lang/Object");
    let name = pool.utf8(b"value");
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
    u16b(&mut output, CLASS_FLAGS);
    u16b(&mut output, this_index);
    u16b(&mut output, super_index);
    u16b(&mut output, 0);
    u16b(&mut output, 1);
    u16b(&mut output, PUBLIC_STATIC_FINAL_FIELD);
    u16b(&mut output, name);
    output
}

/// One class with `bodies` members that declare a body and one that declares none, so a case can
/// compare two classes whose member counts differ and whose bodies are all recoverable.
///
/// The no-body member is the `abstract` shape this presentation states as a declaration: it is what
/// makes "one preparation + one decode per body" a statement about bodies rather than about records.
fn many_bodies_class(name: &[u8], bodies: usize) -> Vec<u8> {
    let names: Vec<String> = (0..bodies).map(|index| format!("body{index}")).collect();
    let mut methods: Vec<MethodSpec<'_>> = names
        .iter()
        .map(|member| MethodSpec {
            flags: PUBLIC_STATIC_METHOD,
            name: member.as_bytes(),
            descriptor: b"()V",
            code: Some(PLAIN_BODY),
        })
        .collect();
    methods.push(MethodSpec {
        flags: PUBLIC_ABSTRACT_METHOD,
        name: b"declaredOnly",
        descriptor: b"()V",
        code: None,
    });
    class_file(name, b"java/lang/Object", &[], CLASS_FLAGS, &[], &methods)
}

/// One class whose **method** table declares two records and stops inside the second: the first
/// member is a complete record with a valid body, the second is a name and a flags field with no
/// descriptor after them.
///
/// This is the shape a damaged member table has when there is a readable body in front of it, which
/// is what a presentation owes an honest answer about: the prefix member is a member this class
/// declares, and "its body could not be decoded" and "this class declares no such member" are two
/// different statements.
fn stopped_method_table_class() -> Vec<u8> {
    let mut pool = Pool::default();
    let this_index = pool.class(b"p/Stopped");
    let super_index = pool.class(b"java/lang/Object");
    let code_name = pool.utf8(b"Code");
    let good_name = pool.utf8(b"good");
    let descriptor = pool.utf8(b"()V");
    let second_name = pool.utf8(b"second");
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
    u16b(&mut output, CLASS_FLAGS);
    u16b(&mut output, this_index);
    u16b(&mut output, super_index);
    u16b(&mut output, 0); // interfaces
    u16b(&mut output, 0); // fields
    u16b(&mut output, 2); // two method records declared
    // methods[0]: a complete record with a `Code` attribute whose body is a real statement.
    let mut body = Vec::new();
    u16b(&mut body, 2);
    u16b(&mut body, 4);
    u32b(
        &mut body,
        u32::try_from(PLAIN_BODY.len()).expect("the fixture body fits u32"),
    );
    body.extend_from_slice(PLAIN_BODY);
    u16b(&mut body, 0);
    u16b(&mut body, 0);
    u16b(&mut output, PUBLIC_STATIC_METHOD);
    u16b(&mut output, good_name);
    u16b(&mut output, descriptor);
    u16b(&mut output, 1);
    u16b(&mut output, code_name);
    u32b(
        &mut output,
        u32::try_from(body.len()).expect("the fixture attribute fits u32"),
    );
    output.extend_from_slice(&body);
    // methods[1]: flags and a name, and then the file ends.
    u16b(&mut output, PUBLIC_STATIC_METHOD);
    u16b(&mut output, second_name);
    output
}

/// A stored-only archive, built with the repository's own `rawzip` dev-dependency.
fn zip_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .expect("the fixture entry starts");
            let mut writer = config.wrap(&mut entry);
            writer
                .write_all(data)
                .expect("the fixture entry is written");
            let (_, descriptor) = writer.finish().expect("the fixture entry closes");
            entry
                .finish(descriptor)
                .expect("the fixture entry finishes");
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

struct TestClassAttribute {
    name: Vec<u8>,
    length_offset: usize,
    data_offset: usize,
    length: usize,
}

struct TestMethodHeader {
    name: Vec<u8>,
    descriptor: Vec<u8>,
    access_offset: usize,
    attribute_count_offset: usize,
    end: usize,
    attributes: Vec<TestClassAttribute>,
}

fn test_u16(bytes: &[u8], offset: usize) -> usize {
    usize::from(u16::from_be_bytes([bytes[offset], bytes[offset + 1]]))
}

fn test_u32(bytes: &[u8], offset: usize) -> usize {
    usize::try_from(u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]))
    .expect("the fixture attribute length fits usize")
}

fn test_put_u16(bytes: &mut [u8], offset: usize, value: usize) {
    let value = u16::try_from(value).expect("the fixture count fits u16");
    bytes[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
}

fn test_put_u32(bytes: &mut [u8], offset: usize, value: usize) {
    let value = u32::try_from(value).expect("the fixture attribute length fits u32");
    bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

fn test_pool(bytes: &[u8]) -> (usize, Vec<Vec<u8>>) {
    let count = test_u16(bytes, 8);
    let mut entries = vec![Vec::new(); count];
    let mut cursor = 10;
    let mut index = 1;
    while index < count {
        let tag = bytes[cursor];
        cursor += 1;
        let width = match tag {
            1 => {
                let length = test_u16(bytes, cursor);
                cursor += 2;
                entries[index] = bytes[cursor..cursor + length].to_vec();
                length
            }
            3 | 4 => 4,
            5 | 6 => 8,
            7 | 8 | 16 | 19 | 20 => 2,
            9 | 10 | 11 | 12 | 17 | 18 => 4,
            15 => 3,
            other => panic!("unexpected constant-pool tag {other}"),
        };
        cursor += width;
        index += if tag == 5 || tag == 6 { 2 } else { 1 };
    }
    (cursor, entries)
}

fn test_method_headers(bytes: &[u8]) -> Vec<TestMethodHeader> {
    let (pool_end, pool) = test_pool(bytes);
    let mut cursor = pool_end + 6;
    let interface_count = test_u16(bytes, cursor);
    cursor += 2 + interface_count * 2;
    let field_count = test_u16(bytes, cursor);
    cursor += 2;
    for _ in 0..field_count {
        let attribute_count = test_u16(bytes, cursor + 6);
        cursor += 8;
        for _ in 0..attribute_count {
            let length = test_u32(bytes, cursor + 2);
            cursor += 6 + length;
        }
    }
    let method_count = test_u16(bytes, cursor);
    cursor += 2;
    let mut methods = Vec::with_capacity(method_count);
    for _ in 0..method_count {
        let start = cursor;
        let name_index = test_u16(bytes, cursor + 2);
        let descriptor_index = test_u16(bytes, cursor + 4);
        let attribute_count_offset = cursor + 6;
        let attribute_count = test_u16(bytes, attribute_count_offset);
        cursor += 8;
        let mut attributes = Vec::with_capacity(attribute_count);
        for _ in 0..attribute_count {
            let attribute_name = test_u16(bytes, cursor);
            let length_offset = cursor + 2;
            let length = test_u32(bytes, length_offset);
            attributes.push(TestClassAttribute {
                name: pool[attribute_name].clone(),
                length_offset,
                data_offset: cursor + 6,
                length,
            });
            cursor += 6 + length;
        }
        methods.push(TestMethodHeader {
            name: pool[name_index].clone(),
            descriptor: pool[descriptor_index].clone(),
            access_offset: start,
            attribute_count_offset,
            end: cursor,
            attributes,
        });
    }
    methods
}

fn patch_method_flags(bytes: &[u8], name: &[u8], descriptor: &[u8], flags: u16) -> Vec<u8> {
    let mut patched = bytes.to_vec();
    let method = test_method_headers(bytes)
        .into_iter()
        .find(|method| method.name.as_slice() == name && method.descriptor.as_slice() == descriptor)
        .expect("the patch method exists");
    patched[method.access_offset..method.access_offset + 2].copy_from_slice(&flags.to_be_bytes());
    patched
}

fn add_deprecated_method_attribute(bytes: &[u8], name: &[u8], descriptor: &[u8]) -> Vec<u8> {
    let (pool_end, _) = test_pool(bytes);
    let old_pool_count = test_u16(bytes, 8);
    let mut patched = bytes.to_vec();
    test_put_u16(&mut patched, 8, old_pool_count + 1);
    let mut utf8 = vec![1];
    u16b(&mut utf8, 10);
    utf8.extend_from_slice(b"Deprecated");
    patched.splice(pool_end..pool_end, utf8);
    let method = test_method_headers(&patched)
        .into_iter()
        .find(|method| method.name.as_slice() == name && method.descriptor.as_slice() == descriptor)
        .expect("the patch method exists");
    let count = test_u16(&patched, method.attribute_count_offset);
    test_put_u16(&mut patched, method.attribute_count_offset, count + 1);
    let mut attribute = Vec::new();
    u16b(&mut attribute, u16::try_from(old_pool_count).unwrap());
    u32b(&mut attribute, 0);
    patched.splice(method.end..method.end, attribute);
    patched
}

fn add_bridge_handler(bytes: &[u8]) -> Vec<u8> {
    let mut patched = bytes.to_vec();
    let method = test_method_headers(bytes)
        .into_iter()
        .find(|method| {
            method.name.as_slice() == b"get"
                && method.descriptor.as_slice() == b"()Ljava/lang/Object;"
        })
        .expect("the bridge method exists");
    let code = method
        .attributes
        .iter()
        .find(|attribute| attribute.name.as_slice() == b"Code")
        .expect("the bridge declares Code");
    let code_length = test_u32(bytes, code.data_offset + 4);
    let handler_count_offset = code.data_offset + 8 + code_length;
    assert_eq!(test_u16(bytes, handler_count_offset), 0);
    let nested_count_offset = handler_count_offset + 2;
    let mut handler = Vec::new();
    u16b(&mut handler, 0);
    u16b(&mut handler, u16::try_from(code_length).unwrap());
    u16b(&mut handler, 0);
    u16b(&mut handler, 0);
    patched.splice(nested_count_offset..nested_count_offset, handler);
    test_put_u16(&mut patched, handler_count_offset, 1);
    test_put_u32(&mut patched, code.length_offset, code.length + 8);
    patched
}

// ---------------------------------------------------------------------------------------------
// The library side of every case
// ---------------------------------------------------------------------------------------------

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

/// The code one request-level refusal carries.
fn error_code(error: &Error) -> String {
    match error {
        Error::InvalidInput { code, .. } | Error::Unsupported { code, .. } => code.clone(),
        other => panic!("unexpected error {other:?}"),
    }
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget())
        .expect("the fixture snapshot opens")
}

fn performed<T>(outcome: OperationOutcome<T>) -> T {
    match outcome {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "expected one bound class, got {} candidate(s) and no execution",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "expected one bound class, got an unfinished selection with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

/// One class-source request over `snapshot`, under an explicit policy.
fn request(
    snapshot: &ArtifactSnapshot,
    class: ClassRef,
    policy: EnvironmentPolicy,
) -> ClassSourceRequest {
    ClassSourceRequest {
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
    }
}

/// The same request over an explicit scope rather than the whole snapshot.
fn scoped_request(
    snapshot: &ArtifactSnapshot,
    class: ClassRef,
    policy: EnvironmentPolicy,
    scope: PhysicalScope,
) -> ClassSourceRequest {
    let mut request = request(snapshot, class, policy);
    request.environment.scope = scope;
    request
}

/// The usage one execution plane carries, whichever terminal state it states.
fn usage_of(execution: &ExecutionReport) -> &UsageSnapshot {
    match execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage,
    }
}

/// The container origin of one snapshot's own root container, as the artifact-tree enumeration
/// states it: a snapshot's root container is the snapshot itself.
fn root_origin(snapshot: &ArtifactSnapshot) -> ContainerOrigin {
    ContainerOrigin {
        snapshot: snapshot.id().clone(),
        root_container: ContainerId("root".into()),
        steps: Vec::new(),
    }
}

/// The declared position of one nested container, addressed by the leaf entry that reaches it: the
/// origin comes from the public artifact-tree enumeration, which is the way a caller learns one.
fn nested_root(snapshot: &ArtifactSnapshot, leaf: &[u8]) -> LoadRoot {
    let tree = Engine::new()
        .enumerate_artifact_tree(snapshot, &mut budget())
        .expect("the fixture tree enumerates");
    let origin = tree
        .containers
        .iter()
        .filter(|container| {
            container
                .origin
                .steps
                .last()
                .is_some_and(|step| step.via_raw_name.0 == leaf)
        })
        .map(|container| container.origin.clone())
        .next()
        .unwrap_or_else(|| {
            panic!(
                "the fixture names exactly one container with leaf `{}`",
                String::from_utf8_lossy(leaf)
            )
        });
    LoadRoot::Container {
        origin,
        prefix: ArchiveNameBytes(Vec::new()),
    }
}

/// The class-source of one name in one snapshot, under the whole task budget.
fn class_source_of(
    snapshot: &ArtifactSnapshot,
    name: &str,
    policy: EnvironmentPolicy,
) -> ClassSourceReport {
    performed(
        Engine::new()
            .class_source(
                slice::from_ref(snapshot),
                &request(
                    snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal(name),
                    },
                    policy,
                ),
                &mut budget(),
            )
            .expect("a legal class-source request is answered"),
    )
}

fn bridge_class_source(
    snapshot: &ArtifactSnapshot,
    name: &str,
    policy: EnvironmentPolicy,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    performed(
        Engine::new()
            .class_source_with_evidence(
                slice::from_ref(snapshot),
                &request(
                    snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal(name),
                    },
                    policy,
                ),
                evidence,
                &mut budget(),
            )
            .expect("the bridge class-source request is legal"),
    )
}

fn enum_switch_class_source(bytes: &[u8]) -> ClassSourceReport {
    let snapshot = open(bytes.to_vec());
    class_source_of(&snapshot, "EnumSwitchSubject", EnvironmentPolicy::PlainJar)
}

struct BridgeProjectionScratch(PathBuf);

impl BridgeProjectionScratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-bridge-source-projection-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the Java comparison directory");
        Self(path)
    }

    fn child(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(&path).expect("create a Java comparison case directory");
        path
    }
}

impl Drop for BridgeProjectionScratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn compile_bridge_runner(directory: &Path, source_files: &[&str]) {
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-classpath"])
        .arg(directory)
        .arg("-d")
        .arg(directory)
        .args(source_files.iter().map(|name| directory.join(name)))
        .output()
        .expect("JDK javac is available for the bridge projection regression");
    assert!(
        compile.status.success(),
        "javac rejected the complete Java 8 source:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
}

fn run_bridge_runner(directory: &Path) -> String {
    let run = Command::new("java")
        .args(["-Xverify:all", "-classpath"])
        .arg(directory)
        .arg("BridgeRunner")
        .output()
        .expect("JDK java is available for the bridge projection regression");
    assert!(
        run.status.success(),
        "the bridge runner failed JVM verification or execution:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the bridge trace is UTF-8")
}

#[test]
fn qualified_type_annotations_are_spelled_inside_their_member_types() {
    let report = class_source_of(
        &open(TYPE_USE_TARGET.to_vec()),
        "TypeUseSubject",
        EnvironmentPolicy::SingleClass,
    );
    let field = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"field")
        .unwrap();
    assert!(
        field
            .declaration
            .as_deref()
            .unwrap()
            .contains("java.lang.@TypeMark(value = \"field\") String field")
    );
    assert_eq!(
        field.type_annotations.field_uses,
        ["@TypeMark(value = \"field\")"]
    );
    let returned = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"value")
        .unwrap();
    assert!(
        returned
            .declaration
            .as_deref()
            .unwrap()
            .contains("java.lang.@TypeMark(value = \"return\") String value()")
    );
    assert_eq!(
        returned.type_annotations.return_uses,
        ["@TypeMark(value = \"return\")"]
    );
    let parameter = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"echo")
        .unwrap();
    assert!(
        parameter
            .declaration
            .as_deref()
            .unwrap()
            .contains("java.lang.@TypeMark(value = \"parameter\") String arg1")
    );
    assert_eq!(
        parameter.type_annotations.parameter_uses,
        [vec!["@TypeMark(value = \"parameter\")".to_owned()]]
    );
}

#[test]
fn primitive_type_annotations_keep_their_facts_and_are_refused() {
    let report = class_source_of(
        &open(PRIMITIVE_TYPE_USE_TARGET.to_vec()),
        "ScalarCases",
        EnvironmentPolicy::SingleClass,
    );
    let field = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"field")
        .unwrap();
    assert!(field.type_annotations.field_uses.is_empty());
    assert!(
        field
            .type_annotations
            .refusals
            .iter()
            .any(|reason| reason.contains("primitive"))
    );
    let answer = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"answer")
        .unwrap();
    assert!(answer.type_annotations.return_uses.is_empty());
    assert!(
        answer
            .type_annotations
            .refusals
            .iter()
            .any(|reason| reason.contains("primitive"))
    );
    let echo = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"echo")
        .unwrap();
    assert!(echo.type_annotations.parameter_uses[0].is_empty());
    assert!(
        echo.type_annotations
            .refusals
            .iter()
            .any(|reason| reason.contains("primitive"))
    );
}

#[test]
fn invisible_type_annotations_keep_their_shell_and_value() {
    let report = class_source_of(
        &open(INVISIBLE_TYPE_USE_TARGET.to_vec()),
        "HiddenTypeUse",
        EnvironmentPolicy::SingleClass,
    );
    let field = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"field")
        .unwrap();
    assert!(
        field
            .declaration
            .as_deref()
            .unwrap()
            .contains("java.lang.@HiddenMark(value = \"field\") String field")
    );
    assert_eq!(
        field.type_annotations.attributes[0].attribute.name.raw().0,
        b"RuntimeInvisibleTypeAnnotations"
    );
    assert_eq!(
        field.type_annotations.field_uses,
        ["@HiddenMark(value = \"field\")"]
    );
}

#[test]
fn same_type_annotations_on_method_and_return_keep_their_separate_facts() {
    let report = class_source_of(
        &open(DUAL_TARGET_TYPE_USE_TARGET.to_vec()),
        "PlacementSubject",
        EnvironmentPolicy::SingleClass,
    );
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"scalarMethod")
        .unwrap();
    assert!(
        method
            .annotations
            .uses
            .iter()
            .any(|annotation| annotation.starts_with("@PlaceMark"))
    );
    assert!(method.type_annotations.return_uses.is_empty());
    assert!(
        method
            .type_annotations
            .refusals
            .iter()
            .any(|reason| reason.contains("declaration and type target"))
    );
    assert_eq!(
        method.type_annotations.attributes[0].annotations[0].target_type,
        0x14
    );
    assert!(
        method.parameter_annotations.uses_by_position[0]
            .iter()
            .any(|annotation| annotation.starts_with("@PlaceMark"))
    );
    assert!(method.type_annotations.parameter_uses[0].is_empty());
    assert!(
        method
            .type_annotations
            .refusals
            .iter()
            .any(|reason| reason.contains("target=0x16")
                && reason.contains("declaration and type target"))
    );
    assert_eq!(
        method.type_annotations.parameter_uses[1],
        ["@PlaceMark(value = \"parameter-qualified\")"]
    );
}

#[test]
fn formal_parameter_target_uses_descriptor_position_after_a_wide_slot() {
    let report = class_source_of(
        &open(POSITIONED_TYPE_USE_TARGET.to_vec()),
        "PositionedTypeUse",
        EnvironmentPolicy::SingleClass,
    );
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"accept")
        .unwrap();
    assert!(method.declaration.as_deref().unwrap().contains(
        "java.lang.String arg2, java.lang.@TypeMark(value = \"position two\") String arg3"
    ));
    let target = &method.type_annotations.attributes[0].annotations[0];
    assert_eq!(target.target_type, 0x16);
    assert_eq!(target.target_info, [2]);
    assert_eq!(
        method.type_annotations.parameter_uses[0],
        Vec::<String>::new()
    );
    assert_eq!(
        method.type_annotations.parameter_uses[1],
        Vec::<String>::new()
    );
    assert_eq!(
        method.type_annotations.parameter_uses[2],
        ["@TypeMark(value = \"position two\")"]
    );
}

fn duplicate_field_type_annotation(class_bytes: &[u8]) -> Vec<u8> {
    let mut budget = budget();
    let class = class_facts(class_bytes, &mut budget).unwrap();
    let shell = class.fields[0]
        .attributes
        .iter()
        .find(|shell| shell.name.raw().0 == b"RuntimeVisibleTypeAnnotations")
        .expect("field type annotation shell");
    let start = usize::try_from(shell.content_span.start).unwrap();
    let end = start + usize::try_from(shell.content_span.length).unwrap();
    let content = &class_bytes[start..end];
    assert_eq!(&content[..2], &[0, 1]);
    let mut replacement = vec![0, 2];
    replacement.extend_from_slice(&content[2..]);
    replacement.extend_from_slice(&content[2..]);
    let mut bytes = class_bytes.to_vec();
    bytes.splice(start..end, replacement.iter().copied());
    let length = u32::try_from(replacement.len()).unwrap();
    let header = usize::try_from(shell.span.start).unwrap() + 2;
    bytes[header..header + 4].copy_from_slice(&length.to_be_bytes());
    bytes
}

fn out_of_range_parameter_target(class_bytes: &[u8]) -> Vec<u8> {
    let mut budget = budget();
    let class = class_facts(class_bytes, &mut budget).unwrap();
    let method = class
        .methods
        .iter()
        .find(|method| method.name.raw().0 == b"echo")
        .unwrap();
    let shell = method
        .attributes
        .iter()
        .find(|shell| shell.name.raw().0 == b"RuntimeVisibleTypeAnnotations")
        .expect("parameter type annotation shell");
    let mut bytes = class_bytes.to_vec();
    let target_index = usize::try_from(shell.content_span.start).unwrap() + 3;
    bytes[target_index] = u8::MAX;
    bytes
}

fn field_target_owned_by_method(class_bytes: &[u8]) -> Vec<u8> {
    let mut budget = budget();
    let class = class_facts(class_bytes, &mut budget).unwrap();
    let shell = class.fields[0]
        .attributes
        .iter()
        .find(|shell| shell.name.raw().0 == b"RuntimeVisibleTypeAnnotations")
        .expect("field type annotation shell");
    let mut bytes = class_bytes.to_vec();
    let target_type = usize::try_from(shell.content_span.start).unwrap() + 2;
    bytes[target_type] = 0x14;
    bytes
}

#[test]
fn duplicate_type_annotation_is_refused_atomically_and_other_targets_survive() {
    let report = class_source_of(
        &open(duplicate_field_type_annotation(TYPE_USE_TARGET)),
        "TypeUseSubject",
        EnvironmentPolicy::SingleClass,
    );
    let field = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"field")
        .unwrap();
    assert_eq!(field.type_annotations.attributes[0].annotations.len(), 2);
    assert!(field.type_annotations.field_uses.is_empty());
    assert!(
        field
            .type_annotations
            .refusals
            .iter()
            .filter(|reason| reason.contains("duplicate annotation type"))
            .count()
            == 2
    );
    let returned = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"value")
        .unwrap();
    assert_eq!(returned.type_annotations.return_uses.len(), 1);
}

#[test]
fn formal_parameter_target_past_descriptor_count_is_refused_without_shift() {
    let report = class_source_of(
        &open(out_of_range_parameter_target(TYPE_USE_TARGET)),
        "TypeUseSubject",
        EnvironmentPolicy::SingleClass,
    );
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"echo")
        .unwrap();
    assert!(method.type_annotations.parameter_uses[0].is_empty());
    assert!(
        method
            .type_annotations
            .refusals
            .iter()
            .any(|reason| reason.contains("outside the descriptor parameter count"))
    );
}

#[test]
fn field_type_attribute_cannot_claim_a_method_return_target() {
    let report = class_source_of(
        &open(field_target_owned_by_method(TYPE_USE_TARGET)),
        "TypeUseSubject",
        EnvironmentPolicy::SingleClass,
    );
    let field = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"field")
        .unwrap();
    assert!(field.type_annotations.field_uses.is_empty());
    assert!(
        field
            .type_annotations
            .refusals
            .iter()
            .any(|reason| reason.contains("does not belong to a field"))
    );
    let returned = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"value")
        .unwrap();
    assert_eq!(returned.type_annotations.return_uses.len(), 1);
}

#[test]
fn single_name_array_and_nested_path_type_uses_are_preserved_as_refusals() {
    let report = class_source_of(
        &open(UNSUPPORTED_TYPE_USE_TARGET.to_vec()),
        "UnsupportedTypeUse",
        EnvironmentPolicy::SingleClass,
    );
    let single = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"peer")
        .unwrap();
    assert!(single.type_annotations.field_uses.is_empty());
    assert!(
        single
            .type_annotations
            .refusals
            .iter()
            .any(|reason| reason.contains("single-segment"))
    );
    let array = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"array")
        .unwrap();
    assert!(array.type_annotations.field_uses.is_empty());
    assert!(
        array
            .type_annotations
            .refusals
            .iter()
            .any(|reason| reason.contains("primitive, array"))
    );
    let generic = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"values")
        .unwrap();
    assert!(generic.type_annotations.field_uses.is_empty());
    assert!(
        generic
            .type_annotations
            .refusals
            .iter()
            .any(|reason| reason.contains("non-empty type_path"))
    );
}

fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    let item = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in {report:?}"));
    &item.text
}

/// Every marker the text carries, in the order they appear, without the leading indentation.
fn markers(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("// jarde:"))
        .collect()
}

/// The one-line marker a member's own text carries, when it carries exactly one.
fn only_marker(text: &str) -> String {
    let found = markers(text);
    assert_eq!(found.len(), 1, "expected one marker in:\n{text}");
    found[0].to_owned()
}

/// Whether every line of the text starts at a multiple of four columns.
fn indentation_is_four_spaces(text: &str) -> bool {
    text.lines().all(|line| {
        let leading = line.len() - line.trim_start_matches(' ').len();
        leading % 4 == 0
    })
}

/// The brace balance of the text, ignoring comment lines (a fallback's reason may quote anything).
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
            "a closing brace with nothing open before it in:\n{text}"
        );
    }
    balance
}

// ---------------------------------------------------------------------------------------------
// One class, one read, one run per member
// ---------------------------------------------------------------------------------------------

/// A real compiled class is presented whole: the declaration, the fields and the members in the
/// class file's own order, each with the report of its own run.
#[test]
fn one_real_class_is_presented_with_its_declaration_and_its_bodies() {
    let snapshot = open(HISTORICAL.to_vec());
    let mut budget = budget();
    let report = performed(
        Engine::new()
            .class_source(
                slice::from_ref(&snapshot),
                &request(
                    &snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::dotted("HistoricalControlFlow"),
                    },
                    EnvironmentPolicy::SingleClass,
                ),
                &mut budget,
            )
            .expect("the fixture is a legal request"),
    );

    // The declaration is the read's own item, and the text opens with the line spelled from it.
    let declaration = report.declaration.as_ref().expect("the class is declared");
    assert_eq!(declaration.name, "HistoricalControlFlow");
    assert_eq!(
        declaration.item.definition, report.class,
        "the item is the definition the report names"
    );
    assert_eq!(
        declaration.declaration,
        "public class HistoricalControlFlow extends java.lang.Object"
    );
    assert!(
        report.text.starts_with(
            "// jarde: presentation of `HistoricalControlFlow` from the class file's own \
             declaration and one recovery run per member.\n"
        ),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("public class HistoricalControlFlow extends java.lang.Object {\n"),
        "{}",
        report.text
    );

    // Every member of the class's own table is published, in table order, with its own run.
    let names: Vec<String> = report
        .methods
        .iter()
        .map(|method| method.item.name.escaped())
        .collect();
    assert_eq!(names, ["<init>", "add", "finallyPath"]);
    for method in &report.methods {
        let ClassSourceOutcome::Recovered { report, analysis } = &method.outcome else {
            panic!("every member of this class declares a body: {method:?}");
        };
        // The two halves are of one run, and each is that run's own plane: the analysis finished here
        // (the sample's bodies are all analyzed), and what the recovery layer delivered is stated by
        // the content plane — `contains_statements` for two members and `explanation_only` for the
        // third (see `an_explanation_only_member_is_marked_and_its_text_is_kept`).
        assert!(matches!(
            analysis.execution,
            ExecutionReport::Complete { .. }
        ));
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        assert!(report.produced());
    }
    // The one body's statement is in the text, under the declaration it belongs to.
    assert!(
        report
            .text
            .contains("    public int add(int arg1, int arg2) {\n"),
        "{}",
        report.text
    );
    assert!(report.text.contains("        return arg1 + arg2;\n"));
    assert!(braces_balance(&report.text) == 0, "{}", report.text);
    assert!(indentation_is_four_spaces(&report.text), "{}", report.text);
    assert!(report.text.ends_with("}\n"));

    // One class header — the binding read, which is also the read the one preparation is built
    // over (D2 3.2: the selected definition is materialized once, never re-read for the
    // preparation) — and one body attempt per member: the member runs decode against that
    // preparation and charge no class read of their own (see
    // `one_preparation_serves_every_member_body`).
    assert_eq!(report.usage.class_headers, 1, "{:?}", report.usage);
    assert_eq!(report.usage.method_bodies, 3, "{:?}", report.usage);
    assert_eq!(
        report.execution,
        ExecutionReport::Complete {
            usage: report.usage.clone()
        }
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    // The field and the candidate search are the class view's own planes, beside this one.
    assert!(report.fields.is_empty());
    assert!(
        report
            .coverage
            .artifact_structural
            .scanned
            .iter()
            .any(|range| range.label == "class_source_bodies" && range.end == 3)
    );
    // The stages every member ran under are published, so one member's run can be reproduced.
    assert_eq!(report.stages, AnalysisStage::ALL.to_vec());
}

/// The same request twice — two fresh snapshots, two fresh budgets — is the same text, byte for
/// byte, and the same per-member markers.
#[test]
fn the_same_request_twice_is_the_same_text() {
    let first = class_source_of(
        &open(HISTORICAL.to_vec()),
        "HistoricalControlFlow",
        EnvironmentPolicy::SingleClass,
    );
    let second = class_source_of(
        &open(HISTORICAL.to_vec()),
        "HistoricalControlFlow",
        EnvironmentPolicy::SingleClass,
    );
    assert_eq!(first.text, second.text);
    assert_eq!(first.coverage, second.coverage);
    assert_eq!(first.fields, second.fields);
    // The reports themselves are not compared whole: a `UsageSnapshot` carries the elapsed clock of
    // the run that produced it. What must not move is every value this presentation *wrote*.
    let written =
        |report: &ClassSourceReport| -> Vec<(String, Option<String>, Vec<String>, String)> {
            report
                .methods
                .iter()
                .map(|method| {
                    (
                        method.declaration.clone().unwrap_or_default(),
                        method.no_body_kind.map(|kind| format!("{kind:?}")),
                        method.markers.clone(),
                        method.text.clone(),
                    )
                })
                .collect()
        };
    assert_eq!(written(&first), written(&second));
}

/// A member that declares no `Code` is a declaration and never a body: no run is charged for it, and
/// the text spells the member Java spells it — with the marker that says why.
#[test]
fn a_member_without_a_body_is_a_declaration_and_never_an_empty_body() {
    let snapshot = open(probe_class());
    let report = class_source_of(&snapshot, "p/Probe", EnvironmentPolicy::SingleClass);

    // The declaration of the whole class, with the interfaces the class file declares: the binary
    // name `p/Probe` states the package `p`, so the text opens with that line and the declaration
    // itself carries the simple name.
    let declaration = report.declaration.as_ref().expect("the class is declared");
    assert_eq!(
        declaration.declaration,
        "public class Probe extends java.lang.Object implements p.Marker, java.io.Serializable"
    );
    assert_eq!(declaration.name, "Probe");
    assert!(report.text.contains("package p;\n\n"), "{}", report.text);

    // The field of the same read, spelled from its descriptor.
    assert_eq!(report.fields.len(), 1);
    let field = &report.fields[0];
    assert_eq!(
        field.declaration.as_deref(),
        Some("public static final int value")
    );
    assert!(field.markers.is_empty());
    assert!(report.text.contains("    public static final int value;\n"));

    // `abstract` and `native` are declarations Java writes without a body, and the marker above each
    // one states that the class file says so.
    let abstract_one = text_of(&report, "abstractOne");
    assert_eq!(
        abstract_one,
        "    // jarde: no body: the member `abstractOne()V` is declared abstract and its \
         declaration carries no Code attribute\n    public abstract void abstractOne();\n"
    );
    let native_one = text_of(&report, "nativeOne");
    assert_eq!(
        native_one,
        "    // jarde: no body: the member `nativeOne()V` is declared native and its declaration \
         carries no Code attribute\n    public native void nativeOne();\n"
    );
    // A member that declares no `Code` and neither flag is a contradiction in the class file, and
    // it is stated as one: the declaration gets a block whose whole content is the marker, so no
    // empty body is ever written for it.
    let contradictory = text_of(&report, "contradictory");
    assert_eq!(
        contradictory,
        "    public void contradictory() {\n        // jarde: no body: the member \
         `contradictory()V` declares no Code attribute and is neither abstract nor native\n    }\n"
    );
    assert!(contradictory.contains("// jarde:"), "not silently dropped");

    // The three members with no body charge nothing: the three bodies are the whole of this
    // request's body work, and the one class read — the binding, over which the one preparation is
    // built — is beside them whatever the member count.
    assert_eq!(report.usage.method_bodies, 3, "{:?}", report.usage);
    assert_eq!(report.usage.class_headers, 1, "{:?}", report.usage);
    let no_body = report
        .methods
        .iter()
        .filter(|method| method.outcome == ClassSourceOutcome::NoBody)
        .count();
    assert_eq!(no_body, 3);
    assert_eq!(
        report
            .methods
            .iter()
            .filter(|method| method.no_body_kind == Some(NoBodyKind::Abstract))
            .count(),
        1
    );
    assert_eq!(
        report
            .methods
            .iter()
            .filter(|method| method.no_body_kind == Some(NoBodyKind::Native))
            .count(),
        1
    );
}

/// A member whose run stopped is marked in the text, its declaration is still written, the members
/// beside it are still presented, and the class report is not `Complete`.
#[test]
fn a_stopped_member_is_marked_and_does_not_stop_the_class() {
    let snapshot = open(probe_class());
    let report = class_source_of(&snapshot, "p/Probe", EnvironmentPolicy::SingleClass);

    // The member whose decode stopped: its declaration is there, its block holds the marker and
    // nothing else, and the artifact that was produced is the empty one the stop contract states.
    let broken = text_of(&report, "broken");
    let marker = only_marker(broken);
    assert!(
        marker.starts_with("// jarde: not recovered: the recovery run for `broken()V` stopped ("),
        "{marker}"
    );
    assert!(
        broken.starts_with("    public static void broken() {\n"),
        "{broken}"
    );
    assert!(broken.ends_with("    }\n"), "{broken}");
    let ClassSourceOutcome::Recovered {
        report: run,
        analysis,
    } = &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"broken")
        .expect("the member is published")
        .outcome
    else {
        panic!("the member's run was performed and refused its artifact");
    };
    assert_eq!(run.content, RecoveryContent::NotProduced);
    assert_eq!(run.text, "");
    assert!(run.source_map.is_empty(), "a stop carries no map");
    assert!(!matches!(
        analysis.execution,
        ExecutionReport::Complete { .. }
    ));

    // The member beside it is presented in full, and the class report states the stop rather than a
    // complete presentation.
    assert!(
        report.text.contains(
            "        // recovered from bytecode; presentation is not claimed to compile\n"
        ),
        "the recoverable member's envelope is in the text: {}",
        report.text
    );
    assert!(
        !matches!(report.execution, ExecutionReport::Complete { .. }),
        "{:?}",
        report.execution
    );
    assert_eq!(report.usage.method_bodies, 3, "{:?}", report.usage);
    assert!(braces_balance(&report.text) == 0, "{}", report.text);
    assert!(indentation_is_four_spaces(&report.text), "{}", report.text);
}

/// A class whose member table stopped is presented as what it is: the declaration the read really
/// established, the members it reached, and the comment that says the rest were never read — no
/// member is invented for the bytes that are missing.
#[test]
fn a_member_table_that_stops_is_stated_and_not_padded() {
    let snapshot = open(truncated_member_table_class());
    let report = class_source_of(&snapshot, "p/Truncated", EnvironmentPolicy::SingleClass);
    assert!(report.declaration.is_some(), "the class itself was read");
    assert!(report.fields.is_empty());
    assert!(report.methods.is_empty());
    assert!(
        report.text.contains("member table stopped at fields[0]"),
        "{}",
        report.text
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error),
        "{:?}",
        report.diagnostics
    );
    assert!(!matches!(
        report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    assert!(
        report
            .coverage
            .artifact_structural
            .skipped
            .iter()
            .any(|range| range.label == "class_fields"),
        "the range that was never read is marked: {:?}",
        report.coverage
    );
    assert!(braces_balance(&report.text) == 0, "{}", report.text);
    assert!(indentation_is_four_spaces(&report.text), "{}", report.text);
}

/// A member table that stops in front of a readable body keeps both statements apart: the prefix
/// member is presented as the member it is, with the stop's own reason where its body would be, and
/// the record behind the stop is **not** published as a member of this class.
#[test]
fn a_method_table_that_stops_keeps_its_prefix_and_claims_no_more() {
    let snapshot = open(stopped_method_table_class());
    let report = class_source_of(&snapshot, "p/Stopped", EnvironmentPolicy::SingleClass);

    // The class is read and presented, and the one member the read reached is published with its own
    // declaration and the marker that says why no body of it was decoded.
    assert!(report.declaration.is_some(), "the class itself was read");
    assert_eq!(report.methods.len(), 1, "{:?}", report.methods);
    let prefix_member = &report.methods[0];
    assert_eq!(prefix_member.item.name.raw().0, b"good");
    assert_eq!(
        prefix_member.declaration.as_deref(),
        Some("public static void good()")
    );
    assert!(
        prefix_member
            .markers
            .iter()
            .any(|marker| marker.contains("classfile_decode")),
        "the member states the stop rather than a body: {:?}",
        prefix_member.markers
    );
    // The record behind the stop is not a member of this presentation: no member conclusion about it
    // is published, in the report or in the text.
    assert!(!report.text.contains("second"), "{}", report.text);
    assert!(
        report.text.contains("member table stopped at methods[1]"),
        "{}",
        report.text
    );
    // No body was decoded from a table that did not read to its end, and the request is not complete.
    assert_eq!(report.usage.method_bodies, 0, "{:?}", report.usage);
    assert_eq!(report.usage.class_headers, 1, "{:?}", report.usage);
    assert!(!matches!(
        report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    assert!(
        report
            .coverage
            .artifact_structural
            .skipped
            .iter()
            .any(|range| range.label == "class_methods"),
        "the range the walk never read is marked: {:?}",
        report.coverage
    );
    assert!(braces_balance(&report.text) == 0, "{}", report.text);
    assert!(indentation_is_four_spaces(&report.text), "{}", report.text);
}

/// An artifact that holds no statement is marked, and the artifact itself — the reasons and the
/// bytecode it quotes — stays in the text instead of being dropped.
#[test]
fn an_explanation_only_member_is_marked_and_its_text_is_kept() {
    let snapshot = open(duplicate_result_class());
    let report = class_source_of(
        &snapshot,
        "p/DuplicateResult",
        EnvironmentPolicy::SingleClass,
    );

    let explanation_only: Vec<&ClassSourceMethod> = report
        .methods
        .iter()
        .filter(|method| {
            matches!(
                &method.outcome,
                ClassSourceOutcome::Recovered { report, .. }
                    if report.content == RecoveryContent::ExplanationOnly
            )
        })
        .collect();
    assert_eq!(
        explanation_only.len(),
        1,
        "the unsupported duplicate shape remains explanation-only"
    );
    for method in explanation_only {
        let ClassSourceOutcome::Recovered { report: run, .. } = &method.outcome else {
            unreachable!("filtered above")
        };
        assert!(run.produced());
        assert!(
            only_marker(&method.text).contains("produced no statement"),
            "{}",
            method.text
        );
        assert!(
            method.text.contains("// @bytecode "),
            "the refusal the artifact quotes is kept:\n{}",
            method.text
        );
    }
    // The recovery runs of those members completed: what they could not do is produce a statement,
    // which the content plane and the marker state rather than the execution plane.
    assert_eq!(
        report.execution,
        ExecutionReport::Complete {
            usage: report.usage.clone()
        }
    );
    assert!(braces_balance(&report.text) == 0, "{}", report.text);
}

/// Two definitions of one name are two candidates and no presentation; one of them, named by its
/// own identity, is presented.
#[test]
fn a_name_two_definitions_answer_to_presents_nothing() {
    let bytes = probe_class();
    let snapshot = open(zip_of(&[
        (b"p/Probe.class", &bytes),
        (b"WEB-INF/classes/p/Probe.class", &bytes),
        (b"META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\n"),
    ]));
    let engine = Engine::new();
    let ambiguous = engine
        .class_source(
            slice::from_ref(&snapshot),
            &request(
                &snapshot,
                ClassRef::Name {
                    class: ClassNameQuery::internal("p/Probe"),
                },
                EnvironmentPolicy::PlainJar,
            ),
            &mut budget(),
        )
        .expect("a legal request is answered");
    let OperationOutcome::Ambiguous(candidates) = ambiguous else {
        panic!("two origins of one name are two candidates: {ambiguous:?}");
    };
    assert_eq!(candidates.candidates.len(), 2);
    assert_eq!(
        candidates
            .candidates
            .iter()
            .filter(|candidate| matches!(candidate, ClassContentItem::ClassDeclaration(_)))
            .count(),
        2
    );
    assert!(matches!(
        candidates.execution,
        ExecutionReport::Complete { .. }
    ));

    // The first candidate's own identity is directly usable, and it reads exactly that definition.
    let ClassContentItem::ClassDeclaration(chosen) = candidates.candidates[0].clone() else {
        unreachable!("a class search publishes class declarations")
    };
    let report = performed(
        engine
            .class_source(
                slice::from_ref(&snapshot),
                &request(
                    &snapshot,
                    ClassRef::Definition {
                        definition: chosen.definition.clone(),
                    },
                    EnvironmentPolicy::PlainJar,
                ),
                &mut budget(),
            )
            .expect("a legal request is answered"),
    );
    assert_eq!(report.class, chosen.definition);
    assert!(
        report
            .text
            .contains("public class Probe extends java.lang.Object")
    );
}

/// An identity of another artifact is an input error, never replaced by a same-named definition of
/// the snapshot at hand.
#[test]
fn an_identity_of_another_snapshot_is_an_input_error() {
    let other = open(probe_class());
    let target = open(HISTORICAL.to_vec());
    let definition = class_source_of(&other, "p/Probe", EnvironmentPolicy::SingleClass).class;
    let error = Engine::new()
        .class_source(
            slice::from_ref(&target),
            &request(
                &target,
                ClassRef::Definition { definition },
                EnvironmentPolicy::SingleClass,
            ),
            &mut budget(),
        )
        .expect_err("an identity of another snapshot is refused");
    assert_eq!(error_code(&error), "operation_target_snapshot_mismatch");
}

/// A request whose item budget cannot pay for its own declaration publishes no declaration, no
/// member and no text, and states the stop where every other report states one.
///
/// The item budget a presentation needs before it may publish anything is the reader's own
/// accounting, so this case walks the boundary instead of naming the count: whatever that count
/// becomes, "no declaration" and "no text" stay the same state, and some budget refuses the
/// declaration itself.
#[test]
fn a_request_that_cannot_pay_for_its_declaration_publishes_no_text() {
    let snapshot = open(probe_class());
    let definition = class_source_of(&snapshot, "p/Probe", EnvironmentPolicy::SingleClass).class;
    let mut refused = None;
    for limit in 1..=16 {
        let mut budget =
            task_budget(&[BudgetOverride::new("result_items", limit).expect("a legal override")])
                .expect("the override is legal");
        let outcome = Engine::new().class_source(
            slice::from_ref(&snapshot),
            &request(
                &snapshot,
                ClassRef::Definition {
                    definition: definition.clone(),
                },
                EnvironmentPolicy::SingleClass,
            ),
            &mut budget,
        );
        let outcome = match outcome {
            // The read itself was refused before it materialized anything: nothing was established
            // to publish, which is the request-level refusal every read in this engine states.
            Err(Error::BudgetExceeded { .. }) => continue,
            Err(error) => panic!("at {limit} item(s): unexpected {error:?}"),
            Ok(outcome) => outcome,
        };
        match outcome {
            // The selection itself did not finish: nothing was bound, so nothing is presented.
            OperationOutcome::Incomplete(candidates) => {
                assert!(!matches!(
                    candidates.execution,
                    ExecutionReport::Complete { .. }
                ));
            }
            OperationOutcome::Performed(report) => {
                assert_eq!(
                    report.declaration.is_none(),
                    report.text.is_empty(),
                    "at {limit} item(s): the text is empty exactly when no declaration was \
                     published"
                );
                match &report.declaration {
                    // A declaration this request paid for is in the text it opens.
                    Some(declaration) => assert!(
                        report.text.contains(&declaration.declaration),
                        "at {limit} item(s): {}",
                        report.text
                    ),
                    // Nothing of the class is presented, and the stop is stated where every other
                    // report states one.
                    None => {
                        assert!(report.methods.is_empty());
                        assert!(report.fields.is_empty());
                        assert!(!matches!(
                            report.execution,
                            ExecutionReport::Complete { .. }
                        ));
                        assert!(
                            report.diagnostics.iter().any(|diagnostic| diagnostic.code
                                == "budget_exceeded_result_items"),
                            "{:?}",
                            report.diagnostics
                        );
                        refused = Some(report);
                    }
                }
            }
            OperationOutcome::Ambiguous(_) => panic!("one definition is never ambiguous"),
        }
    }
    assert!(
        refused.is_some(),
        "some item budget refuses the class item charge itself"
    );
}

/// A stop that is the *request's* ends it: the members after it are not begun, the text states how
/// many the class declares beside how many were presented, and the coverage marks the range that
/// was skipped.
#[test]
fn a_request_that_stops_mid_class_states_the_shortfall() {
    let snapshot = open(probe_class());
    // One body attempt: the first member's run is funded, and the second member's charge is refused.
    let mut budget =
        task_budget(&[BudgetOverride::new("method_bodies", 1).expect("a legal override")])
            .expect("the override is legal");
    let report = performed(
        Engine::new()
            .class_source(
                slice::from_ref(&snapshot),
                &request(
                    &snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal("p/Probe"),
                    },
                    EnvironmentPolicy::SingleClass,
                ),
                &mut budget,
            )
            .expect("a legal request is answered"),
    );
    assert_eq!(report.usage.method_bodies, 1, "{:?}", report.usage);
    let ExecutionReport::Partial { reason, .. } = &report.execution else {
        panic!("the stop is the request's: {:?}", report.execution)
    };
    assert_eq!(
        reason,
        &TerminationReason::BudgetExceeded {
            dimension: BudgetDimension::MethodBodies
        }
    );
    // The member whose own run was refused keeps its own result — the stop is that run's, stated by
    // the two planes of it — and the members behind it were never begun.
    assert_eq!(report.methods.len(), 2, "{:?}", report.methods);
    let second = &report.methods[1];
    let ClassSourceOutcome::Recovered {
        report: run,
        analysis,
    } = &second.outcome
    else {
        panic!("the member's run was performed and stopped inside: {second:?}")
    };
    assert_eq!(run.content, RecoveryContent::NotProduced);
    let ExecutionReport::Partial { reason, .. } = &analysis.execution else {
        panic!(
            "the analysis of that member stopped: {:?}",
            analysis.execution
        )
    };
    assert_eq!(
        reason,
        &TerminationReason::BudgetExceeded {
            dimension: BudgetDimension::MethodBodies
        },
        "the analysis of that member is what the refused charge stopped"
    );
    assert_eq!(second.markers.len(), 1);
    assert!(
        second.markers[0].contains("budget_exceeded_method_bodies"),
        "{}",
        second.markers[0]
    );
    // The text states the shortfall by name instead of presenting a shorter class as a whole one.
    assert!(
        report
            .text
            .contains("// jarde: the class file declares 6 method record(s)"),
        "{}",
        report.text
    );
    assert!(report.text.contains("2 were presented"), "{}", report.text);
    assert!(
        report.text.contains("budget_exceeded_method_bodies"),
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("nativeOne"),
        "a member that was never begun is not presented: {}",
        report.text
    );
    let skipped: Vec<(u64, u64)> = report
        .coverage
        .artifact_structural
        .skipped
        .iter()
        .filter(|range| range.label == "class_source_bodies")
        .map(|range| (range.start, range.end))
        .collect();
    assert_eq!(
        skipped,
        [(2, 3)],
        "the third body is the member this request never began: {:?}",
        report.coverage
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
}

#[test]
fn bridge_admission_needs_the_same_run_shape_source_and_resolved_parent_contract() {
    let jar = open(zip_of(&[
        (b"BridgeApi.class", BRIDGE_API),
        (b"BridgeProbe.class", BRIDGE_PROBE),
    ]));
    let essential = bridge_class_source(
        &jar,
        "BridgeProbe",
        EnvironmentPolicy::PlainJar,
        &RecoveryEvidenceRequest::essential(),
    );
    let all = bridge_class_source(
        &jar,
        "BridgeProbe",
        EnvironmentPolicy::PlainJar,
        &RecoveryEvidenceRequest::all(),
    );
    assert_eq!(essential.bridge_proofs, all.bridge_proofs);
    assert_eq!(essential.bridge_proofs.len(), 1);
    let admitted = &essential.bridge_proofs[0];
    assert!(admitted.admitted, "{:?}", admitted.refusal);
    assert_eq!(admitted.member.name.0.as_slice(), b"get");
    assert_eq!(
        admitted.member.descriptor.0.as_slice(),
        b"()Ljava/lang/Object;"
    );
    assert_eq!(
        admitted.target.as_ref().unwrap().descriptor.0.as_slice(),
        b"()Ljava/lang/String;"
    );
    assert_eq!(admitted.call_bci, Some(1));
    assert!(admitted.projected);
    assert!(
        essential
            .methods
            .windows(2)
            .all(|pair| { pair[0].item.index < pair[1].item.index })
    );
    let bridge_method = essential
        .methods
        .iter()
        .find(|method| method.item.identity == admitted.member)
        .expect("the physical bridge stays in the report");
    let ClassSourceOutcome::Recovered {
        report: original_bridge_body,
        ..
    } = &bridge_method.outcome
    else {
        panic!("the original bridge recovery report remains available")
    };
    assert!(original_bridge_body.text.contains("return this.get();"));
    assert!(
        bridge_method
            .markers
            .iter()
            .any(|marker| { marker.contains("projected bridge") && marker.contains("BCI 1") })
    );
    assert!(
        bridge_method
            .markers
            .iter()
            .all(|marker| bridge_method.text.contains(marker))
    );
    assert!(essential.text.contains(&bridge_method.text));
    assert!(!essential.text.contains("public java.lang.Object get()"));
    assert_eq!(essential.text, all.text);
    let serialized = serde_json::to_value(&essential).expect("class-source proof is JSON evidence");
    assert_eq!(serialized["bridge_proofs"][0]["call_bci"], 1);
    assert_eq!(serialized["bridge_proofs"][0]["projected"], true);
    assert_eq!(
        serialized["bridge_proofs"][0]["member"]["descriptor"],
        serde_json::json!(b"()Ljava/lang/Object;".to_vec())
    );
    assert_eq!(
        serialized["bridge_proofs"][0]["target"]["descriptor"],
        serde_json::json!(b"()Ljava/lang/String;".to_vec())
    );

    let projection_bytes = u64::try_from(bridge_method.markers.last().unwrap().len())
        .expect("the bridge projection marker fits the byte budget");
    assert!(all.usage.output_bytes >= projection_bytes);
    let mut limits = all.limits.clone();
    limits.output_bytes = all.usage.output_bytes - 1;
    let mut constrained_budget = Budget::new(limits);
    let stopped = performed(
        Engine::new()
            .class_source_with_evidence(
                slice::from_ref(&jar),
                &request(
                    &jar,
                    ClassRef::Name {
                        class: ClassNameQuery::internal("BridgeProbe"),
                    },
                    EnvironmentPolicy::PlainJar,
                ),
                &RecoveryEvidenceRequest::all(),
                &mut constrained_budget,
            )
            .expect("a budget stop still returns the class-source report"),
    );
    assert!(matches!(
        stopped.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::OutputBytes
            },
            ..
        }
    ));
    assert!(stopped.bridge_proofs[0].admitted);
    assert!(!stopped.bridge_proofs[0].projected);
    let stopped_bridge = stopped
        .methods
        .iter()
        .find(|method| method.item.identity == admitted.member)
        .expect("the original bridge remains in the physical table");
    assert!(stopped_bridge.text.contains("java.lang.Object get()"));
    assert!(stopped.text.contains(&stopped_bridge.text));

    let fake = open(zip_of(&[(b"FakeBridge.class", FAKE_BRIDGE)]));
    let fake = bridge_class_source(
        &fake,
        "FakeBridge",
        EnvironmentPolicy::PlainJar,
        &RecoveryEvidenceRequest::essential(),
    );
    assert_eq!(fake.bridge_proofs.len(), 1);
    assert!(!fake.bridge_proofs[0].admitted);
    assert!(
        fake.bridge_proofs[0]
            .refusal
            .as_deref()
            .unwrap()
            .contains("pure single forward")
    );
    assert!(!fake.bridge_proofs[0].projected);
    assert!(fake.text.contains("java.lang.Object get()"));

    let orphan = open(zip_of(&[(b"OrphanBridge.class", ORPHAN_BRIDGE)]));
    let orphan = bridge_class_source(
        &orphan,
        "OrphanBridge",
        EnvironmentPolicy::PlainJar,
        &RecoveryEvidenceRequest::essential(),
    );
    assert_eq!(orphan.bridge_proofs.len(), 1);
    assert!(!orphan.bridge_proofs[0].admitted);
    assert!(
        orphan.bridge_proofs[0]
            .refusal
            .as_deref()
            .unwrap()
            .contains("no resolved direct parent")
    );
    assert!(!orphan.bridge_proofs[0].projected);
    assert!(orphan.text.contains("java.lang.Object get()"));

    let single = open(BRIDGE_PROBE.to_vec());
    let single = bridge_class_source(
        &single,
        "BridgeProbe",
        EnvironmentPolicy::SingleClass,
        &RecoveryEvidenceRequest::essential(),
    );
    assert_eq!(single.bridge_proofs.len(), 1);
    assert!(!single.bridge_proofs[0].admitted);
    assert_eq!(
        single.usage.class_headers, 1,
        "the prepared class header was not reread"
    );
    assert!(
        single.bridge_proofs[0]
            .refusal
            .as_deref()
            .unwrap()
            .contains("unresolved"),
        "actual refusal: {:?}",
        single.bridge_proofs[0].refusal
    );
}

#[test]
fn an_admitted_bridge_is_rebuilt_from_the_complete_projected_source() {
    let jar = open(zip_of(&[
        (b"BridgeApi.class", BRIDGE_API),
        (b"BridgeProbe.class", BRIDGE_PROBE),
    ]));
    let report = bridge_class_source(
        &jar,
        "BridgeProbe",
        EnvironmentPolicy::PlainJar,
        &RecoveryEvidenceRequest::all(),
    );
    let proof = report
        .bridge_proofs
        .iter()
        .find(|proof| proof.admitted)
        .expect("the ordinary Java 8 override proves its bridge");
    assert_eq!(proof.call_bci, Some(1));
    assert!(proof.projected);
    assert!(report.text.contains("String get()"));
    assert!(!report.text.contains("Object get()"));

    let scratch = BridgeProjectionScratch::new();
    let original = scratch.child("original");
    fs::write(original.join("BridgeProbe.class"), BRIDGE_PROBE)
        .expect("write the frozen implementation class");
    fs::write(original.join("BridgeApi.class"), BRIDGE_API)
        .expect("write the frozen interface class");
    fs::write(original.join("BridgeRunner.java"), BRIDGE_RUNNER_SOURCE)
        .expect("write the source-only runner");
    compile_bridge_runner(&original, &["BridgeRunner.java"]);
    let original_trace = run_bridge_runner(&original);

    let recovered = scratch.child("recovered");
    fs::write(recovered.join("BridgeProbe.java"), &report.text)
        .expect("write the complete projected class source");
    fs::write(recovered.join("BridgeApi.java"), BRIDGE_API_SOURCE)
        .expect("write the source-level interface contract");
    fs::write(recovered.join("BridgeRunner.java"), BRIDGE_RUNNER_SOURCE)
        .expect("write the source-only runner");
    compile_bridge_runner(
        &recovered,
        &["BridgeProbe.java", "BridgeApi.java", "BridgeRunner.java"],
    );
    let recovered_trace = run_bridge_runner(&recovered);
    assert_eq!(original_trace, "value|value|value\n");
    assert_eq!(recovered_trace, original_trace);

    let javap = Command::new("javap")
        .args(["-p", "-c", "-v", "-classpath"])
        .arg(&recovered)
        .arg("BridgeProbe")
        .output()
        .expect("JDK javap is available for bridge flag inspection");
    assert!(
        javap.status.success(),
        "javap failed:\n{}",
        String::from_utf8_lossy(&javap.stderr)
    );
    let javap = String::from_utf8(javap.stdout).expect("javap output is UTF-8");
    let bridge = javap
        .split_once("public java.lang.Object get();")
        .map(|(_, tail)| tail)
        .expect("javac regenerated the erased Object bridge");
    let bridge = bridge
        .split_once("\n  public ")
        .map_or(bridge, |(method, _)| method);
    assert!(
        bridge.contains("descriptor: ()Ljava/lang/Object;"),
        "{bridge}"
    );
    assert!(
        bridge.contains("ACC_PUBLIC, ACC_BRIDGE, ACC_SYNTHETIC"),
        "{bridge}"
    );
    assert!(
        bridge.contains("// Method get:()Ljava/lang/String;"),
        "the regenerated erased bridge must target the source override:\n{bridge}"
    );
}

#[test]
fn bridge_admission_rejects_handlers_metadata_unwritable_targets_and_ambiguous_parents() {
    // These are explicitly labeled class-file patches: the exception table row is a reader
    // boundary probe and is not claimed to pass JVM verification.
    let handler_class = add_bridge_handler(BRIDGE_PROBE);
    let handler_jar = open(zip_of(&[
        (b"BridgeApi.class", BRIDGE_API),
        (b"BridgeProbe.class", &handler_class),
    ]));
    let handler = bridge_class_source(
        &handler_jar,
        "BridgeProbe",
        EnvironmentPolicy::PlainJar,
        &RecoveryEvidenceRequest::essential(),
    );
    assert!(handler.bridge_proofs.is_empty());
    assert!(matches!(
        handler.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::Error { ref code },
            ..
        } if code == "ir_frame_inconsistent"
    ));

    // Deprecated is a legal zero-length method attribute. It stands for observable metadata whose
    // bridge-copying behavior this proof does not establish; debug tables nested under Code remain
    // allowed because they are not method-level attributes.
    let attributed_class =
        add_deprecated_method_attribute(BRIDGE_PROBE, b"get", b"()Ljava/lang/String;");
    let attributed_jar = open(zip_of(&[
        (b"BridgeApi.class", BRIDGE_API),
        (b"BridgeProbe.class", &attributed_class),
    ]));
    let attributed = bridge_class_source(
        &attributed_jar,
        "BridgeProbe",
        EnvironmentPolicy::PlainJar,
        &RecoveryEvidenceRequest::essential(),
    );
    assert_eq!(attributed.bridge_proofs.len(), 1);
    assert!(!attributed.bridge_proofs[0].admitted);
    assert!(
        attributed.bridge_proofs[0]
            .refusal
            .as_deref()
            .unwrap()
            .contains("metadata")
    );

    // These target modifiers remain on the source-level declaration and javac 23.0.1 --release 8
    // still rebuilds the same 0x1041 public bridge. Final on the target is allowed; final on the
    // bridge itself is not reconstructed and is refused below.
    for (label, flags) in [
        ("final", 0x0011),
        ("synchronized", 0x0021),
        ("strictfp", 0x0801),
    ] {
        let target = patch_method_flags(BRIDGE_PROBE, b"get", b"()Ljava/lang/String;", flags);
        let jar = open(zip_of(&[
            (b"BridgeApi.class", BRIDGE_API),
            (b"BridgeProbe.class", &target),
        ]));
        let report = bridge_class_source(
            &jar,
            "BridgeProbe",
            EnvironmentPolicy::PlainJar,
            &RecoveryEvidenceRequest::essential(),
        );
        assert_eq!(report.bridge_proofs.len(), 1, "{label}");
        assert!(
            report.bridge_proofs[0].admitted,
            "{label}: {:?}",
            report.bridge_proofs[0].refusal
        );
    }

    let final_bridge = patch_method_flags(BRIDGE_PROBE, b"get", b"()Ljava/lang/Object;", 0x1051);
    let final_bridge_jar = open(zip_of(&[
        (b"BridgeApi.class", BRIDGE_API),
        (b"BridgeProbe.class", &final_bridge),
    ]));
    let final_bridge_report = bridge_class_source(
        &final_bridge_jar,
        "BridgeProbe",
        EnvironmentPolicy::PlainJar,
        &RecoveryEvidenceRequest::essential(),
    );
    assert_eq!(final_bridge_report.bridge_proofs.len(), 1);
    assert!(!final_bridge_report.bridge_proofs[0].admitted);
    assert!(
        final_bridge_report.bridge_proofs[0]
            .refusal
            .as_deref()
            .unwrap()
            .contains("modifiers")
    );

    // These flag patches probe target shapes that cannot be reconstructed as the invoked public
    // instance declaration. They are class-file boundary probes, not verifier-valid replacements.
    for (label, flags) in [
        ("private", 0x0002),
        ("static", 0x0009),
        ("synthetic", 0x1001),
    ] {
        let target = patch_method_flags(BRIDGE_PROBE, b"get", b"()Ljava/lang/String;", flags);
        let jar = open(zip_of(&[
            (b"BridgeApi.class", BRIDGE_API),
            (b"BridgeProbe.class", &target),
        ]));
        let report = bridge_class_source(
            &jar,
            "BridgeProbe",
            EnvironmentPolicy::PlainJar,
            &RecoveryEvidenceRequest::essential(),
        );
        assert_eq!(report.bridge_proofs.len(), 1, "{label}");
        assert!(!report.bridge_proofs[0].admitted, "{label}");
    }

    // A direct interface owner that is absent from a complete artifact is still unresolved
    // evidence. This case is separate from SingleClass: PlainJar may resolve inherited methods,
    // but it may not infer one from a missing owner.
    let missing_owner_jar = open(zip_of(&[(b"BridgeProbe.class", BRIDGE_PROBE)]));
    let missing_owner = bridge_class_source(
        &missing_owner_jar,
        "BridgeProbe",
        EnvironmentPolicy::PlainJar,
        &RecoveryEvidenceRequest::essential(),
    );
    assert_eq!(missing_owner.bridge_proofs.len(), 1);
    assert!(!missing_owner.bridge_proofs[0].admitted);
    assert!(
        missing_owner.bridge_proofs[0]
            .refusal
            .as_deref()
            .unwrap()
            .contains("unresolved")
    );

    let ambiguous_jar = open(zip_of(&[
        (b"BridgeApi.class", BRIDGE_API),
        (b"BridgeApi.class", BRIDGE_API),
        (b"BridgeProbe.class", BRIDGE_PROBE),
    ]));
    let ambiguous = bridge_class_source(
        &ambiguous_jar,
        "BridgeProbe",
        EnvironmentPolicy::PlainJar,
        &RecoveryEvidenceRequest::essential(),
    );
    assert_eq!(ambiguous.bridge_proofs.len(), 1);
    assert!(!ambiguous.bridge_proofs[0].admitted);
    assert!(
        ambiguous.bridge_proofs[0]
            .refusal
            .as_deref()
            .unwrap()
            .contains("unresolved")
    );
}

/// A member whose raw descriptor is not a method descriptor is stated as such: no declaration, no
/// run, and the marker that says both — while the members beside it are presented as usual.
#[test]
fn a_member_whose_descriptor_cannot_be_read_is_stated_and_not_run() {
    let bytes = class_file(
        b"p/Odd",
        b"java/lang/Object",
        &[],
        CLASS_FLAGS,
        &[],
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
    let snapshot = open(bytes);
    let report = class_source_of(&snapshot, "p/Odd", EnvironmentPolicy::SingleClass);
    let odd = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"odd")
        .expect("the member is published");
    assert_eq!(odd.outcome, ClassSourceOutcome::Unspelled);
    assert!(odd.declaration.is_none());
    assert_eq!(
        odd.text,
        "    // jarde: not spelled: the descriptor `not-a-descriptor` of the member \
                          `odd` is not a method descriptor, so this presentation writes no \
                          declaration for it and performs no run for it\n"
    );
    // Only the member this presentation can spell was run: the odd one is not work that was skipped.
    assert_eq!(report.usage.method_bodies, 1, "{:?}", report.usage);
    assert!(
        report.text.contains("public void fine()"),
        "{}",
        report.text
    );
    assert!(
        report
            .coverage
            .artifact_structural
            .scanned
            .iter()
            .any(|range| range.label == "class_source_bodies" && range.end == 1)
    );
    assert!(
        report.coverage.artifact_structural.skipped.is_empty(),
        "{:?}",
        report.coverage
    );
}

/// The report is the library's own value: it serializes to the documented shape, and the text it
/// carries is the field a consumer reads.
#[test]
fn the_report_serializes_with_the_text_it_publishes() {
    let snapshot = open(HISTORICAL.to_vec());
    let report = class_source_of(
        &snapshot,
        "HistoricalControlFlow",
        EnvironmentPolicy::SingleClass,
    );
    let document = serde_json::to_value(&report).expect("the report is a document");
    assert_eq!(
        document["text"].as_str().expect("the text is a string"),
        report.text
    );
    assert_eq!(
        document["class"]["location"]["kind"].as_str(),
        Some("standalone_root")
    );
    assert_eq!(
        document["methods"][1]["item"]["name"]["escaped"].as_str(),
        Some("add")
    );
    assert_eq!(
        document["methods"][1]["outcome"]["kind"].as_str(),
        Some("recovered")
    );
    assert_eq!(
        document["methods"][1]["outcome"]["report"]["text"].as_str(),
        Some(
            report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"add")
                .expect("the member is published")
                .outcome
                .clone()
                .into_recovery_text()
                .as_str()
        )
    );
}

#[test]
fn class_retention_annotation_is_spelled_from_its_invisible_attribute() {
    let report = class_source_of(
        &open(CLASS_RETENTION_TARGET.to_vec()),
        "HiddenTarget",
        EnvironmentPolicy::SingleClass,
    );
    let declaration = report.declaration.as_ref().expect("class declaration");
    assert_eq!(declaration.annotation_uses, ["@HiddenTag(value = 5)"]);
    assert!(declaration.annotation_refusals.is_empty());
    assert_eq!(declaration.annotation_attributes.len(), 1);
    assert_eq!(
        declaration.annotation_attributes[0].attribute.name.raw().0,
        b"RuntimeInvisibleAnnotations"
    );
    assert_eq!(declaration.annotation_attributes[0].annotations.len(), 1);
    assert!(
        report.text.find("@HiddenTag(value = 5)").unwrap()
            < report.text.find("class HiddenTarget").unwrap()
    );
    let json = serde_json::to_value(&report).expect("the annotation report serializes");
    assert_eq!(
        json["declaration"]["annotation_uses"][0],
        "@HiddenTag(value = 5)"
    );
    assert_eq!(
        json["declaration"]["annotation_attributes"][0]["annotations"][0]["Annotation"]["type_descriptor"],
        serde_json::to_value(b"LHiddenTag;").unwrap()
    );
}

#[test]
fn class_without_annotation_attributes_gains_no_annotation_content() {
    let report = class_source_of(
        &open(EMPTY_ANNOTATION_TARGET.to_vec()),
        "EmptyTarget",
        EnvironmentPolicy::SingleClass,
    );
    let declaration = report.declaration.expect("class declaration");
    assert!(declaration.annotation_attributes.is_empty());
    assert!(declaration.annotation_uses.is_empty());
    assert!(declaration.annotation_refusals.is_empty());
}

#[test]
fn runtime_visible_class_annotation_uses_the_same_declaration_path() {
    let report = class_source_of(
        &open(RUNTIME_VISIBLE_TARGET.to_vec()),
        "VisibleTarget",
        EnvironmentPolicy::SingleClass,
    );
    let declaration = report.declaration.expect("class declaration");
    assert_eq!(
        declaration.annotation_uses,
        ["@VisibleTag(value = \"visible\")"]
    );
    assert_eq!(
        declaration.annotation_attributes[0].attribute.name.raw().0,
        b"RuntimeVisibleAnnotations"
    );
}

#[test]
fn annotation_type_without_a_body_keeps_its_runtime_visible_class_annotation() {
    let report = class_source_of(
        &open(ANNOTATION_TYPE.to_vec()),
        "HiddenTag",
        EnvironmentPolicy::SingleClass,
    );
    let declaration = report.declaration.expect("annotation class declaration");
    assert_eq!(
        declaration.annotation_uses,
        ["@java.lang.annotation.Retention(value = java.lang.annotation.RetentionPolicy.CLASS)"]
    );
    assert_eq!(report.methods.len(), 1);
    assert_eq!(report.methods[0].no_body_kind, Some(NoBodyKind::Abstract));
    assert!(report.text.contains("@interface HiddenTag"));
}

#[test]
fn class_annotation_named_array_values_keep_nested_annotation_order() {
    let report = class_source_of(
        &open(NESTED_ARRAY_TARGET.to_vec()),
        "DuplicateTarget",
        EnvironmentPolicy::SingleClass,
    );
    let declaration = report.declaration.expect("class declaration");
    assert_eq!(
        declaration.annotation_uses,
        ["@Tags(value = {@Tag(value = \"one\"), @Tag(value = \"two\")})"]
    );
}

#[test]
fn visible_and_invisible_annotations_follow_physical_attribute_order() {
    let report = class_source_of(
        &open(MIXED_RETENTION_TARGET.to_vec()),
        "MixedTarget",
        EnvironmentPolicy::SingleClass,
    );
    let declaration = report.declaration.expect("class declaration");
    let attribute_names: Vec<&[u8]> = declaration
        .annotation_attributes
        .iter()
        .map(|attribute| attribute.attribute.name.raw().0.as_slice())
        .collect();
    assert_eq!(
        attribute_names,
        [
            b"RuntimeVisibleAnnotations".as_slice(),
            b"RuntimeInvisibleAnnotations"
        ]
    );
    assert_eq!(
        declaration.annotation_uses,
        ["@VisibleTag(value = \"visible\")", "@HiddenTag(value = 5)"]
    );
}

#[test]
fn member_declaration_annotations_do_not_consume_type_use_attributes() {
    let report = class_source_of(
        &open(MEMBER_PLACEMENT_TARGET.to_vec()),
        "MemberPlacementTarget",
        EnvironmentPolicy::SingleClass,
    );
    let declaration = report.declaration.expect("class declaration");
    assert!(declaration.annotation_attributes.is_empty());
    assert!(declaration.annotation_uses.is_empty());
    let field = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"field")
        .expect("annotated field");
    assert_eq!(field.annotations.uses, ["@MemberPlacement"]);
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"method")
        .expect("annotated method");
    assert_eq!(method.annotations.uses, ["@MemberPlacement"]);
    assert_eq!(
        method.parameter_annotations.uses_by_position,
        [vec!["@MemberPlacement".to_owned()]]
    );
    assert_eq!(
        report.text.matches("@MemberPlacement").count(),
        3,
        "only the three Runtime*Annotations uses are written; the independent type-use attributes stay unread"
    );
}

#[test]
fn member_annotations_keep_field_method_and_parameter_ownership_in_text_and_json() {
    let report = class_source_of(
        &open(MEMBER_ANNOTATION_TARGET.to_vec()),
        "MemberTagged",
        EnvironmentPolicy::SingleClass,
    );
    let field = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"field")
        .expect("annotated field");
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"value")
        .expect("annotated method");
    assert_eq!(field.annotations.uses, ["@java.lang.Deprecated"]);
    assert_eq!(method.annotations.uses, ["@java.lang.Deprecated"]);
    assert_eq!(
        method.parameter_annotations.uses_by_position,
        [vec!["@java.lang.Deprecated".to_owned()]]
    );
    assert_eq!(
        method.parameter_annotations.attributes[0].parameter_count,
        Some(1)
    );
    assert_eq!(
        method.parameter_annotations.attributes[0].parameters.len(),
        1
    );
    assert!(
        method
            .declaration
            .as_deref()
            .unwrap()
            .contains("value(@java.lang.Deprecated int arg1)")
    );
    assert!(
        report
            .declaration
            .as_ref()
            .unwrap()
            .annotation_attributes
            .is_empty()
    );
    assert!(
        report
            .text
            .find("@java.lang.Deprecated\n    public int field")
            .is_some()
    );
    assert!(
        report
            .text
            .find("@java.lang.Deprecated\n    public int value")
            .is_some()
    );

    let constructor = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"<init>")
        .expect("ordinary constructor");
    assert!(constructor.annotations.attributes.is_empty());
    assert!(constructor.parameter_annotations.attributes.is_empty());

    let json = serde_json::to_value(&report).expect("member annotation report serializes");
    assert_eq!(
        json["fields"][0]["annotations"]["uses"][0],
        "@java.lang.Deprecated"
    );
    assert_eq!(
        json["methods"][1]["parameter_annotations"]["attributes"][0]["parameter_count"],
        1
    );
    assert_eq!(
        json["methods"][1]["parameter_annotations"]["uses_by_position"][0][0],
        "@java.lang.Deprecated"
    );
}

#[test]
fn invisible_member_annotations_align_wide_and_varargs_by_descriptor_position() {
    let report = class_source_of(
        &open(MEMBER_BOUNDARY_TARGET.to_vec()),
        "BoundaryTagged",
        EnvironmentPolicy::SingleClass,
    );
    let field = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"field")
        .expect("invisible field annotation");
    assert_eq!(field.annotations.uses, ["@BoundaryMark(value = 1)"]);
    assert_eq!(
        field.annotations.attributes[0].attribute.name.raw().0,
        b"RuntimeInvisibleAnnotations"
    );

    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"wideAndVarargs")
        .expect("wide varargs method");
    assert_eq!(method.annotations.uses, ["@BoundaryMark(value = 2)"]);
    assert_eq!(
        method.parameter_annotations.uses_by_position,
        [
            vec!["@BoundaryMark(value = 3)".to_owned()],
            vec!["@BoundaryMark(value = 4)".to_owned()],
            vec!["@BoundaryMark(value = 5)".to_owned()],
        ]
    );
    assert!(method.declaration.as_deref().unwrap().contains(
        "wideAndVarargs(@BoundaryMark(value = 3) long arg1, @BoundaryMark(value = 4) double arg3, @BoundaryMark(value = 5) java.lang.String... arg5)"
    ));

    let same_type = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"sameTypeAtDistinctPositions")
        .expect("same type at distinct positions");
    assert_eq!(same_type.annotations.uses, ["@BoundaryMark(value = 6)"]);
    assert_eq!(
        same_type.parameter_annotations.uses_by_position,
        [
            vec!["@BoundaryMark(value = 7)".to_owned()],
            vec!["@BoundaryMark(value = 8)".to_owned()],
        ]
    );
    assert!(same_type.annotations.refusals.is_empty());
    assert!(same_type.parameter_annotations.refusals.is_empty());
}

#[test]
fn parameter_count_mismatch_refuses_groups_without_shifting_other_member_annotations() {
    let report = class_source_of(
        &open(MEMBER_BOUNDARY_COUNT_MISMATCH.to_vec()),
        "BoundaryTagged",
        EnvironmentPolicy::SingleClass,
    );
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"wideAndVarargs")
        .expect("patched wide varargs method");
    assert_eq!(method.annotations.uses, ["@BoundaryMark(value = 2)"]);
    assert_eq!(
        method.parameter_annotations.attributes[0].parameter_count,
        Some(2)
    );
    assert_eq!(
        method.parameter_annotations.uses_by_position,
        [Vec::<String>::new(), Vec::new(), Vec::new()]
    );
    assert!(method.parameter_annotations.refusals.iter().any(|refusal| {
        refusal.contains("declares 2 position(s) for a descriptor with 3 parameter(s)")
    }));
    let declaration = method.declaration.as_deref().unwrap();
    assert!(!declaration.contains("@BoundaryMark(value = 3)"));
    assert!(!declaration.contains("@BoundaryMark(value = 4)"));
    assert!(!declaration.contains("@BoundaryMark(value = 5)"));
    assert!(
        report
            .text
            .contains("// jarde: parameter annotation refused:")
    );
}

#[test]
fn repeated_member_annotation_type_is_refused_atomically_at_its_position() {
    let report = class_source_of(
        &open(MEMBER_BOUNDARY_DUPLICATE.to_vec()),
        "BoundaryTagged",
        EnvironmentPolicy::SingleClass,
    );
    let field = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"field")
        .expect("duplicated field annotation");
    assert!(field.annotations.uses.is_empty());
    assert_eq!(field.annotations.refusals.len(), 2);
    assert_eq!(field.annotations.attributes[0].annotations.len(), 2);
    assert!(
        field
            .markers
            .iter()
            .any(|marker| { marker.contains("duplicate annotation type `LBoundaryMark;`") })
    );
    assert!(!report.text.contains("@BoundaryMark(value = 1)"));

    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"wideAndVarargs")
        .expect("distinct method position");
    assert_eq!(method.annotations.uses, ["@BoundaryMark(value = 2)"]);
}

#[test]
fn unspellable_member_annotation_type_and_element_refuse_the_whole_use() {
    let mut invalid_type = MEMBER_BOUNDARY_TARGET.to_vec();
    let type_descriptor = b"LBoundaryMark;";
    let type_at = invalid_type
        .windows(type_descriptor.len())
        .position(|window| window == type_descriptor)
        .expect("annotation type descriptor in the constant pool");
    invalid_type[type_at..type_at + type_descriptor.len()].copy_from_slice(b"LBoundary-XXX;");
    let report = class_source_of(
        &open(invalid_type),
        "BoundaryTagged",
        EnvironmentPolicy::SingleClass,
    );
    let field = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"field")
        .expect("annotation field");
    assert!(field.annotations.uses.is_empty());
    assert!(field.annotations.refusals[0].contains("not a Java source name"));
    assert_eq!(field.annotations.attributes[0].annotations.len(), 1);

    let mut invalid_element = MEMBER_BOUNDARY_TARGET.to_vec();
    let element_name = b"value";
    let name_at = invalid_element
        .windows(element_name.len())
        .position(|window| window == element_name)
        .expect("annotation element name in the constant pool");
    invalid_element[name_at..name_at + element_name.len()].copy_from_slice(b"bad-n");
    let report = class_source_of(
        &open(invalid_element),
        "BoundaryTagged",
        EnvironmentPolicy::SingleClass,
    );
    let field = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"field")
        .expect("annotation field");
    assert!(field.annotations.uses.is_empty());
    assert!(field.annotations.refusals[0].contains("faithful Java spelling"));
    assert!(!report.text.contains("@BoundaryMark("));
}

#[test]
fn damaged_member_annotation_keeps_its_shell_and_other_attribute_groups() {
    let mut bytes = MEMBER_ANNOTATION_TARGET.to_vec();
    let mut read_budget = budget();
    let class = jarde_reader::classfile::class_facts(&bytes, &mut read_budget)
        .expect("member class structure");
    let annotation = class
        .methods
        .iter()
        .find(|method| method.name.raw().0 == b"value")
        .and_then(|method| {
            method
                .attributes
                .iter()
                .find(|attribute| attribute.name.raw().0 == b"RuntimeVisibleAnnotations")
        })
        .expect("method declaration annotation shell");
    let type_index = usize::try_from(annotation.content_span.start).unwrap() + 2;
    bytes[type_index..type_index + 2].copy_from_slice(&u16::MAX.to_be_bytes());

    let report = class_source_of(&open(bytes), "MemberTagged", EnvironmentPolicy::SingleClass);
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"value")
        .expect("method declaration");
    assert_eq!(method.annotations.attributes.len(), 1);
    assert_eq!(
        method.annotations.attributes[0].attribute.name.raw().0,
        b"RuntimeVisibleAnnotations"
    );
    assert!(method.annotations.attributes[0].annotations.is_empty());
    assert!(method.annotations.refusals[0].contains("attribute read stopped"));
    assert_eq!(
        method.parameter_annotations.uses_by_position,
        [vec!["@java.lang.Deprecated".to_owned()]]
    );
    assert!(report.text.contains("// jarde: member annotation refused:"));
    assert!(report.text.contains("@java.lang.Deprecated int arg1"));
    assert!(matches!(
        report.execution,
        ExecutionReport::Failed { .. } | ExecutionReport::Partial { .. }
    ));
}

fn hidden_annotation_shell(bytes: &[u8]) -> AttributeShell {
    let mut read_budget = budget();
    let facts = class_facts(bytes, &mut read_budget).expect("class structure");
    facts
        .attributes
        .into_iter()
        .find(|attribute| attribute.name.raw().0 == b"RuntimeInvisibleAnnotations")
        .expect("class-retention annotation shell")
}

#[test]
fn unspellable_class_annotation_is_refused_as_a_whole() {
    let mut bytes = CLASS_RETENTION_TARGET.to_vec();
    let descriptor = b"LHiddenTag;";
    let at = bytes
        .windows(descriptor.len())
        .position(|window| window == descriptor)
        .expect("annotation descriptor in the pool");
    bytes[at..at + descriptor.len()].copy_from_slice(b"LHidden-xx;");
    let report = class_source_of(&open(bytes), "HiddenTarget", EnvironmentPolicy::SingleClass);
    let declaration = report.declaration.expect("class declaration");
    assert!(declaration.annotation_uses.is_empty());
    assert_eq!(declaration.annotation_refusals.len(), 1);
    assert!(declaration.annotation_refusals[0].contains("not a Java source name"));
    assert!(report.text.contains("// jarde: class annotation refused:"));
}

#[test]
fn unspellable_class_annotation_element_name_refuses_the_whole_annotation() {
    let mut bytes = CLASS_RETENTION_TARGET.to_vec();
    let name = b"value";
    let at = bytes
        .windows(name.len())
        .position(|window| window == name)
        .expect("annotation element name in the pool");
    bytes[at..at + name.len()].copy_from_slice(b"bad-n");
    let report = class_source_of(&open(bytes), "HiddenTarget", EnvironmentPolicy::SingleClass);
    let declaration = report.declaration.expect("class declaration");
    assert!(declaration.annotation_uses.is_empty());
    assert_eq!(declaration.annotation_refusals.len(), 1);
    assert!(report.text.contains("// jarde: class annotation refused:"));
}

#[test]
fn duplicate_class_annotation_entries_are_refused_without_partial_source() {
    let mut bytes = CLASS_RETENTION_TARGET.to_vec();
    let shell = hidden_annotation_shell(&bytes);
    let start = usize::try_from(shell.content_span.start).unwrap();
    let length = usize::try_from(shell.content_span.length).unwrap();
    assert_eq!(u16::from_be_bytes([bytes[start], bytes[start + 1]]), 1);
    let entry = bytes[start + 2..start + length].to_vec();
    bytes[start..start + 2].copy_from_slice(&2_u16.to_be_bytes());
    let content_end = start + length;
    bytes.splice(content_end..content_end, entry.iter().copied());
    let shell_start = usize::try_from(shell.span.start).unwrap();
    let old_attribute_length =
        u32::from_be_bytes(bytes[shell_start + 2..shell_start + 6].try_into().unwrap());
    bytes[shell_start + 2..shell_start + 6].copy_from_slice(
        &(old_attribute_length + u32::try_from(entry.len()).unwrap()).to_be_bytes(),
    );

    let report = class_source_of(&open(bytes), "HiddenTarget", EnvironmentPolicy::SingleClass);
    let declaration = report.declaration.expect("class declaration");
    assert!(declaration.annotation_uses.is_empty());
    assert_eq!(declaration.annotation_refusals.len(), 2);
    assert!(
        declaration
            .annotation_refusals
            .iter()
            .all(|refusal| refusal.contains("duplicate annotation type"))
    );
    assert!(!report.text.contains("@HiddenTag("));
}

#[test]
fn damaged_class_annotation_attribute_is_reported_as_a_stop() {
    let mut bytes = CLASS_RETENTION_TARGET.to_vec();
    let shell = hidden_annotation_shell(&bytes);
    let value_tag = usize::try_from(shell.content_span.start).unwrap() + 8;
    bytes[value_tag] = b'Q';
    let report = class_source_of(&open(bytes), "HiddenTarget", EnvironmentPolicy::SingleClass);
    let declaration = report.declaration.expect("class declaration remains known");
    assert!(declaration.annotation_uses.is_empty());
    assert_eq!(declaration.annotation_attributes.len(), 1);
    assert!(declaration.annotation_attributes[0].annotations.is_empty());
    assert!(declaration.annotation_refusals[0].contains("classfile_invalid_attribute_content"));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "classfile_invalid_attribute_content" })
    );
    assert!(!matches!(
        report.execution,
        ExecutionReport::Complete { .. }
    ));
}

/// The recovery artifact of one member, for the one case above that compares the two texts.
trait IntoRecoveryText {
    fn into_recovery_text(self) -> String;
}

impl IntoRecoveryText for ClassSourceOutcome {
    fn into_recovery_text(self) -> String {
        match self {
            ClassSourceOutcome::Recovered { report, .. } => report.text,
            ClassSourceOutcome::NoBody
            | ClassSourceOutcome::Unspelled
            | ClassSourceOutcome::Refused { .. } => String::new(),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The input shapes a class is prepared from
// ---------------------------------------------------------------------------------------------

/// A class in an archive is prepared from the very entry the binding bound, in both container
/// shapes: one under a declared entry prefix (`WEB-INF/classes/`, the WAR class layer) and one
/// inside a nested library the artifact tree reaches.
///
/// Both are presented from the physical definition the name search confirmed, under the environment
/// that declares the position the class really lives in — a prefix root for the class layer, the
/// nested container's own origin for the library — so the preparation reads the entry the binding
/// read and the bodies recover against it. The class bytes are read twice, exactly as for a
/// standalone class: the member count does not enter the class-read shape.
#[test]
fn a_class_in_a_container_is_prepared_from_the_entry_it_lives_in() {
    let class = many_bodies_class(b"p/Container", 2);
    let nested_class = many_bodies_class(b"p/Nested", 2);
    let library = zip_of(&[(b"p/Nested.class", &nested_class)]);
    let snapshot = open(zip_of(&[
        (b"WEB-INF/classes/p/Container.class", &class),
        (b"WEB-INF/lib/L.jar", &library),
        (b"META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\n"),
    ]));
    let engine = Engine::new();

    // The WAR class layer: a container root with the entry prefix the class layer really has.
    let class_layer = performed(
        engine
            .class_source(
                slice::from_ref(&snapshot),
                &scoped_request(
                    &snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal("p/Container"),
                    },
                    EnvironmentPolicy::ExplicitClasspath {
                        roots: vec![LoadRoot::Container {
                            origin: root_origin(&snapshot),
                            prefix: ArchiveNameBytes(b"WEB-INF/classes/".to_vec()),
                        }],
                    },
                    PhysicalScope::SnapshotAll,
                ),
                &mut budget(),
            )
            .expect("a legal request is answered"),
    );
    assert_eq!(
        class_layer
            .class
            .location
            .entry()
            .expect("an entry")
            .raw_name
            .0,
        b"WEB-INF/classes/p/Container.class"
    );
    // One class read for the whole request (D2 3.2): the search read the definition it elected,
    // and the one preparation is built over *that* read instead of reading the same entry again.
    assert_eq!(
        class_layer.usage.class_headers, 1,
        "{:?}",
        class_layer.usage
    );
    assert_eq!(
        class_layer.usage.method_bodies, 2,
        "{:?}",
        class_layer.usage
    );
    assert_eq!(
        class_layer.usage.class_bytes,
        2 * class_layer.class.class_bytes.length
    );
    assert!(
        class_layer.text.contains("        return;\n"),
        "{}",
        class_layer.text
    );

    // The nested library: the scope is the whole artifact tree, so the search reaches the entry
    // inside `WEB-INF/lib/L.jar`, and the declared position is that nested container's own origin.
    let nested = performed(
        engine
            .class_source(
                slice::from_ref(&snapshot),
                &scoped_request(
                    &snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal("p/Nested"),
                    },
                    EnvironmentPolicy::ExplicitClasspath {
                        roots: vec![nested_root(&snapshot, b"WEB-INF/lib/L.jar")],
                    },
                    PhysicalScope::ArtifactTree {
                        root_container: ContainerId("root".into()),
                    },
                ),
                &mut budget(),
            )
            .expect("a legal request is answered"),
    );
    let entry = nested.class.location.entry().expect("an entry");
    assert_eq!(entry.raw_name.0, b"p/Nested.class");
    assert_eq!(
        entry.origin.steps.len(),
        1,
        "the class is one container deep"
    );
    assert!(
        nested
            .coverage
            .artifact_structural
            .scanned
            .iter()
            .any(|range| range.label == "navigation_candidates"),
        "the search walked the tree scope: {:?}",
        nested.coverage
    );
    assert_eq!(nested.usage.class_headers, 1, "{:?}", nested.usage);
    assert_eq!(nested.usage.method_bodies, 2, "{:?}", nested.usage);
    assert_eq!(
        nested.usage.class_bytes,
        2 * nested.class.class_bytes.length
    );
    assert!(
        nested
            .text
            .contains("public class Nested extends java.lang.Object {\n")
    );
    assert!(nested.text.contains("        return;\n"), "{}", nested.text);
    assert_eq!(
        nested.execution,
        ExecutionReport::Complete {
            usage: nested.usage.clone()
        }
    );
}

/// A class the reader's strict structure read refuses — a class whose bytes the tolerant class read
/// accepts but whose whole structure does not decode — keeps its presentation, and each member that
/// declares a body states the preparation's own failure.
///
/// This is what the one preparation owes a class like this: the declaration, the fields and every
/// member declaration stay published (the binding read them), and no body is invented or attempted
/// from a class that could not be prepared. The class reads are the same two as for a healthy class,
/// and one diagnostic per member that declares a body names the reader's own code.
#[test]
fn a_class_that_cannot_be_prepared_keeps_its_presentation() {
    let mut bytes = many_bodies_class(b"p/Trailing", 2);
    bytes.push(0);
    let report = class_source_of(&open(bytes), "p/Trailing", EnvironmentPolicy::SingleClass);

    assert!(report.declaration.is_some(), "the class is presented");
    assert_eq!(report.methods.len(), 3, "{:?}", report.methods);
    assert!(
        report
            .text
            .contains("public class Trailing extends java.lang.Object {\n")
    );
    assert!(report.text.contains("public abstract void declaredOnly();"));
    assert!(braces_balance(&report.text) == 0, "{}", report.text);

    let refused: Vec<&ClassSourceMethod> = report
        .methods
        .iter()
        .filter(|method| matches!(method.outcome, ClassSourceOutcome::Refused { .. }))
        .collect();
    assert_eq!(refused.len(), 2, "every body-bearing member states it");
    for method in refused {
        let ClassSourceOutcome::Refused { diagnostics, .. } = &method.outcome else {
            unreachable!("filtered on the refusal above")
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "classfile_trailing_bytes"),
            "the member's own refusal names the reader's code: {diagnostics:?}"
        );
        assert!(
            method
                .markers
                .iter()
                .any(|marker| marker.contains("classfile_trailing_bytes")),
            "{:?}",
            method.markers
        );
    }
    // No body was attempted, and the failure is the request's own plane rather than a success. The
    // class itself was read once — the binding read, over which the preparation that refused these
    // members was attempted (D2 3.2) — and not once per member that states the refusal.
    assert_eq!(report.usage.method_bodies, 0, "{:?}", report.usage);
    assert_eq!(report.usage.class_headers, 1, "{:?}", report.usage);
    assert!(!matches!(
        report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
}

// ---------------------------------------------------------------------------------------------
// The read shape: one preparation, one decode per body
// ---------------------------------------------------------------------------------------------

/// The read shape of one presentation of one class: **one** class read whatever the member count —
/// the binding read, which is also the read the one preparation is built over (D2 3.2) — and one
/// decode per member that declares a body.
///
/// This is the assertion task 7.3 asks for, and it is deliberately a *shape* assertion rather than a
/// count of one fixture: two classes whose member counts differ (6 and 9, each with one member that
/// declares no body) are presented under the whole task budget, and the class reads must be the same
/// one while the body attempts follow the bodies. A return to a per-member run reads the class once
/// per member and fails here: `class_headers` would be 1 + 5 and 1 + 8, and `class_bytes` would grow
/// by a class per member.
#[test]
fn one_preparation_serves_every_member_body() {
    let six = class_source_of(
        &open(many_bodies_class(b"p/Six", 5)),
        "p/Six",
        EnvironmentPolicy::SingleClass,
    );
    let nine = class_source_of(
        &open(many_bodies_class(b"p/Nine", 8)),
        "p/Nine",
        EnvironmentPolicy::SingleClass,
    );

    for (report, bodies) in [(&six, 5_u64), (&nine, 8_u64)] {
        // One class read for the binding, which the one preparation is built over (D2 3.2):
        // `class_headers` is the same one for both classes, and the class bytes are parsed exactly
        // twice — the binding's own member walk and the preparation, each counted once for the
        // class's own length, never once per member.
        assert_eq!(report.usage.class_headers, 1, "{:?}", report.usage);
        assert_eq!(report.usage.method_bodies, bodies, "{:?}", report.usage);
        assert_eq!(
            report.usage.class_bytes,
            2 * report.class.class_bytes.length,
            "the class bytes are parsed by the binding's member walk and by the one preparation \
             built over that same read — two parses of one read, not two reads: {:?}",
            report.usage
        );
        // Every member that declares a body really ran, and the member that declares none is the
        // declaration this presentation writes for it.
        assert_eq!(
            report
                .methods
                .iter()
                .filter(|method| matches!(method.outcome, ClassSourceOutcome::Recovered { .. }))
                .count() as u64,
            bodies
        );
        assert_eq!(
            report
                .methods
                .iter()
                .filter(|method| method.outcome == ClassSourceOutcome::NoBody)
                .count(),
            1
        );
        // The class is presented whole: no member is dropped for having been read through the
        // preparation, and every body's text is in the text.
        assert_eq!(report.methods.len() as u64, bodies + 1);
        assert!(report.text.contains("// jarde: presentation of `p/"));
        assert!(report.text.contains("    public static void body0() {\n"));
        assert!(
            report
                .text
                .contains(&format!("    public static void body{}() {{\n", bodies - 1)),
            "{}",
            report.text
        );
        assert!(braces_balance(&report.text) == 0, "{}", report.text);
    }
    // The two classes differ in member count and not in what one request reads of a class.
    assert_eq!(six.usage.class_headers, nine.usage.class_headers);
    assert!(
        nine.usage.method_bodies > six.usage.method_bodies,
        "the bodies really are more: {:?} vs {:?}",
        six.usage,
        nine.usage
    );

    // And no member's own run read the class: the run after the first adds exactly one body attempt
    // and no class header at all, member after member.
    for (report, bodies) in [(&six, 5_u64), (&nine, 8_u64)] {
        let runs: Vec<&UsageSnapshot> = report
            .methods
            .iter()
            .filter_map(|method| match &method.outcome {
                ClassSourceOutcome::Recovered { analysis, .. } => {
                    Some(usage_of(&analysis.execution))
                }
                ClassSourceOutcome::NoBody
                | ClassSourceOutcome::Unspelled
                | ClassSourceOutcome::Refused { .. } => None,
            })
            .collect();
        assert_eq!(runs.len() as u64, bodies, "every body-bearing member ran");
        for pair in runs.windows(2) {
            assert_eq!(
                pair[1].class_headers, pair[0].class_headers,
                "a member's run charged a class read of its own: {:?}",
                pair[1]
            );
            assert_eq!(
                pair[1].method_bodies,
                pair[0].method_bodies + 1,
                "a member's run is one body attempt: {:?}",
                pair[1]
            );
        }
    }
}
