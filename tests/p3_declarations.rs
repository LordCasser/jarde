//! P3 tasks 6.1/6.2/6.5/6.6: the declaration slice — the package a class's own name states, a
//! member's declared `throws`, an interface member's `default`, and a field's own initializer.
//!
//! Each of the four is a fact the class file's **own** declaration states, and each is written from
//! that fact and nothing else:
//!
//! * the `package` line is the package `this_class` states (`p/Ops2` declares `p`) and the class
//!   declaration carries the simple name beside it — a name with no `/` is the default package and
//!   writes no line at all (task 6.1);
//! * a member's `throws` clause is its own `Exceptions` attribute (JVMS 4.7.4), in the attribute's
//!   own order; no exception is ever inferred from an `athrow` (task 6.2);
//! * `default` is written exactly when the class's own flags say `ACC_INTERFACE`, the member is
//!   neither `static` nor `abstract`, and its declaration carries a `Code` attribute (task 6.5);
//! * a field's initializer is its own `ConstantValue` attribute (JVMS 4.7.2): a value this
//!   presentation has no literal for — a `float`/`double` constant — and a field without the
//!   attribute keep an initializer-less declaration, and nothing is read off a `<clinit>` (task 6.6).
//!
//! The sample is `tests/fixtures/p3-declarations/` (javac 23.0.1 `--release 8 -g:none -d v8
//! Ops2.java Consts.java`; the command is in both sources, and the class files are 52.0, 289 and 351
//! bytes, SHA-256 `a0fcfeb3…fd030e` and `deafe155…b45151`):
//!
//! * `Ops2` is `public interface p.Ops2` with four members, one per case of the `default` rule:
//!   `plus` is `abstract` and declares no `Code`, `zero` declares one and is neither `static` nor
//!   `abstract`, `unit` is `static`, and `load` declares neither `Code` nor `default` but does
//!   declare an `Exceptions` attribute;
//! * `Consts` is `class p.Consts`, whose fields each carry the `ConstantValue` the compiler folded
//!   their initializer into — `int`, `long`, `boolean` and `String` values that spell, a `double` and
//!   a `float` that do not, and one field with no attribute at all.

use jarde::*;
use std::slice;

/// The interface sample: `plus`, `zero`, `unit` and `load`.
const OPS2: &[u8] = include_bytes!("fixtures/p3-declarations/v8/p/Ops2.class");

/// The constants sample: seven fields, one `ConstantValue` each except `plain`.
const CONSTS: &[u8] = include_bytes!("fixtures/p3-declarations/v8/p/Consts.class");

/// A committed sample in the **default** package: the control for "a name with no `/` writes no
/// `package` line", read-only, exactly as the class-source tests present it.
const HISTORICAL: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a standalone CLASS")
}

/// One class-source presentation of a committed sample, under the entry point the CLI calls.
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
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one committed sample answers one definition, got {other:?}"),
    }
}

/// The class declaration of one report, which every sample here presents.
fn declaration_of(report: &ClassSourceReport) -> &ClassSourceDeclaration {
    report
        .declaration
        .as_ref()
        .expect("a presented class declares itself")
}

/// The member of one class, by the raw name its class file declares.
fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
}

/// One member's own text in the assembled source.
fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &member(report, name).text
}

/// One member's declaration as this presentation published it, without its `;` or its block.
fn member_declaration<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    member(report, name)
        .declaration
        .as_deref()
        .unwrap_or_else(|| panic!("the member `{name}` is spelled"))
}

/// The field of one class, by the raw name its class file declares.
fn field<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceField {
    report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no field `{name}` in the sample's field table"))
}

/// One field's declaration as this presentation published it, without its `;`.
fn field_declaration<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    field(report, name)
        .declaration
        .as_deref()
        .unwrap_or_else(|| panic!("the field `{name}` is spelled"))
}

/// The first line of one text that is not one of the envelope's `//` comment lines: the first thing
/// this presentation claims, and where the `package` line has to be.
fn first_statement(text: &str) -> &str {
    text.lines()
        .find(|line| !line.trim_start().starts_with("//"))
        .expect("a presented class states something")
}

/// The names of the members whose published declaration carries one word.
fn members_carrying(report: &ClassSourceReport, word: &str) -> Vec<String> {
    report
        .methods
        .iter()
        .filter(|method| {
            method
                .declaration
                .as_deref()
                .is_some_and(|declaration| declaration.contains(word))
        })
        .map(|method| String::from_utf8_lossy(&method.item.name.raw().0).into_owned())
        .collect()
}

/// The names of the fields whose published declaration carries one word.
fn fields_carrying(report: &ClassSourceReport, word: &str) -> Vec<String> {
    report
        .fields
        .iter()
        .filter(|field| {
            field
                .declaration
                .as_deref()
                .is_some_and(|declaration| declaration.contains(word))
        })
        .map(|field| String::from_utf8_lossy(&field.item.name.raw().0).into_owned())
        .collect()
}

#[test]
fn the_package_line_is_the_package_the_classs_own_name_states() {
    let ops2 = class_source_of(&open(OPS2), "p/Ops2");
    let consts = class_source_of(&open(CONSTS), "p/Consts");

    // `p/Ops2` states the package `p`, and it is the first thing the text claims: the declaration
    // itself carries the simple name, and the internal name stays published in the item it was read
    // from.
    assert_eq!(first_statement(&ops2.text), "package p;");
    assert!(
        ops2.text
            .contains("package p;\n\npublic interface Ops2 {\n"),
        "{}",
        ops2.text
    );
    assert_eq!(declaration_of(&ops2).name, "Ops2");
    assert_eq!(declaration_of(&ops2).declaration, "public interface Ops2");
    assert_eq!(
        declaration_of(&ops2).item.declaration.this_class.raw().0,
        b"p/Ops2"
    );

    // The same rule for an ordinary class: the package is the name's own last `/`, and the
    // declaration is written with what follows it.
    assert_eq!(first_statement(&consts.text), "package p;");
    assert!(
        consts
            .text
            .contains("package p;\n\nclass Consts extends java.lang.Object {\n"),
        "{}",
        consts.text
    );
    assert_eq!(declaration_of(&consts).name, "Consts");
    assert_eq!(
        declaration_of(&consts).declaration,
        "class Consts extends java.lang.Object"
    );

    // A name with no `/` is the default package: source writes no `package` line for it at all, and
    // the declaration itself is the text's first claim.
    let historical = class_source_of(&open(HISTORICAL), "HistoricalControlFlow");
    assert_eq!(
        first_statement(&historical.text),
        "public class HistoricalControlFlow extends java.lang.Object {"
    );
    assert!(
        !historical
            .text
            .lines()
            .any(|line| line.starts_with("package ")),
        "the default package writes no line:\n{}",
        historical.text
    );
}

#[test]
fn a_members_throws_clause_is_its_own_exceptions_attribute() {
    let report = class_source_of(&open(OPS2), "p/Ops2");

    // `load` declares the clause its `Exceptions` attribute states, in the attribute's own order,
    // and it keeps its own declaration: an `abstract` member of an interface is nothing else.
    assert_eq!(
        member_declaration(&report, "load"),
        "public abstract void load(java.lang.String arg1) throws java.io.IOException, \
         java.lang.InterruptedException"
    );
    assert!(
        text_of(&report, "load").contains(
            "    public abstract void load(java.lang.String arg1) throws java.io.IOException, \
             java.lang.InterruptedException;\n"
        ),
        "{}",
        text_of(&report, "load")
    );

    // The clause cannot have been inferred from a body: `load` declares no `Code` attribute at all,
    // so its own declaration is the only thing that could state one.
    assert_eq!(
        member(&report, "load").no_body_kind,
        Some(NoBodyKind::Abstract)
    );

    // A member whose own attribute declares no exception writes no clause, and no member inherits
    // one from another member or from a body.
    for name in ["plus", "zero", "unit"] {
        assert!(
            !member_declaration(&report, name).contains("throws"),
            "`{name}` declares no Exceptions attribute:\n{}",
            text_of(&report, name)
        );
    }
    assert_eq!(members_carrying(&report, "throws"), ["load"]);
}

#[test]
fn an_interface_member_is_default_exactly_when_the_class_and_the_member_say_so() {
    let report = class_source_of(&open(OPS2), "p/Ops2");

    // The class the read established is the interface whose own flags the `default` rule reads.
    assert_eq!(
        declaration_of(&report).item.declaration.access_flags & 0x0200,
        0x0200
    );

    // `zero` is `public` with a `Code` attribute and neither `static` nor `abstract`: the one member
    // of this sample that is `default`, and the keyword stands between the modifiers and the return
    // type.
    assert_eq!(
        member(&report, "zero").item.access_flags & (0x0008 | 0x0400),
        0
    );
    assert_eq!(
        member_declaration(&report, "zero"),
        "public default int zero()"
    );
    assert!(
        text_of(&report, "zero").contains("    public default int zero() {\n"),
        "{}",
        text_of(&report, "zero")
    );

    // `unit` is an interface member too, and its own `static` flag keeps `default` off it.
    assert_eq!(
        member_declaration(&report, "unit"),
        "public static int unit()"
    );
    assert!(
        text_of(&report, "unit").contains("    public static int unit() {\n"),
        "{}",
        text_of(&report, "unit")
    );

    // `plus` and `load` declare no `Code`, so neither is a `default` member: `plus` stays the
    // abstract declaration it is.
    assert_eq!(
        member_declaration(&report, "plus"),
        "public abstract int plus(int arg1, int arg2)"
    );
    assert_eq!(
        member(&report, "plus").no_body_kind,
        Some(NoBodyKind::Abstract)
    );

    // Every member of the sample is spelled once, and the keyword appears exactly where the rule put
    // it.
    assert_eq!(report.methods.len(), 4, "every member is presented");
    assert_eq!(members_carrying(&report, "default"), ["zero"]);
}

#[test]
fn a_fields_initializer_is_the_constant_its_own_attribute_states() {
    let report = class_source_of(&open(CONSTS), "p/Consts");

    // One field per kind the presentation has a literal for: the value is the attribute's own and the
    // spelling is the one Java writes for that type.
    let written = [
        ("N", "static final int N = 3"),
        ("L", "static final long L = 3L"),
        ("FLAG", "static final boolean FLAG = true"),
        ("S", "static final java.lang.String S = \"a\""),
    ];
    for (name, declaration) in written {
        assert_eq!(field_declaration(&report, name), declaration);
        assert!(
            report.text.contains(&format!("    {declaration};\n")),
            "the field is one indented line of the text:\n{}",
            report.text
        );
    }

    // A value this presentation has no literal for — a `double` and a `float` constant — is no
    // initializer at all, and a field with no `ConstantValue` states none either. Nothing is read off
    // a `<clinit>`, which this class does not declare: its one method is the constructor.
    for (name, declaration) in [
        ("D", "static final double D"),
        ("F", "static final float F"),
        ("plain", "static int plain"),
    ] {
        assert_eq!(field_declaration(&report, name), declaration);
        assert!(
            report.text.contains(&format!("    {declaration};\n")),
            "{}",
            report.text
        );
    }

    // Only the fields whose own attribute states a literal carry one, and they are exactly the four
    // above: one `=` per written initializer and none elsewhere.
    assert_eq!(report.fields.len(), 7, "every field is presented");
    assert_eq!(fields_carrying(&report, "="), ["N", "L", "FLAG", "S"]);

    // The class's own declaration is unchanged by any of it, and its one member is the constructor:
    // a class member never gains `default`.
    assert_eq!(
        declaration_of(&report).declaration,
        "class Consts extends java.lang.Object"
    );
    assert_eq!(member_declaration(&report, "<init>"), "Consts()");
    assert_eq!(members_carrying(&report, "default"), Vec::<String>::new());
}

/// A field is published only where its own declaration was established: a field whose
/// `ConstantValue` could not be read is never written as a declaration **without** the initializer
/// the attribute states — a field record states no refusal of its own, so the presentation stops
/// where that read happened (the fields before it keep their spellings).
///
/// The item budget a presentation needs before it reaches a field's attribute is the reader's own
/// accounting, so this case walks the boundary instead of naming the count: the limit it walks up to
/// is the attribute bytes the same request charges when nothing is refused, and the invariant it
/// checks stays the same whatever that count becomes.
#[test]
fn a_field_whose_constant_value_cannot_be_read_is_never_spelled_without_it() {
    let snapshot = open(CONSTS);
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("p/Consts"),
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
    let written = [
        ("N", "static final int N = 3"),
        ("L", "static final long L = 3L"),
        ("FLAG", "static final boolean FLAG = true"),
        ("S", "static final java.lang.String S = \"a\""),
    ];
    let table: Vec<&str> = ["N", "L", "FLAG", "S", "D", "F", "plain"].to_vec();
    // The same request under the task's own budget: the class is presented whole, and what it charged
    // in `attribute_bytes` is the walk's upper bound.
    let whole = match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("the task budget presents the whole sample")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one committed sample answers one definition, got {other:?}"),
    };
    assert!(
        whole.usage.attribute_bytes > 0,
        "the walk needs a bound: {:?}",
        whole.usage
    );
    let mut stopped_at_a_field = false;
    // One is the smallest limit the engine accepts: a dimension that cannot fund one unit of work is
    // not a budget at all.
    for limit in 1..=whole.usage.attribute_bytes {
        let mut budget =
            task_budget(
                &[BudgetOverride::new("attribute_bytes", limit).expect("a legal override")],
            )
            .expect("the override is legal");
        let outcome = match Engine::new().class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        ) {
            // The class read itself was refused before anything was established to present.
            Err(Error::BudgetExceeded { .. }) => continue,
            Err(error) => panic!("at {limit} attribute byte(s): unexpected {error:?}"),
            Ok(outcome) => outcome,
        };
        let report = match outcome {
            OperationOutcome::Performed(report) => report,
            // The search itself did not finish at this limit: nothing was bound, so nothing is
            // established to present and there is no field to check.
            OperationOutcome::Incomplete(_) => continue,
            other => panic!("one committed sample answers one definition, got {other:?}"),
        };
        // What is published is a prefix of the field table, never a field picked out of order, and a
        // field of the four whose attribute states a spellable literal carries it — the one way this
        // presentation can write `= <literal>` is from the attribute it really read.
        let published: Vec<String> = report
            .fields
            .iter()
            .map(|field| String::from_utf8_lossy(&field.item.name.raw().0).into_owned())
            .collect();
        assert_eq!(
            published,
            table[..published.len()],
            "at {limit} attribute byte(s)"
        );
        for (name, declaration) in written {
            if published.iter().any(|field| field == name) {
                assert_eq!(
                    field_declaration(&report, name),
                    declaration,
                    "at {limit} attribute byte(s)"
                );
            }
        }
        if matches!(report.execution, ExecutionReport::Complete { .. }) {
            assert_eq!(published.len(), table.len(), "at {limit} attribute byte(s)");
        }
        // A field's own attribute read is what stopped the presentation when the class-level
        // diagnostic names that dimension: the declaration was established (the report is
        // `Performed`) and no field was published from bytes this request never read.
        if report.fields.is_empty()
            && report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "budget_exceeded_attribute_bytes")
        {
            stopped_at_a_field = true;
            assert!(!report.text.contains(" = "), "{}", report.text);
        }
    }
    assert!(
        stopped_at_a_field,
        "some attribute limit refuses the first field's own ConstantValue read"
    );
}
