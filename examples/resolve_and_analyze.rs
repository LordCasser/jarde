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
//! 3. `Engine::resolve_symbol` on a member symbol, which the member slice (2.3) resolves for
//!    real: the report names the declaration the search selected, the class header it was read
//!    from and the reason for that read; `Engine::declaration_references`, which the
//!    declaration-query slice (2.4) performs for real: it scans the fixture's structure
//!    consumers for candidate use sites of the declared member shape, resolves each candidate's
//!    owner, and reports the candidate whose owner the environment does not provide as
//!    *unresolved* — with its use site and a partial resolution coverage — instead of calling it
//!    excluded; and `Engine::analyze_method`, which the raw-CFG slice (3.3) performs for real as
//!    far as this build goes: it reads the driver method's class definition and body, builds the
//!    raw graph over the decoded instructions, establishes the `jsr`/`ret` call contexts of the
//!    call-context slice (3.4) — none for this fixture, whose 52 dialect inlines its `finally` —
//!    and reports the three phases that completed plus the first phase this build does not
//!    implement,
//! 4. the product planes of one report (`representation`, `quality`, `syntax_status`,
//!    `compile_status`, `semantic_validation`, `verification`, `body`) printed side by
//!    side; the body is `Present` because the analysis really located and read it, the
//!    `Bytecode`/`NotJava`/`NotAttempted`/`NotPerformed` baseline is the P2 delivery, and
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
    CountedBudgetDimension, CoverageState, DeclarationRefQuery, DelegationPolicy, Engine,
    EnvironmentProblemCode, ExecutionReport, HeaderProvider, InspectionMode, JvmBytes, LayoutMode,
    Limits, LoadDomain, LoadRoot, LoaderId, MethodAnalysisRequest, MethodBodyState, ModuleMode,
    MultiReleasePolicy, PhysicalDefinitionId, PhysicalMethodId, PhysicalScope, PhysicalVariant,
    PhysicalView, ProviderId, Quality, ReadReason, ReferenceUse, ResolutionAnalysis,
    ResolutionEnvironment, ResolutionRequest, ResolutionState, ResolvedMemberRef, RuntimeProfile,
    RuntimeUncertainty, RuntimeView, SnapshotId, StageState, SymbolRef, UsageSnapshot,
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
        // The class-name lookup reads one header per attempt, and the member search (2.3)
        // charges one analysis step per class it processes and one dependency depth per layer
        // above the class it starts from. The steps must be funded for any member request; the
        // depth only has to be for a search that climbs (this example's declaration sits in the
        // class the reference names, which is depth 0 and needs no allowance at all).
        class_headers: 1_000,
        dependency_depth: 16,
        analysis_steps: 100_000,
        nested_depth: 8,
        // The one method-analysis request reads one header and one body, and the raw graph it
        // builds charges IR items, edges and worklist steps (3.3). Every other P2 dimension
        // stays at the fail-closed default.
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
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
    let app_roots = vec![LoadRoot::StandaloneClass {
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
        owner: bytes(b"HistoricalControlFlow"),
        name: bytes(b"finallyPath"),
        descriptor: bytes(b"(I)I"),
    };

    // 1. One member resolution request under a legal environment.
    let mut budget = Budget::new(limits());
    let request = ResolutionRequest {
        environment: environment.clone(),
        target: target.clone(),
        use_kind: ReferenceUse::InvokeVirtual,
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
    println!(
        "resolve_symbol.member: reads={:?}",
        report
            .reads
            .iter()
            .map(|read| (read.loader.0.as_str(), read.reason))
            .collect::<Vec<_>>(),
    );
    print_usage("resolve_symbol", &budget.usage());
    assert!(report.environment_problems.is_empty());
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        report
            .resolved
            .as_ref()
            .expect("a resolved member publishes its declaration")
            .member,
        target,
        "the declaration the search selected is the reference's own method"
    );

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

    // 3. One declaration-reference query (2.4): the declaration is the fixture's constructor.
    //    The query scans this snapshot's own structure consumers for candidates that carry the
    //    declaration's name and descriptor — whatever owner they spell — and resolves each
    //    candidate's owner. This fixture's constructor really calls
    //    `java/lang/Object.<init>()V`, so the scan finds one candidate and the resolution cannot
    //    decide it: `java/lang/Object` is not provided, so the candidate stays *unresolved*
    //    with its use site in a diagnostic instead of being reported as excluded.
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
         unresolved_candidates={} has_more={} returned_items={} coverage=({:?}, {:?}) \
         execution={:?} diagnostics={:?}",
        report.analysis,
        report.items.len(),
        report.unsupported_categories.len(),
        report.unresolved_candidates,
        report.has_more,
        report.returned_items,
        report.coverage.artifact_structural.state,
        report.coverage.runtime_resolution.state,
        report.execution,
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>(),
    );
    print_usage("declaration_references", &budget.usage());
    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert!(report.items.is_empty());
    assert_eq!(
        report.unresolved_candidates, 1,
        "the constructor really references `java/lang/Object.<init>()V`, and that owner is not \
         provided, so the candidate is undecided and not excluded"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial,
        "an undecided candidate leaves the resolution plane incomplete"
    );

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
    // The body was located and read by this run, and the canonical CFG was built from the call
    // contexts the phase before it proved: a produced artifact is what makes `quality`
    // `Conservative` rather than `Fallback`. The phases over that artifact — the frames and the
    // stack/local names 4.3 derives from them — run too, and every phase this build declares is
    // implemented, so the pipeline is answered as a complete run.
    //
    // The shape is asserted rather than a count of completed phases: each phase this build
    // gains moves that count, and a number here would be asserting the build's progress instead
    // of what the example is about. What has to hold is the table: every scheduled phase ran and
    // completed, and the run really reached its end.
    assert_eq!(report.body, MethodBodyState::Present);
    let states = report
        .stages
        .iter()
        .map(|stage| stage.state.clone())
        .collect::<Vec<_>>();
    assert!(
        states
            .iter()
            .all(|state| matches!(state, StageState::Completed)),
        "every scheduled phase completed: {states:?}"
    );
    assert!(
        matches!(report.execution, ExecutionReport::Complete { .. }),
        "the pipeline ran to its end: {:?}",
        report.execution
    );
    assert_eq!(
        report.quality,
        Quality::Conservative,
        "a produced canonical CFG is the artifact `quality` classifies"
    );
    assert!(
        report.diagnostics.is_empty(),
        "a run that completed every phase reports no diagnostic: {:?}",
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(report.reads.len(), 1);
    assert_eq!(
        report.reads[0].reason,
        ReadReason::DriverMethodBody,
        "the driver method's own class definition is the header this request read"
    );
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
