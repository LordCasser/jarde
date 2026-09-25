//! P3 stage A acceptance: the **position** a nested expression's checks are taken at, the producers
//! a refused expression owes the answer, and the grouping the printed text owes the tree.
//!
//! Three findings are pinned here, all through the entry point the CLI calls
//! ([`Engine::recover_method`]) over **real compiled samples** — the committed javac 23.0.1
//! `--release 8 -g:none` classes under `tests/fixtures/p3-nested-eval/`,
//! `tests/fixtures/p3-refused-cast/` and `tests/fixtures/p3-nested-arithmetic/`, whose own READMEs
//! state the command, the digests and the bytecode of every member:
//!
//! * **P3-R8.** `nestedLocal(I)I` is `iload_0; iconst_1; iadd; iinc 0,1; iload_0; iadd; ireturn`: the
//!   outer `iadd` reads a value the `iload_0` at BCI **0** produced, and the `iinc` at BCI 3 writes
//!   slot 0 *before* the outer sum is evaluated. The check that a slot's name denotes the value a
//!   load read is therefore a statement about BCI **8** — where the generated text evaluates it —
//!   and not about BCI 2, where the inner `iadd` was produced. Judging the load at the producer's
//!   BCI passes it, and the artifact writes `return arg0 + 1 + arg0;` after `arg0 = arg0 + 1;`,
//!   which answers 17 where `nestedLocal(7)` answers 16. The same rule is checked at the *call*
//!   argument: `nestedCall(I)I` is `iload_0; invokestatic tick; iinc 0,1; iload_0; iadd; ireturn`,
//!   and the text `tick(arg0) + arg0` after the increment would call `tick` on the incremented
//!   value. The recovered body now binds the deferred call before the write —
//!   `int saved0 = tick(arg0); arg0 = arg0 + 1; return saved0 + arg0;` — so the original argument
//!   and one-call effect stay intact while `nestedLocal` continues to quote the value it cannot
//!   name.
//! * **P3-R9.** `fieldCast()Ljava/lang/String;` is `getstatic External.value; checkcast; areturn`.
//!   Historically the cast was refused. The current refusal tests replace the final return with
//!   `pop; aconst_null; areturn` in memory, since ordinary casts now recover. The `getstatic`
//!   writes no statement of its own (a claimed field
//!   *read* is a value: its text lands where it is consumed) — so before the fix the artifact
//!   quoted BCI 3 and 6 alone and named BCI 0 nowhere, while `report.fields` still recorded that
//!   read as `presented`. Running `External`'s static initializer is an observable effect of that
//!   bytecode, so a quote that does not name it has dropped an effect. The instance and chain
//!   shapes are the same rule: `instanceCast` is `aload_0; getfield External.instance; checkcast;
//!   areturn`, whose read can throw (`instanceCast(null)`), and `chainCast` is `getstatic
//!   External.holder; getfield Holder.value; checkcast; areturn`, where the walk must not stop at
//!   the first read: the `getfield`'s own producer, the `getstatic` at BCI 0, is named too.
//! * **Grouping.** The expression tree is the layer's proof and the text is what a caller
//!   receives, so the printer has to state the tree's grouping: `ModLike.inverse32(I)I` is
//!   `iload_1; iconst_2; iload_0; iload_1; imul; isub; imul; istore_1` four times, whose tree is
//!   `local1 * (2 - arg0 * local1)`. Printed by concatenating the operands it came out as
//!   `local1 = local1 * 2 - arg0 * local1;` — `(local1 * 2) - (arg0 * local1)`, another program —
//!   while the run reported `Java`/`Structured`/`contains_statements` and no diagnostic; the
//!   compiled text answered `-81` for `inverse32(-1)` where the class answers `-1`. The four
//!   shapes of the fixture (a product of a subtraction, a subtraction of a subtraction, a
//!   quotient of a product, a difference of a scaled sum) and the two controls whose tree Java's
//!   defaults already state are asserted below by **exact text**, with the executed comparison in
//!   `tests/p3_execution_comparison.rs` as the value evidence.
//! * **The position, not only the parent operator.** The same sample's texts are one evidence
//!   that grouping was decided per **parent node**: a binary operand of the *same* tree shape is
//!   grouped, and a subexpression in any other position is not. `ReceiverGrouping.call` is
//!   `(a + b).substring(1)` — `aload_0; aload_1; append; append; toString; iconst_1; substring` —
//!   whose receiver is the concatenation `a + b`; printed by concatenating the receiver and then
//!   the `.`, the text was `return arg0 + arg1.substring(1);`, which Java reads as
//!   `arg0 + (arg1.substring(1))`. With `("a", "bc")` the class answers `"bc"` and that text
//!   answers `"ac"`, while the run still reported `Java`/`Structured`/`contains_statements` with
//!   no diagnostic. `ReceiverGrouping.length` is the review's second shape (`(a + b).length()`,
//!   whose ungrouped text does not even compile under its `int` return) and `nested` is a
//!   three-operand chain in receiver position; the sample's `plain`, `chained`, `same` and
//!   `argument` members are the controls that must gain nothing (a name receiver, a nested call, a
//!   left-associative chain of the same precedence and a call argument).
//!
//! The controls are what keep the fix from being "refuse every nested expression", "quote every
//! field read" and "parenthesise everything":
//!
//! * `nestedPlain(I)I` is `(x + 1) + (x + 2)`: the same nested-arithmetic shape with **no write
//!   between the loads and the sum**, so both loads still hold their values where the text is
//!   evaluated and the whole body must stay `Java`/`Structured` with the slot names (its right
//!   operand keeps its group — `arg0 + 1 + (arg0 + 2)` — because a left-associative
//!   `arg0 + 1 + arg0 + 2` parses back into a different tree);
//! * `leftRead()` and `rightRead()` read a claimed static field and call once, in either order
//!   (`External.count + tick()` and `tick() + External.count`): a field read composed with a
//!   deferred call is presented, not quoted, and `tick` appears exactly once in each text — the
//!   reasoning that *names* an unaccounted read must not turn a written one into a refusal.
//!
//! The names are the ones the declaration facts produce: the members are `static` and declare at
//! most one `int` or `External` parameter, so slot 0 is named `arg0`; `-g:none` means the class
//! states no debug name and no source name is invented.

use jarde::*;
use std::slice;

/// The committed sample of P3-R8, compiled by javac 23.0.1 `--release 8 -g:none` (see the fixture's
/// README).
const NESTED: &[u8] = include_bytes!("fixtures/p3-nested-eval/v8/NestedEval.class");
/// The committed sample of P3-R9, compiled the same way (see the fixture's README).
const REFUSED: &[u8] = include_bytes!("fixtures/p3-refused-cast/v8/RefusedCast.class");
/// The committed sample of the nested-arithmetic grouping defect, compiled the same way (see the
/// fixture's README).
const NESTED_ARITHMETIC: &[u8] = include_bytes!("fixtures/p3-nested-arithmetic/v8/ModLike.class");
/// The committed sample of the receiver-position grouping defect, compiled the same way (see the
/// fixture's README).
const RECEIVER_GROUPING: &[u8] =
    include_bytes!("fixtures/p3-receiver-grouping/v8/ReceiverGrouping.class");

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

/// The opened sample and the class identity the reader's own header read stated for it.
struct Fixture {
    snapshot: ArtifactSnapshot,
    class_bytes: ClassBytesId,
}

/// Opens one committed sample and reads its header through the reader's own entry point, so that
/// the members presented below are the ones the class declares rather than the ones this file
/// claims.
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
        .recover_method_with_evidence(
            slice::from_ref(&fixture.snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits()),
        )
        .expect("a legal request is answered, not raised")
        .recovery()
        .clone()
}

/// The bytecode indexes the artifact's own quotes name, in the order each quote states them.
///
/// A quote is the answer's statement of which bytecode it could not write: `// @bytecode 8 0` is one
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

/// Make one refused-cast consumer explicit without a class-file parser. The three committed
/// methods below have unique complete Code byte sequences and no trailing instructions; replacing
/// their final `areturn` with `pop; aconst_null; areturn` keeps the declared String return type while
/// making the cast result genuinely unused. The two enclosing Code lengths are adjusted for the
/// inserted bytes, and every other class byte stays unchanged.
fn cast_result_is_popped(bytes: &[u8], code: &[u8]) -> Vec<u8> {
    assert!(code.ends_with(&[0xb0]), "the patch target ends in areturn");
    let sites: Vec<usize> = bytes
        .windows(code.len())
        .enumerate()
        .filter_map(|(index, window)| (window == code).then_some(index))
        .collect();
    assert_eq!(
        sites.len(),
        1,
        "the complete method Code is unique in the frozen fixture"
    );
    let start = sites[0];
    let code_length_at = start
        .checked_sub(4)
        .expect("Code length precedes the method code");
    let attribute_length_at = start
        .checked_sub(12)
        .expect("Code attribute length precedes the method code");
    let old_code_length = u32::from_be_bytes(
        bytes[code_length_at..code_length_at + 4]
            .try_into()
            .expect("Code length is four bytes"),
    );
    assert_eq!(
        old_code_length,
        code.len() as u32,
        "the frozen Code length matches"
    );
    let old_attribute_length = u32::from_be_bytes(
        bytes[attribute_length_at..attribute_length_at + 4]
            .try_into()
            .expect("Code attribute length is four bytes"),
    );
    let mut patched = bytes.to_vec();
    patched.splice(start + code.len() - 1..start + code.len() - 1, [0x57, 0x01]);
    patched[code_length_at..code_length_at + 4]
        .copy_from_slice(&(old_code_length + 2).to_be_bytes());
    patched[attribute_length_at..attribute_length_at + 4]
        .copy_from_slice(&(old_attribute_length + 2).to_be_bytes());
    patched
}

/// The member a field record names, spelled the way the artifact spells it (`External.value`).
fn spelled(owner: &str, name: &str) -> String {
    format!("{}.{}", owner.replace('/', "."), name)
}

/// A presented read needs its own field expression span. A quoted BCI has a source span too, so
/// merely finding the BCI in the map does not establish field presentation.
fn presented_reads_are_accounted_for(member: &str, report: &RecoveryReport) {
    for record in report
        .fields
        .iter()
        .filter(|record| record.presented && record.access == "read")
    {
        assert!(
            report
                .source_map
                .direct_of_bci(record.bci)
                .iter()
                .any(|segment| {
                    let text = segment.text(&report.text);
                    text.contains(&record.name) && !text.contains("@bytecode")
                }),
            "{member}: BCI {} has no direct field expression for `{}`:\n{}",
            record.bci,
            spelled(&record.owner, &record.name),
            report.text
        );
    }
}

/// The control that keeps the R8 fix from refusing every nested arithmetic, and the two controls
/// that keep the R9 fix from quoting every field read.
#[test]
fn a_nested_expression_that_crosses_no_write_is_still_written() {
    let engine = Engine::new();
    let nested = fixture(&engine, NESTED);

    // The control: the same left-nested `+` shape with no write between the two loads and the sum.
    // Both loads still hold what they read where the text is evaluated, so nothing is refused. The
    // parentheses are the printer's grouping and not a refusal: the tree is `Add(Add(x, 1),
    // Add(x, 2))`, and the left-associative text `arg0 + 1 + arg0 + 2` parses back as
    // `((arg0 + 1) + arg0) + 2` — a different tree — so the right operand keeps its own group.
    let report = recover(&engine, &nested, b"nestedPlain", b"(I)I");
    let text = &report.text;
    assert!(
        text.contains("return arg0 + 1 + (arg0 + 2);"),
        "nestedPlain is `(x + 1) + (x + 2)` and the slot still holds what each load read where the \
         sum is evaluated, so the names are the right expressions:\n{text}"
    );
    assert!(
        quoted_bcis(text).is_empty(),
        "nothing in this body had to be quoted:\n{text}"
    );
    assert_eq!(report.representation, Representation::Java, "{report:?}");
    assert_eq!(report.quality, Quality::Structured, "{report:?}");

    // The two field-read controls: one claimed static read composed with one deferred call, in both
    // orders. A read that *is* presented keeps its text, and the call it is composed with is written
    // exactly once — naming an unaccounted read must not cost a written one its statement.
    let refused = fixture(&engine, REFUSED);
    for (member, expected) in [
        (b"leftRead".as_slice(), "return External.count + tick();"),
        (b"rightRead".as_slice(), "return tick() + External.count;"),
    ] {
        let report = recover(&engine, &refused, member, b"()I");
        let text = &report.text;
        let name = String::from_utf8_lossy(member);
        assert!(
            text.contains(expected),
            "{name}: the field read and the call are both written where the return reads them:\n{text}"
        );
        assert_eq!(
            text.matches("tick(").count(),
            1,
            "{name}: the call is written exactly once:\n{text}"
        );
        assert_eq!(report.representation, Representation::Java, "{report:?}");
        assert_eq!(report.quality, Quality::Structured, "{report:?}");
        assert!(
            quoted_bcis(text).is_empty(),
            "{name}: this body presents every instruction it has:\n{text}"
        );
        presented_reads_are_accounted_for(&name, &report);
    }
}

/// P3-R8, at a local read: the check is taken where the generated text is evaluated, not where the
/// value was produced.
#[test]
fn a_nested_arithmetic_is_checked_where_its_text_is_evaluated() {
    let engine = Engine::new();
    let fixture = fixture(&engine, NESTED);
    let report = recover(&engine, &fixture, b"nestedLocal", b"(I)I");
    let text = &report.text;

    // The write the body really performs is still presented: this is a degraded read, not a body
    // that was emptied to pass the test below.
    assert!(
        text.contains("arg0 = arg0 + 1;"),
        "the increment at BCI 3 is a write this layer writes:\n{text}"
    );
    // The forbidden statement, and the reason it is forbidden: the outer `iadd` at BCI 7 reads the
    // value the load at BCI 0 produced, and the `iinc` at BCI 3 wrote slot 0 before BCI 8 evaluates
    // it. `arg0 + 1 + arg0` after `arg0 = arg0 + 1;` is `7 + 1 + 8`: the original `nestedLocal(7)`
    // answers 16 and the written method would answer 17.
    assert!(
        !text.contains("return arg0 + 1 + arg0;"),
        "the load at BCI 0 no longer denotes what slot 0 holds at BCI 8, so the return may not name \
         the slot there: `nestedLocal(7)` is 16 and `return arg0 + 1 + arg0;` after \
         `arg0 = arg0 + 1;` is 17:\n{text}"
    );
    // And the read it refused is not dropped silently: the quote names the consumer's own BCI next
    // to the load's, which is the whole effect the statement it could not write would have carried.
    let quoted = quoted_bcis(text);
    assert!(
        quoted.contains(&8) && quoted.contains(&0),
        "the quote states the `ireturn` at BCI 8 and the load at BCI 0 it could not name: \
         {quoted:?}\n{text}"
    );
    assert!(
        text.contains("BCI 8") && text.contains("BCI 0"),
        "and the reason states both bytecode indexes in words:\n{text}"
    );
    assert!(
        !report.text_of_bci(0).is_empty(),
        "the load's own bytecode is an anchor of the quote:\n{text}"
    );
    // The answer says of itself that it is not claiming Java for the whole body: part of it is
    // quoted bytecode, which is what a refused read is. Before the fix this run claimed Java text
    // with full structure and wrote the other program.
    assert_eq!(report.representation, Representation::Mixed, "{report:?}");
    assert_eq!(report.quality, Quality::Fallback, "{report:?}");
    assert!(report.produced(), "a degraded body is still an answer");
}

/// P3-R8, at a call argument: bind the call's value before the following write changes the slot.
#[test]
fn a_call_argument_is_saved_before_a_following_write() {
    let engine = Engine::new();
    let fixture = fixture(&engine, NESTED);
    let report = recover(&engine, &fixture, b"nestedCall", b"(I)I");
    let text = &report.text;

    assert!(
        text.contains("int saved0 = tick(arg0);"),
        "the call argument is evaluated and saved before the slot mutation:\n{text}"
    );
    assert!(
        text.contains("arg0 = arg0 + 1;"),
        "the increment at BCI 4 remains a statement after the saved call:\n{text}"
    );
    assert!(
        text.contains("return saved0 + arg0;"),
        "the outer sum consumes the saved call value after the mutation:\n{text}"
    );
    assert!(
        text.matches("tick(").count() == 1,
        "the deferred call is emitted exactly once:\n{text}"
    );
    let saved = text
        .find("int saved0 = tick(arg0);")
        .expect("the saved call binding is present");
    let increment = text
        .find("arg0 = arg0 + 1;")
        .expect("the increment is present");
    let returned = text
        .find("return saved0 + arg0;")
        .expect("the return is present");
    assert!(
        saved < increment && increment < returned,
        "the source order follows call, mutation, return:\n{text}"
    );
    assert!(
        quoted_bcis(text).is_empty(),
        "the saved call is fully structured instead of quoted:\n{text}"
    );
    assert_eq!(report.representation, Representation::Java, "{report:?}");
    assert_eq!(report.quality, Quality::Structured, "{report:?}");
    for bci in [0, 1, 4, 9] {
        assert!(
            !report.text_of_bci(bci).is_empty(),
            "the source map accounts for BCI {bci}:\n{text}"
        );
    }
}

/// P3-R9, static read: the quote names the `getstatic` whose class initialization it can run.
#[test]
fn a_refused_static_read_keeps_the_class_initialization_it_can_run() {
    let engine = Engine::new();
    let fixture = fixture(
        &engine,
        &cast_result_is_popped(REFUSED, &[0xb2, 0x00, 0x0d, 0xc0, 0x00, 0x13, 0xb0]),
    );
    let report = recover(&engine, &fixture, b"fieldCast", b"()Ljava/lang/String;");
    let text = &report.text;

    // The patched bytecode is `getstatic External.value; checkcast; pop; aconst_null; areturn`:
    // the field read and the failed cast have no supported consumer, while the final null return
    // remains a valid Java statement. Reading the field can still run External's static initializer.
    let quoted = quoted_bcis(text);
    for bci in [0u32, 3, 6] {
        assert!(
            quoted.contains(&bci),
            "the quote names the `getstatic` at BCI 0, the `checkcast` at BCI 3 and the `pop` at \
             BCI 6: {quoted:?}\n{text}"
        );
    }
    assert!(
        text.contains("return null;"),
        "the patched final return remains valid:\n{text}"
    );
    assert!(
        !report.text_of_bci(0).is_empty(),
        "the read at BCI 0 is an anchor of the quote that accounts for it:\n{text}"
    );
    let read = report
        .fields
        .iter()
        .find(|record| record.bci == 0)
        .expect("the run read the field instruction at BCI 0");
    assert!(
        !read.presented
            && read
                .refusal
                .as_ref()
                .is_some_and(|reason| reason.code == "jre_field_not_emitted"),
        "the quoted static read is traceable but not presented: {read:?}"
    );
    presented_reads_are_accounted_for("fieldCast", &report);
    assert_eq!(report.representation, Representation::Mixed, "{report:?}");
    assert_eq!(report.quality, Quality::Fallback, "{report:?}");
}

/// P3-R9, instance read: the quote names the `getfield` whose `NullPointerException` it can throw.
#[test]
fn a_refused_instance_read_keeps_the_null_pointer_it_can_throw() {
    let engine = Engine::new();
    let fixture = fixture(
        &engine,
        &cast_result_is_popped(REFUSED, &[0x2a, 0xb4, 0x00, 0x15, 0xc0, 0x00, 0x13, 0xb0]),
    );
    let report = recover(
        &engine,
        &fixture,
        b"instanceCast",
        b"(LExternal;)Ljava/lang/String;",
    );
    let text = &report.text;

    // The patched bytecode is `aload_0; getfield External.instance; checkcast; pop; aconst_null;
    // areturn`: the getfield at BCI 1 still dereferences the argument, so the original throws for a
    // null receiver — the fixture's driver runs it — and a quote that dropped the read would drop
    // that observable failure too.
    let quoted = quoted_bcis(text);
    for bci in [1u32, 4, 7] {
        assert!(
            quoted.contains(&bci),
            "the quote names the `getfield` at BCI 1, the `checkcast` at BCI 4 and `pop` at BCI 7: \
             {quoted:?}\n{text}"
        );
    }
    assert!(
        text.contains("return null;"),
        "the patched final return remains valid:\n{text}"
    );
    assert!(
        !report.text_of_bci(1).is_empty(),
        "the read at BCI 1 is an anchor of the quote that accounts for it:\n{text}"
    );
    let read = report
        .fields
        .iter()
        .find(|record| record.bci == 1)
        .expect("the run read the field instruction at BCI 1");
    assert!(
        !read.presented
            && !read.is_static
            && read
                .refusal
                .as_ref()
                .is_some_and(|reason| reason.code == "jre_field_not_emitted"),
        "the quoted instance read is traceable but not presented: {read:?}"
    );
    presented_reads_are_accounted_for("instanceCast", &report);
    assert_eq!(report.representation, Representation::Mixed, "{report:?}");
    assert_eq!(report.quality, Quality::Fallback, "{report:?}");
}

/// P3-R9, a read behind a read: the walk does not stop at the first read it names.
#[test]
fn a_read_behind_another_read_is_named_too() {
    let engine = Engine::new();
    let fixture = fixture(
        &engine,
        &cast_result_is_popped(
            REFUSED,
            &[0xb2, 0x00, 0x18, 0xb4, 0x00, 0x1c, 0xc0, 0x00, 0x13, 0xb0],
        ),
    );
    let report = recover(&engine, &fixture, b"chainCast", b"()Ljava/lang/String;");
    let text = &report.text;

    // The patched bytecode is `getstatic External.holder; getfield Holder.value; checkcast; pop;
    // aconst_null; areturn`: the `getfield` at BCI 3 reads the value the `getstatic` at BCI 0
    // produced, so naming the read that the refusal consumed means naming the read behind it too.
    let quoted = quoted_bcis(text);
    for bci in [0u32, 3, 6, 9] {
        assert!(
            quoted.contains(&bci),
            "the quote names both reads (BCI 0 and BCI 3), the checkcast (BCI 6) and the pop \
             (BCI 9): {quoted:?}\n{text}"
        );
    }
    assert!(
        text.contains("return null;"),
        "the patched final return remains valid:\n{text}"
    );
    for bci in [0u32, 3] {
        assert!(
            !report.text_of_bci(bci).is_empty(),
            "the read at BCI {bci} is an anchor of the quote that accounts for it:\n{text}"
        );
    }
    presented_reads_are_accounted_for("chainCast", &report);
    assert_eq!(report.representation, Representation::Mixed, "{report:?}");
    assert_eq!(report.quality, Quality::Fallback, "{report:?}");
}

/// The report-wide half of P3-R9: no member of either sample may claim a presented field read that
/// the artifact it came with accounts for nowhere.
#[test]
fn every_presented_field_read_is_accounted_for_by_its_artifact() {
    let engine = Engine::new();
    let nested = fixture(&engine, NESTED);
    for (name, descriptor) in [
        (b"nestedLocal".as_slice(), b"(I)I".as_slice()),
        (b"nestedPlain".as_slice(), b"(I)I".as_slice()),
        (b"nestedCall".as_slice(), b"(I)I".as_slice()),
        (b"tick".as_slice(), b"(I)I".as_slice()),
    ] {
        let report = recover(&engine, &nested, name, descriptor);
        presented_reads_are_accounted_for(&String::from_utf8_lossy(name), &report);
    }
    let refused = fixture(&engine, REFUSED);
    for (name, descriptor) in [
        (b"fieldCast".as_slice(), b"()Ljava/lang/String;".as_slice()),
        (
            b"instanceCast".as_slice(),
            b"(LExternal;)Ljava/lang/String;".as_slice(),
        ),
        (b"chainCast".as_slice(), b"()Ljava/lang/String;".as_slice()),
        (b"leftRead".as_slice(), b"()I".as_slice()),
        (b"rightRead".as_slice(), b"()I".as_slice()),
        (b"tick".as_slice(), b"()I".as_slice()),
    ] {
        let report = recover(&engine, &refused, name, descriptor);
        presented_reads_are_accounted_for(&String::from_utf8_lossy(name), &report);
    }
}

// ---------------------------------------------------------------------------------------------
// `bound-recovery-recursion`: the region walk's own recursion, and the stop that now answers it
// ---------------------------------------------------------------------------------------------

/// The body of [`recursion_fixture`]: a latch-tested loop whose body starts at its own header.
///
/// The instructions, with the blocks the canonical graph makes of them — an inner loop's back edge
/// is what ends the outer header's block:
///
/// ```text
///   0: iconst_0          ◄ the prologue
///   1: istore_0
///   2: iconst_0          ◄ H — the outer loop's header *and* its back edge's target (22 → 2)
///   3: istore_1          (i = 0)
///   4: iload_1           ← the inner loop's header, and the target of its own `goto` (17 → 4)
///   5: iconst_3
///   6: if_icmpge 20
///   9: iload_0; iconst_1; iadd; istore_0       (x = x + 1)
///  13: iload_1; iconst_1; iadd; istore_1       (i = i + 1)
///  17: goto 4
///  20: iload_0           ◄ L — the latch, a *separate* block whose whole test is loads and a branch
///  21: iconst_5
///  22: if_icmplt 2       (22 + (−20) = 2: the back edge)
///  25: return
/// ```
///
/// The bytes are the ones `javac 23.0.1 --release 8 -g:none` emits for
/// `int x = 0; int i; do { for (i = 0; i < 3; i = i + 1) { x = x + 1; } } while (x < 5);` — written
/// out here rather than compiled at run time, with the same `StackMapTable` javac declares (see
/// [`recursion_fixture`]).
const RECURSION_BODY: &[u8] = &[
    0x03, // 0: iconst_0
    0x3b, // 1: istore_0
    0x03, // 2: iconst_0      ← H
    0x3c, // 3: istore_1
    0x1b, // 4: iload_1       ← the inner loop's header
    0x06, // 5: iconst_3
    0xa2, 0x00, 0x0e, // 6: if_icmpge 20
    0x1a, // 9: iload_0
    0x04, // 10: iconst_1
    0x60, // 11: iadd
    0x3b, // 12: istore_0
    0x1b, // 13: iload_1
    0x04, // 14: iconst_1
    0x60, // 15: iadd
    0x3c, // 16: istore_1
    0xa7, 0xff, 0xf3, // 17: goto 4
    0x1a, // 20: iload_0      ← L
    0x08, // 21: iconst_5
    0xa1, 0xff, 0xec, // 22: if_icmplt 2
    0xb1, // 25: return
];

/// One hand-assembled class whose only method drives the region walk's own cycle.
///
/// This is the controlled fixture of `bound-recovery-recursion` (its task 3.1), and the shape is
/// the one that change's localization found on the real repro — `javassist/bytecode/CodeAnalyzer`'s
/// `computeMaxStack()I` from `S2-007.war`, whose recovery aborted the process with
/// `fatal runtime error: stack overflow, aborting` before the guard existed. What it mirrors is the
/// **driver**, not the vendor's bytes:
///
/// * the loop is **latch-tested**: its header H is not itself a test block (so `header_tested_loop`
///   finds nothing and `latch_tested_loop` is the shape that applies), and its latch L is a
///   *different* block. H's block ends where the inner loop's `goto` target begins, which is what
///   the real driver's header does too — an inner loop's back edge, not a second test;
/// * the body walk of that shape starts **at H** (`latch_tested_loop` walks from the header), and a
///   walk that starts at the header of the loop whose body it is walking is inside the state it is
///   already in: `loop_region` would take the same decisions — none of them reads the frame — and
///   come back to the same call, which is what ran the stack out.
///
/// The bytes are written here rather than committed, and no compiler runs: the fixture has to be a
/// class whose *walk* cycles, and committing a vendor's bytes for that would tie the regression to
/// an artifact this repository does not own. The pre-fix behaviour of exactly these bytes is
/// recorded once in the change's verification instead of being asserted here (`exit 134` and
/// `fatal runtime error: stack overflow, aborting` for the file this generator writes); what the
/// test below asserts is the bound's answer, not a crash. The class is built the way
/// `crates/jarde-cli/tests/task_cli.rs`'s writer builds the same member, so this change's two
/// acceptance entries — the library's and the process's — run on the *same* bytes.
///
/// The `StackMapTable` is not decoration: a version-52 class whose code has a back edge declares
/// the frames the verifier starts its blocks with, and without them this engine's frame pass
/// refuses to derive the body (the recovery then stops on `jre_ir_table_missing`, a different
/// stop). The two frames here are the ones javac declares for the same shape: an `append_frame`
/// at BCI 2 (locals `[]` + `[int]`), an `append_frame` at BCI 4 (locals `[int, int]`) and a
/// `same_frame` at BCI 20 (its delta is 15).
fn recursion_fixture() -> Vec<u8> {
    fn u16b(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    fn u32b(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    fn utf8(pool: &mut Vec<u8>, text: &[u8]) {
        pool.push(1);
        u16b(
            pool,
            u16::try_from(text.len()).expect("fixture name fits u16"),
        );
        pool.extend_from_slice(text);
    }

    let mut pool: Vec<u8> = Vec::new();
    utf8(&mut pool, b"p/Recursive"); // 1
    pool.push(7); // 2: Class 1
    u16b(&mut pool, 1);
    utf8(&mut pool, b"java/lang/Object"); // 3
    pool.push(7); // 4: Class 3
    u16b(&mut pool, 3);
    utf8(&mut pool, b"method"); // 5
    utf8(&mut pool, b"()V"); // 6
    utf8(&mut pool, b"Code"); // 7
    utf8(&mut pool, b"StackMapTable"); // 8

    let mut output = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0); // minor
    u16b(&mut output, 52); // major: Java 8
    u16b(&mut output, 9); // constant_pool_count
    output.extend_from_slice(&pool);
    u16b(&mut output, 0x0021); // public super
    u16b(&mut output, 2); // this_class
    u16b(&mut output, 4); // super_class
    u16b(&mut output, 0); // interfaces
    u16b(&mut output, 0); // fields
    u16b(&mut output, 1); // methods
    u16b(&mut output, 0x0009); // public static
    u16b(&mut output, 5); // name → "method"
    u16b(&mut output, 6); // descriptor → "()V"
    u16b(&mut output, 1); // attributes
    u16b(&mut output, 7); // "Code"
    let mut code = Vec::new();
    u16b(&mut code, 2); // max_stack
    u16b(&mut code, 2); // max_locals
    u32b(
        &mut code,
        u32::try_from(RECURSION_BODY.len()).expect("fixture body fits u32"),
    );
    code.extend_from_slice(RECURSION_BODY);
    u16b(&mut code, 0); // exception table
    u16b(&mut code, 1); // code attributes
    u16b(&mut code, 8); // "StackMapTable"
    let frames: [u8; 11] = [
        0x00, 0x03, // number_of_entries
        0xfc, 0x00, 0x02, 0x01, // append_frame: BCI 2, +[int]
        0xfc, 0x00, 0x01, 0x01, // append_frame: BCI 4, +[int]
        0x0f, // same_frame: BCI 20 (delta 15)
    ];
    u32b(
        &mut code,
        u32::try_from(frames.len()).expect("fixture frames fit u32"),
    );
    code.extend_from_slice(&frames);
    u32b(
        &mut output,
        u32::try_from(code.len()).expect("fixture attribute fits u32"),
    );
    output.extend_from_slice(&code);
    u16b(&mut output, 0); // class attributes
    output
}

/// The recursion that used to abort answers with a published stop instead.
///
/// The stop is the *existing* stop shape — `outcome = Stopped`, a non-`Complete` execution plane, a
/// diagnostic that names the reason and the position, `content = not_produced`, no text and no
/// segment table — and the reason is a re-entry, not "the input nests too deeply": the walk was
/// about to enter the header of the loop whose body it is walking, which this run has already
/// entered. The same request is run twice here: the stop the bound publishes is decided by the
/// blocks, not by the stack or by the order the tests happen to run in.
#[test]
fn a_recursion_that_re_enters_its_own_loop_stops_instead_of_aborting() {
    let engine = Engine::new();
    let fixture = fixture(&engine, &recursion_fixture());

    let report = recover(&engine, &fixture, b"method", b"()V");
    assert_eq!(
        report.outcome,
        RecoveryOutcome::Stopped(StopReason::Interrupted {
            code: "jre_recursion_reentry",
            at: Some(2),
        }),
        "the walk re-entered the header at BCI 2, and the report says so: {report:?}"
    );
    match &report.execution {
        ExecutionReport::Partial {
            reason: TerminationReason::Error { code },
            ..
        } => assert_eq!(
            code, "jre_recursion_reentry",
            "a stop is a non-Complete execution plane, never a Complete one: {report:?}"
        ),
        other => panic!("a stop is a partial execution, not {other:?}"),
    }
    assert_eq!(report.content, RecoveryContent::NotProduced);
    assert_eq!(report.text, "", "a stop hands out no artifact");
    assert_eq!(report.source_map.len(), 0, "and no segment table");
    assert!(report.regions.is_empty() && report.fallbacks.is_empty());
    assert_eq!(report.representation, Representation::Bytecode);
    assert!(!report.produced());
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "jre_recursion_reentry")
        .unwrap_or_else(|| panic!("the stop states its reason: {report:?}"));
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert!(
        diagnostic.message.contains("BCI 2") && diagnostic.message.contains("re-entered"),
        "the diagnosis names the block it re-entered and the fact that it re-entered it: {}",
        diagnostic.message
    );

    // The same input, again: the stop is decided by the walk's own state, so it is the same stop.
    let again = recover(&engine, &fixture, b"method", b"()V");
    assert_eq!(again.outcome, report.outcome);
    assert_eq!(again.diagnostics, report.diagnostics);
    assert_eq!(
        std::mem::discriminant(&again.execution),
        std::mem::discriminant(&report.execution)
    );
}

// ---------------------------------------------------------------------------------------------
// `fix-nested-arithmetic-value`: the printer's grouping, on the committed shapes
// ---------------------------------------------------------------------------------------------

/// One shape of the grouping fixture: the member, its descriptor, the statement the text must hold,
/// how many times it must hold it, and the ungrouped statement the same tree used to be printed as
/// (empty for the two shapes whose tree Java's own precedence and associativity already state).
type Shape = (
    &'static [u8],
    &'static [u8],
    &'static str,
    usize,
    &'static str,
);

#[test]
fn the_printed_text_keeps_the_arithmetic_tree() {
    let engine = Engine::new();
    let sample = fixture(&engine, NESTED_ARITHMETIC);

    // Each shape: the member, its descriptor, the statement the text must hold, and how many times
    // (a body whose four rounds are one expression, like `inverse32`, states the same line four
    // times), then the ungrouped statement the same tree used to be printed as. The ungrouped text
    // is not merely ugly: `local1 * 2 - arg0 * local1` is `(local1 * 2) - (arg0 * local1)`, a
    // different program, and `inverse32(-1)` answers -1 with the tree and -81 with the text.
    let shapes: &[Shape] = &[
        (
            b"inverse32",
            b"(I)I",
            "local1 = local1 * (2 - arg0 * local1);",
            4,
            "local1 = local1 * 2 - arg0 * local1;",
        ),
        (
            b"scaledDifference",
            b"(II)I",
            "return arg0 * (2 - arg1 * arg0);",
            1,
            "return arg0 * 2 - arg1 * arg0;",
        ),
        (
            b"nestedDifference",
            b"(III)I",
            "return arg0 - (arg1 - arg2);",
            1,
            "return arg0 - arg1 - arg2;",
        ),
        (
            b"nestedQuotient",
            b"(III)I",
            "return arg0 / (arg1 * arg2);",
            1,
            "return arg0 / arg1 * arg2;",
        ),
        (
            b"differenceOfSum",
            b"(II)I",
            "return arg0 - (arg1 + 1) * 2;",
            1,
            "return arg0 - arg1 + 1 * 2;",
        ),
        (
            b"productOfSum",
            b"(III)I",
            "return (arg0 + arg1) * arg2;",
            1,
            // The same text `sumOfProducts` must keep: a sum on the left of a multiplication binds
            // looser than the multiplication, so `arg0 + arg1 * arg2` is `arg0 + (arg1 * arg2)` —
            // a different tree than `(a + b) * c`.
            "return arg0 + arg1 * arg2;",
        ),
        // The two shapes Java's own precedence and associativity already state: they carry no
        // parentheses today and must not gain any (the fix is grouping, not decoration).
        (
            b"sumOfProducts",
            b"(III)I",
            "return arg0 + arg1 * arg2;",
            1,
            "",
        ),
        (
            b"leftNestedSum",
            b"(III)I",
            "return arg0 + arg1 + arg2;",
            1,
            "",
        ),
    ];

    let mut problems = Vec::new();
    for (member, descriptor, statement, times, ungrouped) in shapes {
        let name = String::from_utf8_lossy(member);
        let report = recover(&engine, &sample, member, descriptor);
        let text = &report.text;
        // The evidence is the exact text, because the text is what a caller receives and
        // recompiles. `representation`/`quality` are stated after it as facts about the run; they
        // are structural planes and are no evidence that the text computes what its bytecode does.
        let found = text.matches(statement).count();
        if found != *times {
            problems.push(format!(
                "{name}: the text must state `{statement}` exactly {times} time(s), and it states \
                 it {found}:\n{text}"
            ));
        }
        if !ungrouped.is_empty() && text.contains(ungrouped) {
            problems.push(format!(
                "{name}: the text still holds `{ungrouped}`, which parses back into a different \
                 tree:\n{text}"
            ));
        }
        let quoted = quoted_bcis(text);
        if !quoted.is_empty() {
            problems.push(format!(
                "{name}: this nest is provable, so the body may not be quoted: {quoted:?}\n{text}"
            ));
        }
        if report.representation != Representation::Java {
            problems.push(format!(
                "{name}: the body must stay written whole, and the run states {:?}",
                report.representation
            ));
        }
        if report.quality != Quality::Structured {
            problems.push(format!(
                "{name}: the body must stay structured, and the run states {:?}",
                report.quality
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "an arithmetic operand retains its own value only if the printed text parses back into the \
         tree it was printed from:\n{}",
        problems.join("\n")
    );
}

// ---------------------------------------------------------------------------------------------
// `group-call-receivers`: the grouping is a property of the **position**, not only of a binary
// parent — the fixture is `tests/fixtures/p3-receiver-grouping/`, whose README records the
// compiler, the command, the digests and the bytecode of every member.
// ---------------------------------------------------------------------------------------------

/// One member of the receiver-grouping sample: its name, its descriptor, the statement its text
/// must hold, and the statement the same tree was printed as before the receiver position was
/// grouped (empty for the controls that must not move).
type ReceiverShape = (&'static [u8], &'static [u8], &'static str, &'static str);

/// Every shape of the defect and every control, asserted by **exact text**: the text is what a
/// caller receives and recompiles, and the structural planes (`representation`, `quality`,
/// `content`) are stated after it as facts about the run, not as evidence that the text is the
/// program its bytecode is. The executed values are compared in
/// `tests/p3_execution_comparison.rs`.
#[test]
fn the_printed_text_keeps_the_grouping_of_every_position() {
    let engine = Engine::new();
    let sample = fixture(&engine, RECEIVER_GROUPING);
    let shapes: &[ReceiverShape] = &[
        // The reported shape: the receiver of `substring` is the concatenation `a + b`. The
        // ungrouped text `arg0 + arg1.substring(1)` is `arg0 + (arg1.substring(1))`, and with
        // `("a", "bc")` the class answers "bc" while that text answers "ac".
        (
            b"call",
            b"(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            "return (arg0 + arg1).substring(1);",
            "return arg0 + arg1.substring(1);",
        ),
        // The review's second shape: `(a + b).length()`. Its ungrouped text is not even a program
        // under the member's `int` return (`String` cannot convert to `int`).
        (
            b"length",
            b"(Ljava/lang/String;Ljava/lang/String;)I",
            "return (arg0 + arg1).length();",
            "return arg0 + arg1.length();",
        ),
        // A three-operand chain in receiver position: the tree is `(a + b) + c`, so the group
        // holds the whole chain, not just its first pair.
        (
            b"nested",
            b"(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            "return (arg0 + arg1 + arg2).substring(1);",
            "return arg0 + arg1 + arg2.substring(1);",
        ),
        // The controls: a name receiver, a nested call, a left-associative chain of the same
        // precedence and a call argument all already state their trees, so they gain no
        // parentheses — the fix is grouping, not decoration.
        (
            b"plain",
            b"(Ljava/lang/String;)Ljava/lang/String;",
            "return arg0.trim();",
            "",
        ),
        (
            b"chained",
            b"(Ljava/lang/String;)I",
            "return arg0.trim().length();",
            "",
        ),
        (
            b"same",
            b"(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            "return arg0 + arg1 + arg2;",
            "",
        ),
        (
            b"argument",
            b"(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            "wrap(arg0 + arg1);",
            "",
        ),
        (
            b"wrap",
            b"(Ljava/lang/String;)Ljava/lang/String;",
            "return arg0;",
            "",
        ),
    ];

    let mut problems = Vec::new();
    for (member, descriptor, statement, ungrouped) in shapes {
        let name = String::from_utf8_lossy(member);
        let report = recover(&engine, &sample, member, descriptor);
        let text = &report.text;
        if !text.contains(statement) {
            problems.push(format!(
                "{name}: the text must state `{statement}`:\n{text}"
            ));
        }
        if !ungrouped.is_empty() && text.contains(ungrouped) {
            problems.push(format!(
                "{name}: the text still holds `{ungrouped}`, which parses back into a different \
                 tree:\n{text}"
            ));
        }
        let quoted = quoted_bcis(text);
        if !quoted.is_empty() {
            problems.push(format!(
                "{name}: this shape is provable, so the body may not be quoted: {quoted:?}\n{text}"
            ));
        }
        if report.representation != Representation::Java {
            problems.push(format!(
                "{name}: the body must stay written whole, and the run states {:?}",
                report.representation
            ));
        }
        if report.quality != Quality::Structured {
            problems.push(format!(
                "{name}: the body must stay structured, and the run states {:?}",
                report.quality
            ));
        }
        if report.content != RecoveryContent::ContainsStatements {
            problems.push(format!(
                "{name}: the artifact states statements, and the run states {:?}",
                report.content
            ));
        }
    }
    assert!(
        problems.is_empty(),
        "a subexpression keeps its value only if the printed text parses back into the tree it was \
         printed from, and that is a property of the position it is written in:\n{}",
        problems.join("\n")
    );
}
