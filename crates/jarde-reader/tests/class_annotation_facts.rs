use jarde_reader::budget::{Budget, BudgetDimension, CancellationToken, Limits};
use jarde_reader::classfile::{
    AttributeFacts, CpEntryKind, ElementConstantTag, ElementValueFacts, attribute_facts,
    class_facts, class_member_facts, cp_entry,
};

const INVISIBLE: &[u8] =
    include_bytes!("../../../tests/fixtures/class-annotation-uses/v8/HiddenTarget.class");
const VISIBLE: &[u8] =
    include_bytes!("../../../tests/fixtures/class-annotation-uses/v8/VisibleTarget.class");
const EMPTY: &[u8] =
    include_bytes!("../../../tests/fixtures/class-annotation-uses/v8/EmptyTarget.class");

fn budget() -> Budget {
    Budget::new(unlimited_limits())
}

fn unlimited_limits() -> Limits {
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

#[test]
fn class_retention_annotation_is_read_from_its_lazy_class_shell() {
    let mut budget = budget();
    let facts = class_member_facts(INVISIBLE, &mut budget).unwrap();
    let shell = facts
        .attributes
        .iter()
        .find(|attribute| attribute.name.raw().0 == b"RuntimeInvisibleAnnotations")
        .expect("class attribute shell");
    let class = class_facts(INVISIBLE, &mut budget).unwrap();
    let before_content = budget.usage().attribute_bytes;
    let parsed = attribute_facts(
        INVISIBLE,
        std::slice::from_ref(shell),
        &class.constant_pool,
        &mut budget,
    )
    .unwrap();
    assert_eq!(
        budget.usage().attribute_bytes - before_content,
        shell.span.length
    );
    assert_eq!(parsed.runtime_visible_annotations, []);
    assert_eq!(parsed.runtime_invisible_annotations.len(), 1);
    let ElementValueFacts::Annotation {
        type_descriptor,
        elements,
    } = &parsed.runtime_invisible_annotations[0]
    else {
        panic!("the shell has an annotation entry");
    };
    assert_eq!(type_descriptor.0, b"LHiddenTag;");
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].name.0, b"value");
    let ElementValueFacts::Constant { tag, index } = &elements[0].value else {
        panic!("the annotation value is an integer constant");
    };
    assert_eq!(*tag, ElementConstantTag::Integer);
    assert_eq!(
        cp_entry(&class.constant_pool, index.0).unwrap().kind,
        CpEntryKind::Integer { value: 5 }
    );
}

#[test]
fn no_class_annotation_shell_means_no_annotation_content_read() {
    let mut budget = budget();
    let facts = class_facts(EMPTY, &mut budget).unwrap();
    let before = budget.usage().attribute_bytes;
    let parsed = attribute_facts(EMPTY, &[], &facts.constant_pool, &mut budget).unwrap();
    assert_eq!(parsed, AttributeFacts::default());
    assert_eq!(budget.usage().attribute_bytes, before);
}

#[test]
fn runtime_visible_annotation_preserves_its_utf8_element_value() {
    let mut budget = budget();
    let facts = class_facts(VISIBLE, &mut budget).unwrap();
    let shell = facts
        .attributes
        .iter()
        .find(|attribute| attribute.name.raw().0 == b"RuntimeVisibleAnnotations")
        .expect("runtime-visible class annotation shell");
    let parsed = attribute_facts(
        VISIBLE,
        std::slice::from_ref(shell),
        &facts.constant_pool,
        &mut budget,
    )
    .unwrap();
    let ElementValueFacts::Annotation {
        type_descriptor,
        elements,
    } = &parsed.runtime_visible_annotations[0]
    else {
        panic!("the visible attribute contains one annotation");
    };
    assert_eq!(type_descriptor.0, b"LVisibleTag;");
    assert_eq!(elements[0].name.0, b"value");
    assert_eq!(
        elements[0].value,
        ElementValueFacts::Utf8(jarde_reader::model::JvmBytes(b"visible".to_vec()))
    );
}

#[test]
fn annotation_content_budget_refusal_returns_no_partial_facts() {
    let mut discovery_budget = budget();
    let discovered = class_facts(INVISIBLE, &mut discovery_budget).unwrap();
    let shell = discovered
        .attributes
        .iter()
        .find(|attribute| attribute.name.raw().0 == b"RuntimeInvisibleAnnotations")
        .unwrap();
    let shell_usage = discovery_budget.usage().attribute_bytes;
    let limit = shell_usage + shell.content_span.length - 1;
    let mut limited_limits = unlimited_limits();
    limited_limits.attribute_bytes = limit;
    let mut limited_budget = Budget::new(limited_limits);
    let facts = class_facts(INVISIBLE, &mut limited_budget).unwrap();
    let result = attribute_facts(
        INVISIBLE,
        std::slice::from_ref(shell),
        &facts.constant_pool,
        &mut limited_budget,
    );
    assert!(matches!(
        result,
        Err(jarde_reader::error::Error::BudgetExceeded {
            dimension: BudgetDimension::AttributeBytes,
            ..
        })
    ));
}

#[test]
fn class_attribute_shell_damage_is_a_structural_error() {
    let mut damaged = INVISIBLE.to_vec();
    let mut read_budget = budget();
    let facts = class_member_facts(&damaged, &mut read_budget).unwrap();
    let shell = facts
        .attributes
        .iter()
        .find(|attribute| attribute.name.raw().0 == b"RuntimeInvisibleAnnotations")
        .unwrap();
    let shell_start = usize::try_from(shell.span.start).unwrap();
    damaged[shell_start + 2..shell_start + 6].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(matches!(
        class_member_facts(&damaged, &mut budget()),
        Err(jarde_reader::error::Error::InvalidInput { .. })
    ));
}

#[test]
fn class_attribute_shell_budget_stop_is_not_a_complete_member_read() {
    let mut discovery_budget = budget();
    class_member_facts(INVISIBLE, &mut discovery_budget).unwrap();
    let mut limited = unlimited_limits();
    limited.attribute_bytes = discovery_budget.usage().attribute_bytes - 1;
    let error = class_member_facts(INVISIBLE, &mut Budget::new(limited)).unwrap_err();
    assert!(matches!(
        error,
        jarde_reader::error::Error::BudgetExceeded {
            dimension: BudgetDimension::AttributeBytes,
            ..
        }
    ));
}

#[test]
fn cancellation_before_annotation_content_read_publishes_no_facts() {
    let cancellation = CancellationToken::new();
    let mut budget = Budget::with_cancellation_token(unlimited_limits(), cancellation.clone());
    let facts = class_facts(INVISIBLE, &mut budget).unwrap();
    let shell = facts
        .attributes
        .iter()
        .find(|attribute| attribute.name.raw().0 == b"RuntimeInvisibleAnnotations")
        .unwrap();
    cancellation.cancel();
    assert!(matches!(
        attribute_facts(
            INVISIBLE,
            std::slice::from_ref(shell),
            &facts.constant_pool,
            &mut budget
        ),
        Err(jarde_reader::error::Error::Cancelled { .. })
    ));
}
