//! P2 3.3 acceptance: the raw CFG pass through the public entry point.
//!
//! The graph itself is crate-private (blocks, edges, instruction-level throw sites, handler
//! order, effect facts and the unreachable truth table are pinned by the unit tests of
//! `crates/jarde-jvm/src/cfg.rs`, which build the reader facts directly), so what this file has
//! to prove through the public API is the wiring around it, on real compiled bodies:
//!
//! 1. a request that schedules `RawCfg` really analyzes the driver method: the class definition
//!    is read (one `ClassHeaders` attempt, recorded under `DriverMethodBody`), the body is
//!    located and decoded (one `MethodBodies` attempt plus its `CodeBytes`), the two phases this
//!    build implements complete, the body fact becomes `Present` and the BCI coverage plane is
//!    complete;
//! 2. a body whose decode stopped early — by budget here — is *partial*, not complete: the pass
//!    that ran over the decoded prefix says `Partial`, the coverage plane names the skipped BCI
//!    range, and the execution keeps the reader's own dimension. A stop inside the raw-CFG pass
//!    (its `IrItems` storage items) keeps the phases that already completed;
//! 3. the counted dimensions 3.3 introduced are real and stay within the declared set: one
//!    header read, one body attempt, the decoded bytes, the IR items and the analysis steps —
//!    and nothing else;
//! 4. the same request published twice is the same report, field by field (the wall clock
//!    removed), which is the public side of "no published order is petgraph's";
//! 5. A17 holds in behaviour, not only at source level: the P1 query coordinates of the same
//!    fixture are unchanged after a raw-CFG run;
//! 6. the driver method's own definition is **bound** to the loader the request declares (the 0.1
//!    contract): a definition that loader's order does not select stops the pass under
//!    `resolution_definition_unbound` with its physical read still published, while a definition
//!    the loader really provides performs the whole pipeline — the binding is the loader's own
//!    decision, never snapshot equality;
//! 7. 5.1's failure isolation, on classes that are not one finished method: two members of one
//!    class are answered about themselves — a stop in one report is not a state of the other, and
//!    neither request's body attempt reaches the other member — while a class whose **header**
//!    cannot be decoded answers no member at all instead of fabricating a method result.
//!
//! Sections 1–6 are the 3.2/3.3/3.4 acceptance of the raw-CFG slice; section 7 is the 5.1
//! contract over the same entry point.

use jarde::*;
use std::slice;

/// The committed historical fixtures: the same source compiled to two dialects, one with an
/// inline `finally` (52) and one with a `jsr`/`ret` subroutine (45).
const V52: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");
const V45: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class");

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
    definition: PhysicalDefinitionId,
    /// `HistoricalControlFlow.finallyPath(I)I`
    method: PhysicalMethodId,
}

/// Opens one fixture and derives the physical identity the way the engine does.
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
    Fixture {
        snapshot,
        definition,
        method,
    }
}

/// One caller domain rooted at the fixture, and nothing else: the simplest environment the
/// validator accepts without a problem.
fn environment(fixture: &Fixture) -> ResolutionEnvironment {
    environment_with(
        fixture,
        vec![LoadRoot::Snapshot {
            snapshot: fixture.snapshot.id().clone(),
        }],
    )
}

/// The same environment with an explicit root list: the ordered positions of the `app` loader,
/// which decide what this loader binds.
fn environment_with(fixture: &Fixture, roots: Vec<LoadRoot>) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots,
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

/// The counted BCI range a coverage plane scanned or skipped, as `(start, end)`.
fn bci_ranges(ranges: &[CoverageRange]) -> Vec<(u64, u64)> {
    ranges
        .iter()
        .filter(|range| range.label == "method_code_bci")
        .map(|range| (range.start, range.end))
        .collect()
}

#[test]
fn a_complete_body_completes_the_two_implemented_phases() {
    let fixture = fixture(V52);
    let request = request(
        &fixture,
        fixture.method.clone(),
        vec![AnalysisStage::RawCfg],
    );
    let (report, budget) = analyze(&fixture, &request, limits());

    assert!(report.environment_problems.is_empty());
    assert_eq!(
        report
            .stages
            .iter()
            .map(|result| result.stage)
            .collect::<Vec<_>>(),
        vec![AnalysisStage::RawFacts, AnalysisStage::RawCfg],
        "`RawCfg` schedules its prerequisite and nothing later"
    );
    assert_eq!(
        stage(&report, AnalysisStage::RawFacts),
        StageState::Completed
    );
    assert_eq!(stage(&report, AnalysisStage::RawCfg), StageState::Completed);
    assert_eq!(
        without_wall_clock(&report.execution),
        ExecutionReport::Complete {
            usage: counted_usage(&budget.usage()),
        }
    );
    assert!(diagnostic_codes(&report).is_empty());
    assert_eq!(report.body, MethodBodyState::Present);
    assert_eq!(
        report.reads,
        vec![HeaderRead {
            loader: LoaderId("app".to_string()),
            definition: fixture.definition.clone(),
            reason: ReadReason::DriverMethodBody,
        }],
        "one header read, for the driver method's own body"
    );
    // The whole body of `finallyPath(I)I` is 15 bytes of straight-line code (11 instructions),
    // so the BCI plane covers it completely and the coverage names exactly that range.
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert_eq!(
        bci_ranges(&report.coverage.artifact_structural.scanned),
        vec![(0, 15)]
    );
    assert!(report.coverage.artifact_structural.skipped.is_empty());
    assert_eq!(
        report.coverage.runtime_resolution,
        CoverageDimension::not_requested()
    );
    assert_eq!(
        report.coverage.dynamic_analysis,
        CoverageDimension::not_requested()
    );

    // The counted dimensions of this slice, and nothing else: one header read attempt, one body
    // attempt, the decoded instruction bytes, and the raw pass's own items and steps. The body
    // is straight-line code with one handler no instruction of its protected range can enter,
    // so the graph really holds no edge — the next test shows a body that has transfers.
    let usage = budget.usage();
    assert_eq!(usage.class_headers, 1);
    assert_eq!(usage.method_bodies, 1);
    assert_eq!(usage.code_bytes, 15);
    assert!(
        usage.ir_items >= 11,
        "at least one effect fact per decoded instruction: {}",
        usage.ir_items
    );
    assert!(usage.analysis_steps >= 11);
    assert_eq!(usage.ir_edges, 0);
    assert_eq!(
        usage.normalization_clones, 0,
        "cloning is 3.5's dimension, not this pass's"
    );
}

#[test]
fn a_jsr_subroutine_body_still_has_raw_transfers() {
    // The same source compiled with a `jsr`/`ret` subroutine, whose protected range covers a
    // `jsr` (A09's shape): the raw pass builds the graph, so it charges edges, and the run is
    // complete — 3.4/3.5 are what resolve the return addresses, not this pass refusing the body.
    let fixture = fixture(V45);
    let request = request(
        &fixture,
        fixture.method.clone(),
        vec![AnalysisStage::RawCfg],
    );
    let (report, budget) = analyze(&fixture, &request, limits());

    assert_eq!(
        stage(&report, AnalysisStage::RawFacts),
        StageState::Completed
    );
    assert_eq!(stage(&report, AnalysisStage::RawCfg), StageState::Completed);
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(report.body, MethodBodyState::Present);
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    // 23 bytes of code: two `jsr` calls into one shared subroutine plus the straight-line
    // paths that reach them.
    assert_eq!(budget.usage().code_bytes, 23);
    assert!(
        budget.usage().ir_edges >= 2,
        "the two calls are raw edges: {}",
        budget.usage().ir_edges
    );
    assert!(budget.usage().ir_items > 0);
    assert!(budget.usage().analysis_steps > 0);
}

#[test]
fn a_truncated_body_analyzes_its_reliable_prefix() {
    // `add(II)I` has four one-byte instructions and no exception table, so a two-byte
    // `CodeBytes` budget stops the decode after two of them and every validated target of the
    // decoded prefix is still an instruction start.
    let fixture = fixture(V52);
    let method = PhysicalMethodId {
        owner: fixture.definition.clone(),
        name: bytes(b"add"),
        descriptor: bytes(b"(II)I"),
    };
    let request = request(&fixture, method, vec![AnalysisStage::RawCfg]);
    let (report, budget) = analyze(
        &fixture,
        &request,
        limits_with(|limits| limits.code_bytes = 2),
    );

    assert_eq!(
        stage(&report, AnalysisStage::RawFacts),
        StageState::Partial,
        "the facts cover a prefix of the body"
    );
    assert_eq!(
        stage(&report, AnalysisStage::RawCfg),
        StageState::Partial,
        "the graph covers the same prefix, so it is not a complete graph"
    );
    assert_eq!(report.body, MethodBodyState::Present);
    assert_eq!(
        without_wall_clock(&report.execution),
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::CodeBytes,
            },
            usage: counted_usage(&budget.usage()),
        },
        "the reader's own dimension explains the stop"
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    assert_eq!(
        bci_ranges(&report.coverage.artifact_structural.scanned),
        vec![(0, 2)]
    );
    assert_eq!(
        bci_ranges(&report.coverage.artifact_structural.skipped),
        vec![(2, 4)],
        "the unread suffix is named, not dropped"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["classfile_bytecode_budget_exceeded"]
    );
    assert!(
        budget.usage().ir_items > 0,
        "the decoded prefix was really analyzed"
    );
}

#[test]
fn a_truncated_body_whose_targets_cannot_be_validated_is_not_read_as_corrupt() {
    // The same stop on a body with an exception table: the reader validates the protected
    // ranges against the instruction starts it decoded, and the record ending at BCI 4 of this
    // body no longer lines up with the prefix. That is 1.2's sound-but-incomplete view — the
    // unread suffix may hold the instruction start — so the raw pass reports a `Partial` stage
    // under its own code instead of calling the method damaged or claiming a complete graph.
    let fixture = fixture(V52);
    let request = request(
        &fixture,
        fixture.method.clone(),
        vec![AnalysisStage::RawCfg],
    );
    let (report, budget) = analyze(
        &fixture,
        &request,
        limits_with(|limits| limits.code_bytes = 5),
    );

    assert_eq!(stage(&report, AnalysisStage::RawFacts), StageState::Partial);
    assert_eq!(
        stage(&report, AnalysisStage::RawCfg),
        StageState::Partial,
        "a truncated body is incomplete, never a structured failure"
    );
    assert_eq!(report.body, MethodBodyState::Present);
    assert_eq!(
        without_wall_clock(&report.execution),
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::CodeBytes,
            },
            usage: counted_usage(&budget.usage()),
        }
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec![
            "classfile_bytecode_budget_exceeded",
            "ir_raw_cfg_incomplete_body"
        ]
    );
    assert_eq!(
        bci_ranges(&report.coverage.artifact_structural.scanned),
        vec![(0, 4)]
    );
    assert_eq!(
        bci_ranges(&report.coverage.artifact_structural.skipped),
        vec![(4, 15)]
    );
    assert_eq!(
        budget.usage().ir_items,
        0,
        "no graph was built, so no item of it was charged"
    );
}

#[test]
fn an_ir_budget_stop_keeps_the_completed_prefix() {
    let fixture = fixture(V52);
    let request = request(
        &fixture,
        fixture.method.clone(),
        vec![AnalysisStage::RawCfg],
    );
    // The reader is funded; the raw pass is not: its third IR item is refused.
    let (report, budget) = analyze(
        &fixture,
        &request,
        limits_with(|limits| limits.ir_items = 3),
    );

    assert_eq!(
        stage(&report, AnalysisStage::RawFacts),
        StageState::Completed,
        "the pass before the stop completed"
    );
    assert_eq!(stage(&report, AnalysisStage::RawCfg), StageState::Partial);
    assert_eq!(
        without_wall_clock(&report.execution),
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::IrItems,
            },
            usage: counted_usage(&budget.usage()),
        }
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["budget_exceeded_ir_items"],
        "the stop names the dimension the pass declared"
    );
    assert_eq!(budget.usage().ir_items, 3, "the charges stop at the limit");
    assert_eq!(budget.usage().ir_edges, 0, "no edge was constructed");
    // The body was read before the IR stop, so the body fact stays what it is.
    assert_eq!(report.body, MethodBodyState::Present);
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema,
        "the bytecode plane describes the read, not the IR pass"
    );
}

#[test]
fn the_same_request_publishes_the_same_report() {
    let fixture = fixture(V52);
    let request = request(
        &fixture,
        fixture.method.clone(),
        vec![AnalysisStage::RawCfg],
    );
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
fn the_p1_query_coordinates_are_unchanged_by_a_raw_cfg_run() {
    let fixture = fixture(V52);
    let target = SymbolRef::Method {
        owner: bytes(b"java/lang/Object"),
        name: bytes(b"<init>"),
        descriptor: bytes(b"()V"),
    };
    let query = QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target: QueryTarget::Symbol {
            value: target.clone(),
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

    // The A02 coordinates of the same fixture: the one `invokespecial Object.<init>()V` at BCI 1
    // of `<init>`, on constant-pool index 8 (P1 golden is the full-field evidence; this is the
    // contrast assertion that an analysis run does not move it).
    let mut before = Budget::new(limits());
    assert_eq!(coordinates(&mut before), (Some(1), Some(0xb7), Some(8)));

    let analysis = request(
        &fixture,
        fixture.method.clone(),
        vec![AnalysisStage::RawCfg],
    );
    let (report, _) = analyze(&fixture, &analysis, limits());
    assert_eq!(stage(&report, AnalysisStage::RawCfg), StageState::Completed);

    let mut after = Budget::new(limits());
    assert_eq!(
        coordinates(&mut after),
        (Some(1), Some(0xb7), Some(8)),
        "a raw-CFG run leaves the P1 query coordinates and their count unchanged"
    );
}

/// One constant-pool-and-method builder for the synthetic fixtures below.
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

/// One method of a synthetic class: access flags, raw name, raw descriptor, and the `Code`
/// bytes it declares (no `Code` attribute at all when absent).
type SyntheticMethod<'a> = (u16, &'a [u8], &'a [u8], Option<Vec<u8>>);

/// A structurally valid class file (`major`) with the given methods; a method without code has
/// no `Code` attribute at all.
fn class_bytes(this_class: &[u8], major: u16, methods: &[SyntheticMethod<'_>]) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_name = pool.utf8(this_class);
    let this_class_index = pool.class(this_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let code_name = pool.utf8(b"Code");
    let mut entries = Vec::new();
    for (access, name, descriptor, code) in methods {
        let name_index = pool.utf8(name);
        let descriptor_index = pool.utf8(descriptor);
        let mut method = Vec::new();
        method.extend_from_slice(&access.to_be_bytes());
        method.extend_from_slice(&name_index.to_be_bytes());
        method.extend_from_slice(&descriptor_index.to_be_bytes());
        match code {
            Some(code) => {
                let mut content = Vec::new();
                content.extend_from_slice(&1_u16.to_be_bytes()); // max_stack
                content.extend_from_slice(&1_u16.to_be_bytes()); // max_locals
                content.extend_from_slice(
                    &u32::try_from(code.len())
                        .expect("fixture code fits u32")
                        .to_be_bytes(),
                );
                content.extend_from_slice(code);
                content.extend_from_slice(&0_u16.to_be_bytes()); // exception table
                content.extend_from_slice(&0_u16.to_be_bytes()); // attributes
                method.extend_from_slice(&1_u16.to_be_bytes()); // attributes
                method.extend_from_slice(&code_name.to_be_bytes());
                method.extend_from_slice(
                    &u32::try_from(content.len())
                        .expect("fixture code content fits u32")
                        .to_be_bytes(),
                );
                method.extend_from_slice(&content);
            }
            None => method.extend_from_slice(&0_u16.to_be_bytes()),
        }
        entries.push(method);
    }

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    bytes.extend_from_slice(&0_u16.to_be_bytes());
    bytes.extend_from_slice(&major.to_be_bytes());
    bytes.extend_from_slice(&(pool.count + 1).to_be_bytes());
    bytes.extend_from_slice(&pool.bytes);
    bytes.extend_from_slice(&0x0021_u16.to_be_bytes()); // ACC_PUBLIC | ACC_SUPER
    bytes.extend_from_slice(&this_class_index.to_be_bytes());
    bytes.extend_from_slice(&object_class.to_be_bytes());
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // interfaces
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // fields
    bytes.extend_from_slice(
        &u16::try_from(entries.len())
            .expect("fixture method count fits u16")
            .to_be_bytes(),
    );
    for method in entries {
        bytes.extend_from_slice(&method);
    }
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // attributes
    bytes
}

#[test]
fn a_member_that_declares_no_body_is_a_fact_and_not_a_failed_pass() {
    // `ACC_ABSTRACT` (0x0400) and `ACC_NATIVE` (0x0100), each with no `Code` attribute: the
    // declaration says there is no body, so no pass can run and the request is complete as far as
    // its input allows. The two kinds are separate facts of the declaration, not one "no body"
    // bucket, so both are checked here — 5.1's acceptance names the pair.
    for (access, kind) in [
        (0x0401_u16, NoBodyKind::Abstract),
        (0x0101, NoBodyKind::Native),
    ] {
        let content = class_bytes(b"p/Shape", 52, &[(access, b"area", b"()I", None)]);
        let fixture = fixture(&content);
        let method = PhysicalMethodId {
            owner: fixture.definition.clone(),
            name: bytes(b"area"),
            descriptor: bytes(b"()I"),
        };
        let request = request(&fixture, method, vec![AnalysisStage::RawCfg]);
        let (report, budget) = analyze(&fixture, &request, limits());

        assert_eq!(
            report.body,
            MethodBodyState::DeclaredWithoutBody { no_body_kind: kind },
            "{kind:?} is the fact the declaration itself states"
        );
        assert_eq!(
            report
                .stages
                .iter()
                .map(|result| result.state.clone())
                .collect::<Vec<_>>(),
            vec![StageState::NotPerformed, StageState::NotPerformed],
            "no phase ran: there is nothing to analyze"
        );
        assert_eq!(
            without_wall_clock(&report.execution),
            ExecutionReport::Complete {
                usage: counted_usage(&budget.usage()),
            },
            "a declaration without a body is not a failure"
        );
        assert_eq!(
            report.representation,
            Representation::Bytecode,
            "the representation plane is P2's, and a member without a body does not move it"
        );
        assert_eq!(
            diagnostic_codes(&report),
            vec!["ir_method_declared_without_body"]
        );
        assert_eq!(report.diagnostics[0].severity, DiagnosticSeverity::Info);
        assert!(
            report.diagnostics[0].message.contains(&format!("{kind:?}")),
            "the diagnostic explains why no phase ran and names the kind the flags state: {}",
            report.diagnostics[0].message
        );
        assert_eq!(
            report.coverage,
            Coverage::not_requested(),
            "there is no body, so no BCI range was covered"
        );
        assert_eq!(budget.usage().class_headers, 1);
        assert_eq!(
            budget.usage().method_bodies,
            0,
            "a member without a body is never attempted"
        );
        assert_eq!(budget.usage().ir_items, 0);
    }
}

#[test]
fn a_foreign_method_identity_fails_the_body_pass_under_its_own_code() {
    let fixture = fixture(V52);
    let absent = PhysicalMethodId {
        owner: fixture.definition.clone(),
        name: bytes(b"missing"),
        descriptor: bytes(b"()V"),
    };
    let request = request(&fixture, absent, vec![AnalysisStage::RawCfg]);
    let (report, budget) = analyze(&fixture, &request, limits());

    assert_eq!(
        stage(&report, AnalysisStage::RawFacts),
        StageState::Failed {
            code: "classfile_method_not_found".to_string()
        }
    );
    assert_eq!(
        stage(&report, AnalysisStage::RawCfg),
        StageState::NotPerformed
    );
    assert_eq!(
        without_wall_clock(&report.execution),
        ExecutionReport::Failed {
            reason: TerminationReason::Error {
                code: "classfile_method_not_found".to_string()
            },
            usage: counted_usage(&budget.usage()),
        }
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["classfile_method_not_found"]
    );
    assert_eq!(report.body, MethodBodyState::NotInspected);
    assert_eq!(budget.usage().class_headers, 1);
    assert_eq!(budget.usage().method_bodies, 0);
}

// ---------------------------------------------------------------------------
// 0.1/D25: the driver's class definition is bound to the loader that claims it
// ---------------------------------------------------------------------------

/// A driver definition the declared loader's own order does not select stops the body pass.
///
/// The request claims the definition its own content holds, and the declared `app` loader has two
/// ordered roots whose first position holds a *different* class of the same name. The claim is
/// readable and describes the bytes at its own coordinate; it simply is not a class this loader
/// would ever load, and the 0.1 contract refuses to stamp it with the caller's loader and build
/// runtime semantics on it. The refusal is the whole run's: the body fact stays `NotInspected`,
/// no pass behind `raw_facts` ran, and the two reads that decided it stay published — the physical
/// facts are not withdrawn with the semantics that would have been built on them.
///
/// The control at the end is the same definition under a loader whose order really selects it:
/// one header read, one binding, the whole pipeline performed.
#[test]
fn a_driver_definition_the_declared_loader_does_not_bind_stops_the_body_pass() {
    let driver = fixture(V52);
    let shadow = fixture(&class_bytes(
        b"HistoricalControlFlow",
        52,
        &[(0x0001, b"other", b"()V", Some(vec![0xb1]))],
    ));
    let environment = environment_with(
        &driver,
        vec![
            LoadRoot::Snapshot {
                snapshot: shadow.snapshot.id().clone(),
            },
            LoadRoot::Snapshot {
                snapshot: driver.snapshot.id().clone(),
            },
        ],
    );
    let shadowed_request = MethodAnalysisRequest {
        environment,
        method: driver.method.clone(),
        stages: vec![AnalysisStage::RawCfg],
    };
    let mut budget = Budget::new(limits());
    let report = Engine::new()
        .analyze_method(
            &[driver.snapshot.clone(), shadow.snapshot.clone()],
            &shadowed_request,
            &mut budget,
        )
        .expect("a legal request is answered, not raised");

    assert_eq!(
        stage(&report, AnalysisStage::RawFacts),
        StageState::Failed {
            code: "resolution_definition_unbound".to_string()
        },
        "the definition is not bound to the loader that claims it"
    );
    assert_eq!(
        stage(&report, AnalysisStage::RawCfg),
        StageState::NotPerformed,
        "the phases behind a stopped pass never ran"
    );
    assert_eq!(
        without_wall_clock(&report.execution),
        ExecutionReport::Failed {
            reason: TerminationReason::Error {
                code: "resolution_definition_unbound".to_string()
            },
            usage: counted_usage(&budget.usage()),
        }
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_definition_unbound"],
        "one code, shared with the identity-read refusal it is"
    );
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "resolution_definition_unbound")
        .expect("the stop is published");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert!(
        diagnostic.message.contains("`app`"),
        "the diagnostic names the loader whose order decides: {}",
        diagnostic.message
    );
    assert!(
        diagnostic.message.contains(&shadow.snapshot.id().0),
        "and the definition that order really selects: {}",
        diagnostic.message
    );
    assert_eq!(
        report.body,
        MethodBodyState::NotInspected,
        "no body was located, so none is claimed"
    );
    // The physical facts survive the refusal: the definition's own read is recorded next to the
    // read of the shadowing position that decided against it.
    assert_eq!(
        report.reads,
        vec![
            HeaderRead {
                loader: LoaderId("app".to_string()),
                definition: driver.definition.clone(),
                reason: ReadReason::DriverMethodBody,
            },
            HeaderRead {
                loader: LoaderId("app".to_string()),
                definition: shadow.definition.clone(),
                reason: ReadReason::DriverMethodBody,
            },
        ]
    );
    assert!(
        u64::try_from(report.reads.len()).expect("fixture read count fits u64")
            <= budget.usage().class_headers,
        "reads ({}) must not exceed the charged header attempts ({})",
        report.reads.len(),
        budget.usage().class_headers
    );
    assert_eq!(
        budget.usage().class_headers,
        2,
        "the claimed definition and the position that refuted it were both read"
    );
    assert_eq!(
        budget.usage().method_bodies,
        0,
        "the driver method's body was never even attempted"
    );

    // Control: the same definition, the same request, under a loader whose only position holds
    // it. Nothing about the refusal was about the bytes — this run performs the whole pipeline
    // with the one header read the binding check is answered from.
    let (bound, bound_budget) = analyze(
        &driver,
        &request(&driver, driver.method.clone(), vec![AnalysisStage::RawCfg]),
        limits(),
    );
    assert_eq!(
        stage(&bound, AnalysisStage::RawFacts),
        StageState::Completed
    );
    assert_eq!(stage(&bound, AnalysisStage::RawCfg), StageState::Completed);
    assert_eq!(bound.body, MethodBodyState::Present);
    assert_eq!(bound.reads.len(), 1);
    assert_eq!(bound_budget.usage().class_headers, 1);
    assert_eq!(bound_budget.usage().method_bodies, 1);
}

// ---------------------------------------------------------------------------
// 5.1: one class, several members — every report answers for its own member
// ---------------------------------------------------------------------------

/// The identity of one member of the fixture class, derived the way the request names it.
fn member(fixture: &Fixture, name: &[u8], descriptor: &[u8]) -> PhysicalMethodId {
    PhysicalMethodId {
        owner: fixture.definition.clone(),
        name: bytes(name),
        descriptor: bytes(descriptor),
    }
}

#[test]
fn two_members_of_one_class_are_answered_about_themselves() {
    // One class whose members do not share a fate. `ok()V` is a one-instruction body that runs the
    // whole pipeline; `stop()V` consumes a `new` where only an initialized reference is meaningful
    // and therefore stops the frame phase — the boundary 4.2 draws, not a claim about the bytes.
    // Each request is answered about its own member, and the stop is not a state of the class: a
    // request for `ok` made *after* the stop answers exactly what it answered before it.
    let content = class_bytes(
        b"p/Mixed",
        52,
        &[
            (0x0009, b"ok", b"()V", Some(vec![0xb1])),
            (
                0x0009,
                b"stop",
                b"()V",
                // 0: new p/Mixed (#2)  3: ifnull +3 -> 6  6: return
                Some(vec![0xbb, 0x00, 0x02, 0xc6, 0x00, 0x03, 0xb1]),
            ),
        ],
    );
    let fixture = fixture(&content);
    let ok = member(&fixture, b"ok", b"()V");
    let stop = member(&fixture, b"stop", b"()V");

    let (first_ok, first_ok_budget) = analyze(
        &fixture,
        &request(&fixture, ok.clone(), vec![AnalysisStage::Ssa]),
        limits(),
    );
    assert_eq!(first_ok.method, ok);
    assert_eq!(stage(&first_ok, AnalysisStage::Ssa), StageState::Completed);
    assert_eq!(
        without_wall_clock(&first_ok.execution),
        ExecutionReport::Complete {
            usage: counted_usage(&first_ok_budget.usage()),
        }
    );
    assert_eq!(first_ok.body, MethodBodyState::Present);
    assert_eq!(first_ok.quality, Quality::Conservative);
    assert_eq!(
        first_ok.semantic_validation,
        SemanticValidation::LocalInvariants,
        "the member that ran to the end carries its own evidence"
    );
    assert!(diagnostic_codes(&first_ok).is_empty());
    assert_eq!(
        first_ok_budget.usage().method_bodies,
        1,
        "one body attempt: the requested member's, not the class's"
    );

    let (stopped, stopped_budget) = analyze(
        &fixture,
        &request(&fixture, stop.clone(), vec![AnalysisStage::Ssa]),
        limits(),
    );
    assert_eq!(stopped.method, stop);
    assert_eq!(
        stage(&stopped, AnalysisStage::Frame),
        StageState::Partial,
        "the boundary is inside the frame phase"
    );
    assert_eq!(
        stage(&stopped, AnalysisStage::Ssa),
        StageState::NotPerformed
    );
    assert!(matches!(
        &stopped.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::Error { code },
            ..
        } if code == "ir_frame_deferred"
    ));
    assert_eq!(diagnostic_codes(&stopped), vec!["ir_frame_deferred"]);
    assert_eq!(
        stopped.body,
        MethodBodyState::Present,
        "the body was located and read: the stop is what the phase made of it"
    );
    assert_eq!(
        stopped.semantic_validation,
        SemanticValidation::Unproven,
        "the phase that checks the local invariants never ran in this request"
    );
    assert_eq!(
        stopped_budget.usage().method_bodies,
        1,
        "the stopping member's own body attempt, and only that one"
    );

    // The comparison that isolates the two reports: the same request as the first, made after the
    // stopping one, is field by field the report it was before — and it charges the same counts.
    let (again_ok, again_ok_budget) = analyze(
        &fixture,
        &request(&fixture, ok, vec![AnalysisStage::Ssa]),
        limits(),
    );
    assert_eq!(
        without_elapsed(&again_ok),
        without_elapsed(&first_ok),
        "a stopped member does not leak into another member's report"
    );
    assert_eq!(
        counted_usage(&again_ok_budget.usage()),
        counted_usage(&first_ok_budget.usage())
    );
}

#[test]
fn a_class_whose_header_cannot_be_read_fabricates_no_method_result() {
    // The physical definition a request names is the bytes of a real class file, and here those
    // bytes are a class file whose constant pool is cut off: the snapshot still opens (the magic
    // is intact), and the failure happens where the class-level header is decoded. It is the
    // class's own failure, so the answer about the member is no answer: no body fact, no coverage,
    // no phase behind the read, and the code is the reader's.
    let whole = class_bytes(
        b"p/Broken",
        52,
        &[(0x0009, b"run", b"()V", Some(vec![0xb1]))],
    );
    let truncated = &whole[..16];
    let fixture = fixture(truncated);
    let run = member(&fixture, b"run", b"()V");
    let request = request(&fixture, run.clone(), vec![AnalysisStage::Ssa]);
    let (report, budget) = analyze(&fixture, &request, limits());

    assert_eq!(report.method, run);
    assert_eq!(
        stage(&report, AnalysisStage::RawFacts),
        StageState::Failed {
            code: "classfile_decode".to_string()
        },
        "the class-level header read is the phase that failed"
    );
    assert!(
        report.stages[1..]
            .iter()
            .all(|result| result.state == StageState::NotPerformed),
        "no phase behind the failed read ran: {:?}",
        report.stages
    );
    assert!(matches!(
        &report.execution,
        ExecutionReport::Failed {
            reason: TerminationReason::Error { code },
            ..
        } if code == "classfile_decode"
    ));
    assert_eq!(diagnostic_codes(&report), vec!["classfile_decode"]);
    assert_eq!(report.diagnostics[0].severity, DiagnosticSeverity::Error);
    assert_eq!(
        report.body,
        MethodBodyState::NotInspected,
        "the member's own bytes were never reached, so no body fact is stated"
    );
    assert!(
        !matches!(
            report.body,
            MethodBodyState::Present | MethodBodyState::DeclaredWithoutBody { .. }
        ),
        "a class-level failure must not become a claim about the member"
    );
    assert_eq!(
        report.coverage,
        Coverage::not_requested(),
        "no BCI range of the member was scanned or skipped"
    );
    assert_eq!(
        budget.usage().method_bodies,
        0,
        "the body of a member of an undecodable class is never attempted"
    );
    assert_eq!(
        budget.usage().class_headers,
        1,
        "the one header read failed"
    );
    assert_eq!(
        report.quality,
        Quality::Fallback,
        "no canonical artifact was produced, so the quality plane is not `Conservative`"
    );
}
