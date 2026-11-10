//! Permanent class-source regression for typed local lifetimes in the frozen S2a dev inputs.
//!
//! The jars cover Corretto 8 and OpenJDK 23 classfiles, with and without debug metadata. The
//! sequential method exercises four different JVM categories in reused local storage. The branch
//! method pins the currently visible lexical proof for two mutually exclusive same-type locals and
//! their hoisted merge target without relying on source variable spellings or bytecode offsets.

use jarde::*;
use std::slice;

const JARS: [(&str, &[u8]); 4] = [
    (
        "Corretto 8 with debug metadata",
        include_bytes!("fixtures/s2a-variable-lifetimes/corretto8-debug.jar"),
    ),
    (
        "Corretto 8 without debug metadata",
        include_bytes!("fixtures/s2a-variable-lifetimes/corretto8-no-debug.jar"),
    ),
    (
        "OpenJDK 23 with debug metadata",
        include_bytes!("fixtures/s2a-variable-lifetimes/openjdk23-debug.jar"),
    ),
    (
        "OpenJDK 23 without debug metadata",
        include_bytes!("fixtures/s2a-variable-lifetimes/openjdk23-no-debug.jar"),
    ),
];

const HANDLER_COPIES_JARS: [(&str, &[u8]); 4] = [
    (
        "Corretto 8 with debug metadata",
        include_bytes!("fixtures/s2a-variable-lifetimes/handler-copies/jars/corretto8-debug.jar"),
    ),
    (
        "Corretto 8 without debug metadata",
        include_bytes!(
            "fixtures/s2a-variable-lifetimes/handler-copies/jars/corretto8-no-debug.jar"
        ),
    ),
    (
        "OpenJDK 23 with debug metadata",
        include_bytes!("fixtures/s2a-variable-lifetimes/handler-copies/jars/openjdk23-debug.jar"),
    ),
    (
        "OpenJDK 23 without debug metadata",
        include_bytes!(
            "fixtures/s2a-variable-lifetimes/handler-copies/jars/openjdk23-no-debug.jar"
        ),
    ),
];

const HANDLER_OPERATORS_JARS: [(&str, &[u8]); 4] = [
    (
        "Corretto 8 with debug metadata",
        include_bytes!(
            "fixtures/s2a-variable-lifetimes/handler-operators/jars/corretto8-debug.jar"
        ),
    ),
    (
        "Corretto 8 without debug metadata",
        include_bytes!(
            "fixtures/s2a-variable-lifetimes/handler-operators/jars/corretto8-no-debug.jar"
        ),
    ),
    (
        "OpenJDK 23 with debug metadata",
        include_bytes!(
            "fixtures/s2a-variable-lifetimes/handler-operators/jars/openjdk23-debug.jar"
        ),
    ),
    (
        "OpenJDK 23 without debug metadata",
        include_bytes!(
            "fixtures/s2a-variable-lifetimes/handler-operators/jars/openjdk23-no-debug.jar"
        ),
    ),
];

fn budget() -> Budget {
    task_budget(&[]).expect("task defaults provide a bounded budget")
}

fn source(bytes: &[u8]) -> ClassSourceReport {
    named_source(bytes, "DevCases")
}

fn named_source(bytes: &[u8], class_name: &str) -> ClassSourceReport {
    named_source_with_policy(bytes, class_name, EnvironmentPolicy::PlainJar)
}

fn named_source_with_policy(
    bytes: &[u8],
    class_name: &str,
    policy: EnvironmentPolicy,
) -> ClassSourceReport {
    let engine = Engine::new();
    let mut open_budget = budget();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut open_budget)
        .expect("the frozen plain jar opens");
    let request = ClassSourceRequest {
        source_target: jarde::SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class_name),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    match engine
        .class_source(slice::from_ref(&snapshot), &request, &mut budget())
        .expect("the plain-jar class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("the frozen jar binds one {class_name} definition: {other:?}"),
    }
}

fn method_text<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("DevCases has no method `{name}`"));
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!(
            "`{name}` did not produce a recovered body: {:?}",
            method.outcome
        );
    };
    &report.text
}

fn parameter_name(report: &ClassSourceReport, method_name: &str, index: usize) -> String {
    let declaration = report
        .text
        .lines()
        .find(|line| {
            let line = line.trim_start();
            line.starts_with("public static ")
                && line
                    .split_once('(')
                    .and_then(|(head, _)| head.split_whitespace().last())
                    == Some(method_name)
        })
        .unwrap_or_else(|| panic!("missing declaration for `{method_name}`:\n{}", report.text));
    let parameters = declaration
        .split_once('(')
        .and_then(|(_, tail)| tail.split_once(')'))
        .map(|(parameters, _)| parameters)
        .expect("method declaration has parameter parentheses");
    parameters
        .split(',')
        .nth(index)
        .and_then(|parameter| parameter.split_whitespace().last())
        .unwrap_or_else(|| panic!("missing parameter {index} for `{method_name}`:\n{declaration}"))
        .to_owned()
}

fn declaration_name(line: &str, type_name: &str) -> Option<String> {
    let line = line.trim();
    let tail = line.strip_prefix(type_name)?.strip_prefix(' ')?;
    let before_initializer = tail.split('=').next()?.trim();
    let name = before_initializer
        .strip_suffix(';')
        .unwrap_or(before_initializer)
        .split_whitespace()
        .last()?;
    if name.is_empty() {
        None
    } else {
        Some(name.to_owned())
    }
}

fn typed_declarations<'a>(text: &'a str, type_names: &[&str]) -> Vec<(usize, String, bool)> {
    text.lines()
        .enumerate()
        .filter_map(|(index, line)| {
            type_names.iter().find_map(|type_name| {
                let name = declaration_name(line, type_name)?;
                Some((index, name, line.contains('=')))
            })
        })
        .collect()
}

fn contains_word(text: &str, word: &str) -> bool {
    text.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .any(|part| part == word)
}

fn word_count(text: &str, word: &str) -> usize {
    text.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
        .filter(|part| *part == word)
        .count()
}

#[test]
fn sequential_reused_slot_has_four_independent_typed_variables() {
    for (label, bytes) in JARS {
        let report = source(bytes);
        let text = method_text(&report, "sequentialSlotLifetimes");
        assert!(
            !text.contains("@bytecode") && !text.contains("jarde_refused_body"),
            "{label}: the sequential lifetime method has no fallback marker:\n{text}"
        );

        let int_locals = typed_declarations(text, &["int"]);
        assert_eq!(
            int_locals.len(),
            2,
            "{label}: accumulator and reused-slot int are declared:\n{text}"
        );
        let long_locals = typed_declarations(text, &["long"]);
        let double_locals = typed_declarations(text, &["double"]);
        let string_locals = typed_declarations(text, &["java.lang.String", "String"]);
        for (type_name, typed) in [
            ("long", &long_locals),
            ("double", &double_locals),
            ("String", &string_locals),
        ] {
            assert_eq!(
                typed.len(),
                1,
                "{label}: expected one {type_name} lifetime declaration:\n{text}"
            );
        }
        let names = vec![
            int_locals[1].1.clone(),
            long_locals[0].1.clone(),
            double_locals[0].1.clone(),
            string_locals[0].1.clone(),
        ];
        for name in &names {
            assert!(
                word_count(text, name) >= 2,
                "{label}: declared local `{name}` is also read by its own method:\n{text}"
            );
        }
        let distinct: std::collections::BTreeSet<_> = names.iter().collect();
        assert_eq!(
            distinct.len(),
            4,
            "{label}: the four lifetimes have distinct source variables:\n{text}"
        );
    }
}

#[test]
fn same_type_branch_locals_stay_inside_arms_and_merge_target_stays_hoisted() {
    for (label, bytes) in JARS {
        let report = source(bytes);
        let text = method_text(&report, "branchLocalReferences");
        let declarations = typed_declarations(text, &["java.lang.String", "String"]);
        assert_eq!(
            declarations.len(),
            3,
            "{label}: two arm locals and one merge target are declared:\n{text}"
        );

        let branch = text
            .find("if (")
            .expect("branch method presents its condition");
        let otherwise = text[branch..]
            .find("else")
            .map(|offset| branch + offset)
            .expect("branch method presents its else arm");
        let hoisted: Vec<_> = declarations
            .iter()
            .filter(|(line, _, initialized)| !initialized && *line < text[..branch].lines().count())
            .collect();
        assert_eq!(
            hoisted.len(),
            1,
            "{label}: merge target is declared before the branch:\n{text}"
        );
        let merge_name = &hoisted[0].1;

        let line_start = |line_index: usize| {
            text.lines()
                .take(line_index)
                .map(|line| line.len() + 1)
                .sum::<usize>()
        };
        let then_local = declarations
            .iter()
            .find(|(line, _, initialized)| {
                *initialized && line_start(*line) > branch && line_start(*line) < otherwise
            })
            .expect("the first arm declares its own source local");
        let else_local = declarations
            .iter()
            .find(|(line, _, initialized)| *initialized && line_start(*line) > otherwise)
            .expect("the second arm declares its own source local");
        let then_arm = &text[branch..otherwise];
        let else_arm = &text[otherwise..];
        assert!(
            contains_word(then_arm, &then_local.1),
            "{label}: first local remains in its branch:\n{text}"
        );
        assert!(
            contains_word(else_arm, &else_local.1),
            "{label}: second local remains in its branch:\n{text}"
        );
        assert!(
            then_arm.contains(&format!("{merge_name} = {}", then_local.1)),
            "{label}: first arm assigns the hoisted merge target:\n{text}"
        );
        assert!(
            else_arm.contains(&format!("{merge_name} = {}", else_local.1)),
            "{label}: second arm assigns the hoisted merge target:\n{text}"
        );
    }
}

#[test]
fn handler_protected_inputs_keep_pre_try_initializers_structured() {
    for (label, bytes) in HANDLER_COPIES_JARS {
        let report = named_source(bytes, "HandlerCopies");
        for method_name in ["copy", "initializedByCall"] {
            let method = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == method_name.as_bytes())
                .unwrap_or_else(|| panic!("{label}: missing {method_name}"));
            let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
                panic!(
                    "{label}: {method_name} has no recovered body: {:?}",
                    method.outcome
                );
            };
            assert_eq!(
                body.quality,
                jarde_jvm::ir::Quality::Structured,
                "{label}: {method_name}: {}",
                body.text
            );
            assert!(
                method.markers.is_empty(),
                "{label}: {method_name}: {:?}",
                method.markers
            );

            let lines: Vec<_> = body.text.lines().collect();
            let try_line = lines
                .iter()
                .position(|line| line.trim() == "try {")
                .unwrap_or_else(|| {
                    panic!("{label}: {method_name} has no try block:\n{}", body.text)
                });
            let saved_declarations: Vec<_> = lines[..try_line]
                .iter()
                .enumerate()
                .filter_map(|(index, line)| {
                    let name = declaration_name(line, "int")?;
                    (!line.contains('=')).then_some((index, name))
                })
                .collect();
            assert_eq!(
                saved_declarations.len(),
                1,
                "{label}: {method_name} declares one uninitialized int before try:\n{}",
                body.text
            );
            let (declaration_line, saved) = &saved_declarations[0];
            let initializer = if method_name == "copy" {
                parameter_name(&report, method_name, 1)
            } else {
                "java.lang.Integer.parseInt(\"6\")".to_owned()
            };
            let initialization = format!("{saved} = {initializer};");
            let initialization_line = lines[..try_line]
                .iter()
                .position(|line| line.trim() == initialization)
                .unwrap_or_else(|| {
                    panic!(
                        "{label}: {method_name} initializes `{saved}` from `{initializer}` before try:\n{}",
                        body.text
                    )
                });
            assert!(
                *declaration_line < initialization_line && initialization_line < try_line,
                "{label}: {method_name} declaration and real initializer both precede try:\n{}",
                body.text
            );
        }
    }
}

#[test]
fn handler_pre_try_primitive_operators_and_exception_ownership_are_preserved() {
    for (label, bytes) in HANDLER_OPERATORS_JARS {
        let report = named_source(bytes, "HandlerOperators");
        for (method_name, local_type) in [
            ("intOperators", "int"),
            ("longOperators", "long"),
            ("converted", "int"),
            ("quotientBeforeTry", "int"),
        ] {
            let method = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == method_name.as_bytes())
                .unwrap_or_else(|| panic!("{label}: missing {method_name}"));
            let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
                panic!(
                    "{label}: {method_name} has no recovered body: {:?}",
                    method.outcome
                );
            };
            assert_eq!(
                body.quality,
                jarde_jvm::ir::Quality::Structured,
                "{label}: {method_name}: {}",
                body.text
            );
            assert!(
                method.markers.is_empty(),
                "{label}: {method_name}: {:?}",
                method.markers
            );

            let lines: Vec<_> = body.text.lines().collect();
            let try_line = lines
                .iter()
                .position(|line| line.trim() == "try {")
                .unwrap_or_else(|| {
                    panic!("{label}: {method_name} has no try block:\n{}", body.text)
                });
            let uninitialized: Vec<_> = typed_declarations(&body.text, &[local_type])
                .into_iter()
                .filter(|(_, _, initialized)| !initialized)
                .collect();
            assert_eq!(
                uninitialized.len(),
                1,
                "{label}: {method_name} has one explicitly assigned {local_type} local, not a default initializer:\n{}",
                body.text
            );
            let (declaration_line, saved, _) = &uninitialized[0];
            assert!(
                *declaration_line < try_line,
                "{label}: declaration precedes try:\n{}",
                body.text
            );
            let initializer_line = lines
                .iter()
                .enumerate()
                .find(|(index, line)| {
                    *index > *declaration_line
                        && *index < try_line
                        && line.trim().starts_with(&format!("{saved} = "))
                })
                .map(|(index, _)| index)
                .unwrap_or_else(|| {
                    panic!(
                        "{label}: {method_name} assigns `{saved}` before try:\n{}",
                        body.text
                    )
                });
            assert!(*declaration_line < initializer_line && initializer_line < try_line);

            let initializer = lines[initializer_line].trim();
            assert!(
                !initializer.ends_with("= 0;") && !initializer.ends_with("= 0L;"),
                "{label}: {method_name} retains a real initializer:\n{}",
                body.text
            );
            match method_name {
                "intOperators" | "longOperators" => {
                    for operator in [" & ", " | ", " ^ ", " << ", " >> ", " >>> "] {
                        assert!(
                            initializer.contains(operator),
                            "{label}: {method_name} keeps `{operator}` in pre-try computation:\n{}",
                            body.text
                        );
                    }
                    let right = parameter_name(&report, method_name, 1);
                    assert!(
                        body.text.contains(&format!("return -{saved};")),
                        "{label}: {}",
                        body.text
                    );
                    assert!(
                        body.text.contains(&format!("return -{saved} + {right};")),
                        "{label}: negation and catch update remain in their arms:\n{}",
                        body.text
                    );
                }
                "converted" => assert!(
                    initializer.contains("= (int) "),
                    "{label}: primitive narrowing is the actual pre-try initializer:\n{}",
                    body.text
                ),
                "quotientBeforeTry" => {
                    assert!(
                        initializer.contains(" / "),
                        "{label}: division remains before try:\n{}",
                        body.text
                    );
                    assert!(
                        body.text
                            .contains("catch (java.lang.IllegalArgumentException "),
                        "{label}: the original method catch remains typed to IllegalArgumentException:\n{}",
                        body.text
                    );
                }
                _ => unreachable!(),
            }
        }
    }
}

fn assert_source_map_method_owner(body: &RecoveryReport, name: &[u8], descriptor: &[u8]) {
    let segments = body.source_map.segments();
    if segments.is_empty() {
        return;
    }
    let mut owned_origins = 0;
    for segment in segments {
        let origin = segment.origin();
        for anchor in std::iter::once(origin.primary()).chain(origin.derived()) {
            if let Some(method) = anchor.method() {
                owned_origins += 1;
                assert_eq!(method.name.0, name);
                assert_eq!(method.descriptor.0, descriptor);
            }
        }
    }
    assert!(
        owned_origins > 0,
        "nonempty source map has method-owned BCI origins"
    );
}

#[test]
fn unspellable_call_name_quotes_only_its_dependent_method() {
    let report = named_source_with_policy(
        include_bytes!("fixtures/s2a-variable-lifetimes/handler-copies/invalid-call-name.class"),
        "HandlerCopies",
        EnvironmentPolicy::SingleClass,
    );

    let copy = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"copy")
        .expect("copy control exists");
    let ClassSourceOutcome::Recovered {
        report: copy_body, ..
    } = &copy.outcome
    else {
        panic!("copy control has no recovered body: {:?}", copy.outcome);
    };
    assert_eq!(
        copy_body.quality,
        jarde_jvm::ir::Quality::Structured,
        "{}",
        copy_body.text
    );
    assert!(copy.markers.is_empty(), "{:?}", copy.markers);
    assert_source_map_method_owner(copy_body, b"copy", b"(ZI)I");

    let initialized = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"initializedByCall")
        .expect("invalid call control exists");
    let ClassSourceOutcome::Recovered {
        report: initialized_body,
        ..
    } = &initialized.outcome
    else {
        panic!(
            "the refused dependency still has a report: {:?}",
            initialized.outcome
        );
    };
    assert_eq!(
        initialized_body.quality,
        jarde_jvm::ir::Quality::Fallback,
        "{}",
        initialized_body.text
    );
    assert_eq!(initialized_body.content, RecoveryContent::ExplanationOnly);
    assert!(
        !initialized.markers.is_empty(),
        "the failed method carries its refusal marker"
    );
    assert!(
        initialized_body.text.contains("required write"),
        "{}",
        initialized_body.text
    );
    assert!(
        initialized_body
            .text
            .contains("declarations, reads and dependent effects are quoted together"),
        "{}",
        initialized_body.text
    );
    assert!(
        initialized_body.text.lines().all(|line| {
            let line = line.trim();
            line.is_empty() || line == "{" || line == "}" || line.starts_with("//")
        }),
        "the rejected whole method leaves no hoisted or uninitialized Java local:\n{}",
        initialized_body.text
    );
    assert!(typed_declarations(&initialized_body.text, &["int", "long"]).is_empty());
    assert_source_map_method_owner(initialized_body, b"initializedByCall", b"(ZI)I");
}
