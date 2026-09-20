//! P2 5.3's property tests over **generated** method bodies.
//!
//! The goldens replay a fixed list of inputs; this file drives a fixed-seed generator over many
//! bodies and asserts the invariants that must hold for *every* one of them, whatever the body
//! says. Both directions of the pipeline's own implication are covered by construction: the
//! corpus holds bodies the whole pipeline completes and bodies that stop in one of its phases,
//! and every body of either family is checked against the same stage and plane rules.
//!
//! The generator is written here rather than taken from a property-test crate: the corpus has to
//! be reproducible from the source alone (the seed is a constant, the PRNG is thirty lines), and
//! the number of bodies is part of the assertion instead of a run-dependent detail.
//!
//! What is asserted publicly, per body:
//!
//! * the product planes are the P2 baseline (`Bytecode`, `NotJava`, `NotAttempted`,
//!   `NotPerformed`) — no stop, missing dependency or derived frame may move them;
//! * `semantic_validation` is `LocalInvariants` **exactly** when the `ssa` phase completed, in
//!   both directions, and every stop leaves it `Unproven`;
//! * the stage list is a prefix pattern: a run of `Completed` phases, at most one phase that
//!   stopped, and `NotPerformed` after it — which is what "the published prefix is still there"
//!   means at this layer;
//! * a stop carries the diagnostic its own termination names, and a `Complete` run carries none;
//! * a run that published a canonical graph is `Conservative` and one that stopped before that
//!   phase is `Fallback`; coverage is published exactly when a phase completed;
//! * a body analyzed twice answers with the same report (5.2's determinism property, over
//!   generated inputs).
//!
//! The invariants that are only visible inside the crate — the reachable-set coverage of the
//! canonical graph, the def-use agreement in both directions, the phi arity against the logical
//! predecessors and the origin mapping of a clone — are asserted in `crates/jarde-jvm/src/**`,
//! because the report keeps the IR payload private (design invariant 11). This file cannot see
//! them and does not pretend to.

use jarde::*;
use std::slice;

// ---------------------------------------------------------------------------
// A reproducible corpus
// ---------------------------------------------------------------------------

/// The seed of the corpus. Changing it changes every generated body, so it is part of the
/// expectation: the counts below are asserted with it.
const SEED: u64 = 0x5eed_2026_0919;

/// How many bodies of each family the corpus holds.
const LEGAL_BODIES: usize = 48;
const ILLEGAL_BODIES: usize = 24;

/// The generated **legal** bodies this build refuses, by name.
///
/// The table used to hold three of them — `legal-7`, `legal-22` and `legal-exception-18` — which
/// this build refused with the pass's own contradiction (`ir_ssa_inconsistent`, "one value is
/// named as a phi operand 2 time(s) while the phis hold it as an operand 1 time(s)") on a loop a
/// real JVM verifies and links. That was a defect of `crates/jarde-jvm/src/ssa.rs` and not a fact
/// about the bodies, so it was suspended here rather than written down as the expected behavior:
/// the suspension named the three bodies by their code bytes, and the test below asserted that the
/// refused set was *exactly* the suspended one, so a fourth body failing and the fix landing
/// without this list being emptied both failed here.
///
/// The fix has landed and the table is empty, which is now the statement itself: **every** body of
/// the legal family completes, and a generated body this build starts refusing again fails the
/// corpus instead of being suspended silently. The three bodies stay in the corpus as the
/// regression, and the defect's minimal reproducer is the public-entry case
/// `a_phi_that_takes_its_own_value_as_an_operand_is_still_legal` of `tests/p2_ssa.rs`.
const SUSPENDED: [&str; 0] = [];

/// The corpus bodies the emptied suspension was about, frozen by their code bytes: `legal-7`,
/// `legal-22` and `legal-exception-18` are the shapes that reproduced the defect, so a change to
/// the generator that quietly replaced them must fail here instead of taking the regression with
/// it — the corpus is generated from a fixed seed and its bodies are otherwise unnamed.
///
/// The minimal reproducer of the same defect is the 27-byte loop the public-entry case
/// `a_phi_that_takes_its_own_value_as_an_operand_is_still_legal` of `tests/p2_ssa.rs` states, and
/// the bytes below are the generated shapes of the same family (a loop whose trivial phi takes
/// itself as an operand, with and without a catch-all record over the body).
const REFUSED_REGRESSION_BODIES: [(&str, &str); 3] = [
    (
        "legal-7",
        "043b043c043d1a06603c1a06603b0499001f1a05603b049900171b04603b1a07603c0499ffef1a05603c\
         0499ffef1a07603bb1",
    ),
    (
        "legal-22",
        "043b043c043d043e1b07603c049900031d04603e1d07603e0499000f1d06603c1a04603c0499ffeb1d\
         04603ea7ffe41c05603c0499ffcc1c07603c1d05603eb1",
    ),
    (
        "legal-exception-18",
        "04036c57043b043c043d043e1b05603d0499fff31a05603b1c07603d049900031b07603c1c05603c\
         a7000f1b05603e1c07603c0499fff71d06603e1b04603da7ffec1d07603bb157b1",
    ),
];

/// A SplitMix64: thirty lines of arithmetic, no dependency, and the same corpus on every run.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn below(&mut self, bound: u32) -> u32 {
        u32::try_from(self.next() % u64::from(bound)).expect("a remainder of a u64 fits u32")
    }
}

// ---------------------------------------------------------------------------
// The class file the generated bodies are put in
// ---------------------------------------------------------------------------

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 16,
        output_bytes: 1 << 20,
        class_headers: 10,
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 16,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

fn u16b(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

/// A constant pool of `Utf8` and `Class` entries: it answers the index of the entry each append
/// wrote, so a caller can name it.
#[derive(Default)]
struct Pool {
    bytes: Vec<u8>,
    count: u16,
}

impl Pool {
    fn push(&mut self, entry: &[u8]) -> u16 {
        self.bytes.extend_from_slice(entry);
        self.count += 1;
        self.count
    }

    fn utf8(&mut self, text: &[u8]) -> u16 {
        let mut entry = vec![1];
        u16b(
            &mut entry,
            u16::try_from(text.len()).expect("a fixture name fits u16"),
        );
        entry.extend_from_slice(text);
        self.push(&entry)
    }

    fn class(&mut self, name: u16) -> u16 {
        let mut entry = vec![7];
        u16b(&mut entry, name);
        self.push(&entry)
    }
}

/// A class file of one class `Test extends java/lang/Object` with one `public static` method
/// `method` whose body is `code`, its declared stack and local budget, and its catch-all
/// exception table.
fn class_file(
    major: u16,
    descriptor: &[u8],
    code: &[u8],
    max_stack: u16,
    max_locals: u16,
    handlers: &[(u16, u16, u16)],
) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_name = pool.utf8(b"Test");
    let this_class = pool.class(this_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let name_index = pool.utf8(b"method");
    let descriptor_index = pool.utf8(descriptor);
    let code_name = pool.utf8(b"Code");

    let mut content = Vec::new();
    u16b(&mut content, max_stack);
    u16b(&mut content, max_locals);
    content.extend_from_slice(
        &u32::try_from(code.len())
            .expect("a fixture body fits u32")
            .to_be_bytes(),
    );
    content.extend_from_slice(code);
    u16b(
        &mut content,
        u16::try_from(handlers.len()).expect("fixture handlers fit u16"),
    );
    for (start, end, handler) in handlers {
        u16b(&mut content, *start);
        u16b(&mut content, *end);
        u16b(&mut content, *handler);
        u16b(&mut content, 0); // catch-all: the fixture's pool names no exception class
    }
    u16b(&mut content, 0); // no debug attribute of any kind

    let mut method = Vec::new();
    u16b(&mut method, 0x0009); // ACC_PUBLIC | ACC_STATIC
    u16b(&mut method, name_index);
    u16b(&mut method, descriptor_index);
    u16b(&mut method, 1);
    u16b(&mut method, code_name);
    method.extend_from_slice(
        &u32::try_from(content.len())
            .expect("fixture Code content fits u32")
            .to_be_bytes(),
    );
    method.extend_from_slice(&content);

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0); // minor
    u16b(&mut bytes, major);
    u16b(&mut bytes, pool.count + 1);
    bytes.extend_from_slice(&pool.bytes);
    u16b(&mut bytes, 0x0021); // ACC_PUBLIC | ACC_SUPER
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, object_class);
    u16b(&mut bytes, 0); // interfaces
    u16b(&mut bytes, 0); // fields
    u16b(&mut bytes, 1); // methods
    bytes.extend_from_slice(&method);
    u16b(&mut bytes, 0); // class attributes
    bytes
}

// ---------------------------------------------------------------------------
// Running one generated body and asserting the invariants
// ---------------------------------------------------------------------------

/// One generated body in the environment a request names: the snapshot its bytes opened as, and
/// the physical identity of the method the request analyzes.
struct Fixture {
    snapshot: ArtifactSnapshot,
    method: PhysicalMethodId,
    environment: ResolutionEnvironment,
}

fn fixture(class: &[u8], descriptor: &[u8]) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("a generated class file opens as a standalone CLASS");
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
        name: JvmBytes(b"method".to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
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
    Fixture {
        snapshot,
        method,
        environment,
    }
}

fn analyze(fixture: &Fixture) -> MethodAnalysisReport {
    let request = MethodAnalysisRequest {
        environment: fixture.environment.clone(),
        method: fixture.method.clone(),
        stages: vec![AnalysisStage::Ssa],
    };
    let mut budget = Budget::new(limits());
    Engine::new()
        .analyze_method(slice::from_ref(&fixture.snapshot), &request, &mut budget)
        .expect("a legal request is answered, not raised")
}

/// Runs one generated body through the entry point its descriptor needs.
fn analyze_body(body: &Generated) -> MethodAnalysisReport {
    let fixture = fixture(&body.class, body.descriptor);
    analyze(&fixture)
}

fn diagnostic_codes(report: &MethodAnalysisReport) -> Vec<String> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.clone())
        .collect()
}

/// The stage and plane invariants every body must keep, complete or stopped.
///
/// The returned state is the `ssa` phase's, which is what the corpus counts: the two directions
/// the implication above has to have instances of.
fn assert_stage_invariants(name: &str, report: &MethodAnalysisReport) -> StageState {
    // The product planes are the P2 baseline for every body: a stop, a missing dependency or a
    // derived frame moves none of them.
    assert_eq!(report.representation, Representation::Bytecode, "{name}");
    assert_eq!(report.syntax_status, SyntaxStatus::NotJava, "{name}");
    assert_eq!(report.compile_status, CompileStatus::NotAttempted, "{name}");
    assert_eq!(
        report.verification,
        VerificationStatus::NotPerformed,
        "{name}"
    );

    // The stage list is monotone in phase order: the phases that did their work come first
    // (`Completed`), then the ones that worked over an incomplete prefix or stopped inside
    // themselves (`Partial`), then the ones that never ran (`NotPerformed`). A `Completed` phase
    // after an incomplete one would state that a later phase did its whole work on inputs an
    // earlier one only partly produced, which is exactly what a truncated body must not claim.
    let mut incomplete = false;
    for stage in &report.stages {
        match &stage.state {
            StageState::Completed => assert!(
                !incomplete,
                "{name}: a completed stage appears after an incomplete one: {:?}",
                report.stages
            ),
            StageState::Partial => incomplete = true,
            StageState::Failed { code } => {
                assert!(!code.is_empty(), "{name}: a failed stage names its code");
                incomplete = true;
            }
            StageState::NotPerformed | StageState::NotRequested => incomplete = true,
        }
    }
    // Everything the request asked for was scheduled.
    for stage in &report.requested_stages {
        assert!(
            report.stages.iter().any(|result| result.stage == *stage),
            "{name}: the requested stage {stage:?} is scheduled"
        );
    }

    let ssa = report
        .stages
        .iter()
        .find(|stage| stage.stage == AnalysisStage::Ssa)
        .expect("every generated request schedules the names phase");

    // The one implication the planes carry, in both directions.
    if ssa.state == StageState::Completed {
        assert_eq!(
            report.semantic_validation,
            SemanticValidation::LocalInvariants,
            "{name}: a completed names phase is exactly the evidence that the local invariants \
             held over the names it published"
        );
    } else {
        assert_eq!(
            report.semantic_validation,
            SemanticValidation::Unproven,
            "{name}: no other stage state is evidence of anything"
        );
    }

    match &report.execution {
        ExecutionReport::Complete { .. } => {
            assert!(
                report
                    .stages
                    .iter()
                    .all(|stage| stage.state == StageState::Completed),
                "{name}: a complete run has no stopped phase: {:?}",
                report.stages
            );
            assert!(
                report.diagnostics.is_empty(),
                "{name}: a complete run reports no diagnostic: {:?}",
                diagnostic_codes(report)
            );
            assert_eq!(
                report.quality,
                Quality::Conservative,
                "{name}: a completed pipeline published a canonical graph"
            );
        }
        ExecutionReport::Partial { reason, .. } | ExecutionReport::Failed { reason, .. } => {
            assert!(
                !report.diagnostics.is_empty(),
                "{name}: a stop is explained by a diagnostic"
            );
            let expected = match reason {
                TerminationReason::BudgetExceeded { dimension } => {
                    format!("budget_exceeded_{}", budget_dimension_code(*dimension))
                }
                TerminationReason::Error { code } | TerminationReason::Unsupported { code } => {
                    code.clone()
                }
            };
            assert!(
                report
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == expected),
                "{name}: the stop names its own code {expected:?}: {:?}",
                diagnostic_codes(report)
            );
            // Quality follows the artifact: a run that published the canonical graph is
            // `Conservative`, and one that stopped before that phase is `Fallback`. The canonical
            // phase is `Partial` in **two** different situations — the graph of a reliable decoded
            // prefix was published, or the normalization refused the body — and the stage state
            // alone cannot tell them apart at this layer: the refusal's own code is what does,
            // and the run carries it as a diagnostic. That is an observability limit of the
            // public report, not a rule of the artifact.
            let canonical = report
                .stages
                .iter()
                .find(|stage| stage.stage == AnalysisStage::CanonicalCfg)
                .expect("the canonical phase is scheduled");
            let refused = report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "ir_legacy_normalization_unbounded");
            let published_graph = canonical.state == StageState::Completed
                || (canonical.state == StageState::Partial && !refused);
            let expected_quality = if published_graph {
                Quality::Conservative
            } else {
                Quality::Fallback
            };
            assert_eq!(report.quality, expected_quality, "{name}");
        }
        ExecutionReport::Cancelled { .. } => {
            panic!("{name}: the corpus runs with no cancellation token")
        }
    }

    // Coverage is published exactly when a phase published something: a run that published
    // nothing states no coverage, and one that ran its first phase states one (over the prefix it
    // could read, when the decode stopped early).
    let published = report
        .stages
        .iter()
        .any(|stage| matches!(stage.state, StageState::Completed | StageState::Partial));
    if published {
        assert_ne!(
            report.coverage.artifact_structural.state,
            CoverageState::NotRequested,
            "{name}: the body the completed phases read keeps its coverage"
        );
    } else {
        assert_eq!(
            report.coverage,
            Coverage::not_requested(),
            "{name}: a run that published nothing states no coverage"
        );
    }

    ssa.state.clone()
}

/// The invariants a legal body adds to the stage rules: the whole pipeline completes over it and
/// the planes around it are the ones a completed run states.
fn assert_legal(name: &str, report: &MethodAnalysisReport) {
    assert!(
        matches!(report.execution, ExecutionReport::Complete { .. }),
        "{name}: a legal body completes: {:?} {:?}",
        report.execution,
        diagnostic_codes(report)
    );
    assert_eq!(
        report.stages.len(),
        AnalysisStage::ALL.len(),
        "{name}: the whole pipeline is scheduled"
    );
    assert_eq!(report.body, MethodBodyState::Present, "{name}");
}

#[test]
fn the_generated_corpus_keeps_the_stage_invariants_in_both_directions() {
    let bodies = corpus();
    let mut legal = 0;
    let mut illegal = 0;
    let mut completed = 0;
    let mut uncompleted = 0;
    let mut refused = Vec::new();
    for body in &bodies {
        let report = analyze_body(body);
        let state = assert_stage_invariants(&body.name, &report);
        if body.legal() && !matches!(report.execution, ExecutionReport::Complete { .. }) {
            // A legal body this build refuses is a defect of the build and never an expectation of
            // this corpus: the name is recorded so the assertion below states the rule over the
            // whole corpus, and `assert_legal` reports this body's own diagnostics right here.
            refused.push(body.name.as_str());
            assert_legal(&body.name, &report);
        }
        if body.legal() {
            legal += 1;
        } else {
            illegal += 1;
            assert!(
                matches!(
                    report.execution,
                    ExecutionReport::Partial { .. } | ExecutionReport::Failed { .. }
                ),
                "{}: the illegal shape {:?} stops instead of completing: {:?} {:?}",
                body.name,
                body.shape,
                report.execution,
                diagnostic_codes(&report)
            );
        }
        match state {
            StageState::Completed => completed += 1,
            _ => uncompleted += 1,
        }
    }
    assert_eq!(
        refused, SUSPENDED,
        "every legal body of the corpus completes, and none is suspended: a legal body this build \
         refuses is a defect, and the table above is the one place a suspension would be named"
    );
    // The bodies the defect was reported on are still in the corpus, by their bytes: the corpus is
    // generated, so this is the only thing that names them.
    for (name, expected) in REFUSED_REGRESSION_BODIES {
        let body = bodies
            .iter()
            .find(|body| body.name == name)
            .unwrap_or_else(|| panic!("the corpus holds the regression body {name}"));
        let code = body
            .code
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<_>>()
            .join("");
        assert_eq!(
            code, *expected,
            "{name}: the regression names this body by its bytes, not by its position in the \
             generator"
        );
    }
    // Both directions of the implication have instances of their own, which is what makes the
    // biconditional above a statement about the corpus rather than about one shape.
    assert_eq!(legal, LEGAL_BODIES + LEGAL_BODIES / 2);
    assert_eq!(illegal, ILLEGAL_BODIES);
    assert_eq!(
        completed,
        legal,
        "every legal body completes the names phase ({completed} of {} bodies)",
        bodies.len()
    );
    assert_eq!(
        uncompleted, illegal,
        "every illegal body leaves the names phase uncompleted"
    );
    println!(
        "generated bodies: {} (legal {}, illegal {}); names phase completed {}, uncompleted {}",
        bodies.len(),
        legal,
        illegal,
        completed,
        uncompleted
    );
}

#[test]
fn the_generated_illegal_shapes_stop_where_the_prefix_says() {
    let bodies = corpus();
    let illegal: Vec<&Generated> = bodies.iter().filter(|body| !body.legal()).collect();
    assert_eq!(illegal.len(), ILLEGAL_BODIES);
    let mut shapes: Vec<&'static str> = Vec::new();
    for body in &illegal {
        let shape = body.shape.expect("an illegal body names its shape");
        shapes.push(shape);
        let report = analyze_body(body);
        assert!(
            !matches!(report.execution, ExecutionReport::Complete { .. }),
            "the illegal shape {shape:?} is not answered as a complete run: {:?}",
            report.execution
        );
        assert!(
            !report.diagnostics.is_empty(),
            "the illegal shape {shape:?} is explained by a diagnostic"
        );
        assert_eq!(
            report.semantic_validation,
            SemanticValidation::Unproven,
            "the illegal shape {shape:?} proves nothing about the local invariants"
        );
        // The refusal happens at a phase of the pipeline, not at a plane: the phases before it
        // stay published, which is what the prefix pattern in `assert_stage_invariants` states.
        let stopped_at = report
            .stages
            .iter()
            .position(|stage| {
                !matches!(
                    stage.state,
                    StageState::Completed | StageState::NotPerformed
                )
            })
            .expect("a stopped run has one phase that did not complete");
        assert_eq!(
            stopped_at,
            report
                .stages
                .iter()
                .take_while(|stage| stage.state == StageState::Completed)
                .count(),
            "the published prefix of {shape:?} is the run of completed phases before the stop"
        );
    }
    for shape in [
        "a local no path defined",
        "a join of two different stack depths",
        "a return of a value the stack does not hold",
        "a jsr in the modern dialect",
        "a ret in the modern dialect",
        "an int addition with one operand",
        "an instruction the decode stops inside of",
    ] {
        assert!(
            shapes.contains(&shape),
            "the corpus holds the {shape:?} shape: {shapes:?}"
        );
    }
}

#[test]
fn the_generated_bodies_answer_deterministically() {
    // 5.2's determinism property over generated inputs: the same bytes and the same request
    // answer with the same report, field by field (minus the one wall-clock field).
    fn strip(report: &MethodAnalysisReport) -> serde_json::Value {
        fn normalize(value: &mut serde_json::Value) {
            match value {
                serde_json::Value::Object(map) => {
                    map.remove("elapsed_millis");
                    for child in map.values_mut() {
                        normalize(child);
                    }
                }
                serde_json::Value::Array(items) => {
                    for item in items {
                        normalize(item);
                    }
                }
                _ => {}
            }
        }
        let mut value = serde_json::to_value(report).expect("a report serializes");
        normalize(&mut value);
        value
    }
    let bodies = corpus();
    let mut compared = 0;
    for (index, body) in bodies.iter().enumerate() {
        if index % 4 != 0 {
            continue;
        }
        let first = analyze_body(body);
        let second = analyze_body(body);
        assert_eq!(strip(&first), strip(&second), "{}", body.name);
        compared += 1;
    }
    assert!(
        compared >= 15,
        "the determinism check compared {compared} of {} bodies",
        bodies.len()
    );
}

/// One generated body: the class file it lives in, the descriptor of its method, and whether the
/// corpus expects the pipeline to complete over it.
struct Generated {
    name: String,
    descriptor: &'static [u8],
    class: Vec<u8>,
    code: Vec<u8>,
    /// `Some(shape)` for the illegal family: the rule the body breaks, so the tests can name the
    /// shapes the corpus really drew.
    shape: Option<&'static str>,
}

impl Generated {
    fn legal(&self) -> bool {
        self.shape.is_none()
    }
}

/// A body of `blocks` blocks over `locals` integer locals: the first block hands every local a
/// value, each block does a little arithmetic read-and-write on them, and every block ends in
/// either the method's `return`, a `goto` or a taken conditional branch to another block.
///
/// Nothing here can read a local no path defined, so this is the *legal* family: the pipeline has
/// to complete over every body it produces.
fn legal_body(rng: &mut Rng) -> (Vec<u8>, u16, u16) {
    let blocks = 3 + rng.below(6); // 3..8
    let locals = 2 + rng.below(3); // 2..4
    let mut code = Vec::new();
    let mut starts: Vec<usize> = Vec::new();
    // Two passes: the branch operands are written as placeholders while the blocks are emitted —
    // a branch may name a block that does not exist yet — and fixed up once every block start is
    // known. The entries are (operand position, BCI of the opcode, target block index).
    let mut fixups: Vec<(usize, u32, usize)> = Vec::new();
    for index in 0..blocks {
        starts.push(code.len());
        if index == 0 {
            for local in 0..locals {
                code.push(0x04); // iconst_1
                code.push(0x3b + u8::try_from(local).expect("a local index fits u8"));
            }
        }
        for _ in 0..=rng.below(2) {
            let read = u8::try_from(rng.below(locals)).expect("a local index fits u8");
            let written = u8::try_from(rng.below(locals)).expect("a local index fits u8");
            let value = u8::try_from(rng.below(4)).expect("a small constant fits u8");
            code.push(0x1a + read); // iload_<read>
            code.push(0x04 + value); // iconst_<value + 1>
            code.push(0x60); // iadd
            code.push(0x3b + written); // istore_<written>
        }
        if index + 1 == blocks {
            code.push(0xb1); // return
            continue;
        }
        let target = usize::try_from(rng.below(blocks)).expect("a block index fits usize");
        if rng.below(4) == 0 {
            let opcode_at = u32::try_from(code.len()).expect("a fixture body fits u32");
            code.push(0xa7); // goto
            code.push(0);
            code.push(0);
            fixups.push((code.len() - 2, opcode_at, target));
        } else {
            code.push(0x04); // iconst_1: the condition the branch takes
            let opcode_at = u32::try_from(code.len()).expect("a fixture body fits u32");
            code.push(0x99); // ifeq
            code.push(0);
            code.push(0);
            fixups.push((code.len() - 2, opcode_at, target));
        }
    }
    for (position, from, target) in fixups {
        let offset = i32::try_from(starts[target]).expect("a fixture body fits i32")
            - i32::try_from(from).expect("a fixture body fits i32");
        let offset = i16::try_from(offset).expect("a generated branch fits i16");
        code[position..position + 2].copy_from_slice(&offset.to_be_bytes());
    }
    (
        code,
        4,
        u16::try_from(locals).expect("a local count fits u16"),
    )
}

/// A body of the legal family with a catch-all record over it: `iconst_1; iconst_0; idiv; pop`
/// opens the body, one record covers it, and the handler at the end pops the caught reference and
/// returns. Every block start moves by the four bytes the prefix adds, and the branch operands
/// stay correct because both ends of every branch move together.
fn legal_exception_body(rng: &mut Rng) -> (Vec<u8>, u16, u16, Handlers) {
    const PREFIX: &[u8] = &[0x04, 0x03, 0x6c, 0x57];
    let (code, stack, locals) = legal_body(rng);
    let mut shifted = PREFIX.to_vec();
    shifted.extend_from_slice(&code);
    let body_end = u16::try_from(shifted.len()).expect("a fixture body fits u16");
    shifted.extend_from_slice(&[0x57, 0xb1]); // the handler: pop the reference; return
    (shifted, stack.max(2), locals, vec![(0, body_end, body_end)])
}

/// One catch-all exception table: `(start, end, handler)` per record.
type Handlers = Vec<(u16, u16, u16)>;

/// One body of the **illegal** family, drawn from seven fixed shapes: six this build's own phases
/// refuse (an undefined read, a stack disagreement, a missing operand, a `jsr`/`ret` in the modern
/// dialect, an underflow) and one the reader's own decode stops inside of. Each is a real class
/// file whose method the JVM's rules, this build's dialect rules or the decoder reject.
fn illegal_body(rng: &mut Rng) -> (Vec<u8>, u16, u16, u16, &'static str) {
    match rng.below(7) {
        0 => (
            vec![0x1a, 0xb1], // iload_0; return, with no write of local 0 on any path
            1,
            1,
            52,
            "a local no path defined",
        ),
        1 => (
            vec![0x03, 0x99, 0x00, 0x04, 0x03, 0xb1], // two arms, two stack depths at the join
            1,
            0,
            52,
            "a join of two different stack depths",
        ),
        2 => (
            vec![0x04, 0x57, 0xac], // iconst_1; pop; ireturn
            1,
            0,
            52,
            "a return of a value the stack does not hold",
        ),
        3 => (
            vec![0xa8, 0x00, 0x03, 0xb1, 0x4b, 0xa9, 0x00], // jsr to 3; `astore_0`; `ret 0`
            1,
            1,
            52,
            "a jsr in the modern dialect",
        ),
        4 => (
            vec![0x03, 0x3b, 0xa9, 0x00, 0xb1], // iconst_0; istore_0; ret 0; return
            1,
            1,
            52,
            "a ret in the modern dialect",
        ),
        5 => (
            vec![0x03, 0x60, 0xb1], // iconst_0; iadd; return: the addition has one operand
            1,
            0,
            52,
            "an int addition with one operand",
        ),
        6 => (
            vec![0x10], // a `bipush` whose operand byte never arrives: the decode stops
            1,
            0,
            52,
            "an instruction the decode stops inside of",
        ),
        other => panic!("the corpus draws no illegal shape {other}"),
    }
}

/// The whole corpus: the legal family, the legal exception shapes, and the illegal family.
fn corpus() -> Vec<Generated> {
    let mut rng = Rng(SEED);
    let mut bodies = Vec::new();
    for index in 0..LEGAL_BODIES {
        let (code, stack, locals) = legal_body(&mut rng);
        bodies.push(Generated {
            name: format!("legal-{index}"),
            descriptor: b"()V",
            class: class_file(52, b"()V", &code, stack, locals, &[]),
            code,
            shape: None,
        });
    }
    for index in 0..LEGAL_BODIES / 2 {
        let (code, stack, locals, handlers) = legal_exception_body(&mut rng);
        bodies.push(Generated {
            name: format!("legal-exception-{index}"),
            descriptor: b"()V",
            class: class_file(52, b"()V", &code, stack, locals, &handlers),
            code,
            shape: None,
        });
    }
    for index in 0..ILLEGAL_BODIES {
        let (code, stack, locals, major, shape) = illegal_body(&mut rng);
        let descriptor: &'static [u8] = if shape == "a return of a value the stack does not hold" {
            b"()I"
        } else {
            b"()V"
        };
        bodies.push(Generated {
            name: format!("illegal-{index}"),
            descriptor,
            class: class_file(major, descriptor, &code, stack, locals, &[]),
            code,
            shape: Some(shape),
        });
    }
    bodies
}
