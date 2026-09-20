//! P5 tasks 1.2, 1.3, 2.1 and 2.2: the repeated direct-path baseline, the report a second path is
//! judged by, and the decisions taken from it (nothing is enabled; the key dimensions a cache would
//! have to bind are recorded and anchored to the code that carries them).
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
//!   judged with (task 1.3);
//! * it records the decisions of tasks 2.1 and 2.2 as data the harness checks, not as prose: no
//!   candidate is in the default path, a runnable second configuration has to bring a measured
//!   comparison with it, and the cache key dimensions are anchored to the tokens that carry them
//!   today. Two of those dimensions have no carrier at all, which is part of why no key type is
//!   built (see "The decisions tasks 2.1 and 2.2 ask for").
//!
//! The **container** half of the cache — directed access along an origin chain and the bounded
//! reuse of a container's verified facts — is measured by `tests/p5_container_lookup.rs`, which
//! carries its own fixtures, its off/cold/warm/capacity comparison and the two fingerprints. This
//! file keeps the cache-state census below (`the_engine_has_one_disabled_facts_cache_and_no_index_or_scheduler`),
//! so the set of sources that may name a cache handle is checked in one place.
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

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
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

/// The load root one fixture's own content is: a standalone CLASS snapshot is one whole
/// definition, and a ZIP snapshot is searched in its root container with an empty prefix. The row's
/// subject decides which shape it is; no layout prefix is inferred from the fixture.
fn snapshot_root(snapshot: &ArtifactSnapshot) -> LoadRoot {
    match snapshot.kind() {
        ArtifactKind::StandaloneClass => LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        },
        ArtifactKind::Zip => LoadRoot::Container {
            origin: ContainerOrigin {
                snapshot: snapshot.id().clone(),
                root_container: ContainerId("root".into()),
                steps: Vec::new(),
            },
            prefix: ArchiveNameBytes(Vec::new()),
        },
    }
}

/// The caller domain the single-member row runs under: the subject's own snapshot and nothing else,
/// which is the simplest environment the validator accepts without a problem.
fn single_member_environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![snapshot_root(snapshot)],
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
    /// The same document with every **charge record** removed as well: the input of the *result*
    /// comparison between two configurations.
    ///
    /// The resource half travels inside the result document — a `usage` snapshot sits beside the
    /// items it was charged for — and a cache is supposed to move it, so `evidence` and
    /// `representation` are compared over this form. Only `usage` objects are removed:
    /// statuses, termination reasons, evidence, coverage and diagnostics stay, because those are
    /// results (the same split `p5_facts_cache.rs::without_charges` makes).
    evidence: Value,
    evidence_fingerprint: String,
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
    let mut evidence = normalized.clone();
    let charges = strip_charges(&mut evidence);
    assert!(
        charges >= 1,
        "a published report states what it charged at least once; this one states no charge at all, \
         so the evidence form is not a form of anything"
    );
    let evidence_fingerprint = fingerprint_of(&evidence);
    Published {
        raw,
        normalized,
        fingerprint,
        evidence,
        evidence_fingerprint,
    }
}

/// Delete every `usage` object at any depth, returning how many were deleted.
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

/// Every path at which two documents differ, as `path: left → right`.
///
/// Task 1.3's fingerprint answers *whether* two runs published the same result. This answers *what*
/// moved, which is the half a comparison between two configurations needs: the `Cold and warm
/// results` scenario allows a partial subset to be affected by scheduling, and what it may never
/// hide is a change of the result. Every difference the cache-on comparison finds has to be a
/// charge ([`fn is_a_charge_path`]) — that is the claim, and this is how it is checked instead of
/// asserted.
fn differing_paths(left: &Value, right: &Value) -> Vec<String> {
    let mut paths = Vec::new();
    collect_differences("", left, right, &mut paths);
    paths
}

fn collect_differences(path: &str, left: &Value, right: &Value, into: &mut Vec<String>) {
    match (left, right) {
        (Value::Object(left), Value::Object(right)) => {
            let mut keys: BTreeSet<&String> = left.keys().collect();
            keys.extend(right.keys());
            for key in keys {
                let child = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                match (left.get(key), right.get(key)) {
                    (Some(left), Some(right)) => collect_differences(&child, left, right, into),
                    (Some(left), None) => into.push(format!("{child}: {left} → absent")),
                    (None, Some(right)) => into.push(format!("{child}: absent → {right}")),
                    (None, None) => {}
                }
            }
        }
        (Value::Array(left), Value::Array(right)) => {
            if left.len() != right.len() {
                into.push(format!(
                    "{path}: {} entries → {} entries",
                    left.len(),
                    right.len()
                ));
                return;
            }
            for (index, (left, right)) in left.iter().zip(right).enumerate() {
                collect_differences(&format!("{path}[{index}]"), left, right, into);
            }
        }
        (left, right) if left == right => {}
        (left, right) => into.push(format!("{path}: {left} → {right}")),
    }
}

/// Whether a differing path is a **charge**: a counter inside a usage snapshot, which is the
/// resource half task 1.3's report keeps beside the result and `performance-gates` compares
/// separately.
fn is_a_charge_path(path: &str) -> bool {
    path.split('.').any(|segment| segment == "usage")
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
    /// The cache state the run really ran in. It is a `String` rather than a constant because a row
    /// that attaches a cache reads the label off the handle it attached ([`cache_context`]), so the
    /// record cannot describe a configuration the run did not use.
    cache: String,
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
    /// The published items' derivations, counted by their public spelling. A01's contrast is a
    /// derivation apart (`constant_pool_candidate` against `structural_consumer`), so a path that
    /// served a pool hit as an answer would publish one and be visible here.
    derivations: BTreeMap<String, usize>,
    /// How many published items carry no consumer category. Recorded rather than read as "the
    /// candidate count": the pool probe is one reason an item has a `None` consumer, and the test
    /// that reads this says which reading it takes.
    items_without_a_consumer: usize,
    /// The report as it was serialized, and the normalization of it the fingerprint is taken over.
    published: Value,
    document: Value,
    fingerprint: String,
    /// The published document with every charge record removed, and its digest: the `evidence` and
    /// `representation` planes are compared over these, because two configurations are *meant* to
    /// charge differently and may not differ in what they published.
    evidence: Value,
    evidence_fingerprint: String,
    /// Whether this row publishes a **representation** of source (the recovery rows do: text, its
    /// quality and its representation tag). Recorded per row because the `representation` plane of
    /// the gate only has an object when a pair publishes one, and a plane that is silently absent
    /// must read as absent.
    represents_a_representation: bool,
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
    run_full_range_with(corpus, verified, label, cancelled, None)
}

/// The same row with the P5 2.3 facts cache attached, when the caller names one.
///
/// The request, the snapshot, the budget and the measured section do not move: the cache is a
/// declaration on the budget the row already builds, and everything the row charges for reading,
/// enumerating and publishing stays where it was. What the row's context records as its cache state
/// is read from the handle itself, so a record cannot describe a configuration the run did not use.
fn run_full_range_with(
    corpus: &Corpus,
    verified: &Verified,
    label: &'static str,
    cancelled: bool,
    cache: Option<&FactsCache>,
) -> RunRecord {
    let engine = Engine::new();
    let mut budget = match cache {
        Some(cache) => Budget::new(limits()).with_facts_cache(cache.clone()),
        None => Budget::new(limits()),
    };
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
    context.cache = match cache {
        Some(cache) => cache_context(cache),
        None => REFERENCE_CACHE.to_string(),
    };
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
        derivations: item_derivations(&report.items),
        items_without_a_consumer: report
            .items
            .iter()
            .filter(|item| item.consumer.is_none())
            .count(),
        published: form.raw,
        document: form.normalized,
        fingerprint: form.fingerprint,
        evidence: form.evidence,
        evidence_fingerprint: form.evidence_fingerprint,
        // A full-range query publishes items and evidence; no source text is presented on this path.
        represents_a_representation: false,
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
    run_single_member_with(corpus, verified, label, premise, None)
}

/// The same row with the P5 2.3 facts cache attached: the local path reads the same class header
/// through the same reader entry, so a cache attached to the budget serves this row exactly as it
/// serves the full-range one.
fn run_single_member_with(
    corpus: &Corpus,
    verified: &Verified,
    label: &'static str,
    premise: &MemberTarget,
    cache: Option<&FactsCache>,
) -> RunRecord {
    let engine = Engine::new();
    let mut budget = match cache {
        Some(cache) => Budget::new(limits()).with_facts_cache(cache.clone()),
        None => Budget::new(limits()),
    };
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
    context.cache = match cache {
        Some(cache) => cache_context(cache),
        None => REFERENCE_CACHE.to_string(),
    };
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
        // The local row publishes the presentation of one run's payload, not a list of XRef items,
        // so it has no derivations to count. Its `published_items` is zero for the same reason.
        derivations: BTreeMap::new(),
        items_without_a_consumer: 0,
        published: form.raw,
        document: form.normalized,
        fingerprint: form.fingerprint,
        evidence: form.evidence,
        evidence_fingerprint: form.evidence_fingerprint,
        // The local row publishes the presentation of one run's payload: its text, its quality and
        // its representation tag are the plane the gate names `representation`.
        represents_a_representation: true,
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
        // agree with. The concurrency column still has no implementation to run; the cache column
        // has one as of task 2.3, and it is off unless a row attaches it.
        path: "direct",
        cache: REFERENCE_CACHE.to_string(),
        concurrency: REFERENCE_CONCURRENCY,
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

/// The published items' derivations, counted by their public spelling.
fn item_derivations(items: &[XrefItem]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        let name = serde_json::to_string(&item.derivation)
            .expect("a derivation serializes")
            .trim_matches('"')
            .to_string();
        *counts.entry(name).or_insert(0) += 1;
    }
    counts
}

// -------------------------------------------------------------------------------------------
// The comparison report (task 1.3)
// -------------------------------------------------------------------------------------------

/// Every half of two runs the `performance-gates` spec compares, plus the resources.
///
/// The list is the gate's own wording — *evidence, coverage, representation, diagnostics,
/// snapshot/view identity, peak memory proxy, cancellation and budget results* — mapped onto the
/// values a run really publishes:
///
/// | plane | what carries it here |
/// | --- | --- |
/// | evidence | `fingerprint`, taken over the whole normalized document (items, their origins and their derivations included) |
/// | representation | the same document for a row that publishes one (the local row's `RecoveryReport`) |
/// | coverage | `coverage` |
/// | diagnostics | `diagnostics` |
/// | snapshot/view identity | `identity` — the scope, physical view, runtime profile and providers the run named |
/// | peak memory proxy | `peaks` — the two high-water depths; every other counted dimension is cumulative and lives in `deltas` |
/// | cancellation | `status`, which distinguishes `cancelled` and `partial` from `complete` |
/// | budget results | `termination` — the reason the report states — beside `coverage`'s scanned/skipped ranges |
struct Verdicts {
    status: bool,
    /// The whole document with the wall clock removed, charges included. Two runs of **one**
    /// configuration are equal here; two configurations are compared on [`Verdicts::evidence`],
    /// because the charges are the difference a cache is supposed to make.
    fingerprint: bool,
    /// The documents with their charge records removed: what each run published about the input.
    /// This is the result plane, and it is what `equivalent` requires of two configurations.
    evidence: bool,
    order: bool,
    coverage: bool,
    diagnostics: bool,
    /// The snapshot/view identity both runs state. A configuration that quietly answered over
    /// another view would still publish plausible evidence, so this is compared on its own.
    identity: bool,
    /// The two **high-water** depths, equal. They are the only peaks a usage snapshot keeps (the
    /// other counted dimensions are cumulative totals), so this is the peak half of the memory
    /// proxy; the totals are reported as deltas and cannot be, because a cache is supposed to move
    /// them.
    peaks: bool,
    /// The termination **reason**, equal: `status` says a run stopped, this says why, and
    /// `Cancellation under pressure` requires the reason to be reported rather than inferred from
    /// a smaller result.
    termination: bool,
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
    /// Whether either run publishes a representation (the recovery rows do; the query rows do not),
    /// which is what decides whether the `representation` plane has anything to compare.
    representation: bool,
    /// The paths at which the two runs' charge-free documents differ. The verdicts say *whether* the
    /// evidence moved; this says *what* moved, which is the half a reviewer needs when it did.
    evidence_paths: Vec<String>,
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
            && verdicts.evidence
            && verdicts.order
            && verdicts.coverage
            && verdicts.diagnostics
            && verdicts.identity
            && verdicts.peaks
            && verdicts.termination
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
            evidence: left.evidence_fingerprint == right.evidence_fingerprint,
            order: left.order == right.order,
            coverage: left.coverage == right.coverage,
            diagnostics: left.diagnostics == right.diagnostics,
            identity: identity_of(left) == identity_of(right),
            peaks: left.usage.nested_depth == right.usage.nested_depth
                && left.usage.dependency_depth == right.usage.dependency_depth,
            termination: termination_of(left) == termination_of(right),
        },
        representation: left.represents_a_representation || right.represents_a_representation,
        evidence_paths: differing_paths(&left.evidence, &right.evidence),
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

/// The snapshot/view identity a row ran under: the scope, the physical view, the runtime profile and
/// the provider list, as the row's own context states them.
///
/// The configuration columns are deliberately **not** in it — `path`, `cache`, `concurrency` and
/// `premise` describe how the run was made, and two configurations differ in exactly those. What is
/// compared here is the input the run claims to have been given.
fn identity_of(row: &RunRecord) -> String {
    let context = &row.context;
    format!(
        "scope {} / view {} / profile {} / providers {}",
        context.scope, context.view, context.profile, context.providers
    )
}

/// The termination reason the run's own report states, `none` for a run that completed.
///
/// The query rows publish the execution report at the top level; the local and boundary recovery
/// rows publish the analysis report that holds it under `analysis`. The status tag is read back
/// against the record so a document this function misreads is a failure rather than a quiet
/// "nothing to compare".
fn termination_of(row: &RunRecord) -> String {
    let published = &row.published;
    let execution = if published.get("execution").is_some() {
        &published["execution"]
    } else {
        &published["analysis"]["execution"]
    };
    assert_eq!(
        execution["status"], row.status,
        "the published document's status tag and the row's recorded status disagree, so this \
         reading of the termination is not of the run it describes: {}",
        row.label
    );
    match execution.get("reason") {
        Some(reason) => serde_json::to_string(reason).expect("a reason renders"),
        None => "none".to_string(),
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
            format!("  evidence      {}", verdict(self.verdicts.evidence)),
            format!("  order         {}", verdict(self.verdicts.order)),
            format!("  coverage      {}", verdict(self.verdicts.coverage)),
            format!("  diagnostics   {}", verdict(self.verdicts.diagnostics)),
            format!("  identity      {}", verdict(self.verdicts.identity)),
            format!("  peaks         {}", verdict(self.verdicts.peaks)),
            format!("  termination   {}", verdict(self.verdicts.termination)),
            format!(
                "  equivalent    {}",
                if self.equivalent() {
                    "yes (the wall clock is the only allowed difference)"
                } else {
                    "no"
                }
            ),
        ];
        if !self.evidence_paths.is_empty() {
            lines.push(format!(
                "  evidence paths (right - left): {}",
                self.evidence_paths.join(", ")
            ));
        }
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
        cache: first.context.cache.clone(),
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
// The decisions tasks 2.1 and 2.2 ask for
// -------------------------------------------------------------------------------------------
//
// P5's design decides before it optimizes: "先测量再选择 … 没有收益或回归证据的优化保持 disabled"
// (decision 1), and design decision 5 refuses to fix a number before real corpus and repeats exist.
// The prose authority for what was decided is the change document. What this section adds is the
// half a document cannot hold: the state of each candidate *checked against the matrix this harness
// can actually run*, and the cache key contract anchored to the tokens that carry it in the code.
//
// So enabling a candidate is a change to a run and to this record, in that order, not an edit to a
// paragraph: `every_benefit_claim_needs_a_second_runnable_path` fails the moment a second
// configuration appears in the matrix without a measured comparison recorded beside it.

/// One optimization candidate: what it would change, what is measured about it today, and what would
/// have to exist before it could be enabled.
struct Candidate {
    id: &'static str,
    /// The change the candidate would make to the engine.
    what: &'static str,
    /// The slice that owns the decision, and the slice that would enable it.
    owner: &'static str,
    /// What the measurements recorded in this file say about it.
    basis: &'static str,
    /// The evidence that does not exist.
    missing: &'static str,
    /// The observable fact that reopens the decision. Unknown thresholds are named as unknown
    /// rather than given a number.
    trigger: &'static str,
    /// What the engine can do today, so the decision is read against a ceiling and not a hope.
    ceiling: &'static str,
    /// The shape the enabled version must take, and the gates it has to clear.
    upgrade: &'static str,
    /// Whether a benefit over the reference path has been measured. Today: no candidate has one.
    benefit: Benefit,
    /// Whether this candidate's configuration is the one the engine's default path runs.
    ///
    /// This is the field `performance-gates`' "任一语义差异 MUST 阻断默认启用" turns on. The rule is
    /// not held by the label: `a_candidate_may_be_enabled_only_while_its_differential_is_equivalent`
    /// runs each candidate's differential and refuses `Enabled` for any candidate whose two paths do
    /// not publish the same result on every plane. Today every candidate is `Disabled`, and the
    /// reference configuration of the matrix is the cache-off one, so the default path is the direct
    /// path with no store attached.
    default_state: DefaultState,
}

/// Whether a candidate's configuration is the one the engine's default path runs.
///
/// `Disabled` is the state a candidate keeps until its differential says the two paths publish the
/// same result; `Enabled` is a claim the gate re-checks by running that differential rather than by
/// reading this value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DefaultState {
    Disabled,
    Enabled,
}

/// Whether a candidate's benefit has been measured across two configurations.
enum Benefit {
    /// No comparison exists, and the text says what the reading rests on. A candidate in this state
    /// stays out of the default path (design decision 1).
    Unmeasured { why: &'static str },
    /// A comparison of the reference path against a differently-configured one. This is the only
    /// variant an enabled candidate may carry, and it cannot be written honestly for a matrix that
    /// runs one configuration.
    Measured(MeasuredBenefit),
}

/// A measured benefit: the two run labels, the configuration the candidate side ran in, the counted
/// dimensions the harness saw move, and the wall-clock medians of both sides.
///
/// Which reductions count as a release-worthy benefit is a later decision (design decision 5). What
/// is refused here is a benefit claim that names nothing at all.
struct MeasuredBenefit {
    reference: &'static str,
    candidate: &'static str,
    /// The `(cache, concurrency)` configuration the candidate side ran in. Two runs of the reference
    /// path are not a benefit, so this may not be the reference configuration.
    configuration: (&'static str, &'static str),
    /// `candidate - reference` over every counted dimension, as [`compare`] reports them.
    ///
    /// A static slice rather than a `Vec`, because the claim belongs in the record: `CANDIDATES` is
    /// a `static`, and a `Vec` cannot be built in one, so a `Vec` here would have made the
    /// "candidate carries a measured benefit" state unconstructible — the guard would have been
    /// written around a shape it could never see.
    deltas: &'static [(CountedBudgetDimension, i128)],
    reference_median_micros: u128,
    candidate_median_micros: u128,
}

impl MeasuredBenefit {
    /// Whether the claim points at something that got smaller.
    fn points_at_a_reduction(&self) -> bool {
        self.deltas.iter().any(|(_, delta)| *delta < 0)
            || self.candidate_median_micros < self.reference_median_micros
    }
}

/// The reference configuration of the matrix — `direct`, no cache, one sequential scan — declared
/// once, so the rows and the decision record cannot describe the harness differently. A benefit
/// claim's candidate side is refused if it carries these values: that would be the reference path
/// measured twice.
///
/// It is still the row that runs with **no cache attached**. Task 2.3 built the cache this label
/// used to say did not exist, and built it disabled: a reference row attaches nothing
/// ([`Budget::new`]), and `the_engine_has_one_disabled_facts_cache_and_no_index_or_scheduler` holds
/// that from the other side. The label sorts before [`CACHE_ON`], so the reference configuration of
/// the matrix stays this one — the assertion is in
/// `the_cache_path_publishes_the_same_result_and_reports_what_it_saved`.
const REFERENCE_CACHE: &str = "off: the facts cache exists and is disabled by default, so a \
                               reference row runs the direct path with no cache attached";
const REFERENCE_CONCURRENCY: &str = "1: no parallel scheduler exists (P5 2.x owns one), so every \
                                      row is one sequential scan";

/// The configuration the cache-on row really runs in: the label is this literal, and the test that
/// compares the two paths asserts it equals the label the handle itself describes
/// ([`cache_context`]). Both bounds are the row's own, and the identity is this build's.
const CACHE_ON: &str =
    "on: facts cache (registry 71 entry format 1), capacity 64 entries / 8388608 retained bytes";

/// What a cache-on row may hold: 64 answers, and 8 MiB of retained weight. Every class of the
/// corpus fits; the bounds are declared rather than convenient, because an unbounded store is the
/// state a benchmark would grow into without saying so.
const CACHE_CAPACITY: FactsCapacity = FactsCapacity::new(64, 8 * 1024 * 1024);

/// A fresh cache for one row, under this build's identity.
fn facts_cache() -> FactsCache {
    FactsCache::current(CACHE_CAPACITY)
}

/// The cache state a context records, read from the handle the run attached.
fn cache_context(cache: &FactsCache) -> String {
    format!("on: {}", cache.describe())
}

/// The measured difference of the cache-on row over its reference, by counted dimension, exactly as
/// [`compare`] reports it for the two rows (`candidate - reference`).
///
/// Every entry is **recomputed** by `the_cache_path_publishes_the_same_result_and_reports_what_it_saved`
/// from the two rows themselves and compared against this slice, so the record cannot drift from
/// what the harness measures; the medians below are one measurement on one machine, printed beside
/// the sample by the repeated run, and no test asserts them.
static CACHE_BENEFIT_DELTAS: [(CountedBudgetDimension, i128); 2] = [
    (CountedBudgetDimension::ClassBytes, -555),
    (CountedBudgetDimension::AttributeBytes, -87),
];

/// Wall-clock medians of the two rows, in microseconds, from one run of the repeated measurement on
/// the machine task 1.2 recorded (200 repeats, single-threaded, `minimal-jar`).
///
/// They are recorded because a measured benefit has to state what it compared — the reference row's
/// median was **137 µs** (min 124, p90 148, halves 136/141) and the cache-on row's **116 µs**
/// (min 106, p90 121, halves 115/118), a difference larger than the spread between the halves of
/// either row — and they are **not** asserted: the deterministic half of the claim is
/// [`CACHE_BENEFIT_DELTAS`], and a wall-clock assertion on a shared machine is a flake. One machine,
/// one corpus of hundreds of bytes: this is a reading, not a threshold (design decision 5).
const CACHE_REFERENCE_MEDIAN_MICROS: u128 = 137;
const CACHE_CANDIDATE_MEDIAN_MICROS: u128 = 116;

/// Task 2.1's candidates and task 2.2's, all disabled, each with what would reopen it.
static CANDIDATES: [Candidate; 3] = [
    Candidate {
        id: "merged-queries/single-flight",
        default_state: DefaultState::Disabled,
        what: "More than one request over one snapshot shares one scan: identical work runs once, and \
               each subscriber gets the same published result with its own budget and its own \
               termination status (design decision 4; the `measured-execution` shared-query \
               scenario).",
        owner: "P5 2.1 owns the decision; no slice owns running two overlapping requests, which is \
                what enabling it would need first.",
        basis: "Nothing to share: every entry point takes one request at a time, and no row in this \
                harness runs two requests over one snapshot, so the shared case has no measured cost \
                and its cancellation rule has no subject. What is measured is the single request — \
                ~130 µs median over a 659 B archive and a 303 B class (task 1.2), reproducible to \
                ~10% on this machine, with the first run in a process up to 13× the median.",
        missing: "A caller that really overlaps two requests over one snapshot, and a repeated \
                  measurement of that pair. Without a second subscriber there is no denominator for \
                  'shared' to divide, and no way to exercise 'one subscriber cancels while the other \
                  keeps waiting'.",
        trigger: "When an entry point or a benchmark row can run two requests over one snapshot: \
                  measure the overlapped pair against the same two requests run sequentially, through \
                  this harness. The trigger is that row existing, not a size of win — which win would \
                  be worth a default-on change is not fixed (design decision 5).",
        ceiling: "One request at a time, one configuration in the matrix. `compare` can compare any \
                  two runs, but there are not yet two runs of the same work to compare.",
        upgrade: "A shared request may only publish what the sequential pair publishes (`compare`'s \
                  status, fingerprint, order, coverage and diagnostics, equal), must not reset the \
                  budget when a subscriber cancels (design decision 3), and must keep the cancelled \
                  subscriber's termination separate from the other subscriber's (decision 4).",
        benefit: Benefit::Unmeasured {
            why: "One path, one configuration: a benefit is a difference between two runs, and the \
                  harness can produce one run of this request at a time.",
        },
    },
    Candidate {
        id: "fine-grained-parallel",
        default_state: DefaultState::Disabled,
        what: "Splitting one request's scanning work across workers and merging their results in \
               stable order (design decision 4), leaving coverage and partial semantics as they are.",
        owner: "P5 2.1 owns the decision; no slice implements a scheduler.",
        basis: "The reference path's work on this corpus is hundreds of bytes: the local row \
                materializes exactly one body (303 charged read bytes, 29 analysis steps, 86 IR \
                items) of the three bodies its class declares, and the two full-range rows charge no \
                header and no body while materializing 555 and 909 class bytes. A worker's fixed \
                cost — spawn, hand-off, ordered merge — is measured nowhere in this file, so 'work \
                saved' has never been put next to 'scheduling paid'.",
        missing: "A corpus row whose work is large enough for a split to be the dominant term, a \
                  second path to split it, and the scheduler's own cost measured through the same \
                  harness rather than assumed. The corpus in this repository cannot decide it: the \
                  largest subject is 659 bytes.",
        trigger: "When the harness has a second path and a row whose resource report puts the \
                  per-unit share of the charge first: the candidate difference then has to clear the \
                  harness's own repeat spread on the machine measuring it (today ~10% between full \
                  measurements and up to ~12% between halves of one, so no smaller difference can be \
                  reported as a gain at all). No threshold for 'worth enabling' is fixed here \
                  (design decision 5).",
        ceiling: "Single-threaded: no thread is spawned anywhere in the engine (guarded by \
                  `the_engine_has_one_disabled_facts_cache_and_no_index_or_scheduler`), so one \
                  request is one core; and the first run in a process reaches 13× the median, which \
                  is larger than anything this corpus could show for a scheduling change.",
        upgrade: "A parallel scan has to publish byte-identical results and keep the origin order \
                  (`compare` requires order equality, not a stable permutation), and it has to hold \
                  `Cancellation under pressure` — which needs the pressure corpus of task 3.2: ZIP \
                  bomb, condy graph, irreducible CFG and missing-dependency rows, none of which are \
                  measured today.",
        benefit: Benefit::Unmeasured {
            why: "Nothing is scheduled to compare against, and the work one request does over this \
                  corpus is small enough that the harness cannot distinguish a change of the size a \
                  scheduler could make from its own repeat spread.",
        },
    },
    Candidate {
        id: "facts-cache/index",
        default_state: DefaultState::Disabled,
        what: "Reusing facts between requests — CP/header, X1, resolution, IR/source — under a key \
               bound to the semantic inputs of the layer that holds them (design decision 2; the \
               `facts-cache` spec). **Task 2.3 built the CP/Header layer**, disabled by default; the \
               three layers above it are still not built.",
        owner: "P5 2.2 owns the decision; 2.3 built the CP/Header layer, its invalidation and the \
                direct-path fallback; the slices above it own their own layers.",
        basis: "A facts cache exists as of 2.3 and is **attached by a caller, never enabled by the \
                engine** (`the_engine_has_one_disabled_facts_cache_and_no_index_or_scheduler`; \
                `p5_facts_cache.rs` holds its identity, invalidation and fallback). What it answers \
                is the CP/header parse: one request re-reads and re-parses the same class once per \
                consumer stream, which is what the rows show — the full-range archive row charges \
                555 class bytes for a 185-byte class (three parses of one class) and the local row \
                charges 303 for one, while the cache-on row charges no class byte for the same \
                result. The reads, the enumeration, the origins, the coverage and the diagnostics \
                are untouched by it, because the read still happens and only the parse is answered \
                from the store: the warm row still charges 627 read bytes, and the attribute bytes \
                the scan reads per consumer are still charged — only the parse's own share of them \
                is gone (87 of 174 on this row). The warm row's median is 116 µs against the \
                reference's 137 µs in the repeated measurement that recorded them: one machine, one \
                corpus of hundreds of bytes, no threshold fixed.",
        missing: "The layers above CP/Header: X1's class/resource digest and consumer schema, \
                  resolution's symbol/source context with view/domain/platform and the provider \
                  snapshot, and IR/source's method content with the analysis/recovery versions, the \
                  output level and the naming configuration. Two of those (IR, recovery) still have \
                  no version identity in the code at all, so an entry above this layer could not yet \
                  state what it depends on.",
        trigger: "When the layer above is the dominant cost of repeated requests over one snapshot — \
                  observable as rows in this harness whose charged dimensions repeat above the \
                  CP/header parse — and every dimension of the key record below that layer has a \
                  carrier: build that layer behind its own disabled switch and measure it row by row \
                  against the direct path. This layer's own trigger is met and recorded here: its \
                  dimensions all had carriers, and the comparison is measured.",
        ceiling: "One layer is built and every layer above still re-reads and re-materializes what it \
                  needs; nothing above the CP/header parse survives a request. The dimensions without \
                  a version identity (IR, recovery) cannot even be expressed yet, so the upper layers \
                  cannot be keyed completely.",
        upgrade: "An entry above this layer has to bind its layer's semantic inputs, keep physical \
                  origins separate, never let an incomplete entry stand in for a complete one, and \
                  fall back to the direct path inside the remaining budget (`facts-cache`; design \
                  decision 3) — the mechanism 2.3 built once, per layer. `Cold and warm results` \
                  (A15) and `Optimized versus direct path` are the gates, and `Index candidate \
                  requires verification` is where A01 returns: a hit is a candidate, and a consumer \
                  or a definition still has to verify it.",
        benefit: Benefit::Measured(MeasuredBenefit {
            reference: "full-range-xref/minimal-jar",
            candidate: "cache-on-warm/full-range-xref/minimal-jar",
            configuration: (CACHE_ON, REFERENCE_CONCURRENCY),
            deltas: &CACHE_BENEFIT_DELTAS,
            reference_median_micros: CACHE_REFERENCE_MEDIAN_MICROS,
            candidate_median_micros: CACHE_CANDIDATE_MEDIAN_MICROS,
        }),
    },
];

/// The semantic inputs a facts cache key has to cover, and where each one lives today.
///
/// `design.md` decision 2 names the dimensions (snapshot, view, platform, registry, query, IR,
/// recovery, pass, budget) and `specs/facts-cache` says which layer has to bind what. Task 2.2's
/// shape question — key type or record — is answered **recorded, not built**, for three reasons that
/// are facts about this repository rather than preferences:
///
/// * a key type has no caller: nothing reads a key, so its shape would be settled by guesswork and
///   the first real cache would rewrite it;
/// * two dimensions have no identity in the code at all (the IR and recovery versions), so building
///   the key would first mean inventing the versions it is supposed to hash — which is what P5's
///   Non-Goals refuse to preset;
/// * a key can only be *verified* by comparing a cached run against a direct one (A15's cache half):
///   a test that pinned today's guesses would pin a guess as truth.
///
/// The record is still anchored to code rather than to prose: every `Present` carrier names a token
/// that has to appear in the engine sources, so renaming or deleting the thing a dimension depends
/// on fails `the_recorded_key_dimensions_are_the_ones_the_decision_names`.
struct KeyDimension {
    /// The name `design.md` decision 2 gives this input.
    name: &'static str,
    /// The cache layers whose entries have to include it, from `facts-cache`: `cp-header`, `x1`,
    /// `resolution`, `ir-source`, joined by `+`.
    layers: &'static str,
    /// The requirement this dimension is read from, since `facts-cache` names one input that
    /// decision 2's list has no dimension for.
    source: &'static str,
    /// Where the input is today.
    carrier: Carrier,
}

/// Where a key dimension's input is today.
enum Carrier {
    /// The input exists in the engine, under this token.
    Present {
        what: &'static str,
        token: &'static str,
    },
    /// The dimension is required and has no carrier: a key over it means adding the identity first.
    Absent {
        why: &'static str,
        closed_by: &'static str,
    },
}

static KEY_DIMENSIONS: [KeyDimension; 10] = [
    KeyDimension {
        name: "snapshot",
        layers: "cp-header+x1+resolution+ir-source",
        source: "design decision 2",
        carrier: Carrier::Present {
            what: "The identity of the opened artifact, which the query layer already binds into its \
                   cursor and refuses to mix across snapshots (P1: `a18_open_snapshot_stays_stable_\
                   and_a_reopened_one_rejects_old_cursors`, `cursor_mismatches_bind_snapshot_view_\
                   relation_and_schema`).",
            token: "ArtifactSnapshot",
        },
    },
    KeyDimension {
        name: "view",
        layers: "resolution+ir-source",
        source: "design decision 2",
        carrier: Carrier::Present {
            what: "The physical view a request names, which selects which entries of a \
                   multi-release artifact answer (P1 A06; P4's runtime matrix).",
            token: "PhysicalView",
        },
    },
    KeyDimension {
        name: "platform",
        layers: "resolution+ir-source",
        source: "design decision 2",
        carrier: Carrier::Present {
            what: "The runtime profile a request resolves under: the same bytes answer differently \
                   under another profile, and the profile is what selects the entry.",
            token: "RuntimeProfile",
        },
    },
    KeyDimension {
        name: "registry",
        layers: "cp-header+x1+resolution",
        source: "design decision 2, and `facts-cache`'s `parser/registry 版本和 parse policy`",
        carrier: Carrier::Present {
            what: "The release/modern registry a dialect is read under, together with the policy \
                   tables that decide what a scan reports and what the recovery layer claims \
                   (the reader's patterns, the query layer's plugins).",
            token: "HIGHEST_REGISTERED_MAJOR",
        },
    },
    KeyDimension {
        name: "query",
        layers: "x1+resolution",
        source: "design decision 2",
        carrier: Carrier::Present {
            what: "The request itself — the cursor's binding digest hashes it — and the schema tag \
                   the report carries, so a relation or a consumer category cannot read another's \
                   entry.",
            token: "QUERY_ENGINE_SCHEMA",
        },
    },
    KeyDimension {
        name: "dependency-snapshot",
        layers: "resolution",
        source: "`facts-cache`'s resolution layer (`symbol/source context`, \
                 `view/domain/platform/dependency snapshot`); decision 2's list has no dimension of \
                 its own for it",
        carrier: Carrier::Present {
            what: "The header providers a request resolves against. A missing dependency is a stated \
                   negative rather than a failure (A11), and the same query answers differently once \
                   a provider is added — `facts-cache`'s `Dependency becomes available`.",
            token: "providers",
        },
    },
    KeyDimension {
        name: "IR",
        layers: "ir-source",
        source: "design decision 2",
        carrier: Carrier::Absent {
            why: "The IR is a type, not a version: nothing in the analysis layer states which IR \
                  revision a derived fact came from, and an entry that cannot name its revision \
                  cannot be invalidated when one changes (design decision 2's risk: 变更版本时强制 \
                  失效).",
            closed_by: "The slice that builds an IR/source-layer cache and therefore has to \
                        invalidate it.",
        },
    },
    KeyDimension {
        name: "recovery",
        layers: "ir-source",
        source: "design decision 2",
        carrier: Carrier::Absent {
            why: "No recovery version identity exists either. The nearest thing is the pass set \
                  below, which is narrower than what the recovery layer's answers depend on (the \
                  naming configuration and the output level are inputs to it as well).",
            closed_by: "The same slice: the recovery identity is added by whoever has to invalidate \
                        recovered facts.",
        },
    },
    KeyDimension {
        name: "pass",
        layers: "ir-source",
        source: "design decision 2",
        carrier: Carrier::Present {
            what: "The named recovery passes a run applies, which decide whether a pattern is \
                   claimed or refused.",
            token: "PASSES",
        },
    },
    KeyDimension {
        name: "budget",
        layers: "cp-header+x1+resolution+ir-source",
        source: "design decision 2",
        carrier: Carrier::Present {
            what: "The limits a fact was produced under. A truncated scan is a partial answer, and \
                   `facts-cache` refuses to let an incomplete entry stand in for a complete one \
                   (design decision 3: no budget is reset by re-running).",
            token: "Limits",
        },
    },
];

/// The `(cache, concurrency)` configurations the matrix can be run in, read from the rows this
/// harness produces rather than declared beside them.
///
/// This is what keeps the decisions above tied to runs: if a slice wires a second path into the
/// matrix, this set grows, and the decision record has to grow with it.
fn runnable_configurations(corpus: &Corpus) -> BTreeSet<(String, String)> {
    published_rows(corpus)
        .into_iter()
        .map(|row| {
            (
                row.context.cache.to_string(),
                row.context.concurrency.to_string(),
            )
        })
        .collect()
}

/// One engine source file, read by the guards below.
struct EngineSource {
    path: String,
    text: String,
}

/// Every `.rs` file of the engine crates and the facade, in a deterministic order.
///
/// The guards below check *absences* — no cache, no index, no scheduler — and no run can show that
/// something does not exist, so they read the source. The scope is the engine the decisions are
/// about: `crates/*/src` and the facade's `src/`. Test code, fixtures, fuzz targets and anything
/// reached through a dependency are outside it; so is a cache hidden in a local variable.
fn engine_sources() -> Vec<EngineSource> {
    let root = repository_root();
    let crates = root.join("crates");
    let mut directories = vec![root.join("src")];
    let mut crate_directories: Vec<PathBuf> = fs::read_dir(&crates)
        .unwrap_or_else(|error| panic!("read {}: {error}", crates.display()))
        .map(|entry| entry.expect("a directory entry reads").path())
        .collect();
    crate_directories.sort();
    for crate_directory in crate_directories {
        directories.push(crate_directory.join("src"));
    }
    let mut sources = Vec::new();
    for directory in directories {
        collect_sources(&directory, &mut sources);
    }
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    sources
}

fn collect_sources(directory: &Path, into: &mut Vec<EngineSource>) {
    // A directory that is not there is not an engine source tree: the guard is about the trees that
    // exist rather than about a list of crate names that would go stale.
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries
        .map(|entry| entry.expect("a directory entry reads").path())
        .collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            collect_sources(&path, into);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
            let relative = path
                .strip_prefix(repository_root())
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            into.push(EngineSource {
                path: relative,
                text,
            });
        }
    }
}

/// The machinery that has to stay **absent** from the engine: an index, a scheduler, a thread.
///
/// These five needles are task 2.1's and task 2.2's, unchanged, and their scope is unchanged too:
/// they catch a module or a spawn site in `crates/*/src` and `src/`, and they do not catch an index
/// under another name (`IndexTable`), one kept in a local variable, anything reached through a
/// dependency, or any file outside those trees. `Index…` is deliberately not a name prefix: this
/// repository already declares `pub struct IndexCall` in the recovery layer — a dispatch-table read,
/// not an index — and a rule that refused that line would be a rule about spelling rather than about
/// machinery.
///
/// What changed with task 2.3 is not this list but the claim beside it. 2.1/2.2 asserted that **no
/// cache existed**; 2.3 built one, so the cache half of that claim is now made of three facts this
/// test can hold — the store is declared in one module, **nothing in the engine constructs one**, and
/// the budget every existing entry point builds carries none — while the index and the scheduler are
/// still asserted absent exactly as before.
const MACHINERY: [&str; 5] = [
    "mod index",
    "thread::spawn",
    "thread::scope",
    "rayon",
    "spawn_blocking",
];

/// The tokens that would mean the engine itself switches the facts cache on.
///
/// A cache a caller attaches is the disabled-by-default state; a cache the engine constructs is a
/// cache in the default path, whatever the switch is called. The needles name the three ways one is
/// built, so a later slice that wires one into the CLI, the facade or a default budget fails here
/// and has to move the record with it — the same shape as 2.1/2.2's rule for a second path.
const CACHE_CONSTRUCTIONS: [&str; 3] = [
    "FactsCache::new(",
    "FactsCache::current(",
    "FactsCache::over(",
];

/// The lines of `text` that name a needle, as `line: needle`.
fn machinery_hits(text: &str, needles: &[&str]) -> Vec<String> {
    let mut hits = Vec::new();
    for (number, line) in text.lines().enumerate() {
        for needle in needles {
            if line.contains(needle) {
                hits.push(format!("{}: {needle}", number + 1));
            }
        }
    }
    hits
}

/// The rule a candidate's claim has to satisfy, as a function of the configurations the matrix can
/// run: `Ok(())` when the claim is one this harness could repeat, `Err(reason)` when it is not.
///
/// It is a function rather than a block inside the test so the rule itself is testable: the record as
/// it stands has to pass it, and the claims someone would write while turning a candidate on — a
/// benefit measured in the reference configuration, one measured in a configuration that does not
/// run, one that points at nothing smaller — have to be refused
/// (`the_benefit_rule_refuses_a_claim_the_matrix_cannot_back`).
fn check_candidate(
    candidate: &Candidate,
    configurations: &BTreeSet<(String, String)>,
) -> std::result::Result<(), String> {
    for (field, text) in [
        ("what", candidate.what),
        ("owner", candidate.owner),
        ("basis", candidate.basis),
        ("missing", candidate.missing),
        ("trigger", candidate.trigger),
        ("ceiling", candidate.ceiling),
        ("upgrade", candidate.upgrade),
    ] {
        if text.trim().is_empty() {
            return Err(format!(
                "{}: `{field}` is empty. A candidate kept out of the default path has to say what it \
                 would change, what is measured about it, what is missing, what would reopen it, \
                 what the engine can do today and what shape enabling it must take: design \
                 decision 1 keeps it disabled, not undocumented.",
                candidate.id
            ));
        }
    }
    let reference = configurations
        .iter()
        .next()
        .ok_or_else(|| {
            "the matrix produced no row, so nothing can be measured against it".to_string()
        })?
        .clone();
    match &candidate.benefit {
        Benefit::Unmeasured { why } => {
            if why.trim().is_empty() {
                return Err(format!(
                    "{} is unmeasured and does not say what the reading rests on",
                    candidate.id
                ));
            }
            Ok(())
        }
        Benefit::Measured(measured) => {
            if measured.reference == measured.candidate {
                return Err(format!(
                    "{}: a benefit measured over one run is a measurement of the reference path, not \
                     of the candidate",
                    candidate.id
                ));
            }
            let configuration = (
                measured.configuration.0.to_string(),
                measured.configuration.1.to_string(),
            );
            if !configurations.contains(&configuration) {
                return Err(format!(
                    "{} records a benefit measured as cache `{}` with concurrency `{}`, and the \
                     matrix cannot be run that way. A benefit measured in a configuration this \
                     harness cannot reproduce is not a measurement. The reference configuration is \
                     cache `{}` with concurrency `{}`.",
                    candidate.id,
                    measured.configuration.0,
                    measured.configuration.1,
                    reference.0,
                    reference.1
                ));
            }
            if configuration == reference {
                return Err(format!(
                    "{} records a benefit measured in the reference configuration (cache `{}`, \
                     concurrency `{}`): that is the reference path measured twice, which is the \
                     measurement-shaped claim P5's design (5) and the `performance-gates` spec \
                     refuse. A benefit needs a second path to run.",
                    candidate.id, reference.0, reference.1
                ));
            }
            if !measured.points_at_a_reduction() {
                return Err(format!(
                    "{} claims a measured benefit and names no counted dimension that got smaller \
                     and no shorter median: {:?}, candidate {} µs over reference {} µs",
                    candidate.id,
                    measured.deltas,
                    measured.candidate_median_micros,
                    measured.reference_median_micros
                ));
            }
            Ok(())
        }
    }
}

/// The same candidate with a different benefit: the decision texts stay, the claim changes. Used by
/// the self-test below, and the shape a real benefit claim takes when one exists.
fn with_benefit(candidate: &Candidate, benefit: Benefit) -> Candidate {
    Candidate {
        id: candidate.id,
        what: candidate.what,
        owner: candidate.owner,
        basis: candidate.basis,
        missing: candidate.missing,
        trigger: candidate.trigger,
        ceiling: candidate.ceiling,
        upgrade: candidate.upgrade,
        benefit,
        // The self-test carries the state of the candidate it copies: what a claim can be is not a
        // statement about which path the engine runs, and a copy that invented one would let a
        // refusal test look like a gate.
        default_state: candidate.default_state,
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
/// ordering and comparison claims are stated over, and the cache-on row task 2.3 added.
///
/// The cache-on row is the second `(cache, concurrency)` configuration the matrix can run, which is
/// what task 2.1's rule was written for: the day a second path appears, the record has to carry a
/// measured comparison beside it (`every_benefit_claim_needs_a_second_runnable_path`). It is a
/// **warm** row, and the priming run is its recorded premise rather than part of the measured
/// section, because a cache that has answered nothing yet is the direct path with extra steps.
fn published_rows(corpus: &Corpus) -> Vec<RunRecord> {
    let jar = verified(corpus, &SUBJECTS[0]);
    let class = verified(corpus, &SUBJECTS[1]);
    let premise = locate_member(&class.content);
    vec![
        run_full_range(corpus, &jar, "full-range-xref/minimal-jar", false),
        run_full_range(corpus, &class, "full-range-xref/v52-class", false),
        run_single_member(corpus, &class, "single-member/v52-class", &premise),
        run_full_range_warm(
            corpus,
            &jar,
            "cache-on-warm/full-range-xref/minimal-jar",
            &facts_cache(),
        ),
    ]
}

/// The cache-on row: one priming run through the cache, then the measured one.
///
/// The premise is the same kind of exclusion the local row's header read is, and it is stated as
/// one: the warm-up is real work, it is charged to its own budget, and it is recorded beside the row
/// so the numbers cannot be read as "one request paid for all of this".
fn run_full_range_warm(
    corpus: &Corpus,
    verified: &Verified,
    label: &'static str,
    cache: &FactsCache,
) -> RunRecord {
    let priming = run_full_range_with(
        corpus,
        verified,
        "cache-on-warm-up/full-range-xref/minimal-jar",
        false,
        Some(cache),
    );
    let mut row = run_full_range_with(corpus, verified, label, false, Some(cache));
    row.context.premise = format!(
        "one priming run of the same request through the same cache, measured nowhere (it charged \
         {}); the row below is the second request over the same bytes",
        charges_line(&priming.usage, &CountedBudgetDimension::ALL)
    );
    assert!(
        priming.fingerprint == row.fingerprint
            || priming.usage.class_bytes != row.usage.class_bytes,
        "the priming run answered nothing the measured run had to parse, so the row is not warm: \
         priming charged {} and the row charged {}",
        charges_line(&priming.usage, &CountedBudgetDimension::ALL),
        charges_line(&row.usage, &CountedBudgetDimension::ALL)
    );
    row
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
        4,
        "the child process printed {} marker lines for the four published rows (the three direct \
         rows of task 1.3 and the cache-on row task 2.3 added); if `{MARKER_TEST}` no longer names a \
         test in this binary the child ran nothing, which is not a comparison:\n{stdout}",
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

/// A15's cache half, in the shape task 1.3's report can state it: the cold and the warm path over
/// the same bytes, compared field by field.
///
/// Three comparisons are needed, because they fail differently:
///
/// * **direct versus warm** — the result planes (status, order, coverage, diagnostics) are equal, so
///   answering a parse from the store published nothing else.
/// * **cold versus warm** — the same four planes are equal, and **every difference in the published
///   document is a charge**. That is the whole content of "transparent" here: 1.3's fingerprint
///   normalizes the wall clock and nothing else, so a run that paid for one parse less has a
///   different fingerprint — by exactly the counters of the parse it did not run, and by nothing
///   else. The test states that instead of hiding it behind a second normalization.
/// * **warm versus warm** — *all five* verdicts equal, fingerprint included: the warm path is a
///   function of its input alone, and a second warm request pays exactly what the first warm one
///   paid.
///
/// The record is cross-checked at the end: the two rows of the matrix are compared, and the
/// difference between them has to be the one `CANDIDATES` carries, so the benefit a later slice
/// reads is the one the harness measures.
#[test]
fn the_cache_path_publishes_the_same_result_and_reports_what_it_saved() {
    let corpus = corpus();
    let jar = verified(&corpus, &SUBJECTS[0]);
    let class = verified(&corpus, &SUBJECTS[1]);
    let premise = locate_member(&class.content);

    // The configuration label of the second path is the handle's own description, and the reference
    // configuration of the matrix is still the cache-off one (the rule in
    // `every_benefit_claim_needs_a_second_runnable_path` reads the *first* configuration as the
    // reference).
    let probe = facts_cache();
    assert_eq!(
        cache_context(&probe),
        CACHE_ON,
        "the cache-on label this file records is not the one the handle describes, so the record and \
         the run describe different configurations"
    );
    let configurations = runnable_configurations(&corpus);
    assert_eq!(
        configurations.iter().next().map(|(cache, _)| cache.clone()),
        Some(REFERENCE_CACHE.to_string()),
        "the reference configuration of the matrix is no longer the cache-off one: {configurations:?}"
    );
    assert_eq!(
        configurations.len(),
        2,
        "the matrix runs {} configuration(s); task 2.3 added exactly one (the cache-on one): \
         {configurations:?}",
        configurations.len()
    );

    #[derive(Clone, Copy)]
    enum Row {
        FullRange,
        SingleMember,
    }
    let run = |row: Row, cache: Option<&FactsCache>| match row {
        Row::FullRange => {
            run_full_range_with(&corpus, &jar, "full-range-xref/minimal-jar", false, cache)
        }
        Row::SingleMember => {
            run_single_member_with(&corpus, &class, "single-member/v52-class", &premise, cache)
        }
    };
    let verdict = |equal: bool| if equal { "equal" } else { "different" };

    for (name, row) in [
        ("the archive", Row::FullRange),
        ("the control class", Row::SingleMember),
    ] {
        let direct = run(row, None);
        let store = facts_cache();
        let cold = run(row, Some(&store));
        let warm = run(row, Some(&store));
        let warm_again = run(row, Some(&store));
        assert_eq!(
            direct.status, "complete",
            "{name}: the reference run did not complete, so there is no baseline"
        );

        let first = compare(&direct, &warm);
        assert!(
            first.verdicts.status
                && first.verdicts.order
                && first.verdicts.coverage
                && first.verdicts.diagnostics,
            "{name}: answering a parse from the store changed a result plane:\n{}",
            first.lines().join("\n")
        );
        println!();
        println!("row {name}");
        println!(
            "  direct → warm fingerprint {}",
            verdict(first.verdicts.fingerprint)
        );

        let cold_warm = compare(&cold, &warm);
        assert!(
            cold_warm.verdicts.status
                && cold_warm.verdicts.order
                && cold_warm.verdicts.coverage
                && cold_warm.verdicts.diagnostics,
            "{name}: the warm run published a different result than the cold one:\n{}",
            cold_warm.lines().join("\n")
        );
        let moved = differing_paths(&cold.document, &warm.document);
        let non_charge: Vec<&String> = moved
            .iter()
            .filter(|path| !is_a_charge_path(path))
            .collect();
        assert!(
            non_charge.is_empty(),
            "{name}: the cold and the warm run differ outside the charge records, so 1.3's \
             fingerprint difference is not only the resources: {non_charge:?}"
        );
        // The publication order is required to be equal, not merely stable, so the *only* thing the
        // cold/warm pair may move is what each run paid.
        for line in cold_warm.lines() {
            println!("  {line}");
        }
        println!("  differing paths (all charges): {moved:?}");

        let warm_pair = compare(&warm, &warm_again);
        assert!(
            warm_pair.verdicts.fingerprint,
            "{name}: two warm runs over the same bytes published different documents:\n{}",
            warm_pair.lines().join("\n")
        );
        assert!(
            warm_pair.equivalent(),
            "{name}: two warm runs do not compare equivalent:\n{}",
            warm_pair.lines().join("\n")
        );
        assert!(
            store.report().hits > 0,
            "{name}: no parse was answered from the store, so the comparison is not about a cache: \
             {:?}",
            store.report()
        );
        println!(
            "  warm → warm again fingerprint {} after {} hit(s) over the request",
            verdict(warm_pair.verdicts.fingerprint),
            store.report().hits
        );
    }

    // The record: the two rows of the matrix, the difference between them, and the benefit
    // `CANDIDATES` carries. A benefit claim that no row measures any more is a claim about nothing.
    let rows = published_rows(&corpus);
    let reference = rows
        .iter()
        .find(|row| row.label == "full-range-xref/minimal-jar")
        .expect("the reference row is in the matrix");
    let candidate = rows
        .iter()
        .find(|row| row.label == "cache-on-warm/full-range-xref/minimal-jar")
        .expect("the cache-on row is in the matrix");
    assert_eq!(
        (
            reference.context.cache.as_str(),
            candidate.context.cache.as_str()
        ),
        (REFERENCE_CACHE, CACHE_ON),
        "the two rows do not run the two configurations their contexts state"
    );
    let recorded = compare(reference, candidate);
    assert!(
        recorded.verdicts.status
            && recorded.verdicts.order
            && recorded.verdicts.coverage
            && recorded.verdicts.diagnostics,
        "the cache-on row of the matrix publishes a different result than its reference:\n{}",
        recorded.lines().join("\n")
    );
    let measured: Vec<(CountedBudgetDimension, i128)> = recorded
        .deltas
        .iter()
        .copied()
        .filter(|(_, delta)| *delta != 0)
        .collect();
    assert_eq!(
        measured,
        CACHE_BENEFIT_DELTAS.to_vec(),
        "`CANDIDATES` records a benefit that is not the difference between the two rows:\n{}",
        recorded.lines().join("\n")
    );
    let facts_cache_candidate = CANDIDATES
        .iter()
        .find(|candidate| candidate.id == "facts-cache/index")
        .expect("the facts-cache candidate is in the record");
    assert!(
        matches!(
            &facts_cache_candidate.benefit,
            Benefit::Measured(measured) if measured.configuration.0 == CACHE_ON
                && measured.configuration.1 == REFERENCE_CONCURRENCY
        ),
        "the facts-cache candidate does not record the configuration the cache-on row runs in"
    );
    println!();
    for line in recorded.lines() {
        println!("{line}");
    }
    println!(
        "  recorded in `CANDIDATES`: {} dimension(s) moved, candidate {} µs over reference {} µs \
         (one machine, one run of the repeated measurement)",
        measured.len(),
        CACHE_CANDIDATE_MEDIAN_MICROS,
        CACHE_REFERENCE_MEDIAN_MICROS
    );
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

/// The premise both decisions rest on, machine-checked: the engine has **one** facts cache — the
/// CP/Header layer task 2.3 built — and nothing enables it, and there is still no index and no
/// scheduler.
///
/// This test used to assert that no cache existed at all. That claim is false now, and replacing it
/// with a weaker one would be worse than deleting it, so it is replaced by three claims that are
/// together stronger than the old absence:
///
/// * the index and the scheduler are still absent, in exactly the needles 2.1/2.2 fixed;
/// * the cache is declared in **one** module, and the only other engine files that name it are the
///   crate root that declares the module and the budget that carries a handle — so a second cache
///   cannot appear quietly;
/// * **nothing in the engine constructs one** ([`CACHE_CONSTRUCTIONS`]), which is what
///   disabled-by-default means as a structural fact, and [`Budget::new`] — the constructor every
///   existing entry point already uses — carries none.
///
/// The guard is deliberately coarse and its blind spots are stated rather than discovered later: it
/// finds a cache or index *module or type*, a construction site and a spawned thread, and it does not
/// find a memo table kept in a local variable, anything reached through a dependency, or any file
/// outside `crates/*/src` and `src/`. That is the same kind of boundary P2's A17 guard states for the
/// modules outside its guarded set: a guard is a supplement, and the sentence it backs says which
/// half it covers.
#[test]
fn the_engine_has_one_disabled_facts_cache_and_no_index_or_scheduler() {
    let sources = engine_sources();
    assert!(
        sources.len() > 10,
        "the guard read {} engine source file(s), which is too few to be the engine: the walk or \
         the working directory is wrong, and an empty scan would make the assertions below vacuous",
        sources.len()
    );

    // (1) The index and the scheduler, in the spellings 2.1 and 2.2 fixed: still nothing.
    let mut absent = Vec::new();
    for source in &sources {
        assert!(
            !source.text.trim().is_empty(),
            "{} is empty, so it proves nothing",
            source.path
        );
        for hit in machinery_hits(&source.text, &MACHINERY) {
            absent.push(format!("{}:{hit}", source.path));
        }
    }
    assert!(
        absent.is_empty(),
        "the engine now declares machinery that P5 tasks 2.1 and 2.2 decided not to build: \
         {absent:?}\nEvery decision in this file is recorded against its absence — an index to \
         extend, a scheduler to split one request with. If the machinery is real, measure it through \
         this harness and move the decision with it: a candidate whose second path runs has to carry \
         a measured comparison before it may stay enabled. If it is not, the record is out of date."
    );

    // (2) One cache, one module: the store is declared once and the files that may name it are the
    // module itself, the crate root that declares it and the budget that carries a handle.
    let declaring: Vec<&str> = sources
        .iter()
        .filter(|source| source.text.contains("struct FactsCache"))
        .map(|source| source.path.as_str())
        .collect();
    assert_eq!(
        declaring,
        ["crates/jarde-reader/src/facts_cache.rs"],
        "the engine declares a facts cache in {declaring:?}; P5 2.3 built exactly one, in the \
         reader's `facts_cache` module, and a second one would be a second key, a second fallback \
         and a second thing to invalidate"
    );
    let naming: Vec<&str> = sources
        .iter()
        .filter(|source| source.text.contains("facts_cache"))
        .map(|source| source.path.as_str())
        .collect();
    assert_eq!(
        naming,
        [
            "crates/jarde-reader/src/artifact.rs",
            "crates/jarde-reader/src/budget.rs",
            "crates/jarde-reader/src/classfile.rs",
            "crates/jarde-reader/src/lib.rs",
            "src/lib.rs",
        ],
        "the cache is named in {naming:?}, which is not the budget that carries a handle, the read \
         entries that consult it (the class-file facts and the directed container access), the \
         module declaration and the facade's re-export: a file outside that set has grown a second \
         path into the cache. (The module that declares it names no module path, which is why it is \
         not in this list: the declaration is checked above.)"
    );

    // (3) Nothing in the engine switches it on.
    let mut constructions = Vec::new();
    for source in &sources {
        for hit in machinery_hits(&source.text, &CACHE_CONSTRUCTIONS) {
            constructions.push(format!("{}:{hit}", source.path));
        }
    }
    assert!(
        constructions.is_empty(),
        "an engine source constructs a facts cache: {constructions:?}\nP5 2.3 keeps the cache \
         disabled by default: a caller may attach one to a budget, and the engine may not build one \
         for anybody. If a slice really wants one in a default path, that is a decision this harness \
         measures — a cache-on configuration has to be a row with a measured comparison beside it, \
         and `CANDIDATES` has to carry it."
    );

    // (4) And the constructor every existing entry point uses carries none: the behavioural half of
    //     "disabled by default", which no source scan can state.
    assert!(
        Budget::new(limits()).facts_cache().is_none(),
        "`Budget::new` now carries a facts cache, so every existing caller's path has one"
    );

    // The scan is not vacuous: it catches the shapes it looks for, in the spellings a real one would
    // use, and it leaves alone the line that made the `Index…` prefix rule impossible. That is what
    // makes the results above statements about the sources rather than about patterns that never
    // match.
    assert_eq!(
        machinery_hits("pub mod index;", &MACHINERY),
        vec!["1: mod index".to_string()]
    );
    assert_eq!(
        machinery_hits(
            "let workers = std::thread::scope(|scope| scope);",
            &MACHINERY
        ),
        vec!["1: thread::scope".to_string()]
    );
    assert_eq!(
        machinery_hits("pub struct IndexCall { bci: u32 }", &MACHINERY),
        Vec::<String>::new(),
        "the recovery layer's dispatch-table read is not an index, and the needles may not read it \
         as one"
    );
    assert_eq!(
        machinery_hits(
            "pub struct FactsCache { shared: Arc<Mutex<Shared>> }",
            &CACHE_CONSTRUCTIONS
        ),
        Vec::<String>::new(),
        "a declaration is not a construction, and the needle may not read it as one"
    );
    assert_eq!(
        machinery_hits(
            "let cache = FactsCache::current(CACHE_CAPACITY);",
            &CACHE_CONSTRUCTIONS
        ),
        vec!["1: FactsCache::current(".to_string()],
        "a construction site is exactly what this needle has to catch, in the spelling a caller would \
         use"
    );
    println!(
        "engine sources scanned: {} files under `crates/*/src` and `src/` — one facts cache declared \
         in `crates/jarde-reader/src/facts_cache.rs`, constructed nowhere in the engine, `Budget::new` \
         carrying none; no index module and no thread spawn",
        sources.len()
    );
}

/// Tasks 2.1 and 2.2, as a rule rather than a paragraph: everything the matrix can run has to be
/// measured, and a benefit claim has to be a comparison this harness could repeat.
///
/// Today the matrix has one configuration, so every candidate is unmeasured and stays out of the
/// default path (design decision 1). The rule is what makes that a decision instead of a habit: the
/// day a slice wires a second path into the matrix, this test fails until the comparison is recorded
/// beside it, and the day one is recorded without a second path it fails too.
#[test]
fn every_benefit_claim_needs_a_second_runnable_path() {
    let corpus = corpus();
    let configurations = runnable_configurations(&corpus);
    println!(
        "the matrix can be run in {} (cache, concurrency) configuration(s)",
        configurations.len()
    );
    for configuration in &configurations {
        println!("  cache {}", configuration.0);
        println!("  concurrency {}", configuration.1);
    }
    assert!(
        !configurations.is_empty(),
        "the matrix produced no row, so no candidate below could be measured against anything"
    );
    let reference = configurations
        .iter()
        .next()
        .expect("the set is not empty")
        .clone();
    let second_paths: Vec<&(String, String)> = configurations
        .iter()
        .filter(|configuration| **configuration != reference)
        .collect();

    // (1) Anything the matrix runs as a second configuration has to be measured. A runnable second
    // path with no comparison beside it is what design decision 1 keeps out of the default path; a
    // row that is the reference path wearing a second label is what the `performance-gates` spec
    // refuses as a measurement-shaped claim.
    for configuration in &second_paths {
        assert!(
            CANDIDATES.iter().any(|candidate| matches!(
                &candidate.benefit,
                Benefit::Measured(measured)
                    if measured.configuration.0 == configuration.0
                        && measured.configuration.1 == configuration.1
            )),
            "the matrix can be run as cache `{}` with concurrency `{}`, and no candidate records a \
             measured comparison taken in that configuration. Either it is a second code path — then \
             measure it against the direct path and record the comparison in `CANDIDATES` — or it is \
             the reference path wearing a second label, which P5's design (5) and the \
             `performance-gates` spec refuse.",
            configuration.0,
            configuration.1
        );
    }

    // (2) Every candidate is a complete decision, and any measured benefit is a measurement this
    // harness could repeat: `check_candidate` holds both halves, and the test below holds the rule.
    for candidate in &CANDIDATES {
        if let Err(reason) = check_candidate(candidate, &configurations) {
            panic!("{reason}");
        }
    }
    println!(
        "  {} candidates, {} of them carrying a measured benefit; the reference configuration is \
         the only one the matrix can run",
        CANDIDATES.len(),
        CANDIDATES
            .iter()
            .filter(|candidate| matches!(candidate.benefit, Benefit::Measured(_)))
            .count()
    );
}

/// The rule above is a rule and not a description: a claim the matrix cannot back comes back as a
/// refusal, and the refusal says which case it is.
///
/// Every case here is something someone would write while turning a candidate on. The first is the
/// one that matters most — a cache-on label over the same code path, with the same numbers, which is
/// a measurement of the reference path wearing a second name — and it is refused on the
/// configuration rather than on the intentions of whoever wrote it. The rule's last check — whether
/// the claim points at anything smaller — is asserted on the predicate itself, because every claim
/// that reaches it through `check_candidate` has already been refused for the configuration it
/// names: there is no second configuration to hand it today.
#[test]
fn the_benefit_rule_refuses_a_claim_the_matrix_cannot_back() {
    let corpus = corpus();
    let configurations = runnable_configurations(&corpus);
    let candidate = CANDIDATES
        .iter()
        .find(|candidate| candidate.id == "fine-grained-parallel")
        .expect("the candidate is in the record");

    // The record as it stands passes the rule: a refusal below is about the claim, not about a rule
    // that refuses everything it is shown.
    assert!(
        check_candidate(candidate, &configurations).is_ok(),
        "the candidate as recorded does not satisfy the rule it is recorded under"
    );

    let claim = |reference_label: &'static str,
                 candidate_label: &'static str,
                 configuration: (&'static str, &'static str),
                 deltas: &'static [(CountedBudgetDimension, i128)],
                 micros: u128| {
        Benefit::Measured(MeasuredBenefit {
            reference: reference_label,
            candidate: candidate_label,
            configuration,
            deltas,
            reference_median_micros: 130,
            candidate_median_micros: micros,
        })
    };

    /// A claim with a dimension that charges less, and one with a dimension that did not move.
    const REDUCED: &[(CountedBudgetDimension, i128)] = &[(CountedBudgetDimension::ReadBytes, -627)];
    const UNCHANGED: &[(CountedBudgetDimension, i128)] = &[(CountedBudgetDimension::ReadBytes, 0)];

    let cases: [(&str, Candidate, &str); 3] = [
        (
            "the reference path wearing a second label",
            with_benefit(
                candidate,
                claim(
                    "full-range-xref/minimal-jar",
                    "full-range-xref/minimal-jar (cache on)",
                    (REFERENCE_CACHE, REFERENCE_CONCURRENCY),
                    REDUCED,
                    100,
                ),
            ),
            "reference configuration",
        ),
        (
            "a claim in a configuration the matrix cannot run",
            with_benefit(
                candidate,
                claim(
                    "full-range-xref/minimal-jar",
                    "full-range-xref/minimal-jar (parallel)",
                    ("present: a facts cache", "4: four workers"),
                    REDUCED,
                    100,
                ),
            ),
            "cannot be run that way",
        ),
        (
            "one run compared with itself",
            with_benefit(
                candidate,
                claim(
                    "full-range-xref/minimal-jar",
                    "full-range-xref/minimal-jar",
                    ("present: a facts cache", "4: four workers"),
                    REDUCED,
                    100,
                ),
            ),
            "measured over one run",
        ),
    ];
    for (name, claimed, expected) in cases {
        let refusal = match check_candidate(&claimed, &configurations) {
            Err(refusal) => refusal,
            Ok(()) => panic!("the rule accepted {name}, which the matrix cannot back"),
        };
        assert!(
            refusal.contains(expected),
            "the refusal of {name} does not say which case it is (expected `{expected}` in it): \
             {refusal}"
        );
        println!("  refused {name}: {refusal}");
    }

    // The last check inside the rule — does the claim point at anything smaller — is asserted on the
    // predicate, because every claim that reaches it through `check_candidate` has already been
    // refused for the configuration it names: there is no second configuration to hand it today.
    let unchanged = MeasuredBenefit {
        reference: "full-range-xref/minimal-jar",
        candidate: "full-range-xref/minimal-jar (cache on)",
        configuration: ("present: a facts cache", "1: sequential"),
        deltas: UNCHANGED,
        reference_median_micros: 130,
        candidate_median_micros: 130,
    };
    assert!(
        !unchanged.points_at_a_reduction(),
        "a claim whose counted dimensions all charge the same and whose median did not move is not a \
         benefit"
    );
    let smaller = MeasuredBenefit {
        candidate_median_micros: 129,
        ..unchanged
    };
    assert!(
        smaller.points_at_a_reduction(),
        "a median that moved by a microsecond is the smallest thing this rule can see; asserting \
         only the refusal above would leave the check unable to say yes to anything"
    );
}

/// The built layer against the recorded contract: every key dimension the record binds to the
/// CP/Header layer is carried by the cache task 2.3 built, and no dimension the record binds to
/// another layer is smuggled into it.
///
/// This is where 2.2's record and 2.3's code are held together. The record says which dimensions a
/// layer's entries must include; the built identity is the answer for one layer, and it is checked
/// rather than described: the record's CP/Header list is read out of `KEY_DIMENSIONS`, the identity
/// is asked for its own fields, and the one dimension that is a *rule* rather than a value — the
/// budget, which the record binds to all four layers — is checked by running the case it forbids: a
/// parse a budget stops stores nothing.
#[test]
fn the_built_layer_carries_the_dimensions_the_record_binds_to_it() {
    let cache = facts_cache();
    let identity = cache.identity();
    assert_eq!(
        identity.registry, HIGHEST_REGISTERED_MAJOR,
        "the cache's parser version is not the release registry's, which is the carrier the record \
         names for the `registry` dimension"
    );
    assert_eq!(
        identity.format, FACTS_FORMAT,
        "the entry format the cache reads under is not the one this build writes"
    );

    // The record's own list for this layer, read out of the record.
    let bound: Vec<&str> = KEY_DIMENSIONS
        .iter()
        .filter(|dimension| {
            dimension
                .layers
                .split('+')
                .any(|layer| layer == "cp-header")
        })
        .map(|dimension| dimension.name)
        .collect();
    assert_eq!(
        bound,
        ["snapshot", "registry", "budget"],
        "the record binds {bound:?} to the CP/Header layer, and this layer was built against that \
         list: a dimension added to it has to reach the key, and one removed has to leave it"
    );
    // How this layer carries each of them, stated once. `snapshot` is the content identity: the key
    // is the class bytes' digest and length, so two origins of equal bytes share one entry and no
    // origin is in the key. `registry` and the entry format are the two fields of the identity.
    let carried: Vec<(&str, String)> = vec![
        (
            "snapshot",
            format!(
                "the class content digest and length in the entry key ({} declared field(s) hold \
                 no origin)",
                serde_json::to_value(identity)
                    .expect("an identity serializes")
                    .as_object()
                    .expect("an identity is an object")
                    .len()
            ),
        ),
        (
            "registry",
            format!("identity.registry = {}", identity.registry),
        ),
        (
            "budget",
            "an entry is written only by a parse that ran to the end".to_string(),
        ),
    ];
    assert_eq!(
        carried.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
        bound,
        "the record's list and the carriers this test states are not the same dimensions"
    );
    for (name, how) in &carried {
        assert!(
            !how.trim().is_empty(),
            "{name} has no carrier in this layer"
        );
    }

    // The budget dimension is the one that is a rule, so it is checked by its case: a parse the
    // budget stops must leave the store empty — otherwise an incomplete answer would stand in for a
    // complete one the moment a later request could afford it.
    let bytes = verified(&corpus(), &SUBJECTS[0]).content;
    let mut tight = limits();
    tight.class_bytes = 1;
    let store = facts_cache();
    let mut budget = Budget::new(tight).with_facts_cache(store.clone());
    assert!(
        class_facts(&bytes, &mut budget).is_err(),
        "one class byte cannot pay for the whole class, so this check has nothing to refuse"
    );
    assert_eq!(
        store.report().entries,
        0,
        "a parse the budget stopped was stored: the `budget` dimension is not carried, and an \
         incomplete entry would stand in for a complete one: {:?}",
        store.report()
    );

    // And the layers above are still unbuilt: every dimension the record binds to one of them and
    // *not* to this one is asserted absent from the identity, because a cache that took one of them
    // would miss for a reason that cannot change its answer.
    let others = ["view", "platform", "query", "IR", "recovery", "pass"];
    for name in others {
        assert!(
            !bound.contains(&name),
            "{name} is bound to another layer and this layer's identity took it in"
        );
    }
    println!(
        "the built layer carries {bound:?}: identity registry {} format {}, the content in the key, \
         and completeness on the way in; {:?} stay with the layers above",
        identity.registry, identity.format, others
    );
}

/// The key dimensions of task 2.2: the ones `design.md` decision 2 names, once each, each bound to
/// the layers that have to include it and to the token that carries it in the code.
///
/// The shape chosen — recorded, not built — is explained on `KEY_DIMENSIONS`. What this test holds
/// is the half a sentence cannot: the dimension list is complete against the decision, every layer
/// name is one of the four `facts-cache` binds keys by, and every `Present` carrier really exists in
/// the engine sources, so the record cannot rot into prose that agrees with itself.
#[test]
fn the_recorded_key_dimensions_are_the_ones_the_decision_names() {
    const LAYERS: [&str; 4] = ["cp-header", "x1", "resolution", "ir-source"];
    const NAMED_BY_THE_DECISION: [&str; 9] = [
        "snapshot", "view", "platform", "registry", "query", "IR", "recovery", "pass", "budget",
    ];
    for name in NAMED_BY_THE_DECISION {
        let recorded = KEY_DIMENSIONS
            .iter()
            .filter(|dimension| dimension.name == name)
            .count();
        assert_eq!(
            recorded,
            1,
            "`{name}` is one of the dimensions design decision 2 names, and this record has \
             {recorded} entries for it. A dimension missing from the record is a key that can miss, \
             and a cache that can answer with another configuration's result; a dimension recorded \
             twice is two keys for one input. Recorded: {:?}",
            KEY_DIMENSIONS
                .iter()
                .map(|dimension| dimension.name)
                .collect::<Vec<_>>()
        );
    }
    let sources = engine_sources();
    for dimension in &KEY_DIMENSIONS {
        assert!(
            !dimension.source.trim().is_empty(),
            "{} does not say which requirement it comes from",
            dimension.name
        );
        for layer in dimension.layers.split('+') {
            assert!(
                LAYERS.contains(&layer),
                "{} is bound to layer `{layer}`, which is not one of the four layers `facts-cache` \
                 binds keys by ({LAYERS:?})",
                dimension.name
            );
        }
        match &dimension.carrier {
            Carrier::Present { what, token } => {
                assert!(
                    !what.trim().is_empty(),
                    "{} has no description of what carries it",
                    dimension.name
                );
                assert!(
                    sources.iter().any(|source| source.text.contains(token)),
                    "{} records `{token}` as the token that carries it, and no engine source \
                     contains that token. This record is anchored to code: a renamed or deleted \
                     carrier is a failure here, not a sentence that quietly goes stale.",
                    dimension.name
                );
            }
            Carrier::Absent { why, closed_by } => assert!(
                !why.trim().is_empty() && !closed_by.trim().is_empty(),
                "{} is recorded as having no carrier and does not say why, or does not name the \
                 slice that closes it: an absence is a decision only while it is explained",
                dimension.name
            ),
        }
    }
    let absent = KEY_DIMENSIONS
        .iter()
        .filter(|dimension| matches!(dimension.carrier, Carrier::Absent { .. }))
        .count();
    println!(
        "{} key dimensions recorded against design decision 2, {absent} of them with no carrier in \
         the code yet",
        KEY_DIMENSIONS.len()
    );
    for dimension in &KEY_DIMENSIONS {
        let carrier = match &dimension.carrier {
            Carrier::Present { token, .. } => format!("present under `{token}`"),
            Carrier::Absent { .. } => "no carrier yet".to_string(),
        };
        println!(
            "  {:<20} {:<38} {carrier}",
            dimension.name, dimension.layers
        );
    }
}

/// A14's cancellation half, in the shape this engine can take it today.
///
/// The spec's shared-request scenario — one subscriber cancels while another keeps waiting on the
/// same single-flight request — has no object here: nothing is shared between requests, so there is
/// no single-flight to subscribe to and no rule to test about the subscriber who keeps waiting.
/// What can be tested is the property that rule protects, in the only shape the engine has: a
/// cancelled request must not reach a later request over the same bytes. A scheduler that cancelled
/// "the work for these bytes" instead of "this subscriber's request" would break exactly here, and
/// so would a shared entry that remembered a truncated scan.
#[test]
fn a_cancelled_request_does_not_reach_a_later_one_over_the_same_bytes() {
    let corpus = corpus();
    let jar = verified(&corpus, &SUBJECTS[0]);
    let baseline = run_full_range(&corpus, &jar, "full-range-xref/minimal-jar", false);
    let cancelled = run_full_range(&corpus, &jar, "cancelled/full-range-xref/minimal-jar", true);
    assert_eq!(
        cancelled.status, "cancelled",
        "the middle run of this test has to be the cancelled one and reports `{}`",
        cancelled.status
    );
    let after = run_full_range(&corpus, &jar, "full-range-xref/minimal-jar", false);
    assert_eq!(
        after.status, "complete",
        "a request over the same bytes, run after a cancelled one, reports `{}`: cancellation is per \
         request and nothing may carry it into the next one",
        after.status
    );
    assert_eq!(
        after.fingerprint,
        baseline.fingerprint,
        "the request after the cancelled one published a different result:\n{}",
        compare(&baseline, &after).lines().join("\n")
    );
    assert_eq!(
        after.order, baseline.order,
        "the request after the cancelled one published its items in a different order"
    );
    println!(
        "{} reports `{}` and the next request over the same bytes republished the baseline \
         fingerprint {}",
        cancelled.label,
        cancelled.status,
        short(after.fingerprint.trim_start_matches("blake3:"))
    );
}

/// A01 in the shape the measurement side needs: the reference path publishes facts, not candidates.
///
/// `p1_xref_code.rs::unused_constant_pool_entries_are_candidates_but_never_calls` is A01's
/// acceptance — a pool entry nobody consumes produces no call, no BCI and no consumer. The risk a
/// cache or an index adds is one step over: serving the candidate itself as the answer because the
/// index said so (`facts-cache`, `Index candidate requires verification`). What `compare` holds a
/// future path to is the reference path's own shape, so what is pinned here is that shape on the
/// rows the fingerprint fixes: a `mentions_symbol` scan publishes no `constant_pool_candidate` item
/// and no item without a consumer category.
#[test]
fn the_reference_path_publishes_no_pool_candidate_as_an_item() {
    let corpus = corpus();
    let rows = published_rows(&corpus);
    for row in &rows[..2] {
        assert!(
            row.published_items > 0,
            "{} published nothing, so a claim about what it publishes would be vacuous",
            row.label
        );
        assert_eq!(
            row.derivations
                .get("constant_pool_candidate")
                .copied()
                .unwrap_or(0),
            0,
            "{} published a pool candidate as an item; derivations {:?}",
            row.label,
            row.derivations
        );
        assert_eq!(
            row.items_without_a_consumer, 0,
            "{} published {} item(s) with no consumer category: a pool hit is a candidate, and a \
             reference exists where a consumer verified it (A01)",
            row.label, row.items_without_a_consumer
        );
        println!("{} derivations {:?}", row.label, row.derivations);
    }
    // The local row publishes a payload instead of XRef items, so its empty derivation map is "not
    // this kind of row" and not "no candidates found".
    assert!(rows[2].derivations.is_empty() && rows[2].published_items == 0);
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
        "P5 baseline: the direct path and the cache-on path (P5 2.3), no parallel scheduler, one \
         machine, RUST_TEST_THREADS=1, single-threaded build, {REPEATS} repeats per row"
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
        // The cache-on row task 2.3 added: one shared handle and one priming run, so every repeat
        // measures the **warm** path. The priming run is not part of the sample.
        {
            let cache = facts_cache();
            let warm_up = run_full_range_with(
                &corpus,
                &jar,
                "cache-on warm-up/full-range-xref/minimal-jar",
                false,
                Some(&cache),
            );
            println!(
                "cache-on warm-up (not part of the sample): {}",
                charges_line(&warm_up.usage, &CountedBudgetDimension::ALL)
            );
            repeats(
                "cache-on-warm/full-range-xref/minimal-jar",
                REPEATS,
                move || {
                    run_full_range_with(
                        &corpus,
                        &jar,
                        "cache-on-warm/full-range-xref/minimal-jar",
                        false,
                        Some(&cache),
                    )
                },
            )
        },
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
        // The reference row against the cache-on row over the same bytes: the measurement
        // `CANDIDATES` records a benefit from.
        compare(&rows[0].repeats[0], &rows[4].repeats[0]),
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

// -------------------------------------------------------------------------------------------
// 3.1: the differential, as the gate `Correctness and resource regression gates` states it
// -------------------------------------------------------------------------------------------

/// The planes the requirement names, in its own order.
///
/// A list rather than prose, so a later slice cannot quietly drop one: every comparison reports a
/// verdict for every entry, and a name this file does not carry is a `panic!` rather than a skipped
/// plane.
const GATE_PLANES: [&str; 8] = [
    "evidence",
    "coverage",
    "representation",
    "diagnostics",
    "snapshot/view identity",
    "peak memory proxy",
    "cancellation",
    "budget results",
];

/// What one comparison has on one plane.
enum PlaneComparison {
    /// Both paths publish a value and they were compared field by field; `equal` is the verdict.
    Compared(bool),
    /// Only one path publishes a value today, so there is nothing to compare on this plane.
    NoObject(&'static str),
}

impl PlaneComparison {
    fn equal(&self) -> bool {
        matches!(self, Self::Compared(true))
    }

    fn describe(&self) -> String {
        match self {
            Self::Compared(true) => "equal".to_string(),
            Self::Compared(false) => "DIFFERENT".to_string(),
            Self::NoObject(why) => format!("no comparison object ({why})"),
        }
    }
}

/// One plane of one comparison, read off its verdicts.
///
/// Every plane here has two values today, which is not a claim that every *scenario* does: see
/// [`NO_SECOND_PATH_TODAY`], which names the comparisons the requirement asks for and this
/// repository cannot make at all.
fn plane_comparison(plane: &str, comparison: &Comparison) -> PlaneComparison {
    let verdicts = &comparison.verdicts;
    match plane {
        // Evidence is the whole published document with the wall clock removed: the items, the
        // origins they name, how each was derived and the reads behind them.
        "evidence" => PlaneComparison::Compared(verdicts.evidence),
        "coverage" => PlaneComparison::Compared(verdicts.coverage),
        "representation" if comparison.representation => {
            PlaneComparison::Compared(verdicts.evidence)
        }
        "representation" => PlaneComparison::NoObject(
            "neither row publishes a representation: the query layer publishes items and no source, \
             so this plane is carried by the recovery rows only",
        ),
        "diagnostics" => PlaneComparison::Compared(verdicts.diagnostics),
        "snapshot/view identity" => PlaneComparison::Compared(verdicts.identity),
        // The only peaks a usage snapshot keeps are the two high-water depths; every other counted
        // dimension is a cumulative total and is reported as a delta, because a cache is *supposed*
        // to move it.
        "peak memory proxy" => PlaneComparison::Compared(verdicts.peaks),
        "cancellation" => PlaneComparison::Compared(verdicts.status && verdicts.termination),
        "budget results" => PlaneComparison::Compared(verdicts.coverage && verdicts.termination),
        other => panic!(
            "the requirement names a plane this comparison does not carry: {other}. A plane nobody \
             compares is a plane a gate passes by omission."
        ),
    }
}

/// The comparisons the two scenarios name that have **no second value** in this repository today.
///
/// Listed rather than implied: each is something the requirement asks to be compared, and a reader of
/// the gate's output has to see the absence instead of reading a table of green rows as "everything
/// was compared". None of these is a plane this file skipped — they are paths and instruments that do
/// not exist yet.
const NO_SECOND_PATH_TODAY: [(&str, &str); 5] = [
    (
        "the index path",
        "no index exists — `the_engine_has_one_disabled_facts_cache_and_no_index_or_scheduler` holds \
         that by source scan — so `Optimized versus direct path` has one path to run for it",
    ),
    (
        "the parallel path",
        "no scheduler and no worker exist, so `Reordered parallel results` has no worker order to \
         hold against the stable publication order",
    ),
    (
        "the merged query path (single-flight)",
        "one request at a time: no two requests share work, so `Cancelled shared query` has no \
         subscriber who keeps waiting and no mixed result to refuse",
    ),
    (
        "the modern (P4) facts plane",
        "every row of this matrix runs the reader, the query layer or the recovery layer; no row asks \
         for `ModernFacts`, so the cache has no modern answer that could be compared",
    ),
    (
        "true peak RSS",
        "the harness has no allocator instrumentation: the memory plane is the budget's charge \
         counters plus the two high-water depths, which support 'this path derived more units' and \
         not 'this path used more bytes'",
    ),
];

/// What one row's own entry published, in the single shape [`RunRecord`] takes.
///
/// The three entries of this harness publish three different documents; recording them through one
/// structure is what lets `compare` and the plane table treat them as rows of one matrix.
struct Outcome {
    status: &'static str,
    report_usage: UsageSnapshot,
    coverage: Value,
    diagnostics: Value,
    order: String,
    published_items: u64,
    derivations: BTreeMap<String, usize>,
    items_without_a_consumer: usize,
    published: Value,
    document: Value,
    fingerprint: String,
    evidence: Value,
    evidence_fingerprint: String,
}

fn outcome_query(report: &QueryReport) -> Outcome {
    let form = published(report);
    Outcome {
        status: status_of(&report.execution),
        report_usage: usage_of(&report.execution),
        coverage: serde_json::to_value(&report.coverage).expect("coverage serializes"),
        diagnostics: serde_json::to_value(&report.diagnostics).expect("diagnostics serialize"),
        order: item_identities(&report.items),
        published_items: report.items.len() as u64,
        derivations: item_derivations(&report.items),
        items_without_a_consumer: report
            .items
            .iter()
            .filter(|item| item.consumer.is_none())
            .count(),
        published: form.raw,
        document: form.normalized,
        fingerprint: form.fingerprint,
        evidence: form.evidence,
        evidence_fingerprint: form.evidence_fingerprint,
    }
}

fn outcome_recovery(recovered: &RecoveredMethod) -> Outcome {
    let analysis = recovered.analysis();
    let form = published(recovered);
    Outcome {
        status: status_of(&analysis.execution),
        report_usage: usage_of(&analysis.execution),
        coverage: serde_json::to_value(&analysis.coverage).expect("coverage serializes"),
        diagnostics: serde_json::to_value(&analysis.diagnostics).expect("diagnostics serialize"),
        // A recovery row publishes one payload, not an ordered list of items; its evidence is the
        // document the fingerprint is taken over.
        order: "[]".to_string(),
        published_items: 0,
        derivations: BTreeMap::new(),
        items_without_a_consumer: 0,
        published: form.raw,
        document: form.normalized,
        fingerprint: form.fingerprint,
        evidence: form.evidence,
        evidence_fingerprint: form.evidence_fingerprint,
    }
}

fn outcome_resolution(report: &ResolutionReport) -> Outcome {
    let form = published(report);
    Outcome {
        status: status_of(&report.execution),
        report_usage: usage_of(&report.execution),
        coverage: serde_json::to_value(&report.coverage).expect("coverage serializes"),
        diagnostics: serde_json::to_value(&report.diagnostics).expect("diagnostics serialize"),
        // The ordered half of a resolution answer is its candidate list; the decision itself
        // (`state`, `resolved`) is inside the fingerprint.
        order: serde_json::to_string(&report.candidates).expect("candidates render"),
        published_items: report.candidates.len() as u64,
        derivations: BTreeMap::new(),
        items_without_a_consumer: 0,
        published: form.raw,
        document: form.normalized,
        fingerprint: form.fingerprint,
        evidence: form.evidence,
        evidence_fingerprint: form.evidence_fingerprint,
    }
}

// -------------------------------------------------------------------------------------------
// 3.2: the boundary corpora, run through the same differential
// -------------------------------------------------------------------------------------------

/// The compression methods the fixture writer can state, as the ZIP method codes.
const STORE: u16 = 0;
const DEFLATE: u16 = 8;

/// One corpus of the boundary matrix and what is asked of it.
struct Boundary {
    /// The label every row over this corpus carries, and the subject the context records.
    label: &'static str,
    bytes: Vec<u8>,
    /// The content sources the request names, as the raw entry name and its bytes.
    providers: Vec<(&'static [u8], Vec<u8>)>,
    ask: Ask,
    /// The budget the request starts from. A row that has to run out of something states the reduced
    /// dimension here rather than calling the whole budget "small".
    limits: Limits,
    /// Whether the token is cancelled after the artifact opens, with this corpus's own work — a
    /// member to inflate, a body to analyse — still ahead of it.
    cancel: bool,
    /// The terminal status the row has to publish. A stopped row may never read as `complete`, and
    /// which of `partial`/`failed`/`cancelled` a corpus produces is a fact about the entry that
    /// answers it rather than a preference: the reader's over-limit read is an error, so a scan that
    /// cannot read its candidate fails with the dimension named.
    exit: &'static str,
    /// The spelling the termination reason has to state — the budget dimension the work ran out of,
    /// or empty when the status is the whole reason (a cancellation).
    report: &'static str,
    /// The spellings the published document has to state, for the rows whose corpus is about a shape
    /// (a refusal code, a resolution state, a representation). Empty for the rows whose point is
    /// their budget alone — and every row that claims a shape states it here, so a corpus that
    /// quietly stopped being that shape fails instead of comparing two empty results.
    states: &'static [&'static str],
}

/// What one boundary row asks of its corpus.
enum Ask {
    /// The full-range structural scan the matrix rows run.
    FullRange,
    /// One member of the corpus, presented by the recovery layer.
    Member {
        name: &'static [u8],
        descriptor: &'static [u8],
    },
    /// One method symbol, resolved in an environment rooted at this corpus.
    Resolve {
        owner: &'static [u8],
        name: &'static [u8],
        descriptor: &'static [u8],
    },
}

/// The class of the P3 handler corpus, and the class it names as a resource.
const GUARDED: &[u8] = include_bytes!("fixtures/p3-handlers/v8/Guarded.class");
const GUARDED_RESOURCE: &[u8] = include_bytes!("fixtures/p3-handlers/v8/Res.class");
/// The committed container whose member is DEFLATED: the corpus a cancellation arrives over.
const NESTED: &[u8] = include_bytes!("../fuzz/corpus/artifact_tree/nested.jar");

/// A class file with one bootstrap table and one body, built here rather than taken from a fixture.
///
/// The constant-pool writer is deliberately the smallest one that can state the A05 shape: a
/// `CONSTANT_Dynamic` whose bootstrap argument is another `CONSTANT_Dynamic`, and a bootstrap entry
/// two of them reach. A cache that stored or replayed a partial condy graph would publish different
/// facts over these bytes; the graph's own semantics (shared subgraphs, cycles, the derived budgets)
/// are P4's tests (`p4_modern_facts.rs`), and this row is the differential over such a class.
#[derive(Default)]
struct Pool {
    body: Vec<u8>,
    count: u16,
}

impl Pool {
    fn utf8(&mut self, text: &[u8]) -> u16 {
        let length = u16::try_from(text.len()).expect("a name fits u16");
        self.entry(1, |body| {
            body.extend_from_slice(&length.to_be_bytes());
            body.extend_from_slice(text);
        })
    }

    fn class(&mut self, name: u16) -> u16 {
        self.pair(7, name, 0)
    }

    fn string(&mut self, text: u16) -> u16 {
        self.pair(8, text, 0)
    }

    fn name_and_type(&mut self, name: u16, descriptor: u16) -> u16 {
        self.pair(12, name, descriptor)
    }

    fn method_ref(&mut self, class: u16, name_and_type: u16) -> u16 {
        self.pair(10, class, name_and_type)
    }

    fn method_handle(&mut self, kind: u8, reference: u16) -> u16 {
        self.entry(15, |body| {
            body.push(kind);
            body.extend_from_slice(&reference.to_be_bytes());
        })
    }

    fn dynamic(&mut self, bootstrap: u16, name_and_type: u16) -> u16 {
        self.pair(17, bootstrap, name_and_type)
    }

    fn entry(&mut self, tag: u8, body: impl FnOnce(&mut Vec<u8>)) -> u16 {
        self.body.push(tag);
        body(&mut self.body);
        self.count += 1;
        self.count
    }

    /// A pool entry of `tag` whose body is one or two `u16` fields (`Class`, `String`,
    /// `NameAndType`, `Dynamic`, the three refs); `second` is `0` for the single-field tags, whose
    /// trailing zero is written and then ignored by the reader.
    fn pair(&mut self, tag: u8, first: u16, second: u16) -> u16 {
        self.body.push(tag);
        self.body.extend_from_slice(&first.to_be_bytes());
        if SECOND_FIELD_TAGS.contains(&tag) {
            self.body.extend_from_slice(&second.to_be_bytes());
        }
        self.count += 1;
        self.count
    }

    /// The `constant_pool_count` this pool declares: one past its last index.
    fn declared(&self) -> u16 {
        self.count + 1
    }
}

/// The pool tags whose body is two `u16` fields.
const SECOND_FIELD_TAGS: [u8; 5] = [12, 10, 11, 17, 18];

fn class_bytes_with(
    pool: &Pool,
    major: u16,
    this_class: u16,
    super_class: u16,
    methods: &[Member],
    attributes: &[(u16, Vec<u8>)],
) -> Vec<u8> {
    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    bytes.extend_from_slice(&0_u16.to_be_bytes());
    bytes.extend_from_slice(&major.to_be_bytes());
    bytes.extend_from_slice(&pool.declared().to_be_bytes());
    bytes.extend_from_slice(&pool.body);
    bytes.extend_from_slice(&0x0021_u16.to_be_bytes());
    bytes.extend_from_slice(&this_class.to_be_bytes());
    bytes.extend_from_slice(&super_class.to_be_bytes());
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // interfaces
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // fields
    bytes.extend_from_slice(
        &u16::try_from(methods.len())
            .expect("members fit u16")
            .to_be_bytes(),
    );
    for member in methods {
        bytes.extend_from_slice(&member.access.to_be_bytes());
        bytes.extend_from_slice(&member.name.to_be_bytes());
        bytes.extend_from_slice(&member.descriptor.to_be_bytes());
        bytes.extend_from_slice(
            &u16::try_from(member.attributes.len())
                .expect("attributes fit u16")
                .to_be_bytes(),
        );
        for (name, content) in &member.attributes {
            bytes.extend_from_slice(&name.to_be_bytes());
            bytes.extend_from_slice(
                &u32::try_from(content.len())
                    .expect("an attribute fits u32")
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(content);
        }
    }
    bytes.extend_from_slice(
        &u16::try_from(attributes.len())
            .expect("attributes fit u16")
            .to_be_bytes(),
    );
    for (name, content) in attributes {
        bytes.extend_from_slice(&name.to_be_bytes());
        bytes.extend_from_slice(
            &u32::try_from(content.len())
                .expect("an attribute fits u32")
                .to_be_bytes(),
        );
        bytes.extend_from_slice(content);
    }
    bytes
}

struct Member {
    access: u16,
    name: u16,
    descriptor: u16,
    attributes: Vec<(u16, Vec<u8>)>,
}

/// A `Code` attribute over `instructions`, with the stack and local shapes the instructions need.
fn code_body(instructions: &[u8], max_stack: u16, max_locals: u16) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(&max_stack.to_be_bytes());
    body.extend_from_slice(&max_locals.to_be_bytes());
    body.extend_from_slice(
        &u32::try_from(instructions.len())
            .expect("a body fits u32")
            .to_be_bytes(),
    );
    body.extend_from_slice(instructions);
    body.extend_from_slice(&0_u16.to_be_bytes()); // exception table
    body.extend_from_slice(&0_u16.to_be_bytes()); // attributes
    body
}

/// A `BootstrapMethods` attribute over `(handle, arguments)` entries.
fn bootstrap_body(entries: &[(u16, Vec<u16>)]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(
        &u16::try_from(entries.len())
            .expect("entries fit u16")
            .to_be_bytes(),
    );
    for (handle, arguments) in entries {
        body.extend_from_slice(&handle.to_be_bytes());
        body.extend_from_slice(
            &u16::try_from(arguments.len())
                .expect("arguments fit u16")
                .to_be_bytes(),
        );
        for argument in arguments {
            body.extend_from_slice(&argument.to_be_bytes());
        }
    }
    body
}

/// The `CONSTANT_Dynamic` corpus: a chain into a bootstrap entry two use sites share.
fn condy_bytes() -> Vec<u8> {
    let mut pool = Pool::default();
    let name = pool.utf8(b"p/Condy");
    let this_class = pool.class(name);
    let object = pool.utf8(b"java/lang/Object");
    let super_class = pool.class(object);
    let use_name = pool.utf8(b"use");
    let use_descriptor = pool.utf8(b"()V");
    let code_name = pool.utf8(b"Code");
    let bootstrap_name = pool.utf8(b"BootstrapMethods");
    let descriptor = pool.utf8(b"Ljava/lang/Object;");
    let first_name = pool.utf8(b"condy1");
    let second_name = pool.utf8(b"condy2");
    let third_name = pool.utf8(b"condy3");
    let first_nat = pool.name_and_type(first_name, descriptor);
    let second_nat = pool.name_and_type(second_name, descriptor);
    let third_nat = pool.name_and_type(third_name, descriptor);
    let factory = pool.utf8(b"p/NoSuchFactory");
    let factory_class = pool.class(factory);
    let boom = pool.utf8(b"boom");
    let boom_descriptor = pool.utf8(
        b"(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/CallSite;",
    );
    let boom_nat = pool.name_and_type(boom, boom_descriptor);
    let boom_ref = pool.method_ref(factory_class, boom_nat);
    let handle = pool.method_handle(6, boom_ref);
    let leaf_text = pool.utf8(b"leaf");
    let leaf = pool.string(leaf_text);
    // Entry 0 reaches the third dynamic, entry 1 is the leaf two of them share: the node `leaf` is
    // reached from two entry points, which is A05's shape.
    let first = pool.dynamic(0, first_nat);
    // The second dynamic is an entry point of its own that reaches the shared bootstrap entry
    // directly; nothing has to name it for the reader to expand it, which is what makes the shared
    // node reached "from two entry points".
    pool.dynamic(1, second_nat);
    let third = pool.dynamic(1, third_nat);
    let bootstrap = bootstrap_body(&[(handle, vec![third]), (handle, vec![leaf])]);
    // `ldc_w condy1; pop; return`: one entry point is really used by a body.
    let mut instructions = vec![0x13];
    instructions.extend_from_slice(&first.to_be_bytes());
    instructions.push(0x57);
    instructions.push(0xb1);
    class_bytes_with(
        &pool,
        55,
        this_class,
        super_class,
        &[Member {
            access: 0x0009,
            name: use_name,
            descriptor: use_descriptor,
            attributes: vec![(code_name, code_body(&instructions, 1, 0))],
        }],
        &[(bootstrap_name, bootstrap)],
    )
}

/// The P0 bomb shape, built here: a ZIP whose central directory and local header both declare an
/// uncompressed size of **one**, while the DEFLATE stream really expands to `BOMB_BYTES`.
///
/// The shape is the one the reader's own unit test asserts
/// (`crates/jarde-reader/src/artifact.rs::named_budgets_cover_declared_actual_read_and_entry_count`):
/// the declared size cannot be trusted to size the read, so the stream is watched as it inflates and
/// the request stops at its `entry_bytes` bound **without returning bytes**. A good class is stored
/// beside it, so the same request also parses a class through the cache.
fn bomb_archive() -> Vec<u8> {
    const BOMB_NAME: &[u8] = b"p/Boom.class";
    const BOMB_BYTES: usize = 8 * 1024;
    let mut filler = 0xcafebabe_u32.to_be_bytes().to_vec();
    filler.extend_from_slice(&0_u16.to_be_bytes());
    filler.extend_from_slice(&55_u16.to_be_bytes());
    filler.resize(BOMB_BYTES, 0x40);
    let mut archive = zip_of(&[
        (b"p/Ok.class".to_vec(), minimal_class(b"p/Ok"), STORE),
        (BOMB_NAME.to_vec(), filler, DEFLATE),
    ]);
    // The declaration the reader believes is the central entry's; the local header is left alone,
    // because the reader validates it against the central record (`validate_headers`) and a mismatch
    // there would be caught as a ZIP-structure error instead of being inflated. What is rewritten is
    // the central entry's two sizes **and** the data descriptor's, so the file stays internally
    // consistent while both declare a size the stream does not have — the same pair of edits
    // `named_budgets_cover_declared_actual_read_and_entry_count` makes.
    let central = entry_offset(&archive, b"PK\x01\x02", BOMB_NAME, 46);
    let directory = archive
        .windows(4)
        .position(|window| window == b"PK\x01\x02")
        .expect("the fixture archive has a central directory");
    let descriptor = directory
        .checked_sub(16)
        .expect("the bomb entry has a data descriptor");
    assert_eq!(
        &archive[descriptor..descriptor + 4],
        b"PK\x07\x08",
        "the 16 bytes before the central directory are the bomb entry's data descriptor; this \
         fixture's writer has changed and the sizes below would be rewritten in the wrong place"
    );
    // The **uncompressed** size, in the two records that state it: the central entry and the data
    // descriptor, which agree with each other and with neither the stream nor the local header (the
    // local header is only compared when the entry declares no descriptor, and this one does).
    //
    // The *compressed* size is deliberately left alone: it is what the reader uses to locate the
    // data and its descriptor, so a lie there is a broken archive and not an entry whose declared
    // size understates it. This is exactly the pair of edits
    // `named_budgets_cover_declared_actual_read_and_entry_count` makes, and it is why the reader's
    // bound is on the stream rather than on the declaration.
    for position in [descriptor + 12, central + 24] {
        archive[position..position + 4].copy_from_slice(&1_u32.to_be_bytes());
    }
    archive
}

/// The offset of the last `signature` in `archive` that is followed by `name` `fixed` bytes later.
fn entry_offset(archive: &[u8], signature: &[u8], name: &[u8], fixed: usize) -> usize {
    let window = fixed + name.len();
    archive
        .windows(window)
        .rposition(|candidate| {
            candidate.starts_with(signature) && &candidate[fixed..fixed + name.len()] == name
        })
        .unwrap_or_else(|| {
            panic!(
                "the fixture writer did not put `{}` after a {} signature with a {fixed}-byte fixed \
                 part, so the bomb's declared sizes cannot be rewritten",
                String::from_utf8_lossy(name),
                String::from_utf8_lossy(signature)
            )
        })
}

/// One stored ZIP with the given entries, deflated when `method` says so.
fn zip_of(entries: &[(Vec<u8>, Vec<u8>, u16)]) -> Vec<u8> {
    let mut output = std::io::Cursor::new(Vec::new());
    {
        let mut archive = rawzip::ZipArchiveWriter::new(&mut output);
        for (name, data, method) in entries {
            let (mut entry, config) = archive
                .new_file(rawzip::path::EntryPath::verbatim(name.clone()))
                .compression_method(rawzip::CompressionMethod::new(*method))
                .start()
                .expect("the fixture entry starts");
            if *method == DEFLATE {
                let encoder =
                    flate2::write::DeflateEncoder::new(&mut entry, flate2::Compression::default());
                let mut writer = config.wrap(encoder);
                std::io::Write::write_all(&mut writer, data).expect("the entry is writable");
                let (encoder, descriptor) = writer.finish().expect("the entry closes");
                encoder.finish().expect("the deflate stream closes");
                entry.finish(descriptor).expect("the entry finishes");
            } else {
                let mut writer = config.wrap(&mut entry);
                std::io::Write::write_all(&mut writer, data).expect("the entry is writable");
                let (_, descriptor) = writer.finish().expect("the entry closes");
                entry.finish(descriptor).expect("the entry finishes");
            }
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

/// The cheapest class this reader accepts, with `this_class` set to `name`.
fn minimal_class(name: &[u8]) -> Vec<u8> {
    let mut pool = Vec::new();
    pool.push(1_u8);
    pool.extend_from_slice(
        &u16::try_from(name.len())
            .expect("a name fits u16")
            .to_be_bytes(),
    );
    pool.extend_from_slice(name);
    pool.extend_from_slice(&[7, 0, 1]);
    let object = b"java/lang/Object";
    pool.push(1_u8);
    pool.extend_from_slice(
        &u16::try_from(object.len())
            .expect("a name fits u16")
            .to_be_bytes(),
    );
    pool.extend_from_slice(object);
    pool.extend_from_slice(&[7, 0, 3]);
    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    bytes.extend_from_slice(&0_u16.to_be_bytes());
    bytes.extend_from_slice(&52_u16.to_be_bytes());
    bytes.extend_from_slice(&5_u16.to_be_bytes());
    bytes.extend_from_slice(&pool);
    bytes.extend_from_slice(&0x0021_u16.to_be_bytes());
    bytes.extend_from_slice(&2_u16.to_be_bytes());
    bytes.extend_from_slice(&4_u16.to_be_bytes());
    for _ in 0..4 {
        bytes.extend_from_slice(&0_u16.to_be_bytes());
    }
    bytes
}

/// One class with a named superclass (or none, when `super_name` is empty) and one static member with
/// a body.
fn class_with_super(name: &[u8], super_name: &[u8], member: &[u8]) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_name = pool.utf8(name);
    let this_class = pool.class(this_name);
    // `super_class` of zero is JVMS 4.1's "no superclass": the end of a hierarchy a search can reach,
    // which is what a complete world needs at its top.
    let super_class = if super_name.is_empty() {
        0
    } else {
        let parent_name = pool.utf8(super_name);
        pool.class(parent_name)
    };
    let member_name = pool.utf8(member);
    let descriptor = pool.utf8(b"()V");
    let code_name = pool.utf8(b"Code");
    class_bytes_with(
        &pool,
        52,
        this_class,
        super_class,
        &[Member {
            access: 0x0009,
            name: member_name,
            descriptor,
            attributes: vec![(code_name, code_body(&[0xb1], 0, 0))],
        }],
        &[],
    )
}

/// `p/Orphan extends p/Absent`, with `p/Absent` present or absent.
///
/// A member search on `foo` has two different answers in the two worlds — `UnresolvedDependency`
/// when the supertype cannot be read, `Missing` when it can — and `here` answers `Resolved` in both.
/// The pair is P4's `a_missing_dependency_is_stated_by_name_and_never_as_the_negative_answer`
/// (`p4_x2_states.rs`) over bytes this file builds, so the differential runs over the same shape.
fn orphan_world(with_parent: bool) -> Vec<u8> {
    let orphan = class_with_super(b"p/Orphan", b"p/Absent", b"here");
    let mut entries = vec![(b"p/Orphan.class".to_vec(), orphan, STORE)];
    if with_parent {
        // `p/Absent` is the top of its own hierarchy: a search that reaches it can end, which is
        // what separates "the search read everything and `foo` is not there" from "a dependency was
        // missing".
        entries.push((
            b"p/Absent.class".to_vec(),
            class_with_super(b"p/Absent", b"", b"settle"),
            STORE,
        ));
    }
    zip_of(&entries)
}

/// The boundary matrix: the corpora the adversarial and shared-invariant scenarios name.
///
/// Which existing acceptance each corpus is anchored to, so this file does not become a second
/// authority for it:
///
/// | corpus | its existing acceptance |
/// | --- | --- |
/// | the understated-size ZIP | `crates/jarde-reader/src/artifact.rs::named_budgets_cover_declared_actual_read_and_entry_count` (A08/A14) |
/// | the condy graph | `p4_modern_facts.rs`'s shared-subgraph, cycle and four budget cases (A05) |
/// | the irreducible CFG | `p3_execution_comparison.rs`'s `suppressedCatching` expectancy (A12/A13) and `p3_guard.rs`'s guarded refusals |
/// | the missing dependency | `p3-corpus/v8-missing-dep` (A11) and `p4_x2_states.rs`'s unresolved-by-name case |
/// | cancellation | `p1_query_api.rs::cancellation_is_never_reported_as_complete`, `p1_artifact_tree.rs::budget_and_precancellation_return_non_complete_reliable_prefixes`, `p1_query_bounds.rs::cancellation_at_the_high_fanout_unit_boundary_publishes_nothing` (A14/A18) |
fn boundary_corpora() -> Vec<Boundary> {
    let room = limits();
    let mut corpora = vec![
        Boundary {
            label: "zip-bomb/understated-entry",
            bytes: bomb_archive(),
            providers: Vec::new(),
            ask: Ask::FullRange,
            // The bomb inflates to 8 KiB while both size fields say one byte, so the entry bound is
            // what stops it — the declared size cannot be used to pre-check it.
            limits: Limits {
                entry_bytes: 1024,
                ..room.clone()
            },
            cancel: false,
            exit: "partial",
            report: "entry_bytes",
            states: &[],
        },
        Boundary {
            label: "condy-graph/shared-subgraph",
            bytes: condy_bytes(),
            providers: Vec::new(),
            ask: Ask::FullRange,
            limits: room.clone(),
            cancel: false,
            exit: "complete",
            report: "",
            states: &["bootstrap_edge", "bootstrap_argument"],
        },
        Boundary {
            label: "irreducible-cfg/guarded-suppressedCatching",
            bytes: GUARDED.to_vec(),
            providers: vec![(b"Res.class", GUARDED_RESOURCE.to_vec())],
            ask: Ask::Member {
                name: b"suppressedCatching",
                descriptor: b"()V",
            },
            limits: room.clone(),
            cancel: false,
            exit: "complete",
            report: "",
            states: &["jre_region_irreducible"],
        },
        Boundary {
            label: "irreducible-cfg/guarded-one",
            bytes: GUARDED.to_vec(),
            providers: vec![(b"Res.class", GUARDED_RESOURCE.to_vec())],
            ask: Ask::Member {
                name: b"one",
                descriptor: b"()V",
            },
            limits: room.clone(),
            cancel: false,
            exit: "complete",
            report: "",
            states: &["\"representation\":\"java\""],
        },
        // The broken world: `p/Orphan` names a supertype the snapshot does not hold, so a member
        // search cannot answer for `foo` and states the class it could not read instead of the
        // negative (A11's `UnresolvedDependency`; `p4_x2_states.rs` owns the state's own cases).
        Boundary {
            label: "missing-dependency/unresolved",
            bytes: orphan_world(false),
            providers: Vec::new(),
            ask: Ask::Resolve {
                owner: b"p/Orphan",
                name: b"foo",
                descriptor: b"()V",
            },
            limits: room.clone(),
            cancel: false,
            exit: "complete",
            report: "",
            states: &["unresolved_dependency"],
        },
        // The same world with the supertype present, and the same class: one member the search
        // decides (`here`) and one it decides *negatively* (`foo`). This is A13's shape at the
        // resolution plane — a normal and a failing member of one class — and the pair is what keeps
        // `Missing` from being read as "the dependency was missing".
        Boundary {
            label: "missing-dependency/resolved",
            bytes: orphan_world(true),
            providers: Vec::new(),
            ask: Ask::Resolve {
                owner: b"p/Orphan",
                name: b"here",
                descriptor: b"()V",
            },
            limits: room.clone(),
            cancel: false,
            exit: "complete",
            report: "",
            states: &["\"state\":\"resolved\""],
        },
        Boundary {
            label: "missing-dependency/negative",
            bytes: orphan_world(true),
            providers: Vec::new(),
            ask: Ask::Resolve {
                owner: b"p/Orphan",
                name: b"foo",
                descriptor: b"()V",
            },
            limits: room.clone(),
            cancel: false,
            exit: "complete",
            report: "",
            states: &["\"state\":\"missing\"", "\"unresolved_dependencies\":[]"],
        },
        Boundary {
            label: "cancelled-under-decompression/nested",
            bytes: NESTED.to_vec(),
            providers: Vec::new(),
            ask: Ask::FullRange,
            limits: room.clone(),
            cancel: true,
            exit: "cancelled",
            report: "",
            states: &[],
        },
        Boundary {
            label: "budget-under-ir/guarded-one",
            bytes: GUARDED.to_vec(),
            providers: vec![(b"Res.class", GUARDED_RESOURCE.to_vec())],
            ask: Ask::Member {
                name: b"one",
                descriptor: b"()V",
            },
            limits: Limits {
                analysis_steps: 4,
                ..room.clone()
            },
            cancel: false,
            exit: "partial",
            report: "analysis_steps",
            states: &[],
        },
        Boundary {
            label: "cancelled-under-ir/guarded-one",
            bytes: GUARDED.to_vec(),
            providers: vec![(b"Res.class", GUARDED_RESOURCE.to_vec())],
            ask: Ask::Member {
                name: b"one",
                descriptor: b"()V",
            },
            limits: room,
            cancel: true,
            exit: "cancelled",
            report: "",
            states: &[],
        },
    ];
    // A row that takes its bytes from a builder still has to be able to say which bytes those are.
    for corpus in &mut corpora {
        assert!(
            !corpus.bytes.is_empty(),
            "{}: the boundary corpus is empty",
            corpus.label
        );
    }
    corpora
}

/// What a row's context states about the request it ran, in the words that request can be asked to
/// justify: the scope and view it named, the profile it ran under, the content sources it named and
/// whether the recovery layer was in the path.
struct RequestFacts {
    scope: String,
    view: String,
    profile: String,
    providers: String,
    recovery: String,
    premise: String,
}

/// One boundary row's run, recorded in the shape [`compare`] compares.
///
/// The snapshot is opened and the providers are opened under the **same** budget the request runs
/// under, so a provider's read is part of the request that named it — the boundary corpora are the
/// rows where a request's closure is not its own snapshot.
fn boundary_run(corpus: &Corpus, boundary: &Boundary, cache: Option<&FactsCache>) -> RunRecord {
    let engine = Engine::new();
    let mut budget = match cache {
        Some(cache) => Budget::new(boundary.limits.clone()).with_facts_cache(cache.clone()),
        None => Budget::new(boundary.limits.clone()),
    };
    let token = budget.cancellation_token();
    let started = Instant::now();
    let snapshot = engine
        .open(ArtifactInput::bytes(boundary.bytes.clone()), &mut budget)
        .expect("the boundary corpus opens");
    let providers: Vec<ArtifactSnapshot> = boundary
        .providers
        .iter()
        .map(|(_, bytes)| {
            engine
                .open(ArtifactInput::bytes(bytes.clone()), &mut budget)
                .expect("the provider corpus opens under the same request's budget")
        })
        .collect();
    // The premise: locating the member a recovery row presents is harness work, charged to its own
    // budget outside the measured section — the exclusion the local matrix row already makes
    // (`locate_member`). It runs before the cancellation, because a request that has not named its
    // member yet is not the request this row is about.
    let target = match &boundary.ask {
        Ask::Member { name, descriptor } => {
            let mut premise = Budget::new(limits());
            Some(locate_target(
                &engine,
                &snapshot,
                &mut premise,
                name,
                descriptor,
            ))
        }
        Ask::FullRange | Ask::Resolve { .. } => None,
    };
    // The cancellation arrives after the artifact (and its providers) are open and before the work
    // this row is about: the member the query layer has to inflate, or the body the recovery layer
    // has to analyse. It is delivered where the engine observes it — between units of work — which
    // is the only shape a single-threaded harness can make deterministic.
    if boundary.cancel {
        token.cancel();
    }
    let declared_bodies = target.as_ref().map_or(0, |target| target.declared_bodies);
    let (outcome, facts) = match &boundary.ask {
        Ask::FullRange => {
            let request = full_range_request(&snapshot);
            let report = engine
                .query(&snapshot, &request, &mut budget)
                .expect("a legal query is answered, not raised");
            (
                outcome_query(&report),
                RequestFacts {
                    scope: serde_json::to_string(&request.physical.scope)
                        .expect("a scope serializes"),
                    view: serde_json::to_string(&request.physical).expect("a view serializes"),
                    profile: "none: this entry takes no environment, so no runtime profile is \
                              named"
                        .to_string(),
                    providers: "none: this entry names no content source".to_string(),
                    recovery:
                        "not in this path: `Engine::query` reaches `jarde-query`, which does \
                               not depend on `jarde-java`"
                            .to_string(),
                    premise: "none: the whole measured section is the request".to_string(),
                },
            )
        }
        Ask::Member { name, descriptor } => {
            let environment = boundary_environment(&snapshot, &providers, 8);
            let target = target
                .as_ref()
                .expect("a recovery row located its member before the measured section");
            let request = MethodAnalysisRequest {
                environment,
                method: PhysicalMethodId {
                    owner: PhysicalDefinitionId {
                        location: PhysicalClassLocation::StandaloneRoot {
                            snapshot: snapshot.id().clone(),
                        },
                        class_bytes: target.owner.clone(),
                        variant: PhysicalVariant::Base,
                    },
                    name: bytes(name),
                    descriptor: bytes(descriptor),
                },
                stages: AnalysisStage::ALL.to_vec(),
            };
            let content: Vec<ArtifactSnapshot> = std::iter::once(snapshot.clone())
                .chain(providers.iter().cloned())
                .collect();
            let recovered = engine
                .recover_method(&content, &request, &mut budget)
                .expect("a legal request is answered, not raised");
            (
                outcome_recovery(&recovered),
                RequestFacts {
                    scope: format!(
                        "single member {}{} of {}",
                        String::from_utf8_lossy(name),
                        String::from_utf8_lossy(descriptor),
                        boundary.label
                    ),
                    view: serde_json::to_string(&request.environment.runtime.physical)
                        .expect("a view serializes"),
                    profile: serde_json::to_string(&request.environment.runtime.profile)
                        .expect("a profile serializes"),
                    providers: format!(
                        "{} (this request names no additional content source)",
                        request.environment.providers.len()
                    ),
                    recovery: "the presentation of one run's own payload, gated on the profile \
                               above"
                        .to_string(),
                    premise: format!(
                        "1 header read to locate {} before the measured section, charged to its own \
                         budget; the class declares {} bodies",
                        String::from_utf8_lossy(name),
                        target.declared_bodies
                    ),
                },
            )
        }
        Ask::Resolve {
            owner,
            name,
            descriptor,
        } => {
            let environment = boundary_environment(&snapshot, &providers, 8);
            let request = ResolutionRequest {
                environment,
                target: SymbolRef::Method {
                    owner: bytes(owner),
                    name: bytes(name),
                    descriptor: bytes(descriptor),
                },
                use_kind: ReferenceUse::InvokeStatic,
                caller: CallerContext {
                    loader: LoaderId("app".to_string()),
                    enclosing: None,
                },
                dispatch: None,
            };
            let content: Vec<ArtifactSnapshot> = std::iter::once(snapshot.clone())
                .chain(providers.iter().cloned())
                .collect();
            let report = engine
                .resolve_symbol(&content, &request, &mut budget)
                .expect("a legal request is answered, not raised");
            (
                outcome_resolution(&report),
                RequestFacts {
                    scope: format!(
                        "one symbol {}.{}{}",
                        String::from_utf8_lossy(owner),
                        String::from_utf8_lossy(name),
                        String::from_utf8_lossy(descriptor)
                    ),
                    view: serde_json::to_string(&request.environment.runtime.physical)
                        .expect("a view serializes"),
                    profile: serde_json::to_string(&request.environment.runtime.profile)
                        .expect("a profile serializes"),
                    providers: format!(
                        "{} named content source(s), {} root(s)",
                        request.environment.providers.len(),
                        request.environment.runtime.load_domain.roots.len()
                    ),
                    recovery: "not in this path: `Engine::resolve_symbol` reaches the resolution \
                               layer, which does not present source"
                        .to_string(),
                    premise: "none: locating the symbol is the request".to_string(),
                },
            )
        }
    };
    let wall_micros = started.elapsed().as_micros();
    let context = Context {
        corpus_version: corpus.version.clone(),
        subject: boundary.label,
        subject_bytes: boundary.bytes.len() as u64,
        subject_blake3: blake3::hash(&boundary.bytes).to_hex().to_string(),
        path: "direct",
        cache: match cache {
            Some(cache) => cache_context(cache),
            None => REFERENCE_CACHE.to_string(),
        },
        concurrency: REFERENCE_CONCURRENCY,
        scope: facts.scope,
        view: facts.view,
        profile: facts.profile,
        providers: facts.providers,
        recovery: facts.recovery,
        premise: facts.premise,
        budget: boundary.limits.clone(),
    };
    RunRecord {
        label: boundary.label,
        context,
        status: outcome.status,
        usage: budget.usage(),
        report_usage: outcome.report_usage,
        coverage: Some(outcome.coverage),
        diagnostics: outcome.diagnostics,
        order: outcome.order,
        published_items: outcome.published_items,
        declared_bodies,
        derivations: outcome.derivations,
        items_without_a_consumer: outcome.items_without_a_consumer,
        published: outcome.published,
        document: outcome.document,
        fingerprint: outcome.fingerprint,
        evidence: outcome.evidence,
        evidence_fingerprint: outcome.evidence_fingerprint,
        represents_a_representation: matches!(boundary.ask, Ask::Member { .. }),
        wall_micros,
    }
}

/// The environment a boundary row runs under: the corpus's own snapshot, the providers it names and
/// nothing else.
fn boundary_environment(
    snapshot: &ArtifactSnapshot,
    providers: &[ArtifactSnapshot],
    release: u16,
) -> ResolutionEnvironment {
    let mut roots = vec![snapshot_root(snapshot)];
    roots.extend(providers.iter().map(snapshot_root));
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
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release: release,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: domain.clone(),
        },
        domains: vec![domain],
        providers: providers
            .iter()
            .enumerate()
            .map(|(index, provider)| HeaderProvider {
                id: ProviderId(format!("boundary-headers-{index}")),
                roots: vec![snapshot_root(provider)],
            })
            .collect(),
    }
}

/// Locate `name`/`descriptor` through the reader's own header entry, under the row's own budget.
///
/// This is the same premise the local matrix row runs (`locate_member`), stated for a member the
/// corpus names: the identity the request is built from is the one the header read located, and the
/// member has to be declared with a body, so a corpus that stopped declaring it fails here instead
/// of producing a row about nothing.
fn locate_target(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    budget: &mut Budget,
    name: &[u8],
    descriptor: &[u8],
) -> MemberTarget {
    let inspected = engine
        .inspect_header(snapshot, ClassTarget::Root, budget, InspectionMode::Strict)
        .expect("the boundary corpus's own header is readable");
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
        .filter(|member| {
            member.name.raw().0.as_slice() == name
                && member.descriptor.raw().0.as_slice() == descriptor
        })
        .map(|member| member.name.raw().0.clone())
        .collect();
    assert_eq!(
        declared.len(),
        1,
        "the boundary corpus declares {} member(s) called {}{}, so this row would be about a member \
         it does not have",
        declared.len(),
        String::from_utf8_lossy(name),
        String::from_utf8_lossy(descriptor)
    );
    MemberTarget {
        owner: inspected.source.class_bytes,
        declared_bodies: inspected
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
            .count(),
    }
}

/// The 3.2 regression: every boundary corpus, run with the cache off, cold and warm.
///
/// The five corpora are the ones the two scenarios name — the understated-size ZIP, the condy graph,
/// the irreducible CFG, the missing dependency and the cancellation rows — and the differential over
/// them is the matrix's own: the same three runs, the same planes, the same `compare`. What this test
/// adds to `Cold and warm results` is the *shape* of the inputs: a corpus whose declared sizes lie, a
/// corpus whose facts carry a bootstrap graph, a body the recovery layer refuses, a symbol nothing
/// provides, and two runs whose token is cancelled with decompression or IR work still ahead.
///
/// The `A13` rows are here too: `guarded-one` and `guarded-suppressedCatching` are two members of one
/// class — one presented, one refused — and `missing-dependency/control` beside
/// `missing-dependency/unresolved` is the same shape in the resolution plane. Each row states the
/// spelling its corpus is about, so a row that quietly stopped being that shape fails rather than
/// comparing two empty results.
#[test]
fn every_boundary_corpus_publishes_the_same_result_with_and_without_the_cache() {
    let corpus = corpus();
    let boundaries = boundary_corpora();
    for boundary in &boundaries {
        let store = facts_cache();
        let reference = boundary_run(&corpus, boundary, None);
        let cold = boundary_run(&corpus, boundary, Some(&store));
        let warm = boundary_run(&corpus, boundary, Some(&store));

        println!();
        println!(
            "corpus {} ({} bytes, cache off → cold → warm)",
            boundary.label,
            boundary.bytes.len()
        );
        for (pair, comparison) in [
            ("reference → cold", compare(&reference, &cold)),
            ("cold → warm", compare(&cold, &warm)),
            ("reference → warm", compare(&reference, &warm)),
        ] {
            for plane in GATE_PLANES {
                let verdict = plane_comparison(plane, &comparison);
                println!("  {:<16} {:<24} {}", pair, plane, verdict.describe());
                // A plane with no object reads as absent and is printed as such; a plane that was
                // compared and came out different is the failure.
                assert!(
                    verdict.equal() || matches!(verdict, PlaneComparison::NoObject(_)),
                    "{}: {pair} — the plane `{plane}` is {}, so the cache changed a result on this \
                     corpus:\n{}",
                    boundary.label,
                    verdict.describe(),
                    comparison.lines().join("\n")
                );
                assert_eq!(
                    matches!(verdict, PlaneComparison::NoObject(_)),
                    plane == "representation"
                        && matches!(boundary.ask, Ask::FullRange | Ask::Resolve { .. }),
                    "{}: the `representation` plane is {} on a row that {} a representation, so the \
                     plane table is not reporting this row's own shape",
                    boundary.label,
                    verdict.describe(),
                    if matches!(boundary.ask, Ask::Member { .. }) {
                        "publishes"
                    } else {
                        "does not publish"
                    }
                );
            }
        }

        // The store really was in the loop, and the warm run was really answered by it: a row over a
        // corpus no request parses would compare two cold runs and prove nothing.
        if !boundary.cancel {
            let observed = store.report();
            assert!(
                observed.consultations > 0,
                "{}: the store was never consulted, so this row is not about a cache: {observed:?}",
                boundary.label
            );
            assert!(
                observed.hits > 0,
                "{}: the warm run answered nothing from the store: {observed:?}",
                boundary.label
            );
        }

        // The exit the corpus is about: the budget it runs out of, the cancellation it receives, or
        // a complete run.
        assert_eq!(
            warm.status, boundary.exit,
            "{}: the row publishes `{}` and this corpus's exit is `{}`",
            boundary.label, warm.status, boundary.exit
        );
        let termination = termination_of(&warm);
        println!(
            "  exit {}{}",
            warm.status,
            if termination == "none" {
                String::new()
            } else {
                format!(" stating {termination}")
            }
        );
        if !boundary.report.is_empty() {
            assert!(
                termination.contains(boundary.report),
                "{}: the stopped row states `{termination}` rather than `{}`",
                boundary.label,
                boundary.report
            );
        }
        // "返回已扫描范围和终止原因", and its other half — a run that stopped may not read as
        // complete. The termination reason is asserted above (the status, with the dimension where
        // the status carries one); the range is the coverage document, in the engine's own spelling:
        // a stopped run has at least one dimension that is not `complete_within_schema`, and a
        // stopped *scan* states the ranges it covered and the ordinals it never established.
        if boundary.exit != "complete" {
            let coverage = warm
                .coverage
                .clone()
                .expect("a stopped row publishes the coverage it reached");
            let dimensions = coverage_dimensions(&coverage);
            assert!(
                dimensions
                    .values()
                    .any(|dimension| dimension["state"] != "complete_within_schema"),
                "{}: the run stopped and its coverage claims every dimension was covered: {coverage}",
                boundary.label
            );
            if matches!(boundary.ask, Ask::FullRange) {
                let stated = dimensions.values().any(|dimension| {
                    let non_empty = |key: &str| {
                        dimension
                            .get(key)
                            .and_then(Value::as_array)
                            .is_some_and(|ranges| !ranges.is_empty())
                    };
                    non_empty("scanned") || non_empty("skipped")
                });
                assert!(
                    stated,
                    "{}: a stopped scan reported no scanned and no skipped range: {coverage}",
                    boundary.label
                );
            }
        }

        let document = serde_json::to_string(&warm.document).expect("a document renders");
        for spelling in boundary.states {
            assert!(
                document.contains(spelling),
                "{}: the published result does not state `{spelling}`, so this row is not the shape \
                 it claims to be about: {document}",
                boundary.label,
                spelling = spelling
            );
        }
    }
}

// -------------------------------------------------------------------------------------------
// The gate: a candidate may stand in the default path only while its differential holds
// -------------------------------------------------------------------------------------------

/// One candidate's differential: the rows its configuration was compared on, and — when it has none
/// — the reason there is no second path at all.
struct CandidateDifferential {
    candidate: &'static str,
    configuration: Option<(&'static str, &'static str)>,
    rows: Vec<(String, Comparison)>,
    no_second_path: Option<&'static str>,
}

impl CandidateDifferential {
    fn has_a_second_path(&self) -> bool {
        self.no_second_path.is_none() && !self.rows.is_empty()
    }
}

/// `Correctness and resource regression gates` as a function: `Ok(())` only while every row of the
/// differential publishes the same result on every plane, `Err(reason)` — the reason default
/// enabling is blocked — otherwise.
///
/// This is the executable form of "任一语义差异 MUST 阻断默认启用". It is a function rather than a
/// block inside the gate so the rule itself is testable, exactly like [`check_candidate`]: the
/// record as it stands has to pass it, and a pair that really differs has to be refused
/// (`the_enablement_rule_refuses_a_candidate_whose_paths_differ`).
fn blocking_difference(differential: &CandidateDifferential) -> Option<String> {
    if let Some(why) = differential.no_second_path {
        return Some(format!(
            "{}: no second path exists, so there is nothing to compare: {why}",
            differential.candidate
        ));
    }
    for (row, comparison) in &differential.rows {
        for plane in GATE_PLANES {
            // A plane with no object on a given row is carried by the differential's other rows —
            // the recovery rows publish the representation, the cancelled rows the cancellation —
            // and the gate refuses a differential in which no row carries one at all. What blocks
            // enabling is a plane that was compared and came out different.
            match plane_comparison(plane, comparison) {
                PlaneComparison::Compared(true) | PlaneComparison::NoObject(_) => {}
                PlaneComparison::Compared(false) => {
                    return Some(format!(
                        "{}: {row} — the plane `{plane}` is {}, so default enabling is blocked:\n{}",
                        differential.candidate,
                        "DIFFERENT",
                        comparison.lines().join("\n")
                    ));
                }
            }
        }
    }
    None
}

/// Whether a candidate may stand in the default path: its differential has a second path and every
/// plane of it is equal.
fn enablement_allowed(differential: &CandidateDifferential) -> std::result::Result<(), String> {
    if let Some(reason) = blocking_difference(differential) {
        return Err(reason);
    }
    if !differential.has_a_second_path() {
        return Err(format!(
            "{}: there is no second path to compare, so nothing has been shown about enabling it",
            differential.candidate
        ));
    }
    Ok(())
}

/// The runner a recorded configuration names, or the reason this harness cannot run it.
///
/// The record keeps configurations as text so a candidate can state one that does not exist yet
/// (2.1's and 2.2's do). This is where that text has to meet a path that can really run: a label with
/// no runner is not a comparison, and the gate refuses to substitute the reference path for it.
fn differential_runner(configuration: (&'static str, &'static str)) -> Option<()> {
    if configuration.0 == CACHE_ON && configuration.1 == REFERENCE_CONCURRENCY {
        Some(())
    } else {
        None
    }
}

/// The full differential of one candidate: the matrix's own row for it, the local row, the
/// cancellation row and every boundary corpus, each run with the cache off, cold and warm.
fn candidate_differential(
    candidate: &Candidate,
    corpus: &Corpus,
    boundaries: &[Boundary],
) -> CandidateDifferential {
    let mut rows: Vec<(String, Comparison)> = Vec::new();
    let measured = match &candidate.benefit {
        Benefit::Unmeasured { why } => {
            return CandidateDifferential {
                candidate: candidate.id,
                configuration: None,
                rows,
                no_second_path: Some(why),
            };
        }
        Benefit::Measured(measured) => measured,
    };
    // A recorded configuration with no runner here is a gap, not a comparison: the gate says so
    // instead of measuring the reference path twice and calling it the candidate.
    if differential_runner(measured.configuration).is_none() {
        return CandidateDifferential {
            candidate: candidate.id,
            configuration: Some(measured.configuration),
            rows,
            no_second_path: Some(
                "this harness has no runner for the configuration the record names, so the \
                 candidate side would have to be simulated; a simulated second path is the \
                 measurement-shaped claim the design refuses",
            ),
        };
    }

    // (a) The two matrix rows the record's benefit is measured over, as `published_rows` runs them.
    let matrix = published_rows(corpus);
    let reference_row = matrix
        .iter()
        .find(|row| row.label == measured.reference)
        .unwrap_or_else(|| {
            panic!(
                "the record names `{}`, which the matrix does not run",
                measured.reference
            )
        });
    let candidate_row = matrix
        .iter()
        .find(|row| row.label == measured.candidate)
        .unwrap_or_else(|| {
            panic!(
                "the record names `{}`, which the matrix does not run",
                measured.candidate
            )
        });
    rows.push((
        format!(
            "the matrix row the record names ({} → {})",
            measured.reference, measured.candidate
        ),
        compare(reference_row, candidate_row),
    ));

    // (b) The local row: the same two configurations over the recovery path, which is the only row
    // of the matrix that publishes a representation.
    let class = verified(corpus, &SUBJECTS[1]);
    let premise = locate_member(&class.content);
    let store = facts_cache();
    let local_reference =
        run_single_member_with(corpus, &class, "single-member/v52-class", &premise, None);
    let local_cold = run_single_member_with(
        corpus,
        &class,
        "single-member/v52-class",
        &premise,
        Some(&store),
    );
    let local_warm = run_single_member_with(
        corpus,
        &class,
        "single-member/v52-class",
        &premise,
        Some(&store),
    );
    rows.push((
        "single-member/v52-class (reference → warm)".to_string(),
        compare(&local_reference, &local_warm),
    ));
    rows.push((
        "single-member/v52-class (cold → warm)".to_string(),
        compare(&local_cold, &local_warm),
    ));

    // (c) The cancellation row over the matrix's own archive: both configurations have to answer
    // the same way, and the answer may not be a quieter complete run.
    let jar = verified(corpus, &SUBJECTS[0]);
    let cancelled_reference = run_full_range_with(
        corpus,
        &jar,
        "cancelled/full-range-xref/minimal-jar",
        true,
        None,
    );
    let cancelled_candidate = run_full_range_with(
        corpus,
        &jar,
        "cancelled/full-range-xref/minimal-jar",
        true,
        Some(&facts_cache()),
    );
    rows.push((
        "cancelled/full-range-xref/minimal-jar (reference → cache on)".to_string(),
        compare(&cancelled_reference, &cancelled_candidate),
    ));

    // (d) Every boundary corpus of 3.2, through the same three runs.
    for boundary in boundaries {
        let store = facts_cache();
        let reference = boundary_run(corpus, boundary, None);
        let cold = boundary_run(corpus, boundary, Some(&store));
        let warm = boundary_run(corpus, boundary, Some(&store));
        rows.push((
            format!("{}/reference → cold", boundary.label),
            compare(&reference, &cold),
        ));
        rows.push((
            format!("{}/cold → warm", boundary.label),
            compare(&cold, &warm),
        ));
        rows.push((
            format!("{}/reference → warm", boundary.label),
            compare(&reference, &warm),
        ));
    }
    CandidateDifferential {
        candidate: candidate.id,
        configuration: Some(measured.configuration),
        rows,
        no_second_path: None,
    }
}

/// The gate. Every enabled candidate has to clear its own differential, and a difference anywhere in
/// a measured candidate's differential is a failure of the record — not merely of the candidate —
/// because `CANDIDATES` claims a measured benefit over exactly those rows.
///
/// It is deliberately **not** "if the candidate is enabled, check it": the record of a measured
/// candidate is checked whatever its state, so a semantic difference between the two paths is red
/// the day it appears, and `enablement_allowed` is what states whether the state may move.
#[test]
fn a_candidate_may_be_enabled_only_while_its_differential_is_equivalent() {
    let corpus = corpus();
    let boundaries = boundary_corpora();
    for candidate in &CANDIDATES {
        let differential = candidate_differential(candidate, &corpus, &boundaries);
        println!();
        println!("candidate {} ({:?})", candidate.id, candidate.default_state);
        println!("  configuration {:?}", differential.configuration);
        for (row, comparison) in &differential.rows {
            println!("  row {row}");
            for plane in GATE_PLANES {
                println!(
                    "    {:<24} {}",
                    plane,
                    plane_comparison(plane, comparison).describe()
                );
            }
        }
        if let Some(why) = differential.no_second_path {
            println!("  no second path: {why}");
        }
        // A **measured** claim is checked whatever the candidate's state: the record says a benefit
        // was measured over exactly these two paths, so a difference between them falsifies the
        // record rather than merely blocking the candidate. A candidate with no second path is the
        // other case, and its state (disabled) is the one `enablement_allowed` refuses.
        if matches!(candidate.benefit, Benefit::Measured(_)) {
            if let Some(refusal) = blocking_difference(&differential) {
                panic!(
                    "{refusal}\n\nA semantic difference between the two paths of a measured \
                     candidate blocks default enabling. Fix the path, or move the candidate out of \
                     `Measured` and record what it is: the one thing the record may not do is stay \
                     as it is."
                );
            }
        } else {
            assert!(
                !differential.has_a_second_path(),
                "{} records no measured benefit and this harness ran its differential anyway: {}",
                candidate.id,
                differential.rows.len()
            );
        }
        assert!(
            differential.rows.is_empty()
                || differential
                    .rows
                    .iter()
                    .any(|(_, comparison)| comparison.representation),
            "{}: no row of this differential publishes a representation, so the `representation` \
             plane of the gate would be absent on every row without the reason being recorded",
            candidate.id
        );
        if candidate.default_state == DefaultState::Enabled {
            assert!(
                enablement_allowed(&differential).is_ok(),
                "{} is in the default path while its differential does not clear the gate",
                candidate.id
            );
        }
    }

    println!();
    for (subject, why) in NO_SECOND_PATH_TODAY {
        println!("no second value today: {subject} — {why}");
    }

    // Today's default state, from the three sides that can state it: no candidate is enabled, the
    // reference configuration of the matrix is the cache-off one, and the budget every caller builds
    // carries no store.
    assert!(
        CANDIDATES
            .iter()
            .all(|candidate| candidate.default_state == DefaultState::Disabled),
        "a candidate stands in the default path: {:?}",
        CANDIDATES
            .iter()
            .map(|candidate| (candidate.id, candidate.default_state))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        runnable_configurations(&corpus)
            .iter()
            .next()
            .map(|(cache, _)| cache.clone()),
        Some(REFERENCE_CACHE.to_string()),
        "the reference configuration of the matrix is no longer the cache-off one, so the default \
         path is not the one the differentials are measured against"
    );
    assert!(
        Budget::new(limits()).facts_cache().is_none(),
        "a budget built the way every caller builds one carries a facts cache, so the cache would be \
         on in the default path"
    );
}

/// The gate's rule, on a pair that really differs: a candidate whose two paths disagree may not be
/// enabled, and the refusal names the plane that moved.
#[test]
fn the_enablement_rule_refuses_a_candidate_whose_paths_differ() {
    let corpus = corpus();
    let jar = verified(&corpus, &SUBJECTS[0]);
    let complete = run_full_range(&corpus, &jar, "full-range-xref/minimal-jar", false);
    let cancelled = run_full_range(&corpus, &jar, "cancelled/full-range-xref/minimal-jar", true);
    let differing = CandidateDifferential {
        candidate: "a candidate whose second path stops early",
        configuration: Some((CACHE_ON, REFERENCE_CONCURRENCY)),
        rows: vec![(
            "a cancelled run against a complete one".to_string(),
            compare(&complete, &cancelled),
        )],
        no_second_path: None,
    };
    let refusal = enablement_allowed(&differing).expect_err(
        "a pair that publishes different results is not a differential a candidate may be enabled \
         on",
    );
    assert!(
        refusal.contains("the plane `evidence`"),
        "the refusal has to name the plane that moved, and it says: {refusal}"
    );
    println!("{refusal}");

    // A candidate with one path is blocked by absence, not carried by a comparison of nothing.
    let absent = CandidateDifferential {
        candidate: "a candidate with one path",
        configuration: None,
        rows: Vec::new(),
        no_second_path: Some("nothing to run twice"),
    };
    assert!(enablement_allowed(&absent).is_err());
}
