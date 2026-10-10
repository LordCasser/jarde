//! Real-class CF12 regression for conditional switch fallthrough and exact switch-break origins.
//!
//! This draft intentionally asserts structural invariants and physical provenance rather than a
//! guessed full text/map snapshot. Whole-class compilation/runtime equality is a separate replay.

use jarde_java::{
    ArtifactSubject, DebugLocal, DeclaringClass, MethodFacts, RecoveryEvidenceKind,
    RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::classfile::{
    LocalDebugTable, LocalDebugTypeTable, MethodSelector, inspect_method_bytecode,
};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const CLASS: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/capture/TestSwitchWithFallThroughCase.test/input/TestSwitchWithFallThroughCase$TestCls.class"
);
const CLASS_BLAKE3: &str = "92a103230cd6cbb8ccc535ca33a838ad3c91265b1b784d5ad5a69db824b17edd";
const OWNER: &str = "jadx/tests/integration/switches/TestSwitchWithFallThroughCase$TestCls";
const NAME: &str = "test";
const DESCRIPTOR: &str = "(IZZ)Ljava/lang/String;";

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 100,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 10,
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 16,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

// Keep aligned with the production facade: facts come from the same reader MethodCodeFacts that
// supplied MethodIr, including debug type metadata only when its matching local row is unique.
fn reader_debug_locals(code: Option<&jarde_reader::classfile::MethodCodeFacts>) -> Vec<DebugLocal> {
    let Some(code) = code else {
        return Vec::new();
    };
    let LocalDebugTable::Read(records) = code.debug() else {
        return Vec::new();
    };
    let generic_records = match code.generic_debug() {
        LocalDebugTypeTable::Read(records) => records.as_slice(),
        LocalDebugTypeTable::Absent | LocalDebugTypeTable::Unstated => &[],
    };
    let mut local_counts = std::collections::BTreeMap::new();
    for record in records {
        let key = (
            record.slot,
            record.start_bci,
            record.end_bci,
            record.name.0.as_slice(),
        );
        *local_counts.entry(key).or_insert(0usize) += 1;
    }
    let mut generic_by_local = std::collections::BTreeMap::new();
    for generic in generic_records {
        let key = (
            generic.slot,
            generic.start_bci,
            generic.end_bci,
            generic.name.0.as_slice(),
        );
        generic_by_local
            .entry(key)
            .or_insert_with(Vec::new)
            .push(generic);
    }
    records
        .iter()
        .map(|record| {
            let key = (
                record.slot,
                record.start_bci,
                record.end_bci,
                record.name.0.as_slice(),
            );
            let local = DebugLocal::over(
                record.slot,
                record.name_lossy(),
                record.start_bci,
                record.end_bci,
            );
            if local_counts.get(&key) == Some(&1)
                && let Some([generic]) = generic_by_local.get(&key).map(Vec::as_slice)
            {
                local.with_type_metadata(record.descriptor.0.clone(), generic.signature.0.clone())
            } else {
                local
            }
        })
        .collect()
}

struct Run {
    report: jarde_java::RecoveryReport,
    method: PhysicalMethodId,
    ordinal: jarde_reader::prepared::MethodOrdinal,
    recovery_usage: Option<jarde_reader::budget::UsageSnapshot>,
}

fn recover_target(evidence: RecoveryEvidenceRequest) -> Run {
    recover_target_with_recovery_budget(evidence, None)
}

fn recover_target_with_recovery_budget(
    evidence: RecoveryEvidenceRequest,
    recovery_budget: Option<Budget>,
) -> Run {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
        .expect("complete frozen class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(CLASS).to_hex().to_string()),
            length: u64::try_from(CLASS.len()).expect("frozen class length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
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
    let method = PhysicalMethodId {
        owner: definition,
        name: JvmBytes(NAME.as_bytes().to_vec()),
        descriptor: JvmBytes(DESCRIPTOR.as_bytes().to_vec()),
    };
    let analysis = analyze_method_ir(
        &[snapshot.clone()],
        &MethodAnalysisRequest {
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
            method,
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("full Java 8 method analysis succeeds");
    let method = analysis.report().method.clone();
    assert_eq!(method.name.0.as_slice(), NAME.as_bytes());
    assert_eq!(method.descriptor.0.as_slice(), DESCRIPTOR.as_bytes());
    assert_eq!(method.owner.class_bytes.digest.0, CLASS_BLAKE3);
    assert_eq!(method.owner.class_bytes.length, CLASS.len() as u64);

    // Establish the member ordinal by reading this exact class's method table, not by counting or
    // assuming selector order. This is the same subject contract used by the current real-class
    // source-origin tests and production facade.
    let prepared_read = snapshot
        .prepared_root_class(&mut budget)
        .expect("same frozen root class prepares");
    let prepared = jarde_reader::prepared::PreparedClass::prepare(&prepared_read, &mut budget)
        .expect("same class method table reads completely");
    let ordinals = prepared.locate_method(NAME.as_bytes(), DESCRIPTOR.as_bytes());
    assert_eq!(ordinals.len(), 1, "target declaration is unique");
    let ordinal = ordinals[0];
    let subject = ArtifactSubject::new(
        method.clone(),
        Some(ordinal),
        analysis.report().environment_identity.clone(),
    );

    let declaration = analysis
        .ir()
        .declaration()
        .expect("reader states declaration");
    assert_eq!(declaration.class_name().0.as_slice(), OWNER.as_bytes());
    assert_eq!(declaration.name().0.as_slice(), NAME.as_bytes());
    assert_eq!(declaration.descriptor().0.as_slice(), DESCRIPTOR.as_bytes());
    let facts = RecoveryFacts::new(
        MethodFacts::new(
            String::from_utf8_lossy(&declaration.name().0).into_owned(),
            String::from_utf8_lossy(&declaration.descriptor().0).into_owned(),
            declaration.parameter_slots(),
        )
        .with_access_flags(declaration.access_flags())
        .with_declaring_class(
            DeclaringClass::new(
                String::from_utf8_lossy(&declaration.class_name().0).into_owned(),
                declaration.class_access_flags(),
            )
            .with_inner_class_members(
                declaration
                    .inner_class_members()
                    .iter()
                    .map(|member| String::from_utf8_lossy(&member.0).replace('/', ".")),
            ),
        ),
    )
    .with_debug_locals(reader_debug_locals(analysis.ir().code()));
    let request = RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
        .with_evidence(evidence)
        .with_subject(subject);
    let (report, recovery_usage) = match recovery_budget {
        Some(mut recovery_budget) => {
            let report = recover(&request, &mut recovery_budget);
            (report, Some(recovery_budget.usage()))
        }
        None => (recover(&request, &mut budget), None),
    };
    if report.produced() {
        let binding = report
            .artifact
            .binding()
            .expect("produced artifact is bound");
        assert_eq!(binding.method(), &method);
        assert_eq!(binding.member_ordinal(), Some(ordinal));
        assert_eq!(
            binding.environment(),
            &analysis.report().environment_identity
        );
    }
    Run {
        report,
        method,
        ordinal,
        recovery_usage,
    }
}

fn physical_instructions() -> Vec<jarde_reader::classfile::InstructionFact> {
    let inspection = inspect_method_bytecode(
        CLASS,
        MethodSelector {
            name: JvmBytes(NAME.as_bytes().to_vec()),
            descriptor: JvmBytes(DESCRIPTOR.as_bytes().to_vec()),
        },
        &mut Budget::new(limits()),
    )
    .expect("reader inspects the exact target method");
    assert!(
        inspection.stopped_at.is_none(),
        "complete physical instruction stream"
    );
    inspection.instructions
}

fn assert_physical_coverage(report: &jarde_java::RecoveryReport, method: &PhysicalMethodId) {
    let instructions = physical_instructions();
    for instruction in instructions {
        let segments = report.source_map.of_bci(instruction.bci);
        assert!(
            !segments.is_empty(),
            "BCI {} has no source: {}",
            instruction.bci,
            report.text
        );
        assert!(
            segments.iter().any(|segment| {
                std::iter::once(segment.origin().primary())
                    .chain(segment.origin().derived())
                    .any(|origin| {
                        origin.bci() == instruction.bci && origin.method() == Some(method)
                    })
            }),
            "BCI {} has no origin from the exact physical method: {}",
            instruction.bci,
            report.text,
        );
    }
}

fn matching_switch_brace_end(
    text: &str,
    switch_segment: &jarde_java::source_map::Segment,
) -> usize {
    let snippet = switch_segment.text(text);
    assert!(
        snippet.trim_start().starts_with("switch ("),
        "switch source span: {snippet:?}"
    );
    let open_in_snippet = snippet.find('{').expect("switch has an opening brace");
    let mut depth = 0usize;
    for (offset, byte) in snippet.as_bytes()[open_in_snippet..]
        .iter()
        .copied()
        .enumerate()
    {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return switch_segment.start() + open_in_snippet + offset + 1;
                }
            }
            _ => {}
        }
    }
    panic!("switch source span has no matching closing brace: {snippet:?}");
}

#[test]
fn cf12_conditional_switch_keeps_exact_case_and_exit_sources() {
    assert_eq!(CLASS.len(), 1563, "frozen complete class size changed");
    assert_eq!(blake3::hash(CLASS).to_hex().to_string(), CLASS_BLAKE3);

    let default = recover_target(
        RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
    );
    let all = recover_target(RecoveryEvidenceRequest::all());
    assert!(
        default.report.produced(),
        "default: {:?}",
        default.report.outcome
    );
    assert!(all.report.produced(), "all: {:?}", all.report.outcome);
    assert_eq!(default.method, all.method);
    assert_eq!(default.ordinal, all.ordinal);
    assert_eq!(default.report.text, all.report.text);
    assert_eq!(default.report.source_map, all.report.source_map);
    assert!(
        !all.report.text.contains("@bytecode"),
        "{}",
        all.report.text
    );
    assert_eq!(
        all.report.text.matches("case 2:").count(),
        1,
        "{}",
        all.report.text
    );
    assert!(
        all.report
            .regions
            .iter()
            .all(|region| region.code.is_none()),
        "no fallback region may be presented as success: {:?}",
        all.report.regions
    );
    assert!(
        all.report
            .regions
            .iter()
            .all(|region| { !region.rule.is_some_and(|rule| rule.rule() == "loop") }),
        "the observed canonical graph has no loop: {:?}",
        all.report.regions
    );

    let switch_regions = all
        .report
        .regions
        .iter()
        .filter(|region| region.rule.is_some_and(|rule| rule.rule() == "switch"))
        .collect::<Vec<_>>();
    assert_eq!(
        switch_regions.len(),
        1,
        "one switch is structured: {:?}",
        all.report.regions
    );
    let switch = switch_regions[0];
    assert_eq!(
        switch.blocks.iter().filter(|&&bci| bci == 117).count(),
        1,
        "case 2 is claimed once: {switch:?}"
    );
    assert!(
        !switch.blocks.contains(&171),
        "join 171 remains after the switch: {switch:?}"
    );
    for bci in [0, 32, 59, 63, 67, 92, 117, 121, 146, 149, 171] {
        let owners = all
            .report
            .regions
            .iter()
            .flat_map(|region| region.blocks.iter())
            .filter(|&&owned| owned == bci)
            .count();
        assert_eq!(
            owners, 1,
            "physical block {bci} must have one region owner: {:?}",
            all.report.regions
        );
    }

    // The full switch node's physical source span is the boundary for the common continuation.
    // The real diagnostic places the decoded switch at BCI 7 and its shared join at BCI 171;
    // physical return@195 follows that join. This checks spans, not a guessed Java body snapshot.
    let switch_nodes = all
        .report
        .source_map
        .direct_of_bci(7)
        .into_iter()
        .filter(|segment| {
            segment
                .text(&all.report.text)
                .trim_start()
                .starts_with("switch (")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        switch_nodes.len(),
        1,
        "switch@7 has one complete statement span"
    );
    let switch_end = matching_switch_brace_end(&all.report.text, switch_nodes[0]);
    for bci in [171, 195] {
        let segments = all.report.source_map.of_bci(bci);
        assert!(!segments.is_empty(), "continuation BCI {bci} has source");
        assert!(
            segments.iter().all(|segment| segment.start() >= switch_end),
            "BCI {bci} belongs after the complete switch span, not inside a case: {:?}\n{}",
            segments
                .iter()
                .map(|segment| (
                    segment.start(),
                    segment.end(),
                    segment.text(&all.report.text)
                ))
                .collect::<Vec<_>>(),
            all.report.text,
        );
    }

    // The diagnostic established the only relevant physical targets. Check both are actual JVM
    // goto instructions before requiring their own complete break-statement source spans.
    let physical = physical_instructions();
    for bci in [89, 114] {
        let instruction = physical
            .iter()
            .find(|instruction| instruction.bci == bci)
            .expect("diagnosed transfer is a physical instruction");
        assert!(
            matches!(instruction.opcode, 0xa7 | 0xc8),
            "BCI {bci} must remain goto: {instruction:?}"
        );
        let mapped = all.report.source_map.direct_of_bci(bci);
        assert_eq!(
            mapped.len(),
            1,
            "goto@{bci} owns one direct source span: {mapped:?}"
        );
        let span = mapped[0];
        assert_eq!(span.origin().primary().bci(), bci);
        assert_eq!(span.origin().primary().method(), Some(&all.method));
        assert_eq!(
            span.text(&all.report.text).trim(),
            "break;",
            "goto@{bci} must map its complete break statement"
        );
    }

    assert_physical_coverage(&all.report, &all.method);
}

#[test]
fn cf12_recovery_with_pre_cancelled_public_budget_publishes_no_partial_artifact() {
    use jarde_java::StopReason;
    use jarde_reader::budget::CancellationToken;

    let token = CancellationToken::new();
    token.cancel();
    let recovery_budget = Budget::with_cancellation_token(limits(), token);
    // This is an API-level atomic-cancellation assertion. It does not claim to cancel at a
    // particular conditional-proof checkpoint; exact proof cost requires a separate real trace.
    let stopped =
        recover_target_with_recovery_budget(RecoveryEvidenceRequest::all(), Some(recovery_budget))
            .report;
    assert!(stopped.stop().is_some_and(StopReason::is_cancelled));
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
}

#[test]
fn root_switch_budget_observation() {
    let Ok(raw_limit) = std::env::var("ROOT_SWITCH_ANALYSIS_STEPS") else {
        return;
    };
    let analysis_steps = raw_limit
        .parse::<u64>()
        .expect("ROOT_SWITCH_ANALYSIS_STEPS is a u64");
    let recovery_budget = Budget::new(Limits {
        analysis_steps,
        ..limits()
    });
    let run = recover_target_with_recovery_budget(
        RecoveryEvidenceRequest::all(),
        Some(recovery_budget),
    );
    eprintln!(
        "ROOT_SWITCH_PROOF public limit={} usage={:?} stop={:?} produced={} outcome={:?}\ntext:\n{}\nsource_map={:#?}",
        analysis_steps,
        run.recovery_usage,
        run.report.stop(),
        run.report.produced(),
        run.report.outcome,
        run.report.text,
        run.report.source_map
    );
}
