//! Try/catch locals retain lexical scope and are lifted only when SSA proves the join value.

use jarde::*;
use std::slice;

const FIXTURE: &[u8] =
    include_bytes!("../tests/fixtures/p3-exception-scope/v8/ExceptionScope.class");

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 100,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 10,
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

fn fixture(engine: &Engine) -> (ArtifactSnapshot, ClassBytesId) {
    let mut budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(FIXTURE.to_vec()), &mut budget)
        .expect("the fixture opens as a class");
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the fixture header is readable");
    (snapshot, inspected.source.class_bytes)
}

fn recover(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    bytes: &ClassBytesId,
    name: &[u8],
) -> String {
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let request = MethodAnalysisRequest {
        environment: ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: domain.clone(),
            },
            domains: vec![domain],
            providers: Vec::new(),
        },
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(b"(Z)I".to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    let mut budget = Budget::new(limits());
    let recovered = engine
        .recover_method(slice::from_ref(snapshot), &request, &mut budget)
        .expect("a legal request completes");
    let report = recovered.recovery();
    assert!(
        report.produced(),
        "{}: {:?}",
        String::from_utf8_lossy(name),
        report.outcome
    );
    assert_eq!(
        report.representation,
        Representation::Java,
        "{}",
        report.text
    );
    report.text.clone()
}

#[test]
fn handler_local_stays_in_the_catch_and_both_assignments_can_join() {
    let engine = Engine::new();
    let (snapshot, bytes) = fixture(&engine);

    let catch = recover(&engine, &snapshot, &bytes, b"catchOnly");
    let catch_at = catch
        .find("catch (")
        .expect("the handler is structured: {catch}");
    let local_at = catch
        .find("int local2 = 2;")
        .expect("the catch-local declaration is preserved: {catch}");
    let catch_close = catch[catch_at..]
        .find("\n    }")
        .map(|offset| catch_at + offset)
        .expect("the catch body has a lexical end: {catch}");
    assert!(catch_at < local_at && local_at < catch_close, "{catch}");
    assert!(
        !catch[catch_close..].contains("local2"),
        "catch local escaped: {catch}"
    );

    let joined = recover(&engine, &snapshot, &bytes, b"assignedAcrossTry");
    let declaration = joined
        .find("int local1;")
        .expect("the shared local is declared in the common method block: {joined}");
    let try_at = joined
        .find("try {")
        .expect("the protected region is structured: {joined}");
    assert!(declaration < try_at, "{joined}");
    assert!(
        joined.contains("local1 = 3;"),
        "normal path stores into the lifted local: {joined}"
    );
    assert!(
        joined.contains("local1 = 4;"),
        "handler stores into the lifted local: {joined}"
    );
    assert!(
        joined.contains("return local1;"),
        "the join reads the proven local: {joined}"
    );
}
