//! D0 1.3's counting gates: what the demand paths of this change really do, counted.
//!
//! # What this target is for
//!
//! The D0 phase of `add-demand-driven-core-results` freezes the *work* the later phases must lower
//! and states it as numbers a gate can fail on. Two kinds of number are read here:
//!
//! * **the port** ([`jarde::d0_counts`], a bounded test-support-only counter set): class
//!   materializations, class preparations, body decodes, recovery presentations and the owning
//!   records this facade's own publications built. It is incremented where the work happens, so a
//!   path that does more work than it should says so instead of hiding behind a charge;
//! * **the existing public surface** for the three events whose sites live in layers this round
//!   freezes: consumer work (`QueryReport::coverage.scanned_items` and the usage deltas of the same
//!   request), the optional owning records a recovery report published (per category), and release
//!   (`Arc::strong_count` of the reader's own `PreparedClass::facts_handle`).
//!
//! Nothing here needs a JDK, a network or a writable directory: every fixture is a committed class
//! file read into memory, and the one archive is written into memory by the repository's own
//! `rawzip` dev-dependency.
//!
//! # The frozen numbers
//!
//! Each assertion states the number this revision really produces, named as the D-phase task it
//! belongs to: a later phase that changes the work must change this file *deliberately*, with the
//! new number recorded in the change's verification.
//!
//! **Re-frozen by D2 (3.1–3.3).** The D0 baseline this file was written with — a class-source
//! request materializing its class twice (the binding read and a second read for the preparation)
//! and preparing it once, and a class view decoding a body without preparing the class it decoded
//! it from — is exactly what D2 removed: the selected definition is now materialized once, the one
//! preparation is built over *that* read, and a view that decodes a body prepares the class once
//! for every body it asked for. The figures below are that shape, and the old ones are recorded in
//! the change's verification beside them.

#![cfg(feature = "test-support")]

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::sync::{Arc, Mutex, MutexGuard};

/// A real compiled sample with eight members, every one of them declaring a body.
const SCOPE: &[u8] = include_bytes!("fixtures/p3-scope/v8/Scope.class");

/// A second real class, so the query scope below holds more than one unit.
const SHAPE: &[u8] = include_bytes!("fixtures/p3-declaration/v8/Shape.class");

/// Every dimension a request here uses, bounded generously: this target measures *what* a request
/// did, never how close it came to a limit.
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
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 4,
        dependency_depth: 8,
        elapsed_millis: 60_000,
    }
}

fn fresh_budget() -> Budget {
    Budget::new(limits())
}

/// The counting tests of this binary run one at a time: the port is one process-wide set of
/// counters, so two tests counting at once would each read the other's work.
static GATE: Mutex<()> = Mutex::new(());

fn gate() -> MutexGuard<'static, ()> {
    GATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// One standalone class snapshot, opened from bytes this process already holds.
fn open(bytes: &[u8]) -> ArtifactSnapshot {
    let mut budget = fresh_budget();
    ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("the committed fixture is a readable class file")
}

/// The physical definition of that class, as the operations below name it: the one class the
/// snapshot holds, read through the public listing entry (never by re-deriving an identity here).
fn definition_of(engine: &Engine, snapshot: &ArtifactSnapshot) -> PhysicalDefinitionId {
    let mut budget = fresh_budget();
    let listing = engine
        .list_class_declarations(snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the fixture's one class is listed");
    listing
        .items
        .first()
        .expect("the standalone snapshot holds one class")
        .definition
        .clone()
}

/// The scope-wide view of a fresh snapshot: the whole artifact, from its own root.
fn tree_scope() -> PhysicalScope {
    PhysicalScope::ArtifactTree {
        root_container: ContainerId("root".to_owned()),
    }
}

/// A stored-only archive, built with the repository's own `rawzip` dev-dependency: the fixture is
/// written into memory, so no test here needs a file system.
fn zip_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    const STORE: u16 = 0;
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .expect("the fixture entry starts");
            let mut writer = config.wrap(&mut entry);
            writer
                .write_all(data)
                .expect("the fixture entry is written");
            let (_, descriptor) = writer.finish().expect("the fixture entry closes");
            entry
                .finish(descriptor)
                .expect("the fixture entry finishes");
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

/// The declaration of one class-source request over a physical identity: the "explicit identity"
/// shape D02's gate is about, never a name search.
fn class_source_request(
    snapshot: &ArtifactSnapshot,
    definition: PhysicalDefinitionId,
) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Definition { definition },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    }
}

/// How many owning records each optional category of one recovery report published.
///
/// This is the D3/D4 baseline: at the frozen revision every category is published by the default
/// request, and the change's target is that an unrequested category publishes none.
#[derive(Debug, Default, Eq, PartialEq)]
struct OptionalRecords {
    regions: usize,
    lambdas: usize,
    concats: usize,
    accessors: usize,
    bridges: usize,
    news: usize,
    fields: usize,
    enum_switches: usize,
    init: usize,
    segments: usize,
    rules: usize,
    fallbacks: usize,
    aliased_names: usize,
    diagnostics: usize,
}

impl OptionalRecords {
    fn of(report: &RecoveryReport) -> Self {
        Self {
            regions: report.regions.len(),
            lambdas: report.lambdas.len(),
            concats: report.concats.len(),
            accessors: report.accessors.len(),
            bridges: report.bridges.len(),
            news: report.news.len(),
            fields: report.fields.len(),
            enum_switches: report.enum_switches.len(),
            init: usize::from(report.init.is_some()),
            segments: report.source_map.len(),
            rules: report.rules.len(),
            fallbacks: report.fallbacks.len(),
            aliased_names: report.aliased_names.len(),
            diagnostics: report.diagnostics.len(),
        }
    }

    /// Every published record of the **optional** categories, across categories.
    ///
    /// `fallbacks` and `diagnostics` are necessary results — every selection delivers the fallback
    /// codes and the gaps this run states — so they are counted beside the optional tables rather
    /// than inside them: what an unrequested category must not contribute is a *record*.
    fn optional_total(&self) -> usize {
        self.total() - self.fallbacks - self.diagnostics
    }

    /// Every published record, across categories.
    fn total(&self) -> usize {
        self.regions
            + self.lambdas
            + self.concats
            + self.accessors
            + self.bridges
            + self.news
            + self.fields
            + self.enum_switches
            + self.init
            + self.segments
            + self.rules
            + self.fallbacks
            + self.aliased_names
            + self.diagnostics
    }
}

/// One test's own numbers, printed as one line so a failure is read beside the count that moved.
fn report_counts(label: &str, counted: d0_counts::Counts, usage: &UsageSnapshot) {
    println!(
        "{label}: class_materializations={} class_preparations={} body_decodes={} \
         recovery_runs={} owned_records={} | class_headers={} method_bodies={} code_bytes={} \
         ir_items={} analysis_steps={} result_items={} output_bytes={}",
        counted.class_materializations,
        counted.class_preparations,
        counted.body_decodes,
        counted.recovery_runs,
        counted.owned_records,
        usage.class_headers,
        usage.method_bodies,
        usage.code_bytes,
        usage.ir_items,
        usage.analysis_steps,
        usage.result_items,
        usage.output_bytes,
    );
}

#[test]
fn a_member_only_read_decodes_no_body_and_builds_no_record() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definition_of(&engine, &snapshot);
    let mut budget = fresh_budget();

    // 1. The declaration-only read: one class header, one member walk, and no body at all.
    let before = d0_counts::snapshot();
    let listing = engine
        .list_members(&snapshot, &definition, &mut budget)
        .expect("the fixture's member table is read");
    let counted = before.since(d0_counts::snapshot());
    let usage = budget.usage();
    report_counts("list_members", counted, &usage);
    assert!(
        counted.is_silent(),
        "a member listing prepares nothing, decodes nothing and builds no record"
    );
    assert_eq!(listing.methods().count(), 8);
    assert_eq!(usage.class_headers, 1);
    assert_eq!(usage.method_bodies, 0);
    assert_eq!(usage.code_bytes, 0);
    assert_eq!(usage.ir_items, 0);
    assert_eq!(usage.analysis_steps, 0);

    // 2. The class view with no requested body: exactly the same work, on the composed entry.
    let mut budget = fresh_budget();
    let before = d0_counts::snapshot();
    let view = engine
        .class_view(
            &snapshot,
            &PhysicalScope::SnapshotAll,
            &ClassViewRequest {
                class: ClassRef::Definition {
                    definition: definition.clone(),
                },
                bodies: Vec::new(),
            },
            &mut budget,
        )
        .expect("the class view is answered");
    let counted = before.since(d0_counts::snapshot());
    let usage = budget.usage();
    report_counts("class_view (no body)", counted, &usage);
    let OperationOutcome::Performed(view) = view else {
        panic!("the identity binds one definition");
    };
    assert!(view.bodies.is_empty());
    assert_eq!(counted.class_materializations, 1);
    assert_eq!(counted.class_preparations, 0);
    assert_eq!(counted.body_decodes, 0);
    assert_eq!(counted.recovery_runs, 0);
    assert_eq!(counted.owned_records, 0);
    assert_eq!(usage.class_headers, 1);
    assert_eq!(usage.method_bodies, 0);
    assert_eq!(usage.ir_items, 0);
    assert_eq!(usage.analysis_steps, 0);
    assert_eq!(usage.normalization_clones, 0);
}

#[test]
fn one_requested_body_decodes_one_body_and_runs_no_recovery() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definition_of(&engine, &snapshot);
    let mut budget = fresh_budget();
    let before = d0_counts::snapshot();
    let view = engine
        .class_view(
            &snapshot,
            &PhysicalScope::SnapshotAll,
            &ClassViewRequest {
                class: ClassRef::Definition { definition },
                bodies: vec![BodyRef::Name {
                    name: JvmBytes(b"simple".to_vec()),
                    descriptor: None,
                }],
            },
            &mut budget,
        )
        .expect("the class view is answered");
    let counted = before.since(d0_counts::snapshot());
    let usage = budget.usage();
    report_counts("class_view (one body)", counted, &usage);
    let OperationOutcome::Performed(view) = view else {
        panic!("the identity binds one definition");
    };
    assert_eq!(view.bodies.len(), 1);
    assert!(
        matches!(view.bodies[0], ClassViewBody::Read { .. }),
        "the requested member's body is decoded, not refused"
    );
    assert_eq!(counted.class_materializations, 1);
    // D2 3.2: the body this view decodes is decoded against **one** preparation of the class the
    // binding read — the same read, not a second one — so the class is materialized once and
    // prepared once, whatever the number of requested bodies.
    assert_eq!(counted.class_preparations, 1);
    assert_eq!(counted.body_decodes, 1);
    assert_eq!(counted.recovery_runs, 0);
    assert!(counted.owned_records > 0);
    assert_eq!(usage.class_headers, 1);
    assert_eq!(usage.method_bodies, 1);
    assert_eq!(usage.ir_items, 0);
    assert_eq!(usage.analysis_steps, 0);
    println!("class_view owned_records = {}", counted.owned_records);
}

/// D02's shape, re-frozen by D2, and the D1 evidence selection re-frozen beside it: one
/// materialization of the selected class, one preparation over that same read, one body decode per
/// member that declares a body — and, in the ordinary request, **not one** owning detail record.
///
/// The D0 baseline this test was written with said two materializations (`class_headers == 2`: the
/// binding read and the preparation's own read) and one preparation, and it asserted that "the
/// frozen revision's default request publishes every optional table". D2 3.1/3.2 removed the second
/// materialization; D1 made the ordinary request select no optional evidence at all, so the assertion
/// is re-frozen the other way: the default publishes *none* of them, and the same request with
/// [`RecoveryEvidenceRequest::all`] publishes them with the same text and the same decisions.
#[test]
fn a_class_source_request_materializes_its_class_once_and_prepares_it_once() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definition_of(&engine, &snapshot);
    let mut budget = fresh_budget();
    let before = d0_counts::snapshot();
    let outcome = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &class_source_request(&snapshot, definition.clone()),
            &mut budget,
        )
        .expect("the class-source request is answered");
    let counted = before.since(d0_counts::snapshot());
    let usage = budget.usage();
    report_counts("class_source", counted, &usage);
    let OperationOutcome::Performed(report) = outcome else {
        panic!("the identity binds one definition");
    };
    assert_eq!(report.methods.len(), 8, "every member is presented");
    // D2 3.1/3.2, the figures this test re-freezes: the selected definition is materialized **once**
    // — the binding's own read, over which the one preparation is built — and the request charges
    // one class header for it. The D0 baseline was two of each (the binding read and a second read
    // for the preparation), which is the shape D2 removed.
    assert_eq!(usage.class_headers, 1);
    assert_eq!(counted.class_materializations, 1);
    assert_eq!(counted.class_preparations, 1);
    // One decode and one presentation per member that declares a body.
    assert_eq!(counted.body_decodes, 8);
    assert_eq!(counted.recovery_runs, 8);
    assert_eq!(usage.method_bodies, 8);

    // D1's own re-freeze: the ordinary request publishes **none** of the optional categories, and
    // the request that states the full selection publishes them with the same text and the same
    // decisions. "Nothing is constructed and then hidden" is what the two runs together say: the
    // constructions of the unselected run are counted where they would happen.
    let published: Vec<OptionalRecords> = report
        .methods
        .iter()
        .filter_map(|method| match &method.outcome {
            ClassSourceOutcome::Recovered { report, .. } => Some(OptionalRecords::of(report)),
            _ => None,
        })
        .collect();
    assert_eq!(published.len(), 8);
    let total: usize = published.iter().map(OptionalRecords::optional_total).sum();
    println!("class_source default selection: optional records={total} per-member={published:?}");
    assert_eq!(
        total, 0,
        "the ordinary request materializes no optional detail record at all"
    );
    for method in &report.methods {
        if let ClassSourceOutcome::Recovered { report, .. } = &method.outcome {
            for kind in RecoveryEvidenceKind::SUPPORTED {
                assert_eq!(
                    report.evidence.state(kind),
                    EvidenceState::NotRequested,
                    "the report states the selection it was presented under"
                );
            }
        }
    }

    // The same request, with the full selection: every category the fixture's members really have is
    // materialized, and the class text is what the ordinary request produced — byte for byte.
    let mut detailed_budget = fresh_budget();
    let before = d0_counts::snapshot();
    let detailed = engine
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &class_source_request(&snapshot, definition),
            &RecoveryEvidenceRequest::all(),
            &mut detailed_budget,
        )
        .expect("the same request with an explicit selection is answered");
    let detailed_counts = before.since(d0_counts::snapshot());
    let OperationOutcome::Performed(detailed) = detailed else {
        panic!("the identity binds one definition");
    };
    assert_eq!(
        detailed.text, report.text,
        "the evidence selection does not move one byte of the assembled class"
    );
    assert_eq!(detailed_counts.class_materializations, 1);
    assert_eq!(detailed_counts.class_preparations, 1);
    assert_eq!(detailed_counts.body_decodes, 8);
    assert_eq!(detailed_counts.recovery_runs, 8);
    let detailed_published: Vec<OptionalRecords> = detailed
        .methods
        .iter()
        .filter_map(|method| match &method.outcome {
            ClassSourceOutcome::Recovered { report, .. } => Some(OptionalRecords::of(report)),
            _ => None,
        })
        .collect();
    assert_eq!(detailed_published.len(), 8);
    for (default, full) in published.iter().zip(detailed_published.iter()) {
        assert_eq!(
            (default.diagnostics, default.fallbacks, default.lambdas,),
            (full.diagnostics, full.fallbacks, full.lambdas,),
            "the necessary results and the gaps do not depend on the selection"
        );
    }
    let regions: usize = detailed_published.iter().map(|r| r.regions).sum();
    let segments: usize = detailed_published.iter().map(|r| r.segments).sum();
    let rules: usize = detailed_published.iter().map(|r| r.rules).sum();
    println!("class_source full selection: regions={regions} segments={segments} rules={rules}");
    assert!(
        regions >= 8 && segments >= 8 && rules >= 8,
        "the full selection publishes the categories the fixture really has"
    );
    assert_eq!(detailed_published.len(), detailed.methods.len());
    for method in detailed.methods.iter() {
        if let ClassSourceOutcome::Recovered { report, .. } = &method.outcome {
            for kind in RecoveryEvidenceKind::SUPPORTED {
                assert_eq!(
                    report.evidence.state(kind),
                    EvidenceState::Complete,
                    "{kind:?}"
                );
            }
        }
    }
}

#[test]
fn a_page_limited_query_publishes_one_item_and_decodes_the_whole_first_unit() {
    let _gate = gate();
    let engine = Engine::new();
    let archive = zip_of(&[(b"p/Scope.class", SCOPE), (b"p/Shape.class", SHAPE)]);
    let mut open_budget = fresh_budget();
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(archive), &mut open_budget)
        .expect("the fixture archive is readable");
    let scope = tree_scope();
    let request = |relation, target, consumers, max_items| QueryRequest {
        relation,
        target,
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: scope.clone(),
        },
        consumers: ConsumerSchema::new(1, consumers),
        max_items,
        cursor: None,
    };
    let object_class = || QueryTarget::Symbol {
        value: SymbolRef::Class {
            owner: JvmBytes(b"java/lang/Object".to_vec()),
        },
    };
    let object_constructor = || QueryTarget::Symbol {
        value: SymbolRef::Method {
            owner: JvmBytes(b"java/lang/Object".to_vec()),
            name: JvmBytes(b"<init>".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        },
    };

    // (a) The raw constant-pool probe: the entry-level view of what a page costs.
    let pool = |max_items| {
        request(
            QueryRelation::ConstantPoolContains,
            object_class(),
            vec![ConsumerKind::Type],
            max_items,
        )
    };
    let mut budget = fresh_budget();
    let full = engine
        .query(&snapshot, &pool(0), &mut budget)
        .expect("the whole query is answered");
    let full_usage = budget.usage();
    let mut budget = fresh_budget();
    let before = d0_counts::snapshot();
    let page = engine
        .query(&snapshot, &pool(1), &mut budget)
        .expect("the first page is answered");
    let counted = before.since(d0_counts::snapshot());
    let page_usage = budget.usage();
    println!(
        "pool probe full: items={} scanned_items={} archive_entries={} read_bytes={}",
        full.items.len(),
        full.coverage.scanned_items,
        full_usage.archive_entries,
        full_usage.read_bytes,
    );
    println!(
        "pool probe page: items={} scanned_items={} archive_entries={} read_bytes={} has_more={}",
        page.items.len(),
        page.coverage.scanned_items,
        page_usage.archive_entries,
        page_usage.read_bytes,
        page.page.has_more,
    );
    assert!(
        counted.is_silent(),
        "a query is not a class operation: it must not move the demand-path counters"
    );
    assert_eq!(
        usize::try_from(full.coverage.scanned_items).unwrap(),
        full.items.len(),
        "the whole query scans exactly the items it publishes"
    );
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.page.returned_items, 1);
    assert!(page.page.has_more, "the prefix is not the whole range");
    assert_eq!(page.coverage.scanned_items, 1);
    // The frozen D08 baseline at entry granularity: a one-item page stops after the unit that filled
    // it, so it visits fewer entries and reads fewer bytes than the whole range — and it still visits
    // more than the one item it published.
    assert!(
        page_usage.archive_entries < full_usage.archive_entries,
        "the page stops before the end of the range"
    );
    assert!(page_usage.read_bytes < full_usage.read_bytes);

    // (b) The invocation consumer: the unit-level view, and the boundary the query change moved. A
    // unit's body set used to be decoded before its items were known, so a page of one item paid for
    // every body of the unit it stopped in. The consumer now walks the unit method by method and
    // stops once the page is full, so the page pays only for the bodies up to the one that filled it
    // — and the continuation still covers the rest without repeating what was published.
    let invocations =
        |snapshot: &ArtifactSnapshot, scope: &PhysicalScope, max_items| QueryRequest {
            relation: QueryRelation::MentionsSymbol,
            target: object_constructor(),
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: scope.clone(),
            },
            consumers: ConsumerSchema::new(1, vec![ConsumerKind::Invocation]),
            max_items,
            cursor: None,
        };
    // The reference: the first unit alone, in a scope that holds nothing else. Its whole-range query
    // decodes exactly this unit's bodies, which is the number the next comparison reads.
    let unit = open(SCOPE);
    let mut budget = fresh_budget();
    let unit_full = engine
        .query(
            &unit,
            &invocations(&unit, &PhysicalScope::SnapshotAll, 0),
            &mut budget,
        )
        .expect("the unit's whole invocation query is answered");
    let unit_full_usage = budget.usage();
    let mut budget = fresh_budget();
    let unit_page = engine
        .query(
            &unit,
            &invocations(&unit, &PhysicalScope::SnapshotAll, 1),
            &mut budget,
        )
        .expect("the unit's first page is answered");
    let unit_page_usage = budget.usage();
    println!(
        "invocations one unit: items={} scanned_items={} code_bytes={} read_bytes={} has_more={} \
         | page: items={} scanned_items={} code_bytes={} read_bytes={} has_more={}",
        unit_full.items.len(),
        unit_full.coverage.scanned_items,
        unit_full_usage.code_bytes,
        unit_full_usage.read_bytes,
        unit_full.page.has_more,
        unit_page.items.len(),
        unit_page.coverage.scanned_items,
        unit_page_usage.code_bytes,
        unit_page_usage.read_bytes,
        unit_page.page.has_more,
    );
    assert_eq!(unit_full.items.len(), 1, "the unit holds one matching call");
    assert!(
        unit_page_usage.code_bytes < unit_full_usage.code_bytes,
        "a page of one item stops once that item is published instead of decoding every body of the \
         unit it stops in: {} byte(s) against the unit's {}",
        unit_page_usage.code_bytes,
        unit_full_usage.code_bytes
    );
    assert_eq!(
        unit_page.items.len(),
        unit_full.items.len(),
        "the page holds every match of the unit it stopped in"
    );
    // `has_more` is conservative and never invented: a page that stopped inside the unit states that
    // there may be more, because the suffix it did not walk is unknown rather than empty — the
    // continuation is what settles it, and for this one-unit scope it comes back with no items.
    assert!(
        unit_page.page.has_more,
        "a page that stopped inside its unit leaves the suffix unknown"
    );
    let continuation_cursor = unit_page
        .page
        .cursor
        .clone()
        .expect("a page that stopped inside its unit carries a cursor");
    let mut continuation_budget = fresh_budget();
    let continuation = engine
        .query(
            &unit,
            &QueryRequest {
                cursor: Some(continuation_cursor),
                ..invocations(&unit, &PhysicalScope::SnapshotAll, 1)
            },
            &mut continuation_budget,
        )
        .expect("the continuation of the one-unit page is answered");
    assert!(
        continuation.items.is_empty(),
        "the unit had one match and the page published it: {:?}",
        continuation.items
    );

    // The same page over both units stops before the second one: it reads no byte of it, and its
    // decode is exactly the first unit's.
    let mut budget = fresh_budget();
    let page = engine
        .query(&snapshot, &invocations(&snapshot, &scope, 1), &mut budget)
        .expect("the first page of the two-unit scope is answered");
    let page_usage = budget.usage();
    println!(
        "invocations two units, page: items={} scanned_items={} code_bytes={} read_bytes={} \
         has_more={}",
        page.items.len(),
        page.coverage.scanned_items,
        page_usage.code_bytes,
        page_usage.read_bytes,
        page.page.has_more,
    );
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.coverage.scanned_items, 1);
    assert!(
        page_usage.code_bytes <= unit_full_usage.code_bytes,
        "the page never pays for more bodies than the unit it stopped in holds: {} against {}",
        page_usage.code_bytes,
        unit_full_usage.code_bytes
    );
    assert!(
        page_usage.read_bytes < full_usage.read_bytes,
        "the page reads no byte of the unit after the one that filled it"
    );
    assert!(page.page.has_more, "the page stopped before the next unit");
    assert_eq!(page.page.cursor.is_some(), page.page.has_more);
}

#[test]
fn a_consumer_release_drops_the_prepared_facts_handle_back_to_its_baseline() {
    let _gate = gate();
    let snapshot = open(SCOPE);
    let mut budget = fresh_budget();
    let read = snapshot
        .prepared_root_class(&mut budget)
        .expect("the standalone class prepares");
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
        "the preparation's own ownership is released with it"
    );
    drop(read);
    assert_eq!(
        Arc::strong_count(&facts),
        1,
        "the read's release does not touch the facts handle the preparation handed out"
    );
    let line = format!(
        "release: facts strong_count after drop(prepared) = {}",
        Arc::strong_count(&facts)
    );
    drop(facts);
    println!("{line}");
}
