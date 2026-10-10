//! The frozen CF-08 Java 8 loop gateways preserve both exit paths.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceKind, RecoveryEvidenceRequest, RecoveryFacts,
    RecoveryRequest, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, CountedBudgetDimension, Limits};
use jarde_reader::classfile::{
    DescriptorKind, MethodSelector, descriptor_facts, inspect_method_bytecode,
};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const FIXTURE: &[u8] = include_bytes!("../../../tests/fixtures/p3-loop-gateway/EndlessInts.class");
const NEGATIVES: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-loop-gateway/cf08/LoopGatewayNegatives.class");
// From the same Java 8 class: redirect sharedGateway's BCI 12 to its BCI 20 goto,
// add a BCI 20 StackMapTable frame, and update the enclosing attribute lengths.
// This gives the gateway a second normal predecessor and passes `java -Xverify:all`.
const EXTRA_ENTRY: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-loop-gateway/LoopGatewayExtraEntry.class");
const NO_PREFIX: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-10/one-arm-loop-controls/baseline-root-v1/cases/javac23-original/classes/PlainOneArmLoops.class",
);
const NO_PREFIX_BCIS: [u32; 11] = [0, 1, 2, 3, 6, 7, 8, 11, 14, 17, 18];
const ITERATOR_PREFIX: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-10/em23-variable-postfix-loop/baseline-root-v1/cases/javac23-original/classes/VariablePostfixLoop.class",
);
const IF_JOIN_CF07: &[u8] = include_bytes!(
    "../../../openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1/cases/javac23-original/classes/cf07/LoopCases.class",
);

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

fn recover_method(
    name: &str,
    descriptor: &str,
    evidence: RecoveryEvidenceRequest,
) -> jarde_java::RecoveryReport {
    recover_method_with_budget(name, descriptor, evidence, None)
}

fn recover_method_with_budget(
    name: &str,
    descriptor: &str,
    evidence: RecoveryEvidenceRequest,
    recovery_budget: Option<Budget>,
) -> jarde_java::RecoveryReport {
    recover_class_method_with_budget(
        FIXTURE,
        "cf08/EndlessInts",
        name,
        descriptor,
        evidence,
        recovery_budget,
    )
}

fn recover_class_method_with_budget(
    class: &[u8],
    owner: &str,
    name: &str,
    descriptor: &str,
    evidence: RecoveryEvidenceRequest,
    recovery_budget: Option<Budget>,
) -> jarde_java::RecoveryReport {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("fixture opens");
    let method = PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(class).to_hex().to_string()),
                length: u64::try_from(class.len()).expect("fixture length fits"),
            },
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(name.as_bytes().to_vec()),
        descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
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
    .expect("fixture method analysis completes");
    let parameter_slots = descriptor_facts(descriptor.as_bytes(), DescriptorKind::Method)
        .expect("fixture method descriptor is valid")
        .parameter_slots()
        .expect("method descriptor states its parameter slots");
    let facts = RecoveryFacts::new(
        MethodFacts::new(name, descriptor, parameter_slots)
            .with_access_flags(0x0008)
            .with_declaring_class(DeclaringClass::new(owner, 0x0031)),
    );
    let mut budget = recovery_budget.unwrap_or(budget);
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8).with_evidence(evidence),
        &mut budget,
    )
}

fn braced_body(source: &str, marker: &str) -> std::ops::Range<usize> {
    let marker_at = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing {marker}: {source}"));
    let open = source[marker_at..]
        .find('{')
        .map(|offset| marker_at + offset)
        .unwrap_or_else(|| panic!("missing opening brace after {marker}: {source}"));
    let mut depth = 0usize;
    for (offset, byte) in source.as_bytes().iter().enumerate().skip(open) {
        match *byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return open + 1..offset;
                }
            }
            _ => {}
        }
    }
    panic!("unclosed body after {marker}: {source}");
}

fn loop_form_header_and_body(source: &str) -> (&'static str, &str, std::ops::Range<usize>) {
    let (form, start, marker) = if let Some(start) = source.find("for (") {
        ("for", start, "for (")
    } else if let Some(start) = source.find("while (") {
        ("while", start, "while (")
    } else {
        panic!("selected arm has no for/while loop: {source}");
    };
    let body = braced_body(source, marker);
    let open = body.start - 1;
    (form, &source[start..open], body)
}

fn assert_one_structured_owner_in(
    report: &jarde_java::RecoveryReport,
    range: std::ops::Range<usize>,
    expected_blocks: &[u32],
    entry_text: &str,
    entry_bcis: &[u32],
) {
    assert!(!expected_blocks.is_empty());
    let owners: Vec<_> = report
        .regions
        .iter()
        .filter(|region| {
            expected_blocks
                .iter()
                .all(|bci| region.blocks.contains(bci))
        })
        .collect();
    assert_eq!(
        owners.len(),
        1,
        "expected block starts {expected_blocks:?} lack one common owner: {:?}",
        report.regions
    );
    assert!(
        owners[0].structured,
        "selected arm is not structured: {:?}",
        owners[0]
    );
    for bci in expected_blocks {
        let matching: Vec<_> = report
            .regions
            .iter()
            .filter(|region| region.blocks.contains(bci))
            .collect();
        assert_eq!(
            matching.len(),
            1,
            "block {bci} owners: {:?}",
            report.regions
        );
        assert!(matching[0].structured, "block {bci}: {:?}", matching[0]);
    }
    let entry_anchor = report.source_map.segments().iter().any(|segment| {
        segment.start() >= range.start
            && segment.end() <= range.end
            && segment.text(&report.text).contains(entry_text)
            && std::iter::once(segment.origin().primary())
                .chain(segment.origin().derived())
                .any(|origin| entry_bcis.contains(&origin.bci()))
    });
    assert!(
        entry_anchor,
        "prefix source anchor is outside the selected if arm: {}",
        report.text
    );
}

#[test]
fn cf07_nonempty_if_join_goto_is_derived_from_complete_if() {
    const METHOD_BCIS: [u32; 18] = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 11, 12, 14, 17, 20, 23, 24, 25, 26,
    ];
    const METHOD_TAIL_BCIS: [u32; 5] = [27, 30, 33, 36, 37];
    let evidence = RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap);
    let default = recover_class_method_with_budget(
        IF_JOIN_CF07,
        "cf07/LoopCases",
        "counted",
        "(II)I",
        evidence,
        None,
    );
    let all = recover_class_method_with_budget(
        IF_JOIN_CF07,
        "cf07/LoopCases",
        "counted",
        "(II)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(default.produced(), "default: {:?}", default.outcome);
    assert!(all.produced(), "all: {:?}", all.outcome);
    assert_eq!(default.text, all.text);
    assert_eq!(default.source_map, all.source_map);
    assert!(!all.text.contains("@bytecode"), "{}", all.text);

    for bci in METHOD_BCIS.into_iter().chain(METHOD_TAIL_BCIS) {
        assert!(
            !all.source_map.of_bci(bci).is_empty(),
            "BCI {bci} lost its source: {}",
            all.text
        );
    }
    let if_positions: Vec<_> = all.text.match_indices("if (").map(|(at, _)| at).collect();
    assert_eq!(if_positions.len(), 1, "expected one inner if: {}", all.text);
    let if_keyword = if_positions[0];
    let if_start = all.text[..if_keyword].rfind('\n').map_or(0, |at| at + 1);
    assert!(
        all.text[if_start..if_keyword]
            .bytes()
            .all(|byte| byte == b' ')
    );
    let else_body = braced_body(&all.text, "else {");
    // `braced_body` ends immediately before the else closing brace. Include that brace and
    // the emitter's following newline in the complete If statement span.
    let if_end = else_body.end + 2;
    let outer_while_body = braced_body(&all.text, "while (");
    assert!(
        if_start >= outer_while_body.start && if_end <= outer_while_body.end,
        "complete If statement is outside the outer while body: {}",
        all.text
    );
    let if_spans = all
        .source_map
        .derived_of_bci(20)
        .into_iter()
        .filter(|segment| segment.start() == if_start && segment.end() == if_end)
        .collect::<Vec<_>>();
    assert_eq!(
        if_spans.len(),
        1,
        "goto@20 does not point to the exact complete If span {if_start}..{if_end}: {}",
        all.text
    );
    assert!(
        all.source_map
            .of_bci(14)
            .iter()
            .any(|segment| { segment.text(&all.text).trim_start().starts_with("if (") }),
        "condition@14 lost the inner-if source: {}",
        all.text
    );
    assert!(
        all.source_map
            .derived_of_bci(30)
            .iter()
            .any(|segment| { segment.text(&all.text).trim_start().starts_with("while (") }),
        "outer while latch@30 lost its source: {}",
        all.text
    );

    let mut owner_budget = Budget::new(limits());
    let owner_snapshot = ArtifactSnapshot::open(
        ArtifactInput::bytes(IF_JOIN_CF07.to_vec()),
        &mut owner_budget,
    )
    .expect("frozen class snapshot opens");
    let expected_owner = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: owner_snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(IF_JOIN_CF07).to_hex().to_string()),
            length: IF_JOIN_CF07.len() as u64,
        },
        variant: PhysicalVariant::Base,
    };
    for segment in all.source_map.segments() {
        for origin in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            if origin.bci() == 20 {
                let method = origin.method().expect("goto source has physical method");
                assert_eq!(method.name.0.as_slice(), b"counted");
                assert_eq!(method.descriptor.0.as_slice(), b"(II)I");
                assert_eq!(&method.owner, &expected_owner);
            }
        }
    }
}

#[test]
fn cf07_if_join_origin_rejects_nonjoin_and_nontransfer_terminals() {
    let inspection = inspect_method_bytecode(
        IF_JOIN_CF07,
        MethodSelector {
            name: JvmBytes(b"counted".to_vec()),
            descriptor: JvmBytes(b"(II)I".to_vec()),
        },
        &mut Budget::new(limits()),
    )
    .expect("frozen CF07 method bytecode reads");
    let goto = inspection
        .instructions
        .iter()
        .find(|instruction| instruction.bci == 20)
        .expect("frozen counted method has BCI 20");
    assert_eq!(goto.opcode, 0xa7, "BCI 20 is the frozen goto");
    assert_eq!(goto.width, 3, "frozen goto width");
    let goto_offset = usize::try_from(goto.span.start).expect("class offset fits");

    // Analysis-only mutation: point the physical goto@20 to the outer while exit@33.
    let mut loop_exit = IF_JOIN_CF07.to_vec();
    assert_eq!(
        &loop_exit[goto_offset..goto_offset + 3],
        &[0xa7, 0x00, 0x07]
    );
    loop_exit[goto_offset + 1..goto_offset + 3].copy_from_slice(&13_i16.to_be_bytes());
    assert_eq!(
        20 + i16::from_be_bytes([loop_exit[goto_offset + 1], loop_exit[goto_offset + 2],]) as i32,
        33,
        "mutated goto targets the outer loop exit",
    );
    let loop_exit_report = recover_class_method_with_budget(
        &loop_exit,
        "cf07/LoopCases",
        "counted",
        "(II)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(
        loop_exit_report.produced(),
        "{:?}",
        loop_exit_report.outcome
    );
    assert!(
        !loop_exit_report
            .source_map
            .derived_of_bci(20)
            .iter()
            .any(|segment| segment
                .text(&loop_exit_report.text)
                .trim_start()
                .starts_with("if (")),
        "goto@20 to the outer loop exit gained an If origin: {}",
        loop_exit_report.text
    );
    assert!(
        !loop_exit_report.source_map.of_bci(20).is_empty(),
        "loop-exit BCI 20 lost its existing source: {}",
        loop_exit_report.text
    );

    // Analysis-only mutation: replace goto@20 with iinc slot 2 by 2. The instruction remains
    // three bytes wide and falls through to the other arm@23; it is not a transfer.
    let mut nonterminal = IF_JOIN_CF07.to_vec();
    assert_eq!(
        &nonterminal[goto_offset..goto_offset + 3],
        &[0xa7, 0x00, 0x07]
    );
    nonterminal[goto_offset..goto_offset + 3].copy_from_slice(&[0x84, 0x02, 0x02]);
    let nonterminal_report = recover_class_method_with_budget(
        &nonterminal,
        "cf07/LoopCases",
        "counted",
        "(II)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(
        nonterminal_report.produced(),
        "{:?}",
        nonterminal_report.outcome
    );
    assert!(
        nonterminal_report
            .source_map
            .of_bci(20)
            .iter()
            .any(|segment| segment.origin().primary().bci() == 20),
        "iinc@20 lost its direct assignment source: {}",
        nonterminal_report.text
    );
    assert!(
        !nonterminal_report
            .source_map
            .derived_of_bci(20)
            .iter()
            .any(|segment| segment
                .text(&nonterminal_report.text)
                .trim_start()
                .starts_with("if (")),
        "nonterminal iinc@20 gained an If origin: {}",
        nonterminal_report.text
    );
}

#[test]
fn cf07_if_join_proof_budget_and_cancellation_publish_no_partial_source() {
    use jarde_java::StopReason;
    use jarde_reader::model::ExecutionReport;

    let full = recover_class_method_with_budget(
        IF_JOIN_CF07,
        "cf07/LoopCases",
        "counted",
        "(II)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(limits())),
    );
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("full candidate did not complete: {:?}", full.outcome);
    };
    let mut late = limits();
    late.ir_edges = usage.ir_edges.saturating_sub(1);
    let stopped = recover_class_method_with_budget(
        IF_JOIN_CF07,
        "cf07/LoopCases",
        "counted",
        "(II)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(late)),
    );
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    assert!(matches!(
        stopped.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::IrEdges,
            at: Some(20),
            ..
        })
    ));

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_class_method_with_budget(
        IF_JOIN_CF07,
        "cf07/LoopCases",
        "counted",
        "(II)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}

#[test]
fn cf07_return_arm_loop_latch_is_derived_from_complete_while() {
    const LAST_INDEX_BCIS: [u32; 18] = [
        0, 1, 2, 3, 5, 7, 8, 11, 12, 14, 15, 16, 19, 21, 22, 25, 28, 29,
    ];
    let default = recover_class_method_with_budget(
        IF_JOIN_CF07,
        "cf07/LoopCases",
        "lastIndexOf",
        "([IIII)I",
        RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
        None,
    );
    let all = recover_class_method_with_budget(
        IF_JOIN_CF07,
        "cf07/LoopCases",
        "lastIndexOf",
        "([IIII)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(default.produced(), "default: {:?}", default.outcome);
    assert!(all.produced(), "all: {:?}", all.outcome);
    assert_eq!(default.text, all.text);
    assert_eq!(default.source_map, all.source_map);
    assert!(!all.text.contains("@bytecode"), "{}", all.text);
    for bci in LAST_INDEX_BCIS {
        assert!(
            !all.source_map.of_bci(bci).is_empty(),
            "lastIndexOf BCI {bci} lost its source: {}",
            all.text
        );
    }

    let while_keyword = all
        .text
        .find("while (")
        .expect("lastIndexOf recovered while");
    let while_start = all.text[..while_keyword]
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    let while_body = braced_body(&all.text, "while (");
    // The source span includes the loop's closing brace and the emitter's following newline.
    let while_end = while_body.end + 2;
    let while_span = while_start..while_end;
    assert_one_structured_owner_in(&all, while_span.clone(), &[5, 11, 19, 22], "while (", &[25]);
    let latch_spans: Vec<_> = all
        .source_map
        .derived_of_bci(25)
        .into_iter()
        .filter(|segment| segment.start() == while_span.start && segment.end() == while_span.end)
        .collect();
    assert_eq!(
        latch_spans.len(),
        1,
        "goto@25 must point to exactly the complete while span {while_span:?}: {}",
        all.text
    );
    assert!(
        all.source_map.of_bci(8).iter().any(|segment| {
            segment.start() >= while_span.start && segment.end() <= while_span.end
        }),
        "while condition@8 lost its source: {}",
        all.text
    );
    assert!(
        all.source_map
            .of_bci(16)
            .iter()
            .any(|segment| { segment.text(&all.text).trim_start().starts_with("if (") }),
        "inner condition@16 lost its source: {}",
        all.text
    );

    let mut owner_budget = Budget::new(limits());
    let owner_snapshot = ArtifactSnapshot::open(
        ArtifactInput::bytes(IF_JOIN_CF07.to_vec()),
        &mut owner_budget,
    )
    .expect("frozen class snapshot opens");
    let expected_owner = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: owner_snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(IF_JOIN_CF07).to_hex().to_string()),
            length: IF_JOIN_CF07.len() as u64,
        },
        variant: PhysicalVariant::Base,
    };
    for segment in all.source_map.segments() {
        for origin in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            if origin.bci() == 25 {
                let method = origin.method().expect("latch source has physical method");
                assert_eq!(method.name.0.as_slice(), b"lastIndexOf");
                assert_eq!(method.descriptor.0.as_slice(), b"([IIII)I");
                assert_eq!(&method.owner, &expected_owner);
            }
        }
    }
}

#[test]
fn cf07_return_arm_latch_origin_rejects_wrong_target_nonterminal_and_nonreturn() {
    let inspection = inspect_method_bytecode(
        IF_JOIN_CF07,
        MethodSelector {
            name: JvmBytes(b"lastIndexOf".to_vec()),
            descriptor: JvmBytes(b"([IIII)I".to_vec()),
        },
        &mut Budget::new(limits()),
    )
    .expect("frozen CF07 method bytecode reads");
    let instruction = |bci| {
        inspection
            .instructions
            .iter()
            .find(|instruction| instruction.bci == bci)
            .unwrap_or_else(|| panic!("frozen lastIndexOf is missing BCI {bci}"))
    };
    let goto = instruction(25);
    assert_eq!(goto.opcode, 0xa7);
    assert_eq!(goto.width, 3);
    let goto_offset = usize::try_from(goto.span.start).expect("class offset fits");
    assert_eq!(
        &IF_JOIN_CF07[goto_offset..goto_offset + 3],
        &[0xa7, 0xff, 0xec]
    );

    let has_no_loop_origin = |report: &jarde_java::RecoveryReport| {
        !report.source_map.derived_of_bci(25).iter().any(|segment| {
            segment
                .text(&report.text)
                .trim_start()
                .starts_with("while (")
        })
    };

    // Analysis-only mutation: redirect the real goto@25 from header@5 to loop exit@28.
    let mut wrong_target = IF_JOIN_CF07.to_vec();
    wrong_target[goto_offset + 1..goto_offset + 3].copy_from_slice(&3_i16.to_be_bytes());
    assert_eq!(
        25 + i16::from_be_bytes([wrong_target[goto_offset + 1], wrong_target[goto_offset + 2]])
            as i32,
        28,
        "mutated goto targets the actual outer-loop exit",
    );
    let wrong_target_report = recover_class_method_with_budget(
        &wrong_target,
        "cf07/LoopCases",
        "lastIndexOf",
        "([IIII)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(
        wrong_target_report.produced(),
        "{:?}",
        wrong_target_report.outcome
    );
    assert!(
        has_no_loop_origin(&wrong_target_report),
        "{}",
        wrong_target_report.text
    );
    // Redirecting the only backedge removes the natural loop. The preexisting no-loop
    // If presentation has no source for this transfer; that independent gap is not a
    // latch origin and this slice must not assign it to an invented while.
    assert!(!wrong_target_report.text.contains("while ("));
    assert!(wrong_target_report.source_map.of_bci(25).is_empty());

    // Analysis-only mutation: keep the real three-byte span but replace goto with iinc local4,-1.
    let mut nonterminal = IF_JOIN_CF07.to_vec();
    nonterminal[goto_offset..goto_offset + 3].copy_from_slice(&[0x84, 0x04, 0xff]);
    let nonterminal_report = recover_class_method_with_budget(
        &nonterminal,
        "cf07/LoopCases",
        "lastIndexOf",
        "([IIII)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(
        nonterminal_report.produced(),
        "{:?}",
        nonterminal_report.outcome
    );
    assert!(
        has_no_loop_origin(&nonterminal_report),
        "{}",
        nonterminal_report.text
    );
    assert!(
        nonterminal_report
            .source_map
            .of_bci(25)
            .iter()
            .any(|segment| { segment.origin().primary().bci() == 25 }),
        "nonterminal iinc@25 lost its direct source: {}",
        nonterminal_report.text
    );

    // Analysis-only mutation: make the would-be return arm throw a valid null reference instead.
    // BCI 19..20 remains two bytes (aconst_null; nop), and BCI 21 becomes athrow; no class is run.
    let load = instruction(19);
    let returned = instruction(21);
    assert_eq!(load.opcode, 0x15);
    assert_eq!(load.width, 2);
    assert_eq!(returned.opcode, 0xac);
    assert_eq!(returned.width, 1);
    let mut nonreturn = IF_JOIN_CF07.to_vec();
    let load_offset = usize::try_from(load.span.start).expect("class offset fits");
    let return_offset = usize::try_from(returned.span.start).expect("class offset fits");
    nonreturn[load_offset..load_offset + 2].copy_from_slice(&[0x01, 0x00]);
    nonreturn[return_offset] = 0xbf;
    let nonreturn_report = recover_class_method_with_budget(
        &nonreturn,
        "cf07/LoopCases",
        "lastIndexOf",
        "([IIII)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(
        nonreturn_report.produced(),
        "{:?}",
        nonreturn_report.outcome
    );
    assert!(
        has_no_loop_origin(&nonreturn_report),
        "{}",
        nonreturn_report.text
    );
}

#[test]
fn cf07_return_arm_latch_budget_and_cancellation_publish_no_partial_source() {
    use jarde_java::StopReason;
    use jarde_reader::model::ExecutionReport;

    let full = recover_class_method_with_budget(
        IF_JOIN_CF07,
        "cf07/LoopCases",
        "lastIndexOf",
        "([IIII)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(limits())),
    );
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("full candidate did not complete: {:?}", full.outcome);
    };
    let mut late = limits();
    late.analysis_steps = usage.analysis_steps.saturating_sub(1);
    let stopped = recover_class_method_with_budget(
        IF_JOIN_CF07,
        "cf07/LoopCases",
        "lastIndexOf",
        "([IIII)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(late)),
    );
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    assert!(matches!(
        stopped.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::AnalysisSteps,
            ..
        })
    ));

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_class_method_with_budget(
        IF_JOIN_CF07,
        "cf07/LoopCases",
        "lastIndexOf",
        "([IIII)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}

#[test]
fn cf08_two_gateways_have_one_loop_owner_and_complete_sources() {
    let report = recover_method("find", "(I)I", RecoveryEvidenceRequest::all());
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(report.text.contains("break;"), "{}", report.text);
    assert_eq!(report.text.matches("break;").count(), 1, "{}", report.text);
    assert_eq!(
        report.text.matches("local1 = local1 + 1;").count(),
        1,
        "{}",
        report.text
    );
    for bci in [4, 7, 12, 15, 18, 21, 24, 25] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} lost source: {}",
            report.text
        );
    }
    for bci in [2, 10, 15, 18] {
        let owners: Vec<_> = report
            .regions
            .iter()
            .filter(|r| r.blocks.contains(&bci))
            .collect();
        assert_eq!(owners.len(), 1, "BCI {bci} owners: {:?}", report.regions);
        assert!(owners[0].structured, "BCI {bci} is not structured");
    }
    let loop_owner = report
        .regions
        .iter()
        .find(|r| r.blocks.contains(&15))
        .unwrap();
    assert!(!loop_owner.blocks.contains(&7));
    assert!(!loop_owner.blocks.contains(&24));
}

#[test]
fn cf08_unproved_gateways_keep_physical_quotes() {
    for (class, name, gateway) in [
        (EXTRA_ENTRY, "sharedGateway", 20),
        (NEGATIVES, "differentTarget", 15),
        (NEGATIVES, "effectGateway", 15),
        (NEGATIVES, "exceptionalGateway", 23),
    ] {
        let report = recover_class_method_with_budget(
            class,
            "cf08/LoopGatewayNegatives",
            name,
            "(I)I",
            RecoveryEvidenceRequest::all(),
            None,
        );
        assert!(report.produced(), "{name}: {:?}", report.outcome);
        assert!(report.text.contains("@bytecode"), "{name}: {}", report.text);
        assert!(
            !report.source_map.of_bci(gateway).is_empty(),
            "{name} lost BCI {gateway}"
        );
        assert!(
            !report.regions.iter().any(|region| region.structured
                && region.blocks.contains(&gateway)
                && region.rule.is_some_and(|rule| rule.rule() == "loop")),
            "{name} falsely claimed BCI {gateway}: {:?}",
            report.regions
        );
    }
}

#[test]
fn cf08_gateway_budget_and_cancellation_publish_no_partial_source() {
    use jarde_java::StopReason;
    use jarde_reader::model::ExecutionReport;
    let full = recover_method_with_budget(
        "find",
        "(I)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(limits())),
    );
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("full candidate did not complete: {:?}", full.outcome);
    };
    let mut late = limits();
    late.analysis_steps = usage.analysis_steps - 1;
    let stopped = recover_method_with_budget(
        "find",
        "(I)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(late)),
    );
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    assert!(matches!(
        stopped.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::AnalysisSteps,
            ..
        })
    ));
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_method_with_budget(
        "find",
        "(I)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}

#[test]
fn no_prefix_while_latch_keeps_physical_source_and_one_owner() {
    let default = recover_class_method_with_budget(
        NO_PREFIX,
        "PlainOneArmLoops",
        "noPrefix",
        "(ZI)I",
        RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
        None,
    );
    let all = recover_class_method_with_budget(
        NO_PREFIX,
        "PlainOneArmLoops",
        "noPrefix",
        "(ZI)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(default.produced(), "default: {:?}", default.outcome);
    assert!(all.produced(), "all: {:?}", all.outcome);
    assert_eq!(default.text, all.text);
    assert_eq!(default.source_map, all.source_map);
    assert!(!all.text.contains("@bytecode"), "{}", all.text);

    for bci in NO_PREFIX_BCIS {
        assert!(
            !all.source_map.of_bci(bci).is_empty(),
            "BCI {bci} lost source: {}",
            all.text
        );
    }
    let latch_spans = all.source_map.derived_of_bci(14);
    assert!(
        latch_spans
            .iter()
            .any(|segment| segment.text(&all.text).trim_start().starts_with("while (")),
        "goto@14 is not derived from the loop statement: {}",
        all.text
    );
    let mut owner_budget = Budget::new(limits());
    let owner_snapshot =
        ArtifactSnapshot::open(ArtifactInput::bytes(NO_PREFIX.to_vec()), &mut owner_budget)
            .expect("frozen class snapshot opens");
    let expected_owner = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: owner_snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(NO_PREFIX).to_hex().to_string()),
            length: NO_PREFIX.len() as u64,
        },
        variant: PhysicalVariant::Base,
    };
    for segment in all.source_map.segments() {
        for origin in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            assert!(
                NO_PREFIX_BCIS.contains(&origin.bci()),
                "unexpected source-map BCI {}",
                origin.bci()
            );
            let method = origin
                .method()
                .expect("source-map origin has a physical method");
            assert_eq!(method.name.0.as_slice(), b"noPrefix");
            assert_eq!(method.descriptor.0.as_slice(), b"(ZI)I");
            assert_eq!(&method.owner, &expected_owner);
        }
    }
    for block_bci in [0, 6, 11, 17] {
        let owners: Vec<_> = all
            .regions
            .iter()
            .filter(|region| region.blocks.contains(&block_bci))
            .collect();
        assert_eq!(owners.len(), 1, "BCI {block_bci}: {:?}", all.regions);
        assert!(owners[0].structured, "BCI {block_bci}: {:?}", all.regions);
    }
}

#[test]
fn one_arm_straight_prefix_loop_and_optional_tail_are_closed() {
    // Block starts are frozen from javac23-original/javap.txt: prefix 6, loop
    // header 8, loop body 13, and loopAndTail's exclusive tail 23.
    for (name, guard, tail, blocks, method_bcis) in [
        (
            "prefixWhile",
            "if (arg0) {",
            None,
            &[6, 8, 13][..],
            &[0, 1, 2, 3, 6, 7, 8, 9, 10, 13, 14, 15, 16, 17, 20, 23, 24][..],
        ),
        (
            "takenArm",
            "if (!arg0) {",
            None,
            &[6, 8, 13][..],
            &[0, 1, 2, 3, 6, 7, 8, 9, 10, 13, 14, 15, 16, 17, 20, 23, 24][..],
        ),
        (
            "loopAndTail",
            "if (arg0) {",
            Some("local2 = local2 + 100;"),
            &[6, 8, 13, 23][..],
            &[
                0, 1, 2, 3, 6, 7, 8, 9, 10, 13, 14, 15, 16, 17, 20, 23, 26, 27,
            ][..],
        ),
    ] {
        let default = recover_class_method_with_budget(
            NO_PREFIX,
            "PlainOneArmLoops",
            name,
            "(ZI)I",
            RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
            None,
        );
        let all = recover_class_method_with_budget(
            NO_PREFIX,
            "PlainOneArmLoops",
            name,
            "(ZI)I",
            RecoveryEvidenceRequest::all(),
            None,
        );
        assert!(default.produced(), "{name}: {:?}", default.outcome);
        assert!(all.produced(), "{name}: {:?}", all.outcome);
        assert_eq!(default.text, all.text, "{name}");
        assert_eq!(default.source_map, all.source_map, "{name}");
        assert!(!all.text.contains("@bytecode"), "{name}: {}", all.text);

        let arm = braced_body(&all.text, guard);
        let arm_text = &all.text[arm.clone()];
        let prefix_at = arm_text
            .find("local3 = 0")
            .unwrap_or_else(|| panic!("{name} prefix outside selected arm: {}", all.text));
        let (form, header, loop_body) = loop_form_header_and_body(arm_text);
        if form == "for" {
            assert!(
                header.contains("local3 = 0;"),
                "{name} for-init is wrong: {header}"
            );
            assert!(
                header.contains("local3 < arg1;"),
                "{name} for-condition is wrong: {header}"
            );
            assert!(
                header.contains("local3 = local3 + 1"),
                "{name} for-update is wrong: {header}"
            );
            let for_at = arm_text.find("for (").expect("for marker was just found");
            assert!(
                prefix_at >= for_at && prefix_at < for_at + header.len(),
                "{name} init is not in the selected arm's for header: {}",
                all.text
            );
        } else {
            let loop_at = arm_text
                .find("while (")
                .expect("while marker was just found");
            assert!(
                prefix_at < loop_at,
                "{name} prefix is not before loop: {}",
                all.text
            );
            assert!(
                header.contains("local3 < arg1"),
                "{name} while-condition is wrong: {header}"
            );
        }
        if let Some(tail) = tail {
            let tail_after_loop = arm_text
                .get(loop_body.end..)
                .expect("loop closing brace is within the selected arm");
            assert!(
                tail_after_loop.contains(tail),
                "{name} tail is not after the complete loop and inside the selected arm: {}",
                all.text
            );
        }
        assert_one_structured_owner_in(&all, arm.clone(), blocks, "local3 = 0", &[6, 7]);
        for bci in method_bcis {
            assert!(
                !all.source_map.of_bci(*bci).is_empty(),
                "{name} lost existing or latch source BCI {bci}: {}",
                all.text
            );
        }
        let loop_start = arm.start
            + arm_text
                .find(if form == "for" { "for (" } else { "while (" })
                .expect("the selected loop marker was found");
        // The statement node owns its closing brace and the following newline.
        let loop_end = arm.start + loop_body.end + 2;
        let latch_spans = all.source_map.derived_of_bci(20);
        assert!(
            latch_spans.iter().any(|segment| {
                segment.start() >= arm.start
                    && segment.start() <= loop_start
                    && segment.end() == loop_end
                    && segment.end() <= arm.end
                    && segment.text(&all.text).trim_start().starts_with("for (")
            }),
            "{name} goto@20 is not derived from the complete for statement in its arm: {}",
            all.text
        );
        for segment in &latch_spans {
            for origin in std::iter::once(segment.origin().primary())
                .chain(segment.origin().derived())
                .filter(|origin| origin.bci() == 20)
            {
                let method = origin
                    .method()
                    .expect("latch source has physical method identity");
                assert_eq!(method.name.0.as_slice(), name.as_bytes(), "{name}");
                assert_eq!(method.descriptor.0.as_slice(), b"(ZI)I", "{name}");
                assert_eq!(
                    method.owner.class_bytes.digest.0,
                    blake3::hash(NO_PREFIX).to_hex().to_string()
                );
                assert_eq!(method.owner.class_bytes.length, NO_PREFIX.len() as u64);
                assert_eq!(method.owner.variant, PhysicalVariant::Base);
            }
        }
    }

    let iterator_default = recover_class_method_with_budget(
        ITERATOR_PREFIX,
        "VariablePostfixLoop",
        "countEmpty",
        "(Ljava/util/List;)I",
        RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
        None,
    );
    let iterator = recover_class_method_with_budget(
        ITERATOR_PREFIX,
        "VariablePostfixLoop",
        "countEmpty",
        "(Ljava/util/List;)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(
        iterator_default.produced(),
        "iterator default: {:?}",
        iterator_default.outcome
    );
    assert!(iterator.produced(), "iterator: {:?}", iterator.outcome);
    assert_eq!(iterator_default.text, iterator.text);
    assert_eq!(iterator_default.source_map, iterator.source_map);
    assert!(!iterator.text.contains("@bytecode"), "{}", iterator.text);
    assert!(iterator.text.contains("while ("), "{}", iterator.text);
    assert!(iterator.text.contains("hasNext"), "{}", iterator.text);
    assert!(iterator.text.contains(".next"), "{}", iterator.text);
    let arm = braced_body(&iterator.text, "if (arg0 != null) {");
    // Frozen javac23 branch targets/fallthrough give arm blocks 6, 13, 22,
    // 39, 42; 45 is the shared exit and is deliberately outside the arm.
    assert_one_structured_owner_in(&iterator, arm, &[6, 13, 22, 39, 42], ".iterator()", &[6, 7]);
}

#[test]
fn rejected_for_update_slot_keeps_the_independent_while_latch() {
    let mut matched_updates = 0;
    for name in ["prefixWhile", "takenArm", "loopAndTail"] {
        let inspection = inspect_method_bytecode(
            NO_PREFIX,
            MethodSelector {
                name: JvmBytes(name.as_bytes().to_vec()),
                descriptor: JvmBytes(b"(ZI)I".to_vec()),
            },
            &mut Budget::new(limits()),
        )
        .expect("frozen method bytecode reads");
        let updates: Vec<_> = inspection
            .instructions
            .iter()
            .filter(|instruction| {
                instruction.bci == 17
                    && instruction.opcode == 0x84
                    && usize::try_from(instruction.span.start)
                        .ok()
                        .and_then(|start| NO_PREFIX.get(start..start + 3))
                        == Some(&[0x84, 0x03, 0x01][..])
            })
            .collect();
        assert_eq!(updates.len(), 1, "{name} update byte pattern");
        matched_updates += updates.len();
        let byte_offset = usize::try_from(updates[0].span.start).expect("class offset fits") + 1;
        let mut changed = NO_PREFIX.to_vec();
        assert_eq!(changed[byte_offset], 3, "{name} local slot operand");
        changed[byte_offset] = 2;

        let report = recover_class_method_with_budget(
            &changed,
            "PlainOneArmLoops",
            name,
            "(ZI)I",
            RecoveryEvidenceRequest::all(),
            None,
        );
        assert!(report.produced(), "{name}: {:?}", report.outcome);
        assert!(report.text.contains("while ("), "{name}: {}", report.text);
        assert!(!report.text.contains("for ("), "{name}: {}", report.text);
        assert!(
            report
                .source_map
                .derived_of_bci(20)
                .iter()
                .any(|segment| segment
                    .text(&report.text)
                    .trim_start()
                    .starts_with("while (")),
            "{name} lost the independently proved while latch: {}",
            report.text
        );
    }
    assert_eq!(
        matched_updates, 3,
        "exactly three frozen method bodies matched"
    );
}

#[test]
fn non_backedge_transfer_does_not_gain_a_loop_latch_origin() {
    for name in ["prefixWhile", "takenArm", "loopAndTail"] {
        let inspection = inspect_method_bytecode(
            NO_PREFIX,
            MethodSelector {
                name: JvmBytes(name.as_bytes().to_vec()),
                descriptor: JvmBytes(b"(ZI)I".to_vec()),
            },
            &mut Budget::new(limits()),
        )
        .expect("frozen method bytecode reads");
        let gotos: Vec<_> = inspection
            .instructions
            .iter()
            .filter(|instruction| instruction.bci == 20 && instruction.opcode == 0xa7)
            .collect();
        assert_eq!(gotos.len(), 1, "{name} goto@20");
        let offset = usize::try_from(gotos[0].span.start).expect("class offset fits");
        let mut changed = NO_PREFIX.to_vec();
        assert_eq!(
            &changed[offset..offset + 3],
            &[0xa7, 0xff, 0xf4],
            "{name} goto bytes"
        );
        changed[offset + 1..offset + 3].copy_from_slice(&3_i16.to_be_bytes());
        assert_eq!(
            20 + i16::from_be_bytes([changed[offset + 1], changed[offset + 2]]) as i32,
            23
        );

        // This byte mutation is an analysis counterexample only; it is not submitted to a JVM.
        let report = recover_class_method_with_budget(
            &changed,
            "PlainOneArmLoops",
            name,
            "(ZI)I",
            RecoveryEvidenceRequest::all(),
            None,
        );
        assert!(report.produced(), "{name}: {:?}", report.outcome);
        assert!(!report.text.contains("for ("), "{name}: {}", report.text);
        assert!(!report.text.contains("while ("), "{name}: {}", report.text);
        assert!(
            !report.source_map.derived_of_bci(20).iter().any(|segment| {
                let text = segment.text(&report.text).trim_start();
                text.starts_with("for (") || text.starts_with("while (")
            }),
            "{name} assigned goto@20 to a loop without a back edge: {}",
            report.text
        );
    }
}

#[test]
fn prefixed_one_arm_loop_budget_and_cancellation_publish_no_partial_source() {
    use jarde_java::StopReason;
    use jarde_reader::model::ExecutionReport;

    let full = recover_class_method_with_budget(
        NO_PREFIX,
        "PlainOneArmLoops",
        "prefixWhile",
        "(ZI)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(limits())),
    );
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("full candidate did not complete: {:?}", full.outcome);
    };
    let mut late = limits();
    late.analysis_steps = usage.analysis_steps.saturating_sub(1);
    let stopped = recover_class_method_with_budget(
        NO_PREFIX,
        "PlainOneArmLoops",
        "prefixWhile",
        "(ZI)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(late)),
    );
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    assert!(matches!(
        stopped.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::AnalysisSteps,
            ..
        })
    ));

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_class_method_with_budget(
        NO_PREFIX,
        "PlainOneArmLoops",
        "prefixWhile",
        "(ZI)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}

#[test]
fn no_prefix_latch_budget_and_cancellation_publish_no_partial_source() {
    use jarde_java::StopReason;
    use jarde_reader::model::ExecutionReport;

    let full = recover_class_method_with_budget(
        NO_PREFIX,
        "PlainOneArmLoops",
        "noPrefix",
        "(ZI)I",
        RecoveryEvidenceRequest::essential(),
        Some(Budget::new(limits())),
    );
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("full candidate did not complete: {:?}", full.outcome);
    };
    let mut late = limits();
    late.analysis_steps = usage.analysis_steps.saturating_sub(1);
    let stopped = recover_class_method_with_budget(
        NO_PREFIX,
        "PlainOneArmLoops",
        "noPrefix",
        "(ZI)I",
        RecoveryEvidenceRequest::essential(),
        Some(Budget::new(late)),
    );
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    assert!(matches!(
        stopped.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::AnalysisSteps,
            ..
        })
    ));

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_class_method_with_budget(
        NO_PREFIX,
        "PlainOneArmLoops",
        "noPrefix",
        "(ZI)I",
        RecoveryEvidenceRequest::essential(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}
