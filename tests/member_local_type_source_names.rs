//! A class-source member path may change a local declaration's Java spelling, but never its
//! semantic `Type` or its method-local source-map coordinates.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::slice;

const OUTER: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-27/outer-super-bridge-effects/OuterSuperEffects.class"
);
const MEMBER: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-27/outer-super-bridge-effects/OuterSuperEffects$Member.class"
);
const BASE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-27/outer-super-bridge-effects/EffectsBase.class"
);

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn family_jar() -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in [
            (b"OuterSuperEffects.class".as_slice(), OUTER),
            (b"OuterSuperEffects$Member.class".as_slice(), MEMBER),
            (b"EffectsBase.class".as_slice(), BASE),
        ] {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .expect("start one fixture entry");
            let mut writer = config.wrap(&mut entry);
            writer.write_all(bytes).expect("write fixture class");
            let (_, descriptor) = writer.finish().expect("finish fixture class");
            entry.finish(descriptor).expect("finish fixture entry");
        }
        archive.finish().expect("finish fixture JAR");
    }
    output.into_inner()
}

fn recover(evidence: &RecoveryEvidenceRequest) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(family_jar()), &mut budget())
        .expect("the frozen family JAR opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("OuterSuperEffects"),
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
            loader: LoaderId("app".to_owned()),
        },
    };
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            evidence,
            &mut budget(),
        )
        .expect("the frozen family answers one class-source request")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one frozen family answers one definition, got {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one frozen family has an incomplete selection with {} candidates",
            candidates.candidates.len()
        ),
    }
}

fn request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("OuterSuperEffects"),
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
            loader: LoaderId("app".to_owned()),
        },
    }
}

#[test]
fn proved_member_path_spells_local_declaration_without_changing_body_map() {
    let default = recover(&RecoveryEvidenceRequest::essential());
    let all = recover(&RecoveryEvidenceRequest::all());
    assert_eq!(
        default.text, all.text,
        "evidence selection does not alter source"
    );
    assert!(
        all.text
            .contains("OuterSuperEffects.Member member = outer.new Member();"),
        "the assembled root uses the selected nested source type:\n{}",
        all.text
    );
    assert!(
        all.text.contains("OuterSuperEffects.super.combine(")
            && all.text.contains("tick(1, failFirst)")
            && all.text.contains("tick(2, false)"),
        "the local-type projection leaves the frozen bridge body intact:\n{}",
        all.text
    );
    let ClassSourceMemberFamily::Prepared {
        projection: ClassSourceMemberProjection::Projected { derived },
        ..
    } = &all.member_family
    else {
        panic!(
            "the frozen family keeps its projected source ranges: {:?}",
            all.member_family
        );
    };
    assert!(
        derived
            .iter()
            .any(|projection| projection.kind == MemberFamilyDerivedKind::MemberConstruction),
        "the member construction remains linked to the physical method BCIs"
    );
}

#[test]
fn stopped_class_source_does_not_publish_partial_member_spelling() {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(family_jar()), &mut budget())
        .expect("the frozen family JAR opens");
    let request = request(&snapshot);
    let mut limits = budget().limits().clone();
    limits.output_bytes = 1;
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits),
        )
        .expect("an output budget stop is represented in the operation result");
    if let OperationOutcome::Performed(report) = stopped {
        assert!(
            !report.text.contains("OuterSuperEffects.Member member"),
            "a stopped output cannot publish the projected source declaration"
        );
        assert!(!matches!(
            report.execution,
            ExecutionReport::Complete { .. }
        ));
    }

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::with_cancellation_token(budget().limits().clone(), token),
        )
        .expect("cancellation is represented in the operation result");
    assert!(
        matches!(cancelled, OperationOutcome::Incomplete(_)),
        "pre-cancelled class-source publishes no partial class text"
    );
}

#[test]
fn standalone_method_recovery_does_not_inherit_family_source_aliases() {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(family_jar()), &mut budget())
        .expect("the frozen family JAR opens");
    let report = recover(&RecoveryEvidenceRequest::all());
    let main = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"main")
        .expect("the physical root declares main");
    let request = MethodAnalysisRequest {
        environment: {
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
            ResolutionEnvironment {
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
            }
        },
        method: main.item.identity.clone(),
        stages: report.stages.clone(),
    };
    let standalone = Engine::new()
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("the method's own physical identity answers the request");
    assert_eq!(standalone.analysis().method, main.item.identity);
    assert!(
        !standalone
            .recovery()
            .text
            .contains("OuterSuperEffects.Member member ="),
        "only family assembly supplies the selected source-path alias"
    );
}
