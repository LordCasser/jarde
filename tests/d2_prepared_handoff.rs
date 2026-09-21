//! D2's gates (tasks 3.1–3.4): one materialization, one preparation, and one body decode per body
//! an ordinary operation really decodes.
//!
//! # What this file counts, and why it counts rather than compares
//!
//! D2 is about *work*: the change hands the read a target binding already performed to the
//! preparation that consumes it, so the selected definition is materialized once instead of once per
//! consumer. A comparison of results cannot see that — two runs that read a class once and twice
//! publish the same answer — so every claim here is asserted as a count, taken from the D0
//! counting port ([`jarde::d0_counts`], test-support only, no report field, no fingerprint) and
//! from `usage`:
//!
//! | claim | the counter that decides it |
//! | --- | --- |
//! | the selected definition is materialized once | `class_materializations`, `usage.class_headers` |
//! | one preparation serves every body | `class_preparations` |
//! | exactly the bodies that were asked for are decoded | `body_decodes`, `usage.method_bodies` |
//! | a request holds nothing another request would be answered from | the same figures, twice over |
//!
//! The figures are D2's, and the shape D0 froze (two materializations for a class-source request,
//! no preparation for a class view) is what this file's assertions replaced; the change's
//! verification record holds both sides of that comparison, and the counterexamples (git-style
//! mutations that put the second read back and make these tests fail) are recorded there too.
//!
//! # What it does *not* cover
//!
//! The same-class callee half of task 3.3 needs a class whose body calls a synthetic accessor, and
//! that fixture lives with its own file: `tests/p3_accessor_edges.rs` carries the count gate for it,
//! because building a second copy of that class here would be a second fixture for one fact.

#![cfg(feature = "test-support")]

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

/// The committed 8-member sample every case here reads.
const SCOPE: &[u8] = include_bytes!("fixtures/p3-scope/v8/Scope.class");

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

/// The counting tests of this binary run one at a time: the port is one process-wide set of
/// counters, so two tests counting at once would each read the other's work.
static GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn gate() -> std::sync::MutexGuard<'static, ()> {
    GATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn budget() -> Budget {
    Budget::new(limits())
}

/// One standalone class snapshot, opened from bytes this process already holds.
fn open(bytes: &[u8]) -> ArtifactSnapshot {
    let mut budget = budget();
    ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("the committed fixture is a readable class file")
}

/// One stored-only archive of the given entries, built with the repository's own `rawzip`
/// dependency so no case here needs a file system.
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

/// The one physical definition of a standalone snapshot, read through the public listing entry.
fn definition_of(engine: &Engine, snapshot: &ArtifactSnapshot) -> PhysicalDefinitionId {
    let mut budget = budget();
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

/// The environment one standalone class is read under: its own root, searched with no prefix.
fn environment(snapshot: &ArtifactSnapshot, profile: RuntimeProfile) -> ResolutionEnvironment {
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
            profile,
            load_domain: domain.clone(),
        },
        domains: vec![domain],
        providers: Vec::new(),
    }
}

fn profile() -> RuntimeProfile {
    RuntimeProfile {
        java_release: 8,
        multi_release: MultiReleasePolicy::Disabled,
        layout: LayoutMode::Generic,
    }
}

fn environment_request(snapshot: &ArtifactSnapshot) -> EnvironmentRequest {
    EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope: PhysicalScope::SnapshotAll,
        policy: EnvironmentPolicy::SingleClass,
        profile: profile(),
        loader: LoaderId("app".to_string()),
    }
}

/// The same request over a ZIP snapshot, whose classes are read through an explicit classpath root:
/// the root is the snapshot's own container, searched with no prefix, exactly as the archive's
/// entries are addressed.
fn archive_environment_request(snapshot: &ArtifactSnapshot) -> EnvironmentRequest {
    let mut request = environment_request(snapshot);
    request.policy = EnvironmentPolicy::ExplicitClasspath {
        roots: vec![LoadRoot::Container {
            origin: ContainerOrigin {
                snapshot: snapshot.id().clone(),
                root_container: ContainerId("root".into()),
                steps: Vec::new(),
            },
            prefix: ArchiveNameBytes(Vec::new()),
        }],
    };
    request
}

/// One class-source request over a physical identity.
fn class_source_request(snapshot: &ArtifactSnapshot, class: ClassRef) -> ClassSourceRequest {
    ClassSourceRequest {
        class,
        environment: environment_request(snapshot),
    }
}

/// The same over a ZIP snapshot.
fn archive_class_source_request(
    snapshot: &ArtifactSnapshot,
    class: ClassRef,
) -> ClassSourceRequest {
    ClassSourceRequest {
        class,
        environment: archive_environment_request(snapshot),
    }
}

fn performed<T>(outcome: OperationOutcome<T>) -> T {
    match outcome {
        OperationOutcome::Performed(value) => value,
        // The other answers are named by the cases that expect them; a case that reaches this one
        // expected a value.
        _ => panic!("the request is answered with a value"),
    }
}

fn body(name: &[u8]) -> BodyRef {
    BodyRef::Name {
        name: JvmBytes(name.to_vec()),
        descriptor: None,
    }
}

// ---------------------------------------------------------------------------------------------
// 3.2 — one preparation serves every body a class view decodes
// ---------------------------------------------------------------------------------------------

/// A view that decodes N bodies materializes its class once and prepares it once.
///
/// The three bodies are located and decoded against **one** preparation of the class the binding
/// read — not one per body, and not a class read per body — which is what task 3.2 asks for. A
/// class view that still walked the original per-body entry would prepare nothing and parse the
/// class per body; one that prepared per body would prepare three.
#[test]
fn a_view_decodes_every_requested_body_against_one_preparation() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definition_of(&engine, &snapshot);
    let mut budget = budget();
    let before = d0_counts::snapshot();
    let report = performed(
        engine
            .class_view(
                &snapshot,
                &PhysicalScope::SnapshotAll,
                &ClassViewRequest {
                    class: ClassRef::Definition {
                        definition: definition.clone(),
                    },
                    bodies: vec![body(b"<init>"), body(b"simple"), body(b"reuse")],
                },
                &mut budget,
            )
            .expect("the class view is answered"),
    );
    let counted = before.since(d0_counts::snapshot());
    println!(
        "class_view (three bodies): materializations={} preparations={} body_decodes={} \
         | class_headers={} method_bodies={} class_bytes={}",
        counted.class_materializations,
        counted.class_preparations,
        counted.body_decodes,
        budget.usage().class_headers,
        budget.usage().method_bodies,
        budget.usage().class_bytes,
    );
    assert_eq!(report.bodies.len(), 3, "every requested body is published");
    assert!(
        report
            .bodies
            .iter()
            .all(|body| matches!(body, ClassViewBody::Read { .. })),
        "the three members declare bodies and are decoded: {:?}",
        report.bodies
    );
    assert_eq!(
        counted.class_materializations, 1,
        "the selected definition is materialized once, whatever the number of bodies"
    );
    assert_eq!(
        counted.class_preparations, 1,
        "one preparation serves every body this view decodes"
    );
    assert_eq!(
        counted.body_decodes, 3,
        "and exactly the bodies it asked for"
    );
    assert_eq!(counted.recovery_runs, 0, "a view runs no recovery");
    assert_eq!(budget.usage().class_headers, 1, "{:?}", budget.usage());
    assert_eq!(budget.usage().method_bodies, 3, "{:?}", budget.usage());
    assert_eq!(
        budget.usage().class_bytes,
        2 * report.class.class_bytes.length,
        "two parses of one read — the binding's member walk and the one preparation — never one \
         per body: {:?}",
        budget.usage()
    );
    assert_eq!(budget.usage().ir_items, 0, "and no analysis");
}

/// A view that decodes no body prepares nothing: a declaration-only read stays a declaration read.
#[test]
fn a_view_that_decodes_no_body_prepares_nothing() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definition_of(&engine, &snapshot);
    let mut budget = budget();
    let before = d0_counts::snapshot();
    let report = performed(
        engine
            .class_view(
                &snapshot,
                &PhysicalScope::SnapshotAll,
                &ClassViewRequest {
                    class: ClassRef::Definition { definition },
                    bodies: Vec::new(),
                },
                &mut budget,
            )
            .expect("the class view is answered"),
    );
    let counted = before.since(d0_counts::snapshot());
    println!(
        "class_view (no body): materializations={} preparations={} body_decodes={}",
        counted.class_materializations, counted.class_preparations, counted.body_decodes
    );
    assert!(report.bodies.is_empty());
    assert_eq!(counted.class_materializations, 1);
    assert_eq!(counted.class_preparations, 0);
    assert_eq!(counted.body_decodes, 0);
    assert_eq!(budget.usage().class_headers, 1, "{:?}", budget.usage());
    assert_eq!(budget.usage().method_bodies, 0, "{:?}", budget.usage());
}

// ---------------------------------------------------------------------------------------------
// 3.1 — the binding's own read is what the preparation is built over, on both paths
// ---------------------------------------------------------------------------------------------

/// Both binding paths read the definition they select once, and the preparation is built over that
/// read.
///
/// The identity path's read is the operation's own (`class_materializations` counts it); the name
/// path's is the search's — "which definitions of this name exist" reads them by definition — and
/// the search's candidate reads are the search's own cost, exactly as D0 recorded. What both paths
/// must show is the same thing: **one** class header and **one** preparation, and the same text.
#[test]
fn both_binding_paths_prepare_the_read_they_selected_once() {
    let _gate = gate();
    let engine = Engine::new();
    // The entry states the name the class declares for itself: a package-less `Scope.class`, which
    // is how the committed sample is read everywhere else.
    let archive = zip_of(&[(b"Scope.class", SCOPE)]);
    let snapshot = open(&archive);
    let scope = PhysicalScope::SnapshotAll;
    let mut listing_budget = budget();
    let definition = engine
        .list_class_declarations(&snapshot, &scope, &mut listing_budget)
        .expect("the archive's one class is listed")
        .items
        .first()
        .expect("the archive holds one class")
        .definition
        .clone();

    let run = |class: ClassRef| {
        let mut budget = budget();
        let before = d0_counts::snapshot();
        let report = performed(
            engine
                .class_source(
                    std::slice::from_ref(&snapshot),
                    &archive_class_source_request(&snapshot, class),
                    &mut budget,
                )
                .expect("the class-source request is answered"),
        );
        let counted = before.since(d0_counts::snapshot());
        println!(
            "class_source: materializations={} preparations={} body_decodes={} | class_headers={} \
             method_bodies={}",
            counted.class_materializations,
            counted.class_preparations,
            counted.body_decodes,
            budget.usage().class_headers,
            budget.usage().method_bodies,
        );
        assert_eq!(report.methods.len(), 8, "every member is presented");
        assert_eq!(
            counted.class_preparations, 1,
            "one preparation serves every member body"
        );
        assert_eq!(counted.body_decodes, 8, "and every member declares a body");
        assert_eq!(
            budget.usage().class_headers,
            1,
            "the selected definition is read once, on this path too: {:?}",
            budget.usage()
        );
        assert_eq!(budget.usage().method_bodies, 8, "{:?}", budget.usage());
        report
    };

    let identity = run(ClassRef::Definition {
        definition: definition.clone(),
    });
    let by_name = run(ClassRef::Name {
        class: ClassNameQuery::internal("Scope"),
    });
    assert_eq!(
        identity.text, by_name.text,
        "the two binding paths present the same class the same way"
    );
    assert_eq!(identity.class, by_name.class);
}

/// A name several definitions answer to is still ambiguous, and nothing is prepared for it.
#[test]
fn an_ambiguous_name_prepares_nothing_and_keeps_every_candidate() {
    let _gate = gate();
    let engine = Engine::new();
    // Two entries of one name, both declaring that class: the search confirms both and elects
    // neither, exactly as it does for a class held at two origins.
    let archive = zip_of(&[(b"Scope.class", SCOPE), (b"Scope.class", SCOPE)]);
    let snapshot = open(&archive);
    let mut budget = budget();
    let before = d0_counts::snapshot();
    let outcome = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &archive_class_source_request(
                &snapshot,
                ClassRef::Name {
                    class: ClassNameQuery::internal("Scope"),
                },
            ),
            &mut budget,
        )
        .expect("an ambiguous selection is an answer, not an error");
    let counted = before.since(d0_counts::snapshot());
    let OperationOutcome::Ambiguous(candidates) = outcome else {
        panic!("two definitions declare `p/Scope`");
    };
    let candidates = *candidates;
    println!(
        "class_source (ambiguous): candidates={} materializations={} preparations={} body_decodes={}",
        candidates.candidates.len(),
        counted.class_materializations,
        counted.class_preparations,
        counted.body_decodes
    );
    assert!(!candidates.candidates.is_empty(), "{candidates:?}");
    assert_eq!(
        counted.class_preparations, 0,
        "no definition was elected, so nothing was prepared"
    );
    assert_eq!(counted.body_decodes, 0);
    assert_eq!(budget.usage().method_bodies, 0, "{:?}", budget.usage());
}

// ---------------------------------------------------------------------------------------------
// 3.3 — a direct recovery run reads its class once, and prepares it only for what it shares
// ---------------------------------------------------------------------------------------------

/// A body that names no same-class member is recovered from **one** read and needs no preparation.
///
/// This is the other half of task 3.3's "the driver and the same-class callee join one
/// preparation": when there is no same-class callee read, there is nothing to share the preparation
/// with, and the run — which decodes exactly one body either way — must not pay for one. The callee
/// gate (a body that *does* name such members) lives in `tests/p3_accessor_edges.rs`.
#[test]
fn a_recovery_run_without_same_class_callees_reads_once_and_prepares_nothing() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definition_of(&engine, &snapshot);
    let request = MethodAnalysisRequest {
        environment: environment(&snapshot, profile()),
        method: PhysicalMethodId {
            owner: definition,
            name: JvmBytes(b"simple".to_vec()),
            descriptor: JvmBytes(b"()I".to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    let mut budget = budget();
    let before = d0_counts::snapshot();
    let recovered = engine
        .recover_method(std::slice::from_ref(&snapshot), &request, &mut budget)
        .expect("the member is recovered");
    let counted = before.since(d0_counts::snapshot());
    println!(
        "recover_method (no callees): materializations={} preparations={} body_decodes={} \
         recovery_runs={} | class_headers={} method_bodies={}",
        counted.class_materializations,
        counted.class_preparations,
        counted.body_decodes,
        counted.recovery_runs,
        budget.usage().class_headers,
        budget.usage().method_bodies,
    );
    assert!(recovered.callees().is_none(), "this body names no callee");
    assert_eq!(
        counted.class_materializations, 1,
        "one read of the selected class"
    );
    assert_eq!(
        counted.class_preparations, 0,
        "no same-class callee read consumes a preparation, so none is made"
    );
    assert_eq!(counted.body_decodes, 1, "and one body is decoded");
    assert_eq!(counted.recovery_runs, 1);
    assert_eq!(budget.usage().class_headers, 1, "{:?}", budget.usage());
    assert_eq!(budget.usage().method_bodies, 1, "{:?}", budget.usage());
    assert_eq!(
        recovered
            .analysis()
            .reads
            .iter()
            .filter(|read| read.reason == ReadReason::DriverMethodBody)
            .count(),
        1,
        "the run really read its own definition once: {:?}",
        recovered.analysis().reads
    );
}

/// A request whose declared environment does not name its content reads no class at all.
///
/// The binding path D2 changed is the read the run performs *after* the environment and the request
/// are known to be usable: a rejected environment states its problems and reads nothing, so the
/// handoff cannot turn one into a read.
#[test]
fn a_request_with_a_missing_dependency_reads_no_class() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definition_of(&engine, &snapshot);
    let request = MethodAnalysisRequest {
        environment: environment(&snapshot, profile()),
        method: PhysicalMethodId {
            owner: definition,
            name: JvmBytes(b"simple".to_vec()),
            descriptor: JvmBytes(b"()I".to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    let mut request = request;
    // A root naming a snapshot this request does not provide: the declaration is buildable and the
    // environment validator owns the finding.
    request.environment.runtime.load_domain.roots = vec![LoadRoot::StandaloneClass {
        snapshot: SnapshotId("not-provided-by-this-request".to_string()),
    }];
    request.environment.domains = vec![request.environment.runtime.load_domain.clone()];
    let mut budget = budget();
    let before = d0_counts::snapshot();
    let recovered = engine
        .recover_method(std::slice::from_ref(&snapshot), &request, &mut budget)
        .expect("a rejected environment is a report, not an error");
    let counted = before.since(d0_counts::snapshot());
    println!(
        "recover_method (rejected environment): materializations={} preparations={} body_decodes={}",
        counted.class_materializations, counted.class_preparations, counted.body_decodes
    );
    assert!(
        recovered
            .analysis()
            .environment_problems
            .iter()
            .any(|problem| problem.code == EnvironmentProblemCode::ContentNotProvided),
        "{:?}",
        recovered.analysis().environment_problems
    );
    assert_eq!(
        counted.class_materializations, 0,
        "a rejected environment reads no class: {counted:?}"
    );
    assert_eq!(counted.class_preparations, 0, "{counted:?}");
    assert_eq!(counted.body_decodes, 0, "{counted:?}");
    assert_eq!(budget.usage().class_headers, 0, "{:?}", budget.usage());
    assert_eq!(budget.usage().method_bodies, 0, "{:?}", budget.usage());
}

// ---------------------------------------------------------------------------------------------
// 3.4 — the four store states, consecutive requests, and what is retained
// ---------------------------------------------------------------------------------------------

/// One store configuration, as the D0 gate's own four states.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StoreState {
    /// No store attached at all.
    None,
    /// A store that retains nothing.
    Zero,
    /// A store too small to hold one container product.
    TooSmall,
    /// A store with room for the products of this fixture.
    Roomy,
}

const STORE_STATES: [StoreState; 4] = [
    StoreState::None,
    StoreState::Zero,
    StoreState::TooSmall,
    StoreState::Roomy,
];

impl StoreState {
    fn attach(self, budget: Budget) -> (Budget, Option<FactsCache>) {
        match self {
            StoreState::None => (budget, None),
            StoreState::Zero => {
                let store = FactsCache::new(FactsIdentity::current(), FactsCapacity::none());
                (budget.with_facts_cache(store.clone()), Some(store))
            }
            StoreState::TooSmall => {
                let store = FactsCache::new(FactsIdentity::current(), FactsCapacity::new(0, 0));
                (budget.with_facts_cache(store.clone()), Some(store))
            }
            StoreState::Roomy => {
                let store = FactsCache::current(FactsCapacity::new(64, 1 << 20));
                (budget.with_facts_cache(store.clone()), Some(store))
            }
        }
    }
}

/// One class-source run over a fresh snapshot under one store state.
fn class_source_run(state: StoreState) -> (String, d0_counts::Counts, UsageSnapshot) {
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definition_of(&engine, &snapshot);
    let (mut budget, _store) = state.attach(budget());
    let before = d0_counts::snapshot();
    let report = performed(
        engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &class_source_request(&snapshot, ClassRef::Definition { definition }),
                &mut budget,
            )
            .expect("the class-source request is answered"),
    );
    let counted = before.since(d0_counts::snapshot());
    (report.text, counted, budget.usage())
}

/// The four store states do the same work, and a consecutive request does it again.
///
/// What a store may change is the *charges it answers*: it is a retention decision about facts, not
/// a hidden second lifetime for the operation's own read. The operation's demand figures — one
/// materialization, one preparation, one decode per member — are the same in all four states, and a
/// second, consecutive request over the same store does the same work again: nothing the first
/// request read is retained in a place the second one is answered from.
#[test]
fn the_four_store_states_do_the_same_work_and_no_request_is_free() {
    let _gate = gate();
    let mut reference: Option<(String, d0_counts::Counts)> = None;
    for state in STORE_STATES {
        let (text, counted, usage) = class_source_run(state);
        println!(
            "{state:?}: materializations={} preparations={} body_decodes={} | class_headers={} \
             method_bodies={} class_bytes={}",
            counted.class_materializations,
            counted.class_preparations,
            counted.body_decodes,
            usage.class_headers,
            usage.method_bodies,
            usage.class_bytes
        );
        assert_eq!(counted.class_materializations, 1, "{state:?}");
        assert_eq!(counted.class_preparations, 1, "{state:?}");
        assert_eq!(counted.body_decodes, 8, "{state:?}");
        assert_eq!(usage.class_headers, 1, "{state:?}");
        assert_eq!(usage.method_bodies, 8, "{state:?}");
        match &reference {
            Some((expected, _)) => assert_eq!(&text, expected, "{state:?} presents another class"),
            None => reference = Some((text, counted)),
        }
    }

    // Consecutive requests over one store: each does the same *demand* work, and none is answered
    // from a fact the previous one left behind outside the store.
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definition_of(&engine, &snapshot);
    let store = FactsCache::current(FactsCapacity::new(64, 1 << 20));
    let mut first_counts = Vec::new();
    for run in 1..=2 {
        let mut budget = budget().with_facts_cache(store.clone());
        let before = d0_counts::snapshot();
        performed(
            engine
                .class_source(
                    std::slice::from_ref(&snapshot),
                    &class_source_request(
                        &snapshot,
                        ClassRef::Definition {
                            definition: definition.clone(),
                        },
                    ),
                    &mut budget,
                )
                .expect("the request is answered"),
        );
        let counted = before.since(d0_counts::snapshot());
        println!("consecutive run {run}: {counted:?}");
        assert_eq!(counted.class_materializations, 1, "run {run}");
        assert_eq!(counted.class_preparations, 1, "run {run}");
        assert_eq!(counted.body_decodes, 8, "run {run}");
        first_counts.push(counted);
    }
    assert_eq!(
        first_counts[0], first_counts[1],
        "a consecutive request over the same store did different work"
    );
    // And once the store forgets, the next request rebuilds what it needs rather than being answered
    // from something that outlived the store's own retention.
    store.clear();
    assert_eq!(
        (store.report().entries, store.report().retained_bytes),
        (0, 0),
        "a cleared store retains nothing: {:?}",
        store.report()
    );
    let (_, after_clear, _) = {
        let mut budget = budget().with_facts_cache(store.clone());
        let before = d0_counts::snapshot();
        let report = performed(
            engine
                .class_source(
                    std::slice::from_ref(&snapshot),
                    &class_source_request(
                        &snapshot,
                        ClassRef::Definition {
                            definition: definition.clone(),
                        },
                    ),
                    &mut budget,
                )
                .expect("the request is answered"),
        );
        let counted = before.since(d0_counts::snapshot());
        println!("after clear: {counted:?}");
        assert_eq!(counted.class_materializations, 1);
        assert_eq!(counted.class_preparations, 1);
        assert_eq!(counted.body_decodes, 8);
        (report.text, counted, budget.usage())
    };
    assert_eq!(
        first_counts[0], after_clear,
        "the cleared store's run differed"
    );
}

/// A cancelled request prepares nothing, decodes nothing and retains nothing.
#[test]
fn a_cancelled_request_leaves_nothing_retained() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definition_of(&engine, &snapshot);
    let token = CancellationToken::new();
    token.cancel();
    let store = FactsCache::current(FactsCapacity::new(64, 1 << 20));
    let mut budget =
        Budget::with_cancellation_token(limits(), token).with_facts_cache(store.clone());
    let before = d0_counts::snapshot();
    let outcome = engine.class_source(
        std::slice::from_ref(&snapshot),
        &class_source_request(&snapshot, ClassRef::Definition { definition }),
        &mut budget,
    );
    let counted = before.since(d0_counts::snapshot());
    println!(
        "cancelled class_source: {outcome:?} materializations={} preparations={} body_decodes={}",
        counted.class_materializations, counted.class_preparations, counted.body_decodes
    );
    assert!(
        outcome.is_err(),
        "a cancellation before the first charge is the request's own refusal"
    );
    assert_eq!(counted.class_preparations, 0);
    assert_eq!(counted.body_decodes, 0);
    assert_eq!(counted.recovery_runs, 0);
    assert_eq!(budget.usage().class_headers, 0, "{:?}", budget.usage());
    assert_eq!(
        (store.report().entries, store.report().retained_bytes),
        (0, 0),
        "a cancelled request retained nothing: {:?}",
        store.report()
    );
}

/// A store's retention is the store's own: it does not make a *later* request that carries no store
/// cheaper.
///
/// The rule the reader states for the container products a read may keep (`HeldContainerFacts`: a
/// store's retention is deliberately *not* a hold) has to survive D2's handover: the preparation's
/// read reaches the class's container, and if it recorded a hold for a product the caller's store
/// answered, that store would decide what an unrelated request — one that attached no store at all —
/// is answered from. The two runs below are the same request over one snapshot, one with a roomy
/// store and one with none: both pay their own directory read, so their charges agree.
#[test]
fn a_store_never_makes_the_next_storeless_request_cheaper() {
    let _gate = gate();
    let engine = Engine::new();
    let archive = zip_of(&[(b"Scope.class", SCOPE), (b"Shape.class", SCOPE)]);
    let snapshot = open(&archive);
    let scope = PhysicalScope::SnapshotAll;
    let mut listing_budget = budget();
    let definition = engine
        .list_class_declarations(&snapshot, &scope, &mut listing_budget)
        .expect("the archive's classes are listed")
        .items
        .first()
        .expect("the archive holds classes")
        .definition
        .clone();
    // The store is opened here and stays alive across both runs: that is what makes the second run
    // the interesting one — a product the *store* still holds is exactly what a store-less request
    // must not be answered from.
    let store = match StoreState::Roomy.attach(budget()) {
        (budget, Some(store)) => (budget, store),
        other => panic!("the roomy state names its store: {other:?}"),
    };
    let run = |budget: &mut Budget| {
        let report = performed(
            engine
                .class_source(
                    std::slice::from_ref(&snapshot),
                    &archive_class_source_request(
                        &snapshot,
                        ClassRef::Definition {
                            definition: definition.clone(),
                        },
                    ),
                    budget,
                )
                .expect("the class-source request is answered"),
        );
        report.usage
    };

    let _ = store.1.report();
    let mut with_store_budget = budget();
    with_store_budget = with_store_budget.with_facts_cache(store.1.clone());
    let _ = run(&mut with_store_budget);
    let with_store = with_store_budget.usage();
    let mut without_store_budget = budget();
    let _ = run(&mut without_store_budget);
    let without_store = without_store_budget.usage();
    assert!(
        store.1.report().retained_bytes > 0,
        "the store really kept the products of the first run: {:?}",
        store.1.report()
    );
    println!(
        "with a store: archive_entries={} entry_bytes={} | without: archive_entries={} \
         entry_bytes={}",
        with_store.archive_entries,
        with_store.entry_bytes,
        without_store.archive_entries,
        without_store.entry_bytes
    );
    // The shape, as the two budgets really account for it. The container is reached twice on this
    // request — once to locate the entry the binding read materializes, and once more for the
    // preparation's read — and the fixture's directory holds two entries:
    //
    // * the store-attached run parses the directory once (the first reach), the store remembers the
    //   product, and the second reach is a store hit — 1 entry walked + 2 parsed = 3;
    // * the store-less run parses it twice, because the locator's own reach is not a hold this read
    //   can be answered from — 1 + 2 + 2 = 5.
    //
    // What must not happen is the *third* figure: a store-less request answered from the product the
    // *previous* run's store kept. That would show as the store-less run paying the store-attached
    // run's 3, and this is the assertion that fails when a read records a hold for a store-answered
    // product (the mutation the change's verification record states).
    assert_eq!(
        with_store.archive_entries, 3,
        "the store-attached run's own directory reads: {with_store:?}"
    );
    assert_eq!(
        without_store.archive_entries, 5,
        "the store-less run paid for both of its own directory reads: {without_store:?}"
    );
}
