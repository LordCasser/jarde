//! `re-express-string-concatenation`: the conversion a presented chain performs, and the chain
//! length that used to become a stack.
//!
//! Two findings share one representation, and this file is the permanent regression of both:
//!
//! * **T4** — `new StringBuilder().append(a).append(b).append("!")` was recovered as
//!   `arg0 + arg1 + "!"`: the construction folded `+` from the first operand, so the first `+` was a
//!   **numeric** addition and each `append`'s conversion happened to the sum. Both texts compile,
//!   and with `(1, 2)` the class answers `"12!"` where the text answered `"3!"`. The controlled
//!   input is the committed sample `tests/fixtures/p3-concat-conversion/` (javac 23.0.1
//!   `--release 8 -g:none`, byte count and SHA-256 in its README); the value evidence is
//!   `tests/p3_execution_comparison.rs`, which compiles the recovered text under the member's own
//!   declaration and runs it beside the original.
//! * **T1** — a chain of 1536/2048 `append(s)` calls aborted the debug build (`exit 134`, empty
//!   stdout, `thread 'main' has overflowed its stack`) while 1024 completed. The recursion was the
//!   **printer's** walk over the left-deep tree the construction folded (`emit.rs`'s binary arm,
//!   frame by frame in the change's verification), so the fix is the representation — one part per
//!   `append`, in a sequence — and not a depth limit on the printing. The chain is generated here:
//!   its length is this check's **parameter** (the review's interval and beyond), and the bytes are
//!   written by this file because `javac`'s own recursion needs an enlarged compiler stack for these
//!   inputs (`-J-Xss64m` produced the review's bytes) — a way to produce bytes, not a dependency a
//!   regression may have. The process-level half of the same input is
//!   `crates/jarde-cli/tests/task_cli.rs`.
//!
//! The controls keep the fix from being decoration: a chain that already starts in a string context
//! keeps its text byte for byte, a part that is itself an addition keeps its own group, and a chain
//! of `String` parts gains nothing (`crates/jarde-java/tests/p3_patterns.rs` pins the same property
//! on its own fixture, `"x" + 5 + f()`).

use jarde::*;
// The facade re-exports what a *consumer* of a report needs; this file reads the segment table's own
// vocabulary, so it names the recovery layer directly — the layer whose report it is checking.
use jarde_java::{Provenance, Segment};
use std::collections::BTreeSet;
use std::slice;

/// The committed sample of the conversion defect, compiled by javac 23.0.1 `--release 8 -g:none`
/// (see the fixture's README for the command, the byte count and the digest).
const CONVERSION: &[u8] = include_bytes!("fixtures/p3-concat-conversion/v8/ConcatConversion.class");

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

/// Opens one sample and reads its header through the reader's own entry point, so that the members
/// presented below are the ones the class declares rather than the ones this file claims.
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
    recover_under(engine, fixture, name, descriptor, limits())
}

/// The same run under explicit limits, so that a bounded check can state its own bound.
fn recover_under(
    engine: &Engine,
    fixture: &Fixture,
    name: &[u8],
    descriptor: &[u8],
    limits: Limits,
) -> RecoveryReport {
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
            &mut Budget::new(limits),
        )
        .expect("a legal request is answered, not raised")
        .recovery()
        .clone()
}

/// The member's own `return` statement, as the artifact wrote it, or a panic naming the whole text
/// when there is none.
fn returned(report: &RecoveryReport) -> String {
    report
        .text
        .lines()
        .find(|line| line.trim_start().starts_with("return "))
        .unwrap_or_else(|| panic!("the report has a return statement:\n{}", report.text))
        .trim()
        .to_string()
}

/// One member of the committed sample, as the artifact presents it.
fn presented(sample: &Fixture, name: &[u8], descriptor: &[u8]) -> RecoveryReport {
    let engine = Engine::new();
    let report = recover(&engine, sample, name, descriptor);
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(
        report.representation,
        Representation::Java,
        "{:?}",
        report.diagnostics
    );
    assert_eq!(
        report.quality,
        Quality::Structured,
        "{:?}",
        report.diagnostics
    );
    assert_eq!(report.content, RecoveryContent::ContainsStatements);
    report
}

// ---------------------------------------------------------------------------------------------
// T4: the conversion each `append` performs, in the part that performs it
// ---------------------------------------------------------------------------------------------

/// The reported shape: two parts that need `String.valueOf` before the first `String` part. The
/// text has to start the string concatenation **at** the first of them, so the first `+` is a string
/// concatenation and neither part is added to the other as a number.
#[test]
fn a_chain_that_starts_with_parts_needing_conversion_starts_in_a_string_context() {
    let engine = Engine::new();
    let sample = fixture(&engine, CONVERSION);

    let two = presented(&sample, b"twoIntsThenString", b"(II)Ljava/lang/String;");
    assert_eq!(returned(&two), "return \"\" + arg0 + arg1 + \"!\";");
    assert_eq!(
        two.concats
            .first()
            .expect("the chain is recorded")
            .appends
            .iter()
            .map(|append| (append.bci, append.parameter.as_str()))
            .collect::<Vec<_>>(),
        vec![(8, "int"), (12, "int"), (17, "java.lang.String")],
        "the parts are the chain's own `append` calls, in the bytecode's order"
    );

    // The same rule where the parts are calls: `markA()` and `markB()` are evaluated once each, in
    // this order, and none of the three parts is merged with another.
    let marked = presented(&sample, b"marked", b"()Ljava/lang/String;");
    assert_eq!(
        returned(&marked),
        "return \"\" + markA() + markB() + \"!\";"
    );
    assert_eq!(
        marked.text.matches("markA()").count(),
        1,
        "each part is written once:\n{}",
        marked.text
    );

    // The part that can throw keeps its own position, so the observable part before it runs first
    // (the executed comparison measures that count and order).
    let failing = presented(&sample, b"failing", b"(I)Ljava/lang/String;");
    assert_eq!(
        returned(&failing),
        "return \"\" + markA() + div(100, arg0) + \"!\";"
    );
}

/// **One** part whose value is itself an addition keeps its own group, and **two** parts that are
/// each a value are not merged: the two members are two different programs, and the pre-fix
/// construction wrote the same text for both (`arg0 + arg1 + "!"`).
#[test]
fn a_part_that_is_itself_an_addition_keeps_its_own_group() {
    let engine = Engine::new();
    let sample = fixture(&engine, CONVERSION);

    let sum = presented(&sample, b"onePartIsASum", b"(II)Ljava/lang/String;");
    let two = presented(&sample, b"twoIntsThenString", b"(II)Ljava/lang/String;");
    assert_eq!(returned(&sum), "return \"\" + (arg0 + arg1) + \"!\";");
    assert_eq!(returned(&two), "return \"\" + arg0 + arg1 + \"!\";");
    assert_ne!(
        returned(&sum),
        returned(&two),
        "one part that is an addition and two parts that are values are different programs: \
         `\"\" + (arg0 + arg1) + \"!\"` answers \"3!\" for (1, 2) and `\"\" + arg0 + arg1 + \"!\"` \
         answers \"12!\""
    );
    let sum_chain = sum.concats.first().expect("the chain is recorded");
    assert_eq!(
        sum_chain
            .appends
            .iter()
            .map(|append| append.bci)
            .collect::<Vec<_>>(),
        vec![10, 15],
        "the sum is **one** `append`: the two operands are not the chain's parts"
    );
}

/// `boolean` is the one conversion the value's text does not already carry: a `boolean` is pushed as
/// the `int`-shaped `0`/`1`, so only the `append`'s own parameter descriptor says that
/// `append(true)` writes `"true"` where `"" + 1` would write `"1"`.
#[test]
fn a_boolean_part_is_spelled_by_its_append_descriptor() {
    let engine = Engine::new();
    let sample = fixture(&engine, CONVERSION);

    let literal = presented(&sample, b"booleanLiteral", b"()Ljava/lang/String;");
    assert_eq!(returned(&literal), "return \"\" + true + \"!\";");
    assert!(
        !literal.text.contains("1 +"),
        "the `iconst_1` of `append(true)` is not the numeric `1`:\n{}",
        literal.text
    );
    assert_eq!(
        literal
            .concats
            .first()
            .expect("the chain is recorded")
            .appends
            .iter()
            .map(|append| append.parameter.as_str())
            .collect::<Vec<_>>(),
        vec!["boolean", "java.lang.String"]
    );

    // The same descriptor for a value whose own evidence is `boolean`: the name already spells it,
    // and the chain must not re-decide a local's type here.
    let parameter = presented(&sample, b"booleanParameter", b"(Z)Ljava/lang/String;");
    assert_eq!(returned(&parameter), "return \"\" + arg0 + \"!\";");
}

/// `null` and object parts are `String.valueOf`'s conversion, which is what a `+` in a string
/// context performs — and the empty string the chain starts from is what makes `null` a legal
/// operand at all.
#[test]
fn null_and_object_parts_keep_their_conversion() {
    let engine = Engine::new();
    let sample = fixture(&engine, CONVERSION);

    let null = presented(&sample, b"nullPart", b"()Ljava/lang/String;");
    assert_eq!(returned(&null), "return \"\" + null + \"!\";");
    assert_eq!(
        null.concats
            .first()
            .expect("the chain is recorded")
            .appends
            .first()
            .map(|append| append.parameter.as_str()),
        Some("java.lang.Object")
    );

    let object = presented(
        &sample,
        b"objectPart",
        b"(Ljava/lang/Object;)Ljava/lang/String;",
    );
    assert_eq!(returned(&object), "return \"\" + arg0 + \"!\";");
}

/// The two controls: a chain whose first part is already a `String` keeps its text byte for byte,
/// with no empty string and no other decoration.
#[test]
fn the_two_controls_gain_nothing() {
    let engine = Engine::new();
    let sample = fixture(&engine, CONVERSION);

    let last = presented(
        &sample,
        b"numericLast",
        b"(Ljava/lang/String;I)Ljava/lang/String;",
    );
    assert_eq!(returned(&last), "return arg0 + arg1;");
    assert!(!last.text.contains("\"\""), "{}", last.text);

    let all = presented(
        &sample,
        b"allStrings",
        b"(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
    );
    assert_eq!(returned(&all), "return arg0 + arg1;");
    assert!(!all.text.contains("\"\""), "{}", all.text);
}

/// Each part carries the `append` that converts it, and the empty string a chain starts from when
/// its first part is not a `String` carries the chain's own anchors: it has no instruction behind it,
/// so no anchor of the table claims one the chain did not execute.
#[test]
fn each_part_carries_its_append_and_the_string_context_carries_the_chain() {
    let engine = Engine::new();
    let sample = fixture(&engine, CONVERSION);
    let report = presented(&sample, b"twoIntsThenString", b"(II)Ljava/lang/String;");

    // The member's own instruction starts, read by the reader's bytecode entry: they are the only
    // places this body has, so an anchor may name one of these and nothing else.
    let selector = MethodSelector {
        name: JvmBytes(b"twoIntsThenString".to_vec()),
        descriptor: JvmBytes(b"(II)Ljava/lang/String;".to_vec()),
    };
    let inspection = inspect_method_bytecode(CONVERSION, selector, &mut Budget::new(limits()))
        .expect("the fixture's own bytecode is readable");
    let starts: BTreeSet<u32> = inspection
        .instructions
        .iter()
        .map(|instruction| instruction.bci)
        .collect();

    // The parts: the operand's own load is the part's direct anchor, and the `append` that converts
    // the value it read is the anchor it presents. One segment is exactly that pair.
    for (value, append, text) in [(7u32, 8u32, "arg0"), (11, 12, "arg1")] {
        let parts: Vec<&Segment> = report
            .source_map
            .of_bci(append)
            .into_iter()
            .filter(|segment| segment.origin().mentions(value) == Some(Provenance::Direct))
            .collect();
        assert_eq!(
            parts.len(),
            1,
            "the value at BCI {value} is the part the `append` at BCI {append} converts"
        );
        assert_eq!(parts[0].text(&report.text), text, "{:?}", parts[0]);
        assert_eq!(
            parts[0].origin().mentions(append),
            Some(Provenance::Derived),
            "the part presents the `append` that converts it, it does not claim to be it"
        );
    }

    // The chain node: the `toString` it ends in is its own anchor, and the empty string's text is
    // covered by this same node and by no part of its own.
    let offset = report
        .text
        .find("\"\" + arg0")
        .expect("the text starts its string context with the empty string");
    let carrier = report
        .source_map
        .covering(offset)
        .expect("the empty string is covered by a node");
    assert_eq!(
        carrier.text(&report.text),
        "\"\" + arg0 + arg1 + \"!\"",
        "the empty string belongs to the concatenation's own text"
    );
    assert_eq!(
        carrier.origin().primary().bci(),
        20,
        "the chain's own anchor is its `toString` (the listing above)"
    );
    assert_eq!(
        carrier.origin().primary().provenance(),
        Provenance::Direct,
        "the value of the chain is produced by its `toString`"
    );
    for bci in carrier.origin().bcis() {
        assert!(
            starts.contains(&bci),
            "the chain's anchors name instructions of the body, and BCI {bci} is not one: {:?}",
            carrier.origin().bcis()
        );
    }
    for bci in [0u32, 3, 4, 7, 8, 11, 12, 15, 17, 20] {
        assert!(
            carrier.origin().bcis().contains(&bci),
            "BCI {bci} is one of the chain's own instructions and reaches the artifact: {:?}",
            carrier.origin().bcis()
        );
    }
    assert!(
        !report.text_of_bci(23).is_empty(),
        "the `areturn` the chain's value is returned from anchors the statement around it"
    );
}

/// One hand-built class for the conversion no text can state: a chain whose `append` takes
/// `boolean` and whose value is the `int` literal `5`.
///
/// No compiler emits this — a `boolean` is pushed as the `int`-shaped `0`/`1` — so the fixture is
/// built here, the way `tests/p3_content.rs` builds the shapes no sample has. The overload's
/// conversion is `String.valueOf(boolean)`, the value has no boolean evidence, and the text `"" + 5`
/// would carry a value the bytecode never wrote: the chain must be refused with its bytecode quoted
/// rather than spelled as the integer it looks like.
fn unstateable_boolean_part() -> Vec<u8> {
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
    utf8(&mut pool, b"p/BooleanPart"); // 1
    pool.push(7); // 2: Class 1
    u16b(&mut pool, 1);
    utf8(&mut pool, b"java/lang/Object"); // 3
    pool.push(7); // 4: Class 3
    u16b(&mut pool, 3);
    utf8(&mut pool, b"method"); // 5
    utf8(&mut pool, b"()Ljava/lang/String;"); // 6
    utf8(&mut pool, b"Code"); // 7
    utf8(&mut pool, b"java/lang/StringBuilder"); // 8
    pool.push(7); // 9: Class 8
    u16b(&mut pool, 8);
    utf8(&mut pool, b"<init>"); // 10
    utf8(&mut pool, b"()V"); // 11
    pool.push(12); // 12: NameAndType 10, 11
    u16b(&mut pool, 10);
    u16b(&mut pool, 11);
    pool.push(10); // 13: Methodref 9, 12
    u16b(&mut pool, 9);
    u16b(&mut pool, 12);
    utf8(&mut pool, b"append"); // 14
    utf8(&mut pool, b"(Z)Ljava/lang/StringBuilder;"); // 15
    pool.push(12); // 16: NameAndType 14, 15
    u16b(&mut pool, 14);
    u16b(&mut pool, 15);
    pool.push(10); // 17: Methodref 9, 16
    u16b(&mut pool, 9);
    u16b(&mut pool, 16);
    utf8(&mut pool, b"(Ljava/lang/String;)Ljava/lang/StringBuilder;"); // 18
    pool.push(12); // 19: NameAndType 14, 18
    u16b(&mut pool, 14);
    u16b(&mut pool, 18);
    pool.push(10); // 20: Methodref 9, 19
    u16b(&mut pool, 9);
    u16b(&mut pool, 19);
    utf8(&mut pool, b"toString"); // 21
    utf8(&mut pool, b"()Ljava/lang/String;"); // 22
    pool.push(12); // 23: NameAndType 21, 22
    u16b(&mut pool, 21);
    u16b(&mut pool, 22);
    pool.push(10); // 24: Methodref 9, 23
    u16b(&mut pool, 9);
    u16b(&mut pool, 23);
    utf8(&mut pool, b"!"); // 25
    pool.push(8); // 26: String 25
    u16b(&mut pool, 25);

    let code: Vec<u8> = vec![
        0xbb, 0x00, 0x09, // 0: new java/lang/StringBuilder
        0x59, // 3: dup
        0xb7, 0x00, 0x0d, // 4: invokespecial <init>()V
        0x08, // 7: iconst_5
        0xb6, 0x00, 0x11, // 8: invokevirtual append:(Z)Ljava/lang/StringBuilder;
        0x12, 0x1a, // 11: ldc "!"
        0xb6, 0x00, 0x14, // 13: invokevirtual append:(Ljava/lang/String;)…
        0xb6, 0x00, 0x18, // 16: invokevirtual toString:()Ljava/lang/String;
        0xb0, // 19: areturn
    ];

    let mut output = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0); // minor
    u16b(&mut output, 52); // major: Java 8
    u16b(&mut output, 27); // constant_pool_count: the 26 entries above
    output.extend_from_slice(&pool);
    u16b(&mut output, 0x0021); // public super
    u16b(&mut output, 2); // this_class
    u16b(&mut output, 4); // super_class
    u16b(&mut output, 0); // interfaces
    u16b(&mut output, 0); // fields
    u16b(&mut output, 1); // methods
    u16b(&mut output, 0x0009); // public static
    u16b(&mut output, 5); // name → "method"
    u16b(&mut output, 6); // descriptor
    u16b(&mut output, 1); // attributes
    u16b(&mut output, 7); // "Code"
    let mut attribute = Vec::new();
    u16b(&mut attribute, 2); // max_stack
    u16b(&mut attribute, 0); // max_locals: no parameters
    u32b(
        &mut attribute,
        u32::try_from(code.len()).expect("the fixture body fits u32"),
    );
    attribute.extend_from_slice(&code);
    u16b(&mut attribute, 0); // exception table
    u16b(&mut attribute, 0); // code attributes
    u32b(
        &mut output,
        u32::try_from(attribute.len()).expect("the fixture attribute fits u32"),
    );
    output.extend_from_slice(&attribute);
    u16b(&mut output, 0); // class attributes
    output
}

/// A part whose conversion cannot be stated is refused with its bytecode quoted, never spelled as
/// the value it looks like.
#[test]
fn a_boolean_part_whose_value_is_not_a_boolean_is_refused() {
    let engine = Engine::new();
    let sample = fixture(&engine, &unstateable_boolean_part());
    let report = recover(&engine, &sample, b"method", b"()Ljava/lang/String;");
    assert_eq!(report.representation, Representation::Mixed, "{report:?}");
    assert_eq!(report.quality, Quality::Fallback, "{report:?}");
    assert!(
        report.text.contains("// @bytecode 19"),
        "the refused region quotes the instruction whose value it could not write (the `areturn` at \
         BCI 19):\n{}",
        report.text
    );
    let reason = report
        .text
        .lines()
        .find(|line| line.contains("the `append` at BCI 8"))
        .unwrap_or_else(|| {
            panic!(
                "the refusal names the `append` it could not write:\n{}",
                report.text
            )
        });
    assert!(
        reason.contains("BCI 8") && reason.contains("boolean"),
        "the refusal names the `append` and the conversion it could not write: {reason}"
    );
    assert!(
        !report.text.contains('5'),
        "the numeric literal the `append(boolean)` never wrote is published nowhere in the \
         artifact:\n{}",
        report.text
    );
}

// ---------------------------------------------------------------------------------------------
// T1: a chain's length is not this layer's recursion depth
// ---------------------------------------------------------------------------------------------
/// The chain length the generated fixture is pinned at, and the digest of its bytes: the deep-chain
/// check runs several lengths (the length is its parameter), and this one is the fixed point the
/// bytes are recorded at — the same bytes `crates/jarde-cli/tests/task_cli.rs` writes.
const DEEP_APPENDS: usize = 2048;
/// The blake3 digest of [`deep_concat_fixture`] at [`DEEP_APPENDS`], so a changed generator fails
/// here instead of quietly measuring another input. (The same bytes' SHA-256 is recorded in the
/// change's verification beside the pre-fix abort they produce.)
const DEEP_DIGEST: &str = "d7cb49cedb06f369e1e17c9e82d0cd93dce060214ed57c97fe2f07a1eb645bec";

/// One hand-assembled class whose only member is a straight line of `append` calls.
///
/// This mirrors the review's `DeepConcat2048` reproduction: `new StringBuilder().append(s)`
/// repeated N times and then `toString()`, on the same class, with the same overload
/// (`append(Ljava/lang/String;)`), in one straight line — the shape `javac -J-Xss64m --release 8
/// -g:none` produced the review's bytes from. Faithfulness is the *shape*, not a compiler run:
///
/// * the allocation, its copy and the constructor are the chain's first three instructions, and
///   every `append` is one `aload_0` (the parameter) plus one `invokevirtual`;
/// * the constant pool is the eight entries the shape needs plus the three member references, so
///   the `Code` length is exactly `11 + 4 * N` and the class is `312 + 11 + 4 * N` bytes;
/// * there is no branch and no handler, so the body needs no `StackMapTable` — the frames a
///   version-52 verifier starts its blocks with are the initial frame alone;
/// * the bytes are deterministic, and [`DEEP_DIGEST`] pins them at [`DEEP_APPENDS`].
///
/// The pre-fix behaviour of exactly these bytes (the debug build's `exit 134`, empty stdout and
/// `thread 'main' has overflowed its stack` at N = 1536 and 2048, and the completion at N = 1024) is
/// recorded once in the change's verification; what the tests below assert is the *report*, not a
/// crash.
fn deep_concat_fixture(appends: usize) -> Vec<u8> {
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
    utf8(&mut pool, b"p/DeepConcat"); // 1
    pool.push(7); // 2: Class 1
    u16b(&mut pool, 1);
    utf8(&mut pool, b"java/lang/Object"); // 3
    pool.push(7); // 4: Class 3
    u16b(&mut pool, 3);
    utf8(&mut pool, b"method"); // 5
    utf8(&mut pool, b"(Ljava/lang/String;)Ljava/lang/String;"); // 6
    utf8(&mut pool, b"Code"); // 7
    utf8(&mut pool, b"java/lang/StringBuilder"); // 8
    pool.push(7); // 9: Class 8
    u16b(&mut pool, 8);
    utf8(&mut pool, b"<init>"); // 10
    utf8(&mut pool, b"()V"); // 11
    pool.push(12); // 12: NameAndType 10, 11
    u16b(&mut pool, 10);
    u16b(&mut pool, 11);
    pool.push(10); // 13: Methodref 9, 12
    u16b(&mut pool, 9);
    u16b(&mut pool, 12);
    utf8(&mut pool, b"append"); // 14
    utf8(&mut pool, b"(Ljava/lang/String;)Ljava/lang/StringBuilder;"); // 15
    pool.push(12); // 16: NameAndType 14, 15
    u16b(&mut pool, 14);
    u16b(&mut pool, 15);
    pool.push(10); // 17: Methodref 9, 16
    u16b(&mut pool, 9);
    u16b(&mut pool, 16);
    utf8(&mut pool, b"toString"); // 18
    utf8(&mut pool, b"()Ljava/lang/String;"); // 19
    pool.push(12); // 20: NameAndType 18, 19
    u16b(&mut pool, 18);
    u16b(&mut pool, 19);
    pool.push(10); // 21: Methodref 9, 20
    u16b(&mut pool, 9);
    u16b(&mut pool, 20);

    let mut code: Vec<u8> = Vec::with_capacity(11 + 4 * appends);
    code.push(0xbb); // 0: new
    code.extend_from_slice(&9_u16.to_be_bytes());
    code.push(0x59); // 3: dup
    code.push(0xb7); // 4: invokespecial <init>()V
    code.extend_from_slice(&13_u16.to_be_bytes());
    for _ in 0..appends {
        code.push(0x2a); // aload_0
        code.push(0xb6); // invokevirtual append(Ljava/lang/String;)
        code.extend_from_slice(&17_u16.to_be_bytes());
    }
    code.push(0xb6); // invokevirtual toString()Ljava/lang/String;
    code.extend_from_slice(&21_u16.to_be_bytes());
    code.push(0xb0); // areturn

    let mut output = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0); // minor
    u16b(&mut output, 52); // major: Java 8
    u16b(&mut output, 22); // constant_pool_count: the 21 entries above
    output.extend_from_slice(&pool);
    u16b(&mut output, 0x0021); // public super
    u16b(&mut output, 2); // this_class
    u16b(&mut output, 4); // super_class
    u16b(&mut output, 0); // interfaces
    u16b(&mut output, 0); // fields
    u16b(&mut output, 1); // methods
    u16b(&mut output, 0x0009); // public static
    u16b(&mut output, 5); // name → "method"
    u16b(&mut output, 6); // descriptor
    u16b(&mut output, 1); // attributes
    u16b(&mut output, 7); // "Code"
    let mut attribute = Vec::new();
    u16b(&mut attribute, 2); // max_stack
    u16b(&mut attribute, 1); // max_locals: the `String` parameter
    u32b(
        &mut attribute,
        u32::try_from(code.len()).expect("the fixture body fits u32"),
    );
    attribute.extend_from_slice(&code);
    u16b(&mut attribute, 0); // exception table
    u16b(&mut attribute, 0); // code attributes
    u32b(
        &mut output,
        u32::try_from(attribute.len()).expect("the fixture attribute fits u32"),
    );
    output.extend_from_slice(&attribute);
    u16b(&mut output, 0); // class attributes
    output
}

/// The generated chain is one straight line: the same bytes for the same length, and the shape the
/// listing above states.
#[test]
fn the_generated_chain_is_deterministic_and_its_shape_is_the_one_recorded() {
    let fixture = deep_concat_fixture(DEEP_APPENDS);
    assert_eq!(fixture, deep_concat_fixture(DEEP_APPENDS));
    assert_eq!(
        fixture.len(),
        312 + 11 + 4 * DEEP_APPENDS,
        "the class is the fixed pool and member shell plus the body"
    );
    assert_eq!(blake3::hash(&fixture).to_hex().as_str(), DEEP_DIGEST);
}

/// The chain the pre-fix printer aborted on is presented at every length, and the report is a
/// function of the bytes: the same input twice states the same text and the same record.
#[test]
fn a_deep_chain_is_presented_instead_of_aborting() {
    let engine = Engine::new();
    let descriptor = b"(Ljava/lang/String;)Ljava/lang/String;";
    for appends in [1024usize, 1536, DEEP_APPENDS, 4096] {
        let sample = fixture(&engine, &deep_concat_fixture(appends));
        let report = recover(&engine, &sample, b"method", descriptor);
        assert!(
            report.produced(),
            "N = {appends}: the chain is presentable and this run must present it: {:?}",
            report.stop()
        );
        assert_eq!(report.representation, Representation::Java, "N = {appends}");
        assert_eq!(report.quality, Quality::Structured, "N = {appends}");
        assert_eq!(
            report.content,
            RecoveryContent::ContainsStatements,
            "N = {appends}"
        );
        let chain = report
            .concats
            .first()
            .unwrap_or_else(|| panic!("N = {appends}: the chain is recorded"));
        assert_eq!(chain.appends.len(), appends, "N = {appends}");
        assert!(chain.presented(), "N = {appends}");
        assert_eq!(
            report.text.matches(" + ").count(),
            appends - 1,
            "N = {appends}: one part per `append`, joined by the `+`s the chain performs"
        );
        assert_eq!(
            report.text.matches("arg0").count(),
            appends,
            "N = {appends}: each part is the value its own `append` read"
        );

        // The same run again: the artifact is decided by the bytes, not by the stack, the machine's
        // load or the order the tests run in.
        let repeat = recover(&engine, &sample, b"method", descriptor);
        assert_eq!(repeat.text, report.text, "N = {appends}");
        assert_eq!(repeat.concats, report.concats, "N = {appends}");
    }
}

/// A budget that runs out **inside** the deep chain's emission is the existing published stop: no
/// artifact, no segment table, and no abort while the parts are released.
///
/// The bound is the emitter's own (`OutputBytes`), and it is reached thousands of parts into the
/// chain: the run builds every part first and then stops writing, so the structure this change
/// introduced is the one being released here. The stop names the node it stopped at, and the same
/// bytes under the same bound stop the same way — the bound decides it, not the stack or the load.
#[test]
fn a_deep_chain_under_a_short_output_bound_stops_without_an_artifact() {
    let engine = Engine::new();
    let sample = fixture(&engine, &deep_concat_fixture(DEEP_APPENDS));
    let descriptor = b"(Ljava/lang/String;)Ljava/lang/String;";
    let bound = 12_288;
    let mut short = limits();
    short.output_bytes = bound;
    let report = recover_under(&engine, &sample, b"method", descriptor, short.clone());
    let stop = match &report.outcome {
        RecoveryOutcome::Stopped(stop) => stop.clone(),
        other => panic!("the output bound is what refused, not {other:?}"),
    };
    let StopReason::Budget {
        dimension,
        written,
        limit,
        at,
    } = stop
    else {
        panic!("the stop is the bound the emitter charged against");
    };
    assert_eq!(dimension, CountedBudgetDimension::OutputBytes);
    assert_eq!(limit, bound);
    assert!(
        written > 0 && written < bound,
        "the run wrote {written} bytes before the refused charge"
    );
    let at = at.expect("the stop names the node it was writing");
    assert!(
        at > 0 && (at as usize) < 4 * DEEP_APPENDS,
        "the stop landed inside the chain, not before it: BCI {at}"
    );
    assert_eq!(report.content, RecoveryContent::NotProduced);
    assert_eq!(report.text, "", "a stop hands out no artifact");
    assert_eq!(report.source_map.len(), 0, "and no segment table");
    assert_eq!(report.representation, Representation::Bytecode);
    assert!(!report.produced());

    // The bound decides the stop, not the machine: the same request stops at the same node.
    let repeat = recover_under(&engine, &sample, b"method", descriptor, short);
    assert_eq!(repeat.outcome, report.outcome);
}
