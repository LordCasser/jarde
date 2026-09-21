//! `make-required-conversions-explicit`: a value's **presented** type and the type the position it
//! is written in requires are one decision, taken when the expression is built.
//!
//! The defect this file pins (T5): `append((int) c)` — `c` a `char` — was presented as
//! `"" + arg0 + "!"`, and the two texts are two different programs: the class answers `"65!"` for
//! `'A'` and the text answered `"A!"`, while both compile and every plane of the report says
//! `Java`/`Structured`/`contains_statements`/`complete`. `crates/jarde-java/src/build.rs` saved the
//! `append`'s parameter type and adapted `boolean` alone, and the conversion the rest of the family
//! needed happened **in the printing context** instead of in the expression: a `char`, a `byte` and
//! a `short` share one slot shape with an `int`, so a compiler writes `iload` for `append((int) c)`
//! and `"" + c` is a string concatenation of a *character*.
//!
//! The committed sample is `tests/fixtures/p3-required-conversions/` (javac 23.0.1
//! `--release 8 -g:none`, byte count and SHA-256 in its README); the value evidence is the same
//! sample executed by `tests/p3_execution_comparison.rs`, which compiles every recovered body under
//! a declaration derived from the run's own facts and runs it beside the original. This file pins
//! the **text** — which conversion is written, in which position — and the refusal of the shapes no
//! conversion can state, plus the fact that a value whose presented type already is the required one
//! gains nothing.
//!
//! The two hand-built members at the end are the boundary the design's third decision states: a
//! conversion whose legality is not in this layer's evidence (a `boolean` value in an `int`
//! position — a conversion JLS 5.5 forbids) is refused with its bytecode quoted, never published as
//! text a compiler rejects. `javac` never emits them: a `boolean` argument is only passed to a
//! `boolean` parameter, and `flag()` is compared with `1` rather than passed to an `int` one.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 1031 bytes and the SHA-256).
const SAMPLE: &[u8] =
    include_bytes!("fixtures/p3-required-conversions/v8/RequiredConversions.class");

/// Every member the sample declares, so a renamed fixture fails here instead of covering less than
/// this file claims.
const DECLARED: [(&[u8], &[u8]); 13] = [
    (b"<init>", b"()V"),
    (b"castPart", b"(C)Ljava/lang/String;"),
    (b"castPartLast", b"(Ljava/lang/String;C)Ljava/lang/String;"),
    (b"intPart", b"(I)Ljava/lang/String;"),
    (b"widen", b"(I)I"),
    (b"argued", b"(C)I"),
    (b"arguedByte", b"(B)I"),
    (b"returned", b"(C)I"),
    (b"returnedShort", b"(S)I"),
    (b"kept", b"(C)C"),
    (b"declared", b"(C)I"),
    (b"assigned", b"(C)I"),
    (b"written", b"(C)I"),
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

/// The opened sample and the class identity the reader's own header read stated for it.
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

/// One member of the committed sample, as the artifact presents it: the whole body is Java, and the
/// planes agree with what the text is checked for.
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

/// The member's own statements, as the artifact wrote them: the body without the envelope's
/// `// @method …` header and the braces the declaration puts around it.
fn body(report: &RecoveryReport) -> String {
    report
        .text
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "{" && *line != "}")
        .collect::<Vec<_>>()
        .join("\n")
}

/// The member's own `return` statement, as the artifact wrote it.
fn returned(report: &RecoveryReport) -> String {
    report
        .text
        .lines()
        .find(|line| line.trim_start().starts_with("return "))
        .unwrap_or_else(|| panic!("the report has a return statement:\n{}", report.text))
        .trim()
        .to_string()
}

// ---------------------------------------------------------------------------------------------
// The reported defect (T5): the part states the conversion its own `append` performs
// ---------------------------------------------------------------------------------------------

/// `append((int) c)` is a conversion the presentation has to **write**: the `append`'s own parameter
/// descriptor says `int`, the value it reads is a `char`, and `"" + c` converts a character where
/// the bytecode converted the code unit.
#[test]
fn a_converted_part_states_the_appends_own_type() {
    let engine = Engine::new();
    let sample = fixture(&engine, SAMPLE);

    let cast = presented(&sample, b"castPart", b"(C)Ljava/lang/String;");
    assert_eq!(
        returned(&cast),
        "return \"\" + (int) arg0 + \"!\";",
        "the `append(I)` of a `char` value states the conversion:\n{}",
        cast.text
    );
    assert_eq!(
        cast.concats
            .first()
            .expect("the chain is recorded")
            .appends
            .iter()
            .map(|append| (append.bci, append.parameter.as_str()))
            .collect::<Vec<_>>(),
        vec![(8, "int"), (13, "java.lang.String")],
        "the parts are the chain's own `append` calls, in the bytecode's order"
    );

    // The same conversion in a chain that is already a string concatenation: the empty string is not
    // what decides the value, and the converted part keeps its own group in the right operand.
    let last = presented(
        &sample,
        b"castPartLast",
        b"(Ljava/lang/String;C)Ljava/lang/String;",
    );
    assert_eq!(returned(&last), "return arg0 + (int) arg1;");
    assert!(
        !last.text.contains("\"\""),
        "the first part is a `String`, so the chain gains no empty string:\n{}",
        last.text
    );
}

/// The control: a value whose presented type **is** the type the position requires gains nothing.
/// The parameter's own descriptor states `int`, the value is the `int` argument, and the text is the
/// text it was before the conversion node existed.
#[test]
fn a_part_that_already_meets_its_parameter_gains_nothing() {
    let engine = Engine::new();
    let sample = fixture(&engine, SAMPLE);

    let int = presented(&sample, b"intPart", b"(I)Ljava/lang/String;");
    assert_eq!(returned(&int), "return \"\" + arg0 + \"!\";");
    assert!(
        !int.text.contains("(int)"),
        "an `int` part needs no conversion:\n{}",
        int.text
    );

    // The same for the return position: the member returns what the value is presented as.
    let kept = presented(&sample, b"kept", b"(C)C");
    assert_eq!(returned(&kept), "return arg0;");
    assert!(
        !kept.text.contains("(char)"),
        "the value already is the required type:\n{}",
        kept.text
    );
}

// ---------------------------------------------------------------------------------------------
// The same mechanism in the other positions: an argument, a return, a write
// ---------------------------------------------------------------------------------------------

/// A call's argument meets the **callee's own descriptor**: `widen` takes an `int` and the argument
/// is a `char`/`byte`, which the JVM widens with no instruction — so the text has to state it.
#[test]
fn an_argument_states_the_callees_parameter_type() {
    let engine = Engine::new();
    let sample = fixture(&engine, SAMPLE);

    let argued = presented(&sample, b"argued", b"(C)I");
    assert_eq!(returned(&argued), "return widen((int) arg0);");

    let byte = presented(&sample, b"arguedByte", b"(B)I");
    assert_eq!(
        returned(&byte),
        "return widen((int) arg0);",
        "a `byte` argument to an `int` parameter states the same conversion:\n{}",
        byte.text
    );

    // The callee's own body is a control: its return value is its parameter, and the parameter's
    // descriptor says `int`, so nothing converts.
    let widen = presented(&sample, b"widen", b"(I)I");
    assert_eq!(returned(&widen), "return arg0;");
}

/// A `return` meets the **member's own descriptor**: `returned` declares `int` and its value is a
/// `char`; `returnedShort` is the same with a `short`.
#[test]
fn a_return_states_the_members_own_type() {
    let engine = Engine::new();
    let sample = fixture(&engine, SAMPLE);

    assert_eq!(
        returned(&presented(&sample, b"returned", b"(C)I")),
        "return (int) arg0;"
    );
    assert_eq!(
        returned(&presented(&sample, b"returnedShort", b"(S)I")),
        "return (int) arg0;"
    );
}

/// A write meets the **variable's or the field's own type**: the declaration's type is the plan's
/// decision for the local, and a field write's is the descriptor the pool states.
#[test]
fn a_write_states_the_written_types_own_type() {
    let engine = Engine::new();
    let sample = fixture(&engine, SAMPLE);

    let declared = presented(&sample, b"declared", b"(C)I");
    assert_eq!(
        body(&declared),
        "int local1 = (int) arg0;\nreturn local1;",
        "the declaration's value states the conversion:\n{}",
        declared.text
    );

    let assigned = presented(&sample, b"assigned", b"(C)I");
    assert_eq!(
        body(&assigned),
        "int local1 = 0;\nlocal1 = (int) arg0;\nreturn local1;",
        "the assignment after the declaration states it too:\n{}",
        assigned.text
    );

    let written = presented(&sample, b"written", b"(C)I");
    assert_eq!(
        body(&written),
        "RequiredConversions.field = (int) arg0;\nreturn RequiredConversions.field;",
        "the field write states the field descriptor's type:\n{}",
        written.text
    );
}

// ---------------------------------------------------------------------------------------------
// What the conversion node costs, in the counters the rest of this repository bills
// ---------------------------------------------------------------------------------------------

/// The conversion node is a **build-time** decision: it is an AST node the producer adds, not one
/// more IR item and not one more normalization clone. The counters are asserted for the very member
/// the defect was found on — before the change and after it, the same run bills exactly this — so
/// nothing here is claimed to make the corpus cheaper, and the value the text now states is not paid
/// for with a construction the run performs.
///
/// `ir_items` and `normalization_clones` are the two counters the change's verification asks about;
/// `output_bytes` is where the difference *is* (the six characters of `(int) `), and it is asserted
/// too, because a conversion that costs no IR item still costs text.
#[test]
fn the_conversion_costs_no_ir_item_and_no_normalization_clone() {
    let engine = Engine::new();
    let sample = fixture(&engine, SAMPLE);
    let request = MethodAnalysisRequest {
        environment: environment(&sample.snapshot),
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: sample.snapshot.id().clone(),
                },
                class_bytes: sample.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(b"castPart".to_vec()),
            descriptor: JvmBytes(b"(C)Ljava/lang/String;".to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    let mut budget = Budget::new(limits());
    let recovered = engine
        .recover_method(slice::from_ref(&sample.snapshot), &request, &mut budget)
        .expect("a legal request is answered, not raised");
    let usage = budget.usage();
    println!(
        "castPart(C): ir_items={} ir_edges={} analysis_steps={} normalization_clones={} \
         output_bytes={} text_bytes={}",
        usage.ir_items,
        usage.ir_edges,
        usage.analysis_steps,
        usage.normalization_clones,
        usage.output_bytes,
        recovered.recovery().text.len()
    );
    assert!(
        recovered.recovery().text.contains("(int) arg0"),
        "the artifact of this run is the one the counter is read beside:\n{}",
        recovered.recovery().text
    );
    assert_eq!(
        usage.ir_items, 140,
        "the conversion node is not an IR item: the run bills the IR the same body always did"
    );
    assert_eq!(
        usage.normalization_clones, 0,
        "and it is not a normalization clone: nothing is cloned to state the type"
    );
    // What the conversion *does* cost: the six characters `(int) ` and nothing else. The same member
    // without the conversion node writes 218 bytes of text for this body (the counterexample's run,
    // recorded in the change's verification), and the artifact here is 224.
    assert_eq!(
        recovered.recovery().text.len(),
        224,
        "the conversion's own text is `(int) `, six bytes"
    );
}

// ---------------------------------------------------------------------------------------------
// The boundary: a conversion no evidence states is a refusal, never a guess
// ---------------------------------------------------------------------------------------------

/// The premise: the sample declares exactly the members this file classifies, and every one of them
/// is answered (presented or refused) rather than missed.
#[test]
fn every_declared_member_of_the_sample_is_covered() {
    let engine = Engine::new();
    let sample = fixture(&engine, SAMPLE);
    let mut budget = Budget::new(limits());
    let inspected = engine
        .inspect_header(
            &sample.snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    let declared: Vec<(Vec<u8>, Vec<u8>)> = inspected
        .inspection
        .header
        .methods
        .iter()
        .map(|member| {
            (
                member.name.raw().0.clone(),
                member.descriptor.raw().0.clone(),
            )
        })
        .collect();
    for (name, descriptor) in DECLARED {
        assert!(
            declared
                .iter()
                .any(|(member, spelled)| member == name && spelled == descriptor),
            "the sample declares `{}`",
            String::from_utf8_lossy(name)
        );
        let report = recover(&engine, &sample, name, descriptor);
        assert!(
            report.produced() && !report.text.is_empty(),
            "`{}`: {:?}\n{}",
            String::from_utf8_lossy(name),
            report.outcome,
            report.text
        );
    }
}

/// One member of the hand-built [`unstateable_conversions`]: the two pool indexes its declaration
/// names and the body it carries.
struct HandMember<'a> {
    /// The constant-pool index of the member's name.
    name_pool: u16,
    /// The constant-pool index of its descriptor.
    descriptor_pool: u16,
    /// The body, as the bytes the `Code` attribute holds.
    code: &'a [u8],
    /// The `max_stack` its body needs.
    max_stack: u16,
}

/// One hand-built class for the conversions no compiler emits, because no Java conversion exists
/// between the two types: a `boolean` value passed to an `int` parameter, and a `boolean` value
/// appended through `append(I)`.
///
/// JLS 5.5 states it outright: there is no conversion from `boolean` to any other primitive, so
/// neither a cast nor a widening exists to write — `take(arg0)` for a `boolean` argument is refused
/// by `javac` with `incompatible types: boolean cannot be converted to int`. The bytes are the two
/// shapes a `boolean` and an `int` share; the presentation must refuse the region and quote it.
fn unstateable_conversions() -> Vec<u8> {
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
    utf8(&mut pool, b"p/RequiredBoundary"); // 1
    pool.push(7); // 2: Class 1
    u16b(&mut pool, 1);
    utf8(&mut pool, b"java/lang/Object"); // 3
    pool.push(7); // 4: Class 3
    u16b(&mut pool, 3);
    utf8(&mut pool, b"Code"); // 5
    utf8(&mut pool, b"take"); // 6
    utf8(&mut pool, b"(I)I"); // 7
    utf8(&mut pool, b"probe"); // 8
    utf8(&mut pool, b"(Z)I"); // 9
    pool.push(12); // 10: NameAndType 6, 7
    u16b(&mut pool, 6);
    u16b(&mut pool, 7);
    pool.push(10); // 11: Methodref 2, 10
    u16b(&mut pool, 2);
    u16b(&mut pool, 10);
    utf8(&mut pool, b"chain"); // 12
    utf8(&mut pool, b"(Z)Ljava/lang/String;"); // 13
    utf8(&mut pool, b"java/lang/StringBuilder"); // 14
    pool.push(7); // 15: Class 14
    u16b(&mut pool, 14);
    utf8(&mut pool, b"<init>"); // 16
    utf8(&mut pool, b"()V"); // 17
    pool.push(12); // 18: NameAndType 16, 17
    u16b(&mut pool, 16);
    u16b(&mut pool, 17);
    pool.push(10); // 19: Methodref 15, 18
    u16b(&mut pool, 15);
    u16b(&mut pool, 18);
    utf8(&mut pool, b"append"); // 20
    utf8(&mut pool, b"(I)Ljava/lang/StringBuilder;"); // 21
    pool.push(12); // 22: NameAndType 20, 21
    u16b(&mut pool, 20);
    u16b(&mut pool, 21);
    pool.push(10); // 23: Methodref 15, 22
    u16b(&mut pool, 15);
    u16b(&mut pool, 22);
    utf8(&mut pool, b"(Ljava/lang/String;)Ljava/lang/StringBuilder;"); // 24
    pool.push(12); // 25: NameAndType 20, 24
    u16b(&mut pool, 20);
    u16b(&mut pool, 24);
    pool.push(10); // 26: Methodref 15, 25
    u16b(&mut pool, 15);
    u16b(&mut pool, 25);
    utf8(&mut pool, b"toString"); // 27
    utf8(&mut pool, b"()Ljava/lang/String;"); // 28
    pool.push(12); // 29: NameAndType 27, 28
    u16b(&mut pool, 27);
    u16b(&mut pool, 28);
    pool.push(10); // 30: Methodref 15, 29
    u16b(&mut pool, 15);
    u16b(&mut pool, 29);
    utf8(&mut pool, b"!"); // 31
    pool.push(8); // 32: String 31
    u16b(&mut pool, 31);

    // take(I)I: `iload_0; ireturn` — the callee whose own descriptor states the parameter's type.
    let take: Vec<u8> = vec![0x1a, 0xac];
    // probe(Z)I: `iload_0; invokestatic take:(I)I; ireturn` — the `boolean` value in an `int`
    // position, which the JVM's one slot shape for both makes the only thing that states it.
    let probe: Vec<u8> = vec![0x1a, 0xb8, 0x00, 0x0b, 0xac];
    // chain(Z)Ljava/lang/String;: the same `boolean` value appended through `append(I)`, with the
    // two parts a chain needs and the `toString` it ends in.
    let chain: Vec<u8> = vec![
        0xbb, 0x00, 0x0f, // 0: new java/lang/StringBuilder
        0x59, // 3: dup
        0xb7, 0x00, 0x13, // 4: invokespecial <init>()V
        0x1a, // 7: iload_0
        0xb6, 0x00, 0x17, // 8: invokevirtual append:(I)Ljava/lang/StringBuilder;
        0x12, 0x20, // 11: ldc "!"
        0xb6, 0x00, 0x1a, // 13: invokevirtual append:(Ljava/lang/String;)…
        0xb6, 0x00, 0x1e, // 16: invokevirtual toString:()Ljava/lang/String;
        0xb0, // 19: areturn
    ];

    let mut output = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0); // minor
    u16b(&mut output, 52); // major: Java 8
    u16b(&mut output, 33); // constant_pool_count: the 32 entries above
    output.extend_from_slice(&pool);
    u16b(&mut output, 0x0021); // public super
    u16b(&mut output, 2); // this_class
    u16b(&mut output, 4); // super_class
    u16b(&mut output, 0); // interfaces
    u16b(&mut output, 0); // fields
    let members: [HandMember<'_>; 3] = [
        HandMember {
            name_pool: 6,
            descriptor_pool: 7,
            code: &take,
            max_stack: 1,
        },
        HandMember {
            name_pool: 8,
            descriptor_pool: 9,
            code: &probe,
            max_stack: 1,
        },
        HandMember {
            name_pool: 12,
            descriptor_pool: 13,
            code: &chain,
            max_stack: 2,
        },
    ];
    u16b(
        &mut output,
        u16::try_from(members.len()).expect("member count fits u16"),
    );
    for member in members {
        u16b(&mut output, 0x0009); // public static
        u16b(&mut output, member.name_pool);
        u16b(&mut output, member.descriptor_pool);
        u16b(&mut output, 1); // one attribute: Code
        u16b(&mut output, 5); // "Code"
        let mut attribute = Vec::new();
        u16b(&mut attribute, member.max_stack);
        u16b(&mut attribute, 1); // max_locals: the one `Z` parameter
        u32b(
            &mut attribute,
            u32::try_from(member.code.len()).expect("the fixture body fits u32"),
        );
        attribute.extend_from_slice(member.code);
        u16b(&mut attribute, 0); // exception table
        u16b(&mut attribute, 0); // code attributes
        u32b(
            &mut output,
            u32::try_from(attribute.len()).expect("the fixture attribute fits u32"),
        );
        output.extend_from_slice(&attribute);
    }
    u16b(&mut output, 0); // class attributes
    output
}

/// A value whose presented type and required type no conversion connects is refused with its
/// bytecode quoted — the design's third decision, and the second half of C02.
#[test]
fn a_conversion_no_evidence_states_is_refused() {
    let engine = Engine::new();
    let sample = fixture(&engine, &unstateable_conversions());

    // The argument: `take`'s own descriptor takes an `int`, the value is the member's `boolean`
    // parameter, and the text `take(arg0)` is a call `javac` refuses.
    let probe = recover(&engine, &sample, b"probe", b"(Z)I");
    assert_eq!(probe.representation, Representation::Mixed, "{probe:?}");
    assert_eq!(probe.quality, Quality::Fallback, "{probe:?}");
    assert!(
        !probe.text.contains("take(arg0)"),
        "the text the compiler rejects is published nowhere:\n{}",
        probe.text
    );
    assert!(
        probe.text.contains("// @bytecode"),
        "the refused region quotes its bytecode:\n{}",
        probe.text
    );
    assert!(
        probe.text.contains("boolean") && probe.text.contains("int"),
        "the refusal names the two types it could not connect:\n{}",
        probe.text
    );

    // The chain: `append(I)` reading the same `boolean` value. `"" + arg0` would write `\"true\"`
    // where the bytecode appended the `int` the slot holds, and `(int) arg0` is not a conversion
    // Java has.
    let chain = recover(&engine, &sample, b"chain", b"(Z)Ljava/lang/String;");
    assert_eq!(chain.representation, Representation::Mixed, "{chain:?}");
    assert_eq!(chain.quality, Quality::Fallback, "{chain:?}");
    assert!(
        chain.text.contains("// @bytecode"),
        "the refused chain quotes its bytecode:\n{}",
        chain.text
    );
    assert!(
        !chain.text.contains("(int)") && !chain.text.contains("\"\" +"),
        "neither a conversion nor a concatenation is published for the refused chain:\n{}",
        chain.text
    );
}
