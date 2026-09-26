use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const FIXTURE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-26/named-member-family-stage1/fixture.jar"
);
static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "jarde-member-family-external-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture_class(path: &[u8]) -> Vec<u8> {
    let archive = rawzip::ZipArchive::from_slice(FIXTURE).unwrap();
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
    panic!(
        "class exists in frozen family fixture: {}",
        String::from_utf8_lossy(path)
    );
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

fn report(bytes: Vec<u8>) -> ClassSourceReport {
    report_with_limits(bytes, task_limits(&[]).unwrap())
}

fn report_with_limits(bytes: Vec<u8>, limits: Limits) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = Budget::new(limits);
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .expect("external-use fixture jar opens");
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
        .expect("class source produces a report")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected selection: {other:?}"),
    }
}

#[derive(Clone, Copy)]
enum ExternalReference {
    CaptureField,
    EnclosingConstructor,
}

fn report_with_external_use(reference: ExternalReference) -> ClassSourceReport {
    let temp = TestDirectory::new();
    let source_dir = temp.path().join("defpackage");
    std::fs::create_dir(&source_dir).unwrap();
    let source = match reference {
        ExternalReference::CaptureField => {
            "class NamedMemberFamilyStage1 {\n\
               static class Member {\n\
                 final NamedMemberFamilyStage1 this$0;\n\
                 Member(NamedMemberFamilyStage1 outer) { this.this$0 = outer; }\n\
               }\n\
             }\n\
             class ExternalUse {\n\
               static NamedMemberFamilyStage1 read(NamedMemberFamilyStage1.Member member) {\n\
                 return member.this$0;\n\
               }\n\
             }\n"
        }
        ExternalReference::EnclosingConstructor => {
            "class NamedMemberFamilyStage1 {\n\
               static class Member {\n\
                 Member(NamedMemberFamilyStage1 outer) {}\n\
               }\n\
             }\n\
             class ExternalUse {\n\
               static NamedMemberFamilyStage1.Member create(NamedMemberFamilyStage1 outer) {\n\
                 return new NamedMemberFamilyStage1.Member(outer);\n\
               }\n\
             }\n"
        }
    };
    let source = format!(
        "{source}\nclass VerifyExternalUse {{\n  public static void main(String[] args) throws Exception {{\n    Class.forName(\"ExternalUse\").getDeclaredMethods();\n  }}\n}}\n"
    );
    std::fs::write(source_dir.join("ExternalUse.java"), source).unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "defpackage/ExternalUse.java"])
        .current_dir(temp.path())
        .output()
        .expect("Java 8 javac is installed for the existing fixture tests");
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let external = std::fs::read(source_dir.join("ExternalUse.class")).unwrap();
    let javap = Command::new("javap")
        .args(["-verbose", "defpackage/ExternalUse.class"])
        .current_dir(temp.path())
        .output()
        .expect("javap is installed with javac");
    assert!(javap.status.success());
    let constant_pool = String::from_utf8_lossy(&javap.stdout);
    match reference {
        ExternalReference::CaptureField => assert!(
            constant_pool.contains("Fieldref")
                && constant_pool.contains("NamedMemberFamilyStage1$Member.this$0")
                && constant_pool.contains("LNamedMemberFamilyStage1;"),
            "ExternalUse must carry the exact capture-field Fieldref:\n{constant_pool}"
        ),
        ExternalReference::EnclosingConstructor => assert!(
            constant_pool.contains("Methodref")
                && constant_pool.contains("NamedMemberFamilyStage1$Member.\"<init>\"")
                && constant_pool.contains("(LNamedMemberFamilyStage1;)V"),
            "ExternalUse must carry the exact enclosing-argument constructor Methodref:\n{constant_pool}"
        ),
    }

    let root = fixture_class(b"NamedMemberFamilyStage1.class");
    let child = fixture_class(b"NamedMemberFamilyStage1$Member.class");
    let patched = jar_of(&[
        (b"NamedMemberFamilyStage1.class", &root),
        (b"NamedMemberFamilyStage1$Member.class", &child),
        (b"ExternalUse.class", &external),
    ]);
    let patched_path = temp.path().join("patched.jar");
    std::fs::write(&patched_path, &patched).unwrap();
    let classpath =
        std::env::join_paths([patched_path.as_os_str(), source_dir.as_os_str()]).unwrap();
    let verify = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(classpath)
        .arg("VerifyExternalUse")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "patched {reference_label} consumer must remain verifier-valid: {}",
        String::from_utf8_lossy(&verify.stderr),
        reference_label = match reference {
            ExternalReference::CaptureField => "capture field",
            ExternalReference::EnclosingConstructor => "constructor",
        }
    );
    report(patched)
}

fn assert_external_refusal(report: &ClassSourceReport, reference_label: &str) {
    let ClassSourceMemberFamily::Prepared {
        relation,
        child,
        projection,
        ..
    } = &report.member_family
    else {
        panic!(
            "external consumer must preserve the prepared physical family: {:?}",
            report.member_family
        );
    };
    assert_eq!(relation.root, report.class);
    assert_eq!(relation.simple_name, "Member");
    assert_ne!(report.class, child.class);
    assert_eq!(
        report.class.entry().unwrap().raw_name.0,
        b"NamedMemberFamilyStage1.class"
    );
    assert_eq!(
        child.class.entry().unwrap().raw_name.0,
        b"NamedMemberFamilyStage1$Member.class"
    );
    assert!(
        !report.methods.is_empty(),
        "root physical methods are retained"
    );
    assert!(
        !child.methods.is_empty(),
        "child physical methods are retained"
    );
    assert!(report.text.contains("class NamedMemberFamilyStage1"));
    assert!(child.text.contains("class NamedMemberFamilyStage1$Member"));
    let ClassSourceMemberProjection::Refused { reason } = projection else {
        panic!("external {reference_label} consumer must refuse family projection: {projection:?}");
    };
    assert!(
        reason.contains("ExternalUse"),
        "refusal must retain the external physical identity: {reason}"
    );
}

#[test]
fn external_capture_field_reference_refuses_projection_and_keeps_physical_family() {
    assert_external_refusal(
        &report_with_external_use(ExternalReference::CaptureField),
        "this$0 Fieldref",
    );
}

#[test]
fn external_enclosing_constructor_reference_refuses_projection_and_keeps_physical_family() {
    assert_external_refusal(
        &report_with_external_use(ExternalReference::EnclosingConstructor),
        "<init>(Outer) Methodref",
    );
}

#[test]
fn family_projects_when_no_external_consumer_is_present() {
    let report = report(FIXTURE.to_vec());
    assert!(
        matches!(
            report.member_family,
            ClassSourceMemberFamily::Prepared {
                projection: ClassSourceMemberProjection::Projected { .. },
                ..
            }
        ),
        "family with no external consumer should project: {:?}",
        report.member_family
    );
}

#[test]
fn stopped_external_census_keeps_physical_family_and_refuses_projection() {
    let mut limits = task_limits(&[]).unwrap();
    // Physical family preparation uses 116 entries; the two declaration scans need 50 more.
    limits.archive_entries = 150;
    let stopped = report_with_limits(FIXTURE.to_vec(), limits);
    let ClassSourceMemberFamily::Prepared {
        child,
        projection: ClassSourceMemberProjection::Refused { reason },
        ..
    } = &stopped.member_family
    else {
        panic!(
            "stopped census must retain the prepared physical family: {:?}",
            stopped.member_family
        );
    };
    assert!(reason.contains("external-use closure"), "{reason}");
    assert!(stopped.text.contains("class NamedMemberFamilyStage1"));
    assert!(child.text.contains("class NamedMemberFamilyStage1$Member"));
    assert!(!stopped.methods.is_empty());
    assert!(!child.methods.is_empty());
    assert!(!matches!(
        stopped.execution,
        ExecutionReport::Complete { .. }
    ));
}
