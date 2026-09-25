//! The named-catch range-end exception is accepted only for one proved, terminal return.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const PLAIN: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-25/multicatch-finally-return/PlainMultiCatch.class"
);
const PLAIN_RUNNER: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-25/multicatch-finally-return/PlainRunner.class"
);
const PLAIN_OUTPUT: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-25/multicatch-finally-return/plain-original-run.txt"
);
const PLAIN_JAVAP: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-25/multicatch-finally-return/plain-javap.txt"
);
const COMBINED: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-25/multicatch-finally-return/MultiCatchProbe.class"
);
const POSTFIX: &[u8] =
    include_bytes!("fixtures/p3-typed-catch-boundary-return/v8/BoundaryPostfixProbe.class");
const MONITOR: &[u8] =
    include_bytes!("fixtures/p3-typed-catch-boundary-return/v8/BoundaryMonitorProbe.class");
const POSTFIX_OUTPUT: &str = "escaped=IllegalArgumentException\ncalls=1\n";

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the frozen class opens as a standalone CLASS")
}

fn class_request(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceRequest {
    ClassSourceRequest {
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
    }
}

fn class_source(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    let request = class_request(snapshot, name);
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one standalone class answers one definition, got {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one standalone class has an unfinished selection with {} candidates",
            candidates.candidates.len()
        ),
    }
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("method `{name}` is absent"))
}

fn run<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    let ClassSourceOutcome::Recovered { report, .. } = &method(report, name).outcome else {
        panic!("method `{name}` did not retain its recovery report")
    };
    report
}

fn quoted_bcis(text: &str) -> Vec<u32> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
        .flat_map(str::split_whitespace)
        .filter_map(|bci| bci.parse().ok())
        .collect()
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-typed-catch-boundary-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create Java comparison directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write_class(&self, name: &str, bytes: &[u8]) {
        fs::write(self.path().join(format!("{name}.class")), bytes)
            .expect("write frozen class file");
    }

    fn execute(&self, class: &str) -> String {
        let output = Command::new("java")
            .args(["-Xverify:all", "-classpath"])
            .arg(self.path())
            .arg(class)
            .output()
            .expect("java is available");
        assert!(
            output.status.success(),
            "verified fixture execution failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).expect("fixture output is UTF-8")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn plain_multicatch_range_end_return_is_presented_once() {
    assert!(PLAIN_JAVAP.contains("30: ldc"), "BCI 30 producer is frozen");
    assert!(
        PLAIN_JAVAP.contains("32: areturn"),
        "BCI 32 return is frozen"
    );
    assert_eq!(
        PLAIN_JAVAP
            .lines()
            .filter(|line| line.contains("0    32    33   Class"))
            .count(),
        2,
        "both named rows share [0,32) and handler BCI 33"
    );
    let scratch = Scratch::new("plain");
    scratch.write_class("PlainMultiCatch", PLAIN);
    scratch.write_class("PlainRunner", PLAIN_RUNNER);
    assert_eq!(scratch.execute("PlainRunner"), PLAIN_OUTPUT);
    assert_eq!(PLAIN_OUTPUT.lines().count(), 4);

    let report = class_source(&open(PLAIN), "PlainMultiCatch");
    let choose = method(&report, "choose");
    let text = &choose.text;
    let body = run(&report, "choose");
    let try_at = text.find("try {").expect("the try body is written");
    let catch_at = text
        .find("catch (java.lang.IllegalArgumentException | java.lang.IllegalStateException")
        .expect("the two rows remain one multi-catch clause");
    let returned = text
        .find("return \"ok\";")
        .expect("normal return is written");
    assert!(try_at < returned && returned < catch_at, "{text}");
    assert_eq!(text.matches("catch (").count(), 1, "{text}");
    assert_eq!(text.matches("return \"ok\";").count(), 1, "{text}");
    assert!(!text.contains("jarde: not recovered"), "{text}");
    assert!(
        !quoted_bcis(text).iter().any(|bci| [30, 32].contains(bci)),
        "{text}"
    );
    assert!(
        !body.fallbacks.contains(&"jre_region_exception_edge"),
        "{text}"
    );
    assert_eq!(body.content, RecoveryContent::ContainsStatements, "{text}");
    for bci in [30, 32] {
        assert!(
            !body.source_map.of_bci(bci).is_empty(),
            "normal producer/return BCI {bci} must be represented: {text}"
        );
    }
}

#[test]
fn only_one_value_return_after_the_named_range_is_admitted() {
    let scratch = Scratch::new("postfix");
    scratch.write_class("BoundaryPostfixProbe", POSTFIX);
    assert_eq!(scratch.execute("BoundaryPostfixProbe"), POSTFIX_OUTPUT);

    let report = class_source(&open(POSTFIX), "BoundaryPostfixProbe");
    let member = method(&report, "choosePostfix");
    let body = run(&report, "choosePostfix");
    assert!(member.text.contains("@bytecode"), "{0}", member.text);
    assert!(
        !member.text.contains("return after(local0);"),
        "the unprotected call must remain quoted instead of entering the catch body:\n{}",
        member.text
    );
    assert!(
        body.fallbacks.contains(&"jre_region_exception_edge"),
        "the post-range call must leave the edge unaccounted: {:?}\n{}",
        body.fallbacks,
        member.text
    );
    for bci in [0, 2, 3, 4, 7, 8, 9, 11] {
        assert!(
            !body.source_map.of_bci(bci).is_empty(),
            "the refusal must preserve physical BCI {bci}: {:?}",
            body.source_map.segments()
        );
    }
    assert!(
        quoted_bcis(&member.text).contains(&4),
        "the unprotected call is still named by its source BCI: {}",
        member.text
    );
}

#[test]
fn finally_copy_with_catch_all_is_not_admitted_by_the_return_rule() {
    let report = class_source(&open(COMBINED), "MultiCatchProbe");
    let finally = method(&report, "chooseFinally");
    let body = run(&report, "chooseFinally");
    assert!(finally.text.contains("@bytecode"), "{}", finally.text);
    assert!(!finally.text.contains("finally {"), "{}", finally.text);
    assert!(
        body.fallbacks.contains(&"jre_guard_finally_copy"),
        "the existing finally proof remains the reason to refuse: {:?}\n{}",
        body.fallbacks,
        finally.text
    );
    for bci in [0, 30, 43, 87] {
        assert!(
            !body.source_map.of_bci(bci).is_empty(),
            "finally-copy refusal must keep region source BCI {bci}: {:?}",
            body.source_map.segments()
        );
    }
}

#[test]
fn method_and_explicit_monitors_do_not_use_the_boundary_return_exception() {
    let scratch = Scratch::new("monitor");
    scratch.write_class("BoundaryMonitorProbe", MONITOR);
    assert_eq!(scratch.execute("BoundaryMonitorProbe"), "ok\nok\ncalls=1\n");

    let report = class_source(&open(MONITOR), "BoundaryMonitorProbe");
    for name in ["synchronizedMethod", "explicitMonitor"] {
        let member = method(&report, name);
        let body = run(&report, name);
        assert!(
            !member.text.contains("return \"ok\";"),
            "{name} must not use the range-end return exception:\n{}",
            member.text
        );
        assert!(
            !body.fallbacks.is_empty(),
            "{name} retains an explicit refusal: {}",
            member.text
        );
        assert!(
            body.fallbacks.contains(&"jre_region_exception_edge"),
            "{name} must refuse the boundary edge itself: {:?}\n{}",
            body.fallbacks,
            member.text
        );
        assert!(
            member.text.contains("@bytecode"),
            "{name} retains its physical source:\n{}",
            member.text
        );
    }
}

#[test]
fn a_budget_stop_and_precancellation_publish_no_partial_boundary_return() {
    let snapshot = open(PLAIN);
    let request = class_request(&snapshot, "PlainMultiCatch");
    let complete = class_source(&snapshot, "PlainMultiCatch");

    let mut limits = complete.limits.clone();
    limits.analysis_steps = complete.usage.analysis_steps.saturating_sub(1);
    let mut bounded = Budget::new(limits);
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut bounded,
        )
        .expect("a bounded class-source request is an operation outcome");
    let stopped = match outcome {
        OperationOutcome::Incomplete(candidates) => candidates.execution,
        OperationOutcome::Performed(report) => report.execution,
        other => panic!("unexpected bounded class-source outcome: {other:?}"),
    };
    assert!(
        matches!(
            stopped,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps
                },
                ..
            }
        ),
        "the new boundary proof remains inside the bounded run: {stopped:?}"
    );

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(complete.limits.clone(), token);
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("pre-cancellation is an operation outcome");
    assert!(
        matches!(
            outcome,
            OperationOutcome::Incomplete(candidates)
                if matches!(candidates.execution, ExecutionReport::Cancelled { .. })
        ),
        "a cancelled proof cannot publish partial Java text"
    );
}
