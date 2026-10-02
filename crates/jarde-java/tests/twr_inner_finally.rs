//! The inner explicit `finally` a `try (…)` body holds (`recover-twr-inner-finally`): the inner
//! `try (…) { … } finally { … }` a compound guard lowering writes into the guarded body presents
//! as the nested statement it is, the plain TWR and plain `finally` families keep their exact
//! texts, and the shapes the certificate does not prove — an inner row overlapping the resource
//! suppression machinery among them — keep their whole-method refusals (`jre_guard_finally_copy`,
//! P3 2.4).

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

/// The patrol's frozen T4, whose `nested` is the shape the whole slice is anchored on.
const T4: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/compound-guard-patrol/fixture/T4.class"
);
/// The frozen variants: `v1` (outer TWR + plain inner finally), `v2` (double TWR, no finally),
/// `v3` (the inner finally's guarded body returns), `nested` (the T4 form in its own class) and
/// `solo` (a TWR with its own finally and no enclosing TWR — not this certificate's shape, and
/// it keeps its refusal).
const TF: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/compound-guard-patrol/twrfin/fixture/TF.class"
);
/// The frozen negatives, both verifier-valid single-field patches of T4's own table (see the
/// evidence directory): the inner finally row widened past the inner statement's lowering into
/// the enclosing cleanup and the resource suppression row, and the self-protection row widened
/// over the inner TWR's own suppression machinery.
const T4_OVERLAP: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/compound-guard-patrol/twrfin/fixture/T4Ov.class"
);
const T4_SELF_ROW: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/compound-guard-patrol/twrfin/fixture/T4Self.class"
);
/// The committed pure-domain control: the plain one-resource TWR and the plain straight `finally`
/// whose texts this slice must not move.
const GUARDED: &[u8] = include_bytes!("../../../tests/fixtures/p3-handlers/v8/Guarded.class");

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
fn the_patrols_t4_nested_presents_the_inner_try_finally_in_order() {
    // T4.nested — `try (a) { try (b) { body } finally { mid } }` under `-g:none`: the inner
    // statement carries its own resource between its braces, the mid cleanup runs inside the
    // enclosing braces after the inner statement, and the method's own continuation follows the
    // statement. The close order the presentation states is the bytecode's own: body, b, mid, a.
    let text = recovered_text(T4, "nested", "()Ljava/lang/String;", 0);
    assert_eq!(
        text,
        "// @method nested()Ljava/lang/String;\n// @declaration a static method of `T4`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    try (T4 local0 = new T4(\"a\")) {\n        try (T4 local1 = new T4(\"b\")) {\n            T4.log.append(\"body\");\n        } finally {\n            T4.log.append(\"mid\");\n        }\n    }\n    return T4.log.toString();\n}\n"
    );
}

#[test]
fn the_frozen_variants_present_their_forms_and_the_solo_stays_refused() {
    // v1: the inner statement declares no resource of its own. v2: no finally at all — the flat
    // multi-resource presentation, which the fused-continuation reading now completes. v3: the
    // inner finally's guarded body returns; the saved value's declaration stays inside the
    // braces and the `return` reads it there, before the finally the lowering placed after it.
    let v1 = recovered_text(TF, "v1", "()Ljava/lang/String;", 0);
    assert_eq!(
        v1,
        "// @method v1()Ljava/lang/String;\n// @declaration a static method of `TF`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    try (TF local0 = new TF(\"a\")) {\n        try {\n            TF.log.append(\"body\");\n        } finally {\n            TF.log.append(\"mid\");\n        }\n    }\n    return TF.log.toString();\n}\n"
    );
    let v2 = recovered_text(TF, "v2", "()Ljava/lang/String;", 0);
    assert_eq!(
        v2,
        "// @method v2()Ljava/lang/String;\n// @declaration a static method of `TF`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    try (TF local0 = new TF(\"a\"); TF local1 = new TF(\"b\")) {\n        TF.log.append(\"body\");\n    }\n    return TF.log.toString();\n}\n"
    );
    let v3 = recovered_text(TF, "v3", "()I", 0);
    assert_eq!(
        v3,
        "// @method v3()I\n// @declaration a static method of `TF`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    try (TF local0 = new TF(\"a\")) {\n        try {\n            TF.log.append(\"body\");\n            int local1 = TF.log.length();\n            return local1;\n        } finally {\n            TF.log.append(\"mid\");\n        }\n    }\n}\n"
    );
    let nested = recovered_text(TF, "nested", "()Ljava/lang/String;", 0);
    assert!(
        nested.contains("try (TF local1 = new TF(\"b\"))") && nested.contains("} finally {"),
        "the T4 form presents nested in its own class:\n{nested}"
    );
    assert!(!nested.contains("@bytecode"), "{nested}");
    // A TWR with a `finally` of its own — no enclosing `try (…)` body around it — is a compound
    // this slice does not claim (the finally is the TWR's own clause, not a body form): the
    // method keeps the refusal it had.
    let solo = recovered_text(TF, "solo", "()Ljava/lang/String;", 0);
    assert!(
        solo.contains("@bytecode") && !solo.contains("finally {"),
        "the TWR-own finally stays quoted:\n{solo}"
    );
}

#[test]
fn the_overlapping_inner_rows_keep_their_refusals() {
    // Both patched classes verify (`-Xverify:all`, see the evidence directory) and keep the
    // whole-method quote: an inner finally row that reaches past the inner statement into the
    // enclosing cleanup — over the resource suppression row itself — and a self-protection row
    // widened over the inner TWR's own suppression machinery are not shapes this certificate
    // proves.
    for class in [T4_OVERLAP, T4_SELF_ROW] {
        let text = recovered_text(class, "nested", "()Ljava/lang/String;", 0);
        assert!(
            text.contains("@bytecode") && !text.contains("finally {"),
            "the overlapped shape stays quoted:\n{text}"
        );
    }
}

#[test]
fn the_plain_twr_and_plain_finally_texts_are_byte_identical() {
    // The committed pure-domain presentations this slice must not move: the guarded single
    // resource's body and the straight `finally` copy's own statement keep their exact texts.
    let one = recovered_text(GUARDED, "one", "()V", 0);
    assert_eq!(
        one,
        "// @method one()V\n// @declaration a static method of `Guarded`, member flags 0x0008\n// recovered from bytecode; presentation is not claimed to compile\n{\n    try (Res local0 = open(\"r\")) {\n        body();\n    }\n    return;\n}\n"
    );
    let run = recovered_text(
        include_bytes!("../../../tests/fixtures/p3-finally-straight/v8/FinallyNormal.class"),
        "run",
        "()I",
        0,
    );
    assert_eq!(
        run,
        "// @method run()I\n// @declaration a static method of `FinallyNormal`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    try {\n        int local0 = mark(1);\n        return local0;\n    } finally {\n        mark(2);\n    }\n}\n"
    );
}
