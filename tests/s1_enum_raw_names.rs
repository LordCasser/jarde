//! Enum proof consumers preserve raw physical names and share the Java source-name boundary.
use jarde::*;

fn requested(
    bytes: &[u8],
    owner: &str,
    evidence: RecoveryEvidenceRequest,
) -> Result<OperationOutcome<ClassSourceReport>> {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(owner),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".into()),
        },
    };
    engine.class_source_with_evidence(&[snapshot], &request, &evidence, &mut budget)
}

fn recovered(bytes: &[u8], owner: &str, evidence: RecoveryEvidenceRequest) -> ClassSourceReport {
    match requested(bytes, owner, evidence).unwrap() {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source outcome: {other:?}"),
    }
}

fn check(bytes: &[u8], owner: &str, expected: &str) {
    let mut previous = None;
    for evidence in [
        RecoveryEvidenceRequest::essential(),
        RecoveryEvidenceRequest::all(),
    ] {
        let report = recovered(bytes, owner, evidence);
        assert!(
            report.projection_inputs.enum_projection.is_some(),
            "missing enum projection: {}",
            report.text
        );
        assert!(
            report.text.contains(expected),
            "missing {expected}: {}",
            report.text
        );
        if let Some(text) = previous {
            assert_eq!(report.text, text);
        }
        previous = Some(report.text);
    }
}

#[test]
fn ordinary_enum_name_controls_remain_projected() {
    check(
        include_bytes!("fixtures/s1-enum-raw-names/int-control.jar"),
        "Subject",
        "ZERO(1)",
    );
    check(
        include_bytes!("fixtures/s1-enum-raw-names/terminal-control.jar"),
        "Subject",
        "Effects.record(arg0)",
    );
    check(
        include_bytes!("fixtures/s1-enum-raw-names/map-control.jar"),
        "Subject",
        "BY_NAME",
    );
}

#[test]
fn enum_constant_and_argument_consumers_keep_raw_names() {
    check(
        include_bytes!("fixtures/s1-enum-raw-names/constant-name.jar"),
        "Subject",
        "𐐀ZERO(1)",
    );
    check(
        include_bytes!("fixtures/s1-enum-raw-names/condition-name.jar"),
        "Subject",
        "𐐀choose()",
    );
    check(
        include_bytes!("fixtures/s1-enum-raw-names/int-field.jar"),
        "Subject",
        "Args.𐐀number",
    );
    check(
        include_bytes!("fixtures/s1-enum-raw-names/object-field.jar"),
        "Subject",
        "Args.𐐀object",
    );
    check(
        include_bytes!("fixtures/s1-enum-raw-names/arbitrary-field.jar"),
        "Subject",
        "𐐀narrow",
    );
    check(
        include_bytes!("fixtures/s1-enum-raw-names/object-parameter.jar"),
        "Subject",
        "𐐀Token",
    );
}

#[test]
fn terminal_enum_constructor_uses_shared_identifier_and_class_spelling() {
    check(
        include_bytes!("fixtures/s1-enum-raw-names/terminal-helper-name.jar"),
        "Subject",
        "Effects.𐐀record(arg0)",
    );
    check(
        include_bytes!("fixtures/s1-enum-raw-names/terminal-helper-owner.jar"),
        "Subject",
        "𐐀Effects.record(arg0)",
    );
    check(
        include_bytes!("fixtures/s1-enum-raw-names/terminal-field.jar"),
        "Subject",
        "this.𐐀value = arg0",
    );
}

#[test]
fn enum_map_suffix_uses_the_same_raw_name_boundary() {
    check(
        include_bytes!("fixtures/s1-enum-raw-names/map-field.jar"),
        "Subject",
        "𐐀BY_NAME",
    );
    check(
        include_bytes!("fixtures/s1-enum-raw-names/map-owner.jar"),
        "𐐀Subject",
        "𐐀Subject",
    );
}

#[test]
fn enum_static_field_owner_uses_the_shared_class_name_boundary() {
    check(
        include_bytes!("fixtures/s1-enum-raw-names/object-parameter-underscore.jar"),
        "Subject",
        "_.VALUE",
    );
    check(
        include_bytes!("fixtures/s1-enum-raw-names/int-field-owner-underscore.jar"),
        "Subject",
        "_.Xnumber",
    );
    check(
        include_bytes!("fixtures/s1-enum-raw-names/object-field-owner-underscore.jar"),
        "Subject",
        "_.Xobject",
    );
}

#[test]
fn enum_constant_bodies_keep_supplementary_owner_descriptors_raw() {
    let name = "demo/𐐀LabeledOps";
    let mut previous = None;
    for evidence in [
        RecoveryEvidenceRequest::essential(),
        RecoveryEvidenceRequest::all(),
    ] {
        let report = recovered(
            include_bytes!("fixtures/s1-enum-raw-names/body-owner.jar"),
            name,
            evidence,
        );
        let projection = report.projection_inputs.enum_projection.as_ref().expect(
            "a legal supplementary owner must retain the complete constant-body enum projection",
        );
        assert_eq!(projection.constant_field_indices, [0, 1]);
        assert!(projection.constants_text.contains("TIMES(\"*\") {"));
        assert!(projection.constants_text.contains("DIVIDE(\"/\") {"));
        assert!(report.text.contains("this.label = arg0;"));
        assert!(
            !report
                .text
                .contains("public static final demo.𐐀LabeledOps TIMES;")
        );
        assert_eq!(
            report.class.entry().unwrap().raw_name.0,
            format!("{name}.class").as_bytes(),
        );
        let declaration = report.declaration.as_ref().unwrap();
        assert_eq!(
            declaration.item.declaration.this_class.raw(),
            &JvmBytes::from_unicode(name)
        );
        for field in report
            .fields
            .iter()
            .filter(|field| field.item.access_flags & 0x4000 != 0)
        {
            assert_eq!(
                field.item.descriptor.raw().0,
                b"Ldemo/\xed\xa0\x81\xed\xb0\x80LabeledOps;"
            );
            assert_eq!(field.item.identity.owner, report.class);
        }
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        if let Some(text) = previous {
            assert_eq!(report.text, text);
        }
        previous = Some(report.text);
    }
}

#[test]
fn constant_body_unspellable_owner_stays_an_explicit_source_refusal() {
    for evidence in [
        RecoveryEvidenceRequest::essential(),
        RecoveryEvidenceRequest::all(),
    ] {
        let result = requested(
            include_bytes!("fixtures/s1-enum-raw-names/body-owner-unspellable.jar"),
            "demo/Bad-LabeledOps",
            evidence,
        );
        assert!(matches!(result, Err(Error::Unsupported { code, .. })
            if code == "class_source_name_unspellable"));
    }
}

#[test]
fn constant_body_bad_mutf8_owner_keeps_failed_selection_and_zero_body_work() {
    for evidence in [
        RecoveryEvidenceRequest::essential(),
        RecoveryEvidenceRequest::all(),
    ] {
        let result = requested(
            include_bytes!("fixtures/s1-enum-raw-names/body-owner-bad-mutf8.jar"),
            "demo/BadLabeledOps",
            evidence,
        )
        .unwrap();
        let OperationOutcome::Incomplete(selection) = result else {
            panic!("damaged classfile names must not select a recovery subject");
        };
        let ExecutionReport::Failed { usage, .. } = &selection.execution else {
            panic!("damaged MUTF-8 must remain a failed selection");
        };
        assert!(
            selection
                .diagnostics
                .iter()
                .any(|d| d.code == "classfile_modified_utf8_invalid")
        );
        assert_eq!(usage.method_bodies, 0);
        assert_eq!(usage.ir_items, 0);
        assert!(selection.candidates.is_empty());
    }
}
