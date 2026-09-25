use jarde::*;
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn class_source(bytes: &[u8], name: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    let request = request(&snapshot, name);
    match engine
        .class_source(&[snapshot], &request, &mut task_budget(&[]).unwrap())
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source result: {other:?}"),
    }
}

fn request(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
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
            loader: LoaderId("app".to_owned()),
        },
    }
}

fn compiled_bytes(name: &str, java: &str) -> Vec<u8> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "jarde-ordinary-generic-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&dir).unwrap();
    let path = dir.join(format!("{name}.java"));
    fs::write(&path, java).unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-Xlint:-options"])
        .arg("-d")
        .arg(&dir)
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let bytes = fs::read(dir.join(format!("{name}.class"))).unwrap();
    fs::remove_dir_all(dir).unwrap();
    bytes
}

fn compile_source(name: &str, java: &str) -> ClassSourceReport {
    class_source(&compiled_bytes(name, java), name)
}

fn recompile(name: &str, java: &str) {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "jarde-ordinary-generic-recompile-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&dir).unwrap();
    let path = dir.join(format!("{name}.java"));
    fs::write(&path, java).unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-Xlint:-options"])
        .arg("-d")
        .arg(&dir)
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}\n{java}",
        String::from_utf8_lossy(&compile.stderr)
    );
    fs::remove_dir_all(dir).unwrap();
}

fn java_output(name: &str, java: &str, runner: &str) -> String {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "jarde-ordinary-run-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&dir).unwrap();
    let class = dir.join(format!("{name}.java"));
    let runner_path = dir.join("OrdinaryReflectionRunner.java");
    fs::write(&class, java).unwrap();
    fs::write(&runner_path, runner).unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options"])
        .arg("-d")
        .arg(&dir)
        .arg(&class)
        .arg(&runner_path)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}\n{java}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(&dir)
        .arg("OrdinaryReflectionRunner")
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    fs::remove_dir_all(dir).unwrap();
    String::from_utf8(run.stdout).unwrap()
}

#[test]
fn frozen_identity_shapes_and_raw_control_project_together() {
    let report = compile_source(
        "OrdinaryParameterizedSignatures",
        include_str!(
            "../openspec/evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/OrdinaryParameterizedSignatures.java"
        ),
    );
    for declaration in [
        "java.lang.Iterable<java.lang.String> strings(java.lang.Iterable<java.lang.String> arg0)",
        "java.util.List<? extends java.lang.Number> numbers(java.util.List<? extends java.lang.Number> arg0)",
        "java.util.Map<java.lang.String, java.util.List<java.lang.Integer>> nested(java.util.Map<java.lang.String, java.util.List<java.lang.Integer>> arg0)",
        "java.util.List<java.lang.String>[] arrays(java.util.List<java.lang.String>[] arg0)",
        "java.util.List raw(java.util.List arg0)",
    ] {
        assert!(
            report.text.contains(declaration),
            "{declaration}: {}",
            report.text
        );
    }
    assert!(
        !report.text.contains("generic Signature projection refused"),
        "{}",
        report.text
    );
    recompile("OrdinaryParameterizedSignatures", &report.text);
}

#[test]
fn evidence_selection_preserves_source_and_independent_method_reports() {
    let name = "OrdinaryParameterizedSignatures";
    let bytes = compiled_bytes(
        name,
        include_str!(
            "../openspec/evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/OrdinaryParameterizedSignatures.java"
        ),
    );
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut task_budget(&[]).unwrap())
        .unwrap();
    let request = request(&snapshot, name);
    let run = |evidence: RecoveryEvidenceRequest| match engine
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            &evidence,
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source result: {other:?}"),
    };
    let essential = run(RecoveryEvidenceRequest::essential());
    let all = run(RecoveryEvidenceRequest::all());
    assert_eq!(essential.text, all.text);
    assert_eq!(essential.methods.len(), all.methods.len());
    for (left, right) in essential.methods.iter().zip(&all.methods) {
        assert_eq!(left.item.identity, right.item.identity);
        if let (
            ClassSourceOutcome::Recovered { report: left, .. },
            ClassSourceOutcome::Recovered { report: right, .. },
        ) = (&left.outcome, &right.outcome)
        {
            assert_eq!(left.text, right.text);
        }
    }
}

#[test]
fn budget_and_cancellation_never_publish_a_partial_parameterized_header() {
    let name = "OrdinaryAtomic";
    let bytes = compiled_bytes(
        name,
        r#"
        import java.util.List;
        public class OrdinaryAtomic {
            public static List<String> pass(List<String> input) { return input; }
        }
    "#,
    );
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut task_budget(&[]).unwrap())
        .unwrap();
    let request = request(&snapshot, name);
    let baseline = match engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected baseline: {other:?}"),
    };
    let full = "java.util.List<java.lang.String> pass(java.util.List<java.lang.String> arg0)";
    let erased = "java.util.List pass(java.util.List arg0)";
    assert!(baseline.text.contains(full));
    for (dimension, used) in [
        (
            BudgetDimension::AttributeBytes,
            baseline.usage.attribute_bytes,
        ),
        (
            BudgetDimension::AnalysisSteps,
            baseline.usage.analysis_steps,
        ),
        (BudgetDimension::OutputBytes, baseline.usage.output_bytes),
    ] {
        let mut limits = task_limits(&[]).unwrap();
        let limit = used.saturating_sub(1);
        match dimension {
            BudgetDimension::AttributeBytes => limits.attribute_bytes = limit,
            BudgetDimension::AnalysisSteps => limits.analysis_steps = limit,
            BudgetDimension::OutputBytes => limits.output_bytes = limit,
            _ => unreachable!(),
        }
        let outcome = engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &request,
                &mut Budget::new(limits),
            )
            .unwrap();
        if let OperationOutcome::Performed(report) = outcome {
            if let Some(method) = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"pass")
            {
                assert_eq!(
                    method.item.descriptor.raw().0,
                    b"(Ljava/util/List;)Ljava/util/List;"
                );
                assert!(
                    method
                        .declaration
                        .as_deref()
                        .is_some_and(|text| text.contains(full) || text.contains(erased)),
                    "{dimension:?}: {}",
                    report.text
                );
            }
        }
    }
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let outcome = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::with_cancellation_token(task_limits(&[]).unwrap(), cancellation),
        )
        .unwrap();
    if let OperationOutcome::Performed(report) = outcome {
        assert!(!report.text.contains(full));
    }
}

#[test]
fn ordinary_wildcards_instance_and_bodyless_members_are_complete() {
    let original = r#"
        import java.util.List;
        public abstract class OrdinaryKinds {
            public List<? super Number> instance(List<? super Number> input) { return input; }
            public abstract List<?> absent(List<?> input);
            public native List<String>[] nativeCall(List<String>[] input);
        }
    "#;
    let report = compile_source("OrdinaryKinds", original);
    assert!(report.text.contains("java.util.List<? super java.lang.Number> instance(java.util.List<? super java.lang.Number> arg1)"), "{}", report.text);
    assert!(
        report
            .text
            .contains("abstract java.util.List<?> absent(java.util.List<?> arg1);"),
        "{}",
        report.text
    );
    assert!(report.text.contains("native java.util.List<java.lang.String>[] nativeCall(java.util.List<java.lang.String>[] arg1);"), "{}", report.text);
    recompile("OrdinaryKinds", &report.text);
    let runner = r#"
        import java.lang.reflect.Method;
        import java.util.List;
        public class OrdinaryReflectionRunner {
            public static void main(String[] args) throws Exception {
                for (String name : new String[] {"instance", "absent", "nativeCall"}) {
                    Method method = OrdinaryKinds.class.getMethod(name, name.equals("nativeCall") ? List[].class : List.class);
                    System.out.println(name + ":" + method.getGenericParameterTypes()[0].getTypeName()
                        + ":" + method.getGenericReturnType().getTypeName());
                }
            }
        }
    "#;
    assert_eq!(
        java_output("OrdinaryKinds", original, runner),
        java_output("OrdinaryKinds", &report.text, runner)
    );
}

#[test]
fn interface_bodyless_method_keeps_its_generic_return() {
    let original = r#"
        import java.util.List;
        public interface OrdinaryInterface {
            List<? extends Number> get(List<? extends Number> input);
        }
    "#;
    let report = compile_source("OrdinaryInterface", original);
    assert!(report.text.contains("java.util.List<? extends java.lang.Number> get(java.util.List<? extends java.lang.Number> arg1);"), "{}", report.text);
    recompile("OrdinaryInterface", &report.text);
    let runner = r#"
        import java.util.List;
        import java.lang.reflect.Method;
        public class OrdinaryReflectionRunner {
            public static void main(String[] args) throws Exception {
                Method method = OrdinaryInterface.class.getMethod("get", List.class);
                System.out.print(method.getGenericParameterTypes()[0].getTypeName()
                    + "|" + method.getGenericReturnType().getTypeName());
            }
        }
    "#;
    assert_eq!(
        java_output("OrdinaryInterface", original, runner),
        java_output("OrdinaryInterface", &report.text, runner)
    );
}

#[test]
fn varargs_exception_and_method_annotation_keep_their_positions() {
    let report = compile_source(
        "OrdinaryDecorations",
        r#"
        import java.util.List;
        import java.io.IOException;
        public class OrdinaryDecorations {
            @Deprecated
            @SafeVarargs
            public static List<String>[] spread(@Deprecated List<String>... values) throws IOException {
                return values;
            }
        }
    "#,
    );
    assert!(
        report.text.contains("@java.lang.Deprecated"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("@java.lang.SafeVarargs"),
        "{}",
        report.text
    );
    assert!(report.text.contains("java.util.List<java.lang.String>[] spread(@java.lang.Deprecated java.util.List<java.lang.String>... arg0) throws java.io.IOException"), "{}", report.text);
    recompile("OrdinaryDecorations", &report.text);
}

#[test]
fn class_variable_and_unproved_body_keep_their_separate_evidence() {
    let class_variable = compile_source(
        "OrdinaryClassVariable",
        r#"
        import java.util.List;
        public class OrdinaryClassVariable<T> {
            public List<T> pass(List<T> input) { return input; }
        }
    "#,
    );
    assert!(
        class_variable
            .text
            .contains("class OrdinaryClassVariable<T>"),
        "{}",
        class_variable.text
    );
    assert!(
        class_variable
            .text
            .contains("java.util.List<T> pass(java.util.List<T> arg1)"),
        "{}",
        class_variable.text
    );
    recompile("OrdinaryClassVariable", &class_variable.text);

    let cast = compile_source(
        "OrdinaryCast",
        r#"
        import java.util.List;
        public class OrdinaryCast {
            @SuppressWarnings("unchecked")
            public static List<String> convert(List<Integer> input) { return (List<String>) (List<?>) input; }
        }
    "#,
    );
    assert!(
        cast.text.contains("ordinary_generic_source_unproved"),
        "{}",
        cast.text
    );
    assert!(
        !cast
            .text
            .contains("java.util.List<java.lang.String> convert")
    );
}

#[test]
fn class_variable_direct_return_preserves_generic_caller_and_reflection() {
    let original = r#"
        public class ClassVariableIdentity<U> {
            public U identity(U input) { return input; }
        }
    "#;
    let report = compile_source("ClassVariableIdentity", original);
    assert!(
        report.text.contains("class ClassVariableIdentity<U>"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("U identity(U arg1)"),
        "{}",
        report.text
    );
    let runner = r#"
        public class OrdinaryReflectionRunner {
            public static void main(String[] args) throws Exception {
                ClassVariableIdentity<String> value = new ClassVariableIdentity<String>();
                System.out.println(value.identity("hello"));
                System.out.println(ClassVariableIdentity.class.getTypeParameters()[0].getName());
                System.out.println(ClassVariableIdentity.class.getMethod("identity", Object.class).getGenericReturnType().getTypeName());
            }
        }
    "#;
    assert_eq!(
        java_output("ClassVariableIdentity", original, runner),
        java_output("ClassVariableIdentity", &report.text, runner)
    );
}

#[test]
fn class_variable_header_is_atomic_across_evidence_and_budget() {
    let name = "ClassVariableAtomic";
    let bytes = compiled_bytes(
        name,
        "public class ClassVariableAtomic<U> { public U identity(U input) { return input; } }",
    );
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut task_budget(&[]).unwrap())
        .unwrap();
    let request = request(&snapshot, name);
    let run = |evidence: RecoveryEvidenceRequest, budget: &mut Budget| match engine
        .class_source_with_evidence(std::slice::from_ref(&snapshot), &request, &evidence, budget)
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source result: {other:?}"),
    };
    let baseline = run(
        RecoveryEvidenceRequest::essential(),
        &mut task_budget(&[]).unwrap(),
    );
    let all = run(
        RecoveryEvidenceRequest::all(),
        &mut task_budget(&[]).unwrap(),
    );
    assert_eq!(baseline.text, all.text);
    assert!(baseline.text.contains("class ClassVariableAtomic<U>"));
    for (dimension, used) in [
        (
            BudgetDimension::AttributeBytes,
            baseline.usage.attribute_bytes,
        ),
        (
            BudgetDimension::AnalysisSteps,
            baseline.usage.analysis_steps,
        ),
        (BudgetDimension::OutputBytes, baseline.usage.output_bytes),
    ] {
        let mut limits = task_limits(&[]).unwrap();
        let limit = used.saturating_sub(1);
        match dimension {
            BudgetDimension::AttributeBytes => limits.attribute_bytes = limit,
            BudgetDimension::AnalysisSteps => limits.analysis_steps = limit,
            BudgetDimension::OutputBytes => limits.output_bytes = limit,
            _ => unreachable!(),
        }
        if let OperationOutcome::Performed(report) = engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &request,
                &mut Budget::new(limits),
            )
            .unwrap()
        {
            let class_has_variable = report.declaration.as_ref().is_some_and(|declaration| {
                declaration.declaration.contains("ClassVariableAtomic<U>")
            });
            if !class_has_variable {
                assert!(!report.text.contains("U identity(U"), "{}", report.text);
            }
        }
    }
}

#[test]
fn bounded_class_variable_on_bodyless_interface_preserves_reflection() {
    let original = r#"
        public interface ClassVariableNoBody<U extends Number & java.io.Serializable> {
            U identity(U input);
        }
    "#;
    let report = compile_source("ClassVariableNoBody", original);
    assert!(
        report.text.contains(
            "interface ClassVariableNoBody<U extends java.lang.Number & java.io.Serializable>"
        ),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("U identity(U arg1);"),
        "{}",
        report.text
    );
    let runner = r#"
        public class OrdinaryReflectionRunner {
            public static void main(String[] args) throws Exception {
                System.out.println(ClassVariableNoBody.class.getTypeParameters()[0].getBounds()[0].getTypeName());
                System.out.println(ClassVariableNoBody.class.getMethod("identity", Number.class).getGenericReturnType().getTypeName());
            }
        }
    "#;
    assert_eq!(
        java_output("ClassVariableNoBody", original, runner),
        java_output("ClassVariableNoBody", &report.text, runner)
    );
}

#[test]
fn parameterized_parent_does_not_publish_an_unproved_class_variable_header() {
    let report = compile_source(
        "ClassVariableParent",
        r#"
        public interface ClassVariableParent<U> extends Comparable<U> {
            U identity(U input);
        }
    "#,
    );
    assert!(
        report
            .text
            .contains("parameterized or nested parent needs a separate inherited-member proof"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("interface ClassVariableParent<U>"));
    assert!(!report.text.contains("U identity(U"));
}

#[test]
fn type_use_annotation_is_a_whole_declaration_refusal() {
    let report = compile_source(
        "OrdinaryTypeUse",
        r#"
        import java.util.List;
        import java.lang.annotation.*;
        @Target(ElementType.TYPE_USE) @Retention(RetentionPolicy.RUNTIME) @interface Mark {}
        public class OrdinaryTypeUse {
            public static List<@Mark String> pass(List<@Mark String> input) { return input; }
        }
    "#,
    );
    assert!(
        report.text.contains("ordinary_generic_source_unproved"),
        "{}",
        report.text
    );
    assert!(
        !report
            .text
            .contains("java.util.List<java.lang.String> pass")
    );
}

#[test]
fn parameter_reassignment_cannot_use_the_identity_body_proof() {
    let report = compile_source(
        "OrdinaryParameterWrite",
        r#"
        import java.util.*;
        public class OrdinaryParameterWrite {
            public static List<String> pass(List<String> input) {
                input = new ArrayList<String>(input);
                return input;
            }
        }
    "#,
    );
    assert!(
        report.text.contains("ordinary_generic_source_unproved"),
        "{}",
        report.text
    );
    assert!(
        !report
            .text
            .contains("java.util.List<java.lang.String> pass")
    );
}

#[test]
fn same_class_overload_caller_refuses_the_target_projection() {
    let report = compile_source(
        "Boundaries",
        include_str!(
            "../openspec/evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/Boundaries.java"
        ),
    );
    let target = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"bodyOverload")
        .unwrap();
    assert_eq!(
        target.item.descriptor.raw().0,
        b"(Ljava/util/List;)Ljava/util/List;"
    );
    assert!(
        target.text.contains("generic_call_binding_unproved"),
        "{}",
        target.text
    );
    assert!(target.declaration.as_deref().is_some_and(|declaration| {
        declaration.contains("java.util.List bodyOverload(java.util.List arg0)")
    }));
    let positive = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"bodyPositive")
        .unwrap();
    assert!(positive.declaration.as_deref().is_some_and(|declaration| declaration.contains("java.util.List<java.lang.String> bodyPositive(java.util.List<java.lang.String> arg0)")));
}
