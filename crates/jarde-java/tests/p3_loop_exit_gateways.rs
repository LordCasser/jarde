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
use jarde_reader::classfile::{DescriptorKind, descriptor_facts};
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
    for (name, guard, tail, blocks) in [
        ("prefixWhile", "if (arg0) {", None, &[6, 8, 13][..]),
        ("takenArm", "if (!arg0) {", None, &[6, 8, 13][..]),
        (
            "loopAndTail",
            "if (arg0) {",
            Some("local2 = local2 + 100;"),
            &[6, 8, 13, 23][..],
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
        assert_one_structured_owner_in(&all, arm, blocks, "local3 = 0", &[6, 7]);
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
