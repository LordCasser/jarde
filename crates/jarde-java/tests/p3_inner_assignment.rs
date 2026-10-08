//! CF-06: a physical copy can become one in-condition local write only with one owner.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    StopReason, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest, Quality};
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

const POSITIVE: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-inner-assignment/cf06/InnerAssignCases.class");
const EXTRA_COPY: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-inner-assignment/ExtraCopy.class");
const WRONG_TYPE: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-inner-assignment/WrongType.class");
const NEGATIVE: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-inner-assignment/cf06/NegativeAssignments.class");

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

fn recover_method(
    class: &[u8],
    owner: &str,
    name: &str,
    descriptor: &str,
    parameters: u16,
    access_flags: u16,
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
        name: JvmBytes(name.as_bytes().to_vec()),
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
    .expect("fixture method analysis completes");
    let facts = RecoveryFacts::new(
        MethodFacts::new(name, descriptor, parameters)
            .with_access_flags(access_flags)
            .with_declaring_class(DeclaringClass::new(owner, 0x0021)),
    );
    let mut budget = recovery_budget.unwrap_or(budget);
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    )
}

fn positive(
    name: &str,
    descriptor: &str,
    parameters: u16,
    flags: u16,
) -> jarde_java::RecoveryReport {
    recover_method(
        POSITIVE,
        "cf06/InnerAssignCases",
        name,
        descriptor,
        parameters,
        flags,
        None,
    )
}

#[test]
fn call_result_is_assigned_once_inside_the_reached_condition() {
    let report = positive("lengthBranch", "(Ljava/lang/String;)I", 1, 0x0009);
    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(report.quality, Quality::Structured, "{}", report.text);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(
        report.text.contains("(local1 = arg0.length()) > 5"),
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("arg0.length()").count(),
        1,
        "{}",
        report.text
    );
    assert!(report.text.contains("return local1;"), "{}", report.text);
    for bci in [8, 11, 12, 14, 19, 20] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci}: {}",
            report.text
        );
    }
}

#[test]
fn field_read_is_assigned_once_after_the_short_circuit_call() {
    let report = positive("assignedAndChecked", "(Ljava/lang/String;)Z", 2, 0x0001);
    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(report.quality, Quality::Structured, "{}", report.text);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(
        report.text.contains("(local2 = this.field) != null"),
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("this.call(arg1)").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("this.field").count(),
        1,
        "{}",
        report.text
    );
    assert!(report.text.contains("local2.isEmpty()"), "{}", report.text);
    for bci in [2, 5, 9, 12, 13, 14, 17, 18, 21, 29] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci}: {}",
            report.text
        );
    }
}

#[test]
fn incomplete_copy_type_effect_and_scope_candidates_stay_quoted() {
    let controls = [
        (EXTRA_COPY, "cf06/InnerAssignCases", "lengthBranch", 11),
        (WRONG_TYPE, "cf06/InnerAssignCases", "lengthBranch", 11),
        (NEGATIVE, "cf06/NegativeAssignments", "interleaved", 4),
        (NEGATIVE, "cf06/NegativeAssignments", "exceptional", 4),
    ];
    for (class, owner, name, quoted_bci) in controls {
        let report = recover_method(class, owner, name, "(Ljava/lang/String;)I", 1, 0x0009, None);
        assert!(report.produced(), "{name}: {:?}", report.outcome);
        assert_eq!(report.quality, Quality::Fallback, "{name}: {}", report.text);
        assert!(report.text.contains("@bytecode"), "{name}: {}", report.text);
        assert!(!report.text.contains("local1 ="), "{name}: {}", report.text);
        assert!(
            !report.source_map.of_bci(quoted_bci).is_empty(),
            "{name} BCI {quoted_bci}: {}",
            report.text
        );
    }
}

/// The fifth CF-06 control — `loopCondition`'s loop test — presented in place by
/// `recover-loop-test-copy-store`: the assignment's store already stands at the test's own operand
/// position, so the loop's condition writes it there and nothing moves into or out of the loop.
/// The other four controls above keep their quotes, and this one keeps the same text the
/// short-circuit position's in-place form has always had.
#[test]
fn the_loop_condition_control_presents_the_assignment_in_place() {
    let report = recover_method(
        NEGATIVE,
        "cf06/NegativeAssignments",
        "loopCondition",
        "(Ljava/lang/String;)I",
        1,
        0x0009,
        None,
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(report.quality, Quality::Structured, "{}", report.text);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(
        report
            .text
            .contains("while ((local1 = arg0.length()) > 5) {"),
        "the assignment stays at the test's own position:\n{}",
        report.text
    );
    assert_eq!(
        report.text.matches("arg0.length()").count(),
        1,
        "the call is read once, where the bytecode read it:\n{}",
        report.text
    );
    assert!(report.text.contains("return local1;"), "{}", report.text);
    for bci in [0, 4, 5, 7, 10, 19] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci}: {}",
            report.text
        );
    }
}

#[test]
fn local_assignment_budget_and_cancellation_publish_no_partial_method() {
    for (name, descriptor, parameters, flags) in [
        ("lengthBranch", "(Ljava/lang/String;)I", 1, 0x0009),
        ("assignedAndChecked", "(Ljava/lang/String;)Z", 2, 0x0001),
    ] {
        let full = recover_method(
            POSITIVE,
            "cf06/InnerAssignCases",
            name,
            descriptor,
            parameters,
            flags,
            Some(Budget::new(limits())),
        );
        let ExecutionReport::Complete { usage } = full.execution else {
            panic!("{name} did not complete");
        };
        assert!(usage.analysis_steps > 1);
        let mut late = limits();
        late.analysis_steps = usage.analysis_steps - 1;
        let stopped = recover_method(
            POSITIVE,
            "cf06/InnerAssignCases",
            name,
            descriptor,
            parameters,
            flags,
            Some(Budget::new(late)),
        );
        assert!(!stopped.produced(), "{name}: {}", stopped.text);
        assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
        assert!(matches!(
            stopped.stop(),
            Some(StopReason::Budget {
                dimension: CountedBudgetDimension::AnalysisSteps,
                ..
            })
        ));

        let token = CancellationToken::new();
        token.cancel();
        let cancelled = recover_method(
            POSITIVE,
            "cf06/InnerAssignCases",
            name,
            descriptor,
            parameters,
            flags,
            Some(Budget::with_cancellation_token(limits(), token)),
        );
        assert!(!cancelled.produced());
        assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
        assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
    }
}
