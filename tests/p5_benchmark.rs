//! P5 tasks 1.2 and 1.3: the repeated direct-path baseline, and the report a second path is judged
//! by.
//!
//! P5 decides what to optimize from measurements (design decision 1) against a reference path that
//! stays comparable (decision 3), and a measurement is only a baseline while the bytes it read, the
//! request it ran and the result it published are all fixed. This file is that baseline:
//!
//! * it reads the corpus fingerprint of task 1.1 (`tests/fixtures/corpus-fingerprint.json`) and
//!   checks every file it is about to read against the digest recorded there, so a subject whose
//!   bytes moved fails the harness instead of quietly producing numbers nothing can be compared to;
//! * it runs each row of the matrix **repeatedly**, each repeat on a fresh snapshot and a fresh
//!   budget, and reports the spread instead of one sample;
//! * it defines the **result fingerprint** of a complete run — the serialized report with
//!   `elapsed_millis`, and only `elapsed_millis`, removed — which is what "the same result" means
//!   for the cold/warm half of A15, and it checks that the publication order is a function of the
//!   input alone, which is the baseline P5's decision 4 requires a parallel scheduler to preserve;
//! * it compares two runs field by field (status, fingerprint, published order, coverage,
//!   diagnostics, resources), which is the report shape a cache, index or parallel path will be
//!   judged with (task 1.3).
//!
//! ```text
//! verify:  cargo test --test p5_benchmark --locked
//! measure: cargo test --test p5_benchmark --locked -- --ignored --nocapture p5_repeated_direct_baseline
//! ```
//!
//! What the rows are, and what they are not
//! ---------------------------------------
//!
//! The repository has no cache, no index and no parallel scheduler: `cache` appears only in reader
//! doc comments that say a table is *not* cached. So `direct` is the only path that can be run, and
//! the `cache` and `parallel` columns of the matrix have no second implementation to compare
//! against. They are recorded here as **absent**, with the slice that owns each one, and no row is
//! given a "cache on" label: two labels over one code path would be a measurement-shaped claim that
//! no cache exists — the reading P5's design (5) and the `performance-gates` spec warn against.
//! What is reserved instead is the machinery: [`compare`] is written over *any* two [`RunRecord`]s
//! and its verdicts are [`Verdicts`], so the day a second path exists it is measured and compared by
//! the same code that measures the direct path today.
//!
//! There is equally no `cold`/`warm` pair in the sense the spec means (cache off against cache on).
//! What is measurable today is the spread of the uncached path across repeated runs and the
//! difference between the **first** run in a process and the ones after it; that spread is what a
//! later "the cache made it faster" claim has to clear, so it is reported rather than assumed away.
//!
//! Comparability, said once
//! ------------------------
//!
//! Every number here was taken on one machine with `RUST_TEST_THREADS=1` and a single-threaded
//! build, over the corpus the fingerprint fixes, in repeated runs of this harness. Nothing in this
//! file claims a speedup, a P95, a throughput, a memory figure or a cross-machine comparison: the
//! sample is the committed corpus, which is small, and the rows characterise the *shape* of the
//! direct path (what one request opens, reads, materializes and publishes) at that size. Timings are
//! printed by the measurement below and asserted nowhere, because a wall-clock assertion on a shared
//! machine is a flake, not a gate; what the always-on tests assert is the semantic half — identical
//! results, identical order, recorded resources, terminal status.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::sync::Arc;
use std::time::Instant;

use jarde::*;
use serde_json::{Map, Value, json};

// -------------------------------------------------------------------------------------------
// The corpus fingerprint of task 1.1, read rather than restated
// -------------------------------------------------------------------------------------------

/// The document task 1.1 wrote and verifies, relative to the repository root.
const MANIFEST: &str = "tests/fixtures/corpus-fingerprint.json";
/// The schema tag this file keys on. It refuses a document that does not carry it: a newer
/// fingerprint may reclassify files, and reading it under the older rules would be a silent guess.
const SCHEMA: &str = "jarde-corpus-fingerprint/1";
/// The command that rewrites the document, repeated here so a failure can name it.
const REGENERATE: &str =
    "cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint";

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// One digest the fingerprint recorded for one file.
#[derive(Clone, Debug)]
struct Recorded {
    bytes: u64,
    blake3: String,
}

/// The fingerprint document as this harness reads it: the schema, one version string for the whole
/// corpus, and the recorded digests.
#[derive(Debug)]
struct Corpus {
    schema: String,
    /// A single identity for "the corpus these numbers were taken over". The manifest cannot hold
    /// its own digest, so the consumer computes it — any change to the fingerprinted set moves this
    /// one string, and every row's context records it.
    version: String,
    files: BTreeMap<String, Recorded>,
    document: Value,
}

fn corpus() -> Corpus {
    let path = repository_root().join(MANIFEST);
    let text = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "read {}: {error}. The fingerprint is P5 1.1's and this file is a consumer of it; if it \
             is missing, regenerate it with `{REGENERATE}`.",
            path.display()
        )
    });
    let document: Value = serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("{MANIFEST} is not valid JSON: {error}"));
    let schema = document["schema"]
        .as_str()
        .unwrap_or_else(|| panic!("{MANIFEST} has no `schema` tag"))
        .to_string();
    let files = document["files"]
        .as_array()
        .unwrap_or_else(|| panic!("{MANIFEST} has no `files` array"))
        .iter()
        .map(|entry| {
            let path = entry["path"]
                .as_str()
                .unwrap_or_else(|| panic!("a `files` entry has no path: {entry}"))
                .to_string();
            let recorded = Recorded {
                bytes: entry["bytes"]
                    .as_u64()
                    .unwrap_or_else(|| panic!("{path} has no byte count in {MANIFEST}")),
                blake3: entry["blake3"]
                    .as_str()
                    .unwrap_or_else(|| panic!("{path} has no digest in {MANIFEST}"))
                    .to_string(),
            };
            (path, recorded)
        })
        .collect();
    Corpus {
        schema,
        version: format!("blake3:{}", blake3::hash(text.as_bytes()).to_hex()),
        files,
        document,
    }
}

/// A digest shortened for a report line. The full value stays in the manifest.
fn short(digest: &str) -> String {
    digest.chars().take(16).collect()
}

// -------------------------------------------------------------------------------------------
// The subjects of the matrix: the corpus files the rows read
// -------------------------------------------------------------------------------------------

/// Where the fingerprint classifies a subject. A subject whose file is dropped from that list is
/// not merely unpinned: by task 1.1's rule, it is no longer corpus at all, and a benchmark still
/// reading it would be measuring outside the fingerprint.
enum Classification {
    /// An acceptance row's own corpus list (`acceptance_rows[<id>].corpus[]`).
    Row(&'static str),
    /// A corpus dimension's carrier list (`dimensions[<key>].carriers[]`).
    Dimension(&'static str),
}

impl Classification {
    fn describe(&self) -> String {
        match self {
            Self::Row(id) => format!("acceptance row {id}"),
            Self::Dimension(key) => format!("corpus dimension `{key}`"),
        }
    }

    /// Whether the fingerprint document names `path` as a file carrier of this owner.
    fn names(&self, document: &Value, path: &str) -> bool {
        let carriers = match self {
            Self::Row(id) => document["acceptance_rows"][id]["corpus"].clone(),
            Self::Dimension(key) => document["dimensions"][key]["carriers"].clone(),
        };
        carriers.as_array().is_some_and(|carriers| {
            carriers.iter().any(|carrier| {
                matches!(carrier["kind"].as_str(), Some("file" | "golden"))
                    && carrier["path"].as_str() == Some(path)
            })
        })
    }
}

/// One corpus file a row reads.
struct Subject {
    path: &'static str,
    classified_under: Classification,
    /// What the rows do with the bytes.
    role: &'static str,
}

/// The two subjects of the matrix. Both are read by more than one row: the local row and the
/// full-range row it is compared against run over the *same* bytes, which is what the
/// `measured-execution` spec's local-versus-full-range scenario requires ("在相同语料上运行").
static SUBJECTS: [Subject; 2] = [
    Subject {
        path: "fuzz/corpus/query/minimal-jar",
        // A15's own corpus list names this archive, so the cold/warm half of that row is measured
        // over the bytes A15 names rather than over a fixture chosen for convenience.
        classified_under: Classification::Row("A15"),
        role: "a committed archive: one class, one service entry and a manifest behind a central \
               directory",
    },
    Subject {
        path: "tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class",
        // The recovery dimension is where the single-method boundary is carried (P3's entry reads
        // one body of this class), and the same bytes feed the full-range contrast.
        classified_under: Classification::Dimension("recovery"),
        role: "a committed standalone class file with several members that declare a body",
    },
];

/// One subject's bytes, checked against the fingerprint before any row reads them.
struct Verified {
    subject: &'static Subject,
    bytes: u64,
    blake3: String,
    content: Arc<[u8]>,
}

/// Read one subject and prove it is the file the fingerprint fixes.
///
/// The bytes the rows run over are the bytes read here, so "the fingerprint was checked" and "the
/// run read those bytes" cannot come apart: a rewritten fixture is a failure, not a moved baseline.
fn verified(corpus: &Corpus, subject: &'static Subject) -> Verified {
    let recorded = corpus.files.get(subject.path).unwrap_or_else(|| {
        panic!(
            "{} is a benchmark subject but is not in {MANIFEST}'s file list, so its bytes are not \
             fixed by task 1.1's fingerprint. If it moved, point this file at the new path; if it \
             is new corpus, add it to the fingerprint and regenerate ({REGENERATE}) instead of \
             measuring bytes nothing pins.",
            subject.path
        )
    });
    let path = repository_root().join(subject.path);
    let content =
        fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    assert_eq!(
        content.len() as u64,
        recorded.bytes,
        "{} is {} bytes on disk and {} bytes in the fingerprint",
        subject.path,
        content.len(),
        recorded.bytes
    );
    let digest = blake3::hash(&content).to_hex().to_string();
    assert_eq!(
        digest, recorded.blake3,
        "{} has moved under the fingerprint (recorded blake3 {}, now blake3 {}). A measurement \
         taken over changed bytes is not comparable to the baseline it is filed against; fix the \
         fixture, or re-record the fingerprint as an intended corpus change ({REGENERATE}).",
        subject.path, recorded.blake3, digest
    );
    Verified {
        subject,
        bytes: recorded.bytes,
        blake3: recorded.blake3.clone(),
        content: content.into(),
    }
}

// -------------------------------------------------------------------------------------------
// The requests each row runs
// -------------------------------------------------------------------------------------------

/// The limits every run starts from: the headroom of an ordinary request over this corpus, not a
/// probe. `elapsed_millis` is left unbounded because a wall-clock bound would turn a loaded machine
/// into a different result, and the clock is a measurement rather than a charge.
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
        class_headers: 1_000,
        method_bodies: 1_000,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

/// Every consumer category, so the full-range row is the broadest structural scan the relation
/// admits rather than one category that happens to be cheap.
const ALL_CONSUMERS: [ConsumerKind; 13] = [
    ConsumerKind::Invocation,
    ConsumerKind::Field,
    ConsumerKind::Type,
    ConsumerKind::Constant,
    ConsumerKind::Exception,
    ConsumerKind::Signature,
    ConsumerKind::Annotation,
    ConsumerKind::InnerNest,
    ConsumerKind::Module,
    ConsumerKind::Bootstrap,
    ConsumerKind::Resource,
    ConsumerKind::Verification,
    ConsumerKind::Debug,
];

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

/// The full-range row: `SnapshotAll` in the request's physical view, so the scope the report
/// publishes is the one the scan really walked.
fn full_range_request(snapshot: &ArtifactSnapshot) -> QueryRequest {
    QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        // Every class file names its superclass, so the target is a symbol this corpus states by
        // construction: the row cannot silently become a scan that found nothing.
        target: QueryTarget::Symbol {
            value: SymbolRef::Class {
                owner: bytes(b"java/lang/Object"),
            },
        },
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: ConsumerSchema::new(1, ALL_CONSUMERS),
        max_items: 0,
        cursor: None,
    }
}

/// The caller domain the single-member row runs under: the subject's own snapshot and nothing else,
/// which is the simplest environment the validator accepts without a problem.
fn single_member_environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Snapshot {
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

/// The member the single-member row presents, and the class bytes its identity is built from — the
/// value the premise header read located, never a digest this file invented.
struct MemberTarget {
    owner: ClassBytesId,
    declared_bodies: usize,
}

/// Locate `add(II)I` through the reader's own header entry, charged to its own budget.
///
/// This is harness work, not request work: a caller that already knows the member name does not pay
/// it. It runs outside every measured section, and the one body it charges is recorded beside the
/// row so the numbers cannot be read as "the request read the header twice".
fn locate_member(content: &Arc<[u8]>) -> MemberTarget {
    let mut budget = Budget::new(limits());
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(content.clone()), &mut budget)
        .expect("the subject opens as a standalone CLASS, as the fingerprint fixes it");
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the subject's own header is readable");
    let declared: Vec<Vec<u8>> = inspected
        .inspection
        .header
        .methods
        .iter()
        .filter(|member| {
            member
                .attributes
                .iter()
                .any(|shell| shell.name.raw().0.as_slice() == b"Code")
        })
        .map(|member| member.name.raw().0.clone())
        .collect();
    assert!(
        declared.iter().any(|name| name.as_slice() == b"add"),
        "the subject must still declare `add`, which is the member this row presents; it declares \
         {:?}",
        declared
            .iter()
            .map(|name| String::from_utf8_lossy(name).into_owned())
            .collect::<Vec<_>>()
    );
    assert!(
        declared.len() >= 3,
        "the single-member boundary is only observable on a class that declares more than one body: \
         {} declares {}",
        SUBJECTS[1].path,
        declared.len()
    );
    MemberTarget {
        owner: inspected.source.class_bytes,
        declared_bodies: declared.len(),
    }
}

fn single_member_request(
    snapshot: &ArtifactSnapshot,
    target: &MemberTarget,
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment: single_member_environment(snapshot),
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: target.owner.clone(),
                variant: PhysicalVariant::Base,
            },
            name: bytes(b"add"),
            descriptor: bytes(b"(II)I"),
        },
        stages: AnalysisStage::ALL.to_vec(),
    }
}

// -------------------------------------------------------------------------------------------
// The result fingerprint (task 1.3)
// -------------------------------------------------------------------------------------------

/// Delete every `elapsed_millis` at any depth.
///
/// This is the **only** normalization the fingerprint performs, and it is the repository's existing
/// one: `elapsed_millis` is the wall clock the budget re-reads on every charge, so it is a
/// measurement of the run rather than a result of it, and two runs of one request are equal exactly
/// when they agree everywhere else (the same rule `tests/p2_golden.rs` and `tests/p1_xref_golden.rs`
/// compare their documents under). Deleting anything else would let a changed result read as equal.
///
/// Returns how many fields were deleted, so a caller can assert the document really carried one
/// instead of passing because the field never existed.
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

/// The digest of a normalized document: the canonical rendering is `serde_json`'s, whose object
/// keys are ordered, so the same document always renders to the same bytes.
fn fingerprint_of(document: &Value) -> String {
    format!(
        "blake3:{}",
        blake3::hash(
            serde_json::to_string(document)
                .expect("a JSON value renders")
                .as_bytes()
        )
        .to_hex()
    )
}

/// A published report in the two forms the fingerprint tests need.
///
/// The raw document is kept, not only its digest: what the normalization *does* is a claim about a
/// document, and a hash cannot be asked what it was computed over — the test below has to stamp a
/// different clock onto a real published report and show the digest does not move.
struct Published {
    /// The report as it was serialized, `elapsed_millis` included.
    raw: Value,
    /// The same document with every `elapsed_millis` removed: the fingerprint's input.
    normalized: Value,
    fingerprint: String,
}

fn published<T: serde::Serialize>(report: &T) -> Published {
    let raw = serde_json::to_value(report).expect("a report serializes");
    let mut normalized = raw.clone();
    let removed = strip_elapsed(&mut normalized);
    assert!(
        removed >= 1,
        "a published report carries `elapsed_millis` at least once (the usage snapshot of its \
         execution report); this one carries it {removed} times, so the normalization is not \
         measuring what it claims to"
    );
    let fingerprint = fingerprint_of(&normalized);
    Published {
        raw,
        normalized,
        fingerprint,
    }
}

// -------------------------------------------------------------------------------------------
// The measurement context and one run's record
// -------------------------------------------------------------------------------------------

/// The context the `measured-execution` spec requires a benchmark to record: input fingerprint,
/// snapshot/view/profile identity, registry and recovery configuration, query scope, budget,
/// concurrency and cache state.
///
/// It is **derived from the request the run performed and the fingerprint it read**, never typed
/// beside them, so a request change moves the recorded context with it. Every knob this engine does
/// not have is recorded as absent together with the slice that owns it; a context that simply left
/// them out would let a reader assume they were held fixed.
struct Context {
    corpus_version: String,
    subject: &'static str,
    subject_bytes: u64,
    subject_blake3: String,
    /// The measured path. `direct` is the only one that exists (`design.md` decision 3 makes it the
    /// comparable reference); the absent ones are named below rather than simulated.
    path: &'static str,
    cache: &'static str,
    concurrency: &'static str,
    /// The scope actually requested, as the request states it.
    scope: String,
    /// The physical view the request names, serialized, so snapshot and scope travel together.
    view: String,
    /// The runtime profile, or the reason this entry has none.
    profile: String,
    /// The environment's provider list, or the reason this entry has none: a provider is a content
    /// source the request names, and a context that omitted the count would hide that none did.
    providers: String,
    /// What the recovery layer is doing in this row, and its gate input when it runs.
    recovery: String,
    /// Harness work excluded from the measured section, if any.
    premise: String,
    /// The limits every repeat started from.
    budget: Limits,
}

impl Context {
    fn lines(&self) -> Vec<String> {
        vec![
            format!(
                "corpus {} = {} ({} bytes)",
                self.corpus_version, self.subject, self.subject_bytes
            ),
            format!("subject blake3 {}", short(&self.subject_blake3)),
            format!("path {}", self.path),
            format!("cache {}", self.cache),
            format!("concurrency {}", self.concurrency),
            format!("scope {}", self.scope),
            format!("view {}", self.view),
            format!("profile {}", self.profile),
            format!("providers {}", self.providers),
            format!("recovery {}", self.recovery),
            format!("premise {}", self.premise),
            format!(
                "budget {} counted dimensions, class_headers {}, method_bodies {}, elapsed unbounded",
                CountedBudgetDimension::ALL.len(),
                self.budget.class_headers,
                self.budget.method_bodies
            ),
        ]
    }
}

/// One run of one request: what the request charged, what it published, and what the harness's own
/// clock observed around it.
struct RunRecord {
    label: &'static str,
    context: Context,
    /// The termination status of the run's own report, as published.
    status: &'static str,
    /// Everything the request charged, read from the run's budget.
    usage: UsageSnapshot,
    /// The same figure as the run's own report states it. The two coincide for the query rows and
    /// do not coincide for the local row; see [`outside_report`].
    report_usage: UsageSnapshot,
    /// The published coverage document (three dimensions for the local row, the query layer's
    /// wrapper for the query rows), or `None` when the entry publishes none.
    coverage: Option<Value>,
    diagnostics: Value,
    /// The identities the run published, in publication order.
    order: String,
    published_items: u64,
    /// The committed bodies the premise header read found beside the one this row presents.
    declared_bodies: usize,
    /// The report as it was serialized, and the normalization of it the fingerprint is taken over.
    published: Value,
    document: Value,
    fingerprint: String,
    /// The harness's own high-resolution clock around the whole run. The report's own clock is
    /// whole milliseconds, so on a corpus this size it is zero in every repeat and cannot carry a
    /// distribution; the harness measures its own while the report's stays the value A15's
    /// normalization is defined over.
    wall_micros: u128,
}

/// A usage snapshot with the one measurement field zeroed, for comparing two runs' charges.
fn charges(usage: &UsageSnapshot) -> UsageSnapshot {
    let mut copy = usage.clone();
    copy.elapsed_millis = 0;
    copy
}

/// What the request charged beyond what its own report states, by dimension.
///
/// The two figures coincide for the query rows: `Engine::query`'s report is the whole request's
/// accounting, and the harness asserts that rather than assuming it. They do **not** coincide for
/// the local row: `Engine::recover_method` presents the payload of the analysis run on the same
/// budget, so the presentation's own `output_bytes`, `ir_items` and `analysis_steps` are charged
/// after the report the entry publishes was assembled from the run. Recording the difference is what
/// keeps the two figures from being confused for one: a resource comparison that read
/// `analysis.execution.usage` alone would understate the local path, which is why [`compare`] takes
/// the request's total charge on both sides.
fn outside_report(row: &RunRecord) -> Vec<(CountedBudgetDimension, i128)> {
    deltas_between(&charges(&row.report_usage), &charges(&row.usage))
}

/// `right - left`, per counted dimension, in the declared order.
fn deltas_between(
    left: &UsageSnapshot,
    right: &UsageSnapshot,
) -> Vec<(CountedBudgetDimension, i128)> {
    CountedBudgetDimension::ALL
        .iter()
        .map(|dimension| {
            (
                *dimension,
                i128::from(right.counted_usage(*dimension))
                    - i128::from(left.counted_usage(*dimension)),
            )
        })
        .collect()
}

/// The non-zero entries of a delta list, as one line; `nothing charged differently` when empty.
fn delta_line(deltas: &[(CountedBudgetDimension, i128)]) -> String {
    let moved: Vec<String> = deltas
        .iter()
        .filter(|(_, delta)| *delta != 0)
        .map(|(dimension, delta)| format!("{} {:+}", dimension_name(*dimension), delta))
        .collect();
    if moved.is_empty() {
        "nothing charged differently".to_string()
    } else {
        moved.join(", ")
    }
}

/// The status name of an execution report, as the published tag spells it.
fn status_of(report: &ExecutionReport) -> &'static str {
    match report {
        ExecutionReport::Complete { .. } => "complete",
        ExecutionReport::Partial { .. } => "partial",
        ExecutionReport::Cancelled { .. } => "cancelled",
        ExecutionReport::Failed { .. } => "failed",
    }
}

fn usage_of(report: &ExecutionReport) -> UsageSnapshot {
    match report {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage.clone(),
    }
}

// -------------------------------------------------------------------------------------------
// The three rows
// -------------------------------------------------------------------------------------------

/// One full-range `SnapshotAll` query over the subject, open included in the measured section.
///
/// The unit of measurement is "open the artifact and perform the entry", because that is what a
/// caller pays for a request; the token, when given, is cancelled after the open so the row records
/// the entry's own cancellation answer over an artifact it really opened (a token cancelled before
/// `open` would make `open` itself fail, which measures the artifact layer rather than the query).
fn run_full_range(
    corpus: &Corpus,
    verified: &Verified,
    label: &'static str,
    cancelled: bool,
) -> RunRecord {
    let engine = Engine::new();
    let mut budget = Budget::new(limits());
    let token = budget.cancellation_token();
    let started = Instant::now();
    let snapshot = engine
        .open(ArtifactInput::bytes(verified.content.clone()), &mut budget)
        .expect("the subject opens");
    let request = full_range_request(&snapshot);
    if cancelled {
        token.cancel();
    }
    let report = engine
        .query(&snapshot, &request, &mut budget)
        .expect("a legal query is answered, not raised");
    let wall_micros = started.elapsed().as_micros();
    let form = published(&report);
    let mut context = context_of(corpus, verified);
    context.scope = serde_json::to_string(&request.physical.scope).expect("a scope serializes");
    context.view = serde_json::to_string(&request.physical).expect("a view serializes");
    context.profile =
        "none: this entry takes no environment, so no runtime profile is named (P5's cache key \
         would have to say so too)"
            .to_string();
    context.providers =
        "none: this entry takes no environment, so no content source is named by it".to_string();
    context.recovery =
        "not in this path: `Engine::query` reaches `jarde-query`, which does not depend on \
         `jarde-java`"
            .to_string();
    context.premise = "none: the whole measured section is the request".to_string();
    RunRecord {
        label,
        context,
        status: status_of(&report.execution),
        usage: budget.usage(),
        report_usage: usage_of(&report.execution),
        coverage: Some(serde_json::to_value(&report.coverage).expect("coverage serializes")),
        diagnostics: serde_json::to_value(&report.diagnostics).expect("diagnostics serialize"),
        order: item_identities(&report.items),
        published_items: report.items.len() as u64,
        declared_bodies: 0,
        published: form.raw,
        document: form.normalized,
        fingerprint: form.fingerprint,
        wall_micros,
    }
}

/// One single-member request over the subject, open included and the member located beforehand.
fn run_single_member(
    corpus: &Corpus,
    verified: &Verified,
    label: &'static str,
    premise: &MemberTarget,
) -> RunRecord {
    let engine = Engine::new();
    let mut budget = Budget::new(limits());
    let started = Instant::now();
    let snapshot = engine
        .open(ArtifactInput::bytes(verified.content.clone()), &mut budget)
        .expect("the subject opens");
    let request = single_member_request(&snapshot, premise);
    let recovered = engine
        .recover_method(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("a legal request is answered, not raised");
    let wall_micros = started.elapsed().as_micros();
    let form = published(&recovered);
    let mut context = context_of(corpus, verified);
    context.scope = format!("single member add(II)I of {}", verified.subject.path);
    context.view =
        serde_json::to_string(&request.environment.runtime.physical).expect("a view serializes");
    context.profile =
        serde_json::to_string(&request.environment.runtime.profile).expect("a profile serializes");
    context.providers = format!(
        "{} (this request names no additional content source)",
        request.environment.providers.len()
    );
    context.recovery = format!(
        "the presentation of one run's own payload, gated on the profile above: {}",
        context.profile
    );
    context.premise = format!(
        "1 header read to locate add(II)I, charged to a separate budget and outside the measured \
         section; the class declares {} bodies",
        premise.declared_bodies
    );
    RunRecord {
        label,
        context,
        status: status_of(&recovered.analysis().execution),
        usage: budget.usage(),
        report_usage: usage_of(&recovered.analysis().execution),
        coverage: Some(
            serde_json::to_value(&recovered.analysis().coverage).expect("coverage serializes"),
        ),
        diagnostics: serde_json::to_value(&recovered.analysis().diagnostics)
            .expect("diagnostics serialize"),
        order: "[]".to_string(),
        published_items: 0,
        declared_bodies: premise.declared_bodies,
        published: form.raw,
        document: form.normalized,
        fingerprint: form.fingerprint,
        wall_micros,
    }
}

fn context_of(corpus: &Corpus, verified: &Verified) -> Context {
    Context {
        corpus_version: corpus.version.clone(),
        subject: verified.subject.path,
        subject_bytes: verified.bytes,
        subject_blake3: verified.blake3.clone(),
        // `direct` is the reference path of `design.md` decision 3: the scan every other path has to
        // agree with. The other two columns of the matrix have no implementation to run.
        path: "direct",
        cache: "absent: no cache or index exists in this repository (P5 2.2 owns one), so no run \
                can be labelled cache-on",
        concurrency: "1: no parallel scheduler exists (P5 2.x owns one), so every row is one \
                      sequential scan",
        scope: String::new(),
        view: String::new(),
        profile: String::new(),
        providers: String::new(),
        recovery: String::new(),
        premise: String::new(),
        budget: limits(),
    }
}

/// The identities a scan published, in publication order.
///
/// The identity is the evidence the item states — its source location, its consumer, its operation,
/// its target and how it was derived — never a counter, so two runs that publish the same facts in a
/// different order produce different strings.
fn item_identities(items: &[XrefItem]) -> String {
    let identities: Vec<Value> = items
        .iter()
        .map(|item| {
            json!({
                "consumer": item.consumer,
                "derivation": item.derivation,
                "operation": item.operation,
                "source": item.source,
                "target": item.target,
            })
        })
        .collect();
    serde_json::to_string(&identities).expect("identities render")
}

// -------------------------------------------------------------------------------------------
// The comparison report (task 1.3)
// -------------------------------------------------------------------------------------------

/// Every half of two runs the `performance-gates` spec compares, plus the resources.
struct Verdicts {
    status: bool,
    fingerprint: bool,
    order: bool,
    coverage: bool,
    diagnostics: bool,
}

/// One row of the comparison report: two runs, what they agree on, and what one charged over the
/// other.
///
/// The resource half is read from the runs' **total request** charge rather than from the figure
/// their published reports state, because for the local row those differ (see [`outside_report`]).
/// Both sides of a comparison have to be the same figure, and the request's is the one that covers
/// the whole request on every path.
struct Comparison {
    left: String,
    right: String,
    verdicts: Verdicts,
    /// `right - left` over every counted dimension, in the declared order.
    deltas: Vec<(CountedBudgetDimension, i128)>,
    /// `right - left` of the two high-water depths, which are limits rather than charges.
    depths: Vec<(&'static str, i128)>,
    left_status: &'static str,
    right_status: &'static str,
}

impl Comparison {
    /// Whether the two runs published the same result.
    ///
    /// The allowed difference is the wall clock alone, which the fingerprint already normalizes
    /// away. Publication order is required to be **equal**, not merely "the same set in a stable
    /// order": P5's decision 4 says a parallel scheduler changes scheduling, not the order results
    /// are published in, so a gate that accepted a re-ordering would accept a change the design
    /// forbids.
    fn equivalent(&self) -> bool {
        let verdicts = &self.verdicts;
        verdicts.status
            && verdicts.fingerprint
            && verdicts.order
            && verdicts.coverage
            && verdicts.diagnostics
    }
}

fn compare(left: &RunRecord, right: &RunRecord) -> Comparison {
    let deltas = deltas_between(&left.usage, &right.usage);
    Comparison {
        left: left.label.to_string(),
        right: right.label.to_string(),
        verdicts: Verdicts {
            status: left.status == right.status,
            fingerprint: left.fingerprint == right.fingerprint,
            order: left.order == right.order,
            coverage: left.coverage == right.coverage,
            diagnostics: left.diagnostics == right.diagnostics,
        },
        deltas,
        depths: vec![
            (
                "nested_depth",
                i128::from(right.usage.nested_depth) - i128::from(left.usage.nested_depth),
            ),
            (
                "dependency_depth",
                i128::from(right.usage.dependency_depth) - i128::from(left.usage.dependency_depth),
            ),
        ],
        left_status: left.status,
        right_status: right.status,
    }
}

/// The public spelling of a counted dimension, which is also the suffix of its diagnostic code.
fn dimension_name(dimension: CountedBudgetDimension) -> String {
    serde_json::to_string(&dimension)
        .expect("a dimension serializes")
        .trim_matches('"')
        .to_string()
}

impl Comparison {
    fn lines(&self) -> Vec<String> {
        let verdict = |equal: bool| if equal { "equal" } else { "different" };
        let mut lines = vec![
            format!("comparison {} → {}", self.left, self.right),
            format!(
                "  status        {} → {}",
                self.left_status, self.right_status
            ),
            format!("  fingerprint   {}", verdict(self.verdicts.fingerprint)),
            format!("  order         {}", verdict(self.verdicts.order)),
            format!("  coverage      {}", verdict(self.verdicts.coverage)),
            format!("  diagnostics   {}", verdict(self.verdicts.diagnostics)),
            format!(
                "  equivalent    {}",
                if self.equivalent() {
                    "yes (the wall clock is the only allowed difference)"
                } else {
                    "no"
                }
            ),
        ];
        lines.push("  resources (right - left)".to_string());
        for (dimension, delta) in &self.deltas {
            lines.push(format!(
                "    {:<22} {:+}",
                dimension_name(*dimension),
                delta
            ));
        }
        for (name, delta) in &self.depths {
            lines.push(format!("    {name:<22} {delta:+}"));
        }
        lines
    }
}

/// The three coverage dimensions, from either shape this harness records: the reader's `Coverage`
/// document, or the query layer's wrapper around it.
fn coverage_dimensions(coverage: &Value) -> Map<String, Value> {
    let object = coverage
        .as_object()
        .unwrap_or_else(|| panic!("a coverage document is an object: {coverage}"));
    if object.contains_key("artifact_structural") {
        return object.clone();
    }
    object["dimensions"]
        .as_object()
        .unwrap_or_else(|| panic!("a coverage document names no dimensions: {coverage}"))
        .clone()
}

/// A one-line reading of a coverage document: the state of each dimension and how many ranges it
/// scanned, which is what "物化范围" adds to the counters.
fn coverage_summary(coverage: &Value) -> String {
    let mut parts = Vec::new();
    for (name, dimension) in coverage_dimensions(coverage) {
        let state = dimension["state"].as_str().unwrap_or("?");
        let scanned = dimension["scanned"].as_array().map_or(0, Vec::len);
        let skipped = dimension["skipped"].as_array().map_or(0, Vec::len);
        parts.push(format!(
            "{name}={state} scanned={scanned} skipped={skipped}"
        ));
    }
    parts.join(", ")
}

// -------------------------------------------------------------------------------------------
// Repeats, distributions and the printed report
// -------------------------------------------------------------------------------------------

/// The repeat count of the measurement. Chosen so the reported median is the middle of a sample
/// large enough to show a settling trend, and so the halves self-check printed beside it is
/// meaningful: the first-half and second-half medians disagreeing is visible in the output instead
/// of being averaged into a single number. It is not a statistical threshold and no timing is
/// asserted against it.
const REPEATS: usize = 200;

/// The repeat count of the always-on tests. The determinism and ordering claims are binary, not
/// statistical, so a handful of repeats decides them; the distribution comes from [`REPEATS`].
const SMOKE_REPEATS: usize = 8;

/// One row of the matrix: a label, the context every repeat shared, and the repeats themselves.
struct Row {
    label: &'static str,
    context: Context,
    repeats: Vec<RunRecord>,
}

fn repeats(label: &'static str, count: usize, mut run: impl FnMut() -> RunRecord) -> Row {
    let first = run();
    let context = Context {
        corpus_version: first.context.corpus_version.clone(),
        subject: first.context.subject,
        subject_bytes: first.context.subject_bytes,
        subject_blake3: first.context.subject_blake3.clone(),
        path: first.context.path,
        cache: first.context.cache,
        concurrency: first.context.concurrency,
        scope: first.context.scope.clone(),
        view: first.context.view.clone(),
        profile: first.context.profile.clone(),
        providers: first.context.providers.clone(),
        recovery: first.context.recovery.clone(),
        premise: first.context.premise.clone(),
        budget: first.context.budget.clone(),
    };
    let mut runs = vec![first];
    for _ in 1..count {
        runs.push(run());
    }
    Row {
        label,
        context,
        repeats: runs,
    }
}

/// The spread of a sample, by index rule rather than interpolation: `median` is the element at
/// `len / 2` of the sorted sample (the upper of the two middle elements for an even count), and
/// `p90` the element at `len * 9 / 10`. Both rules are stated so the numbers can be reproduced.
struct Distribution {
    min: u128,
    median: u128,
    p90: u128,
    max: u128,
    first: u128,
    first_half_median: u128,
    second_half_median: u128,
}

impl Distribution {
    fn of(samples: &[u128]) -> Self {
        assert!(!samples.is_empty(), "a distribution needs a sample");
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let middle = |values: &[u128]| values[values.len() / 2];
        Self {
            min: sorted[0],
            median: middle(&sorted),
            p90: sorted[sorted.len() * 9 / 10],
            max: sorted[sorted.len() - 1],
            first: samples[0],
            first_half_median: middle(&sorted[..sorted.len() / 2]),
            second_half_median: middle(&sorted[sorted.len() / 2..]),
        }
    }

    fn line(&self) -> String {
        format!(
            "first {} | min {} | median {} | p90 {} | max {} | halves {}/{}",
            self.first,
            self.min,
            self.median,
            self.p90,
            self.max,
            self.first_half_median,
            self.second_half_median
        )
    }
}

/// The metrics a row records, grouped the way task 1.2 names them.
///
/// `读取字节` and `物化范围` are read from the request's own charges. `内存代理` reads the *same*
/// counters under the memory heading, which is why the grouping is stated: the proxy claim is about
/// which counters may stand for memory and what they cannot say, not about a second measurement.
const BYTES_READ: [CountedBudgetDimension; 4] = [
    CountedBudgetDimension::InputBytes,
    CountedBudgetDimension::EntryBytes,
    CountedBudgetDimension::ReadBytes,
    CountedBudgetDimension::OutputBytes,
];

const MATERIALIZED: [CountedBudgetDimension; 5] = [
    CountedBudgetDimension::ArchiveEntries,
    CountedBudgetDimension::ClassHeaders,
    CountedBudgetDimension::MethodBodies,
    CountedBudgetDimension::AnalysisSteps,
    CountedBudgetDimension::NormalizationClones,
];

/// The memory proxy: the dimensions that grow with how much structure a run derives and holds.
///
/// * `ClassBytes`, `AttributeBytes` and `CodeBytes` are the class-file content the run materialized
///   — a decoder that retains what it read scales with them, so a change that starts holding the
///   whole class instead of the part it needs moves them;
/// * `IrItems` and `IrEdges` are the derived structures the pipeline allocates one unit at a time
///   (frame and local slots, SSA values, phi inputs, origin members; CFG and def-use edges), which
///   is the part of the run whose live size grows with the work rather than with the input;
/// * `ResultItems` is the published payload, the third thing that grows with the corpus. It is a
///   **charge**, not a page size: the archive row charges four and publishes one item, because the
///   budget is charged as candidates are produced rather than as a page is filled. A comparison that
///   read the two as one number would compare a charge against a page, so the harness records both.
///
/// What the proxy cannot say, and why it is still worth recording: none of these is RSS. They are
/// counters the budget layer already keeps, so they see no allocator overhead or fragmentation, they
/// cannot distinguish a peak from a total (only `nested_depth` and `dependency_depth` are
/// high-water marks), they count transients that were freed, and a run can allocate memory without
/// charging any of them. A claim about *memory* needs an allocator or a resident-set measurement
/// that this repository does not take, so what a delta in these dimensions supports is "this path
/// derived or held more units of work", never "this path used more memory".
const MEMORY_PROXIES: [CountedBudgetDimension; 6] = [
    CountedBudgetDimension::ClassBytes,
    CountedBudgetDimension::AttributeBytes,
    CountedBudgetDimension::CodeBytes,
    CountedBudgetDimension::IrItems,
    CountedBudgetDimension::IrEdges,
    CountedBudgetDimension::ResultItems,
];

fn charges_line(usage: &UsageSnapshot, dimensions: &[CountedBudgetDimension]) -> String {
    dimensions
        .iter()
        .map(|dimension| {
            format!(
                "{}={}",
                dimension_name(*dimension),
                usage.counted_usage(*dimension)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Everything the measurement prints for one row, and the semantic assertions that keep the numbers
/// about what they claim to be about.
fn report_row(row: &Row, distribution: &Distribution) {
    println!();
    println!("row {}", row.label);
    for line in row.context.lines() {
        println!("  {line}");
    }
    let first = &row.repeats[0];
    println!(
        "  repeats {} (each: fresh snapshot, fresh budget)",
        row.repeats.len()
    );
    println!(
        "  wall clock (microseconds, harness) {}",
        distribution.line()
    );
    println!(
        "  report clock elapsed_millis {} (whole milliseconds: the corpus is small enough that the \
         report's own clock is zero in every repeat, which is why the distribution above is the \
         harness's)",
        first.usage.elapsed_millis
    );
    println!("  status {}", first.status);
    println!("  bytes read   {}", charges_line(&first.usage, &BYTES_READ));
    println!(
        "  materialized {}",
        charges_line(&first.usage, &MATERIALIZED)
    );
    println!(
        "  memory proxy {}",
        charges_line(&first.usage, &MEMORY_PROXIES)
    );
    println!("  published items {}", first.published_items);
    println!(
        "  charged beyond the published report: {}",
        delta_line(&outside_report(first))
    );
    if let Some(coverage) = &first.coverage {
        println!("  coverage     {}", coverage_summary(coverage));
    }
    println!(
        "  diagnostics  {} (codes: {})",
        first.diagnostics.as_array().map_or(0, Vec::len),
        first
            .diagnostics
            .as_array()
            .map(|entries| entries
                .iter()
                .map(|entry| entry["code"].as_str().unwrap_or("?").to_string())
                .collect::<Vec<_>>()
                .join(", "))
            .unwrap_or_default()
    );
    println!("  result fingerprint {}", first.fingerprint);
    // The measurement is a measurement, but it must still be about a run that was the same run: a
    // repeat that charged something else, stopped differently or published a different document
    // means the row is not measuring one thing, whatever its timings say.
    for (index, repeat) in row.repeats.iter().enumerate() {
        assert_eq!(
            repeat.status, first.status,
            "repeat {index} of {} ended as {} while the first ended as {}",
            row.label, repeat.status, first.status
        );
        assert_eq!(
            repeat.fingerprint, first.fingerprint,
            "repeat {index} of {} published a different result; a distribution over rows that \
             differ is not a distribution over one request",
            row.label
        );
        assert_eq!(
            charges(&repeat.usage),
            charges(&first.usage),
            "repeat {index} of {} charged different dimensions than the first",
            row.label
        );
        assert_eq!(
            repeat.order, first.order,
            "repeat {index} of {} published its items in a different order",
            row.label
        );
    }
}

// -------------------------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------------------------

/// The fingerprint is read, its schema is the one this harness understands, and every subject is
/// the file it fixes — under the classification the fingerprint gives it.
#[test]
fn the_benchmark_subjects_are_the_bytes_the_corpus_fingerprint_fixes() {
    let corpus = corpus();
    assert_eq!(
        corpus.schema, SCHEMA,
        "{MANIFEST} carries schema `{}` and this harness reads `{SCHEMA}`; a newer fingerprint may \
         reclassify files, so read it before measuring over it",
        corpus.schema
    );
    assert!(
        !corpus.files.is_empty(),
        "{MANIFEST} lists no files, so nothing is fixed"
    );
    println!(
        "corpus fingerprint {} : {} files, {}",
        corpus.schema,
        corpus.files.len(),
        corpus.version
    );
    for subject in &SUBJECTS {
        let verified = verified(&corpus, subject);
        assert!(
            subject
                .classified_under
                .names(&corpus.document, subject.path),
            "{} is classified in {MANIFEST} as a carrier of {} and the document does not name it \
             there; a subject the fingerprint no longer carries is outside the corpus it fixes",
            subject.path,
            subject.classified_under.describe()
        );
        println!(
            "  {} ({}) : {} bytes, blake3 {}, {}",
            subject.path,
            subject.classified_under.describe(),
            verified.bytes,
            short(&verified.blake3),
            subject.role
        );
    }
    // The artifact the local row's premise read depends on: if the subject stops declaring several
    // bodies, the single-member boundary has nothing to be a boundary against.
    let target = locate_member(&verified(&corpus, &SUBJECTS[1]).content);
    println!(
        "  {} declares {} bodies; the single-member row presents one of them",
        SUBJECTS[1].path, target.declared_bodies
    );
}

/// A15's measurable half today: repeated runs of the one path that exists publish the same result.
///
/// This is the baseline the cold/warm comparison is defined against, and it is deliberately named
/// as the repeat half. The other half of A15 compares a cache-off run against a cache-on one; no
/// cache exists, so no row can be run on that side and the row itself stays as task 1.1 recorded it.
#[test]
fn repeated_direct_runs_publish_the_same_result_fingerprint() {
    let corpus = corpus();
    let verified = verified(&corpus, &SUBJECTS[0]);
    let row = repeats("full-range-xref/minimal-jar", SMOKE_REPEATS, || {
        run_full_range(&corpus, &verified, "full-range-xref/minimal-jar", false)
    });
    let first = &row.repeats[0];
    println!("{}", first.fingerprint);
    assert_eq!(
        first.status, "complete",
        "the row's request is inside its budget and must complete: {}",
        first.status
    );
    assert!(
        first.published_items > 0,
        "a determinism claim about a scan that published nothing is vacuous"
    );
    for (index, repeat) in row.repeats.iter().enumerate() {
        assert_eq!(
            repeat.fingerprint, first.fingerprint,
            "repeat {index} of {} published a different result after {SMOKE_REPEATS} runs of one \
             request over bytes the fingerprint fixes",
            row.label
        );
        assert_eq!(
            charges(&repeat.usage),
            charges(&first.usage),
            "repeat {index} charged a different set of dimensions"
        );
    }
    // The first run in a process is the only "cold" this engine can be today: the artifact is
    // re-opened and nothing is carried over, and every repeat after it sees the same bytes with the
    // same warm allocator and warm page cache behind them. The claim under test is that the
    // difference between them may show up in time and not in the result.
    println!(
        "first run {} us, median of the rest {} us — a timing difference; the fingerprint above is \
         the same for all {} repeats",
        first.wall_micros,
        Distribution::of(
            &row.repeats[1..]
                .iter()
                .map(|run| run.wall_micros)
                .collect::<Vec<_>>()
        )
        .median,
        row.repeats.len()
    );
}

/// The normalization is exactly one field wide: the clock moves and the fingerprint does not, and
/// anything else moves it.
///
/// The stamping is not decoration. On this corpus a direct run takes well under a millisecond, so
/// the report's own whole-millisecond clock is `0` in every repeat and an unnormalized fingerprint
/// would look stable across repeats — the bug this guards against is invisible to a repeat
/// comparison and has to be provoked. A published report with a different clock is a run that took a
/// different time; it is not a different result, and the fingerprint has to say so.
#[test]
fn the_result_fingerprint_ignores_the_wall_clock_and_nothing_else() {
    let corpus = corpus();
    let verified = verified(&corpus, &SUBJECTS[0]);
    let record = run_full_range(&corpus, &verified, "full-range-xref/minimal-jar", false);

    let mut stamped = record.published.clone();
    let stamped_count = stamp_elapsed(&mut stamped, 12_345);
    assert!(
        stamped_count >= 1,
        "the published document carries `elapsed_millis` ({stamped_count} of them), which is the \
         field this test is about"
    );
    strip_elapsed(&mut stamped);
    assert_eq!(
        fingerprint_of(&stamped),
        record.fingerprint,
        "a run that reports a different wall clock is the same result: `elapsed_millis` is the one \
         field the fingerprint removes ({stamped_count} of them here)"
    );

    let mut moved = record.document.clone();
    let after = move_charge(&mut moved);
    assert_ne!(
        fingerprint_of(&moved),
        record.fingerprint,
        "a document that charged {after} bytes and one that charged one less are different \
         results; a fingerprint that called them equal would hide exactly the resource change a \
         cache or parallel path is measured for"
    );
}

/// Write `value` into every `elapsed_millis` at any depth, and say how many there were.
fn stamp_elapsed(document: &mut Value, value: u64) -> usize {
    let mut stamped = 0;
    match document {
        Value::Object(map) => {
            if let Some(slot) = map.get_mut("elapsed_millis") {
                *slot = json!(value);
                stamped += 1;
            }
            for (_, child) in map.iter_mut() {
                stamped += stamp_elapsed(child, value);
            }
        }
        Value::Array(entries) => {
            for child in entries.iter_mut() {
                stamped += stamp_elapsed(child, value);
            }
        }
        _ => {}
    }
    stamped
}

/// Add one to the `read_bytes` a published usage snapshot charged, and return the new value.
fn move_charge(document: &mut Value) -> u64 {
    let slot = document["execution"]["usage"]["read_bytes"]
        .as_u64()
        .unwrap_or_else(|| {
            panic!("a published report states its `read_bytes` under `execution.usage`: {document}")
        });
    document["execution"]["usage"]["read_bytes"] = json!(slot + 1);
    slot
}

/// One run of every row that publishes a result, in a fixed order: the three `direct` rows the
/// ordering and comparison claims are stated over.
fn published_rows(corpus: &Corpus) -> Vec<RunRecord> {
    let jar = verified(corpus, &SUBJECTS[0]);
    let class = verified(corpus, &SUBJECTS[1]);
    let premise = locate_member(&class.content);
    vec![
        run_full_range(corpus, &jar, "full-range-xref/minimal-jar", false),
        run_full_range(corpus, &class, "full-range-xref/v52-class", false),
        run_single_member(corpus, &class, "single-member/v52-class", &premise),
    ]
}

/// The marker a child process prints for each row it measured: the label, the result fingerprint,
/// and the digest of the publication order, so a cross-process difference says which half moved.
const MARKER: &str = "P5-RESULT-FINGERPRINT";

/// The test the cross-process comparison re-runs. Named here so the parent and the child cannot
/// disagree about which test is the marker: a child that matches nothing prints no marker line and
/// fails the count instead of comparing two empty lists.
const MARKER_TEST: &str = "the_direct_row_fingerprints_are_printed_for_a_cross_process_comparison";

/// The marker test's own work: it measures, it checks that a second run in *this* process agrees,
/// and it prints what a parent process cannot observe for itself.
#[test]
fn the_direct_row_fingerprints_are_printed_for_a_cross_process_comparison() {
    let corpus = corpus();
    let rows = published_rows(&corpus);
    println!("process {} marks {} rows", std::process::id(), rows.len());
    for (index, row) in rows.iter().enumerate() {
        let again = published_rows(&corpus)
            .into_iter()
            .nth(index)
            .expect("the same rows come back in the same order");
        assert_eq!(
            again.fingerprint, row.fingerprint,
            "two runs of {} inside one process disagree",
            row.label
        );
        assert_eq!(
            again.order, row.order,
            "two runs of {} order differently",
            row.label
        );
        println!(
            "{MARKER} {} {} {}",
            row.label,
            row.fingerprint,
            order_digest(row)
        );
    }
}

fn order_digest(row: &RunRecord) -> String {
    format!("blake3:{}", blake3::hash(row.order.as_bytes()).to_hex())
}

/// The publication order is a function of the input alone.
///
/// This is the baseline P5's decision 4 requires of a parallel scheduler ("worker 结果按稳定
/// identity/evidence 顺序合并"): today there is one sequential scan, so what has to be established
/// is that the order it publishes is not an accident of *this* run. Two independent observations are
/// taken, because they fail differently:
///
/// * repeats inside one process, each on a fresh snapshot and a fresh budget, catch an order that
///   follows an allocation or a traversal that is re-entered;
/// * two **independent processes** catch an order that follows a hash container's iteration, which
///   Rust seeds per process and per instance: a `HashMap`-ordered publication can look stable for
///   the life of one test binary and still differ from the next one.
#[test]
fn the_published_order_is_a_function_of_the_input_alone() {
    let corpus = corpus();
    let mine = published_rows(&corpus);
    assert!(
        mine.iter()
            .all(|row| row.published_items > 0 || row.order == "[]"),
        "a row that published items must state them in an order this test can compare"
    );
    assert!(
        mine[0].published_items > 0,
        "the full-range row over the archive published nothing, so an ordering claim about it \
         would be vacuous"
    );
    let expected: Vec<String> = mine
        .iter()
        .map(|row| {
            format!(
                "{MARKER} {} {} {}",
                row.label,
                row.fingerprint,
                order_digest(row)
            )
        })
        .collect();

    for attempt in 0..SMOKE_REPEATS {
        let again = published_rows(&corpus);
        for (index, row) in again.iter().enumerate() {
            assert_eq!(
                row.order, mine[index].order,
                "attempt {attempt} of {} published the same facts in a different order",
                row.label
            );
        }
    }

    let first = marker_lines_in_a_child_process();
    let second = marker_lines_in_a_child_process();
    assert_eq!(
        first, second,
        "two independent processes published different results or different orders for the same \
         input; nothing about the publication order can be relied on until this holds"
    );
    assert_eq!(
        first, expected,
        "a child process disagrees with this process; the publication order (or the result behind \
         it) depends on which process ran it"
    );
    for line in &first {
        println!("{line}");
    }
}

/// Run the marker test in a fresh process and collect the lines it printed.
fn marker_lines_in_a_child_process() -> Vec<String> {
    let binary = std::env::current_exe().expect("the running test binary is known");
    let output = Command::new(&binary)
        .args(["--exact", MARKER_TEST, "--nocapture", "--test-threads", "1"])
        .output()
        .expect("the test binary runs again");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success(),
        "the marker test failed in a child process ({:?}):\n{stdout}\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let lines: Vec<String> = stdout
        .lines()
        .filter(|line| line.starts_with(MARKER))
        .map(str::to_string)
        .collect();
    assert_eq!(
        lines.len(),
        3,
        "the child process printed {} marker lines for the three published rows; if `{MARKER_TEST}` \
         no longer names a test in this binary the child ran nothing, which is not a comparison:\n\
         {stdout}",
        lines.len()
    );
    lines
}

/// The comparison report, over the pair today's engine can produce: one single-member request and
/// one full-range scan over the **same bytes**, plus the run-against-itself control the equal
/// verdicts are read against.
///
/// The materialized range is recorded here rather than re-accepted: P2 and P3 already prove the
/// on-demand boundary (`p3_recovery_entry`, `p2_entry_counts`), and what this file adds is the
/// numbers a comparison between paths will be read beside — how many headers and bodies each row
/// attempted, and what each charged.
#[test]
fn the_local_and_the_full_range_runs_are_compared_field_by_field() {
    let corpus = corpus();
    let class = verified(&corpus, &SUBJECTS[1]);
    let premise = locate_member(&class.content);
    let local = run_single_member(&corpus, &class, "single-member/v52-class", &premise);
    let full = run_full_range(&corpus, &class, "full-range-xref/v52-class", false);

    for row in [&local, &full] {
        assert_eq!(
            row.status, "complete",
            "{} must complete for its numbers to be a baseline: {}",
            row.label, row.status
        );
    }
    // The query rows' own report is the whole request's accounting: the engine's report and the
    // budget agree, so a comparison may read either. Checked rather than assumed, because the local
    // row shows it is not a property of every entry.
    assert_eq!(
        charges(&full.usage),
        charges(&full.report_usage),
        "the query entry charged {} while its report states {}",
        charges_line(&full.usage, &CountedBudgetDimension::ALL),
        charges_line(&full.report_usage, &CountedBudgetDimension::ALL)
    );

    // The single-member boundary, recorded at the benchmark's own level: the class declares several
    // bodies and the request that names one attempts one. This is not P2/P3's acceptance re-run —
    // it is the premise that makes the numbers below comparable at all.
    assert!(
        local.declared_bodies >= 3,
        "the control class must declare more than one body"
    );
    assert_eq!(
        local.usage.class_headers,
        1,
        "the single-member request attempts one header read: {}",
        charges_line(&local.usage, &CountedBudgetDimension::ALL)
    );
    assert_eq!(
        local.usage.method_bodies,
        1,
        "the single-member request attempts one body out of the {} the class declares: {}",
        local.declared_bodies,
        charges_line(&local.usage, &CountedBudgetDimension::ALL)
    );
    assert_eq!(
        full.usage.method_bodies,
        0,
        "the full-range scan takes no body-read charge, because it does not go through the \
         header/body demand path that charges one: {}",
        charges_line(&full.usage, &CountedBudgetDimension::ALL)
    );

    let same_path = compare(&full, &run_full_range(&corpus, &class, full.label, false));
    assert!(
        same_path.equivalent(),
        "two runs of one direct request did not compare equal:\n{}",
        same_path.lines().join("\n")
    );

    let contrast = compare(&local, &full);
    assert_eq!(
        contrast.deltas.len(),
        CountedBudgetDimension::ALL.len(),
        "the resource half of the report covers every counted dimension"
    );
    assert!(
        !contrast.equivalent(),
        "a single-member request and a full-range scan are different requests and must not compare \
         equal:\n{}",
        contrast.lines().join("\n")
    );
    for row in [&local, &full] {
        println!();
        println!("row {}", row.label);
        println!("  scope {}", row.context.scope);
        println!("  status {}", row.status);
        println!("  materialized {}", charges_line(&row.usage, &MATERIALIZED));
        println!("  bytes read   {}", charges_line(&row.usage, &BYTES_READ));
        println!(
            "  memory proxy {}",
            charges_line(&row.usage, &MEMORY_PROXIES)
        );
        println!(
            "  coverage     {}",
            row.coverage
                .as_ref()
                .map_or_else(|| "none".to_string(), coverage_summary)
        );
        println!(
            "  charged beyond the published report: {}",
            delta_line(&outside_report(row))
        );
    }
    for line in same_path.lines().into_iter().chain(contrast.lines()) {
        println!("{line}");
    }
}

/// The cancellation row: a run whose token is cancelled before the entry publishes a terminal
/// cancellation, and the comparison says so instead of reading it as a small complete run.
#[test]
fn a_cancelled_direct_run_is_never_published_as_complete() {
    let corpus = corpus();
    let jar = verified(&corpus, &SUBJECTS[0]);
    let complete = run_full_range(&corpus, &jar, "full-range-xref/minimal-jar", false);
    let cancelled = run_full_range(&corpus, &jar, "cancelled/full-range-xref/minimal-jar", true);

    assert_eq!(
        cancelled.status, "cancelled",
        "a cancelled request reports its own termination: {}",
        cancelled.status
    );
    assert_eq!(
        cancelled.published_items, 0,
        "the cancelled query published items"
    );
    assert_eq!(
        cancelled.order, "[]",
        "a cancelled run published an order it never reached the end of"
    );
    assert!(
        cancelled.usage.input_bytes > 0,
        "the artifact really was opened before the entry ran: {}",
        charges_line(&cancelled.usage, &CountedBudgetDimension::ALL)
    );
    assert!(
        cancelled
            .diagnostics
            .as_array()
            .is_some_and(|entries| !entries.is_empty()),
        "a terminated run states why: {}",
        cancelled.diagnostics
    );

    let contrast = compare(&complete, &cancelled);
    assert!(
        !contrast.equivalent() && !contrast.verdicts.status && !contrast.verdicts.fingerprint,
        "a cancelled run must not compare equal to a complete one:\n{}",
        contrast.lines().join("\n")
    );
    for line in contrast.lines() {
        println!("{line}");
    }
    println!(
        "cancelled usage {}",
        charges_line(&cancelled.usage, &CountedBudgetDimension::ALL)
    );
    println!("cancelled diagnostics {}", cancelled.diagnostics);
}

/// The repeated measurement itself: the numbers this task exists to produce.
///
/// Ignored so that a normal `cargo test` stays a gate rather than a benchmark, and so that no test
/// failure is ever a timing failure. What it prints is the baseline; what it asserts is only that
/// every repeat was the same run.
#[test]
#[ignore = "repeated benchmark: run with `cargo test --test p5_benchmark --locked -- --ignored \
            --nocapture p5_repeated_direct_baseline`"]
fn p5_repeated_direct_baseline() {
    let corpus = corpus();
    let jar = verified(&corpus, &SUBJECTS[0]);
    let class = verified(&corpus, &SUBJECTS[1]);
    let premise = locate_member(&class.content);
    println!(
        "P5 1.2 baseline: the direct path only (no cache, no parallel scheduler), one machine, \
         RUST_TEST_THREADS=1, single-threaded build, {REPEATS} repeats per row"
    );
    println!(
        "corpus fingerprint {} : {} files, {} — the manifest is read and every subject's digest is \
         checked against it before the first run",
        corpus.schema,
        corpus.files.len(),
        corpus.version
    );

    let rows = vec![
        repeats("full-range-xref/minimal-jar", REPEATS, || {
            run_full_range(&corpus, &jar, "full-range-xref/minimal-jar", false)
        }),
        repeats("full-range-xref/v52-class", REPEATS, || {
            run_full_range(&corpus, &class, "full-range-xref/v52-class", false)
        }),
        repeats("single-member/v52-class", REPEATS, || {
            run_single_member(&corpus, &class, "single-member/v52-class", &premise)
        }),
        repeats("cancelled/full-range-xref/minimal-jar", REPEATS, || {
            run_full_range(&corpus, &jar, "cancelled/full-range-xref/minimal-jar", true)
        }),
    ];

    let mut distributions = Vec::new();
    for row in &rows {
        let samples: Vec<u128> = row.repeats.iter().map(|run| run.wall_micros).collect();
        let distribution = Distribution::of(&samples);
        report_row(row, &distribution);
        distributions.push(distribution);
    }

    println!();
    println!("comparisons");
    let comparisons = [
        compare(&rows[1].repeats[0], &rows[2].repeats[0]),
        compare(&rows[0].repeats[0], &rows[0].repeats[REPEATS - 1]),
        compare(&rows[0].repeats[0], &rows[3].repeats[0]),
    ];
    for comparison in &comparisons {
        for line in comparison.lines() {
            println!("{line}");
        }
    }
    println!();
    println!("timing summary (microseconds, harness clock, one machine)");
    for (row, distribution) in rows.iter().zip(&distributions) {
        println!("  {:<40} {}", row.label, distribution.line());
    }
}
