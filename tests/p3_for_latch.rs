//! A for header owns only a proved preheader local initialization and one CFG latch update.

use jarde::*;
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use std::slice;

const CLASS: &[u8] = include_bytes!("fixtures/p3-for-latch/v8/ForBoundaries.class");
const CF07: &[u8] = include_bytes!("fixtures/p3-loop-terminal/LoopCases.class");
const DIRECT_CALL_INIT: &[u8] = include_bytes!(
    "../openspec/changes/recover-proved-computed-for-initializers/results/controls-baseline-root-v1/direct-return-actual/javac8/classes/ComputedForDirectReturn.class"
);
const SHARED_STACK_INPUT: &[u8] = include_bytes!(
    "../openspec/changes/recover-proved-computed-for-initializers/results/shared-stack-input-root-v1/javac8/original/classes/SharedStoreInput.class"
);
const COMPUTED_FOR_CONTROLS: &[u8] = include_bytes!(
    "../openspec/changes/recover-proved-computed-for-initializers/results/controls-baseline-root-v1/actual-controls/javac8/original/ComputedForControls.class"
);

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
fn shared_stack_initializer_refuses_a_for_header_with_complete_physical_sources() {
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
    assert_eq!(body.quality, Quality::Fallback);
    assert_eq!(body.content, RecoveryContent::ExplanationOnly);
    assert_eq!(body.syntax_status, SyntaxStatus::NotJava);
    assert!(!method.text.contains("for ("), "{}", method.text);
    assert!(
        method
            .text
            .contains("copy at BCI 5 has no proved local assignment"),
        "{}",
        method.text
    );
    assert!(method.text.contains("the counted loop's preceding local initialisation did not produce one Java header clause"), "{}", method.text);
    assert_physical_method_coverage(SHARED_STACK_INPUT, "find", "([III)I", method, body);
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
    "../openspec/changes/recover-proved-computed-for-initializers/results/no-handler-call-order-baseline-root-v1/actual-v2/jdk8/original/ComputedForEffectControl.class",
);
const CALL_ORDER_PENDING: &[u8] = include_bytes!(
    "../openspec/changes/recover-proved-computed-for-initializers/results/no-handler-call-order-baseline-root-v1/actual-v2/jdk8/pending/ComputedForEffectControl.class",
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
fn computed_for_controls_keep_fallback_boundaries_and_evidence_parity() {
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
        ("shared", "(I)I", "no proved local assignment"),
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
