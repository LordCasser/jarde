//! Raw JVM identity and Java type spelling are separate boundaries.
use jarde_java::{source_proof::*, type_of_component};
use jarde_reader::classfile::{CpEntryFacts, CpEntryKind, DescriptorKind, descriptor_facts};
use jarde_reader::model::{ByteSpan, JvmBytes};

fn descriptor(name: &[u8]) -> Vec<u8> {
    [b"L", name, b";"].concat()
}
fn field(descriptor: Vec<u8>) -> CpEntryFacts {
    CpEntryFacts {
        index: 1,
        span: ByteSpan::new(0, 1),
        kind: CpEntryKind::FieldRef {
            class_index: 2,
            name_and_type_index: 3,
            owner: JvmBytes::from_unicode("p/Other"),
            name: JvmBytes::from_unicode("value"),
            descriptor: JvmBytes(descriptor),
        },
    }
}

#[test]
fn ascii_and_primitive_array_controls_are_preserved() {
    let shape = snapshot_reference_shape(b"[[Lp/Box;").unwrap();
    assert_eq!(shape.source_type, "p.Box[][]");
    assert_eq!(shape.rank, 2);
    assert_eq!(shape.leaf_class, b"p/Box");
    assert_eq!(
        generic_call_reference_type_from_descriptor(b"[I")
            .unwrap()
            .spell(),
        "int[]"
    );
    assert!(generic_call_reference_type_from_descriptor(b"I").is_none());
    assert!(spellable_source_name(b"p/Box"));
    assert!(!spellable_source_name(b"p/Outer$Inner"));
    assert!(static_fold_pool_entry_names_type(
        &field(descriptor(b"p/Box")),
        b"p/Box"
    ));
}

#[test]
fn snapshot_shapes_decode_mutf8_before_source_spelling() {
    let raw = JvmBytes::from_unicode("p/𐐀Box");
    for (name, spelling, rank) in [
        (raw.0.clone(), "p.𐐀Box", 0),
        (
            [b"[[", descriptor(&raw.0).as_slice()].concat(),
            "p.𐐀Box[][]",
            2,
        ),
    ] {
        let shape = snapshot_reference_shape(&name)
            .expect("a supplementary Java identifier is source-spellable");
        assert_eq!(shape.source_type, spelling);
        assert_eq!(shape.leaf_class, raw.0);
        assert_eq!(shape.rank, rank);
    }
}

#[test]
fn generic_reference_types_use_the_shared_descriptor_spelling() {
    let raw = JvmBytes::from_unicode("p/𐐀Box");
    assert_eq!(
        generic_call_reference_type_from_descriptor(&descriptor(&raw.0))
            .unwrap()
            .spell(),
        "p.𐐀Box"
    );
    for invalid in [
        b"Lp/a-b;".as_slice(),
        b"Lp/class;",
        b"[Lp/9Box;",
        b"Lp/Box;garbage",
    ] {
        assert!(
            generic_call_reference_type_from_descriptor(invalid).is_none(),
            "{invalid:?}"
        );
    }
}

#[test]
fn descriptor_types_reject_source_unspellable_class_names() {
    for raw in ["p/a-b", "p/class", "p/9Box", "a-b/Box", "class/Box"] {
        let descriptor = descriptor(raw.as_bytes());
        let facts = descriptor_facts(&descriptor, DescriptorKind::Field).unwrap();
        assert!(
            type_of_component(facts.single().unwrap()).is_none(),
            "{raw}"
        );
    }
}

#[test]
fn producer_descriptor_identity_never_uses_lossy_display() {
    let raw = JvmBytes::from_unicode("p/𐐀Box");
    assert!(static_fold_pool_entry_names_type(
        &field(descriptor(&raw.0)),
        &raw.0
    ));
    let display = String::from_utf8_lossy(&raw.0);
    assert!(!static_fold_pool_entry_names_type(
        &field(descriptor(display.as_bytes())),
        &raw.0
    ));
    let method = |bytes| CpEntryFacts {
        index: 1,
        span: ByteSpan::new(0, 1),
        kind: CpEntryKind::MethodRef {
            class_index: 2,
            name_and_type_index: 3,
            owner: JvmBytes::from_unicode("p/Other"),
            name: JvmBytes::from_unicode("call"),
            descriptor: JvmBytes(bytes),
        },
    };
    assert!(static_fold_pool_entry_names_type(
        &method([b"()", descriptor(&raw.0).as_slice()].concat()),
        &raw.0
    ));
    assert!(
        !static_fold_pool_entry_names_type(
            &method([b"(", descriptor(&raw.0).as_slice(), b")V"].concat()),
            &raw.0
        ),
        "a parameter mention is still not a producing return type"
    );
}

#[test]
fn source_projection_name_gate_decodes_before_applying_its_policy() {
    assert!(spellable_source_name(&JvmBytes::from_unicode("p/𐐀Box").0));
    for raw in [
        b"p/a-b".as_slice(),
        b"p/class",
        b"p/9Box",
        b"p/Outer$Inner",
        b"p/\xed\xa0\x81",
        b"p/\xf0\x90\x90\x80",
    ] {
        assert!(!spellable_source_name(raw), "{raw:?}");
    }
}

#[test]
fn java8_character_categories_preserve_source_identity() {
    for name in ["€Box", "x\u{0301}", "𐐀Box"] {
        assert!(jarde_java::names::is_java_identifier(name), "{name}");
    }
    for name in ["x²", "\u{0903}Box", "a\u{200b}b"] {
        assert!(!jarde_java::names::is_java_identifier(name), "{name}");
    }
    assert_eq!(
        jarde_java::names::source_class_name(&JvmBytes::from_unicode("_/€Box")),
        Some("_.€Box".to_owned())
    );
    assert!(
        !jarde_java::names::is_java_identifier("_"),
        "the established local-name alias policy stays separate"
    );
}
