//! Java 8 regression for returning the old value of an effectful field/array postfix update.
//! The only permanently compiled positive class is the subject under test; the Java runner is
//! written into a temporary directory. Refusal cases reuse verifier-valid frozen boundary classes.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-postfix-lvalue-values/v8/PostfixLvalueValues.class");
const FIELD_EXTRA_CONSUMER: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/postfix-lvalue/boundaries/cases/field-extra-consumer/BoundaryProbe.class"
);
const ARRAY_EXTRA_CONSUMER: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/postfix-lvalue/boundaries/cases/array-extra-consumer/BoundaryProbe.class"
);
const FIELD_DIFFERENT_MEMBER: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/postfix-lvalue/boundaries/cases/field-different-member/BoundaryProbe.class"
);
const FIELD_DIFFERENT_OWNER: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/postfix-lvalue/boundaries/cases/field-different-owner/BoundaryProbe.class"
);
const ARRAY_DIFFERENT_INDEX: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/postfix-lvalue/boundaries/cases/array-different-index/BoundaryProbe.class"
);
const ARRAY_DIFFERENT_ARRAY: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/postfix-lvalue/boundaries/cases/array-different-array/BoundaryProbe.class"
);
const FIELD_GAP_EFFECT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/postfix-lvalue/boundaries/cases/field-gap-effect/BoundaryProbe.class"
);
const ARRAY_GAP_EFFECT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/postfix-lvalue/boundaries/cases/array-gap-effect/BoundaryProbe.class"
);
const CONTROL_ASSIGNMENT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/postfix-lvalue/boundaries/cases/control-assignment/BoundaryProbe.class"
);
const RUNNER: &str = r#"
public final class PostfixLvalueValuesRunner {
    private static void out(String name, Object value) {
        System.out.println(name + "=" + value);
    }

    private static String attempt(Runnable action) {
        try {
            action.run();
            return "none";
        } catch (Throwable error) {
            return error.getClass().getSimpleName();
        }
    }

    public static void main(String[] args) {
        PostfixLvalueValues.reset();
        out("field.old", PostfixLvalueValues.postReceiver());
        out("field.new", PostfixLvalueValues.selected.value);
        out("field.calls", PostfixLvalueValues.trace);

        PostfixLvalueValues.reset();
        out("array.old", PostfixLvalueValues.postArray());
        out("array.new", PostfixLvalueValues.values[0]);
        out("array.calls", PostfixLvalueValues.trace);

        PostfixLvalueValues.reset();
        PostfixLvalueValues.selected = null;
        out("field.null", attempt(new Runnable() {
            public void run() { PostfixLvalueValues.postReceiver(); }
        }));
        out("field.null.calls", PostfixLvalueValues.trace);

        PostfixLvalueValues.reset();
        PostfixLvalueValues.values = null;
        out("array.null", attempt(new Runnable() {
            public void run() { PostfixLvalueValues.postArray(); }
        }));
        out("array.null.calls", PostfixLvalueValues.trace);

        PostfixLvalueValues.reset();
        PostfixLvalueValues.selectedIndex = 1;
        out("array.bounds", attempt(new Runnable() {
            public void run() { PostfixLvalueValues.postArray(); }
        }));
        out("array.bounds.calls", PostfixLvalueValues.trace);

        PostfixLvalueValues.reset();
        PostfixLvalueValues.selected.value = Integer.MAX_VALUE;
        out("overflow.old", PostfixLvalueValues.postReceiver());
        out("overflow.new", PostfixLvalueValues.selected.value);

        PostfixLvalueValues.reset();
        out("simple.post.old", PostfixLvalueValues.selected.postSimpleField());
        out("simple.post.new", PostfixLvalueValues.selected.value);
        out("simple.pre.new", PostfixLvalueValues.selected.preSimpleField());
    }
}
"#;
const EXPECTED: &str = "field.old=41\nfield.new=42\nfield.calls=R\n\
array.old=70\narray.new=71\narray.calls=AI\n\
field.null=NullPointerException\nfield.null.calls=R\n\
array.null=NullPointerException\narray.null.calls=AI\n\
array.bounds=ArrayIndexOutOfBoundsException\narray.bounds.calls=AI\n\
overflow.old=2147483647\noverflow.new=-2147483648\n\
simple.post.old=41\nsimple.post.new=42\nsimple.pre.new=43\n";

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults to a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the verifier-valid Java 8 class opens as a standalone class")
}

fn class_source_of(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
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
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("a valid class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one standalone class has {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one standalone class has an unfinished selection with {} candidates",
            candidates.candidates.len()
        ),
    }
}

fn method_request(
    snapshot: &ArtifactSnapshot,
    member: &ClassSourceMethod,
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
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
        }
        .build(slice::from_ref(snapshot))
        .expect("the standalone Java 8 environment is valid"),
        method: member.item.identity.clone(),
        stages: AnalysisStage::ALL.to_vec(),
    }
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("missing method `{name}`"))
}

fn temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after the epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "jarde-postfix-values-{}-{nonce}",
        std::process::id()
    ))
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path = temp_dir();
        fs::create_dir(&path).expect("create temporary Java comparison directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, name: &str, contents: &str) {
        fs::write(self.0.join(name), contents).expect("write temporary Java runner");
    }

    fn write_fixture(&self) {
        fs::write(self.0.join("PostfixLvalueValues.class"), FIXTURE)
            .expect("copy frozen subject class");
        self.write("PostfixLvalueValuesRunner.java", RUNNER);
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn compile(dir: &Path, source_files: &[&str]) {
    let output = Command::new("javac")
        .args(["-Xlint:-options", "--release", "8", "-g:none", "-classpath"])
        .arg(dir)
        .arg("-d")
        .arg(dir)
        .args(source_files.iter().map(|name| dir.join(name)))
        .output()
        .expect("javac is available");
    assert!(
        output.status.success(),
        "javac refused the complete source:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn execute(dir: &Path) -> String {
    let output = Command::new("java")
        .args(["-Xverify:all", "-classpath"])
        .arg(dir)
        .arg("PostfixLvalueValuesRunner")
        .output()
        .expect("java is available");
    assert!(
        output.status.success(),
        "verified JVM execution failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the runner output is UTF-8")
}

#[test]
fn returned_old_value_updates_are_single_evaluation_expressions_with_real_sources() {
    let snapshot = open(FIXTURE);
    let report = class_source_of(&snapshot, "PostfixLvalueValues");

    for name in ["postReceiver", "postArray"] {
        let member = method(&report, name);
        assert!(
            !member.text.contains("@bytecode"),
            "{name} should be presented as a complete method:\n{}",
            member.text
        );
        assert!(
            member.text.contains("return ") && member.text.contains("++;"),
            "{name} should return a postfix expression:\n{}",
            member.text
        );
        let ClassSourceOutcome::Recovered { report: body, .. } = &member.outcome else {
            panic!("{name} should have a recovery report: {:?}", member.outcome);
        };
        let bcis: &[u32] = if name == "postReceiver" {
            &[0, 3, 4, 7, 8, 9, 10, 13]
        } else {
            &[0, 3, 6, 7, 8, 9, 10, 11, 12]
        };
        for bci in bcis {
            assert!(
                !body.source_map.of_bci(*bci).is_empty(),
                "{name} omits physical BCI {bci}: {:?}",
                body.source_map.segments()
            );
        }
    }
}

#[test]
fn verifier_valid_unproved_postfix_boundaries_are_refused_with_physical_sources() {
    for (fixture, method_name, bcis) in [
        (
            FIELD_DIFFERENT_MEMBER,
            "fieldDifferentMember",
            &[0, 4, 10, 13][..],
        ),
        (
            FIELD_DIFFERENT_OWNER,
            "fieldDifferentOwner",
            &[0, 4, 10, 13][..],
        ),
        (
            ARRAY_DIFFERENT_INDEX,
            "arrayDifferentIndex",
            &[0, 3, 7, 19, 20][..],
        ),
        (
            ARRAY_DIFFERENT_ARRAY,
            "arrayDifferentArray",
            &[0, 3, 7, 19, 20][..],
        ),
        (
            FIELD_EXTRA_CONSUMER,
            "fieldExtraConsumer",
            &[0, 3, 4, 7, 10, 13, 14, 17][..],
        ),
        (
            ARRAY_EXTRA_CONSUMER,
            "arrayExtraConsumer",
            &[0, 3, 6, 7, 8, 11, 12, 13, 16][..],
        ),
        (FIELD_GAP_EFFECT, "fieldGap", &[0, 3, 7, 13, 16][..]),
        (ARRAY_GAP_EFFECT, "arrayGap", &[0, 3, 6, 10, 14, 15][..]),
    ] {
        let report = class_source_of(&open(fixture), "BoundaryProbe");
        let member = method(&report, method_name);
        assert!(
            member.text.contains("@bytecode"),
            "{method_name} must keep the unsupported chain quoted:\n{}",
            member.text
        );
        assert!(
            !member.text.contains("++;"),
            "{method_name} must not claim a proved postfix update:\n{}",
            member.text
        );
        let ClassSourceOutcome::Recovered { report: body, .. } = &member.outcome else {
            panic!(
                "{method_name} should retain its refusal report: {:?}",
                member.outcome
            );
        };
        for bci in bcis {
            assert!(
                !body.source_map.of_bci(*bci).is_empty(),
                "{method_name} dropped physical BCI {bci}: {:?}",
                body.source_map.segments()
            );
        }
    }
}

#[test]
fn ordinary_assignment_does_not_become_a_postfix_update() {
    let report = class_source_of(&open(CONTROL_ASSIGNMENT), "BoundaryProbe");
    let member = method(&report, "ordinaryAssignment");
    assert!(
        !member.text.contains("++;"),
        "an ordinary assignment is not a postfix update:\n{}",
        member.text
    );
}

#[test]
fn postfix_return_keeps_evidence_and_stops_without_partial_output() {
    let snapshot = open(FIXTURE);
    let full_class = class_source_of(&snapshot, "PostfixLvalueValues");
    let request = method_request(&snapshot, method(&full_class, "postReceiver"));
    let engine = Engine::new();

    let mut full_budget = task_budget(&[]).expect("the task defaults are bounded");
    let full = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut full_budget,
        )
        .expect("the postfix method recovers");
    let full_report = full.recovery();
    assert!(full_report.produced());
    assert!(
        full_report.text.contains("return receiver().value++;")
            || full_report
                .text
                .contains("return PostfixLvalueValues.receiver().value++;"),
        "{}",
        full_report.text
    );

    let mut essential_budget = task_budget(&[]).expect("the task defaults are bounded");
    let essential = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::essential(),
            &mut essential_budget,
        )
        .expect("essential evidence also recovers the method");
    assert_eq!(
        essential.recovery().text,
        full_report.text,
        "evidence selection does not change the body"
    );

    let emitted = u64::try_from(full_report.text.len()).expect("text length fits u64");
    let complete_output = full_budget.usage().output_bytes;
    assert!(
        complete_output > emitted,
        "analysis output precedes the artifact"
    );
    let mut limited_budget = Budget::new(Limits {
        output_bytes: complete_output - emitted,
        ..task_limits(&[]).expect("the task defaults are bounded")
    });
    let limited = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut limited_budget,
        )
        .expect("an output limit is reported");
    assert!(!limited.recovery().produced());
    assert!(limited.recovery().text.is_empty());
    assert!(limited.recovery().source_map.is_empty());
    assert!(matches!(
        limited.recovery().stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::OutputBytes,
            ..
        })
    ));

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled_budget = Budget::with_cancellation_token(
        task_limits(&[]).expect("the task defaults are bounded"),
        token,
    );
    let cancelled = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled_budget,
        )
        .expect("pre-cancellation is reported");
    assert!(matches!(
        cancelled.analysis().execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert!(!cancelled.recovery().produced());
    assert!(cancelled.recovery().text.is_empty());
    assert!(cancelled.recovery().source_map.is_empty());
}

#[test]
#[ignore = "needs javac on PATH to create the depth-bound subject class"]
fn a_receiver_beyond_the_value_depth_bound_is_not_partially_postfixed() {
    const RECEIVER_DEPTH: usize = 32;
    let scratch = Scratch::new();
    let receiver = "next.".repeat(RECEIVER_DEPTH);
    scratch.write(
        "PostfixDepth.java",
        &format!(
            "public final class PostfixDepth {{\n\
             public PostfixDepth next;\n\
             public int value;\n\
             public static PostfixDepth root;\n\
             public static int deep() {{ return root.{receiver}value++; }}\n\
             }}\n"
        ),
    );
    compile(scratch.path(), &["PostfixDepth.java"]);
    let bytes = fs::read(scratch.path().join("PostfixDepth.class"))
        .expect("javac produced the depth-bound subject class");
    let snapshot = open(&bytes);
    let report = class_source_of(&snapshot, "PostfixDepth");
    let request = method_request(&snapshot, method(&report, "deep"));
    let recovered = Engine::new()
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("a value-depth refusal is a recovery result");
    let body = recovered.recovery();
    assert!(body.produced());
    assert!(
        body.text.contains("@bytecode"),
        "the over-depth value remains a refusal:\n{}",
        body.text
    );
    assert!(
        !body.text.contains("++"),
        "the postfix node is not published without its complete receiver:\n{}",
        body.text
    );
    assert!(
        !body.source_map.is_empty(),
        "the refusal retains the physical source map"
    );
}

#[test]
#[ignore = "needs javac and java on PATH for complete-class equivalence"]
fn complete_recovered_class_matches_the_verified_original_runtime() {
    let report = class_source_of(&open(FIXTURE), "PostfixLvalueValues");
    let original = Scratch::new();
    original.write_fixture();
    compile(original.path(), &["PostfixLvalueValuesRunner.java"]);
    let original_output = execute(original.path());
    assert_eq!(original_output, EXPECTED, "frozen Java 8 fixture drifted");

    let generated = Scratch::new();
    generated.write("PostfixLvalueValues.java", &report.text);
    generated.write("PostfixLvalueValuesRunner.java", RUNNER);
    compile(
        generated.path(),
        &["PostfixLvalueValues.java", "PostfixLvalueValuesRunner.java"],
    );
    assert_eq!(
        execute(generated.path()),
        original_output,
        "complete recovered class changed the verified original observations"
    );
}
