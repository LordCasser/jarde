//! P3 1.1 acceptance: the product vocabulary a P3 result has to be able to state.
//!
//! The recovery layer's own declarations (recovery profile, pattern preconditions, rule version,
//! failure fallback) land in 1.3, with the first real pass and the crate that hosts it. What this
//! slice adds is the vocabulary those declarations are *reported* in, and that vocabulary already
//! exists: the planes P2 publishes. P3's `recovery-validation` requirement fixes their value sets
//! one by one — `representation`（Java/Bytecode/Mixed）, `quality`（Structured/Conservative/
//! Fallback）, `syntax_status`（Checked/Unchecked/NotJava）, `compile_status`（NotAttempted/
//! Compiles/Failed）, `semantic_validation`（LocalInvariants/FixtureDifferential/Unproven）and
//! `verification`（Performed/NotPerformed/Failed）— with `Mixed` fixed to the representation plane
//! and never to quality; `java8-recovery` writes the same values in its own sentences
//! (`representation=Mixed/Bytecode`, `quality=Conservative/Fallback`, and a successful
//! `Structured` flag that a degraded result must not publish).
//!
//! This file pins those six closed sets to the sentences and to each other, through the names the
//! CLI adapter uses them by (`jarde::*`), so what is under test is the published vocabulary and
//! not a crate-private twin. Two things are asserted for every value of every plane:
//!
//! 1. it publishes the wire name the report planes use — the `snake_case` spelling derived from
//!    the variant itself, so a value a sentence writes cannot hide behind a different name — and
//!    deserializes back to itself;
//! 2. the value set is exactly the variants the declaring file writes, in declaration order: a
//!    variant added later (1.3's producer, a P4 plane) fails here until this file names the
//!    sentence that requires it. That is the job `EnvironmentProblemCode::ALL` and
//!    `CountedBudgetDimension::ALL` do in `tests/p2_contracts.rs` for their closed sets; the
//!    expectation lives here instead of on the types because no production code iterates these
//!    planes, and a public `ALL` list would exist for this guard alone.
//!
//! The **content classification of a delivered artifact** ([`RecoveryContent`]) is pinned here for
//! the same reason and by the same reading of its declaring file, though it is not one of those six
//! planes: no P2 report carries it, it is stated by the recovery report from the artifact that was
//! committed, and (being a report output and not a report input) it publishes without deserializing.
//! Its set is closed at exactly the three values the `java8-recovery` sentences write, and a fourth
//! value is a change to that contract rather than an addition to it.
//!
//! Neither check is a producer, and neither may become one: this slice adds no variant that a run
//! of this build can emit. The P2 baseline of every plane — `bytecode`, `not_java`,
//! `not_attempted`, `not_performed`, with a quality of `conservative`/`fallback` — is asserted on
//! real requests by `tests/p2_contracts.rs`, `tests/p2_properties.rs`, `tests/p2_frame.rs`,
//! `tests/p2_ssa.rs` and `crates/jarde-cli/tests/json_cli.rs`; those assertions are the evidence
//! that widening the sets changed no output, and this slice left them untouched.

use jarde::{
    CompileStatus, Quality, RecoveryContent, Representation, SemanticValidation, SyntaxStatus,
    VerificationStatus,
};
use std::path::Path;

/// The file that declares the IR report schema and five of its six planes.
const IR_MODULE: &str = "crates/jarde-jvm/src/ir.rs";

/// The file that declares the classfile vocabulary the `verification` plane reuses: P2's report
/// states that plane with the reader's type, so the set is declared where the type lives.
const CLASSFILE_MODULE: &str = "crates/jarde-reader/src/classfile.rs";

/// The file that declares the recovery report and its own closed classification of what the delivered
/// artifact holds. The type lives with the report it belongs to, so this is where its set is read.
const REPORT_MODULE: &str = "crates/jarde-java/src/report.rs";

/// Asserts one plane's whole vocabulary: every listed value publishes its `snake_case` name and
/// round-trips, and the list is exactly the declaring file's variants, in declaration order.
///
/// The variant names are read back from the values themselves (`Debug` on a unit-only enum is the
/// declared name) and compared with the declaration, so the JSON spelling is checked against the
/// name the source writes rather than against a second hand-written list.
macro_rules! plane {
    ($source:expr, $enum:literal, $ty:ty, [$($value:ident),+ $(,)?]) => {{
        let values: Vec<$ty> = vec![$(<$ty>::$value),+];
        let mut names = Vec::new();
        for value in values {
            let name = format!("{value:?}");
            let json = serde_json::to_string(&value).expect("a plane value serializes");
            assert_eq!(
                json,
                format!("\"{}\"", serde_code(&name)),
                "`{name}` must publish the snake_case name the report planes use"
            );
            assert_eq!(
                serde_json::from_str::<$ty>(&json)
                    .unwrap_or_else(|error| panic!("`{name}` must deserialize: {error}")),
                value,
                "`{name}` must deserialize back to itself"
            );
            names.push(name);
        }
        assert_eq!(
            names,
            declared_variants($source, $enum),
            "the values listed for `{}` must be every variant it declares, in declaration order",
            stringify!($ty)
        );
    }};
}

#[test]
fn the_six_report_planes_declare_the_values_the_p3_sentences_write() {
    let ir = read_repository_file(IR_MODULE);
    let classfile = read_repository_file(CLASSFILE_MODULE);

    // `representation`（Java/Bytecode/Mixed）: what the output is made of. `bytecode` is the P2
    // baseline; the recovery layer adds `java` (Java text throughout) and `mixed` (Java regions
    // beside bytecode fallbacks), the value `java8-recovery` writes as `representation=Mixed/
    // Bytecode` for a result that keeps low-level structure. The design's output table and the
    // same requirement put `Structured` on the quality plane and fix `Mixed` to this one
    // ("`Mixed` 只表示 representation，禁止把它当作 quality").
    plane!(
        &ir,
        "Representation",
        Representation,
        [Bytecode, Java, Mixed]
    );

    // `quality`（Structured/Conservative/Fallback）: how much of the input was recovered.
    // `java8-recovery` writes `quality=Conservative/Fallback` and forbids a degraded result from
    // publishing a successful `Structured` flag, and `conservative-output` fixes the P2 baseline
    // of this plane to `Conservative`/`Fallback`.
    plane!(
        &ir,
        "Quality",
        Quality,
        [Conservative, Fallback, Structured]
    );

    // `syntax_status`（Checked/Unchecked/NotJava）: whether the produced text is Java syntax, and
    // whether that syntax was checked. `not_java` is the P2 baseline; the recovery scenario that
    // holds a name or control structure Java cannot express keeps `not_java` as well, with
    // `representation` still `Java` or `Mixed`.
    plane!(
        &ir,
        "SyntaxStatus",
        SyntaxStatus,
        [NotJava, Checked, Unchecked]
    );

    // `compile_status`（NotAttempted/Compiles/Failed）: whether the produced text was really
    // recompiled in a declared environment. `not_attempted` is the P2 baseline and stays the value
    // whenever no compilation ran; `compiles`/`failed` are the controlled recompilation's own
    // outcome (P3 3.3), and `failed` is only for a compilation that really ran.
    plane!(
        &ir,
        "CompileStatus",
        CompileStatus,
        [NotAttempted, Compiles, Failed]
    );

    // `semantic_validation`（LocalInvariants/FixtureDifferential/Unproven）: the strongest
    // evidence that applies. P2 already reserved `FixtureDifferential` for the controlled
    // comparison, which is why 1.1 adds nothing here — P3 3.3's recompile and behavior comparison
    // is that evidence class, per-sample and never claimed by an ordinary request.
    plane!(
        &ir,
        "SemanticValidation",
        SemanticValidation,
        [LocalInvariants, FixtureDifferential, Unproven]
    );

    // `verification`（Performed/NotPerformed/Failed）: whether a declared verification really ran.
    // `not_performed` is the P2 baseline and the state every P0/P1/P2 report keeps; the recovery
    // layer states `performed`/`failed` when its controlled check really runs.
    plane!(
        &classfile,
        "VerificationStatus",
        VerificationStatus,
        [NotPerformed, Performed, Failed]
    );
}

/// The recovery report's own closed classification, read the way the six planes above are read: every
/// value publishes the `snake_case` name derived from the variant, and the list is exactly the
/// variants the declaring file writes, in declaration order.
///
/// The round-trip half of `plane!` is deliberately absent: the report types of `jarde-java` are output
/// documents (`RecoveryReport`, `RecoveryOutcome` and `RegionRecord` all derive `Serialize` alone), so
/// a value of this enum is proved to publish its name, and comparing it with the declaration is what
/// keeps the set closed.
#[test]
fn the_recovery_content_classification_is_the_closed_published_set() {
    // `content`（NotProduced/ExplanationOnly/ContainsStatements）: what the delivered artifact holds.
    // `not_produced` is the stop (nothing was committed, so there is no content to describe),
    // `explanation_only` is an artifact whose whole content is the envelope, its reasons and the
    // bytecode they quote, and `contains_statements` is one the emitter wrote a statement into — a
    // declaration, an assignment, a call, a constructor call, a `return` or a control-flow statement.
    // `Produced` alone says none of that, which is the sentence this set exists for.
    let report = read_repository_file(REPORT_MODULE);
    let values: Vec<RecoveryContent> = vec![
        RecoveryContent::NotProduced,
        RecoveryContent::ExplanationOnly,
        RecoveryContent::ContainsStatements,
    ];
    let mut names = Vec::new();
    for value in values {
        let name = format!("{value:?}");
        assert_eq!(
            serde_json::to_string(&value).expect("a content value serializes"),
            format!("\"{}\"", serde_code(&name)),
            "`{name}` must publish the snake_case name the recovery report uses"
        );
        names.push(name);
    }
    assert_eq!(
        names,
        declared_variants(&report, "RecoveryContent"),
        "the values listed here must be every variant `RecoveryContent` declares, in declaration \
         order: the classification is closed, and a fourth value is a change to that contract"
    );
}

fn read_repository_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{relative} is unreadable at {}: {error}", path.display()))
}

/// Variant names of one unit-only enum, in declaration order, read from its source.
///
/// The values listed by this file can only be as complete as the enum they mirror, and no `match`
/// here sees a variant the enum declares but this file has not been told about: reading the
/// declaration is what closes that loop, exactly as `tests/p2_contracts.rs` closes it for the
/// `ALL` lists.
fn declared_variants(source: &str, enum_name: &str) -> Vec<String> {
    let heading = format!("pub enum {enum_name} {{");
    let Some(start) = source.find(&heading) else {
        panic!("`{heading}` is not declared in the source under test");
    };
    let mut variants = Vec::new();
    for line in source[start + heading.len()..].lines() {
        let line = line.trim();
        if line.starts_with('}') {
            break;
        }
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }
        let name = line
            .split([',', '(', '{', '='])
            .next()
            .unwrap_or_default()
            .trim();
        assert!(
            !name.is_empty() && name.chars().all(|character| character.is_alphanumeric()),
            "unexpected variant text in `{enum_name}`: {line:?}"
        );
        variants.push(name.to_string());
    }
    assert!(
        !variants.is_empty(),
        "`{enum_name}` has no variants to compare against"
    );
    variants
}

/// `snake_case` code serde derives for one variant name.
fn serde_code(variant: &str) -> String {
    let mut code = String::with_capacity(variant.len() + 4);
    for character in variant.chars() {
        if character.is_uppercase() && !code.is_empty() {
            code.push('_');
        }
        code.extend(character.to_lowercase());
    }
    code
}
