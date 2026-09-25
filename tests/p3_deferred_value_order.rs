//! `preserve-deferred-value-order`: the patched class is the execution oracle.
//!
//! Each patched straight-line producer has an independent `mark()` call inserted after the value
//! is computed and before its original return.  The recovered source must save the value at that
//! point, then run the call, then return the saved value.  The nested, branch and null-prefix
//! methods remain ordinary source-shaped controls and are not patched.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-deferred-value-order/v8/DeferredValueOrder.class");
const PROBE_SOURCE: &str = include_str!("fixtures/p3-deferred-value-order/DeferredValueOrder.java");
const EFFECTS_SOURCE: &str = include_str!("fixtures/p3-deferred-value-order/DeferredEffects.java");
const VALUE_SOURCE: &str = include_str!("fixtures/p3-deferred-value-order/OrderValue.java");
const RUNNER_SOURCE: &str =
    include_str!("fixtures/p3-deferred-value-order/DeferredValueOrderRunner.java");

const METHODS: &[(&[u8], &[u8], &[u32])] = &[
    (b"<init>", b"()V", &[1]),
    (b"call", b"()I", &[0, 3]),
    (b"field", b"()I", &[0, 3]),
    (b"array", b"([I)I", &[2, 3]),
    (b"instanceField", b"(LOrderValue;)I", &[1, 4]),
    (b"arrayLength", b"([I)I", &[1, 2]),
    (b"newArray", b"(I)[I", &[1, 3]),
    (b"anewArray", b"(I)[Ljava/lang/String;", &[1, 4]),
    (b"multiArray", b"(I)[[I", &[2, 6]),
    (b"divInt", b"(I)I", &[4, 5]),
    (b"remInt", b"(I)I", &[4, 5]),
    (b"divLong", b"(J)J", &[4, 5]),
    (b"remLong", b"(J)J", &[4, 5]),
    (b"cast", b"(Ljava/lang/Object;)Ljava/lang/String;", &[1, 4]),
    (b"direct", b"()LOrderValue;", &[4, 7]),
    (b"nested", b"()I", &[0, 3]),
    (b"branch", b"([IZ)I", &[1, 7]),
    (b"prefix", b"([I)I", &[1, 7]),
    (b"keepPool", b"()V", &[0]),
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults to a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the deferred-value-order fixture opens as a standalone CLASS")
}

fn request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("DeferredValueOrder"),
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

fn class_source_of(snapshot: &ArtifactSnapshot) -> ClassSourceReport {
    let request = request(snapshot);
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("the all-evidence class-source request is answered")
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

fn class_source_without_evidence(snapshot: &ArtifactSnapshot) -> ClassSourceReport {
    let request = request(snapshot);
    match Engine::new()
        .class_source(slice::from_ref(snapshot), &request, &mut budget())
        .expect("the default class-source request is answered")
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
                "no member `{}{}` in DeferredValueOrder",
                String::from_utf8_lossy(name),
                String::from_utf8_lossy(descriptor)
            )
        })
}

fn recovered<'a>(
    report: &'a ClassSourceReport,
    name: &[u8],
    descriptor: &[u8],
) -> &'a RecoveryReport {
    match &member(report, name, descriptor).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!(
            "`{}{}` was not recovered: {other:?}",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(descriptor)
        ),
    }
}

fn assert_java_structured(method: &ClassSourceMethod, name: &[u8], descriptor: &[u8]) {
    assert!(!method.text.contains("@bytecode"), "{}", method.text);
    let report = match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("recovery outcome is not a report: {other:?}"),
    };
    assert_eq!(
        report.representation,
        Representation::Java,
        "{}",
        method.text
    );
    assert_eq!(report.quality, Quality::Structured, "{}", method.text);
    assert_eq!(
        report.content,
        RecoveryContent::ContainsStatements,
        "{}",
        method.text
    );
    assert!(
        report.produced(),
        "{}{} did not produce a recovery report: {:?}\n{}",
        String::from_utf8_lossy(name),
        String::from_utf8_lossy(descriptor),
        report.stop(),
        method.text
    );
}

fn assert_saved_before_mark(method: &ClassSourceMethod, producer: &str) {
    let text = &method.text;
    let producer_at = text
        .find(producer)
        .unwrap_or_else(|| panic!("producer `{producer}` is missing:\n{text}"));
    let mark_at = text
        .find("DeferredEffects.mark();")
        .unwrap_or_else(|| panic!("mark is missing after `{producer}`:\n{text}"));
    let return_at = text
        .find("return ")
        .unwrap_or_else(|| panic!("return is missing after `{producer}`:\n{text}"));
    assert!(
        producer_at < mark_at && mark_at < return_at,
        "the saved value, independent call and return are out of order:\n{text}"
    );
    assert_eq!(
        text.matches(producer).count(),
        1,
        "producer `{producer}` must be evaluated once:\n{text}"
    );
    assert_eq!(
        text.matches("DeferredEffects.mark();").count(),
        1,
        "mark must execute once:\n{text}"
    );
}

#[test]
fn deferred_values_keep_real_producer_order_and_structural_controls() {
    assert!(PROBE_SOURCE.contains("return DeferredEffects.value();"));
    assert!(PROBE_SOURCE.contains("return values.length;"));
    assert!(PROBE_SOURCE.contains("return holder.value;"));
    assert!(PROBE_SOURCE.contains("return new int[length];"));
    assert!(PROBE_SOURCE.contains("return DeferredEffects.left()+DeferredEffects.right();"));
    assert!(PROBE_SOURCE.contains("if(choose)return values[0]+DeferredEffects.right();"));
    assert!(PROBE_SOURCE.contains("if(values==null)return -1;"));
    assert!(VALUE_SOURCE.contains("public int value;"));
    assert!(RUNNER_SOURCE.contains("runZero"));
    assert!(RUNNER_SOURCE.contains("instanceNull"));

    let report = class_source_of(&open(FIXTURE));
    assert_eq!(report.methods.len(), METHODS.len());
    for &(name, descriptor, _) in METHODS {
        let method = member(&report, name, descriptor);
        assert_java_structured(method, name, descriptor);
    }

    for (name, descriptor, producer) in [
        (
            b"call".as_slice(),
            b"()I".as_slice(),
            "DeferredEffects.value()",
        ),
        (
            b"field".as_slice(),
            b"()I".as_slice(),
            "DeferredEffects.field",
        ),
        (b"array".as_slice(), b"([I)I".as_slice(), "arg0[0]"),
        (
            b"instanceField".as_slice(),
            b"(LOrderValue;)I".as_slice(),
            "arg0.value",
        ),
        (
            b"arrayLength".as_slice(),
            b"([I)I".as_slice(),
            "arg0.length",
        ),
        (b"newArray".as_slice(), b"(I)[I".as_slice(), "new int[arg0]"),
        (
            b"anewArray".as_slice(),
            b"(I)[Ljava/lang/String;".as_slice(),
            "new java.lang.String[arg0]",
        ),
        (
            b"multiArray".as_slice(),
            b"(I)[[I".as_slice(),
            "new int[arg0][1]",
        ),
        (
            b"divInt".as_slice(),
            b"(I)I".as_slice(),
            "arg0 / DeferredEffects.divisor",
        ),
        (
            b"remInt".as_slice(),
            b"(I)I".as_slice(),
            "arg0 % DeferredEffects.divisor",
        ),
        (
            b"divLong".as_slice(),
            b"(J)J".as_slice(),
            "arg0 / DeferredEffects.longDivisor",
        ),
        (
            b"remLong".as_slice(),
            b"(J)J".as_slice(),
            "arg0 % DeferredEffects.longDivisor",
        ),
        (
            b"cast".as_slice(),
            b"(Ljava/lang/Object;)Ljava/lang/String;".as_slice(),
            "(java.lang.String) arg0",
        ),
        (
            b"direct".as_slice(),
            b"()LOrderValue;".as_slice(),
            "new OrderValue()",
        ),
    ] {
        assert_saved_before_mark(member(&report, name, descriptor), producer);
    }

    let nested = member(&report, b"nested", b"()I");
    assert!(
        nested
            .text
            .contains("return DeferredEffects.left() + DeferredEffects.right();"),
        "nested producers should remain inline:\n{}",
        nested.text
    );
    let branch = member(&report, b"branch", b"([IZ)I");
    assert!(
        branch
            .text
            .contains("return arg0[0] + DeferredEffects.right();")
    );
    assert!(branch.text.contains("return DeferredEffects.left();"));
    let prefix = member(&report, b"prefix", b"([I)I");
    assert!(prefix.text.contains("if (arg0 == null)"));
    assert!(prefix.text.contains("return arg0.length;"));
}

#[test]
fn deferred_value_sources_keep_producer_mark_consumer_bcis_and_default_text() {
    let snapshot = open(FIXTURE);
    let report = class_source_of(&snapshot);
    let default = class_source_without_evidence(&snapshot);
    assert_eq!(
        default.text, report.text,
        "evidence selection changes the artifact"
    );

    for &(name, descriptor, bcis) in METHODS {
        let method = member(&report, name, descriptor);
        let recovery = recovered(&report, name, descriptor);
        assert_eq!(
            recovery.evidence.state(RecoveryEvidenceKind::SourceMap),
            EvidenceState::Complete,
            "all source-map evidence was requested for {}{}",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(descriptor)
        );
        for bci in bcis {
            let segments = recovery.source_map.of_bci(*bci);
            assert!(
                !segments.is_empty(),
                "{}{} has no source-map segment for BCI {bci}: {}",
                String::from_utf8_lossy(name),
                String::from_utf8_lossy(descriptor),
                method.text
            );
            assert!(
                segments.iter().any(|segment| {
                    segment.origin().primary().method() == Some(&method.item.identity)
                }),
                "BCI {bci} of {}{} has no segment owned by its real member",
                String::from_utf8_lossy(name),
                String::from_utf8_lossy(descriptor)
            );
        }
        let default_recovery = match &member(&default, name, descriptor).outcome {
            ClassSourceOutcome::Recovered { report, .. } => report,
            other => panic!("default recovery was not recovered: {other:?}"),
        };
        assert!(
            default_recovery.source_map.is_empty(),
            "default recovery unexpectedly built a source map for {}{}",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(descriptor)
        );
        assert_eq!(
            default_recovery
                .evidence
                .state(RecoveryEvidenceKind::SourceMap),
            EvidenceState::NotRequested
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
            "jarde-p3-deferred-value-order-{}-{nonce}",
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
    fs::write(dir.join("DeferredValueOrder.java"), probe).expect("write probe source");
    fs::write(dir.join("DeferredEffects.java"), EFFECTS_SOURCE).expect("write effects source");
    fs::write(dir.join("OrderValue.java"), VALUE_SOURCE).expect("write value source");
    fs::write(dir.join("DeferredValueOrderRunner.java"), RUNNER_SOURCE)
        .expect("write runner source");
}

fn javac(dir: &Path) {
    let output = std::process::Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(dir)
        .args([
            "DeferredEffects.java",
            "OrderValue.java",
            "DeferredValueOrder.java",
            "DeferredValueOrderRunner.java",
        ])
        .current_dir(dir)
        .output()
        .expect("start javac");
    assert!(
        output.status.success(),
        "compiling DeferredValueOrder failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_runner(dir: &Path) -> String {
    let output = std::process::Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(dir)
        .arg("DeferredValueOrderRunner")
        .current_dir(dir)
        .output()
        .expect("execute the deferred-value-order runner");
    assert!(
        output.status.success(),
        "the deferred-value-order runner failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the runner writes UTF-8")
}

#[test]
#[ignore = "requires JDK: compile Engine::class_source output and execute the patched class oracle"]
fn recovered_deferred_value_order_matches_patched_class_runtime() {
    let report = class_source_of(&open(FIXTURE));
    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered = scratch.path().join("recovered");
    fs::create_dir_all(&original).expect("create original comparison directory");
    fs::create_dir_all(&recovered).expect("create recovered comparison directory");

    write_sources(&original, PROBE_SOURCE);
    javac(&original);
    fs::write(original.join("DeferredValueOrder.class"), FIXTURE)
        .expect("install the patched original class");
    let original_output = run_runner(&original);

    write_sources(&recovered, &report.text);
    javac(&recovered);
    let recovered_output = run_runner(&recovered);

    assert_eq!(original_output.lines().count(), 83);
    for marker in [
        "call0:5:12",
        "field0:5:2",
        "array0:7:2",
        "instance0:5:2",
        "call2:java.lang.IllegalStateException:true:12",
        "instance2:java.lang.IllegalStateException:true:2",
        "badCast2:java.lang.ClassCastException:false:0",
        "direct2:java.lang.IllegalStateException:true:12",
        "newArrayZero0:java.lang.NegativeArraySizeException:false:0",
        "divZero0:java.lang.ArithmeticException:false:0",
        "divLongZero2:java.lang.ArithmeticException:false:0",
        "nullArrayLength1:java.lang.NullPointerException:false:0",
        "nullArray2:java.lang.NullPointerException:false:0",
        "emptyArray2:java.lang.ArrayIndexOutOfBoundsException:false:0",
        "instanceNull2:java.lang.NullPointerException:false:0",
        "nested0:10:34",
        "branchTrue0:12:4",
        "prefixNull0:-1:0",
    ] {
        assert!(
            original_output.contains(marker),
            "the patched original runner covers `{marker}`:\n{original_output}"
        );
    }
    assert_eq!(
        recovered_output, original_output,
        "the recovered class preserves captured values, construction, calls and exception order"
    );
}
