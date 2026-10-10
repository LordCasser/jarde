//! Reader-backed public-API draft for conditional-switch scope boundaries.
//!
//! The frozen complete class is the javac 8 input recorded by boundary-public-ir-root-v1.
//! This test pins that input and the public IR shape, then checks the observed conservative
//! outcomes and the complete terminal-case source map.

use jarde_java::{
    ArtifactSubject, DebugLocal, DeclaringClass, MethodFacts, RecoveryContent,
    RecoveryEvidenceKind, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest, pass::JAVA_8,
    recover,
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
use std::collections::BTreeSet;

const CLASS: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-conditional-switch-boundaries/javac8/original/ConditionalSwitchBoundaries.class"
);
const CLASS_LEN: usize = 1494;
const CLASS_BLAKE3: &str = "49795c66605066655f48ac37b862ccd132e43b74d290653afd51762ab1a409b7";
const OWNER: &str = "ConditionalSwitchBoundaries";
const METHODS: [(&str, &str, usize, usize); 4] = [
    ("innerLoopBreak", "(II)Ljava/lang/String;", 10, 13),
    ("innerSwitchBreak", "(II)Ljava/lang/String;", 8, 10),
    ("terminalCase", "(I)Ljava/lang/String;", 6, 6),
    (
        "caughtExceptionThenFallthrough",
        "(II)Ljava/lang/String;",
        9,
        14,
    ),
];

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
    let mut counts = std::collections::BTreeMap::new();
    for record in records {
        let key = (
            record.slot,
            record.start_bci,
            record.end_bci,
            record.name.0.as_slice(),
        );
        *counts.entry(key).or_insert(0usize) += 1;
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
            if counts.get(&key) == Some(&1)
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
    block_bcis: BTreeSet<u32>,
    edges: Vec<(u32, String, u32)>,
}

fn run(name: &str, descriptor: &str, evidence: RecoveryEvidenceRequest) -> Run {
    assert_eq!(
        CLASS.len(),
        CLASS_LEN,
        "the complete frozen class is embedded"
    );
    assert_eq!(blake3::hash(CLASS).to_hex().to_string(), CLASS_BLAKE3);

    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
        .expect("complete frozen class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(CLASS).to_hex().to_string()),
            length: u64::try_from(CLASS.len()).expect("class length fits u64"),
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
    let requested_method = PhysicalMethodId {
        owner: definition,
        name: JvmBytes(name.as_bytes().to_vec()),
        descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
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
            method: requested_method,
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("public reader analysis completes for the exact method");
    let method = analysis.report().method.clone();
    assert_eq!(method.name.0.as_slice(), name.as_bytes());
    assert_eq!(method.descriptor.0.as_slice(), descriptor.as_bytes());
    assert_eq!(method.owner.class_bytes.digest.0, CLASS_BLAKE3);
    assert_eq!(method.owner.class_bytes.length, CLASS_LEN as u64);

    let ir = analysis.ir();
    let canonical = ir.canonical().expect("reader publishes canonical CFG");
    let code = ir.code().expect("reader publishes physical decoded code");
    assert_eq!(code.instructions.len(), code.operands().len());
    let block_bcis = canonical
        .blocks()
        .iter()
        .map(|block| {
            assert!(
                block.id().path().is_empty(),
                "frozen input uses physical canonical blocks"
            );
            block.id().bci()
        })
        .collect::<BTreeSet<_>>();
    let edges = canonical
        .edges()
        .iter()
        .map(|edge| {
            assert!(edge.from().path().is_empty() && edge.to().path().is_empty());
            (
                edge.from().bci(),
                format!("{:?}", edge.kind()),
                edge.to().bci(),
            )
        })
        .collect::<Vec<_>>();
    let prepared_read = snapshot
        .prepared_root_class(&mut budget)
        .expect("same complete class prepares");
    let prepared = jarde_reader::prepared::PreparedClass::prepare(&prepared_read, &mut budget)
        .expect("complete class method table reads");
    let ordinals = prepared.locate_method(name.as_bytes(), descriptor.as_bytes());
    assert_eq!(ordinals.len(), 1, "the exact declaration is unique");
    let ordinal = ordinals[0];
    let subject = ArtifactSubject::new(
        method.clone(),
        Some(ordinal),
        analysis.report().environment_identity.clone(),
    );
    let declaration = ir
        .declaration()
        .expect("reader states the method declaration");
    assert_eq!(declaration.class_name().0.as_slice(), OWNER.as_bytes());
    assert_eq!(declaration.name().0.as_slice(), name.as_bytes());
    assert_eq!(declaration.descriptor().0.as_slice(), descriptor.as_bytes());
    let facts = RecoveryFacts::new(
        MethodFacts::new(
            String::from_utf8_lossy(&declaration.name().0).into_owned(),
            String::from_utf8_lossy(&declaration.descriptor().0).into_owned(),
            declaration.parameter_slots(),
        )
        .with_access_flags(declaration.access_flags())
        .with_declaring_class(DeclaringClass::new(
            String::from_utf8_lossy(&declaration.class_name().0).into_owned(),
            declaration.class_access_flags(),
        )),
    )
    .with_debug_locals(reader_debug_locals(ir.code()));
    let request = RecoveryRequest::new(ir, &facts, JAVA_8)
        .with_evidence(evidence)
        .with_subject(subject);
    let report = recover(&request, &mut budget);
    if report.produced() {
        let binding = report.artifact.binding().expect("public result is bound");
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
        block_bcis,
        edges,
    }
}

fn normal_edges(rows: &[(u32, u32)]) -> BTreeSet<(u32, String, u32)> {
    rows.iter()
        .map(|(from, to)| (*from, "Normal".to_owned(), *to))
        .collect()
}

fn expected_graph(name: &str) -> (BTreeSet<u32>, BTreeSet<(u32, String, u32)>) {
    match name {
        "innerLoopBreak" => (
            [0, 36, 38, 43, 54, 57, 63, 70, 80, 87].into(),
            normal_edges(&[
                (0, 36),
                (0, 70),
                (0, 80),
                (36, 38),
                (38, 43),
                (38, 63),
                (43, 54),
                (43, 57),
                (54, 63),
                (57, 38),
                (63, 70),
                (70, 87),
                (80, 87),
            ]),
        ),
        "innerSwitchBreak" => (
            [0, 36, 56, 66, 73, 80, 90, 97].into(),
            normal_edges(&[
                (0, 36),
                (0, 80),
                (0, 90),
                (36, 56),
                (36, 66),
                (56, 73),
                (66, 73),
                (73, 80),
                (80, 97),
                (90, 97),
            ]),
        ),
        "terminalCase" => (
            [0, 36, 48, 67, 77, 84].into(),
            normal_edges(&[(0, 36), (0, 48), (0, 67), (0, 77), (67, 84), (77, 84)]),
        ),
        "caughtExceptionThenFallthrough" => {
            let mut edges = normal_edges(&[
                (0, 36),
                (0, 68),
                (0, 78),
                (36, 40),
                (36, 45),
                (40, 47),
                (45, 47),
                (47, 68),
                (60, 68),
                (68, 85),
                (78, 85),
            ]);
            for from in [40, 45, 47] {
                edges.insert((from, "Exception { handler_ordinal: 0 }".to_owned(), 60));
            }
            ([0, 36, 40, 45, 47, 60, 68, 78, 85].into(), edges)
        }
        _ => panic!("no frozen graph for {name}"),
    }
}

fn assert_explanation_only(run: &Run, code: &str, bci: u32, blocks: &[u32]) {
    assert!(run.report.outcome.produced(), "{:?}", run.report.outcome);
    assert_eq!(run.report.content, RecoveryContent::ExplanationOnly);
    let refusal = run
        .report
        .regions
        .iter()
        .find(|region| region.bci == bci && region.code == Some(code))
        .unwrap_or_else(|| panic!("expected {code} at BCI {bci}: {:?}", run.report.regions));
    assert_eq!(refusal.blocks.as_slice(), blocks);
    assert!(
        !run.report.text.contains("break;"),
        "a refused boundary must not emit an unlabeled break: {}",
        run.report.text
    );
}

fn terminal_physical_instructions() -> Vec<jarde_reader::classfile::InstructionFact> {
    let inspection = inspect_method_bytecode(
        CLASS,
        MethodSelector {
            name: JvmBytes(b"terminalCase".to_vec()),
            descriptor: JvmBytes(b"(I)Ljava/lang/String;".to_vec()),
        },
        &mut Budget::new(limits()),
    )
    .expect("reader decodes the exact terminalCase method");
    assert!(
        inspection.stopped_at.is_none(),
        "physical decode stopped: {:?}",
        inspection.stopped_at
    );
    inspection.instructions
}

fn assert_full_physical_coverage(report: &jarde_java::RecoveryReport, method: &PhysicalMethodId) {
    for instruction in terminal_physical_instructions() {
        let segments = report.source_map.of_bci(instruction.bci);
        assert!(
            !segments.is_empty(),
            "physical BCI {} is unmapped",
            instruction.bci
        );
        assert!(
            segments.iter().any(|segment| {
                std::iter::once(segment.origin().primary())
                    .chain(segment.origin().derived())
                    .any(|origin| {
                        origin.bci() == instruction.bci && origin.method() == Some(method)
                    })
            }),
            "physical BCI {} has no matching method origin: {segments:?}",
            instruction.bci,
        );
    }
}

fn assert_direct_statement(
    report: &jarde_java::RecoveryReport,
    method: &PhysicalMethodId,
    bci: u32,
    expected_statement: &str,
) {
    let segments = report.source_map.direct_of_bci(bci);
    assert!(
        segments.iter().any(|segment| {
            segment.text(&report.text).trim() == expected_statement
                && segment.origin().primary().bci() == bci
                && segment.origin().primary().method() == Some(method)
        }),
        "BCI {bci} lacks the exact direct statement {expected_statement:?}: {segments:?}\n{}",
        report.text,
    );
}

#[test]
fn loop_nested_switch_and_caught_edges_remain_conservative() {
    for (name, descriptor, expected_blocks, expected_edges) in METHODS {
        if name == "terminalCase" {
            continue;
        }
        let observed = run(name, descriptor, RecoveryEvidenceRequest::all());
        assert_eq!(
            observed.block_bcis.len(),
            expected_blocks,
            "{name} block starts"
        );
        assert_eq!(
            observed.edges.len(),
            expected_edges,
            "{name} canonical edge count"
        );
        let (expected_block_bcis, expected_edge_set) = expected_graph(name);
        assert_eq!(
            observed.block_bcis, expected_block_bcis,
            "{name} physical blocks"
        );
        assert_eq!(
            observed.edges.iter().cloned().collect::<BTreeSet<_>>(),
            expected_edge_set,
            "{name} exact canonical edge set",
        );
        if name != "caughtExceptionThenFallthrough" {
            assert!(
                observed.edges.iter().all(|(_, kind, _)| kind == "Normal"),
                "{name} preserves its observed all-Normal graph: {:?}",
                observed.edges
            );
        }
        if name == "innerLoopBreak" {
            assert!(
                observed.edges.contains(&(57, "Normal".to_owned(), 38)),
                "the physical loop back-edge remains in the canonical graph"
            );
            assert_explanation_only(
                &observed,
                "jre_region_uncovered_blocks",
                38,
                &[38, 43, 63, 54, 57],
            );
        } else if name == "innerSwitchBreak" {
            assert_explanation_only(
                &observed,
                "jre_region_arms_do_not_meet",
                0,
                &[0, 36, 80, 90, 56, 66, 97, 73],
            );
        }

        if name == "caughtExceptionThenFallthrough" {
            let exceptional = observed
                .edges
                .iter()
                .filter(|(_, kind, _)| kind.starts_with("Exception"))
                .collect::<Vec<_>>();
            assert_eq!(
                exceptional.len(),
                3,
                "the reader preserves all three real handler edges"
            );
            assert_eq!(
                exceptional
                    .iter()
                    .map(|(from, _, to)| (*from, *to))
                    .collect::<BTreeSet<_>>(),
                BTreeSet::from([(40, 60), (45, 60), (47, 60)]),
                "caught throw sites reach the physical handler entry",
            );
            assert!(observed.edges.iter().all(|(from, kind, to)| {
                !kind.starts_with("Exception")
                    || (*from, kind.as_str(), *to) == (40, "Exception { handler_ordinal: 0 }", 60)
                    || (*from, kind.as_str(), *to) == (45, "Exception { handler_ordinal: 0 }", 60)
                    || (*from, kind.as_str(), *to) == (47, "Exception { handler_ordinal: 0 }", 60)
            }));
            assert_explanation_only(&observed, "jre_region_uncovered_blocks", 47, &[47]);
        }
    }
}

const TERMINAL_TEXT: &str = r#"// @method terminalCase(I)Ljava/lang/String;
// @declaration a static method of `ConditionalSwitchBoundaries`, member flags 0x0009
// recovered from bytecode; presentation is not claimed to compile
{
    java.lang.StringBuilder local1;
    local1 = new java.lang.StringBuilder();
    switch (arg0) {
        case 0:
            local1.append("R");
            return local1.toString();
        case 1:
            local1.append("T");
            throw new java.lang.IllegalArgumentException((java.lang.String) local1.toString());
        case 2:
            local1.append("C");
            break;
        default:
            local1.append("D");
            break;
    }
    return local1.toString();
}
"#;

#[test]
fn terminal_return_and_throw_keep_exact_text_and_physical_origins() {
    let default = run(
        "terminalCase",
        "(I)Ljava/lang/String;",
        RecoveryEvidenceRequest::essential(),
    );
    let all = run(
        "terminalCase",
        "(I)Ljava/lang/String;",
        RecoveryEvidenceRequest::all(),
    );
    let explicit_map = run(
        "terminalCase",
        "(I)Ljava/lang/String;",
        RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
    );

    assert_eq!(default.method, all.method);
    assert_eq!(all.method, explicit_map.method);
    assert_eq!(default.ordinal, all.ordinal);
    assert_eq!(all.ordinal, explicit_map.ordinal);
    assert!(all.report.outcome.produced());
    assert_eq!(all.report.content, RecoveryContent::ContainsStatements);
    let (terminal_blocks, terminal_edges) = expected_graph("terminalCase");
    assert_eq!(all.block_bcis, terminal_blocks);
    assert_eq!(
        all.edges.iter().cloned().collect::<BTreeSet<_>>(),
        terminal_edges
    );
    assert_eq!(
        default.report.text, all.report.text,
        "default and all evidence preserve the terminal body"
    );
    assert_eq!(
        explicit_map.report.text, all.report.text,
        "requesting only the source map preserves the terminal body"
    );
    assert!(default.report.source_map.is_empty());
    assert_eq!(explicit_map.report.source_map, all.report.source_map);
    assert_eq!(all.report.text, TERMINAL_TEXT);
    assert!(
        all.report
            .regions
            .iter()
            .all(|region| region.code.is_none())
    );

    let opcodes = terminal_physical_instructions()
        .into_iter()
        .map(|instruction| (instruction.bci, instruction.opcode))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(opcodes.get(&47), Some(&0xb0), "areturn at BCI 47");
    assert_eq!(opcodes.get(&66), Some(&0xbf), "athrow at BCI 66");
    assert_eq!(opcodes.get(&88), Some(&0xb0), "areturn at BCI 88");
    assert_direct_statement(&all.report, &all.method, 47, "return local1.toString();");
    assert_direct_statement(
        &all.report,
        &all.method,
        66,
        "throw new java.lang.IllegalArgumentException((java.lang.String) local1.toString());",
    );
    assert_direct_statement(&all.report, &all.method, 88, "return local1.toString();");
    assert_full_physical_coverage(&all.report, &all.method);
}
