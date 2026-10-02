//! The nested `synchronized` regions `recover-nested-monitor-regions` proves: the paired second
//! `monitorenter`/`monitorexit` pair an outer `synchronized` body holds presents as the inner
//! block it is, the single-monitor and two-arm forms keep their exact text, and the shapes the
//! certificate does not prove keep their refusals (`jre_guard_monitor`, P3 2.4).

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

/// The patrol's frozen T4, whose `sync` is the shape the whole slice is anchored on.
const T4: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/compound-guard-patrol/fixture/T4.class"
);
/// The frozen variants: `v1` (the T4 form), `v2` (the inner pair inside the outer's loop), `v3`
/// (the inner body ends in the return), `n3` (three levels — stays refused) and `sr` (the same
/// lock re-entered, whose slot pairing proves and presents the reentrant block).
const NM: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/compound-guard-patrol/nested-mon/fixture/NM.class"
);
/// The inner pair's normal `monitorexit` patched to a `pop`: the pair has no normal exit to pair.
const N_MISSING_EXIT: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/compound-guard-patrol/nested-mon/fixture/NMissingExit.class"
);
/// The inner-position exit's load patched to the outer lock's slot: an unpaired inner exit.
const N_WRONG_EXIT_LOCK: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/compound-guard-patrol/nested-mon/fixture/NWrongExitLock.class"
);
/// The committed single-exit monitor and the committed two-arm one: their texts are the
/// byte-identity control for every shape this slice did not change.
const LOCKED: &[u8] = include_bytes!("../../../tests/fixtures/p3-sync-return/v8/Locked.class");
const MULTI_EXIT: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-22/synchronized-multi-exit/SynchronizedMultiExit.class"
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
fn the_patrols_t4_sync_presents_both_blocks_and_the_loop_between_them() {
    // T4.sync — `synchronized (T4.class) { int s = 0; synchronized (log) { for (…) s += i; }
    // return s; }` under `-g:none`: the loop lives inside the inner block, the `return` inside
    // the outer one, and no instruction stays quoted. (The single-method `recover` entry writes
    // the member's own braces around the statement list.)
    let text = recovered_text(T4, "sync", "(I)I", 1);
    assert_eq!(
        text,
        "// @method sync(I)I\n// @declaration a static method of `T4`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    synchronized (T4.class) {\n        int local2;\n        int local4;\n        local2 = 0;\n        synchronized (T4.log) {\n            for (local4 = 0; local4 < arg0; local4 = local4 + 1) {\n                local2 = local2 + local4;\n            }\n        }\n        return local2;\n    }\n}\n"
    );
}

#[test]
fn the_frozen_variants_present_and_the_three_level_form_stays_refused() {
    // v1 is the T4 form in its own frozen class; v2 is the pair the outer's own loop wraps; v3
    // is the pair whose body ends in the return the outer's exit shape writes inside its braces.
    let v1 = recovered_text(NM, "v1", "(I)I", 1);
    assert_eq!(
        v1,
        "// @method v1(I)I\n// @declaration a static method of `NM`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    synchronized (NM.class) {\n        int local2;\n        int local4;\n        local2 = 0;\n        synchronized (NM.log) {\n            for (local4 = 0; local4 < arg0; local4 = local4 + 1) {\n                local2 = local2 + local4;\n            }\n        }\n        return local2;\n    }\n}\n"
    );
    let v2 = recovered_text(NM, "v2", "(I)I", 1);
    assert_eq!(
        v2,
        "// @method v2(I)I\n// @declaration a static method of `NM`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    synchronized (NM.class) {\n        int local2;\n        int local3;\n        local2 = 0;\n        for (local3 = 0; local3 < arg0; local3 = local3 + 1) {\n            synchronized (NM.log) {\n                local2 = local2 + local3;\n            }\n        }\n        return local2;\n    }\n}\n"
    );
    let v3 = recovered_text(NM, "v3", "(I)I", 1);
    assert!(
        v3.contains("synchronized (NM.log)")
            && v3.contains("return local2;")
            && !v3.contains("@bytecode"),
        "the return stays inside the outer braces:\n{v3}"
    );
    // The same lock re-entered: javac gives each level its own slot, so the pairing proves both
    // and the presentation is the reentrant block the source wrote. The three-level form keeps
    // its refusal: one pair is the most this certificate presents.
    let sr = recovered_text(NM, "sr", "(Ljava/lang/Object;I)I", 2);
    assert_eq!(
        sr.matches("synchronized (arg0)").count(),
        2,
        "the reentrant pair presents nested:\n{sr}"
    );
    assert!(!sr.contains("@bytecode"), "{sr}");
    let n3 = recovered_text(NM, "n3", "(I)I", 1);
    assert!(
        n3.contains("@bytecode") && !n3.contains("synchronized ("),
        "three levels stay refused:\n{n3}"
    );
}

#[test]
fn the_unpaired_and_missing_exit_negatives_keep_their_refusals() {
    // Both patched classes verify (`-Xverify:all`, see the evidence directory) and keep the
    // whole-method quote: a pair whose normal exit is gone, and one whose exit leaves another
    // monitor's slot, are not shapes the pairing proves.
    for class in [N_MISSING_EXIT, N_WRONG_EXIT_LOCK] {
        let text = recovered_text(class, "m", "(I)I", 1);
        assert!(
            text.contains("@bytecode") && !text.contains("synchronized ("),
            "the unproved pair stays quoted:\n{text}"
        );
    }
}

#[test]
fn the_single_exit_and_two_arm_monitor_texts_are_byte_identical() {
    // The two committed monitor presentations this slice must not move: the return shape's own
    // fixture and the two-arm multi-exit one keep their exact texts.
    assert_eq!(
        recovered_text(LOCKED, "locked", "()I", 1),
        "// @method locked()I\n// @declaration an instance method of `Locked`, member flags 0x0000\n// recovered from bytecode; presentation is not claimed to compile\n{\n    synchronized (this) {\n        return this.n;\n    }\n}\n"
    );
    assert_eq!(
        recovered_text(MULTI_EXIT, "choose", "(Ljava/lang/Object;Z)I", 2),
        "// @method choose(Ljava/lang/Object;Z)I\n// @declaration a static method of `SynchronizedMultiExit`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    synchronized (arg0) {\n        if (arg1) {\n            return produce(1);\n        } else {\n            return produce(2);\n        }\n    }\n}\n"
    );
}
