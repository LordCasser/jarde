//! Controls for source-level declared assignability and type-parameter bounds.
//! The parent test module supplies the already pinned physical-header parser/provider helpers.

use super::*;
use jarde_java::generic_types::{declared_assignable, declared_type_bounds};
use jarde_reader::budget::{Budget, CancellationToken};

const BOUNDED: &[u8] =
    include_bytes!("../fixtures/s2b-declared-bounds-r1/classes/bounds/Bounded.class");
const SELF_BOUND: &[u8] =
    include_bytes!("../fixtures/s2b-declared-bounds-r1/classes/bounds/SelfBound.class");
const SELF_CHILD: &[u8] =
    include_bytes!("../fixtures/s2b-declared-bounds-r1/classes/bounds/SelfChild.class");

fn headers_with_bounds() -> Headers {
    let mut headers = Headers::full();
    headers.insert(
        CHILD_LOADER,
        b"bounds/Bounded",
        class_header(BOUNDED, CHILD_LOADER),
    );
    headers.insert(
        CHILD_LOADER,
        b"bounds/SelfBound",
        class_header(SELF_BOUND, CHILD_LOADER),
    );
    headers.insert(
        CHILD_LOADER,
        b"bounds/SelfChild",
        class_header(SELF_CHILD, CHILD_LOADER),
    );
    let list = headers.get(BOOT_LOADER, b"java/util/List").clone();
    headers.insert(CHILD_LOADER, b"java/util/List", list);
    headers
}

fn bind(
    types: &mut DeclaredGenericTypes,
    syntax: &[u8],
    loader: &str,
    headers: &mut Headers,
    budget: &mut Budget,
) -> usize {
    match jarde_java::generic_types::bind_declared_type(
        types,
        &field_type(syntax),
        &LoaderId(loader.to_owned()),
        &[],
        headers,
        budget,
    )
    .expect("binding propagates reader errors")
    {
        DependencyResult::Found(node) => node,
        DependencyResult::Missing => panic!("expected a bound type, got Missing"),
        DependencyResult::Ambiguous => panic!("expected a bound type, got Ambiguous"),
        DependencyResult::Unknown => panic!("expected a bound type, got Unknown"),
    }
}

fn expect_error<T>(result: std::result::Result<T, Error>) -> Error {
    match result {
        Err(error) => error,
        Ok(_) => panic!("expected a budget or cancellation error"),
    }
}

#[test]
fn generic_assignability_preserves_argument_identity_and_raw_direction() {
    let mut headers = headers_with_bounds();
    let mut types = DeclaredGenericTypes::default();
    let mut budget = Budget::new(limits());
    let child_list = bind(
        &mut types,
        b"Ljava/util/List<Lidentity/Foo;>;",
        CHILD_LOADER,
        &mut headers,
        &mut budget,
    );
    let parent_list = bind(
        &mut types,
        b"Ljava/util/List<Lidentity/Foo;>;",
        PARENT_LOADER,
        &mut headers,
        &mut budget,
    );
    assert_ne!(child_list, parent_list);
    let (
        DeclaredGenericType::Reference {
            declaration: child_decl,
            arguments: child_args,
        },
        DeclaredGenericType::Reference {
            declaration: parent_decl,
            arguments: parent_args,
        },
    ) = (&types.types[child_list], &types.types[parent_list])
    else {
        panic!("both List applications are declared references");
    };
    assert_eq!(child_decl.member, parent_decl.member);
    assert_eq!(child_args.len(), 1);
    assert_eq!(parent_args.len(), 1);
    assert_eq!(
        child_decl.definition, parent_decl.definition,
        "both use the same physical List header"
    );
    let (
        DeclaredGenericType::Reference {
            declaration: child_foo,
            ..
        },
        DeclaredGenericType::Reference {
            declaration: parent_foo,
            ..
        },
    ) = (&types.types[child_args[0]], &types.types[parent_args[0]])
    else {
        panic!("both exact List arguments are Foo references");
    };
    assert_ne!(
        child_foo.definition, parent_foo.definition,
        "same binary name Foo resolves to distinct physical definitions by loader"
    );
    assert!(
        matches!(
            declared_assignable(
                &mut types,
                child_list,
                parent_list,
                &mut headers,
                &mut budget
            )
            .unwrap(),
            DependencyResult::Found(false)
        ),
        "List<FooChild> is not assignable to List<FooParent>, despite identical binary names"
    );

    let parameterized = bind(
        &mut types,
        b"Ljava/util/List<Lidentity/Foo;>;",
        CHILD_LOADER,
        &mut headers,
        &mut budget,
    );
    let raw = bind(
        &mut types,
        b"Ljava/util/List;",
        CHILD_LOADER,
        &mut headers,
        &mut budget,
    );
    assert!(
        matches!(
            declared_assignable(&mut types, raw, parameterized, &mut headers, &mut budget).unwrap(),
            DependencyResult::Found(false)
        ),
        "a raw source does not prove a parameterized target"
    );
    assert!(
        matches!(
            declared_assignable(&mut types, parameterized, raw, &mut headers, &mut budget).unwrap(),
            DependencyResult::Found(true)
        ),
        "a parameterized source may flow to the same raw declaration"
    );
}

#[test]
fn class_bounds_and_f_bound_are_checked_without_confusing_recursive_bounds_with_cycles() {
    let mut headers = headers_with_bounds();
    let mut types = DeclaredGenericTypes::default();
    let mut budget = Budget::new(limits());
    let bounded_string = bind(
        &mut types,
        b"Lbounds/Bounded<Ljava/lang/String;>;",
        CHILD_LOADER,
        &mut headers,
        &mut budget,
    );
    assert!(matches!(
        declared_type_bounds(&mut types, bounded_string, &mut headers, &mut budget).unwrap(),
        DependencyResult::Found(true)
    ));

    let bounded_foo = bind(
        &mut types,
        b"Lbounds/Bounded<Lidentity/Foo;>;",
        CHILD_LOADER,
        &mut headers,
        &mut budget,
    );
    assert!(
        matches!(
            declared_type_bounds(&mut types, bounded_foo, &mut headers, &mut budget).unwrap(),
            DependencyResult::Found(false)
        ),
        "Foo does not satisfy Bounded<T extends String>"
    );

    let self_bound_child = bind(
        &mut types,
        b"Lbounds/SelfBound<Lbounds/SelfChild;>;",
        CHILD_LOADER,
        &mut headers,
        &mut budget,
    );
    assert!(
        matches!(
            declared_type_bounds(&mut types, self_bound_child, &mut headers, &mut budget).unwrap(),
            DependencyResult::Found(true)
        ),
        "SelfBound<SelfChild> is a legal recursively bounded application"
    );
}

#[test]
fn same_named_class_and_method_t_are_distinct_and_missing_ancestors_stay_incomplete() {
    let mut headers = headers_with_bounds();
    let bounded = headers.get(CHILD_LOADER, b"bounds/Bounded").clone();
    let generic = bounded.generic_signatures.as_ref().unwrap();
    let (class_signature, class_proof) = generic.class.as_ref().unwrap();
    let mut types = DeclaredGenericTypes::default();
    let mut budget = Budget::new(limits());
    let class_scope = jarde_java::generic_types::bind_declared_parameters(
        &mut types,
        &bounded,
        None,
        &class_signature.type_parameters,
        &class_proof.type_parameters,
        &[],
        &mut headers,
        &mut budget,
    )
    .unwrap()
    .found()
    .unwrap();
    let class_t = class_scope[0].1;
    let method_index = bounded
        .methods
        .iter()
        .position(|member| {
            member.name.raw().0 == b"shadow"
                && member.descriptor.raw().0 == b"(Ljava/lang/Object;)Ljava/lang/Object;"
        })
        .unwrap();
    let (method_signature, method_proof) = generic.methods[method_index].as_ref().unwrap();
    let method_scope = jarde_java::generic_types::bind_declared_parameters(
        &mut types,
        &bounded,
        Some((b"shadow", b"(Ljava/lang/Object;)Ljava/lang/Object;")),
        &method_signature.type_parameters,
        &method_proof.type_parameters,
        &class_scope,
        &mut headers,
        &mut budget,
    )
    .unwrap()
    .found()
    .unwrap();
    types.class_parameters = class_scope.iter().map(|(_, node)| *node).collect();
    types.method_parameters = method_scope
        .iter()
        .skip(class_scope.len())
        .map(|(_, node)| *node)
        .collect();
    let method_t = method_scope[1].1;
    assert_ne!(class_t, method_t);
    assert!(matches!(
        (&types.types[class_t], &types.types[method_t]),
        (DeclaredGenericType::Variable { method: None, name: left, .. },
         DeclaredGenericType::Variable { method: Some(_), name: right, .. })
            if left == right && left.0 == b"T"
    ));
    assert_eq!(
        jarde_java::generic_types::spell_declared_type(&types, class_t, &mut budget).unwrap(),
        None,
        "the class T is hidden by the same-named method T in this source namespace"
    );
    assert_eq!(
        jarde_java::generic_types::spell_declared_type(&types, method_t, &mut budget).unwrap(),
        Some("T".to_owned()),
        "the visible method T keeps its source spelling"
    );
    assert!(matches!(
        declared_assignable(&mut types, class_t, method_t, &mut headers, &mut budget).unwrap(),
        DependencyResult::Found(false)
    ));
    assert!(matches!(
        declared_assignable(&mut types, method_t, class_t, &mut headers, &mut budget).unwrap(),
        DependencyResult::Found(false)
    ));

    let mut incomplete = headers_with_bounds();
    incomplete.remove(CHILD_LOADER, b"java/lang/Object");
    let mut incomplete_types = DeclaredGenericTypes::default();
    let mut incomplete_budget = Budget::new(limits());
    let foo = bind(
        &mut incomplete_types,
        b"Lidentity/Foo;",
        CHILD_LOADER,
        &mut incomplete,
        &mut incomplete_budget,
    );
    let string = bind(
        &mut incomplete_types,
        b"Ljava/lang/String;",
        CHILD_LOADER,
        &mut incomplete,
        &mut incomplete_budget,
    );
    assert!(
        matches!(
            declared_assignable(
                &mut incomplete_types,
                foo,
                string,
                &mut incomplete,
                &mut incomplete_budget
            )
            .unwrap(),
            DependencyResult::Unknown
        ),
        "a missing Object ancestor makes the negative relation incomplete, not false"
    );

    let mut missing = headers_with_bounds();
    let mut missing_types = DeclaredGenericTypes::default();
    let mut missing_budget = Budget::new(limits());
    let bounded_string = bind(
        &mut missing_types,
        b"Lbounds/Bounded<Ljava/lang/String;>;",
        CHILD_LOADER,
        &mut missing,
        &mut missing_budget,
    );
    missing.remove(CHILD_LOADER, b"java/lang/String");
    assert!(
        matches!(
            declared_type_bounds(
                &mut missing_types,
                bounded_string,
                &mut missing,
                &mut missing_budget
            )
            .unwrap(),
            DependencyResult::Missing | DependencyResult::Unknown
        ),
        "a missing bound header is not reported as an incompatibility"
    );
}

#[test]
fn assignability_and_bound_checks_preserve_budget_and_cancellation_errors() {
    let mut headers = headers_with_bounds();
    let mut types = DeclaredGenericTypes::default();
    let mut setup = Budget::new(limits());
    let bounded = bind(
        &mut types,
        b"Lbounds/Bounded<Ljava/lang/String;>;",
        CHILD_LOADER,
        &mut headers,
        &mut setup,
    );
    let string = bind(
        &mut types,
        b"Ljava/lang/String;",
        CHILD_LOADER,
        &mut headers,
        &mut setup,
    );

    let mut constrained = limits();
    constrained.analysis_steps = 0;
    let error = expect_error(declared_assignable(
        &mut types,
        bounded,
        string,
        &mut headers,
        &mut Budget::new(constrained),
    ));
    assert!(matches!(error, Error::BudgetExceeded { .. }));

    let mut constrained = limits();
    constrained.analysis_steps = 0;
    let error = expect_error(declared_type_bounds(
        &mut types,
        bounded,
        &mut headers,
        &mut Budget::new(constrained),
    ));
    assert!(matches!(error, Error::BudgetExceeded { .. }));

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let error = expect_error(declared_assignable(
        &mut types,
        bounded,
        string,
        &mut headers,
        &mut Budget::with_cancellation_token(limits(), cancellation),
    ));
    assert!(matches!(error, Error::Cancelled { .. }));

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let error = expect_error(declared_type_bounds(
        &mut types,
        bounded,
        &mut headers,
        &mut Budget::with_cancellation_token(limits(), cancellation),
    ));
    assert!(matches!(error, Error::Cancelled { .. }));
}
