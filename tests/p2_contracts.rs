//! P2 1.1 acceptance: the public contract is compilable, honest and enforced.
//!
//! This slice delivers no resolver and no IR. What it has to prove through the public API
//! only is therefore:
//!
//! 1. the environment validator reports every declared violation it can decide without
//!    reading a byte, under the closed `EnvironmentProblemCode` set, and never turns one
//!    into a unique resolution (`state = None`) — including the caller-loader check, which is
//!    decided by the request-level entries because the caller identity is part of the request
//!    and not of the environment,
//! 2. the three entry points reject request-level mismatches as input errors and answer a
//!    request they cannot perform with the honest unavailable state: `NotPerformed` / `Failed {
//!    Unsupported }` / three-dimensional `NotRequested` coverage and zero counted usage —
//!    which is the state of a rejected environment for the resolution entry, of the declaration
//!    query (2.4) and of the method analysis (3.x), while 2.1–2.3 perform real lookups,
//! 3. the result planes (representation, quality, syntax, compile, semantic evidence,
//!    verification, coverage, execution) are reported side by side and none is inferred
//!    from another,
//! 4. the serde shape of the new types is pinned, including the two places where the
//!    written contract cannot be spelled directly with serde 1.0.229,
//! 5. A17 holds at source level: every guarded file (all of `crates/jarde-query/src/query.rs`,
//!    the module added beside it in P4 — `crates/jarde-query/src/plugin.rs` — and
//!    `crates/jarde-query/src/xref/**/*.rs`, enumerated from the directory so a new file
//!    counts too) is free of P2 module paths, module aliases, glob imports and P2 type names —
//!    the type names are derived from the P2 modules themselves rather than listed by hand —
//!    while `crates/jarde-jvm/src/engine.rs`, the module allowed to call the P2 entries, is
//!    flagged by the same detector, so the guard is not vacuously true,
//! 6. every `ALL` list is exactly the variant set its enum declares, read from the enum's own
//!    source: a variant left out of `ALL` is either a compile error (the exhaustive matches)
//!    or a failing comparison, and for `AnalysisStage` it would be a silently dropped request,
//! 7. the existing P1 physical query keeps its result identity, coordinates and billing.
//!
//! Environment negatives are exercised through **all three** entry points, because the
//! rejected environment has to reach the report of each one under the same codes; a single
//! entry silently dropping `environment_problems` would otherwise stay invisible. The
//! negatives that mix codes check the problem order as well, so a report that reorders or
//! drops one diagnostic cannot pass by carrying a single code.

use jarde::*;
use std::path::{Path, PathBuf};

const HISTORICAL: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 24,
        archive_entries: 10_000,
        entry_bytes: 1 << 24,
        read_bytes: 1 << 24,
        class_bytes: 1 << 24,
        attribute_bytes: 1 << 24,
        code_bytes: 1 << 24,
        result_items: 1 << 24,
        output_bytes: 1 << 24,
        nested_depth: 8,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

/// Every counted limit is zero, so *any* byte read, item or output byte a P2 entry point
/// charged would fail the request instead of producing a report.
///
/// The wall clock is zero as well, so a budget stop under these limits can be the clock; a test
/// that wants to name the dimension a *work* charge refused funds the clock explicitly.
fn zero_limits() -> Limits {
    Limits {
        input_bytes: 0,
        archive_entries: 0,
        entry_bytes: 0,
        read_bytes: 0,
        class_bytes: 0,
        attribute_bytes: 0,
        code_bytes: 0,
        result_items: 0,
        output_bytes: 0,
        nested_depth: 0,
        elapsed_millis: 0,
        ..Limits::default()
    }
}

/// Every counted limit is zero as well, so the dimension a *work* charge refused is
/// deterministic instead of possibly being the wall clock.
fn zero_work_limits() -> Limits {
    Limits {
        elapsed_millis: u64::MAX,
        ..zero_limits()
    }
}

/// The funded limits with the dimensions a method-analysis run charges on top of them: the two
/// read attempts, the IR storage items and edges, and the analysis steps (3.3). The byte
/// dimensions the reader charges (`class`/`attribute`/`code`) come from [`limits`].
fn analysis_limits() -> Limits {
    Limits {
        class_headers: 10,
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        ..limits()
    }
}

struct Fixture {
    snapshot: ArtifactSnapshot,
    definition: PhysicalDefinitionId,
    /// `HistoricalControlFlow.finallyPath(I)I`
    method: PhysicalMethodId,
}

/// Opens the historical fixture and derives the physical identity the way the engine does.
///
/// Opening and hashing are physical work and pay their own budget; the P2 requests below
/// use fresh budgets, so their counted usage starts at zero.
fn fixture() -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(HISTORICAL.to_vec()), &mut budget)
        .expect("the historical fixture opens as a standalone CLASS");
    assert_eq!(snapshot.kind(), ArtifactKind::StandaloneClass);
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(HISTORICAL).to_hex().to_string()),
            length: u64::try_from(HISTORICAL.len()).expect("fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = method_id(&definition, b"finallyPath", b"(I)I");
    Fixture {
        snapshot,
        definition,
        method,
    }
}

fn method_id(
    definition: &PhysicalDefinitionId,
    name: &[u8],
    descriptor: &[u8],
) -> PhysicalMethodId {
    PhysicalMethodId {
        owner: definition.clone(),
        name: bytes(name),
        descriptor: bytes(descriptor),
    }
}

fn loader(name: &str) -> LoaderId {
    LoaderId(name.to_string())
}

fn domain_with(
    loader: &LoaderId,
    parent: Option<LoaderId>,
    module_mode: ModuleMode,
    delegation: DelegationPolicy,
    roots: Vec<LoadRoot>,
) -> LoadDomain {
    LoadDomain {
        loader: loader.clone(),
        parent_loader: parent,
        delegation,
        roots,
        module_mode,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    }
}

fn domain(loader: &LoaderId, parent: Option<LoaderId>, roots: Vec<LoadRoot>) -> LoadDomain {
    domain_with(
        loader,
        parent,
        ModuleMode::ClassPath,
        DelegationPolicy::ParentFirst,
        roots,
    )
}

fn environment(
    fixture: &Fixture,
    caller: LoadDomain,
    domains: Vec<LoadDomain>,
    providers: Vec<HeaderProvider>,
) -> ResolutionEnvironment {
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: fixture.snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: caller,
        },
        domains,
        providers,
    }
}

/// `app` (parent `platform`, rooted at the fixture) plus the empty `platform` domain, with
/// one provider that names the caller's root.
fn healthy_environment(fixture: &Fixture) -> ResolutionEnvironment {
    let app = loader("app");
    let platform = loader("platform");
    let roots = vec![LoadRoot::Snapshot {
        snapshot: fixture.snapshot.id().clone(),
    }];
    let provider = HeaderProvider {
        id: ProviderId("app-headers".to_string()),
        roots: roots.clone(),
    };
    environment(
        fixture,
        domain(&app, Some(platform.clone()), roots.clone()),
        vec![
            domain(&app, Some(platform.clone()), roots),
            domain(&platform, None, Vec::new()),
        ],
        vec![provider],
    )
}

/// `java/lang/Object.<init>()V`, the symbol the fixture's `<init>` really invokes.
fn constructor() -> SymbolRef {
    SymbolRef::Method {
        owner: bytes(b"java/lang/Object"),
        name: bytes(b"<init>"),
        descriptor: bytes(b"()V"),
    }
}

fn request(environment: ResolutionEnvironment) -> ResolutionRequest {
    ResolutionRequest {
        environment,
        target: constructor(),
        use_kind: ReferenceUse::InvokeSpecial,
        caller: CallerContext {
            loader: loader("app"),
            enclosing: None,
        },
        dispatch: None,
    }
}

/// The same request shape for a class symbol, with the caller loader as a parameter: the
/// resolution slice looks class names up (2.1), and the caller identity is what the newest
/// environment check compares against the environment's own caller domain.
fn class_request(
    environment: ResolutionEnvironment,
    caller: &LoaderId,
    class_name: &[u8],
) -> ResolutionRequest {
    ResolutionRequest {
        environment,
        target: SymbolRef::Class {
            owner: bytes(class_name),
        },
        use_kind: ReferenceUse::ClassReference,
        caller: CallerContext {
            loader: caller.clone(),
            enclosing: None,
        },
        dispatch: None,
    }
}

fn declaration_query(fixture: &Fixture, environment: ResolutionEnvironment) -> DeclarationRefQuery {
    DeclarationRefQuery {
        environment,
        declaration: ResolvedMemberRef {
            loader: loader("app"),
            definition: fixture.definition.clone(),
            member: SymbolRef::Method {
                owner: bytes(b"HistoricalControlFlow"),
                name: bytes(b"<init>"),
                descriptor: bytes(b"()V"),
            },
        },
        scope: PhysicalScope::SnapshotAll,
        consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
        max_items: 0,
    }
}

fn analysis_request(
    fixture: &Fixture,
    environment: ResolutionEnvironment,
    stages: Vec<AnalysisStage>,
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment,
        method: fixture.method.clone(),
        stages,
    }
}

/// Codes of the problems a validation found, in report order.
fn problem_codes(problems: &[EnvironmentProblem]) -> Vec<&'static str> {
    problems
        .iter()
        .map(|problem| problem.code.as_str())
        .collect()
}

/// Subjects of the problems a validation found, in report order.
fn problem_subjects(problems: &[EnvironmentProblem]) -> Vec<EnvironmentSubject> {
    problems
        .iter()
        .map(|problem| problem.subject.clone())
        .collect()
}

fn diagnostic_codes(diagnostics: &[Diagnostic]) -> Vec<&str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

/// Position of one counted dimension in [`CountedBudgetDimension::ALL`].
///
/// The match is exhaustive on purpose: a counted dimension added without being added to
/// `ALL` fails to compile here, so no dimension can silently escape the zero-usage
/// assertions. The budget slice (1.3) adds its dimensions through this same anchor.
fn counted_dimension_index(dimension: CountedBudgetDimension) -> usize {
    match dimension {
        CountedBudgetDimension::InputBytes => 0,
        CountedBudgetDimension::ArchiveEntries => 1,
        CountedBudgetDimension::EntryBytes => 2,
        CountedBudgetDimension::ReadBytes => 3,
        CountedBudgetDimension::ClassBytes => 4,
        CountedBudgetDimension::AttributeBytes => 5,
        CountedBudgetDimension::CodeBytes => 6,
        CountedBudgetDimension::ResultItems => 7,
        CountedBudgetDimension::OutputBytes => 8,
        // The P2 slice, appended after the P0/P1 dimensions so their positions (and every
        // position-based assertion in this file) keep the values P1 established.
        CountedBudgetDimension::ClassHeaders => 9,
        CountedBudgetDimension::MethodBodies => 10,
        CountedBudgetDimension::IrItems => 11,
        CountedBudgetDimension::IrEdges => 12,
        CountedBudgetDimension::AnalysisSteps => 13,
        CountedBudgetDimension::NormalizationClones => 14,
    }
}

/// Position of one problem code in [`EnvironmentProblemCode::ALL`], with the same
/// compile-time anchor: a code that is not added to `ALL` breaks this match.
fn problem_code_index(code: EnvironmentProblemCode) -> usize {
    match code {
        EnvironmentProblemCode::DuplicateLoader => 0,
        EnvironmentProblemCode::CallerDomainMismatch => 1,
        // Inserted after the two domain/caller-domain codes, where the design's schema
        // skeleton lists it: the positions are the declaration order `ALL` has to mirror, and
        // `all_lists_are_complete_and_align_with_the_serde_names` compares them as a sequence.
        EnvironmentProblemCode::CallerLoaderMismatch => 2,
        EnvironmentProblemCode::MissingParent => 3,
        EnvironmentProblemCode::ParentCycle => 4,
        EnvironmentProblemCode::UnsupportedPolicy => 5,
        EnvironmentProblemCode::UnreadableRoot => 6,
        EnvironmentProblemCode::ContentNotProvided => 7,
        EnvironmentProblemCode::ProviderRootUnbound => 8,
    }
}

/// Number of phases the analysis pipeline has today.
const ANALYSIS_STAGE_COUNT: usize = 6;

/// Position of one analysis stage in [`AnalysisStage::ALL`].
///
/// Exhaustive `match`, like the two anchors above, because `ALL` is not a convenience list
/// here: the engine normalizes a request by filtering it through `ALL`, so a stage that is
/// added to the enum but left out of `ALL` is *silently dropped* from `requested_stages` and
/// never scheduled — the silent empty result invariant 6 forbids. With this match a new stage
/// does not compile until it is classified, and the assertions below then require `ALL` to
/// carry it at that very position.
fn stage_index(stage: AnalysisStage) -> usize {
    match stage {
        AnalysisStage::RawFacts => 0,
        AnalysisStage::RawCfg => 1,
        AnalysisStage::LegacyNormalization => 2,
        AnalysisStage::CanonicalCfg => 3,
        AnalysisStage::Frame => 4,
        AnalysisStage::Ssa => 5,
    }
}

/// Serde code of one analysis stage; the same exhaustive anchor for the wire name.
fn stage_code(stage: AnalysisStage) -> &'static str {
    match stage {
        AnalysisStage::RawFacts => "raw_facts",
        AnalysisStage::RawCfg => "raw_cfg",
        AnalysisStage::LegacyNormalization => "legacy_normalization",
        AnalysisStage::CanonicalCfg => "canonical_cfg",
        AnalysisStage::Frame => "frame",
        AnalysisStage::Ssa => "ssa",
    }
}

/// Reads one repository file below `root`, with a path-labelled failure.
fn read_repository_file(root: &Path, relative: &str) -> String {
    let path = root.join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{relative} is unreadable at {}: {error}", path.display()))
}

/// Variant names of one unit-only enum, in declaration order, read from its source.
///
/// The `ALL` lists are only as complete as the enum they mirror, and a `match` in this file
/// cannot see a variant that the `ALL` list has not been told about: a variant added to the
/// enum *and* classified here would otherwise leave `ALL` short while every assertion still
/// compared `ALL` against itself. Reading the declaration is what closes that loop — the
/// variant names come from the type's own definition, not from a second hand-written list.
fn declared_variants(source: &str, enum_name: &str) -> Vec<String> {
    let heading = format!("pub enum {enum_name} {{");
    let Some(start) = source.find(&heading) else {
        panic!("`{heading}` is not declared in the source under test");
    };
    let mut variants = Vec::new();
    for line in source[start + heading.len()..].lines() {
        let line = line.trim();
        if line.starts_with('}') {
            break;
        }
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }
        let name = line
            .split([',', '(', '{', '='])
            .next()
            .unwrap_or_default()
            .trim();
        assert!(
            !name.is_empty() && name.chars().all(|character| character.is_alphanumeric()),
            "unexpected variant text in `{enum_name}`: {line:?}"
        );
        variants.push(name.to_string());
    }
    assert!(
        !variants.is_empty(),
        "`{enum_name}` has no variants to compare against"
    );
    variants
}

/// `snake_case` code serde derives for one variant name.
fn serde_code(variant: &str) -> String {
    let mut code = String::with_capacity(variant.len() + 4);
    for character in variant.chars() {
        if character.is_uppercase() && !code.is_empty() {
            code.push('_');
        }
        code.extend(character.to_lowercase());
    }
    code
}

/// Asserts that an `ALL` list is exactly the declared variant set, in declaration order.
///
/// Both directions matter: a variant missing from `ALL` (a silently dropped stage) and a
/// variant listed in an order the enum does not declare (a phase order that disagrees with
/// the type) are both failures.
fn assert_all_matches_declaration(label: &str, declared: &[String], all_codes: &[String]) {
    let declared_codes = declared
        .iter()
        .map(|name| serde_code(name))
        .collect::<Vec<_>>();
    assert_eq!(
        all_codes, declared_codes,
        "{label} must list every declared variant once, in declaration order"
    );
}

fn counted_usage_is_zero(usage: &UsageSnapshot) -> bool {
    CountedBudgetDimension::ALL
        .iter()
        .all(|dimension| usage.counted_usage(*dimension) == 0)
}

/// One usage snapshot with the wall clock removed: the comparison form of two reads of one budget.
///
/// `elapsed_millis` is a measurement, not a charge: [`Budget::usage`] takes it again on every
/// read, so the snapshot a report published and a later read of the same budget may legitimately
/// differ by a millisecond while every counted dimension is identical. P1 normalizes the same one
/// field the same way, and nothing else is dropped here.
fn counted_usage(usage: &UsageSnapshot) -> UsageSnapshot {
    UsageSnapshot {
        elapsed_millis: 0,
        ..usage.clone()
    }
}

/// One execution report compared with that one measurement removed from its usage.
fn without_wall_clock(execution: &ExecutionReport) -> ExecutionReport {
    match execution {
        ExecutionReport::Complete { usage } => ExecutionReport::Complete {
            usage: counted_usage(usage),
        },
        ExecutionReport::Partial { reason, usage } => ExecutionReport::Partial {
            reason: reason.clone(),
            usage: counted_usage(usage),
        },
        ExecutionReport::Cancelled { usage } => ExecutionReport::Cancelled {
            usage: counted_usage(usage),
        },
        ExecutionReport::Failed { reason, usage } => ExecutionReport::Failed {
            reason: reason.clone(),
            usage: counted_usage(usage),
        },
    }
}

fn usage_of(execution: &ExecutionReport) -> &UsageSnapshot {
    match execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage,
    }
}

/// Code of an `Unsupported` termination, when the execution ended that way.
fn unsupported_code(execution: &ExecutionReport) -> Option<&str> {
    match execution {
        ExecutionReport::Failed {
            reason: TerminationReason::Unsupported { code },
            ..
        } => Some(code.as_str()),
        _ => None,
    }
}

/// The structured code of a failed run (a damaged input rather than an unsupported one).
fn failure_code(execution: &ExecutionReport) -> Option<&str> {
    match execution {
        ExecutionReport::Failed {
            reason: TerminationReason::Error { code },
            ..
        } => Some(code.as_str()),
        _ => None,
    }
}

fn invalid_input_code(error: &Error) -> Option<&str> {
    match error {
        Error::InvalidInput { code, .. } => Some(code.as_str()),
        _ => None,
    }
}

/// A report that performed nothing may not carry a unique resolution, a partial range or a
/// charged counted dimension.
///
/// This is the state of a request whose environment the validator rejected: no capability
/// runs, so nothing can be resolved and nothing can be read.
fn assert_nothing_was_performed(report: &ResolutionReport) {
    assert_eq!(report.analysis, ResolutionAnalysis::NotPerformed);
    assert!(
        report.state.is_none(),
        "state must be None: {:?}",
        report.state
    );
    assert!(report.resolved.is_none());
    assert!(report.candidates.is_empty());
    assert!(
        report.reads.is_empty(),
        "a capability that never ran cannot have read a class header"
    );
    assert_eq!(report.coverage, Coverage::not_requested());
    assert!(counted_usage_is_zero(usage_of(&report.execution)));
}

/// Capability codes of the honest unavailable state, one per P2 entry point.
const RESOLUTION_NOT_IMPLEMENTED: &str = "resolution_not_implemented";
const METHOD_ANALYSIS_NOT_IMPLEMENTED: &str = "method_analysis_not_implemented";

/// Checks the environment problems of one report *and* the diagnostics that must mirror
/// them.
///
/// The diagnostic list is asserted as an exact sequence — the expected problem codes in
/// declaration order, followed by the capability code of that entry — so a dropped,
/// reordered, prefixed or renamed diagnostic fails here. Each environment diagnostic has to
/// keep `Error` severity: a warning would let a rejected environment look like a footnote.
fn assert_report_diagnostics(
    problems: &[EnvironmentProblem],
    diagnostics: &[Diagnostic],
    expected: &[&str],
    entry_code: &str,
    entry: &str,
) {
    assert_eq!(
        problem_codes(problems),
        expected,
        "{entry}: environment problems"
    );
    let expected_diagnostics = expected
        .iter()
        .copied()
        .chain([entry_code])
        .collect::<Vec<_>>();
    assert_eq!(
        diagnostic_codes(diagnostics),
        expected_diagnostics,
        "{entry}: diagnostics must name every environment problem once, under the same \
         code, then the capability code"
    );
    for diagnostic in diagnostics.iter().take(expected.len()) {
        assert_eq!(
            diagnostic.severity,
            DiagnosticSeverity::Error,
            "{entry}: an environment problem is an error: {diagnostic:?}"
        );
    }
}

/// Runs one rejected environment through **all three** entry points.
///
/// Each entry has to answer with the same environment problem codes and the same codes as
/// diagnostics, while keeping its own honest unavailable state: a dropped
/// `environment_problems` list, an emptied diagnostic list or an optimistic
/// "empty but complete" result in either of the three reports fails here. Counted usage
/// stays zero, so no entry may have read a byte to produce these answers.
fn assert_problem_report(fixture: &Fixture, environment: ResolutionEnvironment, expected: &[&str]) {
    let content = std::slice::from_ref(&fixture.snapshot);

    // 1. Symbol resolution.
    let mut budget = Budget::new(zero_limits());
    let report = Engine::new()
        .resolve_symbol(content, &request(environment.clone()), &mut budget)
        .expect("environment problems are reported, not raised");
    assert_nothing_was_performed(&report);
    assert_report_diagnostics(
        &report.environment_problems,
        &report.diagnostics,
        expected,
        RESOLUTION_NOT_IMPLEMENTED,
        "resolve_symbol",
    );
    assert_eq!(
        unsupported_code(&report.execution),
        Some(RESOLUTION_NOT_IMPLEMENTED)
    );
    assert_eq!(report.environment_identity.runtime, environment.runtime);
    assert!(counted_usage_is_zero(&budget.usage()));

    // 2. Declaration references: a rejected environment must not produce a scan that looks
    //    scanned-but-empty, and it must not claim excluded candidates either.
    let mut budget = Budget::new(zero_limits());
    let declaration = Engine::new()
        .declaration_references(
            content,
            &declaration_query(fixture, environment.clone()),
            &mut budget,
        )
        .expect("environment problems are reported, not raised");
    assert_report_diagnostics(
        &declaration.environment_problems,
        &declaration.diagnostics,
        expected,
        RESOLUTION_NOT_IMPLEMENTED,
        "declaration_references",
    );
    assert_eq!(declaration.analysis, ResolutionAnalysis::NotPerformed);
    assert!(declaration.items.is_empty());
    assert!(
        declaration.unsupported_categories.is_empty(),
        "nothing was scanned, so no category can be called unsupported"
    );
    assert_eq!(declaration.unresolved_candidates, 0);
    assert!(!declaration.has_more);
    assert_eq!(declaration.returned_items, 0);
    assert_eq!(declaration.coverage, Coverage::not_requested());
    assert_eq!(
        unsupported_code(&declaration.execution),
        Some(RESOLUTION_NOT_IMPLEMENTED)
    );
    assert_eq!(
        declaration.environment_identity.runtime,
        environment.runtime
    );
    assert!(counted_usage_is_zero(&budget.usage()));

    // 3. Method analysis: the scheduled phases are still listed, still `NotPerformed`, and
    //    the body was still never located.
    let mut budget = Budget::new(zero_limits());
    let analysis = Engine::new()
        .analyze_method(
            content,
            &analysis_request(fixture, environment.clone(), vec![AnalysisStage::Ssa]),
            &mut budget,
        )
        .expect("environment problems are reported, not raised");
    assert_report_diagnostics(
        &analysis.environment_problems,
        &analysis.diagnostics,
        expected,
        METHOD_ANALYSIS_NOT_IMPLEMENTED,
        "analyze_method",
    );
    assert_eq!(analysis.requested_stages, vec![AnalysisStage::Ssa]);
    assert_eq!(
        analysis
            .stages
            .iter()
            .map(|stage| stage.stage)
            .collect::<Vec<_>>(),
        AnalysisStage::ALL.to_vec()
    );
    assert!(
        analysis
            .stages
            .iter()
            .all(|stage| stage.state == StageState::NotPerformed),
        "no phase ran, whatever the environment said"
    );
    assert_eq!(analysis.body, MethodBodyState::NotInspected);
    assert_eq!(analysis.loader, environment.runtime.load_domain.loader);
    assert!(analysis.origin.is_empty());
    assert_eq!(analysis.coverage, Coverage::not_requested());
    assert_eq!(
        unsupported_code(&analysis.execution),
        Some(METHOD_ANALYSIS_NOT_IMPLEMENTED)
    );
    assert_eq!(analysis.environment_identity.runtime, environment.runtime);
    assert!(counted_usage_is_zero(&budget.usage()));
}

// ---------------------------------------------------------------------------
// Environment validation
// ---------------------------------------------------------------------------

#[test]
fn valid_environment_reports_no_problem_and_keeps_the_declaration_order() {
    let fixture = fixture();
    let environment = healthy_environment(&fixture);
    let (problems, identity) =
        validate_environment(std::slice::from_ref(&fixture.snapshot), &environment);

    assert_eq!(problems, Vec::new());
    assert_eq!(identity.runtime, environment.runtime);
    assert_eq!(
        identity.domain_loaders,
        vec![loader("app"), loader("platform")]
    );
    assert_eq!(
        identity.providers,
        vec![ProviderId("app-headers".to_string())]
    );
    assert_eq!(identity.content, vec![fixture.snapshot.id().clone()]);
}

#[test]
fn missing_parent_domain_is_reported_and_resolves_nothing() {
    let fixture = fixture();
    let app = loader("app");
    let missing = loader("missing-platform");
    let roots = vec![LoadRoot::Snapshot {
        snapshot: fixture.snapshot.id().clone(),
    }];
    let environment = environment(
        &fixture,
        domain(&app, Some(missing.clone()), roots.clone()),
        vec![domain(&app, Some(missing.clone()), roots)],
        Vec::new(),
    );

    let (problems, identity) =
        validate_environment(std::slice::from_ref(&fixture.snapshot), &environment);
    assert_eq!(problem_codes(&problems), vec!["missing_parent"]);
    assert_eq!(
        problem_subjects(&problems),
        vec![EnvironmentSubject::Loader(app.clone())]
    );
    assert!(problems[0].message.contains("missing-platform"));
    assert_eq!(identity.domain_loaders, vec![app]);

    // All three entries report the problem under the same code, with one `missing_parent`
    // diagnostic followed by the capability code of the entry.
    assert_problem_report(&fixture, environment, &["missing_parent"]);
}

#[test]
fn parent_cycle_is_reported_for_every_domain_in_the_cycle() {
    let fixture = fixture();
    let app = loader("app");
    let platform = loader("platform");
    let roots = vec![LoadRoot::Snapshot {
        snapshot: fixture.snapshot.id().clone(),
    }];
    let environment = environment(
        &fixture,
        domain(&app, Some(platform.clone()), roots.clone()),
        vec![
            domain(&app, Some(platform.clone()), roots),
            domain(&platform, Some(app.clone()), Vec::new()),
        ],
        Vec::new(),
    );

    let (problems, _) = validate_environment(std::slice::from_ref(&fixture.snapshot), &environment);
    assert_eq!(
        problem_codes(&problems),
        vec!["parent_cycle", "parent_cycle"],
        "one problem per domain whose parent chain re-enters itself"
    );
    assert_eq!(
        problem_subjects(&problems),
        vec![
            EnvironmentSubject::Loader(app.clone()),
            EnvironmentSubject::Loader(platform.clone()),
        ]
    );
    assert!(
        !problems
            .iter()
            .any(|problem| problem.code == EnvironmentProblemCode::MissingParent),
        "a declared parent is not a missing parent, even inside a cycle"
    );
    assert!(problems[0].message.contains("app -> platform -> app"));
    assert_problem_report(&fixture, environment, &["parent_cycle", "parent_cycle"]);
}

#[test]
fn caller_domain_must_be_equal_to_the_domain_declared_under_the_same_loader() {
    let fixture = fixture();
    let app = loader("app");
    let roots = vec![LoadRoot::Snapshot {
        snapshot: fixture.snapshot.id().clone(),
    }];
    // The caller declares the fixture root, the domain under the same loader does not.
    let environment = environment(
        &fixture,
        domain(&app, None, roots),
        vec![domain(&app, None, Vec::new())],
        Vec::new(),
    );

    let (problems, _) = validate_environment(std::slice::from_ref(&fixture.snapshot), &environment);
    assert_eq!(problem_codes(&problems), vec!["caller_domain_mismatch"]);
    assert_eq!(
        problem_subjects(&problems),
        vec![EnvironmentSubject::Loader(app)]
    );
    assert_problem_report(&fixture, environment, &["caller_domain_mismatch"]);
}

#[test]
fn caller_context_must_name_the_loader_of_its_own_domain() {
    let fixture = fixture();
    let app = loader("app");
    let platform = loader("platform");
    let environment = healthy_environment(&fixture);
    let content = std::slice::from_ref(&fixture.snapshot);

    // One environment, one class symbol, two caller identities. The caller that names the
    // loader of `runtime.load_domain` is not a problem and resolves through the fixture's
    // standalone CLASS root. The P0/P1 `limits()` helper leaves every P2 dimension at its
    // fail-closed zero, so a real lookup raises its own header-attempt allowance explicitly.
    let lookup_limits = Limits {
        class_headers: 8,
        ..limits()
    };
    let mut budget = Budget::new(lookup_limits.clone());
    let agreeing = Engine::new()
        .resolve_symbol(
            content,
            &class_request(environment.clone(), &app, b"HistoricalControlFlow"),
            &mut budget,
        )
        .expect("a legal request is answered, not raised");
    assert_eq!(
        agreeing.environment_problems,
        Vec::new(),
        "a caller that names its own domain's loader is not a problem"
    );
    assert_eq!(agreeing.analysis, ResolutionAnalysis::Performed);
    assert_eq!(agreeing.state, Some(ResolutionState::Resolved));
    assert_eq!(
        agreeing
            .resolved
            .as_ref()
            .map(|resolved| resolved.loader.clone()),
        Some(app.clone())
    );
    assert!(
        agreeing.diagnostics.is_empty(),
        "{:?}",
        agreeing.diagnostics
    );
    assert_eq!(budget.usage().class_headers, 1);

    // The disagreeing caller is a problem, and the environment counts as rejected: the same
    // symbol that just resolved resolves nothing here, under the 1.1 honest state with the new
    // problem in front of the capability code.
    let mut budget = Budget::new(lookup_limits);
    let disagreeing = Engine::new()
        .resolve_symbol(
            content,
            &class_request(environment.clone(), &platform, b"HistoricalControlFlow"),
            &mut budget,
        )
        .expect("environment problems are reported, not raised");
    assert_eq!(
        problem_codes(&disagreeing.environment_problems),
        vec!["caller_loader_mismatch"]
    );
    assert_eq!(
        problem_subjects(&disagreeing.environment_problems),
        vec![EnvironmentSubject::Loader(platform.clone())],
        "the subject is the caller identity that has to change"
    );
    let message = &disagreeing.environment_problems[0].message;
    assert!(
        message.contains("platform") && message.contains("app"),
        "the message names both declarations: {message}"
    );
    assert_nothing_was_performed(&disagreeing);
    assert_report_diagnostics(
        &disagreeing.environment_problems,
        &disagreeing.diagnostics,
        &["caller_loader_mismatch"],
        RESOLUTION_NOT_IMPLEMENTED,
        "resolve_symbol",
    );
    assert_eq!(
        unsupported_code(&disagreeing.execution),
        Some(RESOLUTION_NOT_IMPLEMENTED)
    );
    assert!(
        counted_usage_is_zero(&budget.usage()),
        "a rejected environment reads no byte"
    );

    // The check belongs to the entry that carries a caller: a declaration query has no
    // `CallerContext`, so it neither reports the problem nor pretends to have checked it.
    let mut budget = Budget::new(limits());
    let declarations = Engine::new()
        .declaration_references(
            content,
            &declaration_query(&fixture, environment),
            &mut budget,
        )
        .expect("a legal query is answered, not raised");
    assert_eq!(declarations.environment_problems, Vec::new());
    assert_eq!(
        declarations.analysis,
        ResolutionAnalysis::Performed,
        "the query has no caller identity to mismatch, so the environment it was given is the \
         one it scans under"
    );
}

#[test]
fn loader_uniqueness_covers_repeated_and_undeclared_caller_domains() {
    let fixture = fixture();
    let app = loader("app");
    let platform = loader("platform");
    let roots = vec![LoadRoot::Snapshot {
        snapshot: fixture.snapshot.id().clone(),
    }];

    // Two domains under one loader: both the declaration and the caller identity are
    // ambiguous, so the report names the uniqueness violation and never a mismatch.
    let duplicated = environment(
        &fixture,
        domain(&app, None, roots.clone()),
        vec![
            domain(&app, None, roots.clone()),
            domain(&app, None, Vec::new()),
        ],
        Vec::new(),
    );
    let (problems, _) = validate_environment(std::slice::from_ref(&fixture.snapshot), &duplicated);
    assert_eq!(problem_codes(&problems), vec!["duplicate_loader"; 2]);
    assert!(problems.iter().all(|problem| {
        problem.code == EnvironmentProblemCode::DuplicateLoader
            && problem.subject == EnvironmentSubject::Loader(app.clone())
    }));
    assert_problem_report(&fixture, duplicated, &["duplicate_loader"; 2]);

    // The caller loader is not declared at all: it is still a uniqueness violation of the
    // caller identity, and the message says which of the two shapes happened.
    let undeclared = environment(
        &fixture,
        domain(&app, None, roots),
        vec![domain(&platform, None, Vec::new())],
        Vec::new(),
    );
    let (problems, _) = validate_environment(std::slice::from_ref(&fixture.snapshot), &undeclared);
    assert_eq!(problem_codes(&problems), vec!["duplicate_loader"]);
    assert!(problems[0].message.contains("has no domain"));
    assert_problem_report(&fixture, undeclared, &["duplicate_loader"]);
}

#[test]
fn roots_without_provided_content_are_reported_by_index() {
    let fixture = fixture();
    let app = loader("app");
    let absent = SnapshotId("not-provided".to_string());
    let tree = ContainerOrigin {
        snapshot: SnapshotId("not-provided-tree".to_string()),
        root_container: ContainerId("root.zip".to_string()),
        steps: Vec::new(),
    };
    let roots = vec![
        LoadRoot::Snapshot {
            snapshot: fixture.snapshot.id().clone(),
        },
        LoadRoot::Snapshot {
            snapshot: absent.clone(),
        },
        LoadRoot::ArtifactTree { root: tree.clone() },
    ];
    let environment = environment(
        &fixture,
        domain(&app, None, roots.clone()),
        vec![domain(&app, None, roots)],
        Vec::new(),
    );

    let (problems, identity) =
        validate_environment(std::slice::from_ref(&fixture.snapshot), &environment);
    assert_eq!(
        problem_codes(&problems),
        vec!["content_not_provided", "content_not_provided"],
        "the provided root at index 0 is not reported"
    );
    assert_eq!(
        problem_subjects(&problems),
        vec![
            EnvironmentSubject::Root {
                loader: app.clone(),
                index: 1
            },
            EnvironmentSubject::Root {
                loader: app,
                index: 2
            },
        ]
    );
    assert!(problems[0].message.contains("not-provided"));
    assert!(problems[1].message.contains("not-provided-tree"));
    assert_eq!(identity.content, vec![fixture.snapshot.id().clone()]);
    assert_problem_report(
        &fixture,
        environment,
        &["content_not_provided", "content_not_provided"],
    );
}

#[test]
fn environment_problems_of_different_codes_keep_their_declaration_order() {
    let fixture = fixture();
    let app = loader("app");
    let missing = loader("missing-platform");
    let provided = LoadRoot::Snapshot {
        snapshot: fixture.snapshot.id().clone(),
    };

    // Three *different* codes in one environment: the domain declares an unsupported policy,
    // a parent that no domain binds, and a root whose content was not provided. The order the
    // validator reports them in is part of the contract (invariant 2), and a report that
    // reorders or drops one diagnostic is indistinguishable from a correct one as long as
    // every negative case carries a single code.
    let unsupported = domain_with(
        &app,
        Some(missing.clone()),
        ModuleMode::Hybrid,
        DelegationPolicy::ParentFirst,
        vec![
            provided.clone(),
            LoadRoot::Snapshot {
                snapshot: SnapshotId("never-provided".to_string()),
            },
        ],
    );
    let three_codes = environment(&fixture, unsupported.clone(), vec![unsupported], Vec::new());

    let (problems, identity) =
        validate_environment(std::slice::from_ref(&fixture.snapshot), &three_codes);
    assert_eq!(
        problem_codes(&problems),
        vec![
            "unsupported_policy",
            "missing_parent",
            "content_not_provided"
        ],
        "policy, parent and root problems are reported in the validator's fixed order"
    );
    assert_eq!(
        problem_subjects(&problems),
        vec![
            EnvironmentSubject::Loader(app.clone()),
            EnvironmentSubject::Loader(app.clone()),
            EnvironmentSubject::Root {
                loader: app.clone(),
                index: 1
            },
        ],
        "each problem points at the declaration it belongs to"
    );
    assert_eq!(identity.domain_loaders, vec![app.clone()]);
    assert_problem_report(
        &fixture,
        three_codes,
        &[
            "unsupported_policy",
            "missing_parent",
            "content_not_provided",
        ],
    );

    // A second combination with a repeated code in front of a different one: the leading pair
    // is not interchangeable with the trailing problem, and the subjects follow the same
    // order.
    let duplicated = environment(
        &fixture,
        domain(&app, None, vec![provided.clone()]),
        vec![
            domain(&app, None, vec![provided.clone()]),
            domain(&app, None, Vec::new()),
        ],
        vec![HeaderProvider {
            id: ProviderId("stray-headers".to_string()),
            roots: vec![LoadRoot::External {
                id: "host-jdk".to_string(),
            }],
        }],
    );
    let (problems, _) = validate_environment(std::slice::from_ref(&fixture.snapshot), &duplicated);
    assert_eq!(
        problem_codes(&problems),
        vec![
            "duplicate_loader",
            "duplicate_loader",
            "provider_root_unbound"
        ],
        "a repeated loader is reported before the provider that names a stray root"
    );
    assert_problem_report(
        &fixture,
        duplicated,
        &[
            "duplicate_loader",
            "duplicate_loader",
            "provider_root_unbound",
        ],
    );
}

#[test]
fn external_roots_are_unreadable_declarations_not_missing_content() {
    let fixture = fixture();
    let app = loader("app");
    let environment = environment(
        &fixture,
        domain(
            &app,
            None,
            vec![LoadRoot::External {
                id: "host-jdk".to_string(),
            }],
        ),
        vec![domain(
            &app,
            None,
            vec![LoadRoot::External {
                id: "host-jdk".to_string(),
            }],
        )],
        Vec::new(),
    );

    let (problems, _) = validate_environment(std::slice::from_ref(&fixture.snapshot), &environment);
    assert_eq!(problem_codes(&problems), vec!["unreadable_root"]);
    assert_eq!(
        problem_subjects(&problems),
        vec![EnvironmentSubject::Root {
            loader: app,
            index: 0
        }]
    );
    assert!(problems[0].message.contains("host-jdk"));
    assert!(
        !problems
            .iter()
            .any(|problem| problem.code == EnvironmentProblemCode::ContentNotProvided),
        "an external root was declared, it was not left unprovided"
    );
    assert_problem_report(&fixture, environment, &["unreadable_root"]);
}

#[test]
fn provider_roots_must_be_listed_by_a_participating_domain() {
    let fixture = fixture();
    let app = loader("app");
    let roots = vec![LoadRoot::Snapshot {
        snapshot: fixture.snapshot.id().clone(),
    }];
    let environment = environment(
        &fixture,
        domain(&app, None, roots.clone()),
        vec![domain(&app, None, roots.clone())],
        vec![
            HeaderProvider {
                id: ProviderId("app-headers".to_string()),
                roots: roots.clone(),
            },
            HeaderProvider {
                id: ProviderId("foreign-headers".to_string()),
                roots: vec![
                    LoadRoot::Snapshot {
                        snapshot: SnapshotId("foreign".to_string()),
                    },
                    LoadRoot::External {
                        id: "host-jdk".to_string(),
                    },
                ],
            },
        ],
    );

    let (problems, identity) =
        validate_environment(std::slice::from_ref(&fixture.snapshot), &environment);
    assert_eq!(problem_codes(&problems), vec!["provider_root_unbound"]);
    assert_eq!(
        problem_subjects(&problems),
        vec![EnvironmentSubject::Provider(ProviderId(
            "foreign-headers".to_string()
        ))]
    );
    assert!(problems[0].message.contains("[0, 1]"));
    assert_eq!(
        identity.providers,
        vec![
            ProviderId("app-headers".to_string()),
            ProviderId("foreign-headers".to_string()),
        ]
    );
    assert!(
        !problems
            .iter()
            .any(|problem| problem.code == EnvironmentProblemCode::ContentNotProvided),
        "provider roots are bound against domain declarations, not against content"
    );
    assert_problem_report(&fixture, environment, &["provider_root_unbound"]);
}

#[test]
fn unsupported_module_modes_and_delegations_are_policy_problems() {
    let fixture = fixture();
    let app = loader("app");
    let roots = vec![LoadRoot::Snapshot {
        snapshot: fixture.snapshot.id().clone(),
    }];

    let unsupported = [
        (ModuleMode::ModulePath, DelegationPolicy::ParentFirst),
        (ModuleMode::Hybrid, DelegationPolicy::ParentFirst),
        (
            ModuleMode::Custom {
                id: "jigsaw".to_string(),
            },
            DelegationPolicy::ParentFirst,
        ),
        (ModuleMode::Unknown, DelegationPolicy::ParentFirst),
        (
            ModuleMode::ClassPath,
            DelegationPolicy::Custom {
                id: "osgi".to_string(),
            },
        ),
        (ModuleMode::ClassPath, DelegationPolicy::Unknown),
    ];
    for (module_mode, delegation) in unsupported {
        let declared = domain_with(
            &app,
            None,
            module_mode.clone(),
            delegation.clone(),
            roots.clone(),
        );
        let environment = environment(&fixture, declared.clone(), vec![declared], Vec::new());
        let (problems, _) =
            validate_environment(std::slice::from_ref(&fixture.snapshot), &environment);
        assert_eq!(
            problem_codes(&problems),
            vec!["unsupported_policy"],
            "module mode {module_mode:?} with delegation {delegation:?}"
        );
        assert_eq!(
            problem_subjects(&problems),
            vec![EnvironmentSubject::Loader(app.clone())]
        );
        assert_problem_report(&fixture, environment, &["unsupported_policy"]);
    }

    // The two supported policies of this slice stay silent, and a valid environment starts the
    // member resolution (2.3) instead of answering with the unavailable state: the zero budget
    // stops the search at its first charged step. The clock is funded so the refusal is that
    // charge and not the wall clock the all-zero fixture would hit first.
    for delegation in [DelegationPolicy::ParentFirst, DelegationPolicy::ChildFirst] {
        let declared = domain_with(
            &app,
            None,
            ModuleMode::ClassPath,
            delegation.clone(),
            roots.clone(),
        );
        let environment = environment(&fixture, declared.clone(), vec![declared], Vec::new());
        let (problems, _) =
            validate_environment(std::slice::from_ref(&fixture.snapshot), &environment);
        assert_eq!(problems, Vec::new(), "delegation {delegation:?}");

        let mut budget = Budget::new(Limits {
            elapsed_millis: u64::MAX,
            ..zero_limits()
        });
        let report = Engine::new()
            .resolve_symbol(
                std::slice::from_ref(&fixture.snapshot),
                &request(environment),
                &mut budget,
            )
            .expect("a legal request is answered, not raised");
        assert_eq!(report.environment_problems, Vec::new());
        assert_eq!(
            report.analysis,
            ResolutionAnalysis::Performed,
            "a valid environment no longer answers with the unavailable state"
        );
        assert_eq!(report.state, Some(ResolutionState::BudgetExceeded));
        assert_eq!(
            diagnostic_codes(&report.diagnostics),
            vec!["budget_exceeded_analysis_steps"],
            "the member search's first charged step is one analysis step"
        );
        assert!(counted_usage_is_zero(&budget.usage()));
    }
}

// ---------------------------------------------------------------------------
// Request-level mismatches
// ---------------------------------------------------------------------------

#[test]
fn request_snapshot_must_be_one_of_the_provided_snapshots() {
    let fixture = fixture();
    let mut environment = healthy_environment(&fixture);
    environment.runtime.physical.snapshot = SnapshotId("absent".to_string());
    let content = std::slice::from_ref(&fixture.snapshot);

    let error = Engine::new()
        .resolve_symbol(
            content,
            &request(environment.clone()),
            &mut Budget::new(limits()),
        )
        .expect_err("an unprovided snapshot is a request mismatch");
    assert_eq!(
        invalid_input_code(&error),
        Some("resolution_snapshot_mismatch")
    );

    let error = Engine::new()
        .declaration_references(
            content,
            &declaration_query(&fixture, environment.clone()),
            &mut Budget::new(limits()),
        )
        .expect_err("the declaration query checks the same binding");
    assert_eq!(
        invalid_input_code(&error),
        Some("resolution_snapshot_mismatch")
    );

    let error = Engine::new()
        .analyze_method(
            content,
            &analysis_request(&fixture, environment, vec![AnalysisStage::RawFacts]),
            &mut Budget::new(limits()),
        )
        .expect_err("the analysis entry checks the same binding");
    assert_eq!(
        invalid_input_code(&error),
        Some("resolution_snapshot_mismatch")
    );
}

#[test]
fn reference_use_must_describe_the_same_member_kind_as_the_target() {
    let class = SymbolRef::Class {
        owner: bytes(b"p/Target"),
    };
    let field = SymbolRef::Field {
        owner: bytes(b"p/Target"),
        name: bytes(b"value"),
        descriptor: bytes(b"I"),
    };
    let method = constructor();

    // The pure predicate the entry point uses.
    assert!(ReferenceUse::ClassReference.matches_symbol(&class));
    assert!(!ReferenceUse::InvokeVirtual.matches_symbol(&class));
    for use_kind in [ReferenceUse::FieldRead, ReferenceUse::FieldWrite] {
        assert!(use_kind.matches_symbol(&field));
        assert!(!use_kind.matches_symbol(&method));
    }
    for use_kind in [
        ReferenceUse::InvokeStatic,
        ReferenceUse::InvokeSpecial,
        ReferenceUse::InvokeVirtual,
        ReferenceUse::InvokeInterface,
        ReferenceUse::InvokeDynamic,
    ] {
        assert!(use_kind.matches_symbol(&method));
        assert!(!use_kind.matches_symbol(&field));
    }

    let fixture = fixture();
    let environment = healthy_environment(&fixture);
    let content = std::slice::from_ref(&fixture.snapshot);

    // A class reference that names a method is refused before anything is read.
    let mismatched = ResolutionRequest {
        target: SymbolRef::Method {
            owner: bytes(b"HistoricalControlFlow"),
            name: bytes(b"finallyPath"),
            descriptor: bytes(b"(I)I"),
        },
        use_kind: ReferenceUse::ClassReference,
        ..request(environment.clone())
    };
    let error = Engine::new()
        .resolve_symbol(content, &mismatched, &mut Budget::new(zero_limits()))
        .expect_err("a class reference cannot name a method");
    assert_eq!(
        invalid_input_code(&error),
        Some("resolution_target_use_mismatch")
    );

    // The same request with a matching kind is legal, so it is not refused and the resolution
    // really starts: this environment provides no `java/lang/Object`, and the zero budget stops
    // the search at its first step instead of answering as if nothing had been requested.
    let report = Engine::new()
        .resolve_symbol(
            content,
            &request(environment),
            &mut Budget::new(zero_limits()),
        )
        .expect("a matching target and use kind is a legal request");
    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert_eq!(report.state, Some(ResolutionState::BudgetExceeded));
    assert!(report.resolved.is_none());
}

#[test]
fn analysis_request_must_name_at_least_one_stage() {
    let fixture = fixture();
    let environment = healthy_environment(&fixture);
    let content = std::slice::from_ref(&fixture.snapshot);

    let error = Engine::new()
        .analyze_method(
            content,
            &analysis_request(&fixture, environment.clone(), Vec::new()),
            &mut Budget::new(limits()),
        )
        .expect_err("an empty stage set is a request mismatch");
    assert_eq!(invalid_input_code(&error), Some("analysis_no_stages"));

    let report = Engine::new()
        .analyze_method(
            content,
            &analysis_request(&fixture, environment, vec![AnalysisStage::RawFacts]),
            &mut Budget::new(zero_limits()),
        )
        .expect("one requested stage is a legal request");
    assert!(!report.requested_stages.is_empty());
}

// ---------------------------------------------------------------------------
// Honest unavailable state of the three entry points
// ---------------------------------------------------------------------------

#[test]
fn resolution_stops_a_member_request_at_its_first_budgeted_step() {
    // 1.1 pinned the honest unavailable state of a legal member request while no member slice
    // existed. 2.3 performs that resolution, so the same zero budget now reaches the member
    // rules and stops them at their first step: the stop *is* the semantic decision
    // (`BudgetExceeded`), the execution carries the same stop, and the request still reads no
    // byte and records no read. 2.5 enumerates the requested dispatch range only over a
    // declaration that resolved, so the stop also leaves `dispatch` absent — named by
    // `resolution_dispatch_no_declaration` after the stop that produced it — instead of
    // publishing an empty candidate list.
    let fixture = fixture();
    let environment = healthy_environment(&fixture);
    let mut request = request(environment.clone());
    request.dispatch = Some(DispatchScope {
        scope: PhysicalScope::SnapshotAll,
        consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
    });

    // The clock is funded, so the stop names the work the search asked for instead of the
    // all-zero wall clock.
    let mut budget = Budget::new(Limits {
        elapsed_millis: u64::MAX,
        ..zero_limits()
    });
    let report = Engine::new()
        .resolve_symbol(
            std::slice::from_ref(&fixture.snapshot),
            &request,
            &mut budget,
        )
        .expect("a legal request is answered, not raised");

    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert_eq!(report.state, Some(ResolutionState::BudgetExceeded));
    assert!(report.resolved.is_none());
    assert!(report.candidates.is_empty());
    assert!(
        report.dispatch.is_none(),
        "a range without a resolved declaration must not look like an empty one"
    );
    assert!(
        report.reads.is_empty(),
        "the refused step happens before any header is read"
    );
    assert_eq!(report.target, constructor());
    assert_eq!(report.use_kind, ReferenceUse::InvokeSpecial);
    assert_eq!(report.caller.loader, loader("app"));
    assert_eq!(
        report.environment_identity,
        EnvironmentIdentity {
            runtime: environment.runtime.clone(),
            domain_loaders: vec![loader("app"), loader("platform")],
            providers: vec![ProviderId("app-headers".to_string())],
            content: vec![fixture.snapshot.id().clone()],
        }
    );
    assert_eq!(report.environment_problems, Vec::new());
    assert_eq!(
        diagnostic_codes(&report.diagnostics),
        vec![
            "budget_exceeded_analysis_steps",
            "resolution_dispatch_no_declaration"
        ],
        "the stop that ended the resolution is published first, then the range that could not \
         be enumerated over it"
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps
            },
            ..
        }
    ));
    assert!(counted_usage_is_zero(&budget.usage()));
}

#[test]
fn declaration_reference_query_reports_its_stop_instead_of_an_empty_answer() {
    // The declaration query is performed by this engine: it scans the explicit scope with the
    // structure consumers, so a request whose counted budget cannot fund that scan reports the
    // stop it really made — never an empty-but-complete answer, and never a claim that a
    // candidate was excluded. Every counted dimension is zero and only the clock is funded, so
    // the stop names the work charge that was refused.
    let fixture = fixture();
    let environment = healthy_environment(&fixture);
    let query = declaration_query(&fixture, environment.clone());

    let mut budget = Budget::new(Limits {
        elapsed_millis: u64::MAX,
        ..zero_limits()
    });
    let report = Engine::new()
        .declaration_references(std::slice::from_ref(&fixture.snapshot), &query, &mut budget)
        .expect("a legal query is answered, not raised");

    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert!(report.items.is_empty());
    assert_eq!(report.declaration, query.declaration);
    assert_eq!(report.scope, PhysicalScope::SnapshotAll);
    assert_eq!(report.consumers, query.consumers);
    assert!(
        report.unsupported_categories.is_empty(),
        "the consumer schema names no category this engine cannot scan"
    );
    assert_eq!(report.unresolved_candidates, 0);
    assert!(
        report.has_more,
        "the scan stopped before the end of its range, so the empty list is not the answer"
    );
    assert_eq!(report.returned_items, 0);
    assert!(
        report.reads.is_empty(),
        "no charge was accepted, so no class header was read"
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial,
        "the scan did not reach the range it was asked to cover"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial,
        "a scan that stopped may hold candidates no resolution has seen yet"
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ReadBytes
            },
            ..
        }
    ));
    assert_eq!(
        diagnostic_codes(&report.diagnostics),
        vec!["budget_exceeded_read_bytes"]
    );
    assert_eq!(report.environment_identity.runtime, environment.runtime);
    assert!(counted_usage_is_zero(&budget.usage()));
}

#[test]
fn method_analysis_normalizes_the_request_and_schedules_the_prerequisites() {
    let fixture = fixture();
    let environment = healthy_environment(&fixture);
    let request = analysis_request(
        &fixture,
        environment.clone(),
        vec![AnalysisStage::Ssa, AnalysisStage::Frame, AnalysisStage::Ssa],
    );

    let mut budget = Budget::new(analysis_limits());
    let report = Engine::new()
        .analyze_method(
            std::slice::from_ref(&fixture.snapshot),
            &request,
            &mut budget,
        )
        .expect("a legal request is answered, not raised");

    assert_eq!(
        report.requested_stages,
        vec![AnalysisStage::Frame, AnalysisStage::Ssa],
        "requested stages are deduplicated and put into the fixed phase order"
    );
    assert_eq!(
        report
            .stages
            .iter()
            .map(|stage| stage.stage)
            .collect::<Vec<_>>(),
        AnalysisStage::ALL.to_vec(),
        "a request for `Ssa` schedules every earlier phase"
    );
    // The scheduled phases of this build really run in table order: `raw_facts` reads the
    // driver method's class definition and decodes its body, `raw_cfg` builds the raw graph
    // over those facts, `legacy_normalization` establishes the `jsr`/`ret` call contexts (none
    // for this body: the 52 fixture inlines its `finally`), `canonical_cfg` normalizes the graph
    // under them, `frame` derives the frames of that graph, and the `ssa` names 4.3 derives over
    // exactly those frames complete last. Every phase this build declares is implemented, so the
    // request is answered with the whole pipeline performed.
    assert_eq!(
        report
            .stages
            .iter()
            .map(|stage| stage.state.clone())
            .collect::<Vec<_>>(),
        vec![
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
        ]
    );
    assert_eq!(report.method, fixture.method);
    assert_eq!(report.loader, loader("app"));
    assert!(report.origin.is_empty());
    assert_eq!(
        report.reads,
        vec![HeaderRead {
            loader: loader("app"),
            definition: fixture.definition.clone(),
            reason: ReadReason::DriverMethodBody,
        }],
        "the one header read of a method-analysis request is the driver method's own class"
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema,
        "the whole body was decoded, so the BCI plane is complete"
    );
    assert_eq!(
        unsupported_code(&report.execution),
        None,
        "every phase this build declares is implemented, so nothing is refused as unsupported"
    );
    assert!(
        diagnostic_codes(&report.diagnostics).is_empty(),
        "a completed pipeline reports no diagnostic: {:?}",
        diagnostic_codes(&report.diagnostics)
    );
    assert_eq!(report.body, MethodBodyState::Present);
    assert_eq!(
        report.environment_identity,
        EnvironmentIdentity {
            runtime: environment.runtime.clone(),
            domain_loaders: vec![loader("app"), loader("platform")],
            providers: vec![ProviderId("app-headers".to_string())],
            content: vec![fixture.snapshot.id().clone()],
        }
    );
    // The counted dimensions 3.3 uses are real now: one header read, one body attempt, the IR
    // items and steps of the raw graph, and the def-use edges 4.3 bills over the frames. This
    // fixture's body is straight-line code with a handler no instruction of its protected range
    // can enter, so neither the raw graph nor the canonical graph holds an edge; the edges this
    // request bills are the names' own def-use edges, one per use, and a run that stops before
    // the `ssa` phase bills none of them.
    let mut frames_only = request.clone();
    frames_only.stages = vec![AnalysisStage::Frame];
    let mut frames_budget = Budget::new(analysis_limits());
    Engine::new()
        .analyze_method(
            std::slice::from_ref(&fixture.snapshot),
            &frames_only,
            &mut frames_budget,
        )
        .expect("a legal request is answered, not raised");
    let usage = budget.usage();
    assert_eq!(frames_budget.usage().ir_edges, 0, "no edge before 4.3");
    assert_eq!(usage.class_headers, 1);
    assert_eq!(usage.method_bodies, 1);
    assert!(
        usage.code_bytes > 0,
        "the decoded instructions were charged"
    );
    assert!(usage.ir_items > 0, "the raw graph's items were charged");
    assert!(
        usage.ir_edges > frames_budget.usage().ir_edges,
        "4.3 bills one def-use edge per use: {} vs {}",
        usage.ir_edges,
        frames_budget.usage().ir_edges
    );
    assert!(usage.analysis_steps > 0, "the raw pass ran a worklist");

    // A zero budget stops the first pass at its first charge: the prefix semantics of a budget
    // stop, where nothing was read and no phase behind the stop ran.
    let mut budget = Budget::new(zero_work_limits());
    let report = Engine::new()
        .analyze_method(
            std::slice::from_ref(&fixture.snapshot),
            &request,
            &mut budget,
        )
        .expect("a legal request is answered, not raised");
    assert_eq!(
        report
            .stages
            .iter()
            .map(|stage| stage.state.clone())
            .collect::<Vec<_>>(),
        vec![
            StageState::Partial,
            StageState::NotPerformed,
            StageState::NotPerformed,
            StageState::NotPerformed,
            StageState::NotPerformed,
            StageState::NotPerformed,
        ],
        "the pass that was refused is partial and the phases behind it never ran"
    );
    assert!(report.reads.is_empty(), "a refused charge records no read");
    assert_eq!(report.body, MethodBodyState::NotInspected);
    assert_eq!(report.coverage, Coverage::not_requested());
    assert_eq!(
        without_wall_clock(&report.execution),
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ClassHeaders,
            },
            usage: counted_usage(&budget.usage()),
        }
    );
    assert_eq!(
        diagnostic_codes(&report.diagnostics),
        vec!["budget_exceeded_class_headers"]
    );
    assert!(counted_usage_is_zero(&budget.usage()));
    // The semantic plane of a run that was refused before it read anything: the phase that
    // checks the local invariants is scheduled here, but it never ran, so it proved nothing.
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::Unproven,
        "a refused charge is not semantic evidence"
    );

    // A single requested phase does not schedule the phases after it.
    let report = Engine::new()
        .analyze_method(
            std::slice::from_ref(&fixture.snapshot),
            &analysis_request(&fixture, environment, vec![AnalysisStage::Frame]),
            &mut Budget::new(analysis_limits()),
        )
        .expect("a legal request is answered, not raised");
    assert_eq!(report.requested_stages, vec![AnalysisStage::Frame]);
    assert_eq!(
        report
            .stages
            .iter()
            .map(|stage| stage.stage)
            .collect::<Vec<_>>(),
        vec![
            AnalysisStage::RawFacts,
            AnalysisStage::RawCfg,
            AnalysisStage::LegacyNormalization,
            AnalysisStage::CanonicalCfg,
            AnalysisStage::Frame,
        ]
    );
    // Every scheduled phase completed and the request is still `Unproven`: the phase that checks
    // the local invariants is `ssa`, this request never scheduled it, and what a caller asked for
    // does not decide what the run proved.
    assert_eq!(
        report
            .stages
            .iter()
            .filter(|stage| stage.state == StageState::Completed)
            .count(),
        5,
        "the whole scheduled prefix completed"
    );
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::Unproven,
        "a phase this build never scheduled proves nothing"
    );
}

#[test]
fn an_unread_body_is_not_inspected_never_a_body_fact() {
    let fixture = fixture();
    let environment = healthy_environment(&fixture);

    // A budget that refuses the first charge stops the run before any byte was read, so the
    // report states no body fact at all: `NotInspected` is neither `Present` nor a
    // `DeclaredWithoutBody` claim, and it is also what a stopped read leaves behind.
    let mut budget = Budget::new(zero_work_limits());
    let report = Engine::new()
        .analyze_method(
            std::slice::from_ref(&fixture.snapshot),
            &analysis_request(&fixture, environment.clone(), vec![AnalysisStage::RawFacts]),
            &mut budget,
        )
        .expect("a legal request is answered, not raised");
    assert_eq!(report.body, MethodBodyState::NotInspected);
    assert!(report.reads.is_empty());
    assert!(counted_usage_is_zero(&budget.usage()));

    // The fixture declares no method `run()V`: the class definition is read — that read is a
    // fact and is recorded — but there is no body to state, so the body plane stays
    // `NotInspected` and the member lookup's own code is the failure.
    let absent = method_id(&fixture.definition, b"run", b"()V");
    let request = MethodAnalysisRequest {
        environment,
        method: absent.clone(),
        stages: vec![AnalysisStage::RawFacts],
    };
    let mut budget = Budget::new(analysis_limits());
    let report = Engine::new()
        .analyze_method(
            std::slice::from_ref(&fixture.snapshot),
            &request,
            &mut budget,
        )
        .expect("a legal request is answered, not raised");
    assert_eq!(
        report.body,
        MethodBodyState::NotInspected,
        "{absent:?} has no declared body to state"
    );
    assert!(!matches!(
        report.body,
        MethodBodyState::Present | MethodBodyState::DeclaredWithoutBody { .. }
    ));
    assert_eq!(report.stages.len(), 1);
    assert_eq!(
        report.stages[0].state,
        StageState::Failed {
            code: "classfile_method_not_found".to_string()
        }
    );
    assert_eq!(
        failure_code(&report.execution),
        Some("classfile_method_not_found")
    );
    assert_eq!(report.reads.len(), 1, "the header read really happened");
    assert_eq!(report.coverage, Coverage::not_requested());
    assert_eq!(budget.usage().class_headers, 1);
    assert_eq!(
        budget.usage().method_bodies,
        0,
        "a member that is not declared has no body to attempt"
    );
    assert_eq!(budget.usage().ir_items, 0);
}

// ---------------------------------------------------------------------------
// Result planes are separate
// ---------------------------------------------------------------------------

#[test]
fn result_planes_are_reported_side_by_side_and_never_inferred() {
    let fixture = fixture();
    let environment = healthy_environment(&fixture);
    let content = std::slice::from_ref(&fixture.snapshot);
    // The run this test states its planes about publishes the canonical graph and then stops
    // inside a later phase on the step budget. The limit is the exact price of the phases up to
    // that artifact plus one step, so the stop lands behind it — which is the one shape that still
    // shows a produced artifact beside an unfinished run now that every phase this build declares
    // is implemented.
    let mut canonical_budget = Budget::new(analysis_limits());
    Engine::new()
        .analyze_method(
            content,
            &analysis_request(
                &fixture,
                environment.clone(),
                vec![AnalysisStage::CanonicalCfg],
            ),
            &mut canonical_budget,
        )
        .expect("a legal request is answered, not raised");
    let mut stopped = analysis_limits();
    stopped.analysis_steps = canonical_budget.usage().analysis_steps + 1;
    let mut budget = Budget::new(stopped);
    let report = Engine::new()
        .analyze_method(
            content,
            &analysis_request(&fixture, environment, vec![AnalysisStage::Ssa]),
            &mut budget,
        )
        .expect("a legal request is answered, not raised");

    // Product planes: the P2 baseline is bytecode, not Java, not compiled, not verified.
    assert_eq!(report.representation, Representation::Bytecode);
    assert_eq!(report.syntax_status, SyntaxStatus::NotJava);
    assert_eq!(report.compile_status, CompileStatus::NotAttempted);
    assert_eq!(report.verification, VerificationStatus::NotPerformed);
    // The canonical CFG is the artifact this pipeline produces, and this run published one, so
    // the quality plane is `Conservative` — while the run itself still ends inside a phase, which
    // is why the planes are reported side by side.
    assert_eq!(report.quality, Quality::Conservative);
    // The semantic plane is the run's own evidence and this run carries none: the phase that
    // checks the local invariants is the one the step budget stopped, so it never completed and
    // proved nothing. `Unproven` is the honest state of a produced artifact beside a stopped run,
    // and the two assertions on the same field below keep that from being read as a claim.
    assert_eq!(report.semantic_validation, SemanticValidation::Unproven);
    // The body plane states what was located: this run really read the member's body, and
    // `Present` says nothing about how much of the pipeline ran.
    assert_eq!(report.body, MethodBodyState::Present);
    assert_ne!(report.body, MethodBodyState::NotInspected);
    assert!(!matches!(
        report.body,
        MethodBodyState::DeclaredWithoutBody { .. }
    ));
    // `quality` is a property of a produced artifact: it is `Conservative` here because a
    // canonical CFG was produced, and it must not be read as a completed run — a phase behind the
    // artifact runs out of the step budget and stops the request.
    assert!(
        !matches!(report.execution, ExecutionReport::Complete { .. }),
        "a Conservative quality does not mean a completed run"
    );

    // Capability, range, termination and verification are separate planes: the phases up to the
    // canonical graph really completed and the body was fully covered, while a later phase stops
    // on the budget layer's step dimension — and none of that says anything about the product
    // planes above.
    let ExecutionReport::Partial {
        reason:
            TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
            },
        ..
    } = &report.execution
    else {
        panic!(
            "a phase behind the canonical graph stops on the step budget: {:?}",
            report.execution
        );
    };
    assert_eq!(
        diagnostic_codes(&report.diagnostics),
        vec!["budget_exceeded_analysis_steps"]
    );
    assert_eq!(
        report
            .stages
            .iter()
            .filter(|stage| stage.state == StageState::Completed)
            .count(),
        4,
        "`raw_facts`, `raw_cfg`, `legacy_normalization` and `canonical_cfg` completed"
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema,
        "the bytecode range is complete even though a later phase stopped"
    );

    // The planes do not imply one another: a complete bytecode range does not make the
    // runtime plane complete, a budget stop does not become a refused capability, and a
    // present body does not grant semantic evidence or verification.
    assert_ne!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert_ne!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
    assert_eq!(report.verification, VerificationStatus::NotPerformed);
    assert_ne!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
    assert!(
        unsupported_code(&report.execution).is_none(),
        "a stop of the budget layer is not a capability this build refuses: {:?}",
        report.execution
    );
}

#[test]
fn entries_charge_nothing_and_read_no_bytes() {
    let fixture = fixture();
    let environment = healthy_environment(&fixture);
    let content = std::slice::from_ref(&fixture.snapshot);

    // Every counted limit is zero, so a single charged byte would be a `BudgetExceeded`
    // instead of a report.
    let resolution = Engine::new()
        .resolve_symbol(
            content,
            &request(environment.clone()),
            &mut Budget::new(zero_limits()),
        )
        .expect("no byte of content is read");
    assert!(counted_usage_is_zero(usage_of(&resolution.execution)));

    let declarations = Engine::new()
        .declaration_references(
            content,
            &declaration_query(&fixture, environment.clone()),
            &mut Budget::new(zero_limits()),
        )
        .expect("no byte of content is read");
    assert!(counted_usage_is_zero(usage_of(&declarations.execution)));

    let analysis = Engine::new()
        .analyze_method(
            content,
            &analysis_request(&fixture, environment, vec![AnalysisStage::Ssa]),
            &mut Budget::new(zero_limits()),
        )
        .expect("no byte of content is read");
    assert!(counted_usage_is_zero(usage_of(&analysis.execution)));
}

// ---------------------------------------------------------------------------
// Serde contract
// ---------------------------------------------------------------------------

#[test]
fn requests_and_identity_reject_unknown_fields_and_round_trip() {
    let fixture = fixture();
    let environment = healthy_environment(&fixture);

    let request = request(environment.clone());
    let json = serde_json::to_string(&request).expect("the request serializes");
    assert_eq!(
        serde_json::from_str::<ResolutionRequest>(&json).expect("the request round-trips"),
        request
    );
    let mut value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    value["unexpected"] = serde_json::Value::Bool(true);
    assert!(
        serde_json::from_value::<ResolutionRequest>(value).is_err(),
        "request types deny unknown fields"
    );

    let query = declaration_query(&fixture, environment.clone());
    let json = serde_json::to_string(&query).expect("the query serializes");
    assert_eq!(
        serde_json::from_str::<DeclarationRefQuery>(&json).expect("the query round-trips"),
        query
    );
    let mut value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    value["max_items_extra"] = serde_json::Value::Bool(true);
    assert!(serde_json::from_value::<DeclarationRefQuery>(value).is_err());

    let analysis = analysis_request(&fixture, environment, vec![AnalysisStage::Ssa]);
    let json = serde_json::to_string(&analysis).expect("the analysis request serializes");
    assert_eq!(
        serde_json::from_str::<MethodAnalysisRequest>(&json).expect("the analysis round-trips"),
        analysis
    );
    let mut value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    value["stages_extra"] = serde_json::Value::Bool(true);
    assert!(serde_json::from_value::<MethodAnalysisRequest>(value).is_err());

    let identity = EnvironmentIdentity {
        runtime: analysis.environment.runtime.clone(),
        domain_loaders: vec![loader("app")],
        providers: Vec::new(),
        content: vec![fixture.snapshot.id().clone()],
    };
    let json = serde_json::to_string(&identity).expect("the identity serializes");
    assert_eq!(
        serde_json::from_str::<EnvironmentIdentity>(&json).expect("the identity round-trips"),
        identity
    );
    let mut value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    value["digest"] = serde_json::Value::String("later".to_string());
    assert!(serde_json::from_value::<EnvironmentIdentity>(value).is_err());

    // The read record of the 2.2 closure: an identity-bearing value type of the reports, so it
    // round-trips and denies unknown fields like the other result records.
    let read = HeaderRead {
        loader: loader("app"),
        definition: fixture.definition.clone(),
        reason: ReadReason::HierarchyClosure,
    };
    let json = serde_json::to_string(&read).expect("a read record serializes");
    assert_eq!(
        serde_json::from_str::<HeaderRead>(&json).expect("a read record round-trips"),
        read
    );
    let mut value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    value["unexpected"] = serde_json::Value::Bool(true);
    assert!(
        serde_json::from_value::<HeaderRead>(value).is_err(),
        "a read record denies unknown fields"
    );
}

#[test]
fn closed_sets_serialize_as_snake_case_strings() {
    for code in EnvironmentProblemCode::ALL {
        let json = serde_json::to_string(&code).expect("a problem code serializes");
        assert_eq!(
            json,
            format!("\"{}\"", code.as_str()),
            "the diagnostic code and the serde name are one fact"
        );
        assert_eq!(
            serde_json::from_str::<EnvironmentProblemCode>(&json).expect("a code round-trips"),
            code
        );
    }
    assert_eq!(
        serde_json::to_string(&EnvironmentProblemCode::ParentCycle).unwrap(),
        "\"parent_cycle\""
    );
    assert_eq!(
        serde_json::to_string(&ResolutionState::IncompatibleClassChange).unwrap(),
        "\"incompatible_class_change\""
    );
    assert_eq!(
        serde_json::to_string(&ReferenceUse::InvokeDynamic).unwrap(),
        "\"invoke_dynamic\""
    );
    assert_eq!(
        serde_json::to_string(&ResolutionAnalysis::NotPerformed).unwrap(),
        "\"not_performed\""
    );
    assert_eq!(
        serde_json::to_string(&AnalysisStage::LegacyNormalization).unwrap(),
        "\"legacy_normalization\""
    );
    assert_eq!(
        serde_json::to_string(&Quality::Fallback).unwrap(),
        "\"fallback\""
    );
    assert_eq!(
        serde_json::to_string(&Representation::Bytecode).unwrap(),
        "\"bytecode\""
    );
    assert_eq!(
        serde_json::to_string(&SyntaxStatus::NotJava).unwrap(),
        "\"not_java\""
    );
    assert_eq!(
        serde_json::to_string(&CompileStatus::NotAttempted).unwrap(),
        "\"not_attempted\""
    );
    assert_eq!(
        serde_json::to_string(&SemanticValidation::Unproven).unwrap(),
        "\"unproven\""
    );
    assert_eq!(
        serde_json::to_string(&NoBodyKind::Abstract).unwrap(),
        "\"abstract\""
    );
    assert!(
        serde_json::from_str::<EnvironmentProblemCode>("\"unknown_code\"").is_err(),
        "the problem code set is closed"
    );
    assert!(serde_json::from_str::<ResolutionState>("\"not_performed\"").is_err());

    // The read reasons of the 2.2 closure: every variant's wire name is pinned, so renaming
    // one or dropping the snake_case rule cannot pass as a compatible change.
    for (reason, name) in [
        (ReadReason::RequestedDefinition, "requested_definition"),
        (ReadReason::ParentChain, "parent_chain"),
        (ReadReason::HierarchyClosure, "hierarchy_closure"),
        (ReadReason::DispatchScope, "dispatch_scope"),
        (ReadReason::MemberOwner, "member_owner"),
        (ReadReason::DriverMethodBody, "driver_method_body"),
    ] {
        assert_eq!(
            serde_json::to_string(&reason).expect("a read reason serializes"),
            format!("\"{name}\"")
        );
        assert_eq!(
            serde_json::from_str::<ReadReason>(&format!("\"{name}\""))
                .expect("a read reason round-trips"),
            reason
        );
    }
}

#[test]
fn all_lists_are_complete_and_align_with_the_serde_names() {
    // Every `ALL` list is compared against the variants its enum really declares, read from
    // the source that declares them: a variant added to a type without being added to its
    // `ALL` list fails here even when the exhaustive matches have been updated.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // The P2 modules were moved into `jarde-jvm` by the layering work (2.2), so they are read
    // where the guard resolves them — by identity, through the same table — instead of under
    // `src/`, which would go stale the moment the layout moves (it did).
    let environment_source =
        read_repository_file(root, resolve_guarded_file(root, &A17_ENVIRONMENT_MODULE));
    let ir_source = read_repository_file(root, resolve_guarded_file(root, &A17_IR_MODULE));
    // `BudgetDimension` and its `ALL` list moved to the reader crate with the rest of the
    // shared execution base (layer-jarde-crates 1.2); the declaration is read where it lives.
    let budget_source = read_repository_file(root, "crates/jarde-reader/src/budget.rs");

    // `EnvironmentProblemCode::ALL` is the closed set itself: every code appears exactly
    // once, at the index its exhaustive match names, and its `as_str` is the serde name the
    // reports and diagnostics use.
    assert_eq!(EnvironmentProblemCode::ALL.len(), 9);
    let mut indexes = Vec::new();
    for code in EnvironmentProblemCode::ALL {
        let index = problem_code_index(code);
        assert_eq!(EnvironmentProblemCode::ALL[index], code);
        assert_eq!(
            serde_json::to_string(&code).expect("a code serializes"),
            format!("\"{}\"", code.as_str())
        );
        indexes.push(index);
    }
    indexes.sort_unstable();
    assert_eq!(
        indexes,
        (0..EnvironmentProblemCode::ALL.len()).collect::<Vec<_>>(),
        "each code owns one distinct position in ALL"
    );
    assert_all_matches_declaration(
        "EnvironmentProblemCode::ALL",
        &declared_variants(&environment_source, "EnvironmentProblemCode"),
        &EnvironmentProblemCode::ALL
            .iter()
            .map(|code| code.as_str().to_string())
            .collect::<Vec<_>>(),
    );

    // `CountedBudgetDimension::ALL` is the zero-usage assertion's dimension set: adding a
    // counted dimension to `Limits`/`UsageSnapshot` without adding it here (and to the
    // exhaustive match above) does not compile. The P2 slice (1.3) appended its six
    // dimensions after the P0/P1 nine, which this count and the position assertions below
    // pin: the P1 dimensions keep positions 0..=8.
    assert_eq!(CountedBudgetDimension::ALL.len(), 15);
    assert_eq!(
        CountedBudgetDimension::ALL[..9],
        [
            CountedBudgetDimension::InputBytes,
            CountedBudgetDimension::ArchiveEntries,
            CountedBudgetDimension::EntryBytes,
            CountedBudgetDimension::ReadBytes,
            CountedBudgetDimension::ClassBytes,
            CountedBudgetDimension::AttributeBytes,
            CountedBudgetDimension::CodeBytes,
            CountedBudgetDimension::ResultItems,
            CountedBudgetDimension::OutputBytes,
        ],
        "the P0/P1 dimensions keep the positions P1 established"
    );
    let mut indexes = Vec::new();
    for dimension in CountedBudgetDimension::ALL {
        let index = counted_dimension_index(dimension);
        assert_eq!(CountedBudgetDimension::ALL[index], dimension);
        let code = serde_json::to_string(&dimension).expect("a dimension serializes");
        assert_eq!(code, format!("\"{}\"", dimension_code(dimension)));
        indexes.push(index);
    }
    indexes.sort_unstable();
    assert_eq!(
        indexes,
        (0..CountedBudgetDimension::ALL.len()).collect::<Vec<_>>(),
        "each counted dimension owns one distinct position in ALL"
    );
    assert_all_matches_declaration(
        "CountedBudgetDimension::ALL",
        &declared_variants(&budget_source, "CountedBudgetDimension"),
        &CountedBudgetDimension::ALL
            .iter()
            .map(|dimension| dimension_code(*dimension).to_string())
            .collect::<Vec<_>>(),
    );

    // The `BudgetDimension` split agrees with `ALL`: every counted dimension round-trips
    // through the total dimension enum, while the three non-counted dimensions stay out of
    // the counted set. `DependencyDepth` is the second high-water dimension, so it must be
    // rejected here exactly like `NestedDepth` — a dependency depth that could be charged
    // through `charge` would be a second, additive meaning for one limit.
    for dimension in CountedBudgetDimension::ALL {
        let total = BudgetDimension::from(dimension);
        assert_eq!(
            CountedBudgetDimension::try_from(total),
            Ok(dimension),
            "{dimension:?} must be the counted dimension of {total:?}"
        );
        assert!(counted_dimension_index(dimension) < CountedBudgetDimension::ALL.len());
    }
    for total in [
        BudgetDimension::NestedDepth,
        BudgetDimension::DependencyDepth,
        BudgetDimension::ElapsedMillis,
    ] {
        assert!(
            CountedBudgetDimension::try_from(total).is_err(),
            "{total:?} is not a counted dimension"
        );
    }
    assert_eq!(
        serde_json::to_string(&BudgetDimension::DependencyDepth).unwrap(),
        "\"dependency_depth\"",
        "the second high-water dimension has its own wire name"
    );

    // `AnalysisStage::ALL` is both the fixed phase order and the filter the engine normalizes
    // a request through: a stage missing from it cannot be requested at all, so a new stage
    // has to reach this list and the exhaustive `stage_index` match together.
    assert_eq!(AnalysisStage::ALL.len(), ANALYSIS_STAGE_COUNT);
    let mut indexes = Vec::new();
    for stage in AnalysisStage::ALL {
        let index = stage_index(stage);
        assert_eq!(AnalysisStage::ALL[index], stage);
        assert!(
            index < ANALYSIS_STAGE_COUNT,
            "{stage:?} claims a position outside ALL"
        );
        assert_eq!(
            serde_json::to_string(&stage).expect("a stage serializes"),
            format!("\"{}\"", stage_code(stage))
        );
        indexes.push(index);
    }
    indexes.sort_unstable();
    assert_eq!(
        indexes,
        (0..AnalysisStage::ALL.len()).collect::<Vec<_>>(),
        "each stage owns one distinct position in ALL"
    );
    assert_eq!(AnalysisStage::ALL[0], AnalysisStage::RawFacts);
    assert_eq!(
        AnalysisStage::ALL[ANALYSIS_STAGE_COUNT - 1],
        AnalysisStage::Ssa
    );
    assert_all_matches_declaration(
        "AnalysisStage::ALL",
        &declared_variants(&ir_source, "AnalysisStage"),
        &AnalysisStage::ALL
            .iter()
            .map(|stage| stage_code(*stage).to_string())
            .collect::<Vec<_>>(),
    );
}

#[test]
fn every_analysis_stage_is_requestable_and_schedules_its_own_prefix() {
    let fixture = fixture();
    let environment = healthy_environment(&fixture);

    // One request per stage, each asking for exactly that stage. Two failures are caught
    // here and nowhere else:
    //
    // * a stage outside `ALL` is dropped by the engine's normalization, so `requested_stages`
    //   would come back empty or missing the stage — the silent discard of invariant 6;
    // * the scheduled phases are the prefix of the phase order up to the requested stage, so
    //   the engine's own private phase position has to agree with `stage_index`.
    for stage in AnalysisStage::ALL {
        let index = stage_index(stage);
        let mut budget = Budget::new(zero_limits());
        let report = Engine::new()
            .analyze_method(
                std::slice::from_ref(&fixture.snapshot),
                &analysis_request(&fixture, environment.clone(), vec![stage]),
                &mut budget,
            )
            .expect("a legal request is answered, not raised");
        assert_eq!(
            report.requested_stages,
            vec![stage],
            "{stage:?} was requested and must not be silently dropped"
        );
        assert_eq!(
            report
                .stages
                .iter()
                .map(|scheduled| scheduled.stage)
                .collect::<Vec<_>>(),
            AnalysisStage::ALL[..=index].to_vec(),
            "{stage:?} schedules its own prefix of the phase order"
        );
    }

    // The last stage of the order schedules every phase, so `ALL` cannot be a truncated
    // prefix of the pipeline without this failing.
    let report = Engine::new()
        .analyze_method(
            std::slice::from_ref(&fixture.snapshot),
            &analysis_request(&fixture, environment, vec![AnalysisStage::Ssa]),
            &mut Budget::new(zero_limits()),
        )
        .expect("a legal request is answered, not raised");
    assert_eq!(report.stages.len(), ANALYSIS_STAGE_COUNT);
}

/// Serde code of one counted budget dimension, read from its own serialization.
///
/// Kept as a match so a new dimension has to be classified here as well.
fn dimension_code(dimension: CountedBudgetDimension) -> &'static str {
    match dimension {
        CountedBudgetDimension::InputBytes => "input_bytes",
        CountedBudgetDimension::ArchiveEntries => "archive_entries",
        CountedBudgetDimension::EntryBytes => "entry_bytes",
        CountedBudgetDimension::ReadBytes => "read_bytes",
        CountedBudgetDimension::ClassBytes => "class_bytes",
        CountedBudgetDimension::AttributeBytes => "attribute_bytes",
        CountedBudgetDimension::CodeBytes => "code_bytes",
        CountedBudgetDimension::ResultItems => "result_items",
        CountedBudgetDimension::OutputBytes => "output_bytes",
        CountedBudgetDimension::ClassHeaders => "class_headers",
        CountedBudgetDimension::MethodBodies => "method_bodies",
        CountedBudgetDimension::IrItems => "ir_items",
        CountedBudgetDimension::IrEdges => "ir_edges",
        CountedBudgetDimension::AnalysisSteps => "analysis_steps",
        CountedBudgetDimension::NormalizationClones => "normalization_clones",
    }
}

#[test]
fn payload_enums_use_the_kind_tag_and_round_trip() {
    let cases: [(StageState, &str); 3] = [
        (StageState::NotRequested, "{\"kind\":\"not_requested\"}"),
        (StageState::NotPerformed, "{\"kind\":\"not_performed\"}"),
        (
            StageState::Failed {
                code: "canonical_cfg_invalid".to_string(),
            },
            "{\"kind\":\"failed\",\"code\":\"canonical_cfg_invalid\"}",
        ),
    ];
    for (state, expected) in cases {
        let json = serde_json::to_string(&state).expect("a stage state serializes");
        assert_eq!(json, expected);
        assert_eq!(
            serde_json::from_str::<StageState>(&json).expect("a stage state round-trips"),
            state
        );
    }

    let evidence = OpenWorldEvidence::OrderedRoot { index: 2 };
    let json = serde_json::to_string(&evidence).expect("evidence serializes");
    assert_eq!(json, "{\"kind\":\"ordered_root\",\"index\":2}");
    assert_eq!(
        serde_json::from_str::<OpenWorldEvidence>(&json).unwrap(),
        evidence
    );

    let member = OriginMember::MethodPoint {
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: SnapshotId("snap".to_string()),
                },
                class_bytes: ClassBytesId {
                    digest: Digest("digest".to_string()),
                    length: 10,
                },
                variant: PhysicalVariant::Base,
            },
            name: bytes(b"run"),
            descriptor: bytes(b"()V"),
        },
        bci: 7,
    };
    let json = serde_json::to_string(&member).expect("an origin member serializes");
    assert!(
        json.starts_with("{\"kind\":\"method_point\",\"method\":{"),
        "{json}"
    );
    assert!(json.ends_with(",\"bci\":7}"), "{json}");
    assert_eq!(
        serde_json::from_str::<OriginMember>(&json).unwrap(),
        member,
        "an origin member round-trips with class coordinates only"
    );

    let mut origin = OriginSet::default();
    origin.insert(member.clone());
    origin.insert(member.clone());
    assert_eq!(origin.members.len(), 1, "equal members are deduplicated");
    let json = serde_json::to_string(&origin).expect("an origin set serializes");
    assert_eq!(serde_json::from_str::<OriginSet>(&json).unwrap(), origin);
}

#[test]
fn body_state_and_subject_shapes_are_pinned() {
    // The internal tag and the payload field must not both be `kind`, so the payload field
    // name is the line-format name and there is no hidden `rename`.
    let state = MethodBodyState::DeclaredWithoutBody {
        no_body_kind: NoBodyKind::Native,
    };
    let json = serde_json::to_string(&state).expect("a body state serializes");
    assert_eq!(
        json,
        "{\"kind\":\"declared_without_body\",\"no_body_kind\":\"native\"}"
    );
    assert_eq!(
        serde_json::from_str::<MethodBodyState>(&json).expect("a body state round-trips"),
        state
    );
    for (state, expected) in [
        (
            MethodBodyState::NotInspected,
            "{\"kind\":\"not_inspected\"}",
        ),
        (MethodBodyState::Present, "{\"kind\":\"present\"}"),
        (
            MethodBodyState::DeclaredWithoutBody {
                no_body_kind: NoBodyKind::Abstract,
            },
            "{\"kind\":\"declared_without_body\",\"no_body_kind\":\"abstract\"}",
        ),
    ] {
        let json = serde_json::to_string(&state).expect("a body state serializes");
        assert_eq!(json, expected);
        assert_eq!(
            serde_json::from_str::<MethodBodyState>(&json).expect("a body state round-trips"),
            state
        );
    }
    // The old single-`kind` payload shape is not accepted, so no reader can keep reading it.
    assert!(
        serde_json::from_str::<MethodBodyState>(
            "{\"kind\":\"declared_without_body\",\"kind\":\"native\"}"
        )
        .is_err()
    );

    // `EnvironmentSubject` has three newtype variants, which serde cannot serialize under
    // an internal tag, so the shape is the externally tagged one.
    let cases: [(EnvironmentSubject, &str); 3] = [
        (
            EnvironmentSubject::Loader(loader("app")),
            "{\"loader\":\"app\"}",
        ),
        (
            EnvironmentSubject::Provider(ProviderId("app-headers".to_string())),
            "{\"provider\":\"app-headers\"}",
        ),
        (
            EnvironmentSubject::Root {
                loader: loader("app"),
                index: 1,
            },
            "{\"root\":{\"loader\":\"app\",\"index\":1}}",
        ),
    ];
    for (subject, expected) in cases {
        let json = serde_json::to_string(&subject).expect("a subject serializes");
        assert_eq!(json, expected);
        assert_eq!(
            serde_json::from_str::<EnvironmentSubject>(&json).expect("a subject round-trips"),
            subject
        );
    }

    // The fourth variant nests a tagged symbol, so the symbol kind stays readable.
    let symbol = EnvironmentSubject::Symbol(constructor());
    let json = serde_json::to_string(&symbol).expect("a symbol subject serializes");
    assert!(
        json.starts_with("{\"symbol\":{\"kind\":\"method\",\"owner\":"),
        "{json}"
    );
    assert_eq!(
        serde_json::from_str::<EnvironmentSubject>(&json).unwrap(),
        symbol
    );

    let problem = EnvironmentProblem {
        code: EnvironmentProblemCode::ProviderRootUnbound,
        subject: EnvironmentSubject::Root {
            loader: loader("app"),
            index: 0,
        },
        message: "unbound".to_string(),
    };
    let json = serde_json::to_string(&problem).expect("a problem serializes");
    assert_eq!(
        serde_json::from_str::<EnvironmentProblem>(&json).unwrap(),
        problem
    );
}

// ---------------------------------------------------------------------------
// A17: the physical entry points stay free of the P2 modules
// ---------------------------------------------------------------------------

/// One module the A17 guard talks about: a layout-independent identity plus every physical
/// path that identity may have.
///
/// The guard has to hold both before and after the crate split (2.1/2.2), so no path is
/// written down on its own. A guarded module carries the pre-split path under `src/` and the
/// post-split one below `crates/`, and `resolve_layout` asserts that exactly one of them is on
/// disk. Enumerating nothing must fail, and a tree where both layouts coexist must fail loudly
/// rather than let the guard pick the half that is still in place.
struct GuardedModule {
    /// Layout-independent name. Guard messages and sandbox cases use this, not a path.
    identity: &'static str,
    /// Every path this identity may occupy, pre-split first. Exactly one has to exist.
    candidates: &'static [&'static str],
}

const A17_QUERY_MODULE: GuardedModule = GuardedModule {
    identity: "query.rs",
    candidates: &["src/query.rs", "crates/jarde-query/src/query.rs"],
};
const A17_XREF_MODULE: GuardedModule = GuardedModule {
    identity: "xref/mod.rs",
    candidates: &["src/xref/mod.rs", "crates/jarde-query/src/xref/mod.rs"],
};
const A17_XREF_CODE_MODULE: GuardedModule = GuardedModule {
    identity: "xref/code.rs",
    candidates: &["src/xref/code.rs", "crates/jarde-query/src/xref/code.rs"],
};
const A17_XREF_METADATA_MODULE: GuardedModule = GuardedModule {
    identity: "xref/metadata.rs",
    candidates: &[
        "src/xref/metadata.rs",
        "crates/jarde-query/src/xref/metadata.rs",
    ],
};
const A17_XREF_BOOTSTRAP_MODULE: GuardedModule = GuardedModule {
    identity: "xref/bootstrap.rs",
    candidates: &[
        "src/xref/bootstrap.rs",
        "crates/jarde-query/src/xref/bootstrap.rs",
    ],
};
const A17_XREF_RESOURCE_MODULE: GuardedModule = GuardedModule {
    identity: "xref/resource.rs",
    candidates: &[
        "src/xref/resource.rs",
        "crates/jarde-query/src/xref/resource.rs",
    ],
};

/// The six P1 physical-entry modules the guard exists for.
///
/// Only these carry a size expectation: a file added later may legitimately be tiny.
const A17_EXPECTED_MODULES: [GuardedModule; 6] = [
    A17_QUERY_MODULE,
    A17_XREF_MODULE,
    A17_XREF_CODE_MODULE,
    A17_XREF_METADATA_MODULE,
    A17_XREF_BOOTSTRAP_MODULE,
    A17_XREF_RESOURCE_MODULE,
];

/// The directory the wildcard half of the guard walks: `src/xref` before the split, the same
/// directory below `crates/jarde-query` after it.
///
/// The six identities above are the floor, not the whole guarded set: every `*.rs` below this
/// directory is guarded as soon as it exists.
const A17_XREF_DIRECTORY: GuardedModule = GuardedModule {
    identity: "xref/",
    candidates: &["src/xref", "crates/jarde-query/src/xref"],
};

/// The module P4 added beside `query.rs`: the plugin entry, and the rest of the guarded floor.
///
/// The guarded set is the physical-entry surface of X1, and the crate is what that surface is:
/// `query.rs` and the modules below `xref/` were all of it when the guard was written, and
/// `plugin.rs` joined them — an entry that reports a `PhysicalView` and a plugin analysis, and may
/// no more start the resolver or name the recovery layer than the modules beside it. Leaving it out
/// is what would need a reason, since the set is enumerated by identity here and a module the
/// enumeration does not know is a module the table never reads.
///
/// Both homes are listed, pre-split first, exactly as the P1 table spells them, and
/// `resolve_layout` still asserts that exactly one is on disk. The module was added after the split,
/// so the first candidate is the identity's home in the pre-split layout this table is written for
/// rather than a file that ever existed: naming it keeps this identity from being the one member
/// whose path is written down in a single layout, which is what `assert_single_layout` compares.
///
/// The two crates below the guarded one are deliberately **not** added, and that is a judgement
/// about what A17 asserts rather than an omission. `jarde-reader` and `jarde-jvm` are where the P2
/// types *live*: `reflection.rs` reaches `crate::environment`, `crate::resolver` and `crate::ir` on
/// purpose and names `jarde_java::pass::RuleVersion` in prose, and `runtime_matrix.rs` names
/// `jarde_jvm::providers` in prose — the module-path half of the detector matches a source as
/// written, comments included, so both files would be reported by a table they are entitled to
/// write. Measured by adding both identities under this constant: `reflection.rs` comes back with
/// `crate::environment`, `crate::resolver`, `crate::ir` and the resolution/IR type names it has to
/// import, `runtime_matrix.rs` with the single `jarde_jvm::` its prose carries. "A guarded file may
/// not name the P2 modules" is a rule about the physical entries that *read* those planes, not about
/// the modules that own them; and the recovery layer is out of the lower crates' reach structurally,
/// by the same refused dependency edge [`A17_MODULE_TOKENS`] records, so adding them would widen the
/// policy without adding a reach that could be observed.
const A17_PLUGIN_MODULE: GuardedModule = GuardedModule {
    identity: "plugin.rs",
    candidates: &["src/plugin.rs", "crates/jarde-query/src/plugin.rs"],
};

/// The modules the guarded set gained after the P1 table was written: same crate, same surface,
/// addressed by identity like every other one so none of them can fall out of the set silently.
const A17_ADDED_MODULES: [GuardedModule; 1] = [A17_PLUGIN_MODULE];

/// The module that is allowed, and required, to call the P2 entries: the P2 driver.
///
/// It is the guard's positive control, so its path is resolved like every other one and moves
/// with the driver: 2.2 puts it next to the P2 modules in `jarde-jvm`. If that file ends up
/// somewhere else, the resolution fails and says so instead of quietly losing the control.
const A17_CONTROL_MODULE: GuardedModule = GuardedModule {
    identity: "engine.rs",
    candidates: &["src/engine.rs", "crates/jarde-jvm/src/engine.rs"],
};

/// The environment module: the type table's first source, and the module the contract test that
/// compares the `ALL` lists with the enum declarations reads.
const A17_ENVIRONMENT_MODULE: GuardedModule = GuardedModule {
    identity: "environment.rs",
    candidates: &["src/environment.rs", "crates/jarde-jvm/src/environment.rs"],
};

/// The resolver module, addressed by identity like every other one.
const A17_RESOLVER_MODULE: GuardedModule = GuardedModule {
    identity: "resolver.rs",
    candidates: &["src/resolver.rs", "crates/jarde-jvm/src/resolver.rs"],
};

/// The IR module, addressed by identity like every other one.
const A17_IR_MODULE: GuardedModule = GuardedModule {
    identity: "ir.rs",
    candidates: &["src/ir.rs", "crates/jarde-jvm/src/ir.rs"],
};

/// The P2 modules the type table is derived from: their `pub` declarations are its source.
const A17_P2_SOURCE_MODULES: [GuardedModule; 3] =
    [A17_ENVIRONMENT_MODULE, A17_RESOLVER_MODULE, A17_IR_MODULE];

/// Number of guarded files the repository has today.
///
/// The enumeration below is not allowed to silently find nothing or to lose a file, so the
/// real count is asserted as well: `query.rs`, the five modules below the xref directory, and the
/// module added beside `query.rs`.
const A17_GUARDED_FILES: usize = 7;

/// Smallest plausible size of one of the six P1 files, in bytes.
const A17_MIN_SOURCE_LEN: usize = 1_000;

/// Whether a guarded identity is one of the modules that carry that floor.
///
/// The floor is a property of the module, not of the path it happens to have, so the size
/// rule follows the identity through the split exactly as the enumeration does.
fn carries_size_floor(identity: &str) -> bool {
    A17_EXPECTED_MODULES
        .iter()
        .any(|module| module.identity == identity)
}

/// Module-path tokens that make a physical-entry module reach a P2 module.
///
/// The bare `resolver::` / `environment::` / `ir::` forms close the crate-root re-export
/// route (`use crate::ResolutionReport as _;` keeps the type name instead) and the
/// `use crate::{…}` group form.
///
/// `cfg` (the raw CFG constructor of 3.3), `passes` (the pass table) and `call_context` (the
/// call-context builder of 3.4) are P2 modules the physical entries must not reach either, and
/// the derived type table does not cover them: `use crate::cfg::raw_cfg;` names a builder whose
/// own types live below the module and whose name no declaration in the three derived modules
/// contains, so the module path is the only signal. The `super::`-relative spelling of the
/// same reach — `super::cfg::…` from a `src/xref/` module named the crate root those P2 modules
/// lived in before the split — is covered by the bare forms.
///
/// The crate split (2.1/2.2) moves the P2 modules into `crates/jarde-jvm` and the guarded
/// files into `crates/jarde-query`, where the same reach is spelled by package path instead.
/// Both spellings stay in the table: `crate::…` keeps naming the P2 modules inside `jarde-jvm`
/// (the P2 driver is the positive control there), and a guarded file may not reach them
/// through their package path either. `jarde_query::` is the guarded crate's own package path
/// — a guarded file that names it is not talking about itself as `crate::` — and `jarde::` is
/// the facade above both: design §1 makes `jarde` depend on the three packages, so a
/// guarded file reaching for the facade is the same edge pointing back up the layering.
///
/// `jarde_java::` is the recovery layer's package path (P3 1.3a created the crate; the root
/// facade depends on it and the guarded crates do not). It is a token for the reason the other
/// package paths are: the guarded files may not reach the layer that holds region and AST
/// construction.
///
/// How strong that is today is a measured fact rather than a hope (P3 3.4): the guarded crates
/// cannot name the recovery layer at all, and not only because nobody writes the path — the
/// *dependency edge* is refused. Adding `jarde-java` to `crates/jarde-reader/Cargo.toml` or to
/// `crates/jarde-query/Cargo.toml` stops cargo with `error: cyclic package dependency: package
/// jarde-java depends on itself` (the cycle runs
/// `jarde-reader`/`jarde-query` → `jarde-java` → `jarde-jvm` → `jarde-query` → `jarde-reader`), so
/// no source in either crate is even compiled with a `Region` in scope. This token is the
/// supplement to a structural refusal that is stronger than a source guard — and it is what
/// re-checks the statement once the layering is rearranged: a move that stops closing the cycle
/// leaves this guard as the thing that still says no.
const A17_MODULE_TOKENS: [&str; 16] = [
    "crate::environment",
    "crate::resolver",
    "crate::ir",
    "crate::cfg",
    "crate::passes",
    "crate::call_context",
    "environment::",
    "resolver::",
    "ir::",
    "cfg::",
    "passes::",
    "call_context::",
    "jarde_jvm::",
    "jarde_query::",
    "jarde::",
    "jarde_java::",
];

/// Import forms that reach a whole P2 module under a name of the caller's choosing.
///
/// `use crate::*;` re-exports every P2 type without naming any of them, and
/// `use crate::{resolver as r};` renames the module, so neither the path token nor the type
/// name is guaranteed to appear next to the other: the glob and the `… as …` group form are
/// matched on their own.
///
/// The graph dependency is guarded here as well (3.5's A17 obligation): `petgraph::` covers
/// `use petgraph::algo::…` and every fully qualified path alike, and the two other forms cover
/// the alias and the `extern crate` spelling of the same reach. The raw CFG (3.3) is the
/// dependency's only consumer, and it is not a physical entry point: letting `query`/`xref`
/// reach a graph algorithm would be a new construction path in X0/X1 that no budget accounts
/// for.
///
/// The cross-crate reaches follow the same shape: an aliased package path
/// (`use jarde_jvm as jv;`) hides the `jarde_jvm::` token, so the `… as` form and the
/// `extern crate` spelling are matched on their own. `extern crate jarde` is deliberately not
/// a token: it is a prefix of the legitimate `extern crate jarde_reader`, while the facade's
/// own path is already covered by `jarde::` and its alias by `jarde as`. `extern crate
/// jarde_java` is not a prefix of anything else, and the recovery layer's alias is the same
/// shape as the other cross-crate ones.
const A17_IMPORT_TOKENS: [&str; 14] = [
    "crate::*",
    "environment as",
    "resolver as",
    "ir as",
    "petgraph::",
    "petgraph as",
    "extern crate petgraph",
    "jarde_jvm as",
    "jarde_query as",
    "jarde as",
    "jarde_java as",
    "extern crate jarde_jvm",
    "extern crate jarde_query",
    "extern crate jarde_java",
];

/// P2 type names that the P2 modules do not declare themselves.
///
/// `OriginSet`/`OriginMember` live in the shared identity layer
/// (`crates/jarde-reader/src/model.rs`, which P1 also owns), but they are still reachable from
/// the crate root and are still IR-only types.
const A17_EXTRA_TYPE_TOKENS: [&str; 2] = ["OriginSet", "OriginMember"];

/// Public type names one source declares with `pub struct` / `pub enum` / `pub trait` /
/// `pub type`.
fn declared_public_type_names(source: &str) -> Vec<String> {
    declared_type_names(source, true)
}

/// Type names one source declares, optionally restricted to the `pub` ones.
///
/// The visibility filter is a parameter because the two derivations ask different questions. The P2
/// table is read off public API, while the "shared name" subtraction of the recovery table has to
/// see private declarations too: `crates/jarde-query/src/xref/code.rs`'s own private `Shape` collides
/// with the recovery layer's `guard::Shape` exactly as a public one would.
fn declared_type_names(source: &str, public_only: bool) -> Vec<String> {
    let mut names = Vec::new();
    for line in source.lines() {
        let line = line.trim();
        if line.starts_with("//") || line.starts_with('#') {
            continue;
        }
        let rest = if public_only {
            match line.strip_prefix("pub ") {
                Some(rest) => rest,
                None => continue,
            }
        } else {
            strip_visibility(line)
        };
        let Some(rest) = ["struct ", "enum ", "trait ", "type ", "union "]
            .into_iter()
            .find_map(|keyword| rest.strip_prefix(keyword))
        else {
            continue;
        };
        let name = rest
            .split(|character: char| !character.is_alphanumeric() && character != '_')
            .next()
            .unwrap_or_default();
        if !name.is_empty() {
            names.push(name.to_string());
        }
    }
    names
}

/// Drops the visibility prefix of a declaration line — `pub `, `pub(crate) `, `pub(super) ` and
/// `pub(in …) ` alike — and returns a private declaration unchanged.
fn strip_visibility(line: &str) -> &str {
    if let Some(rest) = line.strip_prefix("pub ") {
        return rest;
    }
    if let Some(rest) = line.strip_prefix("pub(")
        && let Some((_, rest)) = rest.split_once(") ")
    {
        return rest;
    }
    line
}

/// Variant names of every `enum` body in one source, at any visibility.
///
/// A variant is a declaration too, and it collides the same way: `ConsumerKind::Type` is the query
/// layer's own name for one of its categories, and `crates/jarde-query/src/xref/code.rs` declares its
/// own private `Shape` with a `DynamicSite` variant. Those words are the guarded layer's vocabulary,
/// so they cannot be read as a reach into the recovery layer — and reading the variants here is what
/// keeps that subtraction derived instead of hand-written.
fn declared_variant_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let lines = source.lines().collect::<Vec<_>>();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim();
        let is_enum = !trimmed.starts_with("//")
            && !trimmed.starts_with('#')
            && strip_visibility(trimmed).starts_with("enum ");
        if is_enum {
            let indentation = line.len() - line.trim_start().len();
            index += 1;
            while index < lines.len() {
                let inner = lines[index];
                let inner_trimmed = inner.trim();
                let closes = inner_trimmed.starts_with('}')
                    && inner.len() - inner.trim_start().len() <= indentation;
                if closes {
                    break;
                }
                if !inner_trimmed.starts_with("//") && !inner_trimmed.starts_with('#') {
                    let name = inner_trimmed
                        .split(|character: char| !character.is_alphanumeric() && character != '_')
                        .next()
                        .unwrap_or_default();
                    if name.starts_with(char::is_uppercase) {
                        names.push(name.to_string());
                    }
                }
                index += 1;
            }
        }
        index += 1;
    }
    names
}

/// The lines of one source that are code: a line whose trimmed text starts with `//` is a comment,
/// and a comment cannot name a type.
///
/// Only the *name* half of the guard reads this (see [`p2_tokens_in`]). The derived names include
/// ordinary words — the recovery layer's `Operation`, or the query layer's own comment saying
/// "Continuation value of one page" — so matching them against prose would report a sentence as a
/// reach into another layer. Module paths keep matching the source as written, which is the guard's
/// long-standing behaviour, and a trailing comment on a code line still counts as code text: this
/// only removes a false positive, it never hides a written reference.
fn code_lines(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The P2 type names the guard looks for, derived from the P2 modules themselves.
///
/// A hand-written list is exactly what the guard must not rely on: a P2 type it forgets is a
/// type that can be imported from a physical entry without the guard noticing. The names come
/// from the declarations, and the test asserts that every derived name is also present in the
/// token table, so a broken derivation cannot pass silently.
///
/// The three modules are addressed by identity, so the derivation reads them where the split
/// put them (`crates/jarde-jvm/src/…` after 2.2) instead of losing the table the moment `src/`
/// stops holding them.
fn derived_p2_type_tokens(root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    for module in &A17_P2_SOURCE_MODULES {
        let relative = resolve_guarded_file(root, module);
        let source = read_repository_file(root, relative);
        let declared = declared_public_type_names(&source);
        assert!(
            !declared.is_empty(),
            "{relative} declares no public type: the derivation is broken"
        );
        names.extend(declared);
    }
    names.extend(A17_EXTRA_TYPE_TOKENS.map(str::to_string));
    names.sort();
    names.dedup();
    names
}

/// The directory the recovery-layer derivation reads: the crate P3 1.3a created, whose declarations
/// are the recovery layer's own vocabulary (region, AST, source map, report and the pattern records).
const A17_RECOVERY_DIRECTORY: GuardedModule = GuardedModule {
    identity: "jarde-java/src",
    candidates: &["crates/jarde-java/src"],
};

/// The source directories of the layers a guarded file may legitimately use: the query crate the
/// guarded files live in, and the two below it.
const A17_SHARED_SOURCE_DIRECTORIES: [GuardedModule; 3] = [
    GuardedModule {
        identity: "jarde-reader/src",
        candidates: &["crates/jarde-reader/src"],
    },
    GuardedModule {
        identity: "jarde-jvm/src",
        candidates: &["crates/jarde-jvm/src"],
    },
    GuardedModule {
        identity: "jarde-query/src",
        candidates: &["crates/jarde-query/src"],
    },
];

/// Every `*.rs` below one resolved directory, in path order, with the text of each.
///
/// The walk is what makes both derivations follow the tree: a file added to any of these crates is
/// read as soon as it exists, without a table to bring along.
fn directory_sources(root: &Path, directory: &GuardedModule) -> Vec<GuardedSource> {
    let resolved = resolve_guarded_directory(root, directory);
    let mut paths = Vec::new();
    collect_rs_files(&root.join(resolved), &mut paths);
    paths.sort();
    assert!(
        !paths.is_empty(),
        "{} holds no `.rs` file: the derivation is broken",
        directory.identity
    );
    paths
        .into_iter()
        .map(|path| {
            let label = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let source = read_repository_file(root, &label);
            GuardedSource {
                identity: label.clone(),
                label,
                source,
            }
        })
        .collect()
}

/// The names a guarded file may carry without reaching the recovery layer: every declared item and
/// every enum variant of the layers it is allowed to use.
///
/// This is the second half of the recovery derivation, and it is derived as well — a name is only
/// usable as a signal when it distinguishes the recovery layer from the vocabulary the guarded files
/// already have. `Provenance` is declared by `jarde-reader`, `Type` is `ConsumerKind::Type` in the
/// query layer's own schema, and `Shape` is that layer's own private reference shape; banning those
/// names would report the query's own code as a reach into the recovery layer. What is read here are
/// *declarations*, never usages, so an injected `use jarde_java::Region;` stays a violation: no layer
/// below declares `Region`.
fn derived_shared_type_tokens(root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    for directory in &A17_SHARED_SOURCE_DIRECTORIES {
        for source in directory_sources(root, directory) {
            names.extend(declared_type_names(&source.source, false));
            names.extend(declared_variant_names(&source.source));
        }
    }
    names.sort();
    names.dedup();
    assert!(
        names.len() >= 300,
        "the shared-vocabulary derivation found only {} names: it is broken",
        names.len()
    );
    names
}

/// Type names of the recovery layer that the guarded files must not carry.
///
/// The names come from the declarations of everything below `crates/jarde-java/src` — a hand-written
/// list is exactly what the guard must not rely on, because a type it forgets is a type a physical
/// entry could import without being noticed — and the names the layers below already declare are
/// subtracted, so what is left is the set that really distinguishes the recovery layer.
///
/// Two properties, said here because they are easy to overstate. The guarded files cannot *name*
/// these types today, and the reason is structural rather than textual: the dependency edge that
/// would be needed is refused by cargo as a cycle (see [`A17_MODULE_TOKENS`], where the refusal was
/// measured), so no guarded source is compiled with the recovery layer in scope and this table is
/// the supplement to that refusal, not a replacement for it — it is what keeps the statement true
/// after the layering is rearranged. And the table is about *names*: it is a textual policy over the
/// guarded files, not a proof about what they can resolve. `OriginSet`/`OriginMember` are the shared
/// identity types the P2 table bans by name (`A17_EXTRA_TYPE_TOKENS`); `OriginSet` is declared by
/// `jarde-reader` and so is subtracted here, which the test asserts instead of assuming.
fn derived_recovery_type_tokens(root: &Path) -> Vec<String> {
    let mut declared = Vec::new();
    for source in directory_sources(root, &A17_RECOVERY_DIRECTORY) {
        declared.extend(declared_public_type_names(&source.source));
    }
    declared.sort();
    declared.dedup();
    assert!(
        declared.len() >= 60,
        "the recovery derivation found only {} type names: it is broken",
        declared.len()
    );

    let shared = derived_shared_type_tokens(root);
    let tokens = declared
        .into_iter()
        .filter(|name| !shared.contains(name))
        .collect::<Vec<_>>();
    for expected in [
        "Region",
        "StmtKind",
        "ExprKind",
        "SourceMap",
        "Segment",
        "RecoveryReport",
        "FallbackReason",
        "RecoveryOutcome",
        "MethodFacts",
        "RegionRecord",
        "Pass",
        "Precondition",
    ] {
        assert!(
            tokens.iter().any(|name| name == expected),
            "{expected} must be derived from the recovery layer and survive the subtraction: \
             {tokens:?}"
        );
    }
    assert!(
        tokens.len() >= 50,
        "the derived recovery table holds only {} names: {tokens:?}",
        tokens.len()
    );
    assert!(
        shared.iter().any(|name| name == "OriginSet"),
        "`OriginSet` is declared by `jarde-reader` (`model.rs`), which is why this table leaves that \
         name to `A17_EXTRA_TYPE_TOKENS`"
    );
    tokens
}

/// One guarded source file: its identity, its path relative to the scanned root — which is what
/// a violation names — and its text.
struct GuardedSource {
    identity: String,
    label: String,
    source: String,
}

/// Removes the whitespace that Rust allows around `::`.
///
/// `crate::\n    resolver \n ::\n ResolutionRequest` is one path in valid Rust, so the guard
/// compares the form where the whitespace around every `::` is gone. Only that whitespace is
/// removed: collapsing all of it would glue `use crate::ir;` into `usecrate::ir` and hide the
/// module path behind an identifier boundary.
fn normalize_module_paths(source: &str) -> String {
    let mut normalized = String::with_capacity(source.len());
    let characters = source.chars().collect::<Vec<_>>();
    let mut index = 0;
    while index < characters.len() {
        if characters[index] == ':' && characters.get(index + 1) == Some(&':') {
            // Trim the whitespace the path separator may carry on both sides.
            while normalized.ends_with(char::is_whitespace) {
                normalized.pop();
            }
            normalized.push_str("::");
            index += 2;
            while characters
                .get(index)
                .is_some_and(|next| next.is_whitespace())
            {
                index += 1;
            }
            continue;
        }
        normalized.push(characters[index]);
        index += 1;
    }
    normalized
}

/// Whether a normalized source contains `token` as a name or path segment.
///
/// The preceding character must not be an identifier character, so `pair::x` does not look
/// like an `ir::` reference while `{resolver::X`, ` crate::resolver` and `crate::resolver`
/// do. A longer name that starts with the token (`EnvironmentProblemCode` for
/// `EnvironmentProblem`) still matches, which is intended: both are P2 types.
fn contains_token(normalized: &str, token: &str) -> bool {
    let mut from = 0;
    while let Some(offset) = normalized[from..].find(token) {
        let index = from + offset;
        let preceded_by_identifier = normalized[..index]
            .chars()
            .next_back()
            .is_some_and(|previous| previous.is_alphanumeric() || previous == '_');
        if !preceded_by_identifier {
            return true;
        }
        from = index + 1;
    }
    false
}

/// Tokens of one source that reach a P2 module or the recovery layer, module paths first.
///
/// The module and import paths are matched against the source as written; the derived type names are
/// matched against its code lines ([`code_lines`]), because those names include ordinary words and a
/// sentence in a comment is not a reference.
fn p2_tokens_in(source: &str, type_tokens: &[String]) -> Vec<String> {
    let normalized = normalize_module_paths(source);
    let code = normalize_module_paths(&code_lines(source));
    A17_MODULE_TOKENS
        .into_iter()
        .map(str::to_string)
        .chain(A17_IMPORT_TOKENS.map(str::to_string))
        .filter(|token| contains_token(&normalized, token))
        .chain(
            type_tokens
                .iter()
                .filter(|token| contains_token(&code, token))
                .cloned(),
        )
        .collect()
}

/// The one path `module` names that exists under `root`, relative to `root`.
///
/// Exactly one candidate has to be there:
///
/// - none means the layout changed and nobody brought this table along. The guard refuses to
///   enumerate nothing, which is how "the files moved, the strings did not" would stay green,
/// - two or more means the pre-split and the post-split tree are both on disk. The guard says
///   so instead of picking whichever one it looked at first: either the move is half done or a
///   stale copy of the module is still there, and either way the guard is not looking at the
///   tree its author meant.
fn resolve_layout(
    root: &Path,
    module: &GuardedModule,
    present: impl Fn(&Path) -> bool,
) -> &'static str {
    let found = module
        .candidates
        .iter()
        .copied()
        .filter(|candidate| present(&root.join(candidate)))
        .collect::<Vec<_>>();
    assert_eq!(
        found.len(),
        1,
        "{}: exactly one layout must provide this module, found {found:?} (candidates {:?})",
        module.identity,
        module.candidates
    );
    found[0]
}

/// The file `module` is, in whichever layout the tree under `root` has.
fn resolve_guarded_file(root: &Path, module: &GuardedModule) -> &'static str {
    resolve_layout(root, module, Path::is_file)
}

/// The directory `module` is, in whichever layout the tree under `root` has.
fn resolve_guarded_directory(root: &Path, module: &GuardedModule) -> &'static str {
    resolve_layout(root, module, Path::is_dir)
}

/// Asserts the guarded sources are all in **one** layout, not half of each.
///
/// Every identity resolves on its own, so a tree that has `query.rs` under `src/` and the
/// xref directory under `crates/jarde-query/src/` — the shape a half-finished move leaves
/// behind — would satisfy each resolution separately, and the file-count and completeness
/// assertions cannot see it: they count what was resolved, and both halves resolve. The
/// guard would then be reading one module from the old home and its siblings from the new
/// one. Comparing the layout roots turns that tree into a failure that says so.
///
/// Every resolved file is compared with the directory, not only the two the check started with, so
/// a module added to the set later is inside the comparison from the moment it is enumerated.
fn assert_single_layout(files: &[(&'static str, &'static str)], xref: &'static str) {
    /// The repository-relative root a guarded path sits under: `src/` before the split,
    /// `crates/` after it. The candidates are written pre-split first, so the order here has
    /// to match their order.
    fn layout_of(relative: &str) -> usize {
        if relative.starts_with("src/") { 0 } else { 1 }
    }
    for (identity, relative) in files {
        assert_eq!(
            layout_of(relative),
            layout_of(xref),
            "{identity} ({relative}) and the guarded directory ({xref}) are in different layouts: \
             a half-moved tree has to fail rather than let each module resolve on its own"
        );
    }
}

/// Reads every guarded source under `root`: the `query.rs` identity, the modules the set gained
/// after it, and every `*.rs` below the resolved xref directory, including nested directories.
///
/// Both halves are resolved by layout, so the same walk covers the pre-split and the
/// post-split tree. The list is enumerated from the directory rather than written out by
/// hand, so a file added to the xref directory — and a `mod` line registering it — is guarded
/// as soon as it exists; the modules added beside `query.rs` are named by identity for the same
/// reason, so one of them cannot leave the set without the resolution failing.
fn guarded_sources(root: &Path) -> Vec<GuardedSource> {
    let mut identities = Vec::new();
    let mut listed = Vec::new();
    for module in std::iter::once(A17_QUERY_MODULE).chain(A17_ADDED_MODULES) {
        let relative = resolve_guarded_file(root, &module);
        identities.push((module.identity, relative));
        listed.push((module.identity.to_string(), root.join(relative)));
    }
    let xref = resolve_guarded_directory(root, &A17_XREF_DIRECTORY);
    assert_single_layout(&identities, xref);
    let mut xref_paths = Vec::new();
    collect_rs_files(&root.join(xref), &mut xref_paths);
    listed.extend(xref_paths.into_iter().map(|path| {
        let relative = path
            .strip_prefix(root.join(xref))
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        (format!("xref/{relative}"), path)
    }));
    // Sorted by identity, so the report order is the same in either layout.
    listed.sort();
    listed
        .into_iter()
        .map(|(identity, path)| {
            let label = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("guarded source {label} is unreadable: {error}"));
            GuardedSource {
                identity,
                label,
                source,
            }
        })
        .collect()
}

fn collect_rs_files(directory: &Path, paths: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory).unwrap_or_else(|error| {
        panic!(
            "guarded directory {} is unreadable: {error}",
            directory.display()
        )
    });
    for entry in entries {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            collect_rs_files(&path, paths);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
}

/// Violations of the guard under `root`: references to a P2 module, and an *expected* P1
/// file that is too small to be real source.
///
/// The size floor applies to the six P1 files only. A module added later may legitimately be
/// small, and failing it with "unexpectedly small" would push the next author to pad a file
/// instead of reading the rule.
fn guard_violations(root: &Path, type_tokens: &[String]) -> Vec<String> {
    let mut violations = Vec::new();
    for source in guarded_sources(root) {
        let tokens = p2_tokens_in(&source.source, type_tokens);
        if !tokens.is_empty() {
            violations.push(format!("{}: {}", source.label, tokens.join(", ")));
            continue;
        }
        if carries_size_floor(&source.identity) && source.source.len() <= A17_MIN_SOURCE_LEN {
            violations.push(format!(
                "{}: expected P1 file is unexpectedly small ({} bytes)",
                source.label,
                source.source.len()
            ));
        }
    }
    violations
}

#[test]
fn physical_entry_modules_do_not_reference_the_p2_modules() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sources = guarded_sources(root);
    let type_tokens = derived_p2_type_tokens(root);

    // The type list is derived from the P2 sources, and the derivation has to be able to see
    // the types the guard is supposed to catch: every derived name must be in the token table
    // *and* must be detected by the same matcher the guard uses, otherwise the table would be
    // complete on paper and blind in practice.
    assert!(
        type_tokens.len() >= 30,
        "the derivation found only {} P2 types: {type_tokens:?}",
        type_tokens.len()
    );
    let table = type_tokens.clone();
    for name in &type_tokens {
        let probe = format!("use crate::{{{name}}};");
        assert!(
            p2_tokens_in(&probe, &table).contains(name),
            "{name} is derived from the P2 sources but the matcher does not flag it"
        );
    }
    for expected in [
        "EnvironmentProblem",
        "ResolutionReport",
        "ResolutionState",
        "DeclarationRefReport",
        "StageState",
        "MethodAnalysisReport",
        "MethodBodyState",
        "OriginSet",
    ] {
        assert!(
            type_tokens.iter().any(|name| name == expected),
            "{expected} must be derived from the P2 sources: {type_tokens:?}"
        );
    }

    // The reference check runs first, so an injected reference is reported by the file that
    // carries it — including a file that was just added below the guarded xref directory.
    assert_eq!(
        guard_violations(root, &type_tokens),
        Vec::<String>::new(),
        "the physical entry points must not start the resolver or the IR"
    );

    // Non-vacuity of the enumeration: the guarded tree really is there, and it is neither
    // losing nor silently gaining a file. Every identity is resolved by layout first — a path
    // shape this table does not know fails there — and has to show up in what the directory
    // walk enumerated, the modules added to the set later included. A new `*.rs` below the xref
    // directory has to be reviewed here (its own contents are already scanned above), which is
    // what stops the guard from decaying into "the files that existed when it was written".
    for module in A17_EXPECTED_MODULES.iter().chain(A17_ADDED_MODULES.iter()) {
        let relative = resolve_guarded_file(root, module);
        assert!(
            sources.iter().any(|source| source.label == relative),
            "{relative} is not enumerated: {:?}",
            sources
                .iter()
                .map(|source| source.label.as_str())
                .collect::<Vec<_>>()
        );
    }
    assert_eq!(
        sources.len(),
        A17_GUARDED_FILES,
        "guarded sources: {:?}",
        sources
            .iter()
            .map(|source| source.label.as_str())
            .collect::<Vec<_>>()
    );

    // Positive control on real repository code: the P2 driver is the module that is allowed,
    // and required, to call the P2 entries, so the same detector flags it — and it is not part
    // of the guarded set, which is what makes the guard a policy rather than a detector that
    // happens to match nothing. The control's own path comes from the same identity table, so
    // 2.2 moving the driver into `jarde-jvm` keeps this control attached to the file it is
    // about instead of pinning `src/engine.rs`.
    let control = resolve_guarded_file(root, &A17_CONTROL_MODULE);
    let engine = read_repository_file(root, control);
    assert!(
        p2_tokens_in(&engine, &type_tokens).contains(&"crate::resolver".to_string()),
        "the detector must flag the module that does call the P2 entries ({control})"
    );
    assert!(
        !sources.iter().any(|source| source.label == control),
        "{control} calls the P2 entries by contract and is not guarded"
    );
}

/// The module that is allowed, and required, to call the recovery layer: the root facade's
/// `recover_method`.
///
/// It is the recovery half's positive control for the same reason `engine.rs` is the P2 half's: the
/// same detector has to flag real repository code that does make the reach, or a table that matches
/// nothing would look like a passing guard. Its path is resolved like every other one, so the control
/// stays attached to the file it is about.
const A17_RECOVERY_CONTROL_MODULE: GuardedModule = GuardedModule {
    identity: "facade.rs",
    candidates: &["src/facade.rs"],
};

/// A17 for the layer P3 added: a physical entry point may not name the recovery layer either.
///
/// The table is derived from the recovery layer's own declarations ([`derived_recovery_type_tokens`]),
/// not written down, so a new type in `jarde-java` is guarded as soon as it is declared. What this
/// does **not** claim: it is not the reason the guarded files cannot name those types today. That
/// reason is structural and was measured in P3 3.4 — the dependency edge that would be needed is
/// refused by cargo as a cycle (`jarde-reader`/`jarde-query` → `jarde-java` → `jarde-jvm` →
/// `jarde-query` → `jarde-reader`), so no guarded source is even compiled with the recovery layer in
/// scope. This test is the supplement that keeps a later rearrangement from going quiet, and its
/// falsification shows the half it really holds: an injected name (and an injected `jarde_java::`
/// path) in a guarded file is reported.
#[test]
fn physical_entry_modules_do_not_reference_the_recovery_layer() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tokens = derived_recovery_type_tokens(root);

    // The derivation is the table, so the table has to be able to see what it bans: every derived
    // name is flagged by the same matcher the guard uses.
    for name in &tokens {
        let probe = format!("use crate::{{{name}}};");
        assert!(
            p2_tokens_in(&probe, &tokens).contains(name),
            "{name} is derived from the recovery layer but the matcher does not flag it"
        );
    }

    // The guarded files carry none of the names, and the crate path a reference would have to write
    // is a module token as well.
    assert_eq!(
        guard_violations(root, &tokens),
        Vec::<String>::new(),
        "the physical entry points must not reach the recovery layer"
    );

    // Positive control on real repository code: the facade names the recovery layer by package path
    // and by type name, and it is not part of the guarded set.
    let control = resolve_guarded_file(root, &A17_RECOVERY_CONTROL_MODULE);
    let facade = read_repository_file(root, control);
    let flagged = p2_tokens_in(&facade, &tokens);
    assert!(
        flagged.contains(&"RecoveryReport".to_string()),
        "the detector must flag the module that does call the recovery layer ({control}): {flagged:?}"
    );
    let paths = p2_tokens_in(&facade, &A17_MODULE_TOKENS.map(str::to_string));
    assert!(
        paths.contains(&"jarde_java::".to_string()),
        "and the same control has to be flagged by the recovery layer's package path ({control}): \
         {paths:?}"
    );
    assert!(
        !guarded_sources(root)
            .iter()
            .any(|source| source.label == control),
        "{control} calls the recovery layer by contract and is not guarded"
    );
}

/// One sandbox layout: what a module identity is prefixed with in a tree built for a case.
///
/// The case tables are written in identities (`query.rs`, `xref/clean.rs`) and run once per
/// layout here, so a guard that only recognizes one of them — the pre-split tree today, the
/// post-split tree after 2.1/2.2 — fails on the other pass instead of passing until the move
/// lands.
struct SandboxLayout {
    /// Name of this layout in the case titles and in the temporary directories.
    name: &'static str,
    /// The prefix every identity carries under this layout.
    prefix: &'static str,
}

impl SandboxLayout {
    /// Where `identity` lives in this layout.
    fn path(&self, identity: &str) -> String {
        format!("{}{identity}", self.prefix)
    }
}

/// The two layouts the guard has to work in: the tree as it is today, and the tree the crate
/// split (2.1/2.2) leaves for the query side.
const SANDBOX_LAYOUTS: [SandboxLayout; 2] = [
    SandboxLayout {
        name: "pre_split",
        prefix: "src/",
    },
    SandboxLayout {
        name: "post_split",
        prefix: "crates/jarde-query/src/",
    },
];

#[test]
fn the_a17_guard_detects_rewritten_references_and_added_files() {
    // The shapes earlier versions of this guard missed, each written into its own throwaway
    // source tree: (a) a new file registered in `mod.rs`, (b) a crate-root re-export path
    // that drops the module name, (c) whitespace and newlines around `::`, (d) a
    // `use crate::{…}` group, (e) a bare module import, (f) a glob import plus a type that a
    // hand-written list happened to omit, (g) a grouped `as` alias, (h) a grouped alias for
    // another P2 module, (i) the cross-crate spellings the split makes reachable. Each case
    // runs in both layouts of `SANDBOX_LAYOUTS` with the same expectations. The tree is built
    // on disk, so enumeration — not a hand-written file list — is what has to find the
    // offender, and the type table is the derived one.
    struct Case {
        name: &'static str,
        /// Files of this case, in module identities: the layout decides where they land.
        files: &'static [(&'static str, &'static str)],
        /// Offending identities, in the order the guard reports them.
        offenders: &'static [&'static str],
        /// Text each offender's violation message has to contain.
        evidence: &'static [&'static str],
        /// Expected-file identities this case deliberately leaves below the size floor, to
        /// test that rule itself. Every other expected-file stub is padded, so a sandbox case
        /// shows the violation it was built for.
        tiny_files: &'static [&'static str],
    }
    const CLEAN_QUERY: &str = "//! clean physical entry\npub fn execute() {}\n";
    const CLEAN_XREF: &str = "pub fn scan() {}\n";
    let cases = [
        Case {
            name: "new_file",
            files: &[
                ("query.rs", CLEAN_QUERY),
                ("xref/mod.rs", "mod extra_probe;\nmod clean;\n"),
                ("xref/extra_probe.rs", "use crate::ir::AnalysisStage;\n"),
                ("xref/clean.rs", CLEAN_XREF),
            ],
            offenders: &["xref/extra_probe.rs"],
            evidence: &["crate::ir"],
            tiny_files: &[],
        },
        Case {
            name: "root_reexport",
            files: &[
                ("query.rs", "use crate::ResolutionReport as _;\n"),
                ("xref/mod.rs", "mod clean;\n"),
                ("xref/clean.rs", CLEAN_XREF),
            ],
            offenders: &["query.rs"],
            evidence: &["ResolutionReport"],
            tiny_files: &[],
        },
        Case {
            name: "whitespace",
            files: &[
                (
                    "query.rs",
                    "use crate::\n    resolver\n    ::\n    ResolutionRequest;\n",
                ),
                ("xref/mod.rs", "mod clean;\n"),
                (
                    "xref/clean.rs",
                    "use crate::\n    ir\n    ::\n    AnalysisStage;\n",
                ),
            ],
            offenders: &["query.rs", "xref/clean.rs"],
            evidence: &["ResolutionRequest", "AnalysisStage"],
            tiny_files: &[],
        },
        Case {
            name: "use_group",
            files: &[
                ("query.rs", "use crate::{resolver::ResolutionRequest};\n"),
                ("xref/mod.rs", "mod clean;\n"),
                ("xref/clean.rs", CLEAN_XREF),
            ],
            offenders: &["query.rs"],
            evidence: &["ResolutionRequest"],
            tiny_files: &[],
        },
        Case {
            // A module import with no sub-item and no P2 type name: only the `crate::ir`
            // path token can catch it, which is why that token is matched against the
            // boundary `use` leaves behind rather than against glued text.
            name: "bare_module_import",
            files: &[
                ("query.rs", "use crate::ir as p2_ir;\n"),
                ("xref/mod.rs", "mod clean;\n"),
                ("xref/clean.rs", CLEAN_XREF),
            ],
            offenders: &["query.rs"],
            evidence: &["crate::ir", "ir as"],
            tiny_files: &[],
        },
        Case {
            // H2: a glob import gives access to every P2 type without naming one, and the
            // type actually used here (`DispatchReport`) is one a hand-written short list is
            // likely to have missed.
            name: "glob_import",
            files: &[
                (
                    "query.rs",
                    "use crate::*;\nfn probe(r: DispatchReport) {}\n",
                ),
                ("xref/mod.rs", "mod clean;\n"),
                ("xref/clean.rs", CLEAN_XREF),
            ],
            offenders: &["query.rs"],
            evidence: &["crate::*", "DispatchReport"],
            tiny_files: &[],
        },
        Case {
            // H1: the group renames the module, so neither `crate::resolver` nor
            // `resolver::` appears; the derived type name is what remains.
            name: "group_alias",
            files: &[
                (
                    "query.rs",
                    "use crate::{resolver as r};\nfn probe(s: r::ResolutionState) {}\n",
                ),
                ("xref/mod.rs", "mod clean;\n"),
                ("xref/clean.rs", CLEAN_XREF),
            ],
            offenders: &["query.rs"],
            evidence: &["resolver as", "ResolutionState"],
            tiny_files: &[],
        },
        Case {
            name: "group_alias_other_module",
            files: &[
                ("query.rs", CLEAN_QUERY),
                ("xref/mod.rs", "mod clean;\n"),
                (
                    "xref/clean.rs",
                    "use crate::{ir as p2};\nfn probe(s: p2::StageState) {}\n",
                ),
            ],
            offenders: &["xref/clean.rs"],
            evidence: &["ir as", "StageState"],
            tiny_files: &[],
        },
        Case {
            // The graph dependency is reached through an ordinary `use`, a renamed alias or
            // the `extern crate` spelling; none of the P2 module tokens fires, so only the
            // petgraph tokens added with 3.3's first consumer can catch these.
            name: "petgraph_import",
            files: &[
                (
                    "query.rs",
                    "use petgraph::algo::kosaraju_scc;\nfn probe() {}\n",
                ),
                ("xref/mod.rs", "mod clean;\n"),
                ("xref/clean.rs", "use petgraph as graphs;\nfn probe() {}\n"),
                ("xref/tiny.rs", "extern crate petgraph;\n"),
            ],
            offenders: &["query.rs", "xref/clean.rs", "xref/tiny.rs"],
            evidence: &["petgraph::", "petgraph as", "extern crate petgraph"],
            tiny_files: &[],
        },
        Case {
            // 3.3 added the raw CFG (`cfg`) and the pass table (`passes`), 3.4 the call-context
            // builder (`call_context`). `use crate::cfg::raw_cfg;` is a construction path that
            // names no derived type, so only the module-path tokens can catch these spellings:
            // each builder through its module path, and a module under an alias of the caller's
            // choosing.
            name: "cfg_and_passes_module_paths",
            files: &[
                ("query.rs", "use crate::cfg::raw_cfg;\nfn probe() {}\n"),
                ("xref/mod.rs", "mod clean;\nmod tiny;\n"),
                (
                    "xref/clean.rs",
                    "use crate::passes::PASSES;\nfn probe() {}\n",
                ),
                (
                    "xref/tiny.rs",
                    "use crate::call_context::call_contexts;\nfn probe() {}\n",
                ),
            ],
            offenders: &["query.rs", "xref/clean.rs", "xref/tiny.rs"],
            evidence: &[
                "crate::cfg",
                "crate::passes",
                "crate::call_context",
                "call_context::",
            ],
            tiny_files: &[],
        },
        Case {
            // The module under an alias of the caller's choosing keeps the path token.
            name: "call_context_module_alias",
            files: &[
                ("query.rs", CLEAN_QUERY),
                ("xref/mod.rs", "mod tiny;\n"),
                (
                    "xref/tiny.rs",
                    "use crate::call_context as contexts;\nfn probe() {}\n",
                ),
            ],
            offenders: &["xref/tiny.rs"],
            evidence: &["crate::call_context"],
            tiny_files: &[],
        },
        Case {
            // The split gives the same reach a second spelling: inside `jarde-jvm` the P2
            // modules stay `crate::…`, but a guarded file in `jarde-query` has to name them by
            // package path, which no pre-split token covered.
            name: "cross_crate_package_path",
            files: &[
                ("query.rs", "use jarde_jvm::resolver::ResolutionReport;\n"),
                ("xref/mod.rs", "mod clean;\n"),
                ("xref/clean.rs", CLEAN_XREF),
            ],
            offenders: &["query.rs"],
            evidence: &["jarde_jvm::"],
            tiny_files: &[],
        },
        Case {
            // ... and the same reach under a name of the caller's choosing, which hides the
            // package path token. `jarde::` is the facade above the guarded crate: design §1
            // has `jarde` depending on the packages, so reaching for it is the edge pointing
            // back up the layering.
            name: "cross_crate_alias_and_facade",
            files: &[
                ("query.rs", CLEAN_QUERY),
                ("xref/mod.rs", "mod clean;\n"),
                (
                    "xref/clean.rs",
                    "use jarde_jvm as jv;\nuse jarde::Engine;\nfn probe() {}\n",
                ),
            ],
            offenders: &["xref/clean.rs"],
            evidence: &["jarde_jvm as", "jarde::"],
            tiny_files: &[],
        },
        Case {
            // The recovery layer is reached by the same spellings as the P2 modules, and the derived
            // recovery table is behind the same matcher: an alias for its package path plus one of
            // its type names is reported, in both layouts.
            name: "recovery_layer_alias",
            files: &[
                ("query.rs", CLEAN_QUERY),
                ("xref/mod.rs", "mod clean;\n"),
                (
                    "xref/clean.rs",
                    "use jarde_java as java;\nlet _: Option<Region> = None;\n",
                ),
            ],
            offenders: &["xref/clean.rs"],
            evidence: &["jarde_java as", "Region"],
            tiny_files: &[],
        },
        Case {
            // The module the guarded set gained after the P1 table is scanned by the same walk and
            // reported by the same matcher: enumerating a file is what makes it guarded, so this
            // case is what keeps the added identity from being listed and then not read.
            name: "added_module_reference",
            files: &[
                ("query.rs", CLEAN_QUERY),
                ("xref/mod.rs", "mod clean;\n"),
                ("xref/clean.rs", CLEAN_XREF),
                ("plugin.rs", "use jarde_java::RecoveryReport;\n"),
            ],
            offenders: &["plugin.rs"],
            evidence: &["jarde_java::", "RecoveryReport"],
            tiny_files: &[],
        },
        Case {
            // The size floor is about the six P1 files, not about the guarded set: a new
            // small module is legitimate and must not be reported as "unexpectedly small".
            name: "tiny_new_module",
            files: &[
                ("query.rs", CLEAN_QUERY),
                ("xref/mod.rs", "mod tiny;\n"),
                ("xref/tiny.rs", "pub fn tiny() {}\n"),
            ],
            offenders: &[],
            evidence: &[],
            tiny_files: &[],
        },
        Case {
            // ... and the same floor still applies to the files it exists for.
            name: "tiny_expected_file",
            files: &[
                ("query.rs", CLEAN_QUERY),
                ("xref/mod.rs", "mod clean;\n"),
                ("xref/clean.rs", CLEAN_XREF),
            ],
            offenders: &["query.rs"],
            evidence: &["unexpectedly small"],
            tiny_files: &["query.rs"],
        },
    ];

    // Both derived tables run through the same matcher, so the sandbox cases cover the recovery layer
    // as well: the set of the case below is the P2 table plus the recovery table.
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut type_tokens = derived_p2_type_tokens(manifest);
    type_tokens.extend(derived_recovery_type_tokens(manifest));
    type_tokens.sort();
    type_tokens.dedup();

    for layout in &SANDBOX_LAYOUTS {
        for case in &cases {
            let root = std::env::temp_dir().join(format!(
                "jarde-a17-guard-{}-{}-{}",
                std::process::id(),
                layout.name,
                case.name
            ));
            let _ = std::fs::remove_dir_all(&root);
            // Every tree carries the modules the guarded set gained after the P1 table, as clean
            // stubs. No case below is about them, and the guard resolves each guarded identity: a
            // tree without one would fail while resolving it instead of showing the fault the case
            // injects. The stubs are written first, so a case that does name one of them — the
            // added-module case does — still writes its own contents.
            write_added_module_stubs(&root, layout);
            for (identity, contents) in case.files.iter().copied() {
                write_sandbox_file(&root, layout, identity, contents, case.tiny_files);
            }

            let violations = guard_violations(&root, &type_tokens);
            let offending = violations
                .iter()
                .map(|violation| {
                    violation
                        .split_once(':')
                        .expect("a violation names its file")
                        .0
                        .to_string()
                })
                .collect::<Vec<_>>();
            let expected = case
                .offenders
                .iter()
                .map(|identity| layout.path(identity))
                .collect::<Vec<_>>();
            assert_eq!(
                offending, expected,
                "case {} in the {} layout: the guard must flag exactly {:?}, got {violations:?}",
                case.name, layout.name, case.offenders
            );
            for evidence in case.evidence {
                assert!(
                    violations
                        .iter()
                        .any(|violation| violation.contains(evidence)),
                    "case {} in the {} layout: the violation must name {evidence}, got \
                     {violations:?}",
                    case.name,
                    layout.name
                );
            }

            // A rewrite of the *same* clean tree stays quiet, so the sandbox itself — including
            // a tiny new module and the padded stubs — does not make every file look like an
            // offender.
            let clean_root = root.join("clean-copy");
            for (identity, contents) in case.files.iter().copied() {
                if case.offenders.contains(&identity) {
                    continue;
                }
                write_sandbox_file(&clean_root, layout, identity, contents, case.tiny_files);
            }
            // `query.rs` and the modules added later are always part of the guarded set, so a clean
            // file stands in for the removed offender. The stand-in is padded by
            // `write_sandbox_file` whenever its identity carries the size floor: in a size case the
            // offender *is* the tiny stub, and the control has to show the tree without the
            // injected fault rather than a tree that is missing a guarded file.
            for module in std::iter::once(A17_QUERY_MODULE).chain(A17_ADDED_MODULES) {
                let placeholder = clean_root.join(layout.path(module.identity));
                if !placeholder.exists() {
                    write_sandbox_file(&clean_root, layout, module.identity, CLEAN_QUERY, &[]);
                }
            }
            assert_eq!(
                guard_violations(&clean_root, &type_tokens),
                Vec::<String>::new(),
                "case {} in the {} layout: without the injected reference nothing is flagged",
                case.name,
                layout.name
            );

            let _ = std::fs::remove_dir_all(&root);
        }
    }
}

/// Writes the modules the guarded set gained after the P1 table into one sandbox tree, clean.
///
/// A sandbox tree is written from the identities a case names, and the guard refuses to enumerate a
/// set with one of its modules missing, so every tree needs them: without the stub the case would
/// fail while resolving `plugin.rs`, which says nothing about the injection it was built for.
fn write_added_module_stubs(root: &Path, layout: &SandboxLayout) {
    // Nothing a case injects, and nothing the guard looks for.
    const CLEAN_ADDED_MODULE: &str = "/// a module of the guarded set, with no reference in it\n";
    for module in &A17_ADDED_MODULES {
        write_sandbox_file(root, layout, module.identity, CLEAN_ADDED_MODULE, &[]);
    }
}

/// Writes one sandbox file at the layout path of `identity`, padding the guarded file names
/// above the size floor.
///
/// A sandbox stub is a placeholder, not the P1 module it is named after, so unless the case is
/// testing the floor itself the stub is padded: otherwise every sandbox case would report the
/// stub as "unexpectedly small" and hide the reference violation it was built for. The floor is
/// decided by identity — the same question the guard asks — so the padding rule cannot drift
/// from the rule it exists for, in either layout.
fn write_sandbox_file(
    root: &Path,
    layout: &SandboxLayout,
    identity: &str,
    contents: &str,
    tiny_files: &[&str],
) {
    let path = root.join(layout.path(identity));
    std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("sandbox directory");
    let padded;
    let contents = if tiny_files.contains(&identity)
        || !carries_size_floor(identity)
        || contents.len() > A17_MIN_SOURCE_LEN
    {
        contents
    } else {
        let mut buffer = contents.to_string();
        while buffer.len() <= A17_MIN_SOURCE_LEN {
            buffer.push_str("// sandbox padding\n");
        }
        padded = buffer;
        &padded
    };
    std::fs::write(&path, contents).expect("sandbox file");
}

// ---------------------------------------------------------------------------
// The P1 physical query keeps its identity
// ---------------------------------------------------------------------------

#[test]
fn engine_query_keeps_its_p1_identity_coordinates_and_billing() {
    let fixture = fixture();
    let target = constructor();
    let query = QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target: QueryTarget::Symbol {
            value: target.clone(),
        },
        physical: PhysicalView {
            snapshot: fixture.snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
        max_items: 0,
        cursor: None,
    };

    let mut budget = Budget::new(limits());
    let report = Engine::new()
        .query(&fixture.snapshot, &query, &mut budget)
        .expect("the physical query keeps working");

    assert_eq!(report.analysis, QueryAnalysis::Performed);
    assert_eq!(report.items.len(), 1);
    let item = &report.items[0];
    assert_eq!(item.consumer, Some(ConsumerKind::Invocation));
    assert_eq!(item.operation, XrefOperation::InvokeSpecial);
    assert_eq!(item.derivation, XrefDerivation::StructuralConsumer);
    assert_eq!(item.certainty, XrefCertainty::Exact);
    assert_eq!(item.resolution, QueryResolution::NotRequested);
    assert_eq!(
        item.target,
        XrefTarget::Symbol {
            value: constructor()
        }
    );
    assert_eq!(item.evidence.constant_pool_index, Some(8));
    assert_eq!(item.evidence.bci, Some(1));
    assert_eq!(item.evidence.opcode, Some(0xb7));
    let span = item.evidence.span.clone().expect("instruction span");
    assert_eq!(span.length, 3);
    assert_eq!(HISTORICAL[usize::try_from(span.start).unwrap()], 0xb7);
    assert_eq!(
        item.source.location,
        Location::Code {
            method: method_id(&fixture.definition, b"<init>", b"()V"),
            bci: 1,
        }
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        report.coverage.dimensions.runtime_resolution.state,
        CoverageState::NotRequested
    );
    // The physical path still reads and bills bytes; the P2 entries did not take that away.
    assert!(budget.usage().class_bytes > 0);
    assert!(!counted_usage_is_zero(&budget.usage()));
}
