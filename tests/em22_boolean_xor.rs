//! The EM-22 Boolean `ixor` literal projection and its typed refusal boundaries.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SCRATCH_ID: AtomicU64 = AtomicU64::new(0);

const PROBE: &str = r#"
package em22;
public final class BooleanXorProbe {
    private int calls;
    static boolean flip(boolean value) { return value ^ true; }
    static boolean keep(boolean value) { return value ^ false; }
    static boolean nested(boolean value, boolean other) { return (value ^ false) & other; }
    static boolean variable(boolean left, boolean right) { return left ^ right; }
    static int intLiteral(int value) { return value ^ 1; }
    static long longLiteral(long value) { return value ^ 1L; }
    boolean left() { calls++; return true; }
    boolean flipCall() { return left() ^ true; }
    boolean keepCall() { return left() ^ false; }
    static boolean eagerOr(boolean left, boolean right) { return left | right; }
}
"#;

const UNKNOWN: &str = r#"
package em22;
public final class BooleanXorUnknown {
    static int value(int input) { return input ^ 1; }
}
"#;

fn budget() -> Budget {
    task_budget(&[]).expect("default task budget is valid")
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget())
        .expect("the compiled class opens")
}

fn source(
    snapshot: &ArtifactSnapshot,
    class: &str,
    budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &ClassSourceRequest {
                class: ClassRef::Name {
                    class: ClassNameQuery::internal(class),
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
            },
            &RecoveryEvidenceRequest::all(),
            budget,
        )
        .expect("the bounded class-source operation returns a report or stop")
}

fn performed(outcome: OperationOutcome<ClassSourceReport>) -> ClassSourceReport {
    match outcome {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => {
            panic!(
                "one class binds uniquely, got {} candidates",
                candidates.candidates.len()
            )
        }
        OperationOutcome::Incomplete(candidates) => panic!(
            "the class completes with default limits, got {} unfinished candidates",
            candidates.candidates.len()
        ),
    }
}

fn method_text(report: &ClassSourceReport, name: &[u8]) -> String {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name)
        .unwrap_or_else(|| panic!("method {} is present", String::from_utf8_lossy(name)));
    method.text.clone()
}

fn compile(directory: &Path, name: &str, source: &str) -> PathBuf {
    fs::create_dir_all(directory).expect("create compilation directory");
    let input = directory.join(format!("{name}.java"));
    fs::write(&input, source).expect("write source");
    let output = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(directory)
        .arg(&input)
        .output()
        .expect("javac is available for the Java 8 fixture");
    assert!(
        output.status.success(),
        "fixture compiles: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    directory.join(format!("em22/{name}.class"))
}

fn patch_unique_utf8(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    assert_eq!(from.len(), to.len());
    let count = usize::from(u16::from_be_bytes([bytes[8], bytes[9]]));
    let mut cursor = 10;
    let mut index = 1;
    let mut patched = bytes.to_vec();
    let mut matches = 0;
    while index < count {
        let tag = bytes[cursor];
        match tag {
            1 => {
                let length =
                    usize::from(u16::from_be_bytes([bytes[cursor + 1], bytes[cursor + 2]]));
                let value = cursor + 3;
                if &bytes[value..value + length] == from {
                    patched[value..value + length].copy_from_slice(to);
                    matches += 1;
                }
                cursor = value + length;
            }
            3 | 4 | 9 | 10 | 11 | 12 | 17 | 18 => cursor += 5,
            5 | 6 => {
                cursor += 9;
                index += 1;
            }
            7 | 8 | 16 | 19 | 20 => cursor += 3,
            15 => cursor += 4,
            other => panic!("unexpected constant pool tag {other}"),
        }
        index += 1;
    }
    assert_eq!(matches, 1, "the descriptor is unique in the boundary class");
    patched
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        let unique = SCRATCH_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-em22-boolean-xor-{}-{nonce}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temporary test directory");
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

#[test]
fn proved_boolean_xor_literals_keep_type_effect_and_origin_facts() {
    let scratch = Scratch::new();
    let classes = scratch.path().join("classes");
    let class = compile(&classes, "BooleanXorProbe", PROBE);
    let report = performed(source(
        &open(fs::read(class).expect("read compiled probe")),
        "em22/BooleanXorProbe",
        &mut budget(),
    ));

    assert!(
        method_text(&report, b"flip").contains("return !arg0;"),
        "{}",
        report.text
    );
    assert!(
        method_text(&report, b"keep").contains("return arg0;"),
        "{}",
        report.text
    );
    assert!(
        method_text(&report, b"nested").contains("return arg0 & arg1;"),
        "nested boolean consumer retains the proved operand type: {}",
        report.text
    );
    assert!(
        method_text(&report, b"flipCall").contains("return !this.left();"),
        "{}",
        report.text
    );
    assert!(
        method_text(&report, b"keepCall").contains("return this.left();"),
        "{}",
        report.text
    );
    assert_eq!(
        method_text(&report, b"flipCall").matches("left()").count(),
        1
    );
    assert_eq!(
        method_text(&report, b"keepCall").matches("left()").count(),
        1
    );
    assert!(
        method_text(&report, b"variable").contains("return arg0 ^ arg1;"),
        "{}",
        report.text
    );
    assert!(
        method_text(&report, b"intLiteral").contains("^ 1"),
        "{}",
        report.text
    );
    assert!(
        method_text(&report, b"longLiteral").contains("^ 1L"),
        "{}",
        report.text
    );
    assert!(
        method_text(&report, b"eagerOr").contains("arg0 | arg1"),
        "{}",
        report.text
    );
    for name in [b"flip".as_slice(), b"keep".as_slice()] {
        let method = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == name)
            .expect("the XOR member is present");
        let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
            panic!("the XOR member is recovered: {}", method.text);
        };
        assert!(
            !report.source_map.of_bci(2).is_empty(),
            "BCI 2 ixor remains an expression source: {}",
            method.text
        );
        assert!(
            !report.source_map.derived_of_bci(1).is_empty(),
            "BCI 1 literal remains a derived source: {}",
            method.text
        );
    }
}

#[test]
fn a_result_boolean_context_does_not_promote_an_unproved_int_xor_operand() {
    let scratch = Scratch::new();
    let classes = scratch.path().join("classes");
    let class = compile(&classes, "BooleanXorUnknown", UNKNOWN);
    let bytes = fs::read(class).expect("read compiled boundary class");
    let bytes = patch_unique_utf8(&bytes, b"(I)I", b"(I)Z");
    let report = performed(source(
        &open(bytes),
        "em22/BooleanXorUnknown",
        &mut budget(),
    ));
    let value = method_text(&report, b"value");
    assert!(!value.contains("!arg0"), "{value}");
    assert!(value.contains("^ 1"), "{value}");
}

#[test]
fn budget_and_cancellation_do_not_publish_a_partially_built_xor_expression() {
    let scratch = Scratch::new();
    let classes = scratch.path().join("classes");
    let class = compile(&classes, "BooleanXorProbe", PROBE);
    let snapshot = open(fs::read(class).expect("read compiled probe"));

    let mut limited = task_budget(&[
        BudgetOverride::new("analysis_steps", 1).expect("the small budget is positive")
    ])
    .expect("the bounded task budget is valid");
    let limited = source(&snapshot, "em22/BooleanXorProbe", &mut limited);
    let no_partial_xor = match &limited {
        OperationOutcome::Performed(report) => {
            !report.text.contains("return !arg0;") && !report.text.contains("^ true")
        }
        OperationOutcome::Incomplete(_) => true,
        OperationOutcome::Ambiguous(_) => false,
    };
    assert!(
        no_partial_xor,
        "budget stop published a partial XOR: {limited:?}"
    );

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(budget().limits().clone(), token);
    assert!(matches!(
        source(&snapshot, "em22/BooleanXorProbe", &mut cancelled),
        OperationOutcome::Incomplete(_)
    ));
}
