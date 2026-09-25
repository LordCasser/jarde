use jarde::*;
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn source(bytes: &[u8], name: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    let request = ClassSourceRequest {
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
    };
    match engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source outcome: {other:?}"),
    }
}

#[test]
fn bounded_method_variable_follows_same_run_parameters() {
    let report = source(
        include_bytes!(
            "../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/GenericMethodProbe.class"
        ),
        "GenericMethodProbe",
    );
    assert!(
        report.text.contains(
            "public static <T extends java.lang.Number> T choose(T arg0, T arg1, boolean arg2)"
        ),
        "{}",
        report.text
    );
    assert!(report.text.contains("return arg2 ? arg0 : arg1;"));
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"choose")
        .unwrap();
    assert_eq!(
        method.item.descriptor.raw().0,
        b"(Ljava/lang/Number;Ljava/lang/Number;Z)Ljava/lang/Number;"
    );
    assert!(
        method
            .markers
            .iter()
            .any(|marker| marker.contains("same-run AST/SSA"))
    );
}

#[test]
fn exceptions_without_generic_throws_suffix_survive() {
    let report = source(
        include_bytes!(
            "../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/GenericThrowsProbe.class"
        ),
        "GenericThrowsProbe",
    );
    assert!(
        report
            .text
            .contains("<T extends java.lang.Number> T choose(T arg0) throws java.io.IOException"),
        "{}",
        report.text
    );
}

#[test]
fn erasure_shape_and_body_refusals_keep_descriptor_declarations() {
    for (name, bytes) in [
        (
            "GenericMethodProbe",
            &include_bytes!(
                "../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/negative-fixtures/classes/object-bound.class"
            )[..],
        ),
        (
            "GenericMethodProbe",
            &include_bytes!(
                "../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/negative-fixtures/classes/unbound-variable.class"
            )[..],
        ),
        (
            "ComplexMethodProbe",
            &include_bytes!(
                "../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/negative-fixtures/classes/complex-method.class"
            )[..],
        ),
        (
            "IncompatibleBodyProbe",
            &include_bytes!(
                "../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/negative-fixtures/classes/incompatible-body.class"
            )[..],
        ),
    ] {
        let report = source(bytes, name);
        assert!(
            !report.text.contains("<T extends"),
            "{name}: {}",
            report.text
        );
        assert!(
            report.text.contains("generic Signature projection refused"),
            "{name}: {}",
            report.text
        );
    }
    let class_variable = source(
        include_bytes!(
            "../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/negative-fixtures/classes/class-variable.class"
        ),
        "ClassVariableProbe",
    );
    assert!(
        class_variable
            .text
            .contains("class ClassVariableProbe<T extends java.lang.Number>"),
        "{}",
        class_variable.text
    );
    assert!(
        class_variable
            .text
            .contains("T choose(T arg1, T arg2, boolean arg3)"),
        "{}",
        class_variable.text
    );
    let incompatible = source(
        include_bytes!(
            "../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/negative-fixtures/classes/incompatible-body.class"
        ),
        "IncompatibleBodyProbe",
    );
    let runner = r#"
        public class GenericReflectionRunner {
            public static void main(String[] args) throws Exception {
                System.out.print(IncompatibleBodyProbe.choose(1, 2, true));
                System.out.print("|" + IncompatibleBodyProbe.class.getDeclaredMethod(
                    "choose", Number.class, Number.class, boolean.class).getTypeParameters().length);
            }
        }
    "#;
    assert_eq!(
        java_output("IncompatibleBodyProbe", &incompatible.text, runner),
        "7|0"
    );
}

fn compiled_source(name: &str, java: &str) -> ClassSourceReport {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "jarde-generic-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&dir).unwrap();
    let path = dir.join(format!("{name}.java"));
    fs::write(&path, java).unwrap();
    let output = Command::new("javac")
        .args(["--release", "8", "-g:none", "-Xlint:-options"])
        .arg("-d")
        .arg(&dir)
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = fs::read(dir.join(format!("{name}.class"))).unwrap();
    let report = source(&bytes, name);
    fs::remove_dir_all(dir).unwrap();
    report
}

fn java_output(name: &str, java: &str, runner: &str) -> String {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "jarde-generic-run-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&dir).unwrap();
    let class = dir.join(format!("{name}.java"));
    let runner_path = dir.join("GenericReflectionRunner.java");
    fs::write(&class, java).unwrap();
    fs::write(&runner_path, runner).unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-Xlint:-options"])
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
        .arg("GenericReflectionRunner")
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
fn neighboring_overload_without_a_caller_does_not_block_the_proved_method() {
    let report = compiled_source(
        "GenericOverloadProbe",
        r#"
        public class GenericOverloadProbe {
            public static <T extends Number> T choose(T a, T b, boolean first) { return first ? a : b; }
            public static Number choose(Number a, Number b) { return a; }
        }
    "#,
    );
    assert!(
        report
            .text
            .contains("<T extends java.lang.Number> T choose(T arg0, T arg1, boolean arg2)"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("java.lang.Number choose(java.lang.Number arg0, java.lang.Number arg1)")
    );
}

#[test]
fn same_class_caller_to_overloaded_name_refuses_projection() {
    let report = compiled_source(
        "GenericCallerProbe",
        r#"
        public class GenericCallerProbe {
            public static <T extends Number> T choose(T a, T b, boolean first) { return first ? a : b; }
            public static Number choose(Number a, Number b) { return a; }
            public static Number call() { return choose(Integer.valueOf(1), Integer.valueOf(2), true); }
        }
    "#,
    );
    assert!(!report.text.contains("<T extends"));
    assert!(
        report.text.contains("generic_call_binding_unproved"),
        "{}",
        report.text
    );
}

#[test]
fn type_use_annotations_and_varargs_are_whole_method_refusals() {
    let annotated = compiled_source(
        "GenericTypeUseProbe",
        r#"
        import java.lang.annotation.*;
        @Target(ElementType.TYPE_USE) @Retention(RetentionPolicy.RUNTIME) @interface Mark {}
        public class GenericTypeUseProbe {
            public static <T extends Number> @Mark T choose(@Mark T a) { return a; }
        }
    "#,
    );
    assert!(!annotated.text.contains("<T extends"));
    assert!(
        annotated.text.contains("generic_source_shape_unproved"),
        "{}",
        annotated.text
    );
    let varargs = compiled_source(
        "GenericVarargsProbe",
        r#"
        public class GenericVarargsProbe {
            @SafeVarargs public static <T extends Number> T choose(T... values) { return values[0]; }
        }
    "#,
    );
    assert!(!varargs.text.contains("<T extends"));
    assert!(
        varargs.text.contains("generic_source_shape_unproved"),
        "{}",
        varargs.text
    );
}

#[test]
fn multiple_local_variables_and_ordinary_parameters_keep_their_positions() {
    let original = r#"
        public class GenericTwoVariablesProbe {
            public static <T extends Number, U extends CharSequence> T pass(T value, U other, String tag, int count) {
                return value;
            }
        }
    "#;
    let report = compiled_source("GenericTwoVariablesProbe", original);
    assert!(
        report.text.contains("<T extends java.lang.Number, U extends java.lang.CharSequence> T pass(T arg0, U arg1, java.lang.String arg2, int arg3)"),
        "{}",
        report.text
    );
    let runner = r#"
        import java.lang.reflect.*;
        import java.util.Arrays;
        public class GenericReflectionRunner {
            public static void main(String[] args) throws Exception {
                Method method = GenericTwoVariablesProbe.class.getDeclaredMethod(
                    "pass", Number.class, CharSequence.class, String.class, int.class);
                TypeVariable<Method>[] variables = method.getTypeParameters();
                System.out.print(GenericTwoVariablesProbe.pass(7, "a", "b", 1));
                System.out.print("|" + variables.length);
                for (TypeVariable<Method> variable : variables) {
                    System.out.print("|" + variable.getName() + ":" + Arrays.toString(variable.getBounds()));
                }
                System.out.print("|" + Arrays.toString(method.getGenericParameterTypes()));
                System.out.print("|" + method.getGenericReturnType());
            }
        }
    "#;
    assert_eq!(
        java_output("GenericTwoVariablesProbe", original, runner),
        java_output("GenericTwoVariablesProbe", &report.text, runner)
    );
}

#[test]
fn direct_return_and_generic_throws_can_share_the_method_variable() {
    let report = compiled_source(
        "GenericThrowsVariableProbe",
        r#"
        public class GenericThrowsVariableProbe {
            public static <T extends Exception> T pass(T value) throws T { return value; }
        }
    "#,
    );
    assert!(
        report
            .text
            .contains("<T extends java.lang.Exception> T pass(T arg0) throws T"),
        "{}",
        report.text
    );
    let runner = r#"
        public class GenericReflectionRunner {
            public static void main(String[] args) throws Exception {
                java.lang.reflect.Method method = GenericThrowsVariableProbe.class.getDeclaredMethod(
                    "pass", Exception.class);
                System.out.print(GenericThrowsVariableProbe.<RuntimeException>pass(new RuntimeException()).getClass().getName());
                System.out.print("|" + method.getTypeParameters().length);
                System.out.print("|" + method.getGenericReturnType());
                System.out.print("|" + method.getGenericExceptionTypes()[0]);
            }
        }
    "#;
    assert_eq!(
        java_output("GenericThrowsVariableProbe", &report.text, runner),
        "java.lang.RuntimeException|1|T|T"
    );
}

#[test]
fn body_assignment_and_call_without_type_variable_proof_are_refused() {
    let assignment = compiled_source(
        "GenericAssignmentProbe",
        r#"
        public class GenericAssignmentProbe {
            public static <T extends Number> T pass(T value) {
                Number copy = value;
                return value;
            }
        }
    "#,
    );
    assert!(
        !assignment.text.contains("<T extends"),
        "{}",
        assignment.text
    );
    assert!(assignment.text.contains("generic_source_shape_unproved"));

    let call = compiled_source(
        "GenericBodyCallProbe",
        r#"
        public class GenericBodyCallProbe {
            public static <T extends Number> T pass(T value) { return helper(value); }
            private static <T extends Number> T helper(T value) { return value; }
        }
    "#,
    );
    let pass = call
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"pass")
        .unwrap();
    assert!(!pass.text.contains("<T extends"), "{}", pass.text);
    assert!(pass.text.contains("generic Signature projection refused"));
}
