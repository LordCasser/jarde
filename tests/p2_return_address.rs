//! P2 3.4 acceptance: the raw `returnAddress` facts and the call contexts through the public
//! entry point.
//!
//! The contexts themselves — one per `jsr` site, the return point of every `ret`, the affected
//! locals and the exception records that cross a call — are a crate-private payload
//! (invariant 11), so the values are pinned by the unit tests of `src/call_context.rs`, which
//! decode the committed ECJ fixtures and build synthetic bodies directly. What this file proves
//! through the public API is the wiring and the planes around that payload:
//!
//! 1. the historical `jsr`/`ret` `finally` of the ECJ 4.6 corpus really analyzes: the pass is the
//!    third phase this build implements, it completes, and it charges analysis steps of its own
//!    without adding an IR item or an edge;
//! 2. the modern dialect of the same source has no call context to build, and the pass charges
//!    nothing for it;
//! 3. a class file of the modern dialect that still holds a legacy opcode is a **violation**, not
//!    a limitation: the raw facts stay, no context is published, the run fails under
//!    `ir_legacy_opcode_forbidden`, and the canonical CFG phase never runs;
//! 4. a body whose decode stopped before its `ret` keeps the call graph unresolved: the reader's
//!    stop and the pass's own diagnostic are both reported, and the raw facts are the answer that
//!    survives;
//! 5. the same request published twice is the same report, field by field (the wall clock
//!    removed), which is the public side of "no published order is the walk's";
//! 6. A17 holds in behaviour, not only at source level: the P1 query coordinates of the `jsr`
//!    fixture are unchanged by a call-context run.

use jarde::*;
use std::slice;

/// The committed historical fixtures: one source compiled to every dialect of the `jsr` era,
/// plus the modern one that inlines the same `finally`.
const V45: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class");
const V46: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v46/HistoricalControlFlow.class");
const V47: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v47/HistoricalControlFlow.class");
const V48: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v48/HistoricalControlFlow.class");
const V52: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

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

/// The same bytes under another class-file version: the dialect of `jsr`/`ret` is decided by the
/// version bytes alone (minor 0, the modern rule).
fn under_version(bytes: &[u8], major: u16) -> Vec<u8> {
    let mut patched = bytes.to_vec();
    patched[4..6].copy_from_slice(&0_u16.to_be_bytes());
    patched[6..8].copy_from_slice(&major.to_be_bytes());
    patched
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
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
        ..Limits::default()
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
    /// `HistoricalControlFlow.finallyPath(I)I`
    method: PhysicalMethodId,
}

/// Opens one fixture and derives the physical identity the engine does.
fn fixture(content: &[u8]) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(content.to_vec()), &mut budget)
        .expect("the historical fixture opens as a standalone CLASS");
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
        owner: definition.clone(),
        name: bytes(b"finallyPath"),
        descriptor: bytes(b"(I)I"),
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

/// The report as JSON with the wall clock removed: the one field two runs of the same request
/// may legitimately differ in.
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

/// One usage snapshot with the wall clock removed: the comparison form of two reads of one budget.
///
/// `elapsed_millis` is a measurement, not a charge: [`Budget::usage`] takes it again on every
/// read, so the snapshot a report published and a later read of the same budget may legitimately
/// differ by a millisecond while every counted dimension is identical. P1 normalizes the same one
/// field the same way, and nothing else is dropped here.
fn counted_usage(usage: &UsageSnapshot) -> UsageSnapshot {
    UsageSnapshot {
        elapsed_millis: 0,
        ..usage.clone()
    }
}

/// One execution report compared with that one measurement removed from its usage.
fn without_wall_clock(execution: &ExecutionReport) -> ExecutionReport {
    match execution {
        ExecutionReport::Complete { usage } => ExecutionReport::Complete {
            usage: counted_usage(usage),
        },
        ExecutionReport::Partial { reason, usage } => ExecutionReport::Partial {
            reason: reason.clone(),
            usage: counted_usage(usage),
        },
        ExecutionReport::Cancelled { usage } => ExecutionReport::Cancelled {
            usage: counted_usage(usage),
        },
        ExecutionReport::Failed { reason, usage } => ExecutionReport::Failed {
            reason: reason.clone(),
            usage: counted_usage(usage),
        },
    }
}

#[test]
fn the_historical_jsr_finally_completes_the_call_context_pass() {
    // The 45–48 corpus really compiles `finally` into two `jsr` calls of one subroutine. The
    // pass that turns that into call contexts is the third phase this build implements: it
    // completes, it is not refused, and it charges analysis steps of its own — while the
    // dimensions `raw_cfg` owns stay exactly what that pass charged.
    for version in 45..=48 {
        let fixture = fixture(historical(version));
        let pipeline = request(&fixture, fixture.method.clone(), vec![AnalysisStage::Ssa]);
        let (report, budget) = analyze(&fixture, &pipeline, limits());
        assert_eq!(
            stage_states(&report),
            vec![
                StageState::Completed,
                StageState::Completed,
                StageState::Completed,
                StageState::Failed {
                    code: "ir_pass_not_implemented".to_string()
                },
                StageState::NotPerformed,
                StageState::NotPerformed,
            ],
            "classfile major {version}: the three implemented phases really ran"
        );
        assert_eq!(
            diagnostic_codes(&report),
            vec!["ir_pass_not_implemented"],
            "classfile major {version}: neither the dialect nor the call graph was refused"
        );
        assert_eq!(report.body, MethodBodyState::Present);
        assert_eq!(report.quality, Quality::Fallback);
        assert_eq!(
            report.coverage.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );

        // The same fixture with the pass unscheduled: the walk is what the difference in
        // analysis steps is, and it adds no IR item and no edge.
        let without = request(
            &fixture,
            fixture.method.clone(),
            vec![AnalysisStage::RawCfg],
        );
        let (_, raw_budget) = analyze(&fixture, &without, limits());
        assert!(
            budget.usage().analysis_steps > raw_budget.usage().analysis_steps,
            "classfile major {version}: the call-context walk charged steps of its own"
        );
        assert_eq!(budget.usage().ir_items, raw_budget.usage().ir_items);
        assert_eq!(budget.usage().ir_edges, raw_budget.usage().ir_edges);
        assert!(
            raw_budget.usage().ir_edges >= 2,
            "the two `jsr` calls of this fixture are raw edges"
        );
        assert_eq!(budget.usage().normalization_clones, 0, "cloning is 3.5");
        // The pass reads the body the reader already decoded: no second read, no second charge.
        assert_eq!(budget.usage().method_bodies, 1);
        assert_eq!(budget.usage().class_headers, 1);
    }
}

#[test]
fn the_modern_dialect_pays_nothing_for_its_empty_context_set() {
    // The same source in 52 inlines the `finally`: no `jsr`, no `ret`, and the empty context
    // set is the complete answer — the pass charges no step for it.
    let fixture = fixture(V52);
    let pipeline = request(&fixture, fixture.method.clone(), vec![AnalysisStage::Ssa]);
    let (report, budget) = analyze(&fixture, &pipeline, limits());
    assert_eq!(
        stage_states(&report),
        vec![
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Failed {
                code: "ir_pass_not_implemented".to_string()
            },
            StageState::NotPerformed,
            StageState::NotPerformed,
        ]
    );
    let without = request(
        &fixture,
        fixture.method.clone(),
        vec![AnalysisStage::RawCfg],
    );
    let (_, raw_budget) = analyze(&fixture, &without, limits());
    assert_eq!(
        budget.usage().analysis_steps,
        raw_budget.usage().analysis_steps,
        "a body without `jsr`/`ret` has no walk to charge"
    );
}

#[test]
fn a_legacy_opcode_in_the_modern_dialect_is_a_violation_not_a_limitation() {
    // The very bytes of the 45 fixture under version 51: the raw facts are kept exactly as they
    // are, no call context is published, the run fails under its own code with an `Error`, and
    // the canonical CFG phase never runs.
    let fixture = fixture(&under_version(V45, 51));
    let request = request(&fixture, fixture.method.clone(), vec![AnalysisStage::Ssa]);
    let (report, budget) = analyze(&fixture, &request, limits());

    assert_eq!(
        stage(&report, AnalysisStage::RawFacts),
        StageState::Completed,
        "the class header and the body were read"
    );
    assert_eq!(
        stage(&report, AnalysisStage::RawCfg),
        StageState::Completed,
        "the raw facts are kept, not deleted"
    );
    assert_eq!(
        stage(&report, AnalysisStage::LegacyNormalization),
        StageState::Failed {
            code: "ir_legacy_opcode_forbidden".to_string()
        }
    );
    assert_eq!(
        stage(&report, AnalysisStage::CanonicalCfg),
        StageState::NotPerformed,
        "nothing canonical is built from forbidden opcodes"
    );
    assert_eq!(
        without_wall_clock(&report.execution),
        ExecutionReport::Failed {
            reason: TerminationReason::Error {
                code: "ir_legacy_opcode_forbidden".to_string(),
            },
            usage: counted_usage(&budget.usage()),
        }
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["ir_legacy_opcode_forbidden"]
    );
    assert_eq!(
        report.diagnostics[0].severity,
        DiagnosticSeverity::Error,
        "a dialect violation is an error, not a warning"
    );
    assert_eq!(report.body, MethodBodyState::Present);
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema,
        "the bytecode plane still describes the body that was read"
    );
    assert_eq!(report.quality, Quality::Fallback);
    assert!(
        budget.usage().ir_edges >= 2,
        "the raw graph was really built"
    );
    assert_eq!(
        budget.usage().normalization_clones,
        0,
        "the forbidden method never reaches a cloning pass"
    );
}

#[test]
fn a_body_whose_decode_stopped_keeps_its_call_graph_unresolved() {
    // 22 of the fixture's 23 code bytes fit the budget, so the decode stops at the `ret` of the
    // shared subroutine: the call contexts cannot be established over the prefix, and nothing
    // is faked. Both reasons are reported — the reader's own stop and the pass's code — and the
    // raw facts of the prefix survive in the coverage plane.
    let fixture = fixture(V45);
    let request = request(&fixture, fixture.method.clone(), vec![AnalysisStage::Ssa]);
    let (report, budget) = analyze(
        &fixture,
        &request,
        limits_with(|limits| limits.code_bytes = 22),
    );

    assert_eq!(
        stage_states(&report),
        vec![
            StageState::Partial,
            StageState::Partial,
            StageState::Partial,
            StageState::NotPerformed,
            StageState::NotPerformed,
            StageState::NotPerformed,
        ],
        "the phases behind the unresolved call graph never ran"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec![
            "classfile_bytecode_budget_exceeded",
            "ir_call_context_unresolved"
        ]
    );
    assert_eq!(
        report.diagnostics[1].severity,
        DiagnosticSeverity::Warning,
        "a call graph that cannot be established is a limitation, not damage"
    );
    assert_eq!(
        without_wall_clock(&report.execution),
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::CodeBytes,
            },
            usage: counted_usage(&budget.usage()),
        },
        "the first stop governs the run: the reader's own dimension"
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    let skipped: Vec<(u64, u64)> = report
        .coverage
        .artifact_structural
        .skipped
        .iter()
        .map(|range| (range.start, range.end))
        .collect();
    assert!(
        skipped.contains(&(21, 23)),
        "the unread suffix is named: {skipped:?}"
    );
    assert_eq!(report.body, MethodBodyState::Present);
    assert!(
        budget.usage().ir_items > 0,
        "the prefix was really analyzed"
    );
}

#[test]
fn the_call_context_walk_stops_on_its_own_step_budget() {
    // The steps `raw_cfg` needs for this body, read from the run that stops there: one more
    // step lets the pipeline start the walk and be refused inside it, so the stop belongs to
    // the call-context pass and not to the graph before it.
    let fixture = fixture(V45);
    let without = request(
        &fixture,
        fixture.method.clone(),
        vec![AnalysisStage::RawCfg],
    );
    let (raw_report, raw_budget) = analyze(&fixture, &without, limits());
    assert_eq!(
        stage(&raw_report, AnalysisStage::RawCfg),
        StageState::Completed
    );
    let steps_before = raw_budget.usage().analysis_steps;

    let request = request(&fixture, fixture.method.clone(), vec![AnalysisStage::Ssa]);
    let (report, budget) = analyze(
        &fixture,
        &request,
        limits_with(|limits| limits.analysis_steps = steps_before + 1),
    );
    assert_eq!(
        stage_states(&report),
        vec![
            StageState::Completed,
            StageState::Completed,
            StageState::Partial,
            StageState::NotPerformed,
            StageState::NotPerformed,
            StageState::NotPerformed,
        ],
        "the graph completed, the walk was refused"
    );
    assert_eq!(
        without_wall_clock(&report.execution),
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
            },
            usage: counted_usage(&budget.usage()),
        }
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["budget_exceeded_analysis_steps"],
        "the stop names the dimension the pass declared"
    );
    assert!(budget.usage().analysis_steps <= steps_before + 1);
    assert_eq!(
        budget.usage().ir_edges,
        raw_budget.usage().ir_edges,
        "the walk adds no edge of its own"
    );
}

#[test]
fn the_same_call_context_request_publishes_the_same_report() {
    let fixture = fixture(V45);
    let request = request(&fixture, fixture.method.clone(), vec![AnalysisStage::Ssa]);
    let (first, first_budget) = analyze(&fixture, &request, limits());
    let (second, second_budget) = analyze(&fixture, &request, limits());

    assert_eq!(
        without_elapsed(&first),
        without_elapsed(&second),
        "two runs of one request are the same report, including every published order"
    );
    for dimension in CountedBudgetDimension::ALL {
        assert_eq!(
            first_budget.usage().counted_usage(dimension),
            second_budget.usage().counted_usage(dimension),
            "{dimension:?} is charged deterministically"
        );
    }
}

#[test]
fn the_p1_query_coordinates_are_unchanged_by_a_call_context_run() {
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
        assert_eq!(report.items.len(), 1);
        let item = &report.items[0];
        (
            item.evidence.bci,
            item.evidence.opcode,
            item.evidence.constant_pool_index,
        )
    };

    // The one `invokespecial Object.<init>()V` of the fixture's constructor, at BCI 1 of
    // `<init>`: the P1 coordinates the analysis path must not move.
    let mut before = Budget::new(limits());
    assert_eq!(coordinates(&mut before), (Some(1), Some(0xb7), Some(8)));

    let analysis = request(&fixture, fixture.method.clone(), vec![AnalysisStage::Ssa]);
    let (report, _) = analyze(&fixture, &analysis, limits());
    assert_eq!(
        stage(&report, AnalysisStage::LegacyNormalization),
        StageState::Completed
    );

    let mut after = Budget::new(limits());
    assert_eq!(
        coordinates(&mut after),
        (Some(1), Some(0xb7), Some(8)),
        "a call-context run leaves the P1 query coordinates and their count unchanged"
    );
}
