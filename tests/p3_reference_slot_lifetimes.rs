//! No-debug complete classes for disjoint ordinary-reference local lifetimes. The sixteen frozen
//! inputs cover eight source shapes from both javac 8 and javac 23; each pair must keep the same
//! two source types and the same recovery text under the essential and full evidence selections.

use jarde::*;
use std::slice;

macro_rules! case {
    ($jdk:ident, $class:ident, $first:literal, $second:literal) => {
        (
            stringify!($class),
            include_bytes!(concat!(
                "fixtures/p3-reference-slot-lifetimes/",
                stringify!($jdk),
                "/",
                stringify!($class),
                ".class"
            )),
            $first,
            $second,
        )
    };
}

macro_rules! debug_case {
    ($group:literal, $jdk:ident, $class:ident, $complete:literal) => {
        (
            stringify!($class),
            include_bytes!(concat!(
                "fixtures/p3-reference-slot-lifetimes/debug/",
                $group,
                "/",
                stringify!($jdk),
                "/",
                stringify!($class),
                ".class"
            )),
            include_str!(concat!(
                "fixtures/p3-reference-slot-lifetimes/debug/",
                $group,
                "/",
                stringify!($jdk),
                "/",
                stringify!($class),
                ".baseline.java"
            )),
            $complete,
        )
    };
}

macro_rules! negative_case {
    ($family:literal, $class:ident) => {
        (
            stringify!($class),
            include_bytes!(concat!(
                "fixtures/p3-reference-slot-lifetimes/negative/",
                $family,
                "/",
                stringify!($class),
                ".class"
            )),
            include_str!(concat!(
                "fixtures/p3-reference-slot-lifetimes/negative/",
                $family,
                "/",
                stringify!($class),
                ".baseline.java"
            )),
        )
    };
}

const CASES: &[(&str, &[u8], &str, &str)] = &[
    case!(javac8, ArrayThenList, "int[]", "java.util.ArrayList"),
    case!(javac23, ArrayThenList, "int[]", "java.util.ArrayList"),
    case!(
        javac8,
        DequeThenBuilder,
        "java.util.ArrayDeque",
        "java.lang.StringBuilder"
    ),
    case!(
        javac23,
        DequeThenBuilder,
        "java.util.ArrayDeque",
        "java.lang.StringBuilder"
    ),
    case!(
        javac8,
        StringThenArray,
        "java.lang.String",
        "java.lang.String[]"
    ),
    case!(
        javac23,
        StringThenArray,
        "java.lang.String",
        "java.lang.String[]"
    ),
    case!(
        javac8,
        StringThenBuilder,
        "java.lang.String",
        "java.lang.StringBuilder"
    ),
    case!(
        javac23,
        StringThenBuilder,
        "java.lang.String",
        "java.lang.StringBuilder"
    ),
    case!(
        javac8,
        BuilderThenArray,
        "java.lang.StringBuilder",
        "char[]"
    ),
    case!(
        javac23,
        BuilderThenArray,
        "java.lang.StringBuilder",
        "char[]"
    ),
    case!(
        javac8,
        ListThenMap,
        "java.util.ArrayList",
        "java.util.HashMap"
    ),
    case!(
        javac23,
        ListThenMap,
        "java.util.ArrayList",
        "java.util.HashMap"
    ),
    case!(
        javac8,
        MapThenBuilder,
        "java.util.HashMap",
        "java.lang.StringBuilder"
    ),
    case!(
        javac23,
        MapThenBuilder,
        "java.util.HashMap",
        "java.lang.StringBuilder"
    ),
    case!(
        javac8,
        StringThenQueue,
        "java.lang.String",
        "java.util.ArrayDeque"
    ),
    case!(
        javac23,
        StringThenQueue,
        "java.lang.String",
        "java.util.ArrayDeque"
    ),
];

const DIFFERENT_NAME_DEBUG: &[(&str, &[u8], &str, bool)] = &[
    debug_case!("different-name", javac8, ArrayThenList, true),
    debug_case!("different-name", javac8, StringThenBuilder, true),
    debug_case!("different-name", javac8, StringThenArray, true),
    debug_case!("different-name", javac8, DequeThenBuilder, true),
    debug_case!("different-name", javac23, ArrayThenList, true),
    debug_case!("different-name", javac23, StringThenBuilder, true),
    debug_case!("different-name", javac23, StringThenArray, true),
    debug_case!("different-name", javac23, DequeThenBuilder, true),
];

macro_rules! same_name_case {
    ($jdk:ident, $class:ident, $first:literal, $second:literal) => {
        (
            stringify!($class),
            include_bytes!(concat!(
                "fixtures/p3-reference-slot-lifetimes/debug/same-name/",
                stringify!($jdk),
                "/",
                stringify!($class),
                ".class"
            )),
            include_str!(concat!(
                "fixtures/p3-reference-slot-lifetimes/debug/same-name/",
                stringify!($jdk),
                "/",
                stringify!($class),
                ".baseline.java"
            )),
            $first,
            $second,
        )
    };
}

const SAME_NAME_DEBUG: &[(&str, &[u8], &str, &str, &str)] = &[
    same_name_case!(
        javac8,
        ListThenMap,
        "java.util.ArrayList",
        "java.util.HashMap"
    ),
    same_name_case!(
        javac8,
        MapThenBuilder,
        "java.util.HashMap",
        "java.lang.StringBuilder"
    ),
    same_name_case!(
        javac8,
        StringThenQueue,
        "java.lang.String",
        "java.util.ArrayDeque"
    ),
    same_name_case!(
        javac8,
        BuilderThenArray,
        "java.lang.StringBuilder",
        "char[]"
    ),
    same_name_case!(
        javac23,
        ListThenMap,
        "java.util.ArrayList",
        "java.util.HashMap"
    ),
    same_name_case!(
        javac23,
        MapThenBuilder,
        "java.util.HashMap",
        "java.lang.StringBuilder"
    ),
    same_name_case!(
        javac23,
        StringThenQueue,
        "java.lang.String",
        "java.util.ArrayDeque"
    ),
    same_name_case!(
        javac23,
        BuilderThenArray,
        "java.lang.StringBuilder",
        "char[]"
    ),
];

const NEGATIVE_CASES: &[(&str, &[u8], &str)] = &[
    negative_case!("same-type", V2),
    negative_case!("cross-phi", V3),
    negative_case!("cfg-loop-phi", V4),
    negative_case!("handler", SlotReuseBoundaries),
    negative_case!("parameter-header", ParameterHeader),
    negative_case!("held-use", HeldUse),
    negative_case!("cfg-backedge-disjoint", RefLoopBack),
];

const NONZERO_HELD_USE_CASES: &[(&[u8], &str)] = &[
    (
        include_bytes!(
            "fixtures/p3-reference-slot-lifetimes/negative/nonzero-held-use/javac8/NonzeroHeldUse.class"
        ),
        include_str!(
            "fixtures/p3-reference-slot-lifetimes/negative/nonzero-held-use/javac8/NonzeroHeldUse.baseline.java"
        ),
    ),
    (
        include_bytes!(
            "fixtures/p3-reference-slot-lifetimes/negative/nonzero-held-use/javac23/NonzeroHeldUse.class"
        ),
        include_str!(
            "fixtures/p3-reference-slot-lifetimes/negative/nonzero-held-use/javac23/NonzeroHeldUse.baseline.java"
        ),
    ),
];

fn opened(bytes: &[u8], name: &str) -> (ArtifactSnapshot, ClassSourceRequest) {
    let mut budget = task_budget(&[]).expect("bounded task budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("frozen Java 8 class opens");
    let request = ClassSourceRequest {
        source_target: jarde::SourceTarget::JAVA_8,
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
    (snapshot, request)
}

fn recover(bytes: &[u8], name: &str, evidence: &RecoveryEvidenceRequest) -> ClassSourceReport {
    let (snapshot, request) = opened(bytes, name);
    let mut budget = task_budget(&[]).expect("bounded task budget");
    match Engine::new()
        .class_source_with_evidence(slice::from_ref(&snapshot), &request, evidence, &mut budget)
        .expect("standalone class source request completes")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one standalone class resolves exactly: {other:?}"),
    }
}

fn declared_names(text: &str, ty: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let declaration = line.trim().strip_prefix(ty)?.trim_start();
            let name = declaration.split_whitespace().next()?;
            let name = name.trim_end_matches(';');
            (name.starts_with("local")
                && name[5..].chars().all(|ch| ch.is_ascii_digit() || ch == '_'))
            .then(|| name.to_owned())
        })
        .collect()
}

fn assert_pair(text: &str, first_type: &str, second_type: &str) {
    let first = declared_names(text, first_type);
    let second = declared_names(text, second_type);
    assert!(
        first
            .iter()
            .any(|name| second.contains(&format!("{name}_2"))),
        "the two types must be separate declarations for one reused slot:\n{text}"
    );
}

/// The one method whose newly proved category-two reuse is allowed to differ from this negative
/// class's frozen source. The four-space signature and method-closing boundary are specific to this
/// class; this deliberately is not a Java parser.
fn category2_adjacent_range(text: &str) -> std::ops::Range<usize> {
    let signature = "    public static long category2Adjacent() {";
    let start = text
        .find(signature)
        .expect("SlotReuseBoundaries has its frozen category2Adjacent signature");
    let close = text[start..]
        .find("\n    }\n")
        .map(|offset| start + offset + "\n    }".len())
        .expect("category2Adjacent has the four-space class-method closing brace");
    start..close
}

/// Compare all of the frozen class output except the one method whose wider-local split is newly
/// supported. Only the in-memory comparison is normalized; the baseline fixture remains untouched.
fn restore_frozen_category2_method(actual: &str, baseline: &str) -> String {
    let actual_range = category2_adjacent_range(actual);
    let baseline_range = category2_adjacent_range(baseline);
    format!(
        "{}{}{}",
        &actual[..actual_range.start],
        &baseline[baseline_range],
        &actual[actual_range.end..]
    )
}

fn assert_category2_adjacent_presentation(report: &ClassSourceReport) {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"category2Adjacent")
        .expect("category2Adjacent remains in the class method table");
    let ClassSourceOutcome::Recovered { report: method, .. } = &method.outcome else {
        panic!("category2Adjacent is recovered as a method body");
    };
    let text = &method.text;
    assert!(
        !text.contains("@bytecode") && !text.contains("jarde_refused_body"),
        "category2Adjacent no longer carries its previous fallback:\n{text}"
    );

    let arrays = declared_names(text, "int[]");
    let wide = declared_names(text, "long");
    let narrow = declared_names(text, "int");
    assert_eq!(
        arrays.len(),
        1,
        "the earlier reference lifetime is declared once:\n{text}"
    );
    assert_eq!(
        wide.len(),
        1,
        "the category-two lifetime is declared once:\n{text}"
    );
    assert_eq!(
        narrow.len(),
        1,
        "the later narrow lifetime is declared once:\n{text}"
    );
    assert_eq!(
        wide[0],
        format!("{}_2", arrays[0]),
        "the wide lifetime owns the reused slot name:\n{text}"
    );

    assert!(
        text.contains(&format!("if ({}[0] == 0)", arrays[0])),
        "the condition reads the array lifetime:\n{text}"
    );
    assert!(
        text.contains("return 0L;"),
        "the zero branch keeps its return:\n{text}"
    );
    assert!(
        text.contains(&format!("(int) {}", wide[0])),
        "the narrow local is initialized from the wide value:\n{text}"
    );
    assert!(
        text.contains(&format!("{} + (long) {}", wide[0], narrow[0])),
        "the return reads both the wide and narrow locals:\n{text}"
    );

    // Other methods in this class still contain frozen refusal markers; this is a method-level
    // recovery, not a claim that the complete negative class became Java or was compiled.
    assert!(
        report.text.contains("@bytecode"),
        "the complete class retains its unrelated partial methods"
    );
}

#[test]
fn all_sixteen_no_debug_classes_keep_each_reference_lifetime_separate() {
    for (name, bytes, first_type, second_type) in CASES {
        let essential = recover(bytes, name, &RecoveryEvidenceRequest::essential());
        let all = recover(bytes, name, &RecoveryEvidenceRequest::all());
        assert_eq!(
            essential.text, all.text,
            "{name}: essential and full evidence must publish the same source"
        );
        assert!(
            all.methods.iter().all(|method| method.markers.is_empty()),
            "{name} has an incomplete method: {:?}",
            all.methods
                .iter()
                .map(|method| (&method.item.name, &method.markers))
                .collect::<Vec<_>>()
        );
        assert!(
            !all.text.contains("@bytecode"),
            "{name} contains a fallback marker:\n{}",
            all.text
        );
        assert_pair(&all.text, first_type, second_type);
    }
}

#[test]
fn different_name_lvt_sources_remain_byte_for_byte_unchanged() {
    assert_eq!(DIFFERENT_NAME_DEBUG.len(), 8);
    for (name, bytes, baseline, complete) in DIFFERENT_NAME_DEBUG {
        assert!(*complete);
        let report = recover(bytes, name, &RecoveryEvidenceRequest::all());
        assert_eq!(report.text, *baseline, "{name}: debug source changed");
        assert!(
            !report.text.contains("@bytecode"),
            "{name}: this baseline row is a recovered debug case"
        );
    }
}

fn replace_once(source: &str, old: &str, new: &str) -> String {
    assert_eq!(
        source.matches(old).count(),
        1,
        "expected one frozen source segment: {old}"
    );
    source.replacen(old, new, 1)
}

fn assert_one_local_declaration_refinement(
    class_name: &str,
    actual: &str,
    baseline: &str,
    current_declaration: &str,
    old_declaration: &str,
) {
    assert_eq!(
        actual.matches(current_declaration).count(),
        1,
        "{class_name}: exactly one current local declaration changed"
    );
    assert_eq!(
        baseline.matches(old_declaration).count(),
        1,
        "{class_name}: exactly one corresponding old local declaration exists"
    );
    let restored = replace_once(actual, current_declaration, old_declaration);
    assert_eq!(
        restored, baseline,
        "{class_name}: all other source bytes match the frozen baseline"
    );
}

fn same_name_expected(name: &str, baseline: &str) -> String {
    match name {
        "ListThenMap" => {
            let expected = replace_once(
                baseline,
                "        v = new java.util.HashMap();",
                "        java.util.HashMap v_2 = new java.util.HashMap();",
            );
            let expected = replace_once(&expected, "        v.put(", "        v_2.put(");
            replace_once(
                &expected,
                "return v.size() + ((java.lang.Integer) v.get(",
                "return v_2.size() + ((java.lang.Integer) v_2.get(",
            )
        }
        "MapThenBuilder" => {
            let expected = replace_once(
                baseline,
                "        v = new java.lang.StringBuilder();",
                "        java.lang.StringBuilder v_2 = new java.lang.StringBuilder();",
            );
            let expected = replace_once(&expected, "        v.append(", "        v_2.append(");
            replace_once(&expected, "return v.toString();", "return v_2.toString();")
        }
        "StringThenQueue" => {
            let expected = replace_once(
                baseline,
                "        v = new java.util.ArrayDeque();",
                "        java.util.ArrayDeque v_2 = new java.util.ArrayDeque();",
            );
            let expected = replace_once(
                &expected,
                "        v.add((java.lang.Object) x);\n        v.add((java.lang.Object) \"!\");",
                "        v_2.add((java.lang.Object) x);\n        v_2.add((java.lang.Object) \"!\");",
            );
            replace_once(
                &expected,
                "(java.lang.String) v.removeFirst()).append((java.lang.String) v.removeLast())",
                "(java.lang.String) v_2.removeFirst()).append((java.lang.String) v_2.removeLast())",
            )
        }
        "BuilderThenArray" => {
            let expected = replace_once(
                baseline,
                "        java.lang.StringBuilder v;",
                "        char[] v_2;",
            );
            let expected = replace_once(
                &expected,
                "        v = new java.lang.StringBuilder(x);",
                "        java.lang.StringBuilder v = new java.lang.StringBuilder(x);",
            );
            let expected = replace_once(
                &expected,
                "        v = new char[n];",
                "        v_2 = new char[n];",
            );
            let expected = replace_once(
                &expected,
                "            v[i] = x.charAt(i);",
                "            v_2[i] = x.charAt(i);",
            );
            replace_once(
                &expected,
                "        // @bytecode 47\n        // the parameter 0 of the invocation at BCI 44 is declared `char[]` presents `java.lang.StringBuilder` but the invocation requires `char[]` and this layer has no safe reference conversion evidence",
                "        return new java.lang.String(v_2);",
            )
        }
        _ => panic!("no typed split expectation for {name}"),
    }
}

fn assert_three_method_evidence(
    essential: &ClassSourceReport,
    all: &ClassSourceReport,
    name: &str,
) {
    assert_eq!(essential.methods.len(), 3, "{name}: essential methods");
    assert_eq!(all.methods.len(), 3, "{name}: full-evidence methods");
    assert!(
        all.methods.iter().all(|method| method.markers.is_empty()),
        "{name}: {:?}",
        all.methods
    );
    for (left, right) in essential.methods.iter().zip(&all.methods) {
        assert_eq!(left.item, right.item, "{name}: member identity changed");
        let identity = left.item.identity.clone();
        let (
            ClassSourceOutcome::Recovered { report: left, .. },
            ClassSourceOutcome::Recovered { report: right, .. },
        ) = (&left.outcome, &right.outcome)
        else {
            panic!("{name}: all three methods remain recovered");
        };
        assert_eq!(
            left.text, right.text,
            "{name}: method source differs by evidence selection"
        );
        assert!(
            !right.source_map.segments().is_empty(),
            "{name}: method source map is present"
        );
        for segment in right.source_map.segments() {
            for origin in
                std::iter::once(segment.origin().primary()).chain(segment.origin().derived().iter())
            {
                assert_eq!(
                    origin.method(),
                    Some(&identity),
                    "{name}: source map crossed methods"
                );
            }
        }
    }
}

#[test]
fn same_name_lvt_sources_split_into_exact_typed_independent_names() {
    assert_eq!(SAME_NAME_DEBUG.len(), 8);
    for (name, bytes, baseline, first_type, second_type) in SAME_NAME_DEBUG {
        let expected = same_name_expected(name, baseline);
        if *name == "BuilderThenArray" {
            assert!(
                expected.contains("java.lang.StringBuilder v = new java.lang.StringBuilder(x);")
            );
            assert!(expected.contains("char[] v_2;"));
            assert!(expected.contains("return new java.lang.String(v_2);"));
        } else {
            let first_declaration = if *name == "StringThenQueue" {
                "java.lang.String v = x.trim();".to_owned()
            } else {
                format!("{first_type} v = new {first_type}();")
            };
            assert!(
                expected.contains(&first_declaration),
                "{name}: first typed declaration"
            );
            assert!(
                expected.contains(&format!("{second_type} v_2 = new {second_type}();")),
                "{name}: independent second typed declaration"
            );
        }
        let essential = recover(bytes, name, &RecoveryEvidenceRequest::essential());
        let all = recover(bytes, name, &RecoveryEvidenceRequest::all());
        assert_eq!(
            essential.text, expected,
            "{name}: essential full-class source"
        );
        assert_eq!(all.text, expected, "{name}: all-evidence full-class source");
        assert_three_method_evidence(&essential, &all, name);
    }
}

#[test]
fn unsupported_reference_boundaries_keep_the_cli9_source_verbatim() {
    assert_eq!(NEGATIVE_CASES.len(), 7);
    for (name, bytes, baseline) in NEGATIVE_CASES {
        let report = recover(bytes, name, &RecoveryEvidenceRequest::all());
        if *name == "SlotReuseBoundaries" {
            assert_category2_adjacent_presentation(&report);
            let normalized = restore_frozen_category2_method(&report.text, baseline);
            assert_eq!(
                normalized, *baseline,
                "{name}: every byte outside category2Adjacent stays frozen"
            );
            continue;
        }
        if matches!(*name, "V3" | "V4") {
            assert_one_local_declaration_refinement(
                name,
                &report.text,
                baseline,
                "        java.lang.Object local1;\n",
                "        int[] local1;\n",
            );
            continue;
        }
        if *name == "HeldUse" {
            assert_one_local_declaration_refinement(
                name,
                &report.text,
                baseline,
                "        java.lang.Object local0 = new java.lang.StringBuilder(\"old\");\n",
                "        java.lang.StringBuilder local0 = new java.lang.StringBuilder(\"old\");\n",
            );
            continue;
        }
        if *name == "RefLoopBack" {
            assert!(
                report
                    .methods
                    .iter()
                    .all(|method| method.markers.is_empty())
            );
            for method in &report.methods {
                assert!(matches!(
                    method.outcome,
                    ClassSourceOutcome::Recovered { .. }
                ));
            }
            let current_lifetime = concat!(
                "            java.lang.StringBuilder local2_2 = new java.lang.StringBuilder(\"second\");\n",
                "            local1 = local1 + local2_2.length();\n",
            );
            let old_lifetime = concat!(
                "            local2 = new java.lang.StringBuilder(\"second\");\n",
                "            local1 = local1 + local2.length();\n",
            );
            assert_eq!(report.text.matches(current_lifetime).count(), 1);
            assert_eq!(baseline.matches(old_lifetime).count(), 1);
            let restored = replace_once(&report.text, current_lifetime, old_lifetime);
            assert_eq!(
                restored, *baseline,
                "{name}: only the proved RefLoopBack split changed"
            );
            continue;
        }
        assert_eq!(
            report.text, *baseline,
            "{name}: unsupported-shape output changed"
        );
    }
}

/// Preserve the frozen class baseline except for its exact local-type refinement and proved
/// compound condition; whole-class identity, source maps, and all unrelated members stay fixed.
#[test]
fn null_first_exact_builder_writes_refine_local_type_and_compound_condition() {
    let (name, bytes, baseline) = negative_case!("unknown-null", NullThenBuilder);
    assert_eq!(baseline.matches("Object local2;").count(), 1);
    let nested_if = concat!(
        "        if (arg0) {\n",
        "            if (local2 == null) {\n",
        "                local1 = local1 + 1;\n",
        "            }\n",
        "        }"
    );
    let compound_if = concat!(
        "        if (arg0 && local2 == null) {\n",
        "            local1 = local1 + 1;\n",
        "        }"
    );
    assert_eq!(baseline.matches(nested_if).count(), 1);
    let typed_expected = baseline.replace("Object local2;", "java.lang.StringBuilder local2;");
    let expected = typed_expected.replace(nested_if, compound_if);
    let essential = recover(bytes, name, &RecoveryEvidenceRequest::essential());
    let all = recover(bytes, name, &RecoveryEvidenceRequest::all());
    assert_eq!(essential.text, expected);
    assert_eq!(all.text, expected);
    assert_eq!(essential.methods.len(), 3);
    assert_eq!(all.methods.len(), 3);
    assert!(all.methods.iter().all(|method| method.markers.is_empty()));
    for (left, right) in essential.methods.iter().zip(&all.methods) {
        assert_eq!(left.item, right.item);
        let identity = left.item.identity.clone();
        let (
            ClassSourceOutcome::Recovered { report: left, .. },
            ClassSourceOutcome::Recovered { report: right, .. },
        ) = (&left.outcome, &right.outcome)
        else {
            panic!("whole-class member recovery remains present");
        };
        assert_eq!(left.text, right.text);
        assert!(!right.source_map.segments().is_empty());
        for segment in right.source_map.segments() {
            for origin in
                std::iter::once(segment.origin().primary()).chain(segment.origin().derived().iter())
            {
                assert_eq!(origin.method(), Some(&identity));
            }
        }
    }
}

#[test]
fn held_reference_across_a_local_overwrite_is_not_published_as_a_slot_alias() {
    let (name, bytes, baseline) = NEGATIVE_CASES
        .iter()
        .find(|(name, _, _)| *name == "HeldUse")
        .expect("the frozen HeldUse negative is present");
    let report = recover(bytes, name, &RecoveryEvidenceRequest::all());
    assert_one_local_declaration_refinement(
        name,
        &report.text,
        baseline,
        "        java.lang.Object local0 = new java.lang.StringBuilder(\"old\");\n",
        "        java.lang.StringBuilder local0 = new java.lang.StringBuilder(\"old\");\n",
    );
    assert!(
        report
            .text
            .contains("the value at BCI 23 is the value local 0 held at BCI 10"),
        "HeldUse still publishes the held-value refusal"
    );
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"run")
        .expect("HeldUse.run is present");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("HeldUse.run recovery is present: {:?}", method.outcome);
    };
    assert!(
        report.aliased_names.is_empty(),
        "an old reference still held on the operand stack cannot alias the reused local: {:?}",
        report.aliased_names
    );
    assert!(
        !report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_name_aliased"),
        "HeldUse.run does not publish a name-alias diagnostic: {:?}",
        report.diagnostics
    );
}

#[test]
fn nonzero_stack_reference_forwarding_keeps_both_jdk_sources_and_metadata_unchanged() {
    for (bytes, baseline) in NONZERO_HELD_USE_CASES {
        let report = recover(bytes, "NonzeroHeldUse", &RecoveryEvidenceRequest::all());
        assert_one_local_declaration_refinement(
            "NonzeroHeldUse",
            &report.text,
            baseline,
            "        java.lang.Object local0 = new java.lang.StringBuilder(\"old\");\n",
            "        java.lang.StringBuilder local0 = new java.lang.StringBuilder(\"old\");\n",
        );
        assert!(
            report.text.contains("// @bytecode 28 25 13 12"),
            "NonzeroHeldUse still publishes its unresolved forwarding refusal"
        );
        let method = report
            .methods
            .iter()
            .find(|method| {
                method.item.name.raw().0 == b"run"
                    && method.item.descriptor.raw().0 == b"()Ljava/lang/Object;"
            })
            .expect("NonzeroHeldUse.run is present");
        let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
            panic!(
                "NonzeroHeldUse.run recovery is present: {:?}",
                method.outcome
            );
        };
        assert!(
            report.aliased_names.is_empty(),
            "a reference checked on a nonzero stack slot cannot alias the reused local: {:?}",
            report.aliased_names
        );
        assert!(
            !report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "jre_name_aliased"),
            "NonzeroHeldUse.run does not publish a name-alias diagnostic: {:?}",
            report.diagnostics
        );
    }
}

#[test]
fn low_analysis_budget_and_cancellation_do_not_publish_a_split_source() {
    let (snapshot, request) = opened(CASES[0].1, CASES[0].0);
    let complete = recover(CASES[0].1, CASES[0].0, &RecoveryEvidenceRequest::all());

    let mut low_budget =
        task_budget(&[BudgetOverride::new("analysis_steps", 1).expect("valid analysis-step cap")])
            .expect("the bounded task budget accepts an analysis-step cap");
    match Engine::new().class_source_with_evidence(
        slice::from_ref(&snapshot),
        &request,
        &RecoveryEvidenceRequest::all(),
        &mut low_budget,
    ) {
        Ok(OperationOutcome::Performed(report)) => {
            assert!(!report.text.is_empty(), "a completed response is not empty");
            assert_ne!(
                report.text, complete.text,
                "a stopped run published full output"
            );
            assert!(
                !report.text.contains("java.util.ArrayList local2_2"),
                "a stopped run published the later segment's identity"
            );
        }
        Ok(_) => {}
        other => panic!("low-budget request returns its ordinary stop result: {other:?}"),
    }

    let token = CancellationToken::new();
    token.cancel();
    let limits = task_budget(&[])
        .expect("bounded task budget")
        .limits()
        .clone();
    let mut cancelled = Budget::with_cancellation_token(limits, token);
    match Engine::new().class_source_with_evidence(
        slice::from_ref(&snapshot),
        &request,
        &RecoveryEvidenceRequest::all(),
        &mut cancelled,
    ) {
        Ok(OperationOutcome::Performed(report)) => {
            assert!(!report.text.is_empty(), "a completed response is not empty");
            assert_ne!(
                report.text, complete.text,
                "a cancelled run published full output"
            );
            assert!(
                !report.text.contains("java.util.ArrayList local2_2"),
                "a cancelled run published the later segment's identity"
            );
        }
        Ok(_) => {}
        other => panic!("cancelled request returns its ordinary stop result: {other:?}"),
    }
}
