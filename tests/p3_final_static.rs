//! `recover-final-static-field-writes`: a whole Java 8 class-source fixture for blank static-final
//! writes.  The committed class contains one `<clinit>` with two ordered calls, one final written
//! in both branch arms, and a no-debug local whose ordinal name would otherwise collide with fields
//! named `local0` and `local0_2`.
//!
//! `FinalStaticSupport.java` and `FinalStaticRunner.java` are source-only JDK inputs.  The ignored
//! test compiles the original helper/runner sources, replaces the compiler-produced probe class
//! with the committed bytes, and then runs both branches in separate JVMs.  The recovered side is
//! compiled from the complete `Engine::class_source_with_evidence` text with the same helper and
//! runner; no method is replaced or omitted.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] = include_bytes!("fixtures/p3-final-static/v8/FinalStaticProbe.class");
const PROBE_SOURCE: &str = include_str!("fixtures/p3-final-static/FinalStaticProbe.java");
const SUPPORT_SOURCE: &str = include_str!("fixtures/p3-final-static/FinalStaticSupport.java");
const RUNNER_SOURCE: &str = include_str!("fixtures/p3-final-static/FinalStaticRunner.java");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed final-static fixture opens as a standalone CLASS")
}

fn class_source_of(
    snapshot: &ArtifactSnapshot,
    evidence: RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("FinalStaticProbe"),
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
    };
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &evidence,
            &mut budget(),
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one committed sample answers one definition, got {} candidate(s)",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one committed sample answers one definition, got an unfinished selection with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the fixture's method table"))
}

fn recovered<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &method(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` was not recovered: {other:?}"),
    }
}

fn all_recovered(report: &ClassSourceReport) {
    assert_eq!(
        report.methods.len(),
        5,
        "constructor, three methods, and <clinit>"
    );
    for name in [
        "<init>",
        "instanceValue",
        "readAfterAssign",
        "snapshot",
        "<clinit>",
    ] {
        let body = recovered(report, name);
        assert!(body.produced(), "`{name}` produced no body: {body:?}");
        assert_eq!(body.representation, Representation::Java, "`{name}`");
        assert_eq!(body.quality, Quality::Structured, "`{name}`");
        assert_eq!(
            body.content,
            RecoveryContent::ContainsStatements,
            "`{name}`"
        );
        assert!(
            !body.text.contains("@bytecode"),
            "`{name}` remains quoted:\n{}",
            body.text
        );
    }
}

#[derive(Clone)]
struct ConstantPoolEntry {
    tag: u8,
    payload: Vec<u8>,
    offset: usize,
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn constant_pool(bytes: &[u8]) -> (Vec<Option<ConstantPoolEntry>>, usize) {
    let count = usize::from(u16_at(bytes, 8));
    let mut entries = vec![None; count];
    let mut offset = 10usize;
    let mut index = 1usize;
    while index < count {
        let entry_offset = offset;
        let tag = bytes[offset];
        offset += 1;
        let width = match tag {
            1 => 2 + usize::from(u16_at(bytes, offset)),
            3 | 4 | 9 | 10 | 11 | 12 | 17 | 18 => 4,
            7 | 8 | 16 | 19 | 20 => 2,
            15 => 3,
            5 | 6 => 8,
            other => panic!("unsupported constant-pool tag {other}"),
        };
        entries[index] = Some(ConstantPoolEntry {
            tag,
            payload: bytes[offset..offset + width].to_vec(),
            offset: entry_offset,
        });
        offset += width;
        index += 1;
        if matches!(tag, 5 | 6) {
            index += 1;
        }
    }
    (entries, offset)
}

fn utf8(entries: &[Option<ConstantPoolEntry>], index: u16) -> &[u8] {
    let entry = entries[usize::from(index)]
        .as_ref()
        .expect("constant-pool entry");
    assert_eq!(entry.tag, 1, "expected a CONSTANT_Utf8 entry");
    &entry.payload[2..]
}

struct FieldSpan {
    flags: usize,
    name_index: usize,
    descriptor_index: usize,
    attributes_count: usize,
    attributes_end: usize,
    name: Vec<u8>,
}

fn field_spans(bytes: &[u8]) -> Vec<FieldSpan> {
    let (entries, mut offset) = constant_pool(bytes);
    offset += 6;
    let interfaces = usize::from(u16_at(bytes, offset));
    offset += 2 + interfaces * 2;
    let fields = usize::from(u16_at(bytes, offset));
    offset += 2;
    let mut result = Vec::with_capacity(fields);
    for _ in 0..fields {
        let flags = offset;
        let name_index = offset + 2;
        let descriptor_index = offset + 4;
        let attributes_count = offset + 6;
        let count = usize::from(u16_at(bytes, attributes_count));
        offset += 8;
        for _ in 0..count {
            let length = usize::try_from(u32::from_be_bytes(
                bytes[offset + 2..offset + 6]
                    .try_into()
                    .expect("attribute length"),
            ))
            .expect("attribute length fits usize");
            offset += 6 + length;
        }
        result.push(FieldSpan {
            flags,
            name_index,
            descriptor_index,
            attributes_count,
            attributes_end: offset,
            name: utf8(&entries, u16_at(bytes, name_index)).to_vec(),
        });
    }
    result
}

fn utf8_index(bytes: &[u8], wanted: &[u8]) -> u16 {
    let (entries, _) = constant_pool(bytes);
    entries
        .iter()
        .enumerate()
        .find_map(|(index, entry)| {
            let entry = entry.as_ref()?;
            (entry.tag == 1 && &entry.payload[2..] == wanted)
                .then(|| u16::try_from(index).expect("constant-pool index"))
        })
        .expect("fixture constant-pool UTF-8 entry")
}

fn with_field_name(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let mut patched = bytes.to_vec();
    let index = utf8_index(bytes, to);
    let field = field_spans(bytes)
        .into_iter()
        .find(|field| field.name == from)
        .expect("field to rename");
    patched[field.name_index..field.name_index + 2].copy_from_slice(&index.to_be_bytes());
    patched
}

fn with_field_flags(bytes: &[u8], name: &[u8], mask: u16, set: bool) -> Vec<u8> {
    let mut patched = bytes.to_vec();
    let field = field_spans(bytes)
        .into_iter()
        .find(|field| field.name == name)
        .expect("field to flag");
    let current = u16_at(bytes, field.flags);
    let flags = if set { current | mask } else { current & !mask };
    patched[field.flags..field.flags + 2].copy_from_slice(&flags.to_be_bytes());
    patched
}

fn with_field_descriptor(bytes: &[u8], name: &[u8], descriptor: &[u8]) -> Vec<u8> {
    let (entries, pool_end) = constant_pool(bytes);
    let count = u16_at(bytes, 8);
    let mut patched = bytes.to_vec();
    let mut entry = vec![
        1,
        0,
        u8::try_from(descriptor.len()).expect("short descriptor"),
    ];
    entry.extend_from_slice(descriptor);
    patched.splice(pool_end..pool_end, entry);
    patched[8..10].copy_from_slice(&(count + 1).to_be_bytes());
    let field = field_spans(&patched)
        .into_iter()
        .find(|field| field.name == name)
        .expect("field to re-describe");
    let new_index = count;
    assert_eq!(entries.len(), usize::from(new_index));
    patched[field.descriptor_index..field.descriptor_index + 2]
        .copy_from_slice(&new_index.to_be_bytes());
    patched
}

fn with_constant_value(bytes: &[u8], name: &[u8]) -> Vec<u8> {
    let (entries, _) = constant_pool(bytes);
    let constant_name = utf8_index(bytes, b"ConstantValue");
    let constant_value = entries
        .iter()
        .enumerate()
        .find_map(|(index, entry)| {
            let entry = entry.as_ref()?;
            (entry.tag == 3 && entry.payload == 17u32.to_be_bytes())
                .then(|| u16::try_from(index).expect("constant-pool index"))
        })
        .expect("fixture integer constant");
    let field = field_spans(bytes)
        .into_iter()
        .find(|field| field.name == name)
        .expect("field to annotate");
    let mut patched = bytes.to_vec();
    let count = u16_at(bytes, field.attributes_count);
    patched[field.attributes_count..field.attributes_count + 2]
        .copy_from_slice(&(count + 1).to_be_bytes());
    patched.splice(
        field.attributes_end..field.attributes_end,
        [
            (constant_name >> 8) as u8,
            constant_name as u8,
            0,
            0,
            0,
            2,
            (constant_value >> 8) as u8,
            constant_value as u8,
        ],
    );
    patched
}

fn with_other_owner(bytes: &[u8], name: &[u8], owner: &[u8]) -> Vec<u8> {
    let (entries, _) = constant_pool(bytes);
    let owner_name = utf8_index(bytes, owner);
    let owner_class = entries
        .iter()
        .enumerate()
        .find_map(|(index, entry)| {
            let entry = entry.as_ref()?;
            (entry.tag == 7 && u16_at(&entry.payload, 0) == owner_name)
                .then(|| u16::try_from(index).expect("constant-pool index"))
        })
        .expect("fixture owner class");
    let name_index = utf8_index(bytes, name);
    let descriptor_index = utf8_index(bytes, b"I");
    let name_type = entries
        .iter()
        .enumerate()
        .find_map(|(index, entry)| {
            let entry = entry.as_ref()?;
            (entry.tag == 12
                && u16_at(&entry.payload, 0) == name_index
                && u16_at(&entry.payload, 2) == descriptor_index)
                .then(|| u16::try_from(index).expect("constant-pool index"))
        })
        .expect("fixture field name-and-type");
    let fieldref = entries
        .iter()
        .filter_map(Option::as_ref)
        .find(|entry| entry.tag == 9 && u16_at(&entry.payload, 2) == name_type)
        .expect("fixture field reference");
    let mut patched = bytes.to_vec();
    patched[fieldref.offset + 1..fieldref.offset + 3].copy_from_slice(&owner_class.to_be_bytes());
    patched
}

fn assert_first_write_remains_qualified(bytes: Vec<u8>, case: &str) {
    let report = class_source_of(&open(&bytes), RecoveryEvidenceRequest::all());
    let text = &recovered(&report, "<clinit>").text;
    assert!(
        text.lines()
            .any(|line| line.contains("first =") && line.contains(".")),
        "{case}: rejected proof must retain a qualified first write:\n{text}"
    );
    assert!(
        !text
            .lines()
            .any(|line| line.trim_start().starts_with("first =")),
        "{case}: rejected proof must not emit a bare first write:\n{text}"
    );
}

#[test]
fn final_static_proof_rejects_missing_mismatch_owner_ambiguity_nonfinal_and_constant() {
    assert_first_write_remains_qualified(
        with_field_name(FIXTURE, b"first", b"Code"),
        "missing declaration",
    );
    assert_first_write_remains_qualified(
        with_field_name(FIXTURE, b"first", b"local0"),
        "same-name ambiguity",
    );
    assert_first_write_remains_qualified(
        with_field_flags(FIXTURE, b"first", 0x0010, false),
        "non-final declaration",
    );
    assert_first_write_remains_qualified(
        with_field_descriptor(FIXTURE, b"first", b"J"),
        "descriptor mismatch",
    );
    assert_first_write_remains_qualified(
        with_constant_value(FIXTURE, b"first"),
        "ConstantValue declaration",
    );
    assert_first_write_remains_qualified(
        with_other_owner(FIXTURE, b"first", b"FinalStaticSupport"),
        "other owner",
    );
}

#[test]
fn fixture_keeps_the_static_final_and_instance_final_boundaries() {
    assert!(PROBE_SOURCE.contains("public static final int first;"));
    assert!(PROBE_SOURCE.contains("public static final int second;"));
    assert!(PROBE_SOURCE.contains("public static final int branchValue;"));
    assert!(PROBE_SOURCE.contains("public static final int local0;"));
    assert!(PROBE_SOURCE.contains("public static final int local0_2;"));
    assert!(PROBE_SOURCE.contains("public static final int constantValue = 17;"));
    assert!(PROBE_SOURCE.contains("private final int instanceValue;"));
    assert!(PROBE_SOURCE.contains("if (FinalStaticSupport.branch())"));
    assert!(SUPPORT_SOURCE.contains("public static int next(String name)"));
    assert!(RUNNER_SOURCE.contains("System.setProperty(\"jarde.final-static.branch\", args[0])"));

    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);
    assert_eq!(
        report.fields.len(),
        8,
        "seven static fields and one instance final"
    );
    assert!(
        report
            .fields
            .iter()
            .any(|field| field.declaration.as_deref()
                == Some("public static final int constantValue = 17"))
    );
    assert!(
        report
            .fields
            .iter()
            .any(|field| field.declaration.as_deref() == Some("private final int instanceValue"))
    );
}

#[test]
fn clinit_uses_simple_blank_final_writes_and_reserves_local_names() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);
    for name in ["<init>", "<clinit>"] {
        let body = recovered(&report, name);
        assert!(
            body.fields.iter().any(|field| field.access == "write"),
            "{name} has a real field write"
        );
        assert!(
            body.fields.iter().all(|field| field.presented),
            "{name}: {:?}",
            body.fields
        );
    }
    let text = &recovered(&report, "<clinit>").text;

    for name in [
        "first",
        "second",
        "local0",
        "local0_2",
        "branchValue",
        "afterAssign",
    ] {
        assert!(
            text.lines()
                .any(|line| line.trim_start().starts_with(&format!("{name} ="))),
            "the blank final `{name}` is written by its simple name:\n{text}"
        );
        assert!(
            !text.contains(&format!("FinalStaticProbe.{name} =")),
            "the blank final `{name}` kept a qualified receiver:\n{text}"
        );
    }

    let first = text.find("first =").expect("first write");
    let second = text.find("second =").expect("second write");
    let local = text.find("local0 =").expect("local0 field write");
    let local_2 = text.find("local0_2 =").expect("local0_2 field write");
    let branch = text.find("branchValue =").expect("branch write");
    let after = text
        .find("afterAssign =")
        .expect("post-assignment read/write");
    assert!(
        first < second && second < local && local < local_2 && local_2 < branch && branch < after
    );
    assert_eq!(
        text.matches("branchValue =").count(),
        2,
        "both branch arms write one final"
    );
    let after_line = text
        .lines()
        .find(|line| line.trim_start().starts_with("afterAssign ="))
        .expect("the post-assignment read/write line");
    for name in ["first", "second", "branchValue"] {
        assert!(
            after_line.contains(name),
            "the post-assignment read mentions `{name}`: {after_line}"
        );
    }

    let local_declaration = text
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("int local"))
        .unwrap_or_else(|| panic!("the no-debug local declaration is missing:\n{text}"));
    let local_name = local_declaration
        .strip_prefix("int ")
        .and_then(|name| name.split([' ', '=', ';']).next())
        .expect("the local declaration has a name");
    assert_ne!(
        local_name, "local0",
        "the generated local must not shadow the field"
    );
    assert_ne!(
        local_name, "local0_2",
        "the generated local must not shadow the second field"
    );
}

#[test]
fn essential_sources_match_all_sources_but_publish_no_source_map() {
    let snapshot = open(FIXTURE);
    let essential = class_source_of(&snapshot, RecoveryEvidenceRequest::essential());
    let all = class_source_of(&snapshot, RecoveryEvidenceRequest::all());
    assert_eq!(
        essential.text, all.text,
        "evidence selection does not change committed source"
    );
    all_recovered(&essential);
    all_recovered(&all);

    for name in [
        "<init>",
        "instanceValue",
        "readAfterAssign",
        "snapshot",
        "<clinit>",
    ] {
        let essential_run = recovered(&essential, name);
        assert!(
            essential_run.source_map.is_empty(),
            "essential `{name}` published source map"
        );
        assert_eq!(
            essential_run
                .evidence
                .state(RecoveryEvidenceKind::SourceMap),
            EvidenceState::NotRequested,
            "essential `{name}` source-map status"
        );

        let all_run = recovered(&all, name);
        assert_eq!(
            all_run.evidence.requested(),
            &RecoveryEvidenceRequest::all(),
            "all `{name}` must echo the complete source request"
        );
        assert!(
            !all_run.source_map.is_empty(),
            "all `{name}` has no source-map segments"
        );
        assert_eq!(
            all_run.evidence.state(RecoveryEvidenceKind::SourceMap),
            EvidenceState::Complete,
            "all `{name}` source-map status"
        );
        if name == "<clinit>" {
            for (write, producer) in [
                (5, 2),
                (13, 10),
                (23, 18),
                (31, 28),
                (45, 42),
                (56, 53),
                (70, 59),
            ] {
                assert!(
                    !all_run.source_map.direct_of_bci(write).is_empty(),
                    "the real field write BCI {write} has no direct source"
                );
                assert!(
                    !all_run.source_map.of_bci(producer).is_empty(),
                    "the producer BCI {producer} has no source coverage"
                );
            }
        }
        for segment in all_run.source_map.segments() {
            let origin = segment.origin();
            assert!(!origin.bcis().is_empty(), "segment has no BCI: {origin:?}");
            assert!(
                origin.primary().member().is_some(),
                "segment has no member: {origin:?}"
            );
            for derived in origin.derived() {
                assert!(
                    derived.member().is_some(),
                    "derived origin has no member: {derived:?}"
                );
            }
        }
    }
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-final-static-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the JDK comparison directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_sources(dir: &Path, probe: &str) {
    fs::write(dir.join("FinalStaticProbe.java"), probe).expect("write the probe source");
    fs::write(dir.join("FinalStaticSupport.java"), SUPPORT_SOURCE)
        .expect("write the source-only helper");
    fs::write(dir.join("FinalStaticRunner.java"), RUNNER_SOURCE)
        .expect("write the source-only runner");
}

fn javac(dir: &Path, probe: &str) {
    fs::create_dir_all(dir).expect("create the javac directory");
    write_sources(dir, probe);
    let output = std::process::Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-g:none")
        .arg("-d")
        .arg(dir)
        .args([
            "FinalStaticSupport.java",
            "FinalStaticProbe.java",
            "FinalStaticRunner.java",
        ])
        .current_dir(dir)
        .output()
        .expect("start javac");
    assert!(
        output.status.success(),
        "compiling generated recovery failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_runner(dir: &Path, branch: bool) -> String {
    let output = std::process::Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg("FinalStaticRunner")
        .arg(branch.to_string())
        .current_dir(dir)
        .output()
        .expect("execute the fixture runner");
    assert!(
        output.status.success(),
        "the fixture runner failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the fixture runner writes UTF-8")
}

#[test]
#[ignore = "requires JDK: compile and execute the original and recovered complete classes"]
fn recovered_complete_class_matches_original_in_both_clinit_branches() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);

    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered_dir = scratch.path().join("recovered");

    // Compile the source-only helper and runner, then put the exact frozen fixture bytes back in
    // place.  This keeps the original side's JVM execution tied to the committed class, while the
    // recovered side is compiled from the full Engine output.
    javac(&original, PROBE_SOURCE);
    fs::write(original.join("FinalStaticProbe.class"), FIXTURE)
        .expect("install the committed original class bytes");
    let original_true = run_runner(&original, true);
    let original_false = run_runner(&original, false);

    javac(&recovered_dir, &report.text);
    let recovered_true = run_runner(&recovered_dir, true);
    let recovered_false = run_runner(&recovered_dir, false);

    assert_eq!(
        original_true,
        "1:2:5:3:4:8:17\nfirst,second,local,local0_2,branch-true\nread=8\ninstance=41\n"
    );
    assert_eq!(
        original_false,
        "1:2:5:3:4:8:17\nfirst,second,local,local0_2,branch-false\nread=8\ninstance=41\n"
    );
    assert_eq!(recovered_true, original_true, "true branch runtime result");
    assert_eq!(
        recovered_false, original_false,
        "false branch runtime result"
    );
}
