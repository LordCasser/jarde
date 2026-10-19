//! Whole-class public API acceptance for the frozen DoWhileBreak2 fixture.
use jarde::*;
use jarde_jvm::ir::Quality;
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use std::collections::BTreeSet;
use std::slice;

const DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/recover-proved-post-test-loop-return-leaves/original/debug/classes/jadx/tests/integration/loops/TestDoWhileBreak2$TestCls.class"
);
const NO_DEBUG_CLASS: &[u8] = include_bytes!(
    "fixtures/recover-proved-post-test-loop-return-leaves/original/no-debug/classes/jadx/tests/integration/loops/TestDoWhileBreak2$TestCls.class"
);
const DEBUG_JAR: &[u8] = include_bytes!(
    "fixtures/recover-proved-post-test-loop-return-leaves/original/debug/TestDoWhileBreak2-full-classes.jar"
);
const NO_DEBUG_JAR: &[u8] = include_bytes!(
    "fixtures/recover-proved-post-test-loop-return-leaves/original/no-debug/TestDoWhileBreak2-full-classes.jar"
);
const OWNER: &str = "jadx/tests/integration/loops/TestDoWhileBreak2$TestCls";
const DESCRIPTOR: &str = "()Ljava/lang/Object;";

type MemberKey = (Vec<u8>, Vec<u8>);

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

fn method_keys(report: &ClassSourceReport) -> BTreeSet<MemberKey> {
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
    assert!(facts.stopped_at.is_none(), "physical scan is complete");
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
        "{}: {name}{descriptor} map exists",
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
    if name == "test" {
        for bci in [0, 17, 19] {
            assert!(physical.contains(&bci), "BCI {bci} is physical");
            assert!(mapped.contains(&bci), "BCI {bci} retains source provenance");
        }
    }
}

fn assert_full_class(input: &Input, report: &ClassSourceReport) {
    let expected_methods = [
        (b"<init>".to_vec(), b"()V".to_vec()),
        (b"test".to_vec(), DESCRIPTOR.as_bytes().to_vec()),
        (b"check".to_vec(), b"()V".to_vec()),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert_eq!(
        report.fields.len(),
        1,
        "{}: complete physical field inventory",
        input.label
    );
    let field = &report.fields[0].item;
    assert_eq!(field.name.raw().0, b"it", "{}: raw field name", input.label);
    assert_eq!(
        field.descriptor.raw().0,
        b"Ljava/util/Iterator;",
        "{}: raw field descriptor",
        input.label
    );
    assert_eq!(
        field.access_flags, 0,
        "{}: package-private instance field",
        input.label
    );
    assert_eq!(
        report.methods.len(),
        3,
        "{}: ctor, test, check remain",
        input.label
    );
    assert_eq!(
        method_keys(report),
        expected_methods,
        "{}: physical method inventory",
        input.label
    );
    for (name, descriptor) in [("<init>", "()V"), ("test", DESCRIPTOR), ("check", "()V")] {
        let member = method(report, name, descriptor);
        assert_eq!(
            member.item.identity.owner, report.class,
            "{}: member owner",
            input.label
        );
        let ClassSourceOutcome::Recovered { report: body, .. } = &member.outcome else {
            panic!("{}: {name}{descriptor} is a recovered body", input.label);
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
            "{}: {name}{descriptor} has no fallback",
            input.label
        );
        assert_source_map(input, member, name, descriptor);
    }
    let ClassSourceOutcome::Recovered { report: body, .. } =
        &method(report, "test", DESCRIPTOR).outcome
    else {
        unreachable!()
    };
    assert!(
        body.text.contains("return "),
        "{}: target keeps its return behavior\n{}",
        input.label,
        body.text
    );
    // The raw Iterator field's existing generic-display diagnostic is independent and permitted.
}

fn assert_default_all(input: &Input) {
    let default = source(input, false);
    let all = source(input, true);
    assert_full_class(input, &default);
    assert_full_class(input, &all);
    assert_eq!(default.text, all.text, "{}: class text stable", input.label);
    for (name, descriptor) in [("<init>", "()V"), ("test", DESCRIPTOR), ("check", "()V")] {
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
                "{}: both evidence requests recover {name}{descriptor}",
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
            "{}: {name}{descriptor} map stable",
            input.label
        );
    }
}

#[test]
fn original_dowhilebreak2_debug_no_debug_class_and_full_jar_default_all_are_structured() {
    let inputs = [
        Input {
            label: "Do2 debug class",
            artifact: DEBUG_CLASS,
            target_class: DEBUG_CLASS,
            policy: EnvironmentPolicy::SingleClass,
        },
        Input {
            label: "Do2 no-debug class",
            artifact: NO_DEBUG_CLASS,
            target_class: NO_DEBUG_CLASS,
            policy: EnvironmentPolicy::SingleClass,
        },
        Input {
            label: "Do2 debug full JAR",
            artifact: DEBUG_JAR,
            target_class: DEBUG_CLASS,
            policy: EnvironmentPolicy::PlainJar,
        },
        Input {
            label: "Do2 no-debug full JAR",
            artifact: NO_DEBUG_JAR,
            target_class: NO_DEBUG_CLASS,
            policy: EnvironmentPolicy::PlainJar,
        },
    ];
    for input in inputs {
        assert_default_all(&input);
    }
}
