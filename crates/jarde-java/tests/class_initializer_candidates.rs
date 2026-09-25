use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryFacts, RecoveryOutcome, RecoveryRequest, StopReason,
    facts::{ACC_ANNOTATION, ACC_INTERFACE},
    report::{ClassInitializerStep, recover_for_class_source},
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CountedBudgetDimension, Limits, UsageSnapshot};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const INTERFACE_INIT: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-interface-field-initializers/v8/InterfaceInitProbe.class"
);
const EXTRA_EFFECT: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-interface-field-initializers/v8/negative/extra-effect/BoundaryProbe.class"
);
const FORWARD_READ: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-interface-field-initializers/v8/negative/forward-read/ForwardProbe.class"
);
const CONSTANT_PHASE: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/constant-phase-boundary/PhaseProbe.class"
);
const EXCEPTION_EDGE: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/forward-binding-exception-edge/exception-handler/ExceptionProbe.class"
);
const ENUM_HELPER_NORMAL: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/enum-switch-labels/original/EnumSwitchSubject$1.class"
);
const ENUM_HELPER_SWAPPED: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/enum-switch-labels/patched-map/EnumSwitchSubject$1.class"
);
const ENUM_HELPER_MULTI_WRITE: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/enum-switch-labels/negative/multi-write/classes/EnumSwitchSubject$1.class"
);
const ENUM_HELPER_DUPLICATE_KEY: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/enum-switch-labels/negative/duplicate-key/classes/EnumSwitchSubject$1.class"
);
const ENUM_HUE_NORMAL: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/enum-switch-labels/original/Hue.class"
);
const ENUM_HUE_ALIASED: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/enum-switch-labels/negative/aliased-enum/classes/Hue.class"
);
const ENUM_HUE_FACTORY_NULL_ELEMENT: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/enum-switch-labels/negative/enum-values-array/factory-null-element-Hue.class"
);
const ENUM_HUE_VALUES_RETURNS_NULL: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-24/enum-switch-labels/negative/enum-values-array/values-returns-null-Hue.class"
);

struct Fixture {
    analysis: jarde_jvm::method_ir::MethodIrAnalysis,
    method: PhysicalMethodId,
    facts: RecoveryFacts,
}

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

fn fixture(class_bytes: &[u8], class_name: &str) -> Fixture {
    fixture_method(class_bytes, class_name, "<clinit>", "()V")
}

fn fixture_method(
    class_bytes: &[u8],
    class_name: &str,
    method_name: &str,
    descriptor: &str,
) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class_bytes.to_vec()), &mut budget)
        .expect("the checked-in class opens");
    let method = PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(class_bytes).to_hex().to_string()),
                length: u64::try_from(class_bytes.len()).expect("fixture length fits u64"),
            },
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(method_name.as_bytes().to_vec()),
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
    let environment = ResolutionEnvironment {
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
    };
    let request = MethodAnalysisRequest {
        environment,
        method: method.clone(),
        stages: AnalysisStage::ALL.to_vec(),
    };
    let analysis = analyze_method_ir(&[snapshot], &request, &mut budget)
        .expect("the class initializer analysis runs");
    let facts = RecoveryFacts::new(
        MethodFacts::new(method_name, descriptor, 0)
            .with_access_flags(0x0008)
            .with_declaring_class(DeclaringClass::new(class_name, 0x0601)),
    );
    Fixture {
        analysis,
        method,
        facts,
    }
}

fn enum_map_result(
    class_bytes: &[u8],
) -> Result<jarde_java::enumswitch::EnumSwitchMapProof, String> {
    let fixture = fixture(class_bytes, "EnumSwitchSubject$1");
    let mut budget = Budget::new(limits());
    jarde_java::enumswitch::prove_enum_switch_map_initializer(
        fixture.analysis.ir(),
        "EnumSwitchSubject$1",
        "$SwitchMap$Hue",
        "Hue",
        &[b"RED".to_vec(), b"BLUE".to_vec(), b"GREEN".to_vec()],
        &[1, 2],
        &mut budget,
    )
    .expect("the proof walk stays within the test budget")
}

fn enum_identity_result(class_bytes: &[u8]) -> Result<(), String> {
    let initializer = fixture_method(class_bytes, "Hue", "<clinit>", "()V");
    let constructor = fixture_method(class_bytes, "Hue", "<init>", "(Ljava/lang/String;I)V");
    let values_factory = fixture_method(class_bytes, "Hue", "$values", "()[LHue;");
    let values_method = fixture_method(class_bytes, "Hue", "values", "()[LHue;");
    let mut budget = Budget::new(limits());
    jarde_java::enumswitch::prove_enum_constant_ordinals(
        initializer.analysis.ir(),
        constructor.analysis.ir(),
        (values_factory.analysis.ir(), values_method.analysis.ir()),
        "Hue",
        &[b"RED".to_vec(), b"BLUE".to_vec(), b"GREEN".to_vec()],
        "$VALUES",
        &mut budget,
    )
    .expect("enum identity proof remains within the test budget")
}

#[test]
fn enum_map_proof_follows_real_initializer_stores_instead_of_field_or_key_order() {
    let normal = enum_map_result(ENUM_HELPER_NORMAL).expect("normal map proof");
    assert_eq!(
        normal
            .entries
            .iter()
            .map(|entry| (entry.key, entry.constant.as_slice(), entry.store_bci))
            .collect::<Vec<_>>(),
        vec![(1, b"RED".as_slice(), 19), (2, b"BLUE".as_slice(), 34)]
    );
    assert_eq!(normal.allocation_bci, 4);
    assert_eq!(normal.table_write_bci, 6);
    assert_eq!(normal.entries[0].handler_bci, 23);
    assert_eq!(normal.entries[1].handler_bci, 38);

    let swapped = enum_map_result(ENUM_HELPER_SWAPPED).expect("swapped map proof");
    assert_eq!(
        swapped
            .entries
            .iter()
            .map(|entry| (entry.key, entry.constant.as_slice(), entry.store_bci))
            .collect::<Vec<_>>(),
        vec![(2, b"RED".as_slice(), 19), (1, b"BLUE".as_slice(), 34)]
    );
}

#[test]
fn enum_map_proof_refuses_multiple_writes_and_non_unique_keys() {
    let multiple =
        enum_map_result(ENUM_HELPER_MULTI_WRITE).expect_err("multiple RED stores refuse");
    assert!(
        !multiple.is_empty(),
        "multiple-write refusal names its boundary"
    );

    let duplicate = enum_map_result(ENUM_HELPER_DUPLICATE_KEY).expect_err("duplicate key refuses");
    assert!(duplicate.contains("repeats"), "{duplicate}");
}

#[test]
fn enum_identity_proof_accepts_distinct_ordinals_and_refuses_aliased_fields() {
    enum_identity_result(ENUM_HUE_NORMAL)
        .expect("ordinary javac enum constants have distinct objects");
    let aliased = enum_identity_result(ENUM_HUE_ALIASED)
        .expect_err("the verifier-valid alias must fail the constant identity proof");
    assert!(aliased.contains("enum <clinit>"), "{aliased}");
}

#[test]
fn enum_identity_proof_consumes_the_actual_factory_and_public_values_bodies() {
    let initializer = fixture_method(ENUM_HUE_NORMAL, "Hue", "<clinit>", "()V");
    let mut budget = Budget::new(limits());
    assert_eq!(
        jarde_java::enumswitch::enum_values_factory_name(
            initializer.analysis.ir(),
            "Hue",
            "$VALUES",
            &mut budget,
        )
        .expect("factory target lookup remains within budget")
        .expect("the actual callref names the factory"),
        "$values"
    );
    let bad_factory = enum_identity_result(ENUM_HUE_FACTORY_NULL_ELEMENT)
        .expect_err("an altered factory element must refuse enum projection");
    assert!(bad_factory.contains("factory"), "{bad_factory}");
    let bad_values = enum_identity_result(ENUM_HUE_VALUES_RETURNS_NULL)
        .expect_err("a public values() returning null must refuse enum projection");
    assert!(bad_values.contains("public values()"), "{bad_values}");
}

fn ir_items(usage: &UsageSnapshot) -> u64 {
    usage.ir_items
}

fn report_usage(report: &jarde_java::RecoveryReport) -> &UsageSnapshot {
    match &report.execution {
        jarde_reader::model::ExecutionReport::Complete { usage }
        | jarde_reader::model::ExecutionReport::Partial { usage, .. }
        | jarde_reader::model::ExecutionReport::Cancelled { usage }
        | jarde_reader::model::ExecutionReport::Failed { usage, .. } => usage,
    }
}

fn clear_elapsed(report: &mut jarde_java::RecoveryReport) {
    let usage = match &mut report.execution {
        jarde_reader::model::ExecutionReport::Complete { usage }
        | jarde_reader::model::ExecutionReport::Partial { usage, .. }
        | jarde_reader::model::ExecutionReport::Cancelled { usage }
        | jarde_reader::model::ExecutionReport::Failed { usage, .. } => usage,
    };
    usage.elapsed_millis = 0;
}

#[test]
fn class_source_sidecar_keeps_field_identity_rhs_and_bytecode_order() {
    let fixture = fixture(INTERFACE_INIT, "InterfaceInitProbe");
    let request = RecoveryRequest::new(
        fixture.analysis.ir(),
        &fixture.facts,
        jarde_java::pass::JAVA_8,
    );
    let mut budget = Budget::new(limits());
    let recovered = recover_for_class_source(&request, &mut budget, false, false);
    assert!(
        recovered.report.produced(),
        "{:?}",
        recovered.report.outcome
    );
    let candidates = recovered.initializer.expect("`<clinit>` gets its sidecar");
    assert_eq!(candidates.member.as_ref(), Some(&fixture.method));

    let writes: Vec<_> = candidates
        .steps
        .iter()
        .filter_map(|step| match step {
            ClassInitializerStep::FieldWrite(write) => Some(write),
            ClassInitializerStep::Other { .. } => None,
        })
        .collect();
    assert_eq!(writes.len(), 3, "{:?}", candidates.steps);
    assert_eq!(
        writes
            .iter()
            .map(|write| (
                write.order,
                write.bci,
                write.name.as_str(),
                write.descriptor.as_str()
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, 5, "FIRST", "Ljava/lang/String;"),
            (1, 13, "SECOND", "Ljava/lang/String;"),
            (2, 25, "TOTAL", "I"),
        ]
    );
    assert_eq!(
        writes[2]
            .field_reads
            .as_ref()
            .expect("same-run field claims account for TOTAL's RHS reads")
            .iter()
            .map(|read| (
                read.bci,
                read.owner.as_str(),
                read.name.as_str(),
                read.descriptor.as_str()
            ))
            .collect::<Vec<_>>(),
        vec![
            (16, "InterfaceInitProbe", "FIRST", "Ljava/lang/String;"),
            (19, "InterfaceInitProbe", "SECOND", "Ljava/lang/String;"),
        ]
    );
    assert!(writes[0].field_reads.as_ref().unwrap().is_empty());
    assert!(writes[1].field_reads.as_ref().unwrap().is_empty());
    assert!(!candidates.has_exception_handlers);
    for write in &writes {
        assert_eq!(write.owner, "InterfaceInitProbe");
        assert_eq!(write.spelled_name, write.name);
        assert!(write.is_static);
        assert_eq!(write.source.primary().bci(), write.bci);
        assert!(write.has_receiver);
    }
    assert!(matches!(
        &writes[0].value.kind,
        jarde_java::ast::ExprKind::Call { .. }
    ));
    assert_eq!(writes[0].value.origin.primary().bci(), 2);
    assert!(candidates.steps.iter().all(|step| match step {
        ClassInitializerStep::FieldWrite(write) => write.order < candidates.steps.len(),
        ClassInitializerStep::Other { order, .. } => *order < candidates.steps.len(),
    }));
}

#[test]
fn sidecar_keeps_forward_and_constant_phase_reads_at_their_real_bcis() {
    for (class_bytes, class_name, expected_name, expected_descriptor, expected_bci) in [
        (FORWARD_READ, "ForwardProbe", "LATE", "I", 0),
        (CONSTANT_PHASE, "PhaseProbe", "LATE", "I", 0),
    ] {
        let fixture = fixture(class_bytes, class_name);
        let request = RecoveryRequest::new(
            fixture.analysis.ir(),
            &fixture.facts,
            jarde_java::pass::JAVA_8,
        );
        let mut budget = Budget::new(limits());
        let recovered = recover_for_class_source(&request, &mut budget, false, false);
        assert!(
            recovered.report.produced(),
            "{:?}",
            recovered.report.outcome
        );
        let candidates = recovered
            .initializer
            .expect("the interface clinit has a sidecar");
        assert!(!candidates.has_exception_handlers);
        let field_write = candidates
            .steps
            .iter()
            .find_map(|step| match step {
                ClassInitializerStep::FieldWrite(write) if write.name == "EARLY" => Some(write),
                ClassInitializerStep::FieldWrite(_) | ClassInitializerStep::Other { .. } => None,
            })
            .expect("EARLY has a recovered field write");
        let reads = field_write
            .field_reads
            .as_ref()
            .expect("the RHS field read has an exact field@1 claim");
        assert_eq!(reads.len(), 1);
        assert_eq!(reads[0].bci, expected_bci);
        assert_eq!(reads[0].owner, class_name);
        assert_eq!(reads[0].name, expected_name);
        assert_eq!(reads[0].descriptor, expected_descriptor);
        assert!(reads[0].is_static);
    }
}

#[test]
fn sidecar_marks_a_real_exception_handler_and_never_calls_it_a_linear_body() {
    let fixture = fixture(EXCEPTION_EDGE, "ExceptionProbe");
    let request = RecoveryRequest::new(
        fixture.analysis.ir(),
        &fixture.facts,
        jarde_java::pass::JAVA_8,
    );
    let mut budget = Budget::new(limits());
    let recovered = recover_for_class_source(&request, &mut budget, false, false);
    assert!(
        recovered.report.produced(),
        "{:?}",
        recovered.report.outcome
    );
    let candidates = recovered
        .initializer
        .expect("the interface clinit has a sidecar");
    assert!(candidates.has_exception_handlers);
}

#[test]
fn candidate_sequence_keeps_unclaimed_top_level_effects_visible() {
    let fixture = fixture(EXTRA_EFFECT, "BoundaryProbe");
    let request = RecoveryRequest::new(
        fixture.analysis.ir(),
        &fixture.facts,
        jarde_java::pass::JAVA_8,
    );
    let mut budget = Budget::new(limits());
    let recovered = recover_for_class_source(&request, &mut budget, false, false);
    assert!(
        recovered.report.produced(),
        "{:?}",
        recovered.report.outcome
    );
    let candidates = recovered.initializer.expect("`<clinit>` gets its sidecar");
    assert_eq!(candidates.member.as_ref(), Some(&fixture.method));
    assert_eq!(candidates.steps.len(), 4, "{:?}", candidates.steps);
    assert!(matches!(
        &candidates.steps[2],
        ClassInitializerStep::Other {
            order: 2,
            bci: 16,
            kind: jarde_java::report::ClassInitializerStatementKind::Expression,
        }
    ));
    assert!(matches!(
        &candidates.steps[3],
        ClassInitializerStep::Other {
            order: 3,
            bci: 19,
            kind: jarde_java::report::ClassInitializerStatementKind::Return,
        }
    ));
}

#[test]
fn candidate_collection_budget_refusal_is_a_real_stop_without_a_partial_sidecar() {
    let fixture = fixture(INTERFACE_INIT, "InterfaceInitProbe");
    let request = RecoveryRequest::new(
        fixture.analysis.ir(),
        &fixture.facts,
        jarde_java::pass::JAVA_8,
    );
    let mut baseline_budget = Budget::new(limits());
    let baseline = jarde_java::recover(&request, &mut baseline_budget);
    assert!(baseline.produced(), "{:?}", baseline.outcome);

    let mut bounded = limits();
    bounded.ir_items = ir_items(report_usage(&baseline)).saturating_add(1);
    let mut budget = Budget::new(bounded);
    let stopped = recover_for_class_source(&request, &mut budget, false, false);
    assert!(stopped.initializer.is_none());
    assert!(stopped.report.text.is_empty());
    assert!(matches!(
        stopped.report.outcome,
        RecoveryOutcome::Stopped(StopReason::Budget {
            dimension: CountedBudgetDimension::IrItems,
            at: Some(5),
            ..
        })
    ));

    let mut output_bounded = limits();
    output_bounded.output_bytes = 0;
    let mut budget = Budget::new(output_bounded);
    let stopped = recover_for_class_source(&request, &mut budget, false, false);
    assert!(matches!(
        stopped.report.outcome,
        RecoveryOutcome::Stopped(_)
    ));
    assert!(stopped.initializer.is_none());
}

#[test]
fn initializer_sidecar_is_limited_to_non_annotation_interfaces() {
    let fixture = fixture(INTERFACE_INIT, "InterfaceInitProbe");
    for flags in [0x0001, ACC_INTERFACE | ACC_ANNOTATION] {
        let facts = RecoveryFacts::new(
            MethodFacts::new("<clinit>", "()V", 0)
                .with_access_flags(0x0008)
                .with_declaring_class(DeclaringClass::new("InterfaceInitProbe", flags)),
        );
        let request = RecoveryRequest::new(fixture.analysis.ir(), &facts, jarde_java::pass::JAVA_8);

        let mut ordinary_budget = Budget::new(limits());
        let mut expected = jarde_java::recover(&request, &mut ordinary_budget);
        let mut adapter_budget = Budget::new(limits());
        let mut actual = recover_for_class_source(&request, &mut adapter_budget, false, false);

        assert!(actual.initializer.is_none(), "class flags {flags:#06x}");
        clear_elapsed(&mut expected);
        clear_elapsed(&mut actual.report);
        assert_eq!(actual.report, expected, "class flags {flags:#06x}");
    }
}
