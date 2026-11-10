//! MethodHandle references preserve raw identities through spelling and closed helper projection.
use jarde::*;
use jarde_jvm::ir::Quality;

fn recovered(bytes: &[u8], evidence: RecoveryEvidenceRequest) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal("Subject"),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".into()),
        },
    };
    match engine
        .class_source_with_evidence(&[snapshot], &request, &evidence, &mut budget)
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source outcome: {other:?}"),
    }
}

fn body<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    let raw = jarde_reader::model::JvmBytes::from_unicode(name);
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw() == &raw)
        .unwrap();
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("the named member was not recovered: {other:?}"),
    }
}

fn equivalent(bytes: &[u8], check: impl Fn(&ClassSourceReport)) {
    let mut text = None;
    for evidence in [
        RecoveryEvidenceRequest::essential(),
        RecoveryEvidenceRequest::all(),
    ] {
        let report = recovered(bytes, evidence);
        check(&report);
        if let Some(previous) = text {
            assert_eq!(report.text, previous);
        }
        text = Some(report.text);
    }
}

fn source_code(report: &ClassSourceReport) -> String {
    report
        .text
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
#[cfg(feature = "test-support")]
fn captured_this_helper_uses_the_same_complete_class_census() {
    let input = include_bytes!("fixtures/s1-lambda-handle-names/captured-this-ascii.jar");
    let mut previous = None;
    for (selection, evidence) in [
        ("essential", RecoveryEvidenceRequest::essential()),
        ("all", RecoveryEvidenceRequest::all()),
    ] {
        let reader_before = jarde_reader::observation::snapshot();
        let ir_before = jarde_jvm::observation::snapshot();
        let report = recovered(input, evidence);
        let reader_after = jarde_reader::observation::snapshot();
        let ir = ir_before.since(jarde_jvm::observation::snapshot());
        println!(
            "S1_HELPER_COUNTS {}",
            serde_json::json!({
                "selection": selection,
                "read_attempts": reader_after.read_attempts - reader_before.read_attempts,
                "read_bytes": reader_after.read_bytes - reader_before.read_bytes,
                "body_decodes": reader_after.body_decode_attempts - reader_before.body_decode_attempts,
                "ir_starts": ir.starts,
                "ir_completions": ir.completions,
            })
        );
        assert!(
            report.text.contains("omitted physical lambda helper"),
            "{}",
            report.text
        );
        assert!(
            !source_code(&report).contains("lambda$run$0"),
            "{}",
            report.text
        );
        assert_eq!(body(&report, "run").quality, Quality::Structured);
        assert!(
            report
                .methods
                .iter()
                .any(|method| method.item.name.raw().0 == b"lambda$run$0"
                    && method.item.access_flags == 0x1002)
        );
        if let Some(text) = previous {
            assert_eq!(report.text, text);
        }
        previous = Some(report.text);
    }
}

#[test]
fn named_method_handles_keep_legal_names_and_constructor_new() {
    for input in [
        include_bytes!("fixtures/s1-lambda-handle-names/static-reference-ascii.jar").as_slice(),
        include_bytes!("fixtures/s1-lambda-handle-names/static-reference-bmp.jar"),
        include_bytes!("fixtures/s1-lambda-handle-names/static-reference-supplementary.jar"),
        include_bytes!("fixtures/s1-lambda-handle-names/static-reference-underscore.jar"),
        include_bytes!("fixtures/s1-lambda-handle-names/unbound-lambda-ascii.jar"),
        include_bytes!("fixtures/s1-lambda-handle-names/unbound-lambda-bmp.jar"),
        include_bytes!("fixtures/s1-lambda-handle-names/unbound-lambda-supplementary.jar"),
        include_bytes!("fixtures/s1-lambda-handle-names/unbound-lambda-underscore.jar"),
        include_bytes!("fixtures/s1-lambda-handle-names/constructor-reference.jar"),
    ] {
        equivalent(input, |report| {
            assert_eq!(body(report, "run").quality, Quality::Structured)
        });
    }
}

fn refuses(inputs: &[&[u8]], raw: &[u8]) {
    for input in inputs {
        equivalent(input, |report| {
            let run = body(report, "run");
            assert_eq!(run.quality, Quality::Fallback, "{}", run.text);
            assert!(run.text.contains("raw method name"), "{}", run.text);
            assert!(run.text.contains(&format!("{raw:?}")), "{}", run.text);
            assert!(run.lambdas.iter().all(|site| !site.presented()));
        });
    }
}

#[test]
fn named_method_handle_calls_refuse_unspellable_names() {
    refuses(
        &[
            include_bytes!("fixtures/s1-lambda-handle-names/static-reference-dash.jar"),
            include_bytes!("fixtures/s1-lambda-handle-names/unbound-lambda-dash.jar"),
        ],
        b"bad-name",
    );
}

#[test]
fn named_method_handle_calls_refuse_surrogates_without_diagnostic_tokens() {
    refuses(
        &[
            include_bytes!("fixtures/s1-lambda-handle-names/static-reference-surrogate.jar"),
            include_bytes!("fixtures/s1-lambda-handle-names/unbound-lambda-surrogate.jar"),
        ],
        &[0xed, 0xa0, 0x80],
    );
}

#[test]
fn helper_projection_matches_supplementary_physical_names() {
    for (input, factory, marker) in [
        (
            include_bytes!("fixtures/s1-lambda-handle-names/helper-inline-ascii.jar").as_slice(),
            "Xcall",
            "omitted physical lambda helper",
        ),
        (
            include_bytes!("fixtures/s1-lambda-handle-names/helper-inline-bmp.jar"),
            "café",
            "omitted physical lambda helper",
        ),
        (
            include_bytes!("fixtures/s1-lambda-handle-names/helper-inline-supplementary.jar"),
            "𐐀",
            "omitted physical lambda helper",
        ),
        (
            include_bytes!("fixtures/s1-lambda-handle-names/helper-inline-underscore.jar"),
            "_",
            "omitted physical lambda helper",
        ),
        (
            include_bytes!("fixtures/s1-lambda-handle-names/helper-rename-ascii.jar"),
            "Xcall",
            "renamed physical lambda helper",
        ),
        (
            include_bytes!("fixtures/s1-lambda-handle-names/helper-rename-bmp.jar"),
            "café",
            "renamed physical lambda helper",
        ),
        (
            include_bytes!("fixtures/s1-lambda-handle-names/helper-rename-supplementary.jar"),
            "𐐀",
            "renamed physical lambda helper",
        ),
        (
            include_bytes!("fixtures/s1-lambda-handle-names/helper-rename-underscore.jar"),
            "_",
            "renamed physical lambda helper",
        ),
    ] {
        equivalent(input, |report| {
            assert!(report.text.contains(marker), "{}", report.text);
            assert_eq!(body(report, factory).quality, Quality::Structured);
            assert!(!report.text.contains('\u{fffd}'), "{}", report.text);
        });
    }
}

#[test]
fn closed_helper_rename_uses_the_declarations_alias_and_keeps_raw_identity() {
    for (input, raw) in [
        (
            include_bytes!("fixtures/s1-lambda-handle-names/helper-rename-dash.jar").as_slice(),
            b"lambda$bad-name".as_slice(),
        ),
        (
            include_bytes!("fixtures/s1-lambda-handle-names/helper-rename-surrogate.jar"),
            b"lambda$\xed\xa0\x80".as_slice(),
        ),
        (
            include_bytes!("fixtures/s1-lambda-handle-names/helper-inline-surrogate.jar"),
            b"lambda$\xed\xa0\x80".as_slice(),
        ),
    ] {
        equivalent(input, |report| {
            assert!(
                report
                    .methods
                    .iter()
                    .any(|method| method.item.name.raw().0 == raw)
            );
            assert!(
                report.text.contains("renamed physical lambda helper"),
                "{}",
                report.text
            );
            assert!(report.text.contains("$jarde("), "{}", report.text);
            assert!(!source_code(report).contains("bad-name("));
            assert!(!source_code(report).contains("\\uD800("));
        });
    }
}

#[test]
fn an_unspellable_physical_helper_can_still_be_completely_inlined() {
    equivalent(
        include_bytes!("fixtures/s1-lambda-handle-names/helper-inline-dash.jar"),
        |report| {
            assert!(
                report.text.contains("omitted physical lambda helper"),
                "{}",
                report.text
            );
            assert!(source_code(report).contains("arg0 + 3"));
            assert!(!source_code(report).contains("bad-name"));
        },
    );
}

#[test]
fn a_generated_prefix_does_not_authorize_an_unclosed_source_reference() {
    refuses(
        &[
            include_bytes!("fixtures/s1-lambda-handle-names/helper-nonprivate.jar"),
            include_bytes!("fixtures/s1-lambda-handle-names/helper-nonsynthetic.jar"),
            include_bytes!("fixtures/s1-lambda-handle-names/instance-nonprivate.jar"),
            include_bytes!("fixtures/s1-lambda-handle-names/instance-nonsynthetic.jar"),
            include_bytes!("fixtures/s1-lambda-handle-names/external-generated-dash.jar"),
        ],
        b"lambda$bad-name",
    );
}

#[test]
fn rejected_staged_aliases_do_not_publish_raw_invalid_calls() {
    equivalent(
        include_bytes!("fixtures/s1-lambda-handle-names/staged-alias-collision.jar"),
        |report| {
            let source = source_code(report);
            assert!(!source.contains("lambda$x-y("), "{source}");
            assert!(!source.contains("lambda$x?y("), "{source}");
        },
    );
}

#[test]
fn a_method_entry_requires_its_own_closed_source_name() {
    let input = include_bytes!("fixtures/s1-lambda-handle-names/helper-inline-dash.jar");
    let class = recovered(input, RecoveryEvidenceRequest::all());
    assert!(class.text.contains("omitted physical lambda helper"));
    let member = class
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"run")
        .unwrap();
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(ArtifactInput::bytes(input.to_vec()), &mut budget)
        .unwrap();
    let domain = LoadDomain {
        loader: LoaderId("app".into()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Container {
            origin: ContainerOrigin {
                snapshot: snapshot.id().clone(),
                root_container: ContainerId("root".into()),
                steps: Vec::new(),
            },
            prefix: ArchiveNameBytes(Vec::new()),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let request = MethodAnalysisRequest {
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
        method: member.item.identity.clone(),
        stages: AnalysisStage::ALL.to_vec(),
    };
    let recovered = engine
        .recover_method_with_evidence(
            &[snapshot],
            SourceTarget::JAVA_8,
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .unwrap();
    let body = recovered.recovery();
    assert_eq!(body.quality, Quality::Fallback, "{}", body.text);
    assert!(body.text.contains("raw method name"));
    assert!(body.lambdas.iter().all(|site| !site.presented()));
    assert_eq!(
        body.text,
        match &member.outcome {
            ClassSourceOutcome::Recovered { report, .. } => &report.text,
            other => panic!("missing physical method report: {other:?}"),
        }
        .as_str()
    );
}

#[test]
fn a_whole_method_quote_does_not_claim_to_present_its_legal_sibling_site() {
    equivalent(
        include_bytes!("fixtures/s1-lambda-handle-names/discarded-legal-site.jar"),
        |report| {
            let run = body(report, "run");
            assert_eq!(run.quality, Quality::Fallback);
            assert!(
                run.text
                    .contains("complete same-run method instruction envelope")
            );
            assert!(run.lambdas.iter().all(|site| !site.presented()));
            assert!(run.lambdas.iter().all(|site| site.refusal.is_some()));
        },
    );
}
