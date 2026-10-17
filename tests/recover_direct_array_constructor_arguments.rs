//! Public class-source integration contract for directly consumed array constructor arguments.
use jarde::*;
use std::slice;

const JAVAC8: &[u8] = include_bytes!(
    "fixtures/constructor-direct-array-arguments/javac8/classes/DirectArrayConstructorArguments.class"
);
const JAVAC23: &[u8] = include_bytes!(
    "fixtures/constructor-direct-array-arguments/javac23/classes/DirectArrayConstructorArguments.class"
);

const POSITIVE: &[(&str, &str)] = &[
    (
        "barePrimitive",
        "(I)LDirectArrayConstructorArguments$ByteSink;",
    ),
    (
        "bareReference",
        "(I)LDirectArrayConstructorArguments$ReferenceSink;",
    ),
    (
        "initializedPrimitive",
        "()LDirectArrayConstructorArguments$IntSink;",
    ),
    (
        "initializedReference",
        "()LDirectArrayConstructorArguments$ReferenceSink;",
    ),
    (
        "nestedChildren",
        "()LDirectArrayConstructorArguments$MatrixSink;",
    ),
    (
        "bareMultianewarray",
        "(II)LDirectArrayConstructorArguments$MatrixSink;",
    ),
    (
        "constructedElements",
        "()LDirectArrayConstructorArguments$ElementSink;",
    ),
    ("directStringCharArray", "()Ljava/lang/String;"),
    ("stringOtherDescriptor", "()Ljava/lang/String;"),
    ("directStringCharArrayEffectful", "()Ljava/lang/String;"),
    (
        "directOtherCharArrayOwner",
        "()LDirectArrayConstructorArguments$CharSink;",
    ),
];
const CONTROL: (&str, &str) = (
    "twoArrayArguments",
    "()LDirectArrayConstructorArguments$PairSink;",
);

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut task_budget(&[]).expect("bounded fixture-open budget"),
        )
        .expect("frozen standalone class opens")
}

fn request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("DirectArrayConstructorArguments"),
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
    }
}

fn source(snapshot: &ArtifactSnapshot, all: bool) -> ClassSourceReport {
    let request = request(snapshot);
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("bounded task defaults");
    match if all {
        engine.class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
    } else {
        engine.class_source(slice::from_ref(snapshot), &request, &mut budget)
    }
    .expect("one frozen physical class has a class-source outcome")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("the standalone class request completes: {other:?}"),
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
        .find(|m| {
            m.item.name.raw().0 == name.as_bytes()
                && m.item.descriptor.raw().0 == descriptor.as_bytes()
        })
        .unwrap_or_else(|| panic!("missing physical method {name}{descriptor}"))
}

fn body<'a>(method: &'a ClassSourceMethod) -> &'a RecoveryReport {
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("the physical method has a recovery report: {other:?}"),
    }
}

fn assert_method_identity_and_allocation_origins(
    method: &ClassSourceMethod,
    report: &RecoveryReport,
    allocation_end: u32,
) {
    let identity = &method.item.identity;
    assert!(
        report
            .source_map
            .segments()
            .iter()
            .flat_map(|s| { std::iter::once(s.origin().primary()).chain(s.origin().derived()) })
            .all(|origin| origin.method() == Some(identity)),
        "every direct and derived origin must identify this exact physical method"
    );
    // The diagnostic assertion branch has a separately recorded legacy provenance gap.
    // All positive methods use an unbounded end; the control checks the complete array expression
    // before the unsupported assertion branch, while retaining every method and origin identity.
    for allocation in report.news.iter().filter(|site| site.head < allocation_end) {
        assert!(
            !report.source_map.of_bci(allocation.head).is_empty(),
            "missing allocation BCI {}",
            allocation.head
        );
        for bci in allocation
            .dup
            .into_iter()
            .chain(allocation.constructor)
            .chain(allocation.arguments.iter().copied())
        {
            assert!(
                !report.source_map.of_bci(bci).is_empty(),
                "missing constructor site BCI {bci}"
            );
        }
    }
}

fn assert_complete_report(report: &ClassSourceReport, leg: &str) {
    for (name, descriptor) in POSITIVE {
        let physical = method(report, name, descriptor);
        let recovered = body(physical);
        assert_eq!(
            recovered.representation,
            Representation::Java,
            "{leg}: {name}"
        );
        assert_eq!(
            recovered.quality,
            Quality::Structured,
            "{leg}: {name}: {}",
            recovered.text
        );
        assert_eq!(
            recovered.content,
            RecoveryContent::ContainsStatements,
            "{leg}: {name}"
        );
        assert!(
            recovered.fallbacks.is_empty(),
            "{leg}: {name}: {:?}\n{}",
            recovered.fallbacks,
            recovered.text
        );
        assert!(
            !recovered.text.contains("@bytecode") && !recovered.text.contains("jarde_refused_body"),
            "{leg}: {name}: {}",
            recovered.text
        );
        assert!(
            recovered.news.iter().all(|site| site.presented()),
            "{leg}: {name}"
        );
        assert_method_identity_and_allocation_origins(physical, recovered, u32::MAX);
    }

    // This unchanged physical method remains a whole-method fallback control. The unsupported
    // assertion allocation is explained at its real BCI rather than rendered as Java syntax.
    let control = method(report, CONTROL.0, CONTROL.1);
    let recovered = body(control);
    assert_eq!(
        recovered.quality,
        Quality::Fallback,
        "{leg}: {}",
        recovered.text
    );
    // Region fallback reasons can be empty when final presentation refuses the whole method.
    assert!(
        recovered.text.contains("trace != 56")
            && recovered.text.contains("@bytecode 49")
            && recovered.text.contains("at BCI 52"),
        "{leg}: the original assertion branch and allocation refusal remain visible:\n{}",
        recovered.text
    );
    assert_method_identity_and_allocation_origins(control, recovered, 49);
}

fn assert_parity(default: &ClassSourceReport, all: &ClassSourceReport, leg: &str) {
    assert_eq!(
        default.text, all.text,
        "{leg}: default/all class text differs"
    );
    for (name, descriptor) in POSITIVE.iter().copied().chain(std::iter::once(CONTROL)) {
        let a = body(method(default, name, descriptor));
        let b = body(method(all, name, descriptor));
        assert_eq!(a.text, b.text, "{leg}: {name} text differs");
        assert_eq!(
            a.source_map, b.source_map,
            "{leg}: {name} source map differs"
        );
        assert_eq!(
            a.fallbacks, b.fallbacks,
            "{leg}: {name} first refusal differs"
        );
    }
}

#[test]
fn direct_array_arguments_generalize_across_both_javac_fixtures_and_evidence_modes() {
    for (leg, bytes) in [("javac8", JAVAC8), ("javac23", JAVAC23)] {
        let snapshot = open(bytes);
        let default = source(&snapshot, false);
        let all = source(&snapshot, true);
        assert_complete_report(&default, leg);
        assert_complete_report(&all, leg);
        assert_parity(&default, &all, leg);
    }
}
