//! A for header owns only a proved preheader local initialization and one CFG latch update.

use jarde::*;
use jarde_jvm::ir::{Quality, Representation};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use std::slice;

const CLASS: &[u8] = include_bytes!("fixtures/p3-for-latch/v8/ForBoundaries.class");
const CF07: &[u8] = include_bytes!("fixtures/p3-loop-terminal/LoopCases.class");
const DIRECT_CALL_INIT: &[u8] = include_bytes!(
    "../tests/fixtures/recovered-inputs/changes/recover-proved-computed-for-initializers/results/controls-baseline-root-v1/direct-return-actual/javac8/classes/ComputedForDirectReturn.class"
);
const SHARED_STACK_INPUT: &[u8] = include_bytes!(
    "../tests/fixtures/recovered-inputs/changes/recover-proved-computed-for-initializers/results/shared-stack-input-root-v1/javac8/original/classes/SharedStoreInput.class"
);
const COMPUTED_FOR_CONTROLS: &[u8] = include_bytes!(
    "../tests/fixtures/recovered-inputs/changes/recover-proved-computed-for-initializers/results/controls-baseline-root-v1/actual-controls/javac8/original/ComputedForControls.class"
);
const CONDITION_EFFECTS: &[u8] =
    include_bytes!("fixtures/java8/cf11-for-condition/ForConditionEffects.class");
const CONDITION_READS: &[u8] =
    include_bytes!("fixtures/java8/cf11-for-condition/ForConditionReads.class");
const CONDITION_REFUSALS: &[u8] =
    include_bytes!("fixtures/java8/cf11-for-condition/ForConditionRefusals.class");

fn class_source_report(
    bytes: &[u8],
    class_name: &str,
    evidence: Option<&RecoveryEvidenceRequest>,
) -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("frozen class opens");
    let request = ClassSourceRequest {
        source_target: jarde::SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class_name),
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
    let result = if let Some(evidence) = evidence {
        engine.class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            evidence,
            &mut budget,
        )
    } else {
        engine.class_source(slice::from_ref(&snapshot), &request, &mut budget)
    }
    .expect("single-class request succeeds");
    let OperationOutcome::Performed(report) = result else {
        panic!("one class-source result is expected")
    };
    report
}

fn method_text(name: &str) -> String {
    class_source_report(CLASS, "ForBoundaries", None)
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("the class has no `{name}` member"))
        .text
        .clone()
}

fn class_method<'a>(
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
        .unwrap_or_else(|| panic!("frozen class has no `{name}{descriptor}` method"))
}

fn recovered_body(method: &ClassSourceMethod) -> &RecoveryReport {
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("method has a recovered body: {other:?}"),
    }
}

fn assert_physical_method_coverage(
    class_bytes: &[u8],
    name: &str,
    descriptor: &str,
    method: &ClassSourceMethod,
    report: &RecoveryReport,
) {
    let mut budget = task_budget(&[]).expect("reader budget is valid");
    let inspection = inspect_method_bytecode(
        class_bytes,
        MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        &mut budget,
    )
    .expect("complete physical method instructions are readable");
    assert!(inspection.stopped_at.is_none());
    for instruction in inspection.instructions {
        let segments = report.source_map.of_bci(instruction.bci);
        assert!(
            !segments.is_empty(),
            "physical BCI {} lost source: {}",
            instruction.bci,
            method.text
        );
        assert!(
            segments.iter().any(|segment| {
                std::iter::once(segment.origin().primary())
                    .chain(segment.origin().derived())
                    .any(|origin| {
                        origin.bci() == instruction.bci
                            && origin.method() == Some(&method.item.identity)
                    })
            }),
            "physical BCI {} has no source owned by the exact method: {}",
            instruction.bci,
            method.text
        );
    }
}

#[test]
fn the_single_induction_update_is_a_for_header() {
    let method = method_text("counted");
    assert!(!method.contains("@bytecode"), "{method}");
    assert!(
        method.contains("for (local2 = 0; local2 < arg0; local2 = local2 + 1)"),
        "{method}"
    );
    assert!(method.contains("local1 = local1 + local2;"), "{method}");
}

#[test]
fn a_post_loop_use_or_another_update_refuses_the_header_projection() {
    for name in ["observedAfter", "twoUpdates", "varyingBound", "noUpdate"] {
        let method = method_text(name);
        assert!(!method.contains("for ("), "{name}: {method}");
        assert!(
            method.contains("while (") && !method.contains("@bytecode"),
            "{name}: {method}"
        );
        if name == "twoUpdates" {
            assert_eq!(
                method.matches("local2 = local2 + 1;").count(),
                2,
                "{method}"
            );
        }
    }
}

#[test]
fn for_proof_obeys_the_request_budget_and_cancellation() {
    let mut open_budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(CLASS.to_vec()), &mut open_budget)
        .expect("frozen class opens");
    let request = ClassSourceRequest {
        source_target: jarde::SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal("ForBoundaries"),
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
    let mut limited = task_budget(&[
        BudgetOverride::new("analysis_steps", 1).expect("analysis steps have a legal bound")
    ])
    .expect("the override is legal");
    let stopped = Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut limited)
        .expect("budget exhaustion is an outcome");
    assert!(
        matches!(
            &stopped,
            OperationOutcome::Performed(result)
                if matches!(result.execution, ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded {
                        dimension: BudgetDimension::AnalysisSteps
                    },
                    ..
                })
        ),
        "analysis work cannot publish a complete class after its bound"
    );

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(Limits::default(), token);
    let stopped = Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut cancelled)
        .expect("cancellation is an outcome");
    assert!(
        matches!(
            stopped,
            OperationOutcome::Incomplete(result)
                if matches!(result.execution, ExecutionReport::Cancelled { .. })
        ),
        "a cancelled proof publishes no complete class"
    );
}

#[test]
fn shared_stack_initializer_keeps_once_only_value_and_proved_header() {
    let default = class_source_report(SHARED_STACK_INPUT, "SharedStoreInput", None);
    let evidence = RecoveryEvidenceRequest::all();
    let all = class_source_report(SHARED_STACK_INPUT, "SharedStoreInput", Some(&evidence));
    let default_method = class_method(&default, "find", "([III)I");
    let method = class_method(&all, "find", "([III)I");
    let default_body = recovered_body(default_method);
    let body = recovered_body(method);
    assert_eq!(default.text, all.text, "evidence mode changed class text");
    assert_eq!(
        default_method.text, method.text,
        "evidence mode changed member text"
    );
    assert_eq!(
        default_body.text, body.text,
        "evidence mode changed body text"
    );
    assert_eq!(
        default_body.source_map, body.source_map,
        "evidence mode changed body mappings"
    );
    assert_eq!(default_body.quality, body.quality);
    assert_eq!(default_body.content, body.content);
    assert_eq!(default_body.syntax_status, body.syntax_status);
    assert_eq!(default_body.representation, body.representation);
    assert_structured_for(method, body);
    assert!(
        method
            .text
            .contains("for (local3 = saved0; local3 >= arg2; local3 = local3 - 1)"),
        "{}",
        method.text
    );
    assert_eq!(method.text.matches("limit()").count(), 1);
    assert_eq!(method.text.matches("limit() - 1").count(), 1);
    assert_eq!(
        method
            .text
            .matches("SharedStoreInput.savedField = saved0;")
            .count(),
        1
    );
    assert!(
        method.text.find("limit() - 1").unwrap()
            < method
                .text
                .find("SharedStoreInput.savedField = saved0;")
                .unwrap()
    );
    assert!(
        method
            .text
            .find("SharedStoreInput.savedField = saved0;")
            .unwrap()
            < method.text.find("for (").unwrap()
    );
    for (bci, text) in [
        (0, "limit()"),
        (5, "saved0"),
        (6, "SharedStoreInput.savedField = saved0"),
        (9, "local3 = saved0"),
        (24, "local3 = local3 - 1"),
    ] {
        assert!(
            !call_order_text_spans(method, bci, text).is_empty(),
            "BCI {bci} lost {text}: {}",
            method.text
        );
    }
    assert_physical_method_coverage(SHARED_STACK_INPUT, "find", "([III)I", method, body);
    assert!(
        body.source_map.of_bci(27).iter().any(|segment| {
            let text = segment.text(&body.text);
            text.contains("for (local3 = saved0;")
                && text.contains("return local3;")
                && !text.contains("return -1;")
        }),
        "the physical backedge must own the for body, not the later miss return"
    );
}

#[test]
fn computed_init_return_leaf_and_latch_keep_one_physical_for_source() {
    let default = class_source_report(CF07, "cf07/LoopCases", None);
    let evidence = RecoveryEvidenceRequest::all();
    let all = class_source_report(CF07, "cf07/LoopCases", Some(&evidence));
    let default_method = class_method(&default, "lastIndexOf", "([IIII)I");
    let method = class_method(&all, "lastIndexOf", "([IIII)I");
    let default_body = recovered_body(default_method);
    let body = recovered_body(method);
    assert_eq!(
        default_method.text, method.text,
        "evidence mode changed the member"
    );
    assert_eq!(
        default_body.text, body.text,
        "evidence mode changed the body"
    );
    assert_eq!(
        default_body.source_map, body.source_map,
        "evidence mode changed mapping"
    );

    assert!(
        method
            .text
            .contains("for (local4 = arg3 - 1; local4 >= arg2; local4 = local4 - 1)"),
        "{}",
        method.text
    );
    assert_eq!(
        method.text.matches("int local4;").count(),
        1,
        "{}",
        method.text
    );
    assert_eq!(
        method.text.matches("arg3 - 1").count(),
        1,
        "{}",
        method.text
    );
    assert_eq!(
        method.text.matches("local4 = local4 - 1").count(),
        1,
        "{}",
        method.text
    );
    assert_eq!(
        method.text.matches("return local4;").count(),
        1,
        "{}",
        method.text
    );
    assert_eq!(
        method.text.matches("return -1;").count(),
        1,
        "{}",
        method.text
    );
    assert!(!method.text.contains("while ("), "{}", method.text);
    assert!(!method.text.contains("@bytecode"), "{}", method.text);

    assert_physical_method_coverage(CF07, "lastIndexOf", "([IIII)I", method, body);
    let latch_segments = body.source_map.of_bci(25);
    assert!(
        latch_segments.iter().any(|segment| {
            segment.origin().primary().bci() == 8
                && segment.origin().primary().method() == Some(&method.item.identity)
                && segment.origin().derived().iter().any(|origin| {
                    origin.bci() == 25 && origin.method() == Some(&method.item.identity)
                })
                && segment
                    .text(&body.text)
                    .contains("for (local4 = arg3 - 1; local4 >= arg2; local4 = local4 - 1)")
                && segment.text(&body.text).contains("return local4;")
                && !segment.text(&body.text).contains("return -1;")
        }),
        "goto@25 must anchor the complete For span and exclude the later -1 return: {}",
        body.text
    );
}

#[test]
fn direct_call_initializer_is_written_once_in_the_for_header() {
    let default = class_source_report(DIRECT_CALL_INIT, "ComputedForDirectReturn", None);
    let evidence = RecoveryEvidenceRequest::all();
    let all = class_source_report(DIRECT_CALL_INIT, "ComputedForDirectReturn", Some(&evidence));
    let default_method = class_method(&default, "find", "([III)I");
    let method = class_method(&all, "find", "([III)I");
    let default_body = recovered_body(default_method);
    let body = recovered_body(method);
    assert_eq!(
        default_method.text, method.text,
        "evidence mode changed the member"
    );
    assert_eq!(
        default_body.text, body.text,
        "evidence mode changed the body"
    );
    assert_eq!(
        default_body.source_map, body.source_map,
        "evidence mode changed mapping"
    );
    assert!(
        method
            .text
            .contains("for (local3 = limit() - 1; local3 >= arg2; local3 = local3 - 1)"),
        "{}",
        method.text
    );
    assert_eq!(
        method.text.matches("limit() - 1").count(),
        1,
        "{}",
        method.text
    );
    assert_eq!(
        method.text.matches("return local3;").count(),
        1,
        "{}",
        method.text
    );
    assert_eq!(
        method.text.matches("return -1;").count(),
        1,
        "{}",
        method.text
    );
    assert!(!method.text.contains("while ("), "{}", method.text);
    assert!(!method.text.contains("@bytecode"), "{}", method.text);
    assert_physical_method_coverage(DIRECT_CALL_INIT, "find", "([III)I", method, body);
}

// Baseline control for a computed preheader initializer separated from its store by `g()`.
// These include the accepted original/pending class files directly, avoiding duplicate fixtures.
const CALL_ORDER_ORIGINAL: &[u8] = include_bytes!(
    "../tests/fixtures/recovered-inputs/changes/recover-proved-computed-for-initializers/results/no-handler-call-order-baseline-root-v1/actual-v2/jdk8/original/ComputedForEffectControl.class",
);
const CALL_ORDER_PENDING: &[u8] = include_bytes!(
    "../tests/fixtures/recovered-inputs/changes/recover-proved-computed-for-initializers/results/no-handler-call-order-baseline-root-v1/actual-v2/jdk8/pending/ComputedForEffectControl.class",
);

fn call_order_find(report: &ClassSourceReport) -> &ClassSourceMethod {
    class_method(report, "find", "([IIIIII)I")
}

fn call_order_bci_spans(method: &ClassSourceMethod, bci: u32) -> Vec<(usize, usize)> {
    recovered_body(method)
        .source_map
        .segments()
        .iter()
        .filter(|segment| segment.origin().bcis().contains(&bci))
        .map(|segment| (segment.start(), segment.end()))
        .collect()
}

fn call_order_text_spans(
    method: &ClassSourceMethod,
    bci: u32,
    needle: &str,
) -> Vec<(usize, usize)> {
    let body = recovered_body(method);
    body.source_map
        .segments()
        .iter()
        .filter(|segment| {
            segment.origin().bcis().contains(&bci)
                && body.text[segment.start()..segment.end()].contains(needle)
        })
        .map(|segment| (segment.start(), segment.end()))
        .collect()
}

fn assert_call_order_all_sources(bytes: &[u8], method: &ClassSourceMethod, expected_bcis: &[u32]) {
    let body = recovered_body(method);
    assert_eq!(
        body.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete
    );
    assert_physical_method_coverage(bytes, "find", "([IIIIII)I", method, body);

    let mut bcis = std::collections::BTreeSet::new();
    for segment in body.source_map.segments() {
        assert!(segment.start() < segment.end(), "{segment:?}");
        assert!(
            segment.end() <= body.text.len(),
            "{segment:?}\n{}",
            body.text
        );
        assert!(body.text.is_char_boundary(segment.start()), "{segment:?}");
        assert!(body.text.is_char_boundary(segment.end()), "{segment:?}");
        let origin = segment.origin();
        for anchor in std::iter::once(origin.primary()).chain(origin.derived()) {
            assert_eq!(anchor.method(), Some(&method.item.identity), "{anchor:?}");
            bcis.insert(anchor.bci());
        }
    }
    assert_eq!(bcis.into_iter().collect::<Vec<_>>(), expected_bcis);
}

const CALL_ORDER_ORIGINAL_BCIS: &[u32] = &[
    0, 3, 4, 5, 7, 10, 12, 13, 16, 17, 19, 20, 21, 24, 26, 27, 30, 33, 34,
];
const CALL_ORDER_PENDING_BCIS: &[u32] = &[
    0, 3, 4, 5, 8, 10, 12, 13, 16, 17, 19, 20, 21, 24, 26, 27, 30, 33, 34,
];

#[test]
fn computed_initializer_call_order_matches_physical_store_anchors() {
    let all = RecoveryEvidenceRequest::all();

    // Original bytecode: f()@0, local store@5, then g()@7. The store is before g,
    // not the final preheader operation, so this control must retain the while form.
    let original_all =
        class_source_report(CALL_ORDER_ORIGINAL, "ComputedForEffectControl", Some(&all));
    let original_default =
        class_source_report(CALL_ORDER_ORIGINAL, "ComputedForEffectControl", None);
    assert_eq!(original_default.text, original_all.text);
    let original = call_order_find(&original_all);
    let original_default_method = call_order_find(&original_default);
    assert_eq!(original_default_method.text, original.text);
    assert_eq!(
        recovered_body(original_default_method).source_map,
        recovered_body(original).source_map
    );
    assert_call_order_all_sources(CALL_ORDER_ORIGINAL, original, CALL_ORDER_ORIGINAL_BCIS);
    let original_text = &original.text;
    assert_eq!(original_text.matches("f()").count(), 1, "{original_text}");
    assert_eq!(original_text.matches("g();").count(), 1, "{original_text}");
    assert!(original_text.contains("while ("), "{original_text}");
    assert!(!original_text.contains("for ("), "{original_text}");
    let original_init = original_text
        .find("local6 = f() - 1;")
        .expect("original initializer");
    let original_call = original_text.find("g();").expect("g call");
    assert!(original_init < original_call, "{original_text}");
    let original_store_spans = call_order_text_spans(original, 5, "local6 = f() - 1;");
    let original_g_spans = call_order_text_spans(original, 7, "g()");
    assert!(!original_store_spans.is_empty() && !original_g_spans.is_empty());
    let first_g = original_g_spans
        .iter()
        .map(|(start, _)| *start)
        .min()
        .unwrap();
    assert!(original_store_spans.iter().all(|(_, end)| *end <= first_g));

    // Pending bytecode: f()@0, g()@5, local store@8. A BCI may also anchor
    // later saved-value uses; identify the producer by the mapped text containing the call itself.
    // Do not assume a `for`/`while` representation or concrete initializer spelling here.
    let pending_all =
        class_source_report(CALL_ORDER_PENDING, "ComputedForEffectControl", Some(&all));
    let pending_default = class_source_report(CALL_ORDER_PENDING, "ComputedForEffectControl", None);
    assert_eq!(pending_default.text, pending_all.text);
    let pending = call_order_find(&pending_all);
    let pending_default_method = call_order_find(&pending_default);
    assert_eq!(pending_default_method.text, pending.text);
    assert_eq!(
        recovered_body(pending_default_method).source_map,
        recovered_body(pending).source_map
    );
    assert_call_order_all_sources(CALL_ORDER_PENDING, pending, CALL_ORDER_PENDING_BCIS);
    let pending_text = &pending.text;
    assert_eq!(pending_text.matches("f()").count(), 1, "{pending_text}");
    assert_eq!(pending_text.matches("g();").count(), 1, "{pending_text}");
    let producer_f_spans = call_order_text_spans(pending, 0, "f()");
    let pending_g_spans = call_order_text_spans(pending, 5, "g()");
    let pending_store_spans = call_order_bci_spans(pending, 8);
    assert!(
        !producer_f_spans.is_empty()
            && !pending_g_spans.is_empty()
            && !pending_store_spans.is_empty()
    );
    let last_f_end = producer_f_spans.iter().map(|(_, end)| *end).max().unwrap();
    let first_g_start = pending_g_spans
        .iter()
        .map(|(start, _)| *start)
        .min()
        .unwrap();
    let last_g_end = pending_g_spans.iter().map(|(_, end)| *end).max().unwrap();
    assert!(last_f_end <= first_g_start, "{pending_text}");
    let loop_start = [pending_text.find("for ("), pending_text.find("while (")]
        .into_iter()
        .flatten()
        .min()
        .expect("the fixture still has a loop");
    assert!(
        pending_store_spans
            .iter()
            .any(|(start, _)| *start >= last_g_end && *start <= loop_start),
        "store BCI 8 needs an initialization span after g() and before the loop body:\n{pending_text}"
    );
}

#[test]
fn computed_for_controls_separate_shared_recovery_from_scope_and_header_proofs() {
    let default = class_source_report(COMPUTED_FOR_CONTROLS, "ComputedForControls", None);
    let all = class_source_report(
        COMPUTED_FOR_CONTROLS,
        "ComputedForControls",
        Some(&RecoveryEvidenceRequest::all()),
    );
    assert_eq!(default.text, all.text);

    let methods = [
        ("shared", "(I)I"),
        ("find", "([III)I"),
        ("scoped", "(IZ)I"),
        ("observedAfter", "(I)I"),
        ("externalWrite", "(I)I"),
    ];
    for (name, descriptor) in methods {
        let plain = class_method(&default, name, descriptor);
        let evidenced = class_method(&all, name, descriptor);
        assert_eq!(plain.text, evidenced.text, "{name}{descriptor}");
        let plain_body = recovered_body(plain);
        let evidenced_body = recovered_body(evidenced);
        assert_eq!(
            plain_body.quality, evidenced_body.quality,
            "{name}{descriptor}"
        );
        assert_eq!(
            plain_body.content, evidenced_body.content,
            "{name}{descriptor}"
        );
        assert_eq!(plain_body.text, evidenced_body.text, "{name}{descriptor}");
        assert_eq!(
            plain_body.source_map, evidenced_body.source_map,
            "{name}{descriptor}"
        );

        for segment in plain_body.source_map.segments() {
            assert!(segment.start() < segment.end(), "{name}{descriptor}");
            assert!(segment.end() <= plain_body.text.len(), "{name}{descriptor}");
            assert!(
                plain_body.text.is_char_boundary(segment.start()),
                "{name}{descriptor}"
            );
            assert!(
                plain_body.text.is_char_boundary(segment.end()),
                "{name}{descriptor}"
            );
            for origin in
                std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
            {
                assert_eq!(
                    origin.method(),
                    Some(&plain.item.identity),
                    "{name}{descriptor}"
                );
            }
        }
    }

    for (name, descriptor, refusal) in [
        ("find", "([III)I", "crosses a quoted fallback region"),
        ("scoped", "(IZ)I", "crosses a quoted fallback region"),
    ] {
        let method = class_method(&default, name, descriptor);
        let body = recovered_body(method);
        assert_eq!(body.quality, Quality::Fallback, "{name}{descriptor}");
        assert_eq!(
            body.content,
            RecoveryContent::ExplanationOnly,
            "{name}{descriptor}"
        );
        assert!(
            body.text.contains("@bytecode"),
            "{name}{descriptor}: {}",
            body.text
        );
        assert!(
            body.text.contains(refusal),
            "{name}{descriptor}: {}",
            body.text
        );
    }

    let shared = class_method(&default, "shared", "(I)I");
    let body = recovered_body(shared);
    assert_eq!(body.quality, Quality::Structured);
    assert_eq!(body.content, RecoveryContent::ContainsStatements);
    assert!(!shared.text.contains("@bytecode") && !shared.text.contains("for ("));
    assert!(shared.text.contains("while (local2 >= arg0)"));
    let order = [
        "int saved0 = f();",
        "local1 = saved0;",
        "local2 = saved0 - 1;",
        "g();",
        "while (",
        "return local1 * 100 + local2;",
    ];
    let positions = order.map(|text| {
        assert_eq!(shared.text.matches(text).count(), 1, "{text}");
        shared.text.find(text).unwrap()
    });
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    for (bci, text) in [
        (0, "f()"),
        (3, "saved0"),
        (4, "local1 = saved0"),
        (7, "local2 = saved0 - 1"),
        (8, "g()"),
        (16, "local2 = local2 - 1"),
    ] {
        assert!(
            !call_order_text_spans(shared, bci, text).is_empty(),
            "BCI {bci} lost {text}: {}",
            shared.text
        );
    }
    assert_physical_method_coverage(COMPUTED_FOR_CONTROLS, "shared", "(I)I", shared, body);
    assert!(
        body.source_map.of_bci(19).iter().any(|segment| {
            let text = segment.text(&body.text);
            text.contains("while (local2 >= arg0)")
                && text.contains("local2 = local2 - 1;")
                && !text.contains("return local1")
        }),
        "the physical backedge must own the while body, not the later shared-value return"
    );

    for (name, descriptor) in [("observedAfter", "(I)I"), ("externalWrite", "(I)I")] {
        let method = class_method(&default, name, descriptor);
        let body = recovered_body(method);
        assert_eq!(body.quality, Quality::Structured, "{name}{descriptor}");
        assert_eq!(
            body.content,
            RecoveryContent::ContainsStatements,
            "{name}{descriptor}"
        );
        assert!(
            body.text.contains("while ("),
            "{name}{descriptor}: {}",
            body.text
        );
        assert!(
            !body.text.contains("for ("),
            "{name}{descriptor}: {}",
            body.text
        );
        assert_physical_method_coverage(COMPUTED_FOR_CONTROLS, name, descriptor, method, body);
    }
}

fn for_header_clauses(source: &str) -> [&str; 3] {
    let marker = source
        .find("for (")
        .unwrap_or_else(|| panic!("missing for header:\n{source}"));
    let clauses_start = marker + "for (".len();
    let header_end = source[clauses_start..]
        .find(") {")
        .map(|offset| clauses_start + offset)
        .unwrap_or_else(|| panic!("missing for body after header:\n{source}"));
    let mut clauses = source[clauses_start..header_end]
        .splitn(3, ';')
        .map(str::trim);
    [
        clauses.next().expect("for initializer"),
        clauses.next().expect("for condition"),
        clauses.next().expect("for update"),
    ]
}

fn assert_counted_for_header(source: &str) -> [&str; 3] {
    assert_eq!(source.matches("for (").count(), 1, "{source}");
    let clauses = for_header_clauses(source);
    let initializer = clauses[0];
    let target = initializer
        .split_once('=')
        .unwrap_or_else(|| panic!("for initializer has an assignment: {source}"))
        .0
        .trim();
    let induction = target.strip_prefix("int ").unwrap_or(target);
    assert!(initializer.contains("= 0"), "initializer is zero: {source}");
    assert!(
        (clauses[1].starts_with(induction) || clauses[1].contains(&format!("[{induction}]")))
            && clauses[1].contains("<"),
        "condition reads the induction local: {source}"
    );
    let (update_target, update_value) = clauses[2]
        .split_once('=')
        .unwrap_or_else(|| panic!("for update writes the induction local: {source}"));
    assert_eq!(update_target.trim(), induction, "{source}");
    let expected_update = format!("{induction} + 1");
    assert_eq!(
        update_value.trim(),
        expected_update.as_str(),
        "the exact +1 update is in the header: {source}"
    );
    let update_source = format!("{induction} = {induction} + 1");
    assert_eq!(
        source.matches(update_source.as_str()).count(),
        1,
        "the induction update has one source owner: {source}"
    );
    clauses
}

fn assert_structured_for(method: &ClassSourceMethod, body: &RecoveryReport) {
    assert!(body.produced(), "{:?}\n{}", body.outcome, method.text);
    assert_eq!(body.representation, Representation::Java, "{}", method.text);
    assert_eq!(body.quality, Quality::Structured, "{}", method.text);
    assert_eq!(
        body.content,
        RecoveryContent::ContainsStatements,
        "{}",
        method.text
    );
    assert!(body.fallbacks.is_empty(), "{}", method.text);
    assert!(!method.text.contains("@bytecode"), "{}", method.text);
}

#[test]
fn condition_effect_producers_remain_ordered_and_the_for_update_is_unique() {
    let default = class_source_report(CONDITION_EFFECTS, "cf11/ForConditionEffects", None);
    let all = class_source_report(
        CONDITION_EFFECTS,
        "cf11/ForConditionEffects",
        Some(&RecoveryEvidenceRequest::all()),
    );
    assert_eq!(default.text, all.text, "evidence mode changed class text");

    for (name, descriptor) in [
        ("sumWithFunctionBounds", "()I"),
        ("sumWithFunctionBoundsAndContinue", "()I"),
        ("sumWithChangingFieldBound", "()I"),
    ] {
        let default_method = class_method(&default, name, descriptor);
        let method = class_method(&all, name, descriptor);
        assert_eq!(default_method.text, method.text, "{name}{descriptor}");
        let default_body = recovered_body(default_method);
        let body = recovered_body(method);
        assert_eq!(default_body.text, body.text, "{name}{descriptor}");
        assert_eq!(
            default_body.source_map, body.source_map,
            "{name}{descriptor}"
        );
        assert_structured_for(method, body);
        let clauses = assert_counted_for_header(&method.text);
        if name == "sumWithChangingFieldBound" {
            assert!(clauses[1].contains("dynamicLimit"), "{}", method.text);
            assert!(method.text.contains("dynamicLimit - 1"), "{}", method.text);
            assert!(!clauses[0].contains("dynamicLimit"), "{}", method.text);
        } else {
            assert_eq!(clauses[1].matches("offset()").count(), 1, "{}", method.text);
            assert_eq!(clauses[1].matches("limit()").count(), 1, "{}", method.text);
            assert!(
                clauses[1].find("offset()").expect("offset in header")
                    < clauses[1].find("limit()").expect("limit in header"),
                "condition calls retain Java left-to-right order: {}",
                method.text
            );
            if name == "sumWithFunctionBoundsAndContinue" {
                // javac's continue arm may be projected as if/else followed by the body-tail
                // update. The semantic invariant here is the unique update in the for header.
                assert!(method.text.contains("+ 4;"), "{}", method.text);
                assert!(method.text.contains("+ 3;"), "{}", method.text);
            }
        }
        assert_physical_method_coverage(CONDITION_EFFECTS, name, descriptor, method, body);
    }
}

#[test]
fn field_array_and_negate_reads_are_for_conditions_but_written_local_bound_stays_while() {
    let default = class_source_report(CONDITION_READS, "cf11/ForConditionReads", None);
    let all = class_source_report(
        CONDITION_READS,
        "cf11/ForConditionReads",
        Some(&RecoveryEvidenceRequest::all()),
    );
    assert_eq!(default.text, all.text, "evidence mode changed class text");

    for (name, descriptor, marker) in [
        ("sumWithChangingFieldBound", "()I", "fieldBound"),
        ("sumWithArrayLength", "([I)I", ".length"),
        ("countWithArrayElement", "([I)I", "values["),
        ("sumWithNegatedField", "()I", "negativeField"),
        ("sumWithNegatedParameter", "(I)I", "value"),
    ] {
        let default_method = class_method(&default, name, descriptor);
        let method = class_method(&all, name, descriptor);
        assert_eq!(default_method.text, method.text, "{name}{descriptor}");
        let default_body = recovered_body(default_method);
        let body = recovered_body(method);
        assert_eq!(default_body.text, body.text, "{name}{descriptor}");
        assert_eq!(
            default_body.source_map, body.source_map,
            "{name}{descriptor}"
        );
        assert_structured_for(method, body);
        let clauses = assert_counted_for_header(&method.text);
        assert!(
            clauses[1].contains(marker),
            "{name}{descriptor}: {}",
            method.text
        );
        if name.starts_with("sumWithNegated") {
            assert!(
                clauses[1].contains('-'),
                "{name}{descriptor}: {}",
                method.text
            );
        }
        assert_physical_method_coverage(CONDITION_READS, name, descriptor, method, body);
    }

    let default_method = class_method(&default, "oldWhileWithWrittenLocalBound", "()I");
    let method = class_method(&all, "oldWhileWithWrittenLocalBound", "()I");
    assert_eq!(default_method.text, method.text);
    let default_body = recovered_body(default_method);
    let body = recovered_body(method);
    assert_eq!(default_body.text, body.text);
    assert_eq!(default_body.source_map, body.source_map);
    assert_eq!(body.quality, Quality::Structured, "{}", method.text);
    assert_eq!(
        body.content,
        RecoveryContent::ContainsStatements,
        "{}",
        method.text
    );
    assert!(method.text.contains("while ("), "{}", method.text);
    assert!(!method.text.contains("for ("), "{}", method.text);
    assert!(method.text.contains("bound = bound - 1"), "{}", method.text);
    assert_physical_method_coverage(
        CONDITION_READS,
        "oldWhileWithWrittenLocalBound",
        "()I",
        method,
        body,
    );
}

#[test]
fn local_assignment_of_a_shared_condition_call_remains_a_structured_while() {
    let default = class_source_report(CONDITION_REFUSALS, "cf11/ForConditionRefusals", None);
    let all = class_source_report(
        CONDITION_REFUSALS,
        "cf11/ForConditionRefusals",
        Some(&RecoveryEvidenceRequest::all()),
    );
    assert_eq!(default.text, all.text, "evidence mode changed class text");
    let default_method = class_method(&default, "sumWithAssignedCallBound", "()I");
    let method = class_method(&all, "sumWithAssignedCallBound", "()I");
    assert_eq!(default_method.text, method.text);
    let default_body = recovered_body(default_method);
    let body = recovered_body(method);
    assert_eq!(default_body.text, body.text);
    assert_eq!(default_body.source_map, body.source_map);
    assert_eq!(body.representation, Representation::Java, "{}", method.text);
    assert_eq!(body.quality, Quality::Structured, "{}", method.text);
    assert_eq!(
        body.content,
        RecoveryContent::ContainsStatements,
        "{}",
        method.text
    );
    assert!(method.text.contains("while ("), "{}", method.text);
    assert!(!method.text.contains("for ("), "{}", method.text);
    assert!(method.text.contains("bound = limit()"), "{}", method.text);
    assert_eq!(method.text.matches("limit()").count(), 1, "{}", method.text);
    assert!(
        method.text.contains("return sum + bound;"),
        "{}",
        method.text
    );
    assert_physical_method_coverage(
        CONDITION_REFUSALS,
        "sumWithAssignedCallBound",
        "()I",
        method,
        body,
    );

    let default_method = class_method(&default, "sumWithLeadingTickAndBreak", "()I");
    let method = class_method(&all, "sumWithLeadingTickAndBreak", "()I");
    assert_eq!(default_method.text, method.text);
    let default_body = recovered_body(default_method);
    let body = recovered_body(method);
    assert_eq!(default_body.text, body.text);
    assert_eq!(default_body.source_map, body.source_map);
    assert_structured_for(method, body);
    assert_eq!(default_body.representation, body.representation);
    assert_eq!(default_body.quality, body.quality);
    assert_eq!(default_body.content, body.content);
    assert_eq!(default_body.fallbacks, body.fallbacks);
    assert_eq!(method.text.matches("while (").count(), 1, "{}", method.text);
    assert!(!method.text.contains("for ("), "{}", method.text);
    let loop_at = method
        .text
        .find("while (true) {")
        .expect("the original loop remains a while");
    let tick_at = method
        .text
        .find("tick();")
        .expect("the per-iteration effect remains explicit");
    let branch_at = method
        .text
        .find("if (i >= limit()) {")
        .expect("the loop exit test remains explicit");
    let limit_at = method
        .text
        .find("limit()")
        .expect("the shared condition call remains explicit");
    assert!(
        loop_at < tick_at && tick_at < branch_at && branch_at < limit_at,
        "{}",
        method.text
    );
    assert_eq!(method.text.matches("tick();").count(), 1, "{}", method.text);
    assert_eq!(method.text.matches("limit()").count(), 1, "{}", method.text);
    assert_eq!(
        method.text.matches("i = i + 1;").count(),
        1,
        "{}",
        method.text
    );
    assert_eq!(
        method.text.matches("sum = sum + i;").count(),
        1,
        "{}",
        method.text
    );
    assert_physical_method_coverage(
        CONDITION_REFUSALS,
        "sumWithLeadingTickAndBreak",
        "()I",
        method,
        body,
    );
}

#[test]
fn long_float_and_double_array_comparisons_keep_their_counted_for_headers() {
    let bytes = include_bytes!("fixtures/java8/cf11-for-condition/ForConditionNumeric.class");
    let default = class_source_report(bytes, "cf11/ForConditionNumeric", None);
    let all = class_source_report(
        bytes,
        "cf11/ForConditionNumeric",
        Some(&RecoveryEvidenceRequest::all()),
    );
    assert_eq!(default.text, all.text);
    for (name, descriptor) in [
        ("countLong", "([JJ)I"),
        ("countFloat", "([FF)I"),
        ("countDouble", "([DD)I"),
    ] {
        let original = class_method(&default, name, descriptor);
        let method = class_method(&all, name, descriptor);
        let body = recovered_body(method);
        let original_body = recovered_body(original);
        assert_eq!(original.text, method.text);
        assert_eq!(original_body.source_map, body.source_map);
        assert_structured_for(method, body);
        let clauses = assert_counted_for_header(&method.text);
        assert!(
            clauses[1].contains("values[") && clauses[1].contains("bound"),
            "{}",
            method.text
        );
        assert_physical_method_coverage(bytes, name, descriptor, method, body);
    }
}

// Retain aggregate baseline assertions while independently exposing later controls.
#[test]
fn hidden_find_refusal_is_independently_exposed() {
    let report = class_source_report(COMPUTED_FOR_CONTROLS, "ComputedForControls", None);
    let method = class_method(&report, "find", "([III)I");
    assert_eq!(
        recovered_body(method).quality,
        Quality::Fallback,
        "{}",
        method.text
    );
    assert!(
        method.text.contains("crosses a quoted fallback region"),
        "{}",
        method.text
    );
}
#[test]
fn hidden_scoped_refusal_is_independently_exposed() {
    let report = class_source_report(COMPUTED_FOR_CONTROLS, "ComputedForControls", None);
    let method = class_method(&report, "scoped", "(IZ)I");
    assert_eq!(
        recovered_body(method).quality,
        Quality::Fallback,
        "{}",
        method.text
    );
    assert!(
        method.text.contains("crosses a quoted fallback region"),
        "{}",
        method.text
    );
}

fn latch_javac(dir: &std::path::Path, names: &[&str]) {
    let output = std::process::Command::new("javac")
        .args(["--release", "8", "-g:none", "-cp"])
        .arg(dir)
        .arg("-d")
        .arg(dir)
        .args(names.iter().map(|name| dir.join(name)))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn latch_whole_class_oracle(bytes: &[u8], name: &str, driver: &str, case: &str) {
    let root = std::env::temp_dir().join(format!(
        "jarde-latch-{case}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let original = root.join("original");
    let recovered = root.join("recovered");
    for dir in [&original, &recovered] {
        std::fs::create_dir_all(dir).unwrap();
    }
    std::fs::write(original.join(format!("{name}.class")), bytes).unwrap();
    let plain = class_source_report(bytes, name, None);
    let all = class_source_report(bytes, name, Some(&RecoveryEvidenceRequest::all()));
    assert_eq!(plain.text, all.text, "evidence changed whole class");
    for method in &plain.methods {
        let evidenced = class_method(
            &all,
            &String::from_utf8_lossy(&method.item.name.raw().0),
            &String::from_utf8_lossy(&method.item.descriptor.raw().0),
        );
        if let ClassSourceOutcome::Recovered { report, .. } = &method.outcome {
            let evidence_report = recovered_body(evidenced);
            assert_eq!(report.source_map, evidence_report.source_map);
            assert_eq!(report.quality, evidence_report.quality);
            assert_eq!(report.content, evidence_report.content);
            assert_eq!(report.syntax_status, evidence_report.syntax_status);
            assert_eq!(report.representation, evidence_report.representation);
        }
    }
    std::fs::write(root.join("source-map-report.txt"), format!("{all:#?}")).unwrap();
    std::fs::write(recovered.join(format!("{name}.java")), &plain.text).unwrap();
    println!("LATCH CASE {case}: {}\n{}", root.display(), plain.text);
    latch_javac(&recovered, &[&format!("{name}.java")]);
    let mut observations = Vec::new();
    for dir in [&original, &recovered] {
        std::fs::write(dir.join("LatchDriver.java"), driver).unwrap();
        latch_javac(dir, &["LatchDriver.java"]);
        let output = std::process::Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(dir)
            .arg("LatchDriver")
            .output()
            .unwrap();
        std::fs::write(dir.join("stdout.txt"), &output.stdout).unwrap();
        std::fs::write(dir.join("stderr.txt"), &output.stderr).unwrap();
        assert!(
            output.status.success(),
            "{case}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        observations.push(output.stdout);
    }
    assert_eq!(observations[0], observations[1], "{case}");
}

#[test]
#[ignore = "requires javac and java; explicit whole-class JVM gate"]
fn shared_latch_source_variants_preserve_ownership_and_semantics() {
    let mut unshifted_copy_bci = None;
    for (case, name, delta, shifted) in [
        ("base", "LatchInputs", 1, false),
        ("renamed", "RenamedInputs", 1, false),
        ("revalued", "LatchInputs", 2, false),
        ("shifted", "LatchInputs", 1, true),
        ("combined", "RenamedInputs", 2, true),
    ] {
        let prefix = if shifted { "prefix();" } else { "" };
        let source = format!(
            r#"final class {name} {{
static int calls, value, saved, events; static boolean boom, mutateBoom; static final RuntimeException FAILURE=new IllegalStateException("chosen");
static void prefix() {{ events=events+1; }}
static int limit() {{ calls=calls+1; if(boom) throw FAILURE; return value; }}
static void mutate() {{ saved=99; events=events+10; if(mutateBoom) throw FAILURE; }}
static int find(int[] a,int target,int lower) {{ {prefix} int i; for(i=saved=limit()-{delta};i>=lower;i--) {{ if(a[i]==target) return i; }} return -1; }}
static int shared(int lower) {{ {prefix} int held; int i=(held=limit())-{delta}; mutate(); while(i>=lower) {{ i--; }} return held*100+i; }}
static int alias(int lower) {{ {prefix} int i=saved=limit()-{delta}; mutate(); while(i>=lower) {{ i--; }} return i; }}
static int extra(int lower) {{ {prefix} int i=saved=limit()-{delta}; while(i>=lower) {{ i--; }} return i; }}
static int ownership(int[] a,int target,int lower) {{ {prefix} int i=saved=limit()-{delta}; mutate(); while(i>=lower) {{ if(a[i]==target) return i; i--; }} return -1; }}
}}"#
        );
        let driver = format!(
            r#"public class LatchDriver {{
static void reset() {{ {name}.calls=0; {name}.value={delta}+2; {name}.saved=77; {name}.events=0; {name}.boom=false; {name}.mutateBoom=false; }}
static void check(int actual,int expected,int saved,int events) {{ if(actual!=expected || {name}.saved!=saved || {name}.calls!=1 || {name}.events!=events) throw new AssertionError(actual+":"+{name}.saved+":"+{name}.calls+":"+{name}.events); System.out.println(actual+":"+saved+":"+events); }}
public static void main(String[] args) {{
reset(); check({name}.find(new int[]{{7,8,9}},9,0),2,2,{shift});
reset(); check({name}.find(new int[]{{7,8,9}},6,0),-1,2,{shift});
reset(); check({name}.find(null,6,3),-1,2,{shift});
reset(); try {{ {name}.find(null,6,0); throw new AssertionError(); }} catch(NullPointerException e) {{ check(0,0,2,{shift}); }}
reset(); try {{ {name}.find(new int[0],6,0); throw new AssertionError(); }} catch(ArrayIndexOutOfBoundsException e) {{ check(0,0,2,{shift}); }}
reset(); {name}.boom=true; try {{ {name}.find(null,6,0); throw new AssertionError(); }} catch(IllegalStateException e) {{ if(e!={name}.FAILURE || !"chosen".equals(e.getMessage())) throw new AssertionError(); check(0,0,77,{shift}); }}
reset(); check({name}.shared(0),({delta}+2)*100-1,99,{mutate});
reset(); {name}.mutateBoom=true; try {{ {name}.shared(0); throw new AssertionError(); }} catch(RuntimeException e) {{ if(e!={name}.FAILURE) throw new AssertionError(); check(0,0,99,{mutate}); }}
reset(); check({name}.alias(0),-1,99,{mutate});
reset(); check({name}.alias(100),2,99,{mutate});
reset(); check({name}.extra(0),-1,2,{shift});
reset(); check({name}.ownership(new int[]{{7,8,9}},9,0),2,99,{mutate});
}} }}"#,
            shift = usize::from(shifted),
            mutate = 10 + usize::from(shifted)
        );
        let renamed = case == "renamed" || case == "combined";
        let rename = |value: String| {
            if renamed {
                value
                    .replace("limit", "bound")
                    .replace("saved", "kept")
                    .replace("find", "seek")
                    .replace("shared", "joint")
            } else {
                value
            }
        };
        let source = rename(source);
        let driver = rename(driver);
        let find_name = if renamed { "seek" } else { "find" };
        let shared_name = if renamed { "joint" } else { "shared" };
        let root =
            std::env::temp_dir().join(format!("jarde-latch-input-{case}-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(format!("{name}.java")), source).unwrap();
        latch_javac(&root, &[&format!("{name}.java")]);
        let bytes = std::fs::read(root.join(format!("{name}.class"))).unwrap();
        let report = class_source_report(&bytes, name, None);
        let find = class_method(&report, find_name, "([III)I");
        assert!(find.text.contains("for ("), "{case}: {}", find.text);
        for (method, descriptor) in [
            (shared_name, "(I)I"),
            ("alias", "(I)I"),
            ("extra", "(I)I"),
            ("ownership", "([III)I"),
        ] {
            let member = class_method(&report, method, descriptor);
            assert!(
                member.text.contains("while (") && !member.text.contains("for ("),
                "{case}: {}",
                member.text
            );
            assert_physical_method_coverage(
                &bytes,
                method,
                descriptor,
                member,
                recovered_body(member),
            );
        }
        assert_physical_method_coverage(&bytes, find_name, "([III)I", find, recovered_body(find));
        let mut budget = task_budget(&[]).unwrap();
        let physical = inspect_method_bytecode(
            &bytes,
            MethodSelector {
                name: JvmBytes(find_name.as_bytes().to_vec()),
                descriptor: JvmBytes(b"([III)I".to_vec()),
            },
            &mut budget,
        )
        .unwrap();
        assert!(physical.stopped_at.is_none());
        let copies: Vec<_> = physical
            .instructions
            .iter()
            .filter(|i| i.opcode == 0x59)
            .collect();
        assert_eq!(copies.len(), 1);
        let copy_bci = copies[0].bci;
        let base = *unshifted_copy_bci.get_or_insert(copy_bci);
        assert_eq!(
            copy_bci,
            base + if shifted { 3 } else { 0 },
            "the observable prefix must actually shift the shared copy"
        );
        std::fs::write(root.join("physical-bytecode.txt"), format!("{physical:#?}")).unwrap();
        latch_whole_class_oracle(&bytes, name, &driver, case);
    }
}

// This derivative changes the ORIGINAL input inventory, never recovered text.
// Refused find/scoped remain covered independently on the complete frozen class.
fn computed_controls_retained_method_derivative() -> Vec<u8> {
    fn u2(bytes: &[u8], at: &mut usize) -> usize {
        let value = usize::from(u16::from_be_bytes(bytes[*at..*at + 2].try_into().unwrap()));
        *at += 2;
        value
    }
    fn attributes(bytes: &[u8], at: &mut usize) {
        let count = u2(bytes, at);
        for _ in 0..count {
            *at += 2;
            let length = u32::from_be_bytes(bytes[*at..*at + 4].try_into().unwrap()) as usize;
            *at += 4 + length;
            assert!(*at <= bytes.len());
        }
    }
    let bytes = COMPUTED_FOR_CONTROLS;
    assert_eq!(&bytes[..4], &[0xca, 0xfe, 0xba, 0xbe]);
    let mut at = 8;
    let count = u2(bytes, &mut at);
    let mut utf8 = vec![None; count];
    let mut classes = vec![0; count];
    let mut names = vec![(0, 0); count];
    let mut refs = Vec::new();
    let mut index = 1;
    while index < count {
        let tag = bytes[at];
        at += 1;
        match tag {
            1 => {
                let length = u2(bytes, &mut at);
                utf8[index] = Some(&bytes[at..at + length]);
                at += length;
            }
            3 | 4 => at += 4,
            5 | 6 => {
                at += 8;
                index += 1;
            }
            7 => classes[index] = u2(bytes, &mut at),
            8 | 16 | 19 | 20 => at += 2,
            9..=11 => {
                let class = u2(bytes, &mut at);
                let name = u2(bytes, &mut at);
                if tag != 9 {
                    refs.push((class, name));
                }
            }
            12 => names[index] = (u2(bytes, &mut at), u2(bytes, &mut at)),
            15 => at += 3,
            17 | 18 => at += 4,
            other => panic!("unexpected constant-pool tag {other}"),
        }
        index += 1;
    }
    at += 2;
    let this_class = u2(bytes, &mut at);
    at += 2;
    let interfaces = u2(bytes, &mut at);
    at += 2 * interfaces;
    let fields = u2(bytes, &mut at);
    for _ in 0..fields {
        at += 6;
        attributes(bytes, &mut at);
    }
    let methods_at = at;
    let methods = u2(bytes, &mut at);
    let mut retained = Vec::new();
    let mut removed = Vec::new();
    for _ in 0..methods {
        let start = at;
        at += 2;
        let name = u2(bytes, &mut at);
        let descriptor = u2(bytes, &mut at);
        attributes(bytes, &mut at);
        let identity = (utf8[name].unwrap(), utf8[descriptor].unwrap());
        if identity == (b"find".as_slice(), b"([III)I".as_slice())
            || identity == (b"scoped".as_slice(), b"(IZ)I".as_slice())
        {
            removed.push(identity);
        } else {
            retained.push(&bytes[start..at]);
        }
    }
    assert_eq!(removed.len(), 2);
    // Conservatively inspect every method reference in the unchanged pool,
    // including references unused by the retained Code attributes.
    for (class, name) in refs {
        if utf8[classes[class]] == utf8[classes[this_class]] {
            let (name, descriptor) = names[name];
            assert!(!removed.contains(&(utf8[name].unwrap(), utf8[descriptor].unwrap())));
        }
    }
    let suffix = &bytes[at..];
    let mut output = bytes[..methods_at].to_vec();
    output.extend_from_slice(&u16::try_from(retained.len()).unwrap().to_be_bytes());
    for record in &retained {
        output.extend_from_slice(record);
    }
    output.extend_from_slice(suffix);
    assert_eq!(&output[..methods_at], &bytes[..methods_at]);
    let mut check = methods_at + 2;
    for record in retained {
        assert_eq!(&output[check..check + record.len()], record);
        check += record.len();
    }
    assert_eq!(&output[check..], suffix);
    output
}

#[test]
#[ignore = "requires javac and java; explicit frozen whole-class JVM gate"]
fn frozen_shared_store_whole_class_matches_original_bytecode() {
    let driver = r#"public class LatchDriver {
static void reset(int value,boolean boom) { SharedStoreInput.limitCalls=0; SharedStoreInput.limitValue=value; SharedStoreInput.savedField=77; SharedStoreInput.throwFromLimit=boom; }
static void check(int actual,int expected,int saved) { if(actual!=expected || SharedStoreInput.limitCalls!=1 || SharedStoreInput.savedField!=saved) throw new AssertionError(actual+":"+SharedStoreInput.limitCalls+":"+SharedStoreInput.savedField); System.out.println(actual+":"+saved+":"+SharedStoreInput.limitCalls); }
public static void main(String[] args) {
reset(3,false); check(SharedStoreInput.find(new int[]{8,7,8},8,0),2,2);
reset(3,false); check(SharedStoreInput.find(new int[]{8,7,8},7,0),1,2);
reset(3,false); check(SharedStoreInput.find(new int[]{8,7,8},6,0),-1,2);
reset(3,false); check(SharedStoreInput.find(null,6,3),-1,2);
reset(0,false); check(SharedStoreInput.find(null,6,0),-1,-1);
reset(3,false); try { SharedStoreInput.find(null,6,0); throw new AssertionError(); } catch(NullPointerException e) { check(0,0,2); }
reset(3,false); try { SharedStoreInput.find(new int[0],6,0); throw new AssertionError(); } catch(ArrayIndexOutOfBoundsException e) { check(0,0,2); }
reset(3,true); try { SharedStoreInput.find(null,6,0); throw new AssertionError(); } catch(IllegalStateException e) { if(!"limit".equals(e.getMessage())) throw new AssertionError(); check(0,0,77); }
reset(Integer.MIN_VALUE,false); try { SharedStoreInput.find(null,6,Integer.MAX_VALUE); throw new AssertionError(); } catch(NullPointerException e) { check(0,0,Integer.MAX_VALUE); }
} }"#;
    latch_whole_class_oracle(
        SHARED_STACK_INPUT,
        "SharedStoreInput",
        driver,
        "frozen-shared-store",
    );
}

#[test]
#[ignore = "requires javac and java; exact original-method input derivative, not a frozen-whole pass"]
fn retained_computed_methods_match_exact_byte_input_derivative() {
    let bytes = computed_controls_retained_method_derivative();
    let driver = r#"public class LatchDriver {
static void reset(int end,int mode) { ComputedForControls.end=end; ComputedForControls.mode=mode; ComputedForControls.trace=0; }
static void check(int actual,int expected,int trace,int end) { if(actual!=expected || ComputedForControls.trace!=trace || ComputedForControls.end!=end) throw new AssertionError(actual+":"+ComputedForControls.trace+":"+ComputedForControls.end); System.out.println(actual+":"+trace+":"+end); }
public static void main(String[] args) {
reset(4,0); check(ComputedForControls.shared(0),399,12,2);
reset(4,0); check(ComputedForControls.shared(5),403,12,2);
reset(0,0); check(ComputedForControls.shared(0),-1,12,2);
reset(-2,0); check(ComputedForControls.shared(0),-203,12,2);
reset(4,1); try { ComputedForControls.shared(0); throw new AssertionError(); } catch(RuntimeException e) { if(e!=ComputedForControls.FAILURE) throw new AssertionError(); check(0,0,1,4); }
reset(4,2); try { ComputedForControls.shared(0); throw new AssertionError(); } catch(RuntimeException e) { if(e!=ComputedForControls.FAILURE) throw new AssertionError(); check(0,0,12,2); }
reset(4,0); check(ComputedForControls.observedAfter(0),-1,1,4);
reset(4,0); check(ComputedForControls.observedAfter(5),3,1,4);
reset(4,0); check(ComputedForControls.externalWrite(0),0,1,4);
reset(4,0); check(ComputedForControls.externalWrite(5),5,1,4);
reset(Integer.MAX_VALUE,0); check(ComputedForControls.shared(Integer.MAX_VALUE),2147483546,12,2);
} }"#;
    latch_whole_class_oracle(
        &bytes,
        "ComputedForControls",
        driver,
        "derived-computed-retained",
    );
}
