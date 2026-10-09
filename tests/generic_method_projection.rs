use jarde::*;
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn source(bytes: &[u8], name: &str) -> ClassSourceReport {
    let mut budget = task_budget(&[]).unwrap();
    match source_with_budget(bytes, name, &mut budget) {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source outcome: {other:?}"),
    }
}

fn source_with_budget(
    bytes: &[u8],
    name: &str,
    budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
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
    engine
        .class_source(std::slice::from_ref(&snapshot), &request, budget)
        .unwrap()
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
fn exact_generic_instance_null_return_projects_and_preserves_physical_method() {
    let original = r#"
        public class GenericNullReturnProbe {
            public <T extends Number> T value() { return null; }
        }
    "#;
    let report = compiled_source("GenericNullReturnProbe", original);
    let value = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"value")
        .expect("the physical value method remains in the method table");
    assert_eq!(
        value.item.descriptor.raw().0,
        b"()Ljava/lang/Number;",
        "projection keeps the physical descriptor"
    );
    assert!(value.declaration.as_deref().is_some_and(|declaration| {
        declaration.contains("public <T extends java.lang.Number> T value()")
    }));
    assert!(value.text.contains("return null;"));
    assert!(
        value
            .text
            .contains("same-run AST/Code/SSA exact null-return")
    );

    let runner = r#"
        import java.lang.reflect.*;
        import java.util.Arrays;
        public class GenericReflectionRunner {
            public static void main(String[] args) throws Exception {
                GenericNullReturnProbe probe = new GenericNullReturnProbe();
                Integer value = probe.<Integer>value();
                Method method = GenericNullReturnProbe.class.getDeclaredMethod("value");
                TypeVariable<Method> variable = method.getTypeParameters()[0];
                System.out.print(value == null);
                System.out.print(":" + variable.getName());
                System.out.print(":" + Arrays.toString(variable.getBounds()));
                System.out.print(":" + method.getGenericReturnType());
            }
        }
    "#;
    assert_eq!(
        java_output("GenericNullReturnProbe", original, runner),
        java_output("GenericNullReturnProbe", &report.text, runner)
    );
}

#[test]
fn exact_parameterized_list_null_return_projects_with_raw_controls_unchanged() {
    let original = r#"
        import java.util.Arrays;
        import java.util.List;
        public class ParameterizedNullReturnProbe {
            public List<String> names;
            public List raw;
            public List<String> id(List<String> input) { return input; }
            public List<String> empty() { return null; }
            public List raw(List input) { return input; }
        }
    "#;
    let report = compiled_source("ParameterizedNullReturnProbe", original);
    let empty = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"empty")
        .expect("the physical empty method remains in the report");
    assert_eq!(empty.item.descriptor.raw().0, b"()Ljava/util/List;");
    assert!(empty.declaration.as_deref().is_some_and(|declaration| {
        declaration.contains("java.util.List<java.lang.String> empty()")
    }));
    assert!(empty.text.contains("return null;"));
    assert!(
        empty
            .text
            .contains("same-run AST/Code/SSA exact null-return")
    );
    assert!(
        report
            .text
            .contains("java.util.List<java.lang.String> names;")
    );
    assert!(report.text.contains("java.util.List raw;"));

    let runner = r#"
        import java.lang.reflect.*;
        import java.util.Arrays;
        import java.util.List;
        public class GenericReflectionRunner {
            public static void main(String[] args) throws Exception {
                Class<?> type = ParameterizedNullReturnProbe.class;
                Method id = type.getDeclaredMethod("id", List.class);
                Method empty = type.getDeclaredMethod("empty");
                Method raw = type.getDeclaredMethod("raw", List.class);
                ParameterizedNullReturnProbe probe = new ParameterizedNullReturnProbe();
                System.out.println(type.getDeclaredField("names").getGenericType());
                System.out.println(id.getGenericParameterTypes()[0]);
                System.out.println(id.getGenericReturnType());
                System.out.println(empty.getGenericReturnType());
                System.out.println(type.getDeclaredField("raw").getGenericType());
                System.out.println(raw.getGenericParameterTypes()[0]);
                System.out.println(raw.getGenericReturnType());
                System.out.println(probe.id(Arrays.asList("ok")).get(0) + ":" + (probe.empty() == null));
            }
        }
    "#;
    let output = java_output("ParameterizedNullReturnProbe", original, runner);
    assert_eq!(
        output,
        concat!(
            "java.util.List<java.lang.String>\n",
            "java.util.List<java.lang.String>\n",
            "java.util.List<java.lang.String>\n",
            "java.util.List<java.lang.String>\n",
            "interface java.util.List\n",
            "interface java.util.List\n",
            "interface java.util.List\n",
            "ok:true\n",
        )
    );
    assert_eq!(
        output,
        java_output("ParameterizedNullReturnProbe", &report.text, runner)
    );
}

#[test]
fn parameterized_null_return_rejects_other_signatures_effects_and_self_calls() {
    for (name, java) in [
        (
            "ParameterizedNullOtherTypeProbe",
            r#"
                import java.util.List;
                public class ParameterizedNullOtherTypeProbe {
                    public List<Integer> empty() { return null; }
                }
            "#,
        ),
        (
            "ParameterizedNullEffectProbe",
            r#"
                import java.util.List;
                public class ParameterizedNullEffectProbe {
                    static int effects;
                    public List<String> empty() { effects++; return null; }
                }
            "#,
        ),
    ] {
        let report = compiled_source(name, java);
        let method = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"empty")
            .expect("the physical empty method remains in the report");
        assert_eq!(method.item.descriptor.raw().0, b"()Ljava/util/List;");
        assert!(!method.declaration.as_deref().is_some_and(|declaration| {
            declaration.contains("java.util.List<java.lang.String> empty()")
        }));
        assert!(method.text.contains("generic Signature projection refused"));
    }
}

#[test]
fn parameterized_null_return_same_class_caller_projects_after_use_proof() {
    let report = compiled_source(
        "ParameterizedNullBindingProbe",
        r#"
            import java.util.List;
            public class ParameterizedNullBindingProbe {
                public List<String> empty() { return null; }
                public List<String> caller() { return empty(); }
            }
        "#,
    );
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"empty")
        .expect("the physical empty method remains in the report");
    assert_eq!(method.item.descriptor.raw().0, b"()Ljava/util/List;");
    // `caller`'s direct `empty()` site is the one same-class use: it names this member's exact
    // descriptor and no sibling of that name is declared, so the null-return projection admits
    // with the same-class binding proved.
    assert!(method.declaration.as_deref().is_some_and(|declaration| {
        declaration.contains("java.util.List<java.lang.String> empty()")
    }));
    assert!(method.text.contains("same-class call binding proved"));
}

#[test]
fn parameterized_null_return_erasure_budget_and_cancellation_keep_raw_method() {
    let mut bytes = compile_class_bytes(
        "ParameterizedNullStopProbe",
        r#"
            import java.util.List;
            public class ParameterizedNullStopProbe {
                public List<String> empty() { return null; }
            }
        "#,
    );
    let descriptor = b"()Ljava/util/List;";
    let wrong_descriptor = b"()Ljava/util/Date;";
    assert_eq!(descriptor.len(), wrong_descriptor.len());
    let descriptor_at = bytes
        .windows(descriptor.len())
        .position(|window| window == descriptor)
        .expect("the physical List descriptor is present in the class constant pool");
    bytes[descriptor_at..descriptor_at + descriptor.len()].copy_from_slice(wrong_descriptor);
    let mismatch = source(&bytes, "ParameterizedNullStopProbe");
    let method = mismatch
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"empty")
        .expect("the physical method remains present after a signature mismatch");
    assert_eq!(method.item.descriptor.raw().0, wrong_descriptor);
    assert!(method.text.contains("jvm_signature_erasure_mismatch"));
    assert!(!method.text.contains("List<java.lang.String> empty()"));

    let bytes = compile_class_bytes(
        "ParameterizedNullStopProbe",
        r#"
            import java.util.List;
            public class ParameterizedNullStopProbe {
                public List<String> empty() { return null; }
            }
        "#,
    );
    let mut output_limited = task_budget(&[BudgetOverride::new("output_bytes", 1).unwrap()])
        .expect("one output byte is a valid stop boundary");
    match source_with_budget(&bytes, "ParameterizedNullStopProbe", &mut output_limited) {
        OperationOutcome::Performed(report) => {
            assert!(!report.text.contains("List<java.lang.String> empty()"));
            assert!(matches!(
                report.execution,
                jarde_reader::model::ExecutionReport::Partial {
                    reason: jarde_reader::model::TerminationReason::BudgetExceeded { .. },
                    ..
                }
            ));
            let method = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"empty")
                .expect("the physical method remains present in the partial report");
            assert_eq!(method.item.descriptor.raw().0, b"()Ljava/util/List;");
        }
        OperationOutcome::Incomplete(report) => assert!(matches!(
            report.execution,
            jarde_reader::model::ExecutionReport::Partial {
                reason: jarde_reader::model::TerminationReason::BudgetExceeded { .. },
                ..
            }
        )),
        other => panic!("unexpected output-limited outcome: {other:?}"),
    }
    let mut cancelled = task_budget(&[]).unwrap();
    cancelled.cancellation_token().cancel();
    assert!(matches!(
        source_with_budget(&bytes, "ParameterizedNullStopProbe", &mut cancelled),
        OperationOutcome::Incomplete(report)
            if matches!(report.execution, jarde_reader::model::ExecutionReport::Cancelled { .. })
    ));
}

#[test]
fn generic_instance_null_return_rejects_unproved_signatures_bodies_and_bindings() {
    let cases = [
        (
            "GenericNullEffectProbe",
            r#"
                public class GenericNullEffectProbe {
                    static int effects;
                    public <T extends Number> T value() { effects++; return null; }
                }
            "#,
            "body effect",
        ),
        (
            "GenericNullHandlerProbe",
            r#"
                public class GenericNullHandlerProbe {
                    public <T extends Number> T value() {
                        try { throw new IllegalStateException(); }
                        catch (IllegalStateException expected) { return null; }
                    }
                }
            "#,
            "exception handler and extra operations",
        ),
        (
            "GenericNullPhiProbe",
            r#"
                public class GenericNullPhiProbe {
                    public <T extends Number> T value(boolean choose) {
                        return choose ? null : null;
                    }
                }
            "#,
            "conditional return and SSA merge",
        ),
        (
            "GenericNullClassVariableProbe",
            r#"
                public class GenericNullClassVariableProbe<U extends Number> {
                    public <T extends Number> T value() { return null; }
                }
            "#,
            "class variable scope",
        ),
        (
            "GenericNullAnnotatedProbe",
            r#"
                import java.lang.annotation.*;
                @Target(ElementType.TYPE_USE) @Retention(RetentionPolicy.RUNTIME)
                @interface GenericNullMark {}
                public class GenericNullAnnotatedProbe {
                    public <T extends Number> @GenericNullMark T value() { return null; }
                }
            "#,
            "type-use annotation",
        ),
        (
            "GenericNullBoundProbe",
            r#"
                public class GenericNullBoundProbe {
                    public <T extends CharSequence> T value() { return null; }
                }
            "#,
            "descriptor and bound",
        ),
        (
            "GenericNullStaticProbe",
            r#"
                public class GenericNullStaticProbe {
                    public static <T extends Number> T value() { return null; }
                }
            "#,
            "static method outside the DT-16 instance slice",
        ),
    ];
    for (name, java, boundary) in cases {
        let report = compiled_source(name, java);
        let method = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"value")
            .unwrap_or_else(|| panic!("{boundary}: physical method missing"));
        assert!(
            !method
                .text
                .contains("<T extends java.lang.Number> T value()"),
            "{boundary}: {}",
            method.text
        );
        assert!(
            method.text.contains("generic Signature projection refused"),
            "{boundary}: {}",
            method.text
        );
        assert!(
            method.declaration.as_deref().is_some_and(|declaration| {
                declaration.contains("java.lang.") && declaration.contains("value(")
            }),
            "{boundary}: physical declaration missing: {method:?}"
        );
    }
}

#[test]
fn generic_instance_null_return_admits_an_arity_disjoint_same_class_overload() {
    let original = r#"
            public class GenericNullBindingProbe {
                public <T extends Number> T value() { return null; }
                public Number value(Number input) { return input; }
                public Number caller() { return value(); }
            }
        "#;
    let report = compiled_source("GenericNullBindingProbe", original);
    let generic = report
        .methods
        .iter()
        .find(|method| method.item.descriptor.raw().0 == b"()Ljava/lang/Number;")
        .expect("the physical generic value() remains in the report");
    // `caller`'s `value()` site names the no-argument member exactly; the one-argument sibling
    // cannot apply there, so the null-return header projects with the binding proved.
    assert!(
        generic
            .text
            .contains("<T extends java.lang.Number> T value()"),
        "{}",
        generic.text
    );
    assert!(generic.text.contains("same-class call binding proved"));
    let sibling = report
        .methods
        .iter()
        .find(|method| method.item.descriptor.raw().0 == b"(Ljava/lang/Number;)Ljava/lang/Number;")
        .expect("the physical sibling remains in the report");
    assert!(
        sibling
            .declaration
            .as_deref()
            .is_some_and(|declaration| declaration.contains("java.lang.Number value("))
    );
    let runner = r#"
        public class GenericReflectionRunner {
            public static void main(String[] args) throws Exception {
                GenericNullBindingProbe probe = new GenericNullBindingProbe();
                java.lang.reflect.Method method = GenericNullBindingProbe.class.getDeclaredMethod("value");
                java.lang.reflect.TypeVariable<?> variable = method.getTypeParameters()[0];
                System.out.print((probe.caller() == null) + ":" + probe.value(Integer.valueOf(7)));
                System.out.print(":" + variable.getGenericDeclaration().equals(method));
                System.out.print(":" + method.getGenericReturnType().equals(variable));
                System.out.print(":" + java.util.Arrays.toString(variable.getBounds()));
            }
        }
    "#;
    assert_eq!(
        java_output("GenericNullBindingProbe", original, runner),
        "true:7:true:true:[class java.lang.Number]"
    );
    assert_eq!(
        java_output("GenericNullBindingProbe", &report.text, runner),
        "true:7:true:true:[class java.lang.Number]"
    );
}

#[test]
fn generic_null_return_preserves_its_bound_in_wider_raw_return_contexts() {
    for context in ["Object", "java.io.Serializable"] {
        let original = format!(
            r#"
            public class GenericNullWiderContextProbe {{
                public <T extends Number> T value() {{ return null; }}
                public Number value(Number input) {{ return input; }}
                public {context} caller() {{ return value(); }}
            }}
        "#
        );
        let report = compiled_source("GenericNullWiderContextProbe", &original);
        assert!(
            report
                .text
                .contains("<T extends java.lang.Number> T value()"),
            "{context}: {}",
            report.text
        );
        let runner = r#"
            public class GenericReflectionRunner {
                public static void main(String[] args) throws Exception {
                    GenericNullWiderContextProbe probe = new GenericNullWiderContextProbe();
                    java.lang.reflect.Method method = GenericNullWiderContextProbe.class.getDeclaredMethod("value");
                    java.lang.reflect.TypeVariable<?> variable = method.getTypeParameters()[0];
                    System.out.print((probe.caller() == null) + ":" + probe.value(Integer.valueOf(7)));
                    System.out.print(":" + variable.getGenericDeclaration().equals(method));
                    System.out.print(":" + method.getGenericReturnType().equals(variable));
                    System.out.print(":" + java.util.Arrays.toString(variable.getBounds()));
                }
            }
        "#;
        let expected = "true:7:true:true:[class java.lang.Number]";
        assert_eq!(
            java_output("GenericNullWiderContextProbe", &original, runner),
            expected
        );
        assert_eq!(
            java_output("GenericNullWiderContextProbe", &report.text, runner),
            expected
        );
    }
}

#[test]
fn already_proved_abstract_method_contract_survives_a_raw_direct_parameter_receiver() {
    let original = r#"
        public abstract class GenericAbstractReceiverProbe {
            public abstract <T extends Number> T target(T value);
            public static Number call(GenericAbstractReceiverProbe value) {
                return value.target(Integer.valueOf(7));
            }
            public Number own() { return this.target(Integer.valueOf(8)); }
        }
    "#;
    let report = compiled_source("GenericAbstractReceiverProbe", original);
    assert!(
        report
            .text
            .contains("<T extends java.lang.Number> T target(T arg1);"),
        "{}",
        report.text
    );
    let runner = r#"
        public class GenericReflectionRunner {
            public static void main(String[] args) throws Exception {
                java.lang.reflect.Method method = GenericAbstractReceiverProbe.class.getDeclaredMethod("target", Number.class);
                java.lang.reflect.TypeVariable<?> variable = method.getTypeParameters()[0];
                System.out.print(java.lang.reflect.Modifier.isAbstract(method.getModifiers()));
                System.out.print(":" + variable.getGenericDeclaration().equals(method));
                System.out.print(":" + method.getGenericReturnType().equals(variable));
                System.out.print(":" + method.getGenericParameterTypes()[0].equals(variable));
                System.out.print(":" + java.util.Arrays.toString(variable.getBounds()));
            }
        }
    "#;
    let expected = "true:true:true:true:[class java.lang.Number]";
    assert_eq!(
        java_output("GenericAbstractReceiverProbe", original, runner),
        expected
    );
    assert_eq!(
        java_output("GenericAbstractReceiverProbe", &report.text, runner),
        expected
    );
}

#[test]
fn generic_null_return_refuses_signature_erasure_mismatch() {
    let mut bytes = compile_class_bytes(
        "GenericNullErasureProbe",
        r#"
            public class GenericNullErasureProbe {
                public <T extends Number> T value() { return null; }
            }
        "#,
    );
    let signature = b"<T:Ljava/lang/Number;>()TT;";
    let wrong = b"<T:Ljava/lang/Object;>()TT;";
    assert_eq!(signature.len(), wrong.len());
    let position = bytes
        .windows(signature.len())
        .position(|window| window == signature)
        .expect("compiled method signature is present in its constant pool");
    bytes[position..position + wrong.len()].copy_from_slice(wrong);
    let report = source(&bytes, "GenericNullErasureProbe");
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"value")
        .unwrap();
    assert_eq!(method.item.descriptor.raw().0, b"()Ljava/lang/Number;");
    assert!(method.text.contains("jvm_signature_erasure_mismatch"));
    assert!(!method.text.contains("<T extends"));
}

#[test]
fn generic_null_return_budget_and_cancellation_never_publish_a_partial_header() {
    let bytes = compile_class_bytes(
        "GenericNullStopProbe",
        r#"
            public class GenericNullStopProbe {
                public <T extends Number> T value() { return null; }
            }
        "#,
    );
    let mut output_limited = task_budget(&[BudgetOverride::new("output_bytes", 1).unwrap()])
        .expect("one output byte is a valid stop boundary");
    match source_with_budget(&bytes, "GenericNullStopProbe", &mut output_limited) {
        OperationOutcome::Performed(report) => {
            assert!(
                !report
                    .text
                    .contains("<T extends java.lang.Number> T value()")
            );
            assert!(matches!(
                report.execution,
                jarde_reader::model::ExecutionReport::Partial {
                    reason: jarde_reader::model::TerminationReason::BudgetExceeded { .. },
                    ..
                }
            ));
            let method = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"value")
                .expect("the partial report retains the physical method identity");
            assert_eq!(method.item.descriptor.raw().0, b"()Ljava/lang/Number;");
        }
        OperationOutcome::Incomplete(report) => {
            assert!(matches!(
                report.execution,
                jarde_reader::model::ExecutionReport::Partial {
                    reason: jarde_reader::model::TerminationReason::BudgetExceeded { .. },
                    ..
                }
            ));
        }
        other => panic!("unexpected output-limited outcome: {other:?}"),
    }

    let mut cancelled = task_budget(&[]).unwrap();
    cancelled.cancellation_token().cancel();
    assert!(matches!(
        source_with_budget(&bytes, "GenericNullStopProbe", &mut cancelled),
        OperationOutcome::Incomplete(report)
            if matches!(report.execution, jarde_reader::model::ExecutionReport::Cancelled { .. })
    ));
    let complete = source(&bytes, "GenericNullStopProbe");
    let physical = complete
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"value")
        .expect("an independent complete query still exposes the physical method");
    assert_eq!(physical.item.descriptor.raw().0, b"()Ljava/lang/Number;");
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
    let bytes = compile_class_bytes(name, java);
    source(&bytes, name)
}

fn compile_class_bytes(name: &str, java: &str) -> Vec<u8> {
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
        .args(["-classpath", "", "-sourcepath", ""])
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
    fs::remove_dir_all(dir).unwrap();
    bytes
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
        .args(["-classpath", "", "-sourcepath", ""])
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
fn same_class_caller_to_arity_disjoint_overload_proves_projection() {
    let original = r#"
        public class GenericCallerProbe {
            public static <T extends Number> T choose(T a, T b, boolean first) { return first ? a : b; }
            public static Number choose(Number a, Number b) { return a; }
            public static Number call() { return choose(Integer.valueOf(1), Integer.valueOf(2), true); }
        }
    "#;
    let report = compiled_source("GenericCallerProbe", original);
    // The three-arity candidate's only same-class site names its exact descriptor, and the
    // two-arity sibling cannot apply at that site, so the binding is proved and the header
    // projects; the sibling keeps whatever its own proof supports.
    assert!(
        report
            .text
            .contains("<T extends java.lang.Number> T choose(T arg0, T arg1, boolean arg2)"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("same-class call binding proved"),
        "{}",
        report.text
    );
    let runner = r#"
        public class GenericReflectionRunner {
            public static void main(String[] args) throws Exception {
                java.lang.reflect.Method method = GenericCallerProbe.class.getDeclaredMethod(
                    "choose", Number.class, Number.class, boolean.class);
                java.lang.reflect.TypeVariable<?> variable = method.getTypeParameters()[0];
                System.out.print(GenericCallerProbe.call() + ":" + GenericCallerProbe.choose(3, 4, false));
                System.out.print(":" + variable.getGenericDeclaration().equals(method));
                System.out.print(":" + method.getGenericReturnType().equals(variable));
                System.out.print(":" + method.getGenericParameterTypes()[0].equals(variable));
                System.out.print(":" + method.getGenericParameterTypes()[1].equals(variable));
                System.out.print(":" + java.util.Arrays.toString(variable.getBounds()));
            }
        }
    "#;
    let expected = "1:4:true:true:true:true:[class java.lang.Number]";
    assert_eq!(
        java_output("GenericCallerProbe", original, runner),
        expected
    );
    assert_eq!(
        java_output("GenericCallerProbe", &report.text, runner),
        expected
    );
}

#[test]
fn raw_direct_return_uses_the_actual_method_bound_and_parameter_positions() {
    let original = r#"
        public class GenericBoundRawCallerProbe {
            private static Exception make(String message) { return new Exception(message); }
            public static <E extends Exception> E select(E left, E right, int marker, boolean first) {
                return first ? left : right;
            }
            public static Exception call() { return select(make("left"), make("right"), 5, false); }
        }
    "#;
    let report = compiled_source("GenericBoundRawCallerProbe", original);
    assert!(
        report
            .text
            .contains("<E extends java.lang.Exception> E select("),
        "{}",
        report.text
    );
    let runner = r#"
        public class GenericReflectionRunner {
            public static void main(String[] args) throws Exception {
                java.lang.reflect.Method method = GenericBoundRawCallerProbe.class.getDeclaredMethod(
                    "select", Exception.class, Exception.class, int.class, boolean.class);
                java.lang.reflect.TypeVariable<?> variable = method.getTypeParameters()[0];
                System.out.print(GenericBoundRawCallerProbe.call().getMessage());
                System.out.print(":" + variable.getGenericDeclaration().equals(method));
                System.out.print(":" + method.getGenericReturnType().equals(variable));
                System.out.print(":" + method.getGenericParameterTypes()[0].equals(variable));
                System.out.print(":" + method.getGenericParameterTypes()[1].equals(variable));
                System.out.print(":" + java.util.Arrays.toString(variable.getBounds()));
            }
        }
    "#;
    let expected = "right:true:true:true:true:[class java.lang.Exception]";
    assert_eq!(
        java_output("GenericBoundRawCallerProbe", original, runner),
        expected
    );
    assert_eq!(
        java_output("GenericBoundRawCallerProbe", &report.text, runner),
        expected
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
fn body_assignment_without_type_variable_proof_is_refused() {
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
}

#[test]
fn method_local_relay_in_a_nongeneric_class_preserves_distinct_method_binders() {
    let original = r#"
        public class GenericBodyCallProbe {
            public static <T extends Number> T pass(T value) { return helper(value); }
            private static <T extends Number> T helper(T value) { return value; }
        }
    "#;
    let call = compiled_source("GenericBodyCallProbe", original);
    let pass = call
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"pass")
        .unwrap();
    assert!(
        pass.text.contains("<T extends java.lang.Number> T pass(T"),
        "{}",
        pass.text
    );
    let runner = r#"
        public class GenericReflectionRunner {
            public static void main(String[] args) throws Exception {
                Integer value = Integer.valueOf(17);
                java.lang.reflect.Method pass = GenericBodyCallProbe.class.getDeclaredMethod("pass", Number.class);
                java.lang.reflect.Method helper = GenericBodyCallProbe.class.getDeclaredMethod("helper", Number.class);
                java.lang.reflect.TypeVariable<?> callerVariable = pass.getTypeParameters()[0];
                java.lang.reflect.TypeVariable<?> calleeVariable = helper.getTypeParameters()[0];
                System.out.print((GenericBodyCallProbe.<Integer>pass(value) == value) + ":"
                    + GenericBodyCallProbe.class.getTypeParameters().length);
                System.out.print(":" + callerVariable.getGenericDeclaration().equals(pass));
                System.out.print(":" + calleeVariable.getGenericDeclaration().equals(helper));
                System.out.print(":" + !callerVariable.equals(calleeVariable));
                System.out.print(":" + (pass.getGenericReturnType().equals(callerVariable)
                    && pass.getGenericParameterTypes()[0].equals(callerVariable)));
                System.out.print(":" + (helper.getGenericReturnType().equals(calleeVariable)
                    && helper.getGenericParameterTypes()[0].equals(calleeVariable)));
                System.out.print(":" + (callerVariable.getBounds()[0] == Number.class
                    && calleeVariable.getBounds()[0] == Number.class));
            }
        }
    "#;
    let expected = "true:0:true:true:true:true:true:true";
    assert_eq!(
        java_output("GenericBodyCallProbe", original, runner),
        expected
    );
    assert_eq!(
        java_output("GenericBodyCallProbe", &call.text, runner),
        expected
    );
}
