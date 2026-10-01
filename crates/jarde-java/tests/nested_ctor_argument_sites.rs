//! The construction sites nested in another construction's argument positions present as the
//! one `new` expression the source had, and every existing construction presentation is
//! byte-identical (P3 2.3, `new@1`).

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

const X1: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/nested-ctor-argument-patrol/fixture/X1.class"
);
const X2: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/nested-ctor-argument-patrol/fixture/X2.class"
);
const X3: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/nested-ctor-argument-patrol/variants-nested/X3.class"
);
const X4: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/nested-ctor-argument-patrol/variants-nested/X4.class"
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
fn the_patrol_anchors_present_the_whole_nested_chain() {
    // X2.nested — the patrol's discriminating probe: `fmt(new Exception("outer", new
    // Exception("inner")))`, two sites of one and the same class.
    assert_eq!(
        recovered_text(X2, "nested", "()Ljava/lang/String;", 0),
        "// @method nested()Ljava/lang/String;\n// @declaration a static method of `X2`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    return fmt(new java.lang.Exception(\"outer\", (java.lang.Throwable) new java.lang.Exception(\"inner\")));\n}\n"
    );
    // X1.main — `readCause(new Exception("top", new Exception("inner")))`, the wrapped-exception
    // chain the patrol froze, now the third statement of a fully recovered body.
    let main = recovered_text(X1, "main", "([Ljava/lang/String;)V", 1);
    assert!(
        main.contains(
            "java.lang.System.out.println((java.lang.String) readCause((java.lang.Throwable) new java.lang.Exception(\"top\", (java.lang.Throwable) new java.lang.Exception(\"inner\"))));"
        ),
        "the nested cause chain is one statement:\n{main}"
    );
    assert!(
        !main.contains("@bytecode"),
        "no instruction stays quoted:\n{main}"
    );
}

#[test]
fn every_frozen_construction_presentation_is_byte_identical() {
    // The three shapes that already recovered before this slice — a sole construction as a
    // method argument, a plain allocation as a method argument and a StringBuilder receiver —
    // keep their exact text.
    assert_eq!(
        recovered_text(X2, "single", "()Ljava/lang/String;", 0),
        "// @method single()Ljava/lang/String;\n// @declaration a static method of `X2`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    return fmt(new java.lang.Exception(\"solo\"));\n}\n"
    );
    assert_eq!(
        recovered_text(X2, "plainNested", "()Ljava/lang/String;", 0),
        "// @method plainNested()Ljava/lang/String;\n// @declaration a static method of `X2`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    return java.lang.String.valueOf(new java.lang.Object());\n}\n"
    );
    assert_eq!(
        recovered_text(X2, "nestedNew", "()Ljava/lang/String;", 0),
        "// @method nestedNew()Ljava/lang/String;\n// @declaration a static method of `X2`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    return java.lang.String.valueOf((java.lang.Object) new java.lang.StringBuilder(\"sb\"));\n}\n"
    );
    // The throwable-wrap throw position and the initCause argument keep theirs, too.
    let wrap = recovered_text(X1, "wrapCtor", "(Ljava/lang/String;)Ljava/lang/String;", 1);
    assert!(
        wrap.contains(
            "java.lang.RuntimeException local2 = new java.lang.RuntimeException(\"wrapped\", (java.lang.Throwable) local1);"
        ) && wrap.contains("throw local2;"),
        "{wrap}"
    );
    let init = recovered_text(
        X1,
        "wrapInitCause",
        "(Ljava/lang/String;)Ljava/lang/String;",
        1,
    );
    assert!(
        init.contains(
            "local1.initCause((java.lang.Throwable) new java.lang.UnsupportedOperationException(\"root\"));"
        ),
        "{init}"
    );
}

#[test]
fn the_variant_constructions_present_and_the_negatives_stay_quoted() {
    // Two nested argument positions, the second position, and one nested class twice.
    assert_eq!(
        recovered_text(X3, "doubleNested", "()Ljava/lang/String;", 0),
        "// @method doubleNested()Ljava/lang/String;\n// @declaration a static method of `X3`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    return java.lang.String.valueOf((java.lang.Object) new X3$TwoNested(new X3$B(\"y\"), new X3$C(\"z\")));\n}\n"
    );
    assert_eq!(
        recovered_text(X3, "secondPosition", "()Ljava/lang/String;", 0),
        "// @method secondPosition()Ljava/lang/String;\n// @declaration a static method of `X3`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    return java.lang.String.valueOf((java.lang.Object) new X3$Tagged(\"first\", new X3$B(\"second\")));\n}\n"
    );
    assert_eq!(
        recovered_text(X3, "sameClassTwice", "()Ljava/lang/String;", 0),
        "// @method sameClassTwice()Ljava/lang/String;\n// @declaration a static method of `X3`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    return java.lang.String.valueOf((java.lang.Object) new X3$TwoSame(new X3$B(\"1\"), new X3$B(\"2\")));\n}\n"
    );
    // The three-layer run, the double-purpose nested value and the cross-block conditional keep
    // their refusals: the body quotes bytecode and writes no partial `new` expression.
    for (name, descriptor, quoted) in [
        ("threeLayer", "()Ljava/lang/String;", "new X4$Top"),
        ("doubleUse", "()Ljava/lang/String;", "new X4$Tag"),
        ("crossBlock", "(Z)Ljava/lang/String;", "new X4$Tag"),
    ] {
        let text = recovered_text(X4, name, descriptor, u16::from(name == "crossBlock"));
        assert!(
            text.contains("@bytecode") && !text.contains(quoted),
            "{name} stays refused:\n{text}"
        );
    }
}
