//! P3 3.1: a named catch can rejoin inside the loop that protects it.

use jarde::*;
use std::slice;

const CLASS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/loop-try-handler-entry/LoopTryHandlerEntry.class"
);
const ARGS_CLASS: &[u8] =
    include_bytes!("fixtures/p3-loop-try-handler-entry/v8/LoopTryHandlerEntryArgs.class");
const SHARED_CLASS: &[u8] =
    include_bytes!("fixtures/p3-loop-try-handler-entry/v8/LoopTrySharedHandler.class");

fn recover(bytes: &[u8], class: &str) -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("class opens");
    let request = request(&snapshot, class);
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("class-source request succeeds")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected outcome: {other:?}"),
    }
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

#[test]
fn catch_rejoins_inside_its_enclosing_loop() {
    let report = recover(ARGS_CLASS, "LoopTryHandlerEntryArgs");
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"loopTry")
        .expect("loopTry member");
    assert!(method.text.contains("while"), "{}", method.text);
    assert!(
        method.text.contains("catch (java.lang.RuntimeException"),
        "{}",
        method.text
    );
    assert!(!method.text.contains("@bytecode"), "{}", method.text);
    let ClassSourceOutcome::Recovered {
        report: recovered, ..
    } = &method.outcome
    else {
        panic!("loopTry did not recover: {:?}", method.outcome);
    };
    for (bci, expected) in [
        (7, "maybeFail(arg0, arg1)"),
        (17, "arg2 = -1"),
        (21, "arg0 = arg0 - 1"),
        (26, "return arg2"),
    ] {
        assert!(
            recovered
                .source_map
                .text_of_bci(&recovered.text, bci)
                .iter()
                .any(|piece| piece.contains(expected)),
            "BCI {bci} lost `{expected}`: {:?}",
            recovered.source_map.of_bci(bci)
        );
    }
    assert!(recovered.source_map.of_bci(15).iter().any(|segment| {
        segment.text(&recovered.text).contains("try")
            && segment
                .origin()
                .derived()
                .iter()
                .any(|origin| origin.bci() == 15)
    }));
    for bci in [17, 18] {
        assert!(
            recovered
                .source_map
                .of_bci(bci)
                .iter()
                .any(|segment| { segment.origin().primary().bci() == bci }),
            "BCI {bci} lost its direct origin"
        );
    }
}

fn with_external_protection(bytes: &[u8], row_index: usize) -> Vec<u8> {
    // `javap -c` fixes these exception-table rows at [4,12) -> 15. Mutate only the
    // chosen row's start to BCI 0, so its stated range also covers the loop test.
    let mut class = bytes.to_vec();
    let rows: Vec<usize> = class
        .windows(6)
        .enumerate()
        .filter_map(|(at, window)| (window == [0, 4, 0, 12, 0, 15]).then_some(at))
        .collect();
    assert!(rows.len() > row_index);
    class[rows[row_index] + 1] = 0;
    class
}

#[test]
fn a_handler_with_incompatible_table_range_is_refused() {
    let malformed = with_external_protection(ARGS_CLASS, 0);
    let report = recover(&malformed, "LoopTryHandlerEntryArgs");
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"loopTry")
        .expect("loopTry member");
    assert!(
        method.text.contains("the graph is not reducible"),
        "{}",
        method.text
    );
}

#[test]
fn every_row_sharing_a_handler_must_belong_to_the_loop() {
    let malformed = with_external_protection(SHARED_CLASS, 1);
    let report = recover(&malformed, "LoopTrySharedHandler");
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"loopTry")
        .expect("loopTry member");
    assert!(
        method.text.contains("the graph is not reducible"),
        "{}",
        method.text
    );
}

#[test]
fn a_preinitialized_local_keeps_its_computed_try_update_and_post_loop_read() {
    let report = recover(CLASS, "LoopTryHandlerEntry");
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"loopTry")
        .expect("loopTry member");
    assert!(!method.text.contains("@bytecode"), "{}", method.text);
    assert!(method.text.contains("int local2;"), "{}", method.text);
    assert!(method.text.contains("while (arg0 > 0)"), "{}", method.text);
    assert!(
        method
            .text
            .contains("local2 = local2 + maybeFail(arg0, arg1)"),
        "{}",
        method.text
    );
    assert!(
        method.text.contains("catch (java.lang.RuntimeException"),
        "{}",
        method.text
    );
    assert!(method.text.contains("return local2;"), "{}", method.text);
    let ClassSourceOutcome::Recovered {
        report: recovered, ..
    } = &method.outcome
    else {
        panic!("loopTry did not recover: {:?}", method.outcome);
    };
    for (bci, expected) in [
        (1, "local2 = 0"),
        (9, "maybeFail(arg0, arg1)"),
        (13, "local2 = local2 + maybeFail(arg0, arg1)"),
        (19, "local2 = -1"),
        (27, "local2"),
    ] {
        assert!(
            recovered
                .source_map
                .text_of_bci(&recovered.text, bci)
                .iter()
                .any(|piece| piece.contains(expected)),
            "BCI {bci} lost `{expected}`: {:?}",
            recovered.source_map.of_bci(bci)
        );
    }
}

#[test]
fn an_unproved_computed_write_refuses_its_cross_region_read() {
    let mut unsupported = CLASS.to_vec();
    // Change only the sum's opcode to `iand`; it remains a valid, protected int computation,
    // but this proof does not claim its producer tree can be published as the required assignment.
    let positions: Vec<_> = unsupported
        .windows(3)
        .enumerate()
        .filter_map(|(at, bytes)| (bytes == [0x60, 0x3d, 0xa7]).then_some(at))
        .collect();
    assert_eq!(positions.len(), 1);
    unsupported[positions[0]] = 0x7e;
    let report = recover(&unsupported, "LoopTryHandlerEntry");
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"loopTry")
        .expect("loopTry member");
    assert!(
        method.text.contains("local 2 crosses a protected region"),
        "{}",
        method.text
    );
    assert!(method.text.contains("@bytecode"), "{}", method.text);
    assert!(
        method.text.contains("@bytecode 0 2 6 17 20 27"),
        "{}",
        method.text
    );
    assert!(!method.text.contains("return local2;"), "{}", method.text);
}

#[test]
fn budget_and_cancellation_publish_no_partial_loop() {
    let engine = Engine::new();
    let mut open_budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = engine
        .open(ArtifactInput::bytes(ARGS_CLASS.to_vec()), &mut open_budget)
        .expect("class opens");
    let request = request(&snapshot, "LoopTryHandlerEntryArgs");
    let limited = engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(Limits {
                output_bytes: 0,
                ..Limits::default()
            }),
        )
        .expect("budget stop is an operation outcome");
    assert!(
        matches!(limited, OperationOutcome::Incomplete(_)),
        "{limited:?}"
    );

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::with_cancellation_token(Limits::default(), token),
        )
        .expect("cancellation is an operation outcome");
    assert!(
        matches!(cancelled, OperationOutcome::Incomplete(_)),
        "{cancelled:?}"
    );
}
