//! The first straight `finally` claim and its structured-body refusal boundary.

use jarde::*;
use std::collections::BTreeSet;
use std::slice;

const STRAIGHT: &[u8] = include_bytes!("fixtures/p3-finally-straight/v8/FinallyNormal.class");
const BRANCHING: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/finally-completion/implicit-cleanup/ImplicitCleanup.class"
);
const LEAD_SNAPSHOT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/finally-lead-snapshot/FinallyLeadSnapshot.class"
);
const COPY_DIVERGENT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/finally-completion/snapshot-boundaries/copy-divergence/CleanupBoundaries.class"
);
const RANGE_NARROWED: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/finally-completion/snapshot-boundaries/range-narrowed/CleanupBoundaries.class"
);

fn source(bytes: &[u8], class: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("bounded defaults");
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("frozen class opens");
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
        other => panic!("one frozen class has one definition: {other:?}"),
    }
}

fn run_text(report: &ClassSourceReport) -> &str {
    &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"run")
        .expect("run method")
        .text
}

#[test]
fn a_straight_proved_copy_has_one_cleanup_and_returns_the_saved_local() {
    let report = source(STRAIGHT, "FinallyNormal");
    let text = run_text(&report);
    assert!(
        text.contains("try {\n            int local0 = mark(1);\n            return local0;\n        } finally {\n            mark(2);\n        }"),
        "{text}"
    );
    assert_eq!(text.matches("mark(2)").count(), 1, "{text}");
    assert!(!text.contains("@bytecode"), "{text}");
    let run = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"run")
        .unwrap();
    let ClassSourceOutcome::Recovered { report, .. } = &run.outcome else {
        panic!("the complete run is recovered: {:?}", run.outcome);
    };
    let anchored: BTreeSet<u32> = report
        .source_map
        .segments()
        .iter()
        .flat_map(|segment| segment.origin().bcis())
        .collect();
    for bci in [0, 1, 4, 5, 6, 9, 10, 11, 12, 13, 14, 17, 18, 19] {
        assert!(anchored.contains(&bci), "BCI {bci}: {anchored:?}");
    }
}

#[test]
fn a_branching_protected_body_does_not_flatten_into_finally() {
    let report = source(BRANCHING, "ImplicitCleanup");
    let text = run_text(&report);
    assert!(
        text.contains("jre_guard_finally_copy") || text.contains("finally` copy"),
        "{text}"
    );
    assert!(!text.contains("} finally {"), "{text}");
    assert!(text.contains("@bytecode"), "{text}");
}

#[test]
fn a_straight_lead_stays_before_the_protected_snapshot_return() {
    let report = source(LEAD_SNAPSHOT, "FinallyLeadSnapshot");
    let text = run_text(&report);
    assert!(
        text.contains(
            "FinallyLeadSnapshot.value = 41;\n        try {\n            int local0 = FinallyLeadSnapshot.value;\n            return local0;\n        } finally {\n            FinallyLeadSnapshot.value = 99;\n        }"
        ),
        "{text}"
    );
    assert_eq!(text.matches("value = 41").count(), 1, "{text}");
    assert_eq!(text.matches("value = 99").count(), 1, "{text}");
    assert!(!text.contains("@bytecode"), "{text}");
    let run = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"run")
        .unwrap();
    let ClassSourceOutcome::Recovered { report, .. } = &run.outcome else {
        panic!("the complete run is recovered: {:?}", run.outcome);
    };
    let anchored: BTreeSet<u32> = report
        .source_map
        .segments()
        .iter()
        .flat_map(|segment| segment.origin().bcis())
        .collect();
    for bci in [0, 2, 5, 8, 9, 11, 14, 15, 16, 17, 19, 22, 23] {
        assert!(anchored.contains(&bci), "BCI {bci}: {anchored:?}");
    }
}

#[test]
fn a_stack_value_crossing_the_protected_boundary_is_quoted() {
    // Move only start_pc from 5 to 2. The class still verifies, but BCI 0's pushed 41 now
    // survives across the exception boundary until the BCI 2 field write consumes it.
    let mut class = LEAD_SNAPSHOT.to_vec();
    let row = [0, 5, 0, 9, 0, 16, 0, 0];
    let offsets: Vec<usize> = class
        .windows(row.len())
        .enumerate()
        .filter_map(|(index, window)| (window == row).then_some(index))
        .collect();
    assert_eq!(
        offsets.len(),
        1,
        "the frozen class has one exact exception row"
    );
    class[offsets[0] + 1] = 2;
    let report = source(&class, "FinallyLeadSnapshot");
    let text = run_text(&report);
    assert!(!text.contains("} finally {"), "{text}");
    assert!(text.contains("@bytecode"), "{text}");
}

#[test]
fn a_divergent_copy_or_narrowed_range_stays_quoted() {
    for class in [COPY_DIVERGENT, RANGE_NARROWED] {
        let report = source(class, "CleanupBoundaries");
        let method = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"snapshotReturn")
            .unwrap();
        let text = &method.text;
        assert!(!text.contains("} finally {"), "{text}");
        assert!(text.contains("@bytecode"), "{text}");
    }
}
