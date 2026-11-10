//! The public class-source entry must honor a selected physical Header over the platform table.
//! The override is intentionally not a valid `ArrayList` implementation of `List`; it is never
//! loaded or executed. See `fixtures/s2a-reference-relations/README.md` and its input manifest.

use jarde::*;
use jarde_jvm::ir::{Quality, Representation};
use std::slice;

const MANIFEST: &str = include_str!("fixtures/s2a-reference-relations/manifest.json");

fn class_source(jar: &[u8]) -> ClassSourceReport {
    named_class_source(jar, "Consumer", EnvironmentPolicy::PlainJar)
}

fn named_class_source(bytes: &[u8], name: &str, policy: EnvironmentPolicy) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("task defaults are bounded");
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("the pinned JAR fixture opens");
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".into()),
        },
    };

    match engine
        .class_source(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("the fixed class-source request completes")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source selection: {other:?}"),
    }
}

fn run_report(report: &ClassSourceReport) -> &RecoveryReport {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"run")
        .expect("Consumer declares run(ArrayList)");
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("run body was not analyzed: {other:?}"),
    }
}

fn fixture_fake_header_pin(compiler: &str) -> serde_json::Value {
    let manifest: serde_json::Value =
        serde_json::from_str(MANIFEST).expect("fixture manifest is valid JSON");
    manifest["fixtures"]
        .as_array()
        .expect("manifest lists fixtures")
        .iter()
        .find(|fixture| {
            fixture["compiler"].as_str() == Some(compiler)
                && fixture["variant"].as_str() == Some("override")
        })
        .and_then(|fixture| fixture.get("fake_header"))
        .expect("override row pins its physical Header")
        .clone()
}

fn assert_selected_fake_header(report: &RecoveryReport, compiler: &str) {
    let expected = fixture_fake_header_pin(compiler);
    assert_eq!(expected["entry_name"], "java/util/ArrayList.class");
    assert_eq!(expected["this_class"], "java/util/ArrayList");
    assert_eq!(expected["super_class"], "java/lang/Object");
    assert_eq!(expected["interfaces_count"], 0);

    let binding = report
        .artifact
        .binding()
        .expect("produced recovery has a read-owned artifact binding");
    let selected = binding
        .dependencies()
        .iter()
        .find(|dependency| {
            dependency
                .definition
                .location
                .entry()
                .is_some_and(|entry| entry.raw_name.0 == b"java/util/ArrayList.class")
        })
        .expect("the physical ArrayList class-path entry is a selected dependency");
    assert_eq!(
        selected.definition.class_bytes.digest.0.as_str(),
        expected["class_bytes_blake3"]
            .as_str()
            .expect("manifest has the fake Header's BLAKE3 classBytes pin")
    );
}

#[test]
fn selected_physical_arraylist_header_blocks_unsupported_list_conversion() {
    let cases: [(&str, &[u8], &[u8]); 2] = [
        (
            "corretto8",
            include_bytes!("fixtures/s2a-reference-relations/corretto8-normal.jar"),
            include_bytes!("fixtures/s2a-reference-relations/corretto8-override.jar"),
        ),
        (
            "openjdk23",
            include_bytes!("fixtures/s2a-reference-relations/openjdk23-normal.jar"),
            include_bytes!("fixtures/s2a-reference-relations/openjdk23-override.jar"),
        ),
    ];

    for (compiler, normal_jar, override_jar) in cases {
        let normal = class_source(normal_jar);
        let normal_run = run_report(&normal);
        assert_eq!(normal_run.representation, Representation::Java);
        assert_eq!(normal_run.quality, Quality::Structured);
        assert_eq!(normal_run.content, RecoveryContent::ContainsStatements);
        assert!(matches!(&normal_run.outcome, RecoveryOutcome::Produced));
        assert!(
            normal_run
                .text
                .contains("return consume((java.util.List) arg0);"),
            "{}",
            normal_run.text
        );

        let overridden = class_source(override_jar);
        let override_run = run_report(&overridden);
        assert_eq!(override_run.representation, Representation::Mixed);
        assert_eq!(override_run.quality, Quality::Fallback);
        assert_eq!(override_run.content, RecoveryContent::ExplanationOnly);
        assert!(matches!(&override_run.outcome, RecoveryOutcome::Produced));
        assert!(
            !override_run
                .text
                .contains("return consume((java.util.List) arg0);"),
            "{}",
            override_run.text
        );
        assert!(
            override_run.text.contains("relation: Some(Incompatible)"),
            "{}",
            override_run.text
        );
        assert_selected_fake_header(override_run, compiler);
    }
}

#[test]
fn selected_physical_parent_edge_does_not_require_target_member_header() {
    let jars: [&[u8]; 4] = [
        include_bytes!(
            "fixtures/s2a-reference-relations/selected-parent-edge/binary/PhysicalEdge-corretto8-debug.jar"
        ),
        include_bytes!(
            "fixtures/s2a-reference-relations/selected-parent-edge/binary/PhysicalEdge-corretto8-no-debug.jar"
        ),
        include_bytes!(
            "fixtures/s2a-reference-relations/selected-parent-edge/binary/PhysicalEdge-openjdk23-debug.jar"
        ),
        include_bytes!(
            "fixtures/s2a-reference-relations/selected-parent-edge/binary/PhysicalEdge-openjdk23-no-debug.jar"
        ),
    ];
    for jar in jars {
        let report = named_class_source(jar, "PhysicalEdge", EnvironmentPolicy::PlainJar);
        let method = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"test")
            .unwrap();
        let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
            panic!("test has Code");
        };
        assert_eq!(body.quality, Quality::Structured, "{}", body.text);
        assert!(method.markers.is_empty(), "{:?}", method.markers);
        assert!(
            body.text
                .contains("consume((EdgeTarget) new PhysicalEdge())"),
            "{}",
            body.text
        );
        assert!(
            body.artifact
                .binding()
                .unwrap()
                .dependencies()
                .iter()
                .all(|dependency| {
                    dependency
                        .definition
                        .location
                        .entry()
                        .is_none_or(|entry| entry.raw_name.0 != b"EdgeTarget.class")
                }),
            "the fixture supplies only PhysicalEdge.class"
        );
    }
}
