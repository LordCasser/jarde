//! A JVM symbol is MUTF-8; an archive entry keeps its own raw UTF-8 path bytes.
use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const SAMPLE: &[u8] = include_bytes!("fixtures/s1-raw-names/RawAlias.class");

fn budget() -> Budget {
    let mut limits = task_budget(&[]).unwrap().limits().clone();
    limits.elapsed_millis = 60_000;
    Budget::new(limits)
}

fn renamed(raw: &[u8]) -> Vec<u8> {
    let needle = b"\x01\x00\x08RawAlias";
    let positions: Vec<_> = SAMPLE
        .windows(needle.len())
        .enumerate()
        .filter_map(|(i, bytes)| (bytes == needle).then_some(i))
        .collect();
    assert_eq!(positions.len(), 1);
    let start = positions[0];
    let mut bytes = SAMPLE[..start + 1].to_vec();
    bytes.extend_from_slice(&u16::try_from(raw.len()).unwrap().to_be_bytes());
    bytes.extend_from_slice(raw);
    bytes.extend_from_slice(&SAMPLE[start + needle.len()..]);
    bytes
}

fn jar(entries: &[(Vec<u8>, Vec<u8>)]) -> ArtifactSnapshot {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.clone()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(data).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        archive.finish().unwrap();
    }
    Engine::new()
        .open(ArtifactInput::bytes(output.into_inner()), &mut budget())
        .unwrap()
}

fn resolve(snapshot: &ArtifactSnapshot, prefix: &[u8], raw: Vec<u8>) -> ResolutionReport {
    let domain = LoadDomain {
        loader: LoaderId("app".into()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Container {
            origin: ContainerOrigin {
                snapshot: snapshot.id().clone(),
                root_container: ContainerId("root".into()),
                steps: Vec::new(),
            },
            prefix: ArchiveNameBytes(prefix.to_vec()),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    Engine::new()
        .resolve_symbol(
            std::slice::from_ref(snapshot),
            &ResolutionRequest {
                environment: ResolutionEnvironment {
                    runtime: RuntimeView {
                        physical: PhysicalView {
                            snapshot: snapshot.id().clone(),
                            scope: PhysicalScope::SnapshotAll,
                        },
                        profile: RuntimeProfile {
                            java_release: 8,
                            multi_release: MultiReleasePolicy::Disabled,
                            layout: LayoutMode::Generic,
                        },
                        load_domain: domain.clone(),
                    },
                    domains: vec![domain],
                    providers: Vec::new(),
                },
                target: SymbolRef::Class {
                    owner: JvmBytes(raw),
                },
                use_kind: ReferenceUse::ClassReference,
                caller: CallerContext {
                    loader: LoaderId("app".into()),
                    enclosing: None,
                },
                dispatch: None,
            },
            &mut budget(),
        )
        .unwrap()
}

#[test]
fn navigation_converts_the_query_and_preserves_distinct_archive_origins() {
    for name in ["p/Ascii", "p/名字", "p/𐐄"] {
        let raw = JvmBytes::from_unicode(name);
        let bytes = renamed(&raw.0);
        let path = format!("WEB-INF/classes/{name}.class").into_bytes();
        let mut entries = vec![(path.clone(), bytes.clone())];
        if raw.0 != name.as_bytes() {
            let mut decoy = b"WEB-INF/classes/".to_vec();
            decoy.extend_from_slice(&raw.0);
            decoy.extend_from_slice(b".class");
            entries.push((decoy, bytes));
        }
        let snapshot = jar(&entries);
        let listing = Engine::new()
            .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget())
            .unwrap();
        assert!(matches!(
            listing.execution,
            ExecutionReport::Complete { .. }
        ));
        assert_eq!(listing.items.len(), entries.len());
        for (item, (entry, _)) in listing.items.iter().zip(&entries) {
            assert_eq!(&item.definition.entry().unwrap().raw_name.0, entry);
            assert_eq!(item.declaration.this_class.raw(), &raw);
        }
        assert_eq!(listing.items[0].binding, ClassNameBinding::PathNameAgrees);
        if entries.len() == 2 {
            assert!(matches!(
                listing.items[1].binding,
                ClassNameBinding::PathNameDiffers { .. }
            ));
        }
        for class in [
            ClassNameQuery::internal(name),
            ClassNameQuery::dotted(name.replace('/', ".")),
        ] {
            assert_eq!(class.internal_name(), &raw);
            let report = Engine::new()
                .find_targets(
                    &snapshot,
                    &PhysicalScope::SnapshotAll,
                    &NavigationQuery {
                        class,
                        member: None,
                    },
                    &mut budget(),
                )
                .unwrap();
            assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
            let classes: Vec<_> = report.classes().collect();
            assert_eq!(classes.len(), 1, "{name}: {report:?}");
            assert_eq!(classes[0].definition.entry().unwrap().raw_name.0, path);
        }
    }
}

#[test]
fn runtime_lookup_transcodes_the_internal_name_and_keeps_the_raw_prefix() {
    let name = "p/𐐄";
    let raw = JvmBytes::from_unicode(name);
    let bytes = renamed(&raw.0);
    for prefix in [b"".as_slice(), b"WEB-INF/classes/", b"\xff/\xfe/"] {
        let mut expected = prefix.to_vec();
        expected.extend_from_slice(name.as_bytes());
        expected.extend_from_slice(b".class");
        let mut decoy = prefix.to_vec();
        decoy.extend_from_slice(&raw.0);
        decoy.extend_from_slice(b".class");
        let snapshot = jar(&[(decoy, bytes.clone()), (expected.clone(), bytes.clone())]);
        let report = resolve(&snapshot, prefix, raw.0.clone());
        let definition = &report
            .resolved
            .as_ref()
            .expect("scalar name resolves")
            .definition;
        assert_eq!(definition.entry().unwrap().ordinal, 1, "{report:?}");
        assert_eq!(definition.entry().unwrap().raw_name.0, expected);
        assert_eq!(report.reads.len(), 1);
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    }
}

#[test]
fn non_scalar_or_damaged_names_are_refused_before_guessing_an_archive_path() {
    let replacement = JvmBytes::from_unicode("\u{fffd}");
    let snapshot = jar(&[(
        "\u{fffd}.class".as_bytes().to_vec(),
        renamed(&replacement.0),
    )]);
    for raw in [
        vec![0xed, 0xa0, 0x81],
        vec![0xed, 0xa0, 0x82],
        vec![0xf0, 0x90, 0x90, 0x80],
    ] {
        let report = resolve(&snapshot, b"", raw.clone());
        assert!(report.resolved.is_none());
        assert!(report.state.is_none(), "{report:?}");
        assert!(report.reads.is_empty());
        assert!(
            matches!(report.execution, ExecutionReport::Failed { .. }),
            "{report:?}"
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "class_entry_name_unrepresentable"),
            "{report:?}"
        );
        assert_eq!(
            report.target,
            SymbolRef::Class {
                owner: JvmBytes(raw)
            }
        );
    }
}

#[test]
fn deserialized_navigation_names_are_refused_before_scanning_instead_of_looking_missing() {
    let snapshot = jar(&[(b"RawAlias.class".to_vec(), SAMPLE.to_vec())]);
    for raw in [
        vec![0xed, 0xa0, 0x81],
        vec![0xed, 0xa0, 0x82],
        vec![0xf0, 0x90, 0x90, 0x80],
        vec![0],
    ] {
        let query: NavigationQuery = serde_json::from_value(serde_json::json!({
            "class": { "spelling": "diagnostic-only", "internal_name": raw },
            "member": null
        }))
        .unwrap();
        let mut bounded = budget();
        let error = Engine::new()
            .find_targets(&snapshot, &PhysicalScope::SnapshotAll, &query, &mut bounded)
            .unwrap_err();
        assert!(matches!(
            error,
            Error::InvalidInput { code, .. } if code == "navigation_class_name_unrepresentable"
        ));
        assert_eq!(bounded.usage().archive_entries, 0);
        assert_eq!(bounded.usage().class_headers, 0);
        assert_eq!(bounded.usage().read_bytes, 0);
    }
}
