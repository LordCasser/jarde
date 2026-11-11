//! CF-06: a physical copy can become one in-condition local write only with one owner.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    StopReason, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest, Quality};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, Limits};
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
        &RecoveryRequest::new(
            analysis.ir(),
            &facts,
            JAVA_8,
            jarde_reader::view::RuntimeProfile {
                java_release: 8,
                multi_release: jarde_reader::view::MultiReleasePolicy::Disabled,
                layout: jarde_reader::view::LayoutMode::Generic,
            },
        )
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

/// The one control that stays quoted: `WrongType.class` swaps BCI 8's method reference to
/// `String.isEmpty:()Z`, so the value stored into `local1` is a `boolean` while the test compares it
/// with the integer literal 5. No Java type admits both, so this shape keeps its quote and is the
/// negative evidence that the admission rule is still in force after the 2026-11-08 widening.
///
/// **Changed 2026-11-08**: `ExtraCopy`, `interleaved` and `exceptional` used to sit in this list and
/// now recover as structured text (see the two tests below). The earlier attribution read this list
/// as "one control became structured, three stayed quoted"; that was an inference, not a
/// measurement — the loop panicked at the first control and never reached the rest. Three of the
/// four had in fact widened. Evidence:
/// `openspec/changes/restructure-demand-driven-recovery/results/s4-region-summary-v1/
/// cross-domain-regression-r1/appendix-expansion-gate.md`.
#[test]
fn incomplete_copy_type_and_scope_candidates_stay_quoted() {
    let report = recover_method(
        WRONG_TYPE,
        "cf06/InnerAssignCases",
        "lengthBranch",
        "(Ljava/lang/String;)I",
        1,
        0x0009,
        None,
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(report.quality, Quality::Fallback, "{}", report.text);
    assert!(report.text.contains("@bytecode"), "{}", report.text);
    assert!(!report.text.contains("local1 ="), "{}", report.text);
    assert!(
        !report.source_map.of_bci(11).is_empty(),
        "BCI 11: {}",
        report.text
    );
}

/// The `interleaved` and `exceptional` controls are admitted too, and their text states the copy
/// once: the two reads of the duplicated call result are one store and one local, not two calls.
///
/// `interleaved` calls a separate `effect()` between the store and the test (`invokestatic` at BCI 6)
/// and `exceptional` puts the same copy/store/test under a live `RuntimeException` handler. In both,
/// BCI 4's `dup` keeps the `length()` result on the stack and BCI 5's `istore_1` stores it, so the
/// operand consumed by the test is a value the body really produced; the recovery names both the
/// stored local and the operand with that one variable.
///
/// **Changed 2026-11-08** (admission-domain expansion, gated). These two used to sit in the quoted
/// control list. Recovered as whole classes, compiled with `javac --release 8` and run against the
/// frozen classes, both presentations are behaviourally identical to the originals — including
/// `interleaved`'s observable `++calls` ordering and `exceptional(null)` returning `-2` through the
/// handler (evidence:
/// `openspec/changes/restructure-demand-driven-recovery/results/s4-region-summary-v1/
/// cross-domain-regression-r1/appendix-expansion-gate.md`). The widening moved `RECOVERY_SEMANTICS`
/// 68 → 69.
#[test]
fn the_interleaved_and_exceptional_controls_present_the_copy_once() {
    let cases = [
        (
            "interleaved",
            "int saved0 = arg0.length();\n    int local1 = saved0;\n    if (saved0 + effect() > 5) {",
            4,
        ),
        (
            "exceptional",
            "int saved0 = arg0.length();\n        int local1 = saved0;\n        if (saved0 > 5) {",
            4,
        ),
    ];
    for (name, expected, quoted_bci) in cases {
        let report = recover_method(
            NEGATIVE,
            "cf06/NegativeAssignments",
            name,
            "(Ljava/lang/String;)I",
            1,
            0x0009,
            None,
        );
        assert!(report.produced(), "{name}: {:?}", report.outcome);
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
            !report.text.contains("@bytecode"),
            "{name}: {}",
            report.text
        );
        assert_eq!(
            report.text.matches("arg0.length()").count(),
            1,
            "{name}: the call is not duplicated: {}",
            report.text
        );
        assert!(
            report.text.contains("int saved0 = arg0.length();"),
            "{name}: {}",
            report.text
        );
        assert!(
            report.text.contains("int local1 = saved0;"),
            "{name}: {}",
            report.text
        );
        assert!(report.text.contains(expected), "{name}: {}", report.text);
        assert!(
            report.text.contains("return local1;"),
            "{name}: {}",
            report.text
        );
        assert!(
            !report.source_map.of_bci(quoted_bci).is_empty(),
            "{name} BCI {quoted_bci}: {}",
            report.text
        );
    }
}
/// presentation rather than a mis-named split.
///
/// `ExtraCopy.class` rewrites BCI 13 of `lengthBranch` from `iconst_5` to `dup` (see
/// `patch_controls.py`), so the method's code is:
///
/// ```text
///   8: invokevirtual #13  // String.length()I     stack: [len]
///  11: dup                                         stack: [len, len]
///  12: istore_1                                    stack: [len]      local1 = len
///  13: dup                                         stack: [len, len]
///  14: if_icmple 19                                pops both -> compares len with len
/// ```
///
/// Both operands of the comparison at BCI 14 come from the one `length()` result that BCI 11 and
/// BCI 13 duplicated; no `iload` of a second slot appears anywhere in this method. The comparison is
/// therefore *genuinely* a slot against itself, and `if (saved0 > saved0)` states exactly that: it is
/// constant-false, always takes the `else` branch, and returns `local1`. That is what the original
/// class does — compiled, run and compared against the frozen class, identical on every input
/// including the `null` that throws (evidence:
/// `openspec/changes/restructure-demand-driven-recovery/results/s4-region-summary-v1/
/// cross-domain-regression-r1/appendix-expansion-gate.md`).
///
/// The alternative reading — that the bytecode compares two *different* slots and the split named
/// both operands alike — would be a behaviour-changing recovery error. The disassembly above rules it
/// out, which is why this shape may be admitted instead of staying quoted.
///
/// **Changed 2026-11-08**: this control previously pinned the quoted fallback. The shape now reaches
/// the same copy/store/test presentation the unpatched `lengthBranch` already had, so it is
/// structured; the widening moved `RECOVERY_SEMANTICS` 68 → 69.
#[test]
fn extra_copy_of_the_call_result_is_a_faithful_self_comparison() {
    let report = recover_method(
        EXTRA_COPY,
        "cf06/InnerAssignCases",
        "lengthBranch",
        "(Ljava/lang/String;)I",
        1,
        0x0009,
        None,
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(report.quality, Quality::Structured, "{}", report.text);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(
        report.text.contains("int saved0 = arg0.length();"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("int local1 = saved0;"),
        "{}",
        report.text
    );
    // The faithful form: the duplicated value compared with itself, not a second variable.
    assert!(
        report.text.contains("if (saved0 > saved0) {"),
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("saved0").count(),
        4,
        "the call result is stored once and read by both operands: {}",
        report.text
    );
    assert_eq!(
        report.text.matches("arg0.length()").count(),
        1,
        "the call is not duplicated for the second operand: {}",
        report.text
    );
    assert!(report.text.contains("return local1;"), "{}", report.text);
    for bci in [0, 1, 4, 7, 8, 11, 12, 13, 14, 17, 18, 19, 20] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci}: {}",
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
                dimension: jarde_reader::budget::BudgetDimension::AnalysisSteps,
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
