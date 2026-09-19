//! P2 4.3 acceptance through the public entry point: the stack/local names over the frames.
//!
//! The names themselves — the entry phis, their arity and their class, the throw-site inputs, the
//! category-2 pairing, the `Top` slots and the trivial-phi removal — are a crate-private payload
//! (invariant 11), so they are pinned by the unit tests of `crates/jarde-jvm/src/ssa.rs`, which
//! drive the pass over real and synthetic bodies and audit the published table against itself.
//! What this file proves through the public API is the wiring and the planes around that payload:
//!
//! 1. the `ssa` phase really runs over the frames of a committed fixture whose body carries no
//!    stack map or local-variable table, and completes the pipeline this build declares;
//! 2. it charges what its row declares: the names are derived items and def-use edges over a
//!    worklist, so a request that reaches the phase bills strictly more than the same request
//!    without it, deterministically and on the dimensions the row names;
//! 3. a stop inside the phase keeps the last valid phase: the frames stay `Completed`, no names
//!    are published, and the termination is the budget layer's own;
//! 4. and a cancellation is a cancellation, not a bound and not a set of names.
//!
//! In every case the product planes stay what they are: `verification` is `NotPerformed` — naming
//! the values of a body is not verifying a method — while `semantic_validation` is the run's own
//! evidence, so the request that completes the phase reports the local invariants it checked and
//! the two runs that stop short of that report `Unproven`.

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
    fixture_of(V52, name, descriptor)
}

/// The same for any class file: the identity of the definition is the digest of the bytes the
/// request is answered from, so a class assembled beside a committed one is not a special case.
fn fixture_of(class: &[u8], name: &[u8], descriptor: &[u8]) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("a class file length fits u64"),
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

/// A `u2` in the class-file byte order.
fn u16b(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

/// One real class file of `Test` with the single `public static` method `method`, whose body is
/// `code`, at class-file version `major` and with no exception table.
///
/// The pool is the smallest one a `Code` attribute needs — the class, its superclass, the member's
/// name and descriptor, and the attribute's own name — so a body assembled here is the body and
/// nothing else: no constant, no field, no debug attribute of any kind.
fn class_of(
    major: u16,
    descriptor: &[u8],
    code: &[u8],
    max_stack: u16,
    max_locals: u16,
) -> Vec<u8> {
    fn utf8(pool: &mut Vec<u8>, text: &[u8]) {
        pool.push(1);
        u16b(
            pool,
            u16::try_from(text.len()).expect("a fixture name fits u16"),
        );
        pool.extend_from_slice(text);
    }
    fn class(pool: &mut Vec<u8>, name: u16) {
        pool.push(7);
        u16b(pool, name);
    }

    let mut pool = Vec::new();
    utf8(&mut pool, b"Test"); // 1
    class(&mut pool, 1); // 2
    utf8(&mut pool, b"java/lang/Object"); // 3
    class(&mut pool, 3); // 4
    utf8(&mut pool, b"method"); // 5
    utf8(&mut pool, descriptor); // 6
    utf8(&mut pool, b"Code"); // 7

    let mut content = Vec::new();
    u16b(&mut content, max_stack);
    u16b(&mut content, max_locals);
    content.extend_from_slice(
        &u32::try_from(code.len())
            .expect("a fixture body fits u32")
            .to_be_bytes(),
    );
    content.extend_from_slice(code);
    u16b(&mut content, 0); // no exception table
    u16b(&mut content, 0); // no `Code` attribute

    let mut method = Vec::new();
    u16b(&mut method, 0x0009); // ACC_PUBLIC | ACC_STATIC
    u16b(&mut method, 5);
    u16b(&mut method, 6);
    u16b(&mut method, 1);
    u16b(&mut method, 7);
    method.extend_from_slice(
        &u32::try_from(content.len())
            .expect("a fixture Code content fits u32")
            .to_be_bytes(),
    );
    method.extend_from_slice(&content);

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0); // minor
    u16b(&mut bytes, major);
    u16b(&mut bytes, 8); // the pool holds entries 1..7
    bytes.extend_from_slice(&pool);
    u16b(&mut bytes, 0x0021); // ACC_PUBLIC | ACC_SUPER
    u16b(&mut bytes, 2); // this
    u16b(&mut bytes, 4); // super
    u16b(&mut bytes, 0); // interfaces
    u16b(&mut bytes, 0); // fields
    u16b(&mut bytes, 1); // methods
    bytes.extend_from_slice(&method);
    u16b(&mut bytes, 0); // class attributes
    bytes
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

/// The four product planes a P2 report always states, whatever the run proved: the P1 baseline
/// and the verifier status this build never raises.
///
/// `semantic_validation` is deliberately not one of them: since 4.3 it is the run's own evidence,
/// so each test below states it for the run that test really performed.
fn assert_planes_stay_p1(report: &MethodAnalysisReport) {
    assert_eq!(report.representation, Representation::Bytecode);
    assert_eq!(report.syntax_status, SyntaxStatus::NotJava);
    assert_eq!(report.compile_status, CompileStatus::NotAttempted);
    assert_eq!(report.verification, VerificationStatus::NotPerformed);
}

#[test]
fn the_ssa_phase_names_a_real_body_and_completes_the_pipeline() {
    // `finallyPath(I)I` of the 52 fixture is an instance method with real control flow, and its
    // calls compile to a `jsr`/`ret` subroutine the canonical phase clones per call site. The
    // names 4.3 derives are over exactly those frames: the phase completes, and it is the last
    // phase this build declares, so the request is answered as a complete run.
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
            StageState::Completed,
        ],
        "every phase this build declares ran: {:?}",
        report.diagnostics
    );
    assert!(
        diagnostic_codes(&report).is_empty(),
        "a completed pipeline reports no diagnostic: {:?}",
        diagnostic_codes(&report)
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(report.body, MethodBodyState::Present);
    assert_planes_stay_p1(&report);
    // This run's own evidence: the phase that checks the local invariants of the published IR —
    // one definition per value, def-use agreement both ways, one phi input per logical
    // predecessor, one value behind a category-2 pair — ran and completed.
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
    // The names are derived storage, so they are really billed: the phase's own row declares the
    // item and edge dimensions it charges.
    assert!(budget.usage().ir_items > 0);
    assert!(budget.usage().ir_edges > 0);
}

/// A loop whose head phi takes **its own value** as an operand, and the run that used to refuse it.
///
/// The 27 code bytes are the minimal reproducer of the second defect the P2 property corpus found
/// (`tests/p2_properties.rs` suspended three of its generated legal bodies for it, and the fuzz
/// side never had to: the shape is a loop, not an exception). A JVM verifies and links the body —
/// it was checked with Corretto 1.8.0_432 through `defineClass` plus `Class.forName(name, true,
/// loader)` — so the contradiction this build reported about it was about its own artifacts.
///
/// ```text
///  0: iconst_1          locals 0, 1 and 2 are one
///  1: istore_0
///  2: iconst_1
///  3: istore_1
///  4: iconst_1
///  5: istore_2
///  6: iconst_1
///  7: ifeq 10           the entry block, whose two branches both land on BCI 10
/// 10: iconst_1          the loop head: 11 branches to itself
/// 11: ifeq 10
/// 14: iconst_1
/// 15: ifeq 10
/// 18: iload_1; iconst_1; iadd; istore_2    local 2 becomes 2
/// 22: iconst_1
/// 23: ifeq 14
/// 26: return
/// ```
///
/// Local 0 at BCI 10 is the merge of three logical inputs — the entry's value, the branch the head
/// takes back to itself, and the path through BCI 14 — and nothing writes that local inside the
/// loop, so BCI 14's own phi for it is folded into the head's value. The head's phi therefore ends
/// up taking **its own value** as one of its operands, and once every path hands it one value it is
/// replaced by it. What the run used to report was `ir_ssa_inconsistent` — "one value is named as
/// a phi operand 2 time(s) while the phis hold it as an operand 1 time(s)" — a count of this
/// build's own records and not a contradiction of the body, because the operand the fold rewrote
/// named the value its own phi defines.
#[test]
fn a_phi_that_takes_its_own_value_as_an_operand_is_still_legal() {
    /// The body, byte for byte, as the suspended corpus entries state it.
    const CODE: &[u8] = &[
        0x04, 0x3b, 0x04, 0x3c, 0x04, 0x3d, 0x04, 0x99, 0x00, 0x03, 0x04, 0x99, 0xff, 0xff, 0x04,
        0x99, 0xff, 0xfb, 0x1b, 0x04, 0x60, 0x3d, 0x04, 0x99, 0xff, 0xf7, 0xb1,
    ];
    // The branch operands really land where the listing above says: 7 + 3 = 10, 11 - 1 = 10,
    // 15 - 5 = 10 and 23 - 9 = 14.
    for (operand, expected) in [(8, 10_i32), (12, 10), (16, 10), (24, 14)] {
        assert_eq!(
            i32::from(i16::from_be_bytes([CODE[operand], CODE[operand + 1]])),
            expected - i32::try_from(operand - 1).expect("a fixture body fits i32"),
            "the branch at BCI {} names its target",
            operand - 1
        );
    }
    // The five other bytes of its `Code`: major 49 is the dialect of the reported reproducer, and
    // the declared maxima are the ones its discoverer recorded.
    let class = class_of(49, b"()V", CODE, 4, 3);
    let fixture = fixture_of(&class, b"method", b"()V");
    let (report, _) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());
    assert_eq!(
        stage_states(&report),
        vec![
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
        ],
        "the whole pipeline completes over the loop: {:?}",
        report.diagnostics
    );
    assert!(
        diagnostic_codes(&report).is_empty(),
        "a legal body reports no contradiction: {:?}",
        report.diagnostics
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(report.body, MethodBodyState::Present);
    assert_planes_stay_p1(&report);
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
}

#[test]
fn the_names_of_a_body_are_billed_and_charged_deterministically() {
    let fixture = fixture(b"finallyPath", b"(I)I");
    let (_, names) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());
    let (_, frames_only) = analyze(&fixture, vec![AnalysisStage::Frame], limits());
    for (dimension, reached, below) in [
        (
            CountedBudgetDimension::IrItems,
            names.usage().ir_items,
            frames_only.usage().ir_items,
        ),
        (
            CountedBudgetDimension::IrEdges,
            names.usage().ir_edges,
            frames_only.usage().ir_edges,
        ),
        (
            CountedBudgetDimension::AnalysisSteps,
            names.usage().analysis_steps,
            frames_only.usage().analysis_steps,
        ),
    ] {
        assert!(
            reached > below,
            "{dimension:?} is charged by the names' own row: {reached} vs {below}"
        );
    }
    // Determinism: the same request charges the same amount, dimension by dimension.
    let (report, again) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    for dimension in CountedBudgetDimension::ALL {
        assert_eq!(
            names.usage().counted_usage(dimension),
            again.usage().counted_usage(dimension),
            "{dimension:?} is charged deterministically"
        );
    }
}

#[test]
fn a_step_budget_that_ends_the_ssa_phase_keeps_the_last_valid_phase() {
    // The exact price of the phases up to the frames, plus one step: the `ssa` phase starts, runs
    // out of the step dimension and stops. The frames it read stay `Completed`, no names are
    // published for the request, and the termination is the budget layer's own rather than a claim
    // about the body.
    let fixture = fixture(b"finallyPath", b"(I)I");
    let (_, frames_only) = analyze(&fixture, vec![AnalysisStage::Frame], limits());
    let mut stopped = limits();
    stopped.analysis_steps = frames_only.usage().analysis_steps + 1;
    let (report, _) = analyze(&fixture, vec![AnalysisStage::Ssa], stopped);
    assert_eq!(
        stage_states(&report),
        vec![
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Partial,
        ],
        "the frames stay the last valid phase"
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps
            },
            ..
        }
    ));
    assert_eq!(
        diagnostic_codes(&report),
        vec!["budget_exceeded_analysis_steps"]
    );
    assert_eq!(
        report.quality,
        Quality::Conservative,
        "a published canonical graph is still the artifact of the run"
    );
    assert_planes_stay_p1(&report);
    // The phase stopped inside itself, so it never reached the checks that would be this run's
    // semantic evidence: `Partial` is not a completed phase and the plane stays `Unproven`.
    assert_eq!(report.semantic_validation, SemanticValidation::Unproven);
}

#[test]
fn a_cancelled_request_publishes_no_names() {
    // Cancellation is not a bound of this phase and must not be reported as one: the run stops
    // under its own termination and no phase publishes anything.
    let fixture = fixture(b"finallyPath", b"(I)I");
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let request = MethodAnalysisRequest {
        environment: environment(&fixture),
        method: fixture.method.clone(),
        stages: vec![AnalysisStage::Ssa],
    };
    let report = Engine::new()
        .analyze_method(slice::from_ref(&fixture.snapshot), &request, &mut budget)
        .expect("a cancelled request is answered, not raised");
    assert!(matches!(
        report.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert_eq!(report.quality, Quality::Fallback);
    assert_eq!(
        report
            .stages
            .iter()
            .filter(|stage| stage.state == StageState::Completed)
            .count(),
        0,
        "a cancelled run publishes nothing"
    );
    // Cancellation is not evidence either: no phase completed, so the run proved nothing about
    // the local invariants — and `Cancelled` must not be read as a claim about the body.
    assert_eq!(report.semantic_validation, SemanticValidation::Unproven);
}
