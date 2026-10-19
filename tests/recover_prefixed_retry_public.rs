//! Public whole-class regressions for the frozen original Loop3 class and full-JAR inputs.
use jarde::*;
use jarde_jvm::ir::Quality;
use jarde_reader::budget::{Budget, CancellationToken};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use jarde_reader::model::ExecutionReport;
use std::collections::BTreeSet;
use std::slice;

const DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/recover-proved-prefixed-catch-retry-loops/original/debug/classes/jadx/tests/integration/loops/TestBreakInLoop3$TestCls.class"
);
const NO_DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/recover-proved-prefixed-catch-retry-loops/original/no-debug/classes/jadx/tests/integration/loops/TestBreakInLoop3$TestCls.class"
);
const DEBUG_JAR: &[u8] = include_bytes!(
    "fixtures/recover-proved-prefixed-catch-retry-loops/original/debug/TestBreakInLoop3-full-classes.jar"
);
const NO_DEBUG_JAR: &[u8] = include_bytes!(
    "fixtures/recover-proved-prefixed-catch-retry-loops/original/no-debug/TestBreakInLoop3-full-classes.jar"
);
const OWNER: &str = "jadx/tests/integration/loops/TestBreakInLoop3$TestCls";
const WRITE_MORE_DESC: &str = "(Ljava/lang/String;)V";

#[derive(Clone)]
struct Input {
    label: &'static str,
    artifact: &'static [u8],
    target_class: &'static [u8],
    policy: EnvironmentPolicy,
}

fn request(snapshot: &ArtifactSnapshot, input: &Input) -> ClassSourceRequest {
    ClassSourceRequest {
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

fn assert_source_map(input: &Input, member: &ClassSourceMethod) {
    let name_bytes = member.item.name.raw();
    let descriptor_bytes = member.item.descriptor.raw();
    let name = std::str::from_utf8(&name_bytes.0).expect("method name is UTF-8");
    let descriptor = std::str::from_utf8(&descriptor_bytes.0).expect("descriptor is UTF-8");
    let ClassSourceOutcome::Recovered { report: body, .. } = &member.outcome else {
        panic!("{}: {name}{descriptor} is recovered", input.label);
    };
    let physical = physical_bcis(input.target_class, name, descriptor);
    assert!(
        !body.source_map.segments().is_empty(),
        "{}: source map exists",
        input.label
    );
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
        }
    }
}

fn assert_full_class(input: &Input, report: &ClassSourceReport) {
    assert_eq!(
        report.methods.len(),
        4,
        "{}: ctor, writeMore, prt and check remain",
        input.label
    );
    assert_eq!(
        report.fields.len(),
        1,
        "{}: only the builder field remains",
        input.label
    );
    let expected = [
        (b"<init>".to_vec(), b"()V".to_vec()),
        (b"writeMore".to_vec(), WRITE_MORE_DESC.as_bytes().to_vec()),
        (b"prt".to_vec(), b"(Ljava/lang/String;)V".to_vec()),
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
        ("writeMore", WRITE_MORE_DESC),
        ("prt", "(Ljava/lang/String;)V"),
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
        assert_source_map(input, member);
    }
    let ClassSourceOutcome::Recovered { report: body, .. } =
        &method(report, "writeMore", WRITE_MORE_DESC).outcome
    else {
        unreachable!()
    };
    for token in ["FileOutputStream", "exists", "continue;", "break;", "prt"] {
        assert!(
            body.text.contains(token),
            "{}: expected effect {token:?}\n{}",
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
        ("writeMore", WRITE_MORE_DESC),
        ("prt", "(Ljava/lang/String;)V"),
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
            "{}: identity stable",
            input.label
        );
        let (
            ClassSourceOutcome::Recovered { report: l, .. },
            ClassSourceOutcome::Recovered { report: r, .. },
        ) = (&left.outcome, &right.outcome)
        else {
            panic!("both evidence selections recover {name}{descriptor}")
        };
        assert_eq!(l.quality, r.quality, "{}: quality stable", input.label);
        assert_eq!(
            l.source_map, r.source_map,
            "{}: source map stable",
            input.label
        );
    }
}

fn inputs() -> [Input; 4] {
    [
        Input {
            label: "Loop3 debug class",
            artifact: DEBUG_CLASS,
            target_class: DEBUG_CLASS,
            policy: EnvironmentPolicy::SingleClass,
        },
        Input {
            label: "Loop3 no-debug class",
            artifact: NO_DEBUG_CLASS,
            target_class: NO_DEBUG_CLASS,
            policy: EnvironmentPolicy::SingleClass,
        },
        Input {
            label: "Loop3 debug full JAR",
            artifact: DEBUG_JAR,
            target_class: DEBUG_CLASS,
            policy: EnvironmentPolicy::PlainJar,
        },
        Input {
            label: "Loop3 no-debug full JAR",
            artifact: NO_DEBUG_JAR,
            target_class: NO_DEBUG_CLASS,
            policy: EnvironmentPolicy::PlainJar,
        },
    ]
}

#[test]
fn original_loop3_debug_no_debug_class_and_full_jar_are_fully_structured_and_stable() {
    for input in inputs() {
        assert_default_all(&input);
    }
}

#[test]
fn original_loop3_class_source_precancellation_never_publishes_partial_text() {
    for input in inputs() {
        for all in [false, true] {
            let engine = Engine::new();
            let mut open_budget = task_budget(&[]).expect("normal input-open budget");
            let snapshot = engine
                .open(
                    ArtifactInput::bytes(input.artifact.to_vec()),
                    &mut open_budget,
                )
                .unwrap_or_else(|error| {
                    panic!("{}: opens before cancellation: {error:?}", input.label)
                });
            let request = request(&snapshot, &input);
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
    }
}
