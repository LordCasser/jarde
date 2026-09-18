//! P2 4.1 acceptance: the descriptor-driven frames through the public entry point.
//!
//! The frame table itself is a crate-private payload (invariant 11), so the value-level rules —
//! the local merge that answers `Top` instead of refusing, the category-2 binding, the
//! `dup`/`swap`/`pop` pairing, the descriptor-driven invocation shapes and D41 — are pinned by
//! the unit tests of `crates/jarde-jvm/src/frame.rs`, which decode real and synthetic bodies
//! directly. What this file proves through the public API is the wiring and the planes around
//! that payload:
//!
//! 1. the `frame` phase really runs over the canonical graph of a committed fixture and
//!    completes it — the body of that class carries no stack map or local-variable table, and
//!    the derivation happens anyway, from the descriptors and the data flow;
//! 2. it charges what its row declares: the frames are derived storage and a worklist walk, so a
//!    request that reaches the phase bills strictly more than the same request without it;
//! 3. a body this build cannot state the frames of yet — a constructor, whose `this` is
//!    initialized by an `invokespecial <init>` — stops under `ir_frame_deferred` with a
//!    `Partial` execution, keeps the raw and canonical facts, and reaches no phase behind it;
//! 4. and in both cases the product planes stay what they are: `verification` is `NotPerformed`
//!    and the semantic evidence is `Unproven`. Deriving frames is not verifying a method.

use jarde::*;
use std::slice;

/// The committed historical fixture: a class whose methods have real bodies, and whose `Code`
/// attributes carry no `StackMapTable` or local-variable table for this derivation to lean on.
const V52: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
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

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

struct Fixture {
    snapshot: ArtifactSnapshot,
    method: PhysicalMethodId,
}

/// Opens the fixture and derives the physical identity of one of its methods.
fn fixture(name: &[u8], descriptor: &[u8]) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(V52.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(V52).to_hex().to_string()),
            length: u64::try_from(V52.len()).expect("fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: bytes(name),
        descriptor: bytes(descriptor),
    };
    Fixture { snapshot, method }
}

/// One caller domain rooted at the fixture, and nothing else.
fn environment(fixture: &Fixture) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Snapshot {
            snapshot: fixture.snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: fixture.snapshot.id().clone(),
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
    }
}

fn analyze(
    fixture: &Fixture,
    stages: Vec<AnalysisStage>,
    limits: Limits,
) -> (MethodAnalysisReport, Budget) {
    let request = MethodAnalysisRequest {
        environment: environment(fixture),
        method: fixture.method.clone(),
        stages,
    };
    let mut budget = Budget::new(limits);
    let report = Engine::new()
        .analyze_method(slice::from_ref(&fixture.snapshot), &request, &mut budget)
        .expect("a legal request is answered, not raised");
    (report, budget)
}

/// The stage states of a report in scheduled order.
fn stage_states(report: &MethodAnalysisReport) -> Vec<StageState> {
    report
        .stages
        .iter()
        .map(|stage| stage.state.clone())
        .collect()
}

fn diagnostic_codes(report: &MethodAnalysisReport) -> Vec<&str> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

/// The code of the diagnostic this report carries, with its severity.
fn diagnostic(report: &MethodAnalysisReport, code: &str) -> DiagnosticSeverity {
    report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == code)
        .unwrap_or_else(|| panic!("the report carries `{code}`: {:#?}", report.diagnostics))
        .severity
}

/// The four product planes a P2 report always states, whatever the run proved.
fn assert_planes_stay_p1(report: &MethodAnalysisReport) {
    assert_eq!(report.representation, Representation::Bytecode);
    assert_eq!(report.syntax_status, SyntaxStatus::NotJava);
    assert_eq!(report.compile_status, CompileStatus::NotAttempted);
    assert_eq!(
        report.verification,
        VerificationStatus::NotPerformed,
        "deriving frames is not verifying the method"
    );
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::Unproven,
        "4.1 proves local invariants of its own states, and 4.3 is what may raise this"
    );
}

#[test]
fn the_frame_phase_completes_a_body_without_any_debug_table() {
    // `finallyPath(I)I` of the 52 fixture is an instance method with real control flow. Its
    // `Code` attribute carries no `StackMapTable` and no local-variable table, so what the frame
    // phase derives comes from the descriptors and the data flow alone — and the phase behind it
    // is still the one this build does not implement.
    let fixture = fixture(b"finallyPath", b"(I)I");
    let (report, budget) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());
    assert_eq!(
        stage_states(&report),
        vec![
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Failed {
                code: "ir_pass_not_implemented".to_string()
            },
        ],
        "the frame phase completed and `ssa` is the phase this build does not implement"
    );
    assert_eq!(diagnostic_codes(&report), vec!["ir_pass_not_implemented"]);
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(report.body, MethodBodyState::Present);
    assert_planes_stay_p1(&report);

    // The frames are derived storage and a worklist walk, so a request that reaches the phase
    // bills strictly more than the same request without it on both dimensions its row declares.
    let (_, without) = analyze(&fixture, vec![AnalysisStage::CanonicalCfg], limits());
    assert!(
        budget.usage().ir_items > without.usage().ir_items,
        "the frame slots are derived items: {} vs {}",
        budget.usage().ir_items,
        without.usage().ir_items
    );
    assert!(
        budget.usage().analysis_steps > without.usage().analysis_steps,
        "the fixpoint walks a worklist: {} vs {}",
        budget.usage().analysis_steps,
        without.usage().analysis_steps
    );
    assert_eq!(
        budget.usage().ir_edges,
        without.usage().ir_edges,
        "this pass builds no edge of its own"
    );
}

#[test]
fn a_constructor_stops_at_the_initialization_boundary() {
    // A constructor's `this` is `uninitializedThis` until its own constructor call runs, and the
    // conversions that flip it are 4.2's. The `invokespecial Object.<init>()V` the fixture's
    // constructor performs is therefore where this build stops: `Partial` under the frame
    // slice's own code, no phase behind it, and never a claim that the bytes contradict
    // themselves.
    let fixture = fixture(b"<init>", b"()V");
    let (report, _) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());
    assert_eq!(
        stage_states(&report),
        vec![
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Partial,
            StageState::NotPerformed,
        ],
        "the frame phase stopped and the phase behind it never ran"
    );
    assert_eq!(diagnostic_codes(&report), vec!["ir_frame_deferred"]);
    assert_eq!(
        diagnostic(&report, "ir_frame_deferred"),
        DiagnosticSeverity::Warning,
        "a boundary of this build is a warning, not damage of the class"
    );
    assert!(
        matches!(
            &report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::Error { code },
                ..
            } if code == "ir_frame_deferred"
        ),
        "a stop at the 4.2 boundary is a partial execution under its own code: {:?}",
        report.execution
    );
    assert_eq!(
        report.quality,
        Quality::Conservative,
        "the canonical CFG of the earlier phase is still the artifact this run produced"
    );
    assert_eq!(report.body, MethodBodyState::Present);
    assert_planes_stay_p1(&report);
}
