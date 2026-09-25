//! The ordinary-interface initializer verdict is an all-or-nothing proof over the class's
//! complete field table and the same-run `<clinit>` candidate sequence.

use jarde::*;
use std::slice;

const POSITIVE: &[u8] =
    include_bytes!("fixtures/p3-interface-field-initializers/v8/InterfaceInitProbe.class");
const EXTRA_EFFECT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/boundaries/extra-effect/BoundaryProbe.class"
);
const DUPLICATE_AND_MISSING: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/boundaries/duplicate-write/BoundaryProbe.class"
);
const BRANCH: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/boundaries/branch/BoundaryProbe.class"
);
const FORWARD_READ: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/forward-binding-exception-edge/forward-binding/ForwardProbe.class"
);
const EXCEPTION_EDGE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/forward-binding-exception-edge/exception-handler/ExceptionProbe.class"
);
const CONSTANT_PHASE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/constant-phase-boundary/PhaseProbe.class"
);

fn budget() -> Budget {
    task_budget(&[]).expect("task defaults are a bounded budget")
}

fn source(bytes: &[u8], name: &str) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the checked-in class fixture opens");
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
    match Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut budget())
        .expect("the fixture answers one class-source request")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one fixture class answers one definition, got {other:?}"),
    }
}

fn runtime_field_indices(report: &ClassSourceReport, names: &[&str]) -> Vec<u64> {
    names
        .iter()
        .map(|name| {
            report
                .fields
                .iter()
                .find(|field| field.item.name.raw().0 == name.as_bytes())
                .unwrap_or_else(|| panic!("fixture has no field `{name}`"))
                .item
                .index
        })
        .collect()
}

fn assert_refused_without_partial_initializers(report: &ClassSourceReport) {
    assert!(
        matches!(
            &report.initializer_proof,
            ClassSourceInitializerProof::Refused { .. }
        ),
        "the complete group must be refused: {:?}",
        report.initializer_proof
    );
    for field in &report.fields {
        if field.item.name.raw().0 != b"CONSTANT" {
            assert!(
                field
                    .declaration
                    .as_deref()
                    .is_some_and(|text| !text.contains(" = ")),
                "a refused group leaves `{}` without a partial initializer: {:?}",
                String::from_utf8_lossy(&field.item.name.raw().0),
                field.declaration
            );
        }
    }
    let clinit = report
        .methods
        .iter()
        .find(|method| method.item.identity.name.0 == b"<clinit>")
        .expect("a rejected runtime initializer keeps its original member record");
    let ClassSourceOutcome::Recovered { report: body, .. } = &clinit.outcome else {
        panic!(
            "a structural refusal does not discard `<clinit>` recovery: {:?}",
            clinit.outcome
        );
    };
    assert!(
        body.produced(),
        "the original `<clinit>` artifact stays available"
    );
    assert!(
        report.text.contains(&clinit.text),
        "a rejected group keeps the original initializer effects in source text"
    );
}

#[test]
fn proves_all_runtime_writes_and_keeps_constantvalue_separate() {
    let report = source(POSITIVE, "InterfaceInitProbe");
    let ClassSourceInitializerProof::Proved { fields } = &report.initializer_proof else {
        panic!(
            "the ordinary straight-line initializer group should prove: {:?}",
            report.initializer_proof
        );
    };
    assert_eq!(
        fields
            .iter()
            .map(|field| field.field_index)
            .collect::<Vec<_>>(),
        runtime_field_indices(&report, &["FIRST", "SECOND", "TOTAL"]),
        "the proof follows actual `<clinit>` writes while retaining physical field identity"
    );
    assert_eq!(
        fields
            .iter()
            .map(|field| field.write_order)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(
        fields.len(),
        3,
        "CONSTANT is initialized in the JVM preparation phase"
    );
    let constant_index = runtime_field_indices(&report, &["CONSTANT"])[0];
    assert!(
        fields
            .iter()
            .all(|field| field.field_index != constant_index)
    );
    assert!(
        report.fields.iter().any(|field| {
            field.item.name.raw().0 == b"CONSTANT"
                && field
                    .declaration
                    .as_deref()
                    .is_some_and(|d| d.contains("= 7"))
        }),
        "the independent ConstantValue field remains its own declaration fact"
    );
}

#[test]
fn rejects_unclaimed_effects_duplicate_and_missing_writes_and_branches_as_whole_groups() {
    for (bytes, name) in [
        (EXTRA_EFFECT, "BoundaryProbe"),
        (DUPLICATE_AND_MISSING, "BoundaryProbe"),
        (BRANCH, "BoundaryProbe"),
    ] {
        let report = source(bytes, name);
        assert_refused_without_partial_initializers(&report);
    }
}

#[test]
fn admits_a_qualified_forward_read_only_when_original_write_time_is_preserved() {
    let report = source(FORWARD_READ, "ForwardProbe");
    let ClassSourceInitializerProof::Proved { fields } = &report.initializer_proof else {
        panic!(
            "the BCI-preserving source order should prove: {:?}",
            report.initializer_proof
        );
    };
    assert_eq!(
        fields
            .iter()
            .map(|field| field.field_index)
            .collect::<Vec<_>>(),
        runtime_field_indices(&report, &["EARLY", "LATE"]),
        "the physical field table is swapped, while proof order follows `<clinit>`"
    );
    assert_eq!(
        fields
            .iter()
            .map(|field| field.write_order)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
}

#[test]
fn refuses_exception_edges_and_runtime_constants_that_would_change_initialization_phase() {
    let exception = source(EXCEPTION_EDGE, "ExceptionProbe");
    assert_refused_without_partial_initializers(&exception);

    let phase = source(CONSTANT_PHASE, "PhaseProbe");
    let ClassSourceInitializerProof::Refused { reason } = &phase.initializer_proof else {
        panic!(
            "a runtime literal cannot become a ConstantValue: {:?}",
            phase.initializer_proof
        );
    };
    assert!(
        reason.contains("constant expression"),
        "the refusal explains the phase-boundary reason: {reason}"
    );
    assert_refused_without_partial_initializers(&phase);
}
