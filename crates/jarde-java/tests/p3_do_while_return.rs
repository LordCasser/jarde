//! The do-while(false) body's conditional `return` recovers, and a quote the presentation
//! reaches may not hold a control-flow-changing exit while the body without it would still
//! compile (`recover-return-in-do-while-false`).
//!
//! The fixtures are the patrol's frozen classes: `L5`/`D1` (the two faces the patrol recorded —
//! the silent miscompilation of a `return "early"` leaf no region owned, and the same shape with
//! a `hits++` side effect beside it) and `DWVariants` (the variant set: the throw face whose
//! leaf reads no local, the same throw reading one, a body with two conditional returns, and
//! the unrecoverable negative whose return leaf has two normal entries).

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryContent, RecoveryEvidenceRequest, RecoveryFacts,
    RecoveryRequest, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, CountedBudgetDimension, Limits};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const L5: &[u8] = include_bytes!("fixtures/p3-do-while-return/L5.class");
const D1: &[u8] = include_bytes!("fixtures/p3-do-while-return/D1.class");
const DW: &[u8] = include_bytes!("fixtures/p3-do-while-return/DWVariants.class");

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

fn recover_class_method(
    class: &[u8],
    owner: &str,
    name: &str,
    descriptor: &str,
    evidence: RecoveryEvidenceRequest,
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
        MethodFacts::new(name, descriptor, 1)
            .with_access_flags(0x0009)
            .with_declaring_class(DeclaringClass::new(owner, 0x0021)),
    );
    let mut budget = recovery_budget.unwrap_or(budget);
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8).with_evidence(evidence),
        &mut budget,
    )
}

/// One `()String` static method of a fixture class, with every evidence category.
fn string_method(class: &[u8], owner: &str, name: &str) -> jarde_java::RecoveryReport {
    recover_class_method(
        class,
        owner,
        name,
        "()Ljava/lang/String;",
        RecoveryEvidenceRequest::all(),
        None,
    )
}

#[test]
fn the_do_while_false_return_bodies_recover() {
    // L5.dblJumpDoWhilePlain: `do { if (i%3==0) break; if (i>7) return "early"; } while (false);`
    // inside a `while` — the silent-miscompilation face of the patrol. The return leaf is an
    // exclusive-predecessor terminal return, the region owns it, and the presentation keeps
    // the early return instead of quoting it behind a compilable partial body.
    let report = string_method(L5, "L5", "dblJumpDoWhilePlain");
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(report.text.contains("return \"early\";"), "{}", report.text);
    assert_eq!(
        report.content,
        RecoveryContent::ContainsStatements,
        "{}",
        report.text
    );
    // The leaf the quote used to hold has exactly one structured owner now.
    let owners: Vec<_> = report
        .regions
        .iter()
        .filter(|region| region.blocks.contains(&26))
        .collect();
    assert_eq!(owners.len(), 1, "{:?}", report.regions);
    assert!(owners[0].structured, "{:?}", report.regions);

    // D1.retInDoWhile: the same shape with a `hits++` side effect after the test. Both the
    // return arm and the effect arm stay, so the class recompiles to `early:5`, not `i=10`.
    let report = string_method(D1, "D1", "retInDoWhile");
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(
        report.text.contains(
            "return new java.lang.StringBuilder().append(\"early:\").append(D1.hits).toString();"
        ) || report.text.contains("return \"early:\" + D1.hits;"),
        "the early return names the hits it read: {}",
        report.text
    );
    assert!(
        report.text.contains("D1.hits = D1.hits + 1;"),
        "{}",
        report.text
    );

    // D1's other return shapes recover too: a plain `do … while (i < 5)` whose entry test
    // returns (`d3`) and a `for` body's conditional return (`f2`).
    let report = string_method(D1, "D1", "retInPlainDo");
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(
        report.text.contains("return \"d\" + arg0;") && report.text.contains("return \"d3\";"),
        "{}",
        report.text
    );
    let report = string_method(D1, "D1", "retInIf");
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(report.text.contains("return \"f2\";"), "{}", report.text);
}

#[test]
fn double_return_body_owns_both_leaves() {
    // Two conditional returns in one do-while(false) body: each leaf has its own exclusive
    // predecessor, so the region owns both arms — the single-leaf bound of the CF-07 slice is
    // gone with the `iload; ireturn` shape it was drawn around.
    let report = string_method(DW, "DWVariants", "doubleReturnDoWhile");
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(
        report.text.contains("return \"a\" + arg0;"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("return \"b\" + arg0;"),
        "{}",
        report.text
    );
}

#[test]
fn the_throw_face_of_the_quote_closes_the_whole_method() {
    // `throwConstDoWhile`: the body's abrupt edge is an `athrow` whose leaf reads no local, so
    // no earlier closure fires — before this slice the partial body compiled with the throw
    // silently dropped (`i=10` and no exception). The failure-closure invariant refuses the
    // whole method instead: no quote the presentation reaches may hold the exceptional exit.
    let report = string_method(DW, "DWVariants", "throwConstDoWhile");
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(
        report.content,
        RecoveryContent::ExplanationOnly,
        "the closed body publishes no statement:\n{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("ends in a control-flow exit at BCI 35"),
        "the quote names the athrow it refused for:\n{}",
        report.text
    );
    assert!(
        report.text.contains("the whole method is quoted"),
        "{}",
        report.text
    );
}

#[test]
fn the_variant_negatives_keep_their_registered_refusals() {
    // `throwInDoWhile`: the throw leaf reads a local, and the abrupt edge is an `athrow` this
    // classification never adopts — whichever whole-method refusal states the method (the
    // local-crossing closure of the declaration layer, or this slice's failure closure when
    // the facts of the run name no crossing local), it publishes no statement and no partial
    // body that could compile without the throw. `sharedLeafDoWhile`: the `||`-spelled return
    // leaf has two normal entries, no branch exclusively owns it, and the arms never meet —
    // the registered refusal.
    for name in ["throwInDoWhile", "sharedLeafDoWhile"] {
        let report = string_method(DW, "DWVariants", name);
        assert!(report.produced(), "{name}: {:?}", report.outcome);
        assert_eq!(
            report.content,
            RecoveryContent::ExplanationOnly,
            "{name}: {}",
            report.text
        );
        assert!(report.text.contains("@bytecode"), "{name}: {}", report.text);
        assert!(
            !report.text.contains("throw new"),
            "{name}: no partial statement survives the refusal:
{}",
            report.text
        );
    }
}

#[test]
fn a_quote_a_stripped_body_cannot_compile_through_keeps_its_partial_presentation() {
    // The completion gate of the closure: a quote that holds a `return` while the body without
    // it would NOT compile (the empty `catch` falls off the end of a value-returning method)
    // keeps the partial presentation it always had. `Patrol2.multiCatch` is that shape — its
    // try body returns, its clause body and the join the clause's value folds at are quoted,
    // and stripping the quotes leaves a method `javac` rejects for a missing return statement,
    // so there is no silently-different compilation to refuse.
    let report = recover_class_method(
        include_bytes!("../../../openspec/evidence/java-syntax-2026-09-26/Patrol2.class"),
        "Patrol2",
        "multiCatch",
        "(I)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report.text.contains("return 0;"),
        "the try body's statements are still written:\n{}",
        report.text
    );
    assert!(
        report.text.contains("@bytecode 44"),
        "the join's quote stays:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("the whole method is quoted"),
        "the closure did not fire for a body that cannot compile without its quote:\n{}",
        report.text
    );
}

#[test]
fn closure_budget_and_cancellation_publish_no_partial_source() {
    use jarde_java::StopReason;
    use jarde_reader::model::ExecutionReport;
    let full = recover_class_method(
        DW,
        "DWVariants",
        "throwConstDoWhile",
        "()Ljava/lang/String;",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(limits())),
    );
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("the full candidate did not complete: {:?}", full.outcome);
    };
    assert!(usage.analysis_steps > 1);
    let mut late = limits();
    late.analysis_steps = usage.analysis_steps - 1;
    let stopped = recover_class_method(
        DW,
        "DWVariants",
        "throwConstDoWhile",
        "()Ljava/lang/String;",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(late)),
    );
    assert!(!stopped.produced());
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
    let cancelled = recover_class_method(
        DW,
        "DWVariants",
        "throwConstDoWhile",
        "()Ljava/lang/String;",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}
