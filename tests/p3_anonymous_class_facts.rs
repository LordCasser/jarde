//! The selected anonymous class's typed nesting facts are handed into class-source assembly.
//!
//! The class remains a separate physical declaration in task 5.1; inlining and sibling resolution
//! belong to task 5.3. This exercises the existing Java 8 capture fixture without changing its
//! source spelling or the class-source report shape.

use jarde::*;

const ANONYMOUS_CLASS: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-capture/AnonymousCaptureCases$1.class"
);

#[test]
fn selected_anonymous_class_stays_a_separate_class_source_declaration() {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("task defaults provide a bounded budget");
    let snapshot = engine
        .open(ArtifactInput::bytes(ANONYMOUS_CLASS.to_vec()), &mut budget)
        .expect("the compiled Java 8 anonymous class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("AnonymousCaptureCases$1"),
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

    let report = match engine
        .class_source(std::slice::from_ref(&snapshot), &request, &mut budget)
        .expect("the class-source request is valid")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => {
            panic!(
                "expected one anonymous class, found {}",
                candidates.candidates.len()
            )
        }
        OperationOutcome::Incomplete(candidates) => {
            panic!(
                "anonymous class selection stopped with {} candidates",
                candidates.candidates.len()
            )
        }
    };

    assert!(report.text.contains("class AnonymousCaptureCases$1"));
    assert!(report.declaration.is_some());
}
