//! `recover-array-initializers`: one proven allocation/store chain becomes one ordered Java
//! initializer, while allocation shapes outside the proof remain on the ordinary array path.

use jarde::*;
use std::slice;

const FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-array-initializers/v8/ArrayInitializerProbe.class");
const NO_ARRAY_FIXTURE: &[u8] = include_bytes!("fixtures/p3-array-types/v8/ArrayTypes.class");
const MULTI_ARRAY_FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-deferred-value-order/v8/DeferredValueOrder.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed Java 8 fixture opens as a standalone CLASS")
}

fn request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    ClassSourceRequest {
        source_target: jarde::SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal("ArrayInitializerProbe"),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    }
}

fn method_request(
    snapshot: &ArtifactSnapshot,
    name: &[u8],
    descriptor: &[u8],
) -> MethodAnalysisRequest {
    let inspected = Engine::new()
        .inspect_header(
            snapshot,
            ClassTarget::Root,
            &mut budget(),
            InspectionMode::Strict,
        )
        .expect("the no-array fixture header is readable");
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let method = PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: inspected.source.class_bytes,
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
    };
    MethodAnalysisRequest {
        environment: ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
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
        method,
        stages: AnalysisStage::ALL.to_vec(),
    }
}

fn class_source(
    snapshot: &ArtifactSnapshot,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request(snapshot),
            evidence,
            budget,
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("the fixture answers one class-source request: {other:?}"),
    }
}

fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in ArrayInitializerProbe"))
}

fn recovery<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &member(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` has no recovery body: {other:?}"),
    }
}

#[test]
fn complete_initializer_chains_have_stable_bodies_and_claim_every_source() {
    let snapshot = open(FIXTURE);
    let essential = class_source(
        &snapshot,
        &RecoveryEvidenceRequest::essential(),
        &mut budget(),
    );
    let all = class_source(&snapshot, &RecoveryEvidenceRequest::all(), &mut budget());
    assert_eq!(
        essential.text, all.text,
        "evidence selection cannot change text"
    );

    for (name, expected, sources) in [
        (
            "literalInts",
            "return new int[]{1, 2, 3, -4};",
            &[
                0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20,
            ][..],
        ),
        (
            "literalStrings",
            "return new java.lang.String[]{\"left\", null, \"right\"};",
            &[0, 1, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 17, 18][..],
        ),
        (
            "effectfulInts",
            "return new int[]{intElement(arg0, 0), intElement(arg0, 1), intElement(arg0, 2)};",
            &[
                0, 1, 3, 4, 5, 6, 7, 10, 11, 12, 13, 14, 15, 18, 19, 20, 21, 22, 23, 26, 27,
            ][..],
        ),
        (
            "effectfulStrings",
            "return new java.lang.String[]{stringElement(arg0, 0), stringElement(arg0, 1), stringElement(arg0, 2)};",
            &[
                0, 1, 4, 5, 6, 7, 8, 11, 12, 13, 14, 15, 16, 19, 20, 21, 22, 23, 24, 27, 28,
            ][..],
        ),
    ] {
        let default_run = recovery(&essential, name);
        let all_run = recovery(&all, name);
        assert_eq!(default_run.text, all_run.text, "`{name}` body is stable");
        assert!(
            all_run.produced(),
            "`{name}` produced no artifact: {all_run:?}"
        );
        assert_eq!(all_run.representation, Representation::Java, "{name}");
        assert_eq!(all_run.quality, Quality::Structured, "{name}");
        assert!(
            all_run.text.contains(expected),
            "`{name}`:\n{}",
            all_run.text
        );
        assert!(
            !all_run.text.contains("@bytecode"),
            "`{name}` still quotes a source instruction:\n{}",
            all_run.text
        );
        assert!(
            default_run.source_map.is_empty(),
            "essential evidence leaves `{name}`'s source map unmaterialized"
        );
        for &bci in sources {
            assert!(
                !all_run.source_map.of_bci(bci).is_empty(),
                "`{name}` omitted initializer BCI {bci} from its source map: {:?}",
                all_run.source_map.segments()
            );
        }
    }

    for (name, expected) in [
        ("emptyInts", "return new int[0];"),
        ("emptyStrings", "return new java.lang.String[0];"),
    ] {
        let body = recovery(&all, name);
        assert!(body.text.contains(expected), "`{name}`:\n{}", body.text);
        assert!(
            !body.text.contains("[]{"),
            "empty ordinary allocation stays sized"
        );
    }
}

#[test]
fn an_exhausted_output_budget_does_not_publish_a_partial_class() {
    let snapshot = open(FIXTURE);
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot),
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(Limits {
                output_bytes: 0,
                ..task_limits(&[]).expect("the task defaults are bounded")
            }),
        )
        .expect("an exhausted output budget is an operation outcome");
    assert!(
        matches!(stopped, OperationOutcome::Incomplete(_)),
        "the whole class is withheld when its output cannot be committed: {stopped:?}"
    );
}

// The array pass's zero-cost early exit is tested directly in jarde-java. At the
// public boundary, derive the complete pipeline cost in this run instead of confusing
// unrelated proof costs with the array pass's cost. N/N-1 verifies a real enforced bound.
fn assert_tight_ir_boundary(bytes: &[u8], name: &[u8], descriptor: &[u8]) {
    let snapshot = open(bytes);
    let request = method_request(&snapshot, name, descriptor);
    for (evidence, with_map) in [
        (RecoveryEvidenceRequest::essential(), false),
        (RecoveryEvidenceRequest::all(), true),
    ] {
        let run = |budget: &mut Budget| {
            Engine::new()
                .recover_method_with_evidence(
                    slice::from_ref(&snapshot),
                    SourceTarget::JAVA_8,
                    &request,
                    &evidence,
                    budget,
                )
                .expect("the valid method request is answered")
        };
        let mut complete_budget = budget();
        let complete = run(&mut complete_budget);
        let body = complete.recovery();
        assert!(body.produced(), "{:?}", body.stop());
        assert!(matches!(body.execution, ExecutionReport::Complete { .. }));
        assert_eq!(body.quality, Quality::Structured);
        assert!(!body.text.contains("@bytecode"));
        let needed = complete_budget.usage().ir_items;
        assert!(needed > 0);
        let mut exact = Budget::new(Limits {
            ir_items: needed,
            ..task_limits(&[]).unwrap()
        });
        let at_limit = run(&mut exact);
        assert!(at_limit.recovery().produced());
        assert_eq!(exact.usage().ir_items, needed);
        assert_eq!(at_limit.recovery().text, body.text);
        assert_eq!(
            at_limit.recovery().source_map.segments(),
            body.source_map.segments()
        );
        if with_map {
            assert_eq!(
                body.evidence.state(RecoveryEvidenceKind::SourceMap),
                EvidenceState::Complete
            );
            assert_eq!(
                at_limit
                    .recovery()
                    .evidence
                    .state(RecoveryEvidenceKind::SourceMap),
                EvidenceState::Complete
            );
        }
        let mut short = Budget::new(Limits {
            ir_items: needed - 1,
            ..task_limits(&[]).unwrap()
        });
        let stopped = run(&mut short);
        let stopped = stopped.recovery();
        if with_map {
            assert!(
                stopped.produced(),
                "optional evidence cannot erase committed text"
            );
            assert_eq!(stopped.text, body.text);
            assert!(stopped.source_map.len() < body.source_map.len());
            assert!(matches!(
                stopped.evidence.state(RecoveryEvidenceKind::SourceMap),
                EvidenceState::Partial { .. }
            ));
            continue;
        }
        assert!(!stopped.produced());
        assert!(
            matches!(stopped.stop(), Some(StopReason::Budget {
            dimension: BudgetDimension::IrItems, limit, ..
        }) if *limit == needed - 1),
            "{:?}",
            stopped.stop()
        );
        assert!(matches!(
            stopped.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::IrItems
                },
                ..
            }
        ));
        assert!(stopped.text.is_empty());
        assert!(stopped.source_map.is_empty());
        assert!(short.usage().ir_items < needed);
    }
}

#[test]
fn a_method_without_array_allocations_keeps_its_tight_ir_budget() {
    assert_tight_ir_boundary(
        NO_ARRAY_FIXTURE,
        b"text",
        b"(Ljava/lang/String;)Ljava/lang/String;",
    );
}

#[test]
fn a_multidimensional_allocation_keeps_its_tight_ir_budget() {
    // The frozen method allocates both dimensions; its 0xc5 opcode is not a
    // one-dimensional initializer candidate regardless of how many dimensions it fills.
    assert_tight_ir_boundary(MULTI_ARRAY_FIXTURE, b"multiArray", b"(I)[[I");
}

#[test]
#[ignore = "requires a JDK to compile and run complete unedited Java 8 classes"]
fn complete_initializer_class_matches_original_jvm_trace() {
    use std::{fs, process::Command};
    struct TempDir(std::path::PathBuf);
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let path =
        std::env::temp_dir().join(format!("jarde-array-budget-oracle-{}", std::process::id()));
    fs::create_dir(&path).unwrap();
    let dir = TempDir(path);
    let snapshot = open(FIXTURE);
    let source = class_source(&snapshot, &RecoveryEvidenceRequest::all(), &mut budget());
    assert!(matches!(source.execution, ExecutionReport::Complete { .. }));
    let mut outputs = Vec::new();
    for generated in [false, true] {
        let path = dir.0.join(if generated { "generated" } else { "original" });
        fs::create_dir(&path).unwrap();
        fs::write(
            path.join("ArrayInitializerRunner.java"),
            include_str!("fixtures/p3-array-initializers/ArrayInitializerRunner.java"),
        )
        .unwrap();
        if generated {
            fs::write(path.join("ArrayInitializerProbe.java"), &source.text).unwrap();
        } else {
            fs::write(path.join("ArrayInitializerProbe.class"), FIXTURE).unwrap();
        }
        let mut compile = Command::new("javac");
        compile.current_dir(&path).args([
            "--release",
            "8",
            "-Xlint:-options",
            "-cp",
            ".",
            "-d",
            ".",
            "ArrayInitializerRunner.java",
        ]);
        if generated {
            compile.arg("ArrayInitializerProbe.java");
        }
        let compile = compile.output().expect("javac is available");
        assert!(
            compile.status.success(),
            "{}\n{}",
            source.text,
            String::from_utf8_lossy(&compile.stderr)
        );
        let run = Command::new("java")
            .current_dir(&path)
            .args(["-Xverify:all", "-cp", ".", "ArrayInitializerRunner"])
            .output()
            .expect("java is available");
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(run.stderr.is_empty());
        outputs.push(run.stdout);
    }
    assert_eq!(String::from_utf8_lossy(&outputs[0]).lines().count(), 14);
    assert_eq!(
        outputs[0], outputs[1],
        "all array values, effects and exception traces agree"
    );
}
