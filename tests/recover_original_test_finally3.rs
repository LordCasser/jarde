//! The original TestFinally3 targets preserve the empty byte-array constructor argument.
use jarde::*;
use std::slice;

const DEBUG: &[u8] = include_bytes!(
    "fixtures/constructor-direct-array-arguments/tf3-debug/TestFinally3$TestCls.class"
);
const NO_DEBUG: &[u8] = include_bytes!(
    "fixtures/constructor-direct-array-arguments/tf3-no-debug/TestFinally3$TestCls.class"
);
const CLASS: &str = "jadx/tests/integration/trycatch/TestFinally3$TestCls";
const TARGETS: &[(&str, &str)] = &[
    ("<init>", "()V"),
    ("getInputStream", "()Ljava/io/InputStream;"),
    ("close", "(Ljava/io/InputStream;)V"),
    ("read", "(Ljava/io/InputStream;)[B"),
    ("validate", "()Z"),
    ("convert", "([B)[B"),
    ("test", "()[B"),
];
const ARRAY_BCIS: &[u32] = &[0, 3, 4, 5, 7, 10];

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut task_budget(&[]).expect("bounded fixture-open budget"),
        )
        .expect("frozen TestFinally3 class opens")
}

fn request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(CLASS),
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
    let req = request(snapshot);
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("bounded task defaults");
    match if all {
        engine.class_source_with_evidence(
            slice::from_ref(snapshot),
            &req,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
    } else {
        engine.class_source(slice::from_ref(snapshot), &req, &mut budget)
    }
    .expect("class-source request completes")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("the standalone physical class completes: {other:?}"),
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
        .unwrap_or_else(|| panic!("missing original method {name}{descriptor}"))
}

fn body<'a>(method: &'a ClassSourceMethod) -> &'a RecoveryReport {
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("original method report exists: {other:?}"),
    }
}

fn assert_report(report: &ClassSourceReport, leg: &str) {
    let actual = report
        .methods
        .iter()
        .map(|m| {
            (
                m.item.name.raw().0.clone(),
                m.item.descriptor.raw().0.clone(),
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    let expected = TARGETS
        .iter()
        .map(|(n, d)| (n.as_bytes().to_vec(), d.as_bytes().to_vec()))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        actual, expected,
        "{leg}: all seven physical methods remain present"
    );

    let target_method = method(report, "getInputStream", "()Ljava/io/InputStream;");
    let target = body(target_method);
    assert_eq!(
        target.quality,
        Quality::Structured,
        "{leg}: {}",
        target.text
    );
    assert_eq!(target.content, RecoveryContent::ContainsStatements, "{leg}");
    assert!(
        target.fallbacks.is_empty(),
        "{leg}: {:?}\n{}",
        target.fallbacks,
        target.text
    );
    assert!(
        target.text.contains("new byte[0]") || target.text.contains("new byte[] {}"),
        "{leg}: empty byte array remains explicit:\n{}",
        target.text
    );
    for bci in ARRAY_BCIS {
        assert!(
            !target.source_map.of_bci(*bci).is_empty(),
            "{leg}: missing array BCI {bci}"
        );
    }
    assert!(
        target
            .source_map
            .segments()
            .iter()
            .flat_map(|s| { std::iter::once(s.origin().primary()).chain(s.origin().derived()) })
            .all(|origin| origin.method() == Some(&target_method.item.identity)),
        "{leg}: exact physical method identity is retained"
    );
}

fn assert_parity(default: &ClassSourceReport, all: &ClassSourceReport, leg: &str) {
    assert_eq!(default.text, all.text, "{leg}: default/all text");
    for (name, descriptor) in TARGETS {
        let a = body(method(default, name, descriptor));
        let b = body(method(all, name, descriptor));
        assert_eq!(a.text, b.text, "{leg}: {name}");
        assert_eq!(a.source_map, b.source_map, "{leg}: {name} map");
        assert_eq!(a.fallbacks, b.fallbacks, "{leg}: {name} fallback");
    }
}

#[test]
fn original_test_finally3_debug_profiles_keep_all_methods_and_array_origins() {
    for (leg, bytes) in [("debug", DEBUG), ("no-debug", NO_DEBUG)] {
        let snapshot = open(bytes);
        let default = source(&snapshot, false);
        let all = source(&snapshot, true);
        assert_report(&default, leg);
        assert_report(&all, leg);
        assert_parity(&default, &all, leg);
    }
}
