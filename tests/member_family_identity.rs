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

/// The same report with the source-map evidence a family fold's provenance anchors need: the
/// static member fold (like the nested-enum fold before it) re-spells body tokens through their
/// source maps, so a request that does not ask for them keeps the separated presentation.
fn report_from_named_source_mapped(
    bytes: Vec<u8>,
    name: &str,
    limits: Limits,
) -> ClassSourceReport {
    report_from_named_with_evidence(
        bytes,
        name,
        limits,
        &RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
    )
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

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn read_u32(bytes: &[u8], offset: usize) -> usize {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize
}

fn skip_attributes(bytes: &[u8], offset: &mut usize) {
    let count = read_u16(bytes, *offset) as usize;
    *offset += 2;
    for _ in 0..count {
        let length = read_u32(bytes, *offset + 2);
        *offset += 6 + length;
    }
}

fn skip_members(bytes: &[u8], offset: &mut usize) {
    let count = read_u16(bytes, *offset) as usize;
    *offset += 2;
    for _ in 0..count {
        *offset += 6;
        skip_attributes(bytes, offset);
    }
}

fn change_child_inner_class_outer(bytes: &[u8], child: &[u8], wrong_outer: &[u8]) -> Vec<u8> {
    let mut patched = bytes.to_vec();
    let cp_count = read_u16(&patched, 8) as usize;
    let mut utf8 = vec![None; cp_count];
    let mut classes = vec![None; cp_count];
    let mut cursor = 10;
    let mut index = 1;
    while index < cp_count {
        match patched[cursor] {
            1 => {
                let length = read_u16(&patched, cursor + 1) as usize;
                utf8[index] = Some(patched[cursor + 3..cursor + 3 + length].to_vec());
                cursor += 3 + length;
            }
            3 | 4 => cursor += 5,
            5 | 6 => {
                cursor += 9;
                index += 1;
            }
            7 => {
                classes[index] = Some(read_u16(&patched, cursor + 1));
                cursor += 3;
            }
            8 | 16 | 19 | 20 => cursor += 3,
            9 | 10 | 11 | 12 | 17 | 18 => cursor += 5,
            15 => cursor += 4,
            tag => panic!("unexpected constant-pool tag {tag}"),
        }
        index += 1;
    }
    let class_index = |name: &[u8]| {
        classes
            .iter()
            .enumerate()
            .find_map(|(index, name_index)| {
                name_index
                    .and_then(|name_index| utf8[name_index as usize].as_deref())
                    .filter(|candidate| *candidate == name)
                    .map(|_| index as u16)
            })
            .expect("fixture constant pool contains requested class")
    };
    let child_index = class_index(child);
    let wrong_outer_index = if wrong_outer.is_empty() {
        0
    } else {
        class_index(wrong_outer)
    };

    let mut offset = cursor + 6;
    let interfaces = read_u16(&patched, offset) as usize;
    offset += 2 + interfaces * 2;
    skip_members(&patched, &mut offset);
    skip_members(&patched, &mut offset);
    let attributes = read_u16(&patched, offset) as usize;
    offset += 2;
    let mut found = false;
    for _ in 0..attributes {
        let name_index = read_u16(&patched, offset) as usize;
        let length = read_u32(&patched, offset + 2);
        let info = offset + 6;
        if utf8[name_index].as_deref() == Some(b"InnerClasses") {
            let count = read_u16(&patched, info) as usize;
            for row in 0..count {
                let entry = info + 2 + row * 8;
                if read_u16(&patched, entry) == child_index {
                    patched[entry + 2..entry + 4].copy_from_slice(&wrong_outer_index.to_be_bytes());
                    found = true;
                }
            }
        }
        offset = info + length;
    }
    assert!(found, "child has one self InnerClasses row");
    patched
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
fn static_member_projects_as_nested_and_keeps_dollar_top_level_independent() {
    const ROOT: &[u8] = include_bytes!(
        "../tests/fixtures/proved-java-structure/static-member-basic/StaticMemberBasic.class"
    );
    const CHILD: &[u8] = include_bytes!(
        "../tests/fixtures/proved-java-structure/static-member-basic/StaticMemberBasic$Leaf.class"
    );
    const TOP: &[u8] = include_bytes!(
        "../tests/fixtures/proved-java-structure/static-member-basic/Named$Top.class"
    );
    let jar = jar_of(&[
        (b"StaticMemberBasic.class", ROOT),
        (b"StaticMemberBasic$Leaf.class", CHILD),
        (b"Named$Top.class", TOP),
    ]);
    let root = report_from_named(jar.clone(), "StaticMemberBasic", task_limits(&[]).unwrap());
    let ClassSourceMemberFamily::Prepared {
        child,
        capture: ClassSourceMemberCapture::StaticNoCapture { .. },
        calls: ClassSourceMemberCalls::StaticProved { sites },
        projection: ClassSourceMemberProjection::Projected { derived },
        ..
    } = &root.member_family
    else {
        panic!(
            "static member family should be fully proved: {:?}",
            root.member_family
        );
    };
    assert_eq!(sites.len(), 1);
    assert!(root.text.contains("static Leaf make()"));
    assert!(root.text.contains("return new Leaf();"));
    assert!(root.text.contains("static class Leaf extends"));
    assert!(root.text.contains("Leaf() {"));
    assert!(!root.text.contains("StaticMemberBasic$Leaf()"));
    assert!(child.text.contains("class StaticMemberBasic$Leaf"));
    assert!(child.text.contains("StaticMemberBasic$Leaf()"));
    assert_eq!(derived.len(), 4);

    let top = report_from_named(jar, "Named$Top", task_limits(&[]).unwrap());
    assert!(top.text.contains("class Named$Top"));
    assert!(!top.text.contains("static class"));

    let temp = TestDirectory::new();
    std::fs::write(temp.path().join("StaticMemberBasic.java"), &root.text).unwrap();
    std::fs::write(
        temp.path().join("Named$Top.java"),
        include_str!("../tests/fixtures/proved-java-structure/static-member-basic/Named$Top.java"),
    )
    .unwrap();
    let compile = Command::new("javac")
        .args([
            "--release",
            "8",
            "-g:none",
            "StaticMemberBasic.java",
            "Named$Top.java",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp", ".", "StaticMemberBasic"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8(run.stdout).unwrap(), "9:4\n");

    let mut constrained = task_limits(&[]).unwrap();
    constrained.output_bytes = root.usage.output_bytes.saturating_sub(1);
    let stopped = report_from_named(
        jar_of(&[
            (b"StaticMemberBasic.class", ROOT),
            (b"StaticMemberBasic$Leaf.class", CHILD),
            (b"Named$Top.class", TOP),
        ]),
        "StaticMemberBasic",
        constrained,
    );
    assert!(!stopped.text.contains("static class Leaf"));
    assert!(
        stopped.text.contains("new StaticMemberBasic$Leaf()"),
        "a refused family must retain the physical construction: {}",
        stopped.text
    );
    assert!(
        stopped.text.contains("StaticMemberBasic$Leaf make()"),
        "a refused family must retain the physical return type: {}",
        stopped.text
    );
    assert!(!stopped.text.contains("\n    Leaf make()"));
    assert!(matches!(
        stopped.member_family,
        ClassSourceMemberFamily::Prepared {
            projection: ClassSourceMemberProjection::Refused { .. },
            ..
        }
    ));

    let cancelled_jar = jar_of(&[
        (b"StaticMemberBasic.class", ROOT),
        (b"StaticMemberBasic$Leaf.class", CHILD),
        (b"Named$Top.class", TOP),
    ]);
    let engine = Engine::new();
    let limits = task_limits(&[]).unwrap();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(cancelled_jar),
            &mut Budget::new(limits.clone()),
        )
        .expect("the frozen archive opens before the request is cancelled");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("StaticMemberBasic"),
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
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(limits, token);
    let outcome = engine
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::essential(),
            &mut cancelled,
        )
        .expect("cancelled source selection is reported");
    assert!(matches!(outcome, OperationOutcome::Incomplete(_)));
}

#[test]
fn declaration_only_static_abstract_member_projects_once_with_physical_anchors() {
    const SOURCE: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-27/em01-declarations/input-single/em01/SingleAbstract.java"
    );
    const RUNNER: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-27/em01-declarations/input-single/em01/Runner.java"
    );
    let compile = |source: &str| {
        let temp = TestDirectory::new();
        std::fs::write(temp.path().join("SingleAbstract.java"), source).unwrap();
        let output = Command::new("javac")
            .args([
                "--release",
                "8",
                "-g:none",
                "-d",
                ".",
                "SingleAbstract.java",
            ])
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let root = std::fs::read(temp.path().join("em01/SingleAbstract.class")).unwrap();
        let child = std::fs::read(temp.path().join("em01/SingleAbstract$A.class")).unwrap();
        (temp, root, child)
    };
    let (temp, root_bytes, child_bytes) = compile(SOURCE);
    let jar = jar_of(&[
        (b"em01/SingleAbstract.class", &root_bytes),
        (b"em01/SingleAbstract$A.class", &child_bytes),
    ]);
    let root = report_from_named(
        jar.clone(),
        "em01/SingleAbstract",
        task_limits(&[]).unwrap(),
    );
    let ClassSourceMemberFamily::Prepared {
        child,
        calls: ClassSourceMemberCalls::StaticDeclarationOnly,
        projection: ClassSourceMemberProjection::Projected { derived },
        ..
    } = &root.member_family
    else {
        panic!(
            "expected declaration-only projection: {:?}",
            root.member_family
        );
    };
    assert_eq!(
        root.text.matches("public static abstract class A").count(),
        1
    );
    assert!(root.text.contains("A() {"), "{}", root.text);
    assert!(root.text.contains("abstract int test2();"), "{}", root.text);
    assert_eq!(derived.len(), 2);
    for kind in [
        MemberFamilyDerivedKind::MemberClassDeclaration,
        MemberFamilyDerivedKind::MemberConstructorName,
    ] {
        let entry = derived.iter().find(|entry| entry.kind == kind).unwrap();
        assert!(entry.anchors.iter().any(|anchor| matches!(anchor,
            MemberFamilyPhysicalAnchor::ClassDefinition { definition } if definition == &child.class)));
    }
    let physical = report_from_named(
        jar.clone(),
        "em01/SingleAbstract$A",
        task_limits(&[]).unwrap(),
    );
    assert!(physical.text.contains("SingleAbstract$A()"));
    assert_eq!(child.class, physical.class);
    std::fs::write(temp.path().join("SingleAbstract.java"), &root.text).unwrap();
    std::fs::write(temp.path().join("Runner.java"), RUNNER).unwrap();
    let output = Command::new("javac")
        .args([
            "--release",
            "8",
            "-g:none",
            "-d",
            ".",
            "SingleAbstract.java",
            "Runner.java",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp", ".", "em01.Runner"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8(run.stdout).unwrap(), "true:1\n");

    let wrong_child = change_child_inner_class_outer(&child_bytes, b"em01/SingleAbstract$A", b"");
    let wrong = report_from_named(
        jar_of(&[
            (b"em01/SingleAbstract.class", &root_bytes),
            (b"em01/SingleAbstract$A.class", &wrong_child),
        ]),
        "em01/SingleAbstract",
        task_limits(&[]).unwrap(),
    );
    assert!(!wrong.text.contains("static abstract class A"));
    assert!(matches!(
        wrong.member_family,
        ClassSourceMemberFamily::Refused { .. }
    ));

    // The static member fold (change `recover-member-class-static-folding`) widened the family
    // these near misses belong to: a single static child whose narrow certificate refused used
    // to keep the separated presentation, and now folds with the child's full physical text.
    // A generic child class folds too since `recover-nested-generic-class-headers`: the
    // nested header carries the child's proved type parameters (`A<T>`), so the fold's own
    // text states the complete Signature projection.
    for (source, folds) in [
        (
            SOURCE.replace("    }\n}", "        int state;\n    }\n}"),
            true,
        ),
        (SOURCE.replace("class A {", "class A<T> {"), true),
        (
            SOURCE.replace("abstract int test2();", "abstract <T> int test2();"),
            true,
        ),
        (
            SOURCE.replace(
                "public static abstract class A",
                "@Deprecated public static abstract class A",
            ),
            true,
        ),
        (
            SOURCE.replace(
                "    public static abstract class A",
                "    static A echo(A value) { return value; }\n    public static abstract class A",
            ),
            true,
        ),
    ] {
        let (_, root, child) = compile(&source);
        let report = report_from_named_source_mapped(
            jar_of(&[
                (b"em01/SingleAbstract.class", &root),
                (b"em01/SingleAbstract$A.class", &child),
            ]),
            "em01/SingleAbstract",
            task_limits(&[]).unwrap(),
        );
        if folds {
            assert!(
                report.text.contains("public static abstract class A"),
                "{}",
                report.text
            );
            assert!(matches!(
                &report.member_family,
                ClassSourceMemberFamily::PreparedStatic {
                    projection: ClassSourceMemberProjection::Projected { .. },
                    ..
                }
            ));
        } else {
            assert!(
                !report.text.contains("static abstract class A"),
                "{}",
                report.text
            );
        }
    }
    let second = SOURCE.replace("    }\n}", "    }\n    static class B {}\n}");
    let (extra, root, child) = compile(&second);
    let other = std::fs::read(extra.path().join("em01/SingleAbstract$B.class")).unwrap();
    let report = report_from_named_source_mapped(
        jar_of(&[
            (b"em01/SingleAbstract.class", &root),
            (b"em01/SingleAbstract$A.class", &child),
            (b"em01/SingleAbstract$B.class", &other),
        ]),
        "em01/SingleAbstract",
        task_limits(&[]).unwrap(),
    );
    // A sibling beside the abstract member is a two-row static family: the fold states both
    // nested declarations where the one-child certificate used to refuse the whole root.
    assert!(report.text.contains("public static abstract class A"));
    assert!(report.text.contains("static class B"));
    assert!(matches!(
        &report.member_family,
        ClassSourceMemberFamily::PreparedStatic {
            projection: ClassSourceMemberProjection::Projected { .. },
            ..
        }
    ));

    let mut low = task_limits(&[]).unwrap();
    low.output_bytes = report_from_named(
        jar.clone(),
        "em01/SingleAbstract",
        task_limits(&[]).unwrap(),
    )
    .usage
    .output_bytes
    .saturating_sub(1);
    let stopped = report_from_named(jar, "em01/SingleAbstract", low);
    assert!(!stopped.text.contains("static abstract class A"));
}

#[test]
fn declaration_pair_projects_both_physical_members_and_origins_once() {
    const ROOT: &[u8] = include_bytes!("fixtures/em01-member-pair/Shape.class");
    const ABSTRACT: &[u8] = include_bytes!("fixtures/em01-member-pair/Shape$A.class");
    const INTERFACE: &[u8] = include_bytes!("fixtures/em01-member-pair/Shape$I.class");
    let jar = jar_of(&[
        (b"em01/Shape.class", ROOT),
        (b"em01/Shape$A.class", ABSTRACT),
        (b"em01/Shape$I.class", INTERFACE),
    ]);
    let report = report_from_named(jar.clone(), "em01/Shape", task_limits(&[]).unwrap());
    let ClassSourceMemberFamily::PreparedPair {
        members,
        projection: ClassSourceMemberProjection::Projected { derived },
    } = &report.member_family
    else {
        panic!("pair not projected: {:?}", report.member_family);
    };
    assert_eq!(members[0].relation.simple_name, "A");
    assert_eq!(members[1].relation.simple_name, "I");
    assert_eq!(report.text.matches("class A extends").count(), 1);
    assert_eq!(report.text.matches("interface I {").count(), 1);
    assert!(
        report.text.find("class A extends").unwrap() < report.text.find("interface I {").unwrap()
    );
    assert!(report.text.contains("public A() {"), "{}", report.text);
    assert!(!report.text.contains("Shape$A()"), "{}", report.text);
    assert!(report.text.contains("abstract int test2();"));
    assert!(report.text.contains("abstract int test();"));
    assert!(report.text.contains("abstract int test3();"));
    assert_eq!(derived.len(), 6);
    assert_eq!(
        derived
            .iter()
            .filter(|item| item.kind == MemberFamilyDerivedKind::MemberClassDeclaration)
            .count(),
        2
    );
    assert_eq!(
        derived
            .iter()
            .filter(|item| item.kind == MemberFamilyDerivedKind::MemberMethodDeclaration)
            .count(),
        3
    );
    for item in derived {
        assert!(item.start < item.end && item.end <= report.text.len());
        assert!(!report.text[item.start..item.end].is_empty());
        assert!(!item.anchors.is_empty());
    }
    for member in members {
        let headers: Vec<_> = derived.iter().filter(|item| item.kind == MemberFamilyDerivedKind::MemberClassDeclaration
            && item.anchors.iter().any(|anchor| matches!(anchor,
                MemberFamilyPhysicalAnchor::ClassDefinition { definition } if definition == &member.child.class))).collect();
        assert_eq!(headers.len(), 1);
        assert!(
            report.text[headers[0].start..headers[0].end].contains(&member.relation.simple_name)
        );
        let methods = member
            .child
            .methods
            .iter()
            .filter(|method| method.item.identity.name.0 != b"<init>")
            .count();
        let signatures: Vec<_> = derived.iter().filter(|item| item.kind == MemberFamilyDerivedKind::MemberMethodDeclaration
            && item.anchors.iter().any(|anchor| matches!(anchor,
                MemberFamilyPhysicalAnchor::MethodSignature { method } if method.owner == member.child.class))).collect();
        assert_eq!(signatures.len(), methods);
        for signature in signatures {
            let span = &report.text[signature.start..signature.end];
            assert!(
                span.starts_with("public abstract int ") && span.ends_with("()"),
                "{span}"
            );
        }
    }
    let constructors: Vec<_> = derived
        .iter()
        .filter(|item| item.kind == MemberFamilyDerivedKind::MemberConstructorName)
        .collect();
    assert_eq!(constructors.len(), 1);
    assert_eq!(
        &report.text[constructors[0].start..constructors[0].end],
        "A"
    );
    assert!(constructors[0].anchors.iter().any(|anchor| matches!(anchor,
        MemberFamilyPhysicalAnchor::MethodPoint { method, bci } if method.owner == members[0].child.class && *bci == 0)));
    for member in members {
        let physical = report_from_named(
            jar.clone(),
            &format!("em01/Shape${}", member.relation.simple_name),
            task_limits(&[]).unwrap(),
        );
        assert_eq!(physical.class, member.child.class);
        assert!(matches!(
            physical.execution,
            ExecutionReport::Complete { .. }
        ));
    }
}

#[test]
fn declaration_pair_near_misses_never_publish_half_a_root() {
    const ROOT: &[u8] = include_bytes!("fixtures/em01-member-pair/Shape.class");
    const ABSTRACT: &[u8] = include_bytes!("fixtures/em01-member-pair/Shape$A.class");
    const INTERFACE: &[u8] = include_bytes!("fixtures/em01-member-pair/Shape$I.class");
    let variants: [(&[u8], &[u8], &[u8]); 7] = [
        (
            ROOT,
            ABSTRACT,
            include_bytes!("fixtures/em01-member-pair/Shape-I-wrong-self.class"),
        ),
        (
            ROOT,
            ABSTRACT,
            include_bytes!("fixtures/em01-member-pair/Shape-I-interface-default.class"),
        ),
        (
            ROOT,
            ABSTRACT,
            include_bytes!("fixtures/em01-member-pair/Shape-I-interface-static.class"),
        ),
        (
            ROOT,
            ABSTRACT,
            include_bytes!("fixtures/em01-member-pair/Shape-I-field.class"),
        ),
        (
            ROOT,
            ABSTRACT,
            include_bytes!("fixtures/em01-member-pair/Shape-I-signature.class"),
        ),
        (
            include_bytes!("fixtures/em01-member-pair/Shape-third-child.class"),
            ABSTRACT,
            INTERFACE,
        ),
        (
            include_bytes!("fixtures/em01-member-pair/Shape-root-use.class"),
            ABSTRACT,
            INTERFACE,
        ),
    ];
    let missing = report_from_named(
        jar_of(&[
            (b"em01/Shape.class", ROOT),
            (b"em01/Shape$A.class", ABSTRACT),
        ]),
        "em01/Shape",
        task_limits(&[]).unwrap(),
    );
    assert!(!missing.text.contains("class A extends"));
    assert!(!missing.text.contains("interface I {"));
    assert!(matches!(&missing.member_family,
        ClassSourceMemberFamily::RefusedPair { children, .. }
            if children.len() == 1 && children[0].class != missing.class));
    for (index, (root, abstract_child, interface_child)) in variants.into_iter().enumerate() {
        let report = report_from_named(
            jar_of(&[
                (b"em01/Shape.class", root),
                (b"em01/Shape$A.class", abstract_child),
                (b"em01/Shape$I.class", interface_child),
            ]),
            "em01/Shape",
            task_limits(&[]).unwrap(),
        );
        if index != 5 {
            assert!(!report.text.contains("class A extends"), "{}", report.text);
            assert!(!report.text.contains("interface I {"), "{}", report.text);
        }
        match index {
            0..=4 => assert!(matches!(&report.member_family,
                ClassSourceMemberFamily::RefusedPair { children, .. } if children.len() == 2)),
            // A third direct row breaks the exact pair, and the static fold owns the three-row
            // family — but this jar does not carry the third child's definition, so the fold
            // refuses exactly as the one-child certificate did: no definition, no fold.
            5 => {
                assert!(!report.text.contains("class A extends"), "{}", report.text);
                assert!(!report.text.contains("interface I {"), "{}", report.text);
                assert!(matches!(
                    &report.member_family,
                    ClassSourceMemberFamily::RefusedPair { .. }
                ));
            }
            6 => assert!(matches!(
                report.member_family,
                ClassSourceMemberFamily::PreparedPair {
                    projection: ClassSourceMemberProjection::Refused { .. },
                    ..
                }
            )),
            _ => unreachable!(),
        }
    }
    let jar = jar_of(&[
        (b"em01/Shape.class", ROOT),
        (b"em01/Shape$A.class", ABSTRACT),
        (b"em01/Shape$I.class", INTERFACE),
    ]);
    let complete = report_from_named(jar.clone(), "em01/Shape", task_limits(&[]).unwrap());
    let mut low = task_limits(&[]).unwrap();
    low.output_bytes = complete.usage.output_bytes.saturating_sub(1);
    let stopped = report_from_named(jar.clone(), "em01/Shape", low);
    assert!(!stopped.text.contains("class A extends"));
    assert!(!stopped.text.contains("interface I {"));
    assert!(matches!(stopped.execution, ExecutionReport::Partial { .. }));
    assert!(matches!(
        stopped.member_family,
        ClassSourceMemberFamily::PreparedPair {
            projection: ClassSourceMemberProjection::Refused { .. },
            ..
        }
    ));
    let engine = Engine::new();
    let mut open_budget = Budget::new(task_limits(&[]).unwrap());
    let snapshot = engine
        .open(ArtifactInput::bytes(jar), &mut open_budget)
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("em01/Shape"),
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
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(task_limits(&[]).unwrap(), token);
    assert!(matches!(
        engine
            .class_source(&[snapshot], &request, &mut cancelled)
            .unwrap(),
        OperationOutcome::Incomplete(_)
    ));
}

#[test]
fn static_member_incomplete_targets_never_publish_partial_nested_source() {
    let compile = |name: &str, source: &str, children: &[&str], top_level: &[&str]| {
        let temp = TestDirectory::new();
        std::fs::write(temp.path().join(format!("{name}.java")), source).unwrap();
        let source_name = format!("{name}.java");
        let output = Command::new("javac")
            .args(["--release", "8", "-g:none", &source_name])
            .current_dir(temp.path())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut owned = vec![(
            format!("{name}.class").into_bytes(),
            std::fs::read(temp.path().join(format!("{name}.class"))).unwrap(),
        )];
        for child in children {
            let filename = format!("{name}${child}.class");
            owned.push((
                filename.as_bytes().to_vec(),
                std::fs::read(temp.path().join(filename)).unwrap(),
            ));
        }
        for class in top_level {
            let filename = format!("{class}.class");
            owned.push((
                filename.as_bytes().to_vec(),
                std::fs::read(temp.path().join(filename)).unwrap(),
            ));
        }
        let entries = owned
            .iter()
            .map(|(name, bytes)| (name.as_slice(), bytes.as_slice()))
            .collect::<Vec<_>>();
        (jar_of(&entries), owned)
    };

    // The static member fold (change `recover-member-class-static-folding`) took over the
    // single-static-child shapes the narrow certificate refused: the child's full physical text
    // becomes one nested declaration, where these negatives used to keep the separated
    // presentation. The `missing` half below keeps its old refusal: no child definition, no fold.
    for (name, source) in [
        (
            "StaticFieldNegative",
            "class StaticFieldNegative { static class Leaf { int state; Leaf() {} } static Leaf make() { return new Leaf(); } }",
        ),
        (
            "StaticCtorNegative",
            "class StaticCtorNegative { static class Leaf { Leaf() { System.nanoTime(); } } static Leaf make() { return new Leaf(); } }",
        ),
    ] {
        let (jar, compiled) = compile(name, source, &["Leaf"], &[]);
        let report = report_from_named_source_mapped(jar.clone(), name, task_limits(&[]).unwrap());
        assert!(report.text.contains("static class Leaf"), "{}", report.text);
        assert!(matches!(
            report.member_family,
            ClassSourceMemberFamily::PreparedStatic {
                projection: ClassSourceMemberProjection::Projected { .. },
                ..
            }
        ));

        let missing = report_from_named(
            jar_of(&[(format!("{name}.class").as_bytes(), &compiled[0].1)]),
            name,
            task_limits(&[]).unwrap(),
        );
        assert!(!missing.text.contains("static class Leaf"));
        assert!(matches!(
            missing.member_family,
            ClassSourceMemberFamily::Refused { .. }
        ));
    }

    let (extra_type_use, _) = compile(
        "StaticEchoNegative",
        "class StaticEchoNegative { static class Leaf { int value() { return 9; } } static Leaf make() { return new Leaf(); } static Leaf echo(Leaf value) { return value; } static int run() { return echo(make()).value(); } }",
        &["Leaf"],
        &[],
    );
    let report = report_from_named_source_mapped(
        extra_type_use,
        "StaticEchoNegative",
        task_limits(&[]).unwrap(),
    );
    // A second use no longer prevents the nested projection: the fold owns every reference the
    // unit states, and each one is re-spelled inside the scope that declares the member.
    assert!(report.text.contains("static class Leaf"), "{}", report.text);
    assert!(
        report.text.contains("static Leaf echo(Leaf arg0)"),
        "{}",
        report.text
    );
    assert!(matches!(
        report.member_family,
        ClassSourceMemberFamily::PreparedStatic {
            projection: ClassSourceMemberProjection::Projected { .. },
            ..
        }
    ));

    let (multiple, _) = compile(
        "StaticPairNegative",
        "class StaticPairNegative { static class Leaf {} static class Other {} }",
        &["Leaf", "Other"],
        &[],
    );
    let report =
        report_from_named_source_mapped(multiple, "StaticPairNegative", task_limits(&[]).unwrap());
    assert!(report.text.contains("static class Leaf"));
    assert!(report.text.contains("static class Other"));
    assert!(matches!(
        report.member_family,
        ClassSourceMemberFamily::PreparedStatic {
            projection: ClassSourceMemberProjection::Projected { .. },
            ..
        }
    ));

    const ROOT: &[u8] = include_bytes!(
        "../tests/fixtures/proved-java-structure/static-member-basic/StaticMemberBasic.class"
    );
    const CHILD: &[u8] = include_bytes!(
        "../tests/fixtures/proved-java-structure/static-member-basic/StaticMemberBasic$Leaf.class"
    );
    let duplicate_child = jar_of(&[
        (b"StaticMemberBasic.class", ROOT),
        (b"StaticMemberBasic$Leaf.class", CHILD),
        (b"StaticMemberBasic$Leaf.class", CHILD),
    ]);
    let report = report_from_named(
        duplicate_child,
        "StaticMemberBasic",
        task_limits(&[]).unwrap(),
    );
    assert!(!report.text.contains("static class Leaf"));
    assert!(matches!(
        report.member_family,
        ClassSourceMemberFamily::Refused { .. }
    ));

    let (_, compiled) = compile(
        "StaticMismatch",
        "class StaticMismatch { static class Leaf { Leaf() {} static Class<?> other() { return OtherRoot.class; } } } class OtherRoot {}",
        &["Leaf"],
        &["OtherRoot"],
    );
    let wrong_child =
        change_child_inner_class_outer(&compiled[1].1, b"StaticMismatch$Leaf", b"OtherRoot");
    let mismatch = jar_of(&[
        (b"StaticMismatch.class", &compiled[0].1),
        (b"StaticMismatch$Leaf.class", &wrong_child),
        (b"OtherRoot.class", &compiled[2].1),
    ]);
    let report = report_from_named(mismatch, "StaticMismatch", task_limits(&[]).unwrap());
    assert!(!report.text.contains("static class Leaf"));
    let ClassSourceMemberFamily::Refused { reason, .. } = &report.member_family else {
        panic!(
            "conflicting typed member rows must refuse the family: {:?}",
            report.member_family
        );
    };
    assert!(
        reason.contains("unique matching InnerClasses self row"),
        "{reason}"
    );
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
fn outer_super_method_bridge_projects_with_physical_origin() {
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
    let ClassSourceMemberProjection::Projected { derived } = projection else {
        panic!("closed bridge must project: {projection:?}");
    };
    assert!(report.text.contains("OuterReceiverCases.super.value()"));
    assert!(!report.text.contains("static int access$101("));
    assert!(child.text.contains("OuterReceiverCases$Member"));
    assert!(derived.iter().any(|entry| {
        entry.kind == MemberFamilyDerivedKind::HiddenOuterSuperBridge
            && entry.anchors.iter().any(|anchor| {
                matches!(anchor,
                MemberFamilyPhysicalAnchor::OuterSuperTarget { owner, name, descriptor, .. }
                    if owner.0 == b"ReceiverBase" && name.0 == b"value" && descriptor.0 == b"()I")
            })
    }));
}

fn em12_jar(case: &str, parent: &str, omit_narrow: bool) -> Vec<u8> {
    let temp = TestDirectory::new();
    std::fs::create_dir(temp.path().join("em12")).unwrap();
    for (name, source) in [
        ("Case", case),
        ("Parent", parent),
        (
            "Arg",
            include_str!(
                "../openspec/evidence/java-syntax-2026-09-27/em12-super-dispatch/input/em12/Arg.java"
            ),
        ),
        (
            "NarrowArg",
            include_str!(
                "../openspec/evidence/java-syntax-2026-09-27/em12-super-dispatch/input/em12/NarrowArg.java"
            ),
        ),
    ] {
        std::fs::write(temp.path().join(format!("em12/{name}.java")), source).unwrap();
    }
    let compiled = Command::new("javac")
        .args([
            "--release",
            "8",
            "-g:none",
            "-d",
            ".",
            "em12/Case.java",
            "em12/Parent.java",
            "em12/Arg.java",
            "em12/NarrowArg.java",
        ])
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let names = ["Case", "Case$Member", "Parent", "Arg", "NarrowArg"];
    let bytes: Vec<_> = names
        .iter()
        .filter(|name| !omit_narrow || **name != "NarrowArg")
        .map(|name| std::fs::read(temp.path().join(format!("em12/{name}.class"))).unwrap())
        .collect();
    let entries: Vec<_> = names
        .iter()
        .filter(|name| !omit_narrow || **name != "NarrowArg")
        .zip(&bytes)
        .map(|(name, bytes)| (format!("em12/{name}.class").into_bytes(), bytes.as_slice()))
        .collect();
    jar_of(
        &entries
            .iter()
            .map(|(name, bytes)| (name.as_slice(), *bytes))
            .collect::<Vec<_>>(),
    )
}

#[test]
fn outer_super_overload_requires_direct_source_parameter_and_complete_subclass_chain() {
    const CASE: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-27/em12-super-dispatch/input/em12/Case.java"
    );
    const PARENT: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-27/em12-super-dispatch/input/em12/Parent.java"
    );
    let projected = report_from_named(
        em12_jar(CASE, PARENT, false),
        "em12/Case",
        task_limits(&[]).unwrap(),
    );
    assert!(
        matches!(
            &projected.member_family,
            ClassSourceMemberFamily::Prepared {
                projection: ClassSourceMemberProjection::Projected { .. },
                ..
            }
        ),
        "{:?}",
        projected.member_family
    );
    assert!(
        projected.text.contains("Case.super.pick(arg1)"),
        "{}",
        projected.text
    );
    assert!(!projected.text.contains("static String access$"));

    let cast = CASE.replace("pick(value)", "pick((Arg) null)");
    let intermediate = CASE.replace(
        "return Case.super.pick(value);",
        "Arg copy = value; return Case.super.pick(copy);",
    );
    let narrow = CASE
        .replace("call(Arg value)", "call(NarrowArg value)")
        .replace("pick(value)", "pick((Arg) value)");
    let applicable = PARENT.replace("pick(NarrowArg value)", "pick(Object value)");
    let generic = PARENT.replace(
        "String pick(NarrowArg value)",
        "<T extends NarrowArg> String pick(T value)",
    );
    let checked = PARENT.replace(
        "pick(NarrowArg value)",
        "pick(NarrowArg value) throws java.io.IOException",
    );
    let varargs = PARENT.replace("pick(NarrowArg value)", "pick(NarrowArg... value)");
    for (case, parent, omit_narrow, must_reach_binding) in [
        (cast.as_str(), PARENT, false, false),
        (intermediate.as_str(), PARENT, false, false),
        (narrow.as_str(), PARENT, false, true),
        (CASE, PARENT, true, true),
        (CASE, applicable.as_str(), false, true),
        (CASE, generic.as_str(), false, true),
        (CASE, checked.as_str(), false, true),
        (CASE, varargs.as_str(), false, true),
    ] {
        let refused = report_from_named(
            em12_jar(case, parent, omit_narrow),
            "em12/Case",
            task_limits(&[]).unwrap(),
        );
        let ClassSourceMemberFamily::Prepared {
            projection: ClassSourceMemberProjection::Refused { reason },
            ..
        } = &refused.member_family
        else {
            panic!("variant should refuse projection: {case}");
        };
        assert!(
            !must_reach_binding || reason.contains("Outer.super source binding refused"),
            "{reason}"
        );
        assert!(!refused.text.contains("Case.super.pick("));
    }
}
