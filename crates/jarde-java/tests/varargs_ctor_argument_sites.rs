//! The varargs inline array chains of a construction's argument run — `new ArrayList<>(Arrays
//! .asList(1, 2, 3))` and its forms — present through the array initializer spelling the bare
//! positions already use, and every existing construction presentation is byte-identical
//! (P3 2.3, `new@1` reading `array@1`).

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::classfile::class_facts;
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const W1: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/varargs-ctor-arg-patrol/fixture/W1.class"
);
const W3: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/varargs-ctor-arg-patrol/fixture/W3.class"
);
const W4: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/varargs-ctor-arg-patrol/fixture/W4.class"
);
const V1: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/varargs-ctor-arg-patrol/variants-vca/V1.class"
);
const V2: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/varargs-ctor-arg-patrol/variants-vca/V2.class"
);
const V3: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/varargs-ctor-arg-patrol/variants-vca/V3.class"
);
const X2: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/nested-ctor-argument-patrol/fixture/X2.class"
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
        class_headers: 100,
        method_bodies: 100,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn recovered_text(class: &[u8], name: &str, descriptor: &str, parameter_slots: u16) -> String {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("frozen class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: class.len() as u64,
        },
        variant: PhysicalVariant::Base,
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
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
            owner: definition,
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    let analysis =
        analyze_method_ir(&[snapshot], &request, &mut budget).expect("frozen method analyzes");
    let header = class_facts(class, &mut budget).expect("frozen class facts parse");
    let member = header
        .methods
        .iter()
        .find(|member| member.name.raw().0 == name.as_bytes())
        .expect("method exists in frozen class");
    let declaring_name = String::from_utf8_lossy(&header.this_class.raw().0).into_owned();
    let facts = RecoveryFacts::new(
        MethodFacts::new(
            name.to_owned(),
            String::from_utf8_lossy(&member.descriptor.raw().0).into_owned(),
            parameter_slots,
        )
        .with_access_flags(member.access_flags)
        .with_declaring_class(DeclaringClass::new(declaring_name, header.access_flags)),
    );
    let report = recover(
        &RecoveryRequest::new(analysis.ir(), &facts, jarde_java::pass::JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::essential()),
        &mut budget,
    );
    report.text
}

#[test]
fn the_patrol_anchors_present_the_varargs_array_argument() {
    // W3.viaArrays — the patrol's discriminating probe: the inline `Integer[]` store chain between
    // the outer `new`'s copy and its constructor, spelled as the initializer the bare positions use.
    assert_eq!(
        recovered_text(W3, "viaArrays", "()I", 0),
        "// @method viaArrays()I\n// @declaration a static method of `W3`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    return new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[]{java.lang.Integer.valueOf(1), java.lang.Integer.valueOf(2), java.lang.Integer.valueOf(3)})).size();\n}\n"
    );
    // W3.viaArraysEmpty — the empty varargs call is a bare allocation the factory call reads.
    assert_eq!(
        recovered_text(W3, "viaArraysEmpty", "()I", 0),
        "// @method viaArraysEmpty()I\n// @declaration a static method of `W3`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    return new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[0])).size();\n}\n"
    );
    // W1.use — the cascade the patrol froze: the whole chain through `sumExt(ints)` recovers.
    let use_body = recovered_text(W1, "use", "()Ljava/lang/String;", 0);
    assert!(
        use_body.contains(
            "java.util.ArrayList local0 = new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[]{java.lang.Integer.valueOf(1), java.lang.Integer.valueOf(2), java.lang.Integer.valueOf(3)}));"
        ) && use_body.contains("return \"\" + sumExt((java.util.List) local0) + \":\" + local1.get(0) + \":\" + nameOf(W1.class);"),
        "the varargs array argument chain and its consumers recover:\n{use_body}"
    );
    assert!(
        !use_body.contains("@bytecode"),
        "no instruction stays quoted:\n{use_body}"
    );
}

#[test]
fn every_bare_and_nested_position_presentation_is_byte_identical() {
    // The three bare varargs shapes that already recovered before this slice keep their exact
    // text: the array initializer spelling the argument positions now reuse is the same one.
    assert_eq!(
        recovered_text(W4, "bare", "()I", 0),
        "// @method bare()I\n// @declaration a static method of `W4`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    return java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[]{java.lang.Integer.valueOf(1), java.lang.Integer.valueOf(2)}).size();\n}\n"
    );
    assert_eq!(
        recovered_text(W4, "fmt", "()Ljava/lang/String;", 0),
        "// @method fmt()Ljava/lang/String;\n// @declaration a static method of `W4`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    return java.lang.String.format(\"%d-%d\", new java.lang.Object[]{java.lang.Integer.valueOf(1), java.lang.Integer.valueOf(2)});\n}\n"
    );
    assert_eq!(
        recovered_text(W4, "assigned", "()I", 0),
        "// @method assigned()I\n// @declaration a static method of `W4`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    java.util.List local0 = java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[]{java.lang.Integer.valueOf(5), java.lang.Integer.valueOf(6)});\n    return ((java.lang.Integer) local0.get(0)).intValue();\n}\n"
    );
    // The sister shape — a nested construction in an argument position — is a different branch of
    // the same walk and keeps its exact text.
    assert_eq!(
        recovered_text(X2, "nested", "()Ljava/lang/String;", 0),
        "// @method nested()Ljava/lang/String;\n// @declaration a static method of `X2`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    return fmt(new java.lang.Exception(\"outer\", (java.lang.Throwable) new java.lang.Exception(\"inner\")));\n}\n"
    );
}

#[test]
fn the_variant_chains_present_and_the_negatives_stay_quoted() {
    // The empty varargs call, the same-component boxed mix, the `HashSet` copy constructor and
    // call-produced elements all present through the one initializer spelling.
    for (name, expected) in [
        (
            "viaEmptyCall",
            "return new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.String[0])).size();",
        ),
        (
            "viaBoxedMix",
            "return new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[]{java.lang.Integer.valueOf(1), java.lang.Integer.valueOf(2), java.lang.Integer.valueOf(3)})).size();",
        ),
        (
            "viaHashSet",
            "return new java.util.HashSet((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.String[]{\"a\", \"b\"})).size();",
        ),
        (
            "viaCallElements",
            "return new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[]{java.lang.Integer.valueOf(7), box(8)})).size();",
        ),
    ] {
        let text = recovered_text(V1, name, "()I", 0);
        assert!(
            text.contains(expected) && !text.contains("@bytecode"),
            "{name} presents the complete chain:\n{text}"
        );
    }
    // The mixed-type varargs call keeps the refusal the bare positions keep: the array element
    // compatibility boundary is `array@1`'s, not the argument position's.
    let mixed = recovered_text(V3, "viaMixed", "()I", 0);
    assert!(
        mixed.contains("@bytecode") && mixed.contains("array component is `java.lang.Number`"),
        "the mixed-boxing chain keeps its element-compatibility refusal:\n{mixed}"
    );
    // A store inside the element run and an array that escapes to a second purpose keep the
    // construction's refusal: the body quotes bytecode and writes no partial `new` expression.
    for name in ["midStatement", "doubleUse"] {
        let text = recovered_text(V2, name, "()I", 0);
        assert!(
            text.contains("@bytecode") && !text.contains("new java.util.ArrayList"),
            "{name} stays refused:\n{text}"
        );
    }
}
