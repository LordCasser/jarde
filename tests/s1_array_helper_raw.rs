//! Array helper proof joins use the same-run physical site, then spell source names.
use jarde::*;

#[test]
fn array_helper_name_owner_and_descriptor_keep_supplementary_identity() {
    for (input, owner, factory, element) in [
        (
            include_bytes!("fixtures/s1-array-helper-raw/ascii.jar").as_slice(),
            "Subject",
            "arrayCtor",
            "int",
        ),
        (
            include_bytes!("fixtures/s1-array-helper-raw/helper-supplementary.jar"),
            "Subject",
            "𐐀",
            "int",
        ),
        (
            include_bytes!("fixtures/s1-array-helper-raw/owner-supplementary.jar"),
            "𐐀Subject",
            "arrayCtor",
            "int",
        ),
        (
            include_bytes!("fixtures/s1-array-helper-raw/element-supplementary.jar"),
            "Subject",
            "arrayCtor",
            "𐐀Item",
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
                panic!("the authored input must perform class-source recovery");
            };
            assert!(
                report.text.contains(&format!("{element}[]::new")),
                "{owner}.{factory}: {}",
                report.text
            );
            assert!(
                report.text.contains("array-constructor site(s)"),
                "{}",
                report.text
            );
            let raw_helper = JvmBytes::from_unicode(&format!("lambda${factory}$0"));
            assert!(
                report
                    .methods
                    .iter()
                    .any(|method| method.item.name.raw() == &raw_helper)
            );
            if let Some(text) = previous {
                assert_eq!(report.text, text);
            }
            previous = Some(report.text);
        }
    }
}
