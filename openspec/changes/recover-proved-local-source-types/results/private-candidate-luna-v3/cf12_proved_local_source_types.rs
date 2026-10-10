//! Permanent test draft for the frozen CF12 local-source-type anchors.
//!
//! This is a private proposal: it has not been compiled or run and is not yet in the repository.

use jarde_java::{
    DebugLocal, DeclaringClass, MethodFacts, RecoveryEvidenceKind, RecoveryEvidenceRequest,
    RecoveryFacts, RecoveryRequest, StopReason, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, CountedBudgetDimension, Limits};
use jarde_reader::classfile::{
    LocalDebugTable, LocalDebugTypeTable, MethodSelector, inspect_method_bytecode,
};
use jarde_reader::model::{
    ClassBytesId, Digest, ExecutionReport, JvmBytes, PhysicalClassLocation,
    PhysicalDefinitionId, PhysicalMethodId, PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode,
    MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty,
    RuntimeView,
};

const CHAR_CLASS: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/full-replay-root-v1/TestSwitch.test/original/classes/jadx/tests/integration/switches/TestSwitch$TestCls.class"
);
const NULL_CLASS: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/full-replay-root-v1/TestSwitchNoDefault.test/original/classes/jadx/tests/integration/switches/TestSwitchNoDefault$TestCls.class"
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
        &RecoveryRequest::new(ir, &facts, JAVA_8).with_evidence(evidence),
        &mut recovery_budget,
    );
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
    (all, method)
}

#[test]
fn cf12_charat_stored_value_drives_char_source_type_and_full_origins() {
    let (report, method) = assert_default_equals_all(
        CHAR_CLASS,
        "jadx/tests/integration/switches/TestSwitch$TestCls",
        "test",
        "(Ljava/lang/String;)Ljava/lang/String;",
    );
    assert!(report.text.contains("char c"), "{}", report.text);
    assert!(report.text.contains("charAt("), "{}", report.text);
    assert!(report.text.contains("append("), "{}", report.text);
    assert!(report.text.contains("switch ("), "{}", report.text);
    assert_full_physical_method_coverage(
        &report,
        &method,
        &method_bcis(CHAR_CLASS, "test", "(Ljava/lang/String;)Ljava/lang/String;"),
    );
}

#[test]
fn cf12_null_first_direct_string_writes_drive_string_source_type_and_full_origins() {
    let (report, method) = assert_default_equals_all(
        NULL_CLASS,
        "jadx/tests/integration/switches/TestSwitchNoDefault$TestCls",
        "test",
        "(I)V",
    );
    assert!(report.text.contains("String s"), "{}", report.text);
    assert!(report.text.contains("null"), "{}", report.text);
    assert!(report.text.contains("switch ("), "{}", report.text);
    assert_full_physical_method_coverage(&report, &method, &method_bcis(NULL_CLASS, "test", "(I)V"));
}

#[test]
fn cf12_local_type_budget_stop_and_cancellation_publish_no_partial_source() {
    let full = recover_class_method_with_budget(
        NULL_CLASS,
        "jadx/tests/integration/switches/TestSwitchNoDefault$TestCls",
        "test",
        "(I)V",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(limits())),
    )
    .0;
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("full report did not complete: {:?}", full.outcome);
    };
    assert!(usage.analysis_steps > 0);
    let mut bounded = limits();
    bounded.analysis_steps = usage.analysis_steps - 1;
    let stopped = recover_class_method_with_budget(
        NULL_CLASS,
        "jadx/tests/integration/switches/TestSwitchNoDefault$TestCls",
        "test",
        "(I)V",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(bounded)),
    )
    .0;
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    assert!(matches!(
        stopped.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::AnalysisSteps,
            ..
        })
    ));

    // The helper analyzes with its own normal analysis budget and installs this token only for the
    // recovery call, so cancellation tests recovery's all-or-nothing publication boundary.
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_class_method_with_budget(
        NULL_CLASS,
        "jadx/tests/integration/switches/TestSwitchNoDefault$TestCls",
        "test",
        "(I)V",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    )
    .0;
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}
