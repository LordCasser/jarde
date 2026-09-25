//! The `recover-narrow-array-stores` sink rule: the exact `bastore`/`castore`/`sastore` instruction
//! and the array's own proven component together authorize a cast around only the stored value.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] = include_bytes!("fixtures/p3-narrow-array-stores/v8/NarrowArrayStores.class");
const SOURCE: &str = include_str!("fixtures/p3-narrow-array-stores/NarrowArrayStores.java");
const EFFECTS: &str = include_str!("fixtures/p3-narrow-array-stores/NarrowArrayStoreEffects.java");
const RUNNER: &str = include_str!("fixtures/p3-narrow-array-stores/NarrowArrayStoresRunner.java");

const METHODS: &[(&[u8], &[u8])] = &[
    (b"<init>", b"()V"),
    (b"storeByte", b"([BII)V"),
    (b"storeChar", b"([CII)V"),
    (b"storeShort", b"([SII)V"),
    (b"storeByteProduced", b"([BIIZ)V"),
    (b"storeCharProduced", b"([CIIZ)V"),
    (b"storeShortProduced", b"([SIIZ)V"),
    (b"ordinaryByte", b"([B)V"),
    (b"ordinaryChar", b"([C)V"),
    (b"ordinaryShort", b"([S)V"),
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
        class_headers: 10,
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut Budget::new(limits()),
        )
        .expect("the patched Java 8 class opens")
}

fn boolean_boundary_fixture() -> Vec<u8> {
    rewrite_descriptor(FIXTURE, b"([BIIZ)V", b"([ZIIZ)V")
}

fn boolean_value_boundary_fixture() -> Vec<u8> {
    rewrite_descriptor(FIXTURE, b"([BII)V", b"([BIZ)V")
}

fn rewrite_descriptor(fixture: &[u8], source: &[u8], target: &[u8]) -> Vec<u8> {
    let mut bytes = fixture.to_vec();
    assert_eq!(
        source.len(),
        target.len(),
        "constant-pool entry size stays fixed"
    );
    assert_eq!(
        bytes
            .windows(source.len())
            .filter(|window| *window == source)
            .count(),
        1,
        "the requested member descriptor has one frozen occurrence"
    );
    let start = bytes
        .windows(source.len())
        .position(|window| window == source)
        .expect("the produced B writer descriptor is present");
    bytes[start..start + source.len()].copy_from_slice(target);
    bytes
}

fn class_source(
    snapshot: &ArtifactSnapshot,
    evidence: RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("NarrowArrayStores"),
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
            &mut Budget::new(limits()),
        )
        .expect("the class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => {
            panic!(
                "one standalone class has {} candidates",
                candidates.candidates.len()
            )
        }
        OperationOutcome::Incomplete(candidates) => panic!(
            "one standalone class has an unfinished selection with {} candidates",
            candidates.candidates.len()
        ),
    }
}

fn recovery_request(
    snapshot: &ArtifactSnapshot,
    name: &[u8],
    descriptor: &[u8],
) -> MethodAnalysisRequest {
    let engine = Engine::new();
    let mut budget = Budget::new(limits());
    let inspected = engine
        .inspect_header(
            snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the fixture header is readable");
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
    MethodAnalysisRequest {
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
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: inspected.source.class_bytes,
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
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
                "missing NarrowArrayStores member `{}{}`",
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

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-narrow-array-stores-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create Java comparison directory");
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

fn compile_java(output: &Path, classpath: Option<&Path>, sources: &[&str]) {
    fs::create_dir_all(output).expect("create Java output directory");
    let mut command = std::process::Command::new("javac");
    command
        .args(["--release", "8", "-g:none", "-d"])
        .arg(output);
    if let Some(classpath) = classpath {
        command.arg("-cp").arg(classpath);
    }
    let result = command.args(sources).output().expect("start javac");
    assert!(
        result.status.success(),
        "javac failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

fn write_support_sources(directory: &Path, class_source: &str) -> Vec<String> {
    fs::create_dir_all(directory).expect("create Java source directory");
    let class = directory.join("NarrowArrayStores.java");
    let effects = directory.join("NarrowArrayStoreEffects.java");
    let runner = directory.join("NarrowArrayStoresRunner.java");
    fs::write(&class, class_source).expect("write NarrowArrayStores.java");
    fs::write(&effects, EFFECTS).expect("write NarrowArrayStoreEffects.java");
    fs::write(&runner, RUNNER).expect("write NarrowArrayStoresRunner.java");
    vec![
        class.to_string_lossy().into_owned(),
        effects.to_string_lossy().into_owned(),
        runner.to_string_lossy().into_owned(),
    ]
}

fn run_runner(classpath: &str, original: bool) -> String {
    let mut command = std::process::Command::new("java");
    command.args(["-Xverify:all", "-cp", classpath, "NarrowArrayStoresRunner"]);
    if original {
        command.arg("original");
    }
    let result = command.output().expect("start Java runtime");
    assert!(
        result.status.success(),
        "Java runner failed for classpath `{classpath}` (original={original}): {}",
        String::from_utf8_lossy(&result.stderr),
    );
    String::from_utf8(result.stdout).expect("runner output is UTF-8")
}

#[test]
fn proven_bcs_stores_cast_only_at_the_matching_store_and_keep_origins() {
    assert_eq!(FIXTURE.len(), 760);
    assert_eq!(&FIXTURE[0..4], &[0xca, 0xfe, 0xba, 0xbe]);
    assert_eq!(&FIXTURE[6..8], &[0, 52]);

    let snapshot = open(FIXTURE);
    let all = class_source(&snapshot, RecoveryEvidenceRequest::all());
    assert_eq!(all.methods.len(), METHODS.len());
    for &(name, descriptor) in METHODS {
        let method = member(&all, name, descriptor);
        let body = recovery(&all, name, descriptor);
        assert!(body.produced(), "{}\n{}", method.text, body.text);
        assert_eq!(body.representation, Representation::Java, "{}", method.text);
        assert_eq!(body.quality, Quality::Structured, "{}", method.text);
        assert!(!method.text.contains("@bytecode"), "{}", method.text);
    }

    for (name, descriptor, cast) in [
        (b"storeByte".as_slice(), b"([BII)V".as_slice(), "(byte)"),
        (b"storeChar", b"([CII)V", "(char)"),
        (b"storeShort", b"([SII)V", "(short)"),
        (b"storeByteProduced", b"([BIIZ)V", "(byte)"),
        (b"storeCharProduced", b"([CIIZ)V", "(char)"),
        (b"storeShortProduced", b"([SIIZ)V", "(short)"),
    ] {
        let method = member(&all, name, descriptor);
        let body = recovery(&all, name, descriptor);
        assert!(method.text.contains(cast), "{}", method.text);
        if name.ends_with(b"Produced") {
            assert_eq!(
                method.text.matches("NarrowArrayStoreEffects.value").count(),
                1,
                "{}",
                method.text
            );
        }
        let store_bci = if name.ends_with(b"Produced") { 7 } else { 3 };
        assert!(
            !body.source_map.direct_of_bci(store_bci).is_empty(),
            "store BCI {store_bci} has a direct source anchor: {}",
            body.text
        );
        if name.ends_with(b"Produced") {
            for bci in [0, 1, 4, 7] {
                assert!(
                    !body.text_of_bci(bci).is_empty(),
                    "array/index/producer/store BCI {bci} remains mapped: {}",
                    body.text
                );
            }
        } else {
            for bci in [0, 1, 2, 3] {
                assert!(
                    !body.text_of_bci(bci).is_empty(),
                    "array/index/value/store BCI {bci} remains mapped: {}",
                    body.text
                );
            }
        }
    }

    // Legal constant narrowing and already narrow source locals keep their existing spelling.
    for (name, descriptor, cast) in [
        (b"ordinaryByte".as_slice(), b"([B)V".as_slice(), "(byte)"),
        (b"ordinaryChar", b"([C)V", "(char)"),
        (b"ordinaryShort", b"([S)V", "(short)"),
    ] {
        let text = &member(&all, name, descriptor).text;
        assert_eq!(
            text.matches(cast).count(),
            1,
            "the int-shaped local is narrowed once: {text}"
        );
        assert!(text.contains("arg0[1] = "), "{text}");
        assert!(text.contains("arg0[2] = "), "{text}");
        assert!(
            !text.contains(&format!("{cast} 0")),
            "constant narrowing stays implicit: {text}"
        );
        assert!(
            !text.contains(&format!("{cast} 1")),
            "constant narrowing stays implicit: {text}"
        );
    }

    let default = class_source(&snapshot, RecoveryEvidenceRequest::essential());
    assert_eq!(
        default.text, all.text,
        "default and all evidence produce identical text"
    );
    let replay = class_source(&snapshot, RecoveryEvidenceRequest::all());
    assert_eq!(replay.text, all.text, "all evidence replay is stable");
}

#[test]
fn boolean_bastore_low_bit_keeps_the_value_and_store_sources() {
    let boundary = boolean_boundary_fixture();
    let snapshot = open(&boundary);
    let report = class_source(&snapshot, RecoveryEvidenceRequest::all());
    let body = recovery(&report, b"storeByteProduced", b"([ZIIZ)V");
    assert_eq!(body.representation, Representation::Java, "{}", body.text);
    assert_eq!(body.quality, Quality::Structured, "{}", body.text);
    assert!(body.text.contains("arg0[arg1] ="), "{}", body.text);
    assert!(body.text.contains("% 2 != 0;"), "{}", body.text);
    assert_eq!(body.text.matches("value(").count(), 1, "{}", body.text);
    for bci in [4, 7, 8] {
        assert!(
            !body.text_of_bci(bci).is_empty(),
            "low-bit store retains BCI {bci}: {}",
            body.text
        );
    }
    assert!(!body.source_map.direct_of_bci(7).is_empty());

    let boolean_value = boolean_value_boundary_fixture();
    let snapshot = open(&boolean_value);
    let report = class_source(&snapshot, RecoveryEvidenceRequest::all());
    let body = recovery(&report, b"storeByte", b"([BIZ)V");
    assert!(
        body.text.contains("presented as `boolean`"),
        "boolean is refused at the proven byte sink: {}",
        body.text
    );
    assert!(
        !body.text_of_bci(3).is_empty(),
        "numeric type refusal retains the actual store BCI 3: {}",
        body.text
    );
    assert!(!body.source_map.direct_of_bci(3).is_empty());
}

#[test]
fn insufficient_output_budget_and_cancellation_stop_the_method_run() {
    let snapshot = open(FIXTURE);
    let request = recovery_request(&snapshot, b"storeByteProduced", b"([BIIZ)V");
    let mut bounded = limits();
    bounded.output_bytes = 1;
    let limited = Engine::new()
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(bounded),
        )
        .expect("the bounded request is answered with its stop")
        .recovery()
        .clone();
    assert!(
        !limited.produced(),
        "a partial conversion must not publish: {}",
        limited.text
    );
    assert!(limited.stop().is_some(), "the budget stop is explicit");

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let cancelled = Engine::new()
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::with_cancellation_token(limits(), cancellation),
        )
        .expect("a pre-cancelled request is reported")
        .recovery()
        .clone();
    assert!(!cancelled.produced());
    assert!(cancelled.stop().is_some());
}

#[test]
#[ignore = "requires JDK; compares all 147 boundary, producer, exception-order and local-store rows"]
fn recovered_core_bcs_methods_match_the_patched_jvm() {
    let snapshot = open(FIXTURE);
    let recovered = class_source(&snapshot, RecoveryEvidenceRequest::all());
    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let patched = scratch.path().join("patched");
    let presented = scratch.path().join("presented");

    let original_sources = write_support_sources(&original, SOURCE);
    compile_java(
        &scratch.path().join("original-classes"),
        None,
        &original_sources
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    let original_output = run_runner(
        &scratch.path().join("original-classes").to_string_lossy(),
        true,
    );

    fs::create_dir_all(&patched).expect("create patched class directory");
    fs::write(patched.join("NarrowArrayStores.class"), FIXTURE)
        .expect("install patched Java 8 class");
    let patched_sources = write_support_sources(&patched, SOURCE);
    fs::remove_file(patched.join("NarrowArrayStores.java"))
        .expect("keep javac on the frozen patched class");
    compile_java(
        &scratch.path().join("patched-runner"),
        Some(&patched),
        &[patched_sources[1].as_str(), patched_sources[2].as_str()],
    );
    let patched_classpath = format!(
        "{}:{}:{}",
        scratch.path().join("patched-runner").display(),
        patched.display(),
        scratch.path().join("original-classes").display()
    );
    let patched_output = run_runner(&patched_classpath, false);
    assert_eq!(original_output.lines().count(), 147);
    assert_eq!(patched_output.lines().count(), 147);

    let presented_sources = write_support_sources(&presented, &recovered.text);
    compile_java(
        &scratch.path().join("presented-classes"),
        None,
        &presented_sources
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    let presented_output = run_runner(
        &scratch.path().join("presented-classes").to_string_lossy(),
        false,
    );
    assert_eq!(presented_output, patched_output);
}
