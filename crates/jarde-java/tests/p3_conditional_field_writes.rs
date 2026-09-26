//! A proved integer conditional consumed directly by boolean field stores must retain the
//! JVM's low-bit boolean conversion for both `putstatic Z` and `putfield Z`.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const CLASS_01: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-conditional-values/field-writes/ConditionalFieldWrites.class"
);
const CLASS_23: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-conditional-values/field-writes/ConditionalFieldWritesNon01.class"
);

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
        nested_depth: 16,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn recover_store(class: &[u8], name: &str, descriptor: &str) -> jarde_java::RecoveryReport {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("fixture opens");
    let method = PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(class).to_hex().to_string()),
                length: u64::try_from(class.len()).expect("fixture length fits"),
            },
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(name.as_bytes().to_vec()),
        descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
    };
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
    let analysis = analyze_method_ir(
        &[snapshot.clone()],
        &MethodAnalysisRequest {
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
            method,
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("fixture analysis completes");
    let facts = RecoveryFacts::new(
        MethodFacts::new(name, descriptor, 1)
            .with_access_flags(if name == "putStatic" { 0x0009 } else { 0x0001 })
            .with_declaring_class(DeclaringClass::new("ConditionalFieldWrites", 0x0031)),
    );
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    )
}

#[test]
fn proved_integer_conditionals_keep_jvm_boolean_field_store_semantics() {
    for (class, selected, other, output) in [
        (CLASS_01, "1", "0", "? 1 : 0"),
        (CLASS_23, "2", "3", "? 2 : 3"),
    ] {
        for (method, descriptor, field, field_access, store_bci) in [
            ("putStatic", "(Z)V", "staticFlag", "staticFlag =", 9),
            ("putInstance", "(Z)V", "instanceFlag", "instanceFlag =", 10),
        ] {
            let report = recover_store(class, method, descriptor);
            assert!(report.produced(), "{method}: {:?}", report.outcome);
            assert!(report.text.contains(output), "{method}: {}", report.text);
            assert!(
                report.text.contains(field_access),
                "{method}: {}",
                report.text
            );
            assert!(
                report.text.contains("% 2 != 0"),
                "{method}: {}",
                report.text
            );
            // The direct body report may carry unrelated local-placement references. The
            // consumer itself must still be emitted as the proved conditional field assignment.
            assert!(report.text.contains("?"), "{method}: {}", report.text);
            assert!(report.text.contains(field), "{method}: {}", report.text);
            assert!(report.text.contains(selected) && report.text.contains(other));
            assert!(
                !report.source_map.of_bci(store_bci).is_empty(),
                "{method}: field-store BCI {store_bci} has no source provenance"
            );
        }
    }

    for class in [CLASS_01, CLASS_23] {
        let report = recover_store(class, "putInteger", "(Z)V");
        assert!(report.produced(), "putInteger: {:?}", report.outcome);
        assert!(report.text.contains("integerField ="), "{}", report.text);
        assert!(report.text.contains("? 2 : 3"), "{}", report.text);
        assert!(!report.text.contains("% 2 != 0"), "{}", report.text);
    }
}
