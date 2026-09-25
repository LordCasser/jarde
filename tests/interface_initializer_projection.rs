//! Whole-class projection of the proved Java 8 interface initializer group.
//!
//! The second input has the same Code and runtime trace as the first, but swaps two physical
//! `field_info` records. The class-source report must retain that physical order while the emitted
//! declarations follow the real `<clinit>` write order; both complete sources are then recompiled
//! and run under full JVM verification.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const ORIGINAL: &[u8] =
    include_bytes!("fixtures/p3-interface-field-initializers/v8/InterfaceInitProbe.class");
const REORDERED: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/field-table-reordered/InterfaceInitProbe.class"
);
const FORWARD_READ: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/forward-binding-exception-edge/forward-binding/ForwardProbe.class"
);
const EFFECTS_SOURCE: &str =
    include_str!("fixtures/p3-interface-field-initializers/InitEffects.java");
const RUNNER_SOURCE: &str =
    include_str!("fixtures/p3-interface-field-initializers/InitRunner.java");
const BOUNDARY_EFFECTS_SOURCE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/forward-binding-exception-edge/forward-binding/original-source/BoundaryEffects.java"
);
const BOUNDARY_RUNNER_SOURCE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-22/interface-field-initializers/forward-binding-exception-edge/forward-binding/original-source/BoundaryRunner.java"
);
const EXPECTED_TRACE: &str = "ABT|A1|B2|4|7\n";
const EXPECTED_FORWARD_TRACE: &str = "L|0|9\n";

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn class_source_outcome(
    bytes: &[u8],
    class_name: &str,
    evidence: &RecoveryEvidenceRequest,
    run_budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the checked-in class fixture opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class_name),
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
    Engine::new()
        .class_source_with_evidence(slice::from_ref(&snapshot), &request, evidence, run_budget)
        .expect("the fixture answers one class-source request")
}

fn class_source(bytes: &[u8], class_name: &str) -> ClassSourceReport {
    match class_source_outcome(
        bytes,
        class_name,
        &RecoveryEvidenceRequest::all(),
        &mut budget(),
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("one fixture class answers one definition, got {other:?}"),
    }
}

fn field_names(report: &ClassSourceReport) -> Vec<String> {
    report
        .fields
        .iter()
        .map(|field| String::from_utf8(field.item.name.raw().0.clone()).expect("ASCII field name"))
        .collect()
}

fn assert_projected(report: &ClassSourceReport, physical_order: &[&str]) {
    let ClassSourceInitializerProof::Proved { fields } = &report.initializer_proof else {
        panic!(
            "the fixture's complete initializer group must be proved: {:?}",
            report.initializer_proof
        );
    };
    assert_eq!(
        field_names(report),
        physical_order
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>(),
        "report fields keep field_info order"
    );
    assert_eq!(
        fields
            .iter()
            .map(|field| field.write_order)
            .collect::<Vec<_>>(),
        vec![0, 1, 2],
        "the initializer sidecar keeps original execution order"
    );

    let json = serde_json::to_value(report).expect("the report has its public JSON form");
    let json_field_indices = json["fields"]
        .as_array()
        .expect("the JSON report keeps the physical fields array")
        .iter()
        .map(|field| {
            field["item"]["index"]
                .as_u64()
                .expect("physical field index")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        json_field_indices,
        report
            .fields
            .iter()
            .map(|field| field.item.index)
            .collect::<Vec<_>>(),
        "projection does not reorder the JSON field vector"
    );

    let source = &report.text;
    let first = source
        .find(" FIRST = InitEffects.next(\"A\")")
        .expect("FIRST initializer");
    let second = source
        .find(" SECOND = InitEffects.next(\"B\")")
        .expect("SECOND initializer");
    let total = source
        .find(" TOTAL = InitEffects.total(InterfaceInitProbe.FIRST, InterfaceInitProbe.SECOND)")
        .expect("TOTAL initializer");
    assert!(
        first < second && second < total,
        "source declarations use putstatic order:\n{source}"
    );
    assert!(
        source.contains(" CONSTANT = 7;"),
        "ConstantValue stays on its own field"
    );
    assert!(
        !source.lines().any(|line| line.trim() == "static {"),
        "the proved interface source does not contain a static initializer block:\n{source}"
    );

    let clinit = report
        .methods
        .iter()
        .find(|method| method.item.identity.name.0 == b"<clinit>")
        .expect("the original method record remains available");
    assert_eq!(clinit.item.identity.descriptor.0, b"()V");
    let ClassSourceOutcome::Recovered { report: body, .. } = &clinit.outcome else {
        panic!(
            "the original clinit recovery remains in the report: {:?}",
            clinit.outcome
        );
    };
    assert!(
        body.produced(),
        "the original clinit report is still produced"
    );
    assert!(
        !body.source_map.of_bci(fields[0].write_bci).is_empty(),
        "the projected write can still be joined to the original clinit source map"
    );
    assert!(
        clinit.text.lines().any(|line| line.trim() == "static {"),
        "the method's own spelling remains auditable even though assembly omits it"
    );
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-interface-initializer-projection-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create a JDK comparison directory");
        Self(path)
    }

    fn child(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(&path).expect("create a Java comparison case directory");
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_support(directory: &Path) -> (PathBuf, PathBuf) {
    let effects = directory.join("InitEffects.java");
    let runner = directory.join("InitRunner.java");
    fs::write(&effects, EFFECTS_SOURCE).expect("write support source");
    fs::write(&runner, RUNNER_SOURCE).expect("write runner source");
    (effects, runner)
}

fn compile_and_run(directory: &Path, sources: &[PathBuf], runner: &str, args: &[&str]) -> String {
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-classpath"])
        .arg(directory)
        .arg("-d")
        .arg(directory)
        .args(sources)
        .output()
        .expect("JDK javac is available for the complete-class regression");
    assert!(
        compile.status.success(),
        "javac rejected the complete Java 8 source:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-classpath"])
        .arg(directory)
        .arg(runner)
        .args(args)
        .output()
        .expect("JDK java is available for the complete-class regression");
    assert!(
        run.status.success(),
        "the runner failed JVM verification or execution:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the fixture trace is UTF-8")
}

fn compare_original_and_projection(
    scratch: &Scratch,
    label: &str,
    bytes: &[u8],
    report: &ClassSourceReport,
) {
    let original_dir = scratch.child(&format!("{label}-original"));
    fs::write(original_dir.join("InterfaceInitProbe.class"), bytes)
        .expect("write the frozen input class");
    let original_sources = write_support(&original_dir);
    let original_trace = compile_and_run(
        &original_dir,
        &[original_sources.0, original_sources.1],
        "InitRunner",
        &[],
    );
    assert_eq!(
        original_trace, EXPECTED_TRACE,
        "the frozen class trace remains the oracle"
    );

    let recovered_dir = scratch.child(&format!("{label}-recovered"));
    let interface = recovered_dir.join("InterfaceInitProbe.java");
    fs::write(&interface, &report.text).expect("write the entire generated interface source");
    let recovered_sources = write_support(&recovered_dir);
    let recovered_trace = compile_and_run(
        &recovered_dir,
        &[interface, recovered_sources.0, recovered_sources.1],
        "InitRunner",
        &[],
    );
    assert_eq!(
        recovered_trace, original_trace,
        "the complete regenerated class preserves effects, values, and ordering"
    );
}

#[test]
fn projects_rhs_in_putstatic_order_and_keeps_the_physical_report_order() {
    let original = class_source(ORIGINAL, "InterfaceInitProbe");
    assert_projected(&original, &["CONSTANT", "FIRST", "SECOND", "TOTAL"]);

    let reordered = class_source(REORDERED, "InterfaceInitProbe");
    assert_projected(&reordered, &["CONSTANT", "SECOND", "FIRST", "TOTAL"]);
    let scratch = Scratch::new();
    compare_original_and_projection(&scratch, "original", ORIGINAL, &original);
    compare_original_and_projection(&scratch, "reordered", REORDERED, &reordered);
}

#[test]
fn qualified_forward_read_keeps_its_default_value_after_source_projection() {
    let report = class_source(FORWARD_READ, "ForwardProbe");
    assert_eq!(field_names(&report), ["LATE", "EARLY"].map(str::to_owned));
    let ClassSourceInitializerProof::Proved { fields } = &report.initializer_proof else {
        panic!(
            "the qualified forward-read group should prove: {:?}",
            report.initializer_proof
        );
    };
    assert_eq!(
        fields
            .iter()
            .map(|field| field.write_order)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    let early = report
        .text
        .find(" EARLY = ForwardProbe.LATE")
        .expect("EARLY's explicit forward read is projected");
    let late = report
        .text
        .find(" LATE = BoundaryEffects.value()")
        .expect("LATE's runtime value is projected");
    assert!(
        early < late,
        "the default-value read remains first:\n{}",
        report.text
    );
    assert!(
        !report.text.lines().any(|line| line.trim() == "static {"),
        "the successfully projected initializer block is omitted"
    );

    let scratch = Scratch::new();
    let original_dir = scratch.child("forward-original");
    fs::write(original_dir.join("ForwardProbe.class"), FORWARD_READ)
        .expect("write the frozen forward-read class");
    let effects = original_dir.join("BoundaryEffects.java");
    let runner = original_dir.join("BoundaryRunner.java");
    fs::write(&effects, BOUNDARY_EFFECTS_SOURCE).expect("write forward-read helper");
    fs::write(&runner, BOUNDARY_RUNNER_SOURCE).expect("write reflective runner");
    let original_trace = compile_and_run(
        &original_dir,
        &[effects, runner],
        "BoundaryRunner",
        &["ForwardProbe"],
    );
    assert_eq!(original_trace, EXPECTED_FORWARD_TRACE);

    let recovered_dir = scratch.child("forward-recovered");
    let probe = recovered_dir.join("ForwardProbe.java");
    fs::write(&probe, &report.text).expect("write recovered forward-read source");
    let effects = recovered_dir.join("BoundaryEffects.java");
    let runner = recovered_dir.join("BoundaryRunner.java");
    fs::write(&effects, BOUNDARY_EFFECTS_SOURCE).expect("write forward-read helper");
    fs::write(&runner, BOUNDARY_RUNNER_SOURCE).expect("write reflective runner");
    let recovered_trace = compile_and_run(
        &recovered_dir,
        &[probe, effects, runner],
        "BoundaryRunner",
        &["ForwardProbe"],
    );
    assert_eq!(
        recovered_trace, original_trace,
        "the qualified forward reference continues to observe the default value"
    );
}

fn assert_no_runtime_initializers(report: &ClassSourceReport) {
    for field in &report.fields {
        if field.item.name.raw().0 != b"CONSTANT" {
            assert!(
                field
                    .declaration
                    .as_deref()
                    .is_some_and(|declaration| !declaration.contains(" = ")),
                "stopped projection leaves `{}` without an initializer: {:?}",
                String::from_utf8_lossy(&field.item.name.raw().0),
                field.declaration
            );
        }
    }
    let clinit = report
        .methods
        .iter()
        .find(|method| method.item.identity.name.0 == b"<clinit>")
        .expect("the original initializer member remains in the report");
    let ClassSourceOutcome::Recovered { report: body, .. } = &clinit.outcome else {
        panic!(
            "the original initializer recovery remains available: {:?}",
            clinit.outcome
        );
    };
    assert!(
        body.produced(),
        "the original initializer artifact is retained"
    );
    assert!(clinit.text.contains("InitEffects.next"));
    assert!(report.text.contains(&clinit.text));
    assert!(report.text.contains("static {"));
}

#[test]
fn default_and_all_evidence_use_the_same_atomic_projection() {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(ORIGINAL.to_vec()), &mut budget())
        .expect("the checked-in class fixture opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("InterfaceInitProbe"),
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
    let default = Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut budget())
        .expect("default class-source recovery completes");
    let all = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("all-evidence class-source recovery completes");
    let (OperationOutcome::Performed(default), OperationOutcome::Performed(all)) = (default, all)
    else {
        panic!("both evidence selections complete class-source recovery")
    };
    assert_projected(&all, &["CONSTANT", "FIRST", "SECOND", "TOTAL"]);
    assert!(matches!(
        default.initializer_proof,
        ClassSourceInitializerProof::Proved { .. }
    ));
    let first = default
        .text
        .find(" FIRST = InitEffects.next(\"A\")")
        .unwrap();
    let second = default
        .text
        .find(" SECOND = InitEffects.next(\"B\")")
        .unwrap();
    let total = default
        .text
        .find(" TOTAL = InitEffects.total(InterfaceInitProbe.FIRST, InterfaceInitProbe.SECOND)")
        .unwrap();
    assert!(first < second && second < total);
    assert_eq!(
        default.text, all.text,
        "evidence selection does not alter projection"
    );
}

#[test]
fn ir_budget_stop_keeps_the_whole_group_unprojected_and_retains_clinit() {
    let complete = class_source(ORIGINAL, "InterfaceInitProbe");
    let mut limits = complete.limits.clone();
    limits.ir_items = complete.usage.ir_items.saturating_sub(1);
    let mut constrained = Budget::new(limits);
    let outcome = class_source_outcome(
        ORIGINAL,
        "InterfaceInitProbe",
        &RecoveryEvidenceRequest::all(),
        &mut constrained,
    );
    let OperationOutcome::Performed(report) = outcome else {
        panic!("a proof-stage budget stop keeps the class report available: {outcome:?}")
    };
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::IrItems
            },
            ..
        }
    ));
    assert!(matches!(
        report.initializer_proof,
        ClassSourceInitializerProof::Refused { .. }
    ));
    assert_no_runtime_initializers(&report);
}

#[test]
fn output_budget_stop_never_publishes_a_prefix_of_the_initializer_group() {
    let complete = class_source(ORIGINAL, "InterfaceInitProbe");
    let mut limits = complete.limits.clone();
    limits.output_bytes = complete.usage.output_bytes.saturating_sub(1);
    let mut constrained = Budget::new(limits);
    let outcome = class_source_outcome(
        ORIGINAL,
        "InterfaceInitProbe",
        &RecoveryEvidenceRequest::all(),
        &mut constrained,
    );
    let OperationOutcome::Performed(report) = outcome else {
        panic!("a projection output stop keeps the class report available: {outcome:?}")
    };
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::OutputBytes
            },
            ..
        }
    ));
    assert!(matches!(
        report.initializer_proof,
        ClassSourceInitializerProof::Proved { .. }
    ));
    assert_no_runtime_initializers(&report);
}

#[test]
fn cancellation_before_projection_publishes_no_class_source_report() {
    let complete = class_source(ORIGINAL, "InterfaceInitProbe");
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(complete.limits.clone(), token);
    let outcome = class_source_outcome(
        ORIGINAL,
        "InterfaceInitProbe",
        &RecoveryEvidenceRequest::all(),
        &mut cancelled,
    );
    assert!(
        matches!(outcome, OperationOutcome::Incomplete(_)),
        "cancellation publishes no partially projected class: {outcome:?}"
    );
}
