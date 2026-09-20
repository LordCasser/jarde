//! The content classification of a delivered artifact: what `RecoveryReport::content` states, and
//! what it must not be read as.
//!
//! The property this file pins is the one the design's Risk section names first: `Produced` says an
//! artifact was delivered, and *nothing* about statements, completeness, compilability or semantic
//! equivalence. The classification therefore has to come from the committed structure — the
//! statements the emitter actually wrote — and never from the text, from a token count, or from the
//! existing `Builder::push` counter of `program.statements` (which counts `StmtKind::Fallback` nodes,
//! and a fallback node writes a reason and the bytecode it quotes, not a statement).
//!
//! Three kinds of input state it, all through the entry point the CLI calls
//! ([`Engine::recover_method`]):
//!
//! * the **committed** javac 23.0.1 samples whose bodies are the cases the acceptance names:
//!   `RefusedCast.fieldCast` (no statement at all — its artifact is comment lines and two braces),
//!   `NestedEval.nestedPlain` (a whole body written as Java), `NestedEval.nestedLocal` (a statement
//!   *and* a quote in one artifact) and `RefusedCast.leftRead`;
//! * a **hand-built** class (`fixture_class` below) for the two shapes no committed sample has: a
//!   body whose only statement is `return;`, and a branch whose `then` arm is empty
//!   (`if (arg0) { } else { … }`, whose condition is still evaluated). The in-memory generator is
//!   the repository's own second fixture kind (`tests/fixtures/README.md`): a compiled sample cannot
//!   be added here without moving the reader's pinned fixture population
//!   (`crates/jarde-reader/src/classfile.rs`, `(44, 168, 44, 86, 8)`), and this class's bytes are
//!   written by the test itself, so nothing about them needs a compiler at run time. The two string
//!   literals it loads are `a//b` and `a/*b*/`: a classifier that stripped comments and counted
//!   tokens would see two different artifacts where the structure is one `return <literal>;`;
//! * the **stops**: an output bound, a cancellation token, and a request whose schedule stops the
//!   payload short of the SSA table. None of them may report a statement, and none of them may carry
//!   text or segments — the stop contract is unchanged, and the classification is written only with
//!   the artifact that was committed.

use jarde::*;
use std::slice;

/// The committed P3-R9 sample: `fieldCast` is a refusal whose artifact is quotes alone, and `leftRead`
/// is a body written whole.
const REFUSED_CAST: &[u8] = include_bytes!("fixtures/p3-refused-cast/v8/RefusedCast.class");

/// The committed P3-R8 sample: `nestedPlain` is written whole, `nestedLocal` keeps the write it can
/// prove beside the quote it owes.
const NESTED_EVAL: &[u8] = include_bytes!("fixtures/p3-nested-eval/v8/NestedEval.class");

// -------------------------------------------------------------------------------------------
// The budget, the environment and the entry helpers every case below is made through.
// -------------------------------------------------------------------------------------------

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

/// One caller domain rooted at the fixture's own snapshot, and nothing else: the simplest environment
/// the library's validator accepts without a problem.
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

/// The opened class and the identity the reader's own header read stated for it.
struct Fixture {
    snapshot: ArtifactSnapshot,
    class_bytes: ClassBytesId,
    /// The `(name, descriptor)` of every member the class declares, as the header read them.
    declared: Vec<(Vec<u8>, Vec<u8>)>,
}

/// Opens one class and reads its header through the reader's own entry point, so that the members
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
    let declared = inspected
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
    Fixture {
        snapshot,
        class_bytes: inspected.source.class_bytes.clone(),
        declared,
    }
}

/// The request of one member of an opened fixture: the physical identity the header read published,
/// and the schedule the case states.
fn request(
    fixture: &Fixture,
    name: &[u8],
    descriptor: &[u8],
    stages: Vec<AnalysisStage>,
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
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
        stages,
    }
}

/// One recovery run under one budget, through the entry point the CLI calls: both halves of the
/// answer, so a case can read the run's own execution plane beside the presentation.
fn recovered_with(
    engine: &Engine,
    fixture: &Fixture,
    name: &[u8],
    descriptor: &[u8],
    stages: Vec<AnalysisStage>,
    budget: &mut Budget,
) -> RecoveredMethod {
    let request = request(fixture, name, descriptor, stages);
    engine
        .recover_method(slice::from_ref(&fixture.snapshot), &request, budget)
        .expect("a legal request is answered, not raised")
}

/// One recovery run under one budget, through the entry point the CLI calls.
fn recover_with(
    engine: &Engine,
    fixture: &Fixture,
    name: &[u8],
    descriptor: &[u8],
    stages: Vec<AnalysisStage>,
    budget: &mut Budget,
) -> RecoveryReport {
    recovered_with(engine, fixture, name, descriptor, stages, budget)
        .recovery()
        .clone()
}

/// One recovery run with the file's own ample budget.
fn recover(engine: &Engine, fixture: &Fixture, name: &[u8], descriptor: &[u8]) -> RecoveryReport {
    recover_with(
        engine,
        fixture,
        name,
        descriptor,
        AnalysisStage::ALL.to_vec(),
        &mut Budget::new(limits()),
    )
}

/// The descriptor the header read stated for one member: a renamed member fails here instead of
/// quietly covering less.
fn descriptor_of(fixture: &Fixture, name: &[u8]) -> Vec<u8> {
    fixture
        .declared
        .iter()
        .find(|(declared, _)| declared.as_slice() == name)
        .map(|(_, descriptor)| descriptor.clone())
        .unwrap_or_else(|| {
            panic!(
                "the fixture declares `{}`: {:?}",
                String::from_utf8_lossy(name),
                fixture.declared
            )
        })
}

// -------------------------------------------------------------------------------------------
// The hand-built fixture: the two shapes no committed sample has.
// -------------------------------------------------------------------------------------------

fn u16b(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn u32b(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn utf8(output: &mut Vec<u8>, value: &[u8]) {
    output.push(1);
    u16b(
        output,
        u16::try_from(value.len()).expect("fixture UTF-8 length fits u16"),
    );
    output.extend_from_slice(value);
}

/// One member of the hand-built fixture: its name, its descriptor, and the `Code` attribute's own
/// bytes and slot counts.
struct HandMember {
    name: &'static str,
    descriptor: &'static str,
    code: &'static [u8],
    max_stack: u16,
    max_locals: u16,
}

/// The two string literals the hand-built bodies load: a classifier that stripped comments would see
/// two different artifacts where the structure is one `return <literal>;`.
const LITERAL_A: &[u8] = b"a//b";
const LITERAL_B: &[u8] = b"a/*b*/";

/// The members of the hand-built class, in method-table order, with the bytecode of each:
///
/// ```text
/// nothing()V                     0: return
/// emptyArm(ZI)I                  0: iload_1; 1: istore_2; 2: iload_0; 3: ifeq 9; 6: goto 13;
///                                9: iload_1; 10: iconst_1; 11: iadd; 12: istore_2; 13: iload_2; 14: ireturn
/// literalA()Ljava/lang/String;   0: ldc "a//b"; 2: areturn
/// literalB()Ljava/lang/String;   0: ldc "a/*b*/"; 2: areturn
/// castA()Ljava/lang/String;      0: aconst_null; 1: checkcast java/lang/String; 4: areturn
/// castB()Ljava/lang/String;      0: nop; 1: aconst_null; 2: checkcast java/lang/String; 5: areturn
/// nestedReturn(Z)I               0: iload_0; 1: ifeq 6; 4: iconst_1; 5: ireturn; 6: iconst_0; 7: ireturn
/// ```
///
/// `emptyArm` is the shape a compiler writes for an `if` whose `then` arm is empty: the branch
/// transfers to the join past the empty arm, the condition is still evaluated, and the artifact is
/// `if (arg0) { } else { local2 = arg1 + 1; }` over a hoisted declaration. `castA`/`castB` are two
/// bodies whose cast no rule proves: their artifacts hold reasons and quoted bytecode and no
/// statement at all, and the two bodies differ by a `nop` so that the *comment wording and the quoted
/// indexes* are not the same either — the classification has to be the same anyway.
///
/// `nestedReturn` is the same `return` statement one indentation level deeper than `nothing`'s, so
/// the two artifacts differ in whitespace while the classification does not.
const HAND_MEMBERS: &[HandMember] = &[
    HandMember {
        name: "nothing",
        descriptor: "()V",
        code: &[0xb1],
        max_stack: 0,
        max_locals: 0,
    },
    HandMember {
        name: "emptyArm",
        descriptor: "(ZI)I",
        code: &[
            0x1b, 0x3d, 0x1a, 0x99, 0x00, 0x06, 0xa7, 0x00, 0x07, 0x1b, 0x04, 0x60, 0x3d, 0x1c,
            0xac,
        ],
        max_stack: 2,
        max_locals: 3,
    },
    HandMember {
        name: "literalA",
        descriptor: "()Ljava/lang/String;",
        code: &[0x12, 9, 0xb0],
        max_stack: 1,
        max_locals: 0,
    },
    HandMember {
        name: "literalB",
        descriptor: "()Ljava/lang/String;",
        code: &[0x12, 15, 0xb0],
        max_stack: 1,
        max_locals: 0,
    },
    HandMember {
        name: "castA",
        descriptor: "()Ljava/lang/String;",
        code: &[0x01, 0xc0, 0x00, 0x14, 0xb0],
        max_stack: 1,
        max_locals: 0,
    },
    HandMember {
        name: "castB",
        descriptor: "()Ljava/lang/String;",
        code: &[0x00, 0x01, 0xc0, 0x00, 0x14, 0xb0],
        max_stack: 1,
        max_locals: 0,
    },
    HandMember {
        name: "nestedReturn",
        descriptor: "(Z)I",
        code: &[0x1a, 0x99, 0x00, 0x05, 0x04, 0xac, 0x03, 0xac],
        max_stack: 1,
        max_locals: 1,
    },
];

/// The hand-built class `Content`: one `Code` attribute per member of [`HAND_MEMBERS`], and a
/// constant pool holding the two string literals and the cast type their bodies name. The bytes are
/// written by this function, so the case that reads them states its own input.
fn fixture_class() -> Vec<u8> {
    let mut output = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0); // minor_version
    u16b(&mut output, 52); // major_version: Java 8, the profile every request declares
    u16b(&mut output, 23); // constant_pool_count = 22 entries + 1
    utf8(&mut output, b"Content"); // 1
    output.push(7);
    u16b(&mut output, 1); // 2: Class Content
    utf8(&mut output, b"java/lang/Object"); // 3
    output.push(7);
    u16b(&mut output, 3); // 4: Class java/lang/Object
    utf8(&mut output, b"Code"); // 5
    utf8(&mut output, b"nothing"); // 6
    utf8(&mut output, b"()V"); // 7
    utf8(&mut output, b"emptyArm"); // 8
    output.push(8);
    u16b(&mut output, 11); // 9: String a//b
    utf8(&mut output, b"(ZI)I"); // 10
    utf8(&mut output, LITERAL_A); // 11
    utf8(&mut output, b"literalA"); // 12
    utf8(&mut output, b"()Ljava/lang/String;"); // 13
    utf8(&mut output, LITERAL_B); // 14
    output.push(8);
    u16b(&mut output, 14); // 15: String a/*b*/
    utf8(&mut output, b"literalB"); // 16
    utf8(&mut output, b"castA"); // 17
    utf8(&mut output, b"castB"); // 18
    utf8(&mut output, b"java/lang/String"); // 19
    output.push(7);
    u16b(&mut output, 19); // 20: Class java/lang/String
    utf8(&mut output, b"nestedReturn"); // 21
    utf8(&mut output, b"(Z)I"); // 22

    u16b(&mut output, 0x21); // access_flags: public super
    u16b(&mut output, 2); // this_class
    u16b(&mut output, 4); // super_class
    u16b(&mut output, 0); // interfaces
    u16b(&mut output, 0); // fields
    u16b(
        &mut output,
        u16::try_from(HAND_MEMBERS.len()).expect("member count fits u16"),
    );
    for member in HAND_MEMBERS {
        let name_index = match member.name {
            "nothing" => 6,
            "emptyArm" => 8,
            "literalA" => 12,
            "literalB" => 16,
            "castA" => 17,
            "castB" => 18,
            "nestedReturn" => 21,
            other => panic!("no pool entry for `{other}`"),
        };
        let descriptor_index = match member.descriptor {
            "()V" => 7,
            "(ZI)I" => 10,
            "()Ljava/lang/String;" => 13,
            "(Z)I" => 22,
            other => panic!("no pool entry for `{other}`"),
        };
        u16b(&mut output, 0x0009); // public static
        u16b(&mut output, name_index);
        u16b(&mut output, descriptor_index);
        u16b(&mut output, 1); // one attribute: Code
        u16b(&mut output, 5); // "Code"
        let mut attribute = Vec::new();
        u16b(&mut attribute, member.max_stack);
        u16b(&mut attribute, member.max_locals);
        u32b(
            &mut attribute,
            u32::try_from(member.code.len()).expect("fixture code length fits u32"),
        );
        attribute.extend_from_slice(member.code);
        u16b(&mut attribute, 0); // exception_table_length
        u16b(&mut attribute, 0); // attributes_count of the Code attribute
        u32b(
            &mut output,
            u32::try_from(attribute.len()).expect("attribute length fits u32"),
        );
        output.extend_from_slice(&attribute);
    }
    u16b(&mut output, 0); // class attributes
    output
}

// -------------------------------------------------------------------------------------------
// The cases.
// -------------------------------------------------------------------------------------------

/// Asserts one produced artifact's classification and a fragment of its own text, so that a case
/// cannot pass by classifying an artifact this file did not intend. The three planes the
/// classification must not move are asserted here too: content is its own axis, not a second quality
/// system.
fn assert_case(report: &RecoveryReport, content: RecoveryContent, text_holds: &str) {
    assert!(
        report.produced(),
        "the case is about a produced artifact: {:?}",
        report.outcome
    );
    assert_eq!(
        report.content, content,
        "`{}`: {}\nregions: {:?}",
        report.method, report.text, report.regions
    );
    assert!(
        report.text.contains(text_holds),
        "`{}` holds `{text_holds}`:\n{}",
        report.method,
        report.text
    );
    assert_eq!(report.compile_status, CompileStatus::NotAttempted);
    assert_eq!(report.semantic_validation, SemanticValidation::Unproven);
    assert_eq!(report.verification, VerificationStatus::NotPerformed);
}

#[test]
fn explanation_only_means_no_emitted_statement() {
    // The discriminating case the design's Risk section names: an artifact whose text is *not* empty
    // — comment lines and two braces — and which holds no statement at all. A classifier that read
    // the text (stripping comments to count what is left, or looking for `//`), or one that exposed
    // the existing `program.statements` counter (which counts the two fallback nodes of this body),
    // would call this `contains_statements`. The body is `getstatic External.value; checkcast;
    // areturn`, both the read and the cast are refused, and the whole body is quoted.
    let engine = Engine::new();
    let fixture = fixture(&engine, REFUSED_CAST);
    let descriptor = descriptor_of(&fixture, b"fieldCast");
    let report = recover(&engine, &fixture, b"fieldCast", &descriptor);

    assert_case(
        &report,
        RecoveryContent::ExplanationOnly,
        "// @bytecode 3 0",
    );
    assert_eq!(report.representation, Representation::Mixed);
    assert_eq!(report.quality, Quality::Fallback);
    // What the artifact is made of, stated over the artifact itself: every line is a comment, a brace
    // or empty. That is what "no statement was emitted" means here, and it is asserted on the text so
    // that the classification is checked against something a reader can see, not against itself.
    for line in report.text.lines() {
        let trimmed = line.trim();
        assert!(
            trimmed.is_empty() || trimmed.starts_with("//") || trimmed == "{" || trimmed == "}",
            "the artifact holds no statement line:\n{}",
            report.text
        );
    }
    assert!(
        report.text.len() > 100,
        "and its text is not empty, which is exactly why the classification may not be read from \
         it:\n{}",
        report.text
    );

    // The same rule for the sample's other two refusals: an artifact of quotes alone, whatever the
    // instructions behind it are.
    for name in [b"instanceCast".as_slice(), b"chainCast"] {
        let descriptor = descriptor_of(&fixture, name);
        let report = recover(&engine, &fixture, name, &descriptor);
        assert_case(&report, RecoveryContent::ExplanationOnly, "// @bytecode");
    }
}

#[test]
fn a_committed_statement_makes_the_artifact_contain_statements() {
    // The three committed controls in the other direction. `nestedPlain` is written whole, one
    // statement; `nestedLocal` holds the write the run can prove *and* the quote it owes, so a
    // fallback beside a statement does not take the statement away; `leftRead` is written whole too.
    let engine = Engine::new();
    let nested = fixture(&engine, NESTED_EVAL);

    let plain = recover(
        &engine,
        &nested,
        b"nestedPlain",
        &descriptor_of(&nested, b"nestedPlain"),
    );
    assert_case(
        &plain,
        RecoveryContent::ContainsStatements,
        // The parentheses are the printer's grouping: `(x + 1) + (x + 2)` is a sum on the right of
        // a sum, and the left-associative text without them would be a different tree.
        "return arg0 + 1 + (arg0 + 2);",
    );
    assert_eq!(plain.representation, Representation::Java);
    assert_eq!(plain.quality, Quality::Structured);

    let local = recover(
        &engine,
        &nested,
        b"nestedLocal",
        &descriptor_of(&nested, b"nestedLocal"),
    );
    assert_case(
        &local,
        RecoveryContent::ContainsStatements,
        "arg0 = arg0 + 1;",
    );
    assert!(
        local.text.contains("// @bytecode 8 0"),
        "the artifact holds the quote it owes beside that statement:\n{}",
        local.text
    );
    assert_eq!(local.representation, Representation::Mixed);
    assert_eq!(local.quality, Quality::Fallback);

    let refused = fixture(&engine, REFUSED_CAST);
    let left = recover(
        &engine,
        &refused,
        b"leftRead",
        &descriptor_of(&refused, b"leftRead"),
    );
    assert_case(
        &left,
        RecoveryContent::ContainsStatements,
        "return External.count + tick();",
    );
}

#[test]
fn the_hand_built_shapes_classify_from_their_structure() {
    let engine = Engine::new();
    let fixture = fixture(&engine, &fixture_class());

    // `return;` alone: the smallest statement a body can hold. It is a statement, not a wrapper or a
    // brace, so the artifact contains one.
    let nothing = recover(
        &engine,
        &fixture,
        b"nothing",
        &descriptor_of(&fixture, b"nothing"),
    );
    assert_case(
        &nothing,
        RecoveryContent::ContainsStatements,
        "    return;\n",
    );
    assert_eq!(nothing.representation, Representation::Java);
    assert_eq!(nothing.quality, Quality::Structured);

    // The empty branch: `if (arg0) { } else { … }`. The `then` arm holds nothing, but the branch
    // evaluates its condition, so the statement is there — and the body holds no fallback at all,
    // which is the other half of why the classification cannot be read from `program.statements`.
    let arm = recover(
        &engine,
        &fixture,
        b"emptyArm",
        &descriptor_of(&fixture, b"emptyArm"),
    );
    assert_case(
        &arm,
        RecoveryContent::ContainsStatements,
        "if (arg0) {\n    } else {",
    );
    assert_eq!(arm.representation, Representation::Java);
    assert_eq!(arm.quality, Quality::Structured);
    assert!(
        !arm.text.contains("// @bytecode"),
        "nothing in this body had to be quoted:\n{}",
        arm.text
    );

    // The two literals: the same `return <expr>;` with `//` and `/* */` inside the string. The texts
    // differ, the classification does not.
    let a = recover(
        &engine,
        &fixture,
        b"literalA",
        &descriptor_of(&fixture, b"literalA"),
    );
    assert_case(&a, RecoveryContent::ContainsStatements, "return \"a//b\";");
    let b = recover(
        &engine,
        &fixture,
        b"literalB",
        &descriptor_of(&fixture, b"literalB"),
    );
    assert_case(
        &b,
        RecoveryContent::ContainsStatements,
        "return \"a/*b*/\";",
    );
    assert_ne!(a.text, b.text, "the two artifacts are not the same text");

    // The two refusals: no statement in either artifact, and the comment lines behind them are not
    // the same words either (different quoted indexes, a different number of reasons) — the
    // classification follows the structure, not the comment.
    let cast_a = recover(
        &engine,
        &fixture,
        b"castA",
        &descriptor_of(&fixture, b"castA"),
    );
    assert_case(&cast_a, RecoveryContent::ExplanationOnly, "// @bytecode 1");
    let cast_b = recover(
        &engine,
        &fixture,
        b"castB",
        &descriptor_of(&fixture, b"castB"),
    );
    assert_case(&cast_b, RecoveryContent::ExplanationOnly, "// @bytecode 0");
    assert!(
        cast_a.text.contains("// @bytecode 1") && cast_b.text.contains("// @bytecode 2"),
        "the two refusals name different bytecode and write different reasons:\n{}\n---\n{}",
        cast_a.text,
        cast_b.text
    );
    assert_ne!(cast_a.text, cast_b.text);
    assert_eq!(cast_a.quality, Quality::Fallback);
    assert_eq!(cast_b.quality, Quality::Fallback);

    // The same `return` one level deeper: different whitespace, same classification.
    let nested = recover(
        &engine,
        &fixture,
        b"nestedReturn",
        &descriptor_of(&fixture, b"nestedReturn"),
    );
    assert_case(
        &nested,
        RecoveryContent::ContainsStatements,
        "        return 1;\n",
    );
    assert_ne!(
        nested.text, nothing.text,
        "the two artifacts differ in their indentation, and in nothing else that matters here"
    );
}

#[test]
fn a_stopped_run_reports_not_produced() {
    let engine = Engine::new();
    let fixture = fixture(&engine, NESTED_EVAL);
    let descriptor = descriptor_of(&fixture, b"nestedLocal");

    // The control: with an ample budget this very member is a produced artifact holding a statement,
    // and the report states how many output bytes the run charged for it.
    let produced = recover(&engine, &fixture, b"nestedLocal", &descriptor);
    assert_eq!(produced.content, RecoveryContent::ContainsStatements);
    assert!(!produced.text.is_empty());
    let charged = output_bytes(&produced);
    let artifact = u64::try_from(produced.text.len()).expect("the artifact length fits u64");
    assert!(charged > artifact, "the run charged more than its artifact");

    // The output bound: the body builds statements, the emitter refuses the first write of the
    // artifact and discards the buffer, so the report states a stop and *not* the content of the AST
    // it had already built. The bound is derived from the run itself rather than guessed: a run that
    // may charge everything the ample one charged except the artifact's own bytes is exactly a run
    // the analysis completes and the first emitter write refuses — a smaller bound would stop the
    // class read, which is a different case.
    let mut tight = Budget::new(Limits {
        output_bytes: charged - artifact,
        ..limits()
    });
    let stopped = recover_with(
        &engine,
        &fixture,
        b"nestedLocal",
        &descriptor,
        AnalysisStage::ALL.to_vec(),
        &mut tight,
    );
    assert!(
        matches!(
            stopped.outcome,
            RecoveryOutcome::Stopped(StopReason::Budget {
                dimension: CountedBudgetDimension::OutputBytes,
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
    assert_eq!(stopped.representation, Representation::Bytecode);
    assert!(!stopped.produced());

    // The cancellation: the caller's token stops the run before the presentation has a payload at
    // all, and the answer is the same kind of stop — the run's own execution plane says it was
    // cancelled, and the report commits no artifact and states no content.
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(limits(), token);
    let recovered = recovered_with(
        &engine,
        &fixture,
        b"nestedLocal",
        &descriptor,
        AnalysisStage::ALL.to_vec(),
        &mut cancelled,
    );
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
    assert_eq!(
        stopped.text, "",
        "a cancelled run hands out no artifact either"
    );
    assert_eq!(stopped.source_map.len(), 0);
    assert!(!stopped.produced());

    // The stop inside a successful response the CLI's own case states too: a request whose schedule
    // never asked for SSA has no payload to present, and the answer is a stop in the report rather
    // than a raised error or an empty body.
    let stopped = recover_with(
        &engine,
        &fixture,
        b"nestedLocal",
        &descriptor,
        vec![AnalysisStage::Frame],
        &mut Budget::new(limits()),
    );
    assert_eq!(
        stopped.outcome,
        RecoveryOutcome::Stopped(StopReason::IrTableMissing { table: "ssa" })
    );
    assert_eq!(stopped.content, RecoveryContent::NotProduced);
    assert_eq!(stopped.text, "");
    assert_eq!(stopped.source_map.len(), 0);
}

/// The output bytes one finished run charged.
fn output_bytes(report: &RecoveryReport) -> u64 {
    match &report.execution {
        ExecutionReport::Complete { usage } => usage.output_bytes,
        other => panic!("a produced run completes: {other:?}"),
    }
}
