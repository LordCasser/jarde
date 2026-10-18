//! A proved `char` assignment in a loop condition widens to its existing `int` local.
use jarde::*;
use jarde_jvm::ir::Representation;
use jarde_reader::budget::{Budget, CancellationToken};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use jarde_reader::model::ExecutionReport;
use std::collections::BTreeSet;
use std::slice;

const UPSTREAM_DEBUG: &[u8] = include_bytes!(
    "fixtures/java8/assignment-condition-widening/upstream-debug/TestLoopConditionInvoke$TestCls.class"
);
const UPSTREAM_NODEBUG: &[u8] = include_bytes!(
    "fixtures/java8/assignment-condition-widening/upstream-no-debug/TestLoopConditionInvoke$TestCls.class"
);
const CONTROL_JAVAC8_DEBUG: &[u8] = include_bytes!(
    "fixtures/java8/assignment-condition-widening/javac8-debug/LoopConditionInvokeControl.class"
);
const CONTROL_JAVAC8_NODEBUG: &[u8] = include_bytes!(
    "fixtures/java8/assignment-condition-widening/javac8-no-debug/LoopConditionInvokeControl.class"
);
const CONTROL_JAVAC23_DEBUG: &[u8] = include_bytes!(
    "fixtures/java8/assignment-condition-widening/javac23-debug/LoopConditionInvokeControl.class"
);
const CONTROL_JAVAC23_NODEBUG: &[u8] = include_bytes!(
    "fixtures/java8/assignment-condition-widening/javac23-no-debug/LoopConditionInvokeControl.class"
);

const UPSTREAM_METHODS: &[(&str, &str)] = &[("<init>", "()V"), ("test", "(C)Z"), ("next", "()C")];
const CONTROL_METHODS: &[(&str, &str)] = &[
    ("<init>", "([CI)V"),
    ("test", "(C)Z"),
    ("next", "()C"),
    ("getPos", "()I"),
    ("getCalls", "()I"),
    ("getEvents", "()I"),
];

struct Fixture {
    label: &'static str,
    bytes: &'static [u8],
    class: &'static str,
    methods: &'static [(&'static str, &'static str)],
}

const FIXTURES: &[Fixture] = &[
    Fixture {
        label: "upstream debug",
        bytes: UPSTREAM_DEBUG,
        class: "jadx/tests/integration/loops/TestLoopConditionInvoke$TestCls",
        methods: UPSTREAM_METHODS,
    },
    Fixture {
        label: "upstream no-debug",
        bytes: UPSTREAM_NODEBUG,
        class: "jadx/tests/integration/loops/TestLoopConditionInvoke$TestCls",
        methods: UPSTREAM_METHODS,
    },
    Fixture {
        label: "control javac8 debug",
        bytes: CONTROL_JAVAC8_DEBUG,
        class: "controls/LoopConditionInvokeControl",
        methods: CONTROL_METHODS,
    },
    Fixture {
        label: "control javac8 no-debug",
        bytes: CONTROL_JAVAC8_NODEBUG,
        class: "controls/LoopConditionInvokeControl",
        methods: CONTROL_METHODS,
    },
    Fixture {
        label: "control javac23 debug",
        bytes: CONTROL_JAVAC23_DEBUG,
        class: "controls/LoopConditionInvokeControl",
        methods: CONTROL_METHODS,
    },
    Fixture {
        label: "control javac23 no-debug",
        bytes: CONTROL_JAVAC23_NODEBUG,
        class: "controls/LoopConditionInvokeControl",
        methods: CONTROL_METHODS,
    },
];

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .expect("the frozen standalone class opens")
}

fn report(
    snapshot: &ArtifactSnapshot,
    class: &str,
    evidence: Option<&RecoveryEvidenceRequest>,
) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
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
    let engine = Engine::new();
    let result = if let Some(evidence) = evidence {
        engine.class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            evidence,
            &mut task_budget(&[]).unwrap(),
        )
    } else {
        engine.class_source(
            slice::from_ref(snapshot),
            &request,
            &mut task_budget(&[]).unwrap(),
        )
    }
    .expect("the class-source request answers");
    match result {
        OperationOutcome::Performed(report) => report,
        other => panic!(
            "{} should have a complete class-source answer: {other:?}",
            class
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
        .unwrap_or_else(|| panic!("missing member {name}{descriptor}"))
}

fn recovered<'a>(
    report: &'a ClassSourceReport,
    name: &str,
    descriptor: &str,
) -> &'a RecoveryReport {
    match &member(report, name, descriptor).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("{name} was not recovered: {other:?}"),
    }
}

fn assert_structured(report: &ClassSourceReport, fixture: &Fixture) {
    assert_eq!(
        report.methods.len(),
        fixture.methods.len(),
        "{} method inventory",
        fixture.label
    );
    for (name, descriptor) in fixture.methods {
        let body = recovered(report, name, descriptor);
        assert!(
            body.produced(),
            "{}.{}{} produced no body",
            fixture.label,
            name,
            descriptor
        );
        assert_eq!(
            body.representation,
            Representation::Java,
            "{}.{}{}",
            fixture.label,
            name,
            descriptor
        );
        assert_eq!(
            body.quality,
            Quality::Structured,
            "{}.{}{}: {}",
            fixture.label,
            name,
            descriptor,
            body.text
        );
        assert_eq!(
            body.content,
            RecoveryContent::ContainsStatements,
            "{}.{}{}",
            fixture.label,
            name,
            descriptor
        );
        assert!(
            !body.text.contains("@bytecode"),
            "{}.{}{} remains quoted:\n{}",
            fixture.label,
            name,
            descriptor,
            body.text
        );
    }
}

fn assert_test_code_map(class_bytes: &[u8], report: &ClassSourceReport, label: &str) {
    let method = member(report, "test", "(C)Z");
    let body = recovered(report, "test", "(C)Z");
    assert_eq!(
        body.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete,
        "{label}"
    );
    let mut reader_budget = task_budget(&[]).expect("bounded reader budget");
    let physical = inspect_method_bytecode(
        class_bytes,
        MethodSelector {
            name: JvmBytes(b"test".to_vec()),
            descriptor: JvmBytes(b"(C)Z".to_vec()),
        },
        &mut reader_budget,
    )
    .expect("complete physical test(C)Z instructions are readable");
    assert!(
        physical.stopped_at.is_none(),
        "{label}: physical Code scan stopped"
    );
    let physical_bcis: BTreeSet<_> = physical
        .instructions
        .iter()
        .map(|instruction| instruction.bci)
        .collect();
    assert!(
        !physical_bcis.is_empty(),
        "{label}: test(C)Z has Code instructions"
    );

    let mut mapped = BTreeSet::new();
    for segment in body.source_map.segments() {
        assert!(
            segment.start() <= segment.end(),
            "{label}: reversed source-map span"
        );
        assert!(
            body.text.get(segment.start()..segment.end()).is_some(),
            "{label}: source-map span is not on UTF-8 boundaries"
        );
        let origin = segment.origin();
        for anchor in std::iter::once(origin.primary()).chain(origin.derived()) {
            assert_eq!(
                anchor.method(),
                Some(&method.item.identity),
                "{label}: source anchor belongs to another or unknown physical method"
            );
            assert!(
                physical_bcis.contains(&anchor.bci()),
                "{label}: source anchor BCI {} is outside physical Code",
                anchor.bci()
            );
            mapped.insert(anchor.bci());
        }
    }
    assert_eq!(
        mapped, physical_bcis,
        "{label}: source map must cover exactly the physical test(C)Z instruction set"
    );
}

fn assert_widened_condition(class_bytes: &[u8], report: &ClassSourceReport, label: &str) {
    let body = recovered(report, "test", "(C)Z");
    assert_test_code_map(class_bytes, report, label);
    assert!(body.text.contains("while ("), "{label}: {}", body.text);
    assert_eq!(
        body.text.matches("next()").count(),
        1,
        "the assigned RHS is evaluated once: {label}: {}",
        body.text
    );
    assert!(body.text.contains("return true"), "{label}: {}", body.text);
    assert!(body.text.contains("return false"), "{label}: {}", body.text);
    assert!(
        !body.text.contains("(int)"),
        "widening must not add a cast: {label}: {}",
        body.text
    );
}

fn assert_same_publication(
    default: &ClassSourceReport,
    essential: &ClassSourceReport,
    all: &ClassSourceReport,
    fixture: &Fixture,
) {
    assert_eq!(
        default.text, essential.text,
        "{} default vs explicit essential class text",
        fixture.label
    );
    assert_eq!(
        default.text, all.text,
        "{} default vs all class text",
        fixture.label
    );
    for (name, descriptor) in fixture.methods {
        let d = recovered(default, name, descriptor);
        let e = recovered(essential, name, descriptor);
        let a = recovered(all, name, descriptor);
        let dm = member(default, name, descriptor);
        let em = member(essential, name, descriptor);
        let am = member(all, name, descriptor);
        assert_eq!(
            dm.text, em.text,
            "{}.{}{} default vs essential member text",
            fixture.label, name, descriptor
        );
        assert_eq!(
            dm.text, am.text,
            "{}.{}{} default vs all member text",
            fixture.label, name, descriptor
        );
        assert_eq!(
            d.text, e.text,
            "{}.{}{} default vs essential raw body",
            fixture.label, name, descriptor
        );
        assert_eq!(
            d.text, a.text,
            "{}.{}{} default vs all raw body",
            fixture.label, name, descriptor
        );
        assert_eq!(
            (d.quality, d.representation, &d.content),
            (e.quality, e.representation, &e.content)
        );
        assert_eq!(
            (d.quality, d.representation, &d.content),
            (a.quality, a.representation, &a.content)
        );
        assert!(
            e.source_map.is_empty(),
            "{}.{}{} explicit essential requested a source map",
            fixture.label,
            name,
            descriptor
        );
        assert_eq!(
            e.evidence.state(RecoveryEvidenceKind::SourceMap),
            EvidenceState::NotRequested,
            "{}.{}{} explicit essential source-map state",
            fixture.label,
            name,
            descriptor
        );
        assert_eq!(
            d.source_map, a.source_map,
            "{}.{}{} default and all maps",
            fixture.label, name, descriptor
        );
    }
}

#[test]
fn fixed_upstream_and_dynamic_controls_recover_the_widened_loop_condition() {
    for fixture in FIXTURES {
        let snapshot = open(fixture.bytes);
        let default = report(&snapshot, fixture.class, None);
        let essential_request = RecoveryEvidenceRequest::essential();
        let essential = report(&snapshot, fixture.class, Some(&essential_request));
        let all_request = RecoveryEvidenceRequest::all();
        let all = report(&snapshot, fixture.class, Some(&all_request));
        assert_structured(&default, fixture);
        assert_structured(&essential, fixture);
        assert_structured(&all, fixture);
        assert_same_publication(&default, &essential, &all, fixture);
        assert_widened_condition(fixture.bytes, &all, fixture.label);
    }
}

#[test]
fn cancellation_does_not_publish_a_partial_condition_body() {
    let fixture = &FIXTURES[0];
    let snapshot = open(fixture.bytes);
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(fixture.class),
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
    let mut budget =
        Budget::with_cancellation_token(task_budget(&[]).unwrap().limits().clone(), token);
    let outcome = Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("cancellation is reported through the public default class-source entry");
    match outcome {
        OperationOutcome::Incomplete(report) => assert!(matches!(
            report.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                report.execution,
                ExecutionReport::Cancelled { .. }
            ));
            assert!(
                report.text.is_empty(),
                "cancelled class source published text"
            );
        }
        other => panic!("unexpected cancelled result: {other:?}"),
    }
}
