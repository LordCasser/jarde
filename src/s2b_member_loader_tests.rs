//! Static regression for defining-loader member references and rejected cross-loader claims.
//! This is a crate-private module connected by `facade.rs` under cfg(test).

use super::*;
use jarde_java::source_proof::RecoveryHeaders;

const PARENT_OWNED: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-member-loader-r1/classes/parent/owned/ParentOwned.class"
);
const PARENT_FOO: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-member-loader-r1/classes/parent/identity/Foo.class"
);
const CHILD_FOO: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-member-loader-r1/classes/child/identity/Foo.class"
);
const OBJECT: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-bound-type-identities-r1/platform/java/lang/Object.class"
);
const STRING: &[u8] = include_bytes!(
    "../crates/jarde-java/tests/fixtures/s2b-bound-type-identities-r1/platform/java/lang/String.class"
);

fn test_limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 100,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 32,
        method_bodies: 32,
        ir_items: 1 << 16,
        ir_edges: 1 << 16,
        analysis_steps: 1 << 16,
        normalization_clones: 1 << 16,
        nested_depth: 8,
        dependency_depth: 16,
        elapsed_millis: u64::MAX,
    }
}

fn snapshot(bytes: &[u8]) -> ArtifactSnapshot {
    ArtifactSnapshot::open(
        ArtifactInput::bytes(bytes.to_vec()),
        &mut Budget::new(test_limits()),
    )
    .expect("frozen classfile opens")
}

fn definition(snapshot: &ArtifactSnapshot, bytes: &[u8]) -> PhysicalDefinitionId {
    PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: jarde_reader::model::Digest(blake3::hash(bytes).to_hex().to_string()),
            length: u64::try_from(bytes.len()).expect("fixture byte length fits u64"),
        },
        variant: PhysicalVariant::Base,
    }
}

fn domain(
    loader: &str,
    parent: Option<&str>,
    delegation: DelegationPolicy,
    roots: Vec<LoadRoot>,
) -> LoadDomain {
    LoadDomain {
        loader: LoaderId(loader.to_owned()),
        parent_loader: parent.map(|name| LoaderId(name.to_owned())),
        delegation,
        roots,
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    }
}

fn root(snapshot: &ArtifactSnapshot) -> LoadRoot {
    LoadRoot::StandaloneClass {
        snapshot: snapshot.id().clone(),
    }
}

fn parent_probe(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    ir: &jarde_jvm::method_ir::MethodIr,
) -> (
    DependencyProofResult<DependencyResult<ResolvedMemberRef>>,
    Vec<ArtifactDependency>,
    ExecutionReport,
    UsageSnapshot,
) {
    let mut limits = test_limits();
    limits.method_bodies = 0;
    let mut budget = Budget::new(limits);
    let mut execution = ExecutionReport::Complete {
        usage: budget.usage(),
    };
    let mut inputs = Vec::new();
    let mut cache = std::collections::BTreeMap::new();
    let mut reads = RecoveryHeaderReads {
        content,
        environment: &request.environment,
        enclosing: Some(&request.method),
        ir: Some(ir),
        cache: &mut cache,
        execution: &mut execution,
        inputs: &mut inputs,
    };
    let symbol = jarde_reader::model::SymbolRef::Method {
        owner: JvmBytes(b"identity/Foo".to_vec()),
        name: JvmBytes(b"label".to_vec()),
        descriptor: JvmBytes(b"()Ljava/lang/String;".to_vec()),
    };
    let result = reads.resolve_member(&symbol, ReferenceUse::InvokeStatic, &mut budget);
    let usage = budget.usage();
    (result, inputs, execution, usage)
}

#[test]
fn parent_method_ir_and_member_resolution_keep_the_parent_definition_loader() {
    let owned = snapshot(PARENT_OWNED);
    let parent_foo = snapshot(PARENT_FOO);
    let child_foo = snapshot(CHILD_FOO);
    let object = snapshot(OBJECT);
    let string = snapshot(STRING);
    let content = vec![
        owned.clone(),
        parent_foo.clone(),
        child_foo.clone(),
        object.clone(),
        string.clone(),
    ];

    let platform = domain(
        "platformLoader",
        None,
        DelegationPolicy::ParentFirst,
        vec![root(&object), root(&string)],
    );
    let parent = domain(
        "parentLoader",
        Some("platformLoader"),
        DelegationPolicy::ParentFirst,
        vec![root(&owned), root(&parent_foo)],
    );
    let child = domain(
        "childLoader",
        Some("parentLoader"),
        DelegationPolicy::ChildFirst,
        vec![root(&child_foo)],
    );
    let child_environment = ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: owned.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: jarde_reader::view::MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: child.clone(),
        },
        domains: vec![child.clone(), parent.clone(), platform.clone()],
        providers: Vec::new(),
    };
    let parent_owned_definition = definition(&owned, PARENT_OWNED);
    let method = PhysicalMethodId {
        owner: parent_owned_definition.clone(),
        name: JvmBytes(b"call".to_vec()),
        descriptor: JvmBytes(b"()Ljava/lang/String;".to_vec()),
    };

    // A child-loader request cannot claim the parent-owned method body. Keep the engine's
    // existing binding contract visible instead of manufacturing an IR under the wrong loader.
    let child_request = jarde_jvm::ir::MethodAnalysisRequest {
        environment: child_environment.clone(),
        method: method.clone(),
        stages: AnalysisStage::ALL.to_vec(),
    };
    let mut child_budget = Budget::new(test_limits());
    let child_analysis =
        jarde_jvm::engine::analyze_method_ir(&content, &child_request, &mut child_budget)
            .expect("the refusal is reported in the analysis report");
    assert!(
        child_analysis.ir().declaration().is_none(),
        "unbound parent body must not publish MethodIr"
    );
    assert!(matches!(
        &child_analysis.report().execution,
        ExecutionReport::Failed { reason: TerminationReason::Error { code }, .. }
            if code == "resolution_definition_unbound"
    ));
    assert!(
        child_analysis
            .report()
            .diagnostics
            .iter()
            .any(|d| d.code == "resolution_definition_unbound")
    );
    println!(
        "child runtime claim: execution={:?}, ir_declaration_present={}",
        child_analysis.report().execution,
        child_analysis.ir().declaration().is_some()
    );

    // Same complete domain graph; the runtime method request now names its actual defining loader.
    let mut parent_environment = child_environment.clone();
    parent_environment.runtime.load_domain = parent.clone();
    assert_eq!(parent_environment.domains, child_environment.domains);
    let parent_request = jarde_jvm::ir::MethodAnalysisRequest {
        environment: parent_environment.clone(),
        method,
        stages: AnalysisStage::ALL.to_vec(),
    };
    let mut parent_budget = Budget::new(test_limits());
    let analysis =
        jarde_jvm::engine::analyze_method_ir(&content, &parent_request, &mut parent_budget)
            .expect("parent-domain method analysis completes");
    let ir = analysis.ir();
    assert_eq!(
        ir.declaration()
            .expect("parent-domain read publishes declaration")
            .identity()
            .owner,
        parent_owned_definition,
    );
    assert!(ir.constant_pool().iter().any(|entry| matches!(
        &entry.kind,
        CpEntryKind::MethodRef { owner, name, descriptor, .. }
            if owner.0 == b"identity/Foo" && name.0 == b"label" && descriptor.0 == b"()Ljava/lang/String;"
    )), "ParentOwned's original constant pool contains the exact symbolic method reference");

    // The proof itself is header-only: method_bodies=0 must still resolve the physical parent Foo.
    let (result, inputs, execution, usage) = parent_probe(&content, &parent_request, ir);
    let binding = match result.expect("parent member-resolution proof returns a decision") {
        DependencyResult::Found(binding) => binding,
        DependencyResult::Missing => panic!("parent Foo.label was not found"),
        DependencyResult::Ambiguous => panic!("parent Foo.label became ambiguous"),
        DependencyResult::Unknown => panic!("parent Foo.label did not resolve conclusively"),
    };
    let parent_loader = parent.loader.clone();
    let expected_parent_foo = definition(&parent_foo, PARENT_FOO);
    let unexpected_child_foo = definition(&child_foo, CHILD_FOO);
    assert_eq!(binding.loader, parent_loader);
    assert_eq!(binding.definition, expected_parent_foo);
    assert!(
        inputs
            .iter()
            .any(|input| input.definition == expected_parent_foo && input.loader == parent_loader)
    );
    assert!(
        !inputs
            .iter()
            .any(|input| input.definition == unexpected_child_foo)
    );
    assert_eq!(usage.method_bodies, 0);
    assert!(matches!(&execution, ExecutionReport::Complete { usage } if usage.method_bodies == 0));
    println!(
        "parent method CP resolution: binding={binding:?}, dependencies={inputs:?}, execution={execution:?}, resolver_budget_usage={usage:?}"
    );

    // Independent name resolution from childLoader proves the competing child Foo exists. It has
    // no enclosing parent-owned method, so this does not claim that the parent's CP resolves there.
    let symbol = jarde_reader::model::SymbolRef::Method {
        owner: JvmBytes(b"identity/Foo".to_vec()),
        name: JvmBytes(b"label".to_vec()),
        descriptor: JvmBytes(b"()Ljava/lang/String;".to_vec()),
    };
    let child_loader = child.loader.clone();
    let mut lookup_limits = test_limits();
    lookup_limits.method_bodies = 0;
    let mut lookup_budget = Budget::new(lookup_limits);
    let report = jarde_jvm::resolve_symbol(
        &content,
        &ResolutionRequest {
            environment: child_environment,
            target: symbol,
            use_kind: ReferenceUse::InvokeStatic,
            caller: jarde_jvm::environment::CallerContext {
                loader: child_loader.clone(),
                enclosing: None,
            },
            dispatch: None,
        },
        &mut lookup_budget,
    )
    .expect("independent child symbol lookup reports a decision");
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    let child_binding = report.resolved.expect("child lookup resolves its root Foo");
    assert_eq!(child_binding.loader, child_loader);
    assert_eq!(child_binding.definition, unexpected_child_foo);
    assert!(
        report
            .reads
            .iter()
            .any(|read| read.definition == unexpected_child_foo)
    );
    assert!(
        matches!(&report.execution, ExecutionReport::Complete { usage } if usage.method_bodies == 0)
    );
    assert_eq!(lookup_budget.usage().method_bodies, 0);
    println!(
        "independent child symbol lookup: binding={child_binding:?}, reads={:?}, execution={:?}, resolver_budget_usage={:?}",
        report.reads,
        report.execution,
        lookup_budget.usage()
    );
}
