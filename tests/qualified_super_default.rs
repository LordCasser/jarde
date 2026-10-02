//! `recover-qualified-super-default-calls`: a qualified super call on a **nested** default method
//! (`A.super.name()` spelled by the binary name the class file carries) is presented when the
//! qualifier is a direct superinterface and the target is its non-abstract instance method.
//!
//! The committed Java 8 fixture is a real compiler output: `Diamond` proves the two qualifiers of
//! the diamond shape independently, `Single` proves the void and parameterized defaults, and the
//! packaged `pkg.PackedSuper` family pins the dotted package spelling. The two refusal shapes are
//! *not* compiler outputs — `javac` refuses a qualifier that is only inherited and a target that is
//! abstract — so each starts from its legal neighbour class and one constant-pool index is repointed
//! in place, the same discipline as `p3_special_refusal`: every patch asserts the frozen bytes it
//! stands on, and the negative stays a refusal for its own reason.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::slice;

const NESTED: &[u8] = include_bytes!("fixtures/qualified-super-default/v8/NestedSuper.class");
const NESTED_A: &[u8] = include_bytes!("fixtures/qualified-super-default/v8/NestedSuper$A.class");
const NESTED_B: &[u8] = include_bytes!("fixtures/qualified-super-default/v8/NestedSuper$B.class");
const NESTED_SUB: &[u8] =
    include_bytes!("fixtures/qualified-super-default/v8/NestedSuper$Sub.class");
const NESTED_ABS: &[u8] =
    include_bytes!("fixtures/qualified-super-default/v8/NestedSuper$Abs.class");
const NESTED_DIAMOND: &[u8] =
    include_bytes!("fixtures/qualified-super-default/v8/NestedSuper$Diamond.class");
const NESTED_SINGLE: &[u8] =
    include_bytes!("fixtures/qualified-super-default/v8/NestedSuper$Single.class");
const NESTED_INDIRECT: &[u8] =
    include_bytes!("fixtures/qualified-super-default/v8/NestedSuper$Indirect.class");
const NESTED_ABSTRACT: &[u8] =
    include_bytes!("fixtures/qualified-super-default/v8/NestedSuper$Abstract.class");
const PACKED: &[u8] = include_bytes!("fixtures/qualified-super-default/v8/pkg/PackedSuper.class");
const PACKED_I: &[u8] =
    include_bytes!("fixtures/qualified-super-default/v8/pkg/PackedSuper$I.class");
const PACKED_USE: &[u8] =
    include_bytes!("fixtures/qualified-super-default/v8/pkg/PackedSuper$Use.class");

fn budget() -> Budget {
    task_budget(&[]).expect("默认任务预算有界")
}

fn jar(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .expect("start fixture jar entry");
            let mut writer = config.wrap(&mut entry);
            writer.write_all(bytes).expect("write fixture jar entry");
            let (_, descriptor) = writer.finish().expect("finish fixture jar entry");
            entry.finish(descriptor).expect("finish fixture jar record");
        }
        archive.finish().expect("finish fixture jar");
    }
    output.into_inner()
}

fn nested_family() -> Vec<u8> {
    jar(&[
        (b"NestedSuper.class", NESTED),
        (b"NestedSuper$A.class", NESTED_A),
        (b"NestedSuper$B.class", NESTED_B),
        (b"NestedSuper$Sub.class", NESTED_SUB),
        (b"NestedSuper$Abs.class", NESTED_ABS),
        (b"NestedSuper$Diamond.class", NESTED_DIAMOND),
        (b"NestedSuper$Single.class", NESTED_SINGLE),
        (b"NestedSuper$Indirect.class", NESTED_INDIRECT),
        (b"NestedSuper$Abstract.class", NESTED_ABSTRACT),
    ])
}

fn packed_family() -> Vec<u8> {
    jar(&[
        (b"pkg/PackedSuper.class", PACKED),
        (b"pkg/PackedSuper$I.class", PACKED_I),
        (b"pkg/PackedSuper$Use.class", PACKED_USE),
    ])
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget())
        .expect("限定 super 默认调用 fixture 可作为 plain JAR 打开")
}

fn class_source_of(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
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
            loader: LoaderId("app".to_owned()),
        },
    };
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("合法的 class-source 请求得到回答")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("一个提交的 fixture 应只回答一个定义，却得到 {other:?}"),
    }
}

fn method_text<'a>(report: &'a ClassSourceReport, class: &str, name: &str) -> &'a str {
    &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("方法 `{name}` 不在 {class} 的方法表中"))
        .text
}

#[test]
fn nested_diamond_qualifiers_are_each_independently_presented() {
    let snapshot = open(nested_family());
    let report = class_source_of(&snapshot, "NestedSuper$Diamond");
    let name = method_text(&report, "NestedSuper$Diamond", "name");
    assert!(
        name.contains("return NestedSuper$A.super.name() + NestedSuper$B.super.name();"),
        "菱形双限定调用按各自证明呈现：{name}"
    );
}

#[test]
fn nested_single_void_and_parameterized_defaults_are_presented() {
    let snapshot = open(nested_family());
    let report = class_source_of(&snapshot, "NestedSuper$Single");
    let name = method_text(&report, "NestedSuper$Single", "name");
    assert!(
        name.contains("\"I:\" + NestedSuper$A.super.name()"),
        "单限定默认调用呈现：{name}"
    );
    let run_log = method_text(&report, "NestedSuper$Single", "runLog");
    assert!(
        run_log.contains("NestedSuper$A.super.log(\"hi\");"),
        "void 默认方法的限定调用呈现为语句：{run_log}"
    );
    let run_greet = method_text(&report, "NestedSuper$Single", "runGreet");
    assert!(
        run_greet.contains("NestedSuper$A.super.greet(7)"),
        "带参默认方法的限定调用呈现：{run_greet}"
    );
}

#[test]
fn packaged_nested_qualifier_is_spelled_with_the_package_dot() {
    let snapshot = open(packed_family());
    let report = class_source_of(&snapshot, "pkg/PackedSuper$Use");
    let name = method_text(&report, "pkg/PackedSuper$Use", "name");
    assert!(
        name.contains("return pkg.PackedSuper$I.super.name();"),
        "包内嵌套限定符按类头的同名拼写呈现：{name}"
    );
}

/// One walk of a class file's constant pool: the (tag, payload) of each entry plus its byte range,
/// enough for the two index patches below. Every unknown tag is a hard error, so a fixture change
/// that invalidates a patch fails here instead of silently patching the wrong slot.
fn constant_pool(bytes: &[u8]) -> (Vec<(u8, Vec<u8>)>, Vec<(usize, usize)>) {
    fn u16_at(bytes: &[u8], at: usize) -> usize {
        u16::from_be_bytes([bytes[at], bytes[at + 1]]) as usize
    }
    let count = u16_at(bytes, 8);
    let mut entries = Vec::with_capacity(count);
    let mut ranges = Vec::with_capacity(count);
    let mut index = 1;
    let mut offset = 10;
    while index < count {
        let tag = bytes[offset];
        let size = match tag {
            1 => 3 + u16_at(bytes, offset + 1),
            3 | 4 => 5,
            5 | 6 => 9,
            7 | 8 | 16 | 19 | 20 => 3,
            9 | 10 | 11 | 12 | 17 | 18 => 5,
            15 => 4,
            other => panic!("未知的常量池 tag {other}（偏移 {offset}）"),
        };
        entries.push((tag, bytes[offset..offset + size].to_vec()));
        ranges.push((offset, offset + size));
        index += if matches!(tag, 5 | 6) { 2 } else { 1 };
        offset += size;
    }
    (entries, ranges)
}

fn utf8_of(entries: &[(u8, Vec<u8>)], index: usize) -> &[u8] {
    let (tag, payload) = &entries[index - 1];
    assert_eq!(*tag, 1, "条目 {index} 应是 Utf8");
    &payload[3..]
}

/// The one constant-pool index of the `CONSTANT_Class` entry whose name is `name`.
fn class_entry_of(entries: &[(u8, Vec<u8>)], name: &[u8]) -> usize {
    let mut found = None;
    for (position, (tag, payload)) in entries.iter().enumerate() {
        if *tag != 7 {
            continue;
        }
        let name_index = u16::from_be_bytes([payload[1], payload[2]]) as usize;
        if utf8_of(entries, name_index) == name {
            assert!(found.is_none(), "类条目 {:?} 应唯一", name);
            found = Some(position + 1);
        }
    }
    found.unwrap_or_else(|| panic!("常量池应有类条目 {name:?}"))
}

fn patch_interfaces_slot(bytes: &mut [u8], this_name: &[u8], from_name: &[u8], to_name: &[u8]) {
    let (entries, ranges) = constant_pool(bytes);
    let target = class_entry_of(&entries, to_name);
    let pool_end = ranges.last().expect("常量池非空").1;
    let this_index = u16::from_be_bytes([bytes[pool_end + 2], bytes[pool_end + 3]]) as usize;
    let (tag, payload) = &entries[this_index - 1];
    assert_eq!(*tag, 7, "this_class 应指向类条目");
    let this_utf8 = u16::from_be_bytes([payload[1], payload[2]]) as usize;
    assert_eq!(
        utf8_of(&entries, this_utf8),
        this_name,
        "接口槽补丁只应写在其声明的类上"
    );
    let count = u16::from_be_bytes([bytes[pool_end + 6], bytes[pool_end + 7]]) as usize;
    let mut slots = Vec::new();
    for slot in 0..count {
        let at = pool_end + 8 + 2 * slot;
        let entry = u16::from_be_bytes([bytes[at], bytes[at + 1]]) as usize;
        let (tag, payload) = &entries[entry - 1];
        assert_eq!(*tag, 7, "接口槽应指向类条目");
        let name_index = u16::from_be_bytes([payload[1], payload[2]]) as usize;
        if utf8_of(&entries, name_index) == from_name {
            slots.push(at);
        }
    }
    let [at] = slots.as_slice() else {
        panic!("应恰好一个 {from_name:?} 接口槽");
    };
    bytes[*at..*at + 2].copy_from_slice(&(target as u16).to_be_bytes());
}

fn patch_interface_methodref_owner(
    bytes: &mut [u8],
    from_owner: &[u8],
    to_owner: &[u8],
    name: &[u8],
) {
    let (entries, ranges) = constant_pool(bytes);
    let target = class_entry_of(&entries, to_owner);
    let from_index = class_entry_of(&entries, from_owner);
    let mut hits = Vec::new();
    for (position, (tag, payload)) in entries.iter().enumerate() {
        if *tag != 11 {
            continue;
        }
        let owner = u16::from_be_bytes([payload[1], payload[2]]) as usize;
        if owner != from_index {
            continue;
        }
        let name_and_type = u16::from_be_bytes([payload[3], payload[4]]) as usize;
        let (_, nat_payload) = &entries[name_and_type - 1];
        let method = u16::from_be_bytes([nat_payload[1], nat_payload[2]]) as usize;
        if utf8_of(&entries, method) == name {
            hits.push(ranges[position].0);
        }
    }
    let [offset] = hits.as_slice() else {
        panic!("应恰好一个 {from_owner:?}.{name:?} 的 InterfaceMethodref");
    };
    bytes[offset + 1..offset + 3].copy_from_slice(&(target as u16).to_be_bytes());
}

#[test]
fn qualifier_inherited_only_through_extends_keeps_the_refusal() {
    // The legal neighbour: `Indirect implements A` calls `A.super.name()` with `A` direct. The
    // patch moves the header's one interface slot to `Sub` (which `extends A`), so `A` is now an
    // indirect superinterface — a shape javac refuses and HotSpot's own verifier rejects ("Bad
    // invokespecial instruction: interface method reference is in an indirect superinterface"),
    // so the class never loads; the refusal below is what a reader of the bytes must still state.
    let mut bytes = NESTED_INDIRECT.to_vec();
    patch_interfaces_slot(
        &mut bytes,
        b"NestedSuper$Indirect",
        b"NestedSuper$A",
        b"NestedSuper$Sub",
    );
    let snapshot = open(jar(&[
        (b"NestedSuper.class", NESTED),
        (b"NestedSuper$A.class", NESTED_A),
        (b"NestedSuper$Sub.class", NESTED_SUB),
        (b"NestedSuper$Indirect.class", &bytes),
    ]));
    let report = class_source_of(&snapshot, "NestedSuper$Indirect");
    let name = method_text(&report, "NestedSuper$Indirect", "name");
    assert!(
        name.contains("names `NestedSuper$A` without a matching direct supertype"),
        "经 extends 间接继承的限定符保持拒绝：{name}"
    );
    assert!(
        !name.contains("super.name()"),
        "间接限定符不得呈现为限定调用：{name}"
    );
}

#[test]
fn abstract_target_in_a_direct_interface_keeps_the_refusal() {
    // The legal neighbour: `Abstract implements Abs, A` calls `A.super.name()` (both direct, `A`
    // concrete). The patch repoints the call's `InterfaceMethodref` owner to `Abs`, whose own
    // member table declares `name()` abstract — verifier-valid (the method resolves; calling it
    // raises AbstractMethodError) but no legal Java 8 source.
    let mut bytes = NESTED_ABSTRACT.to_vec();
    patch_interface_methodref_owner(&mut bytes, b"NestedSuper$A", b"NestedSuper$Abs", b"name");
    let snapshot = open(jar(&[
        (b"NestedSuper.class", NESTED),
        (b"NestedSuper$A.class", NESTED_A),
        (b"NestedSuper$Abs.class", NESTED_ABS),
        (b"NestedSuper$Abstract.class", &bytes),
    ]));
    let report = class_source_of(&snapshot, "NestedSuper$Abstract");
    let name = method_text(&report, "NestedSuper$Abstract", "name");
    assert!(
        name.contains("no selected proof of a legal source qualifier and unique default binding"),
        "直接接口中的 abstract 目标保持证明拒绝：{name}"
    );
    assert!(
        !name.contains("super.name()"),
        "abstract 目标不得呈现为限定调用：{name}"
    );
}
