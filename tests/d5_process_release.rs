//! D5 task 7.2: the process and release gates — what a demand path does to the *process* it runs in.
//!
//! # What 7.2 asks for
//!
//! Task 7.2 asks for gates that run the demand paths in their own debug and release processes and
//! that prove, with reproducible numbers rather than with a promise: **no abort**, **no work left
//! behind**, **real charges** and **bounded ownership**. Each half of that is one child process
//! below, and the parent re-reads what the child measured:
//!
//! | child | what it drives | what it must show |
//! | --- | --- | --- |
//! | [`child_deep_expressions_terminate_normally`] | one method whose body is `1 + 1 + …`, 8, 1024 and 131072 additions deep, and the deepest one under a budget that stops it | the process exits normally at every depth (a stack overflow or an abort would show as a signal, not a status), the run states its own outcome, and the deepest one's stop is the budget's |
//! | [`child_a_stop_at_every_stage_releases_what_it_held`] | one recovery per counted budget dimension, each tightened below what the run's *own measurement* says that body needs | every stop is attributed to the dimension that caused it, the dimension whose refusal is tolerated instead of stopping the run is stated as such, and after each stop the counters have stopped moving and the facts handle is back to its baseline |
//! | [`child_the_abandon_sequence_leaves_no_work_behind`] | enumerate a class, recover two different members, then drop everything | enumeration decodes nothing, each run is charged its own work, and discarding the results releases the prepared facts handle back to its baseline |
//! | [`child_the_cache_and_a_slow_consumer_stay_bounded`] | the bulk operation over one archive, with a one-entry facts store and with a consumer that takes its time | a capacity refusal is a refusal and not a hit, `clear()` empties the store, and a slow consumer is delivered the whole scope while the retained weight stays inside the window's ceiling |
//!
//! # How it is run, and what it needs
//!
//! Every case is `#[ignore]`d, because the *point* of this file is that the same gates run in two
//! build profiles and that the failure mode they exist for — an abort — is only observable from
//! outside the process that aborted:
//!
//! ```text
//! cargo test --test d5_process_release --all-features --locked -- --ignored --nocapture
//! cargo test --release --test d5_process_release --all-features --locked -- --ignored --nocapture
//! ```
//!
//! The child processes are this same test binary, re-invoked by the parent through
//! `std::env::current_exe()` (the repository's own rule for a test that needs a process: no
//! `thread::spawn`, no shell, no `/tmp` path that is not this file's own scratch).
//! `--all-features` is required because the counters this file reads (`jarde::d0_counts`) are the
//! test-support port; the release profile builds the same sources with the same feature.
//!
//! # What is *not* claimed here
//!
//! No wall-clock threshold is asserted anywhere: a duration is only ever something a *slow consumer*
//! does to the producers, and the boundedness this file checks is the window's own published ceiling
//! and the store's own capacity. Task 7.3 owns the timing protocol, and the RSS, throughput and
//! first-result claims it makes are not made here.

mod bulk_support;

use bulk_support::{
    FLAT_PREFIXES, Recorder, container_roots, environment as bulk_environment, flat_fixture, open,
    request as bulk_request, tree_scope,
};
use jarde::*;
use std::collections::BTreeMap;
use std::process::Command;
use std::slice;
use std::sync::Arc;
use std::time::Duration;

/// The prefix every line a child states its own measurements under. The parent reads these instead
/// of trusting an exit code alone: a child that exited zero while its numbers say the opposite is
/// exactly what a gate has to catch.
const CHILD_LINE: &str = "D5-CHILD";

/// The members this file's own cases run in: the eight-member sample the change's counting gates use.
const SCOPE: &[u8] = include_bytes!("fixtures/p3-scope/v8/Scope.class");

/// A second class, for the store's capacity: a one-entry store can only be observed refusing a
/// *second* answer, and two different class files are two different answers.
const SHAPE: &[u8] = include_bytes!("fixtures/p3-declaration/v8/Shape.class");

// -------------------------------------------------------------------------------------------
// The request shape every case here is made under.
// -------------------------------------------------------------------------------------------

/// Every dimension a case here uses, bounded generously: a stop in this file is always the one the
/// case itself caused with a tightened limit.
fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 24,
        archive_entries: 1024,
        entry_bytes: 1 << 24,
        read_bytes: 1 << 24,
        class_bytes: 1 << 24,
        attribute_bytes: 1 << 24,
        code_bytes: 1 << 24,
        result_items: 4096,
        output_bytes: 1 << 24,
        class_headers: 64,
        method_bodies: 64,
        ir_items: 1 << 24,
        ir_edges: 1 << 24,
        analysis_steps: 1 << 24,
        normalization_clones: 1 << 22,
        nested_depth: 4,
        dependency_depth: 8,
        elapsed_millis: 60_000,
    }
}

fn open_class(bytes: &[u8], budget: &mut Budget) -> ArtifactSnapshot {
    ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), budget)
        .expect("the committed fixture is a readable class file")
}

fn environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
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
    ResolutionEnvironment {
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
    }
}

/// The definition of one standalone class, read through the public listing entry.
fn definition_of(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    budget: &mut Budget,
) -> PhysicalDefinitionId {
    engine
        .list_class_declarations(snapshot, &PhysicalScope::SnapshotAll, budget)
        .expect("the fixture's one class is listed")
        .items
        .first()
        .expect("the standalone snapshot holds one class")
        .definition
        .clone()
}

fn method_request(
    snapshot: &ArtifactSnapshot,
    definition: &PhysicalDefinitionId,
    name: &[u8],
    descriptor: &[u8],
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment: environment(snapshot),
        method: PhysicalMethodId {
            owner: definition.clone(),
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    }
}

// -------------------------------------------------------------------------------------------
// The assembled deep expression
// -------------------------------------------------------------------------------------------

/// One class file declaring `public static int deep()` whose body is `1 + 1 + … + 1`, `additions`
/// times.
///
/// The body is deliberately **left-deep** (`((1 + 1) + 1) + …`), so the statement builder, the
/// region walk and the emitter all reach the expression's own depth, and the assembled bytes are
/// this file's own construction: no fixture, no compiler, no `/tmp` path outside this file's scratch.
/// `additions = 0` is the single `iconst_1; ireturn`.
fn deep_expression_class(additions: usize) -> Vec<u8> {
    let utf8 = |bytes: &mut Vec<u8>, text: &[u8]| {
        bytes.push(1);
        bytes.extend_from_slice(&(text.len() as u16).to_be_bytes());
        bytes.extend_from_slice(text);
    };
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&0xcafe_babe_u32.to_be_bytes());
    bytes.extend_from_slice(&0_u16.to_be_bytes());
    bytes.extend_from_slice(&52_u16.to_be_bytes());
    bytes.extend_from_slice(&8_u16.to_be_bytes());
    utf8(&mut bytes, b"Deep"); // 1
    bytes.extend_from_slice(&[7, 0, 1]); // 2: Class #1
    utf8(&mut bytes, b"java/lang/Object"); // 3
    bytes.extend_from_slice(&[7, 0, 3]); // 4: Class #3
    utf8(&mut bytes, b"deep"); // 5
    utf8(&mut bytes, b"()I"); // 6
    utf8(&mut bytes, b"Code"); // 7
    bytes.extend_from_slice(&0x0021_u16.to_be_bytes()); // class flags: public, super
    bytes.extend_from_slice(&2_u16.to_be_bytes());
    bytes.extend_from_slice(&4_u16.to_be_bytes());
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // interfaces
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // fields
    bytes.extend_from_slice(&1_u16.to_be_bytes()); // methods
    bytes.extend_from_slice(&0x0009_u16.to_be_bytes()); // public static
    bytes.extend_from_slice(&5_u16.to_be_bytes());
    bytes.extend_from_slice(&6_u16.to_be_bytes());
    bytes.extend_from_slice(&1_u16.to_be_bytes()); // attributes of this member
    bytes.extend_from_slice(&7_u16.to_be_bytes());
    let mut code = vec![0x04_u8]; // iconst_1 — the start of the sum
    for _ in 0..additions {
        code.push(0x04); // iconst_1
        code.push(0x60); // iadd
    }
    code.push(0xac); // ireturn
    let body = 2 + 2 + 4 + code.len() + 2 + 2;
    bytes.extend_from_slice(&(body as u32).to_be_bytes());
    bytes.extend_from_slice(&2_u16.to_be_bytes()); // max_stack
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // max_locals
    bytes.extend_from_slice(&(code.len() as u32).to_be_bytes());
    bytes.extend_from_slice(&code);
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // exception table
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // attributes of the code
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // attributes of the class
    bytes
}

// -------------------------------------------------------------------------------------------
// What a child states about itself
// -------------------------------------------------------------------------------------------

/// One measurement, as the line the parent reads it from.
fn child_line(case: &str, fields: &[(&str, String)]) -> String {
    let rendered: Vec<String> = fields
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect();
    format!("{CHILD_LINE} {case} {}", rendered.join(" "))
}

/// Every `name=value` field of one child line, so a parent case can assert on a number the *child*
/// measured rather than on the child's exit code alone.
fn fields_of(line: &str) -> BTreeMap<String, String> {
    line.split_whitespace()
        .filter_map(|field| field.split_once('='))
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect()
}

/// The stop a run stated, in the vocabulary a caller reads it in.
///
/// A stop has two shapes and both are statements: an entry that had no payload to hand out refuses
/// with an error, and an entry that had one states the stop *in* it, where the analysis plane names
/// the budget dimension that refused and the recovery plane names what that cost the presentation.
/// Both are read here, because a gate on "which stage stopped" has to see the stage that stopped
/// rather than the shape the stop was published in.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Stop {
    /// The entry answered with a payload that states it stopped.
    Reported {
        /// Every diagnostic code the payload and its analysis stated.
        codes: Vec<String>,
        /// The budget dimension the analysis plane refused, when one did.
        dimension: Option<String>,
    },
    /// The entry refused with an error: no payload exists.
    Refused { message: String },
}

/// One spelling for both sides of an attribution comparison: `ir_items`, `IrItems` and
/// `budget_exceeded_ir_items` all become `iritems`, so the dimension a limit was set on and the
/// dimension a stop names are compared as the same fact however each side spells it.
fn normalized(text: &str) -> String {
    text.chars()
        .filter(char::is_ascii_alphanumeric)
        .collect::<String>()
        .to_ascii_lowercase()
}

impl Stop {
    fn describes(&self, dimension: &str) -> bool {
        let wanted = normalized(dimension);
        match self {
            Self::Reported { codes, dimension } => {
                codes.iter().any(|code| normalized(code).contains(&wanted))
                    || dimension
                        .as_ref()
                        .is_some_and(|named| normalized(named) == wanted)
            }
            Self::Refused { message } => normalized(message).contains(&wanted),
        }
    }

    fn render(&self) -> String {
        match self {
            Self::Reported { codes, dimension } => format!(
                "report:{}:dimension={}",
                codes.join(","),
                dimension.clone().unwrap_or_else(|| "none".to_string())
            ),
            Self::Refused { message } => {
                format!("refused:{}", message.replace(' ', "_"))
            }
        }
    }
}

/// One recovery that either produces an artifact or states why it did not.
fn recovery(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    request: &MethodAnalysisRequest,
    selection: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> std::result::Result<RecoveredMethod, Stop> {
    let recovered = match engine.recover_method_with_evidence(
        slice::from_ref(snapshot),
        request,
        selection,
        budget,
    ) {
        Ok(recovered) => recovered,
        Err(error) => {
            return Err(Stop::Refused {
                message: error.to_string(),
            });
        }
    };
    if recovered.recovery().produced() {
        return Ok(recovered);
    }
    let mut codes: Vec<String> = recovered
        .recovery()
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.clone())
        .collect();
    codes.extend(
        recovered
            .analysis()
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.clone()),
    );
    let dimension = match &recovered.analysis().execution {
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded { dimension },
            ..
        } => Some(format!("{dimension:?}")),
        _ => None,
    };
    // The recovery plane states its own stop: a charge the *emitter* was refused (the output bound is
    // the one that always is) never reaches the analysis plane, and the run's own stop record names
    // the dimension it was refused on.
    let dimension = dimension.or_else(|| match recovered.recovery().outcome {
        RecoveryOutcome::Stopped(StopReason::Budget { dimension, .. }) => {
            Some(format!("{dimension:?}"))
        }
        _ => None,
    });
    Err(Stop::Reported { codes, dimension })
}

/// The counters a case leaves behind: one reading before its work, and the readings that answer
/// "what did it do" and "was anything still in flight when it was over".
struct Ledger {
    before: d0_counts::Counts,
}

impl Ledger {
    /// A ledger opened before the case's work starts.
    fn opened() -> Self {
        Self {
            before: d0_counts::snapshot(),
        }
    }

    /// What the case's own work has moved since the ledger was opened, read now.
    fn counted(&self) -> d0_counts::Counts {
        self.before.since(d0_counts::snapshot())
    }
}

/// The counting tests of this binary run one at a time: the port is one process-wide set of counters,
/// so two cases counting at once would each read the other's work. The children the parent spawns
/// are separate processes and take their own copy of this lock.
fn gate() -> std::sync::MutexGuard<'static, ()> {
    static GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());
    GATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// What one case's counters say, as the two fields a parent reads: the work the case did, and what
/// was still in flight once the case had dropped everything it held. The second is the number this
/// file's "no work left behind" claim is made of — a counter total is the case's *work*, not its
/// leftovers.
fn counter_fields(moved: d0_counts::Counts, after_drop: d0_counts::Counts) -> (String, String) {
    (
        moved.total().to_string(),
        after_drop.total().saturating_sub(moved.total()).to_string(),
    )
}

// -------------------------------------------------------------------------------------------
// The child processes
// -------------------------------------------------------------------------------------------

/// The depths this file drives, and what each of them must answer.
///
/// The layer's own depth ceiling (`MAX_VALUE_DEPTH` in the statement builder) is a *stated* boundary:
/// a body nested deeper than it is presented as a fallback rather than aborting, which is what the
/// last two rows are here to state — including the one under a budget that stops it in the middle.
const DEPTHS: [usize; 3] = [8, 1024, 131_072];

/// The gates 7.2 asks for, and then the process exits normally.
#[ignore = "the process gate: run with `-- --ignored` in both the debug and the release profile"]
#[test]
fn child_deep_expressions_terminate_normally() {
    let _gate = gate();
    let engine = Engine::new();
    for additions in DEPTHS {
        let class = deep_expression_class(additions);
        let mut opening = Budget::new(limits());
        let snapshot = open_class(&class, &mut opening);
        let mut budget = Budget::new(limits());
        let definition = definition_of(&engine, &snapshot, &mut budget);
        let request = method_request(&snapshot, &definition, b"deep", b"()I");
        let ledger = Ledger::opened();
        let mut budget = Budget::new(limits());
        let recovered = recovery(
            &engine,
            &snapshot,
            &request,
            &RecoveryEvidenceRequest::essential(),
            &mut budget,
        )
        .unwrap_or_else(|stop| panic!("depth {additions} stops: {}", stop.render()));
        // What this run states about itself, read while its payload is alive: the payload is dropped
        // before the release reading below, and the assertions that read it come with it.
        let (quality, text_bytes) = (
            recovered.recovery().quality,
            recovered.recovery().text.len(),
        );
        assert!(
            text_bytes > 0,
            "depth {additions}: the run writes the body it produced"
        );
        assert!(
            matches!(quality, ir::Quality::Structured | ir::Quality::Fallback),
            "depth {additions}: the run states its own quality: {quality:?}"
        );
        let deep = additions == *DEPTHS.last().expect("the table states a deepest depth");
        if deep {
            // The deepest case is the one the layer's own depth ceiling decides: its body is refused
            // as a region and quoted, which is the *stated* boundary rather than an abort — the
            // difference this child exists to make observable, because an abort would have taken the
            // process with it and the parent would read a signal instead of an exit status.
            assert!(
                matches!(quality, ir::Quality::Fallback),
                "the deepest body is refused as a fallback rather than presented whole: {quality:?}"
            );
        }
        let usage = budget.usage();
        let moved = ledger.counted();
        drop(recovered);
        drop(snapshot);
        let settled = ledger.counted();
        let (work, after_drop) = counter_fields(moved, settled);
        println!(
            "{}",
            child_line(
                "depth",
                &[
                    ("additions", additions.to_string()),
                    ("outcome", "produced".to_string()),
                    ("quality", format!("{quality:?}").to_ascii_lowercase()),
                    ("text_bytes", text_bytes.to_string()),
                    (
                        "ir_items",
                        usage
                            .counted_usage(CountedBudgetDimension::IrItems)
                            .to_string()
                    ),
                    ("body_decodes", moved.body_decodes.to_string()),
                    ("recovery_runs", moved.recovery_runs.to_string()),
                    ("work", work),
                    ("work_after_drop", after_drop),
                ],
            )
        );
        assert!(
            settled == moved,
            "depth {additions}: dropping the run's payload moves no counter, so nothing of it was \
             still in flight"
        );
        assert!(moved.body_decodes >= 1 && moved.recovery_runs == 1);
    }

    // The same deepest body under a budget that stops it in the middle: the run must state the stop
    // it really hit, and the process must still be here to state it.
    let additions = *DEPTHS.last().expect("the table states a deepest depth");
    let class = deep_expression_class(additions);
    let mut opening = Budget::new(limits());
    let snapshot = open_class(&class, &mut opening);
    let engine = Engine::new();
    let mut budget = Budget::new(limits());
    let definition = definition_of(&engine, &snapshot, &mut budget);
    let request = method_request(&snapshot, &definition, b"deep", b"()I");
    let tight = task_limits(&[BudgetOverride::new("ir_items", 1_000).expect("a legal dimension")])
        .expect("a legal limit");
    let ledger = Ledger::opened();
    let mut budget = Budget::new(tight);
    let stopped = recovery(
        &engine,
        &snapshot,
        &request,
        &RecoveryEvidenceRequest::essential(),
        &mut budget,
    )
    .expect_err("a budget of a thousand items stops this body");
    let settled = ledger.counted();
    let (work, after_drop) = counter_fields(settled, ledger.counted());
    println!(
        "{}",
        child_line(
            "depth-stop",
            &[
                ("additions", additions.to_string()),
                ("stop", stopped.render()),
                ("work", work),
                ("work_after_drop", after_drop),
            ],
        )
    );
    assert!(
        stopped.describes("ir_items"),
        "the stop of the deepest body is attributed to the dimension that caused it: {}",
        stopped.render()
    );
    println!(
        "{}",
        child_line("deep-expressions", &[("ok", "true".to_string())])
    );
}

/// Every counted budget dimension, tightened to its smallest legal limit: each stop is attributed to
/// the dimension that caused it, and none of them leaves work behind.
#[ignore = "the process gate: run with `-- --ignored` in both the debug and the release profile"]
#[test]
fn child_a_stop_at_every_stage_releases_what_it_held() {
    let _gate = gate();
    // One reading of the fixture's class **prepared and released** first, so the handle this file
    // checks below has a baseline it was observed at rather than one assumed.
    let engine = Engine::new();
    let mut opening = Budget::new(limits());
    let snapshot = open_class(SCOPE, &mut opening);
    let mut budget = Budget::new(limits());
    let read = snapshot
        .prepared_root_class(&mut budget)
        .expect("the fixture's class prepares");
    let prepared = jarde_reader::prepared::PreparedClass::prepare(&read, &mut budget)
        .expect("the class structure is verified");
    let facts = Arc::clone(prepared.facts_handle());
    assert_eq!(
        Arc::strong_count(&facts),
        2,
        "the preparation and this observer are the two owners of the facts"
    );
    drop(prepared);
    assert_eq!(
        Arc::strong_count(&facts),
        1,
        "the preparation's own hold is released with it"
    );
    let baseline = Arc::strong_count(&facts);
    drop(read);
    assert_eq!(
        Arc::strong_count(&facts),
        baseline,
        "the read never held the handle the preparation handed out"
    );

    // What this body's one recovery really charges per dimension, measured once with a budget that
    // stops nothing: it is what makes the stopping limit below a fact about the run rather than a
    // number this file guessed.
    let generous = measured_usage(&engine);
    let dimensions = [
        "input_bytes",
        "archive_entries",
        "entry_bytes",
        "read_bytes",
        "class_bytes",
        "attribute_bytes",
        "code_bytes",
        "result_items",
        "output_bytes",
        "class_headers",
        "method_bodies",
        "ir_items",
        "ir_edges",
        "analysis_steps",
        "normalization_clones",
    ];
    let mut attributed = 0usize;
    let mut tolerated = 0usize;
    let mut not_charged = 0usize;
    for dimension in dimensions {
        let needed = generous.counted_usage(counted_dimension(dimension));
        // A dimension this body charges nothing of is not a stage of *this* body: a limit cannot
        // stop what is never charged, and this file says so instead of pretending otherwise.
        if needed <= 1 {
            not_charged += 1;
            println!(
                "{}",
                child_line(
                    "stage-not-charged",
                    &[
                        ("dimension", dimension.to_string()),
                        ("needed", needed.to_string()),
                    ],
                )
            );
            continue;
        }
        // The limits tried, from the largest one that is still below the run's own need down to the
        // smallest legal one. A limit that still presents the body is not a failure of this gate:
        // some refusals are *tolerated* — a refused callee read is a member's own refusal, not the
        // run's stop — and the point here is which of them really ends the run and which does not.
        let mut tried: Vec<u64> = vec![needed - 1, needed / 2, 1];
        tried.dedup();
        let mut stopped_at = None;
        for limit in tried {
            let tight =
                task_limits(&[BudgetOverride::new(dimension, limit)
                    .expect("a legal dimension and a legal limit")])
                .expect("a legal limit set");
            let mut opening = Budget::new(limits());
            let snapshot = open_class(SCOPE, &mut opening);
            let mut budget = Budget::new(limits());
            let definition = definition_of(&engine, &snapshot, &mut budget);
            let request = method_request(&snapshot, &definition, b"receiver", b"(J)J");
            let ledger = Ledger::opened();
            let mut budget = Budget::new(tight);
            let answer = recovery(
                &engine,
                &snapshot,
                &request,
                &RecoveryEvidenceRequest::essential(),
                &mut budget,
            );
            drop(snapshot);
            let settled = ledger.counted();
            let (work, after_drop) = counter_fields(settled, settled_again(&ledger));
            match answer {
                Ok(_) => println!(
                    "{}",
                    child_line(
                        "stage-attempt",
                        &[
                            ("dimension", dimension.to_string()),
                            ("needed", needed.to_string()),
                            ("limit", limit.to_string()),
                            ("result", "presented".to_string()),
                            ("work", work),
                            ("work_after_drop", after_drop),
                        ],
                    )
                ),
                Err(stop) => {
                    println!(
                        "{}",
                        child_line(
                            "stage-stop",
                            &[
                                ("dimension", dimension.to_string()),
                                ("needed", needed.to_string()),
                                ("limit", limit.to_string()),
                                ("stop", stop.render()),
                                ("work", work),
                                ("work_after_drop", after_drop),
                            ],
                        )
                    );
                    assert!(
                        stop.describes(dimension),
                        "the stop of a `{dimension}` budget below the body's own need is attributed \
                         to it: {}",
                        stop.render()
                    );
                    assert_eq!(
                        settled,
                        settled_again(&ledger),
                        "dropping what a stopped run held moves no counter"
                    );
                    stopped_at = Some(limit);
                    break;
                }
            }
        }
        match stopped_at {
            Some(_) => attributed += 1,
            None => {
                // Stated, not hidden: every limit below the run's own need was tolerated, and this
                // line is the record of it (the ledger in `tests/d5_acceptance_ledger.rs` carries
                // the same figures).
                println!(
                    "{}",
                    child_line(
                        "stage-tolerated",
                        &[
                            ("dimension", dimension.to_string()),
                            ("needed", needed.to_string()),
                            ("lowest_limit", "1".to_string()),
                        ],
                    )
                );
                tolerated += 1;
            }
        }
    }
    assert_eq!(
        attributed + tolerated + not_charged,
        dimensions.len(),
        "every counted dimension is classified exactly once"
    );
    assert!(
        attributed >= 7,
        "most of the dimensions this body really charges stop it, and each of those stops is \
         attributed to its dimension: {attributed} stopped, {tolerated} tolerated, {not_charged} \
         never charged"
    );
    println!(
        "{}",
        child_line(
            "stage-tolerance",
            &[
                ("stopped", attributed.to_string()),
                ("tolerated", tolerated.to_string()),
                ("not_charged", not_charged.to_string()),
            ]
        )
    );
    // The ownership baseline is a measurement, not an assumption: after every stop above, and after
    // the last of them dropped what it held, the handle stands where it stood before the first.
    assert_eq!(
        Arc::strong_count(&facts),
        baseline,
        "no stop kept a hold on the facts handle"
    );
    println!(
        "{}",
        child_line(
            "stage-stops",
            &[
                ("attributed", attributed.to_string()),
                ("facts_strong_count", Arc::strong_count(&facts).to_string()),
                ("ok", "true".to_string()),
            ]
        )
    );
}

/// The counted dimension one snake_case name stands for.
///
/// The mapping is stated here rather than derived from the library's own, because the name a limit
/// is set on and the dimension a usage reading is asked for are two spellings of the same fact and a
/// gate that let one follow the other could not catch a stop attributed to the wrong dimension.
fn counted_dimension(name: &str) -> CountedBudgetDimension {
    match name {
        "input_bytes" => CountedBudgetDimension::InputBytes,
        "archive_entries" => CountedBudgetDimension::ArchiveEntries,
        "entry_bytes" => CountedBudgetDimension::EntryBytes,
        "read_bytes" => CountedBudgetDimension::ReadBytes,
        "class_bytes" => CountedBudgetDimension::ClassBytes,
        "attribute_bytes" => CountedBudgetDimension::AttributeBytes,
        "code_bytes" => CountedBudgetDimension::CodeBytes,
        "result_items" => CountedBudgetDimension::ResultItems,
        "output_bytes" => CountedBudgetDimension::OutputBytes,
        "class_headers" => CountedBudgetDimension::ClassHeaders,
        "method_bodies" => CountedBudgetDimension::MethodBodies,
        "ir_items" => CountedBudgetDimension::IrItems,
        "ir_edges" => CountedBudgetDimension::IrEdges,
        "analysis_steps" => CountedBudgetDimension::AnalysisSteps,
        "normalization_clones" => CountedBudgetDimension::NormalizationClones,
        other => panic!("`{other}` is not one of the counted dimensions this gate drives"),
    }
}

/// What one recovery of this file's member charges per dimension, under a budget that stops nothing.
fn measured_usage(engine: &Engine) -> UsageSnapshot {
    let mut opening = Budget::new(limits());
    let snapshot = open_class(SCOPE, &mut opening);
    let mut budget = Budget::new(limits());
    let definition = definition_of(engine, &snapshot, &mut budget);
    let request = method_request(&snapshot, &definition, b"receiver", b"(J)J");
    let mut budget = Budget::new(limits());
    let recovered = recovery(
        engine,
        &snapshot,
        &request,
        &RecoveryEvidenceRequest::essential(),
        &mut budget,
    )
    .expect("the fixture's member is presented under a budget that stops nothing");
    assert!(
        recovered.recovery().produced(),
        "the measured run is the producing one"
    );
    budget.usage()
}

/// The same reading as [`Ledger::counted`], taken once more: the pair is how "the counters have
/// stopped moving" is checked without a clock.
fn settled_again(ledger: &Ledger) -> d0_counts::Counts {
    ledger.counted()
}

/// The abandon sequence: enumerate, switch method, discard — and nothing of it outlives the sequence.
#[ignore = "the process gate: run with `-- --ignored` in both the debug and the release profile"]
#[test]
fn child_the_abandon_sequence_leaves_no_work_behind() {
    let _gate = gate();
    let engine = Engine::new();
    let mut opening = Budget::new(limits());
    let snapshot = open_class(SCOPE, &mut opening);
    let ledger = Ledger::opened();

    // (1) Enumerate: reading a class's declarations and its member table is neither a body decode
    //     nor a recovery, and this run's counters say so.
    let mut budget = Budget::new(limits());
    let listing = engine
        .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the fixture's class is listed");
    let definition = listing.items[0].definition.clone();
    let members = engine
        .list_members(&snapshot, &definition, &mut budget)
        .expect("the fixture's member table is read");
    let enumerating = ledger.counted();
    assert!(
        enumerating.is_silent(),
        "a declaration and member listing decodes no body and runs no recovery: {enumerating:?}"
    );
    assert!(
        members.methods().count() >= 8,
        "the listing really read the member table it was asked for"
    );

    // (2) Switch method: two different members, each charged its own work.
    let mut first_budget = Budget::new(limits());
    let first = recovery(
        &engine,
        &snapshot,
        &method_request(&snapshot, &definition, b"simple", b"()I"),
        &RecoveryEvidenceRequest::essential(),
        &mut first_budget,
    )
    .expect("the first member is presented");
    let after_first = ledger.counted();
    let mut second_budget = Budget::new(limits());
    let second = recovery(
        &engine,
        &snapshot,
        &method_request(&snapshot, &definition, b"receiver", b"(J)J"),
        &RecoveryEvidenceRequest::essential(),
        &mut second_budget,
    )
    .expect("the second member is presented");
    let after_second = ledger.counted();
    let first_only = after_first.since(after_second);
    assert_eq!(
        (
            first_only.class_materializations,
            first_only.body_decodes,
            first_only.recovery_runs
        ),
        (1, 1, 1),
        "switching method is charged exactly the new member's own read, decode and presentation, \
         and nothing of the first one answers for it: {first_only:?}"
    );
    assert_eq!(
        (enumerating.body_decodes, enumerating.recovery_runs),
        (0, 0),
        "nor did the listing before it charge a body"
    );
    assert!(
        first.recovery().text != second.recovery().text,
        "the two members are two different bodies"
    );
    // The second run's own read: no store was attached, so nothing of the first run answered for it.
    assert!(
        second_budget.usage().class_headers >= 1,
        "a request with no store reads the class it needs"
    );

    // (3) Discard: everything the sequence held goes away, and nothing further happens.
    drop(first);
    drop(second);
    let discarded = ledger.counted();
    drop(snapshot);
    let settled = ledger.counted();
    println!(
        "{}",
        child_line(
            "abandon-sequence",
            &[
                ("enumerate", enumerating.total().to_string()),
                ("first", after_first.total().to_string()),
                ("second", after_second.total().to_string()),
                ("discarded", discarded.total().to_string()),
                ("work", settled.total().to_string()),
                (
                    "work_after_drop",
                    settled
                        .total()
                        .saturating_sub(discarded.total())
                        .to_string()
                ),
                ("ok", "true".to_string()),
            ],
        )
    );
    assert_eq!(
        discarded, settled,
        "dropping the results and the snapshot moves no counter: no work outlived the sequence"
    );
    assert_eq!(
        settled.body_decodes, 2,
        "the sequence decoded the two bodies it asked for and no others"
    );
    assert_eq!(settled.recovery_runs, 2);
}

/// The facts store and a slow consumer: a capacity refusal is a refusal, `clear()` empties the
/// store, and slow consumption is delivered the whole scope inside the window's own ceiling.
#[ignore = "the process gate: run with `-- --ignored` in both the debug and the release profile"]
#[test]
fn child_the_cache_and_a_slow_consumer_stay_bounded() {
    let _gate = gate();
    let engine = Engine::new();

    // (a) The class-facts store, on the path that consults it: two standalone classes inspected
    //     through the engine under one budget that carries a one-entry store. The store takes the
    //     first answer, refuses the second, and answers the first class from its own copy on the
    //     second read — a refusal is a re-parse and never a hit.
    let store = FactsCache::current(FactsCapacity::new(1, u64::MAX));
    let mut budget = Budget::new(limits()).with_facts_cache(store.clone());
    let first = open_class(SCOPE, &mut budget);
    engine
        .inspect_header(
            &first,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the first class's header is readable");
    let second = open_class(SHAPE, &mut budget);
    engine
        .inspect_header(
            &second,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the second class's header is readable");
    let after_two = store.report();
    assert_eq!(
        (after_two.stored, after_two.refused_capacity),
        (1, 1),
        "a one-entry store kept the first answer and refused the second: {after_two:?}"
    );
    // The first class again: this one is answered from the store.
    engine
        .inspect_header(
            &first,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the first class's header is answered again");
    let after_hit = store.report();
    assert_eq!(
        (after_hit.hits, after_hit.refused_capacity),
        (1, 1),
        "the kept answer was a hit and the refused one was not: {after_hit:?}"
    );
    store.clear();
    let cleared = store.report();
    assert_eq!(
        (cleared.entries, cleared.retained_bytes),
        (0, 0),
        "`clear()` releases every retained answer: {cleared:?}"
    );

    // (b) The same store on the bulk operation's own container facts: the operation keeps its
    //     residency inside the store's capacity, `clear()` releases all of it, and the answer is the
    //     one the same request gives with no store at all.
    let (snapshot, _opened) = open(flat_fixture());
    let content = vec![snapshot.clone()];
    let mut setup = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut setup, &FLAT_PREFIXES);
    let bulk_environment = bulk_environment(&snapshot, tree_scope(), roots);
    let bulk_request = bulk_request(bulk_environment, 1);
    let container_store = FactsCache::current(FactsCapacity::new(1, u64::MAX));
    let mut budget = Budget::new(bulk_support::limits()).with_facts_cache(container_store.clone());
    let mut sink = Recorder::new();
    let report = engine
        .recover_all(&content, &bulk_request, &mut budget, &mut sink)
        .expect("the fixture's scope is recoverable");
    let cached = container_store.report();
    let answer = (
        report.summary.classes_prepared,
        report.summary.methods_declared,
        report.summary.methods_delivered,
    );
    assert_eq!(
        report.summary.status(),
        "complete",
        "a store that refuses to keep an answer changes no conclusion: {cached:?}"
    );
    assert!(
        cached.entries <= 1 && cached.containers <= 1,
        "the store's residency is its capacity, whatever it was asked for: {cached:?}"
    );
    assert!(
        cached.entries <= 1,
        "the store's residency is its capacity: {cached:?}"
    );
    // `clear()` releases what the bulk operation's own reads left in it, and the report says so.
    container_store.clear();
    let after_clear = container_store.report();
    assert_eq!(
        (after_clear.entries, after_clear.containers),
        (0, 0),
        "`clear()` releases every retained answer: {after_clear:?}"
    );
    assert_eq!(
        after_clear.retained_bytes, 0,
        "and the bytes it retained with them: {after_clear:?}"
    );

    // (c) The same scope with no store at all: the same answer, so nothing of (b) was load-bearing.
    let mut budget = Budget::new(bulk_support::limits());
    let mut plain = Recorder::new();
    let uncached = engine
        .recover_all(&content, &bulk_request, &mut budget, &mut plain)
        .expect("the same scope is recoverable without a store");
    assert_eq!(
        answer,
        (
            uncached.summary.classes_prepared,
            uncached.summary.methods_declared,
            uncached.summary.methods_delivered,
        ),
        "the store changes what the operation costs, never what it answers"
    );

    // (d) A consumer that takes its time: the whole scope is delivered, the retained weight stays
    //     inside the window's published ceiling, and the delivery itself is really charged.
    let mut budget = Budget::new(bulk_support::limits());
    let mut slow = Recorder::new();
    slow.delay = Duration::from_millis(2);
    let slow_report = engine
        .recover_all(&content, &bulk_request, &mut budget, &mut slow)
        .expect("the same scope is recoverable by a slow consumer");
    let delivered = slow_report.summary.methods_delivered;
    assert!(
        slow_report.final_delivered,
        "a slow consumer is handed the stream's final record"
    );
    assert_eq!(
        slow_report.summary.status(),
        "complete",
        "{:?}",
        slow_report.summary
    );
    assert_eq!(
        delivered, uncached.summary.methods_delivered,
        "the consumer's own pace changes nothing about what it is handed"
    );
    assert!(
        slow_report.window.buffered_weight_high_water <= slow_report.window.buffered_weight_limit,
        "the retained weight never crosses the window's ceiling: {:?}",
        slow_report.window
    );
    assert!(
        slow_report.window.largest_result_weight > 0,
        "the run really held results while the consumer answered: {:?}",
        slow_report.window
    );
    assert!(
        slow_report
            .delivery_usage
            .counted_usage(CountedBudgetDimension::ResultItems)
            > 0,
        "delivering to the consumer is really charged: {:?}",
        slow_report.delivery_usage
    );
    assert!(
        slow_report
            .method_usage
            .counted_usage(CountedBudgetDimension::IrItems)
            > 0,
        "and the work behind the records is charged to the methods that did it"
    );
    println!(
        "{}",
        child_line(
            "cache-and-slow-consumer",
            &[
                ("stored", after_two.stored.to_string()),
                ("refused_capacity", after_two.refused_capacity.to_string()),
                ("hits", after_hit.hits.to_string()),
                ("entries_after_clear", after_clear.entries.to_string()),
                ("methods_delivered", delivered.to_string()),
                (
                    "buffered_weight_high_water",
                    slow_report.window.buffered_weight_high_water.to_string()
                ),
                (
                    "buffered_weight_limit",
                    slow_report.window.buffered_weight_limit.to_string()
                ),
                ("final_delivered", slow_report.final_delivered.to_string()),
                ("ok", "true".to_string()),
            ],
        )
    );
}

// -------------------------------------------------------------------------------------------
// The parent: the same gates, in their own processes
// -------------------------------------------------------------------------------------------

/// The children this file owns, and the lines the parent requires of each of them.
///
/// A marker is a *statement the child makes about itself*, and the parent requires it verbatim: the
/// markers below are the ones that carry the claims this file exists for — that every depth answered,
/// that every stopped stage was attributed to the dimension that caused it, that the abandon sequence
/// decoded exactly its two bodies, and that the store and the slow consumer stayed inside their own
/// ceilings.
const CHILDREN: [(&str, &[&str], bool); 4] = [
    (
        "child_deep_expressions_terminate_normally",
        &[
            "D5-CHILD depth additions=8",
            "additions=1024",
            "additions=131072",
            "D5-CHILD depth-stop",
            "work_after_drop=0",
            "D5-CHILD deep-expressions ok=true",
        ],
        true,
    ),
    (
        "child_a_stop_at_every_stage_releases_what_it_held",
        &[
            "D5-CHILD stage-stop dimension=read_bytes",
            "dimension=ir_items",
            "dimension=output_bytes",
            "work_after_drop=0",
            "D5-CHILD stage-stops",
            "ok=true",
        ],
        true,
    ),
    (
        "child_the_abandon_sequence_leaves_no_work_behind",
        &[
            "D5-CHILD abandon-sequence",
            "enumerate=0",
            "work_after_drop=0",
            "D5-CHILD",
        ],
        true,
    ),
    (
        "child_the_cache_and_a_slow_consumer_stay_bounded",
        &[
            "D5-CHILD cache-and-slow-consumer",
            "refused_capacity=1",
            "hits=1",
            "entries_after_clear=0",
            "final_delivered=true",
        ],
        // This child states the store's own residency and the window's own ceiling rather than the
        // demand path's counters: it is checked by the fields those lines carry, not by `work`.
        false,
    ),
];

/// Every gate above runs to a normal exit in a process of its own.
///
/// This is the half of 7.2 that cannot be asserted from inside the process that would fail: an abort
/// — a stack overflow in a recursive walk is the one this file drives for — takes the process with
/// it, so the only place its absence is observable is here, where the *parent* reads the exit status
/// and the child's own numbers. The same cases therefore run in the debug and the release profile,
/// because the two profiles have different stack frames and different recursion limits:
///
/// ```text
/// cargo test --test d5_process_release --all-features --locked -- --ignored --nocapture
/// cargo test --release --test d5_process_release --all-features --locked -- --ignored --nocapture
/// ```
#[ignore = "the process gate: run with `-- --ignored` in both the debug and the release profile"]
#[test]
fn every_gate_runs_to_a_normal_exit_in_its_own_process() {
    let exe = std::env::current_exe().expect("this test knows its own binary");
    let mut checked = 0usize;
    for (child, required, counts) in CHILDREN {
        let output = Command::new(&exe)
            .args([
                "--exact",
                child,
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .output()
            .expect("the child process starts");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(
            output.status.success(),
            "the child `{child}` must exit normally — an abort is what this gate exists for \
             ({:?}):\n{}\n{}",
            output.status,
            stdout,
            String::from_utf8_lossy(&output.stderr)
        );
        // The child's statements are read from wherever they sit in its output: the harness prints a
        // test's own `--nocapture` output after the line that names the test, so a marker is found in
        // the middle of a line as often as at its start.
        let statements: Vec<String> = stdout
            .split(CHILD_LINE)
            .skip(1)
            .map(|rest| rest.lines().next().unwrap_or_default().to_string())
            .collect();
        assert!(
            !statements.is_empty(),
            "the child `{child}` states its own measurements:\n{stdout}"
        );
        for marker in required {
            assert!(
                stdout.contains(marker),
                "the child `{child}` must state `{marker}`:\n{stdout}"
            );
        }
        // The child's own numbers are read here as numbers, because a gate whose child reported work
        // it did not do would pass on its exit status alone. `work_after_drop` is the child's own
        // statement that nothing was in flight once its case had dropped what it held.
        let mut leftover_statements = 0usize;
        let mut work_statements = 0usize;
        for line in &statements {
            let fields = fields_of(line);
            if let Some(leftover) = fields.get("work_after_drop") {
                assert_eq!(
                    leftover.parse::<u64>().expect("a number"),
                    0,
                    "`{child}`: nothing was left in flight when the case dropped what it held: {line}"
                );
                leftover_statements += 1;
            }
            if let Some(work) = fields.get("work") {
                assert!(
                    work.parse::<u64>().expect("a number") > 0,
                    "`{child}`: the case really did the work it states: {line}"
                );
                work_statements += 1;
            }
            if let Some(delivered) = fields.get("methods_delivered") {
                assert!(
                    delivered.parse::<u64>().expect("a number") > 0,
                    "`{child}`: the slow consumer was really handed records: {line}"
                );
            }
            if let Some(high_water) = fields.get("buffered_weight_high_water") {
                let limit: u64 = fields
                    .get("buffered_weight_limit")
                    .expect("a high-water mark is stated with the ceiling it is checked against")
                    .parse::<u64>()
                    .expect("a number");
                assert!(
                    high_water.parse::<u64>().expect("a number") <= limit,
                    "`{child}`: the retained weight stayed inside the window's ceiling: {line}"
                );
            }
        }
        if counts {
            assert!(
                leftover_statements > 0,
                "`{child}`: at least one of its lines states what it left behind"
            );
            assert!(
                work_statements > 0,
                "`{child}`: at least one of its lines states the work it did"
            );
        }
        println!(
            "{CHILD_LINE} parent child={child} exit=0 lines={} markers={}",
            statements.len(),
            required.len()
        );
        checked += 1;
    }
    assert_eq!(checked, CHILDREN.len());
}
