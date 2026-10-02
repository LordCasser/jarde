//! The `finally` clause a `try (…)` statement carries **directly** (`recover-twr-direct-finally`):
//! `try (r) { … } finally { … }` presents with the header unchanged and the clause's body after
//! the braces, the patrol's P3/P1 anchors and the frozen variants recover, and the shapes the
//! certificate does not prove — a `finally` that returns, the three-clause form, a broken copy
//! link, a row over the copies — keep their whole-method refusals.

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

/// The patrol's frozen P3 and P1: the void-body and compound-body anchors the slice states.
const P3: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/twr-clause-patrol/fixture/P3.class"
);
const P1: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/twr-clause-patrol/fixture/P1.class"
);
/// The frozen direct-finally variants (`t[c]f` base, multi-level outer clause, a body that
/// returns through the clause) and the two registered refusals (a `finally` that itself returns,
/// the three-clause form).
const PD: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/twr-clause-patrol/direct/fixture/PD.class"
);
/// The frozen negatives, both verifier-valid single-field patches of PD's own table (see the
/// evidence directory): the normal copy's `ldc` repointed so the two copies disagree, and the
/// multi-resource self-protection row widened over the exceptional copy the handler runs.
const PD_BROKEN: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/twr-clause-patrol/direct/fixture/PDBroken.class"
);
const PD_OVERLAP: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/twr-clause-patrol/direct/fixture/PDOv.class"
);
/// The committed pure-domain control: the plain one-resource TWR and the plain straight `finally`
/// whose texts this slice must not move.
const GUARDED: &[u8] = include_bytes!("../../../tests/fixtures/p3-handlers/v8/Guarded.class");
/// The inner-form control: the TWR-inside-finally shape stays the inner certificate's, and the
/// direct clause never claims its row (the anchors are disjoint: table tail vs body).
const TF: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/compound-guard-patrol/twrfin/fixture/TF.class"
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
fn the_patrols_p3_and_p1_anchors_present_the_direct_clause() {
    // P3.voidBodySoloFin (`t[c]f`, the void body) and P1.soloFinally (`b[c]f`, the compound
    // body) — the two shapes the patrol refused whole (`jre_guard_finally_copy`): the header
    // keeps its own resource, the clause's body renders after the braces, and the method's own
    // return follows the statement.
    let void_body = recovered_text(P3, "voidBodySoloFin", "()Ljava/lang/String;", 0);
    assert_eq!(
        void_body,
        "// @method voidBodySoloFin()Ljava/lang/String;\n// @declaration a static method of `P3`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    try (P3 local0 = new P3()) {\n        touch(local0);\n    } finally {\n        P3.log.append(\"f\");\n    }\n    return P3.log.toString();\n}\n"
    );
    let compound = recovered_text(P1, "soloFinally", "()Ljava/lang/String;", 0);
    assert_eq!(
        compound,
        "// @method soloFinally()Ljava/lang/String;\n// @declaration a static method of `P1`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    try (P1 local0 = new P1()) {\n        P1.log.append(\"b\");\n    } finally {\n        P1.log.append(\"f\");\n    }\n    return P1.log.toString();\n}\n"
    );
}

#[test]
fn the_frozen_variants_present_their_forms() {
    // `soloFin` is the P3 form in its own class; `multiFin` is the two-level shape with the
    // outer statement carrying the clause — the flat multi-resource presentation the TWR
    // certificate already states, with the clause after it (the self-protection row javac adds
    // over the handler's binding store is part of the claim); `bodyReturn` is the body that
    // returns through the clause — the saved value's declaration and `return` stay inside the
    // braces, before the clause the lowering placed after them.
    let solo = recovered_text(PD, "soloFin", "()Ljava/lang/String;", 0);
    assert_eq!(
        solo,
        "// @method soloFin()Ljava/lang/String;\n// @declaration a static method of `PD`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    try (PD local0 = new PD()) {\n        touch(local0);\n    } finally {\n        PD.log.append(\"f\");\n    }\n    return PD.log.toString();\n}\n"
    );
    let multi = recovered_text(PD, "multiFin", "()Ljava/lang/String;", 0);
    assert_eq!(
        multi,
        "// @method multiFin()Ljava/lang/String;\n// @declaration a static method of `PD`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    try (PD local0 = new PD(); PD local1 = new PD()) {\n        touch(local1);\n    } finally {\n        PD.log.append(\"f\");\n    }\n    return PD.log.toString();\n}\n"
    );
    let body = recovered_text(PD, "bodyReturn", "()Ljava/lang/String;", 0);
    assert_eq!(
        body,
        "// @method bodyReturn()Ljava/lang/String;\n// @declaration a static method of `PD`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    try (PD local0 = new PD()) {\n        touch(local0);\n        java.lang.String local1 = PD.log.toString();\n        return local1;\n    } finally {\n        PD.log.append(\"f\");\n    }\n}\n"
    );
}

#[test]
fn the_registered_refusals_keep_their_quotes() {
    // `finReturn` — the clause that itself returns — has no rethrow for the exceptional copy to
    // end in, so it is no `finally` copy pair this certificate reads; `threeClauses` —
    // TWR + catch + finally — is the three-clause form out of this change's scope, and
    // presenting the clause it proves would drop the clause the trailing certificate owns. Both
    // keep the whole-method quote they had.
    for name in ["finReturn", "threeClauses"] {
        let text = recovered_text(PD, name, "()Ljava/lang/String;", 0);
        assert!(
            text.contains("@bytecode") && !text.contains("finally {"),
            "{name} stays quoted:\n{text}"
        );
    }
}

#[test]
fn the_patched_negatives_stay_refused_and_their_neighbours_recover() {
    // Both patches verify (`-Xverify:all`, see the evidence directory). `PDBroken` repoints one
    // `ldc` of the normal copy, so the two copies disagree and the clause is no copy pair;
    // `PDOv` widens the multi-resource self-protection row over the exceptional copy the
    // handler runs, a row that reaches past the statement's own machinery. Each negative's own
    // method keeps its quote while the untouched sibling in the same class still recovers —
    // the refusal is the clause's, not the class's.
    let broken = recovered_text(PD_BROKEN, "soloFin", "()Ljava/lang/String;", 0);
    assert!(
        broken.contains("@bytecode") && !broken.contains("finally {"),
        "the broken copy link stays quoted:\n{broken}"
    );
    let broken_sibling = recovered_text(PD_BROKEN, "multiFin", "()Ljava/lang/String;", 0);
    assert!(
        broken_sibling.contains("} finally {") && !broken_sibling.contains("@bytecode"),
        "the untouched multi-level form still recovers:\n{broken_sibling}"
    );
    let overlap = recovered_text(PD_OVERLAP, "multiFin", "()Ljava/lang/String;", 0);
    assert!(
        overlap.contains("@bytecode") && !overlap.contains("finally {"),
        "the overlapped row stays quoted:\n{overlap}"
    );
    let overlap_sibling = recovered_text(PD_OVERLAP, "soloFin", "()Ljava/lang/String;", 0);
    assert!(
        overlap_sibling.contains("} finally {") && !overlap_sibling.contains("@bytecode"),
        "the untouched single-resource form still recovers:\n{overlap_sibling}"
    );
}

#[test]
fn the_inner_form_and_the_pure_domains_are_unchanged() {
    // The inner certificate's own anchor is a row that starts inside the body, and the direct
    // clause's is the whole statement's first instruction: the two never trade shapes, and
    // `TF.solo` — the direct form the inner slice left refused — now presents as this slice's
    // statement. The pure TWR and pure `finally` families keep their exact texts.
    let v1 = recovered_text(TF, "v1", "()Ljava/lang/String;", 0);
    assert_eq!(
        v1,
        "// @method v1()Ljava/lang/String;\n// @declaration a static method of `TF`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    try (TF local0 = new TF(\"a\")) {\n        try {\n            TF.log.append(\"body\");\n        } finally {\n            TF.log.append(\"mid\");\n        }\n    }\n    return TF.log.toString();\n}\n"
    );
    let solo = recovered_text(TF, "solo", "()Ljava/lang/String;", 0);
    assert_eq!(
        solo,
        "// @method solo()Ljava/lang/String;\n// @declaration a static method of `TF`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    try (TF local0 = new TF(\"b\")) {\n        TF.log.append(\"body\");\n    } finally {\n        TF.log.append(\"mid\");\n    }\n    return TF.log.toString();\n}\n"
    );
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
