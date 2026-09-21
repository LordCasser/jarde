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
    MethodFacts, RecoveryFacts, RecoveryOutcome, RecoveryReport, RecoveryRequest, StopReason,
    escape_string, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{
    AnalysisStage, CompileStatus, MethodAnalysisRequest, Quality, Representation,
    SemanticValidation, SyntaxStatus,
};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::UsageSnapshot;
use jarde_reader::budget::{Budget, CancellationToken, Limits};
use jarde_reader::classfile::VerificationStatus;
use jarde_reader::classfile::{class_facts, method_code_facts, test_class};
use jarde_reader::model::ExecutionReport;
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

/// The same class compiled by the same compiler at class-file version **52** (P3-R7): its
/// `finallyPath(I)I` is the committed body whose decode and canonical graph disagree — the class
/// declares an `any` handler at BCI 9 over `[0, 4)` and the decode reads BCI 9's four instructions,
/// while the graph's only block is `[0, 9)` and lists no dead node. Two `finally` codegens of one
/// compiler, one body each: `v45` keeps every instruction of the handler in a node, `v52` keeps none
/// of them.
const HISTORICAL_V52: &[u8] =
    include_bytes!("../../../tests/fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

/// The `p3-scope` sample's body, compiled by javac 23.0.1 `--release 8 -g`: `reuse(ZI)I` reuses slot
/// 3 for the `then` arm's `c` and the `else` arm's `d`, and its `LocalVariableTable` is the evidence
/// P3 3.4 splits the slot from. This file states that evidence itself (see
/// `a_slot_the_records_cannot_place_is_one_variable_with_the_ordinal_name`), so the bytes are here
/// for the *body*, and the table javac really wrote is asserted where the whole run is driven
/// (`tests/p3_scope.rs`, which reads the LVT through the entry point).
const SCOPE_DEBUG: &[u8] = include_bytes!("../../../tests/fixtures/p3-scope/v8-debug/Scope.class");

/// One usage snapshot with the wall clock removed: the comparison form of two reads of one budget.
///
/// `elapsed_millis` is a measurement, not a charge: `Budget::usage` takes it again on every read,
/// so two runs of one request can differ by a millisecond while every counted dimension is
/// identical. The repository compares reports this way everywhere else (`tests/p2_contracts.rs`
/// states the same pair), and a comparison that does not is a test that fails on the clock rather
/// than on the engine — which is what this file's determinism case did before.
fn counted_usage(usage: &UsageSnapshot) -> UsageSnapshot {
    UsageSnapshot {
        elapsed_millis: 0,
        ..usage.clone()
    }
}

/// One execution report compared with that one measurement removed from its usage.
fn without_wall_clock(execution: &ExecutionReport) -> ExecutionReport {
    match execution {
        ExecutionReport::Complete { usage } => ExecutionReport::Complete {
            usage: counted_usage(usage),
        },
        ExecutionReport::Partial { reason, usage } => ExecutionReport::Partial {
            reason: reason.clone(),
            usage: counted_usage(usage),
        },
        ExecutionReport::Cancelled { usage } => ExecutionReport::Cancelled {
            usage: counted_usage(usage),
        },
        ExecutionReport::Failed { reason, usage } => ExecutionReport::Failed {
            reason: reason.clone(),
            usage: counted_usage(usage),
        },
    }
}

/// The wall clock one report's execution carried, whichever outcome it states.
fn elapsed_of(report: &RecoveryReport) -> u64 {
    usage_of(report).elapsed_millis
}

/// The usage snapshot one report's execution carried.
fn usage_of(report: &RecoveryReport) -> UsageSnapshot {
    match &report.execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage.clone(),
    }
}

/// One report as two runs of it compare: identical but for the wall clock.
fn without_elapsed(report: &RecoveryReport) -> RecoveryReport {
    RecoveryReport {
        execution: without_wall_clock(&report.execution),
        ..report.clone()
    }
}

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
        roots: vec![LoadRoot::StandaloneClass {
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
    .with_debug_locals(stated_names(debug))
}

/// The debug records of a test that states **one name per slot**: each name covers its slot with no
/// range stated, so no slot is ever split for them (P3 3.4).
fn stated_names(debug: Vec<Option<String>>) -> Vec<jarde_java::DebugLocal> {
    debug
        .into_iter()
        .enumerate()
        .filter_map(|(slot, name)| {
            name.map(|name| {
                jarde_java::DebugLocal::named(u16::try_from(slot).unwrap_or(u16::MAX), name)
            })
        })
        .collect()
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
    let arm = text.find("local2 = 1;").expect("the shared arm");
    let default = text.find("default:").expect("the no-match arm");
    let default_arm = text.find("local2 = 2;").expect("the default's code");
    // P3 3.1: the slot both the shared arm and the default write is declared where both can see it —
    // at the start of the region that holds the switch — and each write is a plain assignment.
    let declaration = text
        .find("int local2;")
        .expect("the declaration is hoisted to the switch's own region");
    let switch_at = text.find("switch (local1) {").expect("the selector");
    assert!(
        declaration < switch_at,
        "the declaration precedes every arm that writes the slot:\n{text}"
    );
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
    let then_arm = text.find("local2 = 1;").expect("the fall-through arm");
    let else_arm = text.find("local2 = 2;").expect("the branch arm");
    // P3 3.1: the slot is written in both arms, so a declaration inside either one is out of scope in
    // the other. It is declared where both can see it — before the `if` — and both writes assign.
    let declaration = text
        .find("int local2;")
        .expect("the declaration is hoisted above the `if`");
    assert!(
        declaration < if_at,
        "the declaration precedes the region that holds both writes:\n{text}"
    );
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

/// P3-R7: the very body of the finding, driven through the entry point on the committed fixture.
///
/// `finallyPath(I)I` of the ECJ 4.6.1 **v52** class is the one committed body whose decode and
/// canonical graph disagree: the class declares `Exception table: from 0 to 4 target 9 type any`,
/// the decode reads eleven instructions (BCI 9's `astore_2`, 10's `iinc`, 13's `aload_2` and 14's
/// `athrow` among them), and the graph holds a single block `[0, 9)` with nothing dead. Before this
/// change the run presented the three statements of `[0, 8)` under `java`/`structured` and never
/// mentioned BCI 9/10/13/14 — a reader could not tell "judged dead" from "never seen".
///
/// What this test states is the invariant, not the shape of one message: every decoded instruction
/// of the body is either covered by a canonical block or named as unreachable, **or** the body is
/// refused whole under a reason that quotes every instruction the graph failed to account for.
#[test]
fn an_instruction_no_block_covers_refuses_the_body_it_belongs_to() {
    let payload = analyze(HISTORICAL_V52, b"finallyPath", b"(I)I");
    let facts = facts_of(
        HISTORICAL_V52,
        b"finallyPath",
        3,
        vec![None, Some("arg1".to_string()), None, None],
    );
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, &mut budget);

    // The two facts the refusal is about, read independently of the run's own tables: the bytes
    // decode to eleven instruction starts and declare one `any` handler at BCI 9 over `[0, 4)`.
    let mut decode_budget = Budget::new(limits());
    let header =
        class_facts(HISTORICAL_V52, &mut decode_budget).expect("the fixture is a class file");
    let member = header
        .methods
        .iter()
        .find(|member| member.name.raw().0 == b"finallyPath")
        .expect("the fixture declares finallyPath");
    let code = method_code_facts(HISTORICAL_V52, member, &mut decode_budget).expect("it decodes");
    let starts: Vec<u32> = code
        .instructions
        .iter()
        .map(|instruction| instruction.bci)
        .collect();
    assert_eq!(
        starts,
        vec![0, 1, 2, 3, 4, 7, 8, 9, 10, 13, 14],
        "the class file's own decode of the body"
    );
    assert_eq!(
        code.exception_handlers
            .iter()
            .map(|handler| (handler.start_bci, handler.end_bci, handler.handler_bci))
            .collect::<Vec<(u32, u32, u32)>>(),
        vec![(0, 4, 9)],
        "the class file's own exception table"
    );

    // The graph's account of the same body, computed from the payload's own tables: an instruction
    // is accounted for by a block's half-open span or by a dead node, and nothing else.
    let unaccounted = unaccounted_of(&payload);
    assert_eq!(
        unaccounted,
        vec![9, 10, 13, 14],
        "the handler's four instructions are in no block and in no dead node:\n{}",
        describe(&payload)
    );

    // The product: the whole body is quoted under the reason that names all four, with an anchor for
    // every quoted BCI, and no statement survives that the graph cannot account for.
    assert_eq!(report.representation, Representation::Mixed);
    assert_eq!(report.quality, Quality::Fallback);
    assert_eq!(report.syntax_status, SyntaxStatus::NotJava);
    assert_eq!(
        report.fallbacks,
        vec!["jre_region_unaccounted_instruction"],
        "{:?}",
        report.regions
    );
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "jre_region_unaccounted_instruction")
        .expect("the refusal is diagnosed");
    for bci in &unaccounted {
        assert!(
            diagnostic.message.contains(&bci.to_string()),
            "the reason names BCI {bci}: {}",
            diagnostic.message
        );
    }
    assert_eq!(
        quoted_bcis(&report),
        vec![0, 9, 10, 13, 14],
        "the quote names the blocks it refuses *and* every instruction the graph missed:\n{}",
        report.text
    );
    for bci in quoted_bcis(&report) {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "quoted BCI {bci} has an anchor:\n{}",
            report.text
        );
    }
    for presented in ["int local3", "return local3", "arg1 = arg1 + 2"] {
        assert!(
            !report.text.contains(presented),
            "the body the graph cannot account for is not presented (`{presented}`):\n{}",
            report.text
        );
    }
    assert!(
        !report.text.is_empty(),
        "and it is not emptied either: {:?}",
        report.outcome
    );

    // The member the graph *does* account for is untouched by the check: one class file, one body
    // refused and the other still presented.
    let add = analyze(HISTORICAL_V52, b"add", b"(II)I");
    let add_facts = facts_of(HISTORICAL_V52, b"add", 3, Vec::new());
    let mut add_budget = Budget::new(limits());
    let add_report = recover_body(&add, &add_facts, &mut add_budget);
    assert_eq!(add_report.representation, Representation::Java);
    assert_eq!(add_report.quality, Quality::Structured);
    assert!(add_report.fallbacks.is_empty(), "{:?}", add_report.regions);
    assert!(
        add_report.text.contains("return arg1 + arg2;"),
        "{}",
        add_report.text
    );
}

/// The other half of P3-R7: an instruction a **dead node** holds is accounted for.
///
/// The `v45` body of the same class file and the same compiler keeps the handler inside its graph —
/// the `jsr` normalization creates a node for every call context, so the handler's four instructions
/// are a node the entry cannot reach. That is a statement about them ("dead, and the run can say
/// why"), not the silence the v52 graph leaves, so the reason this test's sibling states must **not**
/// fire here: the check refuses a body the graph has no account of, not one it accounts for.
#[test]
fn a_handlers_instructions_a_dead_node_holds_are_accounted_for() {
    let payload = analyze(HISTORICAL_V45, b"finallyPath", b"(I)I");
    let facts = facts_of(HISTORICAL_V45, b"finallyPath", 1, Vec::new());
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, &mut budget);

    assert!(
        unaccounted_of(&payload).is_empty(),
        "the v45 graph accounts for every instruction it decoded:\n{}",
        describe(&payload)
    );
    assert!(
        !report
            .fallbacks
            .contains(&"jre_region_unaccounted_instruction"),
        "a body the graph accounts for is refused for its own reasons, not this one: {:?}",
        report.regions
    );
}

/// The instruction starts one payload's decode produced that its canonical graph does not account
/// for: neither in any block's half-open span nor in a node the graph lists as dead (P3-R7).
fn unaccounted_of(payload: &Payload) -> Vec<u32> {
    let ir = payload.analysis.ir();
    let code = ir.code().expect("the body decoded");
    let canonical = ir.canonical().expect("the graph is published");
    let mut covered = std::collections::BTreeSet::new();
    for block in canonical.blocks() {
        let start = block
            .blocks()
            .first()
            .copied()
            .unwrap_or_else(|| block.id().bci());
        covered.extend(start..block.end_bci());
    }
    for id in canonical.unreachable() {
        covered.insert(id.bci());
        if let Some(block) = canonical.blocks().iter().find(|block| block.id() == id) {
            let start = block.blocks().first().copied().unwrap_or_else(|| id.bci());
            covered.extend(start..block.end_bci());
        }
    }
    code.instructions
        .iter()
        .map(|instruction| instruction.bci)
        .filter(|bci| !covered.contains(bci))
        .collect()
}

/// Every BCI the artifact's `// @bytecode` quotes name, in the order the text names them.
fn quoted_bcis(report: &RecoveryReport) -> Vec<u32> {
    report
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
        .collect()
}

#[test]
fn a_committed_straight_line_body_is_presented_from_its_own_decode() {
    // The committed corpus, not an assembled fixture: `add(II)I` is `iload_1; iload_2; iadd;
    // ireturn`, and the operations come from the real decode's own BCIs, so nothing in this test
    // knows where the instructions are. `add` is an instance method, so its three parameter slots are
    // `this`, `left` and `right`: slot 0 has no name of its own here because a receiver is named from
    // a declaration fact this file's facts do not state (the member's own flags), and an ordinal name
    // is the deterministic answer without one — the naming layer writes `this` for slot 0 exactly
    // when the flags say the member takes a receiver (`MethodFacts::has_receiver`).
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
        roots: vec![LoadRoot::StandaloneClass {
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
fn a_slot_the_records_cannot_place_is_one_variable_with_the_ordinal_name() {
    // P3 3.4's other half, and the reason the split needs more than two records: a slot the debug
    // table names twice is split **only** when the body's own uses say which variable each use
    // belongs to. Two records whose ranges overlap state no such thing — there is no bytecode range
    // in which the slot carried one and then the other — so neither name is written, and the slot
    // stays the one variable P3 3.1 wrote, declared where every use of it can see it.
    //
    // The body is the committed `reuse(ZI)I`; the evidence is stated **here** rather than read from
    // the class, because the shapes that decline the split are exactly the ones a compiler does not
    // emit: javac's own table is asserted where the whole run is driven (`tests/p3_scope.rs`). Both
    // declining shapes are in the evidence below:
    //
    // * slot 3 carries `c` and `d`, and their ranges `[0, 12)` and `[8, 21)` **overlap**;
    // * slot 2 carries `a` twice with the **same** name over `[10, 13)` and `[19, 21)`, which is one
    //   variable and keeps its name — the P3-R3 shape, not a reuse.
    let payload = analyze(SCOPE_DEBUG, b"reuse", b"(ZI)I");
    let facts = RecoveryFacts::new(MethodFacts::new("reuse", "(ZI)I", 2)).with_debug_locals(vec![
        jarde_java::DebugLocal::over(0, "b", 0, 21),
        jarde_java::DebugLocal::over(1, "seed", 0, 21),
        jarde_java::DebugLocal::over(2, "a", 10, 13),
        jarde_java::DebugLocal::over(2, "a", 19, 21),
        jarde_java::DebugLocal::over(3, "c", 0, 12),
        jarde_java::DebugLocal::over(3, "d", 8, 21),
    ]);
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, &mut budget);

    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(report.representation, Representation::Java, "{report:?}");
    assert_eq!(report.quality, Quality::Structured);
    assert!(
        report.text.trim_end().ends_with(
            "{\n    int a;\n    int local3;\n    if (b) {\n        local3 = seed + 1;\n        a = local3;\n    } else {\n        local3 = seed + 2;\n        a = local3;\n    }\n    return a;\n}"
        ),
        "the unplaceable slot is one variable with its ordinal name, and `a` — one name over two \
         records — keeps the declaration both arms and the join see:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("int c") && !report.text.contains("int d"),
        "neither unplaceable record's name is written:\n{}",
        report.text
    );
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
        without_elapsed(&first),
        without_elapsed(&second),
        "two runs of one request are one report, field by field and not counting the clock"
    );
    // Removing the clock must not be the same as removing the comparison: a report that differs in
    // anything else is still seen as different, and a report that differs only in the clock is not.
    let mut changed = first.clone();
    changed.text.push(' ');
    assert_ne!(
        without_elapsed(&first),
        without_elapsed(&changed),
        "the comparison sees a report that differs in its text"
    );
    assert_eq!(
        elapsed_of(&without_elapsed(&first)),
        0,
        "the comparison form carries no clock to compare"
    );
    let clocked = RecoveryReport {
        execution: ExecutionReport::Complete {
            usage: UsageSnapshot {
                elapsed_millis: 7,
                ..usage_of(&first)
            },
        },
        ..first.clone()
    };
    assert_eq!(
        without_elapsed(&first),
        without_elapsed(&clocked),
        "two reports differing only in the clock are one report"
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

// ---------------------------------------------------------------------------
// P3 2.1: the verified `LambdaMetafactory` shape (acceptance A04)
// ---------------------------------------------------------------------------

/// One class-file assembler for the lambda fixtures.
///
/// What it writes is the shape javac writes for a lambda site: a member whose body holds the
/// `invokedynamic`, the desugared body method beside it **as a real member**, and a
/// `BootstrapMethods` class attribute whose entry names the factory and its three static arguments.
/// Assembling the bytes here rather than committing a jar keeps every field of the fixture visible
/// in the test that depends on it — and the bytes are still a real class file, decoded by the real
/// reader and driven through the real engine.
///
/// The member names and indexes 1..8 are the fixture's own preamble (`Test`, its superclass,
/// `method`, `()V`, `Code`, `BootstrapMethods`); everything a site needs is added after them.
struct Fixture {
    pool: Vec<u8>,
    entries: u16,
    bootstraps: Vec<(u16, Vec<u16>)>,
    bodies: Vec<FixtureBody>,
}

/// One member the fixture declares beside `method` — the desugared body method a compiler generates
/// for the site, so the fixture's implementation handle names something the class really holds.
struct FixtureBody {
    flags: u16,
    name: u16,
    descriptor: u16,
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
}

impl Fixture {
    fn new() -> Self {
        let mut fixture = Self {
            pool: Vec::new(),
            entries: 0,
            bootstraps: Vec::new(),
            bodies: Vec::new(),
        };
        let test = fixture.utf8("Test"); // 1
        fixture.class(test); // 2
        let object = fixture.utf8("java/lang/Object"); // 3
        fixture.class(object); // 4
        fixture.utf8("method"); // 5
        fixture.utf8("()V"); // 6
        fixture.utf8("Code"); // 7
        fixture.utf8("BootstrapMethods"); // 8
        fixture
    }

    fn entry(&mut self, tag: u8, body: &[u8]) -> u16 {
        self.pool.push(tag);
        self.pool.extend_from_slice(body);
        self.entries += 1;
        self.entries
    }

    fn utf8(&mut self, text: &str) -> u16 {
        let bytes = text.as_bytes();
        let mut body = Vec::new();
        body.extend_from_slice(
            &u16::try_from(bytes.len())
                .expect("a fixture name fits u16")
                .to_be_bytes(),
        );
        body.extend_from_slice(bytes);
        self.entry(1, &body)
    }

    fn class(&mut self, name: u16) -> u16 {
        self.entry(7, &name.to_be_bytes())
    }

    fn integer(&mut self, value: i32) -> u16 {
        self.entry(3, &value.to_be_bytes())
    }

    fn name_and_type(&mut self, name: u16, descriptor: u16) -> u16 {
        let mut body = Vec::new();
        body.extend_from_slice(&name.to_be_bytes());
        body.extend_from_slice(&descriptor.to_be_bytes());
        self.entry(12, &body)
    }

    fn method_ref(&mut self, class: u16, name_and_type: u16) -> u16 {
        let mut body = Vec::new();
        body.extend_from_slice(&class.to_be_bytes());
        body.extend_from_slice(&name_and_type.to_be_bytes());
        self.entry(10, &body)
    }

    fn interface_method_ref(&mut self, class: u16, name_and_type: u16) -> u16 {
        let mut body = Vec::new();
        body.extend_from_slice(&class.to_be_bytes());
        body.extend_from_slice(&name_and_type.to_be_bytes());
        self.entry(11, &body)
    }

    fn method_handle(&mut self, kind: u8, reference: u16) -> u16 {
        let mut body = vec![kind];
        body.extend_from_slice(&reference.to_be_bytes());
        self.entry(15, &body)
    }

    fn method_type(&mut self, descriptor: u16) -> u16 {
        self.entry(16, &descriptor.to_be_bytes())
    }

    fn invoke_dynamic(&mut self, bootstrap: u16, name_and_type: u16) -> u16 {
        let mut body = Vec::new();
        body.extend_from_slice(&bootstrap.to_be_bytes());
        body.extend_from_slice(&name_and_type.to_be_bytes());
        self.entry(18, &body)
    }

    /// One `BootstrapMethods` entry, returning its index in the attribute.
    fn bootstrap(&mut self, handle: u16, arguments: Vec<u16>) -> u16 {
        self.bootstraps.push((handle, arguments));
        u16::try_from(self.bootstraps.len() - 1).expect("a fixture has few bootstraps")
    }

    /// Declares one member of the fixture class, with a body that satisfies its descriptor.
    fn body(&mut self, name: &str, descriptor: &str) {
        let name = self.utf8(name);
        let descriptor_index = self.utf8(descriptor);
        let (code, max_stack) = match descriptor.rsplit_once(')').map(|(_, returns)| returns) {
            Some("V") => (vec![0xb1], 0),
            Some("I") => (vec![0x03, 0xac], 1),
            _ => (vec![0x01, 0xb0], 1),
        };
        self.bodies.push(FixtureBody {
            flags: 0x000a, // private static
            name,
            descriptor: descriptor_index,
            max_stack,
            max_locals: parameter_slots(descriptor) + 1,
            code,
        });
    }

    /// The class file: `method()V` with `code`, every declared member, and the bootstrap attribute.
    ///
    /// `site_index` is the pool index of the site's own `InvokeDynamic` entry, which is written into
    /// the four bytes of the `invokedynamic` the caller placed at `site_bci` — the one place the
    /// fixture needs an index it cannot know while it is describing the site.
    fn finish(
        self,
        mut code: Vec<u8>,
        site_bci: usize,
        site_index: u16,
        max_stack: u16,
        max_locals: u16,
    ) -> Vec<u8> {
        assert_eq!(
            code[site_bci], 0xba,
            "the fixture's site must be at the BCI the caller names"
        );
        code[site_bci + 1..site_bci + 3].copy_from_slice(&site_index.to_be_bytes());
        let mut out = Vec::new();
        out.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
        out.extend_from_slice(&0_u16.to_be_bytes()); // minor
        out.extend_from_slice(&52_u16.to_be_bytes()); // major: Java 8
        out.extend_from_slice(&(self.entries + 1).to_be_bytes());
        out.extend_from_slice(&self.pool);
        out.extend_from_slice(&0x0021_u16.to_be_bytes()); // public super
        out.extend_from_slice(&2_u16.to_be_bytes()); // this_class → Test
        out.extend_from_slice(&4_u16.to_be_bytes()); // super_class → java/lang/Object
        out.extend_from_slice(&0_u16.to_be_bytes()); // interfaces
        out.extend_from_slice(&0_u16.to_be_bytes()); // fields
        out.extend_from_slice(
            &u16::try_from(self.bodies.len() + 1)
                .expect("a fixture has few methods")
                .to_be_bytes(),
        );
        out.extend_from_slice(&0x0009_u16.to_be_bytes()); // public static
        out.extend_from_slice(&5_u16.to_be_bytes()); // name → "method"
        out.extend_from_slice(&6_u16.to_be_bytes()); // descriptor → "()V"
        out.extend_from_slice(&1_u16.to_be_bytes()); // one attribute
        out.extend_from_slice(&7_u16.to_be_bytes()); // "Code"
        let mut attribute = Vec::new();
        attribute.extend_from_slice(&max_stack.to_be_bytes());
        attribute.extend_from_slice(&max_locals.to_be_bytes());
        attribute.extend_from_slice(
            &u32::try_from(code.len())
                .expect("a fixture body fits u32")
                .to_be_bytes(),
        );
        attribute.extend_from_slice(&code);
        attribute.extend_from_slice(&0_u16.to_be_bytes()); // exception table
        attribute.extend_from_slice(&0_u16.to_be_bytes()); // Code attributes
        out.extend_from_slice(
            &u32::try_from(attribute.len())
                .expect("a fixture attribute fits u32")
                .to_be_bytes(),
        );
        out.extend_from_slice(&attribute);
        for body in &self.bodies {
            out.extend_from_slice(&body.flags.to_be_bytes());
            out.extend_from_slice(&body.name.to_be_bytes());
            out.extend_from_slice(&body.descriptor.to_be_bytes());
            out.extend_from_slice(&1_u16.to_be_bytes());
            out.extend_from_slice(&7_u16.to_be_bytes()); // "Code"
            let mut attribute = Vec::new();
            attribute.extend_from_slice(&body.max_stack.to_be_bytes());
            attribute.extend_from_slice(&body.max_locals.to_be_bytes());
            attribute.extend_from_slice(
                &u32::try_from(body.code.len())
                    .expect("a fixture body fits u32")
                    .to_be_bytes(),
            );
            attribute.extend_from_slice(&body.code);
            attribute.extend_from_slice(&0_u16.to_be_bytes());
            attribute.extend_from_slice(&0_u16.to_be_bytes());
            out.extend_from_slice(
                &u32::try_from(attribute.len())
                    .expect("a fixture attribute fits u32")
                    .to_be_bytes(),
            );
            out.extend_from_slice(&attribute);
        }
        // The class attributes: `BootstrapMethods` alone, and only when a site named one.
        if self.bootstraps.is_empty() {
            out.extend_from_slice(&0_u16.to_be_bytes());
            return out;
        }
        out.extend_from_slice(&1_u16.to_be_bytes());
        out.extend_from_slice(&8_u16.to_be_bytes()); // "BootstrapMethods"
        let mut content = Vec::new();
        content.extend_from_slice(
            &u16::try_from(self.bootstraps.len())
                .expect("a fixture has few bootstraps")
                .to_be_bytes(),
        );
        for (handle, arguments) in &self.bootstraps {
            content.extend_from_slice(&handle.to_be_bytes());
            content.extend_from_slice(
                &u16::try_from(arguments.len())
                    .expect("a bootstrap has few arguments")
                    .to_be_bytes(),
            );
            for argument in arguments {
                content.extend_from_slice(&argument.to_be_bytes());
            }
        }
        out.extend_from_slice(
            &u32::try_from(content.len())
                .expect("the bootstrap attribute fits u32")
                .to_be_bytes(),
        );
        out.extend_from_slice(&content);
        out
    }
}

/// How many local slots one method descriptor's parameters occupy.
fn parameter_slots(descriptor: &str) -> u16 {
    let mut slots = 0u16;
    let mut chars = descriptor.trim_start_matches('(').chars();
    while let Some(character) = chars.next() {
        match character {
            ')' => break,
            '[' => continue,
            'J' | 'D' => slots += 2,
            _ => slots += 1,
        }
        if character == 'L' {
            for character in chars.by_ref() {
                if character == ';' {
                    break;
                }
            }
        }
    }
    slots
}

/// One lambda site, as the fixtures vary it.
struct Site<'a> {
    /// The site's own name and descriptor: for a lambda site, the SAM's name and the
    /// captures-then-interface descriptor.
    name: &'a str,
    descriptor: &'a str,
    /// The factory: `java/lang/invoke/LambdaMetafactory` for a verified site, anything else for the
    /// A04 counter-examples.
    factory_class: &'a str,
    factory_name: &'a str,
    /// The bootstrap's first static argument: the SAM method type.
    sam: &'a str,
    /// The implementation handle: owner, name, descriptor and method-handle kind.
    implementation: (&'a str, &'a str, &'a str, u8),
    /// The bootstrap's third static argument: the instantiated method type.
    instantiated: &'a str,
    /// The `altMetafactory` flag word, when the fixture states one.
    flags: Option<i32>,
}

/// Assembles one fixture class with one site and one body.
///
/// `code` holds the body with a `0xba 0x00 0x00` placeholder where the site is, and `site_bci` names
/// it; the pool index of the site's own entry is written into those two bytes.
fn assemble(
    site: &Site<'_>,
    code: Vec<u8>,
    site_bci: usize,
    max_stack: u16,
    max_locals: u16,
) -> Vec<u8> {
    let mut fixture = Fixture::new();
    let factory = fixture.utf8(site.factory_class);
    let factory_class = fixture.class(factory);
    let factory_name = fixture.utf8(site.factory_name);
    let factory_descriptor = fixture.utf8(
        "(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/CallSite;",
    );
    let factory_name_and_type = fixture.name_and_type(factory_name, factory_descriptor);
    let factory_ref = fixture.method_ref(factory_class, factory_name_and_type);
    let factory_handle = fixture.method_handle(6, factory_ref);
    let sam = fixture.utf8(site.sam);
    let sam_type = fixture.method_type(sam);
    let (owner, name, descriptor, kind) = site.implementation;
    let owner_text = fixture.utf8(owner);
    let owner_class = fixture.class(owner_text);
    let implementation_name = fixture.utf8(name);
    let implementation_descriptor = fixture.utf8(descriptor);
    let implementation_name_and_type =
        fixture.name_and_type(implementation_name, implementation_descriptor);
    let implementation_ref = if kind == 9 {
        fixture.interface_method_ref(owner_class, implementation_name_and_type)
    } else {
        fixture.method_ref(owner_class, implementation_name_and_type)
    };
    let implementation_handle = fixture.method_handle(kind, implementation_ref);
    let instantiated = fixture.utf8(site.instantiated);
    let instantiated_type = fixture.method_type(instantiated);
    let mut arguments = vec![sam_type, implementation_handle, instantiated_type];
    if let Some(value) = site.flags {
        arguments.push(fixture.integer(value));
    }
    let bootstrap = fixture.bootstrap(factory_handle, arguments);
    let site_name = fixture.utf8(site.name);
    let site_descriptor = fixture.utf8(site.descriptor);
    let site_name_and_type = fixture.name_and_type(site_name, site_descriptor);
    let site_index = fixture.invoke_dynamic(bootstrap, site_name_and_type);
    // The desugared body method the implementation handle names, when it is one of this class's own
    // members: the fixture then holds the member the handle refers to, exactly like a compiler's
    // output does.
    if owner == "Test" && name != "<init>" {
        fixture.body(name, descriptor);
    }
    fixture.finish(code, site_bci, site_index, max_stack, max_locals)
}

/// The verified `LambdaMetafactory` factory every positive fixture names.
const METAFACTORY: &str = "java/lang/invoke/LambdaMetafactory";

/// `()` → `Runnable`, with one `int` captured: `r = () -> Test.lambda$method$0(base);`
///
/// The shape javac writes for a lambda whose body only reads a captured value: the site's descriptor
/// is `(I)Ljava/lang/Runnable;` (one captured `int`, the interface it returns), the SAM method type
/// is `()V` (`Runnable.run`), and the implementation is the **static** body method javac generated
/// beside it, which the fixture declares like any other member.
fn capture_lambda_class() -> Vec<u8> {
    assemble(
        &Site {
            name: "run",
            descriptor: "(I)Ljava/lang/Runnable;",
            factory_class: METAFACTORY,
            factory_name: "metafactory",
            sam: "()V",
            implementation: ("Test", "lambda$method$0", "(I)V", 6),
            instantiated: "()V",
            flags: None,
        },
        vec![
            0x08, // 0: iconst_5
            0x3c, // 1: istore_1        local1 = 5
            0x1b, // 2: iload_1         the captured value
            0xba, 0x00, 0x00, 0x00, 0x00, // 3: invokedynamic
            0x4d, // 8: astore_2        local2 = the lambda
            0xb1, // 9: return
        ],
        3,
        1,
        3,
    )
}

/// The same shape with a SAM parameter as well: `op = x -> Test.lambda$method$0(base, x);`
///
/// `IntUnaryOperator.applyAsInt(int)int`: the site's descriptor is
/// `(I)Ljava/util/function/IntUnaryOperator;`, the SAM method type and the instantiated method type
/// are both `(I)I`, and the implementation is `Test.lambda$method$0(II)I` — the captured value first,
/// then the SAM's parameter, which is the order the presentation has to write.
fn capture_and_parameter_lambda_class() -> Vec<u8> {
    assemble(
        &Site {
            name: "applyAsInt",
            descriptor: "(I)Ljava/util/function/IntUnaryOperator;",
            factory_class: METAFACTORY,
            factory_name: "metafactory",
            sam: "(I)I",
            implementation: ("Test", "lambda$method$0", "(II)I", 6),
            instantiated: "(I)I",
            flags: None,
        },
        vec![
            0x08, // 0: iconst_5
            0x3c, // 1: istore_1        local1 = 5
            0x1b, // 2: iload_1         the captured value
            0xba, 0x00, 0x00, 0x00, 0x00, // 3: invokedynamic
            0x4d, // 8: astore_2
            0xb1, // 9: return
        ],
        3,
        1,
        3,
    )
}

/// Two captured locals, in the site's own order: `r = () -> Test.lambda$method$0(a, b);`
///
/// The shape javac writes for `Runnable r = () -> use(a, b);`: the site's descriptor names two
/// captured `int`s and the SAM takes none, so the presentation's argument list is the captures
/// alone — which is the one list whose *order* a mistake would silently reverse.
fn two_capture_lambda_class() -> Vec<u8> {
    assemble(
        &Site {
            name: "run",
            descriptor: "(II)Ljava/lang/Runnable;",
            factory_class: METAFACTORY,
            factory_name: "metafactory",
            sam: "()V",
            implementation: ("Test", "lambda$method$0", "(II)V", 6),
            instantiated: "()V",
            flags: None,
        },
        vec![
            0x08, // 0: iconst_5
            0x3c, // 1: istore_1        local1 = 5
            0x06, // 2: iconst_3
            0x3d, // 3: istore_2        local2 = 3
            0x1b, // 4: iload_1         the first capture
            0x1c, // 5: iload_2         the second capture
            0xba, 0x00, 0x00, 0x00, 0x00, // 6: invokedynamic
            0x4e, // 11: astore_3       local3 = the lambda
            0xb1, // 12: return
        ],
        6,
        2,
        4,
    )
}

/// A bound method reference: `r = local1::run;`
///
/// The implementation is `java/lang/Runnable.run()V` reached through an `invokeinterface` handle, and
/// the site captures exactly the receiver that handle's invocation needs — so the site *is* a
/// reference to the member and is written as one. (javac writes this shape for `local1::run`.)
fn bound_reference_class() -> Vec<u8> {
    assemble(
        &Site {
            name: "run",
            // The captured receiver is one of the *site's* parameters: `local1::run` captures the
            // receiver, so the descriptor a compiler writes names it.
            descriptor: "(Ljava/lang/Runnable;)Ljava/lang/Runnable;",
            factory_class: METAFACTORY,
            factory_name: "metafactory",
            sam: "()V",
            implementation: ("java/lang/Runnable", "run", "()V", 9),
            instantiated: "()V",
            flags: None,
        },
        vec![
            0x01, // 0: aconst_null
            0x4c, // 1: astore_1        local1 = null
            0x2b, // 2: aload_1         the receiver the site captures
            0xba, 0x00, 0x00, 0x00, 0x00, // 3: invokedynamic
            0x4d, // 8: astore_2
            0xb1, // 9: return
        ],
        3,
        1,
        3,
    )
}

/// An **unbound** method reference to a static member: `f = java.lang.String::valueOf;`
///
/// Nothing is captured, and the implementation is a static member of another class — the shape
/// javac writes for `Function<String,String> f = String::valueOf;`, whose SAM method type is the
/// erased `(Ljava/lang/Object;)Ljava/lang/Object;` and whose instantiated type is
/// `(Ljava/lang/String;)Ljava/lang/String;`.
fn static_reference_class() -> Vec<u8> {
    assemble(
        &Site {
            name: "apply",
            descriptor: "()Ljava/util/function/Function;",
            factory_class: METAFACTORY,
            factory_name: "metafactory",
            sam: "(Ljava/lang/Object;)Ljava/lang/Object;",
            implementation: (
                "java/lang/String",
                "valueOf",
                "(Ljava/lang/Object;)Ljava/lang/String;",
                6,
            ),
            instantiated: "(Ljava/lang/String;)Ljava/lang/String;",
            flags: None,
        },
        vec![
            0xba, 0x00, 0x00, 0x00, 0x00, // 0: invokedynamic
            0x4c, // 5: astore_1
            0xb1, // 6: return
        ],
        0,
        1,
        2,
    )
}

/// A constructor reference: `s = java.lang.Object::new;`
///
/// The implementation handle is a `REF_newInvokeSpecial` one, which the contract states for `Type::new`
/// sites: nothing is captured, the SAM takes no parameters, and the site is a reference to the
/// constructor itself.
fn constructor_reference_class() -> Vec<u8> {
    assemble(
        &Site {
            name: "get",
            descriptor: "()Ljava/util/function/Supplier;",
            factory_class: METAFACTORY,
            factory_name: "metafactory",
            sam: "()Ljava/lang/Object;",
            implementation: ("java/lang/Object", "<init>", "()V", 8),
            instantiated: "()Ljava/lang/Object;",
            flags: None,
        },
        vec![
            0xba, 0x00, 0x00, 0x00, 0x00, // 0: invokedynamic
            0x4c, // 5: astore_1
            0xb1, // 6: return
        ],
        0,
        1,
        2,
    )
}

/// A constructor handle reached **with a capture**: `s = () -> new Test(base);`
///
/// javac writes `Test::new` (the method-reference case below) when nothing is captured and generates
/// a body method when something is; a class file is free to name the constructor directly instead,
/// and this is that site: the same verified shape, written as a lambda whose body constructs the
/// member — the one writing that reaches the `new` node.
fn constructor_lambda_class() -> Vec<u8> {
    assemble(
        &Site {
            name: "get",
            descriptor: "(I)Ljava/util/function/Supplier;",
            factory_class: METAFACTORY,
            factory_name: "metafactory",
            sam: "()Ljava/lang/Object;",
            implementation: ("Test", "<init>", "(I)V", 8),
            instantiated: "()Ljava/lang/Object;",
            flags: None,
        },
        vec![
            0x08, // 0: iconst_5
            0x3c, // 1: istore_1        local1 = 5
            0x1b, // 2: iload_1         the captured value
            0xba, 0x00, 0x00, 0x00, 0x00, // 3: invokedynamic
            0x4d, // 8: astore_2
            0xb1, // 9: return
        ],
        3,
        1,
        3,
    )
}

/// **A04's counter-example**: the same three static arguments, a different factory.
///
/// `Test.myBootstrap` is a hand-written bootstrap of the shape a class file is free to have, and its
/// arguments are *exactly* the ones `metafactory` takes — so the only thing that makes this site not
/// a lambda is the factory itself. A presentation that matched on the argument shapes, on the site's
/// descriptor, or on the presence of a `MethodHandle` in the pool would print a lambda here; this
/// layer does not.
fn arbitrary_bootstrap_class() -> Vec<u8> {
    assemble(
        &Site {
            name: "run",
            descriptor: "(I)Ljava/lang/Runnable;",
            factory_class: "Test",
            factory_name: "myBootstrap",
            sam: "()V",
            implementation: ("Test", "lambda$method$0", "(I)V", 6),
            instantiated: "()V",
            flags: None,
        },
        vec![
            0x08, // 0: iconst_5
            0x3c, // 1: istore_1
            0x1b, // 2: iload_1
            0xba, 0x00, 0x00, 0x00, 0x00, // 3: invokedynamic
            0x4d, // 8: astore_2
            0xb1, // 9: return
        ],
        3,
        1,
        3,
    )
}

/// A verified factory whose implementation does **not** line up with its SAM: `(III)I` where the
/// site binds one capture and the SAM takes one parameter.
fn arity_mismatch_class() -> Vec<u8> {
    assemble(
        &Site {
            name: "applyAsInt",
            descriptor: "(I)Ljava/util/function/IntUnaryOperator;",
            factory_class: METAFACTORY,
            factory_name: "metafactory",
            sam: "(I)I",
            implementation: ("Test", "lambda$method$0", "(III)I", 6),
            instantiated: "(I)I",
            flags: None,
        },
        vec![
            0x08, // 0: iconst_5
            0x3c, // 1: istore_1
            0x1b, // 2: iload_1
            0xba, 0x00, 0x00, 0x00, 0x00, // 3: invokedynamic
            0x4d, // 8: astore_2
            0xb1, // 9: return
        ],
        3,
        1,
        3,
    )
}

/// A verified site whose captured value is a **call's result**: `Test.produce()`.
///
/// The value is on the stack when the site runs, and the call that produced it is a statement of its
/// own in the artifact — so writing it again inside the lambda body would run it a second time, and
/// again at a different moment. The declared `Replayable` precondition refuses the site instead.
fn unreplayable_capture_class() -> Vec<u8> {
    let mut fixture = Fixture::new();
    let produce = fixture.utf8("produce");
    let produce_descriptor = fixture.utf8("()I");
    let produce_name_and_type = fixture.name_and_type(produce, produce_descriptor);
    let produce_ref = fixture.method_ref(2, produce_name_and_type);
    fixture.body("produce", "()I");
    let factory = fixture.utf8(METAFACTORY);
    let factory_class = fixture.class(factory);
    let factory_name = fixture.utf8("metafactory");
    let factory_descriptor = fixture.utf8("()Ljava/lang/invoke/CallSite;");
    let factory_name_and_type = fixture.name_and_type(factory_name, factory_descriptor);
    let factory_ref = fixture.method_ref(factory_class, factory_name_and_type);
    let factory_handle = fixture.method_handle(6, factory_ref);
    let sam = fixture.utf8("(I)I");
    let sam_type = fixture.method_type(sam);
    let owner = fixture.utf8("Test");
    let owner_class = fixture.class(owner);
    let implementation = fixture.utf8("lambda$method$0");
    let implementation_descriptor = fixture.utf8("(II)I");
    let implementation_name_and_type =
        fixture.name_and_type(implementation, implementation_descriptor);
    let implementation_ref = fixture.method_ref(owner_class, implementation_name_and_type);
    let implementation_handle = fixture.method_handle(6, implementation_ref);
    let instantiated = fixture.utf8("(I)I");
    let instantiated_type = fixture.method_type(instantiated);
    fixture.body("lambda$method$0", "(II)I");
    let bootstrap = fixture.bootstrap(
        factory_handle,
        vec![sam_type, implementation_handle, instantiated_type],
    );
    let site_name = fixture.utf8("applyAsInt");
    let site_descriptor = fixture.utf8("(I)Ljava/util/function/IntUnaryOperator;");
    let site_name_and_type = fixture.name_and_type(site_name, site_descriptor);
    let site_index = fixture.invoke_dynamic(bootstrap, site_name_and_type);
    let code = vec![
        0xb8,
        (produce_ref >> 8) as u8,
        produce_ref as u8, // 0: invokestatic Test.produce:()I
        0xba,
        0x00,
        0x00,
        0x00,
        0x00, // 3: invokedynamic
        0x4d, // 8: astore_2
        0xb1, // 9: return
    ];
    fixture.finish(code, 3, site_index, 1, 3)
}

/// A verified factory with `altMetafactory`'s flag word set: markers/bridges/serializable.
fn alt_metafactory_flags_class() -> Vec<u8> {
    assemble(
        &Site {
            name: "run",
            descriptor: "(Ljava/lang/Runnable;)Ljava/lang/Runnable;",
            factory_class: METAFACTORY,
            factory_name: "altMetafactory",
            sam: "()V",
            implementation: ("java/lang/Runnable", "run", "()V", 9),
            instantiated: "()V",
            flags: Some(1),
        },
        vec![
            0x01, // 0: aconst_null
            0x4c, // 1: astore_1
            0x2b, // 2: aload_1
            0xba, 0x00, 0x00, 0x00, 0x00, // 3: invokedynamic
            0x4d, // 8: astore_2
            0xb1, // 9: return
        ],
        3,
        1,
        3,
    )
}

/// A verified site whose instance **nothing reads**: `invokedynamic; pop; return`.
///
/// The site's shape is the verified one — the capture, the SAM and the implementation all line up —
/// so nothing but the missing reader can be the reason it is quoted.
fn unconsumed_site_class() -> Vec<u8> {
    assemble(
        &Site {
            name: "run",
            descriptor: "(I)Ljava/lang/Runnable;",
            factory_class: METAFACTORY,
            factory_name: "metafactory",
            sam: "()V",
            implementation: ("Test", "lambda$method$0", "(I)V", 6),
            instantiated: "()V",
            flags: None,
        },
        vec![
            0x08, // 0: iconst_5
            0x3c, // 1: istore_1
            0x1b, // 2: iload_1
            0xba, 0x00, 0x00, 0x00, 0x00, // 3: invokedynamic
            0x57, // 8: pop
            0xb1, // 9: return
        ],
        3,
        1,
        2,
    )
}

/// The recovery report of one assembled class's `method()V`.
fn recover_class(class: &[u8]) -> jarde_java::RecoveryReport {
    recover_class_under(class, jarde_java::pass::JAVA_8)
}

/// The same, under one named profile.
fn recover_class_under(
    class: &[u8],
    profile: jarde_java::RecoveryProfile,
) -> jarde_java::RecoveryReport {
    let payload = analyze(class, b"method", b"()V");
    let facts = facts_of(class, b"method", 0, Vec::new());
    let mut budget = Budget::new(limits());
    recover(
        &RecoveryRequest::new(payload.analysis.ir(), &facts, profile),
        &mut budget,
    )
}

/// The one site's record, with the fixture's own use site.
fn site_of(report: &jarde_java::RecoveryReport, bci: u32) -> &jarde_java::LambdaRecord {
    assert_eq!(report.lambdas.len(), 1, "{:?}", report.lambdas);
    let site = &report.lambdas[0];
    assert_eq!(site.use_site, bci, "{site:?}");
    site
}

#[test]
fn a_captured_lambda_is_written_with_its_capture_in_order_before_the_sam_parameters() {
    let class = capture_and_parameter_lambda_class();
    let report = recover_class(&class);
    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(
        report.representation,
        Representation::Java,
        "{}\nregions: {:?}\nfallbacks: {:?}",
        report.text,
        report.regions,
        report.fallbacks
    );
    assert_eq!(report.quality, Quality::Structured, "{}", report.text);
    assert!(
        report
            .text
            .contains("java.util.function.IntUnaryOperator local2 = (int p0) -> Test.lambda$method$0(local1, p0);"),
        "the capture comes first, then the SAM's parameter, in the descriptor's order:\n{}",
        report.text
    );
    // The parameters are the SAM's own, in the SAM's order and count — written with the types the
    // *instantiated* method type states, which is the type the implementation is given there.
    assert_eq!(
        report.text.matches("Test.lambda$method$0(").count(),
        1,
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("lambda$method$0(p0, local1)"),
        "{}",
        report.text
    );
    // The site's anchor carries the constant-pool entry the site *is* — and, since P3 3.2, the
    // member body that entry is an index in: a CP index means nothing without the class file that
    // holds the pool. The member is the one the run's own declaration read.
    let site = report
        .source_map
        .direct_of_bci(3)
        .into_iter()
        .next()
        .expect("the dynamic site at BCI 3 anchors a node")
        .origin()
        .primary()
        .clone();
    assert_eq!(
        site.cp(),
        Some(site_of(&report, 3).site_cp),
        "the anchor names the site's own pool entry"
    );
    assert_eq!(
        site.method()
            .map(|method| (method.name.0.clone(), method.descriptor.0.clone())),
        Some((b"method".to_vec(), b"()V".to_vec())),
        "and the member body whose pool that index is an index in"
    );
    assert!(site.member().is_some(), "the anchor is a method point");
    assert!(report.fallbacks.is_empty(), "{:?}", report.fallbacks);
}

#[test]
fn two_captures_are_written_in_the_order_the_site_reads_them() {
    // The order of the captured arguments is the one thing a lambda presentation can silently get
    // wrong: a swapped pair still reads like a lambda, and the value flow is the only thing that
    // says which is which. Both the text and the record are pinned here.
    let class = two_capture_lambda_class();
    let report = recover_class(&class);
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report
            .text
            .contains("() -> Test.lambda$method$0(local1, local2)"),
        "the site reads local1 before local2:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("lambda$method$0(local2, local1)"),
        "and nothing writes them the other way round:\n{}",
        report.text
    );
    let site = site_of(&report, 6);
    assert_eq!(
        site.captures,
        vec![
            jarde_java::LambdaCapture { bci: Some(4) },
            jarde_java::LambdaCapture { bci: Some(5) },
        ],
        "the record states the captures in the order the site reads them"
    );
    assert_eq!(site.sam_method_type.as_deref(), Some("()V"));
    assert_eq!(
        report.representation,
        Representation::Java,
        "{}",
        report.text
    );
}

#[test]
fn a_verified_site_is_recorded_with_its_bootstrap_use_site_and_captures() {
    let class = capture_lambda_class();
    let report = recover_class(&class);
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report
            .text
            .contains("java.lang.Runnable local2 = () -> Test.lambda$method$0(local1);"),
        "{}",
        report.text
    );
    let site = site_of(&report, 3);
    assert_eq!(site.form, Some(jarde_java::LambdaForm::Lambda));
    assert_eq!(site.refusal, None, "{site:?}");
    // The bootstrap, as the class states it: the factory handle with its method-handle kind, the
    // number of static arguments, and the two method types beside the implementation handle.
    assert_eq!(
        site.bootstrap.as_deref(),
        Some("java.lang.invoke.LambdaMetafactory.metafactory (REF_invokeStatic)")
    );
    assert_eq!(site.bootstrap_arguments, 3);
    assert_eq!(site.bootstrap_index, 0);
    assert_eq!(site.sam_method_type.as_deref(), Some("()V"));
    assert_eq!(site.instantiated_method_type.as_deref(), Some("()V"));
    assert_eq!(
        site.implementation.as_deref(),
        Some("Test.lambda$method$0(I)V (REF_invokeStatic)")
    );
    // The use site's own reference, and the SAM the site presents.
    assert_eq!(site.sam_name, "run");
    assert_eq!(site.sam_descriptor, "(I)Ljava/lang/Runnable;");
    assert!(
        site.site_cp > 0,
        "the site's pool entry is stated, not re-derived from the BCI"
    );
    // The captured value, in the order the site reads it.
    assert_eq!(
        site.captures,
        vec![jarde_java::LambdaCapture { bci: Some(2) }],
        "the capture is the load at BCI 2"
    );
    assert!(
        report.rules.contains(&jarde_java::pass::LAMBDA.rule()),
        "which rule produced the shape is in the report: {:?}",
        report.rules
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_lambda_sites"),
        "{:?}",
        report.diagnostics
    );

    // The origin: the lambda node is the site (its own pool entry named), and the captured value's
    // BCI reaches it as *derived* evidence — one expression, more than one original BCI.
    let at_site = report.source_map.direct_of_bci(3);
    let lambda_node = at_site
        .iter()
        .find(|segment| segment.text(&report.text).contains("->"))
        .expect("the lambda node is anchored at the site");
    assert_eq!(
        lambda_node.origin().primary().cp(),
        Some(site.site_cp),
        "the node's own anchor names the site's pool entry"
    );
    let derived = report.source_map.derived_of_bci(2);
    assert_eq!(
        derived.len(),
        2,
        "the captured value's BCI reaches the two nodes whose text reproduces it: {derived:?}"
    );
    let texts: Vec<&str> = derived
        .iter()
        .map(|segment| segment.text(&report.text))
        .collect();
    assert!(
        texts.contains(&"() -> Test.lambda$method$0(local1)"),
        "the lambda node presents the capture it reads: {texts:?}"
    );
    assert!(
        texts.contains(&"Test.lambda$method$0(local1)"),
        "and so does the call whose argument list holds it: {texts:?}"
    );
    assert!(
        report.source_map.direct_of_bci(2).len() == 1,
        "the capture's own text is anchored where it was produced: {:?}",
        report.source_map.direct_of_bci(2)
    );
}

#[test]
fn a_bound_receiver_becomes_a_method_reference_and_a_static_member_a_type_reference() {
    // The bound case: the site's captures are exactly what the handle's receiver needs.
    let report = recover_class(&bound_reference_class());
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report
            .text
            .contains("java.lang.Runnable local2 = local1::run;"),
        "{}",
        report.text
    );
    let site = site_of(&report, 3);
    assert_eq!(site.form, Some(jarde_java::LambdaForm::MethodReference));
    assert_eq!(
        site.implementation.as_deref(),
        Some("java.lang.Runnable.run()V (REF_invokeInterface)")
    );
    assert_eq!(
        site.captures,
        vec![jarde_java::LambdaCapture { bci: Some(2) }]
    );
    assert_eq!(
        report.representation,
        Representation::Java,
        "{}",
        report.text
    );

    // The static case: nothing is captured, and the member is another class's.
    let report = recover_class(&static_reference_class());
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report
            .text
            .contains("java.util.function.Function local1 = java.lang.String::valueOf;"),
        "{}",
        report.text
    );
    let site = site_of(&report, 0);
    assert_eq!(site.form, Some(jarde_java::LambdaForm::MethodReference));
    assert!(site.captures.is_empty(), "{site:?}");
    assert_eq!(site.sam_name, "apply");

    // The constructor case: a `REF_newInvokeSpecial` handle is a reference to the constructor.
    let report = recover_class(&constructor_reference_class());
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report
            .text
            .contains("java.util.function.Supplier local1 = java.lang.Object::new;"),
        "{}",
        report.text
    );
    let site = site_of(&report, 0);
    assert_eq!(site.form, Some(jarde_java::LambdaForm::MethodReference));
    assert_eq!(
        site.implementation.as_deref(),
        Some("java.lang.Object.<init>()V (REF_newInvokeSpecial)")
    );

    // The same handle reached with a captured value: the reference writing cannot carry it, so the
    // site is written as a lambda whose body constructs the member.
    let report = recover_class(&constructor_lambda_class());
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report
            .text
            .contains("java.util.function.Supplier local2 = () -> new Test(local1);"),
        "{}",
        report.text
    );
    let site = site_of(&report, 3);
    assert_eq!(site.form, Some(jarde_java::LambdaForm::Lambda));
    assert_eq!(
        site.implementation.as_deref(),
        Some("Test.<init>(I)V (REF_newInvokeSpecial)")
    );
    assert_eq!(
        site.captures,
        vec![jarde_java::LambdaCapture { bci: Some(2) }]
    );
}

#[test]
fn an_arbitrary_bootstrap_is_never_presented_as_a_lambda() {
    // A04: the site's descriptor, its capture and the *shapes* of its static arguments are exactly
    // the ones a lambda site has — the factory is the only difference. Nothing here may become a
    // lambda, and the refusal has to say which bootstrap the class really named.
    let class = arbitrary_bootstrap_class();
    let report = recover_class(&class);
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        !report.text.contains("->"),
        "no lambda is written for an arbitrary bootstrap:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("::"),
        "and no method reference either:\n{}",
        report.text
    );
    assert!(report.text.contains("// @bytecode"), "{}", report.text);
    assert_eq!(report.representation, Representation::Mixed);
    assert_eq!(report.quality, Quality::Fallback);

    let site = site_of(&report, 3);
    assert_eq!(site.form, None, "{site:?}");
    assert_eq!(
        site.bootstrap.as_deref(),
        Some("Test.myBootstrap (REF_invokeStatic)"),
        "the record states the bootstrap the class really named"
    );
    assert_eq!(site.bootstrap_arguments, 3);
    let refusal = site.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_lambda_bootstrap");
    assert_eq!(
        refusal.rule,
        jarde_java::pass::LAMBDA.rule(),
        "the record names the rule that refused the site"
    );
    assert_eq!(
        refusal.requirement, None,
        "this is a shape that is not a lambda, not a declared precondition that fell short"
    );
    assert!(
        refusal.message.contains("Test.myBootstrap"),
        "the diagnosis names the bootstrap: {}",
        refusal.message
    );
    assert!(
        refusal.message.contains("LambdaMetafactory"),
        "and the factory the contract states: {}",
        refusal.message
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_lambda_bootstrap"
                && diagnostic.message.contains("myBootstrap")),
        "{:?}",
        report.diagnostics
    );
    assert!(
        report.fallbacks.is_empty(),
        "the site is refused as a *statement*, not as a region: {:?}",
        report.fallbacks
    );
}

#[test]
fn a_sam_that_the_implementation_does_not_line_up_with_is_refused() {
    // The site binds two values (one capture and the SAM's one parameter) and the implementation
    // takes three: printing a lambda here would write a call that does not mean what the bytecode
    // did, so the site keeps its bytecode with the counts in the diagnosis.
    let class = arity_mismatch_class();
    let report = recover_class(&class);
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("->"), "{}", report.text);
    let site = site_of(&report, 3);
    assert_eq!(
        site.bootstrap.as_deref(),
        Some("java.lang.invoke.LambdaMetafactory.metafactory (REF_invokeStatic)"),
        "the factory *is* verified: what fell short is the shape behind it"
    );
    assert_eq!(site.form, None, "{site:?}");
    let refusal = site.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_lambda_sam_arity");
    assert!(
        refusal.message.contains("(III)I") && refusal.message.contains("3 parameter"),
        "the diagnosis states the implementation's own arity: {}",
        refusal.message
    );
    assert!(
        refusal.message.contains("1 captured value"),
        "and the values the site would bind: {}",
        refusal.message
    );
    assert_eq!(report.representation, Representation::Mixed);
}

#[test]
fn a_capture_the_shape_may_not_replay_is_refused_under_the_declared_precondition() {
    // The captured value is a call's result: the call is already a statement of the artifact, so
    // writing it again inside the lambda body would run it twice. The `lambda@1` rule states the
    // `Replayable` requirement, and this is the check point that consults it.
    let class = unreplayable_capture_class();
    let report = recover_class(&class);
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("->"), "{}", report.text);
    let site = site_of(&report, 3);
    let refusal = site.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_lambda_capture_not_replayable");
    assert_eq!(
        refusal.requirement.as_deref(),
        Some("a captured value whose text can be read again where the shape writes it"),
        "the refusal names the requirement it consulted: {refusal:?}"
    );
    assert!(
        refusal.message.contains("BCI 0"),
        "and the instruction that fell short: {}",
        refusal.message
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_lambda_capture_not_replayable"),
        "{:?}",
        report.diagnostics
    );
    // The call the capture came from is still written: refusing the site does not drop the effect.
    assert!(report.text.contains("produce();"), "{}", report.text);
    assert_eq!(report.representation, Representation::Mixed);
}

#[test]
fn an_alt_metafactory_flag_word_and_an_unread_instance_are_both_refused() {
    // `altMetafactory` with a flag word: markers, bridges or a serializable site need a presentation
    // this layer does not have, so only the flagless form is one it writes.
    let report = recover_class(&alt_metafactory_flags_class());
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        !report.text.contains("->") && !report.text.contains("::"),
        "{}",
        report.text
    );
    let site = site_of(&report, 3);
    assert_eq!(
        site.bootstrap.as_deref(),
        Some("java.lang.invoke.LambdaMetafactory.altMetafactory (REF_invokeStatic)")
    );
    assert_eq!(site.bootstrap_arguments, 4);
    let refusal = site.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_lambda_bootstrap_arguments");
    assert!(
        refusal.message.contains("flag word 1"),
        "{}",
        refusal.message
    );

    // A verified site whose instance nothing reads: the call it makes has no place in the body, so
    // the site is quoted rather than written off as a value with no effect.
    let report = recover_class(&unconsumed_site_class());
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        !report.text.contains("->") && !report.text.contains("::"),
        "{}",
        report.text
    );
    let site = site_of(&report, 3);
    assert_eq!(
        site.bootstrap.as_deref(),
        Some("java.lang.invoke.LambdaMetafactory.metafactory (REF_invokeStatic)")
    );
    let refusal = site.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_lambda_unconsumed");
    assert!(report.text.contains("// @bytecode"), "{}", report.text);
}

#[test]
fn a_profile_that_does_not_present_java_8_is_refused_the_lambda_rule() {
    // The gate `Pass::admits` reads, with its first production instance: an `invokedynamic` call
    // site is a Java 8 construct, and a run whose profile presents the artifact as Java 7 is not
    // admitted the rule. The body is still presented — the site is refused, the run is not stopped.
    let class = capture_lambda_class();
    let report = recover_class_under(
        &class,
        jarde_java::RecoveryProfile {
            java_release: 7,
            ..jarde_java::pass::JAVA_8
        },
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("->"), "{}", report.text);
    let site = site_of(&report, 3);
    let refusal = site.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_lambda_rule_not_admitted");
    assert!(
        refusal.message.contains("lambda@1") && refusal.message.contains("Java 7"),
        "{}",
        refusal.message
    );
    assert_eq!(
        site.bootstrap, None,
        "the profile gate runs before the table is read: nothing beyond the site's own descriptor is stated"
    );
}

#[test]
fn a_lambda_body_without_debug_metadata_is_named_deterministically() {
    // No `LocalVariableTable`, no `MethodParameters`: the fixture's slots are named by ordinal, and
    // a lambda's parameters get names of their own that cannot collide with them (JLS 6.4 forbids a
    // lambda parameter that shadows an enclosing local).
    let class = capture_and_parameter_lambda_class();
    let report = recover_class(&class);
    let again = recover_class(&class);
    assert_eq!(
        report.text, again.text,
        "the same bytes produce the same names"
    );
    assert!(report.text.contains("int local1 = 5;"), "{}", report.text);
    assert!(report.text.contains("(int p0) ->"), "{}", report.text);
    assert!(
        !report.text.contains("(int local1) ->"),
        "the lambda's parameter does not take the local's name:\n{}",
        report.text
    );
    assert!(
        report.aliased_names.is_empty(),
        "an invented name hides nothing: {:?}",
        report.aliased_names
    );
    assert_eq!(report.syntax_status, SyntaxStatus::Unchecked);
}

#[test]
fn a_budget_that_refuses_the_emission_of_a_lambda_body_hands_out_nothing() {
    // The existing stop semantics, on a body whose statements are built around a lambda: a refusal
    // is a *stop* — no text, no segments, no success — and never an empty body.
    let class = capture_and_parameter_lambda_class();
    let payload = analyze(&class, b"method", b"()V");
    let facts = facts_of(&class, b"method", 0, Vec::new());
    let whole = {
        let mut budget = Budget::new(limits());
        recover(
            &RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8),
            &mut budget,
        )
        .text
        .len()
    };
    let mut budget = Budget::new(Limits {
        output_bytes: u64::try_from(whole - 1).expect("the artifact is small"),
        ..limits()
    });
    let stopped = recover(
        &RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8),
        &mut budget,
    );
    assert!(!stopped.produced(), "{:?}", stopped.outcome);
    assert!(stopped.text.is_empty(), "{:?}", stopped.text);
    assert!(stopped.source_map.is_empty(), "{:?}", stopped.source_map);
    assert!(
        matches!(stopped.outcome, RecoveryOutcome::Stopped(_)),
        "{:?}",
        stopped.outcome
    );
    assert_eq!(stopped.representation, Representation::Bytecode);
}
