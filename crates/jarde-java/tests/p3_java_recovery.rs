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
//! The only facts this file supplies are the method's identity and its debug names — what the 1.1
//! payload does not carry. Everything about what the instructions *do* (a branch's polarity and
//! target, a load's slot, an invocation's owner/name/descriptor, a constant's value) is read out of
//! the payload by [`jarde_java::decode`], from the very decode the analysis run performed: there is
//! no table here to get wrong, and no parameter through which one could be handed in.

use jarde_java::{
    MethodFacts, RecoveryFacts, RecoveryOutcome, RecoveryRequest, StopReason, escape_string,
    recover,
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
use jarde_reader::classfile::{class_facts, method_code_facts, test_class};
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

/// The facts of one method, read from the real class file — identity and debug names only.
///
/// The decoded operations are **not** here any more (P3 1.3b): they travel inside the payload the
/// analysis run produced, so every body in this file is presented from the very decode that ran.
/// Nothing in this test can state a branch's polarity, a load's slot or a constant's value — the
/// only way to change what the presentation reads is to change the bytes.
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
    RecoveryFacts::new(MethodFacts::new(
        String::from_utf8_lossy(name),
        String::from_utf8_lossy(&member.descriptor.raw().0),
        parameters,
    ))
    .with_debug_locals(debug)
}

fn recover_body(
    payload: &Payload,
    facts: &RecoveryFacts,
    budget: &mut Budget,
) -> jarde_java::RecoveryReport {
    recover(
        &RecoveryRequest::new(payload.analysis.ir(), facts, jarde_java::pass::JAVA_8),
        budget,
    )
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

/// `iconst_3; istore_1; header: iload_1; ifeq exit; body: aload_2; lconst_0; run; iload_1; iconst_1;
/// isub; istore_1; goto header; exit: return`
///
/// A header-tested loop whose test reads a local **inside** the loop: the read happens once per
/// iteration, and its condition is the fall-through sense of the branch (`local1 != 0`), because the
/// successor the branch transfers to is the exit.
const WHILE_LOOP: &[u8] = &[
    0x01, // 0: aconst_null
    0x4d, // 1: astore_2
    0x06, // 2: iconst_3
    0x3c, // 3: istore_1
    0x1b, // 4: iload_1            <-- the header
    0x99, 0x00, 0x11, // 5: ifeq 22   <-- the test: transfers to the exit
    0x2c, // 8: aload_2            <-- the body
    0x09, // 9: lconst_0
    0xb9, 0x00, 0x0f, 0x02, 0x00, // 10: invokeinterface Runnable.run:(J)V
    0x1b, // 15: iload_1
    0x04, // 16: iconst_1
    0x64, // 17: isub
    0x3c, // 18: istore_1
    0xa7, 0xff, 0xf1, // 19: goto 4
    0xb1, // 22: return
];

/// `do { aload_2; lconst_0; run; local1 = local1 - 1; } while (local1 != 0);`
///
/// The same body with its test at the *bottom*: the latch's branch transfers back to the header, so
/// the loop repeats when its sense holds — a `do … while`, never a `while`, because the first
/// iteration runs before the test is read.
const DO_WHILE_LOOP: &[u8] = &[
    0x01, // 0: aconst_null
    0x4d, // 1: astore_2
    0x06, // 2: iconst_2         <-- the header, entered from the code before it and from the latch
    0x3c, // 3: istore_1
    0x2c, // 4: aload_2          <-- the body
    0x09, // 5: lconst_0
    0xb9, 0x00, 0x0f, 0x02, 0x00, // 6: invokeinterface Runnable.run:(J)V
    0x1b, // 11: iload_1
    0x04, // 12: iconst_1
    0x64, // 13: isub
    0x3c, // 14: istore_1
    0x1b, // 15: iload_1         <-- the latch
    0x9a, 0xff, 0xf2, // 16: ifne 2   <-- the test: transfers back to the header
    0xb1, // 19: return
];

/// `switch (local1) { case 0: case 1: local2 = 1; default: local2 = 2; } return;`
///
/// A `tableswitch` with two keys sharing one target and a default of its own: the two keys are one
/// arm (a Java `case 0: case 1:`), and the default's block falls through into the join.
const TABLE_SWITCH: &[u8] = &[
    0x03, // 0: iconst_0
    0x3c, // 1: istore_1
    0x1b, // 2: iload_1
    0xaa, // 3: tableswitch (its operands start at 4, already aligned)
    0x00, 0x00, 0x00, 0x1a, // 4: default → +26 = 29
    0x00, 0x00, 0x00, 0x00, // 8: low = 0
    0x00, 0x00, 0x00, 0x01, // 12: high = 1
    0x00, 0x00, 0x00, 0x15, // 16: key 0 → +21 = 24
    0x00, 0x00, 0x00, 0x15, // 20: key 1 → +21 = 24
    0x04, // 24: iconst_1
    0x3d, // 25: istore_2
    0xa7, 0x00, 0x05, // 26: goto 31
    0x05, // 29: iconst_2
    0x3d, // 30: istore_2
    0xb1, // 31: return
];

/// A cycle with **two** entries: the entry block reaches both blocks of it, so neither dominates the
/// other and the component has no header.
///
/// `if (local1 == 0) goto 11; A: iload_1; ifne 11; return; 11: goto 4`
const IRREDUCIBLE: &[u8] = &[
    0x03, // 0: iconst_0
    0x3c, // 1: istore_1
    0x1b, // 2: iload_1     (the entry block: it reaches *both* blocks of the cycle)
    0x99, 0x00, 0x08, // 3: ifeq 11   (→ A when the value is zero)
    0x1b, // 6: iload_1     (B: entered from the entry block)
    0x9a, 0x00, 0x04, // 7: ifne 11   (→ A when the value is non-zero; otherwise BCI 10)
    0xb1, // 10: return
    0xa7, 0xff, 0xfb, // 11: goto 6   (A → B: 11 + (-5) = 6)
];

/// A class file whose exception table holds two records with **crossing** declared ranges.
///
/// `method()V` is `aconst_null; astore_0; aload_0; athrow; return`, with two more `athrow`s as the
/// handler entries. The two records cover `[2, 4)` and `[3, 6)` — neither nested nor disjoint — and
/// both cover the throw site at BCI 3, so the priority between them is the table's order alone.
fn crossing_exception_class() -> Vec<u8> {
    let mut pool: Vec<u8> = Vec::new();
    let entry = |pool: &mut Vec<u8>, tag: u8, body: &[u8]| {
        pool.push(tag);
        pool.extend_from_slice(body);
    };
    let utf8 = |pool: &mut Vec<u8>, text: &str| {
        let bytes = text.as_bytes();
        let mut body = Vec::new();
        body.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
        body.extend_from_slice(bytes);
        entry(pool, 1, &body);
    };
    let class = |pool: &mut Vec<u8>, name_index: u16| entry(pool, 7, &name_index.to_be_bytes());
    utf8(&mut pool, "Test"); // 1
    class(&mut pool, 1); // 2
    utf8(&mut pool, "java/lang/Object"); // 3
    class(&mut pool, 3); // 4
    utf8(&mut pool, "method"); // 5
    utf8(&mut pool, "()V"); // 6
    utf8(&mut pool, "Code"); // 7
    utf8(&mut pool, "java/lang/RuntimeException"); // 8
    class(&mut pool, 8); // 9
    utf8(&mut pool, "java/lang/Error"); // 10
    class(&mut pool, 10); // 11

    let code: Vec<u8> = vec![
        0x01, // 0: aconst_null
        0x4b, // 1: astore_0
        0x2a, // 2: aload_0
        0xbf, // 3: athrow      (the protected throw site: both records cover it)
        0xb1, // 4: return
        0xbf, // 5: athrow      (record 0's handler entry)
        0xbf, // 6: athrow      (record 1's handler entry)
    ];
    let mut attribute = Vec::new();
    attribute.extend_from_slice(&1_u16.to_be_bytes()); // max_stack
    attribute.extend_from_slice(&1_u16.to_be_bytes()); // max_locals
    attribute.extend_from_slice(&(code.len() as u32).to_be_bytes());
    attribute.extend_from_slice(&code);
    attribute.extend_from_slice(&2_u16.to_be_bytes()); // two exception records
    for (start, end, handler, catch) in [
        (2_u16, 4_u16, 5_u16, 9_u16),  // [2, 4) catching RuntimeException
        (3_u16, 6_u16, 6_u16, 11_u16), // [3, 6) catching Error — it crosses the record above
    ] {
        attribute.extend_from_slice(&start.to_be_bytes());
        attribute.extend_from_slice(&end.to_be_bytes());
        attribute.extend_from_slice(&handler.to_be_bytes());
        attribute.extend_from_slice(&catch.to_be_bytes());
    }
    attribute.extend_from_slice(&0_u16.to_be_bytes()); // the Code attribute has no attribute

    let mut out = Vec::new();
    out.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes()); // minor
    out.extend_from_slice(&52_u16.to_be_bytes()); // major: Java 8
    out.extend_from_slice(&12_u16.to_be_bytes()); // constant_pool_count
    out.extend_from_slice(&pool);
    out.extend_from_slice(&0x0021_u16.to_be_bytes()); // public super
    out.extend_from_slice(&2_u16.to_be_bytes()); // this_class
    out.extend_from_slice(&4_u16.to_be_bytes()); // super_class
    out.extend_from_slice(&0_u16.to_be_bytes()); // interfaces
    out.extend_from_slice(&0_u16.to_be_bytes()); // fields
    out.extend_from_slice(&1_u16.to_be_bytes()); // methods
    out.extend_from_slice(&0x0009_u16.to_be_bytes()); // public static
    out.extend_from_slice(&5_u16.to_be_bytes()); // name → "method"
    out.extend_from_slice(&6_u16.to_be_bytes()); // descriptor → "()V"
    out.extend_from_slice(&1_u16.to_be_bytes()); // one attribute
    out.extend_from_slice(&7_u16.to_be_bytes()); // "Code"
    out.extend_from_slice(&(attribute.len() as u32).to_be_bytes());
    out.extend_from_slice(&attribute);
    out.extend_from_slice(&0_u16.to_be_bytes()); // no class attributes
    out
}

#[test]
fn a_while_loop_keeps_its_test_inside_the_statement_and_its_body_in_order() {
    let class = test_class::single_method(52, 8, 3, WHILE_LOOP);
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
    // The statement, its test and its body, in one shape: the test is *inside* the `while`, so the
    // load it makes runs once per evaluation — and the call and the decrement are inside the braces,
    // so they run once per iteration, in the order the bytecode ran them.
    assert!(text.contains("while (local1 != 0) {"), "{text}");
    let cond_at = text.find("while (local1 != 0)").expect("the test");
    let call_at = text.find("local2.run(0L);").expect("the body's call");
    let dec_at = text
        .find("local1 = local1 - 1;")
        .expect("the body's decrement");
    let close_at = text.rfind('}').expect("the loop's closing brace");
    assert!(
        cond_at < call_at && call_at < dec_at && dec_at < close_at,
        "the test comes first, then the call, then the decrement, all inside the loop:\n{text}"
    );
    assert_eq!(
        text.matches("local2.run(0L)").count(),
        1,
        "one call in the text: nothing hoisted it out of the loop and nothing duplicated it:\n{text}"
    );
    assert!(
        !text.contains("local2.run(0L);\n    while"),
        "the call is not written before the loop:\n{text}"
    );
    assert!(report.fallbacks.is_empty(), "{:?}", report.fallbacks);
    // The loop's own test BCI reaches the condition node and the statement.
    assert!(
        report.text_of_bci(5).len() >= 2,
        "the branch at BCI 5 tests the condition and is the statement: {:?}",
        report.text_of_bci(5)
    );
}

#[test]
fn a_do_while_loop_runs_its_body_before_the_test() {
    let class = test_class::single_method(52, 8, 3, DO_WHILE_LOOP);
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
    assert!(text.contains("do {"), "{text}");
    assert!(
        text.contains("} while (local1 != 0);"),
        "the test is the latch's, written after the body:\n{text}"
    );
    let body_at = text.find("local2.run(0L);").expect("the body's call");
    let test_at = text.find("} while (").expect("the test");
    assert!(
        body_at < test_at,
        "the body runs before the test is read:\n{text}"
    );
    assert!(report.fallbacks.is_empty(), "{:?}", report.fallbacks);
}

#[test]
fn a_tableswitch_becomes_one_arm_per_target_with_its_keys_and_default() {
    let class = test_class::single_method(52, 8, 3, TABLE_SWITCH);
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
    assert!(text.contains("switch (local1) {"), "{text}");
    let case0 = text.find("case 0:").expect("the first key");
    let case1 = text.find("case 1:").expect("the second key");
    let arm = text.find("int local2 = 1;").expect("the shared arm");
    let default = text.find("default:").expect("the no-match arm");
    let default_arm = text.find("local2 = 2;").expect("the default's code");
    assert!(
        case0 < case1 && case1 < arm && arm < default && default < default_arm,
        "two labels, their shared arm, then the default's:\n{text}"
    );
    assert_eq!(
        text.matches("case ").count(),
        2,
        "one label per key, and the keys that share a target are one arm:\n{text}"
    );
    // Every arm ends in a `break` of its own, at the arm's own indentation: a Java `case` falls
    // into the case that follows it, so an arm that does not say `break` runs the next arm's code.
    // (The last arm's `break` is redundant and kept anyway: the rule is the rule for every arm.)
    assert_eq!(
        text.matches("break;").count(),
        2,
        "one break per arm, in the arms' own bodies:\n{text}"
    );
    for line in text.lines().filter(|line| line.contains("break;")) {
        assert_eq!(
            line, "            break;",
            "the break is written with its arm:\n{text}"
        );
    }
    assert!(report.fallbacks.is_empty(), "{:?}", report.fallbacks);
}

#[test]
fn a_graph_that_is_not_reducible_is_quoted_whole_with_its_own_reason() {
    let class = test_class::single_method(52, 8, 2, IRREDUCIBLE);
    let payload = analyze(&class, b"method", b"()V");
    let facts = facts_of(&class, b"method", 0, Vec::new());
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, &mut budget);
    assert!(
        report.produced(),
        "the body is presentable as bytecode, not dropped: {:?}",
        report.outcome
    );
    assert_eq!(report.representation, Representation::Mixed);
    assert_eq!(report.quality, Quality::Fallback);
    assert_ne!(
        report.quality,
        Quality::Structured,
        "a body the walk could not structure is never reported as structured"
    );
    assert!(
        report.text.contains("// @bytecode"),
        "{}\nregions: {:?}",
        report.text,
        report.regions
    );
    // The scan itself completed: a quoted region is a *result*, not a partial one.
    assert!(
        matches!(
            report.execution,
            jarde_reader::model::ExecutionReport::Complete { .. }
        ),
        "the scan completed and said so: {:?}",
        report.execution
    );
    assert_eq!(
        report.fallbacks,
        vec!["jre_region_irreducible"],
        "{:?}",
        report.regions
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_region_irreducible"),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn crossing_exception_records_are_quoted_with_the_priority_the_table_states() {
    let class = crossing_exception_class();
    let payload = analyze(&class, b"method", b"()V");
    let facts = facts_of(&class, b"method", 0, Vec::new());
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, &mut budget);
    assert!(
        report.produced(),
        "the body is quoted, not dropped: {:?}",
        report.outcome
    );
    assert_eq!(report.representation, Representation::Mixed);
    assert_eq!(report.quality, Quality::Fallback);
    assert_eq!(
        report.fallbacks,
        vec!["jre_region_crossing_exception_regions"],
        "{:?}",
        report.regions
    );
    assert!(report.text.contains("// @bytecode"), "{}", report.text);
    // The two records the reason names are the table's own, in table order: record 0 is the one a
    // throw inside both ranges reaches first, and it is named first.
    let message = report
        .regions
        .iter()
        .find_map(|region| region.message.clone())
        .expect("the fallback states why");
    assert!(
        message.contains("records 0 and 1"),
        "the priority is the table's order: {message}"
    );
    assert!(
        matches!(
            report.execution,
            jarde_reader::model::ExecutionReport::Complete { .. }
        ),
        "a quoted region is not a partial scan: {:?}",
        report.execution
    );
}

#[test]
fn a_loop_whose_header_writes_state_is_quoted_rather_than_hoisted() {
    // `local1 = 2; while (local1 != 0) { local1 = local1 - 1; } return;` written the way a compiler
    // writes it when the *test itself* is where the write belongs: the header loads, decrements and
    // stores before it branches.
    //
    // A `while (…)` has nowhere to put that store: hoisting it out of the loop would run it once
    // instead of once per iteration, and putting it in the body would run it *after* the test. So
    // the loop is quoted — the invariant is kept by refusing, not by moving the effect.
    const HEADER_WRITE: &[u8] = &[
        0x05, // 0: iconst_2
        0x3c, // 1: istore_1
        0x1b, // 2: iload_1       (the header)
        0x04, // 3: iconst_1
        0x64, // 4: isub
        0x3c, // 5: istore_1      (the write the header makes, once per test)
        0x1b, // 6: iload_1
        0x99, 0x00, 0x06, // 7: ifeq 13
        0xa7, 0xff, 0xf8, // 10: goto 2
        0xb1, // 13: return
    ];
    let class = test_class::single_method(52, 4, 2, HEADER_WRITE);
    let payload = analyze(&class, b"method", b"()V");
    let facts = facts_of(&class, b"method", 0, Vec::new());
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, &mut budget);
    assert!(report.produced(), "{:?}", report.outcome);
    let text = &report.text;
    assert!(
        !text.contains("while"),
        "the write in the test block is not presented as a hoisted statement:\n{text}"
    );
    assert!(text.contains("// @bytecode"), "{text}");
    assert_eq!(
        report.fallbacks.first().copied(),
        Some("jre_region_unmet_precondition"),
        "{:?}\n{}",
        report.regions,
        text
    );
    // The refusal is stated as a *declared* precondition of a named rule, not as a shape that
    // happens to look right: the record cites the rule and its version, and the message says which
    // requirement fell short and at which instruction.
    let record = report
        .regions
        .iter()
        .find(|region| region.code == Some("jre_region_unmet_precondition"))
        .expect("the refused loop is recorded");
    assert_eq!(
        record.rule,
        Some(jarde_java::pass::LOOP.rule()),
        "the record names the rule that refused it, not a shape that was produced"
    );
    let message = record.message.as_deref().expect("the refusal states why");
    assert!(message.contains("loop@1"), "{message}");
    assert!(message.contains("value expression"), "{message}");
    assert!(message.contains("BCI 5"), "{message}");
    assert!(
        report.diagnostics.iter().any(|diagnostic| diagnostic.code
            == "jre_region_unmet_precondition"
            && diagnostic.message.contains("loop@1")),
        "{:?}",
        report.diagnostics
    );
    assert!(
        report.rules.contains(&jarde_java::pass::LOOP.rule()),
        "which rule the record is answerable to is in the report: {:?}",
        report.rules
    );
    // A refused shape is not a failed scan: the walk finished, and which plane says what is stated
    // separately (A13).
    assert!(matches!(
        report.execution,
        jarde_reader::model::ExecutionReport::Complete { .. }
    ));
    assert_eq!(report.representation, Representation::Mixed);
    assert_eq!(report.quality, Quality::Fallback);
}

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
    // A09's half of this slice: the historical `jsr`/`finally` body is **never** reported as a
    // success and never as a structure that merely looks like the source. The run finished (the
    // scan is complete, so the planes do not read `Partial`), the artifact is quoted bytecode, the
    // text claims no `try`/`finally`, and every reason is statable. Recovering the `finally`/`jsr`
    // shape itself is 2.4's, and this test does not claim any of it.
    assert!(
        matches!(
            report.execution,
            jarde_reader::model::ExecutionReport::Complete { .. }
        ),
        "a quoted body is not a partial scan: {:?}",
        report.execution
    );
    for claimed in ["try {", "} finally", "} catch"] {
        assert!(
            !report.text.contains(claimed),
            "the text claims no shape the subset did not prove (`{claimed}`):\n{}",
            report.text
        );
    }
    assert!(
        report
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.is_empty()),
        "{:?}",
        report.diagnostics
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("BCI")),
        "at least one reason names where it could not prove: {:?}",
        report.diagnostics
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
        &RecoveryRequest::new(analysis.ir(), &facts, jarde_java::pass::JAVA_8),
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

#[test]
fn a_body_without_debug_names_is_named_deterministically_and_invents_no_source_scope() {
    // A10's half: the fixture's `Code` carries no `LocalVariableTable`, so this run has no source
    // spelling for any slot. What it writes must be the body's own ordinals — the same ones on
    // every run — and the artifact must not claim a scope, a line or a name the class file never
    // stated. The same bytes with debug evidence are presented afterwards, which is how "the names
    // came from the evidence" is shown to be a statement about the evidence and not about the
    // body.
    let class = test_class::single_method(52, 1, 3, IF_ELSE);
    let payload = analyze(&class, b"method", b"()V");
    let facts = facts_of(&class, b"method", 0, Vec::new());

    let first = {
        let mut budget = Budget::new(limits());
        recover_body(&payload, &facts, &mut budget)
    };
    let second = {
        let mut budget = Budget::new(limits());
        recover_body(&payload, &facts, &mut budget)
    };
    assert_eq!(
        first, second,
        "two runs of one request are one report, field by field"
    );
    assert!(
        first.text.contains("local1") && first.text.contains("local2"),
        "the body's slots are named by their ordinals:\n{}",
        first.text
    );
    assert!(
        !first.text.contains("arg"),
        "the body has no parameter slots, so nothing is named as one:\n{}",
        first.text
    );
    assert!(
        !first.text.to_lowercase().contains("line"),
        "no source position is claimed:\n{}",
        first.text
    );

    // No fabricated scope: every anchor the segment table states is a BCI of this body, and none of
    // them names a constant-pool entry or an attribute this run never read (`cp` stays `None` until
    // 3.2 fills the CP/attribute half of the map).
    assert!(!first.source_map.is_empty(), "{:?}", first.source_map);
    for segment in first.source_map.segments() {
        let origin = segment.origin();
        assert_eq!(origin.primary().cp(), None, "{:?}", origin.primary());
        for derived in origin.derived() {
            assert_eq!(derived.cp(), None, "{derived:?}");
        }
        assert!(
            !origin.bcis().is_empty(),
            "a segment stands for at least one BCI: {origin:?}"
        );
    }

    // The contrast: the same bytes with debug names spelled by the class file are presented with
    // those names, so the ordinal names above are the *absence* of evidence and not a preference.
    let named = facts_of(
        &class,
        b"method",
        0,
        vec![
            Some("receiver".to_string()),
            Some("left".to_string()),
            Some("right".to_string()),
        ],
    );
    let mut budget = Budget::new(limits());
    let named_report = recover_body(&payload, &named, &mut budget);
    assert!(
        named_report.text.contains("left") && named_report.text.contains("right"),
        "the evidence is what names a slot:\n{}",
        named_report.text
    );
    assert!(
        !named_report.text.contains("local1"),
        "the ordinal name is gone once the evidence is there:\n{}",
        named_report.text
    );
    assert!(
        !first.text.contains("left"),
        "a run with no evidence cannot present a spelling it never read:\n{}",
        first.text
    );
    assert_eq!(first.method, named_report.method);
    assert_eq!(
        first.representation, named_report.representation,
        "which slot a name belongs to does not change what the artifact is made of"
    );
}

#[test]
fn a_member_that_cannot_be_presented_leaves_the_member_that_can_alone() {
    // A13's member half: one class file with a member the subset presents and a member it refuses.
    // Each is its own request over its own payload, and neither report may carry the other's shape,
    // stop or diagnostics — that isolation is what lets a caller read one member's result without
    // reading the whole class.
    let presentable = analyze(HISTORICAL_V45, b"add", b"(II)I");
    let refused = analyze(HISTORICAL_V45, b"finallyPath", b"(I)I");
    let add_facts = facts_of(HISTORICAL_V45, b"add", 3, Vec::new());
    let finally_facts = facts_of(HISTORICAL_V45, b"finallyPath", 1, Vec::new());

    let add = {
        let mut budget = Budget::new(limits());
        recover_body(&presentable, &add_facts, &mut budget)
    };
    assert!(add.produced(), "{:?}", add.outcome);
    assert!(add.fallbacks.is_empty(), "{:?}", add.regions);
    assert_eq!(add.method, "add(II)I");
    assert_eq!(add.representation, Representation::Java);
    assert_eq!(add.quality, Quality::Structured);

    let finally = {
        let mut budget = Budget::new(limits());
        recover_body(&refused, &finally_facts, &mut budget)
    };
    assert!(finally.produced(), "{:?}", finally.outcome);
    assert_eq!(finally.method, "finallyPath(I)I");
    assert_eq!(finally.representation, Representation::Mixed);
    assert_eq!(finally.quality, Quality::Fallback);
    assert!(!finally.fallbacks.is_empty(), "{:?}", finally.regions);

    // Each report is about its own member and only it: no BCI, no diagnostic and no region of one
    // appears in the other, whichever order they are asked in.
    assert!(!add.text.contains("finallyPath"), "{}", add.text);
    assert!(
        add.diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.message.contains("finallyPath")),
        "{:?}",
        add.diagnostics
    );
    assert!(!finally.text.contains("add(II)I"), "{}", finally.text);
    assert!(
        add.fallbacks.is_empty() && !finally.fallbacks.is_empty(),
        "the two members' outcomes are their own: {:?} vs {:?}",
        add.fallbacks,
        finally.fallbacks
    );

    // Re-asking each member after the other is the same report, field by field: a refusal (or a
    // presentation) elsewhere changed nothing here, in either order.
    let add_again = {
        let mut budget = Budget::new(limits());
        recover_body(&presentable, &add_facts, &mut budget)
    };
    assert_eq!(add, add_again);
    let finally_again = {
        let mut budget = Budget::new(limits());
        recover_body(&refused, &finally_facts, &mut budget)
    };
    assert_eq!(finally, finally_again);
}

#[test]
fn execution_quality_and_representation_each_state_their_own_thing() {
    // A13's plane half. `representation` says what the artifact is made of, `quality` says how
    // strong the recovered structure is, `execution` says how the run ended — and no plane borrows
    // another's vocabulary. The same body is run under three budgets so that the planes can be
    // seen to move independently.
    let class = test_class::single_method(52, 8, 2, STRAIGHT_LINE);
    let payload = analyze(&class, b"method", b"()V");
    let facts = facts_of(&class, b"method", 0, Vec::new());

    let produced = {
        let mut budget = Budget::new(limits());
        recover_body(&payload, &facts, &mut budget)
    };
    assert!(produced.produced());
    assert_eq!(produced.representation, Representation::Java);
    assert_eq!(produced.quality, Quality::Structured);
    assert!(matches!(
        produced.execution,
        jarde_reader::model::ExecutionReport::Complete { .. }
    ));

    // One byte short of the text this very run produced: the artifact is refused, and the planes
    // split the news the way the spec separates them — the execution plane states the stop, while
    // quality goes on describing what there is (nothing strong) and representation what it is made
    // of. `quality` is never rewritten to an execution value (`Partial` is not a quality).
    let exact = u64::try_from(produced.text.len()).expect("the text fits u64");
    let mut over = Budget::new(Limits {
        output_bytes: exact - 1,
        ..limits()
    });
    let stopped = recover_body(&payload, &facts, &mut over);
    assert!(!stopped.produced(), "{:?}", stopped.outcome);
    assert_eq!(stopped.text, "");
    assert!(stopped.source_map.is_empty());
    assert!(stopped.regions.is_empty() && stopped.fallbacks.is_empty());
    assert_eq!(stopped.representation, Representation::Bytecode);
    assert_eq!(
        stopped.quality,
        Quality::Fallback,
        "`Fallback` here means 'no artifact was produced', and the stop itself is the execution plane's statement"
    );
    assert!(matches!(
        stopped.execution,
        jarde_reader::model::ExecutionReport::Partial {
            reason: jarde_reader::model::TerminationReason::BudgetExceeded { .. },
            ..
        }
    ));
    assert_eq!(
        stopped
            .diagnostics
            .first()
            .map(|diagnostic| diagnostic.code.as_str()),
        Some("jre_output_budget")
    );

    // The other direction, on a body whose region cannot be proved: the run *completed* — the walk
    // visited the whole method — while the artifact is quoted bytecode and its quality is weak. A
    // weak artifact is therefore not a partial scan, and a partial scan is not what quality says.
    let irreducible = test_class::single_method(52, 1, 3, IRREDUCIBLE);
    let irreducible_payload = analyze(&irreducible, b"method", b"()V");
    let irreducible_facts = facts_of(&irreducible, b"method", 0, Vec::new());
    let mut budget = Budget::new(limits());
    let quoted = recover_body(&irreducible_payload, &irreducible_facts, &mut budget);
    assert!(quoted.produced(), "{:?}", quoted.outcome);
    assert_eq!(quoted.representation, Representation::Mixed);
    assert_eq!(quoted.quality, Quality::Fallback);
    assert!(
        matches!(
            quoted.execution,
            jarde_reader::model::ExecutionReport::Complete { .. }
        ),
        "a quoted body is not a partial scan: {:?}",
        quoted.execution
    );
    assert!(quoted.text.contains("// @bytecode"), "{}", quoted.text);

    // And the diagnostics of a stopped run are its own: the stopped run's error never appears in the
    // produced one's report, in either order.
    assert!(
        produced
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "jre_output_budget"),
        "{:?}",
        produced.diagnostics
    );
}
