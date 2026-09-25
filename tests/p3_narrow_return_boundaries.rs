//! Return-boundary evidence for real shared-switch joins and boolean/int JVM boundaries.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const STACK_BYTE: &[u8] = include_bytes!(
    "fixtures/p3-narrow-integer-returns/actual-stack-join/v8/byte/ActualStackJoin.class"
);
const STACK_CHAR: &[u8] = include_bytes!(
    "fixtures/p3-narrow-integer-returns/actual-stack-join/v8/char/ActualStackJoin.class"
);
const STACK_SHORT: &[u8] = include_bytes!(
    "fixtures/p3-narrow-integer-returns/actual-stack-join/v8/short/ActualStackJoin.class"
);
const BOOLEAN: &[u8] = include_bytes!(
    "fixtures/p3-narrow-integer-returns/v8/boolean-boundaries/BooleanReturnBoundaries.class"
);
const BOOLEAN_RAW_TWO: &[u8] = include_bytes!(
    "fixtures/p3-narrow-integer-returns/v8/boolean-boundaries/BooleanRawTwoCaller.class"
);
const NARROW_CLASS: &[u8] =
    include_bytes!("fixtures/p3-narrow-integer-returns/v8/NarrowIntegerReturns.class");
const NARROW_RUNNER: &str =
    include_str!("fixtures/p3-narrow-integer-returns/NarrowIntegerReturnsRunner.java");
const BOUNDARY_RUNNER: &str =
    include_str!("fixtures/p3-narrow-integer-returns/ReturnBoundaryRunner.java");
const STACK_RUNNER: &str =
    include_str!("fixtures/p3-narrow-integer-returns/actual-stack-join/ActualStackJoinRunner.java");
const STACK_EXPECTED_BYTE: &str =
    include_str!("fixtures/p3-narrow-integer-returns/actual-stack-join/v8/byte/expected.txt");
const STACK_EXPECTED_CHAR: &str =
    include_str!("fixtures/p3-narrow-integer-returns/actual-stack-join/v8/char/expected.txt");
const STACK_EXPECTED_SHORT: &str =
    include_str!("fixtures/p3-narrow-integer-returns/actual-stack-join/v8/short/expected.txt");

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

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut Budget::new(limits()),
        )
        .expect("the permanent descriptor-patched class opens")
}

fn class_source(bytes: &[u8], name: &str, java_release: u16) -> ClassSourceReport {
    let snapshot = open(bytes);
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits()),
        )
        .expect("a single-class source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => {
            panic!(
                "single fixture selected {} candidates",
                candidates.candidates.len()
            )
        }
        OperationOutcome::Incomplete(candidates) => panic!(
            "single fixture selection is incomplete: {} candidates",
            candidates.candidates.len()
        ),
    }
}

fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| {
            panic!(
                "method `{name}` is absent; present methods: {:?}",
                report
                    .methods
                    .iter()
                    .map(|method| String::from_utf8_lossy(&method.item.name.raw().0).to_string())
                    .collect::<Vec<_>>()
            )
        })
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-return-boundaries-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the JDK verification directory");
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

fn write(path: &Path, name: &str, bytes: &[u8]) {
    fs::write(path.join(name), bytes).unwrap_or_else(|error| panic!("write {name}: {error}"));
}

fn run_jvm_fixtures() -> (String, String) {
    let scratch = Scratch::new();
    let classes = scratch.path().join("classes");
    fs::create_dir_all(&classes).expect("create class directory");
    write(&classes, "BooleanReturnBoundaries.class", BOOLEAN);
    write(&classes, "BooleanRawTwoCaller.class", BOOLEAN_RAW_TWO);
    write(&classes, "NarrowIntegerReturns.class", NARROW_CLASS);
    fs::write(
        scratch.path().join("ReturnBoundaryRunner.java"),
        BOUNDARY_RUNNER,
    )
    .expect("write boundary runner");
    fs::write(
        scratch.path().join("NarrowIntegerReturnsRunner.java"),
        NARROW_RUNNER,
    )
    .expect("write narrow-return runner");

    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-cp"])
        .arg(&classes)
        .args(["-d"])
        .arg(&classes)
        .args([
            "ReturnBoundaryRunner.java",
            "NarrowIntegerReturnsRunner.java",
        ])
        .current_dir(scratch.path())
        .output()
        .expect("start javac for source-only runners");
    assert!(
        compile.status.success(),
        "runner compilation failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = |main: &str| {
        let output = Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(&classes)
            .arg(main)
            .current_dir(scratch.path())
            .output()
            .unwrap_or_else(|error| panic!("start verified {main}: {error}"));
        assert!(
            output.status.success(),
            "verified {main} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("runner output is UTF-8")
    };
    (
        run("ReturnBoundaryRunner"),
        run("NarrowIntegerReturnsRunner"),
    )
}

fn run_stack_fixture(class: &[u8]) -> String {
    let scratch = Scratch::new();
    let classes = scratch.path().join("classes");
    fs::create_dir_all(&classes).expect("create Java 8 class directory");
    write(&classes, "ActualStackJoin.class", class);
    fs::write(
        scratch.path().join("ActualStackJoinRunner.java"),
        STACK_RUNNER,
    )
    .expect("write actual stack join runner");
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-cp"])
        .arg(&classes)
        .args(["-d"])
        .arg(&classes)
        .arg("ActualStackJoinRunner.java")
        .current_dir(scratch.path())
        .output()
        .expect("start javac for the stack join runner");
    assert!(
        compile.status.success(),
        "stack runner compilation failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let output = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&classes)
        .arg("ActualStackJoinRunner")
        .current_dir(scratch.path())
        .output()
        .expect("run verified major-49 stack-join variant");
    assert!(
        output.status.success(),
        "major-49 stack-join verification failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stack runner output is UTF-8")
}

#[test]
fn field_and_synchronized_narrow_returns_keep_their_operand_and_return_sources() {
    let report = class_source(NARROW_CLASS, "NarrowIntegerReturns", 8);
    for (name, descriptor, operand_bci, write_or_exit_bci, return_bci) in [
        ("postByte", "(I)B", 2, 8, 11),
        ("preChar", "(J)C", 6, 8, 11),
        ("syncByte", "(Ljava/lang/Object;I)B", 4, 6, 7),
        ("syncShort", "(Ljava/lang/Object;JI)S", 5, 8, 9),
    ] {
        let method = member(&report, name);
        assert_eq!(method.item.descriptor.raw().0, descriptor.as_bytes());
        let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
            panic!(
                "{name} did not return a recovered body: {:?}",
                method.outcome
            )
        };
        assert_eq!(
            body.representation,
            Representation::Java,
            "{name}: {}",
            body.text
        );
        assert_eq!(body.quality, Quality::Structured, "{name}");
        assert!(
            !body.text.contains("@bytecode"),
            "the narrow return is presented whole for {name}: {}",
            body.text
        );
        for bci in [operand_bci, write_or_exit_bci, return_bci] {
            assert!(
                !body.text_of_bci(bci).is_empty(),
                "{name} keeps operand/write/exit/return BCI {bci} in the source map: {}",
                body.text
            );
        }
        assert!(
            !body.source_map.direct_of_bci(return_bci).is_empty(),
            "the actual ireturn at BCI {return_bci} directly anchors {name}: {:?}",
            body.source_map.segments()
        );
    }
}

#[test]
fn shared_switch_join_b_c_s_sources_the_real_ireturn() {
    for (class, name, ret, return_bci, arm_bcis, expected) in [
        (
            STACK_BYTE,
            "runByte",
            "B",
            32,
            [20, 26],
            STACK_EXPECTED_BYTE,
        ),
        (
            STACK_CHAR,
            "runChar",
            "C",
            30,
            [20, 25],
            STACK_EXPECTED_CHAR,
        ),
        (
            STACK_SHORT,
            "runShort",
            "S",
            30,
            [20, 25],
            STACK_EXPECTED_SHORT,
        ),
    ] {
        let report = class_source(class, "ActualStackJoin", 8);
        let method = member(&report, name);
        assert_eq!(
            method.item.descriptor.raw().0,
            format!("(I){ret}").as_bytes()
        );
        let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
            panic!("{name} did not answer recovery: {:?}", method.outcome)
        };
        assert_eq!(
            body.representation,
            Representation::Java,
            "{name}: {} / {:?}",
            body.text,
            body.fallbacks
        );
        assert_eq!(body.quality, Quality::Structured, "{name}");
        assert!(
            !body.text.contains("@bytecode"),
            "the shared join is represented in {name}: {}",
            body.text
        );
        assert!(
            !body.source_map.direct_of_bci(return_bci).is_empty(),
            "the shared join ireturn at BCI {return_bci} anchors {name}: {:?}",
            body.source_map.segments()
        );
        for bci in [1, arm_bcis[0], arm_bcis[1], return_bci] {
            assert!(
                !body.text_of_bci(bci).is_empty(),
                "{name} preserves switch operand/control-flow source BCI {bci}: {}",
                body.text
            );
        }
        assert_eq!(
            run_stack_fixture(class),
            expected,
            "verifier-valid original JVM output for {name}"
        );
    }
}

#[test]
fn boolean_operands_are_refused_at_numeric_returns_and_z_keeps_low_bit_semantics() {
    let report = class_source(BOOLEAN, "BooleanReturnBoundaries", 8);
    for (name, descriptor) in [
        ("booleanAsByte", "(Z)B"),
        ("booleanAsChar", "(Z)C"),
        ("booleanAsShort", "(Z)S"),
    ] {
        let method = member(&report, name);
        assert_eq!(method.item.descriptor.raw().0, descriptor.as_bytes());
        let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
            panic!(
                "{name} did not return a refusal report: {:?}",
                method.outcome
            )
        };
        assert_eq!(
            body.representation,
            Representation::Mixed,
            "{name}: {}",
            body.text
        );
        assert_eq!(body.quality, Quality::Fallback, "{name}");
        assert!(
            body.text.contains("@bytecode 1") && body.text.contains("at BCI 0"),
            "the refusal identifies the real return instruction for {name}: {}",
            body.text
        );
        assert!(
            !body.source_map.direct_of_bci(1).is_empty(),
            "the refused `ireturn` at BCI 1 remains source-mapped: {:?}",
            body.source_map.segments()
        );
    }
    let z_method = member(&report, "integerAsBoolean");
    let ClassSourceOutcome::Recovered { report: z_body, .. } = &z_method.outcome else {
        panic!(
            "integerAsBoolean did not return a recovery report: {:?}",
            z_method.outcome
        )
    };
    assert_eq!(z_body.representation, Representation::Java);
    assert_eq!(z_body.quality, Quality::Structured);
    assert!(
        z_body.text.contains("% 2 != 0") && !z_body.text.contains("@bytecode"),
        "the Z ireturn preserves the low bit of its int operand: {}",
        z_body.text
    );
    assert!(
        !z_body.text_of_bci(0).is_empty(),
        "the int load keeps its source"
    );
    assert!(
        !z_body.source_map.direct_of_bci(1).is_empty(),
        "the recovered Z `ireturn` at BCI 1 remains source-mapped: {:?}",
        z_body.source_map.segments()
    );

    let raw_caller_report = class_source(BOOLEAN_RAW_TWO, "BooleanRawTwoCaller", 8);
    for name in ["byteValue", "charValue", "shortValue"] {
        let method = member(&raw_caller_report, name);
        let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
            panic!(
                "raw-two caller {name} did not return a report: {:?}",
                method.outcome
            )
        };
        assert_eq!(
            body.representation,
            Representation::Mixed,
            "{name}: {}",
            body.text
        );
        assert_eq!(body.quality, Quality::Fallback, "{name}");
        assert!(
            body.text.contains("@bytecode 4 1")
                && body.text.contains("is declared `boolean` presents `int`"),
            "the raw 2 caller's argument source is refused and named for {name}: {}",
            body.text
        );
        assert!(
            !body.text.contains("byteValue() {\n        return 1;")
                && !body.text.contains("return (byte) 1;"),
            "the JVM's raw result 2 is not rewritten as a boolean value in {name}: {}",
            body.text
        );
    }

    let (boundary_output, narrow_output) = run_jvm_fixtures();
    let expected_boundary = concat!(
        "booleanAsByte:false:0\nbooleanAsByte:true:1\n",
        "booleanAsChar:false:0\nbooleanAsChar:true:1\n",
        "booleanAsShort:false:0\nbooleanAsShort:true:1\n",
        "byteValue:2\ncharValue:2\nshortValue:2\n",
        "integerAsBoolean:0:false\nintegerAsBoolean:1:true\n",
        "integerAsBoolean:2:false\nintegerAsBoolean:3:true\n",
        "integerAsBoolean:-1:true\n"
    );
    assert_eq!(
        boundary_output, expected_boundary,
        "verified original class values"
    );
    assert_eq!(
        narrow_output.lines().count(),
        49,
        "narrow-return runner cases"
    );
    assert!(
        narrow_output.contains("null:java.lang.NullPointerException"),
        "the synchronized null-monitor exception is part of the original JVM result: {narrow_output}"
    );
}
