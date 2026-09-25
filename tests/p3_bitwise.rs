//! `recover-bitwise-expressions`: integral and eager boolean `& ^ |` recovery.
//!
//! The committed input is one complete positive Java 8 class.  The effects helper and runner are
//! source-only comparison inputs; the ignored test overwrites the original class bytes after
//! compiling those helpers so the JVM oracle consumes the frozen fixture exactly.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] = include_bytes!("fixtures/p3-bitwise/v8/BitwiseProbe.class");
const PROBE_SOURCE: &str = include_str!("fixtures/p3-bitwise/BitwiseProbe.java");
const EFFECTS_SOURCE: &str = include_str!("fixtures/p3-bitwise/BitwiseEffects.java");
const RUNNER_SOURCE: &str = include_str!("fixtures/p3-bitwise/BitwiseProbeRunner.java");

const METHODS: &[(&[u8], &[u8], &[u32], &[&str])] = &[
    (b"<init>", b"()V", &[1], &["super();"]),
    (b"andInt", b"(II)I", &[0, 1, 2, 3], &["return arg0 & arg1;"]),
    (b"orInt", b"(II)I", &[0, 1, 2, 3], &["return arg0 | arg1;"]),
    (b"xorInt", b"(II)I", &[0, 1, 2, 3], &["return arg0 ^ arg1;"]),
    (
        b"andLong",
        b"(JJ)J",
        &[0, 1, 2, 3],
        &["return arg0 & arg2;"],
    ),
    (b"orLong", b"(JJ)J", &[0, 1, 2, 3], &["return arg0 | arg2;"]),
    (
        b"xorLong",
        b"(JJ)J",
        &[0, 1, 2, 3],
        &["return arg0 ^ arg2;"],
    ),
    (
        b"nested",
        b"(II)I",
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        &["return (arg0 | arg1) ^ arg0 & arg1 + 1;"],
    ),
    (
        b"complement",
        b"(I)I",
        &[0, 1, 2, 3],
        &["return arg0 ^ -1;"],
    ),
    (
        b"complementLong",
        b"(J)J",
        &[0, 1, 4, 5],
        &["return arg0 ^ -1L;"],
    ),
    (
        b"andBoolean",
        b"(ZZ)Z",
        &[0, 1, 2, 3],
        &["return arg0 & arg1;"],
    ),
    (
        b"orBoolean",
        b"(ZZ)Z",
        &[0, 1, 2, 3],
        &["return arg0 | arg1;"],
    ),
    (
        b"xorBoolean",
        b"(ZZ)Z",
        &[0, 1, 2, 3],
        &["return arg0 ^ arg1;"],
    ),
    (
        b"constant",
        b"(Z)Z",
        &[0, 1, 2, 3],
        &["return arg0 ^ true;"],
    ),
    (
        b"ordered",
        b"(ZZ)Z",
        &[0, 1, 4, 5, 8, 9],
        &["BitwiseEffects.left(arg0) & BitwiseEffects.right(arg1)"],
    ),
    (
        b"branch",
        b"(ZZ)I",
        &[0, 1, 2, 3, 6, 8, 9, 11],
        &["if (arg0 | arg1)"],
    ),
    (
        b"nested",
        b"(ZZZ)Z",
        &[0, 1, 2, 3, 4, 5, 6, 7],
        &["return arg0 & (arg1 ^ (arg2 | true));"],
    ),
    (
        b"copied",
        b"(ZZZ)Z",
        &[0, 1, 2, 3, 4, 5, 7, 9, 10, 11, 13, 15],
        &["boolean local", "arg0 | arg1", "& arg2"],
    ),
    (
        b"hoisted",
        b"(ZZZ)Z",
        &[0, 1, 2, 3, 4, 5, 8, 9, 10, 11, 12, 13],
        &["boolean local", "arg0 & arg1", "if (arg2)", "arg0 | arg1"],
    ),
    (
        b"passed",
        b"(ZZZ)I",
        &[0, 1, 2, 3, 4, 5, 8],
        &["BitwiseEffects.accept((arg0 ^ arg1) & arg2)"],
    ),
    (
        b"array",
        b"([Z)Z",
        &[0, 1, 2, 3, 4, 5, 6, 7],
        &["return arg0[0] ^ arg0[1];"],
    ),
    (
        b"promoted",
        b"(BC)I",
        &[0, 1, 2, 3],
        &["return arg0 | arg1;"],
    ),
    (
        b"integerLiteralControl",
        b"()I",
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        &["return local0 & local1 ^ local0;"],
    ),
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults to a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed bitwise fixture opens as a standalone CLASS")
}

fn request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    request_for(snapshot, "BitwiseProbe")
}

fn request_for(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceRequest {
    ClassSourceRequest {
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
    }
}

fn class_source_with_budget(
    snapshot: &ArtifactSnapshot,
    class: &str,
    budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    class_source_with_evidence_budget(snapshot, class, &RecoveryEvidenceRequest::all(), budget)
}

fn class_source_with_evidence_budget(
    snapshot: &ArtifactSnapshot,
    class: &str,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request_for(snapshot, class),
            evidence,
            budget,
        )
        .expect("the bounded class-source request reports an outcome")
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
                "no member `{}{}` in BitwiseProbe",
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

#[test]
fn fixture_has_the_complete_integral_and_boolean_surface() {
    assert!(PROBE_SOURCE.contains("return a & b;"));
    assert!(PROBE_SOURCE.contains("return a | b;"));
    assert!(PROBE_SOURCE.contains("return a ^ b;"));
    assert!(PROBE_SOURCE.contains("BitwiseEffects.left(a) & BitwiseEffects.right(b)"));
    assert!(PROBE_SOURCE.contains("boolean first=a|b;boolean copy=first;"));
    assert!(PROBE_SOURCE.contains("byte a,char b"));
    assert!(EFFECTS_SOURCE.contains("throw new IllegalStateException"));
    assert!(RUNNER_SOURCE.contains("e.getClass().getName()"));

    let report = class_source_of(&open(FIXTURE));
    assert_eq!(report.methods.len(), METHODS.len());
    for &(name, descriptor, _, snippets) in METHODS {
        let method = member(&report, name, descriptor);
        assert!(!method.text.contains("@bytecode"), "{}", method.text);
        assert_eq!(
            recovered(&report, name, descriptor).content,
            RecoveryContent::ContainsStatements,
            "{}{} is a statement body",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(descriptor)
        );
        let recovery = recovered(&report, name, descriptor);
        assert_eq!(
            recovery.representation,
            Representation::Java,
            "{}",
            method.text
        );
        assert_eq!(recovery.quality, Quality::Structured, "{}", method.text);
        for snippet in snippets {
            assert!(
                method.text.contains(snippet),
                "{}{} does not contain `{snippet}`:\n{}",
                String::from_utf8_lossy(name),
                String::from_utf8_lossy(descriptor),
                method.text
            );
        }
    }
}

#[test]
fn bitwise_source_uses_real_bcis_and_default_text_is_stable() {
    let snapshot = open(FIXTURE);
    let report = class_source_of(&snapshot);
    let default = class_source_without_evidence(&snapshot);
    assert_eq!(
        default.text, report.text,
        "evidence selection changes the artifact"
    );

    for &(name, descriptor, bcis, _) in METHODS {
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

#[test]
fn a_bitwise_class_source_budget_and_cancellation_are_bounded_outcomes() {
    let snapshot = open(FIXTURE);
    let mut output_stopped = task_budget(&[
        BudgetOverride::new("output_bytes", 1).expect("a positive output bound is valid")
    ])
    .expect("the positive output bound is valid");
    let stopped = class_source_with_budget(&snapshot, "BitwiseProbe", &mut output_stopped);
    assert!(
        matches!(stopped, OperationOutcome::Incomplete(_)),
        "a body that cannot write source does not publish a complete class: {stopped:?}"
    );

    let mut ordinary = budget();
    Engine::new()
        .class_source(
            slice::from_ref(&snapshot),
            &request(&snapshot),
            &mut ordinary,
        )
        .expect("the default class-source request completes");
    let mut source_limits = ordinary.limits().clone();
    source_limits.ir_items = ordinary.usage().ir_items.saturating_add(130);
    let mut source_stopped = Budget::new(source_limits);
    let source_outcome = class_source_with_evidence_budget(
        &snapshot,
        "BitwiseProbe",
        &RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
        &mut source_stopped,
    );
    let partial_source_map = match &source_outcome {
        OperationOutcome::Performed(report) => report.methods.iter().any(|method| {
            let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
                return false;
            };
            matches!(
                report.evidence.state(RecoveryEvidenceKind::SourceMap),
                EvidenceState::Partial { delivered } if delivered > 0
            ) && !report.source_map.is_empty()
        }),
        OperationOutcome::Ambiguous(_) | OperationOutcome::Incomplete(_) => false,
    };
    let source_map_state_summary = match &source_outcome {
        OperationOutcome::Performed(report) => report
            .methods
            .iter()
            .map(|method| {
                let state = match &method.outcome {
                    ClassSourceOutcome::Recovered { report, .. } => format!(
                        "{:?}/{}",
                        report.evidence.state(RecoveryEvidenceKind::SourceMap),
                        report.source_map.len()
                    ),
                    other => format!("{other:?}"),
                };
                format!(
                    "{}={state}",
                    String::from_utf8_lossy(&method.item.name.raw().0)
                )
            })
            .collect::<Vec<_>>(),
        OperationOutcome::Ambiguous(candidates) => {
            vec![format!(
                "ambiguous {} candidates",
                candidates.candidates.len()
            )]
        }
        OperationOutcome::Incomplete(candidates) => {
            vec![format!(
                "incomplete {} candidates",
                candidates.candidates.len()
            )]
        }
    };
    assert!(
        partial_source_map,
        "ordinary ir_items={}, source limit={}, resulting source-map states={source_map_state_summary:?}",
        ordinary.usage().ir_items,
        source_stopped.limits().ir_items
    );

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(budget().limits().clone(), token);
    let stopped = class_source_with_budget(&snapshot, "BitwiseProbe", &mut cancelled);
    assert!(
        matches!(stopped, OperationOutcome::Incomplete(_)),
        "a cancelled bitwise class request does not publish a complete class: {stopped:?}"
    );
}

#[test]
#[ignore = "requires JDK: compile deep-expression and local-dependency bounds at runtime"]
fn deep_bitwise_values_fall_back_and_local_dependencies_reach_a_bounded_fixpoint() {
    const LOCAL_COUNT: usize = 40;
    const DEEP_COUNT: usize = 32;
    let scratch = Scratch::new();
    let classes = scratch.path().join("classes");
    fs::create_dir_all(&classes).expect("create the generated-class directory");

    let deep = " & true".repeat(DEEP_COUNT);
    let mut chain = String::new();
    chain.push_str("boolean v0=seed;\n");
    for index in 1..LOCAL_COUNT {
        chain.push_str(&format!("boolean v{index}=v{} & true;\n", index - 1));
    }
    let source = format!(
        "public class BitwiseDepthProbe {{\n\
         static boolean deep(boolean seed) {{ return seed{deep}; }}\n\
         static boolean chain(boolean seed) {{ {chain}return v{}; }}\n\
         }}\n",
        LOCAL_COUNT - 1
    );
    fs::write(classes.join("BitwiseDepthProbe.java"), source).expect("write generated source");
    let compile = std::process::Command::new("javac")
        .args(["--release", "8", "-g", "-d"])
        .arg(&classes)
        .arg("BitwiseDepthProbe.java")
        .current_dir(&classes)
        .output()
        .expect("start javac");
    assert!(
        compile.status.success(),
        "the generated Java bounds compile: {}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let bytes = fs::read(classes.join("BitwiseDepthProbe.class")).expect("read generated class");
    let report = class_source_of_named(&bytes, "BitwiseDepthProbe");
    let deep = member(&report, b"deep", b"(Z)Z");
    assert!(
        deep.text.contains("@bytecode"),
        "a value deeper than the existing render bound is quoted with its producer: {}",
        deep.text
    );
    let chain = member(&report, b"chain", b"(Z)Z");
    assert_eq!(
        chain.text.matches("boolean v").count(),
        LOCAL_COUNT,
        "all local declarations in the dependency chain use the shared boolean decision: {}",
        chain.text
    );
    assert!(
        chain
            .text
            .contains(&format!("return v{};", LOCAL_COUNT - 1)),
        "the final consumer uses the decided local: {}",
        chain.text
    );
}

fn class_source_of_named(bytes: &[u8], class: &str) -> ClassSourceReport {
    let snapshot = open(bytes);
    match class_source_with_budget(&snapshot, class, &mut budget()) {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one generated class answers one definition, got {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one generated class completes within its budget, got {} unfinished candidates",
            candidates.candidates.len()
        ),
    }
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the epoch")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("jarde-p3-bitwise-{}-{nonce}", std::process::id()));
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
    fs::write(dir.join("BitwiseProbe.java"), probe).expect("write probe source");
    fs::write(dir.join("BitwiseEffects.java"), EFFECTS_SOURCE).expect("write effects source");
    fs::write(dir.join("BitwiseProbeRunner.java"), RUNNER_SOURCE).expect("write runner source");
}

fn javac(dir: &Path) {
    let output = std::process::Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(dir)
        .args([
            "BitwiseEffects.java",
            "BitwiseProbe.java",
            "BitwiseProbeRunner.java",
        ])
        .current_dir(dir)
        .output()
        .expect("start javac");
    assert!(
        output.status.success(),
        "compiling BitwiseProbe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_runner(dir: &Path) -> String {
    let output = std::process::Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(dir)
        .arg("BitwiseProbeRunner")
        .current_dir(dir)
        .output()
        .expect("execute the bitwise fixture runner");
    assert!(
        output.status.success(),
        "the bitwise fixture runner failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the bitwise runner writes UTF-8")
}

#[test]
#[ignore = "requires JDK: compile Engine::class_source output and execute a temporary runner"]
fn recovered_bitwise_probe_matches_original_runtime() {
    let report = class_source_of(&open(FIXTURE));
    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered = scratch.path().join("recovered");
    fs::create_dir_all(&original).expect("create original comparison directory");
    fs::create_dir_all(&recovered).expect("create recovered comparison directory");

    write_sources(&original, PROBE_SOURCE);
    javac(&original);
    assert_eq!(
        fs::read(original.join("BitwiseProbe.class")).expect("read the class compiled from source"),
        FIXTURE,
        "the positive original is the exact `javac --release 8 -g:none` class frozen in-tree"
    );
    fs::write(original.join("BitwiseProbe.class"), FIXTURE)
        .expect("install the frozen original class");
    let original_output = run_runner(&original);

    write_sources(&recovered, &report.text);
    javac(&recovered);
    let recovered_output = run_runner(&recovered);

    assert_eq!(original_output.lines().count(), 268);
    for marker in [
        "andI:0:0=-2147483648",
        "andJ:0:0=-9223372036854775808",
        "ordered:false:true=false:12",
        "throw:1=java.lang.IllegalStateException:left:1",
        "throw:2=java.lang.IllegalArgumentException:right:12",
        "copied:false:true:true=true",
        "hoisted:true:true:false=true",
        "passed:true:false:true=7",
        "promoted:1:32768=32769",
        "integerLiterals=1",
    ] {
        assert!(
            original_output.contains(marker),
            "the original runner covers `{marker}`:\n{original_output}"
        );
    }
    assert_eq!(
        recovered_output, original_output,
        "the complete recovered class preserves bitwise results, eager calls and exceptions"
    );
}
