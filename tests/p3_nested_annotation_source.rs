//! DT-22: a proved direct member annotation is written in its owner while its physical child stays
//! queryable. The binary inputs are the exact Java 8 outputs pinned by the fixed replay.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const HOLDER: &[u8] = include_bytes!("fixtures/dt22-nested-annotation/Holder.class");
const HOLDER_A: &[u8] = include_bytes!("fixtures/dt22-nested-annotation/Holder$A.class");
const DOLLAR_A: &[u8] = include_bytes!("fixtures/dt22-nested-annotation/Dollar$A.class");

fn jar(holder: &[u8], holder_a: &[u8]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in [
            (b"dt22/Holder.class".as_slice(), holder),
            (b"dt22/Holder$A.class".as_slice(), holder_a),
            (b"dt22/Dollar$A.class".as_slice(), DOLLAR_A),
        ] {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(bytes).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        archive.finish().unwrap();
    }
    output.into_inner()
}

fn budget() -> Budget {
    task_budget(&[]).expect("the default class-source budget is bounded")
}

fn report(holder: &[u8], holder_a: &[u8], name: &str, budget: &mut Budget) -> ClassSourceReport {
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(jar(holder, holder_a)), budget)
        .expect("the Java 8 fixture archive opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    match engine
        .class_source(std::slice::from_ref(&snapshot), &request, budget)
        .expect("the selected class-source request is valid")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one fixed fixture definition is unique, got {other:?}"),
    }
}

fn class_access_flags_offset(bytes: &[u8]) -> usize {
    let count = u16::from_be_bytes([bytes[8], bytes[9]]) as usize;
    let mut names = vec![None; count];
    let mut cursor = 10;
    let mut index = 1;
    while index < count {
        match bytes[cursor] {
            1 => {
                let length = u16::from_be_bytes([bytes[cursor + 1], bytes[cursor + 2]]) as usize;
                names[index] = Some(bytes[cursor + 3..cursor + 3 + length].to_vec());
                cursor += 3 + length;
            }
            3 | 4 => cursor += 5,
            5 | 6 => {
                cursor += 9;
                index += 1;
            }
            7 | 8 | 16 | 19 | 20 => cursor += 3,
            9 | 10 | 11 | 12 | 17 | 18 => cursor += 5,
            15 => cursor += 4,
            tag => panic!("unsupported constant-pool tag {tag}"),
        }
        index += 1;
    }
    cursor
}

fn inner_classes_attribute(bytes: &[u8]) -> (usize, usize, usize) {
    let count = u16::from_be_bytes([bytes[8], bytes[9]]) as usize;
    let mut names = vec![None; count];
    let mut cursor = 10;
    let mut index = 1;
    while index < count {
        match bytes[cursor] {
            1 => {
                let length = u16::from_be_bytes([bytes[cursor + 1], bytes[cursor + 2]]) as usize;
                names[index] = Some(bytes[cursor + 3..cursor + 3 + length].to_vec());
                cursor += 3 + length;
            }
            3 | 4 => cursor += 5,
            5 | 6 => {
                cursor += 9;
                index += 1;
            }
            7 | 8 | 16 | 19 | 20 => cursor += 3,
            9 | 10 | 11 | 12 | 17 | 18 => cursor += 5,
            15 => cursor += 4,
            tag => panic!("unsupported constant-pool tag {tag}"),
        }
        index += 1;
    }
    cursor += 6;
    let interfaces = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
    cursor += 2 + interfaces * 2;
    for _ in 0..2 {
        let members = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        cursor += 2;
        for _ in 0..members {
            let attributes = u16::from_be_bytes([bytes[cursor + 6], bytes[cursor + 7]]) as usize;
            cursor += 8;
            for _ in 0..attributes {
                let length = u32::from_be_bytes([
                    bytes[cursor + 2],
                    bytes[cursor + 3],
                    bytes[cursor + 4],
                    bytes[cursor + 5],
                ]) as usize;
                cursor += 6 + length;
            }
        }
    }
    let attributes = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
    cursor += 2;
    for _ in 0..attributes {
        let name_index = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        let length = u32::from_be_bytes([
            bytes[cursor + 2],
            bytes[cursor + 3],
            bytes[cursor + 4],
            bytes[cursor + 5],
        ]) as usize;
        if names[name_index].as_deref() == Some(b"InnerClasses") {
            return (cursor, cursor + 6, length);
        }
        cursor += 6 + length;
    }
    panic!("the Java 8 owner declares InnerClasses")
}

fn annotation_default_attribute(bytes: &[u8]) -> (usize, usize, usize) {
    let count = u16::from_be_bytes([bytes[8], bytes[9]]) as usize;
    let mut names = vec![None; count];
    let mut cursor = 10;
    let mut index = 1;
    while index < count {
        match bytes[cursor] {
            1 => {
                let length = u16::from_be_bytes([bytes[cursor + 1], bytes[cursor + 2]]) as usize;
                names[index] = Some(bytes[cursor + 3..cursor + 3 + length].to_vec());
                cursor += 3 + length;
            }
            3 | 4 => cursor += 5,
            5 | 6 => {
                cursor += 9;
                index += 1;
            }
            7 | 8 | 16 | 19 | 20 => cursor += 3,
            9 | 10 | 11 | 12 | 17 | 18 => cursor += 5,
            15 => cursor += 4,
            tag => panic!("unsupported constant-pool tag {tag}"),
        }
        index += 1;
    }
    cursor += 6;
    let interfaces = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
    cursor += 2 + interfaces * 2;
    let fields = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
    cursor += 2;
    for _ in 0..fields {
        let attributes = u16::from_be_bytes([bytes[cursor + 6], bytes[cursor + 7]]) as usize;
        cursor += 8;
        for _ in 0..attributes {
            let length = u32::from_be_bytes([
                bytes[cursor + 2],
                bytes[cursor + 3],
                bytes[cursor + 4],
                bytes[cursor + 5],
            ]) as usize;
            cursor += 6 + length;
        }
    }
    let methods = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
    cursor += 2;
    for _ in 0..methods {
        let attributes = u16::from_be_bytes([bytes[cursor + 6], bytes[cursor + 7]]) as usize;
        cursor += 8;
        for _ in 0..attributes {
            let name_index = u16::from_be_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
            let length = u32::from_be_bytes([
                bytes[cursor + 2],
                bytes[cursor + 3],
                bytes[cursor + 4],
                bytes[cursor + 5],
            ]) as usize;
            if names[name_index].as_deref() == Some(b"AnnotationDefault") {
                return (cursor, cursor + 6, length);
            }
            cursor += 6 + length;
        }
    }
    panic!("the Java 8 annotation element declares AnnotationDefault")
}

fn rewrite_owner_rows(change: &str) -> Vec<u8> {
    let mut bytes = HOLDER.to_vec();
    let (attribute, payload, length) = inner_classes_attribute(&bytes);
    let old = bytes[payload..payload + length].to_vec();
    let new = match change {
        "missing" => vec![0, 0],
        "duplicate" => [vec![0, 2], old[2..].to_vec(), old[2..].to_vec()].concat(),
        "conflict" => {
            let mut row = old;
            row[9] &= !0x01;
            row
        }
        _ => unreachable!(),
    };
    bytes.splice(payload..payload + length, new.iter().copied());
    let new_length = (new.len() as u32).to_be_bytes();
    bytes[attribute + 2..attribute + 6].copy_from_slice(&new_length);
    bytes
}

fn truncate_annotation_default(bytes: &[u8]) -> Vec<u8> {
    let mut bytes = bytes.to_vec();
    let (attribute, payload, length) = annotation_default_attribute(&bytes);
    assert_eq!(length, 3);
    bytes.remove(payload + length - 1);
    bytes[attribute + 2..attribute + 6].copy_from_slice(&((length - 1) as u32).to_be_bytes());
    bytes
}

#[test]
fn proved_annotation_is_nested_and_the_physical_child_is_preserved() {
    let root = report(HOLDER, HOLDER_A, "dt22/Holder", &mut budget());
    assert!(root.text.contains("public @interface A {"), "{}", root.text);
    assert!(root.text.contains("default 0x1.19999ap0f"));
    let ClassSourceNestedAnnotationFamily::Prepared {
        relation,
        child,
        projection: ClassSourceNestedAnnotationProjection::Projected { derived },
    } = &root.nested_annotation_family
    else {
        panic!(
            "the fixed direct annotation must be projected: {:#?}",
            root.nested_annotation_family
        );
    };
    assert_eq!(relation.simple_name, "A");
    assert!(child.text.contains("public @interface Holder$A {"));
    assert!(child.text.contains("default 0x1.19999ap0f"));
    assert_eq!(derived.len(), 1);
    let projected = &derived[0];
    assert_eq!(
        projected.kind,
        MemberFamilyDerivedKind::NestedAnnotationDeclaration
    );
    assert!(projected.start < projected.end);
    assert!(root.text[projected.start..projected.end].contains("@interface A"));
    assert_eq!(projected.anchors.len(), 2);
    assert!(matches!(
        &projected.anchors[0],
        MemberFamilyPhysicalAnchor::ClassDefinition { definition }
            if definition == &relation.root
    ));
    assert!(matches!(
        &projected.anchors[1],
        MemberFamilyPhysicalAnchor::ClassDefinition { definition }
            if definition == &relation.child
    ));
}

#[test]
fn malformed_child_kind_refuses_projection_but_keeps_the_child_report() {
    let mut wrong_kind = HOLDER_A.to_vec();
    let flags = class_access_flags_offset(&wrong_kind);
    let access = u16::from_be_bytes([wrong_kind[flags], wrong_kind[flags + 1]]) & !0x2000;
    wrong_kind[flags..flags + 2].copy_from_slice(&access.to_be_bytes());
    let root = report(HOLDER, &wrong_kind, "dt22/Holder", &mut budget());
    assert!(!root.text.contains("@interface A"), "{}", root.text);
    let ClassSourceNestedAnnotationFamily::Refused {
        child: Some(child), ..
    } = &root.nested_annotation_family
    else {
        panic!("a wrong child kind must be refused with its physical report");
    };
    assert!(child.text.contains("public interface Holder$A"));
}

#[test]
fn dollar_name_without_an_owner_row_remains_top_level() {
    let root = report(HOLDER, HOLDER_A, "dt22/Dollar$A", &mut budget());
    assert!(matches!(
        root.nested_annotation_family,
        ClassSourceNestedAnnotationFamily::Absent
    ));
    assert!(root.text.contains("public @interface Dollar$A {"));
}

#[test]
fn child_budget_stop_does_not_publish_a_partial_annotation() {
    let mut limits = budget().limits().clone();
    limits.class_headers = 1;
    let root = report(HOLDER, HOLDER_A, "dt22/Holder", &mut Budget::new(limits));
    assert!(!root.text.contains("@interface A"), "{}", root.text);
    assert!(matches!(
        root.nested_annotation_family,
        ClassSourceNestedAnnotationFamily::Refused { .. }
    ));
}

#[test]
fn missing_duplicate_and_conflicting_owner_rows_never_project() {
    for variant in ["missing", "duplicate", "conflict"] {
        let owner = rewrite_owner_rows(variant);
        let root = report(&owner, HOLDER_A, "dt22/Holder", &mut budget());
        assert!(
            !root.text.contains("@interface A"),
            "{variant}: {}",
            root.text
        );
        match variant {
            "missing" => assert!(matches!(
                root.nested_annotation_family,
                ClassSourceNestedAnnotationFamily::Absent
            )),
            _ => assert!(
                matches!(
                    root.nested_annotation_family,
                    ClassSourceNestedAnnotationFamily::Refused { .. }
                ),
                "{variant}: {:?}",
                root.nested_annotation_family
            ),
        }
    }
}

#[test]
fn truncated_default_attribute_refuses_projection_and_keeps_child() {
    let child = truncate_annotation_default(HOLDER_A);
    let root = report(HOLDER, &child, "dt22/Holder", &mut budget());
    assert!(!root.text.contains("@interface A"), "{}", root.text);
    let ClassSourceNestedAnnotationFamily::Refused {
        child: Some(child), ..
    } = &root.nested_annotation_family
    else {
        panic!("a truncated element default must retain its physical child report");
    };
    assert!(child.text.contains("Holder$A"));
}

#[test]
fn cancellation_stops_before_any_nested_declaration_is_published() {
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(jar(HOLDER, HOLDER_A)), &mut budget())
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("dt22/Holder"),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    let token = CancellationToken::new();
    token.cancel();
    let mut stopped = Budget::with_cancellation_token(budget().limits().clone(), token);
    let result = engine.class_source(std::slice::from_ref(&snapshot), &request, &mut stopped);
    assert!(
        matches!(result, Ok(OperationOutcome::Incomplete(_))),
        "{result:?}"
    );
}
