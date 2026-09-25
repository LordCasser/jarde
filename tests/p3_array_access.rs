//! P3 2b in one sample: what an **array** is written as — its creation, its subscripts, its length
//! and its element type (P10).
//!
//! The rules this file pins are the object's own instructions, each of which states exactly one
//! fact:
//!
//! * **P3 2b.1/2b.4** — an element read is the subscript `array[index]`, and an element write is
//!   `array[index] = value;`. The element type is the **array's** own where the frames state it and
//!   the opcode's where they do not: `char pick(char[] a, int i)` returns `a[i]` with no `(char)`
//!   cast, `int code(char[] a, int i)` returns the same text with no `(int)` cast either, and a
//!   `baload` is a `boolean` **only** where the array is proven `[Z` — `[B` is read with the very
//!   same instruction, so the read's shape proves nothing on its own.
//! * **P3 2b.2** — a local the run wrote is declared before it is read. The negative this file
//!   states is the name: no member's text spells a `localN` its own text never declared.
//! * **P3 2b.3** — `multianewarray` is one creation with one length per dimension its operand
//!   states: `new int[arg0][arg1]`, never two one-dimensional creations.
//! * **P3 2b.5** — `anewarray` takes its element from the pool class, and the local's declared name
//!   is the slot the store really wrote (`local1` in a static member, never `local0`).
//! * **the boundary 2b.6 leave** — `arraylength` in a **loop test** remains outside this slice.
//!   The later bounded initializer rule presents the complete `new int[]{…}` chain, including a
//!   local that receives the initialized array.
//!
//! The texts below pin the shape over the class bytes committed in `tests/fixtures/p3-array-access/`
//! (see its `README.md` for the command, the compiler, the 1130 bytes and the digest). Every member
//! is one shape or one control; `created`, `put`, `setByte`, `box` and `cell` also carry the local
//! 2b.2 is about, and `literal` plus `chained` pin the initializer's return and local-store
//! consumers.
//!
//! `enumswitch@1`'s own claim (P3 2.3) is deliberately *not* re-pinned here: that rule's two texts —
//! the presented dispatch table and the read it refuses — are `crates/jarde-java/tests/p3_patterns.rs`',
//! and this slice changed what the refused one writes beside its refusal (the subscript), not what
//! the rule proves.

use jarde::*;
use std::collections::BTreeSet;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 1184 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-array-access/v8/Grid.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a standalone CLASS")
}

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
        other => panic!("one committed sample answers one definition: {other:?}"),
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

/// The `localN` names one member's text reads before any declaration of them (P3 2b.2).
///
/// A recovered body declares a local at the write that fills it (`int[] local2 = new int[arg0];`),
/// so the text's own `localN` spellings are checkable without a compiler: every one of them has to
/// follow its declaration in the same body, or the artifact names a variable nothing declares. The
/// parameters (`argN`) are declared by the member's signature and are not this question, and a
/// quoted (`//`) line declares nothing and reads nothing.
fn undeclared_locals(text: &str) -> Vec<String> {
    let mut declared: BTreeSet<&str> = BTreeSet::new();
    let mut undeclared: Vec<String> = Vec::new();
    for line in text.lines() {
        let code = line.split("//").next().unwrap_or("");
        // A declaration is a type and a name before the `=`: `int[] local2 = …`, `boolean[] local0`.
        if let Some((left, _)) = code.split_once(" = ")
            && let [.., name] = left.split_whitespace().collect::<Vec<_>>().as_slice()
            && name.starts_with("local")
        {
            declared.insert(name);
        }
        for token in code.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '$')) {
            if token.starts_with("local") && !declared.contains(token) {
                undeclared.push(token.to_string());
            }
        }
    }
    undeclared
}

// ---------------------------------------------------------------------------------------------
// P3 2b.1/2b.2: a creation, the element it is indexed with, and the local that holds it
// ---------------------------------------------------------------------------------------------

/// `created(II)I` is `iload_0; newarray int; astore_2; aload_2; iconst_0; iconst_1; iastore;
/// aload_2; iload_1; iaload; ireturn`: one creation, one element write and one element read, and the
/// local the creation filled is declared by the statement that filled it — in the slot the store
/// really wrote (`local2` in a static member whose two parameters take slots 0 and 1).
#[test]
fn an_array_creation_its_subscripts_and_its_local_are_written() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Grid");

    let created = presented(&report, "created");
    assert!(
        created.contains("int[] local2 = new int[arg0];"),
        "the creation is the declaration the store carries:\n{created}"
    );
    assert!(
        created.contains("local2[0] = 1;"),
        "an element write is the subscript assignment:\n{created}"
    );
    assert!(
        created.contains("return local2[arg1];"),
        "an element read is the subscript:\n{created}"
    );
    assert!(
        !created.contains("Object local2") && !created.contains("@bytecode"),
        "the local's own type is the array its creation states:\n{created}"
    );
    assert_eq!(
        undeclared_locals(created),
        Vec::<String>::new(),
        "every local the body writes is declared before it is read:\n{created}"
    );
}

/// `pick([II)I` is `aload_0; iload_1; iaload; ireturn`: a **straight-line** read of an argument
/// array's element, which is what this batch covers. The loop whose test reads `arraylength` and an
/// element is P3 2b.6 and is not presented as this rule's evidence.
#[test]
fn an_element_read_of_a_parameter_array_is_the_subscript() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Grid");

    let pick = presented(&report, "pick");
    assert!(
        pick.contains("return arg0[arg1];"),
        "the read is the subscript the bytecode indexes with:\n{pick}"
    );
    assert!(
        !pick.contains("@bytecode"),
        "nothing is left to quote:\n{pick}"
    );
}

/// `at([[III)I` is `aload_0; iload_1; aaload; iload_2; iaload; ireturn`: the outer read's array is
/// the inner read, and the two subscripts are the same expression one level apart.
#[test]
fn a_nested_read_reindexes_the_subscript_it_read() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Grid");

    let at = presented(&report, "at");
    assert!(
        at.contains("return arg0[arg1][arg2];"),
        "the outer read indexes the inner one:\n{at}"
    );
    assert!(
        !at.contains("(int)") && !at.contains("(int[])"),
        "the outer read claims no element type of its own:\n{at}"
    );
}

// ---------------------------------------------------------------------------------------------
// P3 2b.4: the element type comes from the array's own type, and from the opcode only without one
// ---------------------------------------------------------------------------------------------

/// `flag()Z` is `iconst_3; newarray boolean; astore_0; aload_0; iconst_0; baload; ireturn`: the
/// creation's `atype` states a `boolean[]`, the read of its element is a `boolean`, and the `Z`
/// return needs exactly that evidence — which the array's own type is, because `baload` reads a
/// `[B` with the very same instruction.
#[test]
fn a_boolean_element_read_is_evidence_only_where_the_array_is_proven_boolean() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Grid");

    // The creation states the element type, and the `Z` return of its element is written.
    let flag = presented(&report, "flag");
    assert!(
        flag.contains("boolean[] local0 = new boolean[3];"),
        "the creation states a `boolean[]`:\n{flag}"
    );
    assert!(
        flag.contains("return local0[0];"),
        "the element read is written in the boolean return position:\n{flag}"
    );

    // A `[Z` **parameter** is the same evidence without a creation in the body.
    let test = presented(&report, "test");
    assert!(
        test.contains("return arg0[arg1];"),
        "a proven `[Z` read is a boolean:\n{test}"
    );

    // The control: the same `baload` opcode over a `[B` array is a `byte` read in an `int` member —
    // a subscript with no cast and no `true`/`false`, because nothing in this body proves `[Z`.
    let first = presented(&report, "first");
    assert!(
        first.contains("return arg0[arg1];"),
        "a `[B` read is the subscript:\n{first}"
    );
    assert!(
        !first.contains("true") && !first.contains("false") && !first.contains("(int)"),
        "a `[B` read is neither a boolean nor a conversion:\n{first}"
    );
}

/// The narrow element reads of P3 2b.4: `char`, `byte` and `short` share one value shape with
/// `int`, so the read's own text carries the conversion the bytecode did **not** perform — a cast
/// here would be legal Java the source never had — while the wider families state their element type
/// outright.
#[test]
fn a_narrow_element_read_adds_no_conversion_the_bytecode_did_not_perform() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Grid");

    for (name, expected) in [
        // `letter([CI)C` and `code([CI)I` are the same body: a `char` position keeps the code unit
        // and an `int` position widens it implicitly.
        ("letter", "return arg0[arg1];"),
        ("code", "return arg0[arg1];"),
        ("first", "return arg0[arg1];"),
        ("small", "return arg0[arg1];"),
        ("wide", "return arg0[arg1];"),
        ("real", "return arg0[arg1];"),
        ("precise", "return arg0[arg1];"),
    ] {
        let text = presented(&report, name);
        assert!(text.contains(expected), "`{name}`:\n{text}");
        assert!(
            !text.contains("(int)")
                && !text.contains("(char)")
                && !text.contains("(byte)")
                && !text.contains("(short)")
                && !text.contains("(long)")
                && !text.contains("(float)")
                && !text.contains("(double)"),
            "`{name}` writes no conversion the bytecode did not perform:\n{text}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// P3 2b.1/2b.4: the length, and the element write
// ---------------------------------------------------------------------------------------------

/// `size([I)I` is `aload_0; arraylength; ireturn`: `.length` is a suffix over the array, and it is
/// not a field access — no member is proved here.
#[test]
fn the_length_of_an_array_is_a_suffix_over_it() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Grid");

    let size = presented(&report, "size");
    assert!(
        size.contains("return arg0.length;"),
        "the length is the suffix the bytecode reads:\n{size}"
    );
}

/// `put([III)I` is `aload_0; iload_1; iload_2; iastore; aload_0; iload_1; iaload; ireturn`, and
/// `setByte([BIB)V` is the same shape with a `byte` element: an element write is the subscript
/// assignment, in the position the instruction runs, and the value meets the element type the
/// array's own type states.
#[test]
fn an_element_write_is_the_subscript_assignment() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Grid");

    let put = presented(&report, "put");
    assert!(
        put.contains("arg0[arg1] = arg2;"),
        "the write is the subscript assignment:\n{put}"
    );
    assert!(
        put.contains("return arg0[arg1];"),
        "the read after it is another subscript:\n{put}"
    );

    let set_byte = presented(&report, "setByte");
    assert!(
        set_byte.contains("arg0[arg1] = arg2;"),
        "a `byte` element takes the value as it is:\n{set_byte}"
    );
    assert!(
        !set_byte.contains("(byte)"),
        "no conversion the bytecode did not perform:\n{set_byte}"
    );
}

// ---------------------------------------------------------------------------------------------
// P3 2b.3/2b.5: the two creations that are not `newarray`
// ---------------------------------------------------------------------------------------------

/// `strings(I)[Ljava/lang/String;` is `iload_0; anewarray java/lang/String; areturn` and
/// `box(Ljava/lang/String;)[Ljava/lang/String;` is `iconst_1; anewarray; astore_1; aload_1;
/// iconst_0; aload_0; aastore; aload_1; areturn`: `anewarray` takes its element type from the pool
/// class and allocates one dimension, and the local's declared name is the slot the store wrote —
/// `local1`, because this member is static and slot 0 is its parameter.
#[test]
fn a_reference_creation_takes_its_element_from_the_pool_class() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Grid");

    let strings = presented(&report, "strings");
    assert!(
        strings.contains("return new java.lang.String[arg0];"),
        "the pool class is the element type:\n{strings}"
    );

    let boxed = presented(&report, "box");
    assert!(
        boxed.contains("java.lang.String[] local1 = new java.lang.String[1];"),
        "the declaration is the array the creation states, in the slot really written:\n{boxed}"
    );
    assert!(
        boxed.contains("local1[0] = arg0;") && boxed.contains("return local1;"),
        "the local is written and read by its own name:\n{boxed}"
    );
    assert!(
        !boxed.contains("local0"),
        "a static member's slot 0 is its parameter, not a local:\n{boxed}"
    );
    // The initializer chain `new T[]{…}` is P3 2b.7's and is **not** written here: this shape is a
    // creation and one store.
    assert!(
        !boxed.contains("String[]{") && !boxed.contains("@bytecode"),
        "the creation is written as the creation it is:\n{boxed}"
    );
    assert_eq!(
        undeclared_locals(boxed),
        Vec::<String>::new(),
        "the local is declared before it is read:\n{boxed}"
    );
}

/// `cell(II)I` is `iload_0; iload_1; multianewarray [[I, 2; astore_2; … iastore … iaload; ireturn`:
/// one creation whose lengths are the two operands, written `new int[arg0][arg1]` — never two
/// one-dimensional creations — and the store into it and the read out of it are subscripts one
/// level apart.
#[test]
fn a_multi_dimensional_creation_is_one_creation_with_the_lengths_its_operand_states() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Grid");

    let cell = presented(&report, "cell");
    assert!(
        cell.contains("int[][] local2 = new int[arg0][arg1];"),
        "the dimension count is the operand's and the element is the descriptor's:\n{cell}"
    );
    assert!(
        cell.matches("new int[").count() == 1,
        "the creation is written once:\n{cell}"
    );
    assert!(
        cell.contains("local2[0][0] = 7;") && cell.contains("return local2[0][0];"),
        "both subscripts are the same expression one level apart:\n{cell}"
    );
    assert_eq!(
        undeclared_locals(cell),
        Vec::<String>::new(),
        "the local is declared before it is read:\n{cell}"
    );
}

/// `literal()[I` is `iconst_3; newarray int; dup; …`: the complete ordered chain is now written as
/// one initializer, rather than refusing the final `dup` value or presenting only part of it.
#[test]
fn a_complete_initializer_chain_is_written_at_its_consumer() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Grid");

    let literal = presented(&report, "literal");
    assert!(
        literal.contains("return new int[]{1, 2, 3};"),
        "the complete chain is written once where its value is consumed:\n{literal}"
    );
    assert!(
        !literal.contains("@bytecode"),
        "the chain has no refusal:\n{literal}"
    );
    assert_eq!(
        undeclared_locals(literal),
        Vec::<String>::new(),
        "the return introduces no undeclared local:\n{literal}"
    );
}

/// The same chain **with a local to fill** — `chained(I)[I` is `iconst_1; newarray int; dup;
/// iconst_0; iload_0; iastore; astore_1; aload_1; areturn` — writes the initialization at the
/// declaring store and retains the ordinary local read at return.
#[test]
fn a_local_receives_the_proved_initializer_before_it_is_read() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Grid");

    let chained = presented(&report, "chained");
    assert!(
        chained.contains("int[] local1 = new int[]{arg0};"),
        "the local declaration carries its initialized array:\n{chained}"
    );
    assert!(
        chained.contains("return local1;"),
        "the stored local is read by its declared name:\n{chained}"
    );
    assert!(
        !chained.contains("@bytecode"),
        "the complete chain is recovered:\n{chained}"
    );
    assert_eq!(
        undeclared_locals(chained),
        Vec::<String>::new(),
        "and the text's own names are all declared:\n{chained}"
    );
}
