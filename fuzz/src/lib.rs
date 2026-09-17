//! Fixed requests, bounded entry points and public-contract assertions shared by the two
//! P1 fuzz targets (design 3.3).
//!
//! Both targets treat the *whole fuzz input* as the artifact and reuse the library's own
//! entry points, reader and `Limits`; nothing here re-implements a reader, mutates a
//! request or generates class files. This module only
//!
//! * builds a small, closed set of fixed requests — the `query` target selects one per
//!   input from its first byte, so one corpus can drive several relations without
//!   hand-written input generation,
//! * runs them through the public `Engine` entry points under hard input limits,
//! * asserts the contracts that must hold for *every* outcome, including the damaged,
//!   interrupted and cancelled ones.
//!
//! The assertion set is deliberately small and only asserts documented behaviour: a
//! libFuzzer finding must be a real contract violation, not a check that a legitimate
//! outcome (an empty page, a damaged artifact, a documented continuation seam) trips.
//! Each assertion below names the contract it enforces.

use jarde::{
    ArtifactInput, ArtifactSnapshot, ArtifactTreeReport, Budget, ConsumerKind, ConsumerSchema,
    CoverageState, DelegationPolicy, Engine, ExecutionReport, JvmBytes, LayoutMode, Limits,
    LiteralValue, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    MultiReleaseViewReport, PhysicalScope, PhysicalView, QueryRelation, QueryReport, QueryRequest,
    QueryResolution, QueryTarget, RuntimeProfile, RuntimeUncertainty, RuntimeView, SymbolRef,
    UsageSnapshot, VerificationStatus,
};

/// The symbol, class and literal the committed seeds really carry.
///
/// `fuzz/corpus/generate_seeds.py` writes a `com/example/Seed` class whose `probe()V` body
/// mentions all three, so a valid seed answers at least one request shape without any
/// mutation:
///
/// * `com/example/Fuzz.target:()V` — `invokestatic` (a symbol/method target),
/// * `com/example/Fuzz` — `new` (a symbol/class target),
/// * `fuzz-literal` — `ldc` (a literal target).
pub const FUZZ_OWNER: &[u8] = b"com/example/Fuzz";
pub const FUZZ_MEMBER: &[u8] = b"target";
pub const FUZZ_DESCRIPTOR: &[u8] = b"()V";
pub const FUZZ_LITERAL: &[u8] = b"fuzz-literal";

/// Hard caps every fuzz run works under ("give every engine budget a small limit").
///
/// They are deliberately small: one input must stay far inside the 64 KiB `-max_len` and
/// the 512 MB RSS limit, and every budget-stop path of the engine stays reachable well
/// below these caps, so a fuzzed input does not have to be huge to exercise them.
pub fn limits() -> Limits {
    Limits {
        input_bytes: 256 * 1024,
        archive_entries: 64,
        entry_bytes: 128 * 1024,
        read_bytes: 512 * 1024,
        class_bytes: 128 * 1024,
        attribute_bytes: 64 * 1024,
        code_bytes: 64 * 1024,
        result_items: 64,
        output_bytes: 16 * 1024,
        nested_depth: 2,
        elapsed_millis: 5_000,
    }
}

/// Number of fixed request shapes [`query_request`] selects from.
pub const QUERY_SHAPES: u8 = 5;

/// One fixed query request: relation, target and consumer schema per shape.
///
/// The shapes stay fixed values rather than being derived from the input, so a finding is
/// reproducible from the input bytes alone. Shape 4 names every category P1 declares,
/// `Verification` and `Debug` included, which makes the "declared but not implemented"
/// contract reachable from the corpus (see [`assert_query_contract`]).
pub fn query_request(shape: u8, snapshot: &ArtifactSnapshot) -> QueryRequest {
    use ConsumerKind::{
        Annotation, Bootstrap, Constant, Debug, Exception, Field, InnerNest, Invocation, Module,
        Resource, Signature, Type, Verification,
    };
    let (relation, target, kinds): (QueryRelation, QueryTarget, Vec<ConsumerKind>) =
        match shape % QUERY_SHAPES {
            0 => (
                QueryRelation::MentionsSymbol,
                method_target(),
                vec![Invocation, Type],
            ),
            1 => (
                QueryRelation::MentionsSymbol,
                class_target(),
                vec![Type, Exception, Signature, InnerNest],
            ),
            2 => (
                QueryRelation::LiteralValue,
                literal_target(),
                vec![Constant, Resource],
            ),
            3 => (
                QueryRelation::ConstantPoolContains,
                method_target(),
                vec![Invocation],
            ),
            _ => (
                QueryRelation::MentionsSymbol,
                method_target(),
                vec![
                    Annotation,
                    Bootstrap,
                    Constant,
                    Debug,
                    Exception,
                    Field,
                    InnerNest,
                    Invocation,
                    Module,
                    Resource,
                    Signature,
                    Type,
                    Verification,
                ],
            ),
        };
    QueryRequest {
        relation,
        target,
        physical: physical_view(snapshot, PhysicalScope::SnapshotAll),
        consumers: ConsumerSchema::new(1, kinds),
        // No page limit: the result budget alone decides how much is published.
        max_items: 0,
        cursor: None,
    }
}

fn method_target() -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Method {
            owner: JvmBytes(FUZZ_OWNER.to_vec()),
            name: JvmBytes(FUZZ_MEMBER.to_vec()),
            descriptor: JvmBytes(FUZZ_DESCRIPTOR.to_vec()),
        },
    }
}

fn class_target() -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Class {
            owner: JvmBytes(FUZZ_OWNER.to_vec()),
        },
    }
}

fn literal_target() -> QueryTarget {
    QueryTarget::Literal {
        value: LiteralValue::String {
            value: JvmBytes(FUZZ_LITERAL.to_vec()),
        },
    }
}

fn physical_view(snapshot: &ArtifactSnapshot, scope: PhysicalScope) -> PhysicalView {
    PhysicalView {
        snapshot: snapshot.id().clone(),
        scope,
    }
}

/// The fixed runtime view the `artifact_tree` target binds: a Java 17 class path domain
/// that requests standard multi-release selection.
pub fn runtime_view(snapshot: &ArtifactSnapshot) -> RuntimeView {
    RuntimeView {
        physical: physical_view(snapshot, PhysicalScope::SnapshotAll),
        profile: RuntimeProfile {
            java_release: 17,
            multi_release: MultiReleasePolicy::Enabled,
            layout: LayoutMode::Generic,
        },
        load_domain: LoadDomain {
            loader: LoaderId("fuzz".into()),
            parent_loader: None,
            delegation: DelegationPolicy::ParentFirst,
            roots: vec![LoadRoot::Snapshot {
                snapshot: snapshot.id().clone(),
            }],
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        },
    }
}

/// Opens the fuzz input as an artifact.
///
/// `None` means the input is not an artifact this engine accepts at all: a damaged
/// container or a wrong magic is rejected through the public error path, which is the
/// documented outcome for arbitrary bytes and not a fuzzing failure.
pub fn open(input: &[u8], limits: &Limits) -> Option<ArtifactSnapshot> {
    let mut budget = Budget::new(limits.clone());
    Engine::new()
        .open(ArtifactInput::bytes(input.to_vec()), &mut budget)
        .ok()
}

/// Runs one request, treating a public error as an accepted outcome.
pub fn run_query(
    snapshot: &ArtifactSnapshot,
    request: &QueryRequest,
    limits: &Limits,
) -> Option<QueryReport> {
    let mut budget = Budget::new(limits.clone());
    Engine::new().query(snapshot, request, &mut budget).ok()
}

/// Enumerates the nested artifact tree, treating a public error as an accepted outcome.
pub fn run_tree(snapshot: &ArtifactSnapshot, limits: &Limits) -> Option<ArtifactTreeReport> {
    let mut budget = Budget::new(limits.clone());
    Engine::new()
        .enumerate_artifact_tree(snapshot, &mut budget)
        .ok()
}

/// Runs standard multi-release selection, treating a public error as an accepted outcome.
pub fn run_multi_release(
    snapshot: &ArtifactSnapshot,
    limits: &Limits,
) -> Option<MultiReleaseViewReport> {
    let view = runtime_view(snapshot);
    let mut budget = Budget::new(limits.clone());
    Engine::new()
        .select_multi_release(snapshot, &view, &mut budget)
        .ok()
}

/// Asserts every public query contract that must hold for any outcome.
pub fn assert_query_contract(report: &QueryReport, limits: &Limits) {
    // `returned_items` describes this page exactly.
    assert_eq!(
        report.page.returned_items,
        u64::try_from(report.items.len()).expect("a page length fits u64"),
        "page.returned_items must count the returned items"
    );
    // Bounded publication: one returned item costs one `result_items`.
    assert!(
        u64::try_from(report.items.len()).expect("a page length fits u64") <= limits.result_items,
        "the page published more items than the result-items limit: {} > {}",
        report.items.len(),
        limits.result_items
    );
    // A cursor only continues a scan that stopped before the end; the engine may also
    // report `has_more` without a cursor when it stopped before publishing any new item
    // (the documented continuation seam), so only this direction is asserted.
    assert!(
        report.page.cursor.is_none() || report.page.has_more,
        "a page cursor without `has_more` cannot be continued"
    );

    let structural = &report.coverage.dimensions.artifact_structural;
    if !matches!(report.execution, ExecutionReport::Complete { .. }) {
        assert_eq!(
            structural.state,
            CoverageState::Partial,
            "a damaged, interrupted or cancelled scan must not claim artifact-structural completeness: {:?}",
            report.execution
        );
    }
    if structural.state == CoverageState::CompleteWithinSchema {
        assert!(
            matches!(report.execution, ExecutionReport::Complete { .. }),
            "completeness and a degraded execution cannot both hold: {:?}",
            report.execution
        );
        assert!(
            report.coverage.unsupported_categories.is_empty(),
            "a schema naming categories P1 does not implement cannot be complete within schema"
        );
        assert!(
            structural.skipped.is_empty(),
            "a complete artifact-structural range cannot leave known units unexamined"
        );
    }
    // The consumer schema is echoed, and the categories P1 declares but does not
    // implement are reported instead of being silently treated as scanned.
    let declared_but_unimplemented: Vec<ConsumerKind> = report
        .consumers
        .kinds
        .iter()
        .copied()
        .filter(|kind| matches!(kind, ConsumerKind::Verification | ConsumerKind::Debug))
        .collect();
    assert_eq!(
        report.coverage.unsupported_categories, declared_but_unimplemented,
        "unsupported categories must be exactly the declared-but-unimplemented ones"
    );
    // P1 keeps the two other coverage dimensions not requested.
    assert_eq!(
        report.coverage.dimensions.runtime_resolution.state,
        CoverageState::NotRequested
    );
    assert_eq!(
        report.coverage.dimensions.dynamic_analysis.state,
        CoverageState::NotRequested
    );
    // `scanned_items` counts every item the scan looked at, `unknown_candidates` the
    // subset that could not be decided; a subset can never be larger.
    assert!(
        report.coverage.unknown_candidates <= report.coverage.scanned_items,
        "unknown candidates must be a subset of the scanned items"
    );
    // Every published item answers the requested relation, and P1 never claims a
    // resolution.
    for item in &report.items {
        assert_eq!(
            item.relation, report.relation,
            "an item must answer the requested relation"
        );
        assert_eq!(
            item.resolution,
            QueryResolution::NotRequested,
            "P1 does not resolve definitions, so no item may claim a resolution"
        );
    }
    assert_usage(usage_of(&report.execution), limits);
}

/// Asserts every public artifact-tree contract that must hold for any outcome.
pub fn assert_tree_contract(report: &ArtifactTreeReport, limits: &Limits) {
    // Every reported container, layout node and entry is billed one `result_items`, so
    // the containers and layout nodes alone can never exceed the limit.
    let billed = report
        .containers
        .len()
        .saturating_add(report.layout_nodes.len());
    assert!(
        u64::try_from(billed).expect("a container count fits u64") <= limits.result_items,
        "the tree reported more containers/layout nodes than the result-items limit"
    );
    let structural = &report.coverage.artifact_structural;
    if !matches!(report.execution, ExecutionReport::Complete { .. }) {
        assert_eq!(
            structural.state,
            CoverageState::Partial,
            "a damaged or interrupted tree enumeration must not claim completeness: {:?}",
            report.execution
        );
    }
    if structural.state == CoverageState::CompleteWithinSchema {
        assert!(
            matches!(report.execution, ExecutionReport::Complete { .. }),
            "completeness and a degraded execution cannot both hold"
        );
        assert!(
            structural.skipped.is_empty(),
            "a complete tree cannot leave known entries or pending containers unexamined"
        );
        // The aggregate is complete only when every established container was.
        for container in &report.containers {
            assert!(
                matches!(container.execution, ExecutionReport::Complete { .. }),
                "a complete tree reports only complete containers: {:?}",
                container.execution
            );
        }
    }
    assert_usage(usage_of(&report.execution), limits);
}

/// Asserts every public multi-release contract that must hold for any outcome.
pub fn assert_multi_release_contract(report: &MultiReleaseViewReport, limits: &Limits) {
    // Two result items reserve one container inspection, one more per returned entry
    // evidence and one more per selection, so the reported items stay inside the limit.
    let billed: usize = report
        .containers
        .iter()
        .map(|container| {
            2usize
                .saturating_add(container.entries.len())
                .saturating_add(container.selections.len())
        })
        .sum();
    assert!(
        u64::try_from(billed).expect("a selection count fits u64") <= limits.result_items,
        "multi-release selection reported more items than the result-items limit"
    );
    let runtime = &report.coverage.runtime_resolution;
    if runtime.state == CoverageState::CompleteWithinSchema {
        assert!(
            matches!(report.execution, ExecutionReport::Complete { .. }),
            "runtime-resolution completeness and a degraded execution cannot both hold: {:?}",
            report.execution
        );
    }
    if !matches!(report.execution, ExecutionReport::Complete { .. }) {
        assert_eq!(
            runtime.state,
            CoverageState::Partial,
            "an interrupted selection must not claim runtime-resolution completeness: {:?}",
            report.execution
        );
    }
    assert_eq!(
        report.coverage.dynamic_analysis.state,
        CoverageState::NotRequested
    );
    // P1 never links or verifies a class, so no report may claim verification.
    assert_eq!(report.verification, VerificationStatus::NotPerformed);
    assert_usage(usage_of(&report.execution), limits);
}

/// The run always stays inside the limits it was given.
///
/// `elapsed_millis` is intentionally not checked: it is a measurement, not a charge, and
/// exceeding it is what terminates the run.
fn assert_usage(usage: &UsageSnapshot, limits: &Limits) {
    assert!(
        usage.input_bytes <= limits.input_bytes,
        "input_bytes crossed the limit"
    );
    assert!(
        usage.archive_entries <= limits.archive_entries,
        "archive_entries crossed the limit"
    );
    assert!(
        usage.entry_bytes <= limits.entry_bytes,
        "entry_bytes crossed the limit"
    );
    assert!(
        usage.read_bytes <= limits.read_bytes,
        "read_bytes crossed the limit"
    );
    assert!(
        usage.class_bytes <= limits.class_bytes,
        "class_bytes crossed the limit"
    );
    assert!(
        usage.attribute_bytes <= limits.attribute_bytes,
        "attribute_bytes crossed the limit"
    );
    assert!(
        usage.code_bytes <= limits.code_bytes,
        "code_bytes crossed the limit"
    );
    assert!(
        usage.result_items <= limits.result_items,
        "result_items crossed the limit"
    );
    assert!(
        usage.output_bytes <= limits.output_bytes,
        "output_bytes crossed the limit"
    );
    assert!(
        usage.nested_depth <= limits.nested_depth,
        "nested_depth crossed the limit"
    );
}

fn usage_of(execution: &ExecutionReport) -> &UsageSnapshot {
    match execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage,
    }
}

/// Checks that the committed seeds really reach the paths they are meant to seed.
///
/// This is a normal test module, not a fuzz target: `cargo test` in this workspace
/// replays the corpus once and asserts the non-vacuous facts (the valid seeds answer the
/// fixed targets, the damaged ones are reported instead of claiming completeness). It
/// keeps the corpus honest — a corpus that only ever hits `open` failures would still
/// "pass" a smoke run — and it documents the expected semantics of each seed.
#[cfg(test)]
mod corpus {
    use super::*;
    use jarde::{
        ManifestState, MultiReleaseEntryVariant, MultiReleaseSelectionOutcome, XrefOperation,
    };
    use std::path::{Path, PathBuf};

    fn read(target: &str, name: &str) -> Vec<u8> {
        let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("corpus")
            .join(target)
            .join(name);
        std::fs::read(&path).unwrap_or_else(|error| panic!("cannot read {path:?}: {error}"))
    }

    fn standalone(input: &[u8]) -> ArtifactSnapshot {
        open(input, &limits()).expect("a valid seed must open")
    }

    /// The committed seeds are the tracked corpus; a swapped, truncated or regenerated file
    /// must fail here instead of silently weakening it. The SHA-256 of every seed is
    /// recorded in `fuzz/README.md` and printed by `corpus/generate_seeds.py`.
    #[test]
    fn the_committed_seeds_keep_their_recorded_sizes() {
        let recorded: [(&str, &str, usize); 10] = [
            ("query", "minimal-class", 185),
            ("query", "minimal-jar", 659),
            ("query", "damaged-candidate.jar", 498),
            ("query", "corrupt-payload.jar", 659),
            ("query", "truncated-class", 20),
            ("artifact_tree", "minimal-jar", 659),
            ("artifact_tree", "multi-release.jar", 866),
            ("artifact_tree", "nested.jar", 783),
            ("artifact_tree", "damaged-entry.jar", 498),
            ("artifact_tree", "truncated-jar", 80),
        ];
        for (target, name, size) in recorded {
            assert_eq!(
                read(target, name).len(),
                size,
                "seed {target}/{name} no longer has its recorded size"
            );
        }
    }

    #[test]
    fn query_seed_answers_the_fixed_symbol_invocation() {
        let snapshot = standalone(&read("query", "minimal-class"));
        let report = run_query(&snapshot, &query_request(0, &snapshot), &limits())
            .expect("a valid standalone class must answer a query");
        assert_query_contract(&report, &limits());
        assert_eq!(
            report.coverage.dimensions.artifact_structural.state,
            CoverageState::CompleteWithinSchema,
            "the whole class fits the limits: {:?}",
            report.execution
        );
        let invocations: Vec<&jarde::XrefItem> = report
            .items
            .iter()
            .filter(|item| item.consumer == Some(ConsumerKind::Invocation))
            .collect();
        assert_eq!(invocations.len(), 1, "seed semantics: {:?}", report.items);
        assert_eq!(invocations[0].operation, XrefOperation::InvokeStatic);
        assert_eq!(invocations[0].evidence.bci, Some(4));
    }

    #[test]
    fn query_seed_jar_answers_the_resource_and_constant_pool_shapes() {
        let snapshot = standalone(&read("query", "minimal-jar"));
        for shape in 0..QUERY_SHAPES {
            let report =
                run_query(&snapshot, &query_request(shape, &snapshot), &limits()).expect("report");
            assert_query_contract(&report, &limits());
        }
        // Shape 1 asks for the class the seed instantiates, which the class-candidate
        // entry answers from its `new` instruction.
        let class_report = run_query(&snapshot, &query_request(1, &snapshot), &limits()).unwrap();
        assert!(
            !class_report.items.is_empty(),
            "the `new com/example/Fuzz` must answer a class target: {class_report:?}"
        );
    }

    #[test]
    fn damaged_seeds_are_reported_instead_of_claiming_completeness() {
        // A class candidate whose payload does not start with the class magic: the ZIP and
        // the entry are intact, so the candidate rule is what rejects it.
        let snapshot = standalone(&read("query", "damaged-candidate.jar"));
        let report = run_query(&snapshot, &query_request(0, &snapshot), &limits()).unwrap();
        assert_query_contract(&report, &limits());
        assert!(
            !matches!(report.execution, ExecutionReport::Complete { .. }),
            "a class candidate whose magic is broken cannot complete: {:?}",
            report.execution
        );
        assert!(report.items.is_empty());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "query_class_candidate_malformed"),
            "the damaged candidate must be named: {:?}",
            report.diagnostics
        );

        // A stored entry whose bytes contradict its own CRC never reaches the class
        // reader: the materialization path reports the integrity failure instead.
        let snapshot = standalone(&read("query", "corrupt-payload.jar"));
        let report = run_query(&snapshot, &query_request(0, &snapshot), &limits()).unwrap();
        assert_query_contract(&report, &limits());
        assert!(!matches!(
            report.execution,
            ExecutionReport::Complete { .. }
        ));
        assert!(report.items.is_empty(), "{:?}", report.items);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "entry_integrity"),
            "the integrity failure must be named: {:?}",
            report.diagnostics
        );

        // A standalone CLASS root cut inside its header is a damaged artifact: the engine
        // reports a failure, never a crash and never completeness.
        let snapshot = standalone(&read("query", "truncated-class"));
        let report = run_query(&snapshot, &query_request(0, &snapshot), &limits()).unwrap();
        assert_query_contract(&report, &limits());
        assert!(!matches!(
            report.execution,
            ExecutionReport::Complete { .. }
        ));
    }

    #[test]
    fn artifact_tree_seeds_reach_the_tree_and_the_selection() {
        // One container with the seeded entries, no multi-release manifest.
        let snapshot = standalone(&read("artifact_tree", "minimal-jar"));
        let tree = run_tree(&snapshot, &limits()).expect("a valid jar enumerates");
        assert_tree_contract(&tree, &limits());
        assert_eq!(tree.containers.len(), 1);
        assert_eq!(tree.containers[0].entries.len(), 3);
        let selection = run_multi_release(&snapshot, &limits()).expect("selection runs");
        assert_multi_release_contract(&selection, &limits());
        assert_eq!(
            selection.containers[0].manifest.state,
            ManifestState::Inactive,
            "the seeded manifest carries no Multi-Release attribute"
        );

        // A nested jar is established as a child container of the outer snapshot.
        let snapshot = standalone(&read("artifact_tree", "nested.jar"));
        let tree = run_tree(&snapshot, &limits()).expect("a nested jar enumerates");
        assert_tree_contract(&tree, &limits());
        assert_eq!(
            tree.containers.len(),
            2,
            "the stored child jar must be established as a container: {:?}",
            tree.containers
        );

        // The versioned class of the multi-release seed wins at Java 17, and the base
        // entry is shadowed by it.
        let snapshot = standalone(&read("artifact_tree", "multi-release.jar"));
        let selection = run_multi_release(&snapshot, &limits()).expect("selection runs");
        assert_multi_release_contract(&selection, &limits());
        assert_eq!(
            selection.containers[0].manifest.state,
            ManifestState::Active
        );
        let versioned = selection.containers[0]
            .selections
            .iter()
            .find(|candidate| candidate.logical_path.0 == b"com/example/Seed.class")
            .expect("the seeded class is a selection group");
        let MultiReleaseSelectionOutcome::Selected { entry } = &versioned.outcome else {
            panic!("Java 17 must select the version 9 candidate: {versioned:?}");
        };
        assert_eq!(
            entry.raw_name.0,
            b"META-INF/versions/9/com/example/Seed.class"
        );
        assert!(selection.containers[0].entries.iter().any(|item| {
            item.variant == MultiReleaseEntryVariant::Versioned { release: 9 }
                && item.decision == jarde::MultiReleaseSelectionDecision::Selected
        }));

        // The damaged and truncated seeds stay inside the contract too.
        for name in ["damaged-entry.jar", "truncated-jar"] {
            let input = read("artifact_tree", name);
            let Some(snapshot) = open(&input, &limits()) else {
                continue;
            };
            if let Some(tree) = run_tree(&snapshot, &limits()) {
                assert_tree_contract(&tree, &limits());
            }
            if let Some(selection) = run_multi_release(&snapshot, &limits()) {
                assert_multi_release_contract(&selection, &limits());
            }
        }
    }

    /// The contract checks are the only judgement the smoke gate makes, so they must not
    /// be vacuous: a page that publishes more than its result budget, and a report that
    /// claims completeness while its execution is degraded, have to fail the check. Both
    /// cases are injected into a real report here instead of waiting for the fuzzer to
    /// find them.
    #[test]
    #[should_panic(expected = "result-items limit")]
    fn the_query_check_rejects_a_page_over_its_result_budget() {
        let snapshot = standalone(&read("query", "minimal-class"));
        let mut report = run_query(&snapshot, &query_request(0, &snapshot), &limits()).unwrap();
        let repeated = report.items[0].clone();
        report.items.push(repeated);
        report.page.returned_items = u64::try_from(report.items.len()).expect("length fits");
        let mut starved = limits();
        starved.result_items = 1;
        assert_query_contract(&report, &starved);
    }

    #[test]
    #[should_panic(expected = "must not claim artifact-structural completeness")]
    fn the_query_check_rejects_completeness_over_a_stopped_scan() {
        let snapshot = standalone(&read("query", "damaged-candidate.jar"));
        let mut report = run_query(&snapshot, &query_request(0, &snapshot), &limits()).unwrap();
        assert!(!matches!(
            report.execution,
            ExecutionReport::Complete { .. }
        ));
        report.coverage.dimensions.artifact_structural.state = CoverageState::CompleteWithinSchema;
        assert_query_contract(&report, &limits());
    }
}
