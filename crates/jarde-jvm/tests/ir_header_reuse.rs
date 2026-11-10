use jarde_jvm::engine::{
    analyze_method_ir, resolve_symbol, resolve_symbol_from_ir, resolve_symbol_from_prepared,
};
use jarde_jvm::environment::{CallerContext, ResolutionEnvironment};
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_jvm::resolver::{ReferenceUse, ResolutionAnalysis, ResolutionRequest, ResolutionState};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, BudgetDimension, CancellationToken, Limits};
use jarde_reader::model::{
    ClassBytesId, Digest, ExecutionReport, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
    PhysicalMethodId, PhysicalVariant, SymbolRef, TerminationReason,
};
use jarde_reader::prepared::PreparedClass;
use jarde_reader::view::{
    DelegationPolicy, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};
use std::slice;

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
        class_headers: 16,
        method_bodies: 16,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

/// The shared reader builder emits `Test.method()V`; retarget only its first UTF8 owner entry.
fn class_p_c(code: &[u8]) -> Vec<u8> {
    let mut bytes = jarde_reader::classfile::test_class::single_method(52, 1, 0, code);
    assert_eq!(&bytes[10..13], &[1, 0, 4]);
    assert_eq!(&bytes[13..17], b"Test");
    bytes[11..13].copy_from_slice(&3_u16.to_be_bytes());
    bytes.splice(13..17, b"p/C".iter().copied());
    bytes
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    ArtifactSnapshot::open(
        ArtifactInput::bytes(bytes.to_vec()),
        &mut Budget::new(limits()),
    )
    .expect("the generated standalone class opens")
}

fn definition(snapshot: &ArtifactSnapshot, bytes: &[u8]) -> PhysicalDefinitionId {
    PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(bytes).to_hex().to_string()),
            length: bytes.len() as u64,
        },
        variant: PhysicalVariant::Base,
    }
}

fn prepared_definition(read: &jarde_reader::prepared::PreparedClassRead) -> PhysicalDefinitionId {
    PhysicalDefinitionId {
        location: read.location.clone(),
        class_bytes: read.class_bytes.clone(),
        variant: PhysicalVariant::Base,
    }
}

fn domain(loader: &str, parent: Option<&str>, roots: Vec<LoadRoot>) -> LoadDomain {
    LoadDomain {
        loader: LoaderId(loader.to_owned()),
        parent_loader: parent.map(|name| LoaderId(name.to_owned())),
        delegation: DelegationPolicy::ParentFirst,
        roots,
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    }
}

fn environment(
    snapshot: &ArtifactSnapshot,
    caller: LoadDomain,
    domains: Vec<LoadDomain>,
) -> ResolutionEnvironment {
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: jarde_reader::view::LayoutMode::Generic,
            },
            load_domain: caller,
        },
        domains,
        providers: Vec::new(),
    }
}

fn own_environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
    let app = domain(
        "app",
        None,
        vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
    );
    environment(snapshot, app.clone(), vec![app])
}

fn method_request(
    snapshot: &ArtifactSnapshot,
    bytes: &[u8],
    env: ResolutionEnvironment,
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment: env,
        method: PhysicalMethodId {
            owner: definition(snapshot, bytes),
            name: JvmBytes(b"method".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        },
        stages: vec![AnalysisStage::RawFacts],
    }
}

fn analyze(
    snapshot: &ArtifactSnapshot,
    bytes: &[u8],
    env: ResolutionEnvironment,
) -> jarde_jvm::method_ir::MethodIrAnalysis {
    let request = method_request(snapshot, bytes, env);
    let result = analyze_method_ir(
        slice::from_ref(snapshot),
        &request,
        &mut Budget::new(limits()),
    )
    .expect("the real RawFacts analysis yields the class header and body");
    assert_eq!(
        result.ir().declaration().unwrap().identity(),
        &request.method
    );
    result
}

fn class_request(env: ResolutionEnvironment, loader: &str, owner: &[u8]) -> ResolutionRequest {
    ResolutionRequest {
        environment: env,
        target: SymbolRef::Class {
            owner: JvmBytes(owner.to_vec()),
        },
        use_kind: ReferenceUse::ClassReference,
        caller: CallerContext {
            loader: LoaderId(loader.to_owned()),
            enclosing: None,
        },
        dispatch: None,
    }
}

fn resolved(
    report: &jarde_jvm::resolver::ResolutionReport,
) -> &jarde_jvm::resolver::ResolvedMemberRef {
    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    report.resolved.as_ref().expect("resolved class reference")
}

#[test]
fn reused_standalone_header_answers_its_name_and_absences_without_new_reads() {
    let bytes = class_p_c(&[0xb1]);
    let snapshot = open(&bytes);
    let content = [snapshot.clone()];
    let env = own_environment(&snapshot);
    let analysis = analyze(&snapshot, &bytes, env.clone());
    let ir = analysis.ir();
    let known_definition = ir.declaration().unwrap().identity().owner.clone();

    for owner in [b"p/C".as_slice(), b"p/MissingA", b"p/MissingB"] {
        let request = class_request(env.clone(), "app", owner);
        let mut budget = Budget::new(limits());
        let before = budget.usage();
        let report = resolve_symbol_from_ir(&content, &request, ir, &mut budget).unwrap();
        let after = budget.usage();
        assert_eq!(after.class_headers, before.class_headers);
        assert_eq!(after.method_bodies, before.method_bodies);
        assert!(report.reads.is_empty(), "reused facts are not new reads");
        assert_eq!(
            report.environment_identity,
            analysis.report().environment_identity
        );
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        if owner == b"p/C" {
            let selected = resolved(&report);
            assert_eq!(selected.loader, LoaderId("app".into()));
            assert_eq!(selected.definition, known_definition);
            assert_eq!(selected.member, request.target);
        } else {
            assert_eq!(report.analysis, ResolutionAnalysis::Performed);
            assert_eq!(report.state, Some(ResolutionState::Missing));
            assert!(report.resolved.is_none());
            assert!(report.unresolved_dependencies.is_empty());
        }
    }

    // The direct path is the positive control: it must read the same physical header again.
    let request = class_request(env, "app", b"p/C");
    let mut direct_budget = Budget::new(limits());
    let direct = resolve_symbol(&content, &request, &mut direct_budget).unwrap();
    assert_eq!(resolved(&direct).definition, known_definition);
    assert_eq!(resolved(&direct).loader, LoaderId("app".into()));
    assert_eq!(direct_budget.usage().class_headers, 1);
    assert_eq!(direct_budget.usage().method_bodies, 0);
    assert_eq!(direct.reads.len(), 1);
    assert_eq!(direct.reads[0].definition, known_definition);
    assert_eq!(direct.reads[0].loader, LoaderId("app".into()));
}

#[test]
fn an_earlier_root_with_a_different_definition_shadows_ir_facts() {
    let known_bytes = class_p_c(&[0xb1]);
    let shadow_bytes = class_p_c(&[0x00, 0xb1]);
    let known = open(&known_bytes);
    let shadow = open(&shadow_bytes);
    let content = [shadow.clone(), known.clone()];
    let known_ir = analyze(&known, &known_bytes, own_environment(&known));
    let known_definition = known_ir
        .ir()
        .declaration()
        .unwrap()
        .identity()
        .owner
        .clone();
    let shadow_definition = definition(&shadow, &shadow_bytes);
    assert_ne!(known_definition, shadow_definition);

    let app = domain(
        "app",
        None,
        vec![
            LoadRoot::StandaloneClass {
                snapshot: shadow.id().clone(),
            },
            LoadRoot::StandaloneClass {
                snapshot: known.id().clone(),
            },
        ],
    );
    let env = environment(&shadow, app.clone(), vec![app]);
    let request = class_request(env, "app", b"p/C");
    let mut budget = Budget::new(limits());
    let report = resolve_symbol_from_ir(&content, &request, known_ir.ir(), &mut budget).unwrap();

    let selected = resolved(&report);
    assert_eq!(selected.loader, LoaderId("app".into()));
    assert_eq!(selected.definition, shadow_definition);
    assert_ne!(selected.definition, known_definition);
    assert_eq!(budget.usage().class_headers, 1);
    assert_eq!(budget.usage().method_bodies, 0);
    assert_eq!(report.reads.len(), 1);
    assert_eq!(report.reads[0].definition, shadow_definition);
    assert_eq!(
        report.reads[0].reason,
        jarde_jvm::resolver::ReadReason::RequestedDefinition
    );
}

#[test]
fn a_shared_physical_root_keeps_the_loader_selected_by_parent_first_search() {
    let bytes = class_p_c(&[0xb1]);
    let snapshot = open(&bytes);
    let content = [snapshot.clone()];
    let ir_environment = own_environment(&snapshot);
    let analysis = analyze(&snapshot, &bytes, ir_environment);
    let physical_definition = analysis
        .ir()
        .declaration()
        .unwrap()
        .identity()
        .owner
        .clone();

    let parent = domain(
        "parent",
        None,
        vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
    );
    let app = domain(
        "app",
        Some("parent"),
        vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
    );
    let env = environment(&snapshot, app.clone(), vec![app, parent]);
    let request = class_request(env.clone(), "app", b"p/C");

    let mut reused_budget = Budget::new(limits());
    let reused =
        resolve_symbol_from_ir(&content, &request, analysis.ir(), &mut reused_budget).unwrap();
    let selected = resolved(&reused);
    assert_eq!(selected.loader, LoaderId("parent".into()));
    assert_eq!(selected.definition, physical_definition);
    assert!(
        reused.reads.is_empty(),
        "the same physical facts are not a second read"
    );
    assert_eq!(reused_budget.usage().class_headers, 0);
    assert_eq!(reused_budget.usage().method_bodies, 0);

    let mut direct_budget = Budget::new(limits());
    let direct = resolve_symbol(&content, &request, &mut direct_budget).unwrap();
    assert_eq!(resolved(&direct).loader, LoaderId("parent".into()));
    assert_eq!(resolved(&direct).definition, physical_definition);
    assert_eq!(direct_budget.usage().class_headers, 1);
    assert_eq!(direct.reads.len(), 1);
    assert_eq!(direct.reads[0].loader, LoaderId("parent".into()));
}

#[test]
fn invalid_cancelled_and_required_header_budget_requests_do_not_use_ir_as_a_verdict() {
    let bytes = class_p_c(&[0xb1]);
    let snapshot = open(&bytes);
    let content = [snapshot.clone()];
    let env = own_environment(&snapshot);
    let analysis = analyze(&snapshot, &bytes, env.clone());
    let ir = analysis.ir();

    let mut invalid = class_request(env.clone(), "not-app", b"p/C");
    let mut invalid_budget = Budget::new(limits());
    let invalid_report =
        resolve_symbol_from_ir(&content, &invalid, ir, &mut invalid_budget).unwrap();
    assert_eq!(invalid_report.analysis, ResolutionAnalysis::NotPerformed);
    assert!(invalid_report.state.is_none());
    assert!(!invalid_report.environment_problems.is_empty());
    assert!(invalid_report.reads.is_empty());
    assert_eq!(invalid_budget.usage().class_headers, 0);
    assert_eq!(invalid_budget.usage().method_bodies, 0);
    invalid.caller.loader = LoaderId("app".into());

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled_budget = Budget::with_cancellation_token(limits(), token);
    let cancelled = resolve_symbol_from_ir(&content, &invalid, ir, &mut cancelled_budget).unwrap();
    assert_eq!(cancelled.analysis, ResolutionAnalysis::Performed);
    assert!(cancelled.state.is_none());
    assert!(matches!(
        cancelled.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert!(cancelled.reads.is_empty());
    assert_eq!(cancelled_budget.usage().class_headers, 0);
    assert_eq!(cancelled_budget.usage().method_bodies, 0);

    // This search begins at a different physical root than the IR read. It must pay for the
    // selected root's header; the exact-physical-definition shortcut cannot answer it.
    let shadow_bytes = class_p_c(&[0x00, 0xb1]);
    let shadow = open(&shadow_bytes);
    let content = [shadow.clone(), snapshot.clone()];
    let app = domain(
        "app",
        None,
        vec![
            LoadRoot::StandaloneClass {
                snapshot: shadow.id().clone(),
            },
            LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone(),
            },
        ],
    );
    let shadow_env = environment(&shadow, app.clone(), vec![app]);
    let shadow_request = class_request(shadow_env, "app", b"p/C");
    let mut tight_limits = limits();
    tight_limits.class_headers = 0;
    let mut tight_budget = Budget::new(tight_limits);
    let tight = resolve_symbol_from_ir(&content, &shadow_request, ir, &mut tight_budget).unwrap();
    assert_eq!(tight.analysis, ResolutionAnalysis::Performed);
    assert_eq!(tight.state, Some(ResolutionState::BudgetExceeded));
    assert!(matches!(
        tight.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ClassHeaders
            },
            ..
        }
    ));
    assert!(tight.reads.is_empty());
    assert_eq!(tight_budget.usage().class_headers, 0);
    assert_eq!(tight_budget.usage().method_bodies, 0);
}

#[test]
fn prepared_header_reuse_keeps_search_binding_without_a_second_header_read() {
    let bytes = class_p_c(&[0xb1]);
    let snapshot = open(&bytes);
    let content = [snapshot.clone()];
    let env = own_environment(&snapshot);
    let mut prepare_budget = Budget::new(limits());
    let read = snapshot.prepared_root_class(&mut prepare_budget).unwrap();
    let prepared = PreparedClass::prepare(&read, &mut prepare_budget).unwrap();
    let definition = prepared_definition(&read);
    let request = class_request(env, "app", b"p/C");

    let mut reused_budget = Budget::new(limits());
    let reused = resolve_symbol_from_prepared(
        &content,
        &request,
        &prepared,
        &definition,
        &mut reused_budget,
    )
    .unwrap();
    let selected = resolved(&reused);
    assert_eq!(selected.loader, LoaderId("app".into()));
    assert_eq!(selected.definition, definition);
    assert!(reused.reads.is_empty(), "prepared facts are not a new read");
    assert_eq!(reused_budget.usage().class_headers, 0);
    assert_eq!(reused_budget.usage().method_bodies, 0);

    let mut direct_budget = Budget::new(limits());
    let direct = resolve_symbol(&content, &request, &mut direct_budget).unwrap();
    assert_eq!(resolved(&direct).loader, LoaderId("app".into()));
    assert_eq!(resolved(&direct).definition, definition);
    assert_eq!(direct_budget.usage().class_headers, 1);
    assert_eq!(direct.reads.len(), 1);
}

#[test]
fn prepared_physical_facts_do_not_override_the_selected_loader() {
    let bytes = class_p_c(&[0xb1]);
    let snapshot = open(&bytes);
    let content = [snapshot.clone()];
    let mut prepare_budget = Budget::new(limits());
    let read = snapshot.prepared_root_class(&mut prepare_budget).unwrap();
    let prepared = PreparedClass::prepare(&read, &mut prepare_budget).unwrap();
    let definition = prepared_definition(&read);

    let parent = domain(
        "parent",
        None,
        vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
    );
    let app = domain(
        "app",
        Some("parent"),
        vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
    );
    let env = environment(&snapshot, app.clone(), vec![app, parent]);
    let request = class_request(env, "app", b"p/C");
    let mut budget = Budget::new(limits());
    let report =
        resolve_symbol_from_prepared(&content, &request, &prepared, &definition, &mut budget)
            .unwrap();

    let selected = resolved(&report);
    assert_eq!(selected.loader, LoaderId("parent".into()));
    assert_eq!(selected.definition, definition);
    assert!(report.reads.is_empty());
    assert_eq!(budget.usage().class_headers, 0);
    assert_eq!(budget.usage().method_bodies, 0);
}

#[test]
fn prepared_header_rejects_wrong_definition_and_keeps_cancel_and_budget_stops() {
    let bytes = class_p_c(&[0xb1]);
    let snapshot = open(&bytes);
    let mut prepare_budget = Budget::new(limits());
    let read = snapshot.prepared_root_class(&mut prepare_budget).unwrap();
    let prepared = PreparedClass::prepare(&read, &mut prepare_budget).unwrap();
    let definition = prepared_definition(&read);
    let env = own_environment(&snapshot);
    let request = class_request(env, "app", b"p/C");

    let mut wrong = definition.clone();
    wrong.class_bytes.length += 1;
    let error = resolve_symbol_from_prepared(
        std::slice::from_ref(&snapshot),
        &request,
        &prepared,
        &wrong,
        &mut Budget::new(limits()),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        jarde_reader::error::Error::InvalidInput { code, .. }
            if code == "class_definition_mismatch"
    ));

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled_budget = Budget::with_cancellation_token(limits(), token);
    let cancelled = resolve_symbol_from_prepared(
        std::slice::from_ref(&snapshot),
        &request,
        &prepared,
        &definition,
        &mut cancelled_budget,
    )
    .unwrap();
    assert!(matches!(
        cancelled.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert!(cancelled.reads.is_empty());
    assert_eq!(cancelled_budget.usage().class_headers, 0);
    assert_eq!(cancelled_budget.usage().method_bodies, 0);

    let shadow_bytes = class_p_c(&[0x00, 0xb1]);
    let shadow = open(&shadow_bytes);
    let content = [shadow.clone(), snapshot.clone()];
    let app = domain(
        "app",
        None,
        vec![
            LoadRoot::StandaloneClass {
                snapshot: shadow.id().clone(),
            },
            LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone(),
            },
        ],
    );
    let shadow_env = environment(&shadow, app.clone(), vec![app]);
    let shadow_request = class_request(shadow_env, "app", b"p/C");
    let mut tight_limits = limits();
    tight_limits.class_headers = 0;
    let mut tight_budget = Budget::new(tight_limits);
    let stopped = resolve_symbol_from_prepared(
        &content,
        &shadow_request,
        &prepared,
        &definition,
        &mut tight_budget,
    )
    .unwrap();
    assert_eq!(stopped.state, Some(ResolutionState::BudgetExceeded));
    assert!(matches!(
        stopped.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ClassHeaders
            },
            ..
        }
    ));
    assert!(stopped.reads.is_empty());
    assert_eq!(tight_budget.usage().class_headers, 0);
    assert_eq!(tight_budget.usage().method_bodies, 0);
}
