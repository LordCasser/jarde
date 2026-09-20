//! P3 3.1 acceptance: **one local's type is decided once**, before its statements are built.
//!
//! The defect this file pins was measured on the committed sample `tests/fixtures/p3-hoisted-boolean/`
//! (whose README states the compiler, the command, the digest and the bytecode of every member): the
//! hoisted declaration path (`build.rs::declarations`) reads the first write's descriptor evidence
//! only, while the in-place path (`declare`) also recognized "a read of a local this body already
//! declared `boolean`" — so the *same* value was typed `int` or `boolean` depending on which path
//! reached it first, and the local's type therefore depended on the order the regions were walked in.
//! `copied` and `swapped` are the two orders of one shape, and the pre-fix texts show the dependency:
//!
//! ```text
//! copied(ZI)I  int local3; boolean local2 = arg0; … local3 = local2; … local3 = arg0; … if (local3 != 0) …
//!              javac --release 8: error: incompatible types: boolean cannot be converted to int (twice)
//! swapped(ZI)I boolean local3; … (the same shape with the arms exchanged)
//! ```
//!
//! Everything here goes through the entry point the CLI calls ([`Engine::recover_method`]) over that
//! sample, and the acceptance is the text plus the compiler: `tests/p3_execution_comparison.rs`
//! registers the same members, compiles each body under a declaration derived from the run's own
//! facts and compares the two sides' traces (six members executed, the two refusals recorded).

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 743 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-hoisted-boolean/v8/HoistedBoolean.class");

/// Every member of the sample that declares a body, so a renamed fixture fails here instead of
/// covering less than this file claims.
const DECLARED: [(&[u8], &[u8]); 9] = [
    (b"<init>", b"()V"),
    (b"copied", b"(ZI)I"),
    (b"swapped", b"(ZI)I"),
    (b"relayed", b"(Z)Z"),
    (b"literalArmed", b"(Z)I"),
    (b"fromParameter", b"(ZI)Z"),
    (b"intLocal", b"(I)I"),
    (b"unproven", b"(Z)Z"),
    (b"conflicted", b"(ZI)I"),
];

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

/// One caller domain rooted at the fixture's own snapshot, and nothing else.
fn environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
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
    ResolutionEnvironment {
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
    }
}

/// One opened sample and the class identity the reader's own header read stated for it.
struct Fixture {
    snapshot: ArtifactSnapshot,
    class_bytes: ClassBytesId,
}

/// Opens one sample and reads its header through the reader's own entry point.
fn fixture(engine: &Engine, bytes: &[u8]) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    Fixture {
        snapshot,
        class_bytes: inspected.source.class_bytes.clone(),
    }
}

/// One recovery run over one member of a sample, through the entry point the CLI calls.
fn recover(engine: &Engine, fixture: &Fixture, name: &[u8], descriptor: &[u8]) -> RecoveryReport {
    let request = MethodAnalysisRequest {
        environment: environment(&fixture.snapshot),
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: fixture.snapshot.id().clone(),
                },
                class_bytes: fixture.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    engine
        .recover_method(
            slice::from_ref(&fixture.snapshot),
            &request,
            &mut Budget::new(limits()),
        )
        .expect("a legal request is answered, not raised")
        .recovery()
        .clone()
}

/// One recovery run over one member of a sample, through the entry point the CLI calls, under the
/// caller's own budget: both halves of the answer, so a case can read the run's own execution plane
/// beside the presentation (`tests/p3_content.rs`'s shape).
fn recovered_with_budget(
    engine: &Engine,
    fixture: &Fixture,
    name: &[u8],
    descriptor: &[u8],
    budget: &mut Budget,
) -> RecoveredMethod {
    let request = MethodAnalysisRequest {
        environment: environment(&fixture.snapshot),
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: fixture.snapshot.id().clone(),
                },
                class_bytes: fixture.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    engine
        .recover_method(slice::from_ref(&fixture.snapshot), &request, budget)
        .expect("a legal request is answered, not raised")
}

/// One recovery run under the caller's own budget, as a report.
fn recover_with_budget(
    engine: &Engine,
    fixture: &Fixture,
    name: &[u8],
    descriptor: &[u8],
    budget: &mut Budget,
) -> RecoveryReport {
    recovered_with_budget(engine, fixture, name, descriptor, budget)
        .recovery()
        .clone()
}

/// The text of one member the run presents whole: a body that is not `Java`/`Structured` is a
/// failure of the premise, not a boundary, so the planes are asserted before the text is read.
fn whole_body(engine: &Engine, fixture: &Fixture, name: &[u8], descriptor: &[u8]) -> String {
    let report = recover(engine, fixture, name, descriptor);
    assert_eq!(
        report.representation,
        Representation::Java,
        "`{}`: {}\nregions: {:?}\nfallbacks: {:?}",
        report.method,
        report.text,
        report.regions,
        report.fallbacks
    );
    assert_eq!(report.quality, Quality::Structured, "`{}`", report.method);
    report.text
}

/// The bytecode indexes the artifact's own quotes name, in the order each quote states them.
fn quoted_bcis(text: &str) -> Vec<u32> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
        .flat_map(|bcis| {
            bcis.split_whitespace().map(|bci| {
                bci.parse::<u32>()
                    .expect("a quoted bytecode index is a number")
            })
        })
        .collect()
}

/// The facts a refusal is, as in `tests/p3_boolean_contexts.rs`: the planes that say Java is not
/// claimed, the quote that names the bytecode, its anchor in the segment table, the reason that
/// states the index, and the member the answer belongs to.
fn assert_refused(report: &RecoveryReport, name: &str, bci: u32) {
    assert!(
        report.produced(),
        "`{name}`: a refusal is still an answer: {:?}",
        report.outcome
    );
    assert_eq!(
        report.representation,
        Representation::Mixed,
        "`{name}`: {}\nregions: {:?}",
        report.text,
        report.regions
    );
    assert_eq!(report.quality, Quality::Fallback, "`{name}`");
    assert_eq!(
        report.syntax_status,
        SyntaxStatus::NotJava,
        "`{name}`: a body with a quoted region is not claimed to be Java:\n{}",
        report.text
    );
    assert!(
        quoted_bcis(&report.text).contains(&bci),
        "`{name}`: the quote names the BCI the region could not present ({bci}):\n{}",
        report.text
    );
    assert!(
        !report.text_of_bci(bci).is_empty(),
        "`{name}`: the bytecode the refusal is about is still anchored in the segment table:\n{}",
        report.text
    );
    assert!(
        report.text.contains(&format!("BCI {bci}")),
        "`{name}`: the refusal states the bytecode index it could not type:\n{}",
        report.text
    );
    assert!(
        report.method.starts_with(name),
        "`{name}`: the report names the member it refused: {}",
        report.method
    );
}

// -------------------------------------------------------------------------------------------
// The two orders of one shape: one conclusion, and it does not depend on which arm comes first.
// -------------------------------------------------------------------------------------------

#[test]
fn a_hoisted_local_copied_from_a_boolean_local_is_declared_boolean() {
    // The review's shape. `c` is written in both arms of the branch and read after the join, so its
    // declaration is hoisted above the branch — where the old hoisted path read the first write's
    // descriptor evidence only and declared `int local3`, and the two writes then published boolean
    // values into it: `int local3; … local3 = local2; … local3 = arg0; … if (local3 != 0) …`, text
    // `javac --release 8` refuses twice (`boolean cannot be converted to int`). One decision, taken
    // for the variable before its statements are built, makes the declaration and every use agree.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let text = whole_body(&engine, &fixture, b"copied", b"(ZI)I");

    assert!(
        text.contains("boolean local3;"),
        "the hoisted declaration states the type the plan decided for the variable:\n{text}"
    );
    assert!(
        text.contains("boolean local2 = arg0;"),
        "the local the value is copied from is declared boolean (unchanged by this change):\n{text}"
    );
    assert!(
        text.contains("local3 = local2;") && text.contains("local3 = arg0;"),
        "both writes are checked against that one decision and both are spellable as it:\n{text}"
    );
    assert!(
        text.contains("if (local3) {"),
        "the condition of a decided-boolean local is a truth test:\n{text}"
    );
    assert!(
        !text.contains("int local3") && !text.contains("local3 != 0"),
        "no `int` spelling of a boolean variable may survive anywhere:\n{text}"
    );
}

#[test]
fn the_conclusion_does_not_depend_on_the_order_the_arms_come_in() {
    // `swapped` is `copied` with the two arms exchanged, so the write the bytecode reaches first is
    // the descriptor-proven one rather than the copied local. Under the old rule the two orders got
    // two different types — `copied` read its first write (`c = a`) through the descriptor-only
    // hoisted path and declared `int`, while `swapped`'s first write (`c = b`) was a `Z` parameter's
    // load and declared `boolean`. The decision is now taken for the variable, from its whole
    // evidence, before any statement exists, so the two orders agree on the type, the declaration and
    // every use; only the arms' own text differs, and only in the order the source wrote them.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let copied = whole_body(&engine, &fixture, b"copied", b"(ZI)I");
    let swapped = whole_body(&engine, &fixture, b"swapped", b"(ZI)I");

    for (name, text) in [("copied", &copied), ("swapped", &swapped)] {
        assert!(
            text.contains("boolean local3;"),
            "`{name}`: the declaration's type is the decided one:\n{text}"
        );
        assert!(
            text.contains("if (local3) {"),
            "`{name}`: the truth test does not depend on the order either:\n{text}"
        );
        assert!(
            !text.contains("int local3") && !text.contains("local3 != 0"),
            "`{name}`: no `int` spelling survives:\n{text}"
        );
    }
    // The two orders differ exactly in which arm writes which value, and in nothing else: the
    // declaration line and the condition are the same string in both texts.
    let armed = |text: &str| -> String {
        let start = text
            .find("boolean local3;")
            .expect("the declaration is written");
        let end = text.find("if (local3)").expect("the use is written");
        text[start..end].to_string()
    };
    let (copied_arms, swapped_arms) = (armed(&copied), armed(&swapped));
    assert_ne!(
        copied_arms, swapped_arms,
        "the twin's arms are the other way round, or the comparison is empty"
    );
    assert_eq!(
        copied_arms.matches("local3 =").count(),
        2,
        "each order writes the local twice:\n{copied_arms}"
    );
    assert!(
        copied_arms.contains("local3 = local2;") && copied_arms.contains("local3 = arg0;"),
        "the first order writes the copied local in the arm the source put it in:\n{copied_arms}"
    );
    assert!(
        swapped_arms.contains("local3 = arg0;") && swapped_arms.contains("local3 = local2;"),
        "and the twin writes the same two values in the other order:\n{swapped_arms}"
    );
    let order = |text: &str| -> (&'static str, &'static str) {
        let first = text.find("local3 = ").expect("a write is written");
        if text[first..].starts_with("local3 = local2;") {
            ("local2", "arg0")
        } else {
            ("arg0", "local2")
        }
    };
    assert_eq!(
        order(&copied_arms),
        ("local2", "arg0"),
        "`copied` writes the copied local first:\n{copied_arms}"
    );
    assert_eq!(
        order(&swapped_arms),
        ("arg0", "local2"),
        "`swapped` writes the descriptor-proven value first:\n{swapped_arms}"
    );
}

#[test]
fn a_copy_chain_through_another_boolean_local_stays_boolean() {
    // `boolean x = b; boolean y; boolean z; if (b) { y = x; z = y; } else { … } return z;`: `y`'s
    // evidence arrives from `x` and `z`'s from `y`, so a pass that read each write once, in bytecode
    // order, would meet `z = y` before `y` was decided — and both variables were declared `int`
    // before this change, with the return refused. The decision reaches a fixpoint over the chain
    // before any statement is built, so every variable on it is boolean.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let text = whole_body(&engine, &fixture, b"relayed", b"(Z)Z");

    assert!(
        text.contains("boolean local1 = arg0;"),
        "the chain's first variable is declared from its own descriptor evidence:\n{text}"
    );
    assert!(
        text.contains("boolean local2;") && text.contains("boolean local3;"),
        "and the two the reads reach are declared boolean too:\n{text}"
    );
    assert!(
        text.contains("local2 = local1;") && text.contains("local3 = local2;"),
        "each write of the chain is written as the value it copies:\n{text}"
    );
    assert!(
        text.contains("return local3;"),
        "the return of the chain's last variable is a boolean return:\n{text}"
    );
    assert!(
        !text.contains("int local") && !text.contains("!= 0"),
        "no variable on the chain keeps an `int` spelling:\n{text}"
    );
}

// -------------------------------------------------------------------------------------------
// The two refusals: the structure terminates instead of publishing a contradictory assignment.
// -------------------------------------------------------------------------------------------

#[test]
fn a_write_that_cannot_be_spelled_as_the_decided_type_terminates_its_structure() {
    // `conflicted`: the local's first write is a `true` literal, so the frames' `int` decides it (a
    // `0`/`1` literal may not initiate a boolean decision — `int x = 0;` and `boolean c = true;` are
    // the same bytes), and the other arm stores a `boolean` parameter's load, which cannot be spelled
    // as an `int`. Before this change the second write was published as it was
    // (`int local2; … local2 = arg0; …`, javac: `boolean cannot be converted to int`); now the write
    // is refused, its bytecode is quoted, and the assignment it would have carried is not written.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let report = recover(&engine, &fixture, b"conflicted", b"(ZI)I");
    assert_refused(&report, "conflicted", 10);

    // What the artifact does hold: the declaration the first write states, that write, the quote that
    // names the refused store, and the member's own remaining statements.
    assert!(
        report.text.contains("int local2;"),
        "the declaration the first write states is written:\n{}",
        report.text
    );
    assert!(
        report.text.contains("local2 = 1;"),
        "and the first write, which the decision can be spelled as, is written:\n{}",
        report.text
    );
    assert!(
        report.text.contains("if (local2 != 0) {"),
        "the member's own uses keep the spelling of the decided type:\n{}",
        report.text
    );
    assert!(
        report.text.contains("decided holds `int`"),
        "the refusal states the type the variable's own first write decided:\n{}",
        report.text
    );
    // What it does not hold: the assignment that could not be spelled as that type, and any boolean
    // spelling the decision excludes.
    assert!(
        !report.text.contains("local2 = arg0;"),
        "a refused write is not followed by its assignment:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("boolean local2") && !report.text.contains("= true"),
        "no boolean spelling of an `int`-decided variable may be published:\n{}",
        report.text
    );
}

#[test]
fn a_value_the_evidence_cannot_type_is_refused_where_a_boolean_is_required() {
    // `unproven`: the same body as `literalArmed` under a `Z` descriptor, where the position requires
    // a boolean the evidence does not have (`int x = 0;`/`boolean x = true;` are one bytecode shape,
    // and the sole writes are literals). The layer refuses the return instead of publishing an `int`
    // spelling the member's own signature rejects — the same boundary before and after this change,
    // and the proof that the plan's decision is read where the position's requirement is.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let report = recover(&engine, &fixture, b"unproven", b"(Z)Z");
    assert_refused(&report, "unproven", 12);
    assert!(
        report.text.contains("int local1;"),
        "the declaration keeps the type the literal-only writes decide:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("boolean local1"),
        "no local of this member is presented as a boolean:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("return local1;"),
        "the refused return is not published in its `int` spelling:\n{}",
        report.text
    );
}

// -------------------------------------------------------------------------------------------
// The controls: the types the evidence states keep their text, and the literal-only local stays int.
// -------------------------------------------------------------------------------------------

#[test]
fn the_literal_only_local_keeps_the_type_the_frames_state() {
    // The recorded boundary, and the reason it is recorded rather than closed: the only values written
    // into `x` are `true` and `false`, which is how both a boolean and an `int` are pushed, so the
    // decision reads the frames' `int` (admitting the literal would re-type every `int` local filled
    // with `0`/`1`, an outcome the predecessor change measured as a regression). Both declaration
    // paths give that same answer, the text compiles under the member's own signature, and the two
    // sides return the same values (`tests/p3_execution_comparison.rs`).
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let text = whole_body(&engine, &fixture, b"literalArmed", b"(Z)I");

    assert!(text.contains("int local1;"), "{text}");
    assert!(
        text.contains("local1 = 1;") && text.contains("local1 = 0;"),
        "the literals keep the integer spelling the decision states:\n{text}"
    );
    assert!(
        text.contains("if (local1 != 0) {"),
        "and so does the condition:\n{text}"
    );
    assert!(
        !text.contains("boolean") && !text.contains("true") && !text.contains("false"),
        "no boolean spelling may appear in this member:\n{text}"
    );
}

#[test]
fn the_controls_the_evidence_types_keep_their_text() {
    // The two positive controls the plan names: a local whose writes are all `Z` parameter loads (the
    // predecessor change's `pick` shape) stays `boolean`, and a local whose values are literals and an
    // `int` parameter load stays `int`. Nothing about this change may move them.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);

    let text = whole_body(&engine, &fixture, b"fromParameter", b"(ZI)Z");
    assert!(text.contains("boolean local2;"), "{text}");
    assert!(
        text.contains("local2 = arg0;") && text.contains("return local2;"),
        "{text}"
    );
    assert!(
        !text.contains("int local2") && !text.contains("!= 0"),
        "{text}"
    );

    let text = whole_body(&engine, &fixture, b"intLocal", b"(I)I");
    assert!(text.contains("int local1;"), "{text}");
    assert!(
        text.contains("local1 = 0;")
            && text.contains("local1 = 1;")
            && text.contains("local1 = arg0;"),
        "{text}"
    );
    assert!(
        !text.contains("boolean") && !text.contains("true"),
        "the `0`/`1` stores of an `int` local are not booleans:\n{text}"
    );
}

#[test]
fn the_type_decision_is_billed_and_a_stopped_run_commits_nothing() {
    // The plan's propagation is work of **this** run, not a way around its budget: every queue entry
    // is charged to the same `IrItems` dimension the statements are (`crates/jarde-java/src/build.rs`,
    // `decide_types`) and polled through the same `stop.rs` entry, and no new dimension is added.
    //
    // Both numbers below are this request's own deterministic usage, as the pinned-usage cases in
    // `tests/p3_declaration_handoff.rs` state theirs. `relayed`'s chain has three entries: `x` (its
    // first write stores the `Z` parameter's load, which seeds the queue), then `y` (its write
    // stores a read of `x`) and then `z` — so three charges are the plan's, and the run's total is
    // 378 without them. `IR_ITEMS_BEFORE_THE_PLAN` is the bound at which the plan's own first entry
    // is the charge that is refused: with the plan billed, the first charge that carries a BCI is
    // `x`'s write at BCI 1; with the plan's billing removed, that same bound reaches the first
    // statement's charge at BCI 9 instead, so this assertion is what stops the billing from being
    // dropped silently.
    const RELAYED_IR_ITEMS: u64 = 381;
    const IR_ITEMS_BEFORE_THE_PLAN: u64 = 369;
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let ample = recover(&engine, &fixture, b"relayed", b"(Z)Z");
    let ExecutionReport::Complete { usage } = &ample.execution else {
        panic!("the ample run completes: {:?}", ample.execution);
    };
    assert_eq!(
        usage.ir_items, RELAYED_IR_ITEMS,
        "the run's own usage, which the plan's three entries are part of"
    );

    let mut tight = Budget::new(Limits {
        ir_items: IR_ITEMS_BEFORE_THE_PLAN,
        ..limits()
    });
    let stopped = recover_with_budget(&engine, &fixture, b"relayed", b"(Z)Z", &mut tight);
    assert!(
        matches!(
            stopped.outcome,
            RecoveryOutcome::Stopped(StopReason::Budget {
                dimension: CountedBudgetDimension::IrItems,
                at: Some(1),
                ..
            })
        ),
        "{:?}",
        stopped.outcome
    );
    assert_eq!(stopped.content, RecoveryContent::NotProduced);
    assert_eq!(stopped.text, "", "a stop hands out no artifact");
    assert_eq!(stopped.source_map.len(), 0, "and no segment table");
    assert!(stopped.regions.is_empty() && stopped.fallbacks.is_empty());
    assert!(!stopped.produced());

    // The caller's cancellation is the other half of the same wiring: the run's own execution plane
    // says it was cancelled, and the report commits no artifact and states no content.
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(limits(), token);
    let recovered = recovered_with_budget(&engine, &fixture, b"relayed", b"(Z)Z", &mut cancelled);
    assert!(
        matches!(
            recovered.analysis().execution,
            ExecutionReport::Cancelled { .. }
        ),
        "the cancelled run says so: {:?}",
        recovered.analysis().execution
    );
    let stopped = recovered.recovery();
    assert_eq!(stopped.content, RecoveryContent::NotProduced);
    assert_eq!(stopped.text, "");
    assert!(!stopped.produced());
}

#[test]
fn every_declared_member_is_covered_by_this_file() {
    // The premise: the sample declares every member this file classifies, and each of them is
    // answered (presented or refused) rather than missed.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    for (name, descriptor) in DECLARED {
        let report = recover(&engine, &fixture, name, descriptor);
        let spelled = format!(
            "{}{}",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(descriptor)
        );
        assert_eq!(
            report.method, spelled,
            "the run answers for the member the request named"
        );
        assert!(
            report.produced(),
            "`{spelled}`: {:?}\n{}",
            report.outcome,
            report.text
        );
    }
}
