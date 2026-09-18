//! P2 3.5 acceptance: the bounded `jsr`/`ret` normalization and its CanonicalCFG through the
//! public entry point.
//!
//! The canonical graph itself is a crate-private payload (invariant 11), so the values are pinned
//! by the unit tests of `crates/jarde-jvm/src/canonical.rs`, which decode the committed ECJ
//! fixtures and build synthetic bodies directly. What this file proves through the public API is
//! the wiring, the quality plane and the stops around that payload:
//!
//! 1. the historical `jsr`/`ret` `finally` of the ECJ 4.6 corpus really canonicalizes: the
//!    `canonical_cfg` phase completes, it bills **two** `NormalizationClones` — one per call site
//!    of the one shared subroutine, the live one and the one the raw graph cannot enter — and the
//!    report's quality plane is `Conservative` because a canonical artifact was produced;
//! 2. the modern dialect of the same source has no clone to bill and still canonicalizes;
//! 3. a clone bound of exactly what the body needs completes; one clone less stops the phase with
//!    `Partial` under `ir_legacy_normalization_unbounded`, publishes no canonical fact (the phase
//!    behind it stays `NotPerformed`), and leaves the quality plane at `Fallback`;
//! 4. an exhausted step budget stops the same way, with the call contexts of 3.4b kept;
//! 5. a cancelled request keeps its own termination and never claims a canonical artifact;
//! 6. a `jsr` on an exception path and nested, shared subroutines normalize through the public
//!    entry with real bytes, and two runs of one request are the same report;
//! 7. A17 holds in behaviour: a canonicalization leaves the P1 query coordinates and their count
//!    unchanged.

use jarde::*;
use std::slice;

/// The committed historical fixtures: one source compiled to every dialect of the `jsr` era,
/// plus the modern one that inlines the same `finally`.
const V45: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class");
const V46: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v46/HistoricalControlFlow.class");
const V47: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v47/HistoricalControlFlow.class");
const V48: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v48/HistoricalControlFlow.class");
const V52: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

/// The two clone nodes of the ECJ `finally`: the one subroutine at BCI 17 is entered by the `jsr`
/// at BCI 5 and by the `jsr` at BCI 12, and each call site gets its own copy.
const ECJ_CLONES: u64 = 2;

fn historical(version: u16) -> &'static [u8] {
    match version {
        45 => V45,
        46 => V46,
        47 => V47,
        48 => V48,
        52 => V52,
        _ => panic!("unsupported fixture version {version}"),
    }
}

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
        // Every dimension is named: the clone ceiling of the normalization is one of them, and a
        // fixture that inherited the rest silently would leave it at the default of zero.
    }
}

/// The same limits with one field replaced, for the stop cases below.
fn limits_with(change: impl FnOnce(&mut Limits)) -> Limits {
    let mut limits = limits();
    change(&mut limits);
    limits
}

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

struct Fixture {
    snapshot: ArtifactSnapshot,
    /// `HistoricalControlFlow.finallyPath(I)I`, or `illegal()V` of a synthetic class.
    method: PhysicalMethodId,
}

/// Opens one fixture and derives the physical identity the engine does.
fn fixture(content: &[u8]) -> Fixture {
    fixture_of_method(content, b"finallyPath", b"(I)I")
}

fn fixture_of_method(content: &[u8], name: &[u8], descriptor: &[u8]) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(content.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(content).to_hex().to_string()),
            length: u64::try_from(content.len()).expect("fixture length fits u64"),
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

/// One caller domain rooted at the fixture, and nothing else: the simplest environment the
/// validator accepts without a problem.
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

fn request(
    fixture: &Fixture,
    method: PhysicalMethodId,
    stages: Vec<AnalysisStage>,
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment: environment(fixture),
        method,
        stages,
    }
}

fn analyze(
    fixture: &Fixture,
    request: &MethodAnalysisRequest,
    limits: Limits,
) -> (MethodAnalysisReport, Budget) {
    let mut budget = Budget::new(limits);
    let report = Engine::new()
        .analyze_method(slice::from_ref(&fixture.snapshot), request, &mut budget)
        .expect("a legal request is answered, not raised");
    (report, budget)
}

/// The full pipeline up to the last phase this build implements, cloned for the fake phase that
/// stands behind `canonical_cfg` in the stage list: `frame` is still unimplemented, so a run that
/// reaches it reports `ir_pass_not_implemented` for that phase alone.
fn pipeline(fixture: &Fixture) -> MethodAnalysisRequest {
    request(fixture, fixture.method.clone(), vec![AnalysisStage::Ssa])
}

/// State of one stage of a report.
fn stage(report: &MethodAnalysisReport, stage: AnalysisStage) -> StageState {
    report
        .stages
        .iter()
        .find(|result| result.stage == stage)
        .unwrap_or_else(|| panic!("{stage:?} is scheduled"))
        .state
        .clone()
}

fn diagnostic_codes(report: &MethodAnalysisReport) -> Vec<&str> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

/// The stage states of a report in scheduled order.
fn stage_states(report: &MethodAnalysisReport) -> Vec<StageState> {
    report
        .stages
        .iter()
        .map(|stage| stage.state.clone())
        .collect()
}

/// The report as JSON with the wall clock removed: the one field two runs of the same request may
/// legitimately differ in.
fn without_elapsed(report: &MethodAnalysisReport) -> serde_json::Value {
    fn strip(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("elapsed_millis");
                for child in map.values_mut() {
                    strip(child);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    strip(item);
                }
            }
            _ => {}
        }
    }
    let mut value = serde_json::to_value(report).expect("a report serializes");
    strip(&mut value);
    value
}

#[test]
fn the_historical_finally_normalizes_with_one_clone_per_call_site() {
    // The 45–48 corpus compiles `finally` into one subroutine entered twice: by the `jsr` at BCI 5
    // on the normal path and by the `jsr` at BCI 12 on the handler path. The canonical phase
    // clones the subroutine once per call site — both charges are visible through the public
    // usage plane — completes, and makes the report's quality `Conservative`, because a canonical
    // artifact really was produced.
    for version in 45..=48 {
        let fixture = fixture(historical(version));
        let (report, budget) = analyze(&fixture, &pipeline(&fixture), limits());
        assert_eq!(
            stage_states(&report),
            vec![
                StageState::Completed,
                StageState::Completed,
                StageState::Completed,
                StageState::Completed,
                StageState::Failed {
                    code: "ir_pass_not_implemented".to_string()
                },
                StageState::NotPerformed,
            ],
            "classfile major {version}: the canonical phase completed and the frame phase is the \
             unimplemented one"
        );
        assert_eq!(
            diagnostic_codes(&report),
            vec!["ir_pass_not_implemented"],
            "classfile major {version}: nothing stopped before the unimplemented phase"
        );
        assert_eq!(
            report.quality,
            Quality::Conservative,
            "classfile major {version}: a canonical artifact was produced"
        );
        assert_eq!(
            budget.usage().normalization_clones,
            ECJ_CLONES,
            "classfile major {version}: one clone node per call site of the shared subroutine"
        );
        assert!(
            matches!(
                report.execution,
                ExecutionReport::Failed {
                    reason: TerminationReason::Unsupported { ref code },
                    ..
                } if code == "ir_pass_not_implemented"
            ),
            "classfile major {version}: the run terminates on the phase this build does not \
             implement, after the canonical one: {:?}",
            report.execution
        );

        // The canonicalization is real work on top of the raw graph: it rebuilds the blocks and
        // the edges and walks them, so it is strictly above a raw-only run on all three
        // dimensions it declares.
        let without = request(
            &fixture,
            fixture.method.clone(),
            vec![AnalysisStage::RawCfg],
        );
        let (_, raw_budget) = analyze(&fixture, &without, limits());
        assert!(
            budget.usage().ir_items > raw_budget.usage().ir_items,
            "classfile major {version}: the canonical blocks are derived items: {} vs {}",
            budget.usage().ir_items,
            raw_budget.usage().ir_items
        );
        assert!(
            budget.usage().ir_edges > raw_budget.usage().ir_edges,
            "classfile major {version}: the canonical edges are derived edges: {} vs {}",
            budget.usage().ir_edges,
            raw_budget.usage().ir_edges
        );
        assert!(
            budget.usage().analysis_steps > raw_budget.usage().analysis_steps,
            "classfile major {version}: the clone walk charges steps of its own: {} vs {}",
            budget.usage().analysis_steps,
            raw_budget.usage().analysis_steps
        );
        // The clone dimension is 3.5's alone: nothing before it charges a clone.
        let contexts_only = request(
            &fixture,
            fixture.method.clone(),
            vec![AnalysisStage::LegacyNormalization],
        );
        let (_, context_budget) = analyze(&fixture, &contexts_only, limits());
        assert_eq!(
            context_budget.usage().normalization_clones,
            0,
            "classfile major {version}: the call contexts are not clones"
        );
    }
}

#[test]
fn the_modern_dialect_has_no_clone_to_bill() {
    // The same source in 52 inlines the `finally`: the canonical graph of that body is built like
    // any other, and it bills no clone at all.
    let fixture = fixture(V52);
    let (report, budget) = analyze(&fixture, &pipeline(&fixture), limits());
    assert_eq!(
        stage_states(&report),
        vec![
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Failed {
                code: "ir_pass_not_implemented".to_string()
            },
            StageState::NotPerformed,
        ]
    );
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(budget.usage().normalization_clones, 0);
}

#[test]
fn an_exact_clone_budget_completes_and_one_less_falls_back() {
    // The clone bound is the bound of the artifact: at exactly the two clones the body needs the
    // phase completes; one clone less stops it, publishes no canonical fact and leaves the quality
    // plane at `Fallback`, under the normalization's own code.
    let fixture = fixture(V45);
    let (ample, ample_budget) = analyze(&fixture, &pipeline(&fixture), limits());
    let needed = ample_budget.usage().normalization_clones;
    assert_eq!(needed, ECJ_CLONES);
    assert_eq!(ample.quality, Quality::Conservative);

    let (exact, exact_budget) = analyze(
        &fixture,
        &pipeline(&fixture),
        limits_with(|limits| limits.normalization_clones = needed),
    );
    assert_eq!(
        stage(&exact, AnalysisStage::CanonicalCfg),
        StageState::Completed,
        "exactly the clones the body needs are enough"
    );
    assert_eq!(exact.quality, Quality::Conservative);
    assert_eq!(exact_budget.usage().normalization_clones, needed);

    let (short, short_budget) = analyze(
        &fixture,
        &pipeline(&fixture),
        limits_with(|limits| limits.normalization_clones = needed - 1),
    );
    assert_eq!(
        short_budget.usage().normalization_clones,
        needed - 1,
        "the charge is made before the clone exists, so the refused clone is not counted"
    );
    assert_eq!(
        stage(&short, AnalysisStage::LegacyNormalization),
        StageState::Completed,
        "the proven call contexts of 3.4b are kept"
    );
    assert_eq!(
        stage(&short, AnalysisStage::CanonicalCfg),
        StageState::Partial,
        "a stopped normalization is partial, not complete"
    );
    assert_eq!(
        stage(&short, AnalysisStage::Frame),
        StageState::NotPerformed,
        "no canonical fact was published, so the phase behind it never ran"
    );
    assert_eq!(short.quality, Quality::Fallback);
    assert_eq!(
        diagnostic_codes(&short),
        vec!["ir_legacy_normalization_unbounded"]
    );
    assert!(
        short.diagnostics[0].message.contains("NormalizationClones"),
        "the reason names the measure it stopped on: {}",
        short.diagnostics[0].message
    );
    assert!(matches!(
        short.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::Error { .. },
            ..
        }
    ));
}

#[test]
fn an_exhausted_step_budget_falls_back_with_the_normalization_code() {
    // The steps the two passes before it already spent are the budget the normalization finds
    // left: with none of them left the walk stops on its first step, and the stop is reported as
    // the normalization's own bound rather than as an unexplained partial phase.
    let fixture = fixture(V45);
    let contexts_only = request(
        &fixture,
        fixture.method.clone(),
        vec![AnalysisStage::LegacyNormalization],
    );
    let (_, spent) = analyze(&fixture, &contexts_only, limits());
    let spent_steps = spent.usage().analysis_steps;
    assert!(spent_steps > 0, "the raw graph and the walk charge steps");

    let (report, budget) = analyze(
        &fixture,
        &pipeline(&fixture),
        limits_with(|limits| limits.analysis_steps = spent_steps),
    );
    assert_eq!(
        stage(&report, AnalysisStage::CanonicalCfg),
        StageState::Partial
    );
    assert_eq!(
        stage(&report, AnalysisStage::Frame),
        StageState::NotPerformed
    );
    assert_eq!(report.quality, Quality::Fallback);
    assert_eq!(
        diagnostic_codes(&report),
        vec!["ir_legacy_normalization_unbounded"]
    );
    assert!(
        report.diagnostics[0].message.contains("AnalysisSteps"),
        "the reason names the measure it stopped on: {}",
        report.diagnostics[0].message
    );
    assert_eq!(
        budget.usage().analysis_steps,
        spent_steps,
        "the stop is the first charge the walk could not make"
    );
}

#[test]
fn a_cancelled_request_claims_no_canonical_artifact() {
    // Cancellation is not a bound of the normalization and must not be reported as one: the run
    // keeps its own termination, and the quality plane cannot claim a canonical artifact that was
    // never published.
    let fixture = fixture(V45);
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let report = Engine::new()
        .analyze_method(
            slice::from_ref(&fixture.snapshot),
            &pipeline(&fixture),
            &mut budget,
        )
        .expect("a cancelled request is answered, not raised");
    assert!(matches!(
        report.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert_eq!(report.quality, Quality::Fallback);
    assert_eq!(
        stage(&report, AnalysisStage::CanonicalCfg),
        StageState::NotPerformed,
        "a run that stopped before the phase has no canonical result"
    );
}

/// One synthetic body of the exception-path and nesting test: its name, its `Code`, and its
/// exception table in declaration order.
struct Body {
    name: &'static str,
    code: Vec<u8>,
    handlers: Vec<(u16, u16, u16, Option<u16>)>,
}

/// One constant-pool-and-method builder for the synthetic bodies below: the file-local shape
/// `tests/p2_return_address.rs` uses, with an exception table this slice needs.
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

    fn utf8(&mut self, value: &[u8]) -> u16 {
        let mut entry = vec![1];
        entry.extend_from_slice(
            &u16::try_from(value.len())
                .expect("fixture name fits u16")
                .to_be_bytes(),
        );
        entry.extend_from_slice(value);
        self.push(&entry)
    }

    fn class(&mut self, name: u16) -> u16 {
        let mut entry = vec![7];
        entry.extend_from_slice(&name.to_be_bytes());
        self.push(&entry)
    }
}

/// A structurally valid class file of the `jsr` era (major 50) with one `public static illegal()V`
/// whose `Code` is `code`, the exception table `handlers`, and nothing else.
///
/// `handlers` are `(start_bci, end_bci, handler_bci, catch_type_index)` records in declaration
/// order, so the ordinals the raw graph publishes are the order of this list.
fn jsr_class(code: &[u8], max_locals: u16, handlers: &[(u16, u16, u16, Option<u16>)]) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_name = pool.utf8(b"Test");
    let this_class = pool.class(this_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let code_name = pool.utf8(b"Code");
    let name_index = pool.utf8(b"illegal");
    let descriptor_index = pool.utf8(b"()V");

    let mut content = Vec::new();
    content.extend_from_slice(&1_u16.to_be_bytes()); // max_stack: a `jsr` pushes one word
    content.extend_from_slice(&max_locals.to_be_bytes());
    content.extend_from_slice(
        &u32::try_from(code.len())
            .expect("fixture code fits u32")
            .to_be_bytes(),
    );
    content.extend_from_slice(code);
    content.extend_from_slice(
        &u16::try_from(handlers.len())
            .expect("fixture handlers fit u16")
            .to_be_bytes(),
    );
    for (start, end, handler, catch_type) in handlers {
        content.extend_from_slice(&start.to_be_bytes());
        content.extend_from_slice(&end.to_be_bytes());
        content.extend_from_slice(&handler.to_be_bytes());
        content.extend_from_slice(&catch_type.unwrap_or(0).to_be_bytes());
    }
    content.extend_from_slice(&0_u16.to_be_bytes()); // Code attributes

    let mut method = Vec::new();
    method.extend_from_slice(&0x0009_u16.to_be_bytes()); // ACC_PUBLIC | ACC_STATIC
    method.extend_from_slice(&name_index.to_be_bytes());
    method.extend_from_slice(&descriptor_index.to_be_bytes());
    method.extend_from_slice(&1_u16.to_be_bytes()); // one attribute
    method.extend_from_slice(&code_name.to_be_bytes());
    method.extend_from_slice(
        &u32::try_from(content.len())
            .expect("fixture Code content fits u32")
            .to_be_bytes(),
    );
    method.extend_from_slice(&content);

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // minor
    bytes.extend_from_slice(&50_u16.to_be_bytes()); // major: the last dialect before 51
    bytes.extend_from_slice(&(pool.count + 1).to_be_bytes());
    bytes.extend_from_slice(&pool.bytes);
    bytes.extend_from_slice(&0x0021_u16.to_be_bytes()); // ACC_PUBLIC | ACC_SUPER
    bytes.extend_from_slice(&this_class.to_be_bytes());
    bytes.extend_from_slice(&object_class.to_be_bytes());
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // interfaces
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // fields
    bytes.extend_from_slice(&1_u16.to_be_bytes()); // methods
    bytes.extend_from_slice(&method);
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // class attributes
    bytes
}

/// `athrow` protected by record 0, whose handler holds the `jsr`; the subroutine is a three-block
/// chain of `astore_1`, `nop`, `goto` and `ret 1`.
fn exception_path_code() -> Vec<u8> {
    vec![
        0xbf, // 0: athrow
        0xb1, // 1: return
        0x4b, // 2: astore_0 (the handler entry)
        0xa8, 0x00, 0x04, // 3: jsr 7
        0xb1, // 6: return (the continuation of the call at BCI 3)
        0x4c, // 7: astore_1 (the subroutine entry)
        0x00, // 8: nop
        0xa7, 0x00, 0x03, // 9: goto 12
        0xa9, 0x01, // 12: ret 1
    ]
}

/// `jsr` at BCI 0 enters the routine at BCI 4, which calls the one at BCI 10.
fn nested_calls_code() -> Vec<u8> {
    vec![
        0xa8, 0x00, 0x04, // 0: jsr 4
        0xb1, // 3: return
        0x4b, // 4: astore_0 (the outer subroutine)
        0xa8, 0x00, 0x05, // 5: jsr 10 (the nested call)
        0xa9, 0x00, // 8: ret 0
        0x4c, // 10: astore_1 (the nested subroutine)
        0xa9, 0x01, // 11: ret 1
    ]
}

/// Two call sites (BCI 0 and BCI 3) enter the routine at BCI 7, which calls the one at BCI 13: the
/// nested routine has one clone per outer call path.
fn shared_nested_code() -> Vec<u8> {
    vec![
        0xa8, 0x00, 0x07, // 0: jsr 7
        0xa8, 0x00, 0x04, // 3: jsr 7 (the same subroutine)
        0xb1, // 6: return
        0x4b, // 7: astore_0 (the shared subroutine)
        0xa8, 0x00, 0x05, // 8: jsr 13 (the nested call)
        0xa9, 0x00, // 11: ret 0
        0x4c, // 13: astore_1 (the nested subroutine)
        0xa9, 0x01, // 14: ret 1
    ]
}

#[test]
fn a_jsr_on_an_exception_path_and_nested_calls_normalize() {
    // Real bytes through the public entry: a `jsr` inside a handler entered by the exception
    // table, a nested call inside a subroutine, and a nested call inside a *shared* subroutine.
    // Each body canonicalizes, bills at least one clone, and answers the same request with the
    // same report.
    let bodies = [
        Body {
            name: "a `jsr` on the exception path",
            code: exception_path_code(),
            handlers: vec![(0, 1, 2, None)],
        },
        Body {
            name: "nested calls",
            code: nested_calls_code(),
            handlers: Vec::new(),
        },
        Body {
            name: "a shared subroutine with a nested call",
            code: shared_nested_code(),
            handlers: Vec::new(),
        },
    ];
    for Body {
        name,
        code,
        handlers,
    } in bodies
    {
        let class = jsr_class(&code, 8, &handlers);
        let fixture = fixture_of_method(&class, b"illegal", b"()V");
        let request = request(&fixture, fixture.method.clone(), vec![AnalysisStage::Ssa]);
        let (report, budget) = analyze(&fixture, &request, limits());

        assert_eq!(
            stage_states(&report),
            vec![
                StageState::Completed,
                StageState::Completed,
                StageState::Completed,
                StageState::Completed,
                StageState::Failed {
                    code: "ir_pass_not_implemented".to_string()
                },
                StageState::NotPerformed,
            ],
            "{name}: the canonical phase really completed"
        );
        assert_eq!(report.quality, Quality::Conservative, "{name}");
        assert_eq!(
            diagnostic_codes(&report),
            vec!["ir_pass_not_implemented"],
            "{name}: neither the dialect nor the call graph was refused"
        );
        assert!(
            budget.usage().normalization_clones > 0,
            "{name}: the subroutine is cloned, not inlined: {:?}",
            budget.usage()
        );

        let (again, again_budget) = analyze(&fixture, &request, limits());
        assert_eq!(without_elapsed(&report), without_elapsed(&again), "{name}");
        for dimension in CountedBudgetDimension::ALL {
            assert_eq!(
                budget.usage().counted_usage(dimension),
                again_budget.usage().counted_usage(dimension),
                "{name}: {dimension:?} is charged deterministically"
            );
        }
    }
}

#[test]
fn the_p1_query_coordinates_are_unchanged_by_a_canonical_run() {
    let fixture = fixture(V45);
    let query = QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target: QueryTarget::Symbol {
            value: SymbolRef::Method {
                owner: bytes(b"java/lang/Object"),
                name: bytes(b"<init>"),
                descriptor: bytes(b"()V"),
            },
        },
        physical: PhysicalView {
            snapshot: fixture.snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
        max_items: 0,
        cursor: None,
    };
    let coordinates = |budget: &mut Budget| {
        let report = Engine::new()
            .query(&fixture.snapshot, &query, budget)
            .expect("the fixture query runs");
        // Every field a consumer reads off an item, not just the first one's coordinates: a
        // canonical run that leaked into the query plane could change the count, the use site's
        // kind or its derivation without moving the BCI.
        report
            .items
            .iter()
            .map(|item| {
                (
                    item.derivation,
                    item.certainty,
                    item.evidence.bci,
                    item.evidence.opcode,
                    item.evidence.constant_pool_index,
                )
            })
            .collect::<Vec<_>>()
    };

    // The one `invokespecial Object.<init>()V` of the fixture's constructor, at BCI 1: the P1
    // coordinates a canonicalization must not move.
    let mut before = Budget::new(limits());
    assert_eq!(
        coordinates(&mut before),
        vec![(
            XrefDerivation::StructuralConsumer,
            XrefCertainty::Exact,
            Some(1),
            Some(0xb7),
            Some(8)
        )]
    );

    let analysis = pipeline(&fixture);
    let (report, budget) = analyze(&fixture, &analysis, limits());
    assert_eq!(
        stage(&report, AnalysisStage::CanonicalCfg),
        StageState::Completed
    );
    assert_eq!(
        budget.usage().normalization_clones,
        ECJ_CLONES,
        "the run really did clone, so the comparison is against a run that normalized"
    );

    let mut after = Budget::new(limits());
    assert_eq!(
        coordinates(&mut after),
        coordinates(&mut Budget::new(limits())),
        "a canonical run leaves the P1 query result unchanged, item for item"
    );
    // What this can and cannot see, so the evidence is not read as more than it is: it pins that
    // the two planes stay independent - a query over the same snapshot returns the same thing
    // whether or not a canonical run happened in between. It cannot detect a query that read
    // the canonical graph, because the query plane takes the raw snapshot and never the graph;
    // what forbids that direction is the layering guard (`tests/p2_contracts.rs`), not this
    // test. The fixture also has no reference inside the analyzed method - `finallyPath` only
    // does arithmetic and `jsr` - so a hypothetical double count of a cloned call has nothing
    // here to double.
}
