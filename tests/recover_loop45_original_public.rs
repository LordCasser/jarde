//! Public whole-class regressions for the frozen original Loop4 and Loop5 fixtures.
use jarde::*;
use jarde_jvm::ir::Quality;
use jarde_reader::budget::{Budget, CancellationToken};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use jarde_reader::model::ExecutionReport;
use std::collections::BTreeSet;
use std::slice;

const L4_DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/recover-proved-loop-break-branch-bodies/original/debug/classes/jadx/tests/integration/loops/TestBreakInLoop4$TestCls.class"
);
const L4_NO_DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/recover-proved-loop-break-branch-bodies/original/no-debug/classes/jadx/tests/integration/loops/TestBreakInLoop4$TestCls.class"
);
const L4_DEBUG_JAR: &[u8] = include_bytes!(
    "fixtures/recover-proved-loop-break-branch-bodies/original/debug/TestBreakInLoop4-full-classes.jar"
);
const L4_NO_DEBUG_JAR: &[u8] = include_bytes!(
    "fixtures/recover-proved-loop-break-branch-bodies/original/no-debug/TestBreakInLoop4-full-classes.jar"
);
const L5_DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/recover-named-catch-loop-return-tails/original/debug/classes/jadx/tests/integration/loops/TestBreakInLoop5$TestCls.class"
);
const L5_NO_DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/recover-named-catch-loop-return-tails/original/no-debug/classes/jadx/tests/integration/loops/TestBreakInLoop5$TestCls.class"
);
const L5_DEBUG_JAR: &[u8] = include_bytes!(
    "fixtures/recover-named-catch-loop-return-tails/original/debug/TestBreakInLoop5-full-classes.jar"
);
const L5_NO_DEBUG_JAR: &[u8] = include_bytes!(
    "fixtures/recover-named-catch-loop-return-tails/original/no-debug/TestBreakInLoop5-full-classes.jar"
);

const L4_OWNER: &str = "jadx/tests/integration/loops/TestBreakInLoop4$TestCls";
const L5_OWNER: &str = "jadx/tests/integration/loops/TestBreakInLoop5$TestCls";
const L4_TEST_DESC: &str = "(C)D";
const L5_TEST_DESC: &str = "(Ljava/lang/String;)J";

#[derive(Clone)]
struct Input {
    label: &'static str,
    artifact: &'static [u8],
    target_class: &'static [u8],
    owner: &'static str,
    policy: EnvironmentPolicy,
    target_descriptor: &'static str,
}

fn request(snapshot: &ArtifactSnapshot, input: &Input) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(input.owner),
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
    }
}

fn source(input: &Input, all: bool) -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("bounded request budget");
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(input.artifact.to_vec()), &mut budget)
        .unwrap_or_else(|error| panic!("{}: frozen input opens: {error:?}", input.label));
    let request = request(&snapshot, input);
    let result = if all {
        engine.class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
    } else {
        engine.class_source(slice::from_ref(&snapshot), &request, &mut budget)
    }
    .unwrap_or_else(|error| panic!("{}: class source: {error:?}", input.label));
    match result {
        OperationOutcome::Performed(report) => report,
        other => panic!("{}: complete class source required: {other:?}", input.label),
    }
}

fn method<'a>(
    report: &'a ClassSourceReport,
    name: &str,
    descriptor: &str,
) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|member| {
            member.item.name.raw().0 == name.as_bytes()
                && member.item.descriptor.raw().0 == descriptor.as_bytes()
        })
        .unwrap_or_else(|| panic!("missing method {name}{descriptor}"))
}

fn method_keys(report: &ClassSourceReport) -> BTreeSet<(Vec<u8>, Vec<u8>)> {
    report
        .methods
        .iter()
        .map(|member| {
            (
                member.item.name.raw().0.clone(),
                member.item.descriptor.raw().0.clone(),
            )
        })
        .collect()
}

fn physical_bcis(bytes: &[u8], name: &str, descriptor: &str) -> BTreeSet<u32> {
    let mut budget = task_budget(&[]).expect("reader task budget");
    let facts = inspect_method_bytecode(
        bytes,
        MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        &mut budget,
    )
    .expect("frozen original Code is inspectable");
    assert!(
        facts.stopped_at.is_none(),
        "physical instruction scan is complete"
    );
    facts
        .instructions
        .into_iter()
        .map(|instruction| instruction.bci)
        .collect()
}

fn assert_source_map(input: &Input, member: &ClassSourceMethod, name: &str, descriptor: &str) {
    let ClassSourceOutcome::Recovered { report: body, .. } = &member.outcome else {
        panic!("{}: {name}{descriptor} is recovered", input.label);
    };
    let physical = physical_bcis(input.target_class, name, descriptor);
    assert!(
        !body.source_map.segments().is_empty(),
        "{}: source map exists",
        input.label
    );
    let mut mapped = BTreeSet::new();
    for segment in body.source_map.segments() {
        assert!(segment.start() <= segment.end());
        assert!(
            body.text.get(segment.start()..segment.end()).is_some(),
            "UTF-8 map boundary"
        );
        for anchor in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            assert_eq!(
                anchor.method(),
                Some(&member.item.identity),
                "map method identity"
            );
            assert!(
                physical.contains(&anchor.bci()),
                "BCI {} is physical in {name}{descriptor}",
                anchor.bci()
            );
            mapped.insert(anchor.bci());
        }
    }
    if name == "test" && input.owner == L4_OWNER {
        for bci in [26, 33] {
            assert!(
                physical.contains(&bci),
                "Loop4 BCI {bci} is physically present"
            );
            assert!(
                mapped.contains(&bci),
                "Loop4 branch provenance retains BCI {bci}"
            );
        }
    }
    if name == "test" && input.owner == L5_OWNER {
        assert!(
            physical.contains(&123),
            "Loop5 terminal return BCI 123 is physical"
        );
        assert!(
            mapped.contains(&123),
            "Loop5 source map retains terminal return BCI 123"
        );
    }
}

fn assert_full_class(input: &Input, report: &ClassSourceReport) {
    assert_eq!(
        report.methods.len(),
        3,
        "{}: ctor, test and check remain",
        input.label
    );
    assert!(
        report.fields.is_empty(),
        "{}: target has no fields",
        input.label
    );
    let expected = [
        (b"<init>".to_vec(), b"()V".to_vec()),
        (
            b"test".to_vec(),
            input.target_descriptor.as_bytes().to_vec(),
        ),
        (b"check".to_vec(), b"()V".to_vec()),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(
        method_keys(report),
        expected,
        "{}: physical method inventory",
        input.label
    );

    for (name, descriptor) in [
        ("<init>", "()V"),
        ("test", input.target_descriptor),
        ("check", "()V"),
    ] {
        let member = method(report, name, descriptor);
        assert_eq!(
            member.item.identity.owner, report.class,
            "{}: owner identity",
            input.label
        );
        let ClassSourceOutcome::Recovered { report: body, .. } = &member.outcome else {
            panic!("{}: {name}{descriptor} has a recovered body", input.label);
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
            "{}: no bytecode placeholder",
            input.label
        );
        assert_source_map(input, member, name, descriptor);
    }

    let ClassSourceOutcome::Recovered { report: body, .. } =
        &method(report, "test", input.target_descriptor).outcome
    else {
        unreachable!()
    };
    if input.owner == L4_OWNER {
        assert!(
            body.text.contains("break;"),
            "{}: expected conditional break effect\n{}",
            input.label,
            body.text
        );
    } else {
        for token in [
            "Character.isDigit",
            "charAt",
            "append",
            "break;",
            "Math.ceil",
            "Double.valueOf",
            "return -1",
        ] {
            assert!(
                body.text.contains(token),
                "{}: expected parse/return behavior {token:?}\n{}",
                input.label,
                body.text
            );
        }
        assert!(
            body.text.contains('*'),
            "{}: multiplier expression remains present\n{}",
            input.label,
            body.text
        );
    }
}

fn assert_default_all(input: &Input) {
    let default = source(input, false);
    let all = source(input, true);
    assert_full_class(input, &default);
    assert_full_class(input, &all);
    assert_eq!(
        default.text, all.text,
        "{}: whole-class text stable",
        input.label
    );
    for (name, descriptor) in [
        ("<init>", "()V"),
        ("test", input.target_descriptor),
        ("check", "()V"),
    ] {
        let (left, right) = (
            method(&default, name, descriptor),
            method(&all, name, descriptor),
        );
        assert_eq!(
            left.text, right.text,
            "{}: {name}{descriptor} text stable",
            input.label
        );
        assert_eq!(
            left.item.identity, right.item.identity,
            "{}: {name}{descriptor} identity stable",
            input.label
        );
        let (
            ClassSourceOutcome::Recovered { report: l, .. },
            ClassSourceOutcome::Recovered { report: r, .. },
        ) = (&left.outcome, &right.outcome)
        else {
            panic!(
                "{}: both evidence selections recover {name}{descriptor}",
                input.label
            );
        };
        assert_eq!(
            l.quality, r.quality,
            "{}: {name}{descriptor} quality stable",
            input.label
        );
        assert_eq!(
            l.source_map, r.source_map,
            "{}: {name}{descriptor} source map stable",
            input.label
        );
    }
}

fn loop4_inputs() -> [Input; 4] {
    [
        Input {
            label: "Loop4 debug class",
            artifact: L4_DEBUG_CLASS,
            target_class: L4_DEBUG_CLASS,
            owner: L4_OWNER,
            policy: EnvironmentPolicy::SingleClass,
            target_descriptor: L4_TEST_DESC,
        },
        Input {
            label: "Loop4 no-debug class",
            artifact: L4_NO_DEBUG_CLASS,
            target_class: L4_NO_DEBUG_CLASS,
            owner: L4_OWNER,
            policy: EnvironmentPolicy::SingleClass,
            target_descriptor: L4_TEST_DESC,
        },
        Input {
            label: "Loop4 debug full JAR",
            artifact: L4_DEBUG_JAR,
            target_class: L4_DEBUG_CLASS,
            owner: L4_OWNER,
            policy: EnvironmentPolicy::PlainJar,
            target_descriptor: L4_TEST_DESC,
        },
        Input {
            label: "Loop4 no-debug full JAR",
            artifact: L4_NO_DEBUG_JAR,
            target_class: L4_NO_DEBUG_CLASS,
            owner: L4_OWNER,
            policy: EnvironmentPolicy::PlainJar,
            target_descriptor: L4_TEST_DESC,
        },
    ]
}

fn loop5_inputs() -> [Input; 4] {
    [
        Input {
            label: "Loop5 debug class",
            artifact: L5_DEBUG_CLASS,
            target_class: L5_DEBUG_CLASS,
            owner: L5_OWNER,
            policy: EnvironmentPolicy::SingleClass,
            target_descriptor: L5_TEST_DESC,
        },
        Input {
            label: "Loop5 no-debug class",
            artifact: L5_NO_DEBUG_CLASS,
            target_class: L5_NO_DEBUG_CLASS,
            owner: L5_OWNER,
            policy: EnvironmentPolicy::SingleClass,
            target_descriptor: L5_TEST_DESC,
        },
        Input {
            label: "Loop5 debug full JAR",
            artifact: L5_DEBUG_JAR,
            target_class: L5_DEBUG_CLASS,
            owner: L5_OWNER,
            policy: EnvironmentPolicy::PlainJar,
            target_descriptor: L5_TEST_DESC,
        },
        Input {
            label: "Loop5 no-debug full JAR",
            artifact: L5_NO_DEBUG_JAR,
            target_class: L5_NO_DEBUG_CLASS,
            owner: L5_OWNER,
            policy: EnvironmentPolicy::PlainJar,
            target_descriptor: L5_TEST_DESC,
        },
    ]
}

#[test]
fn original_loop4_debug_and_no_debug_class_and_full_jar_keep_structured_conditional_break_body() {
    for input in loop4_inputs() {
        assert_default_all(&input);
    }
}

#[test]
fn original_loop4_public_class_source_propagates_precancellation_without_partial_text() {
    for input in loop4_inputs() {
        for all in [false, true] {
            assert_precancel(&input, all);
        }
    }
}

#[test]
fn original_loop5_debug_and_no_debug_class_and_full_jar_keep_named_catch_return_tail() {
    for input in loop5_inputs() {
        assert_default_all(&input);
    }
}

#[test]
fn original_loop5_public_class_source_propagates_precancellation_without_partial_text() {
    for input in loop5_inputs() {
        for all in [false, true] {
            assert_precancel(&input, all);
        }
    }
}

fn assert_precancel(input: &Input, all: bool) {
    let engine = Engine::new();
    let mut open_budget = task_budget(&[]).expect("normal input-open budget");
    let snapshot = engine
        .open(
            ArtifactInput::bytes(input.artifact.to_vec()),
            &mut open_budget,
        )
        .unwrap_or_else(|error| {
            panic!(
                "{}: input opens before cancellation: {error:?}",
                input.label
            )
        });
    let request = request(&snapshot, input);
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(open_budget.limits().clone(), token);
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
    .unwrap_or_else(|error| panic!("{}: pre-cancel outcome: {error:?}", input.label));
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
                "{}: no partial class text",
                input.label
            );
        }
        OperationOutcome::Ambiguous(_) => panic!("{}: frozen class is unique", input.label),
    }
}
