//! P2 3.4 acceptance: the raw `returnAddress` facts and the call contexts through the public
//! entry point.
//!
//! The contexts themselves — one per `jsr` site, the return point of every `ret`, the affected
//! locals and the exception records that cross a call — are a crate-private payload
//! (invariant 11), so the values are pinned by the unit tests of
//! `crates/jarde-jvm/src/call_context.rs`, which decode the committed ECJ fixtures and build
//! synthetic bodies directly. What this file proves through the public API is the wiring and
//! the planes around that payload:
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

/// The same bytes under a version whose **minor** the caller names: for a major of 56 or above the
/// format allows `0` and `65535` only, so any other minor contradicts the version it is written
/// under (JVMS 4.1).
fn under_minor(bytes: &[u8], minor: u16) -> Vec<u8> {
    let mut patched = bytes.to_vec();
    patched[4..6].copy_from_slice(&minor.to_be_bytes());
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
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
        // Every dimension of `Limits` is named here on purpose: the canonicalization of 3.5 bills
        // a dimension of its own, and a fixture that inherited the rest silently would leave it
        // at the default of zero.
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
    // dimensions `raw_cfg` owns stay exactly what that pass charged. 3.5's `canonical_cfg` then
    // consumes exactly those contexts and completes as well, and 4.3 names the clones and the
    // continuations of that graph in the last two phases.
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
                StageState::Completed,
                StageState::Completed,
                StageState::Completed,
            ],
            "classfile major {version}: every phase this build implements really ran"
        );
        assert!(
            diagnostic_codes(&report).is_empty(),
            "classfile major {version}: neither the dialect nor the call graph was refused: {:?}",
            diagnostic_codes(&report)
        );
        assert_eq!(report.body, MethodBodyState::Present);
        // The canonical CFG is the artifact of this pipeline, so a run that published one is
        // `Conservative`; the plane says nothing about the phases behind it.
        assert_eq!(report.quality, Quality::Conservative);
        assert_eq!(
            report.coverage.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );

        // The same fixture with the pass unscheduled: the walk is what the difference in
        // analysis steps is, and it derives storage of its own — the plans, the per-context sets
        // and the rows of its visited matrix as `IrItems`, the successor lists as `IrEdges` — so
        // the call-context run is strictly above the raw-graph-only run on both dimensions.
        let without = request(
            &fixture,
            fixture.method.clone(),
            vec![AnalysisStage::RawCfg],
        );
        let (_, raw_budget) = analyze(&fixture, &without, limits());
        // A run that stops at the contexts isolates this pass's own charges from the
        // canonicalization 3.5 runs behind it.
        let contexts_only = request(
            &fixture,
            fixture.method.clone(),
            vec![AnalysisStage::LegacyNormalization],
        );
        let (_, context_budget) = analyze(&fixture, &contexts_only, limits());
        assert!(
            context_budget.usage().analysis_steps > raw_budget.usage().analysis_steps,
            "classfile major {version}: the call-context walk charged steps of its own"
        );
        assert!(
            context_budget.usage().ir_items > raw_budget.usage().ir_items,
            "classfile major {version}: the contexts and their sets are derived items: {} vs {}",
            context_budget.usage().ir_items,
            raw_budget.usage().ir_items
        );
        assert!(
            context_budget.usage().ir_edges > raw_budget.usage().ir_edges,
            "classfile major {version}: the successor lists are derived edges: {} vs {}",
            context_budget.usage().ir_edges,
            raw_budget.usage().ir_edges
        );
        assert!(
            raw_budget.usage().ir_edges >= 2,
            "the two `jsr` calls of this fixture are raw edges"
        );
        assert_eq!(
            context_budget.usage().normalization_clones,
            0,
            "the call contexts are not clones"
        );
        // One clone per call site of the one shared subroutine: the charge 3.5 exists for.
        assert_eq!(
            budget.usage().normalization_clones,
            2,
            "classfile major {version}: the shared subroutine is cloned per call site"
        );
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
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
        ]
    );
    let without = request(
        &fixture,
        fixture.method.clone(),
        vec![AnalysisStage::RawCfg],
    );
    let (_, raw_budget) = analyze(&fixture, &without, limits());
    // The same run without the canonicalization behind it: this is the walk's own charge, and a
    // body whose context set is empty has no walk to pay for.
    let contexts_only = request(
        &fixture,
        fixture.method.clone(),
        vec![AnalysisStage::LegacyNormalization],
    );
    let (_, context_budget) = analyze(&fixture, &contexts_only, limits());
    assert_eq!(
        context_budget.usage().analysis_steps,
        raw_budget.usage().analysis_steps,
        "a body without `jsr`/`ret` has no walk to charge"
    );
    assert_eq!(
        budget.usage().normalization_clones,
        0,
        "and nothing to clone"
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

/// A class file whose **version the format does not allow** is refused by the phase that reads the
/// version at all, and the refusal keeps what the phases before it produced.
///
/// The two arms of the reader's own version rule, on the very bytes of the 45 fixture:
///
/// * major **44** — below the minimum the format defines, so these bytes are not a class file of
///   any version, whatever their body says;
/// * major **56** with minor **1** — a modern version whose minor is neither `0` nor `65535`, the
///   two the format allows for 56 and above.
///
/// Both are the *version's* own verdict and not a fact about the body: the decode of the body is
/// complete and its coverage stays published, and the reader's header plan states the same code
/// for the same bytes. What this case pins is that the analysis path states it too — until now it
/// walked all six phases with no diagnostic at all over a version the format has no class file
/// for, which is what `specs/jvm-ir/spec.md`, "Legacy normalization before canonical frames",
/// states as "非法版本…时保留原始 Bytecode 与原因" — and that it is refused **before** the dialect
/// rules of the body are applied: the 45 fixture's `jsr`/`ret` is a legal legacy opcode, and at
/// major 56 the same opcode is a forbidden modern one, so a run that reported the opcode would
/// have decided a dialect for a version that does not exist.
///
/// The refusal has the same shape as the modern-dialect violation above — the raw facts and the raw
/// graph stay published, nothing canonical is built over them, the run fails with an `Error` under
/// the reader's own code — and the two planes that answer different questions, whether the body was
/// read and whether it verifies, stay untouched.
#[test]
fn a_version_the_format_does_not_allow_is_refused_where_the_dialect_is_decided() {
    for (class, code) in [
        (under_version(V45, 44), "classfile_invalid_major_version"),
        (
            under_minor(&under_version(V45, 56), 1),
            "classfile_invalid_modern_minor_version",
        ),
    ] {
        let fixture = fixture(&class);
        let request = request(&fixture, fixture.method.clone(), vec![AnalysisStage::Ssa]);
        let (report, budget) = analyze(&fixture, &request, limits());

        assert_eq!(
            stage(&report, AnalysisStage::RawFacts),
            StageState::Completed,
            "{code}: the class header and the body were read"
        );
        assert_eq!(
            stage(&report, AnalysisStage::RawCfg),
            StageState::Completed,
            "{code}: the raw facts and the raw graph are kept, not deleted"
        );
        assert_eq!(
            stage(&report, AnalysisStage::LegacyNormalization),
            StageState::Failed {
                code: code.to_string()
            },
            "{code}: the phase that reads the version reports it"
        );
        assert_eq!(
            stage(&report, AnalysisStage::CanonicalCfg),
            StageState::NotPerformed,
            "{code}: nothing canonical is built over a version the format does not allow"
        );
        assert_eq!(
            without_wall_clock(&report.execution),
            ExecutionReport::Failed {
                reason: TerminationReason::Error {
                    code: code.to_string(),
                },
                usage: counted_usage(&budget.usage()),
            },
            "{code}: the run fails under the reader's own code"
        );
        assert_eq!(diagnostic_codes(&report), vec![code]);
        assert_eq!(
            report.diagnostics[0].severity,
            DiagnosticSeverity::Error,
            "{code}: an illegal version is an error, not a warning"
        );
        assert_eq!(report.body, MethodBodyState::Present);
        assert_eq!(
            report.coverage.artifact_structural.state,
            CoverageState::CompleteWithinSchema,
            "{code}: the bytecode plane still describes the body that was read"
        );
        assert_eq!(
            report.quality,
            Quality::Fallback,
            "{code}: no canonical artifact was produced"
        );
        assert_eq!(report.representation, Representation::Bytecode);
        assert_eq!(report.syntax_status, SyntaxStatus::NotJava);
        assert_eq!(report.compile_status, CompileStatus::NotAttempted);
        assert_eq!(
            report.verification,
            VerificationStatus::NotPerformed,
            "{code}: refusing an illegal version is not verifying the method"
        );
        assert_eq!(report.semantic_validation, SemanticValidation::Unproven);
        assert!(
            report.reads.len() == 1,
            "{code}: the one read the request performed is still stated"
        );
    }
}

/// The contrast that keeps the refusal above from being read as "an old version is refused": the
/// same bytes at a version the format allows keep analyzing — the 45 dialect normalizes its
/// `jsr`/`ret` and the whole pipeline completes — so what the case above refuses is the version's
/// own rule and not the age of the class file.
#[test]
fn a_version_the_format_allows_still_analyzes_however_old_it_is() {
    for class in [
        under_version(V45, 45),
        under_minor(&under_version(V45, 56), u16::MAX),
    ] {
        let fixture = fixture(&class);
        let request = request(&fixture, fixture.method.clone(), vec![AnalysisStage::Ssa]);
        let (report, _) = analyze(&fixture, &request, limits());
        assert_eq!(
            stage(&report, AnalysisStage::RawFacts),
            StageState::Completed,
            "a lawful version is read like any other"
        );
        assert!(
            !report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code.starts_with("classfile_invalid")),
            "no version rule is violated here: {:?}",
            diagnostic_codes(&report)
        );
    }
}

/// A body whose decode stopped before its `ret` keeps its call graph unresolved, and both reasons
/// are reported: the reader's own stop and the pass's own code.
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
    assert!(
        budget.usage().ir_edges > raw_budget.usage().ir_edges,
        "the walk's successor lists are derived edges, billed before the step budget stopped the \
         run: {} vs {}",
        budget.usage().ir_edges,
        raw_budget.usage().ir_edges
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

/// One constant-pool-and-method builder for the synthetic D-1 fixture below: the file-local
/// shape `tests/p2_cfg.rs` uses, with the `max_locals` this body needs.
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

/// A structurally valid class file of the `jsr` era (major 50, where `jsr`/`ret` are still
/// legal) with one `public static illegal()V` whose `Code` is `code` and nothing else.
fn illegal_call_graph_class(code: &[u8], max_locals: u16) -> Vec<u8> {
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
    content.extend_from_slice(&0_u16.to_be_bytes()); // exception table
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

#[test]
fn a_live_call_site_behind_a_mid_block_jsr_is_still_refused() {
    // D-1 (0.4), on the public path: the `jsr` at BCI 3 is **not** the first instruction of its
    // block — BCIs 0..3 are ordinary straight-line code — so the call at BCI 3 is a transfer out
    // of block `[0, 6)`, and its continuation is the block at BCI 6. That continuation holds the
    // second call site, whose subroutine body owns no `ret`, so this body is illegal and must be
    // refused:
    //
    //   0  iconst_1        6  jsr +10 -> 16   (return address 9; body without a `ret`)
    //   1  istore_1        9  ireturn
    //   2  nop            10  astore_1        (the body of the call at BCI 3)
    //   3  jsr +7 -> 10   11  ret 1
    //                     13  nop, 14 nop, 15 nop
    //                     16  astore_2        (the body of the call at BCI 6)
    //                     17  return
    //
    // Losing the continuation relation reads the live call site at BCI 6 as dead, skips the
    // refusal, and answers `Established` for a method whose return address is never used: the
    // `LegacyNormalization` phase must stay `Partial` under its own code instead.
    let code = [
        0x04, // 0: iconst_1
        0x3c, // 1: istore_1
        0x00, // 2: nop
        0xa8, 0x00, 0x07, // 3: jsr +7 -> 10 (return address 6)
        0xa8, 0x00, 0x0a, // 6: jsr +10 -> 16 (return address 9)
        0xac, // 9: ireturn
        0x4c, // 10: astore_1
        0xa9, 0x01, // 11: ret 1
        0x00, // 13: nop
        0x00, // 14: nop
        0x00, // 15: nop
        0x4d, // 16: astore_2
        0xb1, // 17: return
    ];
    let fixture = fixture(&illegal_call_graph_class(&code, 3));
    let method = PhysicalMethodId {
        owner: fixture.method.owner.clone(),
        name: bytes(b"illegal"),
        descriptor: bytes(b"()V"),
    };
    let request = request(&fixture, method, vec![AnalysisStage::Ssa]);
    let (report, budget) = analyze(&fixture, &request, limits());

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
        "the raw facts survive, the call graph is refused, and nothing canonical is built"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["ir_call_context_unresolved"]
    );
    assert_eq!(
        report.diagnostics[0].severity,
        DiagnosticSeverity::Warning,
        "an unestablished call graph is a limitation, not damage"
    );
    assert_eq!(
        without_wall_clock(&report.execution),
        ExecutionReport::Partial {
            reason: TerminationReason::Error {
                code: "ir_call_context_unresolved".to_string(),
            },
            usage: counted_usage(&budget.usage()),
        }
    );
    assert_eq!(report.body, MethodBodyState::Present);
    assert!(
        budget.usage().ir_edges >= 2,
        "both calls are raw edges, so the graph really was built: {}",
        budget.usage().ir_edges
    );
}

/// The `LegacyNormalization` stage's outcome for one `illegal()V` body, through the public entry.
///
/// Returns the stage state with the diagnostic codes, so a caller can assert both that the stage
/// did not complete and that it said why.
fn legacy_stage(code: &[u8]) -> (StageState, Vec<String>) {
    let fixture = fixture(&illegal_call_graph_class(code, 3));
    let method = PhysicalMethodId {
        owner: fixture.method.owner.clone(),
        name: bytes(b"illegal"),
        descriptor: bytes(b"()V"),
    };
    let request = request(&fixture, method, vec![AnalysisStage::LegacyNormalization]);
    let (report, _budget) = analyze(&fixture, &request, limits());
    let states = stage_states(&report);
    let state = states
        .get(2)
        .expect("the request asks for the legacy phase, so its state is present")
        .clone();
    let codes = diagnostic_codes(&report)
        .into_iter()
        .map(str::to_string)
        .collect();
    (state, codes)
}

#[test]
fn a_return_address_the_bytes_do_not_prove_stops_the_stage_before_it_completes() {
    // R7 (3.4b), on the public path. The call-context stage used to accept a slot because of
    // where the writes were, not because of what they stored, so a `ret` whose slot held an
    // ordinary value still let the stage complete - and canonicalization would then have
    // consumed a return point the bytes never established.
    //
    // Four bodies whose slot holds something that is not the return address, and two that are
    // legal, each given as the method's whole `Code`:
    let cases: [(&str, &[u8], bool); 6] = [
        // 0: jsr+4; 3: return; 4: astore_0 (the address); 5: ret 0
        (
            "a stored address",
            &[0xa8, 0x00, 0x04, 0xb1, 0x4b, 0xa9, 0x00],
            true,
        ),
        // 4: aconst_null; 5: astore_1; 6: astore_0; 7: ret 1 - local 1 holds null
        (
            "a null in the slot",
            &[0xa8, 0x00, 0x04, 0xb1, 0x01, 0x4c, 0x4b, 0xa9, 0x01],
            false,
        ),
        // 4: pop (the address is gone); 5: aconst_null; 6: astore_0; 7: ret 0
        (
            "a discarded address",
            &[0xa8, 0x00, 0x04, 0xb1, 0x57, 0x01, 0x4b, 0xa9, 0x00],
            false,
        ),
        // 4: astore_0; 5: aload_0 (which may not load a return address); 6: astore_1; 7: ret 1
        (
            "an address carried through aload",
            &[0xa8, 0x00, 0x04, 0xb1, 0x4b, 0x2a, 0x4c, 0xa9, 0x01],
            false,
        ),
        // Two nested calls: the outer stores slot 0, the inner stores slot 1, `ret 0` returns
        // to the outer continuation and `ret 1` to the inner one.
        (
            "a legal nested pair",
            &[
                0xa8, 0x00, 0x04, 0xb1, 0x4b, 0xa8, 0x00, 0x05, 0xa9, 0x00, 0x4c, 0x00, 0x00, 0xa9,
                0x01,
            ],
            true,
        ),
        // The same shape, but after the inner call stores its own slot it writes null into the
        // slot the outer `ret 0` reads - the two calls share one frame.
        (
            "an inner call writing the outer slot",
            &[
                0xa8, 0x00, 0x04, 0xb1, 0x4b, 0xa8, 0x00, 0x05, 0xa9, 0x00, 0x4c, 0x01, 0x4b, 0xa9,
                0x01,
            ],
            false,
        ),
    ];
    for (name, code, legal) in cases {
        let (state, codes) = legacy_stage(code);
        if legal {
            assert_eq!(
                state,
                StageState::Completed,
                "{name} is a legal body and must still complete: {codes:?}"
            );
        } else {
            assert_eq!(
                state,
                StageState::Partial,
                "{name} must stop the stage rather than complete it: {codes:?}"
            );
            assert_eq!(
                codes,
                vec!["ir_call_context_unresolved".to_string()],
                "{name} stops under its own code"
            );
        }
    }
}
