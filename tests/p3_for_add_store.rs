//! Add/store loop updates move to a `for` header only when the latch owns one pure chain.

use jarde::*;
use std::slice;

const SIMPLE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/for-add-store-latch/simple/frozen/ForAddStoreSimple.class"
);
const LABELED: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/for-add-store-latch/frozen/ForAddStoreLatch.class"
);
const BOUNDARIES: &[u8] =
    include_bytes!("fixtures/p3-for-add-store/v8/ForAddStoreBoundaries.class");

fn method_text(class_bytes: &[u8], class_name: &str, method_name: &str) -> String {
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(class_bytes.to_vec()), &mut budget)
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
    let OperationOutcome::Performed(report) = engine
        .class_source(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("single-class request succeeds")
    else {
        panic!("one class-source result is expected")
    };
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == method_name.as_bytes())
        .unwrap_or_else(|| panic!("the class has no `{method_name}` member"))
        .text
        .clone()
}

#[test]
fn pure_add_store_latch_is_one_for_update() {
    let method = method_text(SIMPLE, "ForAddStoreSimple", "run");
    assert!(!method.contains("@bytecode"), "{method}");
    assert!(
        method.contains("for (local3 = 0; local3 < arg0; local3 = local3 + arg1)"),
        "{method}"
    );
    assert_eq!(
        method.matches("local3 = local3 + arg1").count(),
        1,
        "{method}"
    );
    assert!(method.contains("local2 = local2 + local3;"), "{method}");
}

#[test]
fn outer_continue_runs_the_update_and_skips_the_body_tail() {
    let method = method_text(LABELED, "ForAddStoreLatch", "run");
    assert!(!method.contains("@bytecode"), "{method}");
    assert!(
        method.contains("jarde_loop_8: for (local3 = 0; local3 < arg0; local3 = local3 + arg1)"),
        "{method}"
    );
    assert!(method.contains("continue jarde_loop_8;"), "{method}");
    assert!(
        method.contains("for (local6 = 0; local6 < 2; local6 = local6 + 1)"),
        "{method}"
    );
    assert_eq!(
        method.matches("local3 = local3 + arg1").count(),
        1,
        "{method}"
    );
    let continue_at = method.find("continue jarde_loop_8;").unwrap();
    let body_tail_at = method.find("local5 = local5 + 1;").unwrap();
    assert!(continue_at < body_tail_at, "{method}");
}

#[test]
fn changed_step_extra_consumer_shared_effect_and_post_use_do_not_move() {
    for name in [
        "changedStep",
        "extraConsumer",
        "sharedTailEffect",
        "observedAfter",
    ] {
        let method = method_text(BOUNDARIES, "ForAddStoreBoundaries", name);
        assert!(!method.contains("for ("), "{name}: {method}");
        assert!(method.contains("while ("), "{name}: {method}");
        if name == "extraConsumer" {
            assert!(method.contains("@bytecode 13"), "{method}");
        } else {
            assert!(!method.contains("@bytecode"), "{name}: {method}");
        }
    }
    let changed = method_text(BOUNDARIES, "ForAddStoreBoundaries", "changedStep");
    assert!(changed.contains("arg1 = arg1 + 1;"), "{changed}");
    assert!(changed.contains("local3 = local3 + arg1;"), "{changed}");
    let shared = method_text(BOUNDARIES, "ForAddStoreBoundaries", "sharedTailEffect");
    assert!(shared.contains("local3 = local3 + 100;"), "{shared}");
    assert!(shared.contains("local4 = local4 + arg1;"), "{shared}");
    let post = method_text(BOUNDARIES, "ForAddStoreBoundaries", "observedAfter");
    assert!(post.contains("return local2;"), "{post}");
}
