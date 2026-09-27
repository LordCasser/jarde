//! One named catch, two saved returns and one shared catch-all cleanup handler.

use jarde::*;
use std::collections::BTreeSet;
use std::slice;

const CALL: &[u8] =
    include_bytes!("fixtures/p3-shared-catchall-finally/v8/SharedFinallyCall.class");
const EXTRA_RETURN: &[u8] =
    include_bytes!("fixtures/p3-shared-catchall-finally/v8/SharedFinallyExtraReturn.class");
const OTHER_TARGET: &[u8] =
    include_bytes!("fixtures/p3-shared-catchall-finally/v8/SharedFinallyOtherTarget.class");

fn source(bytes: &[u8], class: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("bounded defaults");
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("class opens");
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
    match engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("class source completes")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one class has one definition: {other:?}"),
    }
}

fn handled(report: &ClassSourceReport) -> &ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"handled")
        .expect("handled method")
}

fn replace_once(class: &[u8], before: &[u8], after: &[u8]) -> Vec<u8> {
    assert_eq!(before.len(), after.len());
    let hits: Vec<_> = class
        .windows(before.len())
        .enumerate()
        .filter_map(|(index, bytes)| (bytes == before).then_some(index))
        .collect();
    assert_eq!(hits.len(), 1, "mutation must identify one physical fact");
    let mut changed = class.to_vec();
    changed[hits[0]..hits[0] + before.len()].copy_from_slice(after);
    changed
}

#[test]
fn shared_call_has_one_source_finally_and_all_physical_origins() {
    let report = source(CALL, "SharedFinallyCall");
    let method = handled(&report);
    let text = &method.text;
    assert!(
        text.contains("catch (java.lang.IllegalArgumentException"),
        "{text}"
    );
    assert!(text.contains("return \"normal\";"), "{text}");
    assert!(text.contains("return \"caught\";"), "{text}");
    assert_eq!(text.matches("} finally {").count(), 1, "{text}");
    assert_eq!(text.matches("cleanup();").count(), 1, "{text}");
    assert!(!text.contains("@bytecode"), "{text}");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("handled recovered: {:?}", method.outcome);
    };
    let origins: BTreeSet<u32> = report
        .source_map
        .segments()
        .iter()
        .flat_map(|segment| segment.origin().bcis())
        .collect();
    let physical = [
        0, 1, 4, 5, 8, 11, 12, 14, 17, 18, 20, 21, 24, 25, 26, 27, 29, 30, 33, 34, 35, 36, 39, 40,
    ];
    assert_eq!(origins, BTreeSet::from(physical));
}

#[test]
fn changed_rows_targets_or_extra_completion_cannot_claim_one_finally() {
    let named = [0, 4, 0, 21, 0, 26, 0, 13];
    let try_any = [0, 4, 0, 21, 0, 35, 0, 0];
    let row_pair = [named, try_any].concat();
    let swapped = replace_once(CALL, &row_pair, &[try_any, named].concat());
    let widened = replace_once(CALL, &try_any, &[0, 4, 0, 24, 0, 35, 0, 0]);
    // The catch copy alone calls another real static ()V member of this class. Both the
    // original and this one-index mutation verify under Java 8.
    let other_target = replace_once(
        OTHER_TARGET,
        &[0xb8, 0, 25, 0x2c, 0xb0],
        &[0xb8, 0, 13, 0x2c, 0xb0],
    );
    for (case, bytes) in [
        ("swapped", swapped),
        ("widened", widened),
        ("other_target", other_target),
        ("extra_return", EXTRA_RETURN.to_vec()),
    ] {
        let class = match case {
            "extra_return" => "SharedFinallyExtraReturn",
            "other_target" => "SharedFinallyOtherTarget",
            _ => "SharedFinallyCall",
        };
        let report = source(&bytes, class);
        let text = &handled(&report).text;
        assert!(!text.contains("} finally {"), "{case}: {text}");
        assert!(
            text.contains("@bytecode") || text.contains("stopped"),
            "{case}: {text}"
        );
    }
}
