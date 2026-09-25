use jarde_reader::budget::{Budget, BudgetDimension, CancellationToken, Limits};
use jarde_reader::classfile::{TypeAnnotationFacts, attribute_facts, class_facts};

const CLASS: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-22/type-use-annotations/generated/original/TypeUseSubject.class"
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

#[test]
fn member_type_annotations_retain_owner_target_path_and_value() {
    let mut budget = budget();
    let class = class_facts(CLASS, &mut budget).unwrap();
    let mut all = Vec::<TypeAnnotationFacts>::new();
    for member in class.fields.iter().chain(&class.methods) {
        let shells = member
            .attributes
            .iter()
            .filter(|shell| {
                matches!(
                    shell.name.raw().0.as_slice(),
                    b"RuntimeVisibleTypeAnnotations" | b"RuntimeInvisibleTypeAnnotations"
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        let parsed = attribute_facts(CLASS, &shells, &class.constant_pool, &mut budget).unwrap();
        all.extend(parsed.runtime_visible_type_annotations);
        all.extend(parsed.runtime_invisible_type_annotations);
    }
    assert_eq!(all.len(), 3);
    assert!(all.iter().all(|fact| fact.type_path.is_empty()));
    assert_eq!(
        all.iter().map(|fact| fact.target_type).collect::<Vec<_>>(),
        [0x13, 0x14, 0x16]
    );
    assert!(all.iter().all(|fact| matches!(
        fact.annotation,
        jarde_reader::classfile::ElementValueFacts::Annotation { .. }
    )));
}

#[test]
fn no_type_annotation_shells_do_not_read_content() {
    assert_eq!(
        attribute_facts(CLASS, &[], &[], &mut budget()).unwrap(),
        Default::default()
    );
}

#[test]
fn type_annotation_shell_obeys_attribute_budget_and_cancellation() {
    let mut discovery = budget();
    let class = class_facts(CLASS, &mut discovery).unwrap();
    let shell = class
        .fields
        .iter()
        .flat_map(|member| &member.attributes)
        .chain(class.methods.iter().flat_map(|member| &member.attributes))
        .find(|shell| shell.name.raw().0 == b"RuntimeVisibleTypeAnnotations")
        .expect("visible type annotation shell")
        .clone();

    let constrained = Limits {
        attribute_bytes: shell.span.length - 1,
        ..limits()
    };
    let error = attribute_facts(
        CLASS,
        std::slice::from_ref(&shell),
        &class.constant_pool,
        &mut Budget::new(constrained),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        jarde_reader::error::Error::BudgetExceeded {
            dimension: BudgetDimension::AttributeBytes,
            ..
        }
    ));

    let cancellation = CancellationToken::new();
    let mut cancelled = Budget::with_cancellation_token(limits(), cancellation.clone());
    cancellation.cancel();
    assert!(matches!(
        attribute_facts(CLASS, &[shell], &class.constant_pool, &mut cancelled),
        Err(jarde_reader::error::Error::Cancelled { .. })
    ));
}

#[test]
fn local_variable_target_preserves_its_table_and_charges_each_entry() {
    let mut discovery = budget();
    let class = class_facts(CLASS, &mut discovery).unwrap();
    let shell = class
        .fields
        .iter()
        .flat_map(|member| &member.attributes)
        .find(|shell| shell.name.raw().0 == b"RuntimeVisibleTypeAnnotations")
        .expect("visible type annotation shell")
        .clone();
    let start = usize::try_from(shell.content_span.start).unwrap();
    let end = start + usize::try_from(shell.content_span.length).unwrap();
    let original = &CLASS[start..end];
    assert_eq!(original[2], 0x13);
    let mut replacement = original[..2].to_vec();
    replacement.extend_from_slice(&[0x40, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00]);
    replacement.extend_from_slice(&original[3..]);
    let mut bytes = CLASS.to_vec();
    bytes.splice(start..end, replacement.iter().copied());
    let length = u32::try_from(replacement.len()).unwrap();
    let header = usize::try_from(shell.span.start).unwrap() + 2;
    bytes[header..header + 4].copy_from_slice(&length.to_be_bytes());

    let mut read = budget();
    let patched = class_facts(&bytes, &mut read).unwrap();
    let patched_shell = patched
        .fields
        .iter()
        .flat_map(|member| &member.attributes)
        .find(|shell| shell.name.raw().0 == b"RuntimeVisibleTypeAnnotations")
        .expect("patched visible type annotation shell")
        .clone();
    let parsed = attribute_facts(
        &bytes,
        std::slice::from_ref(&patched_shell),
        &patched.constant_pool,
        &mut read,
    )
    .unwrap();
    let fact = &parsed.runtime_visible_type_annotations[0];
    assert_eq!(fact.target_type, 0x40);
    assert_eq!(fact.target_info, [0, 1, 0, 0, 0, 1, 0, 0]);

    let limited = Limits {
        result_items: 0,
        ..limits()
    };
    assert!(matches!(
        attribute_facts(
            &bytes,
            &[patched_shell],
            &patched.constant_pool,
            &mut Budget::new(limited)
        ),
        Err(jarde_reader::error::Error::BudgetExceeded {
            dimension: BudgetDimension::ResultItems,
            ..
        })
    ));

    let mut truncated = bytes;
    let table_count = start + 3;
    truncated[table_count..table_count + 2].copy_from_slice(&2_u16.to_be_bytes());
    let mut read = budget();
    let truncated_class = class_facts(&truncated, &mut read).unwrap();
    let truncated_shell = truncated_class
        .fields
        .iter()
        .flat_map(|member| &member.attributes)
        .find(|shell| shell.name.raw().0 == b"RuntimeVisibleTypeAnnotations")
        .expect("truncated visible type annotation shell")
        .clone();
    assert!(
        attribute_facts(
            &truncated,
            &[truncated_shell],
            &truncated_class.constant_pool,
            &mut read,
        )
        .is_err()
    );
}
