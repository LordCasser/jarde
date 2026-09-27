use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const ROOT: &[u8] =
    include_bytes!("../openspec/evidence/java-syntax-2026-09-28/em01-generic-member/Generic.class");
const CHILD: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-28/em01-generic-member/Generic$A.class"
);

fn jar(root: &[u8], child: &[u8]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut zip = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in [
            (b"em01/Generic.class".as_slice(), root),
            (b"em01/Generic$A.class".as_slice(), child),
        ] {
            let (mut entry, config) = zip
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(bytes).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        zip.finish().unwrap();
    }
    output.into_inner()
}

fn class_report(root: &[u8], child: &[u8], limits: Limits) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = Budget::new(limits);
    let snapshot = engine
        .open(ArtifactInput::bytes(jar(root, child)), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("em01/Generic"),
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
        .class_source(std::slice::from_ref(&snapshot), &request, &mut budget)
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("Generic selection failed: {other:?}"),
    }
}

fn changed(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    assert_eq!(from.len(), to.len());
    let offsets: Vec<_> = bytes
        .windows(from.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == from).then_some(offset))
        .collect();
    assert_eq!(offsets.len(), 1, "mutation must select one physical span");
    let mut result = bytes.to_vec();
    result[offsets[0]..offsets[0] + from.len()].copy_from_slice(to);
    result
}

fn with_pool_entries(bytes: &[u8], entries: &[u8], added: u16) -> Vec<u8> {
    let count = u16::from_be_bytes([bytes[8], bytes[9]]);
    let mut offset = 10;
    let mut index = 1;
    while index < count {
        let tag = bytes[offset];
        offset += 1;
        match tag {
            1 => {
                let length = usize::from(u16::from_be_bytes([bytes[offset], bytes[offset + 1]]));
                offset += 2 + length;
            }
            3 | 4 | 9 | 10 | 11 | 12 | 17 | 18 => offset += 4,
            5 | 6 => {
                offset += 8;
                index += 1;
            }
            7 | 8 | 16 | 19 | 20 => offset += 2,
            15 => offset += 3,
            _ => panic!("unexpected frozen constant-pool tag {tag}"),
        }
        index += 1;
    }
    let mut result = bytes.to_vec();
    result[8..10].copy_from_slice(&(count + added).to_be_bytes());
    result.splice(offset..offset, entries.iter().copied());
    result
}

fn is_projected(report: &ClassSourceReport) -> bool {
    matches!(
        report.member_family,
        ClassSourceMemberFamily::Prepared {
            projection: ClassSourceMemberProjection::Projected { .. },
            ..
        }
    )
}

#[test]
fn generic_static_member_is_atomic_and_keeps_physical_child() {
    let report = class_report(ROOT, CHILD, task_limits(&[]).unwrap());
    assert!(is_projected(&report), "{:?}", report.member_family);
    assert!(
        report
            .text
            .contains("public static abstract class A<T> implements java.lang.Comparable<A<T>>")
    );
    assert!(report.text.contains("T value;"));
    assert!(report.text.contains("public int compareTo(A<T> arg1)"));
    assert_eq!(report.text.matches("public int compareTo(").count(), 1);
    let ClassSourceMemberFamily::Prepared {
        child, projection, ..
    } = &report.member_family
    else {
        unreachable!()
    };
    assert!(child.text.contains("public abstract class Generic$A"));
    assert!(
        child
            .text
            .contains("public int compareTo(java.lang.Object arg1)")
    );
    let ClassSourceMemberProjection::Projected { derived } = projection else {
        unreachable!()
    };
    assert!(
        derived
            .iter()
            .any(|entry| entry.kind == MemberFamilyDerivedKind::MemberGenericSignature)
    );
    assert!(derived.iter().any(|entry| entry.anchors.iter().any(|anchor| matches!(anchor,
        MemberFamilyPhysicalAnchor::MethodPoint { method, .. } if method.name.0 == b"compareTo" && method.descriptor.0 == b"(Ljava/lang/Object;)I"))));

    let mut output_limited = task_limits(&[]).unwrap();
    output_limited.output_bytes = report.usage.output_bytes.saturating_sub(1);
    let stopped = class_report(ROOT, CHILD, output_limited);
    assert!(!is_projected(&stopped));
    assert!(!stopped.text.contains("class A<T>"));
    assert!(stopped.text.contains("public class Generic"));
    let mut analysis_limited = task_limits(&[]).unwrap();
    analysis_limited.analysis_steps = report.usage.analysis_steps / 2;
    let stopped = class_report(ROOT, CHILD, analysis_limited);
    assert!(!is_projected(&stopped));
    assert!(!stopped.text.contains("class A<T>"));

    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(jar(ROOT, CHILD)),
            &mut Budget::new(task_limits(&[]).unwrap()),
        )
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("em01/Generic"),
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
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let outcome = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::with_cancellation_token(task_limits(&[]).unwrap(), cancellation),
        )
        .unwrap();
    assert!(matches!(outcome, OperationOutcome::Incomplete(_)));
}

#[test]
fn malformed_relation_scope_and_bridge_keep_the_root_physical() {
    let self_row = changed(
        CHILD,
        &[0, 1, 0, 7, 0, 25, 0, 27, 4, 9],
        &[0, 1, 0, 7, 0, 2, 0, 27, 4, 9],
    );
    let interface = changed(CHILD, b"java/lang/Comparable<", b"java/lang/ComparablE<");
    let field_scope = changed(CHILD, &[0, 3, b'T', b'T', b';'], &[0, 3, b'T', b'U', b';']);
    let method_scope = changed(
        CHILD,
        b"(Lem01/Generic$A<TT;>;)I",
        b"(Lem01/Generic$A<TU;>;)I",
    );
    let bridge_effect = changed(
        CHILD,
        &[0x2a, 0x2b, 0xc0, 0, 7, 0xb6, 0, 9, 0xac],
        &[0x2a, 0x59, 0xc0, 0, 7, 0xb6, 0, 9, 0xac],
    );
    // The added Methodref names Object.compareTo(A):int. Verification succeeds because the
    // receiver is assignable to Object; invocation would throw NoSuchMethodError.
    let target_pool = with_pool_entries(CHILD, &[10, 0, 2, 0, 10], 1);
    let bridge_target = changed(&target_pool, &[0xb6, 0, 9, 0xac], &[0xb6, 0, 28, 0xac]);
    // The added Class is a real subtype in the external verifier replay. This cast is valid
    // bytecode but rejects ordinary A instances that the Java-generated bridge accepts.
    let mut cast_entries = vec![1, 0, 12];
    cast_entries.extend_from_slice(b"em01/GenXric");
    cast_entries.extend_from_slice(&[7, 0, 28]);
    let cast_pool = with_pool_entries(CHILD, &cast_entries, 2);
    let bridge_cast = changed(&cast_pool, &[0xc0, 0, 7, 0xb6], &[0xc0, 0, 29, 0xb6]);
    for (name, child) in [
        ("self row", self_row),
        ("interface", interface),
        ("field variable", field_scope),
        ("method variable", method_scope),
        ("bridge effect", bridge_effect),
        ("bridge target", bridge_target),
        ("bridge cast", bridge_cast),
    ] {
        let report = class_report(ROOT, &child, task_limits(&[]).unwrap());
        assert!(!is_projected(&report), "{name}: {:?}", report.member_family);
        assert!(!report.text.contains("class A<T>"), "{name}");
    }
}
