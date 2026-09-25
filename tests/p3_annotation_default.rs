//! P3 task 6.8: an annotation type's members declare their defaults, and the declaration writes them.
//!
//! `String value() default "x"` is not a body and does not compile to one: `javac` writes one
//! `AnnotationDefault` attribute (JVMS 4.7.22) on the member, and the class file's own bytes there
//! are the declaration. The presentation wrote `public abstract java.lang.String value();` — the
//! default was silently absent, which is the one thing this presentation must never do — and the
//! attribute is the same class of fact as a field's `ConstantValue`: the artifact's own statement
//! about the declaration, so it is spelled and never inferred.
//!
//! The sample is `tests/fixtures/p3-annotation-default/` (javac 23.0.1 `--release 8 -g:none`; the
//! command, the 448/725 bytes and the SHA-256 of both class files are in its README):
//!
//! * `Marker`'s members carry one tag each — `s`, `I`, `Z`, `J`, `C`, `e`, `[`, `D` — and each is a
//!   control for the others: the same `CONSTANT_Integer` is `1`, `true` and `'A'` under three tags,
//!   the enum's two pool entries are `Kind.ONE` and not the string `"ONE"`, and the array is written
//!   `{3, 4}`;
//! * `ratio` declares a `D` attribute and is the one member that keeps a declaration without a
//!   `default`: a `double` constant's digits are a spelling rule of their own (task 2c.5), and this
//!   presentation invents no literal;
//! * `Kind` — the enum the `e` value's type index names, written by the same command — presents
//!   normally and declares no `AnnotationDefault`, so it carries no `default` anywhere.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 448 bytes and the SHA-256).
const MARKER: &[u8] = include_bytes!("fixtures/p3-annotation-default/v8/Marker.class");

/// The enum the sample's `e` value names, written by the same command (README: 725 bytes, SHA-256).
const KIND: &[u8] = include_bytes!("fixtures/p3-annotation-default/v8/Kind.class");
const FLOAT_DEFAULTS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/annotation-float-defaults/classes-original/FloatDefaults.class"
);

/// Java 8 nested-annotation default fixture, retained with the audit's complete runner.
const NESTED: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/annotation-default-boundaries/header-minimal/nested/classes-original/Nested.class"
);

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
    class_source_with_evidence(snapshot, name, &RecoveryEvidenceRequest::all())
}

fn class_source_with_evidence(
    snapshot: &ArtifactSnapshot,
    name: &str,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let request = class_source_request(snapshot, name);
    match Engine::new()
        .class_source_with_evidence(slice::from_ref(snapshot), &request, evidence, &mut budget())
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one committed sample answers one definition, got {other:?}"),
    }
}

fn class_source_default(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    match Engine::new()
        .class_source(
            slice::from_ref(snapshot),
            &class_source_request(snapshot, name),
            &mut budget(),
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one committed sample answers one definition, got {other:?}"),
    }
}

fn class_source_request(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceRequest {
    ClassSourceRequest {
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
    }
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
fn declaration_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    member(report, name)
        .declaration
        .as_deref()
        .unwrap_or_else(|| panic!("the member `{name}` is spelled"))
}

#[test]
fn an_annotation_members_default_is_written_from_its_own_attribute() {
    let sample = open(MARKER);
    let report = class_source_of(&sample, "Marker");

    // The class is the annotation type the class file declares. Java supplies its canonical
    // Annotation superinterface implicitly; the structured item still states the physical table.
    assert!(
        report.text.contains("public @interface Marker {\n"),
        "{}",
        report.text
    );
    let declaration = report.declaration.as_ref().expect("the class is declared");
    assert_eq!(declaration.item.declaration.access_flags & 0x2200, 0x2200);
    assert_eq!(
        declaration.item.declaration.interfaces[0].raw().0,
        b"java/lang/annotation/Annotation"
    );

    // Every member's own declaration carries the default its own attribute states, written between
    // the parameter list and the `;` — one literal per tag, and no literal is inferred from another.
    let expected = [
        (
            "value",
            "public abstract java.lang.String value() default \"x\";",
        ),
        ("count", "public abstract int count() default 1;"),
        ("flag", "public abstract boolean flag() default true;"),
        ("big", "public abstract long big() default 5L;"),
        ("grade", "public abstract char grade() default 'A';"),
        ("kind", "public abstract Kind kind() default Kind.ONE;"),
        ("pair", "public abstract int[] pair() default {3, 4};"),
    ];
    for (name, line) in expected {
        assert!(
            text_of(&report, name).contains(line),
            "`{name}`'s declaration is the attribute's own default:\n{}",
            text_of(&report, name)
        );
        assert!(
            report.text.contains(line),
            "the assembled text carries it:\n{}",
            report.text
        );
        assert!(
            report.text.contains(&format!("    {line}\n")),
            "the declaration is one line at its own indentation:\n{}",
            report.text
        );
    }

    // The enum's type is the class the value's type index names and the constant is its own simple
    // name: `Kind.ONE` is neither a quoted string nor a bare `ONE`.
    let kind = text_of(&report, "kind");
    assert!(kind.contains("default Kind.ONE;"), "{kind}");
    assert!(!kind.contains("\"ONE\""), "{kind}");
    assert!(!kind.contains("default ONE"), "{kind}");

    // `ratio` keeps the exact value in its own double constant pool entry.
    let ratio = text_of(&report, "ratio");
    assert!(ratio.contains("default 0x1.0000000000000p-1d;"), "{ratio}");
    assert_eq!(
        declaration_of(&report, "ratio"),
        "public abstract double ratio() default 0x1.0000000000000p-1d"
    );

    // One member states one default and no more, and no member states two: the attribute is read
    // once per member and written once per declaration — in the declaration the report publishes and
    // in the text that carries it.
    for method in &report.methods {
        let declaration = method.declaration.as_deref().unwrap_or_default();
        let written = declaration.matches(" default ").count();
        assert!(written <= 1, "two defaults in `{declaration}`");
        assert_eq!(
            written, 1,
            "the members that declare an attribute write exactly one default: `{declaration}`"
        );
        assert_eq!(
            method.text.matches("default").count(),
            written,
            "the member's own text states it once and nothing else does:\n{}",
            method.text
        );
    }
    assert_eq!(report.methods.len(), 8, "{:?}", report.methods);

    // Nothing about the members' bodies changed: an annotation type's members declare no `Code`, so
    // every member is still the abstract declaration it is with the marker that says so, no run was
    // charged, and the one class read is the binding's.
    assert_eq!(report.usage.class_headers, 1, "{:?}", report.usage);
    assert_eq!(report.usage.method_bodies, 0, "{:?}", report.usage);
    for method in &report.methods {
        assert_eq!(method.outcome, ClassSourceOutcome::NoBody);
        assert_eq!(method.no_body_kind, Some(NoBodyKind::Abstract));
        let markers: Vec<&str> = method
            .text
            .lines()
            .map(str::trim_start)
            .filter(|line| line.starts_with("// jarde:"))
            .collect();
        assert_eq!(markers.len(), 1, "{:?}", method.text);
        assert!(
            markers[0]
                .contains("is declared abstract and its declaration carries no Code attribute"),
            "{}",
            markers[0]
        );
    }
    assert_eq!(
        report.execution,
        ExecutionReport::Complete {
            usage: report.usage.clone()
        }
    );
}

#[test]
fn nested_defaults_keep_attribute_identity_and_request_stop_semantics() {
    let snapshot = open(NESTED);
    let complete = class_source_of(&snapshot, "Nested");
    let essential = class_source_default(&snapshot, "Nested");
    assert_eq!(complete.text, essential.text);
    assert_eq!(complete.declaration, essential.declaration);
    assert!(
        complete
            .text
            .contains("public abstract Inner child() default @Inner(value = 6);")
    );
    assert!(complete.text.contains(
        "public abstract Inner[] children() default {@Inner(value = 2), @Inner(value = 3)};"
    ));

    // This remains a declaration-level presentation over the same physical class. The parsed
    // attribute value is represented in text, not added as a new public JSON fact.
    let json = serde_json::to_value(&complete).expect("the report serializes");
    assert_eq!(
        json["class"],
        serde_json::to_value(&complete.class).unwrap()
    );
    assert!(json["methods"].as_array().unwrap().iter().all(|method| {
        method.get("annotation_default").is_none() && method.get("default_value").is_none()
    }));

    // A bounded class-name search remains an incomplete request with the reader's own stop reason.
    let mut limited = task_budget(&[
        BudgetOverride::new("attribute_bytes", 16).expect("a legal budget dimension")
    ])
    .expect("the override is legal");
    let limited = Engine::new()
        .class_source(
            slice::from_ref(&snapshot),
            &class_source_request(&snapshot, "Nested"),
            &mut limited,
        )
        .expect("the bounded search is an outcome");
    let OperationOutcome::Incomplete(limited) = limited else {
        panic!("the exhausted name search is incomplete: {limited:?}");
    };
    assert!(matches!(
        limited.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AttributeBytes
            },
            ..
        }
    ));

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let mut cancelled = Budget::with_cancellation_token(Limits::default(), cancellation);
    let cancelled = Engine::new().class_source(
        slice::from_ref(&snapshot),
        &class_source_request(&snapshot, "Nested"),
        &mut cancelled,
    );
    assert!(
        matches!(
            cancelled,
            Ok(OperationOutcome::Incomplete(candidates))
                if matches!(candidates.execution, ExecutionReport::Cancelled { .. })
        ),
        "pre-cancelled source request reports cancellation as its execution stop"
    );
}

#[test]
fn floating_defaults_keep_raw_bits_in_java_constant_expressions() {
    let snapshot = open(FLOAT_DEFAULTS);
    let complete = class_source_of(&snapshot, "FloatDefaults");
    let essential = class_source_default(&snapshot, "FloatDefaults");
    assert_eq!(complete.text, essential.text);
    for expected in [
        "negativeZero() default -0.0f;",
        "subnormal() default 0x0.0000000000001p-1022d;",
        "positiveInfinity() default Float.POSITIVE_INFINITY;",
        "canonicalNaN() default Double.NaN;",
    ] {
        assert!(
            complete.text.contains(expected),
            "missing {expected}:\n{}",
            complete.text
        );
    }
    let json = serde_json::to_value(&complete).expect("the report serializes");
    assert!(json["methods"].as_array().unwrap().iter().all(|method| {
        method.get("annotation_default").is_none() && method.get("default_value").is_none()
    }));
}

#[test]
fn an_unspellable_float_descendant_refuses_the_whole_array_default() {
    let bytes = crafted_class(|pool| {
        let descriptor = pool.utf8(b"()[F");
        let good = pool.float(0x3f80_0000);
        let payload_nan = pool.float(0x7fc0_0001);
        let mut values = vec![b'['];
        u16b(&mut values, 2);
        values.extend_from_slice(&tag(b'F', good));
        values.extend_from_slice(&tag(b'F', payload_nan));
        vec![abstract_member(pool, b"values", descriptor, vec![values])]
    });
    let report = crafted_report(&bytes);
    let values = member(&report, "values");
    assert_eq!(
        values.declaration.as_deref(),
        Some("public abstract float[] values()")
    );
    assert!(!values.text.contains("default"), "{}", values.text);
}

/// The enum the `e` value names is a class of its own, and the change does not reach it: it declares
/// no `AnnotationDefault`, so it writes no `default` anywhere — the attribute's presence is the fact,
/// and the class the value *names* is not the class that declares it.
#[test]
fn the_class_an_enum_default_names_declares_no_annotation_default() {
    let sample = open(KIND);
    let report = class_source_of(&sample, "Kind");

    assert!(report.text.contains("enum Kind {\n"), "{}", report.text);
    assert!(!report.text.contains("default"), "{}", report.text);
    for method in &report.methods {
        assert!(
            !method
                .declaration
                .as_deref()
                .unwrap_or_default()
                .contains("default"),
            "{:?}",
            method.declaration
        );
    }
    // Its own members are the class file's: the constants' fields, the constructor the format adds,
    // and the members `javac` writes for an enum.
    let names: Vec<String> = report
        .methods
        .iter()
        .map(|method| method.item.name.escaped())
        .collect();
    for name in ["<init>", "<clinit>", "values", "valueOf"] {
        assert!(names.iter().any(|member| member == name), "{names:?}");
    }
    assert_eq!(
        report
            .fields
            .iter()
            .map(|field| field.item.name.escaped())
            .collect::<Vec<String>>(),
        ["ONE", "TWO", "$VALUES"]
    );
}

// ---------------------------------------------------------------------------------------------
// The crafted probe: the rules the committed sample cannot state
// ---------------------------------------------------------------------------------------------

/// One constant pool that interns each entry once, so a crafted fixture's indexes are stable.
#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    fn intern(&mut self, entry: Vec<u8>) -> u16 {
        if let Some(index) = self.entries.iter().position(|existing| *existing == entry) {
            return u16::try_from(index + 1).expect("the fixture pool index fits u16");
        }
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("the fixture pool index fits u16")
    }

    fn utf8(&mut self, text: &[u8]) -> u16 {
        let mut entry = vec![1];
        u16b(
            &mut entry,
            u16::try_from(text.len()).expect("the fixture text fits u16"),
        );
        entry.extend_from_slice(text);
        self.intern(entry)
    }

    fn class(&mut self, name: &[u8]) -> u16 {
        let name = self.utf8(name);
        let mut entry = vec![7];
        u16b(&mut entry, name);
        self.intern(entry)
    }

    fn integer(&mut self, value: i32) -> u16 {
        let mut entry = vec![3];
        entry.extend_from_slice(&value.to_be_bytes());
        self.intern(entry)
    }

    fn float(&mut self, bits: u32) -> u16 {
        let mut entry = vec![4];
        entry.extend_from_slice(&bits.to_be_bytes());
        self.intern(entry)
    }
}

fn u16b(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn u32b(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes());
}

/// One member of a crafted class: its flags, the pool indexes of its own name and descriptor, the
/// instructions of its `Code` attribute — or `None` for a member that declares no body — and one
/// `AnnotationDefault` content per element of `defaults`.
struct DefaultMember {
    flags: u16,
    name: u16,
    descriptor: u16,
    code: Option<Vec<u8>>,
    defaults: Vec<Vec<u8>>,
}

/// One `element_value` content: the tag byte and the one index every one-index tag names.
fn tag(tag: u8, index: u16) -> Vec<u8> {
    let mut content = vec![tag];
    u16b(&mut content, index);
    content
}

/// One `@` element value with its nested annotation's ordered member/value pairs.
fn annotation_value(type_index: u16, elements: &[(u16, Vec<u8>)]) -> Vec<u8> {
    let mut content = vec![b'@'];
    u16b(&mut content, type_index);
    u16b(
        &mut content,
        u16::try_from(elements.len()).expect("nested annotation elements fit u16"),
    );
    for (name, value) in elements {
        u16b(&mut content, *name);
        content.extend_from_slice(value);
    }
    content
}

/// One member that declares no body, with the `AnnotationDefault` contents it declares.
fn abstract_member(
    pool: &mut Pool,
    name: &[u8],
    descriptor: u16,
    defaults: Vec<Vec<u8>>,
) -> DefaultMember {
    DefaultMember {
        flags: 0x0401, // public abstract
        name: pool.utf8(name),
        descriptor,
        code: None,
        defaults,
    }
}

/// The constant pool and the member table of one crafted class, whose method table is exactly the
/// members `write` builds.
///
/// A crafted class is how this presentation's rules are checked against bytes this repository
/// states: `javac` writes the attribute for an annotation type only, so a member of an ordinary
/// class that declares one, a class literal default and an empty array have no compiler to write
/// them here, and the reader reads them the same way.
fn crafted_class(write: impl FnOnce(&mut Pool) -> Vec<DefaultMember>) -> Vec<u8> {
    crafted_class_with_header(0x0021, &[], write)
}

/// A crafted class with explicitly stated header flags and interfaces.
fn crafted_class_with_header(
    access_flags: u16,
    interfaces: &[&[u8]],
    write: impl FnOnce(&mut Pool) -> Vec<DefaultMember>,
) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_class = pool.class(b"Probe");
    let super_class = pool.class(b"java/lang/Object");
    let interface_indices: Vec<u16> = interfaces.iter().map(|name| pool.class(name)).collect();
    let code_name = pool.utf8(b"Code");
    let default_name = pool.utf8(b"AnnotationDefault");
    let members = write(&mut pool);

    let mut output = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut output, 0);
    u16b(&mut output, 52);
    u16b(
        &mut output,
        u16::try_from(pool.entries.len() + 1).expect("the fixture pool fits u16"),
    );
    for entry in &pool.entries {
        output.extend_from_slice(entry);
    }
    u16b(&mut output, access_flags);
    u16b(&mut output, this_class);
    u16b(&mut output, super_class);
    u16b(
        &mut output,
        u16::try_from(interface_indices.len()).expect("fixture interfaces fit u16"),
    );
    for index in interface_indices {
        u16b(&mut output, index);
    }
    u16b(&mut output, 0); // fields
    u16b(
        &mut output,
        u16::try_from(members.len()).expect("the fixture members fit u16"),
    );
    for member in &members {
        u16b(&mut output, member.flags);
        u16b(&mut output, member.name);
        u16b(&mut output, member.descriptor);
        let attributes = u16::from(member.code.is_some())
            + u16::try_from(member.defaults.len()).expect("the fixture attributes fit u16");
        u16b(&mut output, attributes);
        if let Some(code) = &member.code {
            let mut body = Vec::new();
            u16b(&mut body, 2); // max_stack
            u16b(&mut body, 4); // max_locals
            u32b(
                &mut body,
                u32::try_from(code.len()).expect("the fixture body fits u32"),
            );
            body.extend_from_slice(code);
            u16b(&mut body, 0); // exception table
            u16b(&mut body, 0); // code attributes
            u16b(&mut output, code_name);
            u32b(
                &mut output,
                u32::try_from(body.len()).expect("the fixture attribute fits u32"),
            );
            output.extend_from_slice(&body);
        }
        for content in &member.defaults {
            u16b(&mut output, default_name);
            u32b(
                &mut output,
                u32::try_from(content.len()).expect("the fixture attribute fits u32"),
            );
            output.extend_from_slice(content);
        }
    }
    u16b(&mut output, 0); // class attributes
    output
}

/// One crafted class, presented under the entry point the CLI calls.
fn crafted_report(bytes: &[u8]) -> ClassSourceReport {
    class_source_of(&open(bytes), "Probe")
}

#[test]
fn declaration_headers_keep_ordinary_and_noncanonical_superinterfaces() {
    let ordinary_interface = crafted_class_with_header(0x0601, &[b"p/Parent"], |_| Vec::new());
    let ordinary_report = crafted_report(&ordinary_interface);
    assert_eq!(
        ordinary_report
            .declaration
            .as_ref()
            .expect("interface declaration")
            .declaration,
        "public interface Probe extends p.Parent"
    );

    // An annotation flag with an extra physical interface is outside the canonical Java shape.
    // Keep the old conservative spelling and publish both entries in their original order.
    let noncanonical_annotation = crafted_class_with_header(
        0x2601,
        &[b"java/lang/annotation/Annotation", b"p/Extra"],
        |_| Vec::new(),
    );
    let report = crafted_report(&noncanonical_annotation);
    let declaration = report.declaration.as_ref().expect("annotation declaration");
    assert_eq!(
        declaration.declaration,
        "public @interface Probe extends java.lang.annotation.Annotation, p.Extra"
    );
    assert_eq!(declaration.item.declaration.interfaces.len(), 2);
    assert_eq!(
        declaration.item.declaration.interfaces[1].raw().0,
        b"p/Extra"
    );
}

/// The rules the committed sample cannot state, on one crafted class.
///
/// The sample is an annotation type, whose members all declare the attribute and whose values are the
/// seven tags `javac` writes for it. This class is not one: a `c` class literal (plain, an array, and
/// the empty array of the `[` tag), a `float` inside an array, a nested annotation, the escaping of a
/// string and a character literal, a `Z` value that is no boolean, a member with a **body** that
/// declares a default, and a member with no attribute at all.
#[test]
fn a_default_is_written_wherever_the_attribute_appears_and_only_as_far_as_it_spells() {
    let bytes = crafted_class(|pool| {
        let class_descriptor = pool.utf8(b"()Ljava/lang/Class;");
        let string_descriptor = pool.utf8(b"()Ljava/lang/String;");
        let int_descriptor = pool.utf8(b"()I");
        let char_descriptor = pool.utf8(b"()C");
        let int_array = pool.utf8(b"()[I");
        let float_array = pool.utf8(b"()[F");
        let kind_array = pool.utf8(b"()[LKind;");
        let kind_descriptor = pool.utf8(b"()LKind;");
        let boolean_descriptor = pool.utf8(b"()Z");
        let void_descriptor = pool.utf8(b"()V");
        let text = pool.utf8(b"a\"b\\c");
        let class_value = pool.utf8(b"Ljava/lang/String;");
        let array_class_value = pool.utf8(b"[I");
        let nested_type = pool.utf8(b"LKind;");
        let nested_name = pool.utf8(b"value");
        let other_name = pool.utf8(b"other");
        let invalid_name = pool.utf8(b"bad-name");
        let invalid_type = pool.utf8(b"Lbad-name;");
        let malformed_type = pool.utf8(b"Lbad.name;");
        let nested_string = pool.utf8(b"nested");
        let seven = pool.integer(7);
        let minus_seven = pool.integer(-7);
        let zero = pool.integer(0);
        let two = pool.integer(2);
        let quote = pool.integer(0x27);
        let backslash = pool.integer(0x5c);
        let newline = pool.integer(0x0a);
        let delete = pool.integer(0x7f);
        let separator = pool.integer(0x2028);
        let float_half = pool.float(0x3f00_0000);
        vec![
            // `c`: a class literal is the type its own return descriptor names, and `.class`.
            abstract_member(
                pool,
                b"m_class",
                class_descriptor,
                vec![tag(b'c', class_value)],
            ),
            // `c` over an array descriptor: the same rule, spelled from the element outwards.
            abstract_member(
                pool,
                b"m_array_class",
                class_descriptor,
                vec![tag(b'c', array_class_value)],
            ),
            // `[` with no element: the attribute's own empty array, which is `{}` and not a refusal.
            abstract_member(
                pool,
                b"m_empty",
                int_array,
                vec![{
                    let mut content = vec![b'['];
                    u16b(&mut content, 0);
                    content
                }],
            ),
            // `[` holding a `float`: an array is written only when every element of it is.
            abstract_member(
                pool,
                b"m_floats",
                float_array,
                vec![{
                    let mut content = vec![b'['];
                    u16b(&mut content, 1);
                    content.extend_from_slice(&tag(b'F', float_half));
                    content
                }],
            ),
            // `@`: preserve explicit member names and values, in their attribute order.
            abstract_member(
                pool,
                b"m_nested",
                kind_descriptor,
                vec![annotation_value(
                    nested_type,
                    &[(nested_name, tag(b'I', seven))],
                )],
            ),
            abstract_member(
                pool,
                b"m_nested_empty",
                kind_descriptor,
                vec![annotation_value(nested_type, &[])],
            ),
            abstract_member(
                pool,
                b"m_nested_multi",
                kind_descriptor,
                vec![annotation_value(
                    nested_type,
                    &[
                        (nested_name, tag(b'I', seven)),
                        (other_name, tag(b's', nested_string)),
                    ],
                )],
            ),
            abstract_member(
                pool,
                b"m_nested_array",
                kind_array,
                vec![{
                    let values = [
                        annotation_value(nested_type, &[(nested_name, tag(b'I', seven))]),
                        annotation_value(nested_type, &[(nested_name, tag(b'I', two))]),
                    ];
                    let mut content = vec![b'['];
                    u16b(&mut content, 2);
                    for value in values {
                        content.extend_from_slice(&value);
                    }
                    content
                }],
            ),
            abstract_member(
                pool,
                b"m_nested_unspellable",
                kind_array,
                vec![{
                    let values = [
                        annotation_value(nested_type, &[(nested_name, tag(b'I', seven))]),
                        annotation_value(nested_type, &[(invalid_name, tag(b'I', two))]),
                    ];
                    let mut content = vec![b'['];
                    u16b(&mut content, 2);
                    for value in values {
                        content.extend_from_slice(&value);
                    }
                    content
                }],
            ),
            abstract_member(
                pool,
                b"m_nested_bad_type",
                kind_descriptor,
                vec![annotation_value(invalid_type, &[])],
            ),
            abstract_member(
                pool,
                b"m_nested_malformed_type",
                kind_descriptor,
                vec![annotation_value(malformed_type, &[])],
            ),
            // `s`: the string literal is the one escaping rule this engine escapes strings with.
            abstract_member(pool, b"m_string", string_descriptor, vec![tag(b's', text)]),
            // `C`: one UTF-16 code unit, escaped by the rule every literal here is escaped with.
            abstract_member(pool, b"m_char", char_descriptor, vec![tag(b'C', newline)]),
            abstract_member(pool, b"m_quote", char_descriptor, vec![tag(b'C', quote)]),
            abstract_member(
                pool,
                b"m_backslash",
                char_descriptor,
                vec![tag(b'C', backslash)],
            ),
            abstract_member(pool, b"m_delete", char_descriptor, vec![tag(b'C', delete)]),
            abstract_member(
                pool,
                b"m_separator",
                char_descriptor,
                vec![tag(b'C', separator)],
            ),
            // `I` and `Z`: the number is the literal, and a `Z` that is neither `0` nor `1` is no
            // boolean this presentation can write.
            abstract_member(
                pool,
                b"m_negative",
                int_descriptor,
                vec![tag(b'I', minus_seven)],
            ),
            abstract_member(pool, b"m_false", boolean_descriptor, vec![tag(b'Z', zero)]),
            abstract_member(
                pool,
                b"m_contradiction",
                boolean_descriptor,
                vec![tag(b'Z', two)],
            ),
            // A member with a body: the attribute is the fact, so the default stands in front of the
            // block, and the declaration spelled after the run states the same one.
            DefaultMember {
                flags: 0x0009, // public static
                name: pool.utf8(b"m_body"),
                descriptor: int_descriptor,
                code: Some(vec![0x05, 0xac]), // iconst_2; ireturn
                defaults: vec![tag(b'I', seven)],
            },
            // A member that declares no attribute declares no default.
            abstract_member(pool, b"m_none", void_descriptor, Vec::new()),
        ]
    });
    let report = crafted_report(&bytes);

    // A class, not an annotation type: `default` is written because the member's own attribute states
    // one, and never because of the class's kind.
    assert!(
        report
            .text
            .contains("public class Probe extends java.lang.Object {\n"),
        "{}",
        report.text
    );
    let expected = [
        "public abstract java.lang.Class m_class() default java.lang.String.class;",
        "public abstract java.lang.Class m_array_class() default int[].class;",
        "public abstract int[] m_empty() default {};",
        "public abstract Kind m_nested() default @Kind(value = 7);",
        "public abstract Kind m_nested_empty() default @Kind();",
        "public abstract Kind m_nested_multi() default @Kind(value = 7, other = \"nested\");",
        "public abstract Kind[] m_nested_array() default {@Kind(value = 7), @Kind(value = 2)};",
        "public abstract java.lang.String m_string() default \"a\\\"b\\\\c\";",
        "public abstract char m_char() default '\\n';",
        "public abstract char m_quote() default '\\'';",
        "public abstract char m_backslash() default '\\\\';",
        "public abstract char m_delete() default '\\u007f';",
        "public abstract char m_separator() default '\\u2028';",
        "public abstract int m_negative() default -7;",
        "public abstract boolean m_false() default false;",
        "public static int m_body() default 7 {",
    ];
    for line in expected {
        assert!(
            report.text.contains(line),
            "the text states `{line}`:\n{}",
            report.text
        );
    }
    assert_eq!(
        declaration_of(&report, "m_body"),
        "public static int m_body() default 7"
    );
    assert!(
        report.text.contains("        return 2;\n"),
        "the member's body is presented beside its default:\n{}",
        report.text
    );

    // An unsupported descendant suppresses its whole array; malformed booleans keep their existing
    // all-or-nothing behavior, and a member with no attribute states none.
    for name in [
        "m_nested_unspellable",
        "m_nested_bad_type",
        "m_nested_malformed_type",
        "m_contradiction",
        "m_none",
    ] {
        let declaration = declaration_of(&report, name);
        assert!(
            !declaration.contains("default"),
            "`{name}` writes no default: `{declaration}`"
        );
    }
    assert_eq!(
        declaration_of(&report, "m_floats"),
        "public abstract float[] m_floats() default {0x1.000000p-1f}"
    );
    assert_eq!(
        declaration_of(&report, "m_nested_unspellable"),
        "public abstract Kind[] m_nested_unspellable()"
    );
    assert_eq!(
        declaration_of(&report, "m_nested_bad_type"),
        "public abstract Kind m_nested_bad_type()"
    );
    assert_eq!(
        declaration_of(&report, "m_nested_malformed_type"),
        "public abstract Kind m_nested_malformed_type()"
    );
    assert_eq!(
        declaration_of(&report, "m_none"),
        "public abstract void m_none()"
    );
    // Nothing else moved: the members that declare a body are still the two the class has, and the
    // pool the defaults were resolved against did not add a read.
    assert_eq!(report.usage.class_headers, 1, "{:?}", report.usage);
    assert_eq!(report.usage.method_bodies, 1, "{:?}", report.usage);
}

/// A member whose own `AnnotationDefault` does not read is stated as refused, and the members beside
/// it keep their declarations: one member's attribute failing is that member's failure and never a
/// shorter class.
#[test]
fn a_member_whose_attribute_does_not_read_is_refused_beside_the_members_that_do() {
    let bytes = crafted_class(|pool| {
        let int_descriptor = pool.utf8(b"()I");
        let seven = pool.integer(7);
        vec![
            // One unknown tag: the reader's own structured error, and this member's refusal.
            DefaultMember {
                flags: 0x0401, // public abstract
                name: pool.utf8(b"m_damaged"),
                descriptor: int_descriptor,
                code: None,
                defaults: vec![tag(b'Q', seven)],
            },
            // The control: the member beside it is spelled, default and all.
            abstract_member(pool, b"m_ok", int_descriptor, vec![tag(b'I', seven)]),
        ]
    });
    let report = crafted_report(&bytes);

    let damaged = member(&report, "m_damaged");
    let ClassSourceOutcome::Refused {
        execution,
        diagnostics,
    } = &damaged.outcome
    else {
        panic!("the member's attribute read failed: {damaged:?}");
    };
    assert!(!matches!(execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<&str>>(),
        ["classfile_invalid_attribute_content"]
    );
    // No default is invented for the member whose attribute did not read, and the refusal is written
    // where its body would stand.
    assert_eq!(
        damaged.declaration.as_deref(),
        Some("public abstract int m_damaged()")
    );
    assert!(!damaged.text.contains("default"), "{}", damaged.text);
    assert!(
        damaged
            .markers
            .iter()
            .any(|marker| marker.contains("refused")),
        "{:?}",
        damaged.markers
    );
    // The class is not `Complete`, and the member beside the refusal is still the declaration the
    // attribute states.
    assert!(
        !matches!(report.execution, ExecutionReport::Complete { .. }),
        "{:?}",
        report.execution
    );
    assert_eq!(
        declaration_of(&report, "m_ok"),
        "public abstract int m_ok() default 7"
    );
    assert!(
        report
            .text
            .contains("public abstract int m_ok() default 7;\n"),
        "{}",
        report.text
    );
}
