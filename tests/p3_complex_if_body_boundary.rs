//! Public whole-class regression for an effectful body condition after a proved loop header.
use jarde::*;
use jarde_jvm::ir::Quality;
use jarde_reader::budget::{Budget, BudgetDimension, CancellationToken};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use jarde_reader::model::{ExecutionReport, TerminationReason};
use std::collections::BTreeSet;
use std::slice;

const DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/cf09-complex-if-body-boundary/debug/class/jadx/tests/integration/loops/TestBreakInComplexIf$TestCls.class"
);
const NO_DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/cf09-complex-if-body-boundary/no-debug/class/jadx/tests/integration/loops/TestBreakInComplexIf$TestCls.class"
);
const DEBUG_JAR: &[u8] = include_bytes!("fixtures/cf09-complex-if-body-boundary/debug-plain.jar");
const NO_DEBUG_JAR: &[u8] =
    include_bytes!("fixtures/cf09-complex-if-body-boundary/no-debug-plain.jar");
const OWNER: &str = "jadx/tests/integration/loops/TestBreakInComplexIf$TestCls";
const TEST_METHOD: &str = "test";
const TEST_DESCRIPTOR: &str = "(Ljava/util/Map;I)I";

struct Input {
    label: &'static str,
    bytes: &'static [u8],
    target_class: &'static [u8],
    policy: EnvironmentPolicy,
}

fn request(snapshot: &ArtifactSnapshot, policy: EnvironmentPolicy) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(OWNER),
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

fn source(input: &Input, all: bool) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("normal class-source budget is valid");
    let snapshot = engine
        .open(ArtifactInput::bytes(input.bytes.to_vec()), &mut budget)
        .unwrap_or_else(|error| panic!("{}: frozen input opens: {error:?}", input.label));
    let request = request(&snapshot, input.policy.clone());
    let outcome = if all {
        engine.class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
    } else {
        engine.class_source(slice::from_ref(&snapshot), &request, &mut budget)
    }
    .unwrap_or_else(|error| panic!("{}: class-source answers: {error:?}", input.label));
    match outcome {
        OperationOutcome::Performed(report) => report,
        other => panic!(
            "{}: complete class-source report expected: {other:?}",
            input.label
        ),
    }
}

fn member<'a>(
    report: &'a ClassSourceReport,
    name: &str,
    descriptor: &str,
) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == name.as_bytes()
                && method.item.descriptor.raw().0 == descriptor.as_bytes()
        })
        .unwrap_or_else(|| panic!("missing physical method {name}{descriptor}"))
}

fn physical_bcis(bytes: &[u8], name: &str, descriptor: &str) -> BTreeSet<u32> {
    let mut budget = task_budget(&[]).expect("reader task budget is valid");
    let code = inspect_method_bytecode(
        bytes,
        MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        &mut budget,
    )
    .expect("the original physical Code is inspectable");
    assert!(
        code.stopped_at.is_none(),
        "physical instruction scan is complete"
    );
    code.instructions
        .into_iter()
        .map(|instruction| instruction.bci)
        .collect()
}

fn method_keys(report: &ClassSourceReport) -> BTreeSet<(Vec<u8>, Vec<u8>)> {
    report
        .methods
        .iter()
        .map(|method| {
            (
                method.item.name.raw().0.clone(),
                method.item.descriptor.raw().0.clone(),
            )
        })
        .collect()
}

fn assert_method_map(input: &Input, method: &ClassSourceMethod, name: &str, descriptor: &str) {
    let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
        panic!(
            "{}: method must have a recovery report: {:?}",
            input.label, method.outcome
        );
    };
    let physical = physical_bcis(input.target_class, name, descriptor);
    assert!(
        !body.source_map.segments().is_empty(),
        "{}: method source map is present",
        input.label
    );
    for segment in body.source_map.segments() {
        assert!(
            segment.start() <= segment.end(),
            "{}: ordered UTF-8 span",
            input.label
        );
        assert!(
            body.text.get(segment.start()..segment.end()).is_some(),
            "{}: UTF-8 aligned span",
            input.label
        );
        for anchor in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            assert_eq!(
                anchor.method(),
                Some(&method.item.identity),
                "{}: exact method origin",
                input.label
            );
            assert!(
                physical.contains(&anchor.bci()),
                "{}: BCI {} belongs to physical Code",
                input.label,
                anchor.bci()
            );
        }
    }
}

fn assert_target_contract(input: &Input, report: &ClassSourceReport) {
    assert_eq!(
        report.methods.len(),
        3,
        "{}: constructor, target, and check remain",
        input.label
    );
    assert!(
        report.fields.is_empty(),
        "{}: target has no fields",
        input.label
    );
    let expected = [
        (b"<init>".to_vec(), b"()V".to_vec()),
        (b"test".to_vec(), b"(Ljava/util/Map;I)I".to_vec()),
        (b"check".to_vec(), b"()V".to_vec()),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(
        method_keys(report),
        expected,
        "{}: full target method inventory",
        input.label
    );

    for (name, descriptor) in [
        ("<init>", "()V"),
        (TEST_METHOD, TEST_DESCRIPTOR),
        ("check", "()V"),
    ] {
        let method = member(report, name, descriptor);
        assert_eq!(
            method.item.identity.owner, report.class,
            "{}: member belongs to target",
            input.label
        );
        let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
            panic!(
                "{}: every original target body remains available: {:?}",
                input.label, method.outcome
            );
        };
        assert_eq!(
            body.quality,
            Quality::Structured,
            "{}: {name}{descriptor}\n{}",
            input.label,
            body.text
        );
        assert!(
            !body.text.contains("@bytecode"),
            "{}: source is structured, not quoted",
            input.label
        );
        assert_method_map(input, method, name, descriptor);
    }

    let test = member(report, TEST_METHOD, TEST_DESCRIPTOR);
    let ClassSourceOutcome::Recovered { report: body, .. } = &test.outcome else {
        unreachable!("the target member was checked above")
    };
    assert_eq!(test.item.identity.owner, report.class);
    assert_eq!(test.item.identity.name.0.as_slice(), TEST_METHOD.as_bytes());
    assert_eq!(
        test.item.identity.descriptor.0.as_slice(),
        TEST_DESCRIPTOR.as_bytes()
    );

    // These observable source facts check the effect boundary without requiring a particular
    // equivalent `for`/`while` spelling or debug-local names.
    assert_eq!(
        body.text.matches(".get(").count(),
        1,
        "{}: one lookup expression per iteration\n{}",
        input.label,
        body.text
    );
    assert!(
        body.text.contains("null"),
        "{}: null guard remains\n{}",
        input.label,
        body.text
    );
    assert!(
        body.text.contains(".y"),
        "{}: guarded Point field read remains\n{}",
        input.label,
        body.text
    );
    assert!(
        body.text.contains("100"),
        "{}: counted loop bound remains\n{}",
        input.label,
        body.text
    );
    let null_at = body.text.find("null").expect("checked above");
    let field_at = body.text.find(".y").expect("checked above");
    assert!(
        null_at < field_at,
        "{}: null condition precedes the field read\n{}",
        input.label,
        body.text
    );
    let break_at = body
        .text
        .find("break")
        .unwrap_or_else(|| panic!("{}: loop exit remains\n{}", input.label, body.text));
    assert!(
        body.text.contains("return "),
        "{}: result return remains\n{}",
        input.label,
        body.text
    );
    let after_break = &body.text[break_at..];
    assert!(
        after_break.contains("++")
            || after_break.contains("+= 1")
            || (after_break.contains(" = ") && after_break.contains(" + 1")),
        "{}: length update remains after the break arm\n{}",
        input.label,
        body.text
    );
}

fn assert_budget_stop_without_artifact(input_bytes: &'static [u8], policy: EnvironmentPolicy) {
    let engine = Engine::new();
    let mut open_budget = task_budget(&[]).expect("snapshot-open budget is valid");
    let snapshot = engine
        .open(ArtifactInput::bytes(input_bytes.to_vec()), &mut open_budget)
        .expect("frozen source input opens");
    let request = request(&snapshot, policy);
    let mut budget =
        task_budget(&[BudgetOverride::new("output_bytes", 1).expect("positive output cap")])
            .expect("bounded task budget is valid");
    let outcome = engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("budget exhaustion is an operation outcome");
    match outcome {
        OperationOutcome::Incomplete(selection) => assert!(
            matches!(
                &selection.execution,
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded {
                        dimension: BudgetDimension::OutputBytes
                    },
                    ..
                }
            ),
            "budget stop is explicit: {:?}",
            selection.execution
        ),
        OperationOutcome::Performed(report) => {
            assert!(
                matches!(
                    &report.execution,
                    ExecutionReport::Partial {
                        reason: TerminationReason::BudgetExceeded {
                            dimension: BudgetDimension::OutputBytes
                        },
                        ..
                    }
                ),
                "budget stop is explicit: {:?}",
                report.execution
            );
            assert!(
                report.text.is_empty(),
                "stopped request publishes no partial class text"
            );
            for method in &report.methods {
                if let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome {
                    assert!(
                        body.text.is_empty() && body.source_map.is_empty(),
                        "no partial method body or map"
                    );
                }
            }
        }
        OperationOutcome::Ambiguous(_) => panic!("the frozen input has one target definition"),
    }
}

#[test]
fn original_debug_and_no_debug_class_and_plain_jar_keep_the_effectful_body_boundary() {
    let inputs = [
        Input {
            label: "debug class",
            bytes: DEBUG_CLASS,
            target_class: DEBUG_CLASS,
            policy: EnvironmentPolicy::SingleClass,
        },
        Input {
            label: "debug plain jar",
            bytes: DEBUG_JAR,
            target_class: DEBUG_CLASS,
            policy: EnvironmentPolicy::PlainJar,
        },
        Input {
            label: "no-debug class",
            bytes: NO_DEBUG_CLASS,
            target_class: NO_DEBUG_CLASS,
            policy: EnvironmentPolicy::SingleClass,
        },
        Input {
            label: "no-debug plain jar",
            bytes: NO_DEBUG_JAR,
            target_class: NO_DEBUG_CLASS,
            policy: EnvironmentPolicy::PlainJar,
        },
    ];
    for input in inputs {
        let default = source(&input, false);
        let all = source(&input, true);
        assert_target_contract(&input, &default);
        assert_target_contract(&input, &all);
        assert_eq!(
            default.text, all.text,
            "{}: optional evidence does not change whole-class text",
            input.label
        );
        for (name, descriptor) in [
            ("<init>", "()V"),
            (TEST_METHOD, TEST_DESCRIPTOR),
            ("check", "()V"),
        ] {
            let (left, right) = (
                member(&default, name, descriptor),
                member(&all, name, descriptor),
            );
            assert_eq!(
                left.text, right.text,
                "{}: {name}{descriptor} text",
                input.label
            );
            assert_eq!(
                left.item.identity, right.item.identity,
                "{}: {name}{descriptor} identity",
                input.label
            );
            let (
                ClassSourceOutcome::Recovered {
                    report: left_body, ..
                },
                ClassSourceOutcome::Recovered {
                    report: right_body, ..
                },
            ) = (&left.outcome, &right.outcome)
            else {
                panic!("{}: both runs retain member outcomes", input.label);
            };
            assert_eq!(
                left_body.source_map, right_body.source_map,
                "{}: {name}{descriptor} origins",
                input.label
            );
        }
    }
}

#[test]
fn low_output_budget_and_precancellation_publish_no_partial_class_artifact() {
    assert_budget_stop_without_artifact(DEBUG_CLASS, EnvironmentPolicy::SingleClass);

    let engine = Engine::new();
    let mut open_budget = task_budget(&[]).expect("snapshot-open budget is valid");
    let snapshot = engine
        .open(ArtifactInput::bytes(DEBUG_CLASS.to_vec()), &mut open_budget)
        .expect("original class opens");
    let request = request(&snapshot, EnvironmentPolicy::SingleClass);
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(open_budget.limits().clone(), token);
    let outcome = engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("pre-cancellation is an execution outcome");
    match outcome {
        OperationOutcome::Incomplete(selection) => assert!(matches!(
            &selection.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                &report.execution,
                ExecutionReport::Cancelled { .. }
            ));
            assert!(
                report.text.is_empty(),
                "pre-cancelled request publishes no class text"
            );
            for method in &report.methods {
                if let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome {
                    assert!(
                        body.text.is_empty() && body.source_map.is_empty(),
                        "pre-cancelled request publishes no partial body or map"
                    );
                }
            }
        }
        OperationOutcome::Ambiguous(_) => panic!("the frozen input has one target definition"),
    }
}
