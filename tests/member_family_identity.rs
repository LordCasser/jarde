use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Read, Write};

const FAMILY_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-26/named-member-family-stage1/fixture.jar"
);

fn report_with(limits: Limits) -> ClassSourceReport {
    report_from(FAMILY_JAR.to_vec(), limits)
}

fn report_from(bytes: Vec<u8>, limits: Limits) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = Budget::new(limits);
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .expect("frozen family jar opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("NamedMemberFamilyStage1"),
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
    match engine
        .class_source(std::slice::from_ref(&snapshot), &request, &mut budget)
        .expect("class-source returns a report")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected selection: {other:?}"),
    }
}

fn fixture_class(path: &[u8]) -> Vec<u8> {
    let archive = rawzip::ZipArchive::from_slice(FAMILY_JAR).unwrap();
    let mut entries = archive.entries();
    while let Some(header) = entries.next_entry().unwrap() {
        if header.file_path().as_ref() == path {
            let entry = archive.get_entry(header.wayfinder()).unwrap();
            let decoder = flate2::bufread::DeflateDecoder::new(entry.data());
            let mut reader = entry.verifying_reader(decoder);
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).unwrap();
            return bytes;
        }
    }
    panic!("class is present in frozen jar")
}

fn jar_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut zip = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in entries {
            let (mut entry, config) = zip
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(bytes).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        zip.finish().unwrap();
    }
    output.into_inner()
}

#[test]
fn selected_family_keeps_two_physical_reports_under_one_budget() {
    let report = report_with(task_limits(&[]).unwrap());
    let ClassSourceMemberFamily::Prepared {
        relation,
        child,
        capture,
    } = &report.member_family
    else {
        panic!("expected proved relation, got {:?}", report.member_family);
    };
    assert_eq!(relation.root, report.class);
    assert!(
        matches!(capture, ClassSourceMemberCapture::Proved { .. }),
        "{capture:?}"
    );
    let ClassSourceMemberCapture::Proved { proof } = capture else {
        unreachable!()
    };
    assert_eq!(proof.field_name, "this$0");
    assert_eq!(proof.field_index, 0);
    assert_eq!(proof.constructor.owner, child.class);
    assert_eq!(proof.write_bci, 2);
    assert_eq!(proof.reads.len(), 1);
    assert_eq!(proof.reads[0].method.owner, child.class);
    assert_eq!(proof.reads[0].bci, 8); // other.state at BCI 1 is not a capture read
    assert_eq!(relation.child, child.class);
    assert_eq!(relation.simple_name, "Member");
    assert_eq!(relation.access_flags & 0x0007, 0); // package-private InnerClasses row
    assert_ne!(report.class, child.class);
    assert!(!report.methods.is_empty());
    assert!(!child.methods.is_empty());
    assert!(
        report
            .methods
            .iter()
            .all(|method| method.item.identity.owner == report.class)
    );
    assert!(
        child
            .methods
            .iter()
            .all(|method| method.item.identity.owner == child.class)
    );
    assert!(
        report
            .methods
            .iter()
            .any(|root| child.methods.iter().any(|inner| {
                root.item.index == inner.item.index
                    && root.item.identity.owner != inner.item.identity.owner
            }))
    );
    assert!(report.usage.class_bytes >= child.usage.class_bytes);
    let execution_usage = match &report.execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Failed { usage, .. }
        | ExecutionReport::Cancelled { usage } => usage,
    };
    assert_eq!(&report.usage, execution_usage);
    assert!(report.text.contains("class NamedMemberFamilyStage1"));
    assert!(child.text.contains("class NamedMemberFamilyStage1$Member"));
}

#[test]
fn child_body_stop_keeps_root_and_child_physical_coverage() {
    let full = report_with(task_limits(&[]).unwrap());
    let mut limits = task_limits(&[]).unwrap();
    limits.method_bodies = u64::try_from(full.methods.len()).unwrap();
    let stopped = report_with(limits.clone());
    let ClassSourceMemberFamily::Refused {
        child: Some(child), ..
    } = &stopped.member_family
    else {
        panic!(
            "child identity should survive a body stop: {:?}",
            stopped.member_family
        );
    };
    assert_eq!(stopped.class, full.class);
    assert_eq!(stopped.methods.len(), full.methods.len());
    assert!(!child.methods.is_empty());
    assert!(matches!(stopped.execution, ExecutionReport::Partial { .. }));
    assert!(matches!(child.execution, ExecutionReport::Partial { .. }));
    assert_eq!(stopped.usage.method_bodies, limits.method_bodies);
    assert_eq!(child.usage.method_bodies, limits.method_bodies);
}

#[test]
fn capture_analysis_stop_keeps_completed_physical_child() {
    let full = report_with(task_limits(&[]).unwrap());
    let ClassSourceMemberFamily::Prepared {
        child: full_child, ..
    } = &full.member_family
    else {
        panic!("frozen physical family prepares")
    };
    assert!(full.usage.method_bodies > full_child.usage.method_bodies);
    let mut limits = task_limits(&[]).unwrap();
    limits.method_bodies = full_child.usage.method_bodies;
    let stopped = report_with(limits);
    let ClassSourceMemberFamily::Prepared { child, capture, .. } = &stopped.member_family else {
        panic!(
            "physical family survives capture stop: {:?}",
            stopped.member_family
        )
    };
    assert!(matches!(capture, ClassSourceMemberCapture::Refused { .. }));
    assert!(matches!(child.execution, ExecutionReport::Complete { .. }));
    assert!(matches!(stopped.execution, ExecutionReport::Partial { .. }));
    assert_eq!(child.class, full_child.class);
    assert_eq!(child.methods.len(), full_child.methods.len());
    assert!(stopped.text.contains("class NamedMemberFamilyStage1"));
}

#[test]
fn missing_or_duplicate_selected_child_refuses_family_without_losing_root() {
    let root = fixture_class(b"NamedMemberFamilyStage1.class");
    let child = fixture_class(b"NamedMemberFamilyStage1$Member.class");
    let missing = jar_of(&[(b"NamedMemberFamilyStage1.class", &root)]);
    let duplicate = jar_of(&[
        (b"NamedMemberFamilyStage1.class", &root),
        (b"NamedMemberFamilyStage1$Member.class", &child),
        (b"NamedMemberFamilyStage1$Member.class", &child),
    ]);
    for jar in [missing, duplicate] {
        let report = report_from(jar, task_limits(&[]).unwrap());
        assert!(matches!(
            report.member_family,
            ClassSourceMemberFamily::Refused { child: None, .. }
        ));
        assert!(!report.methods.is_empty());
        assert!(report.text.contains("class NamedMemberFamilyStage1"));
    }
}
