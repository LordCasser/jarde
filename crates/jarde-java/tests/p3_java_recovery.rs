//! P3 1.3 acceptance: the first real recovery closed loop — class bytes in, Java text plus a
//! segment table out.
//!
//! What this file has to prove through the public surface:
//!
//! 1. a **real body** reaches the presentation: bytes assembled by the reader's own class builder —
//!    and one committed ECJ fixture — driven through the real analysis entry point, whose payload
//!    `jarde-java` consumes through the 1.1 handoff;
//! 2. a straight-line body with a call and a `return` produces text **and** a segment table whose
//!    ranges really cover the node text they claim, and whose anchors really reach the BCIs the
//!    body has;
//! 3. an `if`/`else` is written with the arms the branch really picks — the fall-through arm inside
//!    the `if` — and the same branch BCI reaches two nodes, one of them `derived`;
//! 4. a body with no debug metadata still gets deterministic names, and a body whose debug names a
//!    Java keyword cannot spell gets a deterministic alias with `representation = java` and
//!    `syntax_status = not_java`;
//! 5. the output budget and the cancellation token can stop the run inside a node, and a stopped run
//!    hands out **no text, no segments and no success** — never an empty body;
//! 6. a body this slice cannot prove (the committed `jsr`/`finally` fixture) is presented with
//!    explicit bytecode fallbacks, `representation = mixed`, and the quoted BCIs are the body's own.
//!
//! The facts the presentation renders from (the method's identity, its debug names, the decoded
//! operations) are assembled here by the driver side of the seam: they are what a class file states
//! and the 1.1 IR payload deliberately does not publish. `operations_of` classifies the opcodes of
//! exactly the fixtures in this file and panics on anything else, so a fixture cannot silently get
//! facts that do not describe it.

use jarde_java::{
    ArithmeticOp, CallTarget, CompareOp, ConstantValue, InvokeKind, MethodFacts, Operation,
    RecoveryFacts, RecoveryOutcome, RecoveryRequest, StopReason, escape_string, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{
    AnalysisStage, CompileStatus, MethodAnalysisRequest, Quality, Representation,
    SemanticValidation, SyntaxStatus,
};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, Limits};
use jarde_reader::classfile::VerificationStatus;
use jarde_reader::classfile::{InstructionFact, class_facts, method_code_facts, test_class};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::LoaderId;
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

/// The ECJ 4.6.1 fixtures: `add(II)I` is a straight-line body with a return value, and the `v45`
/// `finallyPath(I)I` is the committed `jsr`/`finally` body this slice cannot prove.
const HISTORICAL_V45: &[u8] =
    include_bytes!("../../../tests/fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class");

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
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
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

/// The analyzed payload of one method of one class file, kept alive for the request that reads it.
struct Payload {
    analysis: jarde_jvm::method_ir::MethodIrAnalysis,
}

fn analyze(class: &[u8], name: &[u8], descriptor: &[u8]) -> Payload {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: bytes(name),
        descriptor: bytes(descriptor),
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Snapshot {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let environment = ResolutionEnvironment {
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
    };
    let request = MethodAnalysisRequest {
        environment,
        method: method.clone(),
        stages: AnalysisStage::ALL.to_vec(),
    };
    let analysis = analyze_method_ir(&[snapshot], &request, &mut budget)
        .expect("the analysis of an assembled body runs");
    Payload { analysis }
}

/// The decoded operations of one body, classified from the real decode's own opcodes.
///
/// This is the driver side of the seam: the reader states the BCI and the opcode of every
/// instruction, and this table says what the handful of opcodes these fixtures use *mean* — which is
/// the fact the recovery layer renders from and the IR payload does not carry. An opcode outside the
/// table is [`Operation::Other`] rather than a wrong classification, and the fixtures' own bodies
/// are asserted to be inside it.
fn operations_of(instructions: &[InstructionFact]) -> Vec<(u32, Operation)> {
    instructions
        .iter()
        .map(|instruction| {
            let operation = match instruction.opcode {
                0x00 => Operation::Other,                        // nop
                0x01 => Operation::Push(ConstantValue::Null),    // aconst_null
                0x02 => Operation::Push(ConstantValue::Int(-1)), // iconst_m1
                0x03..=0x08 => {
                    Operation::Push(ConstantValue::Int(i64::from(instruction.opcode) - 0x03))
                } // iconst_0..iconst_5
                0x09 => Operation::Push(ConstantValue::Long(0)), // lconst_0
                0x0a => Operation::Push(ConstantValue::Long(1)), // lconst_1
                0x1a..=0x1d => Operation::Load {
                    slot: u16::from(instruction.opcode - 0x1a),
                }, // iload_0..iload_3
                0x2a..=0x2d => Operation::Load {
                    slot: u16::from(instruction.opcode - 0x2a),
                }, // aload_0..aload_3
                0x3b..=0x3e => Operation::Store {
                    slot: u16::from(instruction.opcode - 0x3b),
                }, // istore_0..istore_3
                0x4b..=0x4e => Operation::Store {
                    slot: u16::from(instruction.opcode - 0x4b),
                }, // astore_0..astore_3
                0x60 => Operation::Arithmetic {
                    op: ArithmeticOp::Add,
                },
                0x64 => Operation::Arithmetic {
                    op: ArithmeticOp::Subtract,
                },
                0x99 | 0x9a => Operation::Comparison {
                    op: if instruction.opcode == 0x99 {
                        CompareOp::JumpIfZero
                    } else {
                        CompareOp::JumpIfNotZero
                    },
                }, // ifeq/ifne
                0x9f | 0xa0 => Operation::Comparison {
                    op: if instruction.opcode == 0x9f {
                        CompareOp::JumpIfSame
                    } else {
                        CompareOp::JumpIfDifferent
                    },
                }, // if_icmpeq/if_icmpne
                // The one symbolic reference the reader's builder writes into its pool: entry 15 is
                // `java/lang/Runnable.run:(J)V`. Reading the pool instead of naming it here is the
                // driver wiring this slice does not claim.
                0xb9 => Operation::Invoke(CallTarget::new(
                    InvokeKind::Interface,
                    "java/lang/Runnable",
                    "run",
                    "(J)V",
                )),
                0xa7 | 0xa8 => Operation::Transfer, // goto/goto_w: structure, not a statement
                0xac => Operation::Return,          // ireturn
                0xb1 => Operation::Return,          // return
                _ => Operation::Other,
            };
            (instruction.bci, operation)
        })
        .collect()
}

/// The facts of one method of one class file, read from the real decode.
fn facts_of(
    class: &[u8],
    name: &[u8],
    parameters: u16,
    debug: Vec<Option<String>>,
) -> RecoveryFacts {
    let mut budget = Budget::new(limits());
    let header = class_facts(class, &mut budget).expect("the fixture is a class file");
    let member = header
        .methods
        .iter()
        .find(|member| member.name.raw().0 == name)
        .expect("the fixture declares the method");
    let code = method_code_facts(class, member, &mut budget).expect("the body decodes");
    let mut facts = RecoveryFacts::new(MethodFacts::new(
        String::from_utf8_lossy(name),
        String::from_utf8_lossy(&member.descriptor.raw().0),
        parameters,
    ))
    .with_debug_locals(debug);
    for (bci, operation) in operations_of(&code.instructions) {
        facts = facts.with_operation(bci, operation);
    }
    facts
}

fn recover_body(
    payload: &Payload,
    facts: &RecoveryFacts,
    budget: &mut Budget,
) -> jarde_java::RecoveryReport {
    recover(&RecoveryRequest::new(payload.analysis.ir(), facts), budget)
}

/// The canonical graph of one payload, written out for a failure message.
///
/// A test that states what the *structure* layer decided has to be able to show the graph the
/// decision was made from, or a failure only says that the answer is wrong, not where it came from.
fn describe(payload: &Payload) -> String {
    let mut described = String::new();
    let ir = payload.analysis.ir();
    if let Some(canonical) = ir.canonical() {
        for block in canonical.blocks() {
            described.push_str(&format!(
                "block bci {} covers {:?}\n",
                block.id().bci(),
                block.blocks()
            ));
        }
        for edge in canonical.edges() {
            described.push_str(&format!(
                "edge {} -> {} ({:?})\n",
                edge.from().bci(),
                edge.to().bci(),
                edge.kind()
            ));
        }
        described.push_str(&format!("unreachable: {:?}\n", canonical.unreachable()));
        if let Ok(view) = jarde_java::NormalFlowView::build(canonical, &mut Budget::new(limits())) {
            for node in 0..view.len() {
                described.push_str(&format!(
                    "view node {node} (bci {}) -> successors {:?}, immediate post-dominator {:?}\n",
                    view.id_of(node).map(|id| id.bci()).unwrap_or(u32::MAX),
                    view.successors(node),
                    view.immediate_post_dominator(node)
                ));
            }
            described.push_str(&format!(
                "view kept {} edges, excluded {:?}, cyclic blocks {}\n",
                view.kept_edges(),
                view.excluded(),
                view.cyclic_blocks()
            ));
        }
    }
    described
}

/// `aconst_null; astore_1; aload_1; lconst_0; invokeinterface Runnable.run:(J)V; return`
///
/// One straight-line body whose call has a real receiver (the local the null was stored into), a real
/// argument, and a `return` after it: the shape the task's first case names.
const STRAIGHT_LINE: &[u8] = &[
    0x01, // 0: aconst_null
    0x4c, // 1: astore_1
    0x2b, // 2: aload_1
    0x09, // 3: lconst_0
    0xb9, 0x00, 0x0f, 0x02, 0x00, // 4: invokeinterface java/lang/Runnable.run:(J)V
    0xb1, // 9: return
];

/// `iconst_0; istore_1; iload_1; ifeq +8; iconst_1; istore_2; goto +5; iconst_2; istore_2; return`
///
/// The branch jumps to the second arm when the tested value is zero, so control falls through to the
/// first arm: an `if` whose condition is `local1 != 0`, its then-body the arm at BCI 6 and its
/// else-body the arm at BCI 11 — the polarity the task asks to check.
const IF_ELSE: &[u8] = &[
    0x03, // 0: iconst_0
    0x3c, // 1: istore_1
    0x1b, // 2: iload_1
    0x99, 0x00, 0x08, // 3: ifeq 11
    0x04, // 6: iconst_1
    0x3d, // 7: istore_2
    0xa7, 0x00, 0x05, // 8: goto 13
    0x05, // 11: iconst_2
    0x3d, // 12: istore_2
    0xb1, // 13: return
];

#[test]
fn a_straight_line_body_with_a_call_reaches_text_and_a_segment_table() {
    let class = test_class::single_method(52, 8, 2, STRAIGHT_LINE);
    let payload = analyze(&class, b"method", b"()V");
    let facts = facts_of(&class, b"method", 0, Vec::new());
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, &mut budget);

    assert!(
        report.produced(),
        "the body is presentable: {:?}",
        report.outcome
    );
    assert_eq!(report.representation, Representation::Java);
    assert_eq!(report.quality, Quality::Structured);
    assert_eq!(report.compile_status, CompileStatus::NotAttempted);
    assert_eq!(report.semantic_validation, SemanticValidation::Unproven);
    assert_eq!(report.verification, VerificationStatus::NotPerformed);
    assert_eq!(report.method, "method()V");
    assert!(
        report.text.contains("local1.run(0L);"),
        "the call, with its receiver and its argument:\n{}",
        report.text
    );
    assert!(report.text.contains("return;\n"), "{}", report.text);
    assert!(
        report.text.contains("Object local1 = null;"),
        "the local is declared with the type the value has:\n{}",
        report.text
    );
    assert!(report.fallbacks.is_empty(), "{:?}", report.fallbacks);

    // The segment table: every node's range holds text, and reading the table back by byte finds a
    // node that starts where this one starts and does not reach past it (a node either is the
    // innermost writer of its first byte or contains it).
    for segment in report.source_map.segments() {
        let text = segment.text(&report.text);
        assert!(!text.is_empty(), "no empty node is mapped");
        let covering = report
            .source_map
            .covering(segment.start())
            .expect("every mapped byte has a node");
        assert_eq!(covering.start(), segment.start());
        assert!(
            covering.end() <= segment.end(),
            "the node found by byte is this one or one it contains: {covering:?} inside {segment:?}"
        );
    }
    assert_eq!(
        report.text_of_bci(4),
        vec!["local1.run(0L)", "    local1.run(0L);\n"],
        "the invocation's own text and the statement that holds it, by its BCI"
    );
    assert_eq!(
        report.text_of_bci(1),
        vec!["    Object local1 = null;\n"],
        "{:?}",
        report.text_of_bci(1)
    );
    assert_eq!(
        report.text_of_bci(9),
        vec!["    return;\n"],
        "{:?}",
        report.text_of_bci(9)
    );
    let bcis: Vec<u32> = report
        .source_map
        .segments()
        .iter()
        .flat_map(|segment| segment.origin().bcis())
        .collect();
    assert!(
        bcis.contains(&3) && bcis.contains(&2),
        "the arguments' own BCIs reached the table: {bcis:?}"
    );
}

#[test]
fn an_if_else_is_written_with_the_arm_the_branch_really_picks() {
    let class = test_class::single_method(52, 8, 3, IF_ELSE);
    let payload = analyze(&class, b"method", b"()V");
    let facts = facts_of(&class, b"method", 0, Vec::new());
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, &mut budget);

    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(
        report.representation,
        Representation::Java,
        "{}\nregions: {:?}\nfallbacks: {:?}\ngraph:\n{}",
        report.text,
        report.regions,
        report.fallbacks,
        describe(&payload)
    );
    let text = &report.text;
    let if_at = text.find("if (").expect("the branch became an if");
    let else_at = text.find("} else {").expect("both arms were written");
    let then_arm = text.find("int local2 = 1;").expect("the fall-through arm");
    let else_arm = text.find("local2 = 2;").expect("the branch arm");
    assert!(
        if_at < then_arm && then_arm < else_at && else_at < else_arm,
        "the fall-through arm is inside the if and the branch's target inside the else:\n{text}"
    );
    assert!(
        text.contains("if (local1 != 0) {"),
        "the condition is the jump sense negated:\n{text}"
    );

    // One BCI, several nodes, two provenances: the `if` statement is the branch (direct), the
    // condition it tests presents it (derived), and the constant the test compares against is a node
    // of its own, anchored where the branch is.
    let branch = 3;
    let direct = report.source_map.direct_of_bci(branch);
    let derived = report.source_map.derived_of_bci(branch);
    assert!(
        !direct.is_empty(),
        "the if statement is the branch: {direct:?}"
    );
    assert_eq!(
        derived.len(),
        1,
        "and the condition presents it: {derived:?}"
    );
    assert_eq!(
        derived[0].text(text),
        "local1 != 0",
        "the condition's own span"
    );
    assert!(
        direct
            .iter()
            .any(|segment| segment.text(text).contains("if (local1 != 0) {")),
        "one of the branch's own nodes is the statement: {direct:?}"
    );
    assert!(
        report.text_of_bci(branch).len() >= 2,
        "the same BCI reached more than one node: {:?}",
        report.text_of_bci(branch)
    );
}

#[test]
fn a_body_without_debug_metadata_gets_deterministic_names() {
    let class = test_class::single_method(52, 8, 2, STRAIGHT_LINE);
    let payload = analyze(&class, b"method", b"()V");
    let facts = facts_of(&class, b"method", 0, Vec::new());
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, &mut budget);
    assert!(
        report.text.contains("Object local1 = null;"),
        "{}",
        report.text
    );
    assert!(
        report.aliased_names.is_empty(),
        "an invented name hides nothing: {:?}",
        report.aliased_names
    );
    assert_eq!(
        report.syntax_status,
        SyntaxStatus::Unchecked,
        "nothing in this slice runs a syntax check"
    );

    // The same body, the same names: the table is a function of the evidence and the slot order.
    let mut other_budget = Budget::new(limits());
    let again = recover_body(&payload, &facts, &mut other_budget);
    assert_eq!(again.text, report.text);

    // With debug names, the evidence is written as it stands.
    let named = facts_of(
        &class,
        b"method",
        0,
        vec![None, Some("receiver".to_string())],
    );
    let mut named_budget = Budget::new(limits());
    let named_report = recover_body(&payload, &named, &mut named_budget);
    assert!(
        named_report.text.contains("Object receiver = null;"),
        "{}",
        named_report.text
    );
    assert!(
        named_report.text.contains("receiver.run(0L);"),
        "{}",
        named_report.text
    );
}

#[test]
fn a_keyword_name_takes_a_deterministic_alias_and_the_syntax_plane_says_so() {
    let class = test_class::single_method(52, 8, 2, STRAIGHT_LINE);
    let payload = analyze(&class, b"method", b"()V");
    let facts = facts_of(&class, b"method", 0, vec![None, Some("int".to_string())]);
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, &mut budget);

    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report.text.contains("Object int_ = null;"),
        "the keyword became a deterministic alias:\n{}",
        report.text
    );
    assert!(report.text.contains("int_.run(0L);"), "{}", report.text);
    assert!(
        !report.text.contains("Object int ="),
        "no identifier is spelled as a keyword"
    );
    // The one combination the P3 vocabulary states: Java presentation, syntax not claimed.
    assert_eq!(report.representation, Representation::Java);
    assert_eq!(report.syntax_status, SyntaxStatus::NotJava);
    assert_eq!(report.aliased_names.len(), 1, "{:?}", report.aliased_names);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_name_aliased"),
        "{:?}",
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.clone())
            .collect::<Vec<_>>()
    );

    // The lexical rules themselves, through the published rule rather than through a body: every
    // vector is one the route decision recorded, including a surrogate pair and a line separator.
    assert_eq!(escape_string("a\"b"), "a\\\"b");
    assert_eq!(escape_string("a\\b"), "a\\\\b");
    assert_eq!(escape_string("a\nb"), "a\\nb");
    assert_eq!(escape_string("\u{0}"), "\\u0000");
    assert_eq!(escape_string("\u{7}"), "\\u0007");
    assert_eq!(escape_string("\u{7f}"), "\\u007f");
    assert_eq!(escape_string("\u{2028}"), "\\u2028");
    assert_eq!(escape_string("😀"), "\\ud83d\\ude00");
    assert_eq!(escape_string("\\u0041"), "\\\\u0041");
}

#[test]
fn a_body_the_subset_cannot_prove_is_quoted_rather_than_emptied() {
    // The committed ECJ v45 fixture: `finallyPath` is the one committed body whose canonical graph
    // is more than a straight line, and it is a `jsr`/`finally` body — exactly the shape this slice
    // refuses to present.
    let payload = analyze(HISTORICAL_V45, b"finallyPath", b"(I)I");
    let facts = facts_of(HISTORICAL_V45, b"finallyPath", 1, Vec::new());
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, &mut budget);

    assert!(
        report.produced(),
        "the body is presented, not dropped: {:?}",
        report.outcome
    );
    assert!(
        !report.fallbacks.is_empty(),
        "a jsr body is a fallback and says so: {:?}",
        report.regions
    );
    assert_eq!(report.representation, Representation::Mixed);
    assert_eq!(report.quality, Quality::Fallback);
    assert_eq!(report.syntax_status, SyntaxStatus::NotJava);
    assert!(
        report.text.contains("// @bytecode"),
        "{}\n{:?}",
        report.text,
        report.regions
    );

    // The quoted BCIs are the body's own: every one of them is an instruction start of the body.
    let mut decode_budget = Budget::new(limits());
    let header =
        class_facts(HISTORICAL_V45, &mut decode_budget).expect("the fixture is a class file");
    let member = header
        .methods
        .iter()
        .find(|member| member.name.raw().0 == b"finallyPath")
        .expect("the fixture declares finallyPath");
    let code = method_code_facts(HISTORICAL_V45, member, &mut decode_budget).expect("it decodes");
    let starts: Vec<u32> = code
        .instructions
        .iter()
        .map(|instruction| instruction.bci)
        .collect();
    let quoted: Vec<u32> = report
        .source_map
        .segments()
        .iter()
        .filter(|segment| {
            segment
                .text(&report.text)
                .trim_start()
                .starts_with("// @bytecode")
        })
        .flat_map(|segment| segment.origin().bcis())
        .collect();
    assert!(!quoted.is_empty(), "the fallback quotes BCIs");
    for bci in &quoted {
        assert!(
            starts.contains(bci),
            "BCI {bci} is not an instruction start of the body"
        );
    }
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.starts_with("jre_region_")),
        "{:?}",
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_committed_straight_line_body_is_presented_from_its_own_decode() {
    // The committed corpus, not an assembled fixture: `add(II)I` is `iload_1; iload_2; iadd;
    // ireturn`, and the operations come from the real decode's own BCIs, so nothing in this test
    // knows where the instructions are. `add` is an instance method, so its three parameter slots are
    // `this`, `left` and `right`: slot 0 has no name of its own here because naming a receiver is a
    // 3.1 decision with evidence this slice does not read (flags and descriptor), and an ordinal name
    // is the deterministic answer until then.
    let payload = analyze(HISTORICAL_V45, b"add", b"(II)I");
    let facts = facts_of(HISTORICAL_V45, b"add", 3, Vec::new());
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, &mut budget);
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report.text.contains("return arg1 + arg2;"),
        "the body's own arithmetic and return:\n{}",
        report.text
    );
    assert_eq!(
        report.representation,
        Representation::Java,
        "{:?}",
        report.regions
    );
    assert_eq!(report.quality, Quality::Structured);
    assert!(
        report.aliased_names.is_empty(),
        "{:?}",
        report.aliased_names
    );
    assert_eq!(
        report.syntax_status,
        SyntaxStatus::Unchecked,
        "no alias was needed and nothing checked the syntax"
    );
    assert!(
        !report.source_map.is_empty(),
        "and the nodes are mapped: {:?}",
        report.source_map
    );
}

#[test]
fn the_output_bound_is_checked_at_the_write_and_a_stop_produces_nothing() {
    let class = test_class::single_method(52, 8, 2, STRAIGHT_LINE);
    let payload = analyze(&class, b"method", b"()V");
    let facts = facts_of(&class, b"method", 0, Vec::new());

    let exact = {
        let mut budget = Budget::new(limits());
        let report = recover_body(&payload, &facts, &mut budget);
        assert!(report.produced());
        u64::try_from(report.text.len()).expect("the text fits u64")
    };

    // Exactly the produced size: the artifact is produced.
    let mut at_bound = Budget::new(Limits {
        output_bytes: exact,
        ..limits()
    });
    let report = recover_body(&payload, &facts, &mut at_bound);
    assert!(
        report.produced(),
        "a run at the bound produced its artifact: {:?}",
        report.outcome
    );
    assert_eq!(u64::try_from(report.text.len()).unwrap(), exact);

    // One byte short: no artifact, no segments, no success state.
    let mut over = Budget::new(Limits {
        output_bytes: exact - 1,
        ..limits()
    });
    let report = recover_body(&payload, &facts, &mut over);
    assert!(
        !report.produced(),
        "a refused write is not a produced artifact"
    );
    assert_eq!(report.text, "", "the buffer was discarded");
    assert!(report.source_map.is_empty(), "and so was the segment table");
    assert_eq!(report.representation, Representation::Bytecode);
    assert_eq!(report.quality, Quality::Fallback);
    assert_eq!(report.syntax_status, SyntaxStatus::NotJava);
    let stop = report.stop().expect("a stopped run states its stop");
    match stop {
        StopReason::Budget { written, limit, .. } => {
            assert_eq!(*limit, exact - 1);
            assert!(
                *written < exact,
                "the stop never claims more than the bound"
            );
        }
        other => panic!("expected the output bound, got {other:?}"),
    }
    match &report.execution {
        jarde_reader::model::ExecutionReport::Partial { reason, .. } => assert!(
            matches!(
                reason,
                jarde_reader::model::TerminationReason::BudgetExceeded { .. }
            ),
            "the execution plane states the bound, not a success: {reason:?}"
        ),
        other => panic!("expected a partial execution, got {other:?}"),
    }
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_output_budget")
    );
}

#[test]
fn a_cancelled_run_produces_nothing_and_says_it_was_cancelled() {
    let class = test_class::single_method(52, 8, 2, STRAIGHT_LINE);
    let payload = analyze(&class, b"method", b"()V");
    let facts = facts_of(&class, b"method", 0, Vec::new());
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let report = recover_body(&payload, &facts, &mut budget);
    assert!(!report.produced(), "there is no artifact to hand out");
    assert_eq!(report.text, "");
    assert!(report.source_map.is_empty());
    assert!(report.stop().expect("a reason").is_cancelled());
    assert!(matches!(
        report.execution,
        jarde_reader::model::ExecutionReport::Cancelled { .. }
    ));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_cancelled"),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn a_payload_without_the_tables_the_recovery_needs_states_that_instead_of_an_empty_body() {
    // The request names a method the fixture does not declare, so the run publishes no payload: the
    // recovery layer has to state that, not present an empty body.
    let class = test_class::single_method(52, 8, 2, STRAIGHT_LINE);
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.clone()), &mut budget)
        .expect("the fixture opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(&class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("the fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: bytes(b"absent"),
        descriptor: bytes(b"()V"),
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Snapshot {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let environment = ResolutionEnvironment {
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
    };
    let request = MethodAnalysisRequest {
        environment,
        method,
        stages: AnalysisStage::ALL.to_vec(),
    };
    let analysis = analyze_method_ir(&[snapshot], &request, &mut budget)
        .expect("a request for an absent method still runs");
    let facts = RecoveryFacts::new(MethodFacts::new("absent", "()V", 0));
    let mut recovery_budget = Budget::new(limits());
    let report = recover(
        &RecoveryRequest::new(analysis.ir(), &facts),
        &mut recovery_budget,
    );
    assert!(!report.produced());
    assert_eq!(report.text, "");
    match report.stop().expect("a stop") {
        StopReason::IrTableMissing { table } => assert_eq!(*table, "canonical"),
        other => panic!("expected the missing table, got {other:?}"),
    }
    assert!(matches!(
        report.outcome,
        RecoveryOutcome::Stopped(StopReason::IrTableMissing { .. })
    ));
}
