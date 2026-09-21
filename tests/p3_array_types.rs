//! P3 acceptance: a **type position** is spelled as a legal Java type, and a descriptor this layer
//! cannot spell is refused instead of published.
//!
//! Everything here goes through the entry point the CLI calls ([`Engine::recover_method`]) over the
//! committed javac 23.0.1 sample `tests/fixtures/p3-array-types/` (whose README states the command,
//! the digest and the bytecode of every member), plus one hand-built class for the descriptors no
//! compiler can state a name for.
//!
//! # Why the planes cannot answer this question
//!
//! A local declared from an array-typed value was reported `Java`/`Structured`/`contains_statements`
//! — the artifact was structurally whole — while its declaration published the frames' own
//! descriptor: `[B local1 = copy(arg0);`, `[Ljava.lang.String; local1 = arg0;`,
//! `[[I local1 = arg0;`, `[[Ljava.lang.String; local1 = arg0;`. `javac --release 8` refuses the
//! first at the `[B` (`illegal start of expression`), the reference forms twice over (`illegal start
//! of expression` at the `[` and `not a statement` at the `.`), and the multi-dimensional ones the
//! same way. A plane that counts statements cannot see a type, so the evidence is the text plus a
//! compiler — which is what the ignored `tests/p3_execution_comparison.rs` row for this sample does
//! with the same bytes.
//!
//! # The facts, and what the fix is
//!
//! The frame pass keeps a named reference in the class file's own descriptor form
//! (`jarde_jvm::frame::RefType::Named`), and arrays are named by their whole descriptor
//! ([`jarde_jvm::frame::parse_field_type`]'s `[` branch). The spelling entry point
//! (`build::spell_reference`) only took the `L…;` wrapping off, so an array descriptor went into the
//! text untouched. It now reads an array through the crate's one descriptor→Java type parser
//! ([`jarde_jvm`]-side is the reader, `lambda::parse_type` is this crate's), which is what makes the
//! declaration here and a lambda parameter spell `[[Ljava/lang/String;` the same way — and it is
//! fallible, so a descriptor with no Java spelling is refused at the position that would have
//! carried it.
//!
//! # The positions that write a type
//!
//! A type reaches the text from exactly five shapes, and the file's tests enumerate them rather than
//! assume them: a local's declaration (the defect here), the header of a proved `try`-with-resources
//! (the same `value_type`, so the same spelling, and no compiler emits a resource whose type is not
//! `AutoCloseable`), a lambda's parameters (spelled by `lambda::parse_type`, which already read
//! arrays), and the two positions that name a class instead of declaring one — a static member's
//! owner and a construction's class, which take the pool's own `CONSTANT_Class` name (and such a name
//! *may* be an array descriptor: JVMS 4.4.1 permits it). The committed sample pins the first, the
//! control test at the end pins the object-name spelling those class positions keep, and the
//! hand-built class pins what each of the others does: the declaration and the two class positions
//! with a descriptor that states no name (refused), and the two class positions with an array-typed
//! name (a recorded boundary — the type is spelled, the position admits no array type, and no legal
//! class file states it).

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 511 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-array-types/v8/ArrayTypes.class");

/// Every member of the sample that declares a body, so a renamed fixture fails here instead of
/// covering less than this file claims.
const DECLARED: [(&[u8], &[u8]); 7] = [
    (b"<init>", b"()V"),
    (b"copy", b"([B)[B"),
    (b"echoed", b"([B)[B"),
    (b"named", b"([Ljava/lang/String;)[Ljava/lang/String;"),
    (b"grid", b"([[I)[[I"),
    (b"table", b"([[Ljava/lang/String;)[[Ljava/lang/String;"),
    (b"text", b"(Ljava/lang/String;)Ljava/lang/String;"),
];

/// The four members whose declaration is the defect: the member, and the descriptor it is named by.
const ARRAY_MEMBERS: [(&[u8], &[u8]); 4] = [
    (b"echoed", b"([B)[B"),
    (b"named", b"([Ljava/lang/String;)[Ljava/lang/String;"),
    (b"grid", b"([[I)[[I"),
    (b"table", b"([[Ljava/lang/String;)[[Ljava/lang/String;"),
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

/// The body of one member the run presents whole: a body that is not `Java`/`Structured` is a
/// failure of the premise, not a boundary, so the planes are asserted before the text is read.
///
/// The artifact's `// @…` envelope legitimately states the member's own name and descriptor
/// (`// @method echoed([B)[B`): that is how the member is identified, and a descriptor in it is not
/// a type position. The body below the envelope is what a compiler reads as the member's text, and
/// what every assertion here is about.
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
    let Some(body) = report.text.split_once("{\n").map(|(_, body)| body) else {
        panic!(
            "`{}`: the artifact has a body:\n{}",
            report.method, report.text
        );
    };
    let Some(body) = body.strip_suffix("}\n") else {
        panic!(
            "`{}`: the artifact's body is closed:\n{}",
            report.method, report.text
        );
    };
    body.to_string()
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

/// The facts a refusal is: the planes that say the artifact does not claim Java for the region, the
/// quote that names the bytecode it could not present, the anchor that keeps that bytecode in the
/// segment table, the reason that states the bytecode index, and the member the answer belongs to.
/// "No Java text was produced" is not one of them — every assertion here is a boundary a caller can
/// locate the refusal with.
///
/// `bci` is the index the reason states (the instruction that could not be written) and `quoted` the
/// indexes the text's quotes must name: a refusal is written where the value was consumed, so the
/// two are the same index for most shapes and not for all (`unnamedNew`'s construction is refused at
/// BCI 4 and the value it produces is consumed at BCI 7).
fn assert_refused(report: &RecoveryReport, name: &str, bci: u32, quoted: &[u32], states: &[&str]) {
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
    let quoted_bcis = quoted_bcis(&report.text);
    for index in quoted {
        assert!(
            quoted_bcis.contains(index),
            "`{name}`: the quote names the BCI the region could not present ({index}):\n{}",
            report.text
        );
    }
    assert!(
        !quoted_bcis.is_empty(),
        "`{name}`: a refused region quotes the bytecode it could not present:\n{}",
        report.text
    );
    for index in &quoted_bcis {
        assert!(
            !report.text_of_bci(*index).is_empty(),
            "`{name}`: every quoted bytecode index stays anchored in the segment table ({index}):\n{}",
            report.text
        );
    }
    assert!(
        report.text.contains(&format!("BCI {bci}")),
        "`{name}`: the refusal states the bytecode index it could not type:\n{}",
        report.text
    );
    for state in states {
        assert!(
            report.text.contains(state),
            "`{name}`: the refusal states `{state}`:\n{}",
            report.text
        );
    }
    assert!(
        report.method.starts_with(name),
        "`{name}`: the report names the member it refused: {}",
        report.method
    );
}

// -------------------------------------------------------------------------------------------
// The committed sample: the four defect shapes and the two controls.
// -------------------------------------------------------------------------------------------

#[test]
fn a_primitive_array_declaration_is_a_java_byte_array() {
    // The review's first shape: `echoed([B)[B` is `byte[] local = copy(value); return local;`, and
    // the declaration was the frames' own `[B`, which `javac --release 8` refuses at the `[`
    // (`illegal start of expression`). The element name is the one the object-name rule states and
    // the dimensions are the descriptor's.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let body = whole_body(&engine, &fixture, b"echoed", b"([B)[B");
    assert_eq!(
        body, "    byte[] local1 = copy(arg0);\n    return local1;\n",
        "the declaration of a `[B` value is a `byte[]`"
    );
}

#[test]
fn a_reference_array_declaration_spells_its_element_like_any_other_class() {
    // `named([Ljava/lang/String;)[Ljava/lang/String;` is `String[] local = value; return local;`,
    // and the declaration was `[Ljava.lang.String;` — which javac refuses twice (the `[` is no
    // expression, and the `.` after the name is no statement).
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let body = whole_body(
        &engine,
        &fixture,
        b"named",
        b"([Ljava/lang/String;)[Ljava/lang/String;",
    );
    assert_eq!(
        body, "    java.lang.String[] local1 = arg0;\n    return local1;\n",
        "the element of a reference array keeps the object-name spelling (`L…;` off, `/` → `.`)"
    );
}

#[test]
fn a_multi_dimensional_array_declaration_states_every_dimension() {
    // `grid([[I)[[I` and `table([[Ljava/lang/String;)[[Ljava/lang/String;` are the two-dimensional
    // shapes: one `[]` per dimension, after the element's own spelling.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);

    let body = whole_body(&engine, &fixture, b"grid", b"([[I)[[I");
    assert_eq!(
        body, "    int[][] local1 = arg0;\n    return local1;\n",
        "a `[[I` value is declared `int[][]`"
    );

    let body = whole_body(
        &engine,
        &fixture,
        b"table",
        b"([[Ljava/lang/String;)[[Ljava/lang/String;",
    );
    assert_eq!(
        body, "    java.lang.String[][] local1 = arg0;\n    return local1;\n",
        "and a `[[Ljava/lang/String;` value is declared `java.lang.String[][]`"
    );
}

#[test]
fn no_type_position_publishes_a_descriptor() {
    // The acceptance the four members share: in the body — the text a compiler reads — a `[` can
    // only be the opening bracket of a Java array type (`[]`). Any other `[` is the descriptor
    // syntax leaking into a type position, which is what the review found (`[B local1 = arg0;`) and
    // what `javac` refuses.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    for (name, descriptor) in ARRAY_MEMBERS {
        let body = whole_body(&engine, &fixture, name, descriptor);
        let leaked: Vec<&str> = body
            .match_indices('[')
            .filter(|(at, _)| body.as_bytes().get(at + 1) != Some(&b']'))
            .map(|(at, _)| body[at..].lines().next().unwrap_or("").trim())
            .collect();
        assert!(
            leaked.is_empty(),
            "`{}`: a `[` that opens no `[]` is a descriptor in a type position:\n{body}",
            String::from_utf8_lossy(name)
        );
        for descriptor_form in ["[B", "[C", "[D", "[F", "[I", "[J", "[S", "[Z", "[L", "[[I"] {
            assert!(
                !body.contains(descriptor_form),
                "`{}`: the body states no `{descriptor_form}` descriptor:\n{body}",
                String::from_utf8_lossy(name)
            );
        }
    }
}

#[test]
fn an_object_type_and_a_primitive_keep_the_spelling_they_had() {
    // The direction that must not move: the fix reads an array descriptor through the same
    // object-name rule the `L…;` form has always used, so an object type's declaration is
    // *byte-for-byte* what it was before (`java.lang.String local1 = arg0;`, the pre-fix text of
    // this member), and a body that declares nothing is untouched.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);

    let body = whole_body(
        &engine,
        &fixture,
        b"text",
        b"(Ljava/lang/String;)Ljava/lang/String;",
    );
    assert_eq!(
        body, "    java.lang.String local1 = arg0;\n    return local1;\n",
        "an object type keeps the declaration it had"
    );

    let body = whole_body(&engine, &fixture, b"copy", b"([B)[B");
    assert_eq!(
        body, "    return arg0;\n",
        "the `[B` helper returns its parameter — no type position, no spelling"
    );
}

// -------------------------------------------------------------------------------------------
// The shapes no compiler states: the same entry point at the positions that cannot hold an array
// type at all, and the descriptors that have no Java spelling to publish anywhere.
// -------------------------------------------------------------------------------------------

/// The six members of the hand-built class `HandBuilt`, with the bytecode of each:
///
/// ```text
/// unnamed(L;)Ljava/lang/Object;        0: aload_0; 1: astore_1; 2: aload_1; 3: areturn
/// elementless()Ljava/lang/Object;      0: getstatic #13; 3: astore_0; 4: aload_0; 5: areturn
/// owner()Ljava/lang/Object;            0: getstatic #20 (Field [I.value:…); 3: areturn
/// allocated()Ljava/lang/Object;        0: new #16; 1: dup; 2: invokespecial #24; 5: areturn
/// unnamedOwner()Ljava/lang/Object;     0: getstatic #30 (Field [.value:…); 3: areturn
/// unnamedNew()Ljava/lang/Object;       0: new #27; 1: dup; 2: invokespecial #31; 5: areturn
/// ```
///
/// The first two state a reference type with **no name to spell**: `unnamed` takes a parameter whose
/// descriptor is the object form with no class name (`L;`) and copies it into a local, and
/// `elementless` reads a `static` field whose pool descriptor is `[L;` — an array whose element has
/// no name. `javac` cannot write either method (the source would have to name a type neither
/// descriptor has), and the frame pass accepts both (it reads the `L…;` form and an array of it), so
/// they are the smallest inputs the spelling entry point has no Java type for.
///
/// The other four carry an **array-typed class name** into the two positions that name a class
/// rather than declare one: a static field read's owner (`getstatic`'s `Fieldref` class, `[I` and
/// `[`) and a construction's class (an `invokespecial <init>`'s `Methodref` class, `[I` and `[`).
/// A `CONSTANT_Class` may hold an array descriptor (JVMS 4.4.1), and the frame pass reads the name
/// it holds; what no legal class file states is either shape *reachable as Java* — JVMS 4.4.2
/// requires a `Fieldref`'s class to be a class or interface type that has the field, and an array
/// type declares no `<init>` — so a source can never name them and javac can never produce these
/// bytes. They are here because the reachability question the change's position list asks is about
/// *this layer's* answer, not about what a compiler emits.
///
/// Hand-built bytes are the repository's fixture kind for exactly this
/// (`tests/p3_boolean_contexts.rs` states the same for its own shapes).
struct HandMember {
    name: &'static str,
    descriptor_index: u16,
    code: &'static [u8],
    max_stack: u16,
    max_locals: u16,
}

const HAND_MEMBERS: &[HandMember] = &[
    HandMember {
        name: "unnamed",
        descriptor_index: 7,
        code: &[0x2a, 0x4c, 0x2b, 0xb0],
        max_stack: 1,
        max_locals: 2,
    },
    HandMember {
        name: "elementless",
        descriptor_index: 9,
        code: &[0xb2, 0x00, 0x0d, 0x4b, 0x2a, 0xb0],
        max_stack: 1,
        max_locals: 1,
    },
    HandMember {
        name: "owner",
        descriptor_index: 9,
        code: &[0xb2, 0x00, 0x14, 0xb0],
        max_stack: 1,
        max_locals: 0,
    },
    HandMember {
        name: "allocated",
        descriptor_index: 9,
        code: &[0xbb, 0x00, 0x10, 0x59, 0xb7, 0x00, 0x18, 0xb0],
        max_stack: 2,
        max_locals: 0,
    },
    HandMember {
        name: "unnamedOwner",
        descriptor_index: 9,
        code: &[0xb2, 0x00, 0x1e, 0xb0],
        max_stack: 1,
        max_locals: 0,
    },
    HandMember {
        name: "unnamedNew",
        descriptor_index: 9,
        code: &[0xbb, 0x00, 0x1b, 0x59, 0xb7, 0x00, 0x1f, 0xb0],
        max_stack: 2,
        max_locals: 0,
    },
];

/// The hand-built class `HandBuilt`: one `Code` attribute per member of [`HAND_MEMBERS`], with the
/// constant pool its own instructions name:
///
/// ```text
/// 1  "HandBuilt"                     16 Class [I              26 "["
/// 2  Class HandBuilt                 17 "value"               27 Class [
/// 3  "java/lang/Object"              18 "Ljava/lang/Object;"   28 "unnamedOwner"
/// 4  Class java/lang/Object          19 NameAndType 17:18      29 "unnamedNew"
/// 5  "Code"                          20 Fieldref [I.value     30 Fieldref [.value
/// 6  "unnamed"                       21 "<init>"              31 Methodref [.<init>
/// 7  "(L;)Ljava/lang/Object;"        22 "()V"
/// 8  "elementless"                   23 NameAndType 21:22
/// 9  "()Ljava/lang/Object;"          24 Methodref [I.<init>
/// 10 "field"                         25 "allocated"
/// 11 "[L;"
/// 12 NameAndType 10:11
/// 13 Fieldref HandBuilt.field
/// 14 "owner"
/// 15 "[I"
/// ```
fn hand_class() -> Vec<u8> {
    let mut output = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0); // minor_version
    u16b(&mut output, 52); // major_version: Java 8, the profile every request declares
    u16b(&mut output, 32); // constant_pool_count = 31 entries + 1
    utf8(&mut output, b"HandBuilt"); // 1
    output.push(7);
    u16b(&mut output, 1); // 2: Class HandBuilt
    utf8(&mut output, b"java/lang/Object"); // 3
    output.push(7);
    u16b(&mut output, 3); // 4: Class java/lang/Object
    utf8(&mut output, b"Code"); // 5
    utf8(&mut output, b"unnamed"); // 6
    utf8(&mut output, b"(L;)Ljava/lang/Object;"); // 7
    utf8(&mut output, b"elementless"); // 8
    utf8(&mut output, b"()Ljava/lang/Object;"); // 9
    utf8(&mut output, b"field"); // 10 -- the field's name
    utf8(&mut output, b"[L;"); // 11 -- its descriptor: an array whose element has no name
    output.push(12);
    u16b(&mut output, 10);
    u16b(&mut output, 11); // 12: NameAndType field:[L;
    output.push(9);
    u16b(&mut output, 2);
    u16b(&mut output, 12); // 13: Fieldref HandBuilt.field:[L;
    utf8(&mut output, b"owner"); // 14
    utf8(&mut output, b"[I"); // 15
    output.push(7);
    u16b(&mut output, 15); // 16: Class [I
    utf8(&mut output, b"value"); // 17
    utf8(&mut output, b"Ljava/lang/Object;"); // 18
    output.push(12);
    u16b(&mut output, 17);
    u16b(&mut output, 18); // 19: NameAndType value:Ljava/lang/Object;
    output.push(9);
    u16b(&mut output, 16);
    u16b(&mut output, 19); // 20: Fieldref [I.value:Ljava/lang/Object;
    utf8(&mut output, b"<init>"); // 21
    utf8(&mut output, b"()V"); // 22
    output.push(12);
    u16b(&mut output, 21);
    u16b(&mut output, 22); // 23: NameAndType <init>:()V
    output.push(10);
    u16b(&mut output, 16);
    u16b(&mut output, 23); // 24: Methodref [I.<init>:()V
    utf8(&mut output, b"allocated"); // 25
    utf8(&mut output, b"["); // 26
    output.push(7);
    u16b(&mut output, 26); // 27: Class [
    utf8(&mut output, b"unnamedOwner"); // 28
    utf8(&mut output, b"unnamedNew"); // 29
    output.push(9);
    u16b(&mut output, 27);
    u16b(&mut output, 19); // 30: Fieldref [.value:Ljava/lang/Object;
    output.push(10);
    u16b(&mut output, 27);
    u16b(&mut output, 23); // 31: Methodref [.<init>:()V
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
            "unnamed" => 6,
            "elementless" => 8,
            "owner" => 14,
            "allocated" => 25,
            "unnamedOwner" => 28,
            "unnamedNew" => 29,
            other => panic!("no pool entry for `{other}`"),
        };
        u16b(&mut output, 0x0009); // public static
        u16b(&mut output, name_index);
        u16b(&mut output, member.descriptor_index);
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

#[test]
fn a_descriptor_that_states_no_java_type_is_refused_at_its_bci() {
    // `unnamed`'s local is filled from a `L;` value: an object descriptor with no class name, so
    // there is no type to spell (`L;` is not a Java type) and the declaration is refused at the
    // instruction that would have carried it, instead of publishing `L; local1 = arg0;`.
    let engine = Engine::new();
    let bytes = hand_class();
    let fixture = fixture(&engine, &bytes);

    let report = recover(&engine, &fixture, b"unnamed", b"(L;)Ljava/lang/Object;");
    assert_refused(
        &report,
        "unnamed",
        1,
        &[1],
        &[
            "`L;`",
            "cannot spell as a Java type",
            "the declaration is refused instead of writing it",
        ],
    );
    assert!(
        !report.text.contains("L; local1"),
        "the descriptor may not be published as the declaration's type:\n{}",
        report.text
    );

    // `elementless` reads a `static` field whose descriptor is `[L;`: the array form whose element
    // has no name. The refusal is the store the declaration would have carried — the array spelling
    // is what could not be produced, not the expression — and the array descriptor is what the
    // refusal names.
    let report = recover(&engine, &fixture, b"elementless", b"()Ljava/lang/Object;");
    assert_refused(
        &report,
        "elementless",
        3,
        &[3],
        &[
            "`[L;`",
            "cannot spell as a Java type",
            "the declaration is refused instead of writing it",
        ],
    );
    // A refused declaration is not followed by the assignment it would have carried
    // (`unify-local-type-decisions`: the two outcomes of `declare()` are distinct, and after
    // "refused" nothing is written): the store at BCI 3 is quoted instead, and the read the
    // declaration could not type is therefore not published either. Before that change the same
    // refusal was followed by `local0 = HandBuilt.field;` — the text this assertion now forbids.
    assert!(
        !report.text.contains("local0 = "),
        "a refused declaration is not followed by its assignment:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("HandBuilt.field"),
        "and an assignment that is not written publishes no expression:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("[L; local"),
        "the array descriptor may not be published as the declaration's type:\n{}",
        report.text
    );
}

#[test]
fn a_class_position_an_array_type_cannot_occupy_is_recorded_as_a_boundary() {
    // The other two positions that name a class: a static field read's owner and a construction's
    // class. An array-typed `CONSTANT_Class` reaches both, and the name is spelled by the array rule
    // like every other type — `int[].value` and `new int[]()` — but a Java *expression* cannot put
    // an array type in either place, and javac refuses both texts whatever they spell:
    //
    // ```text
    // GenOwner.java:3: error: class expected
    //     return int[].value;
    //                  ^
    // GenAllocated.java:3: error: array dimension missing
    //     return new int[]();
    //                     ^
    // ```
    //
    // This is recorded as a boundary and not as an acceptance: no legal class file states either
    // shape (JVMS 4.4.2 requires a `Fieldref`'s class to be a class or interface type that has the
    // field, and an array type declares no `<init>`), and the pre-fix text (`[I.value`, `new [I()`)
    // was refused by javac in exactly the same way, so what changed here is the *spelling* — the
    // type name is now a legal Java type and no descriptor is published — while the limit is the
    // position's and is pre-existing. What the test pins is the spelling (and, with it, that an
    // array descriptor never reaches the text), and the fixture's README records the position.
    let engine = Engine::new();
    let bytes = hand_class();
    let fixture = fixture(&engine, &bytes);

    let report = recover(&engine, &fixture, b"owner", b"()Ljava/lang/Object;");
    assert!(
        report.text.contains("return int[].value;"),
        "the owner of a static read is spelled by the array rule:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("[I.value") && !report.text.contains("[I."),
        "and the descriptor is not published:\n{}",
        report.text
    );

    let report = recover(&engine, &fixture, b"allocated", b"()Ljava/lang/Object;");
    assert!(
        report.text.contains("new int[]("),
        "a construction's class is spelled by the array rule:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("new [I("),
        "and the descriptor is not published:\n{}",
        report.text
    );
}

#[test]
fn an_owner_or_construction_class_with_no_java_spelling_is_refused() {
    // The same two positions with a class name that has no spelling at all (`[`): the owner of a
    // static read and a construction's class each refuse at their own instruction, with the fact
    // named, instead of writing the name into the text. Both are the positions' own refusals — the
    // value (a claimed static read, a construction) is not what failed.
    let engine = Engine::new();
    let bytes = hand_class();
    let fixture = fixture(&engine, &bytes);

    let report = recover(&engine, &fixture, b"unnamedOwner", b"()Ljava/lang/Object;");
    assert_refused(
        &report,
        "unnamedOwner",
        0,
        &[0],
        &["names the owner `[`", "cannot spell as a Java type"],
    );
    assert!(
        !report.text.contains("[.value") && !report.text.contains("return ["),
        "the owner's descriptor may not be published as a receiver:\n{}",
        report.text
    );

    let report = recover(&engine, &fixture, b"unnamedNew", b"()Ljava/lang/Object;");
    assert_refused(
        &report,
        "unnamedNew",
        4,
        &[7],
        &["names the class `[`", "cannot spell as a Java type"],
    );
    assert!(
        !report.text.contains("new [(") && !report.text.contains("return new ["),
        "the construction's class may not be published:\n{}",
        report.text
    );
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
