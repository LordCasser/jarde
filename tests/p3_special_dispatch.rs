//! `preserve-special-call-dispatch`: `invokespecial` keeps class-super, interface-super and
//! private receiver selection instead of becoming a virtual call on the current object.
//!
//! The committed Java 8 fixture is deliberately a real compiler output. The normal assertions
//! describe the source semantics; the ignored JDK check compiles the text returned by
//! `Engine::class_source` itself.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;

const SPECIAL: &[u8] = include_bytes!("fixtures/p3-special-dispatch/v8/SpecialProbe.class");
const BASE_SOURCE: &str = include_str!("fixtures/p3-special-dispatch/BaseProbe.java");
const DEFAULT_SOURCE: &str = include_str!("fixtures/p3-special-dispatch/DefaultProbe.java");
const RUNNER_SOURCE: &str = include_str!("fixtures/p3-special-dispatch/SpecialRunner.java");
const BASE: &[u8] = include_bytes!("fixtures/p3-special-dispatch/v8/BaseProbe.class");
const DEFAULT: &[u8] = include_bytes!("fixtures/p3-special-dispatch/v8/DefaultProbe.class");

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

fn open_special_bundle() -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(jar(&[
                (b"SpecialProbe.class", SPECIAL),
                (b"BaseProbe.class", BASE),
                (b"DefaultProbe.class", DEFAULT),
            ])),
            &mut budget(),
        )
        .expect("特殊调用及其直接类型可作为 plain JAR 打开")
}

fn open_jar_without_dependencies() -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(jar(&[(b"SpecialProbe.class", SPECIAL)])),
            &mut budget(),
        )
        .expect("只含调用方 class 的 plain JAR 可打开")
}

fn class_source_of(snapshot: &ArtifactSnapshot) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("SpecialProbe"),
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
        OperationOutcome::Ambiguous(candidates) => panic!(
            "一个提交的 fixture 应只回答一个定义，却得到 {} 个候选",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "一个提交的 fixture 应只回答一个定义，却得到未完成选择和 {} 个候选",
            candidates.candidates.len()
        ),
    }
}

fn environment(snapshot: &ArtifactSnapshot) -> EnvironmentRequest {
    EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope: PhysicalScope::SnapshotAll,
        policy: EnvironmentPolicy::PlainJar,
        profile: RuntimeProfile {
            java_release: 8,
            multi_release: MultiReleasePolicy::Disabled,
            layout: LayoutMode::Generic,
        },
        loader: LoaderId("app".to_owned()),
    }
}

fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("method `{name}` 不在 SpecialProbe 的方法表中"))
        .text
}

#[test]
fn special_dispatch_text_keeps_each_declared_receiver() {
    let report = class_source_of(&open_special_bundle());

    let constructor = text_of(&report, "<init>");
    assert!(
        constructor.contains("super(7);"),
        "构造器的父类调用保留：{constructor}"
    );

    let value = text_of(&report, "value");
    assert!(
        value.contains("return super.value() + 1;"),
        "直接父类 special 调用不能变成当前类虚调用：{value}"
    );

    let default_call = text_of(&report, "defaultCall");
    assert!(
        default_call.contains("return DefaultProbe.super.value();"),
        "接口 default special 调用必须保留接口限定：{default_call}"
    );

    let own = text_of(&report, "callOwnPrivate");
    assert!(
        own.contains("return this.privateHelper(arg1);"),
        "本类 private 的 this 接收者必须保留：{own}"
    );

    let other = text_of(&report, "callOtherPrivate");
    assert!(
        other.contains("return arg1.privateHelper(arg2);"),
        "本类 private 的其它实例接收者必须保留：{other}"
    );
}

#[test]
fn special_dispatch_arguments_keep_super_shape_and_producers() {
    let report = class_source_of(&open_special_bundle());
    assert!(
        text_of(&report, "superWithSideEffect")
            .contains("return super.valueWith(BaseProbe.sideEffectArgument());"),
        "带副作用参数的父类调用保留求值顺序"
    );
    assert!(
        text_of(&report, "superWithThrowingArgument")
            .contains("return super.valueWith(BaseProbe.throwingArgument());"),
        "抛异常参数仍由父类调用消费"
    );
    assert!(
        text_of(&report, "superThrowing").contains("return super.failWith(3);"),
        "父类抛异常调用保留 class-super"
    );
}

#[test]
fn special_dispatch_keeps_receiver_and_call_origins() {
    let report = class_source_of(&open_special_bundle());
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"value")
        .expect("value method is present");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("value method should have a recovery report")
    };
    assert!(
        !report.source_map.direct_of_bci(0).is_empty(),
        "the aload_0 receiver origin remains mapped: {:?}",
        report.source_map.segments()
    );
    assert!(
        !report.source_map.direct_of_bci(1).is_empty(),
        "the invokespecial call origin remains mapped: {:?}",
        report.source_map.segments()
    );
}

#[test]
fn interface_super_without_selected_dependencies_keeps_a_source_mapped_refusal() {
    let report = class_source_of(&open_jar_without_dependencies());
    let default_call = text_of(&report, "defaultCall");
    assert!(
        default_call.contains("@bytecode 4 1 0"),
        "an interface qualifier without selected dependency definitions stays refused with its BCI: {default_call}"
    );
    assert!(
        default_call.contains("no selected proof"),
        "a verifier-valid target does not substitute for source-legality evidence: {default_call}"
    );
}

#[test]
fn method_only_uses_the_same_proof_and_ordinary_special_calls_read_no_dependencies() {
    let snapshot = open_special_bundle();
    let class = class_source_of(&snapshot);
    let identity = |name: &[u8]| {
        class
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == name)
            .expect("method exists in SpecialProbe")
            .item
            .identity
            .clone()
    };
    let request = |method| MethodOperationRequest {
        method: MethodRef::Method { method },
        environment: environment(&snapshot),
    };

    let mut interface_budget = budget();
    let interface = match Engine::new()
        .recover_target(
            std::slice::from_ref(&snapshot),
            &request(identity(b"defaultCall")),
            &mut interface_budget,
        )
        .expect("method-only interface call is recovered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("method-only request selects one body: {other:?}"),
    };
    assert!(
        interface
            .recovered
            .recovery()
            .text
            .contains("DefaultProbe.super.value()")
    );
    assert!(
        interface.usage.class_headers >= 3,
        "the method body, selected interface and superclass were read: {:?}",
        interface.usage
    );

    let mut ordinary_budget = budget();
    let ordinary = match Engine::new()
        .recover_target(
            std::slice::from_ref(&snapshot),
            &request(identity(b"callOwnPrivate")),
            &mut ordinary_budget,
        )
        .expect("method-only private call is recovered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("method-only request selects one body: {other:?}"),
    };
    assert!(
        ordinary
            .recovered
            .recovery()
            .text
            .contains("this.privateHelper")
    );
    assert_eq!(
        ordinary.usage.class_headers, 1,
        "without an interface-special candidate, the proof gate adds no dependency header read"
    );
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("系统时间在 epoch 之后")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-special-dispatch-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("创建 JDK scratch 目录");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn javac(dir: &Path, files: &[&str]) {
    let output = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-g:none")
        .arg("-d")
        .arg(dir)
        .args(files)
        .current_dir(dir)
        .output()
        .expect("启动 javac");
    assert!(
        output.status.success(),
        "javac 编译真实恢复文本失败：{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "需要 JDK：从 Engine::class_source 生成真实文本后用 javac/java 编译执行；cargo test 默认跳过"]
fn generated_special_dispatch_matches_original_runtime() {
    let report = class_source_of(&open_special_bundle());
    let scratch = Scratch::new();
    std::fs::write(scratch.path().join("BaseProbe.java"), BASE_SOURCE).expect("写入父类源码");
    std::fs::write(scratch.path().join("DefaultProbe.java"), DEFAULT_SOURCE).expect("写入接口源码");
    std::fs::write(scratch.path().join("SpecialProbe.java"), &report.text)
        .expect("写入库生成的 SpecialProbe 文本");
    std::fs::write(scratch.path().join("SpecialRunner.java"), RUNNER_SOURCE)
        .expect("写入原始 driver 源码");
    javac(
        scratch.path(),
        &[
            "BaseProbe.java",
            "DefaultProbe.java",
            "SpecialProbe.java",
            "SpecialRunner.java",
        ],
    );

    let output = Command::new("java")
        .arg("-cp")
        .arg(scratch.path())
        .arg("SpecialRunner")
        .current_dir(scratch.path())
        .output()
        .expect("执行恢复文本的 driver");
    assert!(
        output.status.success(),
        "恢复文本的 driver 执行失败：{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        r#"value=8
defaultCall=11
own=7
other=8
otherNull=java.lang.NullPointerException
superSideEffect=8
sideEffectCount=1
superThrowingArgument=java.lang.IllegalArgumentException
superThrowingParent=java.lang.IllegalStateException
"#
    );
}
