//! The loop bodies holding **two** loop-jump edges (`recover-loop-body-double-jumps`) recover
//! with both edges presented, the single-jump matrix stays byte-for-byte what it was, and the
//! three-jump body keeps its registered refusal.
//!
//! The fixtures are the patrol's frozen classes: `L1`/`L2`/`L5` (the composite and the two
//! minimal reproductions, with `L5.dblJumpDoWhilePlain` keeping its pre-existing `break` +
//! `return` quote) and `DJLoops`/`DJTripleJump` (the variant set: two continue edges to
//! different targets, the zero-regression single-jump matrix, and the three-jump negative).

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
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const DJ_LOOPS: &[u8] = include_bytes!("fixtures/p3-loop-double-jumps/DJLoops.class");
const DJ_TRIPLE: &[u8] = include_bytes!("fixtures/p3-loop-double-jumps/DJTripleJump.class");
const L1: &[u8] = include_bytes!("fixtures/p3-loop-double-jumps/L1.class");
const L2: &[u8] = include_bytes!("fixtures/p3-loop-double-jumps/L2.class");
const L5: &[u8] = include_bytes!("fixtures/p3-loop-double-jumps/L5.class");

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

/// One `int`→`String` static method of a fixture class, with every evidence category.
fn int_string_method(class: &[u8], owner: &str, name: &str) -> jarde_java::RecoveryReport {
    recover_class_method(
        class,
        owner,
        name,
        "(I)Ljava/lang/String;",
        RecoveryEvidenceRequest::all(),
        None,
    )
}

#[test]
fn two_jump_loop_bodies_present_both_edges() {
    // L5.brkSelfContSelf: `break` and `continue` of the same loop, one body.
    let report = int_string_method(L5, "L5", "brkSelfContSelf");
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(report.text.matches("break;").count(), 1, "{}", report.text);
    // The `continue` arm presents as the empty `then` arm ahead of the loop's own update.
    assert!(
        report.text.contains("if (local4 == 1) {"),
        "{}",
        report.text
    );

    // L5.brkSelfContMid: `break` of this loop, `continue` of the middle loop.
    let report = int_string_method(L5, "L5", "brkSelfContMid");
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(report.text.matches("break;").count(), 1, "{}", report.text);

    // L5.dblJumpDoWhile: `continue w` and `break w` inside a `do … while (false)` body.
    let report = recover_class_method(
        L5,
        "L5",
        "dblJumpDoWhile",
        "()Ljava/lang/String;",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(report.text.matches("break;").count(), 1, "{}", report.text);

    // DJLoops.dblContLabels: two `continue` edges, one per target loop.
    let report = int_string_method(DJ_LOOPS, "DJLoops", "dblContLabels");
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(report.text.contains("break loop;"), "{}", report.text);
}

#[test]
fn composite_labelled_families_recover_completely() {
    // L1.deepLabels: the labelled patrol composite — three nested loops, four jump edges.
    let report = int_string_method(L1, "L1", "deepLabels");
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(report.text.matches("break;").count(), 2, "{}", report.text);
    assert_eq!(
        report.text.matches("break loop;").count(),
        1,
        "{}",
        report.text
    );

    // L2.triplePlain: two plain jumps inside a triple nest.
    let report = int_string_method(L2, "L2", "triplePlain");
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(report.text.matches("break;").count(), 1, "{}", report.text);
}

#[test]
fn jump_transfer_blocks_have_one_owner_and_complete_sources() {
    // The two transfer blocks of L5.brkSelfContSelf (BCI 37 `break`, 45 `continue`) and the
    // labelled transfers of L1.deepLabels (27/35) keep exactly one structured owner, and no
    // source anchor is lost.
    for (class, owner, name, anchors) in [
        (L5, "L5", "brkSelfContSelf", vec![37u32, 45]),
        (L1, "L1", "deepLabels", vec![27, 35, 53, 62]),
        (DJ_LOOPS, "DJLoops", "dblContLabels", vec![37, 45]),
    ] {
        let report = int_string_method(class, owner, name);
        for bci in anchors {
            let owners: Vec<_> = report
                .regions
                .iter()
                .filter(|region| region.blocks.contains(&bci))
                .collect();
            assert_eq!(
                owners.len(),
                1,
                "{name}: BCI {bci} owners: {:?}",
                report.regions
            );
            assert!(owners[0].structured, "{name}: BCI {bci} is not structured");
        }
    }
}

#[test]
fn single_jump_matrix_is_unchanged_text() {
    // The pre-existing channels present these bodies exactly as they did before the
    // two-edge proof: the whole method text is pinned, so any drift in the single-jump
    // spelling fails here.
    let cont_self = int_string_method(DJ_LOOPS, "DJLoops", "contSelfOnly");
    assert_eq!(
        cont_self.text,
        r#"// @method contSelfOnly(I)Ljava/lang/String;
// @declaration a static method of `DJLoops`, member flags 0x0009
// recovered from bytecode; presentation is not claimed to compile
{
    java.lang.StringBuilder local1;
    int local2;
    local1 = new java.lang.StringBuilder();
    int local3;
    for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
        int local4;
        for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
            for (local4 = 0; local4 < arg0; local4 = local4 + 1) {
                if (local4 == 1) {
                } else {
                    local1.append(local2).append(local3).append(local4).append(' ');
                }
            }
        }
    }
    return local1.toString();
}
"#
    );
    let cont_mid = int_string_method(DJ_LOOPS, "DJLoops", "contMidLabelOnly");
    assert_eq!(
        cont_mid.text,
        r#"// @method contMidLabelOnly(I)Ljava/lang/String;
// @declaration a static method of `DJLoops`, member flags 0x0009
// recovered from bytecode; presentation is not claimed to compile
{
    java.lang.StringBuilder local1;
    int local2;
    local1 = new java.lang.StringBuilder();
    int local3;
    for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
        int local4;
        for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
            local4 = 0;
            while (local4 < arg0) {
                if (local3 == 2) {
                    break;
                } else {
                    local1.append(local2).append(local3).append(local4).append(' ');
                    local4 = local4 + 1;
                }
            }
        }
    }
    return local1.toString();
}
"#
    );
    let brk_mid = int_string_method(DJ_LOOPS, "DJLoops", "brkMidLabelOnly");
    assert!(brk_mid.text.contains("break loop;"), "{}", brk_mid.text);
    assert!(!brk_mid.text.contains("@bytecode"), "{}", brk_mid.text);
    assert_eq!(brk_mid.text.matches("for (").count(), 2, "{}", brk_mid.text);
}

#[test]
fn break_and_return_channel_keeps_its_quote() {
    // L5.dblJumpDoWhilePlain pairs a `break` with a `return`: the existing channel's
    // presentation — structured loop with the early-return block quoted — is this slice's
    // zero-regression boundary and must not move.
    let report = recover_class_method(
        L5,
        "L5",
        "dblJumpDoWhilePlain",
        "()Ljava/lang/String;",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(report.text.contains("@bytecode 26 28"), "{}", report.text);
}

#[test]
fn three_jump_body_keeps_its_registered_refusal() {
    let report = int_string_method(DJ_TRIPLE, "DJTripleJump", "tripleJump");
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(report.text.contains("@bytecode"), "{}", report.text);
    assert!(
        report.regions.iter().any(|region| !region.structured),
        "the three-jump body was structured against the registered boundary: {:?}",
        report.regions
    );
}

#[test]
fn double_jump_budget_and_cancellation_publish_no_partial_source() {
    use jarde_java::StopReason;
    use jarde_reader::model::ExecutionReport;
    let full = recover_class_method(
        DJ_LOOPS,
        "DJLoops",
        "dblContLabels",
        "(I)Ljava/lang/String;",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(limits())),
    );
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("full candidate did not complete: {:?}", full.outcome);
    };
    let mut late = limits();
    late.analysis_steps = usage.analysis_steps - 1;
    let stopped = recover_class_method(
        DJ_LOOPS,
        "DJLoops",
        "dblContLabels",
        "(I)Ljava/lang/String;",
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
        DJ_LOOPS,
        "DJLoops",
        "dblContLabels",
        "(I)Ljava/lang/String;",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}
