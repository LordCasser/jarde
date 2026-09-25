//! `recover-partial-array-allocations`: preserve the JVM shape of a partial array creation.
//!
//! The committed class is the minimized `core-no-overload` sample.  Its `anewarray` and
//! `multianewarray` instructions create arrays with an unallocated tail; the same producer is then
//! consumed by a return, local store/read, field store, length read, and a two-dimension effectful
//! allocation. The recovered class renders each allocation at its real consumer; the ignored
//! comparison test pins the final result to the frozen JVM oracle's 69 lines.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-partial-array-allocation/v8/PartialArrays.class");
const SOURCE: &str = include_str!("fixtures/p3-partial-array-allocation/PartialArrays.java");
const EFFECTS_SOURCE: &str =
    include_str!("fixtures/p3-partial-array-allocation/PartialArrayEffects.java");
const RUNNER_SOURCE: &str =
    include_str!("fixtures/p3-partial-array-allocation/PartialArraysRunner.java");

const METHODS: &[(&[u8], &[u8])] = &[
    (b"<init>", b"()V"),
    (b"primitive", b"(I)[[I"),
    (b"reference", b"(I)[[Ljava/lang/String;"),
    (b"prefix", b"(II)[[[I"),
    (b"referencePrefix", b"(II)[[[Ljava/lang/Object;"),
    (b"local", b"(I)[[I"),
    (b"field", b"(I)V"),
    (b"length", b"(I)I"),
    (b"effects", b"(II)[[[I"),
    (b"complete", b"(II)[[I"),
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the partial-array fixture opens as a standalone CLASS")
}

fn class_source_of(
    snapshot: &ArtifactSnapshot,
    evidence: RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("PartialArrays"),
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
                "no PartialArrays member `{}{}`",
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

#[test]
fn fixture_bytes_and_allocation_method_shapes_are_frozen() {
    assert_eq!(FIXTURE.len(), 777);
    assert_eq!(
        blake3::hash(FIXTURE).to_hex().as_str(),
        "aea6c2e7fb067ff82ba5547ad58d09c6145ad1dec9dcb7c8654ce757734b4ff4"
    );
    assert_eq!(&FIXTURE[0..4], &[0xca, 0xfe, 0xba, 0xbe]);
    assert_eq!(&FIXTURE[6..8], &[0, 52]);
    assert!(SOURCE.contains("new int[n][]"));
    assert!(SOURCE.contains("new String[n][]"));
    assert!(SOURCE.contains("new int[a][b][]"));
    assert!(SOURCE.contains("new Object[a][b][]"));
    assert!(SOURCE.contains("PartialArrayEffects.dim(1,a)"));
    assert!(RUNNER_SOURCE.contains("for(final int a:new int[]{-1,0,2}"));

    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    assert_eq!(report.methods.len(), METHODS.len());
    for &(name, descriptor) in METHODS {
        let method = member(&report, name, descriptor);
        assert_eq!(method.item.descriptor.raw().0, descriptor);
    }

    let default = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::essential());
    assert_eq!(default.text, report.text, "default and all text must agree");
}

#[test]
fn partial_array_creations_are_presented_at_their_real_consumers() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());

    let primitive = assert_structured(&report, b"primitive", b"(I)[[I");
    assert!(primitive.contains("return new int[arg0][];"), "{primitive}");
    let reference = assert_structured(&report, b"reference", b"(I)[[Ljava/lang/String;");
    assert!(
        reference.contains("return new java.lang.String[arg0][];"),
        "{reference}"
    );
    let prefix = assert_structured(&report, b"prefix", b"(II)[[[I");
    assert!(prefix.contains("return new int[arg0][arg1][];"), "{prefix}");
    let reference_prefix =
        assert_structured(&report, b"referencePrefix", b"(II)[[[Ljava/lang/Object;");
    assert!(
        reference_prefix.contains("return new java.lang.Object[arg0][arg1][];"),
        "{reference_prefix}"
    );

    let local = assert_structured(&report, b"local", b"(I)[[I");
    assert!(
        local.contains("int[][] local1 = new int[arg0][];"),
        "{local}"
    );
    assert!(local.contains("return local1;"), "{local}");
    let field = assert_structured(&report, b"field", b"(I)V");
    assert!(
        field.contains("PartialArrays.held = new int[arg0][];"),
        "{field}"
    );
    let length = assert_structured(&report, b"length", b"(I)I");
    assert!(
        length.contains("return new int[arg0][].length;"),
        "{length}"
    );

    let effects = assert_structured(&report, b"effects", b"(II)[[[I");
    assert!(
        effects.contains("PartialArrayEffects.dim(1, arg0)"),
        "{effects}"
    );
    assert!(
        effects.contains("PartialArrayEffects.dim(2, arg1)"),
        "{effects}"
    );
    assert!(effects.contains("return new int["), "{effects}");
    let complete = assert_structured(&report, b"complete", b"(II)[[I");
    assert!(
        complete.contains("return new int[arg0][arg1];"),
        "{complete}"
    );

    // The allocation site owns the expression, while each dimension producer remains traceable
    // through the composed expression. This is evidence from the committed javac Code, not text
    // matching a separately decompiled source file.
    for (name, descriptor, allocation, dimensions) in [
        (b"primitive".as_slice(), b"(I)[[I".as_slice(), 1, &[0][..]),
        (b"prefix", b"(II)[[[I", 2, &[0, 1][..]),
        (b"effects", b"(II)[[[I", 10, &[2, 7][..]),
    ] {
        let body = recovery(&report, name, descriptor);
        assert!(
            !body.source_map.direct_of_bci(allocation).is_empty(),
            "allocation at BCI {allocation} has no direct source anchor: {}",
            body.text
        );
        for &dimension in dimensions {
            assert!(
                !body.text_of_bci(dimension).is_empty(),
                "dimension at BCI {dimension} has no source anchor: {}",
                body.text
            );
        }
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
            "jarde-p3-partial-array-allocation-{}-{nonce}",
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

fn write_sources(dir: &Path, partial_arrays: &str) {
    fs::write(dir.join("PartialArrays.java"), partial_arrays).expect("write PartialArrays.java");
    fs::write(dir.join("PartialArrayEffects.java"), EFFECTS_SOURCE)
        .expect("write PartialArrayEffects.java");
    fs::write(dir.join("PartialArraysRunner.java"), RUNNER_SOURCE)
        .expect("write PartialArraysRunner.java");
}

fn javac(dir: &Path) {
    let output = std::process::Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(dir)
        .args([
            "PartialArrayEffects.java",
            "PartialArrays.java",
            "PartialArraysRunner.java",
        ])
        .current_dir(dir)
        .output()
        .expect("start javac");
    assert!(
        output.status.success(),
        "compiling PartialArrays failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_runner(dir: &Path) -> String {
    let output = std::process::Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(dir)
        .arg("PartialArraysRunner")
        .current_dir(dir)
        .output()
        .expect("execute the partial-array runner");
    assert!(
        output.status.success(),
        "the partial-array runner failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the partial-array runner writes UTF-8")
}

#[test]
#[ignore = "requires JDK: compile the recovered complete class and compare its 69-line runtime"]
fn recovered_partial_array_class_matches_original_runtime() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered = scratch.path().join("recovered");
    fs::create_dir_all(&original).expect("create original directory");
    fs::create_dir_all(&recovered).expect("create recovered directory");

    write_sources(&original, SOURCE);
    javac(&original);
    fs::write(original.join("PartialArrays.class"), FIXTURE)
        .expect("install the frozen original class");
    let original_output = run_runner(&original);
    assert_eq!(original_output.lines().count(), 69);
    for marker in [
        "primitive-1:0:java.lang.NegativeArraySizeException:false:0",
        "effects-1,-1:1:java.lang.IllegalStateException:true:1",
        "effects2,2:2:java.lang.IllegalStateException:true:12",
        "complete2,2:0:[[I:2:[I:2:0",
    ] {
        assert!(
            original_output.contains(marker),
            "the frozen original covers `{marker}`:\n{original_output}"
        );
    }

    write_sources(&recovered, &report.text);
    javac(&recovered);
    let recovered_output = run_runner(&recovered);
    assert_eq!(
        recovered_output, original_output,
        "the recovered class preserves partial allocation shapes and exception order"
    );
}
