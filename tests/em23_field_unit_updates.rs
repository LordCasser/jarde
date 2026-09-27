//! The EM-23 statement-only current-class `int` field update slice.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SCRATCH_ID: AtomicU64 = AtomicU64::new(0);

const PROBE: &str = r#"
package em23probe;
final class Foreign { int value; }
public final class FieldUnitProbe {
    int instanceField;
    static int staticField;
    volatile int volatileField;
    long wideField;
    Foreign foreign = new Foreign();
    int calls;

    void increment() { instanceField++; }
    static void decrement() { staticField--; }
    void volatileUpdate() { volatileField++; }
    void foreignOwner() { foreign.value++; }
    static void parameterAtSlotZero(Foreign value) { value.value++; }
    void complexReceiver() { receiver().instanceField++; }
    FieldUnitProbe receiver() { calls++; return this; }
    void ping() { calls++; }
    void effectAfterUpdate() { instanceField++; ping(); }
    void caughtUpdate() {
        try { instanceField++; } catch (RuntimeException ignored) { ping(); }
    }
    void wideUpdate() { wideField++; }
    int returnedOldValue() { return instanceField++; }
}
"#;

fn budget() -> Budget {
    task_budget(&[]).expect("the default task budget is valid")
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget())
        .expect("compiled fixture opens")
}

fn source(snapshot: &ArtifactSnapshot, budget: &mut Budget) -> OperationOutcome<ClassSourceReport> {
    Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &ClassSourceRequest {
                class: ClassRef::Name {
                    class: ClassNameQuery::internal("em23probe/FieldUnitProbe"),
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
        .expect("the single-class request is valid")
}

fn performed(outcome: OperationOutcome<ClassSourceReport>) -> ClassSourceReport {
    match outcome {
        OperationOutcome::Performed(report) => report,
        other => panic!("the complete budget performs one class query: {other:?}"),
    }
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|member| member.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("method `{name}` is present"))
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
            "jarde-em23-field-update-{}-{nonce}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create scratch directory");
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

fn compiled_probe() -> (Scratch, ArtifactSnapshot) {
    let scratch = Scratch::new();
    let source_file = scratch.path().join("FieldUnitProbe.java");
    fs::write(&source_file, PROBE).expect("write probe source");
    let output = std::process::Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(scratch.path())
        .arg(&source_file)
        .output()
        .expect("javac is available for Java 8 fixtures");
    assert!(
        output.status.success(),
        "fixture compiles: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let class = scratch.path().join("em23probe/FieldUnitProbe.class");
    (scratch, open(fs::read(class).expect("read compiled probe")))
}

#[test]
fn same_class_int_unit_updates_are_single_source_mapped_postfix_statements() {
    let (_scratch, snapshot) = compiled_probe();
    let report = performed(source(&snapshot, &mut budget()));
    let increment = method(&report, "increment");
    let decrement = method(&report, "decrement");
    assert!(
        increment.text.contains("this.instanceField++;"),
        "{}",
        increment.text
    );
    assert!(
        decrement
            .text
            .contains("em23probe.FieldUnitProbe.staticField--;"),
        "{}",
        decrement.text
    );
    assert_eq!(increment.text.matches("this.instanceField++").count(), 1);
    assert_eq!(
        decrement
            .text
            .matches("em23probe.FieldUnitProbe.staticField--")
            .count(),
        1
    );
    let ClassSourceOutcome::Recovered { report, .. } = &increment.outcome else {
        panic!("increment method is recovered: {}", increment.text);
    };
    assert_eq!(
        report
            .fields
            .iter()
            .map(|field| field.access)
            .collect::<Vec<_>>(),
        ["read", "write"]
    );
    assert!(report.fields.iter().all(|field| field.presented));
    let field_reads: Vec<_> = report
        .fields
        .iter()
        .filter(|field| field.access == "read")
        .collect();
    let field_writes: Vec<_> = report
        .fields
        .iter()
        .filter(|field| field.access == "write")
        .collect();
    assert_eq!(field_reads.len(), 1);
    assert_eq!(field_writes.len(), 1);
    assert!(
        report
            .source_map
            .text_of_bci(&report.text, field_reads[0].bci)
            .iter()
            .any(|segment| segment.contains("instanceField"))
    );
    assert!(
        report
            .source_map
            .text_of_bci(&report.text, field_writes[0].bci)
            .iter()
            .any(|segment| segment.contains("instanceField++"))
    );
    assert!(!increment.text.contains("instanceField += 1"));
    assert!(!decrement.text.contains("staticField = "));
}

#[test]
fn owner_descriptor_volatile_receiver_and_effect_boundaries_do_not_use_this_slice() {
    let (_scratch, snapshot) = compiled_probe();
    let report = performed(source(&snapshot, &mut budget()));
    for (name, spelling) in [
        ("volatileUpdate", "volatileField++"),
        ("foreignOwner", "foreign.value++"),
        ("parameterAtSlotZero", "value.value++"),
        ("complexReceiver", "receiver().instanceField++"),
        ("effectAfterUpdate", "this.instanceField++"),
        ("caughtUpdate", "this.instanceField++"),
        ("wideUpdate", "wideField++"),
    ] {
        let body = &method(&report, name).text;
        assert!(
            !body.contains(spelling),
            "`{name}` crossed the proof boundary:\n{body}"
        );
    }
    let receiver = &method(&report, "complexReceiver").text;
    assert_eq!(receiver.matches("receiver()").count(), 1, "{receiver}");
    let after_effect = &method(&report, "effectAfterUpdate").text;
    assert!(
        after_effect.contains("this.instanceField += 1;"),
        "{after_effect}"
    );
    assert!(after_effect.contains("ping();"), "{after_effect}");
}

#[test]
fn a_returned_old_value_stays_an_expression_and_budget_or_cancellation_stops_atomically() {
    let (_scratch, snapshot) = compiled_probe();
    let report = performed(source(&snapshot, &mut budget()));
    let returned = &method(&report, "returnedOldValue").text;
    assert!(
        returned.contains("return this.instanceField++;"),
        "{returned}"
    );
    assert!(
        !returned
            .lines()
            .any(|line| line.trim() == "this.instanceField++;"),
        "the returned postfix update remains an expression: {returned}"
    );

    let mut limited =
        task_budget(&[BudgetOverride::new("analysis_steps", 1).expect("the limit is positive")])
            .expect("the bounded budget is valid");
    let limited = source(&snapshot, &mut limited);
    match limited {
        OperationOutcome::Performed(report) => {
            assert!(matches!(report.execution, ExecutionReport::Partial { .. }));
            assert!(
                !report.text.contains("this.instanceField++;")
                    && !report.text.contains("staticField--;"),
                "budget stop exposed a partial update: {}",
                report.text
            );
        }
        OperationOutcome::Incomplete(_) => {}
        OperationOutcome::Ambiguous(candidates) => panic!(
            "budget stop did not create ambiguity: {} candidates",
            candidates.candidates.len()
        ),
    }

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(budget().limits().clone(), token);
    assert!(matches!(
        source(&snapshot, &mut cancelled),
        OperationOutcome::Incomplete(_)
    ));
}
