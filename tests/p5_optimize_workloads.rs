//! The fixed workloads `optimize-demand-workloads` measures, with their phases, their counts and
//! the sink ablation (change `optimize-demand-workloads`, tasks 1.2 and 1.3).
//!
//! # Why a test target and not an example
//!
//! Task 1.2 asks for W1–W6a as **reproducible fixed workloads** that depend on no `/tmp` directory
//! and no temporary benchmark corpus. That is exactly what this repository's in-process fixture
//! convention gives: [`bulk_support`]'s ZIP writer over committed `.class` bytes, the same
//! convention `tests/p5_bulk_corpus.rs` uses, and `rawzip` as a dev-dependency. A test target can
//! use both; an example could not without carrying a second fixture builder, so the workloads live
//! here and the campaign driver in `openspec/changes/optimize-demand-workloads/evidence/` runs
//! *this* binary once per sample (one process per sample, as W1 and W6a's cold-start metric
//! require).
//!
//! ```text
//! verify:  cargo test --test p5_optimize_workloads --locked
//! measure: <the test binary> --exact measure_workload --ignored --nocapture
//!          (built by cargo test --test p5_optimize_workloads --no-run --message-format=json;
//!           the campaign script does that, sets the JARDE_OPTIMIZE_* variables and wraps each
//!           process so its wall time and peak RSS are recorded beside the sample)
//! ```
//!
//! # The workloads, as this file defines them
//!
//! | ID | sample | what it runs |
//! | --- | --- | --- |
//! | W1 | `w1-cold-method` | one process: open, find the first member, recover it once |
//! | | `w1-second-request` | the same request again on the same snapshot (control) |
//! | | `w1-second-snapshot` | the same request after a **second** open of the same input |
//! | W2 | `w2-navigation-sequence` | one `class_view` per class over the first eight classes, then the first class again |
//! | W3 | `w3-all-bodies-one-request` | one `class_view` selecting every body of one class |
//! | | `w3-per-method-requests` | the same class's bodies, one request per method |
//! | | `w3-fixed-set-across-classes` | one member per class over the first eight classes |
//! | W4 | `w4-small-page` / `w4-large-page` | `query`, `max_items` 2 against 64, continued to exhaustion |
//! | | `w4-multi-consumer` | the same query with a four-consumer schema |
//! | | `w4-abandon-after-one-page` | one page, then the cursor is dropped |
//! | W5 | `w5-sweep` | a whole-scope sweep over a store whose declared capacity is `tiny` or `roomy` |
//! | | `w5-round-trip` | ten further single-member recoveries against the same store |
//! | W6a | `w6a-export` | the whole-scope export at 1/2/4/6 workers, each in `discard`, `encode` or `write` |
//!
//! # The phases, and what "no double attribution" means here
//!
//! Every sample carries a [`Stages`] ledger of **top-level** phases measured as disjoint intervals
//! of one clock: `open`, `prepare`, `request`, `output`. [`Stages::phase`] refuses to start a phase
//! before the previous one ended, so two phases cannot be attributed the same instant; the sample
//! prints `window_micros` and `unattributed_micros` beside them, so the part of the run no phase
//! claimed is visible rather than absorbed. Stages that really are nested — the sink's own work
//! inside `request`, its encoding inside that, its file write inside *that* — are recorded as
//! [`Stages::nested`] against the phase that contains them and are **never** added to the phase
//! total; the verifier below checks both halves of that rule.
//!
//! The startup phase is deliberately **not** in the ledger: it is the process's own (exec, dyld,
//! test-harness init), and only the process the campaign wraps can see it. The registered reading is
//! `external wall − window_micros`, which is stated as the residual rather than folded into a phase.
//!
//! # Where the timings live
//!
//! Nowhere in the engine. The ledger is this file's own, it prints to stdout as a sidecar line, and
//! no report, stop record or fingerprint carries it: [`the_domain_reports_carry_no_phase_timing`]
//! reads the engine's sources for the ledger's names and the published documents for its keys, and
//! [`the_instrumentation_changes_no_domain_report`] shows the same result with the observation port
//! attached and without it.
//!
//! # Counts
//!
//! Three counted planes are reported separately, never mixed: the request's **usage** (the budget's
//! own charging), the **cache's residency and reuse** ([`FactsReport`]), and the process's **peak
//! RSS** (the campaign wrapper's, `null` when the sample is printed by a bare run). The demand-path
//! counters (`jarde::d0_counts`) and the bulk observation port (`jarde::bulk::BulkProbe`) are read
//! where this build has them (`--features test-support`) and are reported as `null` otherwise.

mod bulk_support;

use bulk_support::{
    DEFLATE, Recorder, STORE, container_roots, environment, fingerprint, limits, open, request,
    tree_scope, zip,
};
use jarde::*;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------------------------
// The fixture
// ---------------------------------------------------------------------------------------------

/// The classes the fixture carries beside the ones [`bulk_support`] already pins.
///
/// Every one is a committed `.class` file of this repository, read as bytes and never rewritten, so
/// the fixture is a total function of bytes the corpus fingerprint (`tests/fixtures/
/// corpus-fingerprint.json`) already fixes. The internal names are the file names — the property
/// [`bulk_support`]'s fixtures state and [`the_fixture_is_the_pinned_bytes`] checks — which is why
/// no two of them may collide: `p3-refused-cast/v8/Holder.class` is deliberately *not* here, because
/// `p3-declaration/v8/Holder.class` already carries that name.
const BOOLEAN_CONTEXTS: &[u8] =
    include_bytes!("fixtures/p3-boolean-contexts/v8/BooleanContexts.class");
const GUARDED: &[u8] = include_bytes!("fixtures/p3-handlers/v8/Guarded.class");
const CONCAT_CONVERSION: &[u8] =
    include_bytes!("fixtures/p3-concat-conversion/v8/ConcatConversion.class");
const INT_COMPARISONS: &[u8] =
    include_bytes!("fixtures/p3-int-comparisons/v8/IntComparisons.class");
const SLOT_TYPES: &[u8] = include_bytes!("fixtures/p3-parameter-slots/v8/SlotTypes.class");
const REQUIRED_CONVERSIONS: &[u8] =
    include_bytes!("fixtures/p3-required-conversions/v8/RequiredConversions.class");
const RECEIVER_GROUPING: &[u8] =
    include_bytes!("fixtures/p3-receiver-grouping/v8/ReceiverGrouping.class");
const HOISTED_BOOLEAN: &[u8] =
    include_bytes!("fixtures/p3-hoisted-boolean/v8/HoistedBoolean.class");
const LOCAL_REWRITE: &[u8] = include_bytes!("fixtures/p3-local-rewrite/v8/LocalRewrite.class");
const MOD_LIKE: &[u8] = include_bytes!("fixtures/p3-nested-arithmetic/v8/ModLike.class");
const NESTED_EVAL: &[u8] = include_bytes!("fixtures/p3-nested-eval/v8/NestedEval.class");
const ARRAY_TYPES: &[u8] = include_bytes!("fixtures/p3-array-types/v8/ArrayTypes.class");
const REFUSED_CAST: &[u8] = include_bytes!("fixtures/p3-refused-cast/v8/RefusedCast.class");
const RES: &[u8] = include_bytes!("fixtures/p3-handlers/v8/Res.class");
const CONCAT_JAVA_8: &[u8] = include_bytes!("fixtures/p4-modern/v8/ConcatJava8.class");
const CODE_ONLY: &[u8] = include_bytes!("fixtures/r2-annotation-positions/v8/CodeOnly.class");
const EXTERNAL: &[u8] = include_bytes!("fixtures/p3-refused-cast/v8/External.class");
const MISSING_DEPENDENCY: &[u8] =
    include_bytes!("fixtures/p3-corpus/v8-missing-dep/MissingDependency.class");
const FLAGS: &[u8] = include_bytes!("fixtures/p3-corpus/v8-g/Flags.class");

/// The prefix the fixture's nested container is declared under.
const NESTED_PREFIX: &[u8] = b"p/";

/// The classes at the fixture's root, in the order the scope walks them: the four `bulk_support`
/// pins and the historical ECJ sample, alternating STORED and DEFLATED as the corpus does.
fn fixture_root_entries() -> Vec<(&'static [u8], &'static [u8], u16)> {
    vec![
        (b"Scope.class", bulk_support::SCOPE, DEFLATE),
        (b"Shape.class", bulk_support::SHAPE, STORE),
        (b"LambdaSample.class", bulk_support::LAMBDA, DEFLATE),
        (b"Holder.class", bulk_support::HOLDER, STORE),
        (
            b"HistoricalControlFlow.class",
            bulk_support::HISTORICAL,
            DEFLATE,
        ),
    ]
}

/// The classes the fixture's nested container holds under [`NESTED_PREFIX`], in walk order.
fn fixture_nested_entries() -> Vec<(&'static [u8], &'static [u8], u16)> {
    vec![
        (b"p/BooleanContexts.class", BOOLEAN_CONTEXTS, DEFLATE),
        (b"p/Guarded.class", GUARDED, DEFLATE),
        (b"p/ConcatConversion.class", CONCAT_CONVERSION, STORE),
        (b"p/IntComparisons.class", INT_COMPARISONS, DEFLATE),
        (b"p/SlotTypes.class", SLOT_TYPES, DEFLATE),
        (b"p/RequiredConversions.class", REQUIRED_CONVERSIONS, STORE),
        (b"p/ReceiverGrouping.class", RECEIVER_GROUPING, DEFLATE),
        (b"p/HoistedBoolean.class", HOISTED_BOOLEAN, DEFLATE),
        (b"p/LocalRewrite.class", LOCAL_REWRITE, STORE),
        (b"p/ModLike.class", MOD_LIKE, DEFLATE),
        (b"p/NestedEval.class", NESTED_EVAL, DEFLATE),
        (b"p/ArrayTypes.class", ARRAY_TYPES, STORE),
        (b"p/RefusedCast.class", REFUSED_CAST, DEFLATE),
        (b"p/Res.class", RES, DEFLATE),
        (b"p/ConcatJava8.class", CONCAT_JAVA_8, STORE),
        (b"p/CodeOnly.class", CODE_ONLY, DEFLATE),
        (b"p/External.class", EXTERNAL, DEFLATE),
        (b"p/MissingDependency.class", MISSING_DEPENDENCY, STORE),
        (b"p/Flags.class", FLAGS, DEFLATE),
    ]
}

/// The fixture: one archive holding a flat half and a nested container, built in memory.
///
/// The nested container is there because the scope's own walk has to descend into it: a workload
/// that only ever reads the root container would not exercise the container a class task reads after
/// the walk has moved past it, which is the shape W5/W6a are about.
fn fixture() -> Vec<u8> {
    let nested = zip(&fixture_nested_entries());
    let mut entries = fixture_root_entries();
    entries.push((b"lib/more.jar", nested.as_slice(), STORE));
    zip(&entries)
}

/// The fixture's classes and their method bodies, as the pinned reading of it.
///
/// `PINNED_FIXTURE` is a digest of the built bytes: the workloads are fixed by *these* bytes, so a
/// later edit that adds a class, changes a compression method or reorders an entry moves it and the
/// verifier below fails rather than quietly measuring a different corpus.
const PINNED_FIXTURE: &str = "145f7903f52e16d087584477c2d30c413023547c16e6d168bea3c558b9030854";
/// The number of classes the fixture declares, and of method records a whole-scope sweep delivers
/// for it.
const PINNED_CLASSES: u64 = 24;
const PINNED_METHODS: u64 = 183;

/// The fixture's digest, as the pinned reading is compared against it.
fn fixture_digest(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

// ---------------------------------------------------------------------------------------------
// The stage ledger
// ---------------------------------------------------------------------------------------------

/// One top-level phase's reading.
#[derive(Clone, Debug, Eq, PartialEq)]
struct TopStage {
    name: &'static str,
    micros: u64,
}

/// One nested stage's reading: it happened *inside* `parent` and is never part of the phase total.
#[derive(Clone, Debug, Eq, PartialEq)]
struct NestedStage {
    name: &'static str,
    parent: &'static str,
    micros: u64,
}

/// One run's stages, measured on one clock.
///
/// Top-level phases are disjoint by construction: [`Stages::phase`] refuses to start one before the
/// previous ended. Nested stages are attached to a phase that has already been recorded, which is
/// how "inside `request`" is stated without the nesting becoming a second claim on the same time.
struct Stages {
    started: Instant,
    cursor: Instant,
    top: Vec<TopStage>,
    nested: Vec<NestedStage>,
}

impl Stages {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            started: now,
            cursor: now,
            top: Vec::new(),
            nested: Vec::new(),
        }
    }

    /// Runs `run` as the top-level phase `name`, and records its interval.
    ///
    /// The panic is the whole point of the type: a phase that begins before the previous one ended
    /// would attribute one instant to two stages, and this harness would then report a decomposition
    /// that cannot be true. It fails here instead.
    fn phase<R>(&mut self, name: &'static str, run: impl FnOnce() -> R) -> R {
        let started = Instant::now();
        assert!(
            started >= self.cursor,
            "phase `{name}` starts before the previous phase ended: `{}` closed at {:?} and this \
             one begins at {:?}",
            self.top.last().map_or("(none)", |stage| stage.name),
            self.cursor,
            started
        );
        let value = run();
        let ended = Instant::now();
        self.top.push(TopStage {
            name,
            micros: micros(ended.duration_since(started)),
        });
        self.cursor = ended;
        value
    }

    /// Records a stage that ran inside the top-level phase `parent`, which must already be closed.
    fn nested(&mut self, name: &'static str, parent: &'static str, micros: u64) {
        assert!(
            self.top.iter().any(|stage| stage.name == parent),
            "nested stage `{name}` names the parent `{parent}`, which no top-level phase of this run \
             recorded: the stages are {:?}",
            self.top.iter().map(|stage| stage.name).collect::<Vec<_>>()
        );
        self.nested.push(NestedStage {
            name,
            parent,
            micros,
        });
    }

    /// The whole window the ledger has measured: from the first phase's start to the last phase's
    /// end, so the figure is a function of what was recorded and not of when a reader asks.
    fn window_micros(&self) -> u64 {
        micros(self.cursor.duration_since(self.started))
    }

    /// The part of the window no top-level phase claimed.
    fn unattributed_micros(&self) -> u64 {
        self.window_micros()
            .saturating_sub(self.top.iter().map(|stage| stage.micros).sum())
    }

    /// What one nested stage's parent really took, when the ledger recorded that parent.
    fn parent_micros(&self, parent: &str) -> Option<u64> {
        self.top
            .iter()
            .find(|stage| stage.name == parent)
            .map(|stage| stage.micros)
    }

    /// What one top-level phase took, or zero when this run did not record it.
    ///
    /// W1 records `open`/`prepare`/`request`/`output` off the same clock; a sample that did not open
    /// a subject of its own records only the phases it really ran, and this answers zero for the
    /// others rather than a figure nobody measured.
    fn micros_of(&self, name: &str) -> u64 {
        self.top
            .iter()
            .find(|stage| stage.name == name)
            .map_or(0, |stage| stage.micros)
    }

    /// The nested stages of one parent, summed.
    fn nested_micros(&self, parent: &str) -> u64 {
        self.nested
            .iter()
            .filter(|stage| stage.parent == parent)
            .map(|stage| stage.micros)
            .sum()
    }

    fn top_value(&self) -> Value {
        Value::Array(
            self.top
                .iter()
                .map(|stage| json!({"name": stage.name, "micros": stage.micros}))
                .collect(),
        )
    }

    fn nested_value(&self) -> Value {
        Value::Array(
            self.nested
                .iter()
                .map(|stage| {
                    json!({"name": stage.name, "parent": stage.parent, "micros": stage.micros})
                })
                .collect(),
        )
    }
}

/// The whole microseconds of one duration, saturating: the unit every recorded reading uses.
fn micros(duration: Duration) -> u64 {
    u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
}

/// The whole nanoseconds of one duration, saturating.
///
/// Only the sink's own per-record accumulators use this: a callback that costs tens of nanoseconds
/// is a real cost of the ablation, and accumulating it in microseconds would round every one of them
/// to zero. The accumulator is converted once, where it is recorded.
fn nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

/// One accumulated nanosecond reading as microseconds, the unit the samples report.
fn as_micros(nanoseconds: u64) -> u64 {
    nanoseconds / 1_000
}

// ---------------------------------------------------------------------------------------------
// The sample
// ---------------------------------------------------------------------------------------------

/// One measured run, as the campaign reads it.
struct Sample {
    workload: &'static str,
    name: &'static str,
    corpus: String,
    config: String,
    instrumented: bool,
    stages: Stages,
    numbers: BTreeMap<&'static str, u64>,
    texts: BTreeMap<&'static str, String>,
    series: BTreeMap<&'static str, Vec<u64>>,
    usage: Option<Value>,
    bulk: Option<Value>,
    counts: Option<Value>,
    cache: Option<Value>,
    probe: Option<Value>,
    domain: String,
}

impl Sample {
    fn new(config: &Config, workload: &'static str, name: &'static str) -> Self {
        Self {
            workload,
            name,
            corpus: config.corpus(),
            config: config.describe(),
            instrumented: config.instrumented,
            stages: Stages::new(),
            numbers: BTreeMap::new(),
            texts: BTreeMap::new(),
            series: BTreeMap::new(),
            usage: None,
            bulk: None,
            counts: None,
            cache: None,
            probe: None,
            domain: String::new(),
        }
    }

    fn number(mut self, name: &'static str, value: u64) -> Self {
        self.numbers.insert(name, value);
        self
    }

    fn text(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.texts.insert(name, value.into());
        self
    }

    fn series(mut self, name: &'static str, values: Vec<u64>) -> Self {
        self.series.insert(name, values);
        self
    }

    fn usage(mut self, usage: &UsageSnapshot) -> Self {
        self.usage = Some(usage_value(usage));
        self
    }

    fn cache(mut self, report: &FactsReport) -> Self {
        self.cache = Some(json!({
            "entries": report.entries,
            "containers": report.containers,
            "retained_bytes": report.retained_bytes,
            "consultations": report.consultations,
            "hits": report.hits,
            "misses": report.misses,
            "stored": report.stored,
            "refused_capacity": report.refused_capacity,
            "refused_capacity_bytes": report.refused_capacity_bytes,
            "container_consultations": report.container_consultations,
            "container_hits": report.container_hits,
            "container_misses": report.container_misses,
            "container_stored": report.container_stored,
            "directory_parses": report.directory_parses,
            "nested_materializations": report.nested_materializations,
            "nested_materialized_bytes": report.nested_materialized_bytes,
        }));
        self
    }

    fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = domain.into();
        self
    }

    fn print(&self) {
        let value = json!({
            "harness": "p5_optimize_workloads",
            "workload": self.workload,
            "sample": self.name,
            "corpus": self.corpus,
            "config": self.config,
            "instrumented": self.instrumented,
            "window_micros": self.stages.window_micros(),
            "phases": self.stages.top_value(),
            "unattributed_micros": self.stages.unattributed_micros(),
            "nested": self.stages.nested_value(),
            "numbers": self.numbers,
            "texts": self.texts,
            "series": self.series,
            "usage": self.usage,
            "bulk": self.bulk,
            "counts": self.counts,
            "cache": self.cache,
            "probe": self.probe,
            "domain": self.domain,
        });
        println!(
            "{}",
            serde_json::to_string(&value).expect("a sample renders as JSON")
        );
    }
}

/// One request's usage, as the counted dimensions the protocol names.
fn usage_value(usage: &UsageSnapshot) -> Value {
    json!({
        "input_bytes": usage.input_bytes,
        "archive_entries": usage.archive_entries,
        "entry_bytes": usage.entry_bytes,
        "read_bytes": usage.read_bytes,
        "class_bytes": usage.class_bytes,
        "attribute_bytes": usage.attribute_bytes,
        "code_bytes": usage.code_bytes,
        "result_items": usage.result_items,
        "output_bytes": usage.output_bytes,
        "class_headers": usage.class_headers,
        "method_bodies": usage.method_bodies,
        "ir_items": usage.ir_items,
        "analysis_steps": usage.analysis_steps,
        "elapsed_millis": usage.elapsed_millis,
    })
}

// ---------------------------------------------------------------------------------------------
// Configuration: one process, one workload, one configuration
// ---------------------------------------------------------------------------------------------

/// What the sink does with every record the operation publishes — the three ablation steps of task
/// 1.3. They are one program on purpose: comparing across programs would put a second build and a
/// second set of process starts into the measurement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    /// Count the record and answer: neither the library's encoding nor any file write happens.
    Discard,
    /// Serialize the record exactly as the CLI's stream serializes it, then drop it.
    Encode,
    /// Serialize it and append the line to a scratch file under cargo's own test temporary
    /// directory (`CARGO_TARGET_TMPDIR`, inside the repository's build output — never `/tmp`).
    Write,
}

impl Mode {
    fn parse(text: &str) -> Self {
        match text {
            "encode" => Self::Encode,
            "write" => Self::Write,
            _ => Self::Discard,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Discard => "discard",
            Self::Encode => "encode",
            Self::Write => "write",
        }
    }
}

/// The declared facts-store capacity a W5/W6a sample runs with.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Capacity {
    /// No capacity at all (`FactsCapacity::none()`): every consultation is a miss and nothing is
    /// retained, so this arm is the direct path with the store attached and answering nothing.
    None,
    /// One entry and 4 KiB: smaller than any fixture, so the store answers almost nothing.
    Tiny,
    /// The host's own default declaration (`1<<14` entries, `1<<27` bytes).
    Roomy,
    /// An entry bound of `entries` with a byte bound far above the fixture's own weight: the
    /// reuse-distance arm. The store refuses past the bound instead of evicting, so this is the
    /// "how far back does a retained answer reach" knob of the O5 investigation.
    Retained { entries: usize },
}

impl Capacity {
    fn parse(text: &str) -> Self {
        match text {
            "none" => Self::None,
            "tiny" => Self::Tiny,
            "roomy" => Self::Roomy,
            other => match other.strip_prefix('e').and_then(|count| count.parse().ok()) {
                Some(entries) => Self::Retained { entries },
                None => panic!(
                    "`{other}` is not a capacity: `none`, `tiny`, `roomy` or `e<entries>` \
                     (`e4` is at most four retained facts) are the ones this harness declares"
                ),
            },
        }
    }

    fn name(self) -> String {
        match self {
            Self::None => "none".to_owned(),
            Self::Tiny => "tiny".to_owned(),
            Self::Roomy => "roomy".to_owned(),
            Self::Retained { entries } => format!("e{entries}"),
        }
    }

    fn capacity(self) -> FactsCapacity {
        match self {
            Self::None => FactsCapacity::none(),
            Self::Tiny => FactsCapacity::new(1, 4096),
            Self::Roomy => FactsCapacity::new(1 << 14, 1 << 27),
            // The byte bound is deliberately far above the fixture's own weight: this arm measures
            // the entry bound, so a byte refusal would be a second cause in the same reading.
            Self::Retained { entries } => FactsCapacity::new(entries, 1 << 27),
        }
    }
}

/// The variables the campaign sets:
/// `JARDE_OPTIMIZE_ARTIFACT`, `JARDE_OPTIMIZE_WORKLOAD`, `JARDE_OPTIMIZE_WORKERS`,
/// `JARDE_OPTIMIZE_SINK`, `JARDE_OPTIMIZE_CAPACITY`, `JARDE_OPTIMIZE_INSTRUMENT`,
/// `JARDE_OPTIMIZE_STOP`.
struct Config {
    workload: String,
    artifact: Option<PathBuf>,
    workers: usize,
    mode: Mode,
    capacity: Capacity,
    instrumented: bool,
    /// W6a only: answer [`SinkControl::Stop`] once this many method records were delivered. `None`
    /// is the ordinary run that takes the whole stream, so a cancellation sample is one variable
    /// of one configuration rather than a second harness.
    stop_after: Option<u64>,
}

impl Config {
    fn from_env() -> Self {
        let variable = |name: &str| std::env::var(name).ok().filter(|value| !value.is_empty());
        Self {
            workload: variable("JARDE_OPTIMIZE_WORKLOAD").unwrap_or_else(|| "w1".to_owned()),
            artifact: variable("JARDE_OPTIMIZE_ARTIFACT").map(PathBuf::from),
            workers: variable("JARDE_OPTIMIZE_WORKERS")
                .and_then(|text| text.parse().ok())
                .unwrap_or(1),
            mode: variable("JARDE_OPTIMIZE_SINK")
                .map(|text| Mode::parse(&text))
                .unwrap_or(Mode::Discard),
            capacity: variable("JARDE_OPTIMIZE_CAPACITY")
                .map(|text| Capacity::parse(&text))
                .unwrap_or(Capacity::Roomy),
            instrumented: variable("JARDE_OPTIMIZE_INSTRUMENT").as_deref() != Some("off"),
            stop_after: variable("JARDE_OPTIMIZE_STOP").and_then(|text| text.parse().ok()),
        }
    }

    /// The corpus label a sample carries: the artifact's file name, or the in-repo fixture.
    fn corpus(&self) -> String {
        match &self.artifact {
            Some(path) => path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.display().to_string()),
            None => "tests/fixtures (in-process fixture)".to_owned(),
        }
    }

    /// The configuration a sample carries, so two samples can be told apart without the command.
    fn describe(&self) -> String {
        format!(
            "workload={} corpus={} workers={} sink={} capacity={} instrument={}",
            self.workload,
            self.corpus(),
            self.workers,
            self.mode.name(),
            self.capacity.name(),
            if self.instrumented { "on" } else { "off" }
        )
    }

    /// The input one run reads, and the bytes behind it.
    fn input(&self) -> (ArtifactInput, Vec<u8>) {
        match &self.artifact {
            Some(path) => {
                let bytes = std::fs::read(path)
                    .unwrap_or_else(|error| panic!("{} is readable: {error}", path.display()));
                (ArtifactInput::Path(path.clone()), bytes)
            }
            None => {
                let bytes = fixture();
                (ArtifactInput::bytes(bytes.clone()), bytes)
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The measured sink
// ---------------------------------------------------------------------------------------------

/// A sink that keeps the stream's semantic projection and pays exactly the layer its mode names.
///
/// The projection is [`bulk_support::Recorder`]'s — the same one the bulk change's own gates compare
/// runs through — so a W6a sample's fingerprint is comparable with those gates' readings rather than
/// being this harness's private opinion of what a result is.
struct Measuring {
    recorder: Recorder,
    mode: Mode,
    destination: Option<(PathBuf, std::fs::File)>,
    records: u64,
    methods_seen: u64,
    stopped: bool,
    stop_after: Option<u64>,
    encoded_bytes: u64,
    written_bytes: u64,
    visit_nanos: u64,
    encode_nanos: u64,
    write_nanos: u64,
    first_method: Option<Instant>,
}

/// How many scratch files this process has opened: one run per configuration, and a second `write`
/// sample in the same process must not overwrite the first one's file.
static SCRATCH_FILES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl Measuring {
    fn new(mode: Mode, stop_after: Option<u64>) -> Self {
        let destination = (mode == Mode::Write).then(|| {
            let ordinal = SCRATCH_FILES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
                "jarde-p5-optimize-{}-{ordinal}.jsonl",
                std::process::id()
            ));
            let file = std::fs::File::create(&path)
                .unwrap_or_else(|error| panic!("{} is writable: {error}", path.display()));
            (path, file)
        });
        Self {
            recorder: Recorder::new(),
            mode,
            destination,
            records: 0,
            methods_seen: 0,
            stopped: false,
            stop_after,
            encoded_bytes: 0,
            written_bytes: 0,
            visit_nanos: 0,
            encode_nanos: 0,
            write_nanos: 0,
            first_method: None,
        }
    }

    /// One record's own layer: the count, then the encoding, then the write, each measured where it
    /// happens so the three can be reported as one ablation rather than as one number.
    fn account<T: serde::Serialize>(&mut self, record: &T) -> Result<()> {
        let visit = Instant::now();
        self.records = self.records.saturating_add(1);
        self.visit_nanos = self.visit_nanos.saturating_add(nanos(visit.elapsed()));
        if self.mode == Mode::Discard {
            return Ok(());
        }
        let encode = Instant::now();
        let line = serde_json::to_vec(record).map_err(|error| Error::Io {
            operation: "p5_optimize_encode".to_owned(),
            message: error.to_string(),
        })?;
        self.encoded_bytes = self.encoded_bytes.saturating_add(line.len() as u64);
        self.encode_nanos = self.encode_nanos.saturating_add(nanos(encode.elapsed()));
        if let Some((_, file)) = self.destination.as_mut() {
            let write = Instant::now();
            file.write_all(&line).map_err(|error| Error::Io {
                operation: "p5_optimize_write".to_owned(),
                message: error.to_string(),
            })?;
            file.write_all(b"\n").map_err(|error| Error::Io {
                operation: "p5_optimize_write".to_owned(),
                message: error.to_string(),
            })?;
            self.written_bytes = self.written_bytes.saturating_add(line.len() as u64 + 1);
            self.write_nanos = self.write_nanos.saturating_add(nanos(write.elapsed()));
        }
        Ok(())
    }

    /// Removes the scratch file this sink wrote, and answers what it held.
    fn finish(&mut self) -> u64 {
        let Some((path, _)) = self.destination.take() else {
            return 0;
        };
        let size = std::fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
        std::fs::remove_file(&path)
            .unwrap_or_else(|error| panic!("{} is removable: {error}", path.display()));
        size
    }

    /// The sink's answer, with this run's own stop folded in.
    ///
    /// A configuration that declared `stop=<n>` really answers [`SinkControl::Stop`] once `n` method
    /// records were delivered, which is what makes a cancellation sample a reading of the operation
    /// rather than of the harness's bookkeeping: the callback is the only place a consumer can stop
    /// the stream, and the record the callback refused is not counted as delivered.
    fn with_stop(&mut self, answer: Result<SinkControl>) -> Result<SinkControl> {
        let answer = answer?;
        if self
            .stop_after
            .is_some_and(|limit| self.methods_seen >= limit)
        {
            self.stopped = true;
            Ok(SinkControl::Stop)
        } else {
            Ok(answer)
        }
    }

    /// Where the path a `write` sink used lives — stated, not assumed, by the verifier.
    fn scratch_root() -> &'static str {
        env!("CARGO_TARGET_TMPDIR")
    }
}

impl RecoverySink for Measuring {
    fn header(
        &mut self,
        event: &BulkHeaderEvent,
        delivery: DeliveryAccount,
    ) -> Result<SinkControl> {
        // The recorder's own `header` is a getter; the sink callback is the trait method, and the
        // qualification is what keeps the two apart.
        let answer = RecoverySink::header(&mut self.recorder, event, delivery);
        self.account(event)?;
        self.with_stop(answer)
    }

    fn class_prepared(&mut self, event: &ClassPreparedEvent) -> Result<SinkControl> {
        let answer = self.recorder.class_prepared(event);
        self.account(event)?;
        self.with_stop(answer)
    }

    fn method(&mut self, event: &MethodResultEvent) -> Result<SinkControl> {
        if self.first_method.is_none() {
            self.first_method = Some(Instant::now());
        }
        let answer = self.recorder.method(event);
        self.account(event)?;
        self.methods_seen = self.methods_seen.saturating_add(1);
        self.with_stop(answer)
    }

    fn class_end(&mut self, event: &ClassEndEvent) -> Result<SinkControl> {
        let answer = self.recorder.class_end(event);
        self.account(event)?;
        self.with_stop(answer)
    }

    fn diagnostic(&mut self, event: &BulkDiagnosticEvent) -> Result<SinkControl> {
        let answer = self.recorder.diagnostic(event);
        self.account(event)?;
        self.with_stop(answer)
    }

    fn final_event(&mut self, event: &BulkFinalEvent) -> Result<SinkControl> {
        let answer = RecoverySink::final_event(&mut self.recorder, event);
        self.account(event)?;
        self.with_stop(answer)
    }
}

// ---------------------------------------------------------------------------------------------
// Domain fingerprints
// ---------------------------------------------------------------------------------------------

/// Removes every `elapsed_millis` from a document, and answers how many it removed.
///
/// This is `p5_benchmark`'s rule, restated here because a test target cannot call another target's
/// helper: a run's clock is a reading of the run, not a result, and it is the *only* field the
/// complete-result comparison of two configurations is allowed to drop.
fn strip_elapsed(value: &mut Value) -> usize {
    let mut removed = 0;
    match value {
        Value::Object(map) => {
            if map.remove("elapsed_millis").is_some() {
                removed += 1;
            }
            for (_, child) in map.iter_mut() {
                removed += strip_elapsed(child);
            }
        }
        Value::Array(entries) => {
            for child in entries.iter_mut() {
                removed += strip_elapsed(child);
            }
        }
        _ => {}
    }
    removed
}

/// The digest of a report with its own clock removed: what "the same result" means here.
fn domain_of<T: serde::Serialize>(report: &T) -> String {
    let mut document = serde_json::to_value(report).expect("a report serializes");
    let removed = strip_elapsed(&mut document);
    assert!(
        removed > 0,
        "the report carries no `elapsed_millis` at all, so this fingerprint would not be the \
         complete-result one the corpus gates compare: {document}"
    );
    format!(
        "blake3:{}",
        blake3::hash(
            serde_json::to_string(&document)
                .expect("a JSON value renders")
                .as_bytes()
        )
        .to_hex()
    )
}

/// The digest of a report with its own clock **and** its charge records removed.
///
/// This is the form two configurations are compared through when one of them is allowed to move the
/// resource readings and is not allowed to move the result: a store that answers from retention
/// charges fewer reads, and that difference is a reading of the run rather than a difference in what
/// was presented. The rule is `p5_facts_cache.rs::without_charges`'s: only `usage` objects are
/// dropped; statuses, termination reasons, evidence, coverage and diagnostics all stay.
fn result_domain_of<T: serde::Serialize>(report: &T) -> String {
    let mut document = serde_json::to_value(report).expect("a report serializes");
    let removed = strip_charges(&mut document);
    assert!(
        removed > 0,
        "the report carries no charge record, so this comparison would be its own document: \
         {document}"
    );
    strip_elapsed(&mut document);
    format!(
        "blake3:{}",
        blake3::hash(
            serde_json::to_string(&document)
                .expect("a JSON value renders")
                .as_bytes()
        )
        .to_hex()
    )
}

/// Removes every `usage` object from a document, and answers how many it removed.
fn strip_charges(value: &mut Value) -> usize {
    let mut removed = 0;
    match value {
        Value::Object(map) => {
            if map.remove("usage").is_some() {
                removed += 1;
            }
            for (_, child) in map.iter_mut() {
                removed += strip_charges(child);
            }
        }
        Value::Array(entries) => {
            for child in entries.iter_mut() {
                removed += strip_charges(child);
            }
        }
        _ => {}
    }
    removed
}

/// The digest of one bulk stream's method records: order, identity, disposition, text and the
/// record's own semantic fingerprint, with every resource reading left out.
fn stream_domain(recorder: &Recorder) -> (String, u64) {
    let methods = recorder.methods();
    let mut hasher = blake3::Hasher::new();
    for record in &methods {
        hasher.update(&record.class_ordinal.to_be_bytes());
        hasher.update(&record.member_ordinal.to_be_bytes());
        hasher.update(format!("{:?}", record.method).as_bytes());
        hasher.update(b"\x00");
        hasher.update(format!("{:?}", record.outcome).as_bytes());
        hasher.update(b"\x00");
        hasher.update(record.text.as_deref().unwrap_or("").as_bytes());
        hasher.update(b"\x00");
        hasher.update(format!("{:?}", record.fingerprint).as_bytes());
        hasher.update(b"\x00");
    }
    (
        format!("blake3:{}", hasher.finalize().to_hex()),
        u64::try_from(methods.len()).unwrap_or(u64::MAX),
    )
}

/// The identities the items of a query page present, in the order the page presents them.
///
/// The projection is what a *result* is here: the relation, the target, the consumer category that
/// produced it and the operation it states. It deliberately reads no usage and no clock, so a page
/// produced under one accounting is comparable with a page produced under another.
fn item_lines(items: &[XrefItem]) -> Vec<String> {
    items
        .iter()
        .map(|item| {
            format!(
                "{:?}|{:?}|{:?}|{:?}",
                item.relation, item.target, item.consumer, item.operation
            )
        })
        .collect()
}

/// The digest of a list of lines: the page-set comparison of W4.
fn lines_domain(lines: &[String]) -> String {
    let mut hasher = blake3::Hasher::new();
    for line in lines {
        hasher.update(line.as_bytes());
        hasher.update(b"\x00");
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

// ---------------------------------------------------------------------------------------------
// Opening a subject
// ---------------------------------------------------------------------------------------------

/// One opened input, with the phases that produced it already recorded in the sample's ledger.
struct Opened {
    engine: Engine,
    snapshot: ArtifactSnapshot,
    scope: PhysicalScope,
    roots: Vec<LoadRoot>,
    classes: Vec<JvmBytes>,
    store: FactsCache,
    open_usage: UsageSnapshot,
    prepare_usage: UsageSnapshot,
}

/// Opens the input and prepares the workload's own targets, as the two phases `open` and `prepare`.
///
/// `prepare` is the caller's discovery: the load roots the scope's class positions are declared at,
/// the class-declaration listing, and nothing else. It is deliberately *outside* the request phase:
/// a whole-scope request's total must not absorb the target discovery that preceded it.
fn open_subject(config: &Config, sample: &mut Sample, capacity: Capacity) -> Opened {
    let (input, _bytes) = config.input();
    open_input(config, sample, capacity, input)
}

/// The same, over bytes the workload built itself rather than over the configuration's input.
///
/// The damaged-suffix samples of W4 need an archive that is *almost* the fixture: same entries, same
/// order, one entry's data byte flipped. Building it here keeps every other reading of the fixture
/// the one the pin fixes, and states the difference as the sample's own subject.
fn open_bytes(config: &Config, sample: &mut Sample, capacity: Capacity, bytes: Vec<u8>) -> Opened {
    open_input(config, sample, capacity, ArtifactInput::bytes(bytes))
}

fn open_input(
    config: &Config,
    sample: &mut Sample,
    capacity: Capacity,
    input: ArtifactInput,
) -> Opened {
    let store = FactsCache::current(capacity.capacity());
    let prefixes: Vec<&[u8]> = if config.artifact.is_none() {
        vec![b"", NESTED_PREFIX]
    } else {
        // An external artifact's classes are declared at its root container's own root: the
        // campaign's corpora are plain jars, and a WAR would need its own prefix list, which is a
        // declaration this harness does not guess.
        vec![b""]
    };
    let engine = Engine::new();
    let mut open_budget = Budget::new(limits_for(config)).with_facts_cache(store.clone());
    let snapshot = sample.stages.phase("open", || {
        engine
            .open(input, &mut open_budget)
            .expect("the input opens")
    });
    let open_usage = open_budget.usage();

    let mut prepare_budget = Budget::new(limits_for(config)).with_facts_cache(store.clone());
    let scope = tree_scope();
    let (roots, classes) = sample.stages.phase("prepare", || {
        let roots = container_roots(&snapshot, &mut prepare_budget, &prefixes);
        let listing = engine
            .list_class_declarations(&snapshot, &scope, &mut prepare_budget)
            .expect("the scope's declarations are listable");
        let classes = listing
            .items
            .iter()
            .map(|item| item.declaration.this_class.raw().clone())
            .collect::<Vec<_>>();
        (roots, classes)
    });
    let prepare_usage = prepare_budget.usage();
    Opened {
        engine,
        snapshot,
        scope,
        roots,
        classes,
        store,
        open_usage,
        prepare_usage,
    }
}

/// The counted ceilings one sample runs under.
///
/// The fixture is a small, fully enumerated scope: the `bulk_support` limits hold a whole sweep of
/// it, and the fixture's own pin says so. A **corpus** is not one request's view — `bcprov`'s
/// 15,003 bodies cost more `ir_items` than any per-request ceiling — so an external artifact runs
/// under the ceilings the CLI's own defaults and `examples/bulk_scope_sweep` declare: the counted
/// dimensions raised to a value a real scope fits, and an hour of wall clock. A stop in a corpus
/// sample would otherwise be a limit of *this harness* read as a property of the run.
fn limits_for(config: &Config) -> Limits {
    if config.artifact.is_none() {
        return limits();
    }
    let overrides = [
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
    ]
    .into_iter()
    .map(|dimension| BudgetOverride::new(dimension, 1 << 40))
    .chain([BudgetOverride::new("elapsed_millis", 3_600_000)])
    .collect::<Result<Vec<_>>>()
    .expect("every override names a counted dimension");
    task_limits(&overrides).expect("the ceilings are the declared dimensions' own")
}

/// One request's own budget: a fresh quota over the same store, as the host's own requests have.
fn request_budget(config: &Config, store: &FactsCache) -> Budget {
    Budget::new(limits_for(config)).with_facts_cache(store.clone())
}

// The demand-path counting port (`jarde::d0_counts`): a build without `test-support` has no reading
// API at all, so the samples of such a build carry `counts: null` and the counters below compile to
// nothing. The call sites stay the same in both builds, which is what "the instrumentation is
// compiled out, not branched around" means here.
#[cfg(feature = "test-support")]
type Counts = jarde::d0_counts::Counts;
#[cfg(not(feature = "test-support"))]
type Counts = ();

#[cfg(feature = "test-support")]
fn counts_before() -> Counts {
    jarde::d0_counts::snapshot()
}

#[cfg(not(feature = "test-support"))]
fn counts_before() -> Counts {}

#[cfg(feature = "test-support")]
fn counts_after(before: Counts) -> Option<Value> {
    let delta = before.since(jarde::d0_counts::snapshot());
    Some(json!({
        "class_materializations": delta.class_materializations,
        "class_preparations": delta.class_preparations,
        "body_decodes": delta.body_decodes,
        "recovery_runs": delta.recovery_runs,
        "owned_records": delta.owned_records,
        "read_detail_records": delta.read_detail_records,
    }))
}

#[cfg(not(feature = "test-support"))]
fn counts_after(_before: Counts) -> Option<Value> {
    None
}

/// The counted readings as plain numbers too.
///
/// The `counts` document is the whole reading, and the campaign's summary prints `numbers`; mirroring
/// the six counters here is what puts "how much of the path was preparation, and how much was
/// decode" beside the durations in a campaign table instead of leaving it inside a nested document.
/// A build without the port has no document and mirrors nothing.
fn count_numbers(sample: &mut Sample) {
    let Some(document) = sample.counts.clone() else {
        return;
    };
    for (field, name) in [
        ("class_materializations", "count_class_materializations"),
        ("class_preparations", "count_class_preparations"),
        ("body_decodes", "count_body_decodes"),
        ("recovery_runs", "count_recovery_runs"),
        ("owned_records", "count_owned_records"),
        ("read_detail_records", "count_read_detail_records"),
    ] {
        if let Some(value) = document.get(field).and_then(Value::as_u64) {
            sample.numbers.insert(name, value);
        }
    }
}

/// A performed outcome, with the harness's own statement that an identity cannot be ambiguous.
fn performed<T>(outcome: OperationOutcome<T>, what: &str) -> T {
    match outcome {
        OperationOutcome::Performed(value) => value,
        OperationOutcome::Ambiguous(_) => {
            panic!("{what} was reported ambiguous, and an identity cannot be")
        }
        OperationOutcome::Incomplete(_) => {
            panic!("{what} was reported incomplete, and an identity cannot be")
        }
    }
}

/// One class of the scope, with the members that carry a body.
struct Target {
    class: JvmBytes,
    definition: PhysicalDefinitionId,
    methods: Vec<PhysicalMethodId>,
}

/// The first `count` classes of a prepared subject, each with the members that carry a body.
///
/// The members are read here, in `prepare`, for the same reason the roots are: a workload that needs
/// to know which member to ask about is stating its own target, and that statement is not part of
/// the request it measures.
fn targets(config: &Config, opened: &Opened, count: usize) -> Vec<Target> {
    let mut targets = Vec::new();
    for class in opened.classes.iter().take(count) {
        let mut budget = request_budget(config, &opened.store);
        let request = ClassViewRequest {
            class: ClassRef::Name {
                class: ClassNameQuery::internal(String::from_utf8_lossy(&class.0).into_owned()),
            },
            bodies: Vec::new(),
        };
        let view = performed(
            opened
                .engine
                .class_view(&opened.snapshot, &opened.scope, &request, &mut budget)
                .expect("a listed class is viewable"),
            "a class declaration",
        );
        let methods = view
            .methods()
            .filter(|method| matches!(method.body, MemberBodyEvidence::CodeAttribute { .. }))
            .map(|method| method.identity.clone())
            .collect::<Vec<_>>();
        targets.push(Target {
            class: class.clone(),
            definition: view.class.clone(),
            methods,
        });
    }
    targets
}

/// The first member of the first classes that carry a body.
fn first_member(config: &Config, opened: &Opened) -> (JvmBytes, PhysicalMethodId) {
    for target in targets(config, opened, 8) {
        if let Some(member) = target.methods.first() {
            return (target.class.clone(), member.clone());
        }
    }
    panic!("no class of this fixture declares a member with a body");
}

// ---------------------------------------------------------------------------------------------
// W1 — one cold single-method request, in a new process and in the library
// ---------------------------------------------------------------------------------------------

fn w1(config: &Config) -> Vec<Sample> {
    // The cold sample: this process opens the input and asks for one method. The campaign runs the
    // whole workload in a fresh process, so "cold" is the process's own word and not a label.
    let mut cold = Sample::new(config, "W1", "w1-cold-method");
    let opened = open_subject(config, &mut cold, Capacity::Roomy);
    let (_, member) = first_member(config, &opened);
    let subject_environment =
        environment(&opened.snapshot, opened.scope.clone(), opened.roots.clone());
    cold = recover_one(
        config,
        cold,
        &opened,
        &member,
        &subject_environment,
        Some(counts_before()),
    );
    let cold_domain = cold.domain.clone();

    // The control: the same member again, on the same snapshot and through the same store.
    let mut second = Sample::new(config, "W1", "w1-second-request");
    second = recover_one(
        config,
        second,
        &opened,
        &member,
        &subject_environment,
        Some(counts_before()),
    );

    // The library-level control: a second snapshot of the same input, in the same process.
    let mut third = Sample::new(config, "W1", "w1-second-snapshot");
    let reopened = open_subject(config, &mut third, Capacity::Roomy);
    let (_, member) = first_member(config, &reopened);
    let reopened_environment = environment(
        &reopened.snapshot,
        reopened.scope.clone(),
        reopened.roots.clone(),
    );
    third = recover_one(
        config,
        third,
        &reopened,
        &member,
        &reopened_environment,
        Some(counts_before()),
    );

    // The class-level presentation of the same class: one `class_source` request over the first
    // class the scope declares, which is the entry the design's own §15 names as the one that had a
    // bind + prepare double read. It runs here, not in a workload of its own, because the reading is
    // about the same class the three single-method samples above ask about.
    let mut source = Sample::new(config, "W1", "w1-class-source");
    let class = opened
        .classes
        .first()
        .expect("the fixture declares a class")
        .clone();
    let mut source_budget = request_budget(config, &opened.store);
    let source_request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(String::from_utf8_lossy(&class.0).into_owned()),
        },
        environment: subject_environment.clone(),
    };
    let before = counts_before();
    let (text_bytes, methods, declaration_present) = source.stages.phase("request", || {
        let report = performed(
            opened
                .engine
                .class_source(
                    std::slice::from_ref(&opened.snapshot),
                    &source_request,
                    &mut source_budget,
                )
                .expect("a declared class has source"),
            "a class declaration",
        );
        (
            report.text.len() as u64,
            report.methods.len() as u64,
            u64::from(report.declaration.is_some()),
        )
    });
    source.counts = counts_after(before);
    count_numbers(&mut source);
    let source = source
        .number("class", 1)
        .number("methods", methods)
        .number("text_bytes", text_bytes)
        .number("declaration_present", declaration_present)
        .usage(&source_budget.usage())
        .cache(&opened.store.report());

    let (cold_open_micros, cold_prepare_micros, cold_output_micros) = (
        cold.stages.micros_of("open"),
        cold.stages.micros_of("prepare"),
        cold.stages.micros_of("output"),
    );
    vec![
        cold.number(
            "result_equals_second_snapshot",
            u64::from(third.domain == cold_domain),
        )
        .number("open_usage_class_headers", opened.open_usage.class_headers)
        .number(
            "prepare_usage_class_headers",
            opened.prepare_usage.class_headers,
        )
        .number("open_micros", cold_open_micros)
        .number("prepare_micros", cold_prepare_micros)
        .number("output_micros", cold_output_micros),
        second,
        third,
        source,
    ]
}

/// One single-member recovery over `opened`, recorded as a `request` phase and an `output` phase.
fn recover_one(
    config: &Config,
    mut sample: Sample,
    opened: &Opened,
    member: &PhysicalMethodId,
    environment: &EnvironmentRequest,
    counts_before: Option<Counts>,
) -> Sample {
    let request = MethodOperationRequest {
        method: MethodRef::Method {
            method: member.clone(),
        },
        environment: environment.clone(),
    };
    let mut budget = request_budget(config, &opened.store);
    let request_started = Instant::now();
    let recovered = sample.stages.phase("request", || {
        performed(
            opened
                .engine
                .recover_target_with_evidence(
                    std::slice::from_ref(&opened.snapshot),
                    &request,
                    &RecoveryEvidenceRequest::essential(),
                    &mut budget,
                )
                .expect("a listed member is recoverable"),
            "a member identity",
        )
    });
    let first_result = micros(request_started.elapsed());
    let usage = budget.usage();
    let domain = sample.stages.phase("output", || domain_of(&recovered));
    sample = sample
        .number("first_result_micros", first_result)
        .number(
            "text_bytes",
            recovered.recovered.recovery().text.len() as u64,
        )
        .usage(&usage)
        .cache(&opened.store.report())
        .domain(domain);
    if let Some(before) = counts_before {
        sample.counts = counts_after(before);
        count_numbers(&mut sample);
    }
    sample
}

// ---------------------------------------------------------------------------------------------
// W2 — successive navigation on one snapshot
// ---------------------------------------------------------------------------------------------

fn w2(config: &Config) -> Vec<Sample> {
    let mut sample = Sample::new(config, "W2", "w2-navigation-sequence");
    let opened = open_subject(config, &mut sample, Capacity::Roomy);
    let targets = targets(config, &opened, 8);
    let mut per_request = Vec::new();
    let mut class_headers = Vec::new();
    let mut body_decodes = Vec::new();
    let mut views = Vec::new();
    let sequence_started = Instant::now();
    // The counted baseline is taken **before** the sequence: a baseline taken after it would make
    // every counter read as zero, and W2's own question is what eight successive navigations did to
    // the classes they asked about.
    let before = counts_before();
    let mut control = 0_u64;
    let mut control_views = Vec::new();
    sample.stages.phase("request", || {
        for (index, target) in targets.iter().enumerate() {
            let mut budget = request_budget(config, &opened.store);
            let request = ClassViewRequest {
                class: ClassRef::Definition {
                    definition: target.definition.clone(),
                },
                bodies: Vec::new(),
            };
            let started = Instant::now();
            let view = performed(
                opened
                    .engine
                    .class_view(&opened.snapshot, &opened.scope, &request, &mut budget)
                    .expect("a declared class is viewable"),
                "a class declaration",
            );
            per_request.push(micros(started.elapsed()));
            class_headers.push(budget.usage().class_headers);
            body_decodes.push(budget.usage().method_bodies);
            views.push(view);
            if index == 0 {
                // The control: the same class again, after the rest of the sequence.
                let mut budget = request_budget(config, &opened.store);
                let started = Instant::now();
                let view = performed(
                    opened
                        .engine
                        .class_view(&opened.snapshot, &opened.scope, &request, &mut budget)
                        .expect("a declared class is viewable again"),
                    "a class declaration",
                );
                control = micros(started.elapsed());
                control_views.push(view);
            }
        }
    });
    let sequence = micros(sequence_started.elapsed());
    let methods = targets
        .iter()
        .map(|target| target.methods.len() as u64)
        .sum::<u64>();
    // The output phase is the caller's own rendering of what came back: the views are serialized
    // there, so their encoding is not charged to the request that produced them.
    let (returned_bytes, sequence_domain) = sample.stages.phase("output", || {
        let mut bytes = 0_u64;
        let mut lines = Vec::new();
        for view in views.iter().chain(control_views.iter()) {
            bytes = bytes
                .saturating_add(serde_json::to_vec(view).map_or(0, |encoded| encoded.len() as u64));
            lines.push(domain_of(view));
        }
        (bytes, lines_domain(&lines))
    });
    sample.counts = counts_after(before);
    count_numbers(&mut sample);
    vec![
        sample
            .number("classes", targets.len() as u64)
            .number("method_bodies_declared", methods)
            .number("sequence_micros", sequence)
            .number("control_repeat_micros", control)
            .number("class_headers_total", class_headers.iter().sum())
            .number("method_bodies_total", body_decodes.iter().sum())
            .number("returned_bytes", returned_bytes)
            .series("per_request_micros", per_request)
            .series("per_request_class_headers", class_headers)
            .series("per_request_method_bodies", body_decodes)
            .text("sequence_domain", sequence_domain)
            .cache(&opened.store.report()),
    ]
}

// ---------------------------------------------------------------------------------------------
// W3 — several methods of one class, then a fixed set across classes
// ---------------------------------------------------------------------------------------------

fn w3(config: &Config) -> Vec<Sample> {
    let mut samples = Vec::new();
    let mut sample = Sample::new(config, "W3", "w3-all-bodies-one-request");
    let opened = open_subject(config, &mut sample, Capacity::Roomy);
    let targets = targets(config, &opened, 8);
    let first = targets.first().expect("the fixture declares a class");
    let environment = environment(&opened.snapshot, opened.scope.clone(), opened.roots.clone());

    // (a) every body of one class, in one request. The view is produced inside `request` and
    // serialized inside `output`: the output phase measures the result's own encoding, not a second
    // request, which is the difference between a phase decomposition and a doubled workload.
    let mut budget = request_budget(config, &opened.store);
    let request = ClassViewRequest {
        class: ClassRef::Definition {
            definition: first.definition.clone(),
        },
        bodies: first
            .methods
            .iter()
            .map(|method| BodyRef::Method {
                method: method.clone(),
            })
            .collect(),
    };
    let before = counts_before();
    let view = sample.stages.phase("request", || {
        performed(
            opened
                .engine
                .class_view(&opened.snapshot, &opened.scope, &request, &mut budget)
                .expect("a declared class is viewable with its bodies"),
            "a class declaration",
        )
    });
    let usage = budget.usage();
    let (returned_bytes, domain) = sample.stages.phase("output", || {
        (
            serde_json::to_vec(&view).map_or(0, |encoded| encoded.len() as u64),
            domain_of(&view),
        )
    });
    sample.counts = counts_after(before);
    count_numbers(&mut sample);
    samples.push(
        sample
            .number("class_methods", first.methods.len() as u64)
            .number("classes", targets.len() as u64)
            .number("request_micros", usage.elapsed_millis)
            .number("method_bodies", usage.method_bodies)
            .number("class_headers", usage.class_headers)
            .number("output_bytes", usage.output_bytes)
            .number("returned_bytes", returned_bytes)
            .text(
                "class",
                String::from_utf8_lossy(&first.class.0).into_owned(),
            )
            .usage(&usage)
            .cache(&opened.store.report())
            .domain(domain),
    );

    // (b) the same bodies, one request per method.
    let mut sample = Sample::new(config, "W3", "w3-per-method-requests");
    let mut per_request = Vec::new();
    let mut usages = Vec::new();
    let mut reports = Vec::new();
    let started = Instant::now();
    let before = counts_before();
    sample.stages.phase("request", || {
        for method in &first.methods {
            let request = MethodOperationRequest {
                method: MethodRef::Method {
                    method: method.clone(),
                },
                environment: environment.clone(),
            };
            let mut budget = request_budget(config, &opened.store);
            let started = Instant::now();
            let recovered = performed(
                opened
                    .engine
                    .recover_target_with_evidence(
                        std::slice::from_ref(&opened.snapshot),
                        &request,
                        &RecoveryEvidenceRequest::essential(),
                        &mut budget,
                    )
                    .expect("a declared member is recoverable"),
                "a member identity",
            );
            per_request.push(micros(started.elapsed()));
            usages.push(budget.usage());
            reports.push(recovered);
        }
    });
    let sequence = micros(started.elapsed());
    let (returned_bytes, sequence_domain) = sample.stages.phase("output", || {
        let mut bytes = 0_u64;
        let mut lines = Vec::new();
        for report in &reports {
            bytes = bytes.saturating_add(
                serde_json::to_vec(report).map_or(0, |encoded| encoded.len() as u64),
            );
            lines.push(domain_of(report));
        }
        (bytes, lines_domain(&lines))
    });
    sample.counts = counts_after(before);
    count_numbers(&mut sample);
    samples.push(
        sample
            .number("class_methods", first.methods.len() as u64)
            .number("sequence_micros", sequence)
            .number(
                "method_bodies_total",
                usages.iter().map(|u| u.method_bodies).sum(),
            )
            .number(
                "class_headers_total",
                usages.iter().map(|u| u.class_headers).sum(),
            )
            .number(
                "archive_entries_total",
                usages.iter().map(|u| u.archive_entries).sum(),
            )
            .number("returned_bytes", returned_bytes)
            .series("per_request_micros", per_request)
            .text("sequence_domain", sequence_domain)
            .cache(&opened.store.report()),
    );

    // (c) a fixed method set across the first classes: one member per class.
    let mut sample = Sample::new(config, "W3", "w3-fixed-set-across-classes");
    let mut per_request = Vec::new();
    let mut usages = Vec::new();
    let mut reports = Vec::new();
    let started = Instant::now();
    sample.stages.phase("request", || {
        for target in &targets {
            let Some(method) = target.methods.first() else {
                continue;
            };
            let request = MethodOperationRequest {
                method: MethodRef::Method {
                    method: method.clone(),
                },
                environment: environment.clone(),
            };
            let mut budget = request_budget(config, &opened.store);
            let started = Instant::now();
            let report = performed(
                opened
                    .engine
                    .recover_target_with_evidence(
                        std::slice::from_ref(&opened.snapshot),
                        &request,
                        &RecoveryEvidenceRequest::essential(),
                        &mut budget,
                    )
                    .expect("a declared member is recoverable"),
                "a member identity",
            );
            per_request.push(micros(started.elapsed()));
            usages.push(budget.usage());
            reports.push(report);
        }
    });
    let sequence = micros(started.elapsed());
    let (returned_bytes, sequence_domain) = sample.stages.phase("output", || {
        let mut bytes = 0_u64;
        let mut lines = Vec::new();
        for report in &reports {
            bytes = bytes.saturating_add(
                serde_json::to_vec(report).map_or(0, |encoded| encoded.len() as u64),
            );
            lines.push(domain_of(report));
        }
        (bytes, lines_domain(&lines))
    });
    samples.push(
        sample
            .number("classes", targets.len() as u64)
            .number("sequence_micros", sequence)
            .number(
                "method_bodies_total",
                usages.iter().map(|u| u.method_bodies).sum(),
            )
            .number(
                "class_headers_total",
                usages.iter().map(|u| u.class_headers).sum(),
            )
            .number("returned_bytes", returned_bytes)
            .series("per_request_micros", per_request)
            .text("sequence_domain", sequence_domain)
            .cache(&opened.store.report()),
    );

    // (d) and (e): every body of the same eight classes, in the two delivery shapes O4 compares.
    // Both arms ask the same `class_view` question about the same members and differ only in how the
    // work is grouped: class-major asks one request per class with every body selected, method-major
    // asks one request per member. The arms are run one after the other on one store, and each arm's
    // own reading is what it cost.
    samples.push(w3_all_bodies_arm(config, &opened, &targets, true));
    samples.push(w3_all_bodies_arm(config, &opened, &targets, false));
    samples
}

/// One decoded member body, as the line the two delivery shapes are compared through.
///
/// The projection is the decode's own facts — the member identity, the frame the code declares, the
/// span it occupies, the instruction stream's own digest and the handler count — because that is what
/// the `class_view` of both shapes publishes. The document *around* those facts (a view's items, its
/// coverage, the charges it states) is a reading of how the run was grouped, not of what it decoded,
/// and is deliberately not part of the comparison.
fn body_line(body: &ClassViewBody) -> Option<String> {
    match body {
        ClassViewBody::Read {
            method,
            max_stack,
            max_locals,
            code_span,
            instructions,
            exception_handlers,
            ..
        } => Some(format!(
            "{method:?}|stack={max_stack}|locals={max_locals}|span={code_span:?}|\
             handlers={}|code=blake3:{}",
            exception_handlers.len(),
            blake3::hash(format!("{instructions:?}").as_bytes()).to_hex()
        )),
        ClassViewBody::NotDeclared { .. } | ClassViewBody::Refused { .. } => None,
    }
}

/// Every body of every class in `targets`, in one of the two delivery shapes.
///
/// `class_major` asks one `class_view` per class, selecting every body that class declares: one
/// preparation serves that class's members, and the members are delivered together. Otherwise the
/// same bodies are asked one request at a time, which is the shape every other W3 sample uses and the
/// one the bulk operation replaces. Both arms publish the same per-member decode facts
/// ([`body_line`]), so their `arm_domain` figures are comparable and
/// [`the_two_delivery_shapes_answer_the_same_bodies`] asserts that they are equal.
fn w3_all_bodies_arm(
    config: &Config,
    opened: &Opened,
    targets: &[Target],
    class_major: bool,
) -> Sample {
    let name = if class_major {
        "w3-batch-per-class-across-classes"
    } else {
        "w3-per-method-across-classes"
    };
    let mut sample = Sample::new(config, "W3", name);
    let mut per_request = Vec::new();
    let mut usages = Vec::new();
    let mut lines = Vec::new();
    let mut bodies = 0_u64;
    let before = counts_before();
    let started = Instant::now();
    sample.stages.phase("request", || {
        for target in targets {
            let mut budget = request_budget(config, &opened.store);
            let selected: Vec<Vec<BodyRef>> = if class_major {
                vec![
                    target
                        .methods
                        .iter()
                        .map(|method| BodyRef::Method {
                            method: method.clone(),
                        })
                        .collect(),
                ]
            } else {
                target
                    .methods
                    .iter()
                    .map(|method| {
                        vec![BodyRef::Method {
                            method: method.clone(),
                        }]
                    })
                    .collect()
            };
            for selection in &selected {
                let request = ClassViewRequest {
                    class: ClassRef::Definition {
                        definition: target.definition.clone(),
                    },
                    bodies: selection.clone(),
                };
                let started = Instant::now();
                let view = performed(
                    opened
                        .engine
                        .class_view(&opened.snapshot, &opened.scope, &request, &mut budget)
                        .expect("a declared class is viewable with its bodies"),
                    "a class declaration",
                );
                per_request.push(micros(started.elapsed()));
                for body in &view.bodies {
                    if let Some(line) = body_line(body) {
                        lines.push(line);
                        bodies = bodies.saturating_add(1);
                    }
                }
            }
            usages.push(budget.usage());
        }
    });
    let sequence = micros(started.elapsed());
    lines.sort();
    let (returned_bytes, arm_domain) = sample
        .stages
        .phase("output", || (0_u64, lines_domain(&lines)));
    sample.counts = counts_after(before);
    count_numbers(&mut sample);
    sample
        .number("classes", targets.len() as u64)
        .number("bodies", bodies)
        .number("sequence_micros", sequence)
        .number(
            "method_bodies_total",
            usages.iter().map(|u| u.method_bodies).sum(),
        )
        .number(
            "class_headers_total",
            usages.iter().map(|u| u.class_headers).sum(),
        )
        .number("returned_bytes", returned_bytes)
        .series("per_request_micros", per_request)
        .text("arm_domain", arm_domain)
        .text(
            "shape",
            if class_major {
                "class-major"
            } else {
                "method-major"
            },
        )
        .cache(&opened.store.report())
}

// ---------------------------------------------------------------------------------------------
// W4 — a query's first page, its continued pages, and an abandoned cursor
// ---------------------------------------------------------------------------------------------

/// One W4 page sequence: the pages the query returned, and what each cost.
struct Pages {
    reports: Vec<QueryReport>,
    page_micros: Vec<u64>,
    /// What each page's own coverage stated it had scanned *by the end of that page*: the sequence of
    /// this figure is how a continuation's rescan shows up in a count instead of only in a duration.
    scanned_series: Vec<u64>,
    scanned_items: u64,
    has_more: bool,
    /// The whole sequence's one budget, as the usage the sequence was charged.
    usage: UsageSnapshot,
}

impl Pages {
    fn items(&self) -> u64 {
        self.reports
            .iter()
            .map(|report| report.items.len() as u64)
            .sum()
    }

    fn pages(&self) -> u64 {
        self.reports.len() as u64
    }
}

/// Runs one W4 page sequence: `max_items`, at most `pages` pages, with the consumers asked.
fn w4_pages(
    config: &Config,
    opened: &Opened,
    max_items: u64,
    pages: usize,
    consumers: Vec<ConsumerKind>,
) -> Pages {
    let mut cursor: Option<QueryCursor> = None;
    let mut reports = Vec::new();
    let mut page_micros = Vec::new();
    let mut scanned_series = Vec::new();
    let mut scanned = 0_u64;
    let mut has_more = false;
    let mut budget = request_budget(config, &opened.store);
    for _ in 0..pages {
        let request = QueryRequest {
            relation: QueryRelation::ConstantPoolContains,
            target: QueryTarget::Symbol {
                value: SymbolRef::Class {
                    owner: JvmBytes(b"java/lang/Object".to_vec()),
                },
            },
            physical: PhysicalView {
                snapshot: opened.snapshot.id().clone(),
                scope: opened.scope.clone(),
            },
            consumers: ConsumerSchema::new(1, consumers.clone()),
            max_items,
            cursor: cursor.clone(),
        };
        let started = Instant::now();
        let report = opened
            .engine
            .query(&opened.snapshot, &request, &mut budget)
            .expect("the fixture is queryable");
        page_micros.push(micros(started.elapsed()));
        scanned = report.coverage.scanned_items;
        scanned_series.push(scanned);
        has_more = report.page.has_more;
        cursor = report.page.cursor.clone();
        reports.push(report);
        if cursor.is_none() {
            break;
        }
    }
    let usage = budget.usage();
    Pages {
        reports,
        page_micros,
        scanned_series,
        scanned_items: scanned,
        has_more,
        usage,
    }
}

/// The caller's own rendering of a page sequence: the `output` phase of a W4 sample, and the item
/// identities the page set is compared through.
fn rendered_pages(sample: &mut Sample, pages: &Pages) -> (u64, String) {
    sample.stages.phase("output", || {
        let mut bytes = 0_u64;
        let mut lines = Vec::new();
        for report in &pages.reports {
            bytes = bytes.saturating_add(
                serde_json::to_vec(report).map_or(0, |encoded| encoded.len() as u64),
            );
            lines.extend(item_lines(&report.items));
        }
        (bytes, lines_domain(&lines))
    })
}

/// The work figures of one W4 page sequence: what its own budget was charged, and the coverage series
/// a continuation states.
///
/// The pages of a sequence share **one** request budget (a page is one request of one search, not one
/// request per page), so the sequence's usage is the reading of "what continuing cost" — and the
/// per-page coverage series is where a boundary rescan shows up as a count rather than only as a
/// duration.
fn pages_work(sample: Sample, pages: &Pages) -> Sample {
    let usage = &pages.usage;
    sample
        .number("archive_entries_total", usage.archive_entries)
        .number("entry_bytes_total", usage.entry_bytes)
        .number("read_bytes_total", usage.read_bytes)
        .number("class_bytes_total", usage.class_bytes)
        .number("result_items", usage.result_items)
        .series("scanned_series", pages.scanned_series.clone())
        .usage(usage)
}

/// The declared bound on a page sequence: a continuation that never ends is a defect, and a
/// workload that stopped at a bound says so through its own `pages`/`has_more` readings.
const PAGE_CAP: usize = 4096;

fn w4(config: &Config) -> Vec<Sample> {
    let consumers = vec![ConsumerKind::Type];
    let mut samples = Vec::new();

    let mut small = Sample::new(config, "W4", "w4-small-page");
    let opened = open_subject(config, &mut small, Capacity::Roomy);
    let before = counts_before();
    let small_pages = small.stages.phase("request", || {
        w4_pages(config, &opened, 2, PAGE_CAP, consumers.clone())
    });
    let (small_bytes, small_domain) = rendered_pages(&mut small, &small_pages);
    small.counts = counts_after(before);
    count_numbers(&mut small);
    samples.push(pages_work(
        small
            .number("max_items", 2)
            .number("items", small_pages.items())
            .number("pages", small_pages.pages())
            .number("scanned_items", small_pages.scanned_items)
            .number("has_more", u64::from(small_pages.has_more))
            .number("returned_bytes", small_bytes)
            .series("page_micros", small_pages.page_micros.clone())
            .text("items_domain", small_domain.clone())
            .cache(&opened.store.report()),
        &small_pages,
    ));

    let mut large = Sample::new(config, "W4", "w4-large-page");
    let large_pages = large.stages.phase("request", || {
        w4_pages(config, &opened, 64, PAGE_CAP, consumers.clone())
    });
    let (large_bytes, large_domain) = rendered_pages(&mut large, &large_pages);
    samples.push(pages_work(
        large
            .number("max_items", 64)
            .number("items", large_pages.items())
            .number("pages", large_pages.pages())
            .number("scanned_items", large_pages.scanned_items)
            .number("has_more", u64::from(large_pages.has_more))
            .number("returned_bytes", large_bytes)
            .series("page_micros", large_pages.page_micros.clone())
            .text("items_domain", large_domain.clone())
            .text(
                "same_items_as_small_page",
                u64::from(large_domain == small_domain).to_string(),
            )
            .cache(&opened.store.report()),
        &large_pages,
    ));

    let mut multi = Sample::new(config, "W4", "w4-multi-consumer");
    let multi_pages = multi.stages.phase("request", || {
        w4_pages(
            config,
            &opened,
            64,
            PAGE_CAP,
            vec![
                ConsumerKind::Type,
                ConsumerKind::Invocation,
                ConsumerKind::Field,
                ConsumerKind::Constant,
            ],
        )
    });
    let (multi_bytes, _) = rendered_pages(&mut multi, &multi_pages);
    samples.push(pages_work(
        multi
            .number("max_items", 64)
            .number("consumers", 4)
            .number("items", multi_pages.items())
            .number("pages", multi_pages.pages())
            .number("scanned_items", multi_pages.scanned_items)
            .number("has_more", u64::from(multi_pages.has_more))
            .number("returned_bytes", multi_bytes)
            .number(
                "items_equal_single_consumer",
                u64::from(
                    lines_domain(
                        &multi_pages
                            .reports
                            .iter()
                            .flat_map(|report| item_lines(&report.items))
                            .collect::<Vec<_>>(),
                    ) == large_domain,
                ),
            )
            .series("page_micros", multi_pages.page_micros.clone())
            .cache(&opened.store.report()),
        &multi_pages,
    ));

    let mut abandon = Sample::new(config, "W4", "w4-abandon-after-one-page");
    let one_page = abandon.stages.phase("request", || {
        w4_pages(config, &opened, 2, 1, consumers.clone())
    });
    let (abandon_bytes, _) = rendered_pages(&mut abandon, &one_page);
    samples.push(pages_work(
        abandon
            .number("max_items", 2)
            .number("items", one_page.items())
            .number("pages", one_page.pages())
            .number("scanned_items", one_page.scanned_items)
            .number("has_more", u64::from(one_page.has_more))
            .number("returned_bytes", abandon_bytes)
            .series("page_micros", one_page.page_micros.clone())
            .cache(&opened.store.report()),
        &one_page,
    ));

    // The damaged suffix: the same archive with its last root entry — `lib/more.jar`, the nested
    // container the scope's own walk descends into last — damaged in one data byte. A page that stops
    // before it never reads it; a scan that reaches it states the damage. The three arms are the same
    // query at three stopping points, which is what makes the comparison a reading of the stop rather
    // than of a second fixture.
    let damaged = damaged_fixture();
    let mut first_page = Sample::new(config, "W4", "w4-damaged-first-page");
    let damaged_subject = open_bytes(config, &mut first_page, Capacity::Roomy, damaged.clone());
    let one = first_page.stages.phase("request", || {
        w4_pages(config, &damaged_subject, 2, 1, consumers.clone())
    });
    let (first_bytes, first_domain) = rendered_pages(&mut first_page, &one);
    samples.push(pages_work(
        first_page
            .number("max_items", 2)
            .number("items", one.items())
            .number("pages", one.pages())
            .number("scanned_items", one.scanned_items)
            .number("has_more", u64::from(one.has_more))
            .number("returned_bytes", first_bytes)
            .number("diagnostics", diagnostic_count(&one))
            .series("page_micros", one.page_micros.clone())
            .text("items_domain", first_domain)
            .text("diagnostic_codes", diagnostic_codes(&one))
            .cache(&damaged_subject.store.report()),
        &one,
    ));

    let mut full = Sample::new(config, "W4", "w4-damaged-full-page");
    let full_subject = open_bytes(config, &mut full, Capacity::Roomy, damaged.clone());
    let whole = full.stages.phase("request", || {
        w4_pages(config, &full_subject, 64, 1, consumers.clone())
    });
    let (full_bytes, full_domain) = rendered_pages(&mut full, &whole);
    samples.push(pages_work(
        full.number("max_items", 64)
            .number("items", whole.items())
            .number("pages", whole.pages())
            .number("scanned_items", whole.scanned_items)
            .number("has_more", u64::from(whole.has_more))
            .number("returned_bytes", full_bytes)
            .number("diagnostics", diagnostic_count(&whole))
            .series("page_micros", whole.page_micros.clone())
            .text("items_domain", full_domain.clone())
            .text("diagnostic_codes", diagnostic_codes(&whole))
            .cache(&full_subject.store.report()),
        &whole,
    ));

    let mut exhausted = Sample::new(config, "W4", "w4-damaged-continued-to-exhaustion");
    let exhausted_subject = open_bytes(config, &mut exhausted, Capacity::Roomy, damaged);
    let all = exhausted.stages.phase("request", || {
        w4_pages(config, &exhausted_subject, 2, PAGE_CAP, consumers.clone())
    });
    let (exhausted_bytes, exhausted_domain) = rendered_pages(&mut exhausted, &all);
    samples.push(pages_work(
        exhausted
            .number("max_items", 2)
            .number("items", all.items())
            .number("pages", all.pages())
            .number("scanned_items", all.scanned_items)
            .number("has_more", u64::from(all.has_more))
            .number("returned_bytes", exhausted_bytes)
            .number("diagnostics", diagnostic_count(&all))
            .number(
                "items_equal_full_page",
                u64::from(exhausted_domain == full_domain),
            )
            .series("page_micros", all.page_micros.clone())
            .text("items_domain", exhausted_domain)
            .text("diagnostic_codes", diagnostic_codes(&all))
            .cache(&exhausted_subject.store.report()),
        &all,
    ));
    samples
}

/// The fixture with one data byte of one class entry flipped.
///
/// The damaged entry is `HistoricalControlFlow.class`: the last *class* of the root container, whose
/// own walk order puts the nested container `lib/more.jar` and its nineteen classes behind it. The
/// nested container itself is left intact on purpose — it is the container the scope's own roots are
/// declared in, so damaging it would break the fixture's environment declaration rather than put a
/// damaged byte behind a page boundary. Every other byte of the archive is the pinned fixture.
fn damaged_fixture() -> Vec<u8> {
    const DAMAGED: &[u8] = b"HistoricalControlFlow.class";
    let mut bytes = fixture();
    let (snapshot, _usage) = open(bytes.clone());
    let mut budget = Budget::new(limits());
    let listing = snapshot
        .enumerate_artifact_tree(&mut budget)
        .expect("the fixture is a readable archive tree");
    let offset = listing
        .containers
        .iter()
        .flat_map(|container| container.entries.iter())
        .find(|entry| entry.id.raw_name.0 == DAMAGED)
        .map(|entry| entry.layout.compressed_data.start)
        .expect("the fixture holds the entry to damage");
    assert!(
        offset > 0 && (offset as usize) < bytes.len(),
        "the damaged entry's data lies inside the archive"
    );
    bytes[offset as usize] ^= 0xff;
    bytes
}

/// How many diagnostics a page sequence stated.
fn diagnostic_count(pages: &Pages) -> u64 {
    pages
        .reports
        .iter()
        .map(|report| report.diagnostics.len() as u64)
        .sum()
}

/// Every diagnostic code a page sequence stated, sorted and deduplicated.
fn diagnostic_codes(pages: &Pages) -> String {
    let mut codes = pages
        .reports
        .iter()
        .flat_map(|report| report.diagnostics.iter())
        .map(|diagnostic| diagnostic.code.clone())
        .collect::<Vec<_>>();
    codes.sort();
    codes.dedup();
    codes.join(",")
}

// ---------------------------------------------------------------------------------------------
// W5 and W6a — the whole-scope sweep
// ---------------------------------------------------------------------------------------------

/// One whole-scope sweep's readings, as the sink and the report state them.
struct Sweep {
    records: u64,
    methods: u64,
    /// The method records the sink itself confirmed before it answered `Stop` (a run that took the
    /// whole stream confirms every one of them).
    methods_seen: u64,
    stopped_by_sink: bool,
    encoded_bytes: u64,
    written_bytes: u64,
    status: String,
    classes: u64,
    declared: u64,
    first_result_micros: u64,
    domain: String,
    summary_domain: String,
    usage: UsageSnapshot,
    entry_usage: UsageSnapshot,
    discovery_usage: UsageSnapshot,
    method_usage: UsageSnapshot,
    delivery_usage: UsageSnapshot,
    /// The summary's own classification of what it delivered, as the contract layer is checked
    /// through it: every bucket, the traversal's completeness, and whether the terminal event
    /// reached the sink.
    outcomes: [u64; 6],
    delivered: u64,
    not_executed: u64,
    prepared: u64,
    refused_classes: u64,
    traversal_complete: bool,
    final_delivered: bool,
    diagnostic_codes: Vec<String>,
    scratch_file_bytes: u64,
    probe: Option<Value>,
}

/// Runs one sweep under `mode`, at `workers`, over the caller's store.
fn sweep(
    config: &Config,
    opened: &Opened,
    sample: &mut Sample,
    workers: usize,
    mode: Mode,
) -> Sweep {
    let environment = environment(&opened.snapshot, opened.scope.clone(), opened.roots.clone());
    let request_started = Instant::now();
    let mut sink = Measuring::new(mode, config.stop_after);
    #[cfg(feature = "test-support")]
    let probe = config
        .instrumented
        .then(|| std::sync::Arc::new(BulkProbe::new()));
    let mut budget = request_budget(config, &opened.store);
    let report = {
        let request = request(environment, workers);
        #[cfg(feature = "test-support")]
        let request = match probe.as_ref() {
            Some(probe) => request.with_probe(probe.clone()),
            None => request,
        };
        sample.stages.phase("request", || {
            opened
                .engine
                .recover_all(
                    std::slice::from_ref(&opened.snapshot),
                    &request,
                    &mut budget,
                    &mut sink,
                )
                .expect("the fixture scope is recoverable")
        })
    };
    let usage = report.usage.clone();
    let scratch_file_bytes = sink.finish();
    let (domain, methods) = stream_domain(&sink.recorder);
    let summary_domain = format!("{:?}", fingerprint(&report.summary));
    sample
        .stages
        .nested("sink_visit", "request", as_micros(sink.visit_nanos));
    sample
        .stages
        .nested("sink_encode", "request", as_micros(sink.encode_nanos));
    sample
        .stages
        .nested("sink_write", "request", as_micros(sink.write_nanos));
    #[cfg(feature = "test-support")]
    let probe_value = config.instrumented.then(|| {
        serde_json::to_value(probe.as_ref().expect("the probe was attached").reading())
            .expect("a probe reading renders")
    });
    #[cfg(not(feature = "test-support"))]
    let probe_value = None;
    Sweep {
        records: sink.records,
        methods,
        methods_seen: sink.methods_seen,
        stopped_by_sink: sink.stopped,
        encoded_bytes: sink.encoded_bytes,
        written_bytes: sink.written_bytes,
        status: report.summary.status().to_owned(),
        classes: report.summary.classes_seen,
        declared: report.summary.methods_declared,
        first_result_micros: sink
            .first_method
            .map(|at| micros(at.duration_since(request_started)))
            .unwrap_or(0),
        domain,
        summary_domain,
        usage,
        entry_usage: report.entry_usage.clone(),
        discovery_usage: report.discovery_usage.clone(),
        method_usage: report.method_usage.clone(),
        delivery_usage: report.delivery_usage.clone(),
        outcomes: [
            report.summary.outcomes.produced,
            report.summary.outcomes.explanation_only,
            report.summary.outcomes.not_produced,
            report.summary.outcomes.refused,
            report.summary.outcomes.no_body,
            report.summary.outcomes.oversized,
        ],
        delivered: report.summary.methods_delivered,
        not_executed: report.summary.methods_not_executed,
        prepared: report.summary.classes_prepared,
        refused_classes: report.summary.classes_refused,
        traversal_complete: report.summary.traversal_complete,
        final_delivered: report.final_delivered,
        diagnostic_codes: {
            let mut codes = sink
                .recorder
                .diagnostics()
                .iter()
                .map(|diagnostic| diagnostic.diagnostic.code.clone())
                .collect::<Vec<_>>();
            codes.sort();
            codes.dedup();
            codes
        },
        scratch_file_bytes,
        probe: probe_value,
    }
}

/// The operation's own accounts, as the four owners it bills beside its total.
///
/// They are readings of the run's *work*, not of its wall clock: the deliverable's three layers keep
/// them apart, and this is the middle one for a bulk sample.
fn bulk_accounts(swept: &Sweep) -> Value {
    json!({
        "total": usage_value(&swept.usage),
        "entry": usage_value(&swept.entry_usage),
        "discovery": usage_value(&swept.discovery_usage),
        "methods": usage_value(&swept.method_usage),
        "delivery": usage_value(&swept.delivery_usage),
    })
}

/// The summary's own classification, as the contract layer of a bulk sample.
fn outcome_numbers(sample: Sample, swept: &Sweep) -> Sample {
    let [
        produced,
        explanation_only,
        not_produced,
        refused,
        no_body,
        oversized,
    ] = swept.outcomes;
    sample
        .number("delivered", swept.delivered)
        .number("not_executed", swept.not_executed)
        .number("classes_prepared", swept.prepared)
        .number("classes_refused", swept.refused_classes)
        .number("traversal_complete", u64::from(swept.traversal_complete))
        .number("final_delivered", u64::from(swept.final_delivered))
        .number("outcome_produced", produced)
        .number("outcome_explanation_only", explanation_only)
        .number("outcome_not_produced", not_produced)
        .number("outcome_refused", refused)
        .number("outcome_no_body", no_body)
        .number("outcome_oversized", oversized)
        .text("diagnostic_codes", swept.diagnostic_codes.join(","))
}

/// The sweep reading every W5/W6a sample carries: counts, usage, the store's own report and the
/// stream's fingerprint.
fn w5(config: &Config) -> Vec<Sample> {
    let mut samples = Vec::new();
    let mut sample = Sample::new(config, "W5", "w5-sweep");
    let opened = open_subject(config, &mut sample, config.capacity);
    let swept = sweep(config, &opened, &mut sample, config.workers, config.mode);
    let mut sample = sample
        .number("records", swept.records)
        .number("methods", swept.methods)
        .number("classes_seen", swept.classes)
        .number("methods_declared", swept.declared)
        .number("first_result_micros", swept.first_result_micros)
        .number("encoded_bytes", swept.encoded_bytes)
        .number("written_bytes", swept.written_bytes)
        .number("scratch_file_bytes", swept.scratch_file_bytes)
        .text("status", swept.status.clone())
        .text("workers", config.workers.to_string())
        .text("sink", config.mode.name())
        .text("capacity", config.capacity.name())
        .usage(&swept.usage)
        .cache(&opened.store.report())
        .domain(swept.domain.clone());
    sample.probe = swept.probe.clone();
    sample.bulk = Some(bulk_accounts(&swept));
    let sample = outcome_numbers(sample, &swept);
    samples.push(sample);

    // The round trip: the same store, ten more single-member requests. This is the "hot" half of
    // W5 — what a store that is still holding an answer changes about the next request's work.
    let mut sample = Sample::new(config, "W5", "w5-round-trip");
    let (class, member) = first_member(config, &opened);
    let environment = environment(&opened.snapshot, opened.scope.clone(), opened.roots.clone());
    let mut per_request = Vec::new();
    let mut usages = Vec::new();
    let started = Instant::now();
    let mut reports = Vec::new();
    let before = counts_before();
    sample.stages.phase("request", || {
        for _ in 0..10 {
            let request = MethodOperationRequest {
                method: MethodRef::Method {
                    method: member.clone(),
                },
                environment: environment.clone(),
            };
            let mut budget = request_budget(config, &opened.store);
            let started = Instant::now();
            let recovered = performed(
                opened
                    .engine
                    .recover_target_with_evidence(
                        std::slice::from_ref(&opened.snapshot),
                        &request,
                        &RecoveryEvidenceRequest::essential(),
                        &mut budget,
                    )
                    .expect("a declared member is recoverable"),
                "a member identity",
            );
            per_request.push(micros(started.elapsed()));
            usages.push(budget.usage());
            reports.push(recovered);
        }
    });
    let sequence = micros(started.elapsed());
    let (returned_bytes, round_trip_domain) = sample.stages.phase("output", || {
        let mut bytes = 0_u64;
        let mut lines = Vec::new();
        for report in &reports {
            bytes = bytes.saturating_add(
                serde_json::to_vec(report).map_or(0, |encoded| encoded.len() as u64),
            );
            lines.push(result_domain_of(report));
        }
        (bytes, lines_domain(&lines))
    });
    sample.counts = counts_after(before);
    count_numbers(&mut sample);
    samples.push(
        sample
            .number("round_trips", 10)
            .number("sequence_micros", sequence)
            .number(
                "class_headers_total",
                usages.iter().map(|u| u.class_headers).sum(),
            )
            .number(
                "archive_entries_total",
                usages.iter().map(|u| u.archive_entries).sum(),
            )
            .number("returned_bytes", returned_bytes)
            .series("per_request_micros", per_request)
            .text("class", String::from_utf8_lossy(&class.0).into_owned())
            .text("round_trip_domain", round_trip_domain)
            .text("sweep_domain", swept.domain.clone())
            .cache(&opened.store.report()),
    );
    samples
}

fn w6a(config: &Config) -> Vec<Sample> {
    // The cancellation arm is one variable of this workload, not a second workload: a configuration
    // that declared `stop=<n>` runs the same export and answers `Stop` after `n` method records, so
    // the sample's own name says which reading it is.
    let cancelled = config.stop_after.is_some();
    let mut sample = Sample::new(
        config,
        "W6a",
        if cancelled {
            "w6a-export-cancelled"
        } else {
            "w6a-export"
        },
    );
    let opened = open_subject(config, &mut sample, config.capacity);
    let before = counts_before();
    let swept = sweep(config, &opened, &mut sample, config.workers, config.mode);
    let mut sample = sample
        .number("workers", config.workers as u64)
        .number("records", swept.records)
        .number("methods", swept.methods)
        .number("methods_confirmed", swept.methods_seen)
        .number("stopped_by_sink", u64::from(swept.stopped_by_sink))
        .number("stop_after", config.stop_after.unwrap_or(0))
        .number("classes_seen", swept.classes)
        .number("methods_declared", swept.declared)
        .number("first_result_micros", swept.first_result_micros)
        .number("encoded_bytes", swept.encoded_bytes)
        .number("written_bytes", swept.written_bytes)
        .number("scratch_file_bytes", swept.scratch_file_bytes)
        .text("status", swept.status.clone())
        .text("sink", config.mode.name())
        .text("summary_domain", swept.summary_domain.clone())
        .usage(&swept.usage)
        .cache(&opened.store.report())
        .domain(swept.domain.clone());
    sample.probe = swept.probe.clone();
    sample.bulk = Some(bulk_accounts(&swept));
    sample.counts = counts_after(before);
    count_numbers(&mut sample);
    vec![outcome_numbers(sample, &swept)]
}

/// The workload vocabulary, as task 1.2 fixes it.
///
/// `w6b` (independent host requests over one snapshot, and the single-flight candidate) is
/// deliberately **not** here: the archived P5 measurement left it `disabled` with a recorded
/// re-entry trigger, and this file does not invent demand for it. Adding one is a decision that
/// touches this list and the registration in the change's evidence, not a silent new string.
const WORKLOADS: [&str; 6] = ["w1", "w2", "w3", "w4", "w5", "w6a"];

/// The workload `config` names, as the samples it prints.
fn run_workload(config: &Config) -> Vec<Sample> {
    match config.workload.as_str() {
        "w1" => w1(config),
        "w2" => w2(config),
        "w3" => w3(config),
        "w4" => w4(config),
        "w5" => w5(config),
        "w6a" => w6a(config),
        other => panic!(
            "`{other}` is not one of the fixed workloads {WORKLOADS:?}. `w6b` (a host running \
             independent requests over one snapshot, and the single-flight candidate) has no \
             recorded demand: `openspec/changes/optimize-demand-workloads/evidence/g0-workloads.md` \
             registers the trigger that would create one, and `docs/support-matrix.md` records that \
             the path does not exist"
        ),
    }
}

/// One measured sample, printed as one JSON line.
///
/// The campaign builds this target once, runs this test once per sample with the `JARDE_OPTIMIZE_*`
/// variables set, and wraps each process so the run's own wall time and peak RSS sit beside the
/// line. Nothing here asserts a duration: the caller reads the samples, and the harness never
/// selects the good ones.
#[test]
#[ignore = "measurement: build it, then run this binary with --exact measure_workload --ignored --nocapture"]
fn measure_workload() {
    let config = Config::from_env();
    for sample in run_workload(&config) {
        sample.print();
    }
}

// ---------------------------------------------------------------------------------------------
// The verifiers
// ---------------------------------------------------------------------------------------------

/// The workload vocabulary, and the absence of `w6b` as a recorded decision rather than an omission.
#[test]
fn the_workload_vocabulary_is_the_one_task_1_2_fixes() {
    assert_eq!(WORKLOADS, ["w1", "w2", "w3", "w4", "w5", "w6a"]);
    assert!(
        !WORKLOADS.contains(&"w6b"),
        "w6b has no recorded demand: see the registration in g0-workloads.md"
    );
    let unnamed = "w6b";
    let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let config = Config {
            workload: unnamed.to_owned(),
            artifact: None,
            workers: 1,
            mode: Mode::Discard,
            capacity: Capacity::Roomy,
            instrumented: false,
            stop_after: None,
        };
        run_workload(&config)
    }));
    let refusal = refused
        .err()
        .and_then(|payload| payload.downcast_ref::<String>().cloned())
        .expect("an unregistered workload is refused");
    assert!(
        refusal.contains("trigger") || refusal.contains("no recorded demand"),
        "the refusal names the decision it is missing: {refusal}"
    );
}

/// The fixture is the pinned bytes, and its documented class and body counts are its own.
///
/// This is the "fixed workload" half of task 1.2 stated as a gate: a later edit that adds, removes
/// or reshapes a class moves the digest and fails here, rather than silently measuring another
/// corpus. The internal-name rule `bulk_support` states (an entry's name is where the class is
/// looked up: `prefix + internal name + ".class"`) is checked on the built archive's own listing.
#[test]
fn the_fixture_is_the_pinned_bytes() {
    let first = fixture();
    let second = fixture();
    assert_eq!(
        first, second,
        "the fixture is a total function of committed bytes"
    );
    let digest = fixture_digest(&first);
    println!("fixture: {} bytes, blake3 {}", first.len(), digest);
    if PINNED_FIXTURE != "unset" {
        assert_eq!(
            digest, PINNED_FIXTURE,
            "the fixture's bytes moved: the W1-W6a baseline is fixed at {PINNED_FIXTURE} and this \
             build produces {digest}. Re-pin only for a workflow change that says so."
        );
    }

    let (snapshot, _usage) = open(first.clone());
    let mut budget = Budget::new(limits());
    let listing = snapshot
        .enumerate_artifact_tree(&mut budget)
        .expect("the fixture is a readable archive tree");
    let mut names = Vec::new();
    for container in &listing.containers {
        for entry in &container.entries {
            names.push(String::from_utf8_lossy(&entry.id.raw_name.0).into_owned());
        }
    }
    let mut classes = names
        .iter()
        .filter(|name| name.ends_with(".class"))
        .cloned()
        .collect::<Vec<_>>();
    classes.sort();
    println!("fixture entries: {names:?}");
    assert_eq!(
        classes.len() as u64,
        PINNED_CLASSES,
        "the fixture declares {PINNED_CLASSES} classes and the archive holds {classes:?}"
    );
    assert!(
        names.iter().any(|name| name == "lib/more.jar"),
        "the fixture's nested container is missing: {names:?}"
    );
    let engine = Engine::new();
    let scope = tree_scope();
    let mut budget = Budget::new(limits());
    let declaration = engine
        .list_class_declarations(&snapshot, &scope, &mut budget)
        .expect("the fixture's declarations are listable");
    assert_eq!(
        declaration.items.len() as u64,
        PINNED_CLASSES,
        "the fixture's declarations are the classes it holds"
    );
    // The workload's own statement of what the fixture holds: a whole-scope sweep delivers one
    // record per declared method, and that count is what W5/W6a's samples are read against.
    let config = Config {
        workload: "w6a".to_owned(),
        artifact: None,
        workers: 1,
        mode: Mode::Discard,
        capacity: Capacity::Roomy,
        instrumented: false,
        stop_after: None,
    };
    let sample = run_workload(&config).pop().expect("w6a prints a sample");
    assert_eq!(
        sample.numbers["classes_seen"], PINNED_CLASSES,
        "the sweep did not see the fixture's classes"
    );
    assert_eq!(
        sample.numbers["methods"], PINNED_METHODS,
        "the sweep delivered {} method records, and the pinned reading is {PINNED_METHODS}",
        sample.numbers["methods"]
    );
    assert_eq!(
        sample.numbers["methods"], sample.numbers["methods_declared"],
        "a fixture whose every declaration has a body delivers one record per declaration"
    );
}

/// The stage ledger's two rules: top-level phases are disjoint, and a nested stage never enters the
/// phase total.
#[test]
fn the_stage_ledger_keeps_phases_disjoint_and_nested_stages_out_of_the_total() {
    let mut stages = Stages::new();
    let mut order = Vec::new();
    stages.phase("open", || {
        order.push("open");
        std::thread::sleep(Duration::from_millis(1));
    });
    stages.phase("prepare", || order.push("prepare"));
    let mut inner = 0_u64;
    stages.phase("request", || {
        let started = Instant::now();
        inner = micros(started.elapsed());
    });
    stages.phase("output", || order.push("output"));
    stages.nested("sink_visit", "request", inner);
    stages.nested("sink_encode", "request", inner / 2);
    assert_eq!(order, ["open", "prepare", "output"]);
    let window = stages.window_micros();
    let top: u64 = stages.top.iter().map(|stage| stage.micros).sum();
    assert!(
        top <= window,
        "the top-level phases sum to {top} µs of a {window} µs window"
    );
    assert_eq!(
        stages.unattributed_micros(),
        window - top,
        "the unattributed remainder is the window minus the phases and nothing else"
    );
    assert!(
        stages.top.iter().all(|stage| stage.name != "sink_visit"),
        "a nested stage is not a top-level phase"
    );
    let parent = stages
        .parent_micros("request")
        .expect("the phase was recorded");
    assert!(
        stages.nested_micros("request") >= inner,
        "the nested readings are reported, not absorbed"
    );
    assert!(
        inner <= parent,
        "a stage measured inside `request` cannot exceed it"
    );
    // A nested stage whose parent does not exist is a misattribution and is refused.
    let refused = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut stages = Stages::new();
        stages.nested("sink_write", "nowhere", 1);
        stages
    }));
    assert!(refused.is_err(), "a nested stage must name a parent phase");
}

/// The three sink modes publish the same result, and pay three different layers.
///
/// Contract: the stream's semantic projection — order, identity, disposition, text — is one, so the
/// ablation is about what the sink does, not about what the operation delivers. Work: `encode` and
/// `write` really run, exactly once per record, and the file really holds the bytes.
#[test]
fn the_three_sink_modes_publish_the_same_result() {
    assert!(
        !Measuring::scratch_root().starts_with("/tmp"),
        "the write ablation must use cargo's own test temporary directory, not /tmp: {}",
        Measuring::scratch_root()
    );
    let samples = [Mode::Discard, Mode::Encode, Mode::Write]
        .into_iter()
        .map(|mode| {
            let config = Config {
                workload: "w6a".to_owned(),
                artifact: None,
                workers: 2,
                mode,
                capacity: Capacity::Roomy,
                instrumented: false,
                stop_after: None,
            };
            let sample = run_workload(&config).pop().expect("w6a prints a sample");
            (mode, sample)
        })
        .collect::<Vec<_>>();
    let domains = samples
        .iter()
        .map(|(_, sample)| sample.domain.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        domains[0], domains[1],
        "the counting sink and the encoding sink saw different streams"
    );
    assert_eq!(
        domains[1], domains[2],
        "the encoding sink and the writing sink saw different streams"
    );
    for (mode, sample) in &samples {
        let records = sample.numbers["records"];
        let methods = sample.numbers["methods"];
        assert!(records > 0 && methods > 0, "{mode:?} delivered nothing");
        assert!(
            methods <= PINNED_METHODS,
            "{mode:?} delivered {methods} methods, and the fixture declares {PINNED_METHODS}"
        );
    }
    let of = |mode: Mode| {
        samples
            .iter()
            .find(|(candidate, _)| *candidate == mode)
            .map(|(_, sample)| sample)
            .expect("every mode was run")
    };
    assert!(
        of(Mode::Discard).numbers["encoded_bytes"] == 0,
        "the counting sink encodes nothing"
    );
    assert!(
        of(Mode::Encode).numbers["encoded_bytes"] > 0,
        "the encoding sink really serializes every record"
    );
    assert!(
        of(Mode::Encode).numbers["written_bytes"] == 0,
        "the encoding sink writes no file"
    );
    assert!(
        of(Mode::Write).numbers["written_bytes"] > 0,
        "the writing sink really appends its lines"
    );
    assert_eq!(
        of(Mode::Write).numbers["written_bytes"],
        of(Mode::Write).numbers["scratch_file_bytes"],
        "the file the writing sink produced holds exactly what it wrote"
    );
    let nested = |sample: &Sample, name: &str| {
        sample
            .stages
            .nested
            .iter()
            .find(|stage| stage.name == name)
            .map_or(0, |stage| stage.micros)
    };
    assert!(
        nested(of(Mode::Discard), "sink_encode") == 0
            && nested(of(Mode::Discard), "sink_write") == 0,
        "the counting sink pays neither encoding nor writing"
    );
    assert!(
        nested(of(Mode::Encode), "sink_encode") > 0 && nested(of(Mode::Encode), "sink_write") == 0,
        "the encoding sink pays encoding alone"
    );
    assert!(
        nested(of(Mode::Write), "sink_write") > 0,
        "the writing sink pays the file write"
    );
}

/// The counted readings of one sample, as the work figures two instrumentations have to agree on.
///
/// Durations are deliberately not in it: the two runs are two runs, and only their *work* is what
/// this comparison is about.
fn counted(sample: &Sample) -> BTreeMap<&'static str, u64> {
    const COUNTED: [&str; 12] = [
        "records",
        "methods",
        "items",
        "pages",
        "returned_bytes",
        "method_bodies",
        "method_bodies_total",
        "class_headers",
        "class_headers_total",
        "archive_entries_total",
        "classes",
        "class_methods",
    ];
    COUNTED
        .into_iter()
        .filter_map(|name| sample.numbers.get(name).map(|value| (name, *value)))
        .collect()
}

/// The instrumentation changes no published result, and its own timings stay outside the reports.
#[test]
fn the_instrumentation_changes_no_domain_report() {
    let run = |instrumented: bool| {
        let config = Config {
            workload: "w3".to_owned(),
            artifact: None,
            workers: 1,
            mode: Mode::Discard,
            capacity: Capacity::Roomy,
            instrumented,
            stop_after: None,
        };
        run_workload(&config)
    };
    let quiet = run(false);
    let watched = run(true);
    assert_eq!(quiet.len(), watched.len());
    for (left, right) in quiet.iter().zip(watched.iter()) {
        assert_eq!(
            left.domain, right.domain,
            "{}: attaching the observation port changed the published result",
            left.name
        );
        // Every counted reading must be identical, with one named exception: the *length* of a
        // returned document embeds that run's own `elapsed_millis`, so a run whose clock crosses a
        // digit boundary returns a few bytes more (the same rule the corpus gates state for
        // `encoded_bytes`/`written_bytes`, and the reason G0 says lengths are not a fingerprint).
        // The allowance is bounded and stated instead of the field being dropped: the counted *work*
        // still has to match exactly.
        const LENGTH_ALLOWANCE: u64 = 64;
        let quiet_counts = counted(left);
        let watched_counts = counted(right);
        for (name, value) in &quiet_counts {
            let other = *watched_counts
                .get(name)
                .unwrap_or_else(|| panic!("{}: the two runs counted different fields", left.name));
            if *name == "returned_bytes" {
                assert!(
                    value.abs_diff(other) <= LENGTH_ALLOWANCE,
                    "{}: the returned length moved by {} bytes between the two instrumentations, \
                     which is more than the elapsed-digit allowance",
                    left.name,
                    value.abs_diff(other)
                );
            } else {
                assert_eq!(
                    *value, other,
                    "{}: the counted work differs between the two instrumentations",
                    left.name
                );
            }
        }
        for (name, document) in [
            ("usage", right.usage.as_ref()),
            ("counts", right.counts.as_ref()),
            ("probe", right.probe.as_ref()),
        ] {
            if let Some(document) = document {
                for forbidden in ["phase", "phases", "ledger", "startup", "stage", "stages"] {
                    let text = serde_json::to_string(document).expect("a document renders");
                    assert!(
                        !text.contains(&format!("\"{forbidden}\"")),
                        "{}: the {name} document carries the harness's `{forbidden}` reading",
                        left.name
                    );
                }
            }
        }
    }
    // The same contrast through the bulk stream, where the port is attached to a real operation.
    let export = |instrumented: bool| {
        let config = Config {
            workload: "w6a".to_owned(),
            artifact: None,
            workers: 2,
            mode: Mode::Encode,
            capacity: Capacity::Roomy,
            instrumented,
            stop_after: None,
        };
        run_workload(&config).pop().expect("w6a prints a sample")
    };
    let quiet = export(false);
    let watched = export(true);
    assert_eq!(
        quiet.domain, watched.domain,
        "the bulk stream's own fingerprint moved when the port was attached"
    );
    assert_eq!(quiet.numbers["records"], watched.numbers["records"]);
    #[cfg(feature = "test-support")]
    assert!(
        quiet.probe.is_none() && watched.probe.is_some(),
        "the port's reading is reported only where it was attached"
    );
    #[cfg(not(feature = "test-support"))]
    assert!(
        quiet.probe.is_none() && watched.probe.is_none(),
        "this build compiles the observation port out, so no sample of it may carry a reading"
    );
    #[cfg(feature = "test-support")]
    {
        let probe = watched.probe.as_ref().expect("a reading");
        assert!(
            probe["records_delivered"].as_u64().unwrap_or(0) > 0,
            "the attached port counted no deliveries: {probe}"
        );
        assert!(
            probe["ledger"].as_array().map(Vec::len).unwrap_or(0) > 0
                && probe["window"].as_array().map(Vec::len).unwrap_or(0) > 0,
            "the attached port kept neither its ledger nor its window sites: {probe}"
        );
    }
}

/// The port counts what the stream really did: every delivered record, and every window call.
///
/// This is the check a removed observation turns red: it reads the port's own figures against the
/// sink's, so deleting the counter (or the callback that feeds it) leaves a sample whose numbers
/// cannot both be true.
#[test]
#[cfg(feature = "test-support")]
fn the_probe_counts_every_delivered_record_and_window_call() {
    let config = Config {
        workload: "w6a".to_owned(),
        artifact: None,
        workers: 2,
        mode: Mode::Discard,
        capacity: Capacity::Roomy,
        instrumented: true,
        stop_after: None,
    };
    let sample = run_workload(&config).pop().expect("w6a prints a sample");
    let probe = sample.probe.as_ref().expect("the port was attached");
    let delivered = probe["records_delivered"].as_u64().expect("a count");
    assert_eq!(
        delivered, sample.numbers["records"],
        "the port counted {delivered} deliveries and the sink was handed {} records: one of the \
         two is not observing the stream",
        sample.numbers["records"]
    );
    let take_front = probe["window"]
        .as_array()
        .expect("window sites")
        .iter()
        .find(|site| site["site"] == "take_front")
        .expect("the coordinator's own site");
    assert!(
        take_front["calls"].as_u64().unwrap_or(0) > 0,
        "the operation's earliest-class site was never called: {take_front}"
    );
    let charge_methods = probe["ledger"]
        .as_array()
        .expect("ledger sites")
        .iter()
        .find(|site| site["site"] == "charge_methods")
        .expect("the method-owner charge site");
    assert!(
        charge_methods["calls"].as_u64().unwrap_or(0) > 0,
        "the total was never charged for the operation's own work: {charge_methods}"
    );
    assert!(
        probe["class_tasks"].as_u64().unwrap_or(0) > 0
            && probe["worker_threads"].as_u64().unwrap_or(0) == 2,
        "the two workers' own readings are missing: {probe}"
    );
    // The nested readings the probe keeps are children of the request, never a second claim on it.
    let request = sample
        .stages
        .parent_micros("request")
        .expect("the request phase");
    let sink_visit = sample
        .stages
        .nested
        .iter()
        .find(|stage| stage.name == "sink_visit")
        .expect("the sink's own stage");
    assert!(
        sink_visit.micros <= request,
        "the sink's own stage ({} µs) exceeds the request that contains it ({request} µs)",
        sink_visit.micros
    );
}

/// W4's pages really cover the scan, and the page size changes the pages and not the items.
#[test]
fn the_pages_cover_the_scan_and_the_page_size_changes_no_item() {
    let config = Config {
        workload: "w4".to_owned(),
        artifact: None,
        workers: 1,
        mode: Mode::Discard,
        capacity: Capacity::Roomy,
        instrumented: false,
        stop_after: None,
    };
    let samples = run_workload(&config);
    let small = samples
        .iter()
        .find(|sample| sample.name == "w4-small-page")
        .expect("the small-page sample");
    let large = samples
        .iter()
        .find(|sample| sample.name == "w4-large-page")
        .expect("the large-page sample");
    assert_eq!(
        small.texts["items_domain"], large.texts["items_domain"],
        "a page's size changed which items the query found"
    );
    assert_eq!(
        small.number_or_zero("items"),
        large.number_or_zero("items"),
        "the two page sizes delivered a different number of items"
    );
    assert!(
        small.number_or_zero("items") > 0,
        "the fixture query found nothing, so the comparison is vacuous"
    );
    assert!(
        small.number_or_zero("pages") > 1,
        "the small page did not need a second page, so continuation was not exercised"
    );
    assert_eq!(
        small.number_or_zero("has_more"),
        0,
        "a query continued to exhaustion must not still claim more items"
    );
    let multi = samples
        .iter()
        .find(|sample| sample.name == "w4-multi-consumer")
        .expect("the multi-consumer sample");
    assert!(
        multi.number_or_zero("items_equal_single_consumer") == 1,
        "asking for four consumers changed the items a constant-pool query presents"
    );
}

impl Sample {
    fn number_or_zero(&self, name: &str) -> u64 {
        self.numbers.get(name).copied().unwrap_or(0)
    }
}

/// One named sample of a run, which the verifiers read whole.
fn sample_named<'a>(samples: &'a [Sample], name: &str) -> &'a Sample {
    samples
        .iter()
        .find(|sample| sample.name == name)
        .unwrap_or_else(|| {
            panic!(
                "this run published no `{name}` sample; it published {:?}",
                samples.iter().map(|sample| sample.name).collect::<Vec<_>>()
            )
        })
}

/// W5's ablation: a store under capacity refuses retention and changes no result.
#[test]
fn the_capacity_refusal_changes_retention_and_not_the_result() {
    let run = |capacity: Capacity| {
        let config = Config {
            workload: "w5".to_owned(),
            artifact: None,
            workers: 2,
            mode: Mode::Discard,
            capacity,
            instrumented: false,
            stop_after: None,
        };
        run_workload(&config)
    };
    let tiny = run(Capacity::Tiny);
    let roomy = run(Capacity::Roomy);
    let tiny_sweep = sample_named(&tiny, "w5-sweep");
    let roomy_sweep = sample_named(&roomy, "w5-sweep");
    assert_eq!(
        tiny_sweep.domain, roomy_sweep.domain,
        "a store that could not retain an answer changed the sweep's own result"
    );
    assert_eq!(
        tiny_sweep.numbers["methods"], roomy_sweep.numbers["methods"],
        "the two capacities delivered a different number of methods"
    );
    let refused = |sample: &Sample| {
        sample.cache.as_ref().expect("the store's report")["refused_capacity"]
            .as_u64()
            .unwrap_or(0)
            + sample.cache.as_ref().expect("the store's report")["refused_capacity_bytes"]
                .as_u64()
                .unwrap_or(0)
    };
    assert!(
        refused(tiny_sweep) > 0,
        "the tiny store refused nothing, so the capacity ablation measured nothing: {:?}",
        tiny_sweep.cache
    );
    assert!(
        refused(roomy_sweep) == 0,
        "the roomy store refused retention of the fixture's own answers: {:?}",
        roomy_sweep.cache
    );
    // The round trip is where retention is supposed to show: the roomy store answers from what it
    // holds, the tiny one keeps answering from its own read.
    let tiny_round = sample_named(&tiny, "w5-round-trip");
    let roomy_round = sample_named(&roomy, "w5-round-trip");
    assert_eq!(
        tiny_round.texts["round_trip_domain"], roomy_round.texts["round_trip_domain"],
        "the two capacities recovered different text on the round trip"
    );
    let reading = |sample: &Sample, name: &str| {
        sample.cache.as_ref().expect("the store's report")[name]
            .as_u64()
            .unwrap_or(0)
    };
    assert!(
        reading(roomy_round, "container_hits") > reading(tiny_round, "container_hits"),
        "the roomy store answered no container lookup from retention: roomy {:?}, tiny {:?}",
        roomy_round.cache,
        tiny_round.cache
    );
    assert!(
        reading(tiny_round, "directory_parses") > reading(roomy_round, "directory_parses"),
        "the tiny store parsed no more container directories than the roomy one, so the ablation \
         measured no avoided work: roomy {:?}, tiny {:?}",
        roomy_round.cache,
        tiny_round.cache
    );
}

/// The worker sequence publishes one result: 1, 2, 4 and 6 workers agree on the stream.
#[test]
fn the_worker_sequence_publishes_one_result() {
    let run = |workers: usize| {
        let config = Config {
            workload: "w6a".to_owned(),
            artifact: None,
            workers,
            mode: Mode::Discard,
            capacity: Capacity::Roomy,
            instrumented: false,
            stop_after: None,
        };
        run_workload(&config).pop().expect("w6a prints a sample")
    };
    let one = run(1);
    for workers in [2, 4, 6] {
        let many = run(workers);
        assert_eq!(
            one.domain, many.domain,
            "{workers} workers published a different stream than one worker: the order, identity, \
             disposition or text of a record moved"
        );
        assert_eq!(
            one.numbers["methods"], many.numbers["methods"],
            "{workers} workers delivered a different number of methods"
        );
        assert_eq!(
            one.texts["status"], many.texts["status"],
            "{workers} workers ended in a different state"
        );
        assert!(
            many.numbers["first_result_micros"] > 0,
            "{workers} workers never delivered a first result"
        );
    }
}

/// The two delivery shapes answer the same bodies: grouping is not a result.
///
/// O4's question is whether asking one class at a time (one preparation serving its members) is the
/// same answer as asking one member at a time, and whether it is the same *work*. This gate holds the
/// first half: the per-member decode facts the two arms publish are one set. The work half is a
/// reading, not an assertion — the campaign measures both arms and the evidence compares them.
#[test]
fn the_two_delivery_shapes_answer_the_same_bodies() {
    let config = Config {
        workload: "w3".to_owned(),
        artifact: None,
        workers: 1,
        mode: Mode::Discard,
        capacity: Capacity::Roomy,
        instrumented: false,
        stop_after: None,
    };
    let samples = run_workload(&config);
    let class_major = sample_named(&samples, "w3-batch-per-class-across-classes");
    let method_major = sample_named(&samples, "w3-per-method-across-classes");
    assert!(
        class_major.number_or_zero("bodies") > 0,
        "neither shape decoded a body, so the comparison is vacuous"
    );
    assert_eq!(
        class_major.number_or_zero("bodies"),
        method_major.number_or_zero("bodies"),
        "the two shapes answered a different number of bodies"
    );
    assert_eq!(
        class_major.texts["arm_domain"], method_major.texts["arm_domain"],
        "asking one class at a time changed a decoded body: the two shapes' member decodes differ"
    );
    assert!(
        class_major.number_or_zero("sequence_micros") > 0
            && method_major.number_or_zero("sequence_micros") > 0,
        "an arm published no duration, so the arms were not both measured"
    );
}

/// A page that stops before a damaged suffix states what it covered, and one that reaches it states
/// the damage.
///
/// The three arms are one query over one damaged archive, stopped at three points: one small page,
/// one full page, and the small-page sequence continued to exhaustion. The contract the task fixes is
/// that an unread suffix is *honest* — not covered, not claimed complete — and that continuing reaches
/// the same result the full walk reaches.
#[test]
fn a_page_that_stops_before_a_damaged_suffix_states_what_it_covered() {
    let config = Config {
        workload: "w4".to_owned(),
        artifact: None,
        workers: 1,
        mode: Mode::Discard,
        capacity: Capacity::Roomy,
        instrumented: false,
        stop_after: None,
    };
    let samples = run_workload(&config);
    let intact = sample_named(&samples, "w4-large-page");
    let first = sample_named(&samples, "w4-damaged-first-page");
    let full = sample_named(&samples, "w4-damaged-full-page");
    let exhausted = sample_named(&samples, "w4-damaged-continued-to-exhaustion");

    assert!(
        intact.number_or_zero("diagnostics") == 0,
        "the intact fixture stated a diagnostics and cannot serve as the control: {:?}",
        intact.texts.get("diagnostic_codes")
    );
    assert!(
        intact.number_or_zero("items") > full.number_or_zero("items"),
        "damaging the nested container changed no item, so the damage is not observable at all: \
         intact {} items, damaged {} items",
        intact.number_or_zero("items"),
        full.number_or_zero("items")
    );
    assert!(
        full.number_or_zero("diagnostics") > 0,
        "the full walk reached the damaged container and stated nothing"
    );
    assert!(
        !full.texts["diagnostic_codes"].is_empty(),
        "the full walk's diagnostics carry no code"
    );

    // The early stop really stopped: it covered less than the walk and says there is more.
    assert_eq!(
        first.number_or_zero("pages"),
        1,
        "the early arm read more than the one page it asked for"
    );
    assert_eq!(
        first.number_or_zero("has_more"),
        1,
        "the early stop claimed the search was over"
    );
    assert!(
        first.number_or_zero("scanned_items") < full.number_or_zero("scanned_items"),
        "the early stop scanned as much as the full walk"
    );
    assert_eq!(
        first.number_or_zero("diagnostics"),
        0,
        "a page that stopped before the damaged byte stated a diagnostic about it"
    );

    // Continuing from the small page reaches the full walk's own answer, damage and all. The
    // damaged entry ends the walk, so the continued sequence still states `has_more`: the honest
    // reading of "the search did not reach the end of the range" rather than a complete-looking set
    // of items (the boundary documents itself that way — `has_more` is `stopped_early || issue`).
    assert!(
        exhausted.number_or_zero("pages") > 1,
        "the continued arm needed no second page, so continuing was not exercised"
    );
    assert_eq!(
        exhausted.number_or_zero("items_equal_full_page"),
        1,
        "continuing the small pages did not reach the same items the full page did"
    );
    assert_eq!(
        exhausted.texts["items_domain"], full.texts["items_domain"],
        "the continued small-page sequence and the full page disagreed on the items"
    );
    assert_eq!(
        exhausted.texts["diagnostic_codes"], full.texts["diagnostic_codes"],
        "the continued small-page sequence and the full page disagreed on the diagnostics"
    );
    assert_eq!(
        exhausted.number_or_zero("has_more"),
        1,
        "the walk stopped at a damaged entry and still claimed it reached the end of the range"
    );
    let intact_small = sample_named(&samples, "w4-small-page");
    assert_eq!(
        intact_small.number_or_zero("has_more"),
        0,
        "the intact fixture's own small-page sequence did not reach the end of the range"
    );
}

/// A stopped export delivers its confirmed prefix and nothing else.
///
/// This is O7's cancellation reading as a gate: the sink confirms `n` method records and answers
/// `Stop`, and the run really stops there — no terminal event, no record the callback refused, and the
/// same configuration without the stop takes the whole stream.
#[test]
fn the_stopped_export_delivers_its_confirmed_prefix_and_stops() {
    let run = |stop_after: Option<u64>| {
        let config = Config {
            workload: "w6a".to_owned(),
            artifact: None,
            workers: 2,
            mode: Mode::Discard,
            capacity: Capacity::Roomy,
            instrumented: false,
            stop_after,
        };
        run_workload(&config).pop().expect("w6a prints a sample")
    };
    let whole = run(None);
    let stopped = run(Some(2));
    assert_eq!(
        whole.number_or_zero("methods"),
        PINNED_METHODS,
        "the uninterrupted run did not take the whole stream"
    );
    assert_eq!(
        whole.number_or_zero("final_delivered"),
        1,
        "the uninterrupted run published no terminal event"
    );
    assert_eq!(
        stopped.number_or_zero("stopped_by_sink"),
        1,
        "the sink asked to stop and the sample does not state that it did"
    );
    assert_eq!(
        stopped.number_or_zero("methods"),
        2,
        "the stopped run delivered more than the prefix its sink confirmed"
    );
    assert_eq!(
        stopped.number_or_zero("methods_confirmed"),
        2,
        "the sink confirmed a different number of methods than the run delivered"
    );
    assert_eq!(
        stopped.number_or_zero("final_delivered"),
        0,
        "a stopped stream published the terminal event of a stream that reached its end"
    );
    // The operation's own account of the same stop: the record whose callback answered
    // `SinkControl::Stop` was handed over and is not counted as a `Continue` confirmation, so the
    // run states an unfinished account rather than a complete one (`delivered` counts confirmations
    // that let the operation go on; `src/bulk.rs::publish`). What matters as contract is that the
    // stopped run does not claim to have taken the stream:
    assert!(
        stopped.number_or_zero("delivered") < stopped.number_or_zero("methods"),
        "the stopped run counted every handed-over record as a confirmed delivery"
    );
    assert!(
        stopped.number_or_zero("delivered") + stopped.number_or_zero("not_executed")
            < stopped.number_or_zero("methods_declared"),
        "the stopped run's own account adds up to everything it declared"
    );
    assert_eq!(
        stopped.number_or_zero("traversal_complete"),
        0,
        "a stopped run claims it walked its whole declared range"
    );
    assert!(
        !stopped.texts["status"].is_empty(),
        "the stopped run stated no status"
    );
}

/// The zero-capacity store retains nothing and changes no result.
///
/// The floor of the capacity ablation: a store declared to hold nothing answers every consultation
/// with a miss, so every request pays the direct path, and the results are the ones the direct path
/// produces.
#[test]
fn the_zero_capacity_store_retains_nothing_and_changes_no_result() {
    let run = |capacity: Capacity| {
        let config = Config {
            workload: "w5".to_owned(),
            artifact: None,
            workers: 2,
            mode: Mode::Discard,
            capacity,
            instrumented: false,
            stop_after: None,
        };
        run_workload(&config)
    };
    let none = run(Capacity::None);
    let roomy = run(Capacity::Roomy);
    let none_sweep = sample_named(&none, "w5-sweep");
    let roomy_sweep = sample_named(&roomy, "w5-sweep");
    assert_eq!(
        none_sweep.domain, roomy_sweep.domain,
        "a store that holds nothing changed the sweep's own result"
    );
    let none_round = sample_named(&none, "w5-round-trip");
    let roomy_round = sample_named(&roomy, "w5-round-trip");
    assert_eq!(
        none_round.texts["round_trip_domain"], roomy_round.texts["round_trip_domain"],
        "the two stores recovered different results on the round trip"
    );
    let reading = |sample: &Sample, name: &str| {
        sample.cache.as_ref().expect("the store's report")[name]
            .as_u64()
            .unwrap_or(0)
    };
    assert_eq!(
        reading(none_round, "containers"),
        0,
        "a store declared to hold no entry retained a container"
    );
    assert_eq!(
        reading(none_round, "retained_bytes"),
        0,
        "a store declared to hold no byte retained bytes"
    );
    assert_eq!(
        reading(none_round, "container_hits"),
        0,
        "a store that holds nothing answered a container lookup"
    );
    assert!(
        reading(none_round, "refused_capacity") + reading(none_round, "refused_capacity_bytes") > 0,
        "the zero-capacity store refused no insertion, so nothing was attempted: {:?}",
        none_round.cache
    );
    assert!(
        reading(roomy_round, "container_hits") > 0,
        "the roomy store answered no container lookup, so the contrast is not the capacity"
    );
    assert!(
        reading(none_round, "container_consultations") > 0,
        "the zero-capacity store was never consulted, so its own cost was not measured"
    );
}

/// The phase ledger's own names are this harness's, and the engine's sources do not carry them.
/// The guard is the "phase time does not enter the domain report" half of task 1.3 that a run
/// cannot show: no reading of a published document can prove that a *later* report will not carry a
/// timing, so this reads the sources the reports are built in. It is a source guard of the same
/// shape as `p5_benchmark`'s: coarse, and its self-check is that its needles really appear in the
/// file that is allowed to hold them.
#[test]
fn the_domain_reports_carry_no_phase_timing() {
    /// The names this harness's ledger uses, and that no engine source may name.
    const PHASE_NEEDLES: [&str; 9] = [
        "Stages",
        "TopStage",
        "NestedStage",
        "window_micros",
        "unattributed_micros",
        "sink_visit",
        "sink_encode",
        "sink_write",
        "first_result_micros",
    ];
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut directories = vec![root.join("src")];
    let crates = root.join("crates");
    let mut crate_directories = std::fs::read_dir(&crates)
        .expect("the workspace's crates are listable")
        .map(|entry| entry.expect("a directory entry").path())
        .collect::<Vec<_>>();
    crate_directories.sort();
    for directory in crate_directories {
        directories.push(directory.join("src"));
    }
    let mut files = 0_usize;
    let mut hits = Vec::new();
    for directory in directories {
        collect_rust_sources(&directory, &mut files, &mut hits, &PHASE_NEEDLES);
    }
    assert!(
        files > 20,
        "the guard read only {files} engine sources, so it is not looking at the engine"
    );
    assert!(
        hits.is_empty(),
        "an engine source names the harness's phase readings, so a stage time may be reaching a \
         report: {hits:?}"
    );
    // The self-check: the needles are this harness's, and they are really here.
    let harness = std::fs::read_to_string(root.join("tests/p5_optimize_workloads.rs"))
        .expect("this harness is readable");
    for needle in PHASE_NEEDLES {
        assert!(
            harness.contains(needle),
            "the guard's needle `{needle}` is not in this harness either, so it guards nothing"
        );
    }
}

fn collect_rust_sources(
    directory: &std::path::Path,
    files: &mut usize,
    hits: &mut Vec<String>,
    needles: &[&str],
) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let mut paths = entries
        .map(|entry| entry.expect("a directory entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            collect_rust_sources(&path, files, hits, needles);
            continue;
        }
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        *files += 1;
        let text = std::fs::read_to_string(&path).expect("a source file is readable");
        for needle in needles {
            if text.contains(needle) {
                hits.push(format!("{}: {needle}", path.display()));
            }
        }
    }
}
