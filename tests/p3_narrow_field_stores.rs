//! `recover-narrow-field-stores`: B/C/S field descriptors require a narrowing cast at the write.
//!
//! The committed class keeps `int` source declarations in its Java inputs and patches only the
//! six target field descriptors plus their `Fieldref` name-and-type descriptors.  The JVM therefore
//! executes real byte/char/short `putfield` and `putstatic` writes while the producer remains an
//! int-shaped value. The recovered class must write the field after one producer evaluation in the
//! original exception order; the ignored test compares it with the patched JVM oracle's complete
//! 267-line output.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] = include_bytes!("fixtures/p3-narrow-field-stores/v8/NarrowFieldStores.class");
const SOURCE: &str = include_str!("fixtures/p3-narrow-field-stores/NarrowFieldStores.java");
const EFFECTS_SOURCE: &str =
    include_str!("fixtures/p3-narrow-field-stores/NarrowFieldStoreEffects.java");
const RUNNER_SOURCE: &str =
    include_str!("fixtures/p3-narrow-field-stores/NarrowFieldStoresRunner.java");

const METHODS: &[(&[u8], &[u8])] = &[
    (b"<init>", b"()V"),
    (b"setByte", b"(I)V"),
    (b"setChar", b"(I)V"),
    (b"setShort", b"(I)V"),
    (b"setByteProduced", b"(IZ)V"),
    (b"setCharProduced", b"(IZ)V"),
    (b"setShortProduced", b"(IZ)V"),
    (b"setByteProducedOn", b"(LNarrowFieldStores;IZ)V"),
    (b"setCharProducedOn", b"(LNarrowFieldStores;IZ)V"),
    (b"setShortProducedOn", b"(LNarrowFieldStores;IZ)V"),
    (b"setStaticByte", b"(I)V"),
    (b"setStaticChar", b"(I)V"),
    (b"setStaticShort", b"(I)V"),
    (b"setStaticByteProduced", b"(IZ)V"),
    (b"setStaticCharProduced", b"(IZ)V"),
    (b"setStaticShortProduced", b"(IZ)V"),
    (b"ordinaryByteLocal", b"()V"),
    (b"ordinaryCharLocal", b"()V"),
    (b"ordinaryShortLocal", b"()V"),
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the narrow-field fixture opens as a standalone CLASS")
}

fn class_source_of(
    snapshot: &ArtifactSnapshot,
    evidence: RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("NarrowFieldStores"),
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
        .expect("the class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one standalone fixture answers one definition, got {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one standalone fixture answers one definition, got an unfinished selection with {} candidates",
            candidates.candidates.len()
        ),
    }
}

fn member<'a>(
    report: &'a ClassSourceReport,
    name: &[u8],
    descriptor: &[u8],
) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == name && method.item.descriptor.raw().0 == descriptor
        })
        .unwrap_or_else(|| {
            panic!(
                "no NarrowFieldStores member `{}{}`",
                String::from_utf8_lossy(name),
                String::from_utf8_lossy(descriptor)
            )
        })
}

fn recovery<'a>(
    report: &'a ClassSourceReport,
    name: &[u8],
    descriptor: &[u8],
) -> &'a RecoveryReport {
    match &member(report, name, descriptor).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!(
            "`{}{}` has no recovery report: {other:?}",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(descriptor)
        ),
    }
}

fn assert_structured(report: &ClassSourceReport, name: &[u8], descriptor: &[u8]) -> String {
    let method = member(report, name, descriptor);
    let body = recovery(report, name, descriptor);
    assert!(body.produced(), "{}", method.text);
    assert_eq!(body.representation, Representation::Java, "{}", method.text);
    assert_eq!(body.quality, Quality::Structured, "{}", method.text);
    assert_eq!(
        body.content,
        RecoveryContent::ContainsStatements,
        "{}",
        method.text
    );
    assert!(!method.text.contains("@bytecode"), "{}", method.text);
    method.text.clone()
}

fn field<'a>(report: &'a ClassSourceReport, name: &[u8]) -> &'a ClassSourceField {
    report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == name)
        .unwrap_or_else(|| panic!("no field `{}`", String::from_utf8_lossy(name)))
}

#[test]
fn patched_field_fixture_has_real_bcs_descriptors_and_all_method_shapes() {
    assert_eq!(FIXTURE.len(), 1526);
    assert_eq!(&FIXTURE[0..4], &[0xca, 0xfe, 0xba, 0xbe]);
    assert_eq!(&FIXTURE[6..8], &[0, 52]);
    assert!(SOURCE.contains("this.byteField = value"));
    assert!(SOURCE.contains("NarrowFieldStoreEffects.value(value, fail)"));
    assert!(RUNNER_SOURCE.contains("null-fail"));
    assert!(RUNNER_SOURCE.contains("Integer.MIN_VALUE"));

    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    assert_eq!(report.methods.len(), METHODS.len());
    for &(name, descriptor) in METHODS {
        assert_eq!(
            member(&report, name, descriptor).item.descriptor.raw().0,
            descriptor
        );
    }
    for (name, descriptor) in [
        (b"byteField".as_slice(), b"B".as_slice()),
        (b"charField".as_slice(), b"C".as_slice()),
        (b"shortField".as_slice(), b"S".as_slice()),
        (b"staticByteField".as_slice(), b"B".as_slice()),
        (b"staticCharField".as_slice(), b"C".as_slice()),
        (b"staticShortField".as_slice(), b"S".as_slice()),
        (b"ordinaryByteField".as_slice(), b"B".as_slice()),
        (b"ordinaryCharField".as_slice(), b"C".as_slice()),
        (b"ordinaryShortField".as_slice(), b"S".as_slice()),
    ] {
        assert_eq!(field(&report, name).item.descriptor.raw().0, descriptor);
    }

    let default = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::essential());
    assert_eq!(default.text, report.text, "default and all text must agree");
}

#[test]
fn narrowed_field_stores_preserve_value_casts_and_producer_order() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    for &(name, descriptor) in METHODS {
        assert_structured(&report, name, descriptor);
    }

    for (name, descriptor, field_name, cast) in [
        (
            b"setByte".as_slice(),
            b"(I)V".as_slice(),
            "byteField",
            "(byte)",
        ),
        (
            b"setChar".as_slice(),
            b"(I)V".as_slice(),
            "charField",
            "(char)",
        ),
        (
            b"setShort".as_slice(),
            b"(I)V".as_slice(),
            "shortField",
            "(short)",
        ),
    ] {
        let text = member(&report, name, descriptor).text.as_str();
        assert!(text.contains(&format!("{field_name} =")), "{text}");
        assert!(text.contains(cast), "{text}");
    }
    for (name, descriptor, field_name, cast) in [
        (
            b"setByteProduced".as_slice(),
            b"(IZ)V".as_slice(),
            "byteField",
            "(byte)",
        ),
        (
            b"setCharProduced".as_slice(),
            b"(IZ)V".as_slice(),
            "charField",
            "(char)",
        ),
        (
            b"setShortProduced".as_slice(),
            b"(IZ)V".as_slice(),
            "shortField",
            "(short)",
        ),
    ] {
        let text = member(&report, name, descriptor).text.as_str();
        assert!(text.contains(&format!("{field_name} =")), "{text}");
        assert!(text.contains(cast), "{text}");
        assert_eq!(
            text.matches("NarrowFieldStoreEffects.value").count(),
            1,
            "{text}"
        );
    }
    for (name, field_name, cast) in [
        ("setStaticByte", "staticByteField", "(byte)"),
        ("setStaticChar", "staticCharField", "(char)"),
        ("setStaticShort", "staticShortField", "(short)"),
    ] {
        let text = member(&report, name.as_bytes(), b"(I)V").text.as_str();
        assert!(text.contains(&format!("{field_name} =")), "{text}");
        assert!(text.contains(cast), "{text}");
    }
    for (name, field_name, cast) in [
        ("setStaticByteProduced", "staticByteField", "(byte)"),
        ("setStaticCharProduced", "staticCharField", "(char)"),
        ("setStaticShortProduced", "staticShortField", "(short)"),
    ] {
        let text = member(&report, name.as_bytes(), b"(IZ)V").text.as_str();
        assert!(text.contains(&format!("{field_name} =")), "{text}");
        assert!(text.contains(cast), "{text}");
        assert_eq!(
            text.matches("NarrowFieldStoreEffects.value").count(),
            1,
            "{text}"
        );
    }

    for (name, descriptor, producer, store) in [
        (b"setByteProduced".as_slice(), b"(IZ)V".as_slice(), 3, 6),
        (b"setByteProducedOn", b"(LNarrowFieldStores;IZ)V", 3, 6),
        (b"setStaticByteProduced", b"(IZ)V", 2, 5),
    ] {
        let body = recovery(&report, name, descriptor);
        assert!(
            !body.source_map.direct_of_bci(store).is_empty(),
            "put at BCI {store} lost its direct source anchor: {}",
            body.text
        );
        assert!(
            !body.text_of_bci(producer).is_empty(),
            "call at BCI {producer} lost its source anchor: {}",
            body.text
        );
    }
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-narrow-field-stores-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create comparison directory");
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

fn write_sources(dir: &Path, class_source: &str) {
    fs::write(dir.join("NarrowFieldStores.java"), class_source)
        .expect("write NarrowFieldStores.java");
    fs::write(dir.join("NarrowFieldStoreEffects.java"), EFFECTS_SOURCE)
        .expect("write NarrowFieldStoreEffects.java");
    fs::write(dir.join("NarrowFieldStoresRunner.java"), RUNNER_SOURCE)
        .expect("write NarrowFieldStoresRunner.java");
}

fn javac(dir: &Path) {
    let output = std::process::Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(dir)
        .args([
            "NarrowFieldStoreEffects.java",
            "NarrowFieldStores.java",
            "NarrowFieldStoresRunner.java",
        ])
        .current_dir(dir)
        .output()
        .expect("start javac");
    assert!(
        output.status.success(),
        "compiling NarrowFieldStores failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_runner(dir: &Path) -> String {
    let output = std::process::Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(dir)
        .arg("NarrowFieldStoresRunner")
        .current_dir(dir)
        .output()
        .expect("execute the narrow-field runner");
    assert!(
        output.status.success(),
        "the narrow-field runner failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the narrow-field runner writes UTF-8")
}

#[test]
#[ignore = "requires JDK: compile the recovered class and compare its 267-line output with the patched class"]
fn recovered_narrow_field_stores_match_patched_runtime() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let patched = scratch.path().join("patched");
    let recovered = scratch.path().join("recovered");
    for dir in [&original, &patched, &recovered] {
        fs::create_dir_all(dir).expect("create comparison directory");
    }

    write_sources(&original, SOURCE);
    javac(&original);
    let original_output = run_runner(&original);

    write_sources(&patched, SOURCE);
    javac(&patched);
    fs::write(patched.join("NarrowFieldStores.class"), FIXTURE)
        .expect("install the frozen patched class");
    let patched_output = run_runner(&patched);
    assert_eq!(original_output.lines().count(), 267);
    assert_eq!(patched_output.lines().count(), 267);
    assert_ne!(
        original_output, patched_output,
        "B/C/S stores must truncate values"
    );
    for marker in [
        "B:instance:value:-32769:stored:-1",
        "C:instance:value:-1:stored:65535",
        "S:static:value:65536:stored:0",
        "B:instance-produced:producer-fail:error:java.lang.IllegalStateException:stored:77:calls:1",
        "C:instance-produced:null-fail:error:java.lang.IllegalStateException:calls:1",
        "S:instance-produced:null-ok:error:java.lang.NullPointerException:calls:1",
    ] {
        assert!(
            patched_output.contains(marker),
            "the patched JVM oracle covers `{marker}`:\n{patched_output}"
        );
    }

    write_sources(&recovered, &report.text);
    javac(&recovered);
    let recovered_output = run_runner(&recovered);
    assert_eq!(
        recovered_output, patched_output,
        "recovered field stores preserve narrowing, producer calls, and receiver order"
    );
}
