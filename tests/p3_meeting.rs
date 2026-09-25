//! P3 2c.29/2c.30/2c.31 in one sample: what a **meeting position** does with a conversion the
//! bytecode left to it, and what the two `pop`s a compiler writes mean.
//!
//! Three spelling questions meet at the same place — the position a value is written into:
//!
//! * **P3 2c.29** — a `char`, a `byte` and a `short` share one slot shape with an `int`, so a
//!   compiler writes **no instruction** when a position widens one of them, and the position itself
//!   performs the conversion: `return arg0.charAt(arg1);`, `int local1 = arg0;` and
//!   `pass(arg0.charAt(0))` are the same programs the bytecode ran, where the `(int)` this layer
//!   used to invent is a cast the source does not have. A conversion with a **real** instruction
//!   (`i2l`, `i2b`) is that instruction's own and stays refused until P3 2c.8 writes it.
//! * **P3 2c.30** — a member whose descriptor returns `char` and whose `return` reads an in-range
//!   `int` constant writes the **character** that code unit stands for (`bipush 65; ireturn` is
//!   `return 'A';`), spelled by the emitter's escape table — the one a `char` `switch` key is
//!   written with. An `int` position keeps the number, and an out-of-range or non-constant value
//!   keeps the refusal it had.
//! * **P3 2c.31** — a `pop` is a compiler's way of discarding an evaluation, and the text beside it
//!   is where that evaluation is written: `aload_0; pop; invokestatic stat` is the qualified call
//!   `arg0.stat()` (writing the bare `stat()` would drop the evaluation the source performed), and
//!   `invokeinterface add; pop` is the call written as a statement (`arg0.add("x");`). A `pop2`,
//!   which discards two slots, is claimed by no shape and keeps its quote.
//!
//! The texts below pin the shape over the class bytes committed in `tests/fixtures/p3-meeting/`
//! (see its `README.md` for the command, the version, the 940 bytes and the digest). Every member is
//! one spelling question or one control: the controls are the two refusals `viaStoreLong` and
//! `trunc`, the `int` return of `stat`, the `pass` call `fieldArg` reaches, and the `pop2` of
//! `pop2Control`.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 940 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-meeting/v8/Meet.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a standalone CLASS")
}

/// One class-source presentation of one committed sample, under the entry point the CLI calls.
fn class_source_of(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    match Engine::new()
        .class_source(slice::from_ref(snapshot), &request, &mut budget())
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one committed sample answers one definition, got {} candidate(s)",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one committed sample answers one definition, got an unfinished selection with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

/// One member of the sample, as the class presentation holds it.
fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
}

/// One member's own recovery run: the member has to have been run, so a member that was renamed, or
/// one whose body this run could not reach at all, fails here instead of covering less than this
/// file claims.
fn run_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &member(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` is not a member this run recovered a body for: {other:?}"),
    }
}

/// One member's body, presented in full: Java, structured, with at least one statement and no
/// `// jarde:` marker — the planes have to agree with the text this file reads.
fn presented<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    let run = run_of(report, name);
    assert!(run.produced(), "`{name}` produced no artifact: {run:?}");
    assert_eq!(
        run.representation,
        Representation::Java,
        "`{name}`: {run:?}"
    );
    assert_eq!(run.quality, Quality::Structured, "`{name}`: {run:?}");
    assert_eq!(
        member(report, name).markers,
        Vec::<String>::new(),
        "`{name}` is recovered in full"
    );
    &member(report, name).text
}

/// One member's body, refused: the run is the weaker representation and the text quotes the
/// bytecode it could not present.
fn refused<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    let run = run_of(report, name);
    assert_eq!(
        run.representation,
        Representation::Mixed,
        "`{name}`: {run:?}"
    );
    assert_eq!(run.quality, Quality::Fallback, "`{name}`: {run:?}");
    let text = &member(report, name).text;
    assert!(
        text.contains("// @bytecode"),
        "`{name}` quotes the bytecode it refused:\n{text}"
    );
    text
}

// ---------------------------------------------------------------------------------------------
// P3 2c.29: the position performs the widening, and the text states the value
// ---------------------------------------------------------------------------------------------

/// The three positions one implicit `char` → `int` widening reaches in this sample: a `return`, a
/// declaration and an invocation's argument. Return and assignment positions perform their own
/// widening; an invocation states its parameter type so overload resolution keeps the target.
#[test]
fn a_position_performs_its_own_widening() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Meet");

    // `at(Ljava/lang/String;I)I`: `invokevirtual String.charAt:(I)C` produces a `char` and the
    // member's own descriptor returns `int`; `javac` wrote no instruction between them.
    let at = presented(&report, "at");
    assert!(
        at.contains("return arg0.charAt(arg1);"),
        "the `return` widens the value itself:\n{at}"
    );
    // `viaStore(C)I`: the `char` parameter is stored into a slot whose own declaration says `int`.
    let via_store = presented(&report, "viaStore");
    assert!(
        via_store.contains("int local1 = arg0;"),
        "the declaration's own type widens the value:\n{via_store}"
    );
    // `fieldArg(Ljava/lang/String;)I`: the `char` the `charAt` produced is passed to `pass(int)`.
    let field_arg = presented(&report, "fieldArg");
    assert!(
        field_arg.contains("return pass((int) arg0.charAt(0));"),
        "the argument states the callee's int parameter for overload resolution:\n{field_arg}"
    );

    // The one direction this rule must not touch: a value whose presented type is already the
    // position's gains nothing.
    let pass = presented(&report, "pass");
    assert!(
        pass.contains("return arg0;") && !pass.contains("(int)"),
        "an `int` value in an `int` position gains nothing:\n{pass}"
    );
    // Return and assignment positions have no overload to select and keep their implicit widening.
    for text in [at, via_store, pass] {
        assert!(
            !text.contains("(int)") && !text.contains("(char)"),
            "a position that performs its own widening writes none:\n{text}"
        );
    }
}

/// The controls of the same rule: a conversion the bytecode really performed is that instruction's
/// own, and a narrowing this layer cannot prove is no conversion at all. Both keep the refusal and
/// the quoted bytecode they had.
#[test]
fn a_conversion_with_a_real_instruction_is_still_refused() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Meet");

    // `viaStoreLong(I)J` is `iload_0; i2l; lstore_1; …`: the `i2l` at BCI 1 is the conversion, and
    // presenting the slot without it would write a `long` the body never held. It is P3 2c.8's to
    // write and stays refused until that rule lands.
    let via_store_long = refused(&report, "viaStoreLong");
    assert!(
        via_store_long.contains("// @bytecode 1"),
        "the `i2l` itself is what the quote names:\n{via_store_long}"
    );

    // `trunc(I)B` is `iload_0; i2b; istore_1; …`: the narrowing `i2b` is no conversion this layer
    // writes, and the `int` value in the `byte` return position is refused too.
    let trunc = refused(&report, "trunc");
    assert!(
        trunc.contains("// @bytecode 1") && trunc.contains("// @bytecode 4"),
        "both the `i2b` and the `byte` return position are named:\n{trunc}"
    );
    assert!(
        trunc.contains("`byte`") && trunc.contains("no conversion"),
        "the refusal states the position that required the type:\n{trunc}"
    );
}

// ---------------------------------------------------------------------------------------------
// P3 2c.30: a `char` position spells the code unit its `int` constant stands for
// ---------------------------------------------------------------------------------------------

/// `grade(I)C` returns constants and declares `char`: each arm writes the character its `bipush`
/// stands for, and the `int` member beside it keeps the number.
#[test]
fn a_char_return_spells_the_code_unit_its_constant_stands_for() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Meet");

    let grade = presented(&report, "grade");
    for literal in ["return 'A';", "return 'B';", "return 'C';"] {
        assert!(
            grade.contains(literal),
            "the `char` return writes the character its constant stands for ({literal}):\n{grade}"
        );
    }
    for number in ["return 65;", "return 66;", "return 67;"] {
        assert!(
            !grade.contains(number),
            "the code unit is stated as a character, not as this `int` spelling:\n{grade}"
        );
    }
    // The arms the payload orders: 90 and 95 fall into `'A'`, 80 is `'B'` and the default `'C'`.
    assert!(
        grade.contains("case 80:\n                return 'B';"),
        "the arms keep the keys the payload states:\n{grade}"
    );

    // The control: the same literal shape in an `int` member keeps the decimal number, because the
    // wording of a character literal is the `char` position's own.
    let stat = presented(&report, "stat");
    assert!(
        stat.contains("return 7;") && !stat.contains("'"),
        "an `int` return keeps the number:\n{stat}"
    );
}

// ---------------------------------------------------------------------------------------------
// P3 2c.31: the two `pop`s a compiler writes are the text beside them
// ---------------------------------------------------------------------------------------------

/// `viaRef(LMeet;)I` is `aload_0; pop; invokestatic Meet.stat:()I; ireturn`: the evaluation the
/// `pop` discarded is the call's **qualifier**, and the call's text carries it.
#[test]
fn a_popped_evaluation_is_the_static_calls_qualifier() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Meet");

    let via_ref = presented(&report, "viaRef");
    assert!(
        via_ref.contains("return arg0.stat();"),
        "the popped evaluation is written as the call's qualifier:\n{via_ref}"
    );
    // The two texts this shape is *not*: the bare name the `pop` would have left behind, and the
    // owner-qualified one a static call of this class's own member does not need.
    assert!(
        !via_ref.contains("return stat();") && !via_ref.contains("Meet.stat("),
        "the qualifier is the evaluation the source performed:\n{via_ref}"
    );
    // The `pop` is not a gap of its own: the statement it stood for is the call's text.
    assert!(
        !via_ref.contains("@bytecode"),
        "no quote is left for the `pop` the call's text accounts for:\n{via_ref}"
    );

    // The control beside it: the same class's `stat()` called without a qualifier keeps the bare
    // name — this rule adds a qualifier only where the bytecode evaluated one (P3 4.4's own rule
    // for the pool's owner is untouched).
    let stat = presented(&report, "stat");
    assert!(
        stat.contains("return 7;") && !stat.contains(".stat("),
        "a static call with no evaluated qualifier keeps its own text:\n{stat}"
    );
}

/// `unchecked(Ljava/util/List;)V` is `… invokeinterface List.add:(…)Z; pop; return`: the call's
/// result is discarded, and the call's own statement is the whole of it.
#[test]
fn a_discarded_call_result_is_the_calls_own_statement() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Meet");

    let unchecked = presented(&report, "unchecked");
    assert!(
        unchecked.contains("arg0.add((java.lang.Object) \"x\");"),
        "the call is written as the statement its discarded result makes it:\n{unchecked}"
    );
    assert!(
        !unchecked.contains("@bytecode"),
        "the `pop` is not quoted: the statement discards what it discarded:\n{unchecked}"
    );
    assert!(
        !unchecked.contains("boolean") && !unchecked.contains("true"),
        "the discarded result is not invented as a value:\n{unchecked}"
    );
}

/// The control of the same rule: a `pop2` discards two slots, no shape of the rule claims one, and
/// the quote it always had is still written beside it.
#[test]
fn no_shape_of_the_rule_claims_a_pop2() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Meet");

    let pop2 = refused(&report, "pop2Control");
    assert!(
        pop2.contains("java.lang.System.nanoTime();"),
        "the call itself is presented as the statement it is:\n{pop2}"
    );
    assert!(
        pop2.contains("// @bytecode 3"),
        "the `pop2` at BCI 3 keeps its own quote:\n{pop2}"
    );
    assert!(
        pop2.contains("not part of the provable subset"),
        "the quote states why the instruction is not presented:\n{pop2}"
    );
}
