//! Diagnostic-only negative closure. Header-only counterexamples are never executed as Java.
//! The analyzed method is the unchanged pinned Consumer.run, through the public Engine entry.
use jarde::*;
use rawzip::{CompressionMethod, ZipArchive};
use std::io::Read;

fn limits() -> Limits {
    let mut limits = task_budget(&[]).unwrap().limits().clone();
    limits.elapsed_millis = u64::MAX;
    limits.dependency_depth = 64;
    limits
}

fn consumer() -> Vec<u8> {
    let archive = ZipArchive::from_slice(include_bytes!(
        "fixtures/s2a-reference-relations/corretto8-override.jar"
    ))
    .unwrap();
    let mut entries = archive.entries();
    while let Some(header) = entries.next_entry().unwrap() {
        if header.file_path().as_ref() != b"Consumer.class" {
            continue;
        }
        let entry = archive.get_entry(header.wayfinder()).unwrap();
        let reader: Box<dyn Read + '_> = match header.compression_method() {
            CompressionMethod::STORE => Box::new(entry.data()),
            CompressionMethod::DEFLATE => {
                Box::new(flate2::bufread::DeflateDecoder::new(entry.data()))
            }
            other => panic!("unexpected compression: {other:?}"),
        };
        let mut bytes = Vec::new();
        entry
            .verifying_reader(reader)
            .read_to_end(&mut bytes)
            .unwrap();
        return bytes;
    }
    panic!("pinned Consumer.class exists")
}

fn header(name: &str, parent: Option<&str>, interfaces: &[&str]) -> Vec<u8> {
    let names: Vec<_> = std::iter::once(name)
        .chain(parent)
        .chain(interfaces.iter().copied())
        .collect();
    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    let word = |bytes: &mut Vec<u8>, value: u16| bytes.extend_from_slice(&value.to_be_bytes());
    word(&mut bytes, 0);
    word(&mut bytes, 52);
    word(&mut bytes, u16::try_from(1 + names.len() * 2).unwrap());
    for name in &names {
        bytes.push(1);
        word(&mut bytes, u16::try_from(name.len()).unwrap());
        bytes.extend_from_slice(name.as_bytes());
        bytes.push(7);
        word(
            &mut bytes,
            u16::try_from(1 + names.iter().position(|n| n == name).unwrap() * 2).unwrap(),
        );
    }
    word(&mut bytes, 0x0021);
    word(&mut bytes, 2);
    word(&mut bytes, if parent.is_some() { 4 } else { 0 });
    word(&mut bytes, u16::try_from(interfaces.len()).unwrap());
    for index in 0..interfaces.len() {
        word(
            &mut bytes,
            u16::try_from((1 + usize::from(parent.is_some()) + index) * 2 + 2).unwrap(),
        );
    }
    for _ in 0..3 {
        word(&mut bytes, 0);
    }
    bytes
}

// Named ZIP entries avoid unrelated standalone-probe reads, which the selected-header
// facade correctly refuses to treat as a single selected dependency proof.
fn dependency_archive(headers: Vec<Vec<u8>>) -> ArtifactSnapshot {
    use rawzip::{ZipArchiveWriter, path::EntryPath};
    use std::io::{Cursor, Write};
    let mut budget = Budget::new(limits());
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for bytes in headers {
            let facts = jarde_reader::classfile::class_member_facts(&bytes, &mut budget).unwrap();
            let mut name = facts.this_class.raw().0.clone();
            name.extend_from_slice(b".class");
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name))
                .compression_method(CompressionMethod::STORE)
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(&bytes).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        archive.finish().unwrap();
    }
    Engine::new()
        .open(ArtifactInput::bytes(output.into_inner()), &mut budget)
        .unwrap()
}

fn dependency_root(snapshot: &ArtifactSnapshot) -> LoadRoot {
    LoadRoot::Container {
        origin: ContainerOrigin {
            snapshot: snapshot.id().clone(),
            root_container: ContainerId("root".into()),
            steps: Vec::new(),
        },
        prefix: ArchiveNameBytes(Vec::new()),
    }
}

fn setup(headers: Vec<Vec<u8>>) -> (Vec<ArtifactSnapshot>, MethodAnalysisRequest) {
    let engine = Engine::new();
    let mut budget = Budget::new(limits());
    let snapshots = vec![
        engine
            .open(ArtifactInput::bytes(consumer()), &mut budget)
            .unwrap(),
        dependency_archive(headers),
    ];
    let inspected = engine
        .inspect_header(
            &snapshots[0],
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .unwrap();
    let domain = LoadDomain {
        loader: LoaderId("app".into()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![
            LoadRoot::StandaloneClass {
                snapshot: snapshots[0].id().clone(),
            },
            dependency_root(&snapshots[1]),
        ],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let request = MethodAnalysisRequest {
        environment: ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: snapshots[0].id().clone(),
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
        },
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshots[0].id().clone(),
                },
                class_bytes: inspected.source.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(b"run".to_vec()),
            descriptor: JvmBytes(b"(Ljava/util/ArrayList;)Ljava/lang/Object;".to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    (snapshots, request)
}

fn recover(snapshots: &[ArtifactSnapshot], request: &MethodAnalysisRequest) -> RecoveryReport {
    Engine::new()
        .recover_method_with_evidence(
            snapshots,
            jarde_java::pass::JAVA_8,
            request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits()),
        )
        .unwrap()
        .recovery()
        .clone()
}

fn assert_relation(report: &RecoveryReport, relation: &str) {
    assert!(
        report.text.contains(&format!("relation: Some({relation})")),
        "{}",
        report.text
    );
    assert!(
        !report
            .text
            .contains("return consume((java.util.List) arg0);"),
        "{}",
        report.text
    );
}

fn array(parent: &str, interfaces: &[&str]) -> Vec<u8> {
    header("java/util/ArrayList", Some(parent), interfaces)
}

#[test]
fn terminal_missing_branch_closes_only_for_supported_releases() {
    for (release, relation) in [(8, "Incompatible"), (23, "Incompatible"), (17, "Missing")] {
        let (snapshots, mut request) = setup(vec![array("java/lang/Object", &[])]);
        request.environment.runtime.profile.java_release = release;
        assert_relation(&recover(&snapshots, &request), relation);
    }
}

#[test]
fn unknown_and_nonterminal_missing_parents_do_not_close() {
    for parent in ["absent/Parent", "java/lang/Exception"] {
        let (snapshots, request) = setup(vec![array(parent, &[])]);
        assert_relation(&recover(&snapshots, &request), "Missing");
    }
}

#[test]
fn terminal_branch_does_not_erase_an_unresolved_sibling() {
    let (snapshots, request) = setup(vec![array("java/lang/Object", &["absent/Interface"])]);
    assert_relation(&recover(&snapshots, &request), "Missing");
}

#[test]
fn selected_physical_terminal_replacement_remains_authoritative() {
    let (snapshots, request) = setup(vec![
        array("java/lang/Object", &[]),
        header("java/lang/Object", Some("absent/Parent"), &[]),
    ]);
    assert_relation(&recover(&snapshots, &request), "Missing");
    let (snapshots, request) = setup(vec![
        array("java/lang/Object", &[]),
        header("java/lang/Object", None, &["java/util/List"]),
    ]);
    let report = recover(&snapshots, &request);
    assert!(
        report.text.contains("consume((java.util.List) arg0)"),
        "{}",
        report.text
    );
}

#[test]
fn terminal_at_walk_depth_boundary_stays_unknown() {
    for (depth, relation) in [(7, "Incompatible"), (8, "Unknown")] {
        let mut headers = vec![array("Chain1", &[])];
        for level in 1..depth {
            let parent = if level + 1 == depth {
                "java/lang/Object".to_owned()
            } else {
                format!("Chain{}", level + 1)
            };
            headers.push(header(&format!("Chain{level}"), Some(&parent), &[]));
        }
        let (snapshots, request) = setup(headers);
        assert_relation(&recover(&snapshots, &request), relation);
    }
}

#[test]
fn terminal_closure_requires_clean_complete_loader_search() {
    for uncertainty in [RuntimeUncertainty::Possible, RuntimeUncertainty::Unknown] {
        let (snapshots, mut request) = setup(vec![array("java/lang/Object", &[])]);
        let mut parent = request.environment.runtime.load_domain.clone();
        parent.loader = LoaderId("parent".into());
        parent.roots.clear();
        parent.external_override = uncertainty;
        request.environment.runtime.load_domain.parent_loader = Some(parent.loader.clone());
        request.environment.domains = vec![request.environment.runtime.load_domain.clone(), parent];
        assert_relation(&recover(&snapshots, &request), "Unknown");
    }
}

#[test]
fn parent_selected_physical_object_is_not_replaced_by_terminal_contract() {
    let (mut snapshots, mut request) = setup(vec![array("java/lang/Object", &[])]);
    let object = dependency_archive(vec![header("java/lang/Object", Some("absent/Parent"), &[])]);
    let mut parent = request.environment.runtime.load_domain.clone();
    parent.loader = LoaderId("parent".into());
    parent.roots = vec![dependency_root(&object)];
    snapshots.push(object);
    request.environment.runtime.load_domain.parent_loader = Some(parent.loader.clone());
    request.environment.domains = vec![request.environment.runtime.load_domain.clone(), parent];
    assert_relation(&recover(&snapshots, &request), "Missing");
}

#[test]
fn stopped_requests_never_emit_the_unsupported_cast() {
    let (snapshots, request) = setup(vec![array("java/lang/Object", &[])]);
    for cancelled in [false, true] {
        let mut constrained = limits();
        if !cancelled {
            constrained.analysis_steps = 0;
        }
        let mut budget = Budget::new(constrained);
        if cancelled {
            budget.cancellation_token().cancel();
        }
        match Engine::new().recover_method_with_evidence(
            &snapshots,
            jarde_java::pass::JAVA_8,
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        ) {
            Ok(result) => {
                assert!(
                    !result
                        .recovery()
                        .text
                        .contains("consume((java.util.List) arg0)")
                );
                if cancelled {
                    assert!(matches!(
                        result.analysis().execution,
                        ExecutionReport::Cancelled { .. }
                    ));
                } else {
                    assert!(matches!(
                        result.analysis().execution,
                        ExecutionReport::Partial {
                            reason: TerminationReason::BudgetExceeded {
                                dimension: BudgetDimension::AnalysisSteps
                            },
                            ..
                        }
                    ));
                }
            }
            Err(jarde_reader::error::Error::Cancelled { .. }) if cancelled => {}
            Err(jarde_reader::error::Error::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
                ..
            }) if !cancelled => {}
            other => panic!("unexpected request stop: {other:?}"),
        }
    }
}

#[test]
fn terminal_branch_does_not_erase_a_cyclic_sibling() {
    let (snapshots, request) = setup(vec![
        array("java/lang/Object", &["Cycle"]),
        header("Cycle", Some("Cycle"), &[]),
    ]);
    assert_relation(&recover(&snapshots, &request), "Unknown");
}

#[test]
fn terminal_closure_rejects_noncanonical_loader_modes() {
    for mode in 0..2 {
        let (snapshots, mut request) = setup(vec![array("java/lang/Object", &[])]);
        let mut parent = request.environment.runtime.load_domain.clone();
        parent.loader = LoaderId("parent".into());
        parent.roots.clear();
        match mode {
            0 => parent.delegation = DelegationPolicy::ChildFirst,
            _ => parent.runtime_transformation = RuntimeUncertainty::Possible,
        }
        request.environment.runtime.load_domain.parent_loader = Some(parent.loader.clone());
        request.environment.domains = vec![request.environment.runtime.load_domain.clone(), parent];
        assert_relation(&recover(&snapshots, &request), "Unknown");
    }
}

#[test]
fn module_path_is_rejected_before_reference_proof() {
    let (snapshots, mut request) = setup(vec![array("java/lang/Object", &[])]);
    request.environment.runtime.load_domain.module_mode = ModuleMode::ModulePath;
    request.environment.domains = vec![request.environment.runtime.load_domain.clone()];
    let result = Engine::new()
        .recover_method_with_evidence(
            &snapshots,
            jarde_java::pass::JAVA_8,
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits()),
        )
        .unwrap();
    assert!(
        result
            .analysis()
            .environment_problems
            .iter()
            .any(|problem| problem.code
                == jarde_jvm::environment::EnvironmentProblemCode::UnsupportedPolicy
                && problem.message.contains("module_path"))
    );
    assert!(
        matches!(result.recovery().outcome, RecoveryOutcome::Stopped(_)),
        "{:?}",
        result.recovery().outcome
    );
    assert!(result.recovery().text.is_empty());
}

#[test]
fn terminal_branch_does_not_erase_an_ambiguous_sibling() {
    let (snapshots, request) = setup(vec![
        array("java/lang/Object", &["Duplicate"]),
        header("Duplicate", Some("java/lang/Object"), &[]),
        header("Duplicate", Some("absent/Other"), &[]),
    ]);
    assert_relation(&recover(&snapshots, &request), "Ambiguous");
}

// A supplier-boundary supplement, not an Engine end-to-end claim: stop after the selected
// physical source has led to missing Object, immediately before the new closure's first charge.
// The public Engine tests above separately cover input selection, reads, and stop presentation.
#[test]
fn terminal_negative_loader_search_propagates_budget_and_cancellation() {
    use jarde_java::source_proof::{
        DependencyResult, RecoveryHeaders, ReferenceClassHeader, proved_reference_widening,
    };
    use jarde_reader::{budget::CountedBudgetDimension, error::Error};
    struct StopAtTerminal {
        header: ReferenceClassHeader,
        cancel: bool,
        reached_terminal: bool,
    }
    impl RecoveryHeaders for StopAtTerminal {
        type Error = Error;
        fn caller_loader(&self) -> &LoaderId {
            &self.header.loader
        }
        fn selected_from(
            &mut self,
            loader: &LoaderId,
            owner: &[u8],
            budget: &mut Budget,
        ) -> std::result::Result<DependencyResult<ReferenceClassHeader>, Error> {
            assert_eq!(loader, &self.header.loader);
            budget.poll()?;
            match owner {
                b"java/util/ArrayList" => Ok(DependencyResult::Found(self.header.clone())),
                b"java/lang/Object" => {
                    assert!(!self.reached_terminal);
                    self.reached_terminal = true;
                    if self.cancel {
                        budget.cancellation_token().cancel();
                    } else {
                        // Exhaust what remains, without relying on any hardcoded pipeline cost.
                        budget.charge(
                            CountedBudgetDimension::AnalysisSteps,
                            budget.limits().analysis_steps - budget.usage().analysis_steps,
                        )?;
                    }
                    Ok(DependencyResult::Missing)
                }
                _ => panic!("unexpected relation lookup: {owner:?}"),
            }
        }
        fn platform_unprovided(
            &mut self,
            _: &[u8],
            _: &mut Budget,
        ) -> std::result::Result<bool, Error> {
            panic!("this relation proof uses selected_from, not platform_unprovided")
        }
    }
    let bytes = array("java/lang/Object", &[]);
    let (_, request) = setup(vec![bytes.clone()]);
    let mut premise_budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.clone()), &mut premise_budget)
        .unwrap();
    let inspected = Engine::new()
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut premise_budget,
            InspectionMode::Strict,
        )
        .unwrap();
    let facts = jarde_reader::classfile::class_member_facts(&bytes, &mut premise_budget).unwrap();
    let header = ReferenceClassHeader::from_facts(
        &facts,
        LoaderId("app".into()),
        PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: inspected.source.class_bytes.clone(),
            variant: PhysicalVariant::Base,
        },
    )
    .unwrap();
    for cancel in [false, true] {
        let mut supplier = StopAtTerminal {
            header: header.clone(),
            cancel,
            reached_terminal: false,
        };
        let mut budget = Budget::new(limits());
        let result = proved_reference_widening(
            &request,
            b"java/util/ArrayList",
            b"java/util/List",
            &mut supplier,
            &mut budget,
        );
        assert!(
            supplier.reached_terminal,
            "the selected physical source reached missing terminal Object"
        );
        match result {
            Err(Error::Cancelled { .. }) if cancel => {}
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
                requested: 1,
                ..
            }) if !cancel => {}
            other => panic!("closed-loader proof did not propagate the exact stop: {other:?}"),
        }
    }
}
