//! One whole physical scope through the bulk operation, with the sink's work selectable.
//!
//! ```text
//! cargo run --release --example bulk_scope_sweep -- <artifact> [scope.json] [roots.json] [mode]
//! ```
//!
//! `mode` is what the sink does with every record the operation publishes:
//!
//! * `discard` — nothing but a count: the library's own work, its window and its encoding of one
//!   record never happen;
//! * `encode` — the record is serialized exactly as the CLI's stream serializes it and then
//!   dropped, so the serialization is paid and no byte is written anywhere;
//! * `write` — the encoded line is appended to a temporary file and confirmed, which is what the
//!   CLI does (minus `create_new` and the destination the caller chose).
//!
//! The three modes are one program on purpose: their difference is the answer to "what does the
//! rest of the pipeline cost beside the library's own work", and a comparison across programs
//! would put a second build and a second set of process starts into the measurement.
//!
//! Nothing here asserts a duration. The example prints one line of counts and one line of the
//! run's own `elapsed_millis`, and the caller decides what to compare.

use jarde::{
    ArtifactInput, ArtifactSnapshot, Budget, BulkDiagnosticEvent, BulkFinalEvent, BulkHeaderEvent,
    BulkRecoveryRequest, ClassEndEvent, ClassPreparedEvent, DeliveryAccount, Engine,
    EnvironmentPolicy, EnvironmentRequest, Error, LayoutMode, LoadRoot, LoaderId,
    MethodResultEvent, MultiReleasePolicy, PhysicalScope, RecoverySink, Result, RuntimeProfile,
    SinkControl, task_limits,
};
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Mode {
    #[default]
    Discard,
    Encode,
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
}

/// A sink that does exactly what its mode says and keeps the two facts worth printing: how many
/// records it was handed, and how many bytes its own encoding produced.
#[derive(Default)]
struct Counting {
    mode: Mode,
    records: u64,
    encoded_bytes: u64,
    destination: Option<std::fs::File>,
}

impl Counting {
    fn account(&mut self, record: &impl serde::Serialize) -> Result<SinkControl> {
        self.records += 1;
        if self.mode == Mode::Discard {
            return Ok(SinkControl::Continue);
        }
        let line = serde_json::to_vec(record).map_err(|error| Error::Io {
            operation: "example_encode".to_owned(),
            message: error.to_string(),
        })?;
        self.encoded_bytes += line.len() as u64;
        if let Some(file) = self.destination.as_mut() {
            file.write_all(&line).map_err(|error| Error::Io {
                operation: "example_write".to_owned(),
                message: error.to_string(),
            })?;
            file.write_all(b"\n").map_err(|error| Error::Io {
                operation: "example_write".to_owned(),
                message: error.to_string(),
            })?;
        }
        Ok(SinkControl::Continue)
    }
}

impl RecoverySink for Counting {
    fn header(
        &mut self,
        event: &BulkHeaderEvent,
        _delivery: DeliveryAccount,
    ) -> Result<SinkControl> {
        self.account(event)
    }
    fn class_prepared(&mut self, event: &ClassPreparedEvent) -> Result<SinkControl> {
        self.account(event)
    }
    fn method(&mut self, event: &MethodResultEvent) -> Result<SinkControl> {
        self.account(event)
    }
    fn class_end(&mut self, event: &ClassEndEvent) -> Result<SinkControl> {
        self.account(event)
    }
    fn diagnostic(&mut self, event: &BulkDiagnosticEvent) -> Result<SinkControl> {
        self.account(event)
    }
    fn final_event(&mut self, _event: &BulkFinalEvent) -> Result<SinkControl> {
        self.records += 1;
        Ok(SinkControl::Continue)
    }
}

fn roots(path: Option<&String>) -> Result<Vec<LoadRoot>> {
    match path {
        Some(path) => {
            let text = std::fs::read_to_string(path).map_err(|error| Error::Io {
                operation: "example_roots".to_owned(),
                message: error.to_string(),
            })?;
            serde_json::from_str(&text).map_err(|error| Error::InvalidInput {
                code: "example_roots_json".to_owned(),
                message: error.to_string(),
            })
        }
        None => Ok(Vec::new()),
    }
}

fn main() -> Result<()> {
    let mut arguments = std::env::args().skip(1);
    let artifact = arguments.next().unwrap_or_else(|| {
        "tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class".to_owned()
    });
    let scope_text = arguments.next();
    let roots_text = arguments.next();
    let mode = Mode::parse(&arguments.next().unwrap_or_else(|| "discard".to_owned()));
    let workers: usize = arguments
        .next()
        .and_then(|text| text.parse().ok())
        .unwrap_or(1);

    // One operation over a whole package is not one request's view: the counted dimensions are
    // raised to a ceiling a real scope fits, the way the CLI's own defaults do it.
    let overrides: Vec<jarde::BudgetOverride> = [
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
    .map(|dimension| jarde::BudgetOverride::new(dimension, 1 << 40))
    .chain([jarde::BudgetOverride::new("elapsed_millis", 3_600_000)])
    .collect::<Result<Vec<_>>>()?;
    let limits = jarde::task_limits(&overrides)?;
    let mut budget = Budget::new(limits);
    // The host holds the store, as the MCP design says it does and as the CLI's own defaults do: one
    // bounded store for the whole operation, so a container's directory is parsed once for the walk
    // rather than once per class that lives in it. Without it a whole-package walk still re-reads
    // every container it revisits, which is the resource shape the operation's caller decides.
    budget = budget.with_facts_cache(jarde::FactsCache::current(jarde::FactsCapacity::new(
        1 << 14,
        1 << 27,
    )));
    let engine = Engine::new();
    let snapshot = engine.open(ArtifactInput::Path(PathBuf::from(&artifact)), &mut budget)?;

    let scope = match scope_text {
        Some(text) => serde_json::from_str(&text).map_err(|error| Error::InvalidInput {
            code: "example_scope_json".to_owned(),
            message: error.to_string(),
        })?,
        None => PhysicalScope::SnapshotAll,
    };
    let declared = roots(roots_text.as_ref())?;
    let policy = if declared.is_empty() {
        EnvironmentPolicy::PlainJar
    } else {
        EnvironmentPolicy::ExplicitClasspath { roots: declared }
    };
    let environment = EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope: scope.clone(),
        policy,
        profile: RuntimeProfile {
            java_release: 8,
            multi_release: MultiReleasePolicy::Disabled,
            layout: LayoutMode::Generic,
        },
        loader: LoaderId("app".to_owned()),
    };

    let mut sink = Counting {
        mode,
        destination: (mode == Mode::Write).then(|| {
            std::fs::File::create(std::env::temp_dir().join("jarde-bulk-example.jsonl"))
                .expect("the example's own scratch file")
        }),
        ..Counting::default()
    };
    let request = BulkRecoveryRequest::for_scope(environment, workers, task_limits(&[])?);
    // The observation port is attached only in a build that has it (`--features test-support`): it
    // adds one JSON line stating what coordinating this run cost, and leaves the two lines above it
    // byte for byte what every earlier measurement read.
    #[cfg(feature = "test-support")]
    let probe = std::sync::Arc::new(jarde::BulkProbe::new());
    #[cfg(feature = "test-support")]
    let request = request.with_probe(probe.clone());
    let report = recovery(&engine, &snapshot, &request, &mut budget, &mut sink)?;

    println!(
        "records={} encoded_bytes={} classes={} methods={}/{} status={:?}",
        sink.records,
        sink.encoded_bytes,
        report.summary.classes_seen,
        report.summary.methods_delivered,
        report.summary.methods_declared,
        report.summary.status(),
    );
    println!(
        "usage: archive_entries={} class_bytes={} method_bodies={} ir_items={} analysis_steps={} elapsed_millis={}",
        report.usage.archive_entries,
        report.usage.class_bytes,
        report.usage.method_bodies,
        report.usage.ir_items,
        report.usage.analysis_steps,
        report.usage.elapsed_millis,
    );
    #[cfg(feature = "test-support")]
    println!(
        "probe={}",
        serde_json::to_string(&probe.reading()).expect("a probe reading serializes")
    );
    Ok(())
}

fn recovery(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    request: &BulkRecoveryRequest,
    budget: &mut Budget,
    sink: &mut Counting,
) -> Result<jarde::BulkRecoveryReport> {
    engine.recover_all(std::slice::from_ref(snapshot), request, budget, sink)
}
