//! Acceptance tests for `recover-raw-receiver-field-selection`.
//!
//! The inputs are class-only jars frozen from Corretto 8u432. The source fixtures and hashes are
//! retained beside the four-leg JDK/JADX/Jarde replay under `openspec/evidence/`.

use jarde::class_source::{ClassSourceField, ClassSourceReport};
use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const RAW_PARAM: &[u8] = include_bytes!("fixtures/raw-receiver-field-selection/RawParam.jar");
const STATIC_RAW_LOCAL: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/StaticRawLocal.jar");
const INSTANCE_RAW_LOCAL: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/InstanceRawLocal.jar");
const TYPED_RECEIVER: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/TypedReceiver.jar");
const THIS_RECEIVER: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/ThisReceiver.jar");
const SHADOW_METHOD_T: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/ShadowMethodT.jar");
const DIRECT_INSTANCE_RAW_PARAM: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/DirectInstanceRawParam.jar");
const WIDE_RAW_PARAM: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/WideRawParam.jar");
const ARRAY_RAW_PARAM: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/ArrayRawParam.jar");
const ARRAY_2D_RAW_PARAM: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/Array2DRawParam.jar");
const PRIMITIVE_ARRAY_RAW_PARAM: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/PrimitiveArrayRawParam.jar");
const BOUND_RAW_PARAM: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/BoundRawParam.jar");
const NULL_RAW_PARAM: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/NullRawParam.jar");
const RETAINED_ALIAS_RAW_PARAM: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/RetainedAliasRawParam.jar");
const MULTI_FORMAL_RAW_PARAM: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/MultiFormalRawParam.jar");
const RAW_OWNER_CHILD: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/RawOwnerChild.jar");
const UNKNOWN_WRITER: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/UnknownWriter.jar");
const UNSAFE_OTHER_WRITER: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/UnsafeOtherWriter.jar");
const SYNTHETIC_ACCESSOR_GUARD_CLASS: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/SyntheticAccessorGuard.class");
const REBOUND_ALIAS_RAW_PARAM: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/ReboundAliasRawParam.jar");
const NESTED_ALIAS_RAW_PARAM: &[u8] =
    include_bytes!("fixtures/raw-receiver-field-selection/NestedAliasRawParam.jar");

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-raw-receiver-{label}-{}-{nonce}-{}",
            std::process::id(),
            NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("the temporary directory is created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn jar_of(class_name: &str, class_bytes: &[u8]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut zip = ZipArchiveWriter::new(&mut output);
        let (mut entry, config) = zip
            .new_file(EntryPath::verbatim(
                format!("{class_name}.class").into_bytes(),
            ))
            .compression_method(CompressionMethod::new(0))
            .start()
            .expect("the class-only jar entry starts");
        let mut writer = config.wrap(&mut entry);
        writer
            .write_all(class_bytes)
            .expect("the class bytes write");
        let (_, descriptor) = writer.finish().expect("the class entry closes");
        entry
            .finish(descriptor)
            .expect("the class entry is finalized");
        zip.finish().expect("the class-only jar closes");
    }
    output.into_inner()
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn u32_at(bytes: &[u8], offset: usize) -> usize {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]) as usize
}

fn skip_attributes(bytes: &[u8], cursor: &mut usize, count: u16) {
    for _ in 0..count {
        let length = u32_at(bytes, *cursor + 2);
        *cursor += 6 + length;
    }
}

fn skip_members(bytes: &[u8], cursor: &mut usize, count: u16) {
    for _ in 0..count {
        let attributes = u16_at(bytes, *cursor + 6);
        *cursor += 8;
        skip_attributes(bytes, cursor, attributes);
    }
}

/// Adds ACC_SYNTHETIC to the real Corretto 8 static setter, after checking its exact four-opcode
/// `(aload_0, aload_1, putfield, return)` shape. The class itself is a compiled source fixture;
/// only this access flag is the intentional synthetic-accessor mutation.
fn synthetic_setter_flags(class_bytes: &[u8]) -> Vec<u8> {
    assert_eq!(&class_bytes[..4], &[0xca, 0xfe, 0xba, 0xbe]);
    let cp_count = u16_at(class_bytes, 8) as usize;
    let mut utf8 = vec![None::<Vec<u8>>; cp_count];
    let mut cursor = 10;
    let mut index = 1;
    while index < cp_count {
        let tag = class_bytes[cursor];
        cursor += 1;
        match tag {
            1 => {
                let length = u16_at(class_bytes, cursor) as usize;
                cursor += 2;
                utf8[index] = Some(class_bytes[cursor..cursor + length].to_vec());
                cursor += length;
            }
            3 | 4 => cursor += 4,
            5 | 6 => {
                cursor += 8;
                index += 1;
            }
            7 | 8 | 16 | 19 | 20 => cursor += 2,
            9 | 10 | 11 | 12 | 17 | 18 => cursor += 4,
            15 => cursor += 3,
            other => panic!("unexpected constant-pool tag {other}"),
        }
        index += 1;
    }
    let class_name = |index: u16| {
        utf8[index as usize]
            .as_deref()
            .expect("the method name or descriptor is UTF-8")
    };

    cursor += 6;
    let interfaces = u16_at(class_bytes, cursor);
    cursor += 2 + usize::from(interfaces) * 2;
    let fields = u16_at(class_bytes, cursor);
    cursor += 2;
    skip_members(class_bytes, &mut cursor, fields);
    let methods = u16_at(class_bytes, cursor);
    cursor += 2;
    let mut flags_offset = None;
    let mut accessor_code = None;
    for _ in 0..methods {
        let start = cursor;
        let flags = u16_at(class_bytes, start);
        let name = class_name(u16_at(class_bytes, start + 2));
        let descriptor = class_name(u16_at(class_bytes, start + 4));
        let attributes = u16_at(class_bytes, start + 6);
        cursor += 8;
        if name == b"access$set" && descriptor == b"(LSyntheticAccessorGuard;Ljava/lang/Object;)V" {
            assert_eq!(flags & 0x0009, 0x0009, "fixture method is public static");
            assert_eq!(
                flags & 0x1000,
                0,
                "the source fixture is not already synthetic"
            );
            flags_offset = Some(start);
            for _ in 0..attributes {
                let attribute_name = class_name(u16_at(class_bytes, cursor));
                let length = u32_at(class_bytes, cursor + 2);
                if attribute_name == b"Code" {
                    let code_length = u32_at(class_bytes, cursor + 10);
                    let code_start = cursor + 14;
                    accessor_code =
                        Some(class_bytes[code_start..code_start + code_length].to_vec());
                }
                cursor += 6 + length;
            }
        } else {
            skip_attributes(class_bytes, &mut cursor, attributes);
        }
    }
    let code = accessor_code.expect("the setter has a Code attribute");
    assert_eq!(code.len(), 6);
    assert_eq!(&code[..2], &[0x2a, 0x2b]);
    assert_eq!(code[2], 0xb5, "the third opcode is putfield");
    assert_eq!(code[5], 0xb1, "the fourth opcode is return");
    let mut result = class_bytes.to_vec();
    let offset = flags_offset.expect("the physical setter is present");
    let flags = u16_at(&result, offset) | 0x1000;
    result[offset..offset + 2].copy_from_slice(&flags.to_be_bytes());
    result
}

fn compile_source_without_input_classpath(source: &str, class_name: &str) {
    let temp = TempDir::new("synthetic-source");
    let java = temp.path().join(format!("{class_name}.java"));
    let output = temp.path().join("classes");
    let empty_classpath = temp.path().join("empty-classpath");
    let empty_sourcepath = temp.path().join("empty-sourcepath");
    std::fs::create_dir_all(&output).expect("candidate output dir is created");
    std::fs::create_dir(&empty_classpath).expect("an actual empty classpath directory is created");
    std::fs::create_dir(&empty_sourcepath)
        .expect("an actual empty sourcepath directory is created");
    std::fs::write(&java, source).expect("the rendered source is written");
    let result = Command::new("javac")
        .args([
            "--release",
            "8",
            "-Xlint:-options",
            "-classpath",
            empty_classpath.to_str().expect("classpath path is UTF-8"),
            "-sourcepath",
            empty_sourcepath.to_str().expect("sourcepath path is UTF-8"),
            "-d",
        ])
        .arg(&output)
        .arg(&java)
        .output()
        .expect("javac runs without the original jar on its classpath");
    assert!(
        result.status.success(),
        "the refused synthetic-accessor field remains source-compatible:\n{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

fn limits() -> Limits {
    jarde::facade::task_limits(&[]).expect("the default task budget is bounded")
}

fn source(jar: &[u8], class: &str) -> ClassSourceReport {
    source_with_evidence(
        jar,
        class,
        &RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
    )
}

fn source_with_evidence(
    jar: &[u8],
    class: &str,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(jar.to_vec()), &mut budget)
        .expect("the frozen class-only jar opens");
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
    match engine
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            evidence,
            &mut budget,
        )
        .expect("the bounded class-source request completes")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one frozen class is not ambiguous or incomplete: {other:?}"),
    }
}

fn field<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceField {
    report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("missing physical field `{name}`:\n{}", report.text))
}

fn assert_field_type(report: &ClassSourceReport, name: &str, spelling: &str) {
    let record = field(report, name);
    assert!(
        record
            .declaration
            .as_deref()
            .is_some_and(|declaration| declaration.contains(spelling)),
        "field `{name}` must use `{spelling}`:\n{}",
        report.text
    );
}

fn assert_field_erased(report: &ClassSourceReport, name: &str, spelling: &str) {
    let record = field(report, name);
    assert!(
        record
            .declaration
            .as_deref()
            .is_some_and(|declaration| declaration.contains(spelling)),
        "field `{name}` must keep its physical erasure `{spelling}`:\n{}",
        report.text
    );
    assert!(
        !record
            .markers
            .iter()
            .any(|marker| marker.contains("projected after descriptor erasure")),
        "an unproved writer must not publish the field Signature: {record:?}"
    );
}

fn method_decl<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .and_then(|method| method.declaration.as_deref())
        .unwrap_or_else(|| panic!("missing declaration for `{name}`:\n{}", report.text))
}

#[test]
fn raw_formals_use_their_field_erasure_including_slots_null_arrays_and_bounds() {
    for (jar, class, field_type) in [
        (RAW_PARAM, "RawParam", "T value"),
        (STATIC_RAW_LOCAL, "StaticRawLocal", "T value"),
        (
            DIRECT_INSTANCE_RAW_PARAM,
            "DirectInstanceRawParam",
            "T value",
        ),
        (WIDE_RAW_PARAM, "WideRawParam", "T value"),
        (ARRAY_RAW_PARAM, "ArrayRawParam", "T[] value"),
        (ARRAY_2D_RAW_PARAM, "Array2DRawParam", "T[] value"),
        (
            PRIMITIVE_ARRAY_RAW_PARAM,
            "PrimitiveArrayRawParam",
            "T value",
        ),
        (BOUND_RAW_PARAM, "BoundRawParam", "T value"),
        (NULL_RAW_PARAM, "NullRawParam", "T value"),
        (RETAINED_ALIAS_RAW_PARAM, "RetainedAliasRawParam", "T value"),
    ] {
        let report = source(jar, class);
        assert_field_type(&report, "value", field_type);
    }
}

#[test]
fn typed_this_and_published_headers_keep_their_own_binder_boundaries() {
    let this_receiver = source(THIS_RECEIVER, "ThisReceiver");
    assert_field_type(&this_receiver, "value", "T value");

    let typed = source(TYPED_RECEIVER, "TypedReceiver");
    assert_field_type(&typed, "value", "T value");
    assert!(
        method_decl(&typed, "put").contains("TypedReceiver ")
            && !method_decl(&typed, "put").contains("TypedReceiver<T>"),
        "the refused writer's emitted receiver remains raw; its Signature was not borrowed:\n{}",
        typed.text
    );
    assert!(
        method_decl(&typed, "observe").contains("TypedReceiver<T>"),
        "the sibling's actually published generic method remains independent:\n{}",
        typed.text
    );

    let shadow = source(SHADOW_METHOD_T, "ShadowMethodT");
    assert_field_type(&shadow, "value", "T value");
    assert!(
        !method_decl(&shadow, "put").contains("<T>"),
        "field projection must not publish the shadowed method binder:\n{}",
        shadow.text
    );

    let multiple = source(MULTI_FORMAL_RAW_PARAM, "MultiFormalRawParam");
    assert_field_type(&multiple, "value", "T value");
    assert!(
        !method_decl(&multiple, "put").contains("<K>")
            && !method_decl(&multiple, "put").contains("<T>"),
        "the field proof does not expand this method's existing generic-header boundary:\n{}",
        multiple.text
    );
    assert!(
        multiple.text.contains(".hashCode();"),
        "the separate key use remains in the recovered method body:\n{}",
        multiple.text
    );
}

#[test]
fn a_folded_this_unknown_writer_other_unsafe_write_and_inherited_owner_stay_closed() {
    let folded_this = source(INSTANCE_RAW_LOCAL, "InstanceRawLocal");
    assert_field_erased(&folded_this, "value", "java.lang.Object value");

    let unknown = source(UNKNOWN_WRITER, "UnknownWriter");
    assert_field_erased(&unknown, "value", "java.lang.Object value");

    let mixed = source(UNSAFE_OTHER_WRITER, "UnsafeOtherWriter");
    assert_field_erased(&mixed, "value", "java.lang.Object value");
    assert!(
        mixed
            .methods
            .iter()
            .any(|method| method.item.name.raw().0 == b"putRaw")
            && mixed
                .methods
                .iter()
                .any(|method| method.item.name.raw().0 == b"putThroughUnprovedCast"),
        "the field report accounts for the class containing both writer methods:\n{}",
        mixed.text
    );

    let inherited = source(RAW_OWNER_CHILD, "RawOwnerChild");
    assert!(
        inherited
            .fields
            .iter()
            .all(|field| field.item.name.raw().0 != b"value"),
        "an inherited Fieldref cannot become a same-class field projection:\n{}",
        inherited.text
    );
}

#[test]
fn synthetic_static_setter_does_not_lend_its_raw_formal_to_a_typed_caller() {
    let mutated = synthetic_setter_flags(SYNTHETIC_ACCESSOR_GUARD_CLASS);
    let mutated_jar = jar_of("SyntheticAccessorGuard", &mutated);

    let essential = source_with_evidence(
        &mutated_jar,
        "SyntheticAccessorGuard",
        &RecoveryEvidenceRequest::essential(),
    );
    let all = source_with_evidence(
        &mutated_jar,
        "SyntheticAccessorGuard",
        &RecoveryEvidenceRequest::all(),
    );
    for report in [&essential, &all] {
        assert_eq!(
            report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"access$set")
                .expect("the physical setter remains represented")
                .item
                .access_flags
                & 0x1008,
            0x1008,
            "the fixture really is a static synthetic setter"
        );
        assert_field_erased(report, "value", "java.lang.Object value");
        compile_source_without_input_classpath(&report.text, "SyntheticAccessorGuard");
    }
    assert_eq!(
        field(&essential, "value").declaration,
        field(&all, "value").declaration,
        "optional accessor evidence cannot change the field proof"
    );
}

#[test]
fn alias_pressure_keeps_rebound_and_reused_local_scopes_conservative() {
    let rebound = source(REBOUND_ALIAS_RAW_PARAM, "ReboundAliasRawParam");
    assert_field_erased(&rebound, "value", "java.lang.Object value");
    assert!(
        field(&rebound, "value")
            .markers
            .iter()
            .any(|marker| marker.contains("field_generic_write_source_unproved")),
        "the rebound alias does not borrow the raw-formal proof:\n{}",
        rebound.text
    );
    assert!(
        rebound.text.contains("if (flag)")
            && rebound.text.contains("alias = replacement;")
            && rebound.text.contains("alias.value = value;"),
        "the branch that can replace the receiver and its field write stay visible:\n{}",
        rebound.text
    );
    compile_source_without_input_classpath(&rebound.text, "ReboundAliasRawParam");

    let nested = source(NESTED_ALIAS_RAW_PARAM, "NestedAliasRawParam");
    assert_field_erased(&nested, "value", "java.lang.Object value");
    assert!(
        field(&nested, "value")
            .markers
            .iter()
            .any(|marker| marker.contains("field_generic_write_source_unproved")),
        "the reused local slot does not borrow the raw-formal proof:\n{}",
        nested.text
    );
    assert!(
        nested
            .text
            .contains("NestedAliasRawParam alias = receiver;")
            && nested.text.contains("alias = receiver;")
            && nested.text.matches("alias.value = value;").count() == 2,
        "both writes and the second assignment to the reused alias stay visible:\n{}",
        nested.text
    );
    compile_source_without_input_classpath(&nested.text, "NestedAliasRawParam");
}

#[test]
fn pre_cancelled_receiver_recovery_is_not_a_silent_success() {
    let engine = Engine::new();
    let mut open_budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(RAW_PARAM.to_vec()), &mut open_budget)
        .expect("the raw-receiver jar opens before the stop is configured");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("RawParam"),
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

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = engine
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
            &mut Budget::with_cancellation_token(limits(), token),
        )
        .expect("pre-cancellation returns a stopped outcome");
    let execution = match cancelled {
        OperationOutcome::Incomplete(candidates) => candidates.execution,
        OperationOutcome::Performed(report) => report.execution,
        OperationOutcome::Ambiguous(candidates) => {
            panic!("one class is not ambiguous: {candidates:?}")
        }
    };
    assert!(
        matches!(execution, ExecutionReport::Cancelled { .. }),
        "cancellation must propagate instead of becoming a receiver refusal: {execution:?}"
    );
}
