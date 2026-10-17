//! A throw-only protected body with one completing cleanup recovers through the existing
//! structured `try`/`finally`. The sealed seven-method class is also kept whole: this pins only its
//! `tryThrowsFinallyRuns` member and does not claim its two explicit-finally controls.
use jarde::*;
use std::slice;

const POSITIVE_LEGS: &[(&str, &[u8])] = &[
    (
        "javac23 --release8",
        include_bytes!(
            "fixtures/recover-exception-only-finally-completion/v8/ExceptionOnlyFinally.class"
        ),
    ),
    (
        "javac8",
        include_bytes!(
            "fixtures/recover-exception-only-finally-completion/v8-javac8/ExceptionOnlyFinally.class"
        ),
    ),
];
const SEALED_LEGS: &[(&str, &[u8])] = &[
    (
        "sealed javac8",
        include_bytes!(
            "fixtures/recover-exception-only-finally-completion/sealed/javac8/FinallyCompletion.class"
        ),
    ),
    (
        "sealed javac23",
        include_bytes!(
            "fixtures/recover-exception-only-finally-completion/sealed/javac23/FinallyCompletion.class"
        ),
    ),
];
const TARGET_BCIS: &[u32] = &[0, 2, 5, 6, 9, 10, 11, 13, 16, 17, 18];

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .expect("the frozen standalone class opens")
}

fn request(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceRequest {
    ClassSourceRequest {
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
    }
}

fn source(snapshot: &ArtifactSnapshot, class: &str, all: bool) -> ClassSourceReport {
    let engine = Engine::new();
    let request = request(snapshot, class);
    let mut budget = task_budget(&[]).expect("bounded task defaults");
    let outcome = if all {
        engine.class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
    } else {
        engine.class_source(slice::from_ref(snapshot), &request, &mut budget)
    }
    .expect("one frozen physical class has a class-source outcome");
    match outcome {
        OperationOutcome::Performed(report) => report,
        other => panic!("the selected standalone class completes: {other:?}"),
    }
}

fn target<'a>(report: &'a ClassSourceReport) -> (&'a ClassSourceMethod, &'a RecoveryReport) {
    let method = report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == b"tryThrowsFinallyRuns"
                && method.item.descriptor.raw().0 == b"()I"
        })
        .expect("the physical target method is retained");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("the target has a method report: {:?}", method.outcome)
    };
    (method, report)
}

fn assert_target(report: &ClassSourceReport, class: &str, leg: &str, expected_methods: &[&str]) {
    let mut actual_methods: Vec<String> = report
        .methods
        .iter()
        .map(|method| String::from_utf8_lossy(&method.item.name.raw().0).into_owned())
        .collect();
    let mut expected_methods: Vec<String> = expected_methods
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    actual_methods.sort();
    expected_methods.sort();
    assert_eq!(
        actual_methods, expected_methods,
        "{leg}: {class} physical methods"
    );
    let (method, body) = target(report);
    assert_eq!(
        body.representation,
        Representation::Java,
        "{leg}: {}",
        body.text
    );
    assert_eq!(body.quality, Quality::Structured, "{leg}: {}", body.text);
    assert_eq!(
        body.content,
        RecoveryContent::ContainsStatements,
        "{leg}: {}",
        body.text
    );
    assert!(
        body.fallbacks.is_empty(),
        "{leg}: {:?}\n{}",
        body.fallbacks,
        body.text
    );
    assert!(!body.text.contains("@bytecode"), "{leg}: {}", body.text);
    assert!(
        !body.text.contains("jarde_refused_body"),
        "{leg}: {}",
        body.text
    );
    assert_eq!(
        body.text.matches("try {").count(),
        1,
        "{leg}: {}",
        body.text
    );
    assert_eq!(
        body.text.matches("finally {").count(),
        1,
        "{leg}: {}",
        body.text
    );
    assert_eq!(
        body.text.matches("mark(7)").count(),
        1,
        "{leg}: {}",
        body.text
    );
    assert_eq!(
        body.text.matches("mark(8)").count(),
        1,
        "{leg}: {}",
        body.text
    );
    let try_at = body.text.find("try {").unwrap();
    let mark_at = body.text.find("mark(7)").unwrap();
    let cleanup_at = body.text.find("finally {").unwrap();
    let cleanup_mark_at = body.text.find("mark(8)").unwrap();
    assert!(
        try_at < mark_at && mark_at < cleanup_at && cleanup_at < cleanup_mark_at,
        "{leg}: {}",
        body.text
    );
    for bci in TARGET_BCIS {
        assert!(
            !body.source_map.of_bci(*bci).is_empty(),
            "{leg}: BCI {bci} has no source origin: {:?}",
            body.source_map.segments()
        );
    }
    assert!(
        body.source_map
            .segments()
            .iter()
            .flat_map(|segment| {
                std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
            })
            .all(|origin| origin.method() == Some(&method.item.identity)),
        "{leg}: each direct/derived source origin names this exact physical method"
    );
}

fn assert_parity(default: &ClassSourceReport, all: &ClassSourceReport, leg: &str) {
    assert_eq!(
        default.text, all.text,
        "{leg}: default/all class text differs"
    );
    let (_, default_body) = target(default);
    let (_, all_body) = target(all);
    assert_eq!(
        default_body.text, all_body.text,
        "{leg}: target text differs"
    );
    assert_eq!(
        default_body.source_map, all_body.source_map,
        "{leg}: target source map differs"
    );
    assert_eq!(
        default_body.fallbacks, all_body.fallbacks,
        "{leg}: target fallback set differs"
    );
}

#[test]
fn both_compilers_recover_the_independent_complete_source_with_all_physical_origins() {
    let mut cross_compiler_text: Option<String> = None;
    for (leg, bytes) in POSITIVE_LEGS {
        let snapshot = open(bytes);
        let default = source(&snapshot, "ExceptionOnlyFinally", false);
        let all = source(&snapshot, "ExceptionOnlyFinally", true);
        assert_target(
            &default,
            "ExceptionOnlyFinally",
            leg,
            &["<init>", "mark", "tryThrowsFinallyRuns", "<clinit>"],
        );
        assert_target(
            &all,
            "ExceptionOnlyFinally",
            leg,
            &["<init>", "mark", "tryThrowsFinallyRuns", "<clinit>"],
        );
        assert_parity(&default, &all, leg);
        if let Some(expected) = &cross_compiler_text {
            assert_eq!(
                &default.text, expected,
                "{leg}: compiler-leg source text differs"
            );
        } else {
            cross_compiler_text = Some(default.text.clone());
        }
    }
}

#[test]
fn both_sealed_originals_recover_only_the_throwing_target_and_keep_all_seven_methods() {
    let mut cross_compiler_text: Option<String> = None;
    for (leg, bytes) in SEALED_LEGS {
        let snapshot = open(bytes);
        let default = source(&snapshot, "FinallyCompletion", false);
        let all = source(&snapshot, "FinallyCompletion", true);
        assert_target(
            &default,
            "FinallyCompletion",
            leg,
            &[
                "<init>",
                "mark",
                "normalReturn",
                "finallyReturns",
                "finallyThrows",
                "tryThrowsFinallyRuns",
                "<clinit>",
            ],
        );
        assert_target(
            &all,
            "FinallyCompletion",
            leg,
            &[
                "<init>",
                "mark",
                "normalReturn",
                "finallyReturns",
                "finallyThrows",
                "tryThrowsFinallyRuns",
                "<clinit>",
            ],
        );
        assert_parity(&default, &all, leg);
        if let Some(expected) = &cross_compiler_text {
            assert_eq!(
                &default.text, expected,
                "{leg}: compiler-leg source text differs"
            );
        } else {
            cross_compiler_text = Some(default.text.clone());
        }
        for name in ["normalReturn", "finallyReturns", "finallyThrows"] {
            assert!(
                default
                    .methods
                    .iter()
                    .any(|method| method.item.name.raw().0 == name.as_bytes()),
                "{leg}: the original physical method `{name}` remains present"
            );
        }
    }
}
