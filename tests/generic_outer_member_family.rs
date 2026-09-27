use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "jarde-generic-outer-family-{}-{}-{}",
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

fn compile_family(source: &str) -> Vec<u8> {
    let temp = TestDirectory::new();
    let package = temp.path().join("dt19");
    std::fs::create_dir(&package).unwrap();
    std::fs::write(package.join("Outer.java"), source).unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-Xlint:-options", "-d"])
        .arg(temp.path())
        .arg(package.join("Outer.java"))
        .output()
        .expect("Java 8 javac is installed for class-source tests");
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let mut output = Cursor::new(Vec::new());
    {
        let mut zip = ZipArchiveWriter::new(&mut output);
        for name in ["Outer.class", "Outer$Inner.class", "Outer$Extra.class"] {
            let path = temp.path().join("dt19").join(name);
            if !path.exists() {
                continue;
            }
            let bytes = std::fs::read(path).unwrap();
            let raw_name = format!("dt19/{name}").into_bytes();
            let (mut entry, config) = zip
                .new_file(EntryPath::verbatim(raw_name))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(&bytes).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        zip.finish().unwrap();
    }
    output.into_inner()
}

fn report_with_budget(bytes: Vec<u8>, budget: &mut Budget) -> ClassSourceReport {
    match outcome_with_budget(bytes, budget) {
        OperationOutcome::Performed(report) => report,
        other => panic!("Outer must resolve: {other:?}"),
    }
}

fn outcome_with_budget(bytes: Vec<u8>, budget: &mut Budget) -> OperationOutcome<ClassSourceReport> {
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut task_budget(&[]).unwrap())
        .expect("family archive opens independently of the class-source budget");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("dt19/Outer"),
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
    engine
        .class_source(std::slice::from_ref(&snapshot), &request, budget)
        .expect("class source returns its bounded outcome")
}

fn source(inner_make: &str, inner_body: &str, extra: &str) -> String {
    format!(
        "package dt19; public class Outer<T> {{ public class Inner {{ {inner_body} }} {inner_make} {extra} }}"
    )
}

fn bounded_source(inner_make: &str, inner_body: &str) -> String {
    format!(
        "package dt19; public class Outer<T extends Number> {{ public class Inner {{ {inner_body} }} {inner_make} }}"
    )
}

fn projected(report: &ClassSourceReport) -> bool {
    matches!(
        &report.member_family,
        ClassSourceMemberFamily::Prepared {
            calls: ClassSourceMemberCalls::Proved { .. },
            projection: ClassSourceMemberProjection::Projected { .. },
            ..
        }
    )
}

#[test]
fn generic_outer_member_family_projects_with_physical_owners_and_source_map() {
    let bytes = compile_family(&source(
        "public Inner make() { return new Inner(); }",
        "public T id(T value) { return value; }",
        "",
    ));
    let report = report_with_budget(bytes, &mut task_budget(&[]).unwrap());
    assert!(projected(&report), "{:?}", report.member_family);
    let ClassSourceMemberFamily::Prepared { child, .. } = &report.member_family else {
        unreachable!();
    };
    assert!(report.text.contains("public class Outer<T>"));
    assert!(report.text.contains("public dt19.Outer<T>.Inner make()"));
    assert!(report.text.contains("return new Inner();"));
    assert!(report.text.contains("public T id(T"));
    assert!(child.text.contains("public class Outer$Inner"));
    assert!(!child.text.contains("public T id(T"));
    let derived = match &report.member_family {
        ClassSourceMemberFamily::Prepared {
            projection: ClassSourceMemberProjection::Projected { derived },
            ..
        } => derived,
        _ => unreachable!(),
    };
    assert!(derived.iter().any(|item| {
        item.anchors.len() >= 2
            && item
                .anchors
                .iter()
                .any(|anchor| matches!(anchor, MemberFamilyPhysicalAnchor::MethodSignature { .. }))
    }));
}

#[test]
fn generic_family_rejects_static_constructor_shape_extra_child_and_effectful_id() {
    let static_make = report_with_budget(
        compile_family(&source(
            "public static Outer<?>.Inner make(Outer<?> outer) { return outer.new Inner(); }",
            "public T id(T value) { return value; }",
            "",
        )),
        &mut task_budget(&[]).unwrap(),
    );
    assert!(!projected(&static_make), "{:?}", static_make.member_family);

    let extra_child = report_with_budget(
        compile_family(&source(
            "public Inner make() { return new Inner(); }",
            "public T id(T value) { return value; }",
            "public class Extra {}",
        )),
        &mut task_budget(&[]).unwrap(),
    );
    assert!(!projected(&extra_child), "{:?}", extra_child.member_family);

    let effectful = report_with_budget(
        compile_family(&source(
            "public Inner make() { return new Inner(); }",
            "public T id(T value) { System.nanoTime(); return value; }",
            "",
        )),
        &mut task_budget(&[]).unwrap(),
    );
    assert!(!projected(&effectful), "{:?}", effectful.member_family);
    if let ClassSourceMemberFamily::Prepared {
        child, projection, ..
    } = &effectful.member_family
    {
        assert!(matches!(
            projection,
            ClassSourceMemberProjection::Refused { .. }
        ));
        assert!(child.text.contains("public class Outer$Inner"));
    }

    let mismatched_signature = report_with_budget(
        compile_family(&source(
            "public Inner make() { return new Inner(); }",
            "public String id(String value) { return value; }",
            "",
        )),
        &mut task_budget(&[]).unwrap(),
    );
    assert!(
        projected(&mismatched_signature),
        "a concrete child method does not inherit the outer T: {:?}",
        mismatched_signature.member_family
    );
    assert!(
        mismatched_signature
            .text
            .contains("java.lang.String id(java.lang.String"),
        "{}",
        mismatched_signature.text
    );

    let wrong_scope_erasure = report_with_budget(
        compile_family(&bounded_source(
            "public Inner make() { return new Inner(); }",
            "public T id(T value) { return value; }",
        )),
        &mut task_budget(&[]).unwrap(),
    );
    assert!(
        !projected(&wrong_scope_erasure),
        "{:?}",
        wrong_scope_erasure.member_family
    );

    let unproved_construction = report_with_budget(
        compile_family(&source(
            "public Inner make() { return null; }",
            "public T id(T value) { return value; }",
            "",
        )),
        &mut task_budget(&[]).unwrap(),
    );
    assert!(
        !projected(&unproved_construction),
        "{:?}",
        unproved_construction.member_family
    );

    let extra_allocation = report_with_budget(
        compile_family(&source(
            "public Inner make() { new Object(); return new Inner(); }",
            "public T id(T value) { return value; }",
            "",
        )),
        &mut task_budget(&[]).unwrap(),
    );
    assert!(
        !projected(&extra_allocation),
        "{:?}",
        extra_allocation.member_family
    );
}

#[test]
fn stopped_generic_family_projection_keeps_physical_reports_without_partial_nested_source() {
    let bytes = compile_family(&source(
        "public Inner make() { return new Inner(); }",
        "public T id(T value) { return value; }",
        "",
    ));
    let complete = report_with_budget(bytes.clone(), &mut task_budget(&[]).unwrap());
    let complete_usage = complete.usage.clone();
    assert!(projected(&complete));
    let mut limits = task_limits(&[]).unwrap();
    limits.output_bytes = complete_usage.output_bytes.saturating_sub(1);
    let stopped = report_with_budget(bytes.clone(), &mut Budget::new(limits));
    assert!(!matches!(
        stopped.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(!matches!(
        stopped.member_family,
        ClassSourceMemberFamily::Prepared {
            projection: ClassSourceMemberProjection::Projected { .. },
            ..
        }
    ));
    assert!(!stopped.text.contains("class Inner"));

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let mut cancelled = Budget::with_cancellation_token(task_limits(&[]).unwrap(), cancellation);
    assert!(matches!(
        outcome_with_budget(bytes, &mut cancelled),
        OperationOutcome::Incomplete { .. }
    ));
}
