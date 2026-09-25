//! A for header owns only a proved preheader local initialization and one CFG latch update.

use jarde::*;
use std::slice;

const CLASS: &[u8] = include_bytes!("fixtures/p3-for-latch/v8/ForBoundaries.class");

fn method_text(name: &str) -> String {
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
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
    let OperationOutcome::Performed(report) = Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("single-class request succeeds")
    else {
        panic!("one class-source result is expected")
    };
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("the class has no `{name}` member"))
        .text
        .clone()
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
