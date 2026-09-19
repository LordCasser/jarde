//! P3 1.3d acceptance: a local slot's **name** is only a name for the value a load read while the
//! slot still holds that value at the point the name is read, and a read that fails that test is
//! quoted instead of being spelled the wrong way.
//!
//! Two review findings are pinned here, both through the entry point the CLI calls
//! ([`Engine::recover_method`]) over a **real compiled sample** — the committed javac 23.0.1
//! `--release 8 -g:none` class under `tests/fixtures/p3-local-rewrite/`, whose own README states the
//! command, the digest and the bytecode of every member:
//!
//! * **P3-R1.** `post(I)I` is `iload_0; iinc 0,1; ireturn`: the value the `ireturn` reads is the one
//!   the load put on the stack *before* the increment wrote the slot. Writing `return arg0;` there
//!   states the incremented value, so the recovered method answers 8 where the original answers 7 —
//!   the text was the opposite program. The property checked is that the artifact no longer writes
//!   that statement, that the write it *can* prove is still written, and that the read it refused is
//!   named by its own BCI next to the reader's;
//! * **P3-R2.** `cast()Ljava/lang/String;` reads the value an invocation produced through a cast no
//!   rule of this slice presents. The answer may refuse to write Java for it, but it must keep the
//!   producer: the invocation at BCI 0 is written once, and it is anchored in the artifact.
//!
//! The control the R1 fix needs is here too, and it is the reason the assertion is written the way
//! it is: `bump(I)I` and `doubleIt(I)I` are `… istore_0; iload_0; ireturn` — the *same* text, a
//! write to slot 0 followed by a return of slot 0 — and there the reload really does read the slot,
//! so those two must keep `return arg0;`. A rule that refused every such body, or one that wrote
//! every slot name unconditionally, fails one half of this file or the other.
//!
//! The names are the ones P3 3.1's declaration facts produce: this sample's members are `static` and
//! declare one or two `int` parameters, so the entry states that slots 0 (and 1) are the signature's
//! and names them `arg0`/`arg1`, while a slot the body declares of its own (`saved`'s `y`, in slot 1
//! of a one-parameter member; `loopAcross`'s `y`, in slot 2 of a two-parameter one) is named
//! `localN`. `-g:none` means the class states no debug name at all, so every one of them is an
//! ordinal name and no source name is invented.
//!
//! The same disagreement is checked at the two **other** consumption points of this slice, because
//! the check is not the return's: `saved(I)I` stores the loaded value after the increment wrote the
//! slot, and `conditional(I)I` tests it in a branch. `loopAcross(II)I` is the cross-block control —
//! a value loaded before a loop whose body rewrites the slot, returned after it, which must be
//! written with the name and not refused.

use jarde::*;
use std::slice;

/// The committed sample, compiled by javac 23.0.1 `--release 8 -g:none` (see the fixture's README).
const FIXTURE: &[u8] = include_bytes!("fixtures/p3-local-rewrite/v8/LocalRewrite.class");

/// The members this sample declares, each with a body: the premise every request below rests on.
const DECLARED: [(&[u8], &[u8]); 8] = [
    (b"post", b"(I)I"),
    (b"bump", b"(I)I"),
    (b"doubleIt", b"(I)I"),
    (b"saved", b"(I)I"),
    (b"conditional", b"(I)I"),
    (b"loopAcross", b"(II)I"),
    (b"cast", b"()Ljava/lang/String;"),
    (b"make", b"()Ljava/lang/Object;"),
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

/// One caller domain rooted at the fixture's own snapshot, and nothing else: the simplest
/// environment the library's validator accepts without a problem.
fn environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
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

/// The opened sample and the class identity the reader's own header read stated for it.
struct Fixture {
    snapshot: ArtifactSnapshot,
    class_bytes: ClassBytesId,
}

/// Opens the committed sample and reads its header through the reader's own entry point, so that the
/// members presented below are the ones the class declares rather than the ones this file claims.
fn fixture(engine: &Engine) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(FIXTURE.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    let declared: Vec<Vec<u8>> = inspected
        .inspection
        .header
        .methods
        .iter()
        .map(|member| member.name.raw().0.clone())
        .collect();
    for (name, _) in DECLARED {
        assert!(
            declared.iter().any(|member| member.as_slice() == name),
            "the sample declares the member this case presents, `{}`: {declared:?}",
            String::from_utf8_lossy(name)
        );
    }
    Fixture {
        snapshot,
        class_bytes: inspected.source.class_bytes.clone(),
    }
}

/// One recovery run over one member of the sample, through the entry point the CLI calls.
fn recover(engine: &Engine, fixture: &Fixture, name: &[u8], descriptor: &[u8]) -> RecoveryReport {
    recover_all(
        engine,
        fixture,
        name,
        descriptor,
        &mut Budget::new(limits()),
    )
    .recovery()
    .clone()
}

/// The same run, whole: the analysis half, the presentation and what the request charged.
fn recover_all(
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

/// The bytecode indexes the artifact's own quotes name, in the order each quote states them.
///
/// A quote is the answer's statement of which bytecode it could not write: `// @bytecode 4 0` is one
/// statement naming the reader and the read it refused, and it is the machine-readable half of the
/// reason written next to it.
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

#[test]
fn a_value_the_slot_no_longer_holds_is_not_returned_through_the_slot_name() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    let report = recover(&engine, &fixture, b"post", b"(I)I");
    let text = &report.text;

    // The write the body really performs is still presented: this is a degraded read, not a body
    // that was emptied to pass the test below.
    assert!(
        text.contains("arg0 = arg0 + 1;"),
        "the increment at BCI 1 is a write this layer writes:\n{text}"
    );
    // The forbidden statement, and the reason it is forbidden *here* while the same statement is
    // required for `bump` below: this body's `ireturn` reads the value the load at BCI 0 put on the
    // stack, and the `iinc` at BCI 1 writes slot 0 before that. So `arg0` at the return denotes the
    // incremented value: writing it returns 8 where `post` returns 7.
    assert!(
        !text.contains("return arg0;"),
        "slot 0 holds the incremented value at BCI 4, so a return of the slot's name states the \
         opposite of the bytecode — `post(7)` answers 7 and the written method would answer 8:\n{text}"
    );
    // And the read it refused is not dropped silently: the quote names the load's own BCI next to
    // the reader's, which is the whole effect the statement it could not write would have carried.
    let quoted = quoted_bcis(text);
    assert!(
        quoted.contains(&0) && quoted.contains(&4),
        "the quote states the load at BCI 0 and the `ireturn` at BCI 4: {quoted:?}\n{text}"
    );
    assert!(
        text.contains("BCI 0") && text.contains("BCI 4"),
        "the reason states both bytecode indexes in words as well:\n{text}"
    );
    // The answer says of itself that it is not claiming Java for the whole body: part of it is
    // quoted bytecode, which is what a refused read is. Before the fix this run claimed Java text
    // with full structure and wrote the opposite program.
    assert_eq!(report.representation, Representation::Mixed, "{report:?}");
    assert_eq!(report.quality, Quality::Fallback, "{report:?}");
    assert!(
        report.produced(),
        "a degraded body is still an answer: {report:?}"
    );
    assert_ne!(
        report.syntax_status,
        SyntaxStatus::Checked,
        "the artifact is not claimed to be Java: {report:?}"
    );
}

/// A `Mixed` artifact's fallback is mapped and stated as completely as a structured region (P3 3.2).
///
/// The property is about the artifact's own record of itself: every bytecode a quote names is an
/// anchor of that quote, and every fallback region the report records states its code and has the
/// diagnostic the run published for it — while the *scan* the run performed stays complete. A
/// degraded presentation is not a degraded read, and `quality` is where the degradation is stated:
/// the coverage plane is not the place to move it to `Partial` (A12/A13/A16).
#[test]
fn a_mixed_artifacts_quotes_are_mapped_whole_and_its_scan_stays_complete() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    let mut budget = Budget::new(limits());
    let recovered = recover_all(&engine, &fixture, b"post", b"(I)I", &mut budget);
    let report = recovered.recovery();
    let text = &report.text;

    // The artifact is the degraded shape this case is about, and it says so.
    assert_eq!(report.representation, Representation::Mixed, "{report:?}");
    assert_eq!(report.quality, Quality::Fallback, "{report:?}");

    // The complete scan and the degraded presentation are two planes, and neither moves the other:
    // the body was decoded to its end (nothing was skipped), and the presentation *of it* is the
    // weaker one. `Partial` is a statement about what was scanned, and nothing here was not scanned.
    let analysis = recovered.analysis();
    assert_eq!(
        analysis.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema,
        "a complete scan stays complete: {analysis:?}"
    );
    assert!(
        analysis.coverage.artifact_structural.skipped.is_empty(),
        "{:?}",
        analysis.coverage.artifact_structural
    );

    // Every bytecode a quote names is an anchor of that quote: the text states which instructions
    // the quote accounts for, and the map answers for each of them — the same list, instruction for
    // instruction.
    let quoted = quoted_bcis(text);
    assert!(!quoted.is_empty(), "{text}");
    for bci in &quoted {
        assert!(
            !report.source_map.of_bci(*bci).is_empty(),
            "the quote that names BCI {bci} is the node the map answers with:\n{text}"
        );
    }
    let mut anchored: Vec<u32> = report
        .source_map
        .segments()
        .iter()
        .filter(|segment| segment.text(text).trim_start().starts_with("// @bytecode"))
        .flat_map(|segment| {
            std::iter::once(segment.origin().primary()).chain(segment.origin().derived().iter())
        })
        .map(|anchor| anchor.bci())
        .collect();
    anchored.sort_unstable();
    anchored.dedup();
    let mut named = quoted.clone();
    named.sort_unstable();
    named.dedup();
    assert_eq!(
        anchored, named,
        "the quotes' own anchors are exactly the bytecodes the quotes name:\n{text}"
    );

    // And the run's record of itself: every region is either presented as Java or quoted, and a
    // quoted region states the code it fell back for — which the run's diagnostics state too.
    let mut presented = 0usize;
    for region in &report.regions {
        if region.structured {
            presented += 1;
            continue;
        }
        let code = region.code.expect("a fallback region states its code");
        assert!(region.message.is_some(), "{region:?}");
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == code),
            "the fallback is stated in the run's diagnostics: {code} not in {:?}",
            report
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .collect::<Vec<_>>()
        );
    }
    assert!(presented > 0, "the mixed artifact is part Java:\n{text}");
}

#[test]
fn a_slot_written_and_read_again_is_still_written_with_the_slot_name() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    for member in [b"bump".as_slice(), b"doubleIt".as_slice()] {
        let report = recover(&engine, &fixture, member, b"(I)I");
        let text = &report.text;
        // These bodies write the slot and then load it: the value the return reads *is* what the
        // slot holds at the return, so the name is the right expression and refusing it would lose
        // a body this layer can present.
        assert!(
            text.contains("return arg0;"),
            "{}: `iload_0` after the write reads slot 0 again, so the name denotes exactly the \
             value being returned:\n{text}",
            String::from_utf8_lossy(member)
        );
        // Nothing was refused here: the control's whole point is that the check admits this shape.
        assert!(
            quoted_bcis(text).is_empty(),
            "{}: this body presents every instruction it has:\n{text}",
            String::from_utf8_lossy(member)
        );
        assert_eq!(report.representation, Representation::Java, "{report:?}");
        assert_eq!(report.quality, Quality::Structured, "{report:?}");
    }
}

#[test]
fn a_store_of_a_superseded_load_is_refused_with_the_read_named() {
    let engine = Engine::new();
    let fixture = fixture(&engine);

    // A **store** consumes the loaded value after the increment wrote the slot: `int y = x++` keeps
    // the value the load produced, so `local1 = arg0;` would carry the incremented value instead.
    let report = recover(&engine, &fixture, b"saved", b"(I)I");
    let text = &report.text;
    assert!(
        text.contains("arg0 = arg0 + 1;") && text.contains("return local1;"),
        "the write and the return this body does have are still presented:\n{text}"
    );
    assert!(
        !text.contains("local1 = arg0;"),
        "the store at BCI 4 writes the value the load at BCI 0 produced, and slot 0 holds the \
         incremented value by then: naming the slot would store the wrong value:\n{text}"
    );
    let quoted = quoted_bcis(text);
    assert!(
        quoted.contains(&4) && quoted.contains(&0),
        "the quote states the store at BCI 4 and the load it could not name, BCI 0: {quoted:?}\n{text}"
    );
    assert!(
        text.contains("BCI 0") && text.contains("BCI 4"),
        "and the reason says so in words:\n{text}"
    );
    assert_eq!(report.representation, Representation::Mixed, "{report:?}");
}

#[test]
fn a_branch_on_a_superseded_load_is_refused_with_the_read_named() {
    let engine = Engine::new();
    let fixture = fixture(&engine);

    // A **branch** consumes it: the test at BCI 4 runs after the slot was incremented, so a
    // condition written from the slot's name would test the wrong value. The whole region is
    // quoted, and the quote names the load as well as the region's own bytecode.
    let report = recover(&engine, &fixture, b"conditional", b"(I)I");
    let text = &report.text;
    assert!(
        !text.contains("if (arg0") && !text.contains("return arg0"),
        "the condition at BCI 4 tests the value the load produced, and slot 0 is written at BCI 1 \
         before it: neither the condition nor a value read later may be written from the slot:\n{text}"
    );
    let quoted = quoted_bcis(text);
    assert!(
        quoted.contains(&4) && quoted.contains(&0),
        "the quote states the branch at BCI 4 and the load at BCI 0 it could not name: \
         {quoted:?}\n{text}"
    );
    assert!(
        text.contains("BCI 0") && text.contains("BCI 4"),
        "and the reason says so in words:\n{text}"
    );
    assert_eq!(report.representation, Representation::Mixed, "{report:?}");
}

#[test]
fn a_value_loaded_before_a_loop_is_still_named_after_the_body_rewrote_the_slot() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    let report = recover(&engine, &fixture, b"loopAcross", b"(II)I");
    let text = &report.text;
    // The control for the other direction of the check: the value `y` holds was loaded before the
    // loop, and the loop body rewrites slot 0 afterwards. That is *not* a disagreement — the name
    // is read where the slot still holds the value the load put in it — so nothing may be refused.
    assert!(
        text.contains("return local2;"),
        "the value loaded before the loop is returned by its own local's name:\n{text}"
    );
    assert!(
        quoted_bcis(text).is_empty(),
        "nothing in this body had to be quoted:\n{text}"
    );
    assert_eq!(report.representation, Representation::Java, "{report:?}");
    assert_eq!(report.quality, Quality::Structured, "{report:?}");
}

#[test]
fn the_producer_behind_a_cast_that_this_slice_cannot_present_stays_in_the_answer() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    let report = recover(&engine, &fixture, b"cast", b"()Ljava/lang/String;");
    let text = &report.text;

    // P3-R2: the invocation at BCI 0 produced the value the cast and the `areturn` read. Whether or
    // not the cast is presented, that call is the effect of the member and cannot vanish from the
    // answer — and it is written **once**, since writing it twice would run it twice.
    assert!(
        text.contains("make("),
        "the invocation that produced the cast value is in the answer:\n{text}"
    );
    assert_eq!(
        text.matches("make(").count(),
        1,
        "the producer is written exactly once:\n{text}"
    );
    assert!(
        !report.source_map.of_bci(0).is_empty(),
        "and it is anchored in the artifact by its own BCI 0: {:?}",
        report.source_map.segments()
    );
}
