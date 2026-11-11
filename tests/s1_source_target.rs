use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const STORE: u16 = 0;

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 24,
        archive_entries: 10_000,
        entry_bytes: 1 << 24,
        read_bytes: 1 << 24,
        class_bytes: 1 << 24,
        attribute_bytes: 1 << 24,
        code_bytes: 1 << 24,
        result_items: 10_000,
        output_bytes: 1 << 24,
        class_headers: 10_000,
        method_bodies: 10_000,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn u16b(out: &mut Vec<u8>, n: u16) {
    out.extend_from_slice(&n.to_be_bytes());
}
fn u32b(out: &mut Vec<u8>, n: u32) {
    out.extend_from_slice(&n.to_be_bytes());
}

#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}
impl Pool {
    fn push(&mut self, e: Vec<u8>) -> u16 {
        self.entries.push(e);
        u16::try_from(self.entries.len()).unwrap()
    }
    fn utf8(&mut self, s: &[u8]) -> u16 {
        let mut e = vec![1];
        u16b(&mut e, u16::try_from(s.len()).unwrap());
        e.extend_from_slice(s);
        self.push(e)
    }
    fn class(&mut self, n: u16) -> u16 {
        let mut e = vec![7];
        u16b(&mut e, n);
        self.push(e)
    }
    fn name_and_type(&mut self, n: u16, d: u16) -> u16 {
        let mut e = vec![12];
        u16b(&mut e, n);
        u16b(&mut e, d);
        self.push(e)
    }
    fn member(&mut self, tag: u8, c: u16, nt: u16) -> u16 {
        let mut e = vec![tag];
        u16b(&mut e, c);
        u16b(&mut e, nt);
        self.push(e)
    }
    fn declared(&self) -> u16 {
        u16::try_from(self.entries.len() + 1).unwrap()
    }
    fn bytes(&self) -> Vec<u8> {
        self.entries.iter().flatten().copied().collect()
    }
}
struct MethodSpec {
    access: u16,
    name: u16,
    desc: u16,
    attrs: Vec<(u16, Vec<u8>)>,
}
fn code_body(max_stack: u16, max_locals: u16, code: &[u8]) -> Vec<u8> {
    let mut b = Vec::new();
    u16b(&mut b, max_stack);
    u16b(&mut b, max_locals);
    u32b(&mut b, u32::try_from(code.len()).unwrap());
    b.extend_from_slice(code);
    u16b(&mut b, 0);
    u16b(&mut b, 0);
    b
}
#[allow(clippy::too_many_arguments)]
fn class_file(
    pool: &Pool,
    major: u16,
    access: u16,
    this: u16,
    sup: u16,
    methods: &[MethodSpec],
) -> Vec<u8> {
    let mut b = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut b, 0);
    u16b(&mut b, major);
    u16b(&mut b, pool.declared());
    b.extend_from_slice(&pool.bytes());
    u16b(&mut b, access);
    u16b(&mut b, this);
    u16b(&mut b, sup);
    u16b(&mut b, 0); // interfaces
    u16b(&mut b, 0); // fields
    u16b(&mut b, u16::try_from(methods.len()).unwrap());
    for m in methods {
        u16b(&mut b, m.access);
        u16b(&mut b, m.name);
        u16b(&mut b, m.desc);
        u16b(&mut b, u16::try_from(m.attrs.len()).unwrap());
        for (name, data) in &m.attrs {
            u16b(&mut b, *name);
            u32b(&mut b, u32::try_from(data.len()).unwrap());
            b.extend_from_slice(data);
        }
    }
    u16b(&mut b, 0);
    b // class attributes
}
fn zip_bytes(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    {
        let mut zip = ZipArchiveWriter::new(&mut out);
        for (name, data) in entries {
            let (mut e, cfg) = zip
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .unwrap();
            let mut writer = cfg.wrap(&mut e);
            writer.write_all(data).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            e.finish(descriptor).unwrap();
        }
        zip.finish().unwrap();
    }
    out.into_inner()
}

/// p/Join.main invokes p/All.run. The only byte-level change is the variant's class major
/// and superclass, so origin/physical variant identifies root, v11, and v17 independently.
fn join_class(major: u16, super_name: &[u8]) -> Vec<u8> {
    let mut p = Pool::default();
    let own_name = p.utf8(b"p/Join");
    let own = p.class(own_name);
    let sup_name = p.utf8(super_name);
    let sup = p.class(sup_name);
    let all_name = p.utf8(b"p/All");
    let all = p.class(all_name);
    let run = p.utf8(b"run");
    let void = p.utf8(b"()V");
    let run_nat = p.name_and_type(run, void);
    let target = p.member(10, all, run_nat);
    let main = p.utf8(b"main");
    let code_name = p.utf8(b"Code");
    let mut code = vec![0xb8];
    u16b(&mut code, target);
    code.push(0xb1);
    class_file(
        &p,
        major,
        0x0021,
        own,
        sup,
        &[MethodSpec {
            access: 0x0009,
            name: main,
            desc: void,
            attrs: vec![(code_name, code_body(0, 0, &code))],
        }],
    )
}
fn all_class() -> Vec<u8> {
    let mut p = Pool::default();
    let own_name = p.utf8(b"p/All");
    let own = p.class(own_name);
    let object_name = p.utf8(b"java/lang/Object");
    let object = p.class(object_name);
    let run = p.utf8(b"run");
    let void = p.utf8(b"()V");
    let code_name = p.utf8(b"Code");
    class_file(
        &p,
        55,
        0x0021,
        own,
        object,
        &[MethodSpec {
            access: 0x0009,
            name: run,
            desc: void,
            attrs: vec![(code_name, code_body(0, 0, &[0xb1]))],
        }],
    )
}
fn mr_jar() -> Vec<u8> {
    let manifest = b"Manifest-Version: 1.0\r\nMulti-Release: true\r\n\r\n";
    let root = join_class(52, b"java/lang/Object");
    let v11 = join_class(55, b"java/lang/Object");
    let v17 = join_class(61, b"java/lang/Object");
    let all = all_class();
    zip_bytes(&[
        (b"META-INF/MANIFEST.MF", manifest),
        (b"p/Join.class", &root),
        (b"META-INF/versions/11/p/Join.class", &v11),
        (b"META-INF/versions/17/p/Join.class", &v17),
        (b"p/All.class", &all),
    ])
}
fn runtime11() -> RuntimeProfile {
    RuntimeProfile {
        java_release: 11,
        multi_release: MultiReleasePolicy::Enabled,
        layout: LayoutMode::Generic,
    }
}
fn env(snapshot: &ArtifactSnapshot) -> EnvironmentRequest {
    EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope: PhysicalScope::SnapshotAll,
        policy: EnvironmentPolicy::PlainJar,
        profile: runtime11(),
        loader: LoaderId("app".into()),
    }
}
fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut Budget::new(limits()),
        )
        .unwrap()
}
fn resolve(
    snapshot: &ArtifactSnapshot,
    environment: ResolutionEnvironment,
    owner: &[u8],
    name: &[u8],
) -> ResolutionReport {
    Engine::new()
        .resolve_symbol(
            std::slice::from_ref(snapshot),
            &ResolutionRequest {
                environment,
                target: SymbolRef::Method {
                    owner: JvmBytes(owner.to_vec()),
                    name: JvmBytes(name.to_vec()),
                    descriptor: JvmBytes(b"()V".to_vec()),
                },
                use_kind: ReferenceUse::InvokeStatic,
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

#[test]
fn source_targets_preserve_mr_symbol_and_ir_identity_and_separate_artifact_configuration() {
    let engine = Engine::new();
    let snapshot = open(&mr_jar());
    let mut environment = env(&snapshot);
    let view = environment.build(std::slice::from_ref(&snapshot)).unwrap();
    let mr = engine
        .select_multi_release(&snapshot, &view.runtime, &mut Budget::new(limits()))
        .unwrap();
    let container = &mr.containers[0];
    let chosen = container
        .selections
        .iter()
        .find(|row| row.logical_path.0 == b"p/Join.class")
        .unwrap();
    let multi_release::MultiReleaseSelectionOutcome::Selected { entry: winner } = &chosen.outcome
    else {
        panic!("MR selection is complete")
    };
    assert_eq!(winner.raw_name.0, b"META-INF/versions/11/p/Join.class");
    // Standard MR selection is per-container, not a loader policy. Declare its selected prefix
    // explicitly, then retain the base root for the unchanged p/All dependency.
    environment.policy = EnvironmentPolicy::ExplicitClasspath {
        roots: vec![
            LoadRoot::Container {
                origin: container.origin.clone(),
                prefix: ArchiveNameBytes(
                    winner
                        .raw_name
                        .0
                        .strip_suffix(b"p/Join.class")
                        .unwrap()
                        .to_vec(),
                ),
            },
            LoadRoot::Container {
                origin: container.origin.clone(),
                prefix: ArchiveNameBytes(Vec::new()),
            },
        ],
    };
    let runtime = environment.build(std::slice::from_ref(&snapshot)).unwrap();
    let selected = resolve(&snapshot, runtime.clone(), b"p/Join", b"main")
        .resolved
        .unwrap();
    assert_eq!(
        selected.definition.variant,
        PhysicalVariant::MultiRelease { version: 11 }
    );
    assert_eq!(selected.definition.location.entry().unwrap(), winner);
    let request = MethodOperationRequest {
        environment,
        method: MethodRef::Method {
            method: PhysicalMethodId {
                owner: selected.definition,
                name: JvmBytes(b"main".to_vec()),
                descriptor: JvmBytes(b"()V".to_vec()),
            },
        },
    };
    let mut recovered = Vec::new();
    let mut dependencies = Vec::new();
    for target in [SourceTarget::JAVA_7, SourceTarget::JAVA_8] {
        let OperationOutcome::Performed(report) = engine
            .recover_target(
                std::slice::from_ref(&snapshot),
                target,
                &request,
                &mut Budget::new(limits()),
            )
            .unwrap()
        else {
            panic!("physical target is unique")
        };
        assert_eq!(report.recovered.recovery().source_target, target);
        assert_eq!(report.recovered.recovery().runtime_profile, runtime11());
        assert!(report.recovered.recovery().produced());
        dependencies.push(
            resolve(&snapshot, runtime.clone(), b"p/All", b"run")
                .resolved
                .unwrap(),
        );
        recovered.push(report);
    }
    assert_eq!(dependencies[0], dependencies[1]);
    assert_eq!(
        dependencies[0].member,
        SymbolRef::Method {
            owner: JvmBytes(b"p/All".to_vec()),
            name: JvmBytes(b"run".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec())
        }
    );
    let a = recovered[0].recovered.recovery();
    let b = recovered[1].recovered.recovery();
    assert_eq!(recovered[0].method, recovered[1].method);
    assert_eq!(
        recovered[0].recovered.analysis().environment_identity,
        recovered[1].recovered.analysis().environment_identity
    );
    assert_eq!(
        recovered[0].recovered.analysis().reads,
        recovered[1].recovered.analysis().reads
    );
    assert_eq!(a.text, b.text);
    let binding7 = a.artifact.binding().unwrap();
    let binding8 = b.artifact.binding().unwrap();
    assert_eq!(binding7.environment(), binding8.environment());
    assert_eq!(binding7.text_digest(), binding8.text_digest());
    let mismatch = binding7.mismatch(binding8).unwrap();
    assert_eq!(
        mismatch.dimensions(),
        &[jarde_java::ArtifactDimension::Configuration]
    );
    // Invalid output configuration cannot poison the independent physical/IR APIs.
    for release in [0, 6, 9, 11, u16::MAX] {
        assert!(SourceTarget::new(release).is_err());
    }
    let OperationOutcome::Performed(pure) = engine
        .analyze_target(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::new(limits()),
        )
        .unwrap()
    else {
        panic!("pure analysis completes")
    };
    assert_eq!(pure.method, recovered[0].method);
    assert_eq!(
        pure.analysis.environment_identity,
        recovered[0].recovered.analysis().environment_identity
    );
    assert!(matches!(
        pure.analysis.execution,
        ExecutionReport::Complete { .. }
    ));
}

fn source(bytes: &[u8], class: &str, target: SourceTarget) -> ClassSourceReport {
    let snapshot = open(bytes);
    let mut environment = env(&snapshot);
    environment.policy = if snapshot.kind() == ArtifactKind::Zip {
        EnvironmentPolicy::PlainJar
    } else {
        EnvironmentPolicy::SingleClass
    };
    environment.profile = RuntimeProfile {
        java_release: 8,
        multi_release: MultiReleasePolicy::Disabled,
        layout: LayoutMode::Generic,
    };
    let OperationOutcome::Performed(report) = Engine::new()
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &ClassSourceRequest {
                source_target: target,
                class: ClassRef::Name {
                    class: ClassNameQuery::internal(class),
                },
                environment,
            },
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits()),
        )
        .unwrap()
    else {
        panic!("unique fixture")
    };
    report
}

#[test]
fn source7_refuses_lambda_method_reference_interface_bodies_and_type_uses_without_reselecting_input()
 {
    let cases: &[(&[u8], &str, bool)] = &[
        (
            include_bytes!("fixtures/p3-lambda-adaptation/v8/LambdaAdaptationProbe.class"),
            "LambdaAdaptationProbe",
            false,
        ),
        (
            include_bytes!("fixtures/p3-lambda-adaptation/v8/CapturedLambdaAdaptationProbe.class"),
            "CapturedLambdaAdaptationProbe",
            false,
        ),
        (
            include_bytes!("fixtures/p3-declarations/v8/p/Ops2.class"),
            "p/Ops2",
            true,
        ),
        (
            include_bytes!(
                "../tests/fixtures/recovered-inputs/evidence/java-syntax-2026-09-22/type-use-annotations/generated/original/TypeUseSubject.class"
            ),
            "TypeUseSubject",
            true,
        ),
    ];
    for (bytes, class, whole_unit_refused) in cases {
        let seven = source(bytes, class, SourceTarget::JAVA_7);
        let eight = source(bytes, class, SourceTarget::JAVA_8);
        assert_eq!(seven.class, eight.class);
        assert_eq!(seven.runtime_profile, eight.runtime_profile);
        assert_eq!(seven.methods.len(), eight.methods.len());
        if *whole_unit_refused {
            assert!(
                seven.text.contains("source_target_refused"),
                "{class}: {}",
                seven.text
            );
        } else {
            assert!(
                eight.text.contains("->") || eight.text.contains("::"),
                "{class}: {}",
                eight.text
            );
            assert!(
                !seven.text.contains("->") && !seven.text.contains("::"),
                "{class}: {}",
                seven.text
            );
            assert!(seven.methods.iter().any(|method| match &method.outcome {
                ClassSourceOutcome::Recovered { report, .. } => report.lambdas.iter().any(|site| {
                    site.refusal
                        .as_ref()
                        .is_some_and(|refusal| refusal.code == "jre_lambda_rule_not_admitted")
                }),
                _ => false,
            }));
        }
        for method in &seven.methods {
            if let ClassSourceOutcome::Recovered { report, .. } = &method.outcome {
                assert_eq!(report.source_target, SourceTarget::JAVA_7);
                assert_eq!(report.runtime_profile, seven.runtime_profile);
            }
        }
    }
}

#[test]
fn java7_keeps_twr_but_refuses_proved_qualified_super_calls() {
    let twr = include_bytes!(
        "../tests/fixtures/recovered-inputs/evidence/java-syntax-2026-10-04/twr-javac8-codegen-patrol/fixture/real-javac8-TR/TR.class"
    );
    let seven = source(twr, "TR", SourceTarget::JAVA_7);
    let eight = source(twr, "TR", SourceTarget::JAVA_8);
    assert!(seven.text.contains("try (TR"), "{}", seven.text);
    assert_eq!(seven.text, eight.text);
    let bytes = zip_bytes(&[
        (
            b"NestedSuper.class",
            include_bytes!("fixtures/qualified-super-default/v8/NestedSuper.class"),
        ),
        (
            b"NestedSuper$A.class",
            include_bytes!("fixtures/qualified-super-default/v8/NestedSuper$A.class"),
        ),
        (
            b"NestedSuper$Single.class",
            include_bytes!("fixtures/qualified-super-default/v8/NestedSuper$Single.class"),
        ),
    ]);
    let seven = source(&bytes, "NestedSuper$Single", SourceTarget::JAVA_7);
    let eight = source(&bytes, "NestedSuper$Single", SourceTarget::JAVA_8);
    assert_eq!(seven.class, eight.class);
    assert!(eight.text.contains(".super."), "{}", eight.text);
    assert!(
        !seven
            .text
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .any(|line| line.contains(".super.")),
        "{}",
        seven.text
    );
    assert!(
        seven.text.contains("requires Java source target 8"),
        "{}",
        seven.text
    );
}

// Two independent methods deliberately use the same call BCI. Class-source must supply each
// body's own widening proof, under the selected loader graph, to the existing Java consumer.
fn widening_scope_class() -> Vec<u8> {
    let mut p = Pool::default();
    let own = p.utf8(b"p/Scope");
    let this = p.class(own);
    let object = p.utf8(b"java/lang/Object");
    let sup = p.class(object);
    let code_name = p.utf8(b"Code");
    let mut methods = Vec::new();
    for (name, source, target, sink) in [
        (
            b"alpha".as_slice(),
            b"(Lp/S;)V".as_slice(),
            b"(Lp/T;)V".as_slice(),
            b"sinkA".as_slice(),
        ),
        (
            b"beta".as_slice(),
            b"(Lp/U;)V".as_slice(),
            b"(Lp/V;)V".as_slice(),
            b"sinkB".as_slice(),
        ),
    ] {
        let name = p.utf8(name);
        let source = p.utf8(source);
        let sink = p.utf8(sink);
        let target = p.utf8(target);
        let nt = p.name_and_type(sink, target);
        let call = p.member(10, this, nt);
        let mut code = vec![0x2a, 0xb8]; // aload_0; invokestatic at BCI 1
        u16b(&mut code, call);
        code.push(0xb1);
        methods.push(MethodSpec {
            access: 0x0009,
            name,
            desc: source,
            attrs: vec![(code_name, code_body(1, 1, &code))],
        });
        methods.push(MethodSpec {
            access: 0x0009,
            name: sink,
            desc: target,
            attrs: vec![(code_name, code_body(0, 1, &[0xb1]))],
        });
    }
    class_file(&p, 52, 0x0021, this, sup, &methods)
}

fn widening_scope_header(name: &[u8], parent: &[u8]) -> Vec<u8> {
    let mut p = Pool::default();
    let name = p.utf8(name);
    let this = p.class(name);
    let name = p.utf8(parent);
    let sup = p.class(name);
    class_file(&p, 52, 0x0021, this, sup, &[])
}

#[test]
fn explicit_domains_preserve_the_declaration_and_require_one_caller() {
    let snapshot = open(&mr_jar());
    let content = std::slice::from_ref(&snapshot);
    let mut request = env(&snapshot);
    let mut caller = request.build(content).unwrap().runtime.load_domain;
    caller.parent_loader = Some(LoaderId("parent".into()));
    caller.delegation = DelegationPolicy::ChildFirst;
    let mut parent = caller.clone();
    parent.loader = LoaderId("parent".into());
    parent.parent_loader = None;
    let domains = vec![parent, caller.clone()];
    let providers = vec![HeaderProvider {
        id: ProviderId("headers".into()),
        roots: caller.roots.clone(),
    }];
    request.policy = EnvironmentPolicy::ExplicitDomains {
        domains: domains.clone(),
        providers: providers.clone(),
    };
    let built = request.build(content).unwrap();
    assert_eq!(built.domains, domains);
    assert_eq!(built.providers, providers);
    assert_eq!(built.runtime.load_domain, caller);
    assert_eq!(built.runtime.profile, request.profile);
    assert_eq!(built.runtime.physical.snapshot, request.snapshot);
    assert_eq!(built.runtime.physical.scope, request.scope);
    let encoded = serde_json::to_vec(&request).unwrap();
    assert_eq!(
        serde_json::from_slice::<EnvironmentRequest>(&encoded).unwrap(),
        request
    );
    for domains in [vec![], vec![caller.clone(), caller]] {
        request.policy = EnvironmentPolicy::ExplicitDomains {
            domains,
            providers: vec![],
        };
        let error = request.build(content).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("environment_policy_caller_domain_not_unique")
        );
        let mut budget = Budget::new(limits());
        assert!(
            Engine::new()
                .class_source_with_evidence(
                    content,
                    &ClassSourceRequest {
                        source_target: SourceTarget::JAVA_8,
                        class: ClassRef::Name {
                            class: ClassNameQuery::internal("p/Join")
                        },
                        environment: request.clone(),
                    },
                    &RecoveryEvidenceRequest::essential(),
                    &mut budget
                )
                .is_err()
        );
        assert_eq!(budget.usage().method_bodies, 0);
    }
}

#[test]
fn class_source_widening_keeps_method_bci_and_defining_loader_scopes() {
    let scope = widening_scope_class();
    let facts =
        jarde_reader::classfile::class_member_facts(&scope, &mut Budget::new(limits())).unwrap();
    for name in [b"alpha".as_slice(), b"beta".as_slice()] {
        let method = facts
            .methods
            .iter()
            .find(|m| m.name.raw().0 == name)
            .unwrap();
        let code =
            jarde_reader::classfile::method_code_facts(&scope, method, &mut Budget::new(limits()))
                .unwrap();
        assert_eq!(
            code.instructions
                .iter()
                .filter(|i| i.opcode == 0xb8)
                .map(|i| i.bci)
                .collect::<Vec<_>>(),
            vec![1]
        );
    }
    let source = widening_scope_header(b"p/S", b"p/Mid");
    let middle = widening_scope_header(b"p/Mid", b"p/T");
    let broken = widening_scope_header(b"p/Mid", b"java/lang/Object");
    let target = widening_scope_header(b"p/T", b"java/lang/Object");
    let other_source = widening_scope_header(b"p/U", b"p/V");
    let other_target = widening_scope_header(b"p/V", b"java/lang/Object");
    let parent = open(&zip_bytes(&[
        (b"p/S.class", &source),
        (b"p/Mid.class", &middle),
        (b"p/T.class", &target),
    ]));
    for (context, child_source, child_middle, child_target, alpha_proved) in [
        ("child-valid", true, middle.as_slice(), true, true),
        ("child-broken", true, broken.as_slice(), true, false),
        ("parent-chain", false, broken.as_slice(), false, true),
        (
            "parent-target-shadowed",
            false,
            middle.as_slice(),
            true,
            false,
        ),
    ] {
        let mut entries = vec![
            (b"p/Scope.class".as_slice(), scope.as_slice()),
            (b"p/Mid.class", child_middle),
            (b"p/U.class", other_source.as_slice()),
            (b"p/V.class", other_target.as_slice()),
        ];
        if child_source {
            entries.push((b"p/S.class", &source));
        }
        if child_target {
            entries.push((b"p/T.class", &target));
        }
        let child = open(&zip_bytes(&entries));
        let content = [child.clone(), parent.clone()];
        let mut environment = env(&child);
        let mut caller = environment.build(&content).unwrap().runtime.load_domain;
        caller.parent_loader = Some(LoaderId("parent".into()));
        caller.delegation = DelegationPolicy::ChildFirst;
        let mut parent_domain = caller.clone();
        parent_domain.loader = LoaderId("parent".into());
        parent_domain.parent_loader = None;
        parent_domain.delegation = DelegationPolicy::ParentFirst;
        parent_domain.roots = vec![LoadRoot::Container {
            origin: ContainerOrigin {
                snapshot: parent.id().clone(),
                root_container: ContainerId("root".into()),
                steps: vec![],
            },
            prefix: ArchiveNameBytes(vec![]),
        }];
        environment.policy = EnvironmentPolicy::ExplicitDomains {
            domains: vec![caller, parent_domain],
            providers: vec![],
        };
        let built = environment.build(&content).unwrap();
        for (owner, local) in [
            (b"p/S".as_slice(), child_source),
            (b"p/Mid".as_slice(), true),
            (b"p/T".as_slice(), child_target),
        ] {
            let selected = Engine::new()
                .resolve_symbol(
                    &content,
                    &ResolutionRequest {
                        environment: built.clone(),
                        target: SymbolRef::Class {
                            owner: JvmBytes(owner.to_vec()),
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
                .resolved
                .unwrap();
            assert_eq!(
                selected.loader,
                LoaderId(if local { "app" } else { "parent" }.into()),
                "{context}"
            );
            assert_eq!(
                selected.definition.snapshot(),
                if local { child.id() } else { parent.id() },
                "{context}"
            );
        }
        for target in [SourceTarget::JAVA_7, SourceTarget::JAVA_8] {
            let mut text = None;
            for evidence in [
                RecoveryEvidenceRequest::essential(),
                RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
                RecoveryEvidenceRequest::all(),
            ] {
                let OperationOutcome::Performed(report) = Engine::new()
                    .class_source_with_evidence(
                        &content,
                        &ClassSourceRequest {
                            source_target: target,
                            class: ClassRef::Name {
                                class: ClassNameQuery::internal("p/Scope"),
                            },
                            environment: environment.clone(),
                        },
                        &evidence,
                        &mut Budget::new(limits()),
                    )
                    .unwrap()
                else {
                    panic!("{context}: unique class")
                };
                assert!(
                    matches!(report.execution, ExecutionReport::Complete { .. }),
                    "{context}: {:?}",
                    report.execution
                );
                for (name, cast, forbidden, proved) in [
                    (b"alpha".as_slice(), "(p.T)", "(p.V)", alpha_proved),
                    (b"beta".as_slice(), "(p.V)", "(p.T)", true),
                ] {
                    let method = report
                        .methods
                        .iter()
                        .find(|m| m.item.name.raw().0 == name)
                        .unwrap();
                    assert_eq!(method.item.identity.owner.snapshot(), child.id());
                    let ClassSourceOutcome::Recovered {
                        report: body,
                        analysis,
                    } = &method.outcome
                    else {
                        panic!("{context}: recovered method {name:?}")
                    };
                    assert!(matches!(
                        analysis.execution,
                        ExecutionReport::Complete { .. }
                    ));
                    assert_eq!(
                        body.text.contains(cast),
                        proved,
                        "{context} {name:?}: {}",
                        body.text
                    );
                    assert!(
                        !body.text.contains(forbidden),
                        "{context} {name:?}: {}",
                        body.text
                    );
                    assert_eq!(
                        body.text.contains("no safe reference conversion evidence"),
                        !proved,
                        "{context} {name:?}: {}",
                        body.text
                    );
                }
                if let Some(previous) = &text {
                    assert_eq!(&report.text, previous, "{context}: evidence changed source");
                }
                text = Some(report.text.clone());
                println!(
                    "AW_SCOPE_REPORT {}",
                    serde_json::json!({"context":context,"target":target,"evidence":evidence,"report":report})
                );
            }
        }
    }
}
