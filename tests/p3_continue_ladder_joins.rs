//! Public whole-class regressions for the proved continue-ladder join.
use jarde::*;
use jarde_jvm::ir::Quality;
use jarde_reader::budget::{Budget, CancellationToken};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use jarde_reader::model::ExecutionReport;
use std::collections::BTreeSet;
use std::slice;

const DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/recover-proved-loop-continue-ladder-joins/original/debug/classes/jadx/tests/integration/loops/TestContinueInLoop$TestCls.class"
);
const NO_DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/recover-proved-loop-continue-ladder-joins/original/no-debug/classes/jadx/tests/integration/loops/TestContinueInLoop$TestCls.class"
);
const DEBUG_JAR: &[u8] =
    include_bytes!("fixtures/recover-proved-loop-continue-ladder-joins/original/debug-plain.jar");
const NO_DEBUG_JAR: &[u8] = include_bytes!(
    "fixtures/recover-proved-loop-continue-ladder-joins/original/no-debug-plain.jar"
);
const OWNER: &str = "jadx/tests/integration/loops/TestContinueInLoop$TestCls";
const METHOD: &str = "test";
const DESCRIPTOR: &str = "([II)V";

struct Input {
    label: &'static str,
    bytes: &'static [u8],
    policy: EnvironmentPolicy,
    code_bytes: &'static [u8],
}

fn source(input: &Input, all: bool) -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("normal class-source budget is valid");
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(input.bytes.to_vec()), &mut budget)
        .unwrap_or_else(|error| panic!("{}: frozen input opens: {error:?}", input.label));
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(OWNER),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: input.policy.clone(),
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
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

fn target(report: &ClassSourceReport) -> &ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == METHOD.as_bytes()
                && method.item.descriptor.raw().0 == DESCRIPTOR.as_bytes()
        })
        .expect("the physical target method is in the complete class report")
}

fn physical_bcis(bytes: &[u8]) -> BTreeSet<u32> {
    let mut budget = task_budget(&[]).expect("reader task budget is valid");
    let code = inspect_method_bytecode(
        bytes,
        MethodSelector {
            name: JvmBytes(METHOD.as_bytes().to_vec()),
            descriptor: JvmBytes(DESCRIPTOR.as_bytes().to_vec()),
        },
        &mut budget,
    )
    .expect("the original target Code is inspectable");
    assert!(
        code.stopped_at.is_none(),
        "physical instruction scan is complete"
    );
    code.instructions
        .into_iter()
        .map(|instruction| instruction.bci)
        .collect()
}

fn assert_semantics(input: &Input, report: &ClassSourceReport) {
    assert_eq!(
        report.methods.len(),
        2,
        "{}: constructor and test",
        input.label
    );
    assert!(
        report
            .methods
            .iter()
            .any(|method| method.item.name.raw().0 == b"<init>"
                && method.item.descriptor.raw().0 == b"()V"),
        "{}: constructor remains in the full class report",
        input.label
    );
    assert_eq!(report.fields.len(), 1, "{}: public field f", input.label);
    let field = &report.fields[0].item;
    assert_eq!(field.name.raw().0, b"f");
    assert_eq!(field.descriptor.raw().0, b"I");

    let method = target(report);
    assert_eq!(method.item.identity.owner, report.class);
    assert_eq!(method.item.identity.name.0, METHOD.as_bytes());
    assert_eq!(method.item.identity.descriptor.0, DESCRIPTOR.as_bytes());
    let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
        panic!(
            "{}: target was not recovered: {:?}",
            input.label, method.outcome
        );
    };
    assert_eq!(
        body.quality,
        Quality::Structured,
        "{}:\n{}",
        input.label,
        body.text
    );
    assert!(
        !body.text.contains("@bytecode"),
        "{}:\n{}",
        input.label,
        body.text
    );

    // These source-level constraints capture the key distinction: the equality arm continues
    // before the later conditional break, while normal array-update paths reach that test; the
    // field effect remains after the loop. Identifier spelling may vary with debug metadata.
    let continue_at = body
        .text
        .find("continue;")
        .unwrap_or_else(|| panic!("{}: direct continue is absent:\n{}", input.label, body.text));
    let break_at = body.text.find("break;").unwrap_or_else(|| {
        panic!(
            "{}: conditional break is absent:\n{}",
            input.label, body.text
        )
    });
    let break_if_at = body.text[..break_at].rfind("if (").unwrap_or_else(|| {
        panic!(
            "{}: break is not under a condition:\n{}",
            input.label, body.text
        )
    });
    assert!(
        continue_at < break_if_at,
        "{}: continue must bypass the later break test:\n{}",
        input.label,
        body.text
    );
    let field_at = body.text.rfind(".f").unwrap_or_else(|| {
        panic!(
            "{}: post-loop field update is absent:\n{}",
            input.label, body.text
        )
    });
    assert!(
        break_at < field_at,
        "{}: field update follows the loop:\n{}",
        input.label,
        body.text
    );
    assert!(
        body.text.contains('['),
        "{}: array update is absent:\n{}",
        input.label,
        body.text
    );
    let before_break = &body.text[..break_if_at];
    assert!(
        before_break.lines().any(|line| line.contains('[')
            && (line.contains("++") || line.contains("+= 1") || line.contains("+ 1"))),
        "{}: incrementing array arm precedes the break test:\n{}",
        input.label,
        body.text
    );
    assert!(
        before_break.lines().any(|line| line.contains('[')
            && (line.contains("--") || line.contains("-= 1") || line.contains("- 1"))),
        "{}: decrementing array arm precedes the break test:\n{}",
        input.label,
        body.text
    );

    let physical = physical_bcis(input.code_bytes);
    assert!(
        !body.source_map.segments().is_empty(),
        "{}: target map is present",
        input.label
    );
    for segment in body.source_map.segments() {
        assert!(
            body.text.get(segment.start()..segment.end()).is_some(),
            "{}: valid UTF-8 map span",
            input.label
        );
        for anchor in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            assert_eq!(
                anchor.method(),
                Some(&method.item.identity),
                "{}: map owner",
                input.label
            );
            assert!(
                physical.contains(&anchor.bci()),
                "{}: BCI {} belongs to target Code",
                input.label,
                anchor.bci()
            );
        }
    }
}

#[test]
fn original_debug_no_debug_class_and_plain_jar_share_the_public_source_contract() {
    let inputs = [
        Input {
            label: "debug class",
            bytes: DEBUG_CLASS,
            policy: EnvironmentPolicy::SingleClass,
            code_bytes: DEBUG_CLASS,
        },
        Input {
            label: "debug plain jar",
            bytes: DEBUG_JAR,
            policy: EnvironmentPolicy::PlainJar,
            code_bytes: DEBUG_CLASS,
        },
        Input {
            label: "no-debug class",
            bytes: NO_DEBUG_CLASS,
            policy: EnvironmentPolicy::SingleClass,
            code_bytes: NO_DEBUG_CLASS,
        },
        Input {
            label: "no-debug plain jar",
            bytes: NO_DEBUG_JAR,
            policy: EnvironmentPolicy::PlainJar,
            code_bytes: NO_DEBUG_CLASS,
        },
    ];
    for input in inputs {
        let default = source(&input, false);
        let all = source(&input, true);
        assert_semantics(&input, &default);
        assert_semantics(&input, &all);
        assert_eq!(
            default.text, all.text,
            "{}: evidence does not change whole-class text",
            input.label
        );
        let (default_target, all_target) = (target(&default), target(&all));
        assert_eq!(
            default_target.text, all_target.text,
            "{}: method text",
            input.label
        );
        assert_eq!(
            default_target.item.identity, all_target.item.identity,
            "{}: physical identity",
            input.label
        );
        let (
            ClassSourceOutcome::Recovered {
                report: default_body,
                ..
            },
            ClassSourceOutcome::Recovered {
                report: all_body, ..
            },
        ) = (&default_target.outcome, &all_target.outcome)
        else {
            panic!("{}: both target outcomes remain recovered", input.label);
        };
        assert_eq!(
            default_body.source_map, all_body.source_map,
            "{}: evidence does not change the target map",
            input.label
        );
    }
}

#[test]
fn original_public_class_source_propagates_precancellation() {
    let mut open_budget = task_budget(&[]).expect("normal open budget is valid");
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(DEBUG_CLASS.to_vec()), &mut open_budget)
        .expect("original class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(OWNER),
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
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(open_budget.limits().clone(), token);
    let outcome = engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("pre-cancellation is returned as an execution outcome");
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
            assert!(
                report.text.is_empty(),
                "pre-cancellation publishes no class text"
            );
        }
        OperationOutcome::Ambiguous(_) => panic!("the original class is unique"),
    }
}
