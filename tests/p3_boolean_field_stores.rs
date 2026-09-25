//! `recover-boolean-field-stores`: only a verified `Z` field-write position consumes an
//! int-shaped expression by its low bit, while the existing boolean proof and source map remain.

use jarde::*;
use jarde_reader::classfile::{CpEntryKind, class_facts};
use std::fs;
use std::path::{Path, PathBuf};
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] = include_bytes!("fixtures/p3-boolean-field-stores/v8/ZFieldStores.class");
const SOURCE: &str = include_str!("fixtures/p3-boolean-field-stores/ZFieldStores.java");
const EFFECTS_SOURCE: &str =
    include_str!("fixtures/p3-boolean-field-stores/ZFieldStoreEffects.java");
const RUNNER_SOURCE: &str =
    include_str!("fixtures/p3-boolean-field-stores/ZFieldStoresRunner.java");
const PRE_IMPLEMENTATION_RUNTIME: &str =
    include_str!("fixtures/p3-boolean-field-stores/pre-implementation-runtime.stdout");
const PATCHED_RUNTIME: &str =
    include_str!("fixtures/p3-boolean-field-stores/patched-runtime.stdout");
const CODE_HASH_EVIDENCE: &str = include_str!("fixtures/p3-boolean-field-stores/code-sha256.json");

const CODE_BLAKE3: &[(&str, &str, &str, usize)] = &[
    (
        "<init>",
        "()V",
        "715be7da3c3f0810b13ed1b7f8cdd81a18151a311698c98d3b729d057df6e4f2",
        5,
    ),
    (
        "putInstance",
        "(I)V",
        "9b828d792bac65bc561a9965940b715b9b217abb69c4e18be078cade7767bb51",
        6,
    ),
    (
        "putInstanceProduced",
        "(IZ)V",
        "835378cf3a0c776c17e011f76b2107354a58156e064a361f7d29567bee5bb6f9",
        10,
    ),
    (
        "putStatic",
        "(I)V",
        "c8a17f6d3f616d67d4dd913b2ccaee2db4bef6fb811e5c2f2f0b944cb2f7b26c",
        5,
    ),
    (
        "putStaticProduced",
        "(IZ)V",
        "d924754f55edc0f71d9ae4ad4ef69afe2bcc49de184611c2dd81012a1ba4d085",
        9,
    ),
    (
        "putOn",
        "(LZFieldStores;I)V",
        "9b828d792bac65bc561a9965940b715b9b217abb69c4e18be078cade7767bb51",
        6,
    ),
    (
        "putProducedOn",
        "(LZFieldStores;IZ)V",
        "835378cf3a0c776c17e011f76b2107354a58156e064a361f7d29567bee5bb6f9",
        10,
    ),
    (
        "putOrdinaryInstance",
        "(Z)V",
        "b3040f38e75e55d387f798dd83576e40591368fb4dcb3ada3cb7f8d95c3e20e2",
        6,
    ),
    (
        "putOrdinaryInstanceTrue",
        "()V",
        "c7fa4a222fd9c1931aec2db7eec3d1495638c0c5bfbdf89a80d76aec1c20c7e7",
        6,
    ),
    (
        "putOrdinaryStatic",
        "(Z)V",
        "3249903137f1849b33200b31bffddcd516fbce5ce5afd19de8ac19c540c6adfb",
        5,
    ),
    (
        "putOrdinaryStaticTrue",
        "()V",
        "cb3c7a58a9794134aa51fd52414ec843a7bb4a7effc0a561104a581a0e609747",
        5,
    ),
];

const METHODS: &[(&[u8], &[u8])] = &[
    (b"<init>", b"()V"),
    (b"putInstance", b"(I)V"),
    (b"putInstanceProduced", b"(IZ)V"),
    (b"putStatic", b"(I)V"),
    (b"putStaticProduced", b"(IZ)V"),
    (b"putOn", b"(LZFieldStores;I)V"),
    (b"putProducedOn", b"(LZFieldStores;IZ)V"),
    (b"putOrdinaryInstance", b"(Z)V"),
    (b"putOrdinaryInstanceTrue", b"()V"),
    (b"putOrdinaryStatic", b"(Z)V"),
    (b"putOrdinaryStaticTrue", b"()V"),
];

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 100,
        method_bodies: 100,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

fn budget() -> Budget {
    Budget::new(limits())
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the Z-field fixture opens as a standalone CLASS")
}

fn request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("ZFieldStores"),
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

fn class_source_of(
    snapshot: &ArtifactSnapshot,
    evidence: RecoveryEvidenceRequest,
) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request(snapshot),
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
        OperationOutcome::Incomplete(candidates) => {
            panic!("the Z-field fixture did not complete: {candidates:?}")
        }
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
                "no ZFieldStores member `{}{}`",
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

fn code_hashes(bytes: &[u8]) -> Vec<(String, String, String, usize)> {
    let facts = class_facts(bytes, &mut budget()).expect("the frozen class structure parses");
    facts
        .methods
        .iter()
        .map(|method| {
            let code = method
                .attributes
                .iter()
                .find(|attribute| attribute.name.raw().0 == b"Code")
                .expect("every frozen method has one Code attribute");
            let start = usize::try_from(code.content_span.start).expect("class span fits usize");
            let code_length = u32::from_be_bytes(
                bytes[start + 4..start + 8]
                    .try_into()
                    .expect("Code carries a four-byte array length"),
            ) as usize;
            let code_start = start + 8;
            let code_end = code_start + code_length;
            assert!(code_end <= start + code.content_span.length as usize);
            let code_bytes = &bytes[code_start..code_end];
            (
                String::from_utf8_lossy(&method.name.raw().0).into_owned(),
                String::from_utf8_lossy(&method.descriptor.raw().0).into_owned(),
                blake3::hash(code_bytes).to_hex().to_string(),
                code_length,
            )
        })
        .collect()
}

#[test]
fn frozen_fixture_pins_descriptors_code_and_the_preimplementation_red() {
    assert_eq!(&FIXTURE[..4], &[0xca, 0xfe, 0xba, 0xbe]);
    assert_eq!(&FIXTURE[6..8], &[0, 52]);
    assert!(SOURCE.contains("public int instanceFlag;"));
    assert!(SOURCE.contains("public static int staticFlag;"));
    assert!(SOURCE.contains("this.instanceFlag = ZFieldStoreEffects.value(value, fail);"));
    assert!(RUNNER_SOURCE.contains("Integer.MIN_VALUE"));
    assert!(RUNNER_SOURCE.contains("null-produced-fail"));
    assert!(EFFECTS_SOURCE.contains("calls++"));

    let facts = class_facts(FIXTURE, &mut budget()).expect("the class facts parse");
    assert_eq!(facts.major_version, 52);
    assert_eq!(facts.methods.len(), METHODS.len());
    for &(name, descriptor) in METHODS {
        assert!(
            facts.methods.iter().any(|method| {
                method.name.raw().0 == name && method.descriptor.raw().0 == descriptor
            }),
            "missing method {}{}",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(descriptor)
        );
    }
    for (name, descriptor) in [
        (b"instanceFlag".as_slice(), b"Z".as_slice()),
        (b"staticFlag".as_slice(), b"Z".as_slice()),
        (b"ordinaryInstanceFlag".as_slice(), b"Z".as_slice()),
        (b"ordinaryStaticFlag".as_slice(), b"Z".as_slice()),
    ] {
        assert!(
            facts.fields.iter().any(|field| {
                field.name.raw().0 == name && field.descriptor.raw().0 == descriptor
            })
        );
    }
    for (name, descriptor) in [
        (b"instanceFlag".as_slice(), b"Z".as_slice()),
        (b"staticFlag".as_slice(), b"Z".as_slice()),
    ] {
        assert!(facts.constant_pool.iter().any(|entry| {
            matches!(
                &entry.kind,
                CpEntryKind::FieldRef {
                    name: field_name,
                    descriptor: field_descriptor,
                    ..
                } if field_name.0 == name && field_descriptor.0 == descriptor
            )
        }));
    }

    let hashes = code_hashes(FIXTURE);
    assert_eq!(hashes.len(), 11);
    assert_eq!(
        hashes,
        CODE_BLAKE3
            .iter()
            .map(|(name, descriptor, hash, length)| (
                (*name).to_owned(),
                (*descriptor).to_owned(),
                (*hash).to_owned(),
                *length,
            ))
            .collect::<Vec<_>>()
    );
    let evidence: serde_json::Value =
        serde_json::from_str(CODE_HASH_EVIDENCE).expect("the preserved Code SHA evidence parses");
    let records = evidence.as_array().expect("the evidence is a method list");
    assert_eq!(records.len(), hashes.len());
    for (record, (name, descriptor, _, code_length)) in records.iter().zip(&hashes) {
        assert_eq!(record["method"].as_str(), Some(name.as_str()));
        assert_eq!(record["descriptor"].as_str(), Some(descriptor.as_str()));
        assert_eq!(record["code_length"].as_u64(), Some(*code_length as u64));
        assert_eq!(record["source_code_sha256"], record["patched_code_sha256"]);
        assert_eq!(record["unchanged"], true);
    }

    assert_eq!(
        PRE_IMPLEMENTATION_RUNTIME.lines().count(),
        PATCHED_RUNTIME.lines().count()
    );
    assert_eq!(PATCHED_RUNTIME.lines().count(), 40);
    let different_lines = PRE_IMPLEMENTATION_RUNTIME
        .lines()
        .zip(PATCHED_RUNTIME.lines())
        .filter(|(before, after)| before != after)
        .count();
    assert_eq!(
        different_lines, 26,
        "the frozen pre-fix runtime remains RED"
    );
    assert_eq!(
        include_str!("fixtures/p3-boolean-field-stores/pre-implementation-red.txt").trim(),
        "status: RED\njarde-runtime-status: 0\npatched-runtime-status: 0\njarde-vs-patched-different-lines: 26"
    );
}

#[test]
fn z_field_consumers_keep_boolean_proofs_and_source_origins() {
    let snapshot = open(FIXTURE);
    let report = class_source_of(&snapshot, RecoveryEvidenceRequest::all());
    let default = class_source_of(&snapshot, RecoveryEvidenceRequest::essential());
    assert_eq!(default.text, report.text, "default and all text must agree");
    assert_eq!(report.methods.len(), METHODS.len());

    for &(name, descriptor) in METHODS {
        let method = member(&report, name, descriptor);
        let body = recovery(&report, name, descriptor);
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
    }

    for (name, descriptor) in [
        (b"putInstance".as_slice(), b"(I)V".as_slice()),
        (b"putInstanceProduced".as_slice(), b"(IZ)V".as_slice()),
        (b"putStatic".as_slice(), b"(I)V".as_slice()),
        (b"putStaticProduced".as_slice(), b"(IZ)V".as_slice()),
        (b"putOn".as_slice(), b"(LZFieldStores;I)V".as_slice()),
        (
            b"putProducedOn".as_slice(),
            b"(LZFieldStores;IZ)V".as_slice(),
        ),
    ] {
        let text = &member(&report, name, descriptor).text;
        assert!(
            text.contains("% 2 != 0"),
            "{}{}: {text}",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(descriptor)
        );
    }

    for (name, descriptor) in [
        (b"putOrdinaryInstance".as_slice(), b"(Z)V".as_slice()),
        (b"putOrdinaryInstanceTrue".as_slice(), b"()V".as_slice()),
        (b"putOrdinaryStatic".as_slice(), b"(Z)V".as_slice()),
        (b"putOrdinaryStaticTrue".as_slice(), b"()V".as_slice()),
    ] {
        let text = &member(&report, name, descriptor).text;
        assert!(
            !text.contains("% 2"),
            "existing boolean proof changed: {text}"
        );
    }

    for (name, descriptor, producer_bci, put_bci) in [
        (b"putInstanceProduced".as_slice(), b"(IZ)V".as_slice(), 3, 6),
        (b"putStaticProduced".as_slice(), b"(IZ)V".as_slice(), 2, 5),
        (
            b"putProducedOn".as_slice(),
            b"(LZFieldStores;IZ)V".as_slice(),
            3,
            6,
        ),
    ] {
        let body = recovery(&report, name, descriptor);
        assert!(
            !body.source_map.direct_of_bci(put_bci).is_empty(),
            "put at BCI {put_bci} lost its direct origin: {}",
            body.text
        );
        assert!(
            !body.text_of_bci(producer_bci).is_empty(),
            "producer at BCI {producer_bci} disappeared: {}",
            body.text
        );
        let text = &member(&report, name, descriptor).text;
        assert_eq!(
            text.matches("ZFieldStoreEffects.value").count(),
            1,
            "{text}"
        );
    }
}

#[test]
fn z_field_budget_and_cancellation_stops_publish_no_partial_class() {
    let snapshot = open(FIXTURE);
    let engine = Engine::new();
    let stopped = engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot),
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(Limits {
                output_bytes: 0,
                ..limits()
            }),
        )
        .expect("the exhausted output budget is an operation outcome");
    assert!(
        matches!(stopped, OperationOutcome::Incomplete(_)),
        "zero output budget publishes no complete class: {stopped:?}"
    );

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot),
            &RecoveryEvidenceRequest::all(),
            &mut Budget::with_cancellation_token(limits(), token),
        )
        .expect("cancellation is an operation outcome");
    assert!(
        matches!(cancelled, OperationOutcome::Incomplete(_)),
        "cancellation publishes no complete class: {cancelled:?}"
    );
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-boolean-field-stores-{}-{nonce}",
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

fn write_support(dir: &Path) {
    fs::write(dir.join("ZFieldStoreEffects.java"), EFFECTS_SOURCE)
        .expect("write ZFieldStoreEffects.java");
    fs::write(dir.join("ZFieldStoresRunner.java"), RUNNER_SOURCE)
        .expect("write ZFieldStoresRunner.java");
}

fn javac(dir: &Path, source_files: &[&str], classpath: Option<&Path>) {
    let mut command = std::process::Command::new("javac");
    command.args(["--release", "8", "-g:none", "-d"]).arg(dir);
    if let Some(classpath) = classpath {
        command.arg("-cp").arg(classpath);
    }
    let output = command
        .args(source_files)
        .current_dir(dir)
        .output()
        .expect("start javac");
    assert!(
        output.status.success(),
        "javac failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_runner(dir: &Path) -> String {
    let output = std::process::Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(dir)
        .arg("ZFieldStoresRunner")
        .current_dir(dir)
        .output()
        .expect("execute the Z-field runner");
    assert!(
        output.status.success(),
        "the Z-field runner failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the runner writes UTF-8")
}

#[test]
#[ignore = "requires JDK: compile the complete recovered class and compare its 40 rows with the patched JVM"]
fn recovered_z_field_stores_match_the_patched_jvm() {
    let snapshot = open(FIXTURE);
    let report = class_source_of(&snapshot, RecoveryEvidenceRequest::all());
    let scratch = Scratch::new();
    let source = scratch.path().join("source");
    let patched = scratch.path().join("patched");
    let recovered = scratch.path().join("recovered");
    for dir in [&source, &patched, &recovered] {
        fs::create_dir_all(dir).expect("create comparison directory");
    }

    fs::write(source.join("ZFieldStores.java"), SOURCE).expect("write source class");
    write_support(&source);
    javac(
        &source,
        &[
            "ZFieldStoreEffects.java",
            "ZFieldStores.java",
            "ZFieldStoresRunner.java",
        ],
        None,
    );
    let source_output = run_runner(&source);

    fs::write(patched.join("ZFieldStores.class"), FIXTURE).expect("write frozen patched class");
    write_support(&patched);
    javac(
        &patched,
        &["ZFieldStoreEffects.java", "ZFieldStoresRunner.java"],
        Some(&patched),
    );
    let patched_output = run_runner(&patched);
    assert_eq!(patched_output, PATCHED_RUNTIME, "frozen patched JVM oracle");

    fs::write(recovered.join("ZFieldStores.java"), &report.text)
        .expect("write the complete recovered class");
    write_support(&recovered);
    javac(
        &recovered,
        &[
            "ZFieldStoreEffects.java",
            "ZFieldStores.java",
            "ZFieldStoresRunner.java",
        ],
        None,
    );
    let recovered_output = run_runner(&recovered);
    assert_eq!(source_output.lines().count(), 40);
    assert_eq!(recovered_output, patched_output);
    assert_eq!(recovered_output.lines().count(), 40);
}
