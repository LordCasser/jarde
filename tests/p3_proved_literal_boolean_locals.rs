//! Literal-only local boolean recovery through the public whole-class facade.
use jarde::*;
use jarde_jvm::ir::{Quality, Representation};
use jarde_reader::budget::{Budget, CancellationToken};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use jarde_reader::model::ExecutionReport;
use std::{collections::BTreeSet, slice};

const INTERNAL: &str = "jadx/tests/integration/loops/TestBreakWithLabel$TestCls";
const ORIGINAL_DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/cf09-literal-boolean-locals/original/debug/classes/jadx/tests/integration/loops/TestBreakWithLabel$TestCls.class"
);
const ORIGINAL_DEBUG_JAR: &[u8] =
    include_bytes!("fixtures/cf09-literal-boolean-locals/original/debug-full-classes.jar");
const ORIGINAL_NO_DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/cf09-literal-boolean-locals/original/no-debug/classes/jadx/tests/integration/loops/TestBreakWithLabel$TestCls.class"
);
const ORIGINAL_NO_DEBUG_JAR: &[u8] =
    include_bytes!("fixtures/cf09-literal-boolean-locals/original/no-debug-full-classes.jar");
const REFUSALS_JAVAC8: &[u8] =
    include_bytes!("fixtures/cf09-literal-boolean-locals/controls/jdk8/BooleanLocalRefusals.class");
const REFUSALS_JAVAC23: &[u8] = include_bytes!(
    "fixtures/cf09-literal-boolean-locals/controls/jdk23/BooleanLocalRefusals.class"
);
const CONTROL: &str = "cf09/literalboolean/BooleanLocalRefusals";

fn budget() -> Budget {
    task_budget(&[]).expect("task limits are bounded")
}
fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("class or plain JAR opens")
}
fn source_request(
    snapshot: &ArtifactSnapshot,
    class: &str,
    policy: EnvironmentPolicy,
) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
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
fn reports(
    snapshot: &ArtifactSnapshot,
    class: &str,
    policy: EnvironmentPolicy,
) -> (ClassSourceReport, ClassSourceReport) {
    let request = source_request(snapshot, class, policy);
    let engine = Engine::new();
    let default = match engine
        .class_source(slice::from_ref(snapshot), &request, &mut budget())
        .expect("default class-source request answers")
    {
        OperationOutcome::Performed(r) => r,
        other => panic!("one selected class: {other:?}"),
    };
    let all = match engine
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("all-evidence class-source request answers")
    {
        OperationOutcome::Performed(r) => r,
        other => panic!("one selected class: {other:?}"),
    };
    (default, all)
}
fn method<'a>(
    report: &'a ClassSourceReport,
    name: &str,
    descriptor: &str,
) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|m| {
            m.item.name.raw().0 == name.as_bytes()
                && m.item.descriptor.raw().0 == descriptor.as_bytes()
        })
        .unwrap_or_else(|| panic!("missing method {name}{descriptor}"))
}
fn recovery<'a>(m: &'a ClassSourceMethod) -> &'a RecoveryReport {
    match &m.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("method was not analyzed: {other:?}"),
    }
}
fn assert_original_flag(report: &ClassSourceReport, bytes: &[u8], label: &str) {
    let m = method(report, "test", "([[II)Z");
    let body = recovery(m);
    assert_eq!(
        body.representation,
        Representation::Java,
        "{label}: {}",
        body.text
    );
    assert_eq!(body.quality, Quality::Structured, "{label}: {}", body.text);
    assert_eq!(body.content, RecoveryContent::ContainsStatements, "{label}");
    assert!(!body.text.contains("@bytecode"), "{label}: {}", body.text);
    let decl = body
        .text
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("boolean "))
        .unwrap_or_else(|| panic!("{label}: missing typed flag declaration:\n{}", body.text));
    let local = decl
        .strip_prefix("boolean ")
        .unwrap()
        .trim_end_matches(';')
        .split(" = ")
        .next()
        .unwrap()
        .trim();
    let initialized_inline = decl.ends_with(" = false;");
    let initialized_separately = body
        .text
        .lines()
        .map(str::trim)
        .any(|line| line == format!("{local} = false;"));
    assert!(
        initialized_inline || initialized_separately,
        "{label}: false producer is not presented as a Boolean assignment:\n{}",
        body.text
    );
    assert!(
        body.text.contains(&format!("{local} = true;")),
        "{label}: {}",
        body.text
    );
    let loop_label = body
        .text
        .lines()
        .map(str::trim)
        .find_map(|line| line.split_once(": for (").map(|(label, _)| label))
        .unwrap_or_else(|| panic!("{label}: missing labeled for loop:\n{}", body.text));
    assert!(
        !loop_label.is_empty() && body.text.contains(&format!("break {loop_label};")),
        "{label}: break does not target the recovered loop label:\n{}",
        body.text
    );
    assert!(
        body.text
            .contains(&format!("println(\"found: \" + {local})")),
        "{label}: {}",
        body.text
    );
    assert!(
        body.text.contains(&format!("return {local};")),
        "{label}: {}",
        body.text
    );
    let physical = inspect_method_bytecode(
        bytes,
        MethodSelector {
            name: JvmBytes(b"test".to_vec()),
            descriptor: JvmBytes(b"([[II)Z".to_vec()),
        },
        &mut budget(),
    )
    .expect("physical Code is readable");
    assert!(
        physical.stopped_at.is_none(),
        "{label}: complete physical Code scan"
    );
    let physical_bcis: BTreeSet<u32> = physical.instructions.iter().map(|ins| ins.bci).collect();
    let mut anchored = BTreeSet::new();
    for segment in body.source_map.segments() {
        assert!(
            body.text.get(segment.start()..segment.end()).is_some(),
            "{label}: source span is UTF-8 aligned"
        );
        for anchor in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            assert_eq!(
                anchor.method(),
                Some(&m.item.identity),
                "{label}: source anchor owner is the exact method"
            );
            assert!(
                physical_bcis.contains(&anchor.bci()),
                "{label}: anchor is a physical instruction"
            );
            anchored.insert(anchor.bci());
        }
    }
    assert!(
        !anchored.is_empty(),
        "{label}: expected physical method anchors"
    );
}

#[test]
fn original_flag_is_boolean_through_class_and_plain_jar_default_and_all_profiles() {
    for (profile, class_bytes, jar_bytes) in [
        ("debug", ORIGINAL_DEBUG_CLASS, ORIGINAL_DEBUG_JAR),
        ("no-debug", ORIGINAL_NO_DEBUG_CLASS, ORIGINAL_NO_DEBUG_JAR),
    ] {
        for (input, bytes, policy) in [
            ("class", class_bytes, EnvironmentPolicy::SingleClass),
            ("plain-jar", jar_bytes, EnvironmentPolicy::PlainJar),
        ] {
            let snapshot = open(bytes);
            let (default, all) = reports(&snapshot, INTERNAL, policy);
            let cell = format!("{profile}/{input}");
            assert_eq!(
                default.text, all.text,
                "{cell}: evidence selection changes whole-class text"
            );
            let d = method(&default, "test", "([[II)Z");
            let a = method(&all, "test", "([[II)Z");
            assert_eq!(
                d.item.identity, a.item.identity,
                "{cell}: method identity changed with evidence"
            );
            assert_eq!(
                recovery(d).source_map,
                recovery(a).source_map,
                "{cell}: method map changed with evidence"
            );
            assert_original_flag(&default, class_bytes, &format!("{cell}/default"));
            assert_original_flag(&all, class_bytes, &format!("{cell}/all"));
        }
    }
}

#[test]
fn refusal_controls_keep_integer_consumers_and_reject_the_non_boolean_store() {
    for (compiler, class_bytes) in [("javac8", REFUSALS_JAVAC8), ("javac23", REFUSALS_JAVAC23)] {
        let snapshot = open(class_bytes);
        let (default, all) = reports(&snapshot, CONTROL, EnvironmentPolicy::SingleClass);
        assert_eq!(
            default.text, all.text,
            "{compiler}: evidence changes control class text"
        );
        for report in [&default, &all] {
            let bad = method(report, "writeTwo", "(Z)Z");
            let bad_body = recovery(bad);
            assert_eq!(bad_body.quality, Quality::Fallback, "{compiler}");
            assert!(
                !bad_body.text.contains("boolean found"),
                "{compiler}: invalid 0/2 store was typed boolean:\n{}",
                bad_body.text
            );
            for (name, descriptor) in [("mixedIntegerSinks", "(Z)I"), ("branchOnly", "(I)I")] {
                let text = &recovery(method(report, name, descriptor)).text;
                assert!(
                    !text.contains("boolean found"),
                    "{compiler}/{name} promoted an integer-only use:\n{text}"
                );
                assert!(
                    text.contains("int found"),
                    "{compiler}/{name} keeps its integer local:\n{text}"
                );
            }
            let parameter = recovery(method(report, "parameterOnlyBranch", "(I)I"));
            assert!(parameter.text.contains("if (value != 0)"), "{compiler}");
            assert!(
                report.text.contains("int parameterOnlyBranch(int value)"),
                "{compiler}: the public signature retains the integer parameter"
            );
        }
    }
}

#[test]
fn pre_cancelled_public_class_source_does_not_publish_a_class_artifact() {
    let snapshot = open(ORIGINAL_DEBUG_CLASS);
    let request = source_request(&snapshot, INTERNAL, EnvironmentPolicy::SingleClass);
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(budget().limits().clone(), token);
    match Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut cancelled)
        .expect("cancellation is a public outcome")
    {
        OperationOutcome::Incomplete(report) => assert!(matches!(
            report.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                report.execution,
                ExecutionReport::Cancelled { .. }
            ));
            assert!(report.text.is_empty());
        }
        other => panic!("unexpected cancellation outcome: {other:?}"),
    }
}
