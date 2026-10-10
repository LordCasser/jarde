//! No-debug complete classes for disjoint ordinary-reference local lifetimes. The sixteen frozen
//! inputs cover eight source shapes from both javac 8 and javac 23; each pair must keep the same
//! two source types and the same recovery text under the essential and full evidence selections.

use jarde::*;
use std::slice;

macro_rules! case {
    ($jdk:ident, $class:ident, $first:literal, $second:literal) => {
        (
            stringify!($class),
            include_bytes!(concat!(
                "fixtures/p3-reference-slot-lifetimes/",
                stringify!($jdk),
                "/",
                stringify!($class),
                ".class"
            )),
            $first,
            $second,
        )
    };
}

macro_rules! debug_case {
    ($group:literal, $jdk:ident, $class:ident, $complete:literal) => {
        (
            stringify!($class),
            include_bytes!(concat!(
                "fixtures/p3-reference-slot-lifetimes/debug/",
                $group,
                "/",
                stringify!($jdk),
                "/",
                stringify!($class),
                ".class"
            )),
            include_str!(concat!(
                "fixtures/p3-reference-slot-lifetimes/debug/",
                $group,
                "/",
                stringify!($jdk),
                "/",
                stringify!($class),
                ".baseline.java"
            )),
            $complete,
        )
    };
}

macro_rules! negative_case {
    ($family:literal, $class:ident) => {
        (
            stringify!($class),
            include_bytes!(concat!(
                "fixtures/p3-reference-slot-lifetimes/negative/",
                $family,
                "/",
                stringify!($class),
                ".class"
            )),
            include_str!(concat!(
                "fixtures/p3-reference-slot-lifetimes/negative/",
                $family,
                "/",
                stringify!($class),
                ".baseline.java"
            )),
        )
    };
}

const CASES: &[(&str, &[u8], &str, &str)] = &[
    case!(javac8, ArrayThenList, "int[]", "java.util.ArrayList"),
    case!(javac23, ArrayThenList, "int[]", "java.util.ArrayList"),
    case!(
        javac8,
        DequeThenBuilder,
        "java.util.ArrayDeque",
        "java.lang.StringBuilder"
    ),
    case!(
        javac23,
        DequeThenBuilder,
        "java.util.ArrayDeque",
        "java.lang.StringBuilder"
    ),
    case!(
        javac8,
        StringThenArray,
        "java.lang.String",
        "java.lang.String[]"
    ),
    case!(
        javac23,
        StringThenArray,
        "java.lang.String",
        "java.lang.String[]"
    ),
    case!(
        javac8,
        StringThenBuilder,
        "java.lang.String",
        "java.lang.StringBuilder"
    ),
    case!(
        javac23,
        StringThenBuilder,
        "java.lang.String",
        "java.lang.StringBuilder"
    ),
    case!(
        javac8,
        BuilderThenArray,
        "java.lang.StringBuilder",
        "char[]"
    ),
    case!(
        javac23,
        BuilderThenArray,
        "java.lang.StringBuilder",
        "char[]"
    ),
    case!(
        javac8,
        ListThenMap,
        "java.util.ArrayList",
        "java.util.HashMap"
    ),
    case!(
        javac23,
        ListThenMap,
        "java.util.ArrayList",
        "java.util.HashMap"
    ),
    case!(
        javac8,
        MapThenBuilder,
        "java.util.HashMap",
        "java.lang.StringBuilder"
    ),
    case!(
        javac23,
        MapThenBuilder,
        "java.util.HashMap",
        "java.lang.StringBuilder"
    ),
    case!(
        javac8,
        StringThenQueue,
        "java.lang.String",
        "java.util.ArrayDeque"
    ),
    case!(
        javac23,
        StringThenQueue,
        "java.lang.String",
        "java.util.ArrayDeque"
    ),
];

const DIFFERENT_NAME_DEBUG: &[(&str, &[u8], &str, bool)] = &[
    debug_case!("different-name", javac8, ArrayThenList, true),
    debug_case!("different-name", javac8, StringThenBuilder, true),
    debug_case!("different-name", javac8, StringThenArray, true),
    debug_case!("different-name", javac8, DequeThenBuilder, true),
    debug_case!("different-name", javac23, ArrayThenList, true),
    debug_case!("different-name", javac23, StringThenBuilder, true),
    debug_case!("different-name", javac23, StringThenArray, true),
    debug_case!("different-name", javac23, DequeThenBuilder, true),
];

const SAME_NAME_DEBUG: &[(&str, &[u8], &str, bool)] = &[
    debug_case!("same-name", javac8, ListThenMap, false),
    debug_case!("same-name", javac8, MapThenBuilder, false),
    debug_case!("same-name", javac8, StringThenQueue, false),
    debug_case!("same-name", javac8, BuilderThenArray, false),
    debug_case!("same-name", javac23, ListThenMap, false),
    debug_case!("same-name", javac23, MapThenBuilder, false),
    debug_case!("same-name", javac23, StringThenQueue, false),
    debug_case!("same-name", javac23, BuilderThenArray, false),
];

const NEGATIVE_CASES: &[(&str, &[u8], &str)] = &[
    negative_case!("same-type", V2),
    negative_case!("cross-phi", V3),
    negative_case!("cfg-loop-phi", V4),
    negative_case!("handler", SlotReuseBoundaries),
    negative_case!("parameter-header", ParameterHeader),
    negative_case!("held-use", HeldUse),
    negative_case!("cfg-backedge-disjoint", RefLoopBack),
];

const NONZERO_HELD_USE_CASES: &[(&[u8], &str)] = &[
    (
        include_bytes!(
            "fixtures/p3-reference-slot-lifetimes/negative/nonzero-held-use/javac8/NonzeroHeldUse.class"
        ),
        include_str!(
            "fixtures/p3-reference-slot-lifetimes/negative/nonzero-held-use/javac8/NonzeroHeldUse.baseline.java"
        ),
    ),
    (
        include_bytes!(
            "fixtures/p3-reference-slot-lifetimes/negative/nonzero-held-use/javac23/NonzeroHeldUse.class"
        ),
        include_str!(
            "fixtures/p3-reference-slot-lifetimes/negative/nonzero-held-use/javac23/NonzeroHeldUse.baseline.java"
        ),
    ),
];

fn opened(bytes: &[u8], name: &str) -> (ArtifactSnapshot, ClassSourceRequest) {
    let mut budget = task_budget(&[]).expect("bounded task budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("frozen Java 8 class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
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
    (snapshot, request)
}

fn recover(bytes: &[u8], name: &str, evidence: &RecoveryEvidenceRequest) -> ClassSourceReport {
    let (snapshot, request) = opened(bytes, name);
    let mut budget = task_budget(&[]).expect("bounded task budget");
    match Engine::new()
        .class_source_with_evidence(slice::from_ref(&snapshot), &request, evidence, &mut budget)
        .expect("standalone class source request completes")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one standalone class resolves exactly: {other:?}"),
    }
}

fn declared_names(text: &str, ty: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let declaration = line.trim().strip_prefix(ty)?.trim_start();
            let name = declaration.split_whitespace().next()?;
            let name = name.trim_end_matches(';');
            (name.starts_with("local")
                && name[5..].chars().all(|ch| ch.is_ascii_digit() || ch == '_'))
            .then(|| name.to_owned())
        })
        .collect()
}

fn assert_pair(text: &str, first_type: &str, second_type: &str) {
    let first = declared_names(text, first_type);
    let second = declared_names(text, second_type);
    assert!(
        first
            .iter()
            .any(|name| second.contains(&format!("{name}_2"))),
        "the two types must be separate declarations for one reused slot:\n{text}"
    );
}

#[test]
fn all_sixteen_no_debug_classes_keep_each_reference_lifetime_separate() {
    for (name, bytes, first_type, second_type) in CASES {
        let essential = recover(bytes, name, &RecoveryEvidenceRequest::essential());
        let all = recover(bytes, name, &RecoveryEvidenceRequest::all());
        assert_eq!(
            essential.text, all.text,
            "{name}: essential and full evidence must publish the same source"
        );
        assert!(
            all.methods.iter().all(|method| method.markers.is_empty()),
            "{name} has an incomplete method: {:?}",
            all.methods
                .iter()
                .map(|method| (&method.item.name, &method.markers))
                .collect::<Vec<_>>()
        );
        assert!(
            !all.text.contains("@bytecode"),
            "{name} contains a fallback marker:\n{}",
            all.text
        );
        assert_pair(&all.text, first_type, second_type);
    }
}

#[test]
fn different_name_lvt_sources_remain_byte_for_byte_unchanged() {
    assert_eq!(DIFFERENT_NAME_DEBUG.len(), 8);
    for (name, bytes, baseline, complete) in DIFFERENT_NAME_DEBUG {
        assert!(*complete);
        let report = recover(bytes, name, &RecoveryEvidenceRequest::all());
        assert_eq!(report.text, *baseline, "{name}: debug source changed");
        assert!(
            !report.text.contains("@bytecode"),
            "{name}: this baseline row is a recovered debug case"
        );
    }
}

#[test]
fn same_name_lvt_sources_keep_their_existing_failure_verbatim() {
    assert_eq!(SAME_NAME_DEBUG.len(), 8);
    for (name, bytes, baseline, complete) in SAME_NAME_DEBUG {
        assert!(!*complete);
        let report = recover(bytes, name, &RecoveryEvidenceRequest::all());
        assert_eq!(
            report.text, *baseline,
            "{name}: preserved same-name result changed"
        );
    }
}

#[test]
fn unsupported_reference_boundaries_keep_the_cli9_source_verbatim() {
    assert_eq!(NEGATIVE_CASES.len(), 7);
    for (name, bytes, baseline) in NEGATIVE_CASES {
        let report = recover(bytes, name, &RecoveryEvidenceRequest::all());
        assert_eq!(
            report.text, *baseline,
            "{name}: unsupported-shape output changed"
        );
    }
}

/// Null followed by one exact constructor producer is now a proved reference local. Preserve the
/// historical baseline and require the entire class to differ only in that declaration's type.
#[test]
fn null_first_exact_builder_writes_refine_only_the_local_type() {
    let (name, bytes, baseline) = negative_case!("unknown-null", NullThenBuilder);
    assert_eq!(baseline.matches("Object local2;").count(), 1);
    let expected = baseline.replace("Object local2;", "java.lang.StringBuilder local2;");
    let essential = recover(bytes, name, &RecoveryEvidenceRequest::essential());
    let all = recover(bytes, name, &RecoveryEvidenceRequest::all());
    assert_eq!(essential.text, expected);
    assert_eq!(all.text, expected);
    assert_eq!(essential.methods.len(), 3);
    assert_eq!(all.methods.len(), 3);
    assert!(all.methods.iter().all(|method| method.markers.is_empty()));
    for (left, right) in essential.methods.iter().zip(&all.methods) {
        assert_eq!(left.item, right.item);
        let identity = left.item.identity.clone();
        let (
            ClassSourceOutcome::Recovered { report: left, .. },
            ClassSourceOutcome::Recovered { report: right, .. },
        ) = (&left.outcome, &right.outcome)
        else {
            panic!("whole-class member recovery remains present");
        };
        assert_eq!(left.text, right.text);
        assert!(!right.source_map.segments().is_empty());
        for segment in right.source_map.segments() {
            for origin in
                std::iter::once(segment.origin().primary()).chain(segment.origin().derived().iter())
            {
                assert_eq!(origin.method(), Some(&identity));
            }
        }
    }
}

#[test]
fn held_reference_across_a_local_overwrite_is_not_published_as_a_slot_alias() {
    let (name, bytes, baseline) = NEGATIVE_CASES
        .iter()
        .find(|(name, _, _)| *name == "HeldUse")
        .expect("the frozen HeldUse negative is present");
    let report = recover(bytes, name, &RecoveryEvidenceRequest::all());
    assert_eq!(report.text, *baseline, "HeldUse source stays verbatim");
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"run")
        .expect("HeldUse.run is present");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("HeldUse.run recovery is present: {:?}", method.outcome);
    };
    assert!(
        report.aliased_names.is_empty(),
        "an old reference still held on the operand stack cannot alias the reused local: {:?}",
        report.aliased_names
    );
    assert!(
        !report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_name_aliased"),
        "HeldUse.run does not publish a name-alias diagnostic: {:?}",
        report.diagnostics
    );
}

#[test]
fn nonzero_stack_reference_forwarding_keeps_both_jdk_sources_and_metadata_unchanged() {
    for (bytes, baseline) in NONZERO_HELD_USE_CASES {
        let report = recover(bytes, "NonzeroHeldUse", &RecoveryEvidenceRequest::all());
        assert_eq!(
            report.text, *baseline,
            "NonzeroHeldUse source stays verbatim"
        );
        let method = report
            .methods
            .iter()
            .find(|method| {
                method.item.name.raw().0 == b"run"
                    && method.item.descriptor.raw().0 == b"()Ljava/lang/Object;"
            })
            .expect("NonzeroHeldUse.run is present");
        let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
            panic!(
                "NonzeroHeldUse.run recovery is present: {:?}",
                method.outcome
            );
        };
        assert!(
            report.aliased_names.is_empty(),
            "a reference checked on a nonzero stack slot cannot alias the reused local: {:?}",
            report.aliased_names
        );
        assert!(
            !report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "jre_name_aliased"),
            "NonzeroHeldUse.run does not publish a name-alias diagnostic: {:?}",
            report.diagnostics
        );
    }
}

#[test]
fn low_analysis_budget_and_cancellation_do_not_publish_a_split_source() {
    let (snapshot, request) = opened(CASES[0].1, CASES[0].0);
    let complete = recover(CASES[0].1, CASES[0].0, &RecoveryEvidenceRequest::all());

    let mut low_budget =
        task_budget(&[BudgetOverride::new("analysis_steps", 1).expect("valid analysis-step cap")])
            .expect("the bounded task budget accepts an analysis-step cap");
    match Engine::new().class_source_with_evidence(
        slice::from_ref(&snapshot),
        &request,
        &RecoveryEvidenceRequest::all(),
        &mut low_budget,
    ) {
        Ok(OperationOutcome::Performed(report)) => {
            assert!(!report.text.is_empty(), "a completed response is not empty");
            assert_ne!(
                report.text, complete.text,
                "a stopped run published full output"
            );
            assert!(
                !report.text.contains("java.util.ArrayList local2_2"),
                "a stopped run published the later segment's identity"
            );
        }
        Ok(_) => {}
        other => panic!("low-budget request returns its ordinary stop result: {other:?}"),
    }

    let token = CancellationToken::new();
    token.cancel();
    let limits = task_budget(&[])
        .expect("bounded task budget")
        .limits()
        .clone();
    let mut cancelled = Budget::with_cancellation_token(limits, token);
    match Engine::new().class_source_with_evidence(
        slice::from_ref(&snapshot),
        &request,
        &RecoveryEvidenceRequest::all(),
        &mut cancelled,
    ) {
        Ok(OperationOutcome::Performed(report)) => {
            assert!(!report.text.is_empty(), "a completed response is not empty");
            assert_ne!(
                report.text, complete.text,
                "a cancelled run published full output"
            );
            assert!(
                !report.text.contains("java.util.ArrayList local2_2"),
                "a cancelled run published the later segment's identity"
            );
        }
        Ok(_) => {}
        other => panic!("cancelled request returns its ordinary stop result: {other:?}"),
    }
}
