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
const DT31_GROUPED_ENUM_SWITCH_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-27/dt31-grouped-enum-switch-impl/grouped-input.jar"
);
const DT31_GROUPED_ENUM_SWITCH_NEGATIVE_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-27/dt31-grouped-enum-switch-impl/negative-input.jar"
);
const ANONYMOUS_INTERFACE_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-interface-basic/AnonymousInterfaceBasic.class"
);
const ANONYMOUS_INTERFACE_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-interface-basic/AnonymousInterfaceBasic$1.class"
);
const ANONYMOUS_INTERFACE_API: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-interface-basic/I.class");
const ANONYMOUS_INNER_THIS_ROOT: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-inner-this/Inner.class");
const ANONYMOUS_INNER_THIS_CHILD: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-inner-this/Inner$1.class");
const ANONYMOUS_SUPER_DIRECT_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-super-direct/AnonymousSuperDirect.class"
);
const ANONYMOUS_SUPER_DIRECT_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-super-direct/AnonymousSuperDirect$1.class"
);
const ANONYMOUS_SUPER_DIRECT_BASE: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-super-direct/Base.class");
const ANONYMOUS_SUPER_CROSS_OWNER: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-cross-class-use/Owner.class");
const ANONYMOUS_SUPER_CROSS_CHILD: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-cross-class-use/Owner$1.class");
const ANONYMOUS_SUPER_CROSS_BASE: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-cross-class-use/Base.class");
const ANONYMOUS_SUPER_CROSS_OTHER: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-cross-class-use/Other.class");
const DT08_CAPTURE_SOURCE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-27/dt08-local-capture/input/p/Capture.java"
);
const DT08_RUNNER_SOURCE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-27/dt08-local-capture/input/p/Runner.java"
);
const EM03_SOURCE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-27/em03-method-signatures/input/em03/Signatures.java"
);
const SAME_PACKAGE_PARENT_FIELD_SOURCE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-27/dt29-same-package-parent-field-writes/fixtures/SamePackageParentFamily.java"
);
const TYPED_FUNCTIONAL_REFS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-28/dt27-typed-functional/TypedRefs.class"
);

#[test]
fn typed_functional_references_publish_one_complete_header_body_and_source() {
    let snapshot = open(TYPED_FUNCTIONAL_REFS.to_vec());
    let request = request(
        &snapshot,
        ClassRef::Name {
            class: ClassNameQuery::internal("dt27/TypedRefs"),
        },
        EnvironmentPolicy::SingleClass,
    );
    let report = performed(
        Engine::new()
            .class_source(slice::from_ref(&snapshot), &request, &mut budget())
            .expect("the frozen class source is recoverable"),
    );
    for (name, reference, bci, cp) in [
        (b"parse".as_slice(), "Integer::parseInt", 0, 13),
        (b"bound".as_slice(), "this::length", 1, 17),
        (b"supplier".as_slice(), "this::label", 1, 20),
    ] {
        let method = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == name)
            .expect("the physical method remains present");
        assert!(method.text.contains(reference), "{}", method.text);
        assert!(method.text.contains("generic Signature"), "{}", method.text);
        let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
            panic!("the method has no same-run recovery");
        };
        assert!(
            report
                .source_map
                .of_bci(bci)
                .iter()
                .any(|segment| { segment.origin().primary().cp() == Some(cp) })
        );
    }

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(budget().limits().clone(), token);
    let stopped = Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut cancelled)
        .expect("cancellation is an execution state");
    match stopped {
        OperationOutcome::Incomplete(selection) => {
            assert!(matches!(
                selection.execution,
                ExecutionReport::Cancelled { .. }
            ));
        }
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                report.execution,
                ExecutionReport::Cancelled { .. }
            ));
            assert!(!report.text.contains("Integer::parseInt"));
        }
        OperationOutcome::Ambiguous(_) => panic!("the frozen class is unambiguous"),
    }
}

#[test]
fn same_package_parent_field_writes_keep_owner_source_and_stop_boundaries() {
    let scratch = BridgeProjectionScratch::new();
    let directory = scratch.child("same-package-parent-fields");
    let source = directory.join("SamePackageParentFamily.java");
    fs::write(&source, SAME_PACKAGE_PARENT_FIELD_SOURCE).expect("write the frozen Java fixture");
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-Xlint:-options", "-d"])
        .arg(&directory)
        .arg(&source)
        .output()
        .expect("the installed JDK provides javac");
    assert!(
        compile.status.success(),
        "javac rejected the frozen fixture:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let family = fs::read(directory.join("dt29/SamePackageParentFamily.class"))
        .expect("the outer family class was compiled");
    let parent = fs::read(directory.join("dt29/SamePackageParentFamily$A.class"))
        .expect("the parent class was compiled");
    let child = fs::read(directory.join("dt29/SamePackageParentFamily$B.class"))
        .expect("the child class was compiled");
    let snapshot = open(zip_of(&[
        (b"dt29/SamePackageParentFamily.class", &family),
        (b"dt29/SamePackageParentFamily$A.class", &parent),
        (b"dt29/SamePackageParentFamily$B.class", &child),
    ]));
    let engine = Engine::new();
    let class_request = request(
        &snapshot,
        ClassRef::Name {
            class: ClassNameQuery::internal("dt29/SamePackageParentFamily$B"),
        },
        EnvironmentPolicy::PlainJar,
    );
    let report = performed(
        engine
            .class_source(slice::from_ref(&snapshot), &class_request, &mut budget())
            .expect("the complete parent field class source is available"),
    );
    assert!(
        report
            .text
            .contains("((dt29.SamePackageParentFamily$A) this).protectedField = arg1;")
    );
    assert!(
        report
            .text
            .contains("((dt29.SamePackageParentFamily$A) this).packagePrivateField = arg1;")
    );
    assert!(!report.text.contains("@bytecode 7") && !report.text.contains("@bytecode 12"));
    let setter = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"set")
        .expect("the physical setter remains reported");
    let setter_report = match &setter.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("the setter remains a recovered body: {other:?}"),
    };
    assert!(!setter_report.source_map.of_bci(7).is_empty());
    assert!(!setter_report.source_map.of_bci(12).is_empty());

    let mut constrained =
        task_budget(&[BudgetOverride::new("method_bodies", 1).expect("a legal body limit")])
            .expect("the request budget is valid");
    let stopped = performed(
        engine
            .class_source(slice::from_ref(&snapshot), &class_request, &mut constrained)
            .expect("the body limit is reported in the class view"),
    );
    assert!(matches!(
        stopped.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::MethodBodies
            },
            ..
        }
    ));

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(budget().limits().clone(), token);
    let outcome = engine
        .class_source(slice::from_ref(&snapshot), &class_request, &mut cancelled)
        .expect("cancellation remains a stated class-source outcome");
    match outcome {
        OperationOutcome::Incomplete(selection) => {
            assert!(matches!(
                selection.execution,
                ExecutionReport::Cancelled { .. }
            ));
        }
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                report.execution,
                ExecutionReport::Cancelled { .. }
            ));
            assert!(report.text.is_empty() || !report.text.contains("= arg1;"));
        }
        OperationOutcome::Ambiguous(_) => panic!("the fixture class name is unique"),
    }
}

#[test]
fn direct_override_projection_requires_selected_complete_ordinary_parent() {
    let scratch = BridgeProjectionScratch::new();
    let root = scratch.child("direct-override");
    let base = root.join("Base.java");
    let child = root.join("Child.java");
    fs::write(&base, "package p; public class Base { void f() {} }").unwrap();
    fs::write(
        &child,
        "package p; public class Child extends Base { public void f() {} }",
    )
    .unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&root)
        .args([&base, &child])
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let base_bytes = fs::read(root.join("p/Base.class")).unwrap();
    let child_bytes = fs::read(root.join("p/Child.class")).unwrap();
    let jar = open(zip_of(&[
        (b"p/Base.class", &base_bytes),
        (b"p/Child.class", &child_bytes),
    ]));
    let report = class_source_of(&jar, "p/Child", EnvironmentPolicy::PlainJar);
    assert_eq!(
        report.text.matches("@Override").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(report.direct_override_proofs.len(), 1);
    let proof = &report.direct_override_proofs[0];
    assert_eq!(proof.child.name.0, b"f");
    assert_eq!(proof.parent.name.0, b"f");
    assert_eq!(proof.parent.descriptor.0, b"()V");
    let physical = report
        .methods
        .iter()
        .find(|method| method.item.identity == proof.child)
        .unwrap();
    assert!(!physical.text.contains("@Override"));
    assert!(physical.annotations.uses.is_empty());
    assert!(report.text.contains(&physical.text));
    let mut bounded =
        task_budget(&[BudgetOverride::new("output_bytes", report.usage.output_bytes - 1).unwrap()])
            .unwrap();
    let stopped = performed(
        Engine::new()
            .class_source(
                slice::from_ref(&jar),
                &request(
                    &jar,
                    ClassRef::Name {
                        class: ClassNameQuery::internal("p/Child"),
                    },
                    EnvironmentPolicy::PlainJar,
                ),
                &mut bounded,
            )
            .unwrap(),
    );
    assert!(!stopped.text.contains("@Override"));
    assert!(stopped.direct_override_proofs.is_empty());
    assert!(!matches!(
        stopped.execution,
        ExecutionReport::Complete { .. }
    ));

    for entries in [
        vec![(b"p/Child.class".as_slice(), child_bytes.as_slice())],
        vec![
            (b"p/Base.class".as_slice(), base_bytes.as_slice()),
            (b"p/Base.class".as_slice(), base_bytes.as_slice()),
            (b"p/Child.class".as_slice(), child_bytes.as_slice()),
        ],
    ] {
        let snapshot = open(zip_of(&entries));
        let negative = class_source_of(&snapshot, "p/Child", EnvironmentPolicy::PlainJar);
        assert!(!negative.text.contains("@Override"), "{}", negative.text);
        assert!(negative.direct_override_proofs.is_empty());
    }

    for (variant, source) in [
        (
            "private",
            "package p; public class Base { private void f() {} }",
        ),
        (
            "static",
            "package p; public class Base { static void f() {} }",
        ),
        (
            "final",
            "package p; public class Base { final void f() {} }",
        ),
        (
            "final-class",
            "package p; public final class Base { void f() {} }",
        ),
        (
            "signature",
            "package p; public class Base<T> { void f() {} }",
        ),
    ] {
        let directory = scratch.child(variant);
        let path = directory.join("Base.java");
        fs::write(&path, source).unwrap();
        let compiled = Command::new("javac")
            .args(["--release", "8", "-g:none", "-d"])
            .arg(&directory)
            .arg(&path)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let changed_base = fs::read(directory.join("p/Base.class")).unwrap();
        let snapshot = open(zip_of(&[
            (b"p/Base.class", &changed_base),
            (b"p/Child.class", &child_bytes),
        ]));
        let negative = class_source_of(&snapshot, "p/Child", EnvironmentPolicy::PlainJar);
        assert!(
            !negative.text.contains("@Override"),
            "{variant}: {}",
            negative.text
        );
        assert!(negative.direct_override_proofs.is_empty());
    }
    for (variant, source) in [
        (
            "child-static",
            "package p; public class Child extends Base { public static void f() {} }",
        ),
        (
            "wrong-descriptor",
            "package p; public class Child extends Base { public void f(int value) {} }",
        ),
        (
            "child-signature",
            "package p; public class Child<T> extends Base { public void f() {} }",
        ),
    ] {
        let directory = scratch.child(variant);
        let variant_base = directory.join("Base.java");
        let variant_child = directory.join("Child.java");
        fs::write(&variant_base, "package p; public class Base {}").unwrap();
        fs::write(&variant_child, source).unwrap();
        let compiled = Command::new("javac")
            .args(["--release", "8", "-g:none", "-d"])
            .arg(&directory)
            .args([&variant_base, &variant_child])
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let changed_child = fs::read(directory.join("p/Child.class")).unwrap();
        let snapshot = open(zip_of(&[
            (b"p/Base.class", &base_bytes),
            (b"p/Child.class", &changed_child),
        ]));
        let negative = class_source_of(&snapshot, "p/Child", EnvironmentPolicy::PlainJar);
        assert!(
            !negative.text.contains("@Override"),
            "{variant}: {}",
            negative.text
        );
        assert!(negative.direct_override_proofs.is_empty());
    }
}

#[test]
fn method_parameters_name_body_and_declaration_only_without_lvt() {
    let scratch = BridgeProjectionScratch::new();
    for (variant, options, expected) in [
        (
            "parameters",
            &["-g:none", "-parameters"][..],
            "named(java.lang.String paramStr, final int number)",
        ),
        (
            "no-parameters",
            &["-g:none"][..],
            "named(java.lang.String arg1, int arg2)",
        ),
        (
            "lvt",
            &["-g", "-parameters"][..],
            "named(java.lang.String paramStr, int number)",
        ),
    ] {
        let directory = scratch.child(variant);
        let source = directory.join("Signatures.java");
        fs::write(&source, EM03_SOURCE).unwrap();
        let compiled = Command::new("javac")
            .args(["--release", "8"])
            .args(options)
            .args(["-d"])
            .arg(&directory)
            .arg(&source)
            .output()
            .expect("javac is available for the Java 8 parameter regression");
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let bytes = fs::read(directory.join("em03/Signatures.class")).unwrap();
        let snapshot = open(bytes);
        let report = class_source_of(&snapshot, "em03/Signatures", EnvironmentPolicy::SingleClass);
        assert!(report.text.contains(expected), "{variant}: {}", report.text);
        let body_name = if variant == "no-parameters" {
            "arg1"
        } else {
            "paramStr"
        };
        assert!(
            report.text.contains(&format!("return {body_name};")),
            "{variant}: {}",
            report.text
        );
        if variant == "parameters" {
            let facts = class_facts(
                &fs::read(directory.join("em03/Signatures.class")).unwrap(),
                &mut budget(),
            )
            .unwrap();
            let named = facts
                .methods
                .iter()
                .find(|method| method.name.raw().0 == b"named")
                .unwrap();
            let shell = named
                .attributes
                .iter()
                .find(|shell| shell.name.raw().0 == b"MethodParameters")
                .unwrap();
            let start = usize::try_from(shell.content_span.start).unwrap();
            let mut missing_name = fs::read(directory.join("em03/Signatures.class")).unwrap();
            missing_name[start + 1..start + 3].copy_from_slice(&0_u16.to_be_bytes());
            let fallback = class_source_of(
                &open(missing_name),
                "em03/Signatures",
                EnvironmentPolicy::SingleClass,
            );
            assert!(
                fallback
                    .text
                    .contains("named(java.lang.String arg1, int arg2)"),
                "{}",
                fallback.text
            );
            assert!(fallback.text.contains("return arg1;"), "{}", fallback.text);

            let mut duplicate_name = fs::read(directory.join("em03/Signatures.class")).unwrap();
            let first_name = duplicate_name[start + 1..start + 3].to_vec();
            duplicate_name[start + 5..start + 7].copy_from_slice(&first_name);
            let fallback = class_source_of(
                &open(duplicate_name),
                "em03/Signatures",
                EnvironmentPolicy::SingleClass,
            );
            assert!(
                fallback
                    .text
                    .contains("named(java.lang.String arg1, int arg2)"),
                "{}",
                fallback.text
            );
            assert!(fallback.text.contains("return arg1;"), "{}", fallback.text);

            let mut wrong_count = fs::read(directory.join("em03/Signatures.class")).unwrap();
            wrong_count[start] = 1;
            wrong_count.drain(start + 5..start + 9);
            let length = usize::try_from(shell.span.start).unwrap() + 2;
            wrong_count[length..length + 4].copy_from_slice(&5_u32.to_be_bytes());
            let fallback = class_source_of(
                &open(wrong_count),
                "em03/Signatures",
                EnvironmentPolicy::SingleClass,
            );
            assert!(
                fallback
                    .text
                    .contains("named(java.lang.String arg1, int arg2)"),
                "{}",
                fallback.text
            );
            assert!(fallback.text.contains("return arg1;"), "{}", fallback.text);

            let mut bad_flags = fs::read(directory.join("em03/Signatures.class")).unwrap();
            bad_flags[start + 3..start + 5].copy_from_slice(&1_u16.to_be_bytes());
            let refused = class_source_of(
                &open(bad_flags),
                "em03/Signatures",
                EnvironmentPolicy::SingleClass,
            );
            let named = refused
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"named")
                .unwrap();
            assert!(matches!(named.outcome, ClassSourceOutcome::Refused { .. }));
            assert!(!refused.text.contains("named(java.lang.String paramStr"));
        }
    }
}

#[test]
fn method_parameters_follow_wide_slots_and_refuse_name_collisions() {
    let scratch = BridgeProjectionScratch::new();
    for (variant, source, expected, returned) in [
        (
            "wide",
            "package em03; public class Wide { public long named(long first, final double second) { return first; } }",
            "named(long first, final double second)",
            "return first;",
        ),
        (
            "collision",
            "package em03; public class Wide { public long named(long wide, String arg2) { return wide; } }",
            "named(long arg1, java.lang.String arg3)",
            "return arg1;",
        ),
    ] {
        let directory = scratch.child(variant);
        let path = directory.join("Wide.java");
        fs::write(&path, source).unwrap();
        let compiled = Command::new("javac")
            .args(["--release", "8", "-g:none", "-parameters", "-d"])
            .arg(&directory)
            .arg(&path)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let snapshot = open(fs::read(directory.join("em03/Wide.class")).unwrap());
        let report = class_source_of(&snapshot, "em03/Wide", EnvironmentPolicy::SingleClass);
        assert!(report.text.contains(expected), "{variant}: {}", report.text);
        assert!(report.text.contains(returned), "{variant}: {}", report.text);
    }
}

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
fn enum_switch_projection_groups_distinct_tables_atomically() {
    let snapshot = open(DT31_GROUPED_ENUM_SWITCH_JAR.to_vec());
    let engine = Engine::new();
    let request = request(
        &snapshot,
        ClassRef::Name {
            class: ClassNameQuery::internal("grouped/Subject"),
        },
        EnvironmentPolicy::PlainJar,
    );
    let all = performed(
        engine
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut budget(),
            )
            .expect("the grouped enum class-source request is legal"),
    );
    let essential = performed(
        engine
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::essential(),
                &mut budget(),
            )
            .expect("the essential grouped enum class-source request is legal"),
    );
    assert_eq!(all.text, essential.text);
    assert_eq!(
        all.enum_switch_proofs.len(),
        2,
        "{:#?}",
        all.enum_switch_proofs
    );
    assert!(
        all.enum_switch_proofs.iter().all(|proof| proof.projected),
        "{:#?}",
        all.enum_switch_proofs
    );
    assert!(all.text.contains("switch (arg0)"));
    assert!(all.text.contains("switch (arg1)"));
    assert!(all.text.contains("case ONE:"));
    assert!(all.text.contains("case CAT:"));
    assert_eq!(
        all.text
            .matches("jarde: enum switch projected at BCI")
            .count(),
        2
    );

    let method = all
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"select")
        .expect("the physical selector method is present");
    let standalone = Engine::new()
        .recover_method(
            slice::from_ref(&snapshot),
            &MethodAnalysisRequest {
                environment: request
                    .environment
                    .build(slice::from_ref(&snapshot))
                    .expect("the physical class-source environment builds"),
                method: method.item.identity.clone(),
                stages: AnalysisStage::ALL.to_vec(),
            },
            &mut budget(),
        )
        .expect("independent method recovery remains available");
    assert!(
        standalone
            .recovery()
            .text
            .contains("$SwitchMap$grouped$Count")
    );
    assert!(standalone.recovery().text.contains("case 1:"));
    assert!(!standalone.recovery().text.contains("case ONE:"));

    let cap = all.usage.output_bytes.saturating_sub(1);
    let mut constrained =
        task_budget(&[BudgetOverride::new("output_bytes", cap).expect("valid output cap")])
            .expect("the constrained output budget is valid");
    let stopped = performed(
        engine
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut constrained,
            )
            .expect("an output stop retains its class-source report"),
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
    assert!(
        stopped
            .enum_switch_proofs
            .iter()
            .all(|proof| !proof.projected)
    );
    assert!(
        stopped
            .text
            .contains("$SwitchMap$grouped$Count[arg0.ordinal()]")
    );
    assert!(
        stopped
            .text
            .contains("$SwitchMap$grouped$Animal[arg1.ordinal()]")
    );
}

#[test]
fn enum_switch_projection_withholds_sibling_when_one_shared_table_proof_fails() {
    let snapshot = open(DT31_GROUPED_ENUM_SWITCH_NEGATIVE_JAR.to_vec());
    let report = class_source_of(&snapshot, "grouped/Subject", EnvironmentPolicy::PlainJar);
    assert_eq!(
        report.enum_switch_proofs.len(),
        2,
        "{:#?}",
        report.enum_switch_proofs
    );
    assert!(
        report
            .enum_switch_proofs
            .iter()
            .all(|proof| !proof.projected)
    );
    assert!(
        report
            .text
            .contains("$SwitchMap$grouped$Count[arg0.ordinal()]")
    );
    assert!(
        report
            .text
            .contains("$SwitchMap$grouped$Animal[arg1.ordinal()]")
    );
    let failed = report
        .enum_switch_proofs
        .iter()
        .find(|proof| proof.table_name.contains("Animal"))
        .expect("the corrupted Animal map has a proof record");
    assert!(
        failed
            .refusal
            .as_deref()
            .is_some_and(|reason| reason.contains("selected table `$SwitchMap$grouped$Animal`")),
        "{failed:#?}"
    );
    let sibling = report
        .enum_switch_proofs
        .iter()
        .find(|proof| proof.table_name.contains("Count"))
        .expect("the Count sibling has a proof record");
    assert!(
        sibling
            .refusal
            .as_deref()
            .is_some_and(|reason| reason.contains("Animal") && reason.contains("withheld")),
        "{sibling:#?}"
    );
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

fn patch_one_byte_utf8(bytes: &[u8], original: u8, replacement: u8) -> Vec<u8> {
    let mut patched = bytes.to_vec();
    let count = test_u16(bytes, 8);
    let mut cursor = 10;
    let mut index = 1;
    let mut found = 0;
    while index < count {
        let tag = bytes[cursor];
        cursor += 1;
        match tag {
            1 => {
                let length = test_u16(bytes, cursor);
                if length == 1 && bytes[cursor + 2] == original {
                    patched[cursor + 2] = replacement;
                    found += 1;
                }
                cursor += 2 + length;
                index += 1;
            }
            3 | 4 | 9 | 10 | 11 | 12 | 17 | 18 => {
                cursor += if matches!(tag, 3 | 4) { 4 } else { 4 };
                index += 1;
            }
            5 | 6 => {
                cursor += 8;
                index += 2;
            }
            7 | 8 | 16 | 19 | 20 => {
                cursor += 2;
                index += 1;
            }
            15 => {
                cursor += 3;
                index += 1;
            }
            other => panic!("unexpected constant-pool tag {other}"),
        }
    }
    assert_eq!(
        found, 1,
        "the one-byte descriptor is unique in this fixture"
    );
    patched
}

fn test_field_reference_index(bytes: &[u8], owner: &[u8], name: &[u8], descriptor: &[u8]) -> u16 {
    let count = test_u16(bytes, 8);
    let mut utf8 = vec![Vec::new(); count];
    let mut classes = vec![None; count];
    let mut name_and_types = vec![None; count];
    let mut fields = vec![None; count];
    let mut cursor = 10;
    let mut index = 1;
    while index < count {
        let tag = bytes[cursor];
        cursor += 1;
        match tag {
            1 => {
                let length = test_u16(bytes, cursor);
                utf8[index] = bytes[cursor + 2..cursor + 2 + length].to_vec();
                cursor += 2 + length;
                index += 1;
            }
            7 => {
                classes[index] = Some(test_u16(bytes, cursor));
                cursor += 2;
                index += 1;
            }
            9 => {
                fields[index] = Some((test_u16(bytes, cursor), test_u16(bytes, cursor + 2)));
                cursor += 4;
                index += 1;
            }
            12 => {
                name_and_types[index] =
                    Some((test_u16(bytes, cursor), test_u16(bytes, cursor + 2)));
                cursor += 4;
                index += 1;
            }
            3 | 4 => {
                cursor += 4;
                index += 1;
            }
            5 | 6 => {
                cursor += 8;
                index += 2;
            }
            8 | 16 | 19 | 20 => {
                cursor += 2;
                index += 1;
            }
            10 | 11 | 17 | 18 => {
                cursor += 4;
                index += 1;
            }
            15 => {
                cursor += 3;
                index += 1;
            }
            other => panic!("unexpected constant-pool tag {other}"),
        }
    }
    let class_index = (1..count)
        .find(|candidate| classes[*candidate].is_some_and(|name| utf8[name] == owner))
        .expect("the field owner class exists");
    let name_and_type = (1..count)
        .find(|candidate| {
            name_and_types[*candidate]
                .is_some_and(|(n, d)| utf8[n] == name && utf8[d] == descriptor)
        })
        .expect("the field name and descriptor exist");
    (1..count)
        .find(|candidate| fields[*candidate] == Some((class_index, name_and_type)))
        .and_then(|candidate| u16::try_from(candidate).ok())
        .expect("the exact Fieldref exists")
}

fn test_class_index(bytes: &[u8], owner: &[u8]) -> u16 {
    let count = test_u16(bytes, 8);
    let mut utf8 = vec![Vec::new(); count];
    let mut classes = vec![None; count];
    let mut cursor = 10;
    let mut index = 1;
    while index < count {
        let tag = bytes[cursor];
        cursor += 1;
        match tag {
            1 => {
                let length = test_u16(bytes, cursor);
                utf8[index] = bytes[cursor + 2..cursor + 2 + length].to_vec();
                cursor += 2 + length;
                index += 1;
            }
            7 => {
                classes[index] = Some(test_u16(bytes, cursor));
                cursor += 2;
                index += 1;
            }
            3 | 4 => {
                cursor += 4;
                index += 1;
            }
            5 | 6 => {
                cursor += 8;
                index += 2;
            }
            8 | 16 | 19 | 20 => {
                cursor += 2;
                index += 1;
            }
            9 | 10 | 11 | 12 | 17 | 18 => {
                cursor += 4;
                index += 1;
            }
            15 => {
                cursor += 3;
                index += 1;
            }
            other => panic!("unexpected constant-pool tag {other}"),
        }
    }
    (1..count)
        .find(|candidate| classes[*candidate].is_some_and(|name| utf8[name] == owner))
        .and_then(|candidate| u16::try_from(candidate).ok())
        .expect("the replacement owner class exists")
}

fn test_cp_entry_offset(bytes: &[u8], wanted: u16) -> usize {
    let count = test_u16(bytes, 8);
    let mut cursor = 10;
    let mut index = 1;
    while index < count {
        if index == usize::from(wanted) {
            return cursor;
        }
        let tag = bytes[cursor];
        cursor += match tag {
            1 => 3 + test_u16(bytes, cursor + 1),
            3 | 4 => 5,
            5 | 6 => 9,
            7 | 8 | 16 | 19 | 20 => 3,
            9 | 10 | 11 | 12 | 17 | 18 => 5,
            15 => 4,
            other => panic!("unexpected constant-pool tag {other}"),
        };
        index += if matches!(tag, 5 | 6) { 2 } else { 1 };
    }
    panic!("constant-pool index {wanted} is absent");
}

fn patch_capture_field_owner(bytes: &[u8], owner: &[u8]) -> Vec<u8> {
    let fieldref = test_field_reference_index(bytes, b"p/Capture$1", b"val$d", b"D");
    let class_index = test_class_index(bytes, owner);
    let mut patched = bytes.to_vec();
    let offset = test_cp_entry_offset(bytes, fieldref);
    test_put_u16(&mut patched, offset + 1, usize::from(class_index));
    patched
}

fn patch_capture_constructor_extra_write(bytes: &[u8]) -> Vec<u8> {
    let fieldref = test_field_reference_index(bytes, b"p/Capture$1", b"val$d", b"D");
    let method = test_method_headers(bytes)
        .into_iter()
        .find(|method| method.name == b"<init>" && method.descriptor == b"(D)V")
        .expect("the capture constructor exists");
    let code = method
        .attributes
        .iter()
        .find(|attribute| attribute.name == b"Code")
        .expect("the capture constructor has Code");
    let old_length = test_u32(bytes, code.data_offset + 4);
    let return_at = code.data_offset + 8 + old_length - 1;
    let fieldref = fieldref.to_be_bytes();
    let extra_write = [0x2a, 0x27, 0xb5, fieldref[0], fieldref[1]];
    let mut patched = bytes.to_vec();
    patched.splice(return_at..return_at, extra_write);
    test_put_u32(&mut patched, code.data_offset + 4, old_length + 5);
    test_put_u32(&mut patched, code.length_offset, code.length + 5);
    patched
}

fn patch_capture_constructor_exception_range(bytes: &[u8]) -> Vec<u8> {
    let method = test_method_headers(bytes)
        .into_iter()
        .find(|method| method.name == b"<init>" && method.descriptor == b"(D)V")
        .expect("the capture constructor exists");
    let code = method
        .attributes
        .iter()
        .find(|attribute| attribute.name == b"Code")
        .expect("the capture constructor has Code");
    let code_length = test_u32(bytes, code.data_offset + 4);
    let exception_count = code.data_offset + 8 + code_length;
    assert_eq!(test_u16(bytes, exception_count), 0);
    let mut patched = bytes.to_vec();
    test_put_u16(&mut patched, exception_count, 1);
    // A valid catch-all range spanning the capture write and ending at the return BCI.
    patched.splice(
        exception_count + 2..exception_count + 2,
        [
            0, 0, 0, 9, // start_pc=0, end_pc=9
            0, 9, 0, 0, // handler_pc=9, catch_type=0
        ],
    );
    test_put_u32(&mut patched, code.length_offset, code.length + 8);
    patched
}

fn patch_capture_root_parameter_slot_reuse(bytes: &[u8]) -> Vec<u8> {
    let method = test_method_headers(bytes)
        .into_iter()
        .find(|method| method.name == b"create" && method.descriptor == b"(D)Ljava/lang/Runnable;")
        .expect("the capture factory exists");
    let code = method
        .attributes
        .iter()
        .find(|attribute| attribute.name == b"Code")
        .expect("the capture factory has Code");
    let insertion = code.data_offset + 8 + 5;
    let mut patched = bytes.to_vec();
    // Reload the same entry slot and consume it before invokespecial; this makes the captured
    // parameter slot have an additional use while preserving the allocation's operand stack.
    patched.splice(insertion..insertion, [0x26, 0x58]); // dload_0; pop2
    test_put_u16(&mut patched, code.data_offset, 6);
    test_put_u32(
        &mut patched,
        code.data_offset + 4,
        test_u32(bytes, code.data_offset + 4) + 2,
    );
    test_put_u32(&mut patched, code.length_offset, code.length + 2);
    patched
}

fn append_capture_field_method_handle(bytes: &[u8]) -> Vec<u8> {
    let fieldref = test_field_reference_index(bytes, b"p/Capture$1", b"val$d", b"D");
    let (pool_end, _) = test_pool(bytes);
    let count = test_u16(bytes, 8);
    let fieldref = fieldref.to_be_bytes();
    let method_handle = [15, 1, fieldref[0], fieldref[1]]; // REF_getField
    let mut patched = bytes.to_vec();
    test_put_u16(&mut patched, 8, count + 1);
    patched.splice(pool_end..pool_end, method_handle);
    patched
}

fn patch_capture_read_field_owner(bytes: &[u8], owner: &[u8]) -> Vec<u8> {
    let owner_index = test_class_index(bytes, owner);
    let name_and_type = test_name_and_type_index(bytes, b"val$d", b"D");
    let (pool_end, _) = test_pool(bytes);
    let count = test_u16(bytes, 8);
    let owner_index = owner_index.to_be_bytes();
    let name_and_type = name_and_type.to_be_bytes();
    let fieldref = [
        9,
        owner_index[0],
        owner_index[1],
        name_and_type[0],
        name_and_type[1],
    ];
    let mut patched = bytes.to_vec();
    test_put_u16(&mut patched, 8, count + 1);
    patched.splice(pool_end..pool_end, fieldref);
    let run = test_method_headers(&patched)
        .into_iter()
        .find(|method| method.name == b"run" && method.descriptor == b"()V")
        .expect("the capture run method exists");
    let code = run
        .attributes
        .iter()
        .find(|attribute| attribute.name == b"Code")
        .expect("the capture run method has Code");
    let length = test_u32(&patched, code.data_offset + 4);
    let code_start = code.data_offset + 8;
    let getfield_bci = (0..length)
        .find(|bci| patched[code_start + bci] == 0xb4)
        .expect("run contains a field read");
    test_put_u16(&mut patched, code_start + getfield_bci + 1, count);
    patched
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

fn test_class_attributes(bytes: &[u8]) -> Vec<TestClassAttribute> {
    let methods = test_method_headers(bytes);
    let mut cursor = methods
        .last()
        .map_or_else(|| test_pool(bytes).0 + 6, |method| method.end);
    let count = test_u16(bytes, cursor);
    cursor += 2;
    let (_, pool) = test_pool(bytes);
    let mut attributes = Vec::with_capacity(count);
    for _ in 0..count {
        let name_index = test_u16(bytes, cursor);
        let length_offset = cursor + 2;
        let length = test_u32(bytes, length_offset);
        attributes.push(TestClassAttribute {
            name: pool[name_index].clone(),
            length_offset,
            data_offset: cursor + 6,
            length,
        });
        cursor += 6 + length;
    }
    attributes
}

fn test_name_and_type_index(bytes: &[u8], name: &[u8], descriptor: &[u8]) -> u16 {
    let count = test_u16(bytes, 8);
    let mut cursor = 10;
    let mut utf8 = vec![Vec::new(); count];
    let mut candidates = Vec::new();
    let mut index = 1;
    while index < count {
        let tag = bytes[cursor];
        match tag {
            1 => {
                let length = test_u16(bytes, cursor + 1);
                utf8[index] = bytes[cursor + 3..cursor + 3 + length].to_vec();
                cursor += 3 + length;
                index += 1;
            }
            12 => {
                candidates.push((
                    index,
                    test_u16(bytes, cursor + 1),
                    test_u16(bytes, cursor + 3),
                ));
                cursor += 5;
                index += 1;
            }
            3 | 4 | 9 | 10 | 11 | 17 | 18 => {
                cursor += 5;
                index += 1;
            }
            5 | 6 => {
                cursor += 9;
                index += 2;
            }
            7 | 8 | 16 | 19 | 20 => {
                cursor += 3;
                index += 1;
            }
            15 => {
                cursor += 4;
                index += 1;
            }
            other => panic!("unexpected constant-pool tag {other}"),
        }
    }
    u16::try_from(
        candidates
            .into_iter()
            .find_map(|(candidate, name_index, descriptor_index)| {
                (utf8[usize::from(name_index)] == name
                    && utf8[usize::from(descriptor_index)] == descriptor)
                    .then_some(candidate)
            })
            .expect("the name-and-type entry exists in the pool"),
    )
    .expect("the name-and-type index fits u16")
}

fn patch_enclosing_method(bytes: &[u8], name: &[u8], descriptor: &[u8]) -> Vec<u8> {
    let attribute = test_class_attributes(bytes)
        .into_iter()
        .find(|attribute| attribute.name == b"EnclosingMethod")
        .expect("the anonymous child has EnclosingMethod");
    assert_eq!(attribute.length, 4);
    let name_and_type = test_name_and_type_index(bytes, name, descriptor);
    let mut patched = bytes.to_vec();
    test_put_u16(
        &mut patched,
        attribute.data_offset + 2,
        usize::from(name_and_type),
    );
    patched
}

fn test_method_reference(bytes: &[u8], owner: &[u8], name: &[u8], descriptor: &[u8]) -> u16 {
    let count = test_u16(bytes, 8);
    let mut cursor = 10;
    let mut utf8 = vec![Vec::new(); count];
    let mut classes = vec![None; count];
    let mut name_and_types = vec![None; count];
    let mut references = vec![None; count];
    let mut index = 1;
    while index < count {
        let tag = bytes[cursor];
        match tag {
            1 => {
                let length = test_u16(bytes, cursor + 1);
                utf8[index] = bytes[cursor + 3..cursor + 3 + length].to_vec();
                cursor += 3 + length;
                index += 1;
            }
            7 => {
                classes[index] = Some(test_u16(bytes, cursor + 1) as u16);
                cursor += 3;
                index += 1;
            }
            9 | 10 | 11 => {
                references[index] = Some((
                    test_u16(bytes, cursor + 1) as u16,
                    test_u16(bytes, cursor + 3) as u16,
                ));
                cursor += 5;
                index += 1;
            }
            12 => {
                name_and_types[index] = Some((
                    test_u16(bytes, cursor + 1) as u16,
                    test_u16(bytes, cursor + 3) as u16,
                ));
                cursor += 5;
                index += 1;
            }
            3 | 4 | 17 | 18 => {
                cursor += 5;
                index += 1;
            }
            5 | 6 => {
                cursor += 9;
                index += 2;
            }
            8 | 16 | 19 | 20 => {
                cursor += 3;
                index += 1;
            }
            15 => {
                cursor += 4;
                index += 1;
            }
            other => panic!("unexpected constant-pool tag {other}"),
        }
    }
    (1..count)
        .find_map(|candidate| {
            let (class_index, name_and_type_index) = references[candidate]?;
            let class_name_index = classes[usize::from(class_index)]?;
            let (name_index, descriptor_index) = name_and_types[usize::from(name_and_type_index)]?;
            (utf8[usize::from(class_name_index)] == owner
                && utf8[usize::from(name_index)] == name
                && utf8[usize::from(descriptor_index)] == descriptor)
                .then_some(u16::try_from(candidate).expect("the constant-pool index fits u16"))
        })
        .expect("the requested method reference exists in the constant pool")
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

#[test]
fn proved_anonymous_interface_projects_from_both_physical_method_asts() {
    let bytes = zip_of(&[
        (b"AnonymousInterfaceBasic.class", ANONYMOUS_INTERFACE_ROOT),
        (
            b"AnonymousInterfaceBasic$1.class",
            ANONYMOUS_INTERFACE_CHILD,
        ),
        (b"I.class", ANONYMOUS_INTERFACE_API),
    ]);
    let snapshot = open(bytes);
    let engine = Engine::new();
    let root = performed(
        engine
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request(
                    &snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal("AnonymousInterfaceBasic"),
                    },
                    EnvironmentPolicy::PlainJar,
                ),
                &RecoveryEvidenceRequest::all(),
                &mut budget(),
            )
            .expect("the root class source is available"),
    );
    assert!(
        root.text.contains("new I() {"),
        "text={} family={:#?} diagnostics={:#?}",
        root.text,
        root.member_family,
        root.diagnostics
    );
    assert!(root.text.contains("public int value()"), "{}", root.text);
    assert!(root.text.contains("return 7;"), "{}", root.text);
    assert!(
        root.text.contains("\n            public int value()"),
        "the anonymous member is indented inside the allocation: {}",
        root.text
    );
    assert!(
        root.text.contains("\n                return 7;"),
        "the child statement is indented inside its method: {}",
        root.text
    );
    assert!(!root.text.contains("new AnonymousInterfaceBasic$1()"));
    let essential_root = performed(
        engine
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request(
                    &snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal("AnonymousInterfaceBasic"),
                    },
                    EnvironmentPolicy::PlainJar,
                ),
                &RecoveryEvidenceRequest::essential(),
                &mut budget(),
            )
            .expect("the essential-evidence root class source is available"),
    );
    assert_eq!(essential_root.text, root.text);
    let root_method = root
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"make")
        .expect("the direct-return method remains physically reported");
    assert!(matches!(
        &root_method.outcome,
        ClassSourceOutcome::Recovered { report, .. }
            if report.source_map.segments().iter().any(|segment| {
                segment.origin().primary().method() == Some(&root_method.item.identity)
            })
    ));

    let child = performed(
        engine
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request(
                    &snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal("AnonymousInterfaceBasic$1"),
                    },
                    EnvironmentPolicy::PlainJar,
                ),
                &RecoveryEvidenceRequest::all(),
                &mut budget(),
            )
            .expect("the child physical class source is available independently"),
    );
    let value = child
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"value")
        .expect("the child method remains physically reported");
    assert!(matches!(
        &value.outcome,
        ClassSourceOutcome::Recovered { report, .. }
            if report.source_map.segments().iter().any(|segment| {
                segment.origin().primary().method() == Some(&value.item.identity)
            })
    ));
}

fn dt07_nested_anonymous_snapshot(
    scratch: &BridgeProjectionScratch,
    nested_override: Option<&str>,
    other_source: Option<&str>,
    corrupt_parent_descriptor: bool,
    corrupt_inner_code: bool,
    corrupt_inner_enclosing: bool,
    read_inner_capture: bool,
) -> ArtifactSnapshot {
    let directory = scratch.child("dt07-nested-input");
    let source_directory = directory.join("p");
    fs::create_dir_all(&source_directory).expect("create the frozen Java package directory");
    for (name, source) in [
        (
            "Action.java",
            include_str!(
                "../openspec/evidence/java-syntax-2026-09-27/dt07-nested-anonymous/input/p/Action.java"
            ),
        ),
        (
            "Factory.java",
            include_str!(
                "../openspec/evidence/java-syntax-2026-09-27/dt07-nested-anonymous/input/p/Factory.java"
            ),
        ),
        (
            "Nested.java",
            nested_override.unwrap_or(include_str!(
                "../openspec/evidence/java-syntax-2026-09-27/dt07-nested-anonymous/input/p/Nested.java"
            )),
        ),
        (
            "Runner.java",
            include_str!(
                "../openspec/evidence/java-syntax-2026-09-27/dt07-nested-anonymous/input/p/Runner.java"
            ),
        ),
    ] {
        fs::write(source_directory.join(name), source).expect("write the frozen DT-07 source");
    }
    let sources = vec![
        source_directory.join("Action.java"),
        source_directory.join("Factory.java"),
        source_directory.join("Nested.java"),
        source_directory.join("Runner.java"),
    ];
    if let Some(source) = other_source {
        fs::write(source_directory.join("Other.java"), source)
            .expect("write the cross-class anonymous-child use");
    }
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&directory)
        .args(sources)
        .output()
        .expect("JDK javac is available for the fixed DT-07 input");
    assert!(
        compile.status.success(),
        "javac rejected the frozen DT-07 source:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    if other_source.is_some() {
        let compile_other = Command::new("javac")
            .args(["--release", "8", "-g:none", "-classpath"])
            .arg(&directory)
            .arg("-d")
            .arg(&directory)
            .arg(source_directory.join("Other.java"))
            .output()
            .expect("JDK javac can compile the separate anonymous-child consumer");
        assert!(
            compile_other.status.success(),
            "javac rejected the cross-class use:\n{}",
            String::from_utf8_lossy(&compile_other.stderr)
        );
    }
    let mut names = vec![
        "Action.class",
        "Factory.class",
        "Nested.class",
        "Nested$1.class",
        "Nested$1$1.class",
        "Runner.class",
    ];
    if other_source.is_some() {
        names.push("Other.class");
    }
    let mut class_bytes: Vec<_> = names
        .iter()
        .map(|name| {
            fs::read(directory.join("p").join(name))
                .unwrap_or_else(|error| panic!("read compiled class {name}: {error}"))
        })
        .collect();
    if corrupt_parent_descriptor {
        let descriptor = b"()Lp/Action;";
        let replacement = b"()Lp/Except;";
        let position = class_bytes[3]
            .windows(descriptor.len())
            .position(|window| window == descriptor)
            .expect("the parent implementation has the direct Action return descriptor");
        class_bytes[3][position..position + replacement.len()].copy_from_slice(replacement);
    }
    if corrupt_inner_code {
        let run = test_method_headers(&class_bytes[4])
            .into_iter()
            .find(|method| method.name == b"run" && method.descriptor == b"()V")
            .expect("the inner child implements Action.run()");
        let code = run
            .attributes
            .iter()
            .find(|attribute| attribute.name == b"Code")
            .expect("Action.run() has a Code attribute");
        class_bytes[4][code.data_offset + 8] = 0xcb;
    }
    if corrupt_inner_enclosing {
        class_bytes[4] = patch_enclosing_method(&class_bytes[4], b"<init>", b"()V");
    }
    if read_inner_capture {
        let run = test_method_headers(&class_bytes[4])
            .into_iter()
            .find(|method| method.name == b"run" && method.descriptor == b"()V")
            .expect("the inner child implements Action.run()");
        let code = run
            .attributes
            .iter()
            .find(|attribute| attribute.name == b"Code")
            .expect("Action.run() has a Code attribute");
        let code_start = code.data_offset + 8;
        let old_length = test_u32(&class_bytes[4], code.data_offset + 4);
        let field =
            test_method_reference(&class_bytes[4], b"p/Nested$1$1", b"this$0", b"Lp/Nested$1;");
        let mut prefix = vec![0x2a, 0xb4];
        prefix.extend_from_slice(&field.to_be_bytes());
        prefix.push(0x57);
        class_bytes[4].splice(code_start..code_start, prefix);
        test_put_u32(&mut class_bytes[4], code.data_offset + 4, old_length + 5);
        test_put_u32(&mut class_bytes[4], code.length_offset, code.length + 5);
    }
    let entries: Vec<_> = names
        .iter()
        .zip(&class_bytes)
        .map(|(name, bytes)| {
            let archive_name = format!("p/{name}");
            (archive_name.into_bytes(), bytes.as_slice())
        })
        .collect();
    let entries: Vec<_> = entries
        .iter()
        .map(|(name, bytes)| (name.as_slice(), *bytes))
        .collect();
    open(zip_of(&entries))
}

#[test]
fn proved_two_level_anonymous_interfaces_project_as_one_nested_root_expression() {
    let scratch = BridgeProjectionScratch::new();
    let snapshot = dt07_nested_anonymous_snapshot(&scratch, None, None, false, false, false, false);
    let root = class_source_of(&snapshot, "p/Nested", EnvironmentPolicy::PlainJar);
    assert!(
        matches!(
            root.anonymous_interface_projection,
            ClassSourceAnonymousInterfaceProjection::Projected { .. }
        ),
        "projection={:#?} diagnostics={:#?}",
        root.anonymous_interface_projection,
        root.diagnostics
    );
    assert!(root.text.contains("new p.Factory() {"), "{}", root.text);
    assert!(root.text.contains("new p.Action() {"), "{}", root.text);
    assert!(!root.text.contains("Nested$1"), "{}", root.text);
    assert!(root.text.contains("public void run()"), "{}", root.text);
    let physical_child = class_source_of(&snapshot, "p/Nested$1$1", EnvironmentPolicy::PlainJar);
    assert!(
        physical_child.text.contains("class Nested$1$1"),
        "the physical child remains independently queryable: {}",
        physical_child.text
    );
}

#[test]
fn nested_anonymous_projection_refuses_duplicate_inner_allocations_and_constructor_effects() {
    let scratch = BridgeProjectionScratch::new();
    let variants = [
        (
            "duplicate-inner-site",
            "package p; public final class Nested { public static int trace; public static Factory create() { return new Factory() { public Action make() { if (System.nanoTime() == 0) return new Action() { public void run() { trace++; } }; return new Action() { public void run() { trace++; } }; } }; } }",
        ),
        (
            "inner-constructor-effect",
            "package p; public final class Nested { public static int trace; public static Factory create() { return new Factory() { public Action make() { return new Action() { { System.nanoTime(); } public void run() { trace++; } }; } }; } }",
        ),
        (
            "parent-field",
            "package p; public final class Nested { public static int trace; public static Factory create() { return new Factory() { int state; public Action make() { return new Action() { public void run() { trace += state; } }; } }; } }",
        ),
    ];
    for (name, source) in variants {
        let case = BridgeProjectionScratch(scratch.child(name));
        let snapshot =
            dt07_nested_anonymous_snapshot(&case, Some(source), None, false, false, false, false);
        let root = class_source_of(&snapshot, "p/Nested", EnvironmentPolicy::PlainJar);
        assert!(
            !matches!(
                root.anonymous_interface_projection,
                ClassSourceAnonymousInterfaceProjection::Projected { .. }
            ),
            "{name}: projection was published: {:#?}",
            root.anonymous_interface_projection
        );
        assert!(
            !root.text.contains("new p.Factory() {"),
            "{name}: {}",
            root.text
        );
    }
}

#[test]
fn nested_anonymous_projection_refuses_mismatched_relations_and_incomplete_leaf() {
    let scratch = BridgeProjectionScratch::new();
    for (name, corrupt_descriptor, corrupt_code, corrupt_enclosing, read_capture) in [
        ("parent-descriptor", true, false, false, false),
        ("leaf-code", false, true, false, false),
        ("inner-enclosing-method", false, false, true, false),
        ("read-immediate-parent-capture", false, false, false, true),
    ] {
        let case = BridgeProjectionScratch(scratch.child(name));
        let snapshot = dt07_nested_anonymous_snapshot(
            &case,
            None,
            None,
            corrupt_descriptor,
            corrupt_code,
            corrupt_enclosing,
            read_capture,
        );
        let root = class_source_of(&snapshot, "p/Nested", EnvironmentPolicy::PlainJar);
        assert!(
            !matches!(
                root.anonymous_interface_projection,
                ClassSourceAnonymousInterfaceProjection::Projected { .. }
            ),
            "{name}: projection was published: {:#?}",
            root.anonymous_interface_projection
        );
        assert!(
            !root.text.contains("new p.Action() {"),
            "{name}: {}",
            root.text
        );
    }
}

#[test]
fn nested_anonymous_projection_refuses_cross_class_identity_use_of_the_grandchild() {
    let scratch = BridgeProjectionScratch::new();
    let snapshot = dt07_nested_anonymous_snapshot(
        &scratch,
        None,
        Some(
            "package p; final class Other { static Action use() { return new Nested$1$1(null); } }",
        ),
        false,
        false,
        false,
        false,
    );
    let root = class_source_of(&snapshot, "p/Nested", EnvironmentPolicy::PlainJar);
    assert!(
        !matches!(
            root.anonymous_interface_projection,
            ClassSourceAnonymousInterfaceProjection::Projected { .. }
        ),
        "projection was published: {:#?}",
        root.anonymous_interface_projection
    );
    assert!(!root.text.contains("new p.Action() {"), "{}", root.text);
}

#[test]
fn nested_anonymous_projection_budget_and_cancellation_never_publish_partial_text() {
    let scratch = BridgeProjectionScratch::new();
    let snapshot = dt07_nested_anonymous_snapshot(&scratch, None, None, false, false, false, false);
    let request = request(
        &snapshot,
        ClassRef::Name {
            class: ClassNameQuery::internal("p/Nested"),
        },
        EnvironmentPolicy::PlainJar,
    );
    let engine = Engine::new();
    let complete = performed(
        engine
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut budget(),
            )
            .expect("the complete nested request is legal"),
    );
    assert!(complete.text.contains("new p.Action() {"));
    let cap = complete.usage.output_bytes.saturating_sub(1);
    let mut constrained = task_budget(&[
        BudgetOverride::new("output_bytes", cap).expect("the budget override is valid")
    ])
    .expect("the constrained task budget is valid");
    let stopped = performed(
        engine
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut constrained,
            )
            .expect("a budget stop retains the class report"),
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
    assert!(
        !stopped.text.contains("new p.Action() {"),
        "{}",
        stopped.text
    );

    let token = CancellationToken::new();
    token.cancel();
    let limits = task_budget(&[])
        .expect("default task limits are valid")
        .limits()
        .clone();
    let mut cancelled = Budget::with_cancellation_token(limits, token);
    let outcome = engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("cancellation is reported as an incomplete operation");
    match outcome {
        OperationOutcome::Incomplete(selection) => assert!(matches!(
            selection.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Performed(report) => {
            assert!(!report.text.contains("new p.Action() {"), "{}", report.text);
        }
        OperationOutcome::Ambiguous(_) => panic!("the exact root class is unambiguous"),
    }
}

#[test]
fn anonymous_interface_projects_a_proved_root_double_capture_and_recompiles() {
    let scratch = BridgeProjectionScratch::new();
    let original = scratch.child("dt08-original");
    let package = original.join("p");
    fs::create_dir_all(&package).expect("create source package");
    fs::write(package.join("Capture.java"), DT08_CAPTURE_SOURCE).expect("write Capture.java");
    fs::write(package.join("Runner.java"), DT08_RUNNER_SOURCE).expect("write Runner.java");
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&original)
        .arg(package.join("Capture.java"))
        .arg(package.join("Runner.java"))
        .output()
        .expect("javac is available for the Java 8 capture regression");
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let root_bytes = fs::read(original.join("p/Capture.class")).expect("read compiled root");
    let child_bytes = fs::read(original.join("p/Capture$1.class")).expect("read compiled child");
    let snapshot = open(zip_of(&[
        (b"p/Capture.class", &root_bytes),
        (b"p/Capture$1.class", &child_bytes),
    ]));
    let root = class_source_of(&snapshot, "p/Capture", EnvironmentPolicy::PlainJar);
    assert!(
        root.text.contains("return new java.lang.Runnable() {"),
        "text={} diagnostics={:?}",
        root.text,
        root.diagnostics
    );
    assert!(root.text.contains("println(arg0)"), "{}", root.text);
    assert!(!root.text.contains("Capture$1"), "{}", root.text);

    let child = class_source_of(&snapshot, "p/Capture$1", EnvironmentPolicy::PlainJar);
    assert!(child.text.contains("val$d"), "{}", child.text);
    assert!(child.text.contains("this.val$d = arg1"), "{}", child.text);

    let class_request = request(
        &snapshot,
        ClassRef::Name {
            class: ClassNameQuery::internal("p/Capture"),
        },
        EnvironmentPolicy::PlainJar,
    );
    let complete = performed(
        Engine::new()
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &class_request,
                &RecoveryEvidenceRequest::all(),
                &mut budget(),
            )
            .expect("the complete capture report is available"),
    );
    let mut constrained = task_budget(&[BudgetOverride::new(
        "analysis_steps",
        complete.usage.analysis_steps.saturating_sub(1),
    )
    .expect("the analysis-step override is valid")])
    .expect("the constrained capture budget is valid");
    let stopped = performed(
        Engine::new()
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &class_request,
                &RecoveryEvidenceRequest::all(),
                &mut constrained,
            )
            .expect("the budget stop remains a class report"),
    );
    assert!(
        !stopped.text.contains("new java.lang.Runnable() {"),
        "{}",
        stopped.text
    );
    assert!(
        stopped.text.contains("new p.Capture$1(arg0)"),
        "{}",
        stopped.text
    );

    let token = CancellationToken::new();
    token.cancel();
    let limits = task_budget(&[])
        .expect("default task limits are valid")
        .limits()
        .clone();
    let mut cancelled = Budget::with_cancellation_token(limits, token);
    let cancelled = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &class_request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("cancellation remains an operation outcome");
    match cancelled {
        OperationOutcome::Performed(report) => {
            assert!(
                !report.text.contains("new java.lang.Runnable() {"),
                "{}",
                report.text
            );
        }
        OperationOutcome::Incomplete(selection) => assert!(matches!(
            selection.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Ambiguous(_) => panic!("one fixed capture class binds uniquely"),
    }

    let recompilation = scratch.child("dt08-recompiled");
    let package = recompilation.join("p");
    fs::create_dir_all(&package).expect("create recompile package");
    fs::write(package.join("Capture.java"), &root.text).expect("write recovered root");
    fs::write(package.join("Runner.java"), DT08_RUNNER_SOURCE).expect("write runner");
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&recompilation)
        .arg(package.join("Capture.java"))
        .arg(package.join("Runner.java"))
        .output()
        .expect("javac is available for the recovered source");
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&recompilation)
        .arg("p.Runner")
        .output()
        .expect("java is available for the verified capture run");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), "2.5\n-0.0\n");
}

#[test]
fn anonymous_double_capture_refuses_a_changed_slot_descriptor_and_cross_class_use() {
    let scratch = BridgeProjectionScratch::new();
    let cases = [
        (
            "changed-slot",
            DT08_CAPTURE_SOURCE.to_owned(),
            None,
            "anonymous_capture_argument_unproved",
        ),
        (
            "reused-parameter-slot",
            DT08_CAPTURE_SOURCE.to_owned(),
            None,
            "anonymous_capture_argument_unproved",
        ),
        (
            "changed-descriptor",
            DT08_CAPTURE_SOURCE.to_owned(),
            None,
            "anonymous_child_shape_unproved",
        ),
        (
            "wrong-field-owner",
            DT08_CAPTURE_SOURCE.to_owned(),
            None,
            "anonymous_child_shape_unproved",
        ),
        (
            "wrong-read-field-owner",
            DT08_CAPTURE_SOURCE.to_owned(),
            None,
            "anonymous_child_shape_unproved",
        ),
        (
            "extra-write",
            DT08_CAPTURE_SOURCE.to_owned(),
            None,
            "anonymous_child_shape_unproved",
        ),
        (
            "exception-range",
            DT08_CAPTURE_SOURCE.to_owned(),
            None,
            "anonymous_child_shape_unproved",
        ),
        (
            "method-handle-use",
            DT08_CAPTURE_SOURCE.to_owned(),
            None,
            "anonymous_child_shape_unproved",
        ),
        (
            "cross-class-use",
            DT08_CAPTURE_SOURCE.to_owned(),
            Some(
                "package p; final class Other { static Runnable extra() { return new Capture$1(); } }\n",
            ),
            "anonymous_interface_child_additional_use",
        ),
    ];
    for (name, source, other, refusal) in cases {
        let directory = scratch.child(name);
        let package = directory.join("p");
        fs::create_dir_all(&package).expect("create source package");
        fs::write(package.join("Capture.java"), source).expect("write capture source");
        let compile = Command::new("javac")
            .args(["--release", "8", "-g:none", "-d"])
            .arg(&directory)
            .arg(package.join("Capture.java"))
            .output()
            .expect("javac is available for capture boundaries");
        assert!(
            compile.status.success(),
            "{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        if let Some(other) = other {
            fs::write(package.join("Other.java"), other).expect("write cross-class user");
            let compile = Command::new("javac")
                .args(["--release", "8", "-g:none", "-classpath"])
                .arg(&directory)
                .args(["-d"])
                .arg(&directory)
                .arg(package.join("Other.java"))
                .output()
                .expect("javac is available for the cross-class use");
            assert!(
                compile.status.success(),
                "{}",
                String::from_utf8_lossy(&compile.stderr)
            );
        }
        let root_bytes = fs::read(package.join("Capture.class")).expect("read root class");
        let root_bytes = if name == "changed-slot" {
            let mut patched = root_bytes;
            let method = test_method_headers(&patched)
                .into_iter()
                .find(|method| {
                    method.name == b"create" && method.descriptor == b"(D)Ljava/lang/Runnable;"
                })
                .expect("the fixed create method exists");
            let code = method
                .attributes
                .iter()
                .find(|attribute| attribute.name == b"Code")
                .expect("create has Code");
            assert_eq!(
                patched[code.data_offset + 8 + 4],
                0x26,
                "BCI 4 is the sole dload argument"
            );
            patched[code.data_offset + 8 + 4] = 0x0e; // dconst_0: same constructor slot, wrong value
            patched
        } else if name == "reused-parameter-slot" {
            patch_capture_root_parameter_slot_reuse(&root_bytes)
        } else {
            root_bytes
        };
        let child_bytes = fs::read(package.join("Capture$1.class")).expect("read child class");
        let child_bytes = if name == "changed-descriptor" {
            patch_one_byte_utf8(&child_bytes, b'D', b'I')
        } else if name == "wrong-field-owner" {
            patch_capture_field_owner(&child_bytes, b"java/lang/Object")
        } else if name == "wrong-read-field-owner" {
            patch_capture_read_field_owner(&child_bytes, b"java/lang/Object")
        } else if name == "extra-write" {
            patch_capture_constructor_extra_write(&child_bytes)
        } else if name == "exception-range" {
            patch_capture_constructor_exception_range(&child_bytes)
        } else if name == "method-handle-use" {
            append_capture_field_method_handle(&child_bytes)
        } else {
            child_bytes
        };
        let mut entries = vec![
            (b"p/Capture.class".as_slice(), root_bytes.as_slice()),
            (b"p/Capture$1.class".as_slice(), child_bytes.as_slice()),
        ];
        let other_bytes =
            other.map(|_| fs::read(package.join("Other.class")).expect("read user class"));
        if let Some(bytes) = other_bytes.as_ref() {
            entries.push((b"p/Other.class".as_slice(), bytes.as_slice()));
        }
        let snapshot = open(zip_of(&entries));
        let root = class_source_of(&snapshot, "p/Capture", EnvironmentPolicy::PlainJar);
        if name == "reused-parameter-slot" {
            assert!(root.text.contains("not recovered"), "{}", root.text);
        } else {
            assert!(
                root.text.contains("new p.Capture$1("),
                "{name}: {}",
                root.text
            );
        }
        assert!(
            !root.text.contains("new java.lang.Runnable() {"),
            "{name}: {}",
            root.text
        );
        if name != "reused-parameter-slot" {
            assert!(
                root.diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == refusal),
                "{name}: {:?}",
                root.diagnostics
            );
        }
        let child = class_source_of(&snapshot, "p/Capture$1", EnvironmentPolicy::PlainJar);
        assert!(child.text.contains("val$"), "{name}: {}", child.text);
    }
}

#[test]
fn anonymous_inner_this_refuses_an_additional_local_capture_and_keeps_physical_child() {
    let scratch = BridgeProjectionScratch::new();
    let directory = scratch.child("inner-this-extra-local");
    fs::write(
        directory.join("Inner.java"),
        "public class Inner { static Object observed; int value = 37;\n\
         Runnable make(int captured, int second) { return new Runnable() {\n\
         public void run() { observed = Inner.this; Inner.this.value += captured + second; }\
         }; } }\n",
    )
    .expect("write the enclosing-instance plus local-capture class");
    compile_java_8(&directory, "Inner.java", &directory);
    let root_bytes = fs::read(directory.join("Inner.class")).expect("read the root class");
    let child_bytes = fs::read(directory.join("Inner$1.class")).expect("read the child class");
    let snapshot = open(zip_of(&[
        (b"Inner.class", &root_bytes),
        (b"Inner$1.class", &child_bytes),
    ]));
    let root = class_source_of(&snapshot, "Inner", EnvironmentPolicy::PlainJar);
    assert!(root.text.contains("new Inner$1("), "{}", root.text);
    assert!(
        !root.text.contains("new java.lang.Runnable() {"),
        "{}",
        root.text
    );

    let mut child_budget = budget();
    let child = performed(
        Engine::new()
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request(
                    &snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal("Inner$1"),
                    },
                    EnvironmentPolicy::PlainJar,
                ),
                &RecoveryEvidenceRequest::all(),
                &mut child_budget,
            )
            .expect("the physical child report with source maps is available"),
    );
    assert!(child.text.contains("this$0"), "{}", child.text);
    assert!(child.text.contains("captured"), "{}", child.text);
    assert!(child.text.contains("second"), "{}", child.text);
}

#[test]
fn anonymous_inner_this_refuses_a_mismatched_enclosing_method_identity() {
    let scratch = BridgeProjectionScratch::new();
    let directory = scratch.child("inner-this-enclosing-mismatch");
    fs::write(
        directory.join("Inner.java"),
        "public class Inner { static Object observed;\n\
         static void helper() {}\n\
         Runnable make() { return new Runnable() { public void run() {\n\
         observed = Inner.this; Inner.helper(); Inner.this.hashCode();\
         } }; } }\n",
    )
    .expect("write an anonymous class whose pool names another enclosing method");
    compile_java_8(&directory, "Inner.java", &directory);
    let root_bytes = fs::read(directory.join("Inner.class")).expect("read the root class");
    let original_child =
        fs::read(directory.join("Inner$1.class")).expect("read the anonymous child");
    let child_bytes = patch_enclosing_method(&original_child, b"helper", b"()V");
    let snapshot = open(zip_of(&[
        (b"Inner.class", &root_bytes),
        (b"Inner$1.class", &child_bytes),
    ]));
    let root = class_source_of(&snapshot, "Inner", EnvironmentPolicy::PlainJar);
    assert!(root.text.contains("new Inner$1("), "{}", root.text);
    assert!(
        !root.text.contains("new java.lang.Runnable() {"),
        "{}",
        root.text
    );
    let child = class_source_of(&snapshot, "Inner$1", EnvironmentPolicy::PlainJar);
    assert!(child.text.contains("this$0"), "{}", child.text);
}

#[test]
fn anonymous_inner_this_refuses_a_second_owner_allocation_and_keeps_physical_child() {
    let mut root = ANONYMOUS_INNER_THIS_ROOT.to_vec();
    let methods = test_method_headers(ANONYMOUS_INNER_THIS_ROOT);
    let make = methods
        .iter()
        .find(|method| method.name == b"make")
        .expect("the direct-return factory exists");
    let make_code = make
        .attributes
        .iter()
        .find(|attribute| attribute.name == b"Code")
        .expect("the factory has Code");
    let make_start = make_code.data_offset + 8;
    let child_class = test_u16(ANONYMOUS_INNER_THIS_ROOT, make_start + 1);
    let constructor = test_method_reference(
        ANONYMOUS_INNER_THIS_ROOT,
        b"Inner$1",
        b"<init>",
        b"(LInner;)V",
    );
    let main = methods
        .iter()
        .find(|method| method.name == b"main")
        .expect("the runner exists");
    let main_code = main
        .attributes
        .iter()
        .find(|attribute| attribute.name == b"Code")
        .expect("the runner has Code");
    let code_start = main_code.data_offset + 8;
    let old_code_length = test_u32(ANONYMOUS_INNER_THIS_ROOT, main_code.data_offset + 4);
    assert_eq!(
        ANONYMOUS_INNER_THIS_ROOT[code_start + old_code_length - 1],
        0xb1
    );

    // Reuse the same physical child from a second method in the owner. This tests the
    // owner-wide allocation inventory rather than only duplicate sites in the candidate method.
    let mut second_site = vec![0xbb];
    u16b(
        &mut second_site,
        u16::try_from(child_class).expect("the child class index fits u16"),
    );
    second_site.extend([0x59, 0x2b, 0xb7]); // dup; aload_1 (the existing Inner local); invokespecial
    u16b(&mut second_site, constructor);
    second_site.push(0x57); // pop
    root.splice(
        code_start + old_code_length - 1..code_start + old_code_length - 1,
        second_site.clone(),
    );
    test_put_u16(&mut root, main_code.data_offset, 3);
    test_put_u32(
        &mut root,
        main_code.data_offset + 4,
        old_code_length + second_site.len(),
    );
    test_put_u32(
        &mut root,
        main_code.length_offset,
        main_code.length + second_site.len(),
    );

    let snapshot = open(zip_of(&[
        (b"Inner.class", &root),
        (b"Inner$1.class", ANONYMOUS_INNER_THIS_CHILD),
    ]));
    let report = class_source_of(&snapshot, "Inner", EnvironmentPolicy::PlainJar);
    assert!(report.text.contains("new Inner$1(this)"), "{}", report.text);
    assert!(
        !report.text.contains("new java.lang.Runnable() {"),
        "{}",
        report.text
    );
    let child = class_source_of(&snapshot, "Inner$1", EnvironmentPolicy::PlainJar);
    assert!(child.text.contains("this$0"), "{}", child.text);
}

#[test]
fn anonymous_inner_this_refuses_a_fallback_child_method_without_partial_projection() {
    let mut child_bytes = ANONYMOUS_INNER_THIS_CHILD.to_vec();
    let run = test_method_headers(ANONYMOUS_INNER_THIS_CHILD)
        .into_iter()
        .find(|method| method.name == b"run" && method.descriptor == b"()V")
        .expect("the anonymous Runnable method exists");
    let code = run
        .attributes
        .iter()
        .find(|attribute| attribute.name == b"Code")
        .expect("run() has Code");
    child_bytes[code.data_offset + 8] = 0xcb; // reserved opcode makes this physical method mixed
    let snapshot = open(zip_of(&[
        (b"Inner.class", ANONYMOUS_INNER_THIS_ROOT),
        (b"Inner$1.class", &child_bytes),
    ]));
    let root = class_source_of(&snapshot, "Inner", EnvironmentPolicy::PlainJar);
    assert!(root.text.contains("new Inner$1(this)"), "{}", root.text);
    assert!(
        !root.text.contains("new java.lang.Runnable() {"),
        "{}",
        root.text
    );
    let child = class_source_of(&snapshot, "Inner$1", EnvironmentPolicy::PlainJar);
    assert!(child.text.contains("this$0"), "{}", child.text);
    let run = child
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"run")
        .expect("the physical fallback remains listed");
    assert!(
        matches!(run.outcome, ClassSourceOutcome::Recovered { ref report, .. }
        if report.quality == jarde::ir::Quality::Fallback
            && !report.outcome.produced()),
        "physical child method must retain its stopped fallback: {:?}",
        run.outcome
    );
}

#[test]
fn anonymous_inner_this_refuses_a_cross_class_constructor_reference() {
    let scratch = BridgeProjectionScratch::new();
    let directory = scratch.child("inner-this-cross-class-reference");
    fs::write(directory.join("Inner.class"), ANONYMOUS_INNER_THIS_ROOT)
        .expect("write the frozen owner class");
    fs::write(directory.join("Inner$1.class"), ANONYMOUS_INNER_THIS_CHILD)
        .expect("write the frozen anonymous class");
    fs::write(
        directory.join("Other.java"),
        "final class Other { static Runnable make(Inner outer) { return new Inner$1(outer); } }\n",
    )
    .expect("write a second physical owner of the anonymous constructor");
    compile_java_8(&directory, "Other.java", &directory);
    let other = fs::read(directory.join("Other.class")).expect("read the second owner");
    let snapshot = open(zip_of(&[
        (b"Inner.class", ANONYMOUS_INNER_THIS_ROOT),
        (b"Inner$1.class", ANONYMOUS_INNER_THIS_CHILD),
        (b"Other.class", &other),
    ]));
    let root = class_source_of(&snapshot, "Inner", EnvironmentPolicy::PlainJar);
    assert!(root.text.contains("new Inner$1(this)"), "{}", root.text);
    assert!(
        !root.text.contains("new java.lang.Runnable() {"),
        "{}",
        root.text
    );
    assert!(
        class_source_of(&snapshot, "Inner$1", EnvironmentPolicy::PlainJar)
            .text
            .contains("this$0")
    );
}

#[test]
fn anonymous_inner_this_budget_and_cancellation_never_publish_partial_projection() {
    let snapshot = open(zip_of(&[
        (b"Inner.class", ANONYMOUS_INNER_THIS_ROOT),
        (b"Inner$1.class", ANONYMOUS_INNER_THIS_CHILD),
    ]));
    let engine = Engine::new();
    let root_request = request(
        &snapshot,
        ClassRef::Name {
            class: ClassNameQuery::internal("Inner"),
        },
        EnvironmentPolicy::PlainJar,
    );
    let complete = performed(
        engine
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &root_request,
                &RecoveryEvidenceRequest::all(),
                &mut budget(),
            )
            .expect("the captured anonymous source is available"),
    );
    assert!(complete.text.contains("new java.lang.Runnable() {"));
    assert!(complete.text.contains("Inner.this"));
    let make = complete
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"make")
        .expect("the caller method remains physically reported");
    let make_report = match &make.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("the caller method recovered: {other:?}"),
    };
    let child_request = request(
        &snapshot,
        ClassRef::Name {
            class: ClassNameQuery::internal("Inner$1"),
        },
        EnvironmentPolicy::PlainJar,
    );
    let physical_child = performed(
        Engine::new()
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &child_request,
                &RecoveryEvidenceRequest::all(),
                &mut budget(),
            )
            .expect("the independent child source map is available"),
    );
    let run = physical_child
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"run")
        .expect("the physical child method remains listed");
    let run_report = match &run.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("the physical child method recovered: {other:?}"),
    };
    assert!(
        !run_report.source_map.of_bci(1).is_empty(),
        "physical source map does not retain the first capture read: {:#?} text={}",
        run_report.source_map.segments(),
        run.text
    );
    assert!(!run_report.source_map.of_bci(8).is_empty());
    assert!(make_report.source_map.segments().iter().any(|segment| {
        segment.origin().primary().method() == Some(&make.item.identity)
            && segment.mentions(4).is_some()
    }));
    let constructor = physical_child
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"<init>")
        .expect("the physical child constructor remains listed");
    let constructor_report = match &constructor.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("the physical child constructor recovered: {other:?}"),
    };
    assert!(!constructor_report.source_map.of_bci(2).is_empty());

    let cap = complete.usage.output_bytes.saturating_sub(1);
    let mut constrained =
        task_budget(&[BudgetOverride::new("output_bytes", cap).expect("valid output cap")])
            .expect("the constrained budget is valid");
    let stopped = performed(
        engine
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &root_request,
                &RecoveryEvidenceRequest::all(),
                &mut constrained,
            )
            .expect("a projection budget stop retains a physical root report"),
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
    assert!(stopped.text.contains("new Inner$1("), "{}", stopped.text);
    assert!(!stopped.text.contains("new java.lang.Runnable() {"));
    let physical_child = class_source_of(&snapshot, "Inner$1", EnvironmentPolicy::PlainJar);
    assert!(physical_child.text.contains("this$0"));

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(budget().limits().clone(), token);
    let outcome = engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &root_request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("a cancelled source request has a terminal outcome");
    match outcome {
        OperationOutcome::Incomplete(selection) => {
            assert!(matches!(
                selection.execution,
                ExecutionReport::Cancelled { .. }
            ));
        }
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                report.execution,
                ExecutionReport::Cancelled { .. }
            ));
            assert!(!report.text.contains("new java.lang.Runnable() {"));
        }
        OperationOutcome::Ambiguous(_) => panic!("the frozen Inner class is unique"),
    }
    assert!(
        class_source_of(&snapshot, "Inner$1", EnvironmentPolicy::PlainJar)
            .text
            .contains("this$0")
    );
}

#[test]
fn proved_anonymous_superclass_forwards_the_original_ordered_arguments() {
    let snapshot = open(zip_of(&[
        (b"AnonymousSuperDirect.class", ANONYMOUS_SUPER_DIRECT_ROOT),
        (
            b"AnonymousSuperDirect$1.class",
            ANONYMOUS_SUPER_DIRECT_CHILD,
        ),
        (b"Base.class", ANONYMOUS_SUPER_DIRECT_BASE),
    ]));
    let root = class_source_of(
        &snapshot,
        "AnonymousSuperDirect",
        EnvironmentPolicy::PlainJar,
    );
    assert!(
        root.text.contains("new Base(next(), next()) {"),
        "text={} diagnostics={:#?}",
        root.text,
        root.diagnostics
    );
    assert_eq!(
        root.text.matches("new Base(next(), next()) {").count(),
        1,
        "{}",
        root.text
    );
    assert!(
        root.text.contains("return super.sum() + 1;"),
        "{}",
        root.text
    );
    assert!(
        !root.text.contains("new AnonymousSuperDirect$1("),
        "{}",
        root.text
    );

    let essential = performed(
        Engine::new()
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request(
                    &snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal("AnonymousSuperDirect"),
                    },
                    EnvironmentPolicy::PlainJar,
                ),
                &RecoveryEvidenceRequest::essential(),
                &mut budget(),
            )
            .expect("essential-evidence class source is available"),
    );
    assert_eq!(essential.text, root.text);

    let child = class_source_of(
        &snapshot,
        "AnonymousSuperDirect$1",
        EnvironmentPolicy::PlainJar,
    );
    assert!(
        child
            .text
            .contains("class AnonymousSuperDirect$1 extends Base")
    );
    let base = class_source_of(&snapshot, "Base", EnvironmentPolicy::PlainJar);
    assert!(base.text.contains("Base(int ") && base.text.contains("long "));
}

#[test]
fn anonymous_superclass_refuses_reordered_or_reused_constructor_slots() {
    let original = [0x2a, 0x1b, 0x1c, 0xb7];
    let variants = [
        ("reused-slot", [0x2a, 0x1b, 0x1b, 0xb7]),
        ("reordered-slots", [0x2a, 0x1c, 0x1b, 0xb7]),
    ];
    for (label, replacement) in variants {
        let mut child = ANONYMOUS_SUPER_DIRECT_CHILD.to_vec();
        let start = child
            .windows(original.len())
            .position(|window| window == original)
            .expect("the anonymous constructor's direct forwarding bytecode");
        child[start..start + replacement.len()].copy_from_slice(&replacement);
        let snapshot = open(zip_of(&[
            (b"AnonymousSuperDirect.class", ANONYMOUS_SUPER_DIRECT_ROOT),
            (b"AnonymousSuperDirect$1.class", &child),
            (b"Base.class", ANONYMOUS_SUPER_DIRECT_BASE),
        ]));
        let report = class_source_of(
            &snapshot,
            "AnonymousSuperDirect",
            EnvironmentPolicy::PlainJar,
        );
        assert!(
            report
                .text
                .contains("new AnonymousSuperDirect$1(next(), next())"),
            "{label}: {}",
            report.text
        );
        assert!(
            !report.text.contains("new Base(next(), next()) {"),
            "{label}"
        );
        let child_report = class_source_of(
            &snapshot,
            "AnonymousSuperDirect$1",
            EnvironmentPolicy::PlainJar,
        );
        assert!(
            child_report
                .text
                .contains("class AnonymousSuperDirect$1 extends Base")
        );
    }
}

#[test]
fn anonymous_superclass_refuses_a_different_parent_constructor_overload() {
    let mut child = ANONYMOUS_SUPER_DIRECT_CHILD.to_vec();
    let pool_count = test_u16(&child, 8);
    let mut cursor = 10;
    let mut index = 1;
    let mut utf8 = vec![Vec::new(); pool_count];
    let mut class_names = vec![None; pool_count];
    let mut init_name = None;
    while index < pool_count {
        let tag = child[cursor];
        let entry_start = cursor;
        cursor += 1;
        match tag {
            1 => {
                let length = test_u16(&child, cursor);
                cursor += 2;
                utf8[index] = child[cursor..cursor + length].to_vec();
                cursor += length;
            }
            7 => {
                class_names[index] = Some(test_u16(&child, cursor));
                cursor += 2;
            }
            3 | 4 | 9 | 10 | 11 | 12 | 17 | 18 => {
                cursor += if matches!(tag, 3 | 4) { 4 } else { 4 }
            }
            5 | 6 => {
                cursor += 8;
                index += 1;
            }
            8 | 16 | 19 | 20 => cursor += 2,
            15 => cursor += 3,
            other => panic!("unexpected constant-pool tag {other}"),
        }
        if tag == 1 && utf8[index] == b"<init>" {
            init_name = Some(index);
        }
        assert!(cursor > entry_start);
        index += 1;
    }
    let pool_end = cursor;
    let base_class = class_names
        .iter()
        .enumerate()
        .find_map(|(index, name)| name.filter(|name| utf8[*name] == b"Base").map(|_| index));
    let descriptor_index = pool_count;
    let name_and_type_index = pool_count + 1;
    let method_reference_index = pool_count + 2;
    let mut additions = Vec::new();
    additions.push(1);
    u16b(&mut additions, 5);
    additions.extend_from_slice(b"(IJ)V");
    additions.push(12);
    u16b(
        &mut additions,
        u16::try_from(init_name.expect("the constructor name constant exists"))
            .expect("the name index fits u16"),
    );
    u16b(
        &mut additions,
        u16::try_from(descriptor_index).expect("the descriptor index fits u16"),
    );
    additions.push(10);
    u16b(
        &mut additions,
        u16::try_from(base_class.expect("the Base class constant exists"))
            .expect("the class index fits u16"),
    );
    u16b(
        &mut additions,
        u16::try_from(name_and_type_index).expect("the name and type index fits u16"),
    );
    child.splice(pool_end..pool_end, additions);
    test_put_u16(&mut child, 8, pool_count + 3);

    let constructor = test_method_headers(&child)
        .into_iter()
        .find(|method| method.name == b"<init>")
        .expect("the physical constructor exists");
    let code = constructor
        .attributes
        .iter()
        .find(|attribute| attribute.name == b"Code")
        .expect("the constructor has Code");
    let code_start = code.data_offset + 8;
    assert_eq!(child[code_start + 3], 0xb7);
    child[code_start + 4..code_start + 6].copy_from_slice(
        &u16::try_from(method_reference_index)
            .expect("the overload reference index fits u16")
            .to_be_bytes(),
    );

    let snapshot = open(zip_of(&[
        (b"AnonymousSuperDirect.class", ANONYMOUS_SUPER_DIRECT_ROOT),
        (b"AnonymousSuperDirect$1.class", &child),
        (b"Base.class", ANONYMOUS_SUPER_DIRECT_BASE),
    ]));
    let report = class_source_of(
        &snapshot,
        "AnonymousSuperDirect",
        EnvironmentPolicy::PlainJar,
    );
    assert!(
        report
            .text
            .contains("new AnonymousSuperDirect$1(next(), next())")
    );
    assert!(!report.text.contains("new Base(next(), next()) {"));
}

#[test]
fn anonymous_superclass_refuses_a_second_allocation_bci_for_the_same_child() {
    let original = ANONYMOUS_SUPER_DIRECT_ROOT;
    let methods = test_method_headers(original);
    let make = methods
        .iter()
        .find(|method| method.name == b"make")
        .expect("the direct-return factory exists");
    let make_code = make
        .attributes
        .iter()
        .find(|attribute| attribute.name == b"Code")
        .expect("the direct-return factory has Code");
    let make_start = make_code.data_offset + 8;
    let child_class = test_u16(original, make_start + 1);
    let constructor = test_u16(original, make_start + 11);

    let mut root = original.to_vec();
    let main = test_method_headers(original)
        .into_iter()
        .find(|method| method.name == b"main")
        .expect("the main method exists");
    let main_code = main
        .attributes
        .iter()
        .find(|attribute| attribute.name == b"Code")
        .expect("main has Code");
    let main_start = main_code.data_offset + 8;
    let old_code_length = test_u32(original, main_code.data_offset + 4);
    assert_eq!(original[main_start + old_code_length - 1], 0xb1);

    // Add a second allocation of the same physical child after the print. This is a distinct BCI
    // in the same owner, so owner-wide allocation identity must refuse the projection.
    let mut second_site = vec![0xbb];
    u16b(
        &mut second_site,
        u16::try_from(child_class).expect("the class reference fits u16"),
    );
    second_site.extend([0x59, 0x06, 0x07, 0xb7]); // dup; iconst_3; iconst_4; invokespecial
    u16b(
        &mut second_site,
        u16::try_from(constructor).expect("the constructor reference fits u16"),
    );
    second_site.push(0x57); // pop
    let old_attribute_length = main_code.length;
    let insertion = main_start + old_code_length - 1;
    root.splice(insertion..insertion, second_site.clone());
    test_put_u16(&mut root, main_code.data_offset, 4);
    test_put_u32(
        &mut root,
        main_code.data_offset + 4,
        old_code_length + second_site.len(),
    );
    test_put_u32(
        &mut root,
        main_code.length_offset,
        old_attribute_length + second_site.len(),
    );

    let snapshot = open(zip_of(&[
        (b"AnonymousSuperDirect.class", &root),
        (
            b"AnonymousSuperDirect$1.class",
            ANONYMOUS_SUPER_DIRECT_CHILD,
        ),
        (b"Base.class", ANONYMOUS_SUPER_DIRECT_BASE),
    ]));
    let report = class_source_of(
        &snapshot,
        "AnonymousSuperDirect",
        EnvironmentPolicy::PlainJar,
    );
    assert!(
        report
            .text
            .contains("new AnonymousSuperDirect$1(next(), next())"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("new Base(next(), next()) {"));
}

#[test]
fn anonymous_superclass_refuses_captures_fields_and_constructor_effects() {
    let scratch = BridgeProjectionScratch::new();
    let variants = [
        (
            "capture-field",
            "public class AnonymousSuperDirect { static Base make(int captured) { return new Base(1) { int state = captured; int sum() { return super.sum() + state; } }; } }\nclass Base { Base(int x) {} int sum() { return 1; } }\n",
        ),
        (
            "instance-field",
            "public class AnonymousSuperDirect { static Base make() { return new Base(1) { int state = 3; int sum() { return super.sum() + state; } }; } }\nclass Base { Base(int x) {} int sum() { return 1; } }\n",
        ),
        (
            "constructor-effect",
            "public class AnonymousSuperDirect { static void touch() {} static Base make() { return new Base(1) { { touch(); } int sum() { return super.sum(); } }; } }\nclass Base { Base(int x) {} int sum() { return 1; } }\n",
        ),
    ];
    for (label, source) in variants {
        let directory = scratch.child(label);
        fs::write(directory.join("AnonymousSuperDirect.java"), source)
            .expect("write anonymous superclass variant");
        compile_java_8(&directory, "AnonymousSuperDirect.java", &directory);
        let root =
            fs::read(directory.join("AnonymousSuperDirect.class")).expect("read compiled root");
        let child = fs::read(directory.join("AnonymousSuperDirect$1.class"))
            .expect("read compiled anonymous child");
        let base = fs::read(directory.join("Base.class")).expect("read compiled Base");
        let snapshot = open(zip_of(&[
            (b"AnonymousSuperDirect.class", &root),
            (b"AnonymousSuperDirect$1.class", &child),
            (b"Base.class", &base),
        ]));
        let report = class_source_of(
            &snapshot,
            "AnonymousSuperDirect",
            EnvironmentPolicy::PlainJar,
        );
        assert!(
            report.text.contains("new AnonymousSuperDirect$1("),
            "{label}: {}",
            report.text
        );
        assert!(!report.text.contains("new Base(1) {"), "{label}");
    }
}

#[test]
fn anonymous_superclass_recovers_one_proved_static_int_initializer_block() {
    let scratch = BridgeProjectionScratch::new();
    let directory = scratch.child("static-int-initializer");
    fs::write(
        directory.join("AnonymousInit.java"),
        "public class AnonymousInit { static int value; static Base make() { return new Base() { { value = 1; } public void run() { value += 7; } }; } }\nclass Base { public void run() {} }\n",
    ).expect("write initializer source");
    compile_java_8(&directory, "AnonymousInit.java", &directory);
    let root = fs::read(directory.join("AnonymousInit.class")).expect("read root");
    let child = fs::read(directory.join("AnonymousInit$1.class")).expect("read child");
    let base = fs::read(directory.join("Base.class")).expect("read Base");
    let snapshot = open(zip_of(&[
        (b"AnonymousInit.class", &root),
        (b"AnonymousInit$1.class", &child),
        (b"Base.class", &base),
    ]));
    let report = class_source_of(&snapshot, "AnonymousInit", EnvironmentPolicy::PlainJar);
    let initializer = report
        .text
        .find("AnonymousInit.value = 1;")
        .expect("initializer statement is projected");
    let method = report
        .text
        .find("public void run()")
        .expect("override is projected");
    assert!(initializer < method, "{}", report.text);
    assert!(
        !report.text.contains("new AnonymousInit$1"),
        "{}",
        report.text
    );
    let request = request(
        &snapshot,
        ClassRef::Name {
            class: ClassNameQuery::internal("AnonymousInit"),
        },
        EnvironmentPolicy::PlainJar,
    );
    let mut constrained = task_budget(&[BudgetOverride::new(
        "output_bytes",
        report.usage.output_bytes.saturating_sub(1),
    )
    .expect("valid output budget")])
    .expect("valid constrained budget");
    let stopped = performed(
        Engine::new()
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut constrained,
            )
            .expect("budget stop returns a class report"),
    );
    assert!(
        stopped.text.contains("new AnonymousInit$1()"),
        "{}",
        stopped.text
    );
    assert!(
        !stopped.text.contains("AnonymousInit.value = 1;"),
        "{}",
        stopped.text
    );
    let token = CancellationToken::new();
    token.cancel();
    let limits = task_budget(&[]).expect("default limits").limits().clone();
    let mut cancelled = Budget::with_cancellation_token(limits, token);
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("cancellation returns a class report")
    {
        OperationOutcome::Incomplete(selection) => assert!(matches!(
            selection.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                report.execution,
                ExecutionReport::Cancelled { .. }
            ));
            assert!(
                !report.text.contains("AnonymousInit.value = 1;"),
                "{}",
                report.text
            );
            assert!(
                report.text.contains("new AnonymousInit$1()"),
                "{}",
                report.text
            );
        }
        OperationOutcome::Ambiguous(_) => panic!("fixed class identity cannot be ambiguous"),
    }
    let physical_child = class_source_of(&snapshot, "AnonymousInit$1", EnvironmentPolicy::PlainJar);
    assert!(physical_child.text.contains("AnonymousInit$1()"));

    let negative_dir = scratch.child("multiple-initializer-effects");
    fs::write(
        negative_dir.join("AnonymousInit.java"),
        "public class AnonymousInit { static int value; static int other; static Base make() { return new Base() { { value = 1; other = 2; } public void run() {} }; } }\nclass Base { public void run() {} }\n",
    ).expect("write multi-effect source");
    compile_java_8(&negative_dir, "AnonymousInit.java", &negative_dir);
    let root = fs::read(negative_dir.join("AnonymousInit.class")).expect("read negative root");
    let child = fs::read(negative_dir.join("AnonymousInit$1.class")).expect("read negative child");
    let base = fs::read(negative_dir.join("Base.class")).expect("read negative Base");
    let snapshot = open(zip_of(&[
        (b"AnonymousInit.class", &root),
        (b"AnonymousInit$1.class", &child),
        (b"Base.class", &base),
    ]));
    let report = class_source_of(&snapshot, "AnonymousInit", EnvironmentPolicy::PlainJar);
    assert!(
        report.text.contains("new AnonymousInit$1()"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("new Base() {"), "{}", report.text);
}

#[test]
fn anonymous_constructor_initializer_rejects_wrong_field_owner_descriptor_and_handlers() {
    let scratch = BridgeProjectionScratch::new();
    let variants = [
        (
            "wrong-owner",
            "public class AnonymousInit { static Base make() { return new Base() { { Holder.value = 1; } public void run() {} }; } }\nclass Holder { static int value; }\nclass Base { public void run() {} }\n",
        ),
        (
            "wrong-descriptor",
            "public class AnonymousInit { static long value; static Base make() { return new Base() { { value = 1; } public void run() {} }; } }\nclass Base { public void run() {} }\n",
        ),
        (
            "exception-handler",
            "public class AnonymousInit { static int value; static Base make() { return new Base() { { try { value = 1; } catch (RuntimeException ignored) {} } public void run() {} }; } }\nclass Base { public void run() {} }\n",
        ),
    ];
    for (label, source) in variants {
        let directory = scratch.child(label);
        fs::write(directory.join("AnonymousInit.java"), source).expect("write negative source");
        compile_java_8(&directory, "AnonymousInit.java", &directory);
        let root = fs::read(directory.join("AnonymousInit.class")).expect("read root");
        let child = fs::read(directory.join("AnonymousInit$1.class")).expect("read child");
        let base = fs::read(directory.join("Base.class")).expect("read Base");
        let mut entries = vec![
            (b"AnonymousInit.class".as_slice(), root.as_slice()),
            (b"AnonymousInit$1.class".as_slice(), child.as_slice()),
            (b"Base.class".as_slice(), base.as_slice()),
        ];
        let holder = directory.join("Holder.class");
        let holder_bytes = holder
            .exists()
            .then(|| fs::read(holder).expect("read Holder"));
        if let Some(bytes) = holder_bytes.as_ref() {
            entries.push((b"Holder.class", bytes));
        }
        let snapshot = open(zip_of(&entries));
        let report = class_source_of(&snapshot, "AnonymousInit", EnvironmentPolicy::PlainJar);
        assert!(
            report.text.contains("new AnonymousInit$1()"),
            "{label}: {}",
            report.text
        );
        assert!(
            !report.text.contains("new Base() {"),
            "{label}: {}",
            report.text
        );
    }
}

#[test]
fn anonymous_superclass_refuses_nested_parent_source_names_without_a_type_certificate() {
    let scratch = BridgeProjectionScratch::new();
    let directory = scratch.child("nested-parent-name");
    fs::write(
        directory.join("AnonymousSuperDirect.java"),
        "public class AnonymousSuperDirect { static Outer.Base make() { return new Outer.Base(1) { int sum() { return super.sum() + 1; } }; } }\nclass Outer { static class Base { Base(int value) {} int sum() { return 1; } } }\n",
    )
    .expect("write nested superclass source");
    compile_java_8(&directory, "AnonymousSuperDirect.java", &directory);
    let root = fs::read(directory.join("AnonymousSuperDirect.class")).expect("read root");
    let child = fs::read(directory.join("AnonymousSuperDirect$1.class")).expect("read child");
    let outer = fs::read(directory.join("Outer.class")).expect("read Outer");
    let parent = fs::read(directory.join("Outer$Base.class")).expect("read nested Base");
    let snapshot = open(zip_of(&[
        (b"AnonymousSuperDirect.class", &root),
        (b"AnonymousSuperDirect$1.class", &child),
        (b"Outer.class", &outer),
        (b"Outer$Base.class", &parent),
    ]));
    let report = class_source_of(
        &snapshot,
        "AnonymousSuperDirect",
        EnvironmentPolicy::PlainJar,
    );
    assert!(report.text.contains("new AnonymousSuperDirect$1("));
    assert!(!report.text.contains("new Outer$Base(1) {"));
}

#[test]
fn anonymous_superclass_refuses_cross_class_identity_use() {
    let snapshot = open(zip_of(&[
        (b"Owner.class", ANONYMOUS_SUPER_CROSS_OWNER),
        (b"Owner$1.class", ANONYMOUS_SUPER_CROSS_CHILD),
        (b"Base.class", ANONYMOUS_SUPER_CROSS_BASE),
        (b"Other.class", ANONYMOUS_SUPER_CROSS_OTHER),
    ]));
    let report = class_source_of(&snapshot, "Owner", EnvironmentPolicy::PlainJar);
    assert!(report.text.contains("new Owner$1()"), "{}", report.text);
    assert!(!report.text.contains("new Base() {"), "{}", report.text);
}

#[test]
fn anonymous_superclass_budget_and_cancellation_never_publish_partial_source() {
    let snapshot = open(zip_of(&[
        (
            "AnonymousSuperDirect.class".as_bytes(),
            ANONYMOUS_SUPER_DIRECT_ROOT,
        ),
        (
            "AnonymousSuperDirect$1.class".as_bytes(),
            ANONYMOUS_SUPER_DIRECT_CHILD,
        ),
        (b"Base.class", ANONYMOUS_SUPER_DIRECT_BASE),
    ]));
    let request = request(
        &snapshot,
        ClassRef::Name {
            class: ClassNameQuery::internal("AnonymousSuperDirect"),
        },
        EnvironmentPolicy::PlainJar,
    );
    let complete = performed(
        Engine::new()
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut budget(),
            )
            .expect("complete source is available"),
    );
    assert!(complete.text.contains("new Base(next(), next()) {"));

    let mut constrained = task_budget(&[BudgetOverride::new(
        "output_bytes",
        complete.usage.output_bytes.saturating_sub(1),
    )
    .expect("the output budget override is valid")])
    .expect("the constrained budget is valid");
    let stopped = performed(
        Engine::new()
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut constrained,
            )
            .expect("a budget stop preserves the class report"),
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
    assert!(stopped.text.contains("new AnonymousSuperDirect$1("));
    assert!(!stopped.text.contains("new Base(next(), next()) {"));

    let token = CancellationToken::new();
    token.cancel();
    let limits = task_budget(&[])
        .expect("default task limits are valid")
        .limits()
        .clone();
    let mut cancelled = Budget::with_cancellation_token(limits, token);
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("cancellation is a normal operation result")
    {
        OperationOutcome::Incomplete(selection) => assert!(matches!(
            selection.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                report.execution,
                ExecutionReport::Cancelled { .. }
            ));
            assert!(!report.text.contains("new Base(next(), next()) {"));
        }
        OperationOutcome::Ambiguous(_) => panic!("fixed class identity cannot be ambiguous"),
    }
}

#[test]
fn anonymous_superclass_refuses_missing_parent_or_incomplete_child_body() {
    let missing_parent = open(zip_of(&[
        (
            "AnonymousSuperDirect.class".as_bytes(),
            ANONYMOUS_SUPER_DIRECT_ROOT,
        ),
        (
            "AnonymousSuperDirect$1.class".as_bytes(),
            ANONYMOUS_SUPER_DIRECT_CHILD,
        ),
    ]));
    let missing = class_source_of(
        &missing_parent,
        "AnonymousSuperDirect",
        EnvironmentPolicy::PlainJar,
    );
    assert!(missing.text.contains("new AnonymousSuperDirect$1("));
    assert!(!missing.text.contains("new Base(next(), next()) {"));

    let mut incomplete_child = ANONYMOUS_SUPER_DIRECT_CHILD.to_vec();
    let sum_tail = [0x04, 0x60, 0xac];
    let at = incomplete_child
        .windows(sum_tail.len())
        .position(|window| window == sum_tail)
        .expect("the child implementation's final arithmetic sequence");
    incomplete_child[at] = 0xfe;
    let snapshot = open(zip_of(&[
        (
            "AnonymousSuperDirect.class".as_bytes(),
            ANONYMOUS_SUPER_DIRECT_ROOT,
        ),
        ("AnonymousSuperDirect$1.class".as_bytes(), &incomplete_child),
        (b"Base.class", ANONYMOUS_SUPER_DIRECT_BASE),
    ]));
    let report = class_source_of(
        &snapshot,
        "AnonymousSuperDirect",
        EnvironmentPolicy::PlainJar,
    );
    assert!(report.text.contains("new AnonymousSuperDirect$1("));
    assert!(!report.text.contains("new Base(next(), next()) {"));

    let child = class_source_of(
        &snapshot,
        "AnonymousSuperDirect$1",
        EnvironmentPolicy::PlainJar,
    );
    let sum = child
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"sum")
        .expect("the malformed child method remains physically reported");
    assert!(matches!(
        &sum.outcome,
        ClassSourceOutcome::Recovered { report, .. }
            if report.representation == jarde::ir::Representation::Mixed
    ));
}

#[test]
fn anonymous_interface_projection_refuses_a_second_non_direct_same_class_allocation() {
    let scratch = BridgeProjectionScratch::new();
    let directory = scratch.child("same-class-second-site");
    fs::write(directory.join("I.class"), ANONYMOUS_INTERFACE_API)
        .expect("write the frozen interface dependency");
    fs::write(
        directory.join("AnonymousInterfaceBasic.class"),
        ANONYMOUS_INTERFACE_ROOT,
    )
    .expect("write the enclosing root required by javac's InnerClasses validation");
    fs::write(
        directory.join("AnonymousInterfaceBasic$1.class"),
        ANONYMOUS_INTERFACE_CHILD,
    )
    .expect("write the frozen anonymous child");
    fs::write(
        directory.join("AnonymousInterfaceBasic.java"),
        "public class AnonymousInterfaceBasic {\n\
         static I make() { return new I() { public int value() { return 7; } }; }\n\
         static I extra() { I local = new AnonymousInterfaceBasic$1(); return local; }\n\
         }\n",
    )
    .expect("write a direct-return candidate plus a non-direct use of the same child");
    compile_java_8(&directory, "AnonymousInterfaceBasic.java", &directory);
    let root_bytes = fs::read(directory.join("AnonymousInterfaceBasic.class"))
        .expect("read the compiled root class");
    let child_bytes = fs::read(directory.join("AnonymousInterfaceBasic$1.class"))
        .expect("read the compiled anonymous child");
    assert_eq!(
        child_bytes, ANONYMOUS_INTERFACE_CHILD,
        "the compiled direct-return body must retain the frozen child's physical identity"
    );

    let snapshot = open(zip_of(&[
        (b"AnonymousInterfaceBasic.class", &root_bytes),
        (b"AnonymousInterfaceBasic$1.class", &child_bytes),
        (b"I.class", ANONYMOUS_INTERFACE_API),
    ]));
    let report = class_source_of(
        &snapshot,
        "AnonymousInterfaceBasic",
        EnvironmentPolicy::PlainJar,
    );
    assert!(
        report.text.contains("new AnonymousInterfaceBasic$1()"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("new I() {"), "{}", report.text);
}

#[test]
fn anonymous_interface_projection_refuses_a_cross_class_use_of_the_same_child() {
    let scratch = BridgeProjectionScratch::new();
    let directory = scratch.child("cross-class-use");
    fs::write(directory.join("I.class"), ANONYMOUS_INTERFACE_API)
        .expect("write the frozen interface dependency");
    fs::write(
        directory.join("AnonymousInterfaceBasic.class"),
        ANONYMOUS_INTERFACE_ROOT,
    )
    .expect("write the enclosing root required by javac's InnerClasses validation");
    fs::write(
        directory.join("AnonymousInterfaceBasic$1.class"),
        ANONYMOUS_INTERFACE_CHILD,
    )
    .expect("write the frozen anonymous child");
    fs::write(
        directory.join("Other.java"),
        "final class Other { static I extra() { return new AnonymousInterfaceBasic$1(); } }\n",
    )
    .expect("write a separate class that constructs the same child");
    compile_java_8(&directory, "Other.java", &directory);
    let other_bytes =
        fs::read(directory.join("Other.class")).expect("read the compiled cross-class user");
    let snapshot = open(zip_of(&[
        (b"AnonymousInterfaceBasic.class", ANONYMOUS_INTERFACE_ROOT),
        (
            b"AnonymousInterfaceBasic$1.class",
            ANONYMOUS_INTERFACE_CHILD,
        ),
        (b"I.class", ANONYMOUS_INTERFACE_API),
        (b"Other.class", &other_bytes),
    ]));
    let report = class_source_of(
        &snapshot,
        "AnonymousInterfaceBasic",
        EnvironmentPolicy::PlainJar,
    );
    assert!(
        report.text.contains("new AnonymousInterfaceBasic$1()"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("new I() {"), "{}", report.text);
}

#[test]
fn anonymous_interface_projection_refuses_fields_and_constructor_effects() {
    let scratch = BridgeProjectionScratch::new();
    let variants = [
        (
            "capture",
            "public class AnonymousInterfaceBasic { static I make(int captured) { return new I() { public int value() { return captured; } }; } }\n",
        ),
        (
            "field",
            "public class AnonymousInterfaceBasic { static I make() { return new I() { int state = 3; public int value() { return state; } }; } }\n",
        ),
        (
            "initializer-effect",
            "public class AnonymousInterfaceBasic { static I make() { return new I() { { System.nanoTime(); } public int value() { return 7; } }; } }\n",
        ),
    ];
    for (name, source) in variants {
        let directory = scratch.child(name);
        fs::write(directory.join("I.class"), ANONYMOUS_INTERFACE_API)
            .expect("write the frozen interface dependency");
        fs::write(directory.join("AnonymousInterfaceBasic.java"), source)
            .expect("write the anonymous implementation variant");
        compile_java_8(&directory, "AnonymousInterfaceBasic.java", &directory);
        let root_bytes = fs::read(directory.join("AnonymousInterfaceBasic.class"))
            .expect("read the compiled root class");
        let child_bytes = fs::read(directory.join("AnonymousInterfaceBasic$1.class"))
            .expect("read the compiled anonymous child");
        let snapshot = open(zip_of(&[
            (b"AnonymousInterfaceBasic.class", &root_bytes),
            (b"AnonymousInterfaceBasic$1.class", &child_bytes),
            (b"I.class", ANONYMOUS_INTERFACE_API),
        ]));
        let report = class_source_of(
            &snapshot,
            "AnonymousInterfaceBasic",
            EnvironmentPolicy::PlainJar,
        );
        assert!(
            report.text.contains("new AnonymousInterfaceBasic$1("),
            "{name}: {}",
            report.text
        );
        assert!(
            !report.text.contains("new I() {"),
            "{name}: {}",
            report.text
        );
    }
}

#[test]
fn anonymous_interface_projection_refuses_an_incomplete_child_method() {
    let mut child_bytes = ANONYMOUS_INTERFACE_CHILD.to_vec();
    let value = test_method_headers(ANONYMOUS_INTERFACE_CHILD)
        .into_iter()
        .find(|method| method.name == b"value" && method.descriptor == b"()I")
        .expect("the frozen anonymous child has value()");
    let code = value
        .attributes
        .iter()
        .find(|attribute| attribute.name == b"Code")
        .expect("value() has Code");
    let code_start = code.data_offset + 8;
    child_bytes[code_start] = 0xcb; // reserved opcode: the physical Code can no longer be fully recovered
    let snapshot = open(zip_of(&[
        (b"AnonymousInterfaceBasic.class", ANONYMOUS_INTERFACE_ROOT),
        (b"AnonymousInterfaceBasic$1.class", &child_bytes),
        (b"I.class", ANONYMOUS_INTERFACE_API),
    ]));
    let report = class_source_of(
        &snapshot,
        "AnonymousInterfaceBasic",
        EnvironmentPolicy::PlainJar,
    );
    assert!(
        report.text.contains("new AnonymousInterfaceBasic$1()"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("new I() {"), "{}", report.text);
}

#[test]
fn anonymous_interface_projection_refuses_non_source_child_method_flags() {
    let method = test_method_headers(ANONYMOUS_INTERFACE_CHILD)
        .into_iter()
        .find(|method| method.name == b"value" && method.descriptor == b"()I")
        .expect("the frozen child implements I.value()");
    for flags in [0x0009, 0x0041, 0x1001] {
        let mut child_bytes = ANONYMOUS_INTERFACE_CHILD.to_vec();
        test_put_u16(&mut child_bytes, method.access_offset, flags);
        let snapshot = open(zip_of(&[
            (b"AnonymousInterfaceBasic.class", ANONYMOUS_INTERFACE_ROOT),
            (b"AnonymousInterfaceBasic$1.class", &child_bytes),
            (b"I.class", ANONYMOUS_INTERFACE_API),
        ]));
        let root = class_source_of(
            &snapshot,
            "AnonymousInterfaceBasic",
            EnvironmentPolicy::PlainJar,
        );
        assert!(
            root.text.contains("new AnonymousInterfaceBasic$1()"),
            "flags={flags:#x}: {}",
            root.text
        );
        assert!(
            !root.text.contains("new I() {"),
            "flags={flags:#x}: {}",
            root.text
        );
    }
}

#[test]
fn anonymous_interface_projection_refuses_a_mixed_quality_child_method() {
    let scratch = BridgeProjectionScratch::new();
    let directory = scratch.child("mixed-quality-child");
    fs::write(
        directory.join("AnonymousInterfaceBasic.java"),
        "interface I { Target value(); }\n\
         class Target { Target(int value) {} }\n\
         class Side { static void effect() {} }\n\
         public class AnonymousInterfaceBasic { static I make() { return new I() {\n\
         public Target value() { return new Target(1); }\n\
         public void touch() { Side.effect(); }\
         }; } }\n",
    )
    .expect("write the mixed-quality anonymous source fixture");
    compile_java_8(&directory, "AnonymousInterfaceBasic.java", &directory);
    let root_bytes = fs::read(directory.join("AnonymousInterfaceBasic.class"))
        .expect("read the compiled root class");
    let mut child_bytes = fs::read(directory.join("AnonymousInterfaceBasic$1.class"))
        .expect("read the compiled anonymous child");
    let value = test_method_headers(&child_bytes)
        .into_iter()
        .find(|method| method.name == b"value" && method.descriptor == b"()LTarget;")
        .expect("the compiled child has value()");
    let code = value
        .attributes
        .iter()
        .find(|attribute| attribute.name == b"Code")
        .expect("value() has Code");
    let new_class = test_u16(&child_bytes, code.data_offset + 8 + 1);
    let effect = test_method_reference(&child_bytes, b"Side", b"effect", b"()V");
    let constructor = test_method_reference(&child_bytes, b"Target", b"<init>", b"(I)V");
    let code_start = code.data_offset + 8;
    let old_code_length = test_u32(&child_bytes, code.data_offset + 4);
    assert_eq!(old_code_length, 9, "javac's value() shape is stable");
    let mut code_body = vec![0xbb];
    u16b(
        &mut code_body,
        u16::try_from(new_class).expect("the class constant-pool index fits u16"),
    );
    code_body.push(0x59); // dup
    code_body.push(0xb8); // invokestatic Side.effect()V while the new value is unconsumed
    u16b(&mut code_body, effect);
    code_body.push(0x04); // iconst_1
    code_body.push(0xb7); // invokespecial Target.<init>(I)V
    u16b(&mut code_body, constructor);
    code_body.push(0xb0); // areturn
    assert_eq!(code_body.len(), 12);
    test_put_u16(&mut child_bytes, code.data_offset, 3);
    test_put_u32(&mut child_bytes, code.data_offset + 4, code_body.len());
    test_put_u32(
        &mut child_bytes,
        code.length_offset,
        code.length + code_body.len() - old_code_length,
    );
    child_bytes.splice(code_start..code_start + old_code_length, code_body);

    let interface_bytes = fs::read(directory.join("I.class")).expect("read I.class");
    let target_bytes = fs::read(directory.join("Target.class")).expect("read Target.class");
    let side_bytes = fs::read(directory.join("Side.class")).expect("read Side.class");
    let snapshot = open(zip_of(&[
        (b"AnonymousInterfaceBasic.class", &root_bytes),
        (b"AnonymousInterfaceBasic$1.class", &child_bytes),
        (b"I.class", &interface_bytes),
        (b"Target.class", &target_bytes),
        (b"Side.class", &side_bytes),
    ]));
    let child = class_source_of(
        &snapshot,
        "AnonymousInterfaceBasic$1",
        EnvironmentPolicy::PlainJar,
    );
    let value = child
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"value")
        .expect("the physical child method remains reportable");
    assert!(
        matches!(
            &value.outcome,
            ClassSourceOutcome::Recovered { report, .. }
                if report.produced()
                    && report.representation == jarde::ir::Representation::Mixed
                    && report.quality != jarde::ir::Quality::Structured
        ),
        "value outcome: {:?}",
        value.outcome
    );
    let root = class_source_of(
        &snapshot,
        "AnonymousInterfaceBasic",
        EnvironmentPolicy::PlainJar,
    );
    assert!(
        root.text.contains("new AnonymousInterfaceBasic$1()"),
        "{}",
        root.text
    );
    assert!(!root.text.contains("new I() {"), "{}", root.text);
}

#[test]
fn anonymous_interface_projection_budget_stop_keeps_the_original_root_text() {
    let snapshot = open(zip_of(&[
        (b"AnonymousInterfaceBasic.class", ANONYMOUS_INTERFACE_ROOT),
        (
            b"AnonymousInterfaceBasic$1.class",
            ANONYMOUS_INTERFACE_CHILD,
        ),
        (b"I.class", ANONYMOUS_INTERFACE_API),
    ]));
    let engine = Engine::new();
    let request = request(
        &snapshot,
        ClassRef::Name {
            class: ClassNameQuery::internal("AnonymousInterfaceBasic"),
        },
        EnvironmentPolicy::PlainJar,
    );
    let complete = performed(
        engine
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut budget(),
            )
            .expect("the unconstrained projection request is legal"),
    );
    assert!(complete.text.contains("new I() {"));

    let cap = complete.usage.output_bytes.saturating_sub(1);
    let mut constrained = task_budget(&[
        BudgetOverride::new("output_bytes", cap).expect("the output budget override is valid")
    ])
    .expect("the constrained budget is valid");
    let stopped = performed(
        engine
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut constrained,
            )
            .expect("a projection budget stop leaves the class report available"),
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
    assert!(
        stopped.text.contains("new AnonymousInterfaceBasic$1()"),
        "{}",
        stopped.text
    );
    assert!(!stopped.text.contains("new I() {"), "{}", stopped.text);
}

#[test]
fn anonymous_interface_projection_cancellation_never_publishes_partial_source() {
    let snapshot = open(zip_of(&[
        (b"AnonymousInterfaceBasic.class", ANONYMOUS_INTERFACE_ROOT),
        (
            b"AnonymousInterfaceBasic$1.class",
            ANONYMOUS_INTERFACE_CHILD,
        ),
        (b"I.class", ANONYMOUS_INTERFACE_API),
    ]));
    let request = request(
        &snapshot,
        ClassRef::Name {
            class: ClassNameQuery::internal("AnonymousInterfaceBasic"),
        },
        EnvironmentPolicy::PlainJar,
    );
    let token = CancellationToken::new();
    token.cancel();
    let limits = task_budget(&[])
        .expect("default task limits are valid")
        .limits()
        .clone();
    let mut cancelled = Budget::with_cancellation_token(limits, token);
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("cancellation is reported as a partial operation outcome");
    match outcome {
        OperationOutcome::Incomplete(selection) => assert!(matches!(
            selection.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                report.execution,
                ExecutionReport::Cancelled { .. }
            ));
            assert!(!report.text.contains("new I() {"), "{}", report.text);
        }
        OperationOutcome::Ambiguous(_) => panic!("one frozen class binds uniquely"),
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

fn compile_java_8(directory: &Path, source_name: &str, classpath: &Path) {
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-classpath"])
        .arg(classpath)
        .arg("-d")
        .arg(directory)
        .arg(directory.join(source_name))
        .output()
        .expect("JDK javac is available for Java 8 class-source negatives");
    assert!(
        compile.status.success(),
        "javac rejected the negative fixture:\n{}",
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
