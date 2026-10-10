//! Permanent source-map regression test draft for frozen discarded-call origins.
//!
//! This is a private proposal: it has not been compiled or run and is not yet in the repository.

use jarde_java::{
    ArtifactSubject, DebugLocal, DeclaringClass, MethodFacts, RecoveryEvidenceKind,
    RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::classfile::{
    LocalDebugTable, LocalDebugTypeTable, MethodSelector, inspect_method_bytecode,
};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation,
    PhysicalDefinitionId, PhysicalMethodId, PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode,
    MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty,
    RuntimeView,
};

const PROBE_CLASS: &[u8] = include_bytes!(
    "../../../openspec/changes/preserve-proved-discarded-call-origins/results/baseline-root-v1/javac23/original/discardprobe/DiscardedCallSourceProbe.class"
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

// Keep this byte-for-byte aligned with src/facade.rs::debug_locals: this is the same MethodIr
// reader table, not a test-authored local name or range.
fn reader_debug_locals(
    code: Option<&jarde_reader::classfile::MethodCodeFacts>,
) -> Vec<DebugLocal> {
    let Some(code) = code else {
        return Vec::new();
    };
    let LocalDebugTable::Read(records) = code.debug() else {
        return Vec::new();
    };
    let generic_records = match code.generic_debug() {
        LocalDebugTypeTable::Read(records) => records.as_slice(),
        LocalDebugTypeTable::Absent | LocalDebugTypeTable::Unstated => &[],
    };
    let mut local_counts = std::collections::BTreeMap::new();
    for record in records {
        let key = (
            record.slot,
            record.start_bci,
            record.end_bci,
            record.name.0.as_slice(),
        );
        *local_counts.entry(key).or_insert(0usize) += 1;
    }
    let mut generic_by_local = std::collections::BTreeMap::new();
    for generic in generic_records {
        let key = (
            generic.slot,
            generic.start_bci,
            generic.end_bci,
            generic.name.0.as_slice(),
        );
        generic_by_local
            .entry(key)
            .or_insert_with(Vec::new)
            .push(generic);
    }
    records
        .iter()
        .map(|record| {
            let key = (
                record.slot,
                record.start_bci,
                record.end_bci,
                record.name.0.as_slice(),
            );
            let local = DebugLocal::over(
                record.slot,
                record.name_lossy(),
                record.start_bci,
                record.end_bci,
            );
            if local_counts.get(&key) == Some(&1)
                && let Some([generic]) = generic_by_local.get(&key).map(Vec::as_slice)
            {
                local.with_type_metadata(
                    record.descriptor.0.clone(),
                    generic.signature.0.clone(),
                )
            } else {
                local
            }
        })
        .collect()
}

fn recover_class_method_with_budget(
    class: &[u8],
    owner: &str,
    name: &str,
    descriptor: &str,
    evidence: RecoveryEvidenceRequest,
    recovery_budget: Option<Budget>,
) -> (jarde_java::RecoveryReport, PhysicalMethodId) {
    let mut analysis_budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut analysis_budget)
        .expect("frozen class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("frozen class length fits"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: JvmBytes(name.as_bytes().to_vec()),
        descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
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
            method: method.clone(),
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut analysis_budget,
    )
    .expect("frozen method completes real IR analysis");
    // Match the production facade's trusted subject: analysis report identity/environment come
    // from this exact analysis, while the member ordinal comes from a reader lookup over the same
    // original class bytes and exact raw name + descriptor. Do not infer an ordinal from selector
    // order or method count.
    assert_eq!(analysis.report().method, method);
    let prepared_read = snapshot
        .prepared_root_class(&mut analysis_budget)
        .expect("same frozen class prepares for its method table");
    let prepared = jarde_reader::prepared::PreparedClass::prepare(
        &prepared_read,
        &mut analysis_budget,
    )
    .expect("same frozen class member table reads");
    let ordinals = prepared.locate_method(name.as_bytes(), descriptor.as_bytes());
    assert_eq!(ordinals.len(), 1, "selector must locate exactly one declared record");
    let member_ordinal = ordinals[0];
    let subject = ArtifactSubject::new(
        analysis.report().method.clone(),
        Some(member_ordinal),
        analysis.report().environment_identity.clone(),
    );
    let ir = analysis.ir();
    let declaration = ir.declaration().expect("same read states the method declaration");
    assert_eq!(String::from_utf8_lossy(&declaration.class_name().0), owner);
    assert_eq!(declaration.name().0.as_slice(), name.as_bytes());
    assert_eq!(declaration.descriptor().0.as_slice(), descriptor.as_bytes());
    let debug_locals = reader_debug_locals(ir.code());
    let facts = RecoveryFacts::new(
        MethodFacts::new(
            String::from_utf8_lossy(&declaration.name().0).into_owned(),
            String::from_utf8_lossy(&declaration.descriptor().0).into_owned(),
            declaration.parameter_slots(),
        )
        .with_access_flags(declaration.access_flags())
        .with_declaring_class(
            DeclaringClass::new(
                String::from_utf8_lossy(&declaration.class_name().0).into_owned(),
                declaration.class_access_flags(),
            )
            .with_inner_class_members(declaration.inner_class_members().iter().map(|member| {
                String::from_utf8_lossy(&member.0).replace('/', ".")
            })),
        ),
    )
    .with_debug_locals(debug_locals);
    let mut recovery_budget = recovery_budget.unwrap_or(analysis_budget);
    let report = recover(
        &RecoveryRequest::new(ir, &facts, JAVA_8)
            .with_evidence(evidence)
            .with_subject(subject),
        &mut recovery_budget,
    );
    if report.produced() {
        let binding = report.artifact.binding().expect("trusted subject binds the artifact");
        assert_eq!(binding.method(), &analysis.report().method);
        assert_eq!(binding.member_ordinal(), Some(member_ordinal));
        assert_eq!(binding.environment(), &analysis.report().environment_identity);
    }
    (report, method)
}

fn method_bcis(class: &[u8], name: &str, descriptor: &str) -> Vec<u32> {
    let inspection = inspect_method_bytecode(
        class,
        MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        &mut Budget::new(limits()),
    )
    .expect("frozen method bytecode reads");
    inspection.instructions.iter().map(|instruction| instruction.bci).collect()
}

fn assert_full_physical_method_coverage(
    report: &jarde_java::RecoveryReport,
    method: &PhysicalMethodId,
    bcis: &[u32],
) {
    for bci in bcis {
        let segments = report.source_map.of_bci(*bci);
        assert!(!segments.is_empty(), "missing source for BCI {bci}: {}", report.text);
        assert!(
            segments.iter().any(|segment| {
                std::iter::once(segment.origin().primary())
                    .chain(segment.origin().derived())
                    .any(|origin| origin.bci() == *bci && origin.method() == Some(method))
            }),
            "BCI {bci} has no source from exact physical method {method:?}: {}",
            report.text,
        );
    }
}

fn read_unique_ordinal(
    class: &[u8],
    name: &str,
    descriptor: &str,
) -> jarde_reader::prepared::MethodOrdinal {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("frozen class opens for the trusted declaration lookup");
    let read = snapshot
        .prepared_root_class(&mut budget)
        .expect("frozen root class is read");
    let prepared = jarde_reader::prepared::PreparedClass::prepare(&read, &mut budget)
        .expect("frozen member table is complete");
    let ordinals = prepared.locate_method(name.as_bytes(), descriptor.as_bytes());
    assert_eq!(ordinals.len(), 1, "selector must locate exactly one declared record");
    ordinals[0]
}

fn assert_default_equals_all(
    class: &[u8],
    owner: &str,
    name: &str,
    descriptor: &str,
) -> (jarde_java::RecoveryReport, PhysicalMethodId) {
    let default_evidence =
        RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap);
    let (default, method) = recover_class_method_with_budget(
        class, owner, name, descriptor, default_evidence, None,
    );
    let (all, all_method) = recover_class_method_with_budget(
        class, owner, name, descriptor, RecoveryEvidenceRequest::all(), None,
    );
    assert!(default.produced(), "default: {:?}", default.outcome);
    assert!(all.produced(), "all: {:?}", all.outcome);
    assert_eq!(method, all_method);
    assert_eq!(default.text, all.text);
    assert_eq!(default.source_map, all.source_map);
    assert!(!all.text.contains("@bytecode"), "{}", all.text);
    let binding = all.artifact.binding().expect("subject binds the committed artifact");
    assert_eq!(binding.method(), &method);
    assert_eq!(binding.member_ordinal(), Some(read_unique_ordinal(class, name, descriptor)));
    (all, method)
}


fn assert_map_signature(
    report: &jarde_java::RecoveryReport,
    expected: &[(usize, usize, u32, &[u32])],
) {
    let actual = report
        .source_map
        .segments()
        .iter()
        .map(|segment| {
            (
                segment.start(),
                segment.end(),
                segment.origin().primary().bci(),
                segment
                    .origin()
                    .derived()
                    .iter()
                    .map(|origin| origin.bci())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    let expected = expected
        .iter()
        .map(|(start, end, primary, derived)| {
            (*start, *end, *primary, derived.to_vec())
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "complete source-map signature changed: {}", report.text);
}

fn assert_call_pop(
    name: &str,
    descriptor: &str,
    call_bci: u32,
    pop_bci: u32,
    expected_body: &str,
    expected_statement_span: &str,
    expected_segments: &[(usize, usize, u32, &[u32])],
) {
    let owner = "discardprobe/DiscardedCallSourceProbe";
    let (report, method) = assert_default_equals_all(PROBE_CLASS, owner, name, descriptor);
    assert_eq!(report.text, expected_body, "old body text changed for {name}");
    assert_full_physical_method_coverage(
        &report,
        &method,
        &method_bcis(PROBE_CLASS, name, descriptor),
    );
    assert_map_signature(&report, expected_segments);
    assert!(report.source_map.direct_of_bci(pop_bci).is_empty());
    let mapped = report.source_map.derived_of_bci(pop_bci);
    assert_eq!(mapped.len(), 1, "one complete statement owns pop {pop_bci}");
    let segment = mapped[0];
    assert_eq!(segment.text(&report.text), expected_statement_span);
    assert_eq!(segment.origin().primary().bci(), call_bci);
    assert_eq!(segment.origin().primary().method(), Some(&method));
    assert_eq!(segment.origin().derived().len(), 1);
    assert_eq!(segment.origin().derived()[0].bci(), pop_bci);
    assert_eq!(segment.origin().derived()[0].method(), Some(&method));
}

#[test]
fn frozen_probe_has_exact_identity() {
    assert_eq!(PROBE_CLASS.len(), 979);
    assert_eq!(
        blake3::hash(PROBE_CLASS).to_hex().to_string(),
        "8282057a2be25b6c8d8dfa2f15151442d2bd396bed83bb6924ce58e0360e33a3"
    );
}

#[test]
fn proved_static_virtual_and_interface_call_pops_map_to_complete_statements() {
    const STATIC_BODY: &str = "// @method discardStatic(Z)V\n// @declaration a static method of `discardprobe.DiscardedCallSourceProbe`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    give(arg0);\n    return;\n}\n";
    const APPEND_BODY: &str = "// @method discardAppend(Ljava/lang/String;)Ljava/lang/String;\n// @declaration a static method of `discardprobe.DiscardedCallSourceProbe`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    java.lang.StringBuilder local1 = new java.lang.StringBuilder();\n    local1.append(arg0);\n    return local1.toString();\n}\n";
    const LIST_BODY: &str = "// @method discardListAdd(Ljava/lang/String;)Ljava/lang/String;\n// @declaration a static method of `discardprobe.DiscardedCallSourceProbe`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    java.util.ArrayList local1 = new java.util.ArrayList();\n    local1.add((java.lang.Object) arg0);\n    return (java.lang.String) local1.get(0);\n}\n";

    assert_call_pop(
        "discardStatic", "(Z)V", 1, 4, STATIC_BODY, "    give(arg0);\n",
        &[(203, 207, 0, &[]), (198, 208, 1, &[]), (194, 210, 1, &[4]), (210, 222, 5, &[])],
    );
    assert_call_pop(
        "discardAppend", "(Ljava/lang/String;)Ljava/lang/String;", 10, 13,
        APPEND_BODY, "    local1.append(arg0);\n",
        &[
            (265, 294, 4, &[0, 3]), (228, 296, 7, &[]), (300, 306, 8, &[]),
            (314, 318, 9, &[]), (300, 319, 10, &[]), (296, 321, 10, &[13]),
            (332, 338, 14, &[]), (332, 349, 15, &[]), (321, 351, 18, &[]),
        ],
    );
    assert_call_pop(
        "discardListAdd", "(Ljava/lang/String;)Ljava/lang/String;", 10, 15,
        LIST_BODY, "    local1.add((java.lang.Object) arg0);\n",
        &[
            (262, 287, 4, &[0, 3]), (229, 289, 7, &[]), (293, 299, 8, &[]),
            (323, 327, 9, &[]), (304, 327, 9, &[10]), (293, 328, 10, &[]),
            (289, 330, 10, &[15]), (360, 366, 16, &[]), (371, 372, 17, &[]),
            (360, 373, 18, &[]), (341, 373, 23, &[]), (330, 375, 26, &[]),
        ],
    );
}

#[test]
fn consumed_and_local_deferred_calls_keep_their_existing_body_and_map() {
    const RETURN_BODY: &str = "// @method consumeReturn()Ljava/lang/String;\n// @declaration a static method of `discardprobe.DiscardedCallSourceProbe`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    return give(false);\n}\n";
    const LOCAL_BODY: &str = "// @method deferToLocal()Ljava/lang/String;\n// @declaration a static method of `discardprobe.DiscardedCallSourceProbe`, member flags 0x0009\n// recovered from bytecode; presentation is not claimed to compile\n{\n    java.lang.String local0 = give(false);\n    return local0;\n}\n";
    let owner = "discardprobe/DiscardedCallSourceProbe";

    let (returned, return_method) =
        assert_default_equals_all(PROBE_CLASS, owner, "consumeReturn", "()Ljava/lang/String;");
    assert_eq!(returned.text, RETURN_BODY);
    assert_full_physical_method_coverage(
        &returned, &return_method,
        &method_bcis(PROBE_CLASS, "consumeReturn", "()Ljava/lang/String;"),
    );
    assert_map_signature(
        &returned,
        &[(226, 231, 0, &[]), (221, 232, 1, &[]), (210, 234, 4, &[])],
    );
    assert!(returned.source_map.derived_of_bci(1).is_empty());

    let (deferred, deferred_method) =
        assert_default_equals_all(PROBE_CLASS, owner, "deferToLocal", "()Ljava/lang/String;");
    assert_eq!(deferred.text, LOCAL_BODY);
    assert_full_physical_method_coverage(
        &deferred, &deferred_method,
        &method_bcis(PROBE_CLASS, "deferToLocal", "()Ljava/lang/String;"),
    );
    assert_map_signature(
        &deferred,
        &[(244, 249, 0, &[]), (239, 250, 1, &[]), (209, 252, 4, &[]),
          (263, 269, 5, &[]), (252, 271, 6, &[])],
    );
    assert!(deferred.source_map.derived_of_bci(1).is_empty());
}
