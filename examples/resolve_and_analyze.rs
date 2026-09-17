//! Prints the P2 entry points' honest result for one historical class file.
//!
//! The example shows five things:
//!
//! 1. a physical use of the P1 surface (open the snapshot, read the header) on its own
//!    budget, so the P2 request budget below provably stays at zero,
//! 2. a class-name lookup (2.1) organized by the request closure (2.2): the class symbol is
//!    resolved for real through the declared search order — here the caller's only root is
//!    this standalone CLASS file, which declares its own name — so the report names the
//!    selected position, the header read it caused and the demand that caused it,
//! 3. `Engine::resolve_symbol` on a member symbol and `Engine::declaration_references` /
//!    `Engine::analyze_method` on one explicit `ResolutionEnvironment`: the reports say
//!    `NotPerformed` / `Failed { Unsupported }` / `NotRequested` and list the scheduled
//!    method-analysis phases as `NotPerformed`,
//! 4. the product planes of one report (`representation`, `quality`, `syntax_status`,
//!    `compile_status`, `semantic_validation`, `verification`, `body`) printed side by
//!    side; the body stays `NotInspected` because nothing was located or read, and
//!    `quality = Fallback` is printed with it only as "not Conservative", not as a claim
//!    that a fallback recovery happened,
//! 5. a second environment whose caller domain declares a parent that no domain binds:
//!    the report carries `MissingParent` instead of starting a resolver or falling back to
//!    a flat classpath.
//!
//! Usage: `cargo run --example resolve_and_analyze [path/to/class]`; without an argument
//! the historical v52 fixture is used.

use jarde::{
    AnalysisStage, ArtifactInput, Budget, CallerContext, ClassTarget, ConsumerKind, ConsumerSchema,
    CountedBudgetDimension, DeclarationRefQuery, DelegationPolicy, Engine, EnvironmentProblemCode,
    ExecutionReport, HeaderProvider, InspectionMode, JvmBytes, LayoutMode, Limits, LoadDomain,
    LoadRoot, LoaderId, MethodAnalysisRequest, MethodBodyState, ModuleMode, MultiReleasePolicy,
    PhysicalDefinitionId, PhysicalMethodId, PhysicalScope, PhysicalVariant, PhysicalView,
    ProviderId, ReadReason, ReferenceUse, ResolutionEnvironment, ResolutionRequest,
    ResolutionState, ResolvedMemberRef, RuntimeProfile, RuntimeUncertainty, RuntimeView,
    SnapshotId, SymbolRef, TerminationReason, UsageSnapshot,
};
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn limits() -> Limits {
    Limits {
        input_bytes: 16 * 1024 * 1024,
        archive_entries: 1_000,
        entry_bytes: 16 * 1024 * 1024,
        read_bytes: 32 * 1024 * 1024,
        class_bytes: 16 * 1024 * 1024,
        attribute_bytes: 8 * 1024 * 1024,
        code_bytes: 4 * 1024 * 1024,
        result_items: 100_000,
        output_bytes: 32 * 1024 * 1024,
        // The class-name lookup reads one header per attempt; every other P2 dimension stays
        // at the fail-closed default, because this example performs no closure and no IR work.
        class_headers: 1_000,
        nested_depth: 8,
        elapsed_millis: 30_000,
        ..Limits::default()
    }
}

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

fn domain(loader: &LoaderId, parent: Option<LoaderId>, roots: Vec<LoadRoot>) -> LoadDomain {
    LoadDomain {
        loader: loader.clone(),
        parent_loader: parent,
        delegation: DelegationPolicy::ParentFirst,
        roots,
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    }
}

/// Builds one environment whose caller domain is `app` and whose only provider names the
/// caller's roots.
fn build_environment(
    snapshot: SnapshotId,
    app: LoadDomain,
    domains: Vec<LoadDomain>,
) -> ResolutionEnvironment {
    let providers = vec![HeaderProvider {
        id: ProviderId("app-headers".to_string()),
        roots: app.roots.clone(),
    }];
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot,
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: app,
        },
        domains,
        providers,
    }
}

fn print_usage(label: &str, usage: &UsageSnapshot) {
    let counted = CountedBudgetDimension::ALL
        .iter()
        .map(|dimension| format!("{dimension:?}={}", usage.counted_usage(*dimension)))
        .collect::<Vec<_>>()
        .join(" ");
    println!(
        "{label}.usage counted[{counted}] nested_depth={} dependency_depth={} elapsed_millis={}",
        usage.nested_depth, usage.dependency_depth, usage.elapsed_millis
    );
}

fn run(path: PathBuf) -> jarde::Result<()> {
    let engine = Engine::new();

    // Physical context: opening and reading the class is P1 work and pays its own budget.
    // The P2 request budget below is a different `Budget`, so a resolution request that
    // touched one byte would show up as a nonzero counted usage or as `BudgetExceeded`.
    let mut context = Budget::new(limits());
    let snapshot = engine.open(ArtifactInput::Path(path), &mut context)?;
    let source = engine.inspect_header(
        &snapshot,
        ClassTarget::Root,
        &mut context,
        InspectionMode::Forensic,
    )?;
    let definition = PhysicalDefinitionId {
        location: source.source.location.clone(),
        class_bytes: source.source.class_bytes.clone(),
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition.clone(),
        name: bytes(b"finallyPath"),
        descriptor: bytes(b"(I)I"),
    };
    println!(
        "physical: snapshot={} kind={:?} class={} methods={} class_bytes={}",
        snapshot.id().0,
        snapshot.kind(),
        source.inspection.header.this_class.escaped(),
        source.inspection.header.methods.len(),
        source.source.class_bytes.length,
    );

    let app = LoaderId("app".to_string());
    let platform = LoaderId("platform".to_string());
    let snapshot_id = snapshot.id().clone();
    let app_roots = vec![LoadRoot::Snapshot {
        snapshot: snapshot_id.clone(),
    }];
    let environment = build_environment(
        snapshot_id.clone(),
        domain(&app, Some(platform.clone()), app_roots.clone()),
        vec![
            domain(&app, Some(platform.clone()), app_roots.clone()),
            domain(&platform, None, Vec::new()),
        ],
    );

    let caller = CallerContext {
        loader: app.clone(),
        enclosing: Some(method.clone()),
    };
    let target = SymbolRef::Method {
        owner: bytes(b"java/lang/Object"),
        name: bytes(b"<init>"),
        descriptor: bytes(b"()V"),
    };

    // 1. One symbol resolution request under a legal environment.
    let mut budget = Budget::new(limits());
    let request = ResolutionRequest {
        environment: environment.clone(),
        target: target.clone(),
        use_kind: ReferenceUse::InvokeSpecial,
        caller: caller.clone(),
        dispatch: None,
    };
    let report = engine.resolve_symbol(std::slice::from_ref(&snapshot), &request, &mut budget)?;
    println!(
        "resolve_symbol: analysis={:?} state={:?} resolved={:?} candidates={} dispatch={} \
         environment_problems={}",
        report.analysis,
        report.state,
        report.resolved,
        report.candidates.len(),
        report.dispatch.is_some(),
        report.environment_problems.len(),
    );
    println!(
        "resolve_symbol: coverage=({:?}, {:?}, {:?}) execution={:?} diagnostics={:?}",
        report.coverage.artifact_structural.state,
        report.coverage.runtime_resolution.state,
        report.coverage.dynamic_analysis.state,
        report.execution,
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>(),
    );
    print_usage("resolve_symbol", &budget.usage());
    assert!(report.environment_problems.is_empty());
    assert!(report.state.is_none());

    // 2. One class-name lookup under the same environment. The caller's only root is this
    //    standalone CLASS file, which declares its own name, so the lookup selects it through
    //    `this_class` and charges exactly one header read attempt.
    let mut budget = Budget::new(limits());
    let request = ResolutionRequest {
        environment: environment.clone(),
        target: SymbolRef::Class {
            owner: bytes(b"HistoricalControlFlow"),
        },
        use_kind: ReferenceUse::ClassReference,
        caller: caller.clone(),
        dispatch: None,
    };
    let report = engine.resolve_symbol(std::slice::from_ref(&snapshot), &request, &mut budget)?;
    println!(
        "resolve_symbol.class: analysis={:?} state={:?} resolved={:?} candidates={} \
         coverage={:?} execution={:?} diagnostics={:?}",
        report.analysis,
        report.state,
        report.resolved,
        report.candidates.len(),
        report.coverage.runtime_resolution.state,
        report.execution,
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>(),
    );
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        report
            .resolved
            .as_ref()
            .map(|resolved| resolved.loader.clone()),
        Some(app.clone())
    );
    // 2.2: the report names every class header the request read, at most once per
    // `(definition, loader)`, with the demand that caused the read. This lookup reads the
    // definition it selected and nothing else, so the list has one entry.
    assert_eq!(report.reads.len(), 1);
    assert_eq!(report.reads[0].reason, ReadReason::RequestedDefinition);
    assert_eq!(
        report.reads[0].definition,
        report
            .resolved
            .as_ref()
            .expect("a resolved lookup publishes its definition")
            .definition
    );
    println!(
        "resolve_symbol.class: reads={:?}",
        report
            .reads
            .iter()
            .map(|read| (read.loader.0.as_str(), read.reason))
            .collect::<Vec<_>>(),
    );
    assert!(report.environment_problems.is_empty());
    print_usage("resolve_symbol.class", &budget.usage());

    // 3. One declaration-reference query: the declaration is the fixture's constructor.
    let declaration = ResolvedMemberRef {
        loader: app.clone(),
        definition: definition.clone(),
        member: SymbolRef::Method {
            owner: bytes(b"HistoricalControlFlow"),
            name: bytes(b"<init>"),
            descriptor: bytes(b"()V"),
        },
    };
    let mut budget = Budget::new(limits());
    let query = DeclarationRefQuery {
        environment: environment.clone(),
        declaration,
        scope: PhysicalScope::SnapshotAll,
        consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
        max_items: 0,
    };
    let report =
        engine.declaration_references(std::slice::from_ref(&snapshot), &query, &mut budget)?;
    println!(
        "declaration_references: analysis={:?} items={} unsupported_categories={} \
         unresolved_candidates={} has_more={} returned_items={} execution={:?}",
        report.analysis,
        report.items.len(),
        report.unsupported_categories.len(),
        report.unresolved_candidates,
        report.has_more,
        report.returned_items,
        report.execution,
    );
    print_usage("declaration_references", &budget.usage());
    assert!(report.items.is_empty());

    // 4. One method analysis request: `Frame` and `Ssa` are requested out of order and
    //    with a duplicate, so the normalized request and the scheduled phase list differ.
    let mut budget = Budget::new(limits());
    let analysis = MethodAnalysisRequest {
        environment: environment.clone(),
        method: method.clone(),
        stages: vec![AnalysisStage::Ssa, AnalysisStage::Frame, AnalysisStage::Ssa],
    };
    let report = engine.analyze_method(std::slice::from_ref(&snapshot), &analysis, &mut budget)?;
    println!(
        "analyze_method: snapshot={} name={} descriptor={} requested_stages={:?}",
        method.owner.location.snapshot().0,
        String::from_utf8_lossy(&method.name.0),
        String::from_utf8_lossy(&method.descriptor.0),
        report.requested_stages,
    );
    println!(
        "analyze_method.stages: {:?}",
        report
            .stages
            .iter()
            .map(|stage| (stage.stage, &stage.state))
            .collect::<Vec<_>>(),
    );
    println!(
        "analyze_method: planes representation={:?} quality={:?} syntax_status={:?} \
         compile_status={:?} semantic_validation={:?} verification={:?} body={:?} loader={} \
         origin_members={}",
        report.representation,
        report.quality,
        report.syntax_status,
        report.compile_status,
        report.semantic_validation,
        report.verification,
        report.body,
        report.loader.0,
        report.origin.members.len(),
    );
    // No class byte was read, so no body fact is stated, and `quality = Fallback` only
    // means "not Conservative": it is not evidence that a fallback recovery ran.
    assert_eq!(report.body, MethodBodyState::NotInspected);
    assert!(matches!(
        report.execution,
        ExecutionReport::Failed {
            reason: TerminationReason::Unsupported { .. },
            ..
        }
    ));
    println!(
        "analyze_method: coverage=({:?}, {:?}, {:?}) execution={:?} diagnostics={:?}",
        report.coverage.artifact_structural.state,
        report.coverage.runtime_resolution.state,
        report.coverage.dynamic_analysis.state,
        report.execution,
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>(),
    );
    print_usage("analyze_method", &budget.usage());

    // 5. A second environment whose caller domain declares a parent no domain binds. The
    //    report keeps the original symbol and reports the problem instead of resolving.
    let missing = LoaderId("missing-platform".to_string());
    let broken = build_environment(
        snapshot_id,
        domain(&app, Some(missing.clone()), app_roots.clone()),
        vec![domain(&app, Some(missing), app_roots)],
    );
    let mut budget = Budget::new(limits());
    let request = ResolutionRequest {
        environment: broken,
        target,
        use_kind: ReferenceUse::InvokeSpecial,
        caller,
        dispatch: None,
    };
    let report = engine.resolve_symbol(std::slice::from_ref(&snapshot), &request, &mut budget)?;
    println!(
        "missing-parent: analysis={:?} state={:?} target={:?} coverage=({:?}, {:?}, {:?}) \
         execution={:?}",
        report.analysis,
        report.state,
        report.target,
        report.coverage.artifact_structural.state,
        report.coverage.runtime_resolution.state,
        report.coverage.dynamic_analysis.state,
        report.execution,
    );
    for problem in &report.environment_problems {
        println!(
            "missing-parent.problem: code={} subject={:?} message={}",
            problem.code.as_str(),
            problem.subject,
            problem.message,
        );
    }
    assert!(
        report
            .environment_problems
            .iter()
            .any(|problem| problem.code == EnvironmentProblemCode::MissingParent)
    );
    assert!(report.state.is_none());
    print_usage("missing-parent", &budget.usage());
    Ok(())
}

fn main() -> ExitCode {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_default();
    let path = match args.next() {
        Some(path) => PathBuf::from(path),
        None => Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class"),
    };
    if args.next().is_some() {
        eprintln!(
            "usage: {} [standalone.class]",
            PathBuf::from(program).display()
        );
        return ExitCode::from(2);
    }
    match run(path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("resolve_and_analyze: {error}");
            ExitCode::FAILURE
        }
    }
}
