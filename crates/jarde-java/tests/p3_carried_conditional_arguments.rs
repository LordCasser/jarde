use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, CountedBudgetDimension, Limits};
use jarde_reader::model::{
    ClassBytesId, Digest, ExecutionReport, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
    PhysicalMethodId, PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

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

fn report(
    class: &[u8],
    owner: &str,
    method_name: &str,
    descriptor: &str,
    parameters: u16,
    flags: u16,
) -> jarde_java::RecoveryReport {
    report_with_recovery_budget(
        class,
        owner,
        method_name,
        descriptor,
        parameters,
        flags,
        None,
    )
}

fn report_with_recovery_budget(
    class: &[u8],
    owner: &str,
    method_name: &str,
    descriptor: &str,
    parameters: u16,
    flags: u16,
    recovery_budget: Option<Budget>,
) -> jarde_java::RecoveryReport {
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
        name: JvmBytes(method_name.as_bytes().to_vec()),
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
        MethodFacts::new(method_name, descriptor, parameters)
            .with_access_flags(flags)
            .with_declaring_class(DeclaringClass::new(owner, 0x0031)),
    );
    let mut budget = recovery_budget.unwrap_or(budget);
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    )
}

const PAIR: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-25/constructor-conditional-delegation/ConstructorPairProbe.class"
);
const TYPE: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-carried-conditional-arguments/CarriedTypeMismatch.class"
);
const THIRD: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-carried-conditional-arguments/CarriedThirdArgument.class"
);
const EFFECT: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-carried-conditional-arguments/CarriedInterveningEffect.class"
);
const NON_PROLOGUE: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-carried-conditional-arguments/CarriedNonPrologue.class"
);
const METHODS: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-carried-conditional-arguments/CarriedMethodCalls.class"
);

#[test]
fn two_constructor_arguments_are_ordered_and_source_complete() {
    let report = report(
        PAIR,
        "ConstructorPairProbe",
        "<init>",
        "(Ljava/lang/String;I)V",
        3,
        1,
    );
    assert!(report.produced());
    assert!(
        report
            .text
            .contains("this(arg2 == 1 ? arg1 : \"\", arg2 == 0 ? \"\" : arg1);"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    for bci in [0, 1, 2, 3, 6, 7, 10, 12, 13, 16, 18, 21, 22, 25] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "missing BCI {bci}"
        );
    }
}

#[test]
fn carried_values_in_static_and_instance_calls_keep_argument_order() {
    for (name, flags, parameters) in [("staticCall", 0x0009, 2), ("instanceCall", 0x0001, 3)] {
        let report = report(
            METHODS,
            "CarriedMethodCalls",
            name,
            "(ZZ)Ljava/lang/String;",
            parameters,
            flags,
        );
        assert!(report.produced(), "{name}: {report:?}");
        assert!(
            !report.text.contains("@bytecode"),
            "{name}: {}",
            report.text
        );
        let (first, second) = if name == "staticCall" {
            ("arg0", "arg1")
        } else {
            ("arg1", "arg2")
        };
        assert!(
            report.text.contains(&format!(
                "{first} ? mark(\"A\", \"a\") : mark(\"B\", \"b\")"
            )),
            "{name}: {}",
            report.text
        );
        assert!(
            report.text.contains(&format!(
                "{second} ? mark(\"C\", \"c\") : mark(\"D\", \"d\")"
            )),
            "{name}: {}",
            report.text
        );
    }
}

#[test]
fn invalid_type_extra_argument_and_non_prologue_refuse_atomically() {
    for (class, owner, descriptor, parameters, bcis) in [
        (
            TYPE,
            "CarriedTypeMismatch",
            "(Ljava/lang/Object;ZI)V",
            4,
            &[0, 1, 2, 5, 6, 9, 11, 12, 15, 17, 20, 22, 25][..],
        ),
        (
            THIRD,
            "CarriedThirdArgument",
            "(Ljava/lang/String;I)V",
            3,
            &[0, 1, 2, 3, 6, 7, 10, 12, 13, 16, 18, 21, 23, 24, 27][..],
        ),
        (
            EFFECT,
            "CarriedInterveningEffect",
            "(Ljava/lang/String;I)V",
            3,
            &[0, 1, 2, 3, 6, 7, 10, 12, 15, 16, 19, 21, 24, 26, 29][..],
        ),
        (
            NON_PROLOGUE,
            "CarriedNonPrologue",
            "(Ljava/lang/String;I)V",
            3,
            &[
                0, 1, 4, 5, 8, 9, 10, 11, 14, 15, 18, 20, 21, 24, 26, 29, 31, 34, 37,
            ][..],
        ),
    ] {
        let report = report(class, owner, "<init>", descriptor, parameters, 1);
        assert!(report.produced(), "{owner}: {report:?}");
        assert!(
            report.text.contains("@bytecode"),
            "{owner}: {}",
            report.text
        );
        if owner == "CarriedInterveningEffect" {
            assert!(
                report.text.contains("independent instruction"),
                "{}",
                report.text
            );
        }
        assert!(
            !report.text.contains("this(") || owner == "CarriedNonPrologue",
            "{owner}: {}",
            report.text
        );
        for bci in bcis {
            assert!(
                !report.source_map.of_bci(*bci).is_empty(),
                "{owner}: missing quoted BCI {bci}: {}",
                report.text
            );
        }
    }
}

#[test]
fn carried_pair_budget_and_cancellation_publish_nothing() {
    use jarde_java::StopReason;

    let mut small = limits();
    small.ir_items = 1;
    let stopped = report_with_recovery_budget(
        PAIR,
        "ConstructorPairProbe",
        "<init>",
        "(Ljava/lang/String;I)V",
        3,
        1,
        Some(Budget::new(small)),
    );
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    assert!(matches!(
        stopped.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::IrItems,
            ..
        })
    ));

    let complete = report_with_recovery_budget(
        PAIR,
        "ConstructorPairProbe",
        "<init>",
        "(Ljava/lang/String;I)V",
        3,
        1,
        Some(Budget::new(limits())),
    );
    let ExecutionReport::Complete { usage } = complete.execution else {
        panic!("the unrestricted pair recovery must complete");
    };
    let mut refusing = 1;
    let mut publishing = usage.ir_items;
    while refusing + 1 < publishing {
        let middle = (refusing + publishing) / 2;
        let mut probe_limits = limits();
        probe_limits.ir_items = middle;
        let probe = report_with_recovery_budget(
            PAIR,
            "ConstructorPairProbe",
            "<init>",
            "(Ljava/lang/String;I)V",
            3,
            1,
            Some(Budget::new(probe_limits)),
        );
        if probe.produced() {
            publishing = middle;
        } else {
            refusing = middle;
        }
    }
    assert!(refusing > usage.ir_items / 2);
    let mut near_complete = limits();
    near_complete.ir_items = refusing;
    let late_stop = report_with_recovery_budget(
        PAIR,
        "ConstructorPairProbe",
        "<init>",
        "(Ljava/lang/String;I)V",
        3,
        1,
        Some(Budget::new(near_complete)),
    );
    assert!(!late_stop.produced());
    assert!(late_stop.text.is_empty() && late_stop.source_map.is_empty());
    assert!(matches!(
        late_stop.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::IrItems,
            ..
        })
    ));

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = report_with_recovery_budget(
        PAIR,
        "ConstructorPairProbe",
        "<init>",
        "(Ljava/lang/String;I)V",
        3,
        1,
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}
