//! Public regression coverage for the proved entry-split integer-prefix loop.
use jarde::*;
use jarde_jvm::ir::{Quality, Representation};
use jarde_reader::budget::{Budget, CancellationToken, CountedBudgetDimension};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use jarde_reader::model::ExecutionReport;
use std::collections::BTreeSet;
use std::slice;

const POSITIVE: &[u8] = include_bytes!("fixtures/cf08-entry-split/cf08/EntrySplitCases.class");
const REFUSALS: &[u8] = include_bytes!("fixtures/cf08-entry-split/cf08/EntrySplitRefusals.class");
const CLASS: &str = "cf08/EntrySplitCases";
const REFUSAL_CLASS: &str = "cf08/EntrySplitRefusals";

fn report(bytes: &[u8], class: &str, all: bool) -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("default task budget is valid");
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("fixture class opens");
    let request = ClassSourceRequest {
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
    };
    let result = if all {
        engine.class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
    } else {
        engine.class_source(slice::from_ref(&snapshot), &request, &mut budget)
    }
    .expect("class-source request returns an outcome");
    match result {
        OperationOutcome::Performed(report) => report,
        other => panic!("fixture request completes: {other:?}"),
    }
}

fn member<'a>(
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
        .unwrap_or_else(|| panic!("missing {name}{descriptor}"))
}

fn body(method: &ClassSourceMethod) -> &RecoveryReport {
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("method has its own recovery result: {other:?}"),
    }
}

fn physical_bcis(bytes: &[u8], name: &str, descriptor: &str) -> BTreeSet<u32> {
    let mut budget = task_budget(&[]).expect("reader budget is valid");
    let code = inspect_method_bytecode(
        bytes,
        MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        &mut budget,
    )
    .expect("complete physical Code is readable");
    assert!(code.stopped_at.is_none(), "Code scan is complete");
    code.instructions
        .into_iter()
        .map(|instruction| instruction.bci)
        .collect()
}

fn assert_full_map(bytes: &[u8], method: &ClassSourceMethod, name: &str, descriptor: &str) {
    let recovered = body(method);
    let physical = physical_bcis(bytes, name, descriptor);
    let mut mapped = BTreeSet::new();
    for segment in recovered.source_map.segments() {
        assert!(segment.start() <= segment.end());
        assert!(
            recovered.text.get(segment.start()..segment.end()).is_some(),
            "UTF-8 aligned span"
        );
        for anchor in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            assert_eq!(
                anchor.method(),
                Some(&method.item.identity),
                "source owner is exact method"
            );
            assert!(
                physical.contains(&anchor.bci()),
                "anchor lies in physical Code"
            );
            mapped.insert(anchor.bci());
        }
    }
    assert_eq!(
        mapped, physical,
        "all and only physical instruction BCIs are mapped"
    );
}

fn assert_prefix_is_shown_twice(method: &ClassSourceMethod, prefix_bci: u32) {
    let recovered = body(method);
    let statements: Vec<(usize, usize, String)> = recovered
        .source_map
        .of_bci(prefix_bci)
        .into_iter()
        .filter_map(|segment| {
            let statement = segment.text(&recovered.text).trim();
            let assignment = statement.ends_with(';')
                && !statement.contains('\n')
                && !statement.starts_with("//")
                && !statement.contains("if (")
                && !statement.contains("while (")
                && (statement.contains(" += ")
                    || statement.contains(" -= ")
                    || (statement.contains(" = ")
                        && (statement.contains(" + ") || statement.contains(" - "))));
            assignment.then(|| (segment.start(), segment.end(), statement.to_owned()))
        })
        .collect();
    let mut pair = None;
    'outer: for left in 0..statements.len() {
        for right in left + 1..statements.len() {
            if statements[left].2 == statements[right].2
                && (statements[left].1 <= statements[right].0
                    || statements[right].1 <= statements[left].0)
            {
                pair = Some((&statements[left], &statements[right]));
                break 'outer;
            }
        }
    }
    assert!(
        pair.is_some(),
        "one physical prefix BCI has two disjoint identical assignment statements, excluding enclosing control spans: {}",
        recovered.text
    );
}

#[test]
fn entry_split_integer_loop_accepts_signed_slot_and_polarity_variants() {
    let default = report(POSITIVE, CLASS, false);
    let all = report(POSITIVE, CLASS, true);
    assert_eq!(default.text, all.text);
    assert_eq!(
        default.methods.len(),
        5,
        "constructor plus four fixture methods"
    );
    for (name, descriptor, prefix_bci) in [
        ("test", "(IZI)I", 4),
        ("signed", "(IZI)I", 4),
        ("shifted", "(IIIZI)I", 4),
        ("inverted", "(IZI)I", 14),
    ] {
        let ordinary = member(&default, name, descriptor);
        let evidenced = member(&all, name, descriptor);
        let recovered = body(ordinary);
        assert_eq!(
            ordinary.text, evidenced.text,
            "{name}: evidence mode preserves member text"
        );
        assert_eq!(
            recovered.source_map,
            body(evidenced).source_map,
            "{name}: map is stable"
        );
        assert_eq!(
            recovered.quality,
            Quality::Structured,
            "{name}: {}",
            recovered.text
        );
        assert_eq!(recovered.representation, Representation::Java);
        assert!(
            !recovered.text.contains("@bytecode"),
            "{name}: {}",
            recovered.text
        );
        assert!(
            recovered.text.contains("if ("),
            "initial path is explicit: {}",
            recovered.text
        );
        assert!(
            recovered.text.contains("while ("),
            "ordinary loop is explicit: {}",
            recovered.text
        );
        assert!(
            recovered.text.contains("return "),
            "actual exit value is retained: {}",
            recovered.text
        );
        assert_prefix_is_shown_twice(ordinary, prefix_bci);
        assert_full_map(POSITIVE, ordinary, name, descriptor);
    }
}

#[test]
fn entry_split_controls_without_the_complete_int_proof_remain_quoted() {
    let default = report(REFUSALS, REFUSAL_CLASS, false);
    let all = report(REFUSALS, REFUSAL_CLASS, true);
    assert_eq!(default.text, all.text);
    assert_eq!(
        default.methods.len(),
        5,
        "constructor plus four verifier-valid controls"
    );
    for (name, descriptor) in [
        ("booleanPrefix", "(ZZI)I"),
        ("bytePrefix", "(BZI)I"),
        ("alias", "(IZ)I"),
        ("callPrefix", "(IZI)I"),
    ] {
        let ordinary = member(&default, name, descriptor);
        let evidenced = member(&all, name, descriptor);
        let recovered = body(ordinary);
        assert_eq!(
            ordinary.text, evidenced.text,
            "{name}: evidence mode preserves refusal"
        );
        assert_eq!(
            recovered.source_map,
            body(evidenced).source_map,
            "{name}: refusal map is stable"
        );
        assert_eq!(
            recovered.quality,
            Quality::Fallback,
            "{name}: {}",
            recovered.text
        );
        assert!(
            recovered.text.contains("@bytecode"),
            "no partial structured prefix: {}",
            recovered.text
        );
        assert_full_map(REFUSALS, ordinary, name, descriptor);
    }
}

#[test]
fn entry_split_candidate_stops_do_not_publish_a_half_prefix() {
    let mut open_budget = task_budget(&[]).expect("valid open budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(POSITIVE.to_vec()), &mut open_budget)
        .expect("fixture opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(CLASS),
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
    for dimension in ["analysis_steps", "output_bytes"] {
        let mut budget =
            task_budget(&[BudgetOverride::new(dimension, 1).expect("valid bounded override")])
                .expect("budget override is legal");
        let stopped = Engine::new()
            .class_source(slice::from_ref(&snapshot), &request, &mut budget)
            .expect("a budget stop is a report");
        assert!(
            matches!(
                stopped,
                OperationOutcome::Incomplete(_)
                    | OperationOutcome::Performed(ClassSourceReport {
                        execution: ExecutionReport::Partial { .. },
                        ..
                    })
            ),
            "{dimension}: {stopped:?}"
        );
    }
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(Limits::default(), token);
    let stopped = Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut cancelled)
        .expect("cancellation is a report");
    let OperationOutcome::Incomplete(candidates) = stopped else {
        panic!("pre-cancellation must stop target binding: {stopped:?}");
    };
    assert!(matches!(
        candidates.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert!(candidates.candidates.is_empty());
}

const TOPOLOGY: &[u8] =
    include_bytes!("fixtures/cf08-entry-split/cf08/EntrySplitTopologyRefusals.class");
const LEGACY: &[u8] = include_bytes!("fixtures/cf08-entry-split/cf08/EntrySplitLegacyCases.class");
const TOPOLOGY_CLASS: &str = "cf08/EntrySplitTopologyRefusals";
const LEGACY_CLASS: &str = "cf08/EntrySplitLegacyCases";

#[test]
fn entry_split_topology_and_stack_controls_refuse_whole_methods() {
    let default = report(TOPOLOGY, TOPOLOGY_CLASS, false);
    let all = report(TOPOLOGY, TOPOLOGY_CLASS, true);
    assert_eq!(default.text, all.text);
    assert_eq!(
        default.methods.len(),
        5,
        "constructor plus four verifier candidates"
    );
    for (name, descriptor) in [
        ("extraEntry", "(IZI)I"),
        ("extraExit", "(IZI)I"),
        ("extraLive", "(IZZI)I"),
        ("stackEscape", "(IZI)I"),
    ] {
        let ordinary = member(&default, name, descriptor);
        let evidenced = member(&all, name, descriptor);
        let recovered = body(ordinary);
        assert_eq!(ordinary.text, evidenced.text);
        assert_eq!(recovered.source_map, body(evidenced).source_map);
        assert_eq!(
            recovered.quality,
            Quality::Fallback,
            "{name}: {}",
            recovered.text
        );
        assert!(
            recovered.text.contains("@bytecode"),
            "no partial body: {}",
            recovered.text
        );
        assert_full_map(TOPOLOGY, ordinary, name, descriptor);
    }
}

#[test]
fn entry_split_legacy_subroutine_remains_outside_java8_proof() {
    let default = report(LEGACY, LEGACY_CLASS, false);
    let all = report(LEGACY, LEGACY_CLASS, true);
    let ordinary = member(&default, "legacy", "(IZI)I");
    let evidenced = member(&all, "legacy", "(IZI)I");
    assert_eq!(ordinary.text, evidenced.text);
    assert_eq!(body(ordinary).source_map, body(evidenced).source_map);
    assert_eq!(body(ordinary).quality, Quality::Fallback);
    assert_eq!(body(ordinary).representation, Representation::Mixed);
    assert_eq!(
        body(ordinary)
            .fallbacks
            .iter()
            .filter(|code| **code == "jre_region_subroutine_entry")
            .count(),
        2
    );
    assert!(
        body(ordinary)
            .fallbacks
            .contains(&"jre_region_uncovered_blocks")
    );
    assert!(body(ordinary).text.contains("if (!arg1)"));
    assert!(body(ordinary).text.contains("if (arg2 != 0)"));
    assert!(body(ordinary).text.contains("return arg0;"));
    assert!(body(ordinary).text.contains("@bytecode 4"));
    assert!(body(ordinary).text.contains("@bytecode 14 17"));
    assert_full_map(LEGACY, ordinary, "legacy", "(IZI)I");
}

const EXCEPTION: &[u8] =
    include_bytes!("fixtures/cf08-entry-split/cf08/EntrySplitExceptionRefusal.class");
const EXCEPTION_CLASS: &str = "cf08/EntrySplitExceptionRefusal";

#[test]
fn entry_split_exceptional_prefix_remains_whole_method_refusal() {
    let default = report(EXCEPTION, EXCEPTION_CLASS, false);
    let all = report(EXCEPTION, EXCEPTION_CLASS, true);
    assert_eq!(default.text, all.text);
    assert_eq!(
        default.methods.len(),
        2,
        "constructor and the exception control"
    );
    let ordinary = member(&default, "exceptionPrefix", "(IZII)I");
    let evidenced = member(&all, "exceptionPrefix", "(IZII)I");
    assert_eq!(ordinary.text, evidenced.text);
    assert_eq!(body(ordinary).source_map, body(evidenced).source_map);
    assert_eq!(
        body(ordinary).quality,
        Quality::Fallback,
        "{}",
        body(ordinary).text
    );
    assert!(body(ordinary).text.contains("@bytecode"));
    let mut budget = task_budget(&[]).expect("reader budget is valid");
    let code = inspect_method_bytecode(
        EXCEPTION,
        MethodSelector {
            name: JvmBytes(b"exceptionPrefix".to_vec()),
            descriptor: JvmBytes(b"(IZII)I".to_vec()),
        },
        &mut budget,
    )
    .expect("exception control Code is inspectable");
    assert!(code.stopped_at.is_none());
    assert_eq!(code.exception_handler_count, 1);
    assert_eq!(code.exception_handlers.len(), 1);
    assert_eq!(
        (
            code.exception_handlers[0].start_bci,
            code.exception_handlers[0].end_bci,
            code.exception_handlers[0].handler_bci
        ),
        (4, 8, 20)
    );
    assert_full_map(EXCEPTION, ordinary, "exceptionPrefix", "(IZII)I");
}

#[test]
fn entry_split_analysis_request_can_omit_ssa_without_fabricating_that_stage() {
    let engine = Engine::new();
    let mut open_budget = task_budget(&[]).expect("valid bounded default task budget");
    let snapshot = engine
        .open(ArtifactInput::bytes(POSITIVE.to_vec()), &mut open_budget)
        .expect("positive candidate bytes open");
    let mut header_budget = task_budget(&[]).expect("valid bounded default task budget");
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut header_budget,
            InspectionMode::Strict,
        )
        .expect("target header is inspectable");
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let requested = AnalysisStage::ALL[..AnalysisStage::ALL.len() - 1].to_vec();
    let request = MethodAnalysisRequest {
        environment: ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: domain.clone(),
            },
            domains: vec![domain],
            providers: Vec::new(),
        },
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: inspected.source.class_bytes,
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(b"test".to_vec()),
            descriptor: JvmBytes(b"(IZI)I".to_vec()),
        },
        stages: requested.clone(),
    };
    let mut analysis_budget = task_budget(&[]).expect("valid bounded default task budget");
    let result = engine
        .recover_method(
            std::slice::from_ref(&snapshot),
            &request,
            &mut analysis_budget,
        )
        .expect("a request for the available prefix is answered");
    assert_eq!(result.analysis().requested_stages, requested);
    assert_eq!(result.analysis().stages.len(), AnalysisStage::ALL.len() - 1);
    assert!(
        result
            .analysis()
            .stages
            .iter()
            .all(|stage| stage.stage != AnalysisStage::Ssa),
        "the absent SSA stage must remain absent in the actual report"
    );
    let recovered = result.recovery();
    assert_eq!(
        recovered.outcome,
        RecoveryOutcome::Stopped(StopReason::IrTableMissing { table: "ssa" })
    );
    assert_eq!(recovered.content, RecoveryContent::NotProduced);
    assert!(recovered.text.is_empty() && recovered.source_map.is_empty());
    assert_eq!(recovered.representation, Representation::Bytecode);
    assert_eq!(
        recovered
            .diagnostics
            .first()
            .map(|diagnostic| diagnostic.code.as_str()),
        Some("jre_ir_table_missing")
    );
}

#[test]
fn entry_split_accepts_nonzero_value_slot_and_counter_up() {
    const INPUT: &[u8] =
        include_bytes!("fixtures/cf08-entry-split/cf08/EntrySplitSlotCounterCases.class");
    let default = report(INPUT, "cf08/EntrySplitSlotCounterCases", false);
    let all = report(INPUT, "cf08/EntrySplitSlotCounterCases", true);
    assert_eq!(default.text, all.text);
    assert_eq!(default.methods.len(), 3);
    for (name, descriptor) in [("permuted", "(ZII)I"), ("countUp", "(IZI)I")] {
        let ordinary = member(&default, name, descriptor);
        let evidenced = member(&all, name, descriptor);
        assert_eq!(body(ordinary).quality, Quality::Structured);
        assert_eq!(body(ordinary).representation, Representation::Java);
        assert_eq!(ordinary.text, evidenced.text);
        assert_eq!(body(ordinary).source_map, body(evidenced).source_map);
        assert_full_map(INPUT, ordinary, name, descriptor);
        assert_prefix_is_shown_twice(ordinary, 4);
    }
}

// Append to p3_entry_split_int_loops.rs. Test-only; no production seam or mutable IR construction.

fn entry_split_method_request(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    name: &str,
    descriptor: &str,
    stages: Vec<AnalysisStage>,
) -> MethodAnalysisRequest {
    let mut header_budget = task_budget(&[]).expect("header budget is valid");
    let inspected = engine
        .inspect_header(
            snapshot,
            ClassTarget::Root,
            &mut header_budget,
            InspectionMode::Strict,
        )
        .expect("fixture header is inspectable");
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    MethodAnalysisRequest {
        environment: ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: domain.clone(),
            },
            domains: vec![domain],
            providers: Vec::new(),
        },
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: inspected.source.class_bytes,
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        stages,
    }
}

fn entry_split_recover_with_budget(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    request: &MethodAnalysisRequest,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> RecoveredMethod {
    engine
        .recover_method_with_evidence(std::slice::from_ref(snapshot), request, evidence, budget)
        .expect("a budget boundary is returned in the report")
}

fn prefix_assignment_spans(report: &RecoveryReport, bci: u32) -> Vec<(usize, usize, String)> {
    let mut spans: Vec<_> = report
        .source_map
        .of_bci(bci)
        .into_iter()
        .filter_map(|segment| {
            let text = segment.text(&report.text).trim();
            (text.ends_with(';')
                && !text.contains('\n')
                && !text.starts_with("//")
                && !text.contains("if (")
                && !text.contains("while (")
                && (text.contains(" += ")
                    || text.contains(" -= ")
                    || (text.contains(" = ") && (text.contains(" + ") || text.contains(" - ")))))
            .then(|| (segment.start(), segment.end(), text.to_owned()))
        })
        .collect();
    spans.sort_by_key(|span| span.0);
    spans
}

fn assert_map_segments_are_a_valid_prefix(
    report: &RecoveryReport,
    method: &PhysicalMethodId,
    physical: &BTreeSet<u32>,
) {
    for segment in report.source_map.segments() {
        assert!(segment.start() <= segment.end());
        assert!(
            report.text.get(segment.start()..segment.end()).is_some(),
            "UTF-8 span boundary"
        );
        for anchor in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            assert_eq!(
                anchor.method(),
                Some(method),
                "map prefix stays bound to the exact target method"
            );
            assert!(
                physical.contains(&anchor.bci()),
                "partial map does not invent a physical BCI"
            );
        }
    }
}

#[test]
fn entry_split_output_byte_stops_hit_each_prefix_copy_without_publishing_text() {
    let engine = Engine::new();
    let mut open_budget = task_budget(&[]).expect("fixture open budget is valid");
    let snapshot = engine
        .open(ArtifactInput::bytes(POSITIVE.to_vec()), &mut open_budget)
        .expect("positive fixture opens");
    let request = entry_split_method_request(
        &engine,
        &snapshot,
        "test",
        "(IZI)I",
        AnalysisStage::ALL.to_vec(),
    );
    let mut full_budget = task_budget(&[]).expect("full budget is valid");
    let full = entry_split_recover_with_budget(
        &engine,
        &snapshot,
        &request,
        &RecoveryEvidenceRequest::all(),
        &mut full_budget,
    );
    let full_report = full.recovery();
    assert!(full_report.produced());
    let spans = prefix_assignment_spans(full_report, 4);
    assert_eq!(
        spans.len(),
        2,
        "two emitted copies are independently source-mapped: {}",
        full_report.text
    );
    assert_eq!(spans[0].2, spans[1].2);
    assert!(
        spans[0].1 <= spans[1].0,
        "two statement spans do not overlap"
    );

    let mut essential_budget = task_budget(&[]).expect("essential budget is valid");
    let essential = entry_split_recover_with_budget(
        &engine,
        &snapshot,
        &request,
        &RecoveryEvidenceRequest::essential(),
        &mut essential_budget,
    );
    assert!(essential.recovery().produced());
    assert_eq!(
        essential.recovery().text,
        full_report.text,
        "evidence selection does not change body bytes"
    );
    assert!(matches!(
        essential
            .recovery()
            .evidence
            .state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::NotRequested
    ));

    let max_output = full_budget.usage().output_bytes;
    assert!(max_output > 1);
    for (copy_index, span) in spans.iter().enumerate() {
        // Reading the standalone class charges the same output budget before IR publication.
        // Start above that known cost; the accepted cut must still be inside the emitter span.
        let mut low = POSITIVE.len() as u64 + 1;
        assert!(
            max_output > low,
            "the completed run includes class-read and emitter bytes"
        );
        let mut high = max_output;
        let mut found = None;
        for _ in 0..32 {
            if low > high {
                break;
            }
            let limit = low + (high - low) / 2;
            let mut budget = task_budget(&[
                BudgetOverride::new("output_bytes", limit).expect("positive output limit")
            ])
            .expect("limited output budget is valid");
            let stopped = entry_split_recover_with_budget(
                &engine,
                &snapshot,
                &request,
                &RecoveryEvidenceRequest::essential(),
                &mut budget,
            );
            match &stopped.recovery().outcome {
                RecoveryOutcome::Stopped(StopReason::Budget {
                    dimension: CountedBudgetDimension::OutputBytes,
                    written,
                    at,
                    ..
                }) => {
                    assert_eq!(stopped.recovery().content, RecoveryContent::NotProduced);
                    assert!(
                        stopped.recovery().text.is_empty()
                            && stopped.recovery().source_map.is_empty()
                    );
                    assert!(
                        budget.usage().output_bytes > 0 && budget.usage().output_bytes <= limit
                    );
                    if *written >= span.0 as u64 && *written < span.1 as u64 {
                        assert_eq!(
                            *at,
                            Some(4),
                            "the output stop is attributed to the physical prefix instruction"
                        );
                        eprintln!(
                            "ENTRY_SPLIT_CUT phase=prefix_{copy_index} limit={limit} span={}..{} written={written} at={at:?} usage={:?}",
                            span.0,
                            span.1,
                            budget.usage()
                        );
                        found = Some((limit, *written, budget.usage().output_bytes));
                        break;
                    }
                    if *written < span.0 as u64 {
                        low = limit + 1;
                    } else {
                        high = limit.saturating_sub(1);
                    }
                }
                RecoveryOutcome::Produced => {
                    high = limit.saturating_sub(1);
                }
                other => panic!("unexpected output stop for copy {copy_index}: {other:?}"),
            }
        }
        assert!(
            found.is_some(),
            "no output bound cut inside prefix copy {copy_index} span {:?}; full output usage {}",
            span,
            max_output
        );
        let (_, written, charged) = found.unwrap();
        assert!(
            charged >= written,
            "stopped bytes remain charged; no budget refund"
        );
    }
}

#[test]
fn entry_split_sourcemap_stop_keeps_the_complete_body_and_only_a_reliable_prefix() {
    let engine = Engine::new();
    let mut open_budget = task_budget(&[]).expect("fixture open budget is valid");
    let snapshot = engine
        .open(ArtifactInput::bytes(POSITIVE.to_vec()), &mut open_budget)
        .expect("positive fixture opens");
    let request = entry_split_method_request(
        &engine,
        &snapshot,
        "test",
        "(IZI)I",
        AnalysisStage::ALL.to_vec(),
    );
    let evidence = RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap);
    let mut full_budget = task_budget(&[]).expect("full map budget is valid");
    let full =
        entry_split_recover_with_budget(&engine, &snapshot, &request, &evidence, &mut full_budget);
    let full_report = full.recovery();
    assert!(full_report.produced());
    assert!(matches!(
        full_report.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete
    ));
    let class_report = report(POSITIVE, CLASS, false);
    assert_eq!(
        full_report.text,
        body(member(&class_report, "test", "(IZI)I")).text
    );
    let full_map_len = full_report.source_map.len();
    assert!(full_map_len > 1);

    let mut essential_budget = task_budget(&[]).expect("body-only budget is valid");
    let essential = entry_split_recover_with_budget(
        &engine,
        &snapshot,
        &request,
        &RecoveryEvidenceRequest::essential(),
        &mut essential_budget,
    );
    assert!(essential.recovery().produced());
    assert_eq!(essential.recovery().text, full_report.text);
    let complete_items = full_budget.usage().ir_items;
    let core_items = essential_budget.usage().ir_items;
    assert!(complete_items > core_items);

    let mut low = core_items.saturating_add(1);
    let mut high = complete_items.saturating_sub(1);
    let mut found = None;
    for _ in 0..32 {
        if low > high {
            break;
        }
        let limit = low + (high - low) / 2;
        let mut budget =
            task_budget(&[BudgetOverride::new("ir_items", limit).expect("positive IR limit")])
                .expect("source map limit is valid");
        let partial =
            entry_split_recover_with_budget(&engine, &snapshot, &request, &evidence, &mut budget);
        let report = partial.recovery();
        match report.evidence.state(RecoveryEvidenceKind::SourceMap) {
            EvidenceState::Partial { delivered } => {
                assert!(
                    report.produced(),
                    "the completed Java body remains published"
                );
                assert_eq!(
                    report.text, full_report.text,
                    "the map-phase stop does not remove body text"
                );
                assert!(!report.source_map.is_empty() && report.source_map.len() < full_map_len);
                assert_eq!(
                    delivered as usize,
                    report.source_map.len(),
                    "only complete recorded map spans are delivered"
                );
                assert_map_segments_are_a_valid_prefix(
                    report,
                    &request.method,
                    &physical_bcis(POSITIVE, "test", "(IZI)I"),
                );
                assert!(
                    matches!(
                        &report.execution,
                        ExecutionReport::Partial {
                            reason: TerminationReason::BudgetExceeded {
                                dimension: BudgetDimension::IrItems
                            },
                            ..
                        }
                    ),
                    "the actual IR budget caused the evidence stop: {:?}",
                    report.execution
                );
                assert!(
                    report
                        .source_map
                        .segments()
                        .iter()
                        .all(|segment| full_report.source_map.segments().contains(segment)),
                    "each delivered span is an entire node from the complete map"
                );
                assert!(budget.usage().ir_items > core_items && budget.usage().ir_items <= limit);
                eprintln!(
                    "ENTRY_SPLIT_CUT phase=source_map limit={limit} delivered={delivered} full_map={full_map_len} execution={:?} usage={:?}",
                    report.execution,
                    budget.usage()
                );
                found = Some((limit, delivered, budget.usage().ir_items));
                break;
            }
            EvidenceState::Complete => high = limit.saturating_sub(1),
            EvidenceState::NotPerformed | EvidenceState::NotRequested => {
                low = limit.saturating_add(1)
            }
        }
    }
    assert!(
        found.is_some(),
        "bounded IR-limit search found no SourceMap partial-prefix cut: core={core_items}, full={complete_items}"
    );
}

// The suite already imports Budget and CancellationToken; this additional reader budget type is
// needed by the exact output-byte StopReason matcher above:
// use jarde_reader::budget::CountedBudgetDimension;
