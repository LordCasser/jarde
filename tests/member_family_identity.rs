use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "jarde-member-family-{}-{}-{}",
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

const FAMILY_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-26/named-member-family-stage1/fixture.jar"
);

fn report_with(limits: Limits) -> ClassSourceReport {
    report_from(FAMILY_JAR.to_vec(), limits)
}

fn report_from(bytes: Vec<u8>, limits: Limits) -> ClassSourceReport {
    report_from_named(bytes, "NamedMemberFamilyStage1", limits)
}

fn report_from_named(bytes: Vec<u8>, name: &str, limits: Limits) -> ClassSourceReport {
    report_from_named_with_evidence(bytes, name, limits, &RecoveryEvidenceRequest::essential())
}

fn report_from_named_with_evidence(
    bytes: Vec<u8>,
    name: &str,
    limits: Limits,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = Budget::new(limits);
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .expect("frozen family jar opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
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
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            evidence,
            &mut budget,
        )
        .expect("class-source returns a report")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected selection: {other:?}"),
    }
}

#[test]
fn family_derived_ranges_keep_exact_source_and_physical_owners() {
    let mut essential_text_and_ranges = None;
    for evidence in [
        RecoveryEvidenceRequest::essential(),
        RecoveryEvidenceRequest::all(),
    ] {
        let report = report_from_named_with_evidence(
            FAMILY_JAR.to_vec(),
            "NamedMemberFamilyStage1",
            task_limits(&[]).unwrap(),
            &evidence,
        );
        let ClassSourceMemberFamily::Prepared {
            child,
            capture: ClassSourceMemberCapture::Proved { proof },
            calls: ClassSourceMemberCalls::Proved { sites },
            projection: ClassSourceMemberProjection::Projected { derived },
            ..
        } = &report.member_family
        else {
            panic!(
                "family must project with either evidence selection: {:?}",
                report.member_family
            )
        };
        assert_eq!(derived.len(), 5);
        if let Some((text, ranges)) = &essential_text_and_ranges {
            assert_eq!(&report.text, text);
            assert_eq!(derived, ranges);
        } else {
            essential_text_and_ranges = Some((report.text.clone(), derived.clone()));
        }
        for entry in derived {
            assert!(entry.start < entry.end && entry.end <= report.text.len());
            assert!(report.text.is_char_boundary(entry.start));
            assert!(report.text.is_char_boundary(entry.end));
            assert!(!report.text[entry.start..entry.end].is_empty());
            assert!(!entry.anchors.is_empty());
        }
        let by_kind = |kind| derived.iter().find(|entry| entry.kind == kind).unwrap();
        let construction = by_kind(MemberFamilyDerivedKind::MemberConstruction);
        assert_eq!(
            &report.text[construction.start..construction.end],
            "new Member"
        );
        assert!(matches!(
            &construction.anchors[0],
            MemberFamilyPhysicalAnchor::MethodPoint { method, bci }
                if method == &sites[0].caller && *bci == 27 && method.owner == report.class
        ));
        assert!(matches!(
            &construction.anchors[2],
            MemberFamilyPhysicalAnchor::ConstructorParameter { method, index }
                if method == &proof.constructor && *index == 0 && method.owner == child.class
        ));
        let read = by_kind(MemberFamilyDerivedKind::CapturedOuterRead);
        assert_eq!(
            &report.text[read.start..read.end],
            "NamedMemberFamilyStage1.this"
        );
        assert!(matches!(
            &read.anchors[0],
            MemberFamilyPhysicalAnchor::MethodPoint { method, bci }
                if method == &proof.reads[0].method && *bci == 8 && method.owner == child.class
        ));
        let field = by_kind(MemberFamilyDerivedKind::HiddenCaptureField);
        assert!(report.text[field.start..field.end].contains("class Member extends"));
        assert!(matches!(
            &field.anchors[0],
            MemberFamilyPhysicalAnchor::Field { field, index }
                if field.owner == child.class && *index == proof.field_index
        ));
        for kind in [
            MemberFamilyDerivedKind::HiddenConstructorParameter,
            MemberFamilyDerivedKind::HiddenCaptureWrite,
        ] {
            let entry = by_kind(kind);
            assert!(report.text[entry.start..entry.end].contains("Member() {"));
        }
        assert!(matches!(
            &by_kind(MemberFamilyDerivedKind::HiddenCaptureWrite).anchors[0],
            MemberFamilyPhysicalAnchor::MethodPoint { method, bci }
                if method == &proof.constructor && *bci == proof.write_bci
        ));
        let serialized = serde_json::to_value(&report).unwrap();
        assert_eq!(
            serialized["member_family"]["projection"]["state"],
            "projected"
        );
        assert_eq!(
            serialized["member_family"]["projection"]["derived"]
                .as_array()
                .unwrap()
                .len(),
            5
        );
        assert_eq!(
            serialized["member_family"]["child"]["class"],
            serde_json::to_value(&child.class).unwrap()
        );
        if evidence == RecoveryEvidenceRequest::all() {
            let root_ctor = report
                .methods
                .iter()
                .find(|method| method.item.identity.name.0 == b"<init>")
                .unwrap();
            let child_ctor = child
                .methods
                .iter()
                .find(|method| method.item.identity.name.0 == b"<init>")
                .unwrap();
            let (
                ClassSourceOutcome::Recovered {
                    report: root_recovery,
                    ..
                },
                ClassSourceOutcome::Recovered {
                    report: child_recovery,
                    ..
                },
            ) = (&root_ctor.outcome, &child_ctor.outcome)
            else {
                panic!("constructors recovered")
            };
            assert!(!root_recovery.source_map.of_bci(0).is_empty());
            assert!(!child_recovery.source_map.of_bci(0).is_empty());
            assert_ne!(
                root_ctor.item.identity.owner,
                child_ctor.item.identity.owner
            );
            assert!(root_recovery.source_map.of_bci(0).iter().any(|segment| {
                segment.origin().primary().method() == Some(&root_ctor.item.identity)
            }));
            assert!(child_recovery.source_map.of_bci(0).iter().any(|segment| {
                segment.origin().primary().method() == Some(&child_ctor.item.identity)
            }));
            assert_eq!(
                serialized["methods"][root_ctor.item.index as usize]["outcome"]["report"]["source_map"],
                serde_json::to_value(&root_recovery.source_map).unwrap()
            );
            assert_eq!(
                serialized["member_family"]["child"]["methods"][child_ctor.item.index as usize]["outcome"]
                    ["report"]["source_map"],
                serde_json::to_value(&child_recovery.source_map).unwrap()
            );
        }
    }
}

#[test]
fn two_member_allocations_in_one_caller_get_distinct_source_ranges() {
    let temp = TestDirectory::new();
    std::fs::write(
        temp.path().join("MultiCall.java"),
        "class MultiCall { class Member { Member() {} int value() { return 1; } } int run(MultiCall outer) { return outer.new Member().value() + outer.new Member().value(); } }",
    )
    .unwrap();
    let output = Command::new("javac")
        .args(["--release", "8", "-g:none", "MultiCall.java"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let root = std::fs::read(temp.path().join("MultiCall.class")).unwrap();
    let child = std::fs::read(temp.path().join("MultiCall$Member.class")).unwrap();
    let report = report_from_named(
        jar_of(&[
            (b"MultiCall.class", &root),
            (b"MultiCall$Member.class", &child),
        ]),
        "MultiCall",
        task_limits(&[]).unwrap(),
    );
    let ClassSourceMemberFamily::Prepared {
        calls: ClassSourceMemberCalls::Proved { sites },
        projection: ClassSourceMemberProjection::Projected { derived },
        ..
    } = &report.member_family
    else {
        let reason = match &report.member_family {
            ClassSourceMemberFamily::Prepared { projection, .. } => format!("{projection:?}"),
            other => format!("{other:?}"),
        };
        panic!("two-call family did not project: {reason}")
    };
    assert_eq!(sites.len(), 2);
    assert_eq!(sites[0].caller, sites[1].caller);
    assert_ne!(sites[0].allocation_bci, sites[1].allocation_bci);
    let constructions: Vec<_> = derived
        .iter()
        .filter(|entry| entry.kind == MemberFamilyDerivedKind::MemberConstruction)
        .collect();
    assert_eq!(constructions.len(), 2);
    assert_ne!(constructions[0].start, constructions[1].start);
    for (entry, site) in constructions.into_iter().zip(sites) {
        assert_eq!(&report.text[entry.start..entry.end], "new Member");
        assert!(
            matches!(&entry.anchors[0], MemberFamilyPhysicalAnchor::MethodPoint { method, bci } if method == &site.caller && *bci == site.allocation_bci)
        );
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
        calls,
        ..
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
    let ClassSourceMemberCalls::Proved { sites } = calls else {
        panic!("frozen family call is refused: {calls:?}");
    };
    assert_eq!(sites.len(), 1);
    assert_eq!(sites[0].caller.owner, report.class);
    assert_eq!(sites[0].allocation_bci, 27);
    assert_eq!(sites[0].null_check_bci, 33);
    assert_eq!(sites[0].constructor_bci, 37);
    assert_eq!(sites[0].constructor, proof.constructor);
    assert!(sites[0].ordinary_argument_bcis.is_empty());
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
    assert!(
        matches!(
            report.member_family,
            ClassSourceMemberFamily::Prepared {
                projection: ClassSourceMemberProjection::Projected { .. },
                ..
            }
        ),
        "{:?}",
        match &report.member_family {
            ClassSourceMemberFamily::Prepared { projection, .. } => Some(projection),
            _ => None,
        }
    );
    assert!(report.text.contains("outer.new Member()"));
    assert!(report.text.contains("NamedMemberFamilyStage1.this"));
    assert!(report.text.contains("other.state"));
    assert!(!report.text.contains("this.this$0"));
    assert_eq!(report.text.matches("class Member extends").count(), 1);
    assert_eq!(report.text.matches("Member() {").count(), 1);
    assert_eq!(report.text.matches("int read(").count(), 1);
    assert_eq!(report.text.matches("static int access$000(").count(), 1);
    assert!(!report.text.contains("NamedMemberFamilyStage1$Member("));
    assert!(child.text.contains("class NamedMemberFamilyStage1$Member"));
}

fn compile_and_verify(source: &str) -> String {
    let temp = TestDirectory::new();
    std::fs::write(temp.path().join("NamedMemberFamilyStage1.java"), source).unwrap();
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "NamedMemberFamilyStage1.java"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp", ".", "NamedMemberFamilyStage1"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).unwrap()
}

#[test]
fn projected_stage1_compiles_and_matches_original_and_jadx_java8_behavior() {
    let report = report_with(task_limits(&[]).unwrap());
    assert!(matches!(
        report.member_family,
        ClassSourceMemberFamily::Prepared {
            projection: ClassSourceMemberProjection::Projected { .. },
            ..
        }
    ));
    let original = include_str!(
        "../openspec/evidence/java-syntax-2026-09-26/named-member-family-stage1/NamedMemberFamilyStage1.java"
    );
    let jadx = include_str!(
        "../openspec/evidence/java-syntax-2026-09-26/named-member-family-stage1/recompiled-jadx/NamedMemberFamilyStage1.java"
    );
    for source in [original, jadx, &report.text] {
        assert_eq!(compile_and_verify(source), "2011\n20\n");
    }
    let temp = TestDirectory::new();
    let jar = temp.path().join("fixture.jar");
    std::fs::write(&jar, FAMILY_JAR).unwrap();
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&jar)
        .arg("NamedMemberFamilyStage1")
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8(run.stdout).unwrap(), "2011\n20\n");
}

#[test]
fn family_call_rerender_restores_effect_and_null_exception_order() {
    const SOURCE: &str = r#"public class FamilyEffects {
    static int effects;
    static int tick(){ effects=effects+1; return effects; }
    class Member { Member(int unused){} int value(){ return effects; } }
    static int run(FamilyEffects outer){ return outer.new Member(tick()).value(); }
}
"#;
    const RUNNER: &str = r#"public class FamilyEffectsRunner {
    public static void main(String[] args) {
        FamilyEffects.effects = 0;
        System.out.println(FamilyEffects.run(new FamilyEffects()) + ":" + FamilyEffects.effects);
        try { FamilyEffects.run(null); System.out.println("missing NPE"); }
        catch (NullPointerException expected) { System.out.println("NPE:" + FamilyEffects.effects); }
    }
}
"#;
    fn compile(dir: &TestDirectory) {
        let output = Command::new("javac")
            .args([
                "--release",
                "8",
                "-g",
                "FamilyEffects.java",
                "FamilyEffectsRunner.java",
            ])
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fn run(dir: &TestDirectory) -> String {
        let output = Command::new("java")
            .args(["-Xverify:all", "-cp", ".", "FamilyEffectsRunner"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }
    let original = TestDirectory::new();
    std::fs::write(original.path().join("FamilyEffects.java"), SOURCE).unwrap();
    std::fs::write(original.path().join("FamilyEffectsRunner.java"), RUNNER).unwrap();
    compile(&original);
    let root = std::fs::read(original.path().join("FamilyEffects.class")).unwrap();
    let child = std::fs::read(original.path().join("FamilyEffects$Member.class")).unwrap();
    let report = report_from_named(
        jar_of(&[
            (b"FamilyEffects.class", &root),
            (b"FamilyEffects$Member.class", &child),
        ]),
        "FamilyEffects",
        task_limits(&[]).unwrap(),
    );
    let ClassSourceMemberFamily::Prepared {
        child, projection, ..
    } = &report.member_family
    else {
        panic!("effect family relation is not prepared");
    };
    assert!(
        matches!(projection, ClassSourceMemberProjection::Projected { .. }),
        "{projection:?}"
    );
    let run_method = report
        .methods
        .iter()
        .find(|method| method.item.identity.name.0 == b"run")
        .unwrap();
    assert_eq!(run_method.markers.len(), 1);
    assert!(run_method.markers[0].contains("produced no statement"));
    assert!(
        !report
            .text
            .contains("not recovered: the recovery run for `run")
    );
    assert!(report.text.contains("outer.new Member(tick())"));
    assert!(child.text.contains("this$0"));
    let projected = TestDirectory::new();
    std::fs::write(projected.path().join("FamilyEffects.java"), &report.text).unwrap();
    std::fs::write(projected.path().join("FamilyEffectsRunner.java"), RUNNER).unwrap();
    compile(&projected);
    assert_eq!(run(&original), "1:1\nNPE:1\n");
    assert_eq!(run(&projected), run(&original));
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

#[test]
fn member_call_without_early_null_check_retains_physical_source() {
    // This frozen variant replaces main's three-byte requireNonNull call with three nops;
    // the following pop keeps the operand stack balanced, and -Xverify:all accepts the jar.
    let report = report_from(
        include_bytes!(
            "../openspec/evidence/java-syntax-2026-09-26/named-member-family-calls/null-check-removed.jar"
        )
        .to_vec(),
        task_limits(&[]).unwrap(),
    );
    let ClassSourceMemberFamily::Prepared {
        child: child_report,
        capture: ClassSourceMemberCapture::Proved { .. },
        calls: ClassSourceMemberCalls::Refused {
            sites, refusals, ..
        },
        ..
    } = &report.member_family
    else {
        panic!(
            "null-check removal must refuse only the call: {:?}",
            report.member_family
        );
    };
    assert!(sites.is_empty());
    assert_eq!(refusals.len(), 1);
    assert_eq!(refusals[0].allocation_bci, 27);
    assert_eq!(refusals[0].caller.owner, report.class);
    assert!(report.text.contains("class NamedMemberFamilyStage1"));
    assert!(report.methods.iter().any(|method| {
        method.item.identity.name.0 == b"main"
            && matches!(method.outcome, ClassSourceOutcome::Recovered { .. })
    }));
    assert!(child_report.text.contains("this$0"));
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    let serialized = serde_json::to_value(&report).unwrap();
    assert_eq!(
        serialized["member_family"]["projection"]["state"],
        "refused"
    );
    assert!(
        serialized["member_family"]["projection"]
            .get("derived")
            .is_none()
    );
}

#[test]
fn qualified_member_call_preserves_internal_and_preceding_effect_positions() {
    let report = report_from_named(
        include_bytes!(
            "../openspec/evidence/java-syntax-2026-09-26/named-member-family-calls/qualified-effects.jar"
        )
        .to_vec(),
        "NamedMemberFamilyCalls",
        task_limits(&[]).unwrap(),
    );
    let ClassSourceMemberFamily::Prepared {
        child,
        capture: ClassSourceMemberCapture::Proved { proof },
        calls: ClassSourceMemberCalls::Proved { sites },
        ..
    } = &report.member_family
    else {
        panic!(
            "effect fixture must prove both calls: {:?}",
            report.member_family
        );
    };
    assert_eq!(child.class, proof.constructor.owner);
    assert_eq!(sites.len(), 2);
    let internal = sites
        .iter()
        .find(|site| site.caller.name.0 == b"internal")
        .unwrap();
    assert_eq!((internal.allocation_bci, internal.null_check_bci), (0, 6));
    assert_eq!(internal.ordinary_argument_bcis, [13]);
    let before = sites
        .iter()
        .find(|site| site.caller.name.0 == b"before")
        .unwrap();
    assert_eq!((before.allocation_bci, before.null_check_bci), (7, 13));
    assert_eq!(before.ordinary_argument_bcis, [17]);
    let projection = match &report.member_family {
        ClassSourceMemberFamily::Prepared { projection, .. } => projection,
        _ => unreachable!(),
    };
    assert!(
        matches!(projection, ClassSourceMemberProjection::Refused { .. }),
        "{projection:?}"
    );
    assert!(!report.text.contains("class Member"));
    assert!(report.text.contains("class NamedMemberFamilyCalls"));
    assert!(child.text.contains("class NamedMemberFamilyCalls$Member"));
}

#[test]
fn disagreeing_child_relation_cannot_authorize_package_private_call() {
    let report = report_from(
        include_bytes!(
            "../openspec/evidence/java-syntax-2026-09-26/named-member-family-calls/wrong-relation.jar"
        )
        .to_vec(),
        task_limits(&[]).unwrap(),
    );
    let ClassSourceMemberFamily::Refused {
        child: Some(child), ..
    } = &report.member_family
    else {
        panic!(
            "wrong relation must refuse family: {:?}",
            report.member_family
        );
    };
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert!(matches!(child.execution, ExecutionReport::Complete { .. }));
    assert!(
        report
            .methods
            .iter()
            .any(|method| method.item.identity.name.0 == b"main")
    );
    assert!(
        child
            .methods
            .iter()
            .any(|method| method.item.identity.name.0 == b"<init>")
    );
    assert!(report.text.contains("class NamedMemberFamilyStage1"));
    assert!(child.text.contains("class NamedMemberFamilyStage1$Member"));
}

#[test]
fn projection_budget_stop_keeps_capture_calls_and_both_physical_reports() {
    let complete = report_with(task_limits(&[]).unwrap());
    let mut limits = task_limits(&[]).unwrap();
    limits.method_bodies = complete.usage.method_bodies - 1;
    let stopped = report_with(limits);
    let ClassSourceMemberFamily::Prepared {
        child,
        capture: ClassSourceMemberCapture::Proved { .. },
        calls: ClassSourceMemberCalls::Proved { .. },
        projection: ClassSourceMemberProjection::Refused { reason },
        ..
    } = &stopped.member_family
    else {
        panic!("projection must stop after local proofs");
    };
    assert!(reason.contains("projection stopped") || reason.contains("incomplete"));
    assert!(
        matches!(stopped.execution, ExecutionReport::Partial { .. }),
        "complete bodies={} stopped bodies={} execution={:?}",
        complete.usage.method_bodies,
        stopped.usage.method_bodies,
        stopped.execution
    );
    assert!(matches!(child.execution, ExecutionReport::Complete { .. }));
    assert_eq!(stopped.methods.len(), complete.methods.len());
    assert!(!stopped.text.contains("class Member"));
    assert!(child.text.contains("this$0"));
    let serialized = serde_json::to_value(&stopped).unwrap();
    assert_eq!(
        serialized["member_family"]["projection"]["state"],
        "refused"
    );
    assert!(
        serialized["member_family"]["projection"]
            .get("derived")
            .is_none()
    );
}

#[test]
fn family_output_budget_refusal_keeps_both_physical_texts() {
    let complete = report_with(task_limits(&[]).unwrap());
    let mut limits = task_limits(&[]).unwrap();
    limits.output_bytes = complete.usage.output_bytes - 1;
    let stopped = report_with(limits);
    let ClassSourceMemberFamily::Prepared {
        child, projection, ..
    } = &stopped.member_family
    else {
        panic!("physical family should remain prepared");
    };
    assert!(
        matches!(projection, ClassSourceMemberProjection::Refused { .. }),
        "{projection:?}"
    );
    assert!(matches!(stopped.execution, ExecutionReport::Partial { .. }));
    assert!(!stopped.text.contains("class Member"));
    assert!(child.text.contains("this$0"));
    assert_eq!(stopped.methods.len(), complete.methods.len());
}

#[test]
fn outer_super_method_bridge_is_not_projected() {
    let report = report_from_named(
        include_bytes!("../openspec/evidence/java-syntax-2026-09-26/named-member-outer-receiver/variants/fixture.jar").to_vec(),
        "OuterReceiverCases",
        task_limits(&[]).unwrap(),
    );
    let ClassSourceMemberFamily::Prepared {
        child, projection, ..
    } = &report.member_family
    else {
        panic!("physical super-bridge family should remain prepared");
    };
    assert!(
        matches!(projection, ClassSourceMemberProjection::Refused { reason } if reason.contains("Outer.super method bridge")),
        "{projection:?}"
    );
    assert!(report.text.contains("access$"));
    assert!(child.text.contains("OuterReceiverCases$Member"));
    assert!(!report.text.contains("class Member"));
    assert!(!report.text.contains("OuterReceiverCases.super.value()"));
}
