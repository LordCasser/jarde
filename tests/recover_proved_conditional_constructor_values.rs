//! Public-report regression for frozen CF11 loop and conditional-construction classes.

use jarde::*;
use jarde_jvm::ir::{Quality, Representation};
use jarde_reader::budget::{Budget, CancellationToken, UsageSnapshot};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use jarde_reader::model::{ExecutionReport, TerminationReason};
use std::slice;

const FIXTURE: &[u8] = include_bytes!(
    "fixtures/java8/cf11-conditional-constructor-values/UpstreamSequentialLoops2.class"
);
const POSITIVE_FIXTURE: &[u8] = include_bytes!(
    "fixtures/java8/cf11-conditional-constructor-values/ConditionalConstructionPositive.class"
);
const INTERNAL_CLASS: &str = "cf11/UpstreamSequentialLoops2";
const METHOD: &str = "asciiToLowerCase";
const DESCRIPTOR: &str = "(Ljava/lang/String;)Ljava/lang/String;";

fn request_for(snapshot: &ArtifactSnapshot, internal_class: &str) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(internal_class),
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

fn request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    request_for(snapshot, INTERNAL_CLASS)
}

fn open_fixture(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut task_budget(&[]).expect("bounded open budget"),
        )
        .expect("the frozen standalone Java 8 class opens")
}

fn open() -> ArtifactSnapshot {
    open_fixture(FIXTURE)
}

fn source_with_evidence(
    snapshot: &ArtifactSnapshot,
    request: &ClassSourceRequest,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            request,
            evidence,
            &mut task_budget(&[]).expect("bounded task defaults"),
        )
        .expect("the public class-source request answers")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("the fixed class binds once: {other:?}"),
    }
}

fn source_default(snapshot: &ArtifactSnapshot, request: &ClassSourceRequest) -> ClassSourceReport {
    match Engine::new()
        .class_source(
            slice::from_ref(snapshot),
            request,
            &mut task_budget(&[]).expect("bounded task defaults"),
        )
        .expect("the public default class-source request answers")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("the fixed class binds once: {other:?}"),
    }
}

fn target<'a>(report: &'a ClassSourceReport) -> (&'a ClassSourceMethod, &'a RecoveryReport) {
    let method = report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == METHOD.as_bytes()
                && method.item.descriptor.raw().0 == DESCRIPTOR.as_bytes()
        })
        .expect("the fixed physical method is in the class table");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("the target has one recovery report: {:?}", method.outcome);
    };
    (method, report)
}

fn new_summary(report: &RecoveryReport) -> &str {
    &report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "jre_new_sites")
        .expect("the construction summary remains public without RuleDetails")
        .message
}

fn new_diagnostics(report: &RecoveryReport) -> Vec<(String, String)> {
    report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == "jre_new_sites" || diagnostic.code.starts_with("jre_new_")
        })
        .map(|diagnostic| (diagnostic.code.clone(), diagnostic.message.clone()))
        .collect()
}

fn assert_source_for_bci(body: &RecoveryReport, member: &ClassSourceMethod, bci: u32) {
    let segments = body.source_map.of_bci(bci);
    assert!(
        !segments.is_empty(),
        "physical constructor BCI {bci} has no map segment"
    );
    assert!(
        segments.iter().any(|segment| {
            std::iter::once(segment.origin().primary())
                .chain(segment.origin().derived())
                .any(|origin| origin.bci() == bci && origin.method() == Some(&member.item.identity))
        }),
        "BCI {bci} has no exact origin for the target physical method: {segments:?}"
    );
}

fn assert_utf8_map(body: &RecoveryReport) {
    for segment in body.source_map.segments() {
        assert!(segment.start() < segment.end());
        assert!(body.text.is_char_boundary(segment.start()));
        assert!(body.text.is_char_boundary(segment.end()));
        assert!(body.text.get(segment.start()..segment.end()).is_some());
    }
}

fn assert_complete_body(body: &RecoveryReport) {
    assert!(body.produced(), "{:?}", body.outcome);
    assert_eq!(body.representation, Representation::Java);
    assert_eq!(body.quality, Quality::Structured);
    assert!(matches!(&body.execution, ExecutionReport::Complete { .. }));
    assert!(body.fallbacks.is_empty(), "{}", body.text);
}

fn assert_range_outcomes_for_other_methods(
    report: &ClassSourceReport,
    selected_identity: &PhysicalMethodId,
    range: BytecodeRange,
    expected_invalid_methods: &[(&str, &str)],
) {
    let mut expected: Vec<(Vec<u8>, Vec<u8>)> = expected_invalid_methods
        .iter()
        .map(|(name, descriptor)| (name.as_bytes().to_vec(), descriptor.as_bytes().to_vec()))
        .collect();
    let mut observed_invalid = Vec::new();
    for method in &report.methods {
        if &method.item.identity == selected_identity {
            continue;
        }
        let key = (
            method.item.name.raw().0.clone(),
            method.item.descriptor.raw().0.clone(),
        );
        let should_refuse = expected.contains(&key);
        let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
            panic!(
                "non-target {:?} has a method-local recovery result: {:?}",
                method.item.identity, method.outcome
            );
        };
        assert_eq!(body.evidence.requested().driver_bci_range(), Some(range));
        if should_refuse {
            assert!(
                matches!(
                    body.stop(),
                    Some(StopReason::EvidenceRefused {
                        code: "jre_evidence_range_invalid",
                        ..
                    })
                ),
                "non-target {:?} records its exact out-of-method range refusal: {:?}",
                method.item.identity,
                body.stop()
            );
            assert!(!body.produced());
            assert_eq!(body.text, "");
            assert!(body.source_map.is_empty());
            assert_eq!(body.content, RecoveryContent::NotProduced);
            assert!(body.news.is_empty());
            for kind in body.evidence.requested().kinds() {
                assert_eq!(
                    body.evidence.state(kind),
                    EvidenceState::NotPerformed,
                    "the refused member did not materialize {kind:?}"
                );
            }
            observed_invalid.push(key);
        } else {
            assert!(
                body.produced(),
                "valid non-target range for {:?}",
                method.item.identity
            );
            assert!(matches!(&body.execution, ExecutionReport::Complete { .. }));
        }
    }
    expected.sort();
    observed_invalid.sort();
    assert_eq!(
        observed_invalid, expected,
        "the listed non-target members, and only those members, reject this shared driver range"
    );
}

#[test]
fn conditional_constructor_public_reports_agree_across_evidence_selections() {
    let snapshot = open();
    let request = request(&snapshot);

    let default = source_default(&snapshot, &request);
    let all = source_with_evidence(&snapshot, &request, &RecoveryEvidenceRequest::all());
    let no_rule_details = source_with_evidence(
        &snapshot,
        &request,
        &RecoveryEvidenceRequest::essential()
            .with_kind(RecoveryEvidenceKind::SourceMap)
            .with_kind(RecoveryEvidenceKind::RegionDetails)
            .with_kind(RecoveryEvidenceKind::NameDetails),
    );

    let (default_method, default_body) = target(&default);
    let (all_method, all_body) = target(&all);
    let (no_rule_method, no_rule_body) = target(&no_rule_details);
    for (label, method, body) in [
        ("default", default_method, default_body),
        ("all", all_method, all_body),
        ("without RuleDetails", no_rule_method, no_rule_body),
    ] {
        assert_complete_body(body);
        assert_utf8_map(body);
        assert!(
            !method.text.contains("@bytecode") && !method.text.contains("jarde_refused_body"),
            "{label} retained a refusal marker: {}",
            method.text
        );
    }

    assert_eq!(default.text, all.text);
    assert_eq!(default.text, no_rule_details.text);
    assert_eq!(default_body.text, all_body.text);
    assert_eq!(default_body.text, no_rule_body.text);
    assert_eq!(default_body.source_map, all_body.source_map);
    assert_eq!(default_body.source_map, no_rule_body.source_map);
    assert_eq!(
        new_diagnostics(default_body),
        new_diagnostics(all_body),
        "the construction summary and gaps do not depend on RuleDetails delivery"
    );
    assert_eq!(new_diagnostics(default_body), new_diagnostics(no_rule_body));
    assert_eq!(
        default_body
            .evidence
            .state(RecoveryEvidenceKind::RuleDetails),
        EvidenceState::NotRequested
    );
    assert_eq!(
        no_rule_body
            .evidence
            .state(RecoveryEvidenceKind::RuleDetails),
        EvidenceState::NotRequested
    );
    assert_eq!(
        all_body.evidence.state(RecoveryEvidenceKind::RuleDetails),
        EvidenceState::Complete
    );
    assert!(default_body.news.is_empty());
    assert!(no_rule_body.news.is_empty());

    assert!(
        new_diagnostics(all_body)
            .iter()
            .all(|(code, _)| code == "jre_new_sites"),
        "the successful candidate has no new@1 refusal gap: {:?}",
        new_diagnostics(all_body)
    );
    assert!(
        new_summary(default_body).contains("1 presented as `new`, 0 refused"),
        "the all-selection summary says what the single NewRecord should say: {}",
        new_summary(default_body)
    );
    assert_eq!(all_body.news.len(), 1, "one fixed allocation candidate");
    let record = &all_body.news[0];
    assert!(
        record.presented,
        "the committed conditional construction is presented: {record:?}"
    );
    assert_eq!(record.class, "java/lang/String");
    assert!(record.dup.is_some());
    assert!(record.constructor.is_some());
    assert!(record.refusal.is_none());

    let mut bytecode_budget = task_budget(&[]).expect("bounded bytecode budget");
    let inspection = inspect_method_bytecode(
        FIXTURE,
        MethodSelector {
            name: JvmBytes(METHOD.as_bytes().to_vec()),
            descriptor: JvmBytes(DESCRIPTOR.as_bytes().to_vec()),
        },
        &mut bytecode_budget,
    )
    .expect("the target physical instructions are readable");
    assert!(inspection.stopped_at.is_none());
    let instruction = |bci| {
        inspection
            .instructions
            .iter()
            .find(|instruction| instruction.bci == bci)
            .unwrap_or_else(|| panic!("recorded BCI {bci} is not a physical instruction"))
    };
    assert_eq!(instruction(record.head).opcode, 0xbb, "new opcode");
    let dup = record.dup.expect("the presented javac allocation has dup");
    assert_eq!(instruction(dup).opcode, 0x59, "dup opcode");
    let constructor = record.constructor.expect("constructor BCI");
    assert_eq!(
        instruction(constructor).opcode,
        0xb7,
        "invokespecial opcode"
    );
    for bci in std::iter::once(record.head)
        .chain(std::iter::once(dup))
        .chain(std::iter::once(constructor))
        .chain(record.arguments.iter().copied())
    {
        assert_source_for_bci(all_body, all_method, bci);
    }

    let selected_range =
        BytecodeRange::new(record.head, record.head + instruction(record.head).width);
    let ranged = source_with_evidence(
        &snapshot,
        &request,
        &RecoveryEvidenceRequest::all().with_driver_bci_range(selected_range),
    );
    let (ranged_method, ranged_body) = target(&ranged);
    assert_eq!(ranged_body.text, all_body.text);
    assert_eq!(ranged_method.text, all_method.text);
    assert_eq!(new_diagnostics(ranged_body), new_diagnostics(all_body));
    assert_eq!(
        ranged_body.news, all_body.news,
        "the target method's range selects the record by its physical allocation BCI"
    );
    assert_eq!(ranged_method.item.identity, all_method.item.identity);
    assert_utf8_map(ranged_body);
    assert!(ranged_body.source_map.len() <= all_body.source_map.len());
    assert_source_for_bci(ranged_body, ranged_method, record.head);
    assert_range_outcomes_for_other_methods(
        &ranged,
        &ranged_method.item.identity,
        selected_range,
        &[("<init>", "()V"), ("<clinit>", "()V")],
    );
}

const POSITIVE_INTERNAL_CLASS: &str = "cf11/ConditionalConstructionPositive";

fn method_by_identity<'a>(
    report: &'a ClassSourceReport,
    name: &str,
    descriptor: &str,
) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == name.as_bytes()
                && method.item.descriptor.raw().0 == descriptor.as_bytes()
        })
        .unwrap_or_else(|| panic!("missing physical method {name}{descriptor}"))
}

#[test]
fn conditional_constructor_positive_all_evidence_reports_four_exact_methods() {
    let snapshot = open_fixture(POSITIVE_FIXTURE);
    let request = request_for(&snapshot, POSITIVE_INTERNAL_CLASS);
    let all = source_with_evidence(&snapshot, &request, &RecoveryEvidenceRequest::all());

    let targets = [
        (
            "oneConstructedArm",
            "(ZZZLjava/lang/String;)Ljava/lang/String;",
            1usize,
        ),
        (
            "twoConstructedArms",
            "(ZZZZLjava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            2usize,
        ),
        ("writeStaticSlot", "(ZZZLjava/lang/String;)V", 1usize),
        ("writeInstanceSlot", "(ZZZLjava/lang/String;)V", 1usize),
    ];
    for (name, descriptor, expected_count) in targets {
        let method = method_by_identity(&all, name, descriptor);
        let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
            panic!("{name} has a public recovery report: {:?}", method.outcome);
        };
        assert_complete_body(body);
        assert_utf8_map(body);
        assert_eq!(body.news.len(), expected_count, "{name} NewRecord count");
        assert!(
            new_summary(body).contains(&format!("{expected_count} presented as `new`, 0 refused")),
            "{name} summary agrees with its records: {}",
            new_summary(body)
        );
        assert_eq!(
            new_diagnostics(body)
                .iter()
                .filter(|(code, _)| code != "jre_new_sites")
                .count(),
            0,
            "{name} has no constructor refusal gaps"
        );
        for record in &body.news {
            assert!(record.presented, "{name}: {record:?}");
            assert_eq!(record.class, "java/lang/String");
            assert!(record.dup.is_some());
            assert!(record.constructor.is_some());
            assert!(record.refusal.is_none());
            let mut bytecode_budget = task_budget(&[]).expect("bounded bytecode budget");
            let inspection = inspect_method_bytecode(
                POSITIVE_FIXTURE,
                MethodSelector {
                    name: JvmBytes(name.as_bytes().to_vec()),
                    descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
                },
                &mut bytecode_budget,
            )
            .expect("target physical instructions are readable");
            assert!(inspection.stopped_at.is_none());
            let instruction = |bci| {
                inspection
                    .instructions
                    .iter()
                    .find(|instruction| instruction.bci == bci)
                    .unwrap_or_else(|| panic!("recorded BCI {bci} is not an instruction in {name}"))
            };
            assert_eq!(instruction(record.head).opcode, 0xbb, "{name} new opcode");
            let dup = record.dup.expect("presented allocation has dup");
            assert_eq!(instruction(dup).opcode, 0x59, "{name} dup opcode");
            let constructor = record.constructor.expect("constructor BCI");
            assert_eq!(
                instruction(constructor).opcode,
                0xb7,
                "{name} invokespecial opcode"
            );
            for bci in std::iter::once(record.head)
                .chain(std::iter::once(dup))
                .chain(std::iter::once(constructor))
                .chain(record.arguments.iter().copied())
            {
                assert_source_for_bci(body, method, bci);
            }
        }
    }
}

#[test]
fn conditional_constructor_range_excluding_all_new_bcis_keeps_summary_and_text() {
    let snapshot = open_fixture(POSITIVE_FIXTURE);
    let request = request_for(&snapshot, POSITIVE_INTERNAL_CLASS);
    let all = source_with_evidence(&snapshot, &request, &RecoveryEvidenceRequest::all());
    let name = "twoConstructedArms";
    let descriptor = "(ZZZZLjava/lang/String;Ljava/lang/String;)Ljava/lang/String;";
    let (all_method, all_body) = {
        let method = method_by_identity(&all, name, descriptor);
        let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
            panic!("{name} has a report: {:?}", method.outcome);
        };
        (method, body)
    };
    assert_eq!(all_body.news.len(), 2);
    let mut bytecode_budget = task_budget(&[]).expect("bounded bytecode budget");
    let inspection = inspect_method_bytecode(
        POSITIVE_FIXTURE,
        MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        &mut bytecode_budget,
    )
    .expect("target physical instructions are readable");
    assert!(inspection.stopped_at.is_none());
    let site_bcis: Vec<u32> = all_body
        .news
        .iter()
        .flat_map(|record| {
            std::iter::once(record.head)
                .chain(record.dup)
                .chain(record.constructor)
                .chain(record.arguments.iter().copied())
        })
        .collect();
    let selected_bci = all_body
        .source_map
        .segments()
        .iter()
        .flat_map(|segment| {
            std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        })
        .filter(|origin| origin.method() == Some(&all_method.item.identity))
        .map(|origin| origin.bci())
        .find(|bci| !site_bcis.contains(bci))
        .expect("the full target map has a physical position outside both construction records");
    assert!(!site_bcis.contains(&selected_bci));
    let selected_instruction = inspection
        .instructions
        .iter()
        .find(|instruction| instruction.bci == selected_bci)
        .expect("the selected mapped BCI is a physical instruction");
    let excluded_range = BytecodeRange::new(
        selected_bci,
        selected_bci + u32::from(selected_instruction.width),
    );
    let ranged = source_with_evidence(
        &snapshot,
        &request,
        &RecoveryEvidenceRequest::all().with_driver_bci_range(excluded_range),
    );
    let ranged_method = method_by_identity(&ranged, name, descriptor);
    let ClassSourceOutcome::Recovered {
        report: ranged_body,
        ..
    } = &ranged_method.outcome
    else {
        panic!("ranged {name} has a report: {:?}", ranged_method.outcome);
    };
    assert_eq!(ranged_body.text, all_body.text);
    assert_eq!(ranged_method.text, all_method.text);
    assert_eq!(new_diagnostics(ranged_body), new_diagnostics(all_body));
    assert!(
        ranged_body.news.is_empty(),
        "the range excludes every NewRecord BCI"
    );
    assert_eq!(ranged_method.item.identity, all_method.item.identity);
    assert_utf8_map(ranged_body);
    assert!(ranged_body.source_map.len() <= all_body.source_map.len());
    assert_source_for_bci(ranged_body, ranged_method, selected_bci);
    assert_range_outcomes_for_other_methods(
        &ranged,
        &ranged_method.item.identity,
        excluded_range,
        &[
            ("condition", "(ZZ)Z"),
            ("argument", "(Ljava/lang/String;Z)Ljava/lang/String;"),
        ],
    );
}

#[test]
fn conditional_constructor_public_late_stop_and_precancel_publish_atomically() {
    let snapshot = open();
    let request = request(&snapshot);
    let complete = source_with_evidence(&snapshot, &request, &RecoveryEvidenceRequest::all());
    let complete_method = method_by_identity(&complete, METHOD, DESCRIPTOR);
    let expected_identity = complete_method.item.identity.clone();
    let (_, complete_body) = target(&complete);
    let ExecutionReport::Complete { usage } = &complete_body.execution else {
        panic!("the complete target body has measured usage");
    };
    let target_steps = usage.analysis_steps;
    assert!(target_steps > 1, "the measured target has analysis work");

    // The class also has a later static initializer. Use the target body's measured cumulative
    // usage so the stop must occur in this exact member, rather than subtracting from the class.
    let mut limits = task_budget(&[])
        .expect("bounded task defaults")
        .limits()
        .clone();
    limits.analysis_steps = target_steps - 1;
    let analysis_step_limit = limits.analysis_steps;
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits),
        )
        .expect("the late budget stop is an operation outcome");
    log_stop(&stopped);
    assert_late_analysis_stop(&stopped, &expected_identity, analysis_step_limit);

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(
        task_budget(&[])
            .expect("bounded task defaults")
            .limits()
            .clone(),
        token,
    );
    let result = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("pre-cancellation is an ordinary reported outcome");
    assert_precancel(result);
}

fn log_stop(outcome: &OperationOutcome<ClassSourceReport>) {
    match outcome {
        OperationOutcome::Performed(report) => {
            let target_stop = report.methods.iter().find_map(|method| {
                if method.item.name.raw().0 != METHOD.as_bytes()
                    || method.item.descriptor.raw().0 != DESCRIPTOR.as_bytes()
                {
                    return None;
                }
                Some(match &method.outcome {
                    ClassSourceOutcome::Recovered { report, .. } => {
                        format!("body_stop={:?}", report.stop().map(|stop| stop.at()))
                    }
                    ClassSourceOutcome::Refused {
                        execution,
                        diagnostics,
                    } => {
                        format!("body_refusal={execution:?}; diagnostics={diagnostics:?}")
                    }
                    other => format!("body_outcome={other:?}"),
                })
            });
            eprintln!(
                "conditional-constructor late stop: execution={:?}; usage={:?}; target={target_stop:?}; diagnostics={:?}",
                report.execution, report.usage, report.diagnostics
            );
        }
        OperationOutcome::Incomplete(report) => eprintln!(
            "conditional-constructor late stop: execution={:?}; diagnostics={:?}; candidates={:?}",
            report.execution, report.diagnostics, report.candidates
        ),
        OperationOutcome::Ambiguous(candidates) => {
            panic!("the fixed standalone class is not ambiguous: {candidates:?}")
        }
    }
}

fn assert_late_analysis_stop(
    outcome: &OperationOutcome<ClassSourceReport>,
    expected_identity: &PhysicalMethodId,
    analysis_step_limit: u64,
) {
    let report = match outcome {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Incomplete(report) => panic!(
            "budget stopped before the target method was reached: execution={:?}; candidates={:?}",
            report.execution, report.candidates
        ),
        OperationOutcome::Ambiguous(candidates) => {
            panic!("the fixed standalone class is not ambiguous: {candidates:?}")
        }
    };
    assert!(matches!(
        &report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps
            },
            ..
        }
    ));
    assert!(report.usage.analysis_steps > 0);
    assert!(report.usage.analysis_steps <= analysis_step_limit);
    let method = report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == METHOD.as_bytes()
                && method.item.descriptor.raw().0 == DESCRIPTOR.as_bytes()
        })
        .expect("the exact target method appears in the partial class report");
    assert_eq!(
        &method.item.identity, expected_identity,
        "the stopped member has the exact identity from the complete run"
    );
    let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
        panic!(
            "target method has a body-level stop report: {:?}",
            method.outcome
        );
    };
    assert!(
        matches!(
            &body.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps
                },
                ..
            }
        ),
        "the selected method itself stopped: {:?}",
        body.execution
    );
    assert!(
        !body.produced(),
        "the stopped method publishes no partial body"
    );
    assert!(body.text.is_empty());
    assert!(body.source_map.is_empty());
    assert_eq!(body.content, RecoveryContent::NotProduced);
    assert!(body.news.is_empty());
    let stop = body
        .stop()
        .expect("target method records its real stop location");
    assert!(
        stop.at().is_some(),
        "stop retains physical method location: {stop:?}"
    );
    let method_usage = match &body.execution {
        ExecutionReport::Partial { usage, .. } => usage,
        other => panic!("target method stop carries its own usage: {other:?}"),
    };
    eprintln!(
        "conditional-constructor target late stop: method={:?}; dimension=AnalysisSteps; report_usage={:?}; method_usage={:?}; location={:?}",
        method.item.identity,
        report.usage,
        method_usage,
        stop.at()
    );
}

fn assert_precancel(outcome: OperationOutcome<ClassSourceReport>) {
    let execution = match &outcome {
        OperationOutcome::Incomplete(report) => &report.execution,
        OperationOutcome::Performed(report) => &report.execution,
        OperationOutcome::Ambiguous(candidates) => {
            panic!("the fixed standalone class is not ambiguous: {candidates:?}")
        }
    };
    let usage = match execution {
        ExecutionReport::Cancelled { usage } => usage,
        other => panic!("pre-cancellation is stated as Cancelled: {other:?}"),
    };
    let mut counted = usage.clone();
    counted.elapsed_millis = 0;
    assert_eq!(
        counted,
        UsageSnapshot::default(),
        "pre-cancel charges no counted work"
    );
    if let OperationOutcome::Performed(report) = &outcome {
        let mut report_counted = report.usage.clone();
        report_counted.elapsed_millis = 0;
        assert_eq!(
            report_counted,
            UsageSnapshot::default(),
            "the class report also states zero counted work"
        );
    }

    match outcome {
        OperationOutcome::Incomplete(report) => {
            assert!(report.candidates.is_empty());
        }
        OperationOutcome::Performed(report) => {
            assert!(report.declaration.is_none());
            assert!(report.fields.is_empty());
            assert!(report.methods.is_empty());
            assert!(report.text.is_empty());
        }
        OperationOutcome::Ambiguous(_) => unreachable!(),
    }
}

#[test]
fn conditional_sites_are_not_presented_when_the_final_body_is_explanation_only() {
    let fixture = include_bytes!(
        "../openspec/evidence/java-syntax-2026-10-02/nested-ctor-argument-patrol/variants-nested/X4.class"
    );
    let snapshot = open_fixture(fixture);
    let request = request_for(&snapshot, "X4");
    let default = source_default(&snapshot, &request);
    let all = source_with_evidence(&snapshot, &request, &RecoveryEvidenceRequest::all());
    let default_method = method_by_identity(&default, "crossBlock", "(Z)Ljava/lang/String;");
    let all_method = method_by_identity(&all, "crossBlock", "(Z)Ljava/lang/String;");
    let ClassSourceOutcome::Recovered {
        report: default_body,
        ..
    } = &default_method.outcome
    else {
        panic!("the default run returns its refusal report");
    };
    let ClassSourceOutcome::Recovered {
        report: all_body, ..
    } = &all_method.outcome
    else {
        panic!("the all-evidence run returns its refusal report");
    };
    assert_eq!(default.text, all.text);
    assert_eq!(new_diagnostics(default_body), new_diagnostics(all_body));
    assert_eq!(all_body.representation, Representation::Mixed);
    assert_eq!(all_body.quality, Quality::Fallback);
    assert!(
        all_body.produced(),
        "the explanation artifact was committed"
    );
    assert_eq!(all_body.content, RecoveryContent::ExplanationOnly);
    assert_eq!(default_body.content, RecoveryContent::ExplanationOnly);
    assert!(
        !all_body.regions.is_empty() && all_body.regions.iter().all(|region| region.structured),
        "the decoded regions are structured; the final body builder did not emit Java statements"
    );
    assert!(
        all_body.fallbacks.is_empty(),
        "fallbacks lists Region-level fallback reasons; this failure is an explanation-only final body"
    );
    assert!(default_body.news.is_empty());
    assert_eq!(
        all_body
            .news
            .iter()
            .map(|record| record.head)
            .collect::<Vec<_>>(),
        [0, 10, 22]
    );
    assert!(all_body.news.iter().all(|record| !record.presented));
    assert_eq!(
        all_body
            .news
            .iter()
            .map(|record| record
                .refusal
                .as_ref()
                .expect("every candidate is refused")
                .code)
            .collect::<Vec<_>>(),
        [
            "jre_new_shape",
            "jre_new_conditional_not_emitted",
            "jre_new_conditional_not_emitted",
        ]
    );
    assert_eq!(
        new_summary(all_body),
        "3 construction candidate(s) read under new@1: 0 presented as `new`, 3 refused"
    );
    for record in &all_body.news[1..] {
        assert_eq!(
            record
                .refusal
                .as_ref()
                .expect("a candidate was not emitted")
                .code,
            "jre_new_conditional_not_emitted"
        );
        assert!(record.constructor.is_some());
        assert!(record.dup.is_some());
    }
}
