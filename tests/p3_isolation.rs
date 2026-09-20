//! P3 A17: the recovery layer is constructed where it is charged, and the physical query is not.
//!
//! A17 says an X1 entry point constructs no CFG, no SSA and no AST, and that "no source came out" is
//! not a proof of it. P2 argued the behavioural half from the dimensions that existed then —
//! resolver, CFG, SSA and region construction have no counter of their own — and said so. This file is
//! the half P2 could not write, because the Region and Java-AST layers did not exist yet (A17's own
//! row: the P2 evidence is a budget proxy, and P3 owns the real recovery path's isolation regression).
//!
//! One fixture, two entries through the public facade, and the same usage dimensions read on both
//! runs:
//!
//! * `Engine::query` (X1, `MentionsSymbol`) really scans — it publishes the invocation it found, and
//!   it really decoded the instruction stream — and every construction dimension stays 0, with the
//!   read accounting at 0 headers and 0 bodies. The recovery layer's existence adds no read to it
//!   (A16).
//! * `Engine::recover_method` on the same snapshot charges all of them: the IR, its edges, the steps
//!   and the canonical clones are constructed by the run that presents a body, which is where they
//!   belong — the construction happens, and it happens there.
//!
//! The fixture is the committed ECJ 4.6.1 class of the `jsr` era (45.3). Its `finallyPath` really
//! holds a `jsr`/`ret` subroutine, so the canonical pass really clones a block once per entrance and
//! `normalization_clones` below is a count of an actual construction rather than of a floor.
//!
//! The presented run's artifact is a **stated** fallback rather than Java — that body's `jsr`/`ret`
//! subroutines are refused by the region pass, so the text quotes the BCIs it could not place — and
//! that is the point of the case: the construction is charged on the run that read the body, whatever
//! the quality plane of the artifact it managed to present. The Java/Structured path's own acceptance
//! is `tests/p3_recovery_entry.rs` (A16) and the corpus work of P3 3.3.
//!
//! What this file does **not** prove, said where it could be mistaken for it: there is still no
//! per-construction counter for the region pass or the AST, so "the query started neither" is the
//! absence of every charge those passes raise, not a counter that names them. The source half of the
//! guard is `tests/p2_contracts.rs`, which derives the forbidden table from the recovery layer's own
//! declarations and scans the guarded files with it; the two halves fail for different reasons and
//! neither substitutes for the other.

use jarde::*;
use std::slice;

/// The committed historical fixture: ECJ 4.6.1, `-source 1.3`, class-file version 45.3, whose
/// `finallyPath(I)I` compiles `finally` into two `jsr` calls of one subroutine.
const V45: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class");

/// The member this file presents: the one that holds the subroutine.
const MEMBER: (&[u8], &[u8]) = (b"finallyPath", b"(I)I");

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
    }
}

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

/// One caller domain rooted at the fixture's own snapshot, and nothing else: the simplest
/// environment the library's validator accepts without a problem.
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

/// The one query this file sends: the `java/lang/Object.<init>()V` invocation the fixture's own
/// constructor holds, so a run that publishes nothing is a run that did not scan (the same request
/// P2 5.2's A17 case sends).
fn query_request(snapshot: &ArtifactSnapshot) -> QueryRequest {
    QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target: QueryTarget::Symbol {
            value: SymbolRef::Method {
                owner: bytes(b"java/lang/Object"),
                name: bytes(b"<init>"),
                descriptor: bytes(b"()V"),
            },
        },
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
        max_items: 0,
        cursor: None,
    }
}

/// The member the recovery run presents, named the way the library derives the identity it read.
fn recovery_request(
    snapshot: &ArtifactSnapshot,
    inspected: &EngineHeaderReport,
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment: environment(snapshot),
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: inspected.source.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: bytes(MEMBER.0),
            descriptor: bytes(MEMBER.1),
        },
        stages: AnalysisStage::ALL.to_vec(),
    }
}

/// The usage snapshot of a finished analysis run.
fn usage_of(report: &MethodAnalysisReport) -> UsageSnapshot {
    match &report.execution {
        ExecutionReport::Complete { usage } => usage.clone(),
        other => panic!("a legal request completes: {other:?}"),
    }
}

/// The four construction dimensions A17 is about, read through the counted enum so that a renamed
/// dimension is a compile error instead of a silently unchecked field.
fn construction(usage: &UsageSnapshot) -> Vec<(CountedBudgetDimension, u64)> {
    [
        CountedBudgetDimension::IrItems,
        CountedBudgetDimension::IrEdges,
        CountedBudgetDimension::AnalysisSteps,
        CountedBudgetDimension::NormalizationClones,
    ]
    .into_iter()
    .map(|dimension| (dimension, usage.counted_usage(dimension)))
    .collect()
}

#[test]
fn a_physical_query_constructs_none_of_the_recovery_layer_and_the_recovery_run_constructs_it() {
    // One snapshot for both runs: the two entries are asked about the same bytes, so the numbers below
    // are about where the construction happens and not about two different inputs.
    let engine = Engine::new();
    let mut premise_budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(V45.to_vec()), &mut premise_budget)
        .expect("the fixture opens as a standalone CLASS");

    // 1. X1 through the public entry: the physical query the A17 obligation is written about.
    let mut query_budget = Budget::new(limits());
    let report = engine
        .query(&snapshot, &query_request(&snapshot), &mut query_budget)
        .expect("the fixture query runs");
    assert_eq!(report.analysis, QueryAnalysis::Performed);
    assert_eq!(
        report.items.len(),
        1,
        "the constructor really invokes `java/lang/Object.<init>()V`: {:?}",
        report.items
    );
    assert_eq!(report.items[0].consumer, Some(ConsumerKind::Invocation));
    let queried = query_budget.usage();
    assert!(
        queried.class_bytes > 0 && queried.code_bytes > 0,
        "the physical scan really read the class and decoded the instruction stream, so the zeroes \
         below are not a query that did nothing: {queried:?}"
    );
    let queried_construction = construction(&queried);
    assert!(
        queried_construction.iter().all(|(_, count)| *count == 0),
        "a physical query constructs no IR item, no edge, no step and no clone, however real the \
         recovery layer is: {queried_construction:?}"
    );
    assert_eq!(
        (queried.class_headers, queried.method_bodies),
        (0, 0),
        "and it charges no header read and no body attempt: the recovery layer existing beside it \
         adds no read (A16). The physical scan's own evidence is `class_bytes`/`code_bytes` above, \
         which is P2 5.2's accounting and not the P2 pass accounting"
    );

    // 2. The same fixture through the entry that presents a method body.
    let mut inspected_budget = Budget::new(limits());
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut inspected_budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    let request = recovery_request(&snapshot, &inspected);
    let mut recovery_budget = Budget::new(limits());
    let recovered = engine
        .recover_method(slice::from_ref(&snapshot), &request, &mut recovery_budget)
        .expect("a legal request is answered, not raised");

    let presented = usage_of(recovered.analysis());
    let presented_construction = construction(&presented);
    assert!(
        presented_construction.iter().all(|(_, count)| *count > 0),
        "the run that presents a body builds the IR, its edges, the steps and the canonical clones — \
         the construction really happens, and it happens in the run that reads the body: \
         {presented_construction:?}"
    );
    assert_eq!(
        presented.class_headers, 1,
        "the presented run read the driver member's own class definition once: {presented:?}"
    );
    assert_eq!(
        presented.method_bodies, 1,
        "and it attempted exactly one body — the one it presents: {presented:?}"
    );
    assert!(presented.code_bytes > 0, "and a body really was decoded");

    // The artifact is of that member: the entry presented the body the run above charged for.
    let artifact = recovered.recovery();
    assert_eq!(artifact.method, "finallyPath(I)I");
    assert!(artifact.produced(), "{:?}", artifact.outcome);
    assert!(!artifact.text.is_empty(), "{:?}", artifact.outcome);
}
