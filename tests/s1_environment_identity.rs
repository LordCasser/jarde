//! S1 A1.4: complete environment declarations bind resolution results.
use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const STORE: u16 = 0;

fn limits() -> Limits {
    let mut limits = task_budget(&[]).unwrap().limits().clone();
    limits.elapsed_millis = u64::MAX;
    limits
}

fn class_bytes(name: &[u8], major: u16) -> Vec<u8> {
    let object = b"java/lang/Object";
    let mut pool = Vec::new();
    for (i, value) in [name, object].into_iter().enumerate() {
        pool.push(1_u8);
        pool.extend_from_slice(&u16::try_from(value.len()).unwrap().to_be_bytes());
        pool.extend_from_slice(value);
        pool.push(7_u8);
        pool.extend_from_slice(&u16::try_from(1 + i * 2).unwrap().to_be_bytes());
    }
    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    bytes.extend_from_slice(&0_u16.to_be_bytes());
    bytes.extend_from_slice(&major.to_be_bytes());
    bytes.extend_from_slice(&5_u16.to_be_bytes());
    bytes.extend_from_slice(&pool);
    bytes.extend_from_slice(&0x0021_u16.to_be_bytes());
    bytes.extend_from_slice(&2_u16.to_be_bytes());
    bytes.extend_from_slice(&4_u16.to_be_bytes());
    for _ in 0..4 {
        bytes.extend_from_slice(&0_u16.to_be_bytes());
    }
    bytes
}

fn zip(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(data).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        archive.finish().unwrap();
    }
    output.into_inner()
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    ArtifactSnapshot::open(ArtifactInput::bytes(bytes), &mut Budget::new(limits())).unwrap()
}

fn root(snapshot: &ArtifactSnapshot, prefix: &[u8]) -> LoadRoot {
    LoadRoot::Container {
        origin: ContainerOrigin {
            snapshot: snapshot.id().clone(),
            root_container: ContainerId("root".into()),
            steps: Vec::new(),
        },
        prefix: ArchiveNameBytes(prefix.to_vec()),
    }
}

fn domain(
    name: &str,
    parent: Option<&str>,
    delegation: DelegationPolicy,
    roots: Vec<LoadRoot>,
) -> LoadDomain {
    LoadDomain {
        loader: LoaderId(name.into()),
        parent_loader: parent.map(|name| LoaderId(name.into())),
        delegation,
        roots,
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    }
}

fn environment(
    runtime_snapshot: &ArtifactSnapshot,
    caller: LoadDomain,
    domains: Vec<LoadDomain>,
    providers: Vec<HeaderProvider>,
) -> ResolutionEnvironment {
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: runtime_snapshot.id().clone(),
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

fn resolve(content: &[ArtifactSnapshot], env: ResolutionEnvironment) -> ResolutionReport {
    Engine::new()
        .resolve_symbol(
            content,
            &ResolutionRequest {
                environment: env,
                target: SymbolRef::Class {
                    owner: JvmBytes(b"p/S".to_vec()),
                },
                use_kind: ReferenceUse::ClassReference,
                caller: CallerContext {
                    loader: LoaderId("app".into()),
                    enclosing: None,
                },
                dispatch: None,
            },
            &mut Budget::new(limits()),
        )
        .unwrap()
}

fn only_selection(report: &ResolutionReport) -> (&LoaderId, &PhysicalDefinitionId) {
    let selected = report
        .resolved
        .as_ref()
        .expect("valid environment selects p/S");
    (&selected.loader, &selected.definition)
}

#[test]
fn noncaller_roots_order_is_identity_and_changes_the_selected_definition() {
    let first = open(zip(&[(b"p/S.class", &class_bytes(b"p/S", 52))]));
    let second = open(zip(&[(b"p/S.class", &class_bytes(b"p/S", 51))]));
    let caller = domain(
        "app",
        Some("platform"),
        DelegationPolicy::ParentFirst,
        vec![],
    );
    let platform_a = domain(
        "platform",
        None,
        DelegationPolicy::ParentFirst,
        vec![root(&first, b""), root(&second, b"")],
    );
    let platform_b = domain(
        "platform",
        None,
        DelegationPolicy::ParentFirst,
        vec![root(&second, b""), root(&first, b"")],
    );
    let left = resolve(
        &[first.clone(), second.clone()],
        environment(
            &first,
            caller.clone(),
            vec![caller.clone(), platform_a],
            vec![],
        ),
    );
    let right = resolve(
        &[first.clone(), second.clone()],
        environment(&first, caller.clone(), vec![caller, platform_b], vec![]),
    );
    assert_ne!(left.environment_identity, right.environment_identity);
    assert_eq!(only_selection(&left).0, &LoaderId("platform".into()));
    assert_eq!(only_selection(&right).0, &LoaderId("platform".into()));
    assert_ne!(only_selection(&left).1, only_selection(&right).1);
}

#[test]
fn noncaller_prefix_change_is_identity_and_changes_the_selected_definition() {
    let one_bytes = class_bytes(b"p/S", 52);
    let two_bytes = class_bytes(b"p/S", 51);
    let archive = open(zip(&[
        (b"one/p/S.class", &one_bytes),
        (b"two/p/S.class", &two_bytes),
    ]));
    let caller = domain(
        "app",
        Some("platform"),
        DelegationPolicy::ParentFirst,
        vec![],
    );
    let make = |prefix: &[u8]| {
        let platform = domain(
            "platform",
            None,
            DelegationPolicy::ParentFirst,
            vec![root(&archive, prefix)],
        );
        environment(
            &archive,
            caller.clone(),
            vec![caller.clone(), platform],
            vec![],
        )
    };
    let one = resolve(std::slice::from_ref(&archive), make(b"one/"));
    let two = resolve(std::slice::from_ref(&archive), make(b"two/"));
    assert_ne!(one.environment_identity, two.environment_identity);
    assert_ne!(only_selection(&one).1, only_selection(&two).1);
}

#[test]
fn noncaller_delegation_and_parent_changes_are_identity_and_change_selection() {
    let boot = open(zip(&[(b"p/S.class", &class_bytes(b"p/S", 50))]));
    let platform = open(zip(&[(b"p/S.class", &class_bytes(b"p/S", 51))]));
    let app = open(zip(&[(b"p/S.class", &class_bytes(b"p/S", 52))]));
    let app_domain = domain(
        "app",
        Some("platform"),
        DelegationPolicy::ParentFirst,
        vec![root(&app, b"")],
    );
    let platform_parent_first = domain(
        "platform",
        Some("boot"),
        DelegationPolicy::ParentFirst,
        vec![root(&platform, b"")],
    );
    let platform_child_first = domain(
        "platform",
        Some("boot"),
        DelegationPolicy::ChildFirst,
        vec![root(&platform, b"")],
    );
    let content = [app.clone(), platform.clone(), boot.clone()];
    let parent_first = resolve(
        &content,
        environment(
            &app,
            app_domain.clone(),
            vec![
                app_domain.clone(),
                platform_parent_first,
                domain(
                    "boot",
                    None,
                    DelegationPolicy::ParentFirst,
                    vec![root(&boot, b"")],
                ),
            ],
            vec![],
        ),
    );
    let child_first = resolve(
        &content,
        environment(
            &app,
            app_domain.clone(),
            vec![
                app_domain.clone(),
                platform_child_first,
                domain(
                    "boot",
                    None,
                    DelegationPolicy::ParentFirst,
                    vec![root(&boot, b"")],
                ),
            ],
            vec![],
        ),
    );
    assert_ne!(
        parent_first.environment_identity,
        child_first.environment_identity
    );
    assert_eq!(only_selection(&parent_first).0, &LoaderId("boot".into()));
    assert_eq!(only_selection(&child_first).0, &LoaderId("platform".into()));

    let without_boot_parent = domain(
        "platform",
        None,
        DelegationPolicy::ParentFirst,
        vec![root(&platform, b"")],
    );
    let changed_parent = resolve(
        &content,
        environment(
            &app,
            app_domain.clone(),
            vec![
                app_domain,
                without_boot_parent,
                domain(
                    "boot",
                    None,
                    DelegationPolicy::ParentFirst,
                    vec![root(&boot, b"")],
                ),
            ],
            vec![],
        ),
    );
    assert_ne!(
        parent_first.environment_identity,
        changed_parent.environment_identity
    );
    assert_eq!(
        only_selection(&changed_parent).0,
        &LoaderId("platform".into())
    );
}

#[test]
fn module_mode_changes_identity_and_refuses_selection() {
    let app = open(zip(&[(b"p/S.class", &class_bytes(b"p/S", 52))]));
    let caller = domain(
        "app",
        Some("platform"),
        DelegationPolicy::ParentFirst,
        vec![],
    );
    let platform = domain(
        "platform",
        None,
        DelegationPolicy::ParentFirst,
        vec![root(&app, b"")],
    );
    let base = resolve(
        std::slice::from_ref(&app),
        environment(
            &app,
            caller.clone(),
            vec![caller.clone(), platform.clone()],
            vec![],
        ),
    );
    let mut changed = platform;
    changed.module_mode = ModuleMode::ModulePath;
    let report = resolve(
        std::slice::from_ref(&app),
        environment(
            &app,
            caller,
            vec![
                domain(
                    "app",
                    Some("platform"),
                    DelegationPolicy::ParentFirst,
                    vec![],
                ),
                changed,
            ],
            vec![],
        ),
    );
    assert_ne!(base.environment_identity, report.environment_identity);
    assert!(report.resolved.is_none());
    assert!(!report.environment_problems.is_empty());
}

#[test]
fn noncaller_uncertainty_changes_identity_without_changing_selection() {
    let app = open(zip(&[(b"p/S.class", &class_bytes(b"p/S", 52))]));
    let caller = domain(
        "app",
        Some("platform"),
        DelegationPolicy::ParentFirst,
        vec![],
    );
    let base_platform = domain(
        "platform",
        None,
        DelegationPolicy::ParentFirst,
        vec![root(&app, b"")],
    );
    let base = resolve(
        std::slice::from_ref(&app),
        environment(
            &app,
            caller.clone(),
            vec![caller.clone(), base_platform.clone()],
            vec![],
        ),
    );
    for runtime_transformation in [RuntimeUncertainty::Possible, RuntimeUncertainty::Unknown] {
        let mut changed = base_platform.clone();
        changed.runtime_transformation = runtime_transformation;
        let report = resolve(
            std::slice::from_ref(&app),
            environment(&app, caller.clone(), vec![caller.clone(), changed], vec![]),
        );
        assert_ne!(base.environment_identity, report.environment_identity);
        assert_eq!(only_selection(&base).0, only_selection(&report).0);
        assert_eq!(only_selection(&base).1, only_selection(&report).1);
    }
    let mut changed = base_platform;
    changed.external_override = RuntimeUncertainty::Possible;
    let report = resolve(
        std::slice::from_ref(&app),
        environment(&app, caller.clone(), vec![caller, changed], vec![]),
    );
    assert_ne!(base.environment_identity, report.environment_identity);
    assert_eq!(only_selection(&base).1, only_selection(&report).1);
}

#[test]
fn provider_root_declaration_is_identity_even_when_selection_is_unchanged() {
    let selected = open(zip(&[(b"p/S.class", &class_bytes(b"p/S", 52))]));
    let unused = open(zip(&[(b"other/C.class", &class_bytes(b"other/C", 52))]));
    let caller = domain(
        "app",
        None,
        DelegationPolicy::ParentFirst,
        vec![root(&selected, b""), root(&unused, b"")],
    );
    let provider_a = HeaderProvider {
        id: ProviderId("headers".into()),
        roots: vec![root(&selected, b"")],
    };
    let provider_b = HeaderProvider {
        id: ProviderId("headers".into()),
        roots: vec![root(&unused, b"")],
    };
    let env = |provider| {
        environment(
            &selected,
            caller.clone(),
            vec![caller.clone()],
            vec![provider],
        )
    };
    let a = resolve(&[selected.clone(), unused.clone()], env(provider_a));
    let b = resolve(&[selected.clone(), unused.clone()], env(provider_b));
    assert_ne!(a.environment_identity, b.environment_identity);
    assert_eq!(only_selection(&a).0, only_selection(&b).0);
    assert_eq!(only_selection(&a).1, only_selection(&b).1);
}

// ArtifactInput has only Path/Bytes variants, and ArtifactSnapshot::id is a digest of the bytes.
// There is no display label to mutate through this API, so a display-label invariance test cannot
// be expressed here without inventing a field or changing artifact content.

#[test]
fn stale_artifact_is_refused_after_only_a_noncaller_environment_change() {
    let engine = Engine::new();
    let holder = open(zip(&[(
        b"Holder.class",
        include_bytes!("fixtures/p3-declaration/v8/Holder.class"),
    )]));
    let first = open(zip(&[(b"p/S.class", &class_bytes(b"p/S", 52))]));
    let second = open(zip(&[(b"p/S.class", &class_bytes(b"p/S", 51))]));
    let caller = domain(
        "app",
        Some("parent"),
        DelegationPolicy::ChildFirst,
        vec![root(&holder, b"")],
    );
    let env = |roots| {
        environment(
            &holder,
            caller.clone(),
            vec![
                caller.clone(),
                domain("parent", None, DelegationPolicy::ParentFirst, roots),
            ],
            vec![],
        )
    };
    let content = [holder.clone(), first.clone(), second.clone()];
    let owner = engine
        .list_class_declarations(
            &holder,
            &PhysicalScope::SnapshotAll,
            &mut Budget::new(limits()),
        )
        .unwrap()
        .items[0]
        .definition
        .clone();
    let request = |environment| MethodAnalysisRequest {
        environment,
        method: PhysicalMethodId {
            owner: owner.clone(),
            name: JvmBytes(b"value".to_vec()),
            descriptor: JvmBytes(b"()I".to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    let left = engine
        .recover_method_with_evidence(
            &content,
            SourceTarget::JAVA_8,
            &request(env(vec![root(&first, b""), root(&second, b"")])),
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits()),
        )
        .unwrap();
    assert!(left.recovery().produced());
    let old = left.recovery().artifact.binding().unwrap().clone();
    let right = engine
        .recover_method_with_evidence(
            &content,
            SourceTarget::JAVA_8,
            &request(env(vec![root(&second, b""), root(&first, b"")])),
            &RecoveryEvidenceRequest::all().with_expected_artifact(old),
            &mut Budget::new(limits()),
        )
        .unwrap();
    assert!(right.recovery().produced());
    assert_eq!(left.recovery().text, right.recovery().text);
    println!("agreement={:?}", right.recovery().artifact.agreement());
    match right.recovery().artifact.agreement() {
        ArtifactAgreement::Mismatched { mismatch } => {
            assert!(mismatch.disagrees_on(ArtifactDimension::Environment))
        }
        other => panic!("a changed parent declaration must refuse old evidence: {other:?}"),
    }
}

#[test]
fn changing_only_input_path_keeps_identity_and_physical_selection() {
    let directory = std::env::temp_dir().join(format!(
        "jarde-s1-identity-path-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let bytes = zip(&[(b"p/S.class", &class_bytes(b"p/S", 52))]);
    let first_path = directory.join("first-display-name.jar");
    let second_path = directory.join("renamed-display-name.jar");
    std::fs::write(&first_path, &bytes).unwrap();
    std::fs::write(&second_path, &bytes).unwrap();
    let first = ArtifactSnapshot::open(ArtifactInput::Path(first_path), &mut Budget::new(limits()))
        .unwrap();
    let second =
        ArtifactSnapshot::open(ArtifactInput::Path(second_path), &mut Budget::new(limits()))
            .unwrap();
    assert_eq!(first.id(), second.id());
    let run = |snapshot: &ArtifactSnapshot| {
        let caller = domain(
            "app",
            None,
            DelegationPolicy::ParentFirst,
            vec![root(snapshot, b"")],
        );
        resolve(
            std::slice::from_ref(snapshot),
            environment(snapshot, caller.clone(), vec![caller], vec![]),
        )
    };
    let a = run(&first);
    let b = run(&second);
    assert_eq!(a.environment_identity, b.environment_identity);
    assert_eq!(only_selection(&a), only_selection(&b));
    std::fs::remove_dir_all(directory).unwrap();
}
