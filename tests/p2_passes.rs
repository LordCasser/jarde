//! P2 3.2 acceptance: the startup validation of a method-analysis request's stage set.
//!
//! The pass table itself is crate-private (its phases, facts, dependencies, invalidation and
//! its own structural faults are pinned by the unit tests of `crates/jarde-jvm/src/passes.rs`),
//! so what this file has to prove through the public API is the wiring around it:
//!
//! 1. the validation accepts every request a caller can shape — all 63 non-empty stage sets,
//!    including the unordered and the repeated ones — and each one is answered with a report
//!    whose stages are the prefix of the phase order up to the last requested phase. A schedule
//!    the validator refuses would raise an input error instead of producing this report, so the
//!    report is the evidence that the validator accepted this schedule — the table it validates
//!    against is crate-private, so how the table and the phases agree is pinned by the unit
//!    tests of `crates/jarde-jvm/src/passes.rs`, not here. Since 3.3 the run is real, so the
//!    same 63 sets also show what the passes did: with a budget that refuses the very first
//!    charge the first pass is `Partial` and nothing behind it ran, and with a funded budget
//!    the two implemented phases are `Completed` while the first unimplemented one is `Failed`;
//! 2. the schedule is read from the request's phases alone: the reported `stages` is the prefix
//!    of the phase order up to the last requested phase, once each, in phase order — which is
//!    the prefix the table schedules, because the table has one pass per phase (unit test);
//! 3. the request checks of 1.1 keep their precedence and their codes: an empty stage set is
//!    still `analysis_no_stages`, and a snapshot the content does not provide is still
//!    `resolution_snapshot_mismatch` *before* the stage set is looked at. A rejected request
//!    publishes nothing: no report, no stage list, no charged dimension.

use jarde::*;
use std::slice;

/// The committed historical fixture: one class with a real body, so a request that is answered
/// is a request about a method that exists.
const HISTORICAL: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

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
        nested_depth: 8,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

/// Every counted limit is zero, so a dimension the request charges would fail the request
/// instead of producing a report: a request the engine refuses publishes no report at all, and
/// this is what says so.
fn zero_limits() -> Limits {
    Limits {
        input_bytes: 0,
        archive_entries: 0,
        entry_bytes: 0,
        read_bytes: 0,
        class_bytes: 0,
        attribute_bytes: 0,
        code_bytes: 0,
        result_items: 0,
        output_bytes: 0,
        nested_depth: 0,
        elapsed_millis: 0,
        ..Limits::default()
    }
}

/// The same fail-closed limits with the wall clock funded, so the dimension a *work* charge
/// refused is the deterministic one.
fn zero_work_limits() -> Limits {
    Limits {
        elapsed_millis: u64::MAX,
        ..zero_limits()
    }
}

/// The funded limits plus the dimensions a method-analysis run charges (3.3).
fn analysis_limits() -> Limits {
    Limits {
        class_headers: 10,
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        ..limits()
    }
}

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

fn counted_usage_is_zero(usage: &UsageSnapshot) -> bool {
    CountedBudgetDimension::ALL
        .iter()
        .all(|dimension| usage.counted_usage(*dimension) == 0)
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

fn invalid_input_code(error: &Error) -> Option<&str> {
    match error {
        Error::InvalidInput { code, .. } => Some(code.as_str()),
        _ => None,
    }
}

struct Fixture {
    snapshot: ArtifactSnapshot,
    /// `HistoricalControlFlow.finallyPath(I)I`
    method: PhysicalMethodId,
}

/// Opens the historical fixture and derives the physical identity the way the engine does.
///
/// Opening and hashing are physical work and pay their own budget; the analysis requests below
/// use fresh budgets, so their counted usage starts at zero.
fn fixture() -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(HISTORICAL.to_vec()), &mut budget)
        .expect("the historical fixture opens as a standalone CLASS");
    assert_eq!(snapshot.kind(), ArtifactKind::StandaloneClass);
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(HISTORICAL).to_hex().to_string()),
            length: u64::try_from(HISTORICAL.len()).expect("fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: bytes(b"finallyPath"),
        descriptor: bytes(b"(I)I"),
    };
    Fixture { snapshot, method }
}

/// One caller domain rooted at the fixture, and nothing else: the simplest environment the
/// validator accepts without a problem, so a legal request here is legal for its own reasons.
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

fn analysis_request(
    fixture: &Fixture,
    environment: ResolutionEnvironment,
    stages: Vec<AnalysisStage>,
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment,
        method: fixture.method.clone(),
        stages,
    }
}

/// Index of a stage in the fixed phase order, from the public list itself.
fn stage_index(stage: AnalysisStage) -> usize {
    AnalysisStage::ALL
        .iter()
        .position(|candidate| *candidate == stage)
        .expect("every stage is in the phase order")
}

/// The prefix of the phase order a request for `stages` schedules: every phase up to the last
/// requested one, once each, in order.
fn scheduled_prefix(stages: &[AnalysisStage]) -> Vec<AnalysisStage> {
    let last = stages
        .iter()
        .copied()
        .map(stage_index)
        .max()
        .expect("a non-empty stage set has a last phase");
    AnalysisStage::ALL[..=last].to_vec()
}

#[test]
fn every_stage_set_is_accepted_and_answered_with_the_state_of_its_passes() {
    let fixture = fixture();
    let environment = environment(&fixture);

    // Every non-empty stage set a caller can write, including the singletons and the whole
    // pipeline: the startup validation must not refuse any of them, and the answer must be the
    // report of the passes that really ran. `raw_facts` is always scheduled, and a budget that
    // refuses its first charge stops it there: the first pass is `Partial`, nothing behind it
    // ran, and not one counted dimension was charged.
    for mask in 1u32..(1u32 << AnalysisStage::ALL.len()) {
        let stages: Vec<AnalysisStage> = AnalysisStage::ALL
            .into_iter()
            .enumerate()
            .filter(|(index, _)| mask & (1 << index) != 0)
            .map(|(_, stage)| stage)
            .collect();
        let request = analysis_request(&fixture, environment.clone(), stages.clone());
        let mut budget = Budget::new(zero_work_limits());
        let report = Engine::new()
            .analyze_method(slice::from_ref(&fixture.snapshot), &request, &mut budget)
            .unwrap_or_else(|error| panic!("{stages:?} is a legal request: {error}"));

        assert!(
            report.environment_problems.is_empty(),
            "{stages:?}: the fixture environment is valid"
        );
        assert_eq!(
            report.requested_stages,
            AnalysisStage::ALL
                .into_iter()
                .filter(|stage| stages.contains(stage))
                .collect::<Vec<_>>(),
            "{stages:?} is normalized to the phase order"
        );
        let expected = scheduled_prefix(&stages);
        assert_eq!(
            report
                .stages
                .iter()
                .map(|scheduled| scheduled.stage)
                .collect::<Vec<_>>(),
            expected,
            "{stages:?} schedules its own phase prefix"
        );
        assert_eq!(
            report.stages[0].state,
            StageState::Partial,
            "{stages:?}: the refused first pass is partial, not performed"
        );
        assert!(
            report.stages[1..]
                .iter()
                .all(|scheduled| scheduled.state == StageState::NotPerformed),
            "{stages:?}: a phase behind the stop never ran"
        );
        assert_eq!(
            report.body,
            MethodBodyState::NotInspected,
            "{stages:?}: the stop preceded the read, so no body fact is stated"
        );
        assert!(report.reads.is_empty());
        assert_eq!(report.coverage, Coverage::not_requested());
        assert_eq!(
            without_wall_clock(&report.execution),
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::ClassHeaders,
                },
                usage: counted_usage(&budget.usage()),
            },
            "{stages:?}: the pass that stopped names the dimension it was refused"
        );
        assert!(
            counted_usage_is_zero(&budget.usage()),
            "{stages:?}: a refused charge charges nothing"
        );
    }

    // The last phase schedules the whole pipeline, so the phase prefix cannot be a truncated
    // part of the order without this failing. With the run funded, the four phases this build
    // implements complete, the first one it does not implement fails, and the phases behind
    // that failure stay `NotPerformed` instead of looking performed.
    let request = analysis_request(&fixture, environment, vec![AnalysisStage::Ssa]);
    let mut budget = Budget::new(analysis_limits());
    let report = Engine::new()
        .analyze_method(slice::from_ref(&fixture.snapshot), &request, &mut budget)
        .expect("the whole pipeline schedules");
    assert_eq!(report.stages.len(), AnalysisStage::ALL.len());
    assert_eq!(
        report
            .stages
            .iter()
            .map(|stage| stage.state.clone())
            .collect::<Vec<_>>(),
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
    assert_eq!(
        report.body,
        MethodBodyState::Present,
        "the funded run really read the driver method's body"
    );
}

#[test]
fn the_schedule_is_the_phase_prefix_whatever_order_the_request_names() {
    let fixture = fixture();
    let environment = environment(&fixture);
    let expected = scheduled_prefix(&[AnalysisStage::Frame, AnalysisStage::Ssa]);
    assert_eq!(
        expected,
        AnalysisStage::ALL.to_vec(),
        "this slice's example is the whole pipeline"
    );

    // Order and repetition in the request are the report's business (1.1 normalization) and
    // never the schedule's: the three requests below ask for the same phases.
    for stages in [
        vec![AnalysisStage::Ssa, AnalysisStage::Frame, AnalysisStage::Ssa],
        vec![AnalysisStage::Frame, AnalysisStage::Ssa],
        vec![AnalysisStage::Ssa, AnalysisStage::Ssa, AnalysisStage::Frame],
    ] {
        let request = analysis_request(&fixture, environment.clone(), stages.clone());
        let report = Engine::new()
            .analyze_method(
                slice::from_ref(&fixture.snapshot),
                &request,
                &mut Budget::new(zero_limits()),
            )
            .expect("a legal request is answered");
        assert_eq!(
            report.requested_stages,
            vec![AnalysisStage::Frame, AnalysisStage::Ssa],
            "{stages:?} asks for the same phases"
        );
        assert_eq!(
            report
                .stages
                .iter()
                .map(|scheduled| scheduled.stage)
                .collect::<Vec<_>>(),
            expected,
            "{stages:?} schedules the same prefix"
        );
    }
}

#[test]
fn an_empty_stage_set_is_rejected_and_publishes_nothing() {
    let fixture = fixture();
    let request = analysis_request(&fixture, environment(&fixture), Vec::new());
    let mut budget = Budget::new(zero_limits());
    let error = Engine::new()
        .analyze_method(slice::from_ref(&fixture.snapshot), &request, &mut budget)
        .expect_err("a request without a stage asks for nothing");
    assert_eq!(invalid_input_code(&error), Some("analysis_no_stages"));
    assert!(
        counted_usage_is_zero(&budget.usage()),
        "a rejected request charges no dimension"
    );
}

#[test]
fn the_request_shape_checks_keep_their_precedence_over_the_pass_table() {
    let fixture = fixture();
    // The content provides no snapshot *and* the stage set is empty: the request shapes are
    // checked first and the pass validation is appended after them, so the answer is the
    // snapshot mismatch of 1.1 — the startup validation of 3.2 does not take over the codes of
    // the checks it follows, and a request rejected there publishes no stage list at all.
    let request = analysis_request(&fixture, environment(&fixture), Vec::new());
    let error = Engine::new()
        .analyze_method(&[], &request, &mut Budget::new(zero_limits()))
        .expect_err("the snapshot is not provided by the content");
    assert_eq!(
        invalid_input_code(&error),
        Some("resolution_snapshot_mismatch")
    );

    // Same content, legal stages: still the snapshot mismatch, not a schedule error.
    let request = analysis_request(
        &fixture,
        environment(&fixture),
        vec![AnalysisStage::RawFacts],
    );
    let error = Engine::new()
        .analyze_method(&[], &request, &mut Budget::new(zero_limits()))
        .expect_err("the snapshot is still not provided");
    assert_eq!(
        invalid_input_code(&error),
        Some("resolution_snapshot_mismatch")
    );
}
