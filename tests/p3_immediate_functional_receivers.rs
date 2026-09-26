//! `type-immediate-functional-receivers`: three frozen Java 8 classes exercise direct functional
//! receivers and the already-typed local control through the public class-source entry point.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
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
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-immediate-receiver-{}-{nonce}",
            std::process::id()
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
