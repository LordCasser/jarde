//! Recovery from the frozen upstream CF12 local-source-type anchors.

use jarde_java::{
    ArtifactSubject, DebugLocal, DeclaringClass, MethodFacts, RecoveryEvidenceKind,
    RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest, StopReason, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, Limits};
use jarde_reader::classfile::{
    LocalDebugTable, LocalDebugTypeTable, MethodSelector, inspect_method_bytecode,
};
use jarde_reader::model::{
    ClassBytesId, Digest, ExecutionReport, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
    PhysicalMethodId, PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
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
fn reader_debug_locals(code: Option<&jarde_reader::classfile::MethodCodeFacts>) -> Vec<DebugLocal> {
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
                local.with_type_metadata(record.descriptor.0.clone(), generic.signature.0.clone())
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
    let snapshot =
        ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut analysis_budget)
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
    let prepared =
        jarde_reader::prepared::PreparedClass::prepare(&prepared_read, &mut analysis_budget)
            .expect("same frozen class member table reads");
    let ordinals = prepared.locate_method(name.as_bytes(), descriptor.as_bytes());
    assert_eq!(
        ordinals.len(),
        1,
        "selector must locate exactly one declared record"
    );
    let member_ordinal = ordinals[0];
    let subject = ArtifactSubject::new(
        analysis.report().method.clone(),
        Some(member_ordinal),
        analysis.report().environment_identity.clone(),
    );
    let ir = analysis.ir();
    let declaration = ir
        .declaration()
        .expect("same read states the method declaration");
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
            .with_inner_classes(declaration.inner_classes().clone()),
        ),
    )
    .with_debug_locals(debug_locals);
    let mut recovery_budget = recovery_budget.unwrap_or(analysis_budget);
    let report = recover(
        &RecoveryRequest::new(
            ir,
            &facts,
            JAVA_8,
            jarde_reader::view::RuntimeProfile {
                java_release: 8,
                multi_release: jarde_reader::view::MultiReleasePolicy::Disabled,
                layout: jarde_reader::view::LayoutMode::Generic,
            },
        )
        .with_evidence(evidence)
        .with_subject(subject),
        &mut recovery_budget,
    );
    if report.produced() {
        assert_eq!(
            report.input_origin,
            jarde_java::report::RecoveryInputOrigin::CallerSupplied
        );
        assert!(
            report.artifact.binding().is_none(),
            "caller subject metadata is not a verified read binding"
        );
        assert_eq!(analysis.ir().declaration().unwrap().identity(), &method);
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
    inspection
        .instructions
        .iter()
        .map(|instruction| instruction.bci)
        .collect()
}

fn assert_full_physical_method_coverage(
    report: &jarde_java::RecoveryReport,
    method: &PhysicalMethodId,
    bcis: &[u32],
) {
    let missing: Vec<_> = bcis
        .iter()
        .copied()
        .filter(|bci| {
            !report.source_map.of_bci(*bci).iter().any(|segment| {
                std::iter::once(segment.origin().primary())
                    .chain(segment.origin().derived())
                    .any(|origin| origin.bci() == *bci && origin.method() == Some(method))
            })
        })
        .collect();
    assert!(
        missing.is_empty(),
        "missing exact physical sources for BCIs {missing:?}: {}",
        report.text
    );
}

fn assert_default_equals_all(
    class: &[u8],
    owner: &str,
    name: &str,
    descriptor: &str,
) -> (jarde_java::RecoveryReport, PhysicalMethodId) {
    let default_evidence =
        RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap);
    let (default, method) =
        recover_class_method_with_budget(class, owner, name, descriptor, default_evidence, None);
    let (all, all_method) = recover_class_method_with_budget(
        class,
        owner,
        name,
        descriptor,
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(default.produced(), "default: {:?}", default.outcome);
    assert!(all.produced(), "all: {:?}", all.outcome);
    assert_eq!(method, all_method);
    assert_eq!(default.text, all.text);
    assert_eq!(default.source_map, all.source_map);
    assert!(!all.text.contains("@bytecode"), "{}", all.text);
    assert!(all.artifact.binding().is_none());
    assert_eq!(
        all.input_origin,
        jarde_java::report::RecoveryInputOrigin::CallerSupplied
    );
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
    assert_full_physical_method_coverage(
        &report,
        &method,
        &method_bcis(NULL_CLASS, "test", "(I)V"),
    );
}

#[test]
fn cf12_recovery_budget_stop_and_cancellation_publish_no_partial_source() {
    // This is an all-or-nothing recovery-boundary regression only. `full_usage - 1` may stop
    // before the newly added producer proof. The separate proof-budget test below uses actual
    // prefixes recorded from these same class bytes, real SlotUse/SSA facts and fresh recovery.
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
            dimension: jarde_reader::budget::BudgetDimension::AnalysisSteps,
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

// Boundary assertions come from the preserved real trace; the temporary trace was removed.
use jarde_java::{RecoveryContent, RecoveryOutcome};
use jarde_jvm::ir::Quality;

const TYPED_BOUNDARY_DEBUG: &[u8] = include_bytes!(
    "../../../openspec/changes/recover-proved-local-source-types/results/boundary-preflight-root-v1/javac23-debug/classes/LocalSourceTypesBoundaries.class"
);
const TYPED_BOUNDARY_NODEBUG: &[u8] = include_bytes!(
    "../../../openspec/changes/recover-proved-local-source-types/results/boundary-preflight-root-v1/javac23-no-debug/classes/LocalSourceTypesBoundaries.class"
);
const TYPED_BOUNDARY_OWNER: &str = "LocalSourceTypesBoundaries";

fn assert_typed_boundary_structured(
    class: &[u8],
    name: &str,
    descriptor: &str,
    expected_declaration: &str,
    forbidden_declaration: Option<&str>,
) -> String {
    let (report, method) = assert_default_equals_all(class, TYPED_BOUNDARY_OWNER, name, descriptor);
    assert_eq!(
        report.quality,
        Quality::Structured,
        "{name}: {}",
        report.text
    );
    assert!(
        report.fallbacks.is_empty(),
        "{name}: {:?}",
        report.fallbacks
    );
    assert!(
        report.text.contains(expected_declaration),
        "{name}: {}",
        report.text
    );
    if let Some(forbidden) = forbidden_declaration {
        assert!(!report.text.contains(forbidden), "{name}: {}", report.text);
    }
    assert_full_physical_method_coverage(&report, &method, &method_bcis(class, name, descriptor));
    report.text
}

#[test]
fn typed_boundary_four_char_seeds_have_specific_types_and_full_origins_in_both_debug_profiles() {
    let fixtures = [
        ("javac23-debug", TYPED_BOUNDARY_DEBUG),
        ("javac23-no-debug", TYPED_BOUNDARY_NODEBUG),
    ];
    let cases = [
        (
            "charCallAndLiteralWrites",
            "(IZZ)Ljava/lang/String;",
            "char value;",
            "char local3;",
        ),
        (
            "charFieldSeed",
            "()Ljava/lang/String;",
            "char value =",
            "char local0 =",
        ),
        ("charI2cSeed", "(I)I", "char value =", "char local1 ="),
        (
            "charEntryParameterSeed",
            "(CZ)Ljava/lang/String;",
            "char value;",
            "char local2;",
        ),
    ];
    for (profile, class) in fixtures {
        for (name, descriptor, debug_declaration, nodebug_declaration) in cases {
            let expected = if profile == "javac23-debug" {
                debug_declaration
            } else {
                nodebug_declaration
            };
            assert_typed_boundary_structured(class, name, descriptor, expected, None);
        }
    }
}

#[test]
fn typed_boundary_range_arithmetic_and_input_copy_remain_int_in_both_debug_profiles() {
    let fixtures = [TYPED_BOUNDARY_DEBUG, TYPED_BOUNDARY_NODEBUG];
    let cases = [
        (
            "intWithOutOfRangeWrites",
            "(II)I",
            "int value;",
            "int local2;",
        ),
        (
            "intWithArithmeticWrite",
            "(Z)I",
            "int value;",
            "int local1;",
        ),
        (
            "intWithUnknownCopyMerge",
            "(ZI)I",
            "int value;",
            "int local3;",
        ),
    ];
    for (profile_index, class) in fixtures.into_iter().enumerate() {
        for (name, descriptor, debug_declaration, nodebug_declaration) in cases {
            let expected = if profile_index == 0 {
                debug_declaration
            } else {
                nodebug_declaration
            };
            let text = assert_typed_boundary_structured(class, name, descriptor, expected, None);
            assert!(!text.contains("char value"), "{name}: {text}");
            assert!(!text.contains("char local"), "{name}: {text}");
        }
    }
}

#[test]
fn typed_boundary_null_first_exact_string_stays_string_in_both_debug_profiles() {
    for (profile_index, class) in [TYPED_BOUNDARY_DEBUG, TYPED_BOUNDARY_NODEBUG]
        .into_iter()
        .enumerate()
    {
        let expected = if profile_index == 0 {
            "java.lang.String value;"
        } else {
            "java.lang.String local1;"
        };
        assert_typed_boundary_structured(
            class,
            "exactStringWritesAfterNull",
            "(I)V",
            expected,
            Some("Object value;"),
        );
    }
}

#[test]
fn typed_boundary_mixed_all_null_and_unknown_reference_stay_object_in_both_debug_profiles() {
    let cases = [
        (
            "mixedReferenceWrites",
            "(Z)V",
            "Object value;",
            "Object local1;",
        ),
        ("allNullWrites", "(Z)V", "Object value;", "Object local1;"),
        (
            "unknownReferenceCopy",
            "(ZLjava/lang/Object;)V",
            "Object value;",
            "Object local2;",
        ),
    ];
    for (profile_index, class) in [TYPED_BOUNDARY_DEBUG, TYPED_BOUNDARY_NODEBUG]
        .into_iter()
        .enumerate()
    {
        for (name, descriptor, debug_declaration, nodebug_declaration) in cases {
            let expected = if profile_index == 0 {
                debug_declaration
            } else {
                nodebug_declaration
            };
            let text = assert_typed_boundary_structured(class, name, descriptor, expected, None);
            assert!(!text.contains("String value;"), "{name}: {text}");
            assert!(!text.contains("String local"), "{name}: {text}");
        }
    }
}

fn missing_physical_bcis(
    report: &jarde_java::RecoveryReport,
    method: &PhysicalMethodId,
    bcis: &[u32],
) -> Vec<u32> {
    bcis.iter()
        .copied()
        .filter(|bci| {
            !report.source_map.of_bci(*bci).iter().any(|segment| {
                std::iter::once(segment.origin().primary())
                    .chain(segment.origin().derived())
                    .any(|origin| origin.bci() == *bci && origin.method() == Some(method))
            })
        })
        .collect()
}

fn recover_slot_reuse(
    class: &[u8],
    evidence: RecoveryEvidenceRequest,
) -> (jarde_java::RecoveryReport, PhysicalMethodId) {
    recover_class_method_with_budget(
        class,
        TYPED_BOUNDARY_OWNER,
        "possibleSlotReuse",
        "(Z)Ljava/lang/String;",
        evidence,
        None,
    )
}

/// `possibleSlotReuse` writes slot 2 twice with no overlap in liveness: `istore_2` at BCI 10 writes
/// an `int` that `iload_2` at BCI 11 reads, and `astore_2` at BCI 16 writes a `String`-valued
/// `Object` that `aload_2` at BCI 54 reads. Those are two independent destinations, so `reuse`'s
/// `exclusive_definition_split` proves them apart — each definition owns its own block, neither
/// block reaches the other's, and `local_value_stays_before` proves the earlier value dies before
/// the later write — and the presentation declares two source variables instead of refusing the
/// one slot as contradictory.
///
/// **Changed 2026-11-08** (admission-domain expansion, gated, not a narrowing). Until this gate the
/// test pinned the older behaviour: one source variable per slot, so the two types collided and the
/// region was refused as an exact fallback. The split was already the accepted mechanism; this
/// shape simply reaches it. Evidence and the compile/run comparison that justifies widening the
/// admitted domain: `openspec/changes/restructure-demand-driven-recovery/results/
/// s4-region-summary-v1/cross-domain-regression-r1/appendix-expansion-gate.md`. Because the same
/// input now recovers differently, `RECOVERY_SEMANTICS` moved 68 → 69.
#[test]
fn typed_boundary_reused_slot_splits_into_independent_source_variables_in_default_and_all() {
    let fixtures = [
        (
            TYPED_BOUNDARY_DEBUG,
            "stringValue",
            "value",
            "value_2",
            "number",
        ),
        (
            TYPED_BOUNDARY_NODEBUG,
            "arg0",
            "local2",
            "local2_2",
            "local1",
        ),
    ];
    for (class, condition, int_name, string_name, number_name) in fixtures {
        let (default, method) = recover_slot_reuse(
            class,
            RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
        );
        let (all, all_method) = recover_slot_reuse(class, RecoveryEvidenceRequest::all());
        assert_eq!(method, all_method);

        // The split is admitted: structured text, no quoted bytecode, and no fallback at all.
        assert_eq!(default.quality, Quality::Structured, "{}", default.text);
        assert_eq!(all.quality, Quality::Structured, "{}", all.text);
        assert_eq!(default.outcome, RecoveryOutcome::Produced);
        assert_eq!(all.outcome, RecoveryOutcome::Produced);
        assert_eq!(default.text, all.text);
        assert_eq!(default.source_map, all.source_map);
        assert!(default.fallbacks.is_empty(), "{:?}", default.fallbacks);
        assert!(!default.text.contains("@bytecode"), "{}", default.text);

        // One slot, two source variables: each definition gets its own destination and its own
        // type, and neither is spelled with the other's.
        assert!(
            default
                .text
                .contains(&format!("int {int_name} = {condition} ? 65536 : -1;")),
            "{}",
            default.text
        );
        assert!(
            default
                .text
                .contains(&format!("java.lang.String {string_name};")),
            "{}",
            default.text
        );
        assert!(
            default
                .text
                .contains(&format!("java.lang.String {number_name};")),
            "{}",
            default.text
        );
        assert_eq!(
            default.text.matches(&format!("int {int_name}")).count(),
            1,
            "the int destination is declared once: {}",
            default.text
        );
        assert_eq!(
            default
                .text
                .matches(&format!("java.lang.String {string_name}"))
                .count(),
            1,
            "the String destination is declared once: {}",
            default.text
        );

        // The earlier `int` write is read by the `Integer.toString` call and the later `String`
        // writes only ever reach the string destination — the ranges really are disjoint.
        assert!(
            default
                .text
                .contains(&format!("{number_name} = java.lang.Integer.toString({int_name});")),
            "{}",
            default.text
        );
        assert!(
            default
                .text
                .contains(&format!("{string_name} = \"reused-string\";")),
            "{}",
            default.text
        );
        assert!(
            default
                .text
                .contains(&format!("{string_name} = new java.lang.String(\"other-string\");")),
            "{}",
            default.text
        );
        assert!(
            default
                .text
                .contains(&format!("return {number_name} + \":\" + {string_name};")),
            "{}",
            default.text
        );

        // Two ranges still cover every physical instruction of the method: splitting the slot does
        // not move a byte out of the source map.
        assert_eq!(
            missing_physical_bcis(
                &default,
                &method,
                &method_bcis(class, "possibleSlotReuse", "(Z)Ljava/lang/String;"),
            ),
            Vec::<u32>::new(),
            "{}",
            default.text
        );
    }
}

fn assert_typed_boundary_budget_stop(
    class: &[u8],
    owner: &str,
    name: &str,
    descriptor: &str,
    limit: u64,
    at: u32,
) {
    let (report, _) = recover_class_method_with_budget(
        class,
        owner,
        name,
        descriptor,
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(Limits {
            analysis_steps: limit,
            ..limits()
        })),
    );
    assert!(matches!(
        &report.outcome,
        RecoveryOutcome::Stopped(StopReason::Budget {
            dimension: jarde_reader::budget::BudgetDimension::AnalysisSteps,
            written: 0,
            limit: observed_limit,
            at: Some(observed_at),
            ..
        }) if *observed_limit == limit && *observed_at == at
    ));
    assert_eq!(report.content, RecoveryContent::NotProduced);
    assert!(report.artifact.binding().is_none());
    assert!(report.text.is_empty());
    assert!(report.source_map.is_empty());
}

#[test]
fn typed_boundary_proof_budget_stops_before_publication_at_observed_sites() {
    // Root re-observed the same type-proof write@29 and null proof@1 after the switch
    // certificate's real work was added; keep these stops inside the type proof.
    //
    // Re-pinned 2026-11-08 (cross-domain-regression-r1): 365 → 628 and 130 → 349. The earlier
    // readings were taken before `report.rs::recover_inner` gained its `collect_reference_relations`
    // pre-pass, which walks a direct request's own call sites once before any pass runs. A fresh
    // sweep over 1..=3000 finds the same two sites at higher offsets and confirms each re-pin is
    // the *first* limit that reaches its site (char: 627 stops at at=26, 628 at at=29; null: 348
    // stops at at=0, 349 at at=1). These are the suite's observed boundary limits, not product
    // pins: the assertion — the proof stops before publication, with no text and no artifact — is
    // unchanged.
    assert_typed_boundary_budget_stop(
        CHAR_CLASS,
        "jadx/tests/integration/switches/TestSwitch$TestCls",
        "test",
        "(Ljava/lang/String;)Ljava/lang/String;",
        628,
        29,
    );
    assert_typed_boundary_budget_stop(
        NULL_CLASS,
        "jadx/tests/integration/switches/TestSwitchNoDefault$TestCls",
        "test",
        "(I)V",
        349,
        1,
    );
}
