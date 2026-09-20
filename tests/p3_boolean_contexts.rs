//! P3 acceptance: a value in a **boolean context** is presented as a boolean, and a value the layer
//! cannot prove boolean there is refused instead of published in an `int` spelling.
//!
//! Everything here goes through the entry point the CLI calls ([`Engine::recover_method`]) over the
//! committed javac 23.0.1 sample `tests/fixtures/p3-boolean-contexts/` (whose README states the
//! command, the digest and the bytecode of every member), plus one hand-built class for the two
//! shapes no compiler emits.
//!
//! # Why the planes cannot answer this question
//!
//! The two defects this file pins were reported as `Java`/`Structured`/`contains_statements`/
//! `complete` with no diagnostic: the artifact was structurally whole and its *values* were typed
//! wrongly. `isZero(I)Z`'s body was written `return 1;`/`return 0;` inside a `boolean` method and
//! `parity(I)I`'s condition was written `if (flag() != 0)` over a `boolean` call, and `javac
//! --release 8` refuses both texts under the member's own declaration (`int cannot be converted to
//! boolean`, `incomparable types: boolean and int`). A plane that counts statements cannot see a
//! type, so the evidence has to be the text and the compiler — which is what the ignored
//! `tests/p3_execution_comparison.rs` row for this sample does with the same bytes.
//!
//! # The fact that decides, and the evidence that proves
//!
//! The frames state one slot shape for the four int-sized primitives, so the *descriptor* is the
//! only fact that says a position holds a `boolean` ([`MethodFacts::parameter_types`] for the
//! parameters, the member's own return descriptor for the return position, the callee's descriptor
//! for a call's result and the pool's for a claimed field read). The cases below pin each one, and
//! pin the two directions a fix must not take: an `int` method still writes `return 1;`/`!= 0`, and
//! a proof the layer does not have is a refusal rather than a guess.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 944 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-boolean-contexts/v8/BooleanContexts.class");

/// Every member of the sample that declares a body, so a renamed fixture fails here instead of
/// covering less than this file claims.
const DECLARED: [(&[u8], &[u8]); 18] = [
    (b"<init>", b"()V"),
    (b"isZero", b"(I)Z"),
    (b"flag", b"()Z"),
    (b"parity", b"(I)I"),
    (b"passed", b"(Z)Z"),
    (b"callFlag", b"()Z"),
    (b"fieldFlag", b"()Z"),
    (b"localFromCall", b"()Z"),
    (b"pick", b"(IZZ)Z"),
    (b"fromLocal", b"(Z)Z"),
    (b"assignFromCall", b"(Z)Z"),
    (b"throughLocal", b"(Z)Z"),
    (b"negated", b"(Z)Z"),
    (b"staticFlagCount", b"()I"),
    (b"intLocal", b"(I)I"),
    (b"count", b"(Z)I"),
    (b"nonzero", b"(I)I"),
    (b"answer", b"()I"),
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
        .recover_method(
            slice::from_ref(&fixture.snapshot),
            &request,
            &mut Budget::new(limits()),
        )
        .expect("a legal request is answered, not raised")
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

/// The facts a refusal is: the planes that say the artifact does not claim Java for the region, the
/// quote that names the bytecode index it could not present, the anchor that keeps that bytecode in
/// the segment table, the reason that states the index, and the member the answer belongs to. "No
/// Java text was produced" is not one of them — every assertion here is a boundary a caller can
/// locate the refusal with.
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
// The committed sample: the two defect shapes and the evidence that must survive the fix.
// -------------------------------------------------------------------------------------------

#[test]
fn a_boolean_return_is_written_as_a_boolean() {
    // The review's first shape: `isZero(I)Z` is `if (x == 0) { return true; } return false;` and
    // its two `ireturn`s were written `return 1;`/`return 0;` — text javac refuses under the
    // member's own declaration (`incompatible types: int cannot be converted to boolean`).
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let text = whole_body(&engine, &fixture, b"isZero", b"(I)Z");
    assert!(text.contains("if (arg0 == 0) {"), "{text}");
    assert!(
        text.contains("return true;") && text.contains("return false;"),
        "the two returns of a `Z` method are booleans, typed by the member's own descriptor:\n{text}"
    );
    assert!(
        !text.contains("return 1;") && !text.contains("return 0;"),
        "the `int` spelling of a boolean return is what javac refused:\n{text}"
    );

    // The same shape with one return: `flag()Z` is `return true;`, and it was `return 1;`.
    let text = whole_body(&engine, &fixture, b"flag", b"()Z");
    assert!(text.contains("return true;"), "{text}");
    assert!(!text.contains("return 1;"), "{text}");
}

#[test]
fn a_boolean_call_result_is_a_truth_test_not_an_int_comparison() {
    // The review's second shape: `parity(I)I` is `if (flag()) { return 1; } return 0;` and the
    // condition was written `if (flag() != 0)`, which javac refuses (`incomparable types: boolean
    // and int`). The evidence is the callee's own descriptor (`flag:()Z`), not the shape of the
    // call and not a guess from the callee's name.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let text = whole_body(&engine, &fixture, b"parity", b"(I)I");
    assert!(text.contains("if (flag()) {"), "{text}");
    assert!(
        !text.contains("flag() != 0") && !text.contains("flag() == 0"),
        "a `boolean` result is not comparable with `0`:\n{text}"
    );
}

#[test]
fn the_evidence_the_layer_states_keeps_its_presentation() {
    // Each evidence item in the return position and in the condition position: a value the layer
    // *can* prove boolean must keep being presented, not be refused for want of a proof it has.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);

    // A `boolean` parameter's load returned: `passed(Z)Z` is `return b;`.
    let text = whole_body(&engine, &fixture, b"passed", b"(Z)Z");
    assert!(text.contains("return arg0;"), "{text}");

    // A call whose callee descriptor returns `Z` returned: `callFlag()Z` is `return flag();`.
    let text = whole_body(&engine, &fixture, b"callFlag", b"()Z");
    assert!(text.contains("return flag();"), "{text}");

    // A claimed field read whose pool descriptor is `Z` returned: `fieldFlag()Z` is
    // `return staticFlag;`. Without that fact this shape would be refused although the class states
    // the type of the member outright, and the text it has today already compiles.
    let text = whole_body(&engine, &fixture, b"fieldFlag", b"()Z");
    assert!(
        text.contains("return BooleanContexts.staticFlag;"),
        "{text}"
    );

    // The same fact as a condition: `if (staticFlag != 0)` is refused by javac
    // (`incomparable types: boolean and int`), so the condition of a proven `boolean` field is a
    // truth test too.
    let text = whole_body(&engine, &fixture, b"staticFlagCount", b"()I");
    assert!(text.contains("if (BooleanContexts.staticFlag) {"), "{text}");
    assert!(!text.contains("!= 0"), "{text}");
}

#[test]
fn a_local_filled_from_proven_boolean_evidence_is_declared_boolean() {
    // The third site of the same family. `Builder::declare` read the value the **store** wrote,
    // whose SSA definition is the `Store` instruction itself — never the `load` its evidence
    // predicate requires — so no local was ever declared `boolean` and `boolean c = b;` came out
    // `int local1 = arg0;`. `throughLocal(Z)Z` is that shape at its smallest: `iload_0; istore_1;
    // iload_1; ireturn`, whose pre-extension text javac refuses twice over
    // (`incompatible types: boolean cannot be converted to int` on the declaration, `int cannot be
    // converted to boolean` on the return).
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);
    let text = whole_body(&engine, &fixture, b"throughLocal", b"(Z)Z");
    assert!(
        text.contains("boolean local1 = arg0;"),
        "a local filled from a `Z` parameter's load is declared `boolean`:\n{text}"
    );
    assert!(
        text.contains("return local1;"),
        "and the local's own declaration is what proves its later use boolean:\n{text}"
    );
    assert!(
        !text.contains("int local1") && !text.contains("return local1 != 0"),
        "neither the declaration nor the use keeps the `int` spelling:\n{text}"
    );

    // The same declaration from a call whose callee descriptor returns `Z`: `boolean c = flag();`.
    let text = whole_body(&engine, &fixture, b"localFromCall", b"()Z");
    assert!(text.contains("boolean local0 = flag();"), "{text}");
    assert!(text.contains("return local0;"), "{text}");

    // A variable declared above the region that fills it states its type there (the hoisted
    // declaration path in `declarations()` reads the first write's **stored** value), and a value
    // merged out of the two arms is read through that declaration.
    let text = whole_body(&engine, &fixture, b"pick", b"(IZZ)Z");
    assert!(text.contains("boolean local3;"), "{text}");
    assert!(
        text.contains("local3 = arg1;") && text.contains("local3 = arg2;"),
        "{text}"
    );
    assert!(text.contains("return local3;"), "{text}");
    assert!(!text.contains("int local3"), "{text}");

    // A local copied from a local the body already declared boolean: `boolean d = c;`.
    let text = whole_body(&engine, &fixture, b"fromLocal", b"(Z)Z");
    assert!(
        text.contains("boolean local1 = arg0;") && text.contains("boolean local2 = local1;"),
        "{text}"
    );
    assert!(text.contains("return local2;"), "{text}");

    // A store into a variable whose type the *signature* states: `b = flag();` on a `Z` parameter.
    let text = whole_body(&engine, &fixture, b"assignFromCall", b"(Z)Z");
    assert!(text.contains("arg0 = flag();"), "{text}");
    assert!(text.contains("return arg0;"), "{text}");
}

#[test]
fn an_int_context_keeps_its_int_shape() {
    // The direction that must not move: an `int` method returns `int`, a genuine `int` comparison
    // stays a comparison, a local filled with `0`/`1` stays `int`, and the P3-R5 parameter path keeps
    // the text it already had. Refusing or re-typing these would trade the shapes above for a layer
    // that writes another program.
    let engine = Engine::new();
    let fixture = fixture(&engine, SAMPLE);

    let text = whole_body(&engine, &fixture, b"answer", b"()I");
    assert!(text.contains("return 1;"), "{text}");
    assert!(!text.contains("return true;"), "{text}");

    let text = whole_body(&engine, &fixture, b"nonzero", b"(I)I");
    assert!(text.contains("if (arg0 != 0) {"), "{text}");

    let text = whole_body(&engine, &fixture, b"count", b"(Z)I");
    assert!(text.contains("if (arg0) {"), "{text}");
    assert!(!text.contains("!= 0"), "{text}");

    // The literal control: `int x = 0; … x = 1;` is the same bytecode shape as `boolean c = true;`,
    // and a local's declaration reads no literal as a boolean — the member returns `int`, so the
    // declarations and the uses must stay `int`.
    let text = whole_body(&engine, &fixture, b"intLocal", b"(I)I");
    assert!(text.contains("int local1;"), "{text}");
    assert!(
        text.contains("local1 = 0;") && text.contains("local1 = 1;"),
        "{text}"
    );
    assert!(text.contains("local1 = arg0;"), "{text}");
    assert!(text.contains("return local1;"), "{text}");
    assert!(
        !text.contains("boolean") && !text.contains("true") && !text.contains("false"),
        "the `0`/`1` stores of an `int` local are not booleans:\n{text}"
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

// -------------------------------------------------------------------------------------------
// The integer binary comparison: the position decides, and no operand is read as a boolean.
// -------------------------------------------------------------------------------------------

/// The committed sample of the comparison-context regression: javac 23.0.1, `--release 8 -g:none`
/// (see `tests/fixtures/p3-int-comparisons/README.md` for the command, the size and the digest).
const INT_COMPARISONS: &[u8] =
    include_bytes!("fixtures/p3-int-comparisons/v8/IntComparisons.class");

/// Every member of [`INT_COMPARISONS`] that declares a body, so a renamed fixture fails here
/// instead of covering less than this file claims.
const INT_COMPARISONS_DECLARED: [(&[u8], &[u8]); 10] = [
    (b"<init>", b"()V"),
    (b"oneFirst", b"(I)I"),
    (b"zeroFirst", b"(I)I"),
    (b"oneLess", b"(I)I"),
    (b"oneLast", b"(I)I"),
    (b"zeroLast", b"(I)I"),
    (b"nonzero", b"(I)I"),
    (b"isZero", b"(I)Z"),
    (b"count", b"(Z)I"),
    (b"throughLocal", b"(Z)Z"),
];

/// The five comparison shapes and the variable-operand control, each with the condition its own
/// evidence spells. `1 == n`, `0 < n` and `1 < n` are the regression `5a8c36a` introduced; `n == 1`
/// and `n > 0` are the same rule on the right-hand side and in the other comparison direction.
const COMPARISONS: [(&[u8], &str, &str); 6] = [
    (b"oneFirst", "if (1 == arg0) {", "true == arg0"),
    (b"zeroFirst", "if (0 < arg0) {", "false < arg0"),
    (b"oneLess", "if (1 < arg0) {", "true < arg0"),
    (b"oneLast", "if (arg0 == 1) {", "arg0 == true"),
    (b"zeroLast", "if (arg0 > 0) {", "arg0 > false"),
    (b"nonzero", "if (arg0 != 0) {", "arg0 != false"),
];

#[test]
fn an_integer_comparison_keeps_its_integer_literals_on_either_side() {
    // The regression this fix corrects, and the reason it is one: `if_icmp*` reads two `int`s, and
    // the fact that its **result** is a boolean says nothing about them. `condition` asked
    // `boolean_value` — whose evidence includes the `0`/`1` literal — before it looked at the test's
    // shape, so `iconst_1; iload_0; if_icmpne` was spelled `if (true == arg0)`, text `javac
    // --release 8` refuses (`incomparable types: boolean and int`) under the member's own
    // declaration. `5a8c36a` introduced it; before that (`fa6dc6e`) the same bytes were `1 == arg0`.
    // The position decides before any operand is spelled, and both sides keep their own evidence.
    let engine = Engine::new();
    let fixture = fixture(&engine, INT_COMPARISONS);
    for (name, expected, refused) in COMPARISONS {
        let text = whole_body(&engine, &fixture, name, b"(I)I");
        assert!(
            text.contains(expected),
            "`{}` is an integer binary comparison, so its operands keep their integer spelling:\n{text}",
            String::from_utf8_lossy(name)
        );
        assert!(
            !text.contains(refused) && !text.contains("true") && !text.contains("false"),
            "the boolean spelling javac refuses may not appear in `{}`:\n{text}",
            String::from_utf8_lossy(name)
        );
    }
}

#[test]
fn the_boolean_contexts_of_the_comparison_sample_keep_their_presentation() {
    // The controls the same bytes carry, so that the fix cannot be "spell every literal as an
    // integer": a `Z` return of a `0`/`1` literal is still `true`/`false`, a proven boolean
    // parameter's zero test is still a truth test, and a local a write declared `boolean` still
    // types its later uses. `p3-boolean-contexts/` is the second copy of this control.
    let engine = Engine::new();
    let fixture = fixture(&engine, INT_COMPARISONS);

    let text = whole_body(&engine, &fixture, b"isZero", b"(I)Z");
    assert!(
        text.contains("if (arg0 == 0) {")
            && text.contains("return true;")
            && text.contains("return false;"),
        "a `Z` method's `return` of a literal is a boolean, and its `int` operand is not:\n{text}"
    );

    let text = whole_body(&engine, &fixture, b"count", b"(Z)I");
    assert!(
        text.contains("if (arg0) {") && !text.contains("!= 0"),
        "a zero test on a proven boolean parameter is a truth test:\n{text}"
    );

    let text = whole_body(&engine, &fixture, b"throughLocal", b"(Z)Z");
    assert!(
        text.contains("boolean local1 = arg0;") && text.contains("return local1;"),
        "a local filled from a `Z` parameter is declared `boolean`:\n{text}"
    );
}

#[test]
fn every_declared_member_of_the_comparison_sample_is_covered() {
    // The premise: the sample declares every member this file classifies, and each of them is
    // answered (presented or refused) rather than missed.
    let engine = Engine::new();
    let fixture = fixture(&engine, INT_COMPARISONS);
    for (name, descriptor) in INT_COMPARISONS_DECLARED {
        let report = recover(&engine, &fixture, name, descriptor);
        let spelled = format!(
            "{}{}",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(descriptor)
        );
        assert!(
            report.produced() && !report.text.is_empty(),
            "`{spelled}`: {:?}\n{}",
            report.outcome,
            report.text
        );
    }
}

/// One hand-built class for the boundary the predecessor change recorded and handed on: a
/// `Test::Pair` comparison whose left operand is proven boolean (a call whose callee descriptor
/// returns `Z`) and whose right operand is the `int` literal `1`. The two bodies are
/// [`BOUNDARY_MEMBERS`].
///
/// `javac` folds `flag() == true` into `flag()`, so no compiler emits this shape, and the rule that
/// fixed a pair comparison's operand *spelling* (each operand keeps its own evidence) wrote it
/// `flag() == 1`, which `javac --release 8` refuses (`incomparable types: boolean and int`). That
/// change recorded the shape as a boundary and stated that its disposition — refuse it or keep it —
/// belonged to the next change's conflicting-types rule; `unify-local-type-decisions` took the
/// decision and refuses it: a comparison whose two spellings cannot stand in one comparison has no
/// Java text to publish, so the structure terminates with its bytecode quoted instead of claiming
/// Java for text the compiler rejects. The literal's *spelling* is not what decides this — a `0`/`1`
/// literal is an `int` until a position requires a boolean, which is why `1 == arg0` and `arg0 > 0`
/// keep their text.
fn boundary_class() -> Vec<u8> {
    let mut output = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0); // minor_version
    u16b(&mut output, 52); // major_version: Java 8, the profile every request declares
    u16b(&mut output, 12); // constant_pool_count = 11 entries + 1
    utf8(&mut output, b"Boundary"); // 1
    output.push(7);
    u16b(&mut output, 1); // 2: Class Boundary
    utf8(&mut output, b"java/lang/Object"); // 3
    output.push(7);
    u16b(&mut output, 3); // 4: Class java/lang/Object
    utf8(&mut output, b"Code"); // 5
    utf8(&mut output, b"flag"); // 6
    utf8(&mut output, b"()Z"); // 7
    utf8(&mut output, b"probe"); // 8
    utf8(&mut output, b"()I"); // 9
    output.push(12);
    u16b(&mut output, 6);
    u16b(&mut output, 7); // 10: NameAndType flag:()Z
    output.push(10);
    u16b(&mut output, 2);
    u16b(&mut output, 10); // 11: Methodref Boundary.flag:()Z

    u16b(&mut output, 0x21); // access_flags: public super
    u16b(&mut output, 2); // this_class
    u16b(&mut output, 4); // super_class
    u16b(&mut output, 0); // interfaces
    u16b(&mut output, 0); // fields
    u16b(
        &mut output,
        u16::try_from(BOUNDARY_MEMBERS.len()).expect("member count fits u16"),
    );
    for member in BOUNDARY_MEMBERS {
        let name_index = match member.name {
            "flag" => 6,
            "probe" => 8,
            other => panic!("no pool entry for `{other}`"),
        };
        let descriptor_index = match member.descriptor {
            "()Z" => 7,
            "()I" => 9,
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

/// The two members of the hand-built [`boundary_class`], with the bytecode of each:
///
/// ```text
/// flag()Z    0: iconst_1; 1: ireturn          (`04 ac`)
/// probe()I   0: invokestatic flag:()Z; 3: iconst_1; 4: if_icmpne 9; 7: iconst_1; 8: ireturn;
///            9: iconst_0; 10: ireturn        (`b8 00 0b 04 a0 00 05 04 ac 03 ac`)
/// ```
const BOUNDARY_MEMBERS: &[HandMember] = &[
    HandMember {
        name: "flag",
        descriptor: "()Z",
        code: &[0x04, 0xac],
        max_stack: 1,
        max_locals: 0,
    },
    HandMember {
        name: "probe",
        descriptor: "()I",
        code: &[
            0xb8, 0x00, 0x0b, 0x04, 0xa0, 0x00, 0x05, 0x04, 0xac, 0x03, 0xac,
        ],
        max_stack: 2,
        max_locals: 0,
    },
];

#[test]
fn a_proven_boolean_operand_beside_an_int_literal_is_refused() {
    // The disposition the predecessor change handed to `unify-local-type-decisions`: a `Test::Pair`
    // comparison that mixes a proven boolean (the `Z` call `flag()`) with an operand it does not
    // prove boolean (the `int` literal `1`) is refused. The predecessor decision about *spelling*
    // stands — the literal is not re-spelled as a boolean to make the text look boolean-typed — and
    // the operand each keeps its own evidence; what this rule adds is that the two spellings cannot
    // stand in one comparison, which javac states as `incomparable types: boolean and int`. The
    // structure therefore terminates with its bytecode quoted (the region is `if_icmpne` at BCI 4,
    // whose block the quote names) instead of publishing text the compiler rejects while the report
    // claims Java.
    let engine = Engine::new();
    let bytes = boundary_class();
    let fixture = fixture(&engine, &bytes);
    let report = recover(&engine, &fixture, b"probe", b"()I");
    assert_refused(&report, "probe", 4);
    assert!(
        report.text.contains("incomparable types: boolean and int"),
        "the refusal states the compiler's own verdict about the pair of operands:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("flag() == 1") && !report.text.contains("flag() != 1"),
        "the comparison no Java spelling accepts may not be published:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("flag() == true") && !report.text.contains("true == flag()"),
        "and the literal may not be spelled as a boolean to make the text look boolean-typed:\n{}",
        report.text
    );
}

// -------------------------------------------------------------------------------------------
// The shapes no compiler emits: a proof the layer does not have is a refusal, never a guess.
// -------------------------------------------------------------------------------------------

/// One member of the hand-built class: its name, descriptor, `Code` bytes and slot counts.
struct HandMember {
    name: &'static str,
    descriptor: &'static str,
    code: &'static [u8],
    max_stack: u16,
    max_locals: u16,
}

/// The bodies `javac` cannot be asked for, with the bytecode of each:
///
/// ```text
/// intReturn(I)Z          0: iload_0; 1: ireturn
/// intLiteral()Z          0: iconst_2; 1: ireturn
/// literalCondition()I    0: iconst_1; 1: ifeq 6; 4: iconst_1; 5: ireturn; 6: iconst_0; 7: ireturn
/// overwrite(Z)Z          0: iload_0; 1: istore_1; 2: iconst_2; 3: istore_1; 4: iload_1; 5: ireturn
/// paramOverwrite(Z)Z     0: iconst_2; 1: istore_0; 2: iload_0; 3: ireturn
/// ```
///
/// `intReturn` is the JVM's own view of the two int-shaped primitives: an `ireturn` in a method
/// whose descriptor says `Z` verifies, and a compiler cannot write it. `intLiteral` is the same
/// question with a literal that is *not* a boolean: `2` is an `int` value in a boolean context, and
/// the layer has no fact that says otherwise. `literalCondition` is a branch whose operand is the
/// literal `1` — a truth test in a boolean context, and the *only* shape that reaches the literal
/// evidence in the condition position (`javac` folds a constant condition away). `overwrite` writes
/// a value no evidence proves boolean into a local its own first write declared `boolean`, and
/// `paramOverwrite` does the same to a parameter the member's descriptor declares `Z`: a compiler
/// cannot emit either, because the source that would (`c = 2;` with a `boolean c`) is not Java.
const HAND_MEMBERS: &[HandMember] = &[
    HandMember {
        name: "intReturn",
        descriptor: "(I)Z",
        code: &[0x1a, 0xac],
        max_stack: 1,
        max_locals: 1,
    },
    HandMember {
        name: "intLiteral",
        descriptor: "()Z",
        code: &[0x05, 0xac],
        max_stack: 1,
        max_locals: 0,
    },
    HandMember {
        name: "literalCondition",
        descriptor: "()I",
        code: &[0x04, 0x99, 0x00, 0x05, 0x04, 0xac, 0x03, 0xac],
        max_stack: 1,
        max_locals: 0,
    },
    HandMember {
        name: "overwrite",
        descriptor: "(Z)Z",
        code: &[0x1a, 0x3c, 0x05, 0x3c, 0x1b, 0xac],
        max_stack: 1,
        max_locals: 2,
    },
    HandMember {
        name: "paramOverwrite",
        descriptor: "(Z)Z",
        code: &[0x05, 0x3b, 0x1a, 0xac],
        max_stack: 1,
        max_locals: 1,
    },
];

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

/// The hand-built class `Hand`: one `Code` attribute per member of [`HAND_MEMBERS`]. The bytes are
/// written by this function, so the case that reads them states its own input — and no committed
/// sample has to carry a body no compiler produces.
fn hand_class() -> Vec<u8> {
    let mut output = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0); // minor_version
    u16b(&mut output, 52); // major_version: Java 8, the profile every request declares
    u16b(&mut output, 15); // constant_pool_count = 14 entries + 1
    utf8(&mut output, b"Hand"); // 1
    output.push(7);
    u16b(&mut output, 1); // 2: Class Hand
    utf8(&mut output, b"java/lang/Object"); // 3
    output.push(7);
    u16b(&mut output, 3); // 4: Class java/lang/Object
    utf8(&mut output, b"Code"); // 5
    utf8(&mut output, b"intReturn"); // 6
    utf8(&mut output, b"(I)Z"); // 7
    utf8(&mut output, b"intLiteral"); // 8
    utf8(&mut output, b"()Z"); // 9
    utf8(&mut output, b"literalCondition"); // 10
    utf8(&mut output, b"()I"); // 11
    utf8(&mut output, b"overwrite"); // 12
    utf8(&mut output, b"(Z)Z"); // 13
    utf8(&mut output, b"paramOverwrite"); // 14

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
            "intReturn" => 6,
            "intLiteral" => 8,
            "literalCondition" => 10,
            "overwrite" => 12,
            "paramOverwrite" => 14,
            other => panic!("no pool entry for `{other}`"),
        };
        let descriptor_index = match member.descriptor {
            "(I)Z" => 7,
            "()Z" => 9,
            "()I" => 11,
            "(Z)Z" => 13,
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

#[test]
fn a_value_without_boolean_evidence_is_refused_in_a_z_return() {
    // `intReturn(I)Z` returns an `int` value from a `boolean` method. Before this rule the layer
    // published `return arg0;` under the member's own `Z` descriptor — text javac refuses
    // (`incompatible types: int cannot be converted to boolean`) while the report claimed Java and
    // full structure. The value has no evidence that it is a boolean, so the region is refused.
    let engine = Engine::new();
    let bytes = hand_class();
    let fixture = fixture(&engine, &bytes);
    let report = recover(&engine, &fixture, b"intReturn", b"(I)Z");
    assert_refused(&report, "intReturn", 1);
    assert!(
        !report.text.contains("return arg0;"),
        "the refused value may not be published in its `int` spelling:\n{}",
        report.text
    );
    assert!(
        report.text.contains("returns `Z`") && report.text.contains("no evidence"),
        "the refusal states the context that could not be typed:\n{}",
        report.text
    );

    // And the same rule for a literal that is not a boolean: `2` is an `int` in a `Z` return.
    let report = recover(&engine, &fixture, b"intLiteral", b"()Z");
    assert_refused(&report, "intLiteral", 1);
    assert!(
        !report.text.contains("return 2;"),
        "a non-boolean literal is not published in a boolean context:\n{}",
        report.text
    );
}

#[test]
fn a_literal_operand_is_a_truth_test() {
    // `literalCondition()I` is `iconst_1; ifeq …`: the branch tests the literal `1`, which is how a
    // boolean is pushed. In a boolean context it is `true`, not `1`, exactly as `typed_arguments`
    // spells a `1` passed to a `Z` parameter. The hand-built class is the only way to reach this
    // rule: a compiler folds a constant condition away before it writes a branch.
    let engine = Engine::new();
    let bytes = hand_class();
    let fixture = fixture(&engine, &bytes);
    let text = whole_body(&engine, &fixture, b"literalCondition", b"()I");
    assert!(text.contains("if (true) {"), "{text}");
    assert!(
        !text.contains("1 != 0"),
        "the literal is a boolean in a boolean context, not an `int` compared with `0`:\n{text}"
    );
}

#[test]
fn a_store_without_boolean_evidence_into_a_boolean_variable_is_refused() {
    // The store side of the same rule, in both directions of "the run knows the variable holds a
    // boolean": a local its own first write declared `boolean`, and a parameter the member's
    // descriptor declares `Z`. `local1 = 2;` beside `boolean local1 = …;` is text the variable's own
    // type rejects, exactly like the `int` spelling of a boolean return — so the region is refused
    // and the refusal names the variable and the BCI.
    let engine = Engine::new();
    let bytes = hand_class();
    let fixture = fixture(&engine, &bytes);

    let report = recover(&engine, &fixture, b"overwrite", b"(Z)Z");
    assert_refused(&report, "overwrite", 3);
    assert!(
        report.text.contains("boolean local1 = arg0;"),
        "the declaration this run wrote is what makes the later store's context boolean:\n{}",
        report.text
    );
    assert!(
        report.text.contains("return local1;"),
        "and the refusal is the store alone — the member's own boolean use is still written:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("local1 = 2;"),
        "the unproven store may not be published:\n{}",
        report.text
    );
    assert!(
        report.text.contains("`local1`") && report.text.contains("holds a `boolean`"),
        "the refusal names the variable whose type it could not satisfy:\n{}",
        report.text
    );

    let report = recover(&engine, &fixture, b"paramOverwrite", b"(Z)Z");
    assert_refused(&report, "paramOverwrite", 1);
    assert!(
        report.text.contains("`arg0`"),
        "a store into a `Z` parameter is refused the same way:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("arg0 = 2;"),
        "the unproven store may not be published:\n{}",
        report.text
    );
}
