//! The exact setter descriptor stays raw through field/frame proof and paired emission.
use jarde::class_source::ClassSourceOutcome;
use jarde::*;
use jarde_jvm::ir::Quality;

#[test]
fn write_accessor_owner_value_and_array_descriptors_keep_raw_identity() {
    for (input, owner, value_descriptor) in [
        (
            include_bytes!("fixtures/s1-write-accessor-raw/ascii.jar").as_slice(),
            "Subject",
            "I",
        ),
        (
            include_bytes!("fixtures/s1-write-accessor-raw/owner-supplementary.jar"),
            "𐐀Subject",
            "I",
        ),
        (
            include_bytes!("fixtures/s1-write-accessor-raw/value-supplementary.jar"),
            "Subject",
            "L𐐀Item;",
        ),
        (
            include_bytes!("fixtures/s1-write-accessor-raw/array-supplementary.jar"),
            "Subject",
            "[L𐐀Item;",
        ),
    ] {
        let mut previous = None;
        for evidence in [
            RecoveryEvidenceRequest::essential(),
            RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
            RecoveryEvidenceRequest::all(),
        ] {
            let engine = Engine::new();
            let mut budget = task_budget(&[]).unwrap();
            let snapshot = engine
                .open(ArtifactInput::bytes(input.to_vec()), &mut budget)
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
            let OperationOutcome::Performed(report) = engine
                .class_source_with_evidence(&[snapshot], &request, &evidence, &mut budget)
                .unwrap()
            else {
                panic!("authored class source performs");
            };
            let helper = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"access$002")
                .unwrap();
            assert_eq!(
                helper.item.descriptor.raw(),
                &JvmBytes::from_unicode(&format!(
                    "(L{owner};{value_descriptor}){value_descriptor}"
                ))
            );
            let ClassSourceOutcome::Recovered { report: body, .. } = &helper.outcome else {
                panic!("helper does not recover: {}", helper.text);
            };
            assert_eq!(body.quality, Quality::Structured, "{}", body.text);
            assert!(body.text.contains("arg0.value = arg1;"), "{}", body.text);
            assert!(body.text.contains("return arg1;"), "{}", body.text);
            if let Some(text) = previous {
                assert_eq!(report.text, text);
            }
            previous = Some(report.text);
        }
    }
}
