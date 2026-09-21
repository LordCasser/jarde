//! Shared support for the five `bulk_recovery_*` targets.
//!
//! The bulk operation is one library call over a whole physical scope, so every target that
//! exercises it needs the same three things: a fixture scope that really holds several classes and
//! a nested container, a request built the way a caller builds one, and a sink that records what it
//! was handed instead of writing it somewhere. This module is those three things, once.
//!
//! # Fixtures
//!
//! Both fixtures are **honest artifacts**: every class entry holds a real class file of this
//! repository's own committed fixtures, and the entry's name is where the class is looked up under
//! the environment's own prefix rule (`prefix + internal name + ".class"`). Neither fixture is
//! checked in as bytes — the ZIP writer is the repository's `rawzip` dev-dependency — so the shapes
//! below are also the statement of what each one holds.
//!
//! * [`flat_fixture`] is one archive with four class entries at its root: `Scope.class`,
//!   `Shape.class`, `LambdaSample.class` and `Holder.class`, in that order, and nothing else. It is
//!   the scope whose order is exactly this list.
//! * [`nested_fixture`] is the same four classes with the fourth **inside a nested archive**:
//!   `lib/holder.jar` (stored, so a reader can reach it without inflating anything) holds one entry,
//!   `d/Holder.class`, and the traversal descends into it at its own entry, so the scope is
//!   `Scope.class`, `Shape.class`, `LambdaSample.class`, `d/Holder.class` — the nested class last.
//! * [`handover_fixture`] holds **two nested containers and no class at the root's own level**:
//!   `lib/classes.jar` (deflated) holds the four classes under `p/` and `res/assets.jar` (stored)
//!   holds a manifest and a resource entry, so the walk leaves the container every class came from
//!   while classes dispatched from it are still waiting to be read — the shape a class task's
//!   container handover exists for.
//!
//! [`FLAT_PREFIXES`] and [`NESTED_PREFIXES`] are the load prefixes of those two scopes.
//! [`container_roots`] turns one prefix list into the load roots the environment declares: each
//! prefix names the container that really holds class entries under it, which is what makes the
//! environment honest rather than a declaration of containers nobody looked at.
//!
//! # The recording sink
//!
//! [`Recorder`] answers every callback on the coordinator thread, keeps the event, and then decides
//! what to answer from [`Recorder::method_behaviour`]. [`Behaviour::StopAt`] and
//! [`Behaviour::FailAt`] both **confirm** the first `n` method records and stop or fail on the record
//! after them, which is the prefix semantics a consumer has: what it confirmed stands, and the record
//! it refused is not one of them.
//!
//! # Fingerprints
//!
//! Two runs of the same input with different worker counts publish the same per-method content in
//! the same order, but they can never publish the same *resource readings*: elapsed time and usage
//! figures are properties of a run, not of its result. [`Fingerprint`] is therefore a digest of the
//! semantic fields only — identity, order, content, outcome — and [`fingerprint`] states exactly
//! which fields of a summary it reads.
//!
//! This module is compiled into several test targets, and each of them uses a different part of it:
//! the fixture a target does not read is not an unused definition, it is another target's fixture.
#![allow(dead_code)]

use flate2::write::DeflateEncoder;
use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::time::Duration;

// ---------------------------------------------------------------------------------------------
// The fixture classes
// ---------------------------------------------------------------------------------------------

/// A ZIP entry that is stored rather than deflated.
pub const STORE: u16 = 0;

/// A ZIP entry that is deflated.
pub const DEFLATE: u16 = 8;

/// `Scope`: a class whose members cover the local-variable shapes the recovery layer presents.
pub const SCOPE: &[u8] = include_bytes!("fixtures/p3-scope/v8/Scope.class");

/// `Shape`: an interface, so one of its members is a declaration with no `Code` attribute at all.
pub const SHAPE: &[u8] = include_bytes!("fixtures/p3-declaration/v8/Shape.class");

/// `LambdaSample`: a class with `invokedynamic` sites, so a member's result quotes lambda evidence.
pub const LAMBDA: &[u8] = include_bytes!("fixtures/b2-bootstrap-descriptor/v8/LambdaSample.class");

/// `Holder`: a class with a `<clinit>`, a constructor and both kinds of ordinary member.
///
/// Its bytes declare the class `Holder` in the default package while the nested fixture stores them
/// under `d/Holder.class`: the entry's path is where the environment looks a name up, and the name a
/// class declares is its own. That mismatch is the fixture's point — a physical scope is read by
/// identity, not by a name derived from a path.
pub const HOLDER: &[u8] = include_bytes!("fixtures/p3-declaration/v8/Holder.class");

/// The ECJ 4.6.1 `-source 1.3` sample of the `jsr`/`ret` era whose `finallyPath(I)I` really holds a
/// subroutine, so its presentation is an explanation rather than statements.
pub const HISTORICAL: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class");

/// The load prefixes of [`flat_fixture`]'s one container: the root's own root.
pub const FLAT_PREFIXES: [&[u8]; 1] = [b""];

/// The load prefixes of [`nested_fixture`]'s two containers, in the order the traversal visits them.
///
/// The root container provides the three classes at its own root, and the nested container provides
/// `Holder` under the prefix its entry lives at: the class declares its name `Holder`, and the entry
/// that holds it is `d/Holder.class`.
pub const NESTED_PREFIXES: [&[u8]; 2] = [b"", b"d/"];

/// The load prefix of [`handover_fixture`]'s class container: the one position the environment
/// searches, declared on the container that really holds the classes. The asset container provides
/// nothing and declares no root.
pub const HANDOVER_PREFIXES: [&[u8]; 1] = [b"p/"];

/// A small `META-INF/MANIFEST.MF`, so [`handover_fixture`]'s second container really holds a
/// non-class entry of the shape an archive usually carries.
const MANIFEST: &[u8] = b"Manifest-Version: 1.0\nCreated-By: jarde bulk fixture\n";

/// An opaque resource entry, so that container holds something that is not a class and is not an
/// archive candidate either.
const RESOURCE: &[u8] = b"jarde handover fixture: this container holds no class\n";

// ---------------------------------------------------------------------------------------------
// Fixtures, scopes and requests
// ---------------------------------------------------------------------------------------------

/// The limits every budget of these targets is opened with: wide enough that no dimension of the
/// fixtures' own work is what stops a run, and with no deadline, so a stop is always the one the
/// test caused.
pub fn limits() -> Limits {
    Limits {
        input_bytes: 16 * 1024 * 1024,
        archive_entries: 10_000,
        entry_bytes: 16 * 1024 * 1024,
        read_bytes: 64 * 1024 * 1024,
        class_bytes: 16 * 1024 * 1024,
        attribute_bytes: 16 * 1024 * 1024,
        code_bytes: 16 * 1024 * 1024,
        result_items: 100_000,
        output_bytes: 64 * 1024 * 1024,
        class_headers: 10_000,
        method_bodies: 10_000,
        ir_items: 1 << 22,
        ir_edges: 1 << 22,
        analysis_steps: 1 << 22,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

/// One stored-or-deflated ZIP with the given entries, in the order given.
///
/// The order is the central-directory order the reader publishes as entry ordinals, which is why
/// every fixture here places its entries deliberately.
pub fn zip(entries: &[(&[u8], &[u8], u16)]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data, method) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(*method))
                .start()
                .expect("the fixture entry starts");
            if *method == DEFLATE {
                let encoder = DeflateEncoder::new(&mut entry, flate2::Compression::default());
                let mut writer = config.wrap(encoder);
                writer
                    .write_all(data)
                    .expect("the fixture entry is writable");
                let (encoder, descriptor) = writer.finish().expect("the fixture entry closes");
                encoder.finish().expect("the delegate is finished");
                entry
                    .finish(descriptor)
                    .expect("the fixture entry finishes");
            } else {
                let mut writer = config.wrap(&mut entry);
                writer
                    .write_all(data)
                    .expect("the fixture entry is writable");
                let (_, descriptor) = writer.finish().expect("the fixture entry closes");
                entry
                    .finish(descriptor)
                    .expect("the fixture entry finishes");
            }
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

/// Four classes at one archive's root, in the order the scope walks them.
pub fn flat_fixture() -> Vec<u8> {
    zip(&[
        (b"Scope.class", SCOPE, DEFLATE),
        (b"Shape.class", SHAPE, DEFLATE),
        (b"LambdaSample.class", LAMBDA, DEFLATE),
        (b"Holder.class", HOLDER, DEFLATE),
    ])
}

/// The same four classes, with the last one inside a nested archive the scope descends into.
pub fn nested_fixture() -> Vec<u8> {
    let nested = zip(&[(b"d/Holder.class", HOLDER, DEFLATE)]);
    zip(&[
        (b"Scope.class", SCOPE, DEFLATE),
        (b"Shape.class", SHAPE, DEFLATE),
        (b"LambdaSample.class", LAMBDA, DEFLATE),
        (b"lib/holder.jar", &nested, STORE),
    ])
}

/// One nested container that holds every class of the scope, and a second one that holds none.
///
/// The root container holds two archives and no class entry, so the whole scope sits one level down:
/// `lib/classes.jar` (deflated) holds the four classes under the prefix `p/`, and `res/assets.jar`
/// (stored) holds a manifest and a resource entry. The traversal descends into the class container
/// first, yields its four classes, and goes on into the asset container — so the walk really does
/// leave the container the classes came from while they are being read, and a class task that reads
/// after it (the delay a test can ask for holds every task at the head) finds nothing holding that
/// container but the handover the walk gave it.
///
/// The second container deliberately holds **no class**: a second container that provided classes
/// would be a second search position, and every member's loader binding query would then walk it —
/// parsing a directory no consumer is holding, which is the cache's own business rather than the
/// handover this fixture exists to measure.
pub fn handover_fixture() -> Vec<u8> {
    let classes = zip(&[
        (b"p/Scope.class", SCOPE, STORE),
        (b"p/Shape.class", SHAPE, DEFLATE),
        (b"p/LambdaSample.class", LAMBDA, DEFLATE),
        (b"p/Holder.class", HOLDER, STORE),
    ]);
    let assets = zip(&[
        (b"META-INF/MANIFEST.MF", MANIFEST, STORE),
        (b"res/data.bin", RESOURCE, STORE),
    ]);
    zip(&[
        (b"lib/classes.jar", &classes, DEFLATE),
        (b"res/assets.jar", &assets, STORE),
    ])
}

/// Opens one in-memory snapshot, together with the usage the opening itself cost.
///
/// The snapshot is the only product a caller keeps; the usage is handed back so a target can state
/// what the read before the operation cost, and so the operation's own account is never confused
/// with it (the operation folds in the budget it is handed, not this one).
pub fn open(bytes: Vec<u8>) -> (ArtifactSnapshot, UsageSnapshot) {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes), &mut budget)
        .expect("the fixture is a readable archive");
    (snapshot, budget.usage())
}

/// The whole-tree scope of a fresh ZIP snapshot.
pub fn tree_scope() -> PhysicalScope {
    PhysicalScope::ArtifactTree {
        root_container: ContainerId("root".to_owned()),
    }
}

/// The load roots of `snapshot`, one per prefix, in the order the prefixes are given.
///
/// Each prefix is matched to the container that really holds class entries under it — the first
/// container of the tree walk, not already claimed by an earlier prefix, with an entry whose raw
/// name starts with the prefix and ends with `.class`. A prefix no container serves is a broken
/// fixture or a broken prefix list, and this helper says so instead of declaring a root at a
/// position that provides nothing.
pub fn container_roots(
    snapshot: &ArtifactSnapshot,
    budget: &mut Budget,
    prefixes: &[&[u8]],
) -> Vec<LoadRoot> {
    let report = snapshot
        .enumerate_artifact_tree(budget)
        .expect("the fixture's artifact tree is readable");
    let mut claimed = vec![false; report.containers.len()];
    let mut roots = Vec::with_capacity(prefixes.len());
    for prefix in prefixes {
        let found = report
            .containers
            .iter()
            .enumerate()
            .find(|(index, container)| {
                !claimed[*index]
                    && container.entries.iter().any(|entry| {
                        let name = entry.id.raw_name.0.as_slice();
                        name.starts_with(prefix) && name.ends_with(b".class")
                    })
            });
        let Some((index, container)) = found else {
            panic!(
                "no unclaimed container of this fixture holds a class entry under prefix {:?}",
                String::from_utf8_lossy(prefix)
            );
        };
        claimed[index] = true;
        roots.push(LoadRoot::Container {
            origin: container.origin.clone(),
            prefix: ArchiveNameBytes(prefix.to_vec()),
        });
    }
    roots
}

/// The environment declaration these targets run under: the caller's own roots, in the caller's own
/// order, at Java 8 with no multi-release expansion and no detected layout.
pub fn environment(
    snapshot: &ArtifactSnapshot,
    scope: PhysicalScope,
    roots: Vec<LoadRoot>,
) -> EnvironmentRequest {
    EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope,
        policy: EnvironmentPolicy::ExplicitClasspath { roots },
        profile: RuntimeProfile {
            java_release: 8,
            multi_release: MultiReleasePolicy::Disabled,
            layout: LayoutMode::Generic,
        },
        loader: LoaderId("app".to_owned()),
    }
}

/// The request a caller that has one environment and one worker count means: the environment's own
/// view, the published defaults for every capacity, and the per-method limits these targets use.
pub fn request(environment: EnvironmentRequest, workers: usize) -> BulkRecoveryRequest {
    BulkRecoveryRequest::for_scope(environment, workers, limits())
}

// ---------------------------------------------------------------------------------------------
// The recording sink
// ---------------------------------------------------------------------------------------------

/// What a [`Recorder`] does once it has answered its callbacks.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Behaviour {
    /// Take the whole stream.
    #[default]
    Continue,
    /// Confirm the first `n` method records and answer [`SinkControl::Stop`] on the record after
    /// them: the confirmed prefix stands, and the record that stopped the stream was refused by this
    /// consumer rather than counted by it.
    StopAt(u64),
    /// Confirm the first `n` method records and return an I/O failure for the record after them.
    /// The failure's code is `bulk_test_sink_failure`, so a report that located it can be checked
    /// for it by name.
    FailAt(u64),
}

/// The test's own observation of one recorded event, run on the coordinator thread inside the
/// callback that received it. It is how a target watches the stream — counting live classes,
/// sleeping over a record, cancelling the operation — without a second thread of its own.
pub type SinkHook = Box<dyn FnMut(&Recorded)>;

/// The bytes one delivered result **owns**, as this module's own reading of the same public surface the
/// library reads them from: the text's own buffer, the source map's segment table, and the tables the two
/// reports hold — each by its **capacity** where the surface hands over a table this process owns, and by
/// its length where it hands over a slice.
///
/// It is deliberately a *subset* of the library's own weight model (no struct frames, no member
/// identities, no declaration facts, no strings inside the record types the facade does not expose), so a
/// case can state "the weight covers what a result owns" without restating the library's formula.
pub fn owned_bytes(recovered: &RecoveredMethod) -> u64 {
    fn buffer<T>(values: &Vec<T>) -> u64 {
        u64::try_from(values.capacity())
            .unwrap_or(u64::MAX)
            .saturating_mul(u64::try_from(std::mem::size_of::<T>()).unwrap_or(u64::MAX))
    }
    fn slice<T>(values: &[T]) -> u64 {
        u64::try_from(values.len())
            .unwrap_or(u64::MAX)
            .saturating_mul(u64::try_from(std::mem::size_of::<T>()).unwrap_or(u64::MAX))
    }
    fn strings(values: &Vec<String>) -> u64 {
        let mut bytes = buffer(values);
        for value in values {
            bytes = bytes.saturating_add(u64::try_from(value.capacity()).unwrap_or(u64::MAX));
        }
        bytes
    }
    fn diagnostics(values: &Vec<Diagnostic>) -> u64 {
        let mut bytes = buffer(values);
        for value in values {
            bytes = bytes
                .saturating_add(u64::try_from(value.code.capacity()).unwrap_or(u64::MAX))
                .saturating_add(u64::try_from(value.message.capacity()).unwrap_or(u64::MAX));
        }
        bytes
    }

    let recovery = recovered.recovery();
    let analysis = recovered.analysis();
    let mut bytes = u64::try_from(recovery.text.capacity()).unwrap_or(u64::MAX);
    bytes = bytes
        .saturating_add(slice(recovery.source_map.segments()))
        .saturating_add(u64::try_from(recovery.method.capacity()).unwrap_or(u64::MAX))
        .saturating_add(strings(&recovery.aliased_names))
        .saturating_add(diagnostics(&recovery.diagnostics))
        .saturating_add(buffer(&recovery.rules))
        .saturating_add(buffer(&recovery.regions))
        .saturating_add(buffer(&recovery.lambdas))
        .saturating_add(buffer(&recovery.concats))
        .saturating_add(buffer(&recovery.accessors))
        .saturating_add(buffer(&recovery.bridges))
        .saturating_add(buffer(&recovery.news))
        .saturating_add(buffer(&recovery.fields))
        .saturating_add(buffer(&recovery.enum_switches))
        .saturating_add(buffer(&recovery.fallbacks))
        .saturating_add(buffer(&analysis.requested_stages))
        .saturating_add(buffer(&analysis.stages))
        .saturating_add(buffer(&analysis.reads))
        .saturating_add(buffer(&analysis.environment_problems))
        .saturating_add(diagnostics(&analysis.diagnostics));
    for lambda in &recovery.lambdas {
        for owned in [
            lambda.bootstrap.as_ref(),
            lambda.sam_method_type.as_ref(),
            lambda.instantiated_method_type.as_ref(),
            lambda.implementation.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            bytes = bytes.saturating_add(u64::try_from(owned.capacity()).unwrap_or(u64::MAX));
        }
        bytes = bytes
            .saturating_add(u64::try_from(lambda.sam_name.capacity()).unwrap_or(u64::MAX))
            .saturating_add(u64::try_from(lambda.sam_descriptor.capacity()).unwrap_or(u64::MAX))
            .saturating_add(buffer(&lambda.captures));
    }
    bytes
}

/// The figure the older weight model charged for one result: the text's **length** plus a fixed cost per
/// retained record.
///
/// It is kept beside [`owned_bytes`] as the *proxy* a case compares against: a result owns far more than
/// this, which is exactly what a model that charged the proxy would miss.
pub fn proxy_bytes(recovered: &RecoveredMethod) -> u64 {
    const RECORD_WEIGHT: u64 = 64;
    let recovery = recovered.recovery();
    let analysis = recovered.analysis();
    let records = recovery.regions.len()
        + recovery.lambdas.len()
        + recovery.concats.len()
        + recovery.accessors.len()
        + recovery.bridges.len()
        + recovery.news.len()
        + recovery.fields.len()
        + recovery.enum_switches.len()
        + recovery.fallbacks.len()
        + recovery.aliased_names.len()
        + recovery.rules.len()
        + recovery.diagnostics.len()
        + recovery.source_map.len()
        + analysis.diagnostics.len()
        + analysis.stages.len()
        + recovered.callees().map_or(0, |read| read.members().len());
    u64::try_from(recovery.text.len())
        .unwrap_or(u64::MAX)
        .saturating_add(RECORD_WEIGHT.saturating_mul(u64::try_from(records).unwrap_or(u64::MAX)))
}

/// One method record, as the sink received it and as the targets compare it.
///
/// The library's own [`MethodResultEvent`] holds the whole delivery; this is the projection the
/// stream's *observable* result is compared through: identity, disposition, text, weight and the
/// reason a result was refused. It is what a consumer can see, and a fingerprint over it is what two
/// runs must agree on.
#[derive(Clone, Debug)]
pub struct MethodRecord {
    /// The delivering class's position in the stream.
    pub class_ordinal: u64,
    /// The member record's position in its class's declaration order.
    pub member_ordinal: u64,
    /// The member's physical identity.
    pub method: PhysicalMethodId,
    /// The disposition this record counts in.
    pub outcome: BulkMethodOutcome,
    /// The Java text this record carried, and `None` when it carried none: a declaration without a
    /// body, a refusal and a result the window discarded all carry no text.
    pub text: Option<String>,
    /// The retention weight the record asked the window for; zero for every record that retains no
    /// artifact.
    pub weight: u64,
    /// For a result the fixed per-result ceiling refused: the weight it was accounted at and the
    /// ceiling it exceeded.
    pub oversized: Option<(u64, u64)>,
    /// Why the run's own presentation stopped, when it did: a method's local allowance, a shape the
    /// recovery refuses, an incomplete table — the reason the artifact is not there.
    pub stop_reason: Option<StopReason>,
    /// The run's own diagnostic codes, without their messages.
    pub diagnostic_codes: Vec<String>,
    /// What this result owns, as this module reads it from the public surface: see [`owned_bytes`].
    pub owned_bytes: u64,
    /// The proxy the older weight model charged for the same result: see [`proxy_bytes`].
    pub proxy_bytes: u64,
    /// The semantic fingerprint of this record: its identity, content and disposition, with every
    /// resource reading left out.
    pub fingerprint: Fingerprint,
}

impl MethodRecord {
    /// The record's place in the stream: its class's ordinal, its own member ordinal, and the
    /// physical identity of the member.
    pub fn key(&self) -> (u64, u64, PhysicalMethodId) {
        (self.class_ordinal, self.member_ordinal, self.method.clone())
    }

    /// Projects one delivered event onto the fields the stream is compared through.
    fn of(event: &MethodResultEvent) -> Self {
        let text = match &event.delivery {
            MethodDelivery::Recovered(recovered) => {
                let text = recovered.recovery().text.clone();
                if text.is_empty() { None } else { Some(text) }
            }
            _ => None,
        };
        let oversized = match &event.delivery {
            MethodDelivery::Oversized { weight, limit, .. } => Some((*weight, *limit)),
            _ => None,
        };
        // The run's own stop, when it states one: the reason a method has no artifact is part of what
        // the stream says about it, so it is compared with the rest of the record.
        let stop_reason = match &event.delivery {
            MethodDelivery::Recovered(recovered) => recovered.recovery().outcome.stop().cloned(),
            _ => None,
        };
        let diagnostic_codes = match &event.delivery {
            MethodDelivery::Refused { diagnostics, .. } => {
                diagnostics.iter().map(|item| item.code.clone()).collect()
            }
            MethodDelivery::Oversized {
                diagnostic_codes, ..
            } => diagnostic_codes.clone(),
            _ => Vec::new(),
        };
        let mut fields = Fields::new();
        fields.number(event.class_ordinal);
        fields.number(event.member_ordinal);
        fields.structured(&event.method);
        fields.structured(&event.outcome());
        fields.bytes(text.as_deref().unwrap_or("").as_bytes());
        fields.number(event.weight);
        if let Some((weight, limit)) = oversized {
            fields.number(weight);
            fields.number(limit);
        }
        if let Some(reason) = &stop_reason {
            fields.structured(reason);
        }
        for code in &diagnostic_codes {
            fields.text(code);
        }
        for plane in planes(&event.delivery) {
            fields.termination(plane);
        }
        let fingerprint = fields.done();
        let (owned_bytes, proxy_bytes) = match &event.delivery {
            MethodDelivery::Recovered(recovered) => {
                (owned_bytes(recovered), proxy_bytes(recovered))
            }
            _ => (0, 0),
        };
        Self {
            owned_bytes,
            proxy_bytes,
            class_ordinal: event.class_ordinal,
            member_ordinal: event.member_ordinal,
            method: event.method.clone(),
            outcome: event.outcome(),
            text,
            weight: event.weight,
            oversized,
            stop_reason,
            diagnostic_codes,
            fingerprint,
        }
    }
}

/// Every execution plane one delivery states: the run's own and the presentation's, in that order.
///
/// The same planes [`MethodDelivery::execution_incomplete`] reads, taken here so a fingerprint can
/// state each of them without reading their usage.
fn planes(delivery: &MethodDelivery) -> Vec<&ExecutionReport> {
    match delivery {
        MethodDelivery::Recovered(recovered) => vec![
            &recovered.analysis().execution,
            &recovered.recovery().execution,
        ],
        MethodDelivery::NoBody { analysis } => vec![&analysis.execution],
        MethodDelivery::Refused { execution, .. } | MethodDelivery::Oversized { execution, .. } => {
            vec![execution]
        }
    }
}

/// One event, kept in the order the sink received it.
///
/// The terminal event is boxed because its summary is the largest payload of the stream and every
/// other event would otherwise be padded to its size.
#[derive(Clone, Debug)]
pub enum Recorded {
    Header(BulkHeaderEvent),
    ClassPrepared(ClassPreparedEvent),
    Method(MethodRecord),
    ClassEnd(ClassEndEvent),
    Diagnostic(BulkDiagnosticEvent),
    Final(Box<BulkFinalEvent>),
}

/// A [`RecoverySink`] that records instead of writing.
///
/// Every callback is answered on the coordinator thread, keeps its event, runs [`Recorder::hook`] on
/// it, waits [`Recorder::delay`] and then answers [`Recorder::method_behaviour`]. Nothing here is
/// `Send` or `Sync` and nothing here needs to be: the trait asks for neither.
pub struct Recorder {
    /// Every event the sink was handed, in the order it was handed over.
    pub events: Vec<Recorded>,
    /// The test's own observation of each event, run inside the callback that received it.
    pub hook: Option<SinkHook>,
    /// How long this consumer takes to answer each callback: a consumer slower than its producers.
    pub delay: Duration,
    /// What this consumer does after it has confirmed its method records.
    pub method_behaviour: Behaviour,
    /// The operation's output account, as the header handed it over.
    pub delivery: Option<DeliveryAccount>,
    /// How many method records this sink has been handed.
    methods_seen: u64,
}

impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}

impl Recorder {
    /// A consumer that takes the whole stream and answers at once.
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            hook: None,
            delay: Duration::ZERO,
            method_behaviour: Behaviour::Continue,
            delivery: None,
            methods_seen: 0,
        }
    }

    /// The header event, when the stream published one.
    pub fn header(&self) -> Option<&BulkHeaderEvent> {
        self.events.iter().find_map(|event| match event {
            Recorded::Header(header) => Some(header),
            _ => None,
        })
    }

    /// Every prepared class, in the order they were delivered.
    pub fn prepared(&self) -> Vec<&ClassPreparedEvent> {
        self.events
            .iter()
            .filter_map(|event| match event {
                Recorded::ClassPrepared(prepared) => Some(prepared),
                _ => None,
            })
            .collect()
    }

    /// Every method record, in the order they were delivered.
    pub fn methods(&self) -> Vec<MethodRecord> {
        self.events
            .iter()
            .filter_map(|event| match event {
                Recorded::Method(method) => Some(method.clone()),
                _ => None,
            })
            .collect()
    }

    /// Every class end, in the order they were delivered.
    pub fn class_ends(&self) -> Vec<&ClassEndEvent> {
        self.events
            .iter()
            .filter_map(|event| match event {
                Recorded::ClassEnd(end) => Some(end),
                _ => None,
            })
            .collect()
    }

    /// Every located diagnostic, in the order they were delivered.
    pub fn diagnostics(&self) -> Vec<&BulkDiagnosticEvent> {
        self.events
            .iter()
            .filter_map(|event| match event {
                Recorded::Diagnostic(diagnostic) => Some(diagnostic),
                _ => None,
            })
            .collect()
    }

    /// The terminal event, when the stream reached it.
    pub fn final_event(&self) -> Option<&BulkFinalEvent> {
        self.events.iter().find_map(|event| match event {
            Recorded::Final(final_event) => Some(final_event.as_ref()),
            _ => None,
        })
    }

    /// Every event that belongs to one class, in stream order: its `ClassPrepared` (or the
    /// diagnostics published at it), its method records and its `ClassEnd`.
    pub fn class_stream(&self, class_ordinal: u64) -> Vec<&Recorded> {
        self.events
            .iter()
            .filter(|event| match event {
                Recorded::ClassPrepared(prepared) => prepared.class_ordinal == class_ordinal,
                Recorded::Method(method) => method.class_ordinal == class_ordinal,
                Recorded::ClassEnd(end) => end.class_ordinal == class_ordinal,
                Recorded::Diagnostic(diagnostic) => diagnostic.class_ordinal == Some(class_ordinal),
                Recorded::Header(_) | Recorded::Final(_) => false,
            })
            .collect()
    }

    /// Records one event and answers it.
    fn take(&mut self, recorded: Recorded) -> Result<SinkControl> {
        let method_seen = match &recorded {
            Recorded::Method(_) => {
                self.methods_seen = self.methods_seen.saturating_add(1);
                self.methods_seen
            }
            _ => 0,
        };
        self.events.push(recorded);
        if let Some(hook) = self.hook.as_mut() {
            hook(self.events.last().expect("the event was just recorded"));
        }
        if !self.delay.is_zero() {
            std::thread::sleep(self.delay);
        }
        match self.method_behaviour {
            Behaviour::Continue => Ok(SinkControl::Continue),
            Behaviour::StopAt(confirmed) if method_seen > confirmed => Ok(SinkControl::Stop),
            Behaviour::FailAt(confirmed) if method_seen > confirmed => Err(Error::Io {
                operation: "bulk_test_sink_failure".to_owned(),
                message: format!(
                    "the test's consumer confirmed {confirmed} method record(s) and stopped being \
                     writable"
                ),
            }),
            _ => Ok(SinkControl::Continue),
        }
    }
}

impl RecoverySink for Recorder {
    fn header(
        &mut self,
        event: &BulkHeaderEvent,
        delivery: DeliveryAccount,
    ) -> Result<SinkControl> {
        // The account travels with the header; a recording sink keeps it so a test can state what the
        // operation still allowed at the moment a record was handed over.
        self.delivery = Some(delivery);
        self.take(Recorded::Header(event.clone()))
    }

    fn class_prepared(&mut self, event: &ClassPreparedEvent) -> Result<SinkControl> {
        self.take(Recorded::ClassPrepared(event.clone()))
    }

    fn method(&mut self, event: &MethodResultEvent) -> Result<SinkControl> {
        self.take(Recorded::Method(MethodRecord::of(event)))
    }

    fn class_end(&mut self, event: &ClassEndEvent) -> Result<SinkControl> {
        self.take(Recorded::ClassEnd(event.clone()))
    }

    fn diagnostic(&mut self, event: &BulkDiagnosticEvent) -> Result<SinkControl> {
        self.take(Recorded::Diagnostic(event.clone()))
    }

    fn final_event(&mut self, event: &BulkFinalEvent) -> Result<SinkControl> {
        self.take(Recorded::Final(Box::new(event.clone())))
    }
}

// ---------------------------------------------------------------------------------------------
// Fingerprints
// ---------------------------------------------------------------------------------------------

/// A digest of one value's semantic fields.
///
/// It answers one question: do two runs state the same thing? It is deliberately not a hash of the
/// value: the readings a run produces about *itself* — the usage and elapsed figures of every
/// execution plane — are left out, because a one-worker run and a four-worker run are allowed to
/// differ there and nowhere else.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Fingerprint(u64);

/// The accumulator a [`Fingerprint`] is built from: FNV-1a over the field bytes, so the digest is a
/// function of the fields in the order they are fed and of nothing else.
struct Fields(u64);

impl Fields {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }

    fn bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
        // A field boundary, so that `["ab"]` and `["a", "b"]` are different fields.
        self.0 ^= 0xff;
        self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
    }

    fn text(&mut self, text: &str) {
        self.bytes(text.as_bytes());
    }

    fn number(&mut self, value: u64) {
        self.bytes(&value.to_be_bytes());
    }

    /// Feeds one structural value through its `Debug` rendering: identities, enums and reason codes
    /// are structure, and this renders every field of that structure without reading anything about
    /// the run that produced it.
    fn structured(&mut self, value: &impl std::fmt::Debug) {
        self.text(&format!("{value:?}"));
    }

    /// Feeds what an execution plane *states*, not what it *spent*: its state and, for a stop, the
    /// reason. The usage and elapsed readings are not part of a result.
    fn termination(&mut self, execution: &ExecutionReport) {
        match execution {
            ExecutionReport::Complete { .. } => self.text("complete"),
            ExecutionReport::Partial { reason, .. } => {
                self.text("partial");
                self.structured(reason);
            }
            ExecutionReport::Cancelled { .. } => self.text("cancelled"),
            ExecutionReport::Failed { reason, .. } => {
                self.text("failed");
                self.structured(reason);
            }
        }
    }

    fn done(self) -> Fingerprint {
        Fingerprint(self.0)
    }
}

/// The semantic fingerprint of one bulk summary.
///
/// What it reads: the view, the declared semantic configuration (the three capacities, the
/// per-method limits and the facts capacity), every discovery/execution/delivery count, every
/// per-outcome bucket, whether the traversal reached the end, and the aggregate state with the
/// reason it stopped for. What it does not read: the worker counts (the caller's scheduling
/// parameter, which the targets remove before comparing), the shared pool's capacity and its
/// high-water mark (both derived from the effective worker count, so a fingerprint that read them
/// would read the scheduling parameter back in), the operation's usage, the entry usage and the
/// elapsed time.
pub fn fingerprint(summary: &BulkSummary) -> Fingerprint {
    let mut fields = Fields::new();
    fields.structured(&summary.view);
    fields.structured(&summary.limits.method);
    fields.number(summary.limits.max_class_bytes);
    fields.number(summary.limits.max_result_weight);
    fields.number(summary.limits.max_buffered_result_weight);
    fields.structured(&summary.limits.facts_capacity);
    fields.number(summary.classes_seen);
    fields.number(summary.classes_prepared);
    fields.number(summary.classes_refused);
    fields.number(summary.methods_declared);
    fields.number(summary.methods_executed);
    fields.number(summary.methods_delivered);
    fields.number(summary.methods_not_executed);
    fields.number(summary.outcomes.produced);
    fields.number(summary.outcomes.explanation_only);
    fields.number(summary.outcomes.not_produced);
    fields.number(summary.outcomes.refused);
    fields.number(summary.outcomes.no_body);
    fields.number(summary.outcomes.oversized);
    fields.number(u64::from(summary.traversal_complete));
    fields.termination(&summary.execution);
    fields.done()
}
