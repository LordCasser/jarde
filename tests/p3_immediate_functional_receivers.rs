//! `type-immediate-functional-receivers`: three frozen Java 8 classes exercise direct functional
//! receivers and the already-typed local control through the public class-source entry point.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const DIRECT_ARRAY: &[u8] = include_bytes!(
    "fixtures/p3-immediate-functional-receivers/v8/direct-array/DirectArrayCtorRef.class"
);
const DIRECT_METHOD: &[u8] = include_bytes!(
    "fixtures/p3-immediate-functional-receivers/v8/direct-method/DirectMethodRef.class"
);
const BOUND_CONTROL: &[u8] = include_bytes!(
    "fixtures/p3-immediate-functional-receivers/v8/bound-control/BoundFunctionalReceiver.class"
);

#[derive(Clone, Debug)]
enum TestCpEntry {
    Utf8(String),
    Class(u16),
    NameAndType(u16),
    Member(u16, u16),
    MethodHandle { kind_offset: usize, reference: u16 },
    Other,
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn u32_at(bytes: &[u8], offset: usize) -> usize {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize
}

fn test_constant_pool(bytes: &[u8]) -> (Vec<Option<TestCpEntry>>, usize) {
    let count = usize::from(u16_at(bytes, 8));
    let mut pool = vec![None; count];
    let mut cursor = 10;
    let mut index = 1;
    while index < count {
        let tag = bytes[cursor];
        cursor += 1;
        let entry = match tag {
            1 => {
                let length = usize::from(u16_at(bytes, cursor));
                cursor += 2;
                let value = String::from_utf8_lossy(&bytes[cursor..cursor + length]).into_owned();
                cursor += length;
                TestCpEntry::Utf8(value)
            }
            7 => {
                let class = u16_at(bytes, cursor);
                cursor += 2;
                TestCpEntry::Class(class)
            }
            9..=11 => {
                let class = u16_at(bytes, cursor);
                let name_type = u16_at(bytes, cursor + 2);
                cursor += 4;
                TestCpEntry::Member(class, name_type)
            }
            12 => {
                let name = u16_at(bytes, cursor);
                let descriptor = u16_at(bytes, cursor + 2);
                cursor += 4;
                let _ = descriptor;
                TestCpEntry::NameAndType(name)
            }
            15 => {
                let kind_offset = cursor;
                let reference = u16_at(bytes, cursor + 1);
                cursor += 3;
                TestCpEntry::MethodHandle {
                    kind_offset,
                    reference,
                }
            }
            3 | 4 | 17 | 18 => {
                cursor += 4;
                TestCpEntry::Other
            }
            5 | 6 => {
                cursor += 8;
                pool[index] = Some(TestCpEntry::Other);
                index += 1;
                TestCpEntry::Other
            }
            8 | 16 | 19 | 20 => {
                cursor += 2;
                TestCpEntry::Other
            }
            _ => panic!("unexpected classfile constant-pool tag {tag}"),
        };
        pool[index] = Some(entry);
        index += 1;
    }
    (pool, cursor)
}

fn cp_utf8(pool: &[Option<TestCpEntry>], index: u16) -> &str {
    match pool[usize::from(index)].as_ref().unwrap() {
        TestCpEntry::Utf8(value) => value,
        other => panic!("expected UTF8 entry, got {other:?}"),
    }
}

fn method_handle_kind_offset(bytes: &[u8], target_name: &str) -> usize {
    let (pool, _) = test_constant_pool(bytes);
    for entry in pool.iter().flatten() {
        let TestCpEntry::MethodHandle {
            kind_offset,
            reference,
        } = entry
        else {
            continue;
        };
        let Some(TestCpEntry::Member(class_index, name_type_index)) =
            pool[usize::from(*reference)].as_ref()
        else {
            continue;
        };
        let TestCpEntry::Class(class_name_index) =
            pool[usize::from(*class_index)].as_ref().unwrap()
        else {
            continue;
        };
        let TestCpEntry::NameAndType(name_index) =
            pool[usize::from(*name_type_index)].as_ref().unwrap()
        else {
            continue;
        };
        let _ = cp_utf8(&pool, *class_name_index);
        if cp_utf8(&pool, *name_index) == target_name {
            return *kind_offset;
        }
    }
    panic!("classfile has no method handle for {target_name}");
}

fn lambda_helper_name(bytes: &[u8], source_method: &str) -> String {
    let (pool, mut cursor) = test_constant_pool(bytes);
    cursor += 6;
    let interface_count = usize::from(u16_at(bytes, cursor));
    cursor += 2 + interface_count * 2;
    let fields_count = usize::from(u16_at(bytes, cursor));
    cursor += 2;
    for _ in 0..fields_count {
        let attributes = usize::from(u16_at(bytes, cursor + 6));
        cursor += 8;
        for _ in 0..attributes {
            let length = u32_at(bytes, cursor + 2);
            cursor += 6 + length;
        }
    }
    let methods_count = usize::from(u16_at(bytes, cursor));
    cursor += 2;
    let prefix = format!("lambda${source_method}$");
    let mut helper = None;
    for _ in 0..methods_count {
        let name_index = u16_at(bytes, cursor + 2);
        let attributes = usize::from(u16_at(bytes, cursor + 6));
        let name = cp_utf8(&pool, name_index);
        if name.starts_with(&prefix) {
            assert!(
                helper.replace(name.to_owned()).is_none(),
                "{source_method} has more than one lambda helper in the fixture"
            );
        }
        cursor += 8;
        for _ in 0..attributes {
            let length = u32_at(bytes, cursor + 2);
            cursor += 6 + length;
        }
    }
    helper.unwrap_or_else(|| panic!("classfile has no lambda helper for {source_method}"))
}

fn mutate_method_flag(bytes: &mut [u8], target_name: &str, mask: u16) {
    let (pool, mut cursor) = test_constant_pool(bytes);
    cursor += 6;
    let interface_count = usize::from(u16_at(bytes, cursor));
    cursor += 2 + interface_count * 2;
    let fields_count = usize::from(u16_at(bytes, cursor));
    cursor += 2;
    for _ in 0..fields_count {
        let attributes = usize::from(u16_at(bytes, cursor + 6));
        cursor += 8;
        for _ in 0..attributes {
            let length = u32_at(bytes, cursor + 2);
            cursor += 6 + length;
        }
    }
    let methods_count = usize::from(u16_at(bytes, cursor));
    cursor += 2;
    for _ in 0..methods_count {
        let flags_offset = cursor;
        let name_index = u16_at(bytes, cursor + 2);
        let attributes = usize::from(u16_at(bytes, cursor + 6));
        cursor += 8;
        if cp_utf8(&pool, name_index) == target_name {
            let flags = u16_at(bytes, flags_offset) ^ mask;
            bytes[flags_offset..flags_offset + 2].copy_from_slice(&flags.to_be_bytes());
            return;
        }
        for _ in 0..attributes {
            let length = u32_at(bytes, cursor + 2);
            cursor += 6 + length;
        }
    }
    panic!("classfile has no method {target_name}");
}

struct Fixture {
    name: &'static str,
    class: &'static str,
    runner: &'static str,
    runner_name: &'static str,
    bytes: &'static [u8],
    output: &'static str,
}

const FIXTURES: [Fixture; 3] = [
    Fixture {
        name: "direct-array",
        class: "DirectArrayCtorRef",
        runner: include_str!(
            "fixtures/p3-immediate-functional-receivers/v8/direct-array/DirectArrayCtorRunner.java"
        ),
        runner_name: "DirectArrayCtorRunner",
        bytes: DIRECT_ARRAY,
        output: "0\n3\njava.lang.NegativeArraySizeException\n",
    },
    Fixture {
        name: "direct-method",
        class: "DirectMethodRef",
        runner: include_str!(
            "fixtures/p3-immediate-functional-receivers/v8/direct-method/DirectMethodRunner.java"
        ),
        runner_name: "DirectMethodRunner",
        bytes: DIRECT_METHOD,
        output: "0\n3\n7\n",
    },
    Fixture {
        name: "bound-control",
        class: "BoundFunctionalReceiver",
        runner: include_str!(
            "fixtures/p3-immediate-functional-receivers/v8/bound-control/BoundFunctionalRunner.java"
        ),
        runner_name: "BoundFunctionalRunner",
        bytes: BOUND_CONTROL,
        output: "0|0\n3|3\njava.lang.NegativeArraySizeException|1\n",
    },
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed fixture opens as a standalone class")
}

fn request(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_string()),
        },
    }
}

fn class_source(
    snapshot: &ArtifactSnapshot,
    class: &str,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request(snapshot, class),
            evidence,
            &mut budget(),
        )
        .expect("the fixture's class-source request is valid")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one standalone class selects exactly once, got {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "the bounded fixture request completes, got {} candidates",
            candidates.candidates.len()
        ),
    }
}

fn body<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no method `{name}` in the fixture"));
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` was not recovered: {other:?}"),
    }
}

fn method_identity(
    snapshot: &ArtifactSnapshot,
    name: &[u8],
    descriptor: &[u8],
) -> PhysicalMethodId {
    let header = Engine::new()
        .inspect_header(
            snapshot,
            ClassTarget::Root,
            &mut budget(),
            InspectionMode::Strict,
        )
        .expect("the fixture header is readable");
    PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: header.source.class_bytes,
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
    }
}

fn recover_method(
    snapshot: &ArtifactSnapshot,
    identity: &PhysicalMethodId,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> OperationOutcome<MethodRecoveryReport> {
    let request = MethodOperationRequest {
        method: MethodRef::Method {
            method: identity.clone(),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_string()),
        },
    };
    Engine::new()
        .recover_target_with_evidence(slice::from_ref(snapshot), &request, evidence, budget)
        .expect("the physical method request is valid")
}

fn assert_no_partial_java_body(outcome: OperationOutcome<MethodRecoveryReport>, label: &str) {
    match outcome {
        OperationOutcome::Incomplete(_) => {}
        OperationOutcome::Ambiguous(candidates) => panic!(
            "the standalone method identity is not ambiguous: {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Performed(report) => {
            let recovery = report.recovered.recovery();
            if recovery.produced() {
                assert_eq!(
                    recovery.representation,
                    Representation::Bytecode,
                    "{label} may publish a whole quote, never a partial Java receiver cast:\n{}",
                    recovery.text
                );
                assert!(recovery.text.contains("@bytecode"), "{label}");
            } else {
                assert!(recovery.text.is_empty(), "{label} published partial text");
            }
            assert!(
                recovery.source_map.is_empty(),
                "{label} published a partial map"
            );
        }
    }
}

fn compile_and_run(directory: &Path, runner_name: &str) -> String {
    let compile = Command::new("javac")
        .args(["--release", "8", "-classpath"])
        .arg(directory)
        .arg("-d")
        .arg(directory)
        .arg(directory.join(format!("{runner_name}.java")))
        .output()
        .expect("JDK javac is available for the complete-class regression");
    assert!(
        compile.status.success(),
        "javac rejected the complete source:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let output = Command::new("java")
        .args(["-Xverify:all", "-classpath"])
        .arg(directory)
        .arg(runner_name)
        .output()
        .expect("JDK java is available for the complete-class regression");
    assert!(
        output.status.success(),
        "the Java 8 runner failed verification or execution:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the fixture output is UTF-8")
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos();
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-immediate-receiver-{}-{nonce}-{id}",
            std::process::id(),
        ));
        fs::create_dir_all(&path).expect("create a JDK comparison directory");
        Self(path)
    }

    fn child(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(&path).expect("create a JDK comparison case directory");
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_receiver_cast_origin(
    report: &RecoveryReport,
    target: &str,
    factory_bci: u32,
    call_bci: u32,
) {
    let cast = report
        .source_map
        .segments()
        .iter()
        .find(|segment| {
            let text = segment.text(&report.text);
            text.contains(target)
                && (text.contains(" -> ") || text.contains("::"))
                && !text.contains(".apply")
                && !text.contains("applyAsInt")
        })
        .unwrap_or_else(|| panic!("no mapped receiver cast to `{target}`:\n{}", report.text));
    let origin = cast.origin();
    assert_eq!(origin.primary().bci(), factory_bci);
    assert!(
        origin.primary().cp().is_some(),
        "factory BCI keeps its pool entry"
    );
    assert_eq!(
        origin
            .derived()
            .iter()
            .map(|anchor| anchor.bci())
            .collect::<Vec<_>>(),
        [call_bci],
        "the call is the cast's only derived origin; no checkcast BCI is invented"
    );
}

#[test]
fn all_three_frozen_classes_keep_typed_immediate_receivers_and_compile_whole() {
    let scratch = Scratch::new();
    for fixture in &FIXTURES {
        let snapshot = open(fixture.bytes);
        let essential = class_source(
            &snapshot,
            fixture.class,
            &RecoveryEvidenceRequest::essential(),
        );
        let complete = class_source(&snapshot, fixture.class, &RecoveryEvidenceRequest::all());
        assert_eq!(
            essential.text, complete.text,
            "{} source is independent of evidence selection",
            fixture.name
        );

        match fixture.name {
            "direct-array" => {
                let essential_body = body(&essential, "make");
                let all_body = body(&complete, "make");
                assert!(all_body.text.contains("java.util.function.IntFunction"));
                assert!(all_body.text.contains(".apply(n)"));
                assert!(
                    all_body.text.contains("lambda$make$0"),
                    "synthetic helper remains"
                );
                assert!(essential_body.source_map.is_empty());
                assert_receiver_cast_origin(all_body, "java.util.function.IntFunction", 0, 6);
            }
            "direct-method" => {
                let essential_body = body(&essential, "abs");
                let all_body = body(&complete, "abs");
                assert!(
                    all_body
                        .text
                        .contains("java.util.function.IntUnaryOperator")
                );
                assert!(all_body.text.contains(".applyAsInt(n)"));
                assert!(essential_body.source_map.is_empty());
                assert_receiver_cast_origin(all_body, "java.util.function.IntUnaryOperator", 0, 6);
            }
            "bound-control" => {
                for name in ["array", "method"] {
                    let local_body = body(&complete, name);
                    assert!(local_body.text.contains(" f = "));
                    assert!(!local_body.text.contains("((java.util.function."));
                    assert!(body(&essential, name).source_map.is_empty());
                }
            }
            _ => unreachable!(),
        }

        let original = scratch.child(&format!("{}-original", fixture.name));
        fs::write(
            original.join(format!("{}.class", fixture.class)),
            fixture.bytes,
        )
        .expect("write the frozen original class");
        fs::write(
            original.join(format!("{}.java", fixture.runner_name)),
            fixture.runner,
        )
        .expect("write the source-only runner");
        assert_eq!(
            compile_and_run(&original, fixture.runner_name),
            fixture.output,
            "{} original class output",
            fixture.name
        );

        let recovered = scratch.child(&format!("{}-jarde", fixture.name));
        fs::write(
            recovered.join(format!("{}.java", fixture.class)),
            &complete.text,
        )
        .expect("write the unmodified Jarde class source");
        fs::write(
            recovered.join(format!("{}.java", fixture.runner_name)),
            fixture.runner,
        )
        .expect("write the source-only runner");
        assert_eq!(
            compile_and_run(&recovered, fixture.runner_name),
            fixture.output,
            "{} recovered complete class output",
            fixture.name
        );
    }
}

#[test]
fn class_source_projects_array_constructor_atomically_with_same_named_helper() {
    let scratch = Scratch::new();
    let original = scratch.child("array-maker-original");
    fs::write(
        original.join("ArrayCtorSubject.java"),
        "public class ArrayCtorSubject {\n\
             static ArrayMaker arrayCtor() { return int[]::new; }\n\
             static int run() { return arrayCtor().make(7).length; }\n\
         }\n",
    )
    .expect("write the Java 8 array-maker fixture");
    fs::write(
        original.join("ArrayMaker.java"),
        "interface ArrayMaker { int[] make(Integer n); }\n",
    )
    .expect("write the Java 8 functional interface");
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&original)
        .arg(original.join("ArrayMaker.java"))
        .arg(original.join("ArrayCtorSubject.java"))
        .output()
        .expect("JDK javac is available for the array-maker fixture");
    assert!(
        compile.status.success(),
        "javac rejected the original fixture:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let bytes = fs::read(original.join("ArrayCtorSubject.class"))
        .expect("javac writes the array-maker class");
    let snapshot = open(&bytes);
    let recovered = class_source(
        &snapshot,
        "ArrayCtorSubject",
        &RecoveryEvidenceRequest::all(),
    );
    let array_ctor = body(&recovered, "arrayCtor");
    assert!(
        recovered.text.contains("int[]::new"),
        "the assembled source projects the typed non-generic SAM:\n{}",
        recovered.text
    );
    assert!(array_ctor.text.contains("lambda$arrayCtor$0"));
    assert!(!array_ctor.text.contains("int[]::new"));
    assert!(
        recovered.methods.iter().any(|method| {
            method.item.name.raw().0 == b"lambda$arrayCtor$0"
                && method.text.contains("lambda$arrayCtor$0")
        }),
        "the physical synthetic helper remains in the method report"
    );

    let emitted = scratch.child("array-maker-emitted");
    fs::write(emitted.join("ArrayCtorSubject.java"), &recovered.text)
        .expect("write the reconstructed array-maker class");
    fs::write(
        emitted.join("ArrayMaker.java"),
        "interface ArrayMaker { int[] make(Integer n); }\n",
    )
    .expect("write the SAM declaration");
    fs::write(
        emitted.join("ArrayCtorRunner.java"),
        "public class ArrayCtorRunner { public static void main(String[] args) { System.out.println(ArrayCtorSubject.run()); } }\n",
    )
    .expect("write the execution runner");
    let recompile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&emitted)
        .arg(emitted.join("ArrayMaker.java"))
        .arg(emitted.join("ArrayCtorSubject.java"))
        .arg(emitted.join("ArrayCtorRunner.java"))
        .output()
        .expect("JDK javac is available for reconstructed-source validation");
    assert!(
        recompile.status.success(),
        "javac rejected the reconstructed whole source:\n{}\n{}",
        recovered.text,
        String::from_utf8_lossy(&recompile.stderr)
    );
    let original_runner = scratch.child("array-maker-original-run");
    fs::copy(
        original.join("ArrayCtorSubject.class"),
        original_runner.join("ArrayCtorSubject.class"),
    )
    .expect("copy the original physical subject class");
    fs::copy(
        original.join("ArrayMaker.class"),
        original_runner.join("ArrayMaker.class"),
    )
    .expect("copy the original SAM class");
    fs::write(
        original_runner.join("ArrayCtorRunner.java"),
        "public class ArrayCtorRunner { public static void main(String[] args) { System.out.println(ArrayCtorSubject.run()); } }\n",
    )
    .expect("write the execution runner beside the original class");
    let original_runner_compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-classpath"])
        .arg(&original_runner)
        .arg("-d")
        .arg(&original_runner)
        .arg(original_runner.join("ArrayCtorRunner.java"))
        .output()
        .expect("JDK javac is available for the original execution runner");
    assert!(
        original_runner_compile.status.success(),
        "javac rejected the original execution runner:\n{}",
        String::from_utf8_lossy(&original_runner_compile.stderr)
    );
    let original_execution = Command::new("java")
        .args(["-Xverify:all", "-classpath"])
        .arg(&original_runner)
        .arg("ArrayCtorRunner")
        .output()
        .expect("JDK java is available for original execution");
    assert!(original_execution.status.success());
    assert_eq!(String::from_utf8_lossy(&original_execution.stdout), "7\n");
    let execution = Command::new("java")
        .args(["-Xverify:all", "-classpath"])
        .arg(&emitted)
        .arg("ArrayCtorRunner")
        .output()
        .expect("JDK java is available for the reconstructed-source execution");
    assert!(
        execution.status.success(),
        "the reconstructed program failed verification or execution:\n{}",
        String::from_utf8_lossy(&execution.stderr)
    );
    assert_eq!(execution.stdout, original_execution.stdout);
}

#[test]
fn raw_function_array_constructor_keeps_its_physical_helper_and_lambda_site() {
    let snapshot = open(include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-26/boxed-sam-adaptations/v8/BoxedSamProbe.class"
    ));
    let recovered = class_source(&snapshot, "BoxedSamProbe", &RecoveryEvidenceRequest::all());
    let method = recovered
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"arrayCtor")
        .expect("the frozen probe has its boxed array-constructor method");
    let declaration = method.declaration.as_deref().unwrap_or_default();
    assert!(
        declaration.contains("java.util.function.Function"),
        "{declaration}"
    );
    assert!(
        !declaration.contains('<'),
        "the frozen target remains raw: {declaration}"
    );
    assert!(
        !recovered.text.contains("int[]::new"),
        "raw Function's Object SAM input is not sufficient evidence for array projection:\n{}",
        recovered.text
    );
    assert!(recovered.text.contains("lambda$arrayCtor$0"));
    assert!(recovered.methods.iter().any(|method| {
        method.item.name.raw().0 == b"lambda$arrayCtor$0"
            && method.text.contains("lambda$arrayCtor$0")
    }));
    let body = body(&recovered, "arrayCtor");
    assert!(body.text.contains("lambda$arrayCtor$0"), "{}", body.text);
}

#[test]
fn array_constructor_projection_budget_stop_keeps_method_and_helper_together() {
    let scratch = Scratch::new();
    let source = scratch.child("array-maker-budget-original");
    fs::write(
        source.join("ArrayCtorSubject.java"),
        "public class ArrayCtorSubject { static ArrayMaker arrayCtor() { return int[]::new; } }\n",
    )
    .expect("write the array-maker subject");
    fs::write(
        source.join("ArrayMaker.java"),
        "interface ArrayMaker { int[] make(Integer n); }\n",
    )
    .expect("write the SAM declaration");
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&source)
        .arg(source.join("ArrayMaker.java"))
        .arg(source.join("ArrayCtorSubject.java"))
        .output()
        .expect("JDK javac is available for the budget fixture");
    assert!(compile.status.success());
    let bytes = fs::read(source.join("ArrayCtorSubject.class")).expect("read javac fixture");
    let snapshot = open(&bytes);
    let complete = class_source(
        &snapshot,
        "ArrayCtorSubject",
        &RecoveryEvidenceRequest::all(),
    );
    let mut limits = complete.limits.clone();
    limits.ir_items = complete.usage.ir_items.saturating_sub(1);
    let mut constrained = Budget::new(limits);
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot, "ArrayCtorSubject"),
            &RecoveryEvidenceRequest::all(),
            &mut constrained,
        )
        .expect("a class-source budget stop returns its partial report");
    let OperationOutcome::Performed(report) = outcome else {
        panic!("the stopped class-source request keeps its report: {outcome:?}");
    };
    assert!(!report.text.contains("int[]::new"));
    assert!(report.text.contains("lambda$arrayCtor$0"));
    let array_ctor = body(&report, "arrayCtor");
    assert!(array_ctor.text.contains("lambda$arrayCtor$0"));
}

#[test]
fn captured_array_length_stays_a_lambda_call_and_keeps_its_physical_helper() {
    let scratch = Scratch::new();
    let original = scratch.child("captured-array-original");
    fs::write(
        original.join("CapturedArray.java"),
        "import java.util.function.Supplier;\n\
         public class CapturedArray {\n\
             static Supplier<int[]> array(int n) { return () -> new int[n]; }\n\
         }\n",
    )
    .expect("write the javac capture fixture");
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&original)
        .arg(original.join("CapturedArray.java"))
        .output()
        .expect("JDK javac is available for the captured-array regression");
    assert!(
        compile.status.success(),
        "javac rejected the original fixture:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let bytes = fs::read(original.join("CapturedArray.class"))
        .expect("javac writes the captured-array class");
    let snapshot = open(&bytes);
    let recovered = class_source(&snapshot, "CapturedArray", &RecoveryEvidenceRequest::all());
    let array = body(&recovered, "array");
    assert_eq!(array.lambdas.len(), 1, "{:?}", array.lambdas);
    assert_eq!(
        array.lambdas[0].form,
        Some(LambdaForm::Lambda),
        "the zero-argument SAM binds its array length at creation time"
    );
    assert!(
        array.text.contains("lambda$array$0"),
        "the physical helper call is retained:\n{}",
        array.text
    );
    assert!(
        !array.text.contains("int[]::new"),
        "a captured length cannot become a no-capture array constructor reference:\n{}",
        array.text
    );
    assert!(
        recovered.text.contains("lambda$array$0"),
        "the physical helper declaration remains in class-source:\n{}",
        recovered.text
    );

    let emitted = scratch.child("captured-array-emitted");
    fs::write(emitted.join("CapturedArray.java"), &recovered.text)
        .expect("write the reconstructed class source");
    let recompile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&emitted)
        .arg(emitted.join("CapturedArray.java"))
        .output()
        .expect("JDK javac is available for reconstructed-source validation");
    assert!(
        recompile.status.success(),
        "javac rejected the captured-array recovery:\n{}\n{}",
        recovered.text,
        String::from_utf8_lossy(&recompile.stderr)
    );
}

#[test]
fn receiver_cast_output_and_ir_budgets_and_cancellation_do_not_publish_partial_java() {
    let snapshot = open(DIRECT_METHOD);
    let identity = method_identity(&snapshot, b"abs", b"(I)I");
    let evidence = RecoveryEvidenceRequest::essential();
    let complete = match recover_method(&snapshot, &identity, &evidence, &mut budget()) {
        OperationOutcome::Performed(report) => report,
        other => panic!("the immediate method-reference body completes: {other:?}"),
    };
    let recovery = complete.recovered.recovery();
    assert!(recovery.produced());
    assert!(
        recovery
            .text
            .contains("java.util.function.IntUnaryOperator")
    );
    assert!(complete.usage.output_bytes > 0);
    assert!(complete.usage.ir_items > 0);

    let mut output_limits = complete.limits.clone();
    output_limits.output_bytes = complete
        .usage
        .output_bytes
        .checked_sub(1)
        .expect("the complete body writes output");
    let output_stopped = recover_method(
        &snapshot,
        &identity,
        &evidence,
        &mut Budget::new(output_limits),
    );
    assert_no_partial_java_body(output_stopped, "one byte below the measured receiver body");

    let mut ir_limits = complete.limits.clone();
    ir_limits.ir_items = complete
        .usage
        .ir_items
        .checked_sub(1)
        .expect("the complete body charges IR items");
    let ir_stopped = recover_method(&snapshot, &identity, &evidence, &mut Budget::new(ir_limits));
    assert_no_partial_java_body(ir_stopped, "one IR item below the complete receiver body");

    let token = jarde::budget::CancellationToken::new();
    token.cancel();
    let cancelled = recover_method(
        &snapshot,
        &identity,
        &evidence,
        &mut Budget::with_cancellation_token(complete.limits, token),
    );
    assert_no_partial_java_body(cancelled, "a pre-cancelled receiver body");
}

#[test]
fn no_capture_primitive_lambda_helpers_inline_as_one_class_projection() {
    let scratch = Scratch::new();
    let original = scratch.child("lambda-helper-original");
    fs::write(original.join("LambdaSubject.java"), "import java.util.function.*;\npublic final class LambdaSubject {\n  static IntSupplier zero() { return () -> 7; }\n  static IntUnaryOperator one() { return x -> x + 10; }\n  static IntBinaryOperator two() { return (x,y) -> x * 10 + y; }\n  static int run() { return zero().getAsInt() + one().applyAsInt(5) + two().applyAsInt(4,2); }\n}\n").unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&original)
        .arg(original.join("LambdaSubject.java"))
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let bytes = fs::read(original.join("LambdaSubject.class")).unwrap();
    let helpers = [
        lambda_helper_name(&bytes, "zero"),
        lambda_helper_name(&bytes, "one"),
        lambda_helper_name(&bytes, "two"),
    ];
    let snapshot = open(&bytes);
    let recovered = class_source(&snapshot, "LambdaSubject", &RecoveryEvidenceRequest::all());
    for helper in &helpers {
        assert!(
            !recovered.text.contains(&format!("{}(", helper)),
            "helper call or declaration leaked: {}",
            recovered.text
        );
        assert!(
            recovered
                .methods
                .iter()
                .any(|method| method.item.name.raw().0 == helper.as_bytes()),
            "physical method report must remain for {helper}"
        );
    }
    assert!(recovered.text.contains("return () -> 7;"));
    assert!(recovered.text.contains("return (int arg0) -> arg0 + 10;"));
    assert!(
        recovered
            .text
            .contains("return (int arg0, int arg1) -> arg0 * 10 + arg1;")
    );
    let emitted = scratch.child("lambda-helper-emitted");
    fs::write(emitted.join("LambdaSubject.java"), &recovered.text).unwrap();
    fs::write(emitted.join("LambdaRunner.java"), "public class LambdaRunner { public static void main(String[] a) { System.out.println(LambdaSubject.run()); } }\n").unwrap();
    let recompile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&emitted)
        .arg(emitted.join("LambdaSubject.java"))
        .arg(emitted.join("LambdaRunner.java"))
        .output()
        .unwrap();
    assert!(
        recompile.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&recompile.stdout),
        String::from_utf8_lossy(&recompile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&emitted)
        .arg("LambdaRunner")
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "64\n");
}

#[test]
fn direct_int_and_this_plus_int_captures_inline_with_call_time_semantics() {
    let scratch = Scratch::new();
    let original = scratch.child("captured-lambda-original");
    fs::write(
        original.join("CaptureCases.java"),
        "import java.util.function.*;\n\
         public final class CaptureCases {\n\
           public IntUnaryOperator add(int base) { return x -> x + base; }\n\
           public IntSupplier bound(int delta) { return () -> this.number() + delta; }\n\
           public int number() { return -3; }\n\
         }\n",
    )
    .unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&original)
        .arg(original.join("CaptureCases.java"))
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let bytes = fs::read(original.join("CaptureCases.class")).unwrap();
    let snapshot = open(&bytes);
    let helpers = [
        lambda_helper_name(&bytes, "add"),
        lambda_helper_name(&bytes, "bound"),
    ];
    let recovered = class_source(&snapshot, "CaptureCases", &RecoveryEvidenceRequest::all());
    assert!(recovered.text.contains("return (int p0) -> p0 + arg1;"));
    assert!(
        recovered
            .text
            .contains("return () -> this.number() + arg1;")
    );
    for helper in &helpers {
        assert!(
            !recovered.text.contains(&format!("{helper}(")),
            "helper call or declaration leaked: {helper}\n{}",
            recovered.text
        );
    }
    for helper in &helpers {
        assert!(
            recovered
                .methods
                .iter()
                .any(|method| { method.item.name.raw().0 == helper.as_bytes() }),
            "physical helper method report was dropped for {helper}"
        );
    }
    let emitted = scratch.child("captured-lambda-emitted");
    fs::write(emitted.join("CaptureCases.java"), &recovered.text).unwrap();
    fs::write(
        emitted.join("Runner.java"),
        "public final class Runner { public static void main(String[] a) { CaptureCases c = new CaptureCases(); System.out.println(c.add(12).applyAsInt(-5) + \":\" + c.bound(2).getAsInt()); } }\n",
    )
    .unwrap();
    let recompile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&emitted)
        .arg(emitted.join("CaptureCases.java"))
        .arg(emitted.join("Runner.java"))
        .output()
        .unwrap();
    assert!(
        recompile.status.success(),
        "{}\n{}",
        recovered.text,
        String::from_utf8_lossy(&recompile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&emitted)
        .arg("Runner")
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "7:-1\n");
}

#[test]
fn effectful_capture_source_inlines_its_local_capture_once_with_call_time_semantics() {
    let scratch = Scratch::new();
    let original = scratch.child("captured-lambda-effect-original");
    fs::write(
        original.join("CaptureEffect.java"),
        "import java.util.function.*;\n\
         public final class CaptureEffect {\n\
           static int next() { return 3; }\n\
           static IntUnaryOperator effect() { int base = next(); return x -> x + base; }\n\
           static int run() { return effect().applyAsInt(4); }\n\
         }\n",
    )
    .unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&original)
        .arg(original.join("CaptureEffect.java"))
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let snapshot = open(&fs::read(original.join("CaptureEffect.class")).unwrap());
    let recovered = class_source(&snapshot, "CaptureEffect", &RecoveryEvidenceRequest::all());
    // The captured local is embedded exactly where the call already read it — once — and the
    // companion's physical method leaves the text.
    assert!(
        recovered
            .text
            .contains("return (int arg1) -> arg1 + local0;"),
        "{}",
        recovered.text
    );
    let helper = lambda_helper_name(
        &fs::read(original.join("CaptureEffect.class")).unwrap(),
        "effect",
    );
    assert!(
        !recovered.text.contains(&format!("{helper}(")),
        "helper call or declaration leaked: {helper}\n{}",
        recovered.text
    );
    let emitted = scratch.child("captured-lambda-effect-emitted");
    fs::write(emitted.join("CaptureEffect.java"), &recovered.text).unwrap();
    fs::write(
        emitted.join("Runner.java"),
        "public class Runner { public static void main(String[] a) { System.out.println(CaptureEffect.run()); } }\n",
    )
    .unwrap();
    let recompile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&emitted)
        .arg(emitted.join("CaptureEffect.java"))
        .arg(emitted.join("Runner.java"))
        .output()
        .unwrap();
    assert!(
        recompile.status.success(),
        "{}\n{}",
        recovered.text,
        String::from_utf8_lossy(&recompile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&emitted)
        .arg("Runner")
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "7\n");
}

#[test]
fn phi_merged_capture_source_is_refused_without_hiding_its_helper() {
    let scratch = Scratch::new();
    let original = scratch.child("captured-lambda-phi-negative");
    fs::write(
        original.join("CapturePhi.java"),
        "import java.util.function.*;\n\
         public final class CapturePhi {\n\
           static IntUnaryOperator merged(boolean choose) { int base; if (choose) base = 2; else base = 3; return x -> x + base; }\n\
         }\n",
    )
    .unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&original)
        .arg(original.join("CapturePhi.java"))
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let snapshot = open(&fs::read(original.join("CapturePhi.class")).unwrap());
    let recovered = class_source(&snapshot, "CapturePhi", &RecoveryEvidenceRequest::all());
    assert!(recovered.text.contains("lambda$merged$0"));
    assert!(
        recovered.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "lambda_helper_projection_refused"
                && diagnostic.message.contains("lambda$merged$0")
                && diagnostic.message.contains("could not take its projection")
        }),
        "the refused site must keep its physical companion, with the reason located: {:?}",
        recovered.diagnostics
    );
}

#[test]
fn captured_lambda_ir_budget_stop_keeps_physical_helpers_and_does_not_inline() {
    let scratch = Scratch::new();
    let original = scratch.child("captured-lambda-budget-original");
    fs::write(
        original.join("CaptureBudget.java"),
        "import java.util.function.*;\n\
         public final class CaptureBudget {\n\
           IntUnaryOperator add(int base) { return x -> x + base; }\n\
           IntSupplier bound(int delta) { return () -> this.number() + delta; }\n\
           int number() { return -3; }\n\
         }\n",
    )
    .unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&original)
        .arg(original.join("CaptureBudget.java"))
        .output()
        .unwrap();
    assert!(compile.status.success());
    let bytes = fs::read(original.join("CaptureBudget.class")).unwrap();
    let helpers = [
        lambda_helper_name(&bytes, "add"),
        lambda_helper_name(&bytes, "bound"),
    ];
    let snapshot = open(&bytes);
    let complete = class_source(&snapshot, "CaptureBudget", &RecoveryEvidenceRequest::all());
    let mut limits = complete.limits.clone();
    limits.ir_items = complete.usage.ir_items.saturating_sub(1);
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot, "CaptureBudget"),
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits),
        )
        .unwrap();
    let OperationOutcome::Performed(stopped) = outcome else {
        panic!("the budget stop retains a report: {outcome:?}")
    };
    assert!(stopped.text.contains(&helpers[0]));
    assert!(stopped.text.contains(&helpers[1]));
    assert!(
        !stopped
            .text
            .contains("inlined exact primitive lambda helper")
    );
    assert!(
        stopped.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "lambda_helper_projection_refused"
                || diagnostic.code.contains("budget")
                || diagnostic.message.contains("budget")
        }),
        "budget stop must remain visible: {:?}",
        stopped.diagnostics
    );

    let token = jarde::budget::CancellationToken::new();
    token.cancel();
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot, "CaptureBudget"),
            &RecoveryEvidenceRequest::all(),
            &mut Budget::with_cancellation_token(complete.limits.clone(), token),
        )
        .unwrap();
    assert!(
        matches!(outcome, OperationOutcome::Incomplete(_)),
        "pre-cancellation publishes no class-source text or helper omission: {outcome:?}"
    );
}

#[test]
fn captured_helper_handle_or_physical_flags_mismatch_refuses_omission() {
    let scratch = Scratch::new();
    let original = scratch.child("captured-lambda-identity-original");
    fs::write(
        original.join("CaptureIdentity.java"),
        "import java.util.function.*;\n\
         public final class CaptureIdentity {\n\
           IntUnaryOperator add(int base) { return x -> x + base; }\n\
           IntSupplier bound(int delta) { return () -> this.number() + delta; }\n\
           int number() { return -3; }\n\
         }\n",
    )
    .unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&original)
        .arg(original.join("CaptureIdentity.java"))
        .output()
        .unwrap();
    assert!(compile.status.success());
    let clean_bytes = fs::read(original.join("CaptureIdentity.class")).unwrap();
    let add_helper = lambda_helper_name(&clean_bytes, "add");
    let bound_helper = lambda_helper_name(&clean_bytes, "bound");

    let mut wrong_handle = clean_bytes.clone();
    let handle_offset = method_handle_kind_offset(&wrong_handle, &add_helper);
    assert_eq!(wrong_handle[handle_offset], 6);
    wrong_handle[handle_offset] = 7;
    let handle_snapshot = open(&wrong_handle);
    let handle_report = class_source(
        &handle_snapshot,
        "CaptureIdentity",
        &RecoveryEvidenceRequest::all(),
    );
    assert!(handle_report.text.contains(&add_helper));
    assert!(
        handle_report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "lambda_helper_projection_refused"
                && diagnostic.message.contains(&add_helper)
        }),
        "wrong handle must be visible as a refusal: {:?}",
        handle_report.diagnostics
    );

    let mut wrong_flags = clean_bytes;
    mutate_method_flag(&mut wrong_flags, &bound_helper, 0x0002);
    let flags_snapshot = open(&wrong_flags);
    let flags_report = class_source(
        &flags_snapshot,
        "CaptureIdentity",
        &RecoveryEvidenceRequest::all(),
    );
    assert!(flags_report.text.contains(&bound_helper));
    assert!(
        flags_report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "lambda_helper_projection_refused"
                && diagnostic.message.contains(&bound_helper)
        }),
        "wrong helper flags must be visible as a refusal: {:?}",
        flags_report.diagnostics
    );
}

#[test]
fn captured_int_parameter_precedes_sam_parameter_in_the_helper_mapping() {
    let scratch = Scratch::new();
    let original = scratch.child("captured-lambda-order-original");
    fs::write(
        original.join("CaptureOrder.java"),
        "import java.util.function.*;\n\
         public final class CaptureOrder {\n\
           IntUnaryOperator subtract(int base) { return x -> x - base; }\n\
         }\n",
    )
    .unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&original)
        .arg(original.join("CaptureOrder.java"))
        .output()
        .unwrap();
    assert!(compile.status.success());
    let bytes = fs::read(original.join("CaptureOrder.class")).unwrap();
    let snapshot = open(&bytes);
    let recovered = class_source(&snapshot, "CaptureOrder", &RecoveryEvidenceRequest::all());
    assert!(recovered.text.contains("return (int p0) -> p0 - arg1;"));
    let helper = lambda_helper_name(&bytes, "subtract");
    assert!(
        !recovered.text.contains(&format!("{helper}(")),
        "helper call or declaration leaked: {helper}\n{}",
        recovered.text
    );
    let emitted = scratch.child("captured-lambda-order-emitted");
    fs::write(emitted.join("CaptureOrder.java"), recovered.text).unwrap();
    fs::write(
        emitted.join("Runner.java"),
        "public final class Runner { public static void main(String[] a) { System.out.println(new CaptureOrder().subtract(2).applyAsInt(5)); } }\n",
    )
    .unwrap();
    let recompile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&emitted)
        .arg(emitted.join("CaptureOrder.java"))
        .arg(emitted.join("Runner.java"))
        .output()
        .unwrap();
    assert!(
        recompile.status.success(),
        "{}",
        String::from_utf8_lossy(&recompile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&emitted)
        .arg("Runner")
        .output()
        .unwrap();
    assert!(run.status.success());
    assert_eq!(String::from_utf8_lossy(&run.stdout), "3\n");
}

#[test]
fn a_branchy_lambda_helper_renames_while_the_straight_one_inlines() {
    let scratch = Scratch::new();
    let original = scratch.child("lambda-helper-negative");
    fs::write(original.join("LambdaNegative.java"), "import java.util.function.*;\npublic final class LambdaNegative {\n  static IntUnaryOperator simple() { return x -> x + 1; }\n  static IntUnaryOperator branch() { return x -> x > 0 ? x : 0; }\n  static int run() { return simple().applyAsInt(4) + branch().applyAsInt(-7); }\n}\n").unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&original)
        .arg(original.join("LambdaNegative.java"))
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let bytes = fs::read(original.join("LambdaNegative.class")).unwrap();
    let simple_helper = lambda_helper_name(&bytes, "simple");
    let branch_helper = lambda_helper_name(&bytes, "branch");
    let snapshot = open(&bytes);
    let recovered = class_source(&snapshot, "LambdaNegative", &RecoveryEvidenceRequest::all());
    // The straight body inlines and its physical method leaves the text; the conditional body
    // keeps its companion under the `$jarde` name javac never synthesizes, and the call names it.
    assert!(recovered.text.contains("return (int arg0) -> arg0 + 1;"));
    assert!(
        !recovered.text.contains(&format!("{simple_helper}(")),
        "the inlined companion leaked: {simple_helper}\n{}",
        recovered.text
    );
    assert!(recovered.text.contains(&format!("{branch_helper}$jarde(")));
    assert!(
        recovered
            .text
            .contains(&format!("private static int {branch_helper}$jarde("))
    );
    let emitted = scratch.child("lambda-negative-emitted");
    fs::write(emitted.join("LambdaNegative.java"), &recovered.text).unwrap();
    fs::write(
        emitted.join("Runner.java"),
        "public class Runner { public static void main(String[] a) { System.out.println(LambdaNegative.run()); } }\n",
    )
    .unwrap();
    let recompile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&emitted)
        .arg(emitted.join("LambdaNegative.java"))
        .arg(emitted.join("Runner.java"))
        .output()
        .unwrap();
    assert!(
        recompile.status.success(),
        "{}\n{}",
        recovered.text,
        String::from_utf8_lossy(&recompile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&emitted)
        .arg("Runner")
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "5\n");
}

#[test]
fn an_overdeep_lambda_helper_renames_instead_of_walking_an_unbounded_body() {
    let scratch = Scratch::new();
    let original = scratch.child("lambda-helper-deep");
    let expression = format!("x{}", " + 1".repeat(300));
    fs::write(
        original.join("LambdaDeep.java"),
        format!(
            "import java.util.function.*;\npublic final class LambdaDeep {{\n  static IntUnaryOperator deep() {{ return x -> {expression}; }}\n}}\n"
        ),
    )
    .unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&original)
        .arg(original.join("LambdaDeep.java"))
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let snapshot = open(&fs::read(original.join("LambdaDeep.class")).unwrap());
    let recovered = class_source(&snapshot, "LambdaDeep", &RecoveryEvidenceRequest::all());
    // The over-deep straight body stays out of the bounded inline proof and keeps its companion
    // under the `$jarde` name instead — the whole class still recompiles and runs.
    assert!(recovered.text.contains("lambda$deep$0$jarde"));
    assert!(
        recovered
            .text
            .contains("private static int lambda$deep$0$jarde(")
    );
}

#[test]
fn lambda_helper_projection_budget_stop_does_not_publish_partial_helpers() {
    let scratch = Scratch::new();
    let original = scratch.child("lambda-helper-budget");
    fs::write(original.join("LambdaBudget.java"), "import java.util.function.*;\npublic final class LambdaBudget {\n  static IntSupplier zero() { return () -> 7; }\n  static IntUnaryOperator one() { return x -> x + 1; }\n}\n").unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&original)
        .arg(original.join("LambdaBudget.java"))
        .output()
        .unwrap();
    assert!(compile.status.success());
    let bytes = fs::read(original.join("LambdaBudget.class")).unwrap();
    let helpers = [
        lambda_helper_name(&bytes, "zero"),
        lambda_helper_name(&bytes, "one"),
    ];
    let snapshot = open(&bytes);
    let complete = class_source(&snapshot, "LambdaBudget", &RecoveryEvidenceRequest::all());
    let mut stopped_between_helpers = None;
    for cut in (1..=2000).step_by(10) {
        let mut limits = complete.limits.clone();
        limits.output_bytes = complete.usage.output_bytes.saturating_sub(cut);
        let mut budget = Budget::new(limits);
        let outcome = Engine::new()
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request(&snapshot, "LambdaBudget"),
                &RecoveryEvidenceRequest::all(),
                &mut budget,
            )
            .unwrap();
        let OperationOutcome::Performed(report) = outcome else {
            panic!("expected a class-source report: {outcome:?}")
        };
        if report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "lambda_helper_projection_refused"
                && diagnostic
                    .message
                    .contains("projection stopped before atomic commit")
        }) {
            stopped_between_helpers = Some(report);
            break;
        }
    }
    let stopped = stopped_between_helpers
        .expect("a budget between helper projections must stop the atomic group");
    assert!(
        stopped.diagnostics.iter().any(|diagnostic| diagnostic.code
            == "lambda_helper_projection_refused"
            && diagnostic.message.contains(&helpers[1])
            && diagnostic
                .message
                .contains("projection stopped before atomic commit")),
        "the budget must stop while staging the second helper: {:?}",
        stopped.diagnostics
    );
    assert!(stopped.text.contains(&helpers[0]));
    assert!(stopped.text.contains(&helpers[1]));
    assert!(
        !stopped.text.contains("lambda companion body inlined"),
        "a stopped projection publishes no inlined site: {}",
        stopped.text
    );
    assert!(
        !stopped.text.contains("omitted physical lambda helper"),
        "a stopped projection omits no companion: {}",
        stopped.text
    );
}
