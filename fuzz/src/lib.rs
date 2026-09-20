//! Fixed requests, bounded entry points and public-contract assertions shared by the two
//! P1 fuzz targets (design 3.3) and the P2 method-analysis target (design 5.3).
//!
//! Every target treats the *whole fuzz input* as the artifact and reuses the library's own
//! entry points, reader and `Limits`; nothing here re-implements a reader, mutates a
//! request or generates class files. This module only
//!
//! * runs a small, closed set of fixed requests for every opened artifact, so file magic
//!   cannot prevent a valid CLASS or JAR from reaching a request shape,
//! * runs them through the public `Engine` entry points under hard input limits,
//! * asserts the contracts that must hold for *every* outcome, including the damaged,
//!   interrupted and cancelled ones.
//!
//! The assertion set is deliberately small and only asserts documented behaviour: a
//! libFuzzer finding must be a real contract violation, not a check that a legitimate
//! outcome (an empty page, a damaged artifact, a documented continuation seam) trips.
//! Each assertion below names the contract it enforces.

use jarde::{
    AnalysisStage, ArchiveNameBytes, ArtifactInput, ArtifactKind, ArtifactSnapshot,
    ArtifactTreeReport, Budget, ClassHeader, ClassSource, ClassTarget, CompileStatus, ConsumerKind,
    ConsumerSchema, ContainerId, ContainerOrigin, CountedBudgetDimension, CoverageState,
    DelegationPolicy, Engine, ExecutionReport, InspectionMode, JvmBytes, LayoutMode, Limits,
    LiteralValue, LoadDomain, LoadRoot, LoaderId, MethodAnalysisReport, MethodAnalysisRequest,
    MethodBodyState, ModuleMode, MultiReleasePolicy, MultiReleaseViewReport, PhysicalClassLocation,
    PhysicalDefinitionId, PhysicalMethodId, PhysicalScope, PhysicalVariant, PhysicalView, Quality,
    QueryRelation, QueryReport, QueryRequest, QueryResolution, QueryTarget, ReadReason,
    Representation, ResolutionEnvironment, RuntimeProfile, RuntimeUncertainty, RuntimeView,
    SemanticValidation, StageState, SymbolRef, SyntaxStatus, TerminationReason, UsageSnapshot,
    VerificationStatus, physical_variant_for_path,
};

/// The load root one opened fuzz input really is: a standalone CLASS input is one whole
/// definition, and a ZIP input is searched in its root container with an empty prefix. The
/// input decides which shape it is; no layout prefix is ever inferred from it.
pub fn snapshot_root(snapshot: &ArtifactSnapshot) -> LoadRoot {
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
        // Every P2 dimension stays at the fail-closed zero of the base: the committed seeds
        // exercise the P0/P1 entry points, which do not charge them yet (3.x/4.x wire them
        // up), so a non-zero value here would only weaken the bound this harness checks.
        ..Limits::default()
    }
}

/// Number of fixed request shapes exercised for every opened artifact.
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
            roots: vec![snapshot_root(snapshot)],
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

/// The query target's complete driver, also used by the corpus regression tests.
///
/// One open and at most five sequential queries bound the work per input. Each operation
/// has its own small budget; each report is checked and dropped before the next query.
/// Public query errors do not skip the remaining shapes. The observer borrows the
/// outcome so tests can check real routing/results without retaining all five reports;
/// the fuzz target supplies a no-op observer.
pub fn exercise_query(input: &[u8], mut observe: impl FnMut(u8, Option<&QueryReport>)) {
    let limits = limits();
    let Some(snapshot) = open(input, &limits) else {
        return;
    };
    for shape in 0..QUERY_SHAPES {
        let request = query_request(shape, &snapshot);
        let report = run_query(&snapshot, &request, &limits);
        if let Some(report) = &report {
            assert_query_contract(report, &limits);
        }
        observe(shape, report.as_ref());
    }
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
/// Counted dimensions are read through [`CountedBudgetDimension::ALL`] and the
/// `counted_limit`/`counted_usage` accessors rather than asserted field by field, so a
/// dimension added to the budget (as P2's six did) is covered by this bound the moment it
/// joins `ALL` — a hand-written list would keep compiling while silently skipping it. The
/// two high-water dimensions and the clock are separate slots, not counted dimensions:
/// `nested_depth` and `dependency_depth` are asserted directly, and `elapsed_millis` is
/// intentionally not checked at all — it is a measurement, not a charge, and exceeding it is
/// what terminates the run.
fn assert_usage(usage: &UsageSnapshot, limits: &Limits) {
    for dimension in CountedBudgetDimension::ALL {
        let used = usage.counted_usage(dimension);
        let allowed = limits.counted_limit(dimension);
        assert!(
            used <= allowed,
            "{dimension:?} crossed the limit: usage {used} > limit {allowed}"
        );
    }
    assert!(
        usage.nested_depth <= limits.nested_depth,
        "nested_depth crossed the limit"
    );
    assert!(
        usage.dependency_depth <= limits.dependency_depth,
        "dependency_depth crossed the limit"
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

// ---------------------------------------------------------------------------
// P2 method analysis (design 5.3)
//
// What this layer can see is the report and nothing else. The canonical CFG, the call
// contexts, the frames and the SSA table are crate-private payloads (invariant 11), so
// nothing here asserts a structure it cannot read: the checks below are the *public
// implications* of those payloads — the scheduled phase prefix and its stop rule, the
// execution that explains the stop, the one artifact that makes the report `Conservative`,
// the one phase whose completion raises `semantic_validation`, and the budget bound every
// counted dimension keeps.
// ---------------------------------------------------------------------------

/// Number of fixed request shapes exercised for every derived driver method.
///
/// The stage set and the budget class of a shape are decided by the shape number alone, so a
/// libFuzzer finding stays reproducible from the input bytes: the driver method is derived
/// from the input, but the derivation is deterministic.
pub const ANALYSIS_SHAPES: u8 = 9;

/// Budget of one method-analysis request that must be able to run the whole pipeline.
///
/// The P1 caps stay what they are (`..limits()`); the P2 dimensions are the six the IR
/// pipeline charges, and they are raised explicitly because the P1 base leaves every one of
/// them at the fail-closed zero. The two high-water marks are the small non-zero values a
/// single-root class-path environment can really reach.
pub fn analysis_limits() -> Limits {
    Limits {
        class_headers: 4,
        method_bodies: 4,
        ir_items: 64 * 1024,
        ir_edges: 64 * 1024,
        analysis_steps: 64 * 1024,
        normalization_clones: 4 * 1024,
        nested_depth: 8,
        dependency_depth: 4,
        ..limits()
    }
}

/// The same request under a budget that must stop the pipeline after the read.
///
/// One header read and one body attempt are affordable and every derived dimension is not, so
/// a shape that runs under this budget exercises the stops this build reports instead of
/// completing: a `Partial` or `Failed` phase, an execution that names the stop, and the
/// diagnostic that explains it.
pub fn analysis_limits_minimal() -> Limits {
    Limits {
        class_headers: 1,
        method_bodies: 1,
        ir_items: 8,
        ir_edges: 8,
        analysis_steps: 2,
        normalization_clones: 1,
        ..analysis_limits()
    }
}

/// The whole pipeline under an ample budget whose one tight dimension is the cloning.
///
/// A body with no `jsr` call site completes: the pass charges nothing for an empty context
/// set. A body with call sites reaches the pass that consumes them and stops at its own bound,
/// which is the one outcome whose `Partial` canonical phase and `Fallback` quality the public
/// planes cannot tell apart from a published canonical graph of a decoded prefix — so this
/// shape is what keeps that ambiguity exercised instead of assumed away.
pub fn analysis_limits_clone_starved() -> Limits {
    Limits {
        normalization_clones: 1,
        ..analysis_limits()
    }
}

/// Requested stage set of one analysis shape, as the request names it.
///
/// The shapes walk the six phase prefixes, so every stage of the pipeline is asked for by at
/// least two shapes — once where the pipeline can reach it and, for shapes 1, 2, 4 and 7,
/// under the budget that stops it ([`analysis_shape_limits`]). Shape 8 asks for the whole
/// pipeline under the clone-starved budget, and the last shape names its stages out of phase
/// order and twice: a requested set is normalized to the fixed phase order and deduplicated,
/// which is what [`normalized_stages`] spells out for the report comparison.
pub fn analysis_stages(shape: u8) -> Vec<AnalysisStage> {
    use AnalysisStage::{CanonicalCfg, Frame, LegacyNormalization, RawCfg, RawFacts, Ssa};
    match shape % ANALYSIS_SHAPES {
        0 => vec![RawFacts],
        1 => vec![RawCfg],
        2 => vec![LegacyNormalization],
        3 => vec![LegacyNormalization],
        4 => vec![CanonicalCfg],
        5 => vec![CanonicalCfg],
        6 => vec![Ssa],
        7 => vec![Ssa, Frame, Ssa],
        _ => vec![Ssa],
    }
}

/// Budget of one analysis shape: shapes 1, 2, 4 and 7 are the starved ones — the raw graph,
/// the call contexts, the canonicalization and the whole pipeline — shape 8 is the whole
/// pipeline with only the cloning dimension tight, and 0, 3, 5 and 6 run under the budget that
/// can complete the prefix they ask for.
pub fn analysis_shape_limits(shape: u8) -> Limits {
    match shape % ANALYSIS_SHAPES {
        1 | 2 | 4 | 7 => analysis_limits_minimal(),
        8 => analysis_limits_clone_starved(),
        _ => analysis_limits(),
    }
}

/// Requested phases in fixed phase order without duplicates: the normalization the engine
/// applies to every request, so a driver and a corpus test can compare a report against the
/// request it answered.
pub fn normalized_stages(stages: &[AnalysisStage]) -> Vec<AnalysisStage> {
    AnalysisStage::ALL
        .into_iter()
        .filter(|stage| stages.contains(stage))
        .collect()
}

/// One fixed method-analysis request: the shape's stage set against `method`.
pub fn analysis_request(
    shape: u8,
    snapshot: &ArtifactSnapshot,
    method: &PhysicalMethodId,
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment: analysis_environment(snapshot),
        method: method.clone(),
        stages: analysis_stages(shape),
    }
}

/// The one caller domain every analysis shape runs under.
///
/// A single class-path domain whose only root is the whole snapshot, with standard
/// multi-release selection off — the driver only ever derives `Base` definitions — and the
/// loader name the reports then publish. Java 17 is the release the P1 multi-release view
/// already uses; a method-analysis request reads no versioned entry through it.
pub fn analysis_environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("fuzz".into()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![snapshot_root(snapshot)],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: physical_view(snapshot, PhysicalScope::SnapshotAll),
            profile: RuntimeProfile {
                java_release: 17,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: domain.clone(),
        },
        domains: vec![domain],
        providers: Vec::new(),
    }
}

/// Archive entries whose header the driver probes for a body, at most.
///
/// Each probe is a bounded header read with its own budget, so the cap keeps a fuzz input
/// with many class entries from turning into an unbounded number of reads.
const PROBED_ENTRIES: usize = 8;

/// The driver method of one opened artifact, derived from the input.
///
/// The whole input is the artifact, so the method a request names has to come from the bytes:
/// the standalone `CLASS` root's header is read, or — for a `ZIP` — the snapshot's first few
/// `Base` class entries are probed, and the first member that declares a `Code` attribute wins.
/// An artifact without such a member (a resource-only jar, an unreadable header, a candidate
/// whose magic is broken) has no driver method, which is an accepted outcome: the driver has
/// nothing to ask about and returns without a report.
///
/// The definition comes from the header read itself (`ClassSource`), so the claim a request
/// makes about its class is the one the read really bound: the location, the class bytes and
/// the variant derived from the entry name, exactly as the loader compares them.
pub fn find_driver_method(
    snapshot: &ArtifactSnapshot,
    limits: &Limits,
) -> Option<PhysicalMethodId> {
    if let Some(method) = driver_method_of(snapshot, ClassTarget::Root, limits) {
        return Some(method);
    }
    let mut budget = Budget::new(limits.clone());
    let entries = Engine::new().enumerate(snapshot, &mut budget).ok()?;
    entries
        .entries
        .iter()
        .filter(|entry| entry.id.raw_name.0.ends_with(b".class"))
        .filter(|entry| {
            physical_variant_for_path(&entry.id.raw_name.0) == PhysicalVariant::Base
        })
        .take(PROBED_ENTRIES)
        .find_map(|entry| driver_method_of(snapshot, ClassTarget::Entry(entry), limits))
}

/// Reads one class header and returns its first member with a `Code` attribute.
///
/// A header this build cannot read — the wrong target kind for the snapshot, a damaged
/// candidate, a refused charge — is `None`: reaching no member is an accepted outcome, not a
/// contract violation.
fn driver_method_of(
    snapshot: &ArtifactSnapshot,
    target: ClassTarget<'_>,
    limits: &Limits,
) -> Option<PhysicalMethodId> {
    let mut budget = Budget::new(limits.clone());
    let report = Engine::new()
        .inspect_header(snapshot, target, &mut budget, InspectionMode::Strict)
        .ok()?;
    let (name, descriptor) = member_with_body(&report.inspection.header)?;
    Some(PhysicalMethodId {
        owner: definition_of(&report.source),
        name,
        descriptor,
    })
}

/// The first member of a header that declares a `Code` attribute, as the raw name and
/// descriptor a request names it by.
fn member_with_body(header: &ClassHeader) -> Option<(JvmBytes, JvmBytes)> {
    header.methods.iter().find_map(|member| {
        let has_code = member
            .attributes
            .iter()
            .any(|shell| shell.name.raw().0.as_slice() == b"Code");
        has_code.then(|| {
            (
                JvmBytes(member.name.raw().0.clone()),
                JvmBytes(member.descriptor.raw().0.clone()),
            )
        })
    })
}

/// The physical definition one materialized class is, as its location, bytes and variant.
///
/// The variant is the reader's own syntactic derivation of the entry name, which is exactly
/// what the loader compares a claimed definition against; the location and the class bytes are
/// the ones the header read itself published, so no digest is recomputed here.
fn definition_of(source: &ClassSource) -> PhysicalDefinitionId {
    let variant = match &source.location {
        PhysicalClassLocation::StandaloneRoot { .. } => PhysicalVariant::Base,
        PhysicalClassLocation::ArchiveEntry { entry } => {
            physical_variant_for_path(&entry.raw_name.0)
        }
    };
    PhysicalDefinitionId {
        location: source.location.clone(),
        class_bytes: source.class_bytes.clone(),
        variant,
    }
}

/// Runs one method-analysis request under `limits`, treating a public error as an accepted
/// outcome, and keeps the usage of that request's own budget.
///
/// A public error is not a report: `None` means the request was refused before it could be
/// answered (a snapshot the content does not provide, an empty stage set), which is the
/// documented outcome for an input the driver built wrong and not a fuzzing failure.
pub fn run_analysis(
    snapshot: &ArtifactSnapshot,
    request: &MethodAnalysisRequest,
    limits: &Limits,
) -> (Option<MethodAnalysisReport>, UsageSnapshot) {
    let mut budget = Budget::new(limits.clone());
    let report = Engine::new()
        .analyze_method(std::slice::from_ref(snapshot), request, &mut budget)
        .ok();
    (report, budget.usage())
}

/// The method-analysis target's complete driver, also used by the corpus regression tests.
///
/// One open, one method derivation (at most one header read for a standalone class or eight
/// entry probes for a zip) and at most one request per shape bound the work per input.
/// Each request has its own budget; each report is checked and dropped before the next shape.
/// A public error does not skip the remaining shapes, and neither does a request whose report
/// the engine refused: only "no driver method exists" ends the run, and that is a fact about
/// the input. The observer borrows the outcome so tests can check real stage and plane values
/// without retaining nine reports; the fuzz target supplies a no-op observer.
pub fn exercise_method_analysis(input: &[u8], mut observe: impl FnMut(u8, Option<&MethodAnalysisReport>)) {
    let limits = limits();
    let Some(snapshot) = open(input, &limits) else {
        return;
    };
    let Some(method) = find_driver_method(&snapshot, &limits) else {
        return;
    };
    for shape in 0..ANALYSIS_SHAPES {
        let shape_limits = analysis_shape_limits(shape);
        let request = analysis_request(shape, &snapshot, &method);
        let (report, _usage) = run_analysis(&snapshot, &request, &shape_limits);
        if let Some(report) = &report {
            // The report answers about the method the request named and echoes the requested
            // phases in their normalized form; both are the request half of the contract the
            // assertion set below does not see.
            assert_eq!(
                report.method, method,
                "the report must answer about the method the request named"
            );
            assert_eq!(
                report.requested_stages,
                normalized_stages(&request.stages),
                "the report echoes the requested phases, normalized to the fixed order"
            );
            assert_eq!(
                report.loader, request.environment.runtime.load_domain.loader,
                "the report names the caller domain's own loader"
            );
            assert_analysis_contract(report, &shape_limits);
        }
        observe(shape, report.as_ref());
    }
}

/// Asserts every public method-analysis contract that must hold for any outcome.
///
/// The checks are grouped by the plane they belong to and each names the contract it enforces.
/// A legitimate outcome — a rejected environment, a member without a body, a damaged body, a
/// budget stop — must pass all of them; a real contract violation must fail at least one.
pub fn assert_analysis_contract(report: &MethodAnalysisReport, limits: &Limits) {
    // -- The scheduled phases are the requested ones plus the prerequisites of the last one.
    // A request with no phase is an input error, so a report always has a non-empty requested
    // set; the fixed pass table's prerequisites are the phases before a pass, so the scheduled
    // list is the phase order's prefix that ends at the last requested phase.
    assert!(
        !report.requested_stages.is_empty(),
        "a request that asked for no phase is refused, so every report has a requested set"
    );
    assert_eq!(
        report.requested_stages,
        normalized_stages(&report.requested_stages),
        "requested phases are in fixed phase order and without duplicates"
    );
    let scheduled: Vec<AnalysisStage> = report
        .stages
        .iter()
        .map(|result| result.stage)
        .collect();
    assert!(
        !scheduled.is_empty(),
        "every request schedules at least the phase it asked for"
    );
    assert_eq!(
        scheduled,
        AnalysisStage::ALL[..scheduled.len()].to_vec(),
        "the scheduled phases are the fixed phase order's prefix"
    );
    let last_requested = *report
        .requested_stages
        .last()
        .expect("the requested set is non-empty");
    let scheduled_for_last = AnalysisStage::ALL
        .iter()
        .position(|stage| *stage == last_requested)
        .expect("a requested phase is one of the fixed phases")
        + 1;
    assert_eq!(
        scheduled.len(),
        scheduled_for_last,
        "the scheduled prefix ends at the last requested phase, so its length is decided by \
         the request alone"
    );

    // -- The stop rule of the phase prefix: a phase behind a stop was never reached.
    //
    // Every stop leaves the phases it did not reach `NotPerformed`, so `NotPerformed` is a
    // trailing region and a `Failed` phase is the last performed one. A `Partial` phase is not
    // a stop of this kind: a body whose decode stopped early keeps a `Partial` phase for every
    // pass that ran over the reliable prefix, without breaking the pipeline (the stop already
    // happened inside `raw_facts`).
    let states: Vec<&StageState> = report.stages.iter().map(|result| &result.state).collect();
    let mut saw_not_performed = false;
    for state in &states {
        if **state == StageState::NotPerformed {
            saw_not_performed = true;
        } else {
            assert!(
                !saw_not_performed,
                "a phase behind a stop must stay `NotPerformed`: {:?}",
                report.stages
            );
        }
        assert!(
            **state != StageState::NotRequested,
            "a scheduled phase is performed or explicitly not performed, never `NotRequested`"
        );
    }
    for (index, state) in states.iter().enumerate() {
        if let StageState::Failed { .. } = state {
            assert!(
                states[index + 1..]
                    .iter()
                    .all(|state| **state == StageState::NotPerformed),
                "the phases behind a failed phase must stay `NotPerformed`: {:?}",
                report.stages
            );
        }
    }

    // -- The execution plane explains the phase results, and every stop is named.
    let all_completed = states.iter().all(|state| **state == StageState::Completed);
    let any_performed_stop = states.iter().any(|state| {
        matches!(state, StageState::Partial | StageState::Failed { .. })
    });
    if all_completed {
        assert!(
            matches!(report.execution, ExecutionReport::Complete { .. }),
            "a request whose every scheduled phase completed completed: {:?}",
            report.execution
        );
    }
    if any_performed_stop {
        assert!(
            !matches!(report.execution, ExecutionReport::Complete { .. }),
            "a phase that stopped cannot be part of a completed request: {:?}",
            report.execution
        );
    }
    let diagnostic_codes: Vec<&str> = report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect();
    if !matches!(report.execution, ExecutionReport::Complete { .. }) {
        assert!(
            !report.diagnostics.is_empty(),
            "a stopped request names the stop: {:?}",
            report.execution
        );
        if let ExecutionReport::Failed { reason, .. } = &report.execution {
            let code = termination_code(reason);
            assert!(
                diagnostic_codes.contains(&code.as_str()),
                "the failure `{code}` is reported as a diagnostic: {diagnostic_codes:?}"
            );
        }
    }
    // A failed phase is explained by a diagnostic under its own code, and the one failure that
    // happens without a failed phase is the environment the validator rejected: no phase ran,
    // and the capability itself is the named reason.
    for state in &states {
        if let StageState::Failed { code } = state {
            assert!(
                diagnostic_codes.contains(&code.as_str()),
                "the failed phase `{code}` is reported as a diagnostic: {diagnostic_codes:?}"
            );
        }
    }
    if matches!(report.execution, ExecutionReport::Failed { .. })
        && !states
            .iter()
            .any(|state| matches!(state, StageState::Failed { .. }))
    {
        assert!(
            !report.environment_problems.is_empty(),
            "a failure without a failed phase is a rejected environment: {:?}",
            report.execution
        );
        assert!(
            states.iter().all(|state| **state == StageState::NotPerformed),
            "a rejected environment performs no phase: {:?}",
            report.stages
        );
    }
    // Every environment problem is reported twice, structured and as a diagnostic of the same
    // closed-set code; the structured problem itself is not an error, it is part of the report.
    for problem in &report.environment_problems {
        assert!(
            diagnostic_codes.contains(&problem.code.as_str()),
            "the environment problem `{:?}` is reported as a diagnostic of the same code: \
             {diagnostic_codes:?}",
            problem.code
        );
    }

    // -- The body and coverage planes describe the read.
    if report.coverage.artifact_structural.state != CoverageState::NotRequested {
        assert_eq!(
            report.body,
            MethodBodyState::Present,
            "a method-code coverage plane exists only for a located body"
        );
    }
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::NotRequested,
        "a method-analysis request resolves no runtime symbol"
    );
    assert_eq!(
        report.coverage.dynamic_analysis.state,
        CoverageState::NotRequested,
        "a method-analysis request performs no dynamic analysis"
    );
    for range in report
        .coverage
        .artifact_structural
        .scanned
        .iter()
        .chain(&report.coverage.artifact_structural.skipped)
    {
        assert!(
            range.start <= range.end,
            "a coverage range cannot end before it starts: {range:?}"
        );
    }
    if let MethodBodyState::DeclaredWithoutBody { .. } = report.body {
        assert!(
            states.iter().all(|state| **state == StageState::NotPerformed),
            "no phase can run on a member that declares no body: {:?}",
            report.stages
        );
        assert!(
            matches!(report.execution, ExecutionReport::Complete { .. }),
            "a member without a body is a fact of the declaration, not a stop: {:?}",
            report.execution
        );
    }

    // -- The product planes of this build, and the one phase that raises a semantic claim.
    //
    // `quality` is about the artifact a run *published*: the canonical CFG is that artifact, so
    // a phase that completed published one and its report is `Conservative`, while a phase that
    // never ran or failed published nothing and its report is `Fallback`. A `Partial` canonical
    // phase is the one state the public planes cannot decide — it is both "the canonical graph
    // of a reliable decoded prefix was published" and "the pass stopped at its own bound without
    // publishing" — so neither direction is asserted for it; the run names which of the two
    // happened with the diagnostic code of the bound. `semantic_validation` is the run's own
    // evidence: only the `ssa` phase that completed raises `LocalInvariants`, and a per-sample
    // fixture differential is not a claim a production request makes.
    let canonical_state = report
        .stages
        .iter()
        .find(|result| result.stage == AnalysisStage::CanonicalCfg)
        .map(|result| &result.state);
    match canonical_state {
        Some(StageState::Completed) => assert_eq!(
            report.quality,
            Quality::Conservative,
            "a completed canonical phase published the artifact that makes a report \
             `Conservative`"
        ),
        Some(StageState::NotPerformed | StageState::Failed { .. }) | None => assert_eq!(
            report.quality,
            Quality::Fallback,
            "a run that never published a canonical graph is `Fallback`: {:?}",
            report.stages
        ),
        Some(StageState::Partial) | Some(StageState::NotRequested) => {}
    }
    if report.quality == Quality::Conservative {
        assert!(
            matches!(
                canonical_state,
                Some(StageState::Completed | StageState::Partial)
            ),
            "only the phase that publishes the canonical graph can make a report \
             `Conservative`: {:?}",
            report.stages
        );
    }
    let ssa_completed = report
        .stages
        .iter()
        .any(|result| result.stage == AnalysisStage::Ssa && result.state == StageState::Completed);
    if ssa_completed {
        assert_eq!(
            report.semantic_validation,
            SemanticValidation::LocalInvariants,
            "the completed `ssa` phase is the evidence of the local invariants"
        );
    } else {
        assert_eq!(
            report.semantic_validation,
            SemanticValidation::Unproven,
            "no completed `ssa` phase, no semantic evidence"
        );
    }
    assert_eq!(report.representation, Representation::Bytecode);
    assert_eq!(
        report.verification,
        VerificationStatus::NotPerformed,
        "analyzing a method is not verifying it"
    );
    assert_eq!(report.syntax_status, SyntaxStatus::NotJava);
    assert_eq!(report.compile_status, CompileStatus::NotAttempted);

    // -- The reads and the budget of the request.
    //
    // A method-analysis request reads its own class definition and nothing else, under the one
    // reason that may name a body read; a refused charge and a rejected environment record
    // nothing, so the record count is bounded by the charged header attempts.
    let execution_usage = usage_of(&report.execution);
    assert!(
        u64::try_from(report.reads.len()).expect("a read count fits u64")
            <= execution_usage.class_headers,
        "a request records at most one read per charged header attempt"
    );
    for read in &report.reads {
        assert_eq!(
            read.reason,
            ReadReason::DriverMethodBody,
            "the only reason a method-analysis request publishes is the driver method's body: \
             {:?}",
            report.reads
        );
    }
    assert_usage(execution_usage, limits);
}

/// Stable code of one termination reason, the code a `Failed` phase states.
fn termination_code(reason: &TerminationReason) -> String {
    match reason {
        TerminationReason::Error { code } | TerminationReason::Unsupported { code } => code.clone(),
        TerminationReason::BudgetExceeded { dimension } => format!(
            "budget_exceeded_{}",
            jarde::artifact::budget_dimension_code(*dimension)
        ),
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
    fn query_driver_exercises_every_shape_for_class_and_jar() {
        for name in ["minimal-class", "minimal-jar"] {
            let mut visited = Vec::new();
            exercise_query(&read("query", name), |shape, report| {
                visited.push(shape);
                let report = report.expect("every valid seed shape returns a report");
                assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
                assert!(!report.items.is_empty(), "seed {name}, shape {shape}");
                let expected_relation = match shape {
                    2 => jarde::QueryRelation::LiteralValue,
                    3 => jarde::QueryRelation::ConstantPoolContains,
                    _ => jarde::QueryRelation::MentionsSymbol,
                };
                assert_eq!(report.relation, expected_relation);
                if shape == 4 {
                    assert_eq!(
                        report.coverage.unsupported_categories,
                        vec![ConsumerKind::Verification, ConsumerKind::Debug]
                    );
                    assert_ne!(
                        report.coverage.dimensions.artifact_structural.state,
                        CoverageState::CompleteWithinSchema
                    );
                }
            });
            // A literal list pins the corpus's expected coverage independently of the
            // driver/shape count. Restoring the magic-byte selector fails this assertion.
            assert_eq!(visited, [0, 1, 2, 3, 4], "seed {name}");
        }
    }

    #[test]
    fn query_driver_checks_every_shape_for_damaged_seeds() {
        for name in [
            "damaged-candidate.jar",
            "corrupt-payload.jar",
            "truncated-class",
        ] {
            let mut visited = Vec::new();
            exercise_query(&read("query", name), |shape, report| {
                visited.push(shape);
                let report = report.expect("an opened damaged seed reports its failure");
                assert!(!matches!(
                    report.execution,
                    ExecutionReport::Complete { .. }
                ));
                assert_eq!(
                    report.coverage.dimensions.artifact_structural.state,
                    CoverageState::Partial,
                    "seed {name}, shape {shape}"
                );
            });
            assert_eq!(visited, [0, 1, 2, 3, 4], "seed {name}");
        }
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

    /// The method-analysis seeds, in the order the generator writes them, with the size the
    /// tracked file must keep: a swapped, truncated or regenerated seed has to fail here
    /// instead of silently weakening the corpus.
    const METHOD_ANALYSIS_SEEDS: [(&str, usize); 6] = [
        ("jsr-ret.class", 258),
        ("legacy-clone.class", 131),
        ("wide-switch.class", 194),
        ("exception-overlap.class", 214),
        ("exception-overlap-mixed.class", 189),
        ("wide-switch.jar", 334),
    ];

    /// One seed's opened snapshot and the method the driver derives from it.
    fn derived_method(name: &str) -> (ArtifactSnapshot, PhysicalMethodId) {
        let snapshot = standalone(&read("method_analysis", name));
        let method = find_driver_method(&snapshot, &limits())
            .unwrap_or_else(|| panic!("seed {name} has a member with a body"));
        (snapshot, method)
    }

    /// One explicit method-analysis request under the ample budget, with its usage.
    fn analysis_of(
        snapshot: &ArtifactSnapshot,
        method: &PhysicalMethodId,
        stages: Vec<AnalysisStage>,
    ) -> (MethodAnalysisReport, UsageSnapshot) {
        let request = MethodAnalysisRequest {
            environment: analysis_environment(snapshot),
            method: method.clone(),
            stages,
        };
        let (report, usage) = run_analysis(snapshot, &request, &analysis_limits());
        let report = report.expect("a legal request is answered, not raised");
        assert_analysis_contract(&report, &analysis_limits());
        (report, usage)
    }

    /// The stage states of one report, in scheduled order.
    fn stage_states(report: &MethodAnalysisReport) -> Vec<StageState> {
        report
            .stages
            .iter()
            .map(|result| result.state.clone())
            .collect()
    }

    #[test]
    fn the_committed_method_analysis_seeds_keep_their_recorded_sizes() {
        for (name, size) in METHOD_ANALYSIS_SEEDS {
            assert_eq!(
                read("method_analysis", name).len(),
                size,
                "seed method_analysis/{name} no longer has its recorded size"
            );
        }
    }

    /// The driver derives a method from the input and every fixed shape is really asked.
    ///
    /// A seed whose first member with a body is the shape it is named after is the point of
    /// the corpus, so this test pins the derivation for all five seeds; the visited shapes are
    /// asserted as a list because a driver that skipped a shape would still "pass" a smoke run
    /// that only checked the first one.
    #[test]
    fn the_method_analysis_driver_reaches_every_shape_for_every_seed() {
        let derived: [(&str, &[u8], &[u8]); 6] = [
            ("jsr-ret.class", b"<init>", b"()V"),
            ("legacy-clone.class", b"shared", b"()V"),
            ("wide-switch.class", b"pick", b"(I)I"),
            ("exception-overlap.class", b"guarded", b"(I)I"),
            ("exception-overlap-mixed.class", b"guarded", b"(I)I"),
            ("wide-switch.jar", b"pick", b"(I)I"),
        ];
        for (name, expected_name, expected_descriptor) in derived {
            let (snapshot, method) = derived_method(name);
            assert_eq!(method.name.0, expected_name, "seed {name}");
            assert_eq!(method.descriptor.0, expected_descriptor, "seed {name}");
            let snapshot_id = snapshot.id().clone();

            let mut visited = Vec::new();
            let mut starved = 0_u32;
            let mut completed = 0_u32;
            exercise_method_analysis(&read("method_analysis", name), |shape, report| {
                visited.push(shape);
                let report = report.expect("a derived method answers every shape");
                // The driver already asserted the contract; this is the corpus's own reading
                // of the two budget classes it needs to stay non-vacuous.
                assert_eq!(
                    report.method.owner.snapshot(),
                    &snapshot_id,
                    "seed {name}, shape {shape}: the report is about the derived snapshot"
                );
                if matches!(report.execution, ExecutionReport::Complete { .. }) {
                    completed += 1;
                }
                if report
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code.starts_with("budget_exceeded_"))
                {
                    starved += 1;
                }
            });
            assert_eq!(
                visited,
                [0, 1, 2, 3, 4, 5, 6, 7, 8],
                "seed {name}: every fixed shape is asked, in order"
            );
            assert!(
                starved >= 1,
                "seed {name}: the minimal budget of shapes 1, 2, 4 and 7 must really stop a run"
            );
            assert!(
                completed >= 3,
                "seed {name}: the ample budget must really complete the shapes it affords"
            );
        }
    }

    /// The `jsr`/`ret` seed: the committed historical fixture, copied byte for byte.
    ///
    /// Its first member with a body is `<init>`, which the driver derives and completes. The
    /// `jsr`/`ret` method is asked for explicitly: `finallyPath(I)I` is the body the 45 dialect
    /// compiles `finally` into, so its whole pipeline completing and its clone dimension being
    /// charged is the evidence this seed exists for.
    #[test]
    fn the_historical_jsr_fixture_completes_and_charges_the_cloning_dimension() {
        let (snapshot, derived) = derived_method("jsr-ret.class");
        let (report, _) = analysis_of(&snapshot, &derived, vec![AnalysisStage::Ssa]);
        assert_eq!(stage_states(&report), vec![StageState::Completed; 6]);
        assert!(report.diagnostics.is_empty());

        let finally_path = PhysicalMethodId {
            owner: derived.owner.clone(),
            name: JvmBytes(b"finallyPath".to_vec()),
            descriptor: JvmBytes(b"(I)I".to_vec()),
        };
        let (report, usage) = analysis_of(&snapshot, &finally_path, vec![AnalysisStage::Ssa]);
        assert_eq!(
            stage_states(&report),
            vec![StageState::Completed; 6],
            "the historical `jsr`/`ret` finally completes every phase: {:?}",
            report.diagnostics
        );
        assert!(
            usage.normalization_clones > 0,
            "the 45 dialect really clones its shared subroutine: {usage:?}"
        );
        assert_eq!(report.quality, Quality::Conservative);
        assert_eq!(
            report.semantic_validation,
            SemanticValidation::LocalInvariants
        );
        assert_eq!(
            report.coverage.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );
    }

    /// The shared-subroutine seed: two `jsr` sites, one body, one `ret`.
    ///
    /// The driver derives the `jsr` body itself here, so the seed reaches the call contexts and
    /// the clone normalization through the target's own path, not only through a hand-built
    /// request. A run that stops before the canonicalization charges no clone, which is what
    /// keeps the charged dimension tied to the pass that really clones.
    #[test]
    fn the_legacy_clone_seed_charges_clones_only_when_the_normalization_ran() {
        let (snapshot, method) = derived_method("legacy-clone.class");
        let (report, usage) = analysis_of(&snapshot, &method, vec![AnalysisStage::Ssa]);
        assert_eq!(stage_states(&report), vec![StageState::Completed; 6]);
        assert!(report.diagnostics.is_empty());
        assert!(
            usage.normalization_clones > 0,
            "the shared subroutine is cloned per call site: {usage:?}"
        );
        assert_eq!(report.quality, Quality::Conservative);
        assert_eq!(
            report.semantic_validation,
            SemanticValidation::LocalInvariants
        );

        let (report, raw) = analysis_of(&snapshot, &method, vec![AnalysisStage::RawCfg]);
        assert_eq!(
            stage_states(&report),
            vec![StageState::Completed, StageState::Completed]
        );
        assert_eq!(
            raw.normalization_clones, 0,
            "a run that stops before the canonicalization clones nothing"
        );
    }

    /// The clone-starved shape: the shared subroutine's clones are what stops the pipeline.
    ///
    /// The canonical phase is `Partial` and the quality is `Fallback` here, which is exactly the
    /// outcome the public planes cannot tell apart from "the canonical graph of a decoded prefix
    /// was published" — the report names which one happened with the bound's own code, and the
    /// contract check must accept the state without inventing either claim.
    #[test]
    fn the_clone_starved_shape_stops_the_canonicalization_at_its_bound() {
        let (snapshot, method) = derived_method("legacy-clone.class");
        let shape = 8;
        let limits = analysis_shape_limits(shape);
        assert_eq!(limits.normalization_clones, 1);
        let request = analysis_request(shape, &snapshot, &method);
        let (report, usage) = run_analysis(&snapshot, &request, &limits);
        let report = report.expect("a legal request is answered");
        assert_analysis_contract(&report, &limits);

        assert_eq!(
            stage_states(&report),
            vec![
                StageState::Completed,
                StageState::Completed,
                StageState::Completed,
                StageState::Partial,
                StageState::NotPerformed,
                StageState::NotPerformed,
            ],
            "the cloning stops the canonical phase and the phases behind it: {:?}",
            report.diagnostics
        );
        assert_eq!(
            report.quality,
            Quality::Fallback,
            "a bound this pass ran into publishes no canonical graph"
        );
        assert_eq!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::Error {
                    code: "ir_legacy_normalization_unbounded".to_string(),
                },
                usage,
            },
            "the stop is the pass's own code, with the usage of this request"
        );
        assert!(report.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "ir_legacy_normalization_unbounded"
        }));
    }

    /// The wide/switch seed: both switch forms and `wide` local access in one body.
    ///
    /// The decoded body length is asserted, so the seed cannot be replaced by an empty class
    /// that happens to complete: 80 bytes is the seeded body, and its 301 local slots are what
    /// makes the wide form the only way to address the tail of the window.
    #[test]
    fn the_wide_switch_seed_completes_over_both_switch_forms() {
        let (snapshot, method) = derived_method("wide-switch.class");
        let (report, usage) = analysis_of(&snapshot, &method, vec![AnalysisStage::Ssa]);
        assert_eq!(stage_states(&report), vec![StageState::Completed; 6]);
        assert!(report.diagnostics.is_empty());
        assert_eq!(
            usage.code_bytes, 80,
            "the derived method's body is the seeded one"
        );
        assert_eq!(
            report.coverage.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );
        assert_eq!(
            report.semantic_validation,
            SemanticValidation::LocalInvariants
        );
    }

    /// The overlap seeds: three exception records over one throw site, in two flavours.
    ///
    /// The first flavour enters each handler with one exception type and completes the whole
    /// pipeline. The second one enters the first handler with a typed record and a catch-all
    /// at once; the body still reads completely, and what the pipeline makes of the mixed
    /// entry state is reported through the same planes as any other outcome — which outcome it
    /// is, is the engine's business and is not frozen here, because this seed's job is to keep
    /// the mixed merge reachable for the fuzzer.
    #[test]
    fn the_exception_overlap_seeds_cover_both_handler_entry_merges() {
        let (snapshot, method) = derived_method("exception-overlap.class");
        let (report, usage) = analysis_of(&snapshot, &method, vec![AnalysisStage::Ssa]);
        assert_eq!(
            stage_states(&report),
            vec![StageState::Completed; 6],
            "the overlapping table completes every phase: {:?}",
            report.diagnostics
        );
        assert_eq!(usage.code_bytes, 17);
        assert_eq!(
            report.coverage.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );

        let (snapshot, method) = derived_method("exception-overlap-mixed.class");
        let (report, usage) = analysis_of(&snapshot, &method, vec![AnalysisStage::Ssa]);
        assert_eq!(
            report.body,
            MethodBodyState::Present,
            "the mixed table is a body like any other"
        );
        assert_eq!(usage.code_bytes, 17);
        assert_eq!(
            report.coverage.artifact_structural.state,
            CoverageState::CompleteWithinSchema,
            "the mixed table is read completely: {:?}",
            report.diagnostics
        );
    }

    /// The ZIP seed: the same body as the standalone class, derived through the entry path.
    ///
    /// A `ZIP` has no class root, so the driver enumerates the snapshot and probes the headers
    /// of its `Base` class entries; the definition it derives is the entry's own identity, and
    /// the request has to bind it through the loader's search the same way the standalone
    /// claim does.
    #[test]
    fn the_zip_seed_derives_an_archive_entry_and_completes_it() {
        let (snapshot, method) = derived_method("wide-switch.jar");
        assert!(
            matches!(
                method.owner.location,
                PhysicalClassLocation::ArchiveEntry { .. }
            ),
            "the derivation reads the entry's own bytes: {method:?}"
        );
        assert_eq!(method.name.0, b"pick");
        let (report, usage) = analysis_of(&snapshot, &method, vec![AnalysisStage::Ssa]);
        assert_eq!(
            stage_states(&report),
            vec![StageState::Completed; 6],
            "the entry-derived body completes like the standalone one: {:?}",
            report.diagnostics
        );
        assert_eq!(usage.code_bytes, 80, "the entry's body is the seeded one");
        assert_eq!(
            report.coverage.artifact_structural.state,
            CoverageState::CompleteWithinSchema
        );
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

    /// The method-analysis check has to fail on a phase that ran after a stop, because a report
    /// that reaches a phase behind a stop is exactly the kind of engine bug the check exists
    /// for. The real report of a starved run leaves the trailing phases `NotPerformed`; one of
    /// them published as completed must fail the check.
    #[test]
    #[should_panic(expected = "behind a stop")]
    fn the_analysis_check_rejects_a_phase_after_a_stop() {
        let (snapshot, derived) = derived_method("legacy-clone.class");
        let request = MethodAnalysisRequest {
            environment: analysis_environment(&snapshot),
            method: derived,
            stages: vec![AnalysisStage::Ssa],
        };
        let (report, _) = run_analysis(&snapshot, &request, &analysis_limits_minimal());
        let mut report = report.expect("a legal request is answered");
        let last = report
            .stages
            .last_mut()
            .expect("the request schedules a phase");
        assert_eq!(
            last.state,
            StageState::NotPerformed,
            "the starved budget stops before the last phase"
        );
        last.state = StageState::Completed;
        assert_analysis_contract(&report, &analysis_limits_minimal());
    }

    /// Quality is a claim about the artifact a run published, so a `Conservative` report whose
    /// canonical phase never published one has to fail the check — otherwise the one plane
    /// that says "a canonical CFG exists" would be unfalsifiable.
    #[test]
    #[should_panic(expected = "never published a canonical graph")]
    fn the_analysis_check_rejects_conservative_quality_without_a_published_graph() {
        let (snapshot, derived) = derived_method("legacy-clone.class");
        let request = MethodAnalysisRequest {
            environment: analysis_environment(&snapshot),
            method: derived,
            stages: vec![AnalysisStage::RawFacts],
        };
        let (report, _) = run_analysis(&snapshot, &request, &analysis_limits());
        let mut report = report.expect("a legal request is answered");
        assert_eq!(report.quality, Quality::Fallback);
        report.quality = Quality::Conservative;
        assert_analysis_contract(&report, &analysis_limits());
    }

    /// A stopped run must name the stop, so a report whose execution is degraded and whose
    /// diagnostics were dropped has to fail the check: the coupling between the execution plane
    /// and the diagnostics is what makes a stop readable instead of silent.
    #[test]
    #[should_panic(expected = "names the stop")]
    fn the_analysis_check_rejects_a_stop_without_a_diagnostic() {
        let (snapshot, derived) = derived_method("legacy-clone.class");
        let request = MethodAnalysisRequest {
            environment: analysis_environment(&snapshot),
            method: derived,
            stages: vec![AnalysisStage::Ssa],
        };
        let (report, _) = run_analysis(&snapshot, &request, &analysis_limits_minimal());
        let mut report = report.expect("a legal request is answered");
        assert!(!matches!(report.execution, ExecutionReport::Complete { .. }));
        report.diagnostics.clear();
        assert_analysis_contract(&report, &analysis_limits_minimal());
    }

    /// The scheduled prefix is decided by the request: a report whose scheduled list does not
    /// reach the last requested phase has to fail the check, so the scheduled length cannot
    /// silently disagree with the phases the request asked for.
    #[test]
    #[should_panic(expected = "ends at the last requested phase")]
    fn the_analysis_check_rejects_a_schedule_that_stops_before_a_requested_phase() {
        let (snapshot, derived) = derived_method("legacy-clone.class");
        let request = MethodAnalysisRequest {
            environment: analysis_environment(&snapshot),
            method: derived,
            stages: vec![AnalysisStage::Ssa],
        };
        let (report, _) = run_analysis(&snapshot, &request, &analysis_limits());
        let mut report = report.expect("a legal request is answered");
        assert_eq!(report.requested_stages, vec![AnalysisStage::Ssa]);
        // The six scheduled phases are the prerequisites of `ssa`; claiming the request asked
        // only for the first phase leaves the schedule longer than the request can explain.
        report.requested_stages = vec![AnalysisStage::RawFacts];
        assert_analysis_contract(&report, &analysis_limits());
    }
}
