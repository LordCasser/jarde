use jarde_reader::budget::{Budget, BudgetDimension, CancellationToken, Limits};
use jarde_reader::classfile::{
    AttributeFacts, ParameterAnnotationFacts, attribute_facts, class_facts,
};

const VISIBLE: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-22/member-annotation-uses/generated/original/MemberTagged.class"
);
const INVISIBLE: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-22/member-annotation-uses/boundaries/generated/legal/BoundaryTagged.class"
);
const COUNT_MISMATCH: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-22/member-annotation-uses/boundaries/generated/patched/parameter-count-mismatch/BoundaryTagged.class"
);

fn limits() -> Limits {
    Limits {
        input_bytes: u64::MAX,
        archive_entries: u64::MAX,
        entry_bytes: u64::MAX,
        read_bytes: u64::MAX,
        class_bytes: u64::MAX,
        attribute_bytes: u64::MAX,
        code_bytes: u64::MAX,
        result_items: u64::MAX,
        output_bytes: u64::MAX,
        nested_depth: u64::MAX,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

fn budget() -> Budget {
    Budget::new(limits())
}

fn method_shells<'a>(
    class: &'a jarde_reader::classfile::ClassFacts,
    name: &[u8],
) -> Vec<&'a jarde_reader::classfile::AttributeShell> {
    let method = class
        .methods
        .iter()
        .find(|method| method.name.raw().0 == name)
        .expect("method from the frozen class");
    method
        .attributes
        .iter()
        .filter(|attribute| {
            matches!(
                attribute.name.raw().0.as_slice(),
                b"RuntimeVisibleAnnotations"
                    | b"RuntimeInvisibleAnnotations"
                    | b"RuntimeVisibleParameterAnnotations"
                    | b"RuntimeInvisibleParameterAnnotations"
            )
        })
        .collect()
}

fn parsed_parameters(facts: Option<ParameterAnnotationFacts>) -> ParameterAnnotationFacts {
    facts.expect("parameter annotation shell produced typed facts")
}

#[test]
fn visible_parameter_annotation_groups_keep_the_attribute_u1_and_descriptor_order() {
    let mut budget = budget();
    let class = class_facts(VISIBLE, &mut budget).unwrap();
    let method = method_shells(&class, b"value");
    let parameter = method
        .iter()
        .find(|attribute| attribute.name.raw().0 == b"RuntimeVisibleParameterAnnotations")
        .expect("visible parameter annotation shell");
    let parsed = attribute_facts(
        VISIBLE,
        &[(*parameter).clone()],
        &class.constant_pool,
        &mut budget,
    )
    .unwrap();
    let groups = parsed_parameters(parsed.runtime_visible_parameter_annotations);
    assert_eq!(groups.parameter_count, 1);
    assert_eq!(groups.parameters.len(), 1);
    assert_eq!(groups.parameters[0].len(), 1);
    assert_eq!(parsed.runtime_invisible_parameter_annotations, None);
}

#[test]
fn invisible_parameter_annotations_keep_wide_and_varargs_positions() {
    let mut budget = budget();
    let class = class_facts(INVISIBLE, &mut budget).unwrap();
    let method = method_shells(&class, b"wideAndVarargs");
    let parameter = method
        .iter()
        .find(|attribute| attribute.name.raw().0 == b"RuntimeInvisibleParameterAnnotations")
        .expect("invisible parameter annotation shell");
    let parsed = attribute_facts(
        INVISIBLE,
        &[(*parameter).clone()],
        &class.constant_pool,
        &mut budget,
    )
    .unwrap();
    let groups = parsed_parameters(parsed.runtime_invisible_parameter_annotations);
    assert_eq!(groups.parameter_count, 3);
    assert_eq!(groups.parameters.len(), 3);
    assert!(
        groups
            .parameters
            .iter()
            .all(|annotations| annotations.len() == 1)
    );
    assert_eq!(parsed.runtime_visible_parameter_annotations, None);
}

#[test]
fn parameter_count_mismatch_is_preserved_as_the_u1_the_attribute_declares() {
    let mut budget = budget();
    let class = class_facts(COUNT_MISMATCH, &mut budget).unwrap();
    let method = method_shells(&class, b"wideAndVarargs");
    let parameter = method
        .iter()
        .find(|attribute| attribute.name.raw().0 == b"RuntimeInvisibleParameterAnnotations")
        .expect("patched parameter annotation shell");
    let parsed = attribute_facts(
        COUNT_MISMATCH,
        &[(*parameter).clone()],
        &class.constant_pool,
        &mut budget,
    )
    .unwrap();
    let groups = parsed_parameters(parsed.runtime_invisible_parameter_annotations);
    assert_eq!(groups.parameter_count, 2);
    assert_eq!(groups.parameters.len(), 2);
}

#[test]
fn no_annotation_shells_do_not_read_attribute_content() {
    let parsed = attribute_facts(VISIBLE, &[], &[], &mut budget()).unwrap();
    assert_eq!(parsed, AttributeFacts::default());
}

#[test]
fn parameter_attribute_requires_its_exact_declared_end() {
    let mut discovery = budget();
    let class = class_facts(INVISIBLE, &mut discovery).unwrap();
    let parameter = method_shells(&class, b"wideAndVarargs")
        .into_iter()
        .find(|attribute| attribute.name.raw().0 == b"RuntimeInvisibleParameterAnnotations")
        .expect("invisible parameter annotation shell")
        .clone();
    let mut shortened = parameter;
    shortened.span.length -= 1;
    shortened.content_span.length -= 1;
    assert!(
        attribute_facts(
            INVISIBLE,
            &[shortened],
            &class.constant_pool,
            &mut discovery,
        )
        .is_err()
    );
}

#[test]
fn parameter_annotation_attribute_budget_refusal_publishes_no_facts() {
    let mut discovery = budget();
    let class = class_facts(INVISIBLE, &mut discovery).unwrap();
    let parameter = method_shells(&class, b"wideAndVarargs")
        .into_iter()
        .find(|attribute| attribute.name.raw().0 == b"RuntimeInvisibleParameterAnnotations")
        .expect("invisible parameter annotation shell")
        .clone();
    let limit = parameter.span.length - 1;
    let mut limited = limits();
    limited.attribute_bytes = limit;
    let mut limited_budget = Budget::new(limited);
    let error = attribute_facts(
        INVISIBLE,
        &[parameter],
        &class.constant_pool,
        &mut limited_budget,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        jarde_reader::error::Error::BudgetExceeded {
            dimension: BudgetDimension::AttributeBytes,
            ..
        }
    ));
}

#[test]
fn cancellation_before_parameter_annotation_read_publishes_no_facts() {
    let cancellation = CancellationToken::new();
    let mut budget = Budget::with_cancellation_token(limits(), cancellation.clone());
    let class = class_facts(INVISIBLE, &mut budget).unwrap();
    let parameter = method_shells(&class, b"wideAndVarargs")
        .into_iter()
        .find(|attribute| attribute.name.raw().0 == b"RuntimeInvisibleParameterAnnotations")
        .expect("invisible parameter annotation shell")
        .clone();
    cancellation.cancel();
    assert!(matches!(
        attribute_facts(INVISIBLE, &[parameter], &class.constant_pool, &mut budget),
        Err(jarde_reader::error::Error::Cancelled { .. })
    ));
}
