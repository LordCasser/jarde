use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 24,
        archive_entries: 10_000,
        entry_bytes: 1 << 24,
        read_bytes: 1 << 24,
        class_bytes: 1 << 24,
        attribute_bytes: 1 << 24,
        code_bytes: 1 << 24,
        result_items: 10_000,
        output_bytes: 1 << 24,
        nested_depth: 8,
        elapsed_millis: u64::MAX,
    }
}
fn u16b(v: &mut Vec<u8>, n: u16) {
    v.extend_from_slice(&n.to_be_bytes());
}
fn utf8(v: &mut Vec<u8>, s: &[u8]) {
    v.push(1);
    u16b(v, s.len() as u16);
    v.extend_from_slice(s);
}
fn class(major: u16) -> Vec<u8> {
    let mut v = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut v, 0);
    u16b(&mut v, major);
    u16b(&mut v, 5);
    utf8(&mut v, b"p/A");
    v.push(7);
    u16b(&mut v, 1);
    utf8(&mut v, b"java/lang/Object");
    v.push(7);
    u16b(&mut v, 3);
    u16b(&mut v, 0x21);
    u16b(&mut v, 2);
    u16b(&mut v, 4);
    u16b(&mut v, 0);
    u16b(&mut v, 0);
    u16b(&mut v, 0);
    u16b(&mut v, 0);
    v
}
fn zip(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for &(name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
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
    output.into_inner()
}
fn view(snapshot: &ArtifactSnapshot, release: u16, policy: MultiReleasePolicy) -> RuntimeView {
    RuntimeView {
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        profile: RuntimeProfile {
            java_release: release,
            multi_release: policy,
            layout: LayoutMode::Generic,
        },
        load_domain: LoadDomain {
            loader: LoaderId("test".into()),
            parent_loader: None,
            delegation: DelegationPolicy::ParentFirst,
            roots: vec![LoadRoot::Snapshot {
                snapshot: snapshot.id().clone(),
            }],
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        },
    }
}
fn selected(report: &MultiReleaseViewReport) -> Vec<u8> {
    let selection = report.containers[0]
        .selections
        .iter()
        .find(|s| s.logical_path.0 == b"p/A.class")
        .unwrap();
    match &selection.outcome {
        MultiReleaseSelectionOutcome::Selected { entry } => entry.raw_name.0.clone(),
        other => panic!("not selected: {other:?}"),
    }
}

#[test]
fn a06_selects_root_11_17_and_falls_back_without_v17() {
    let root = class(52);
    let v11 = class(55);
    let v17 = class(61);
    let bytes = zip(&[
        (
            b"META-INF/MANIFEST.MF",
            b"Manifest-Version: 1.0\r\nMulti-Release: true\r\n\r\n",
        ),
        (b"p/A.class", &root),
        (b"META-INF/versions/11/p/A.class", &v11),
        (b"META-INF/versions/17/p/A.class", &v17),
    ]);
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .unwrap();
    for (release, expected) in [
        (8, b"p/A.class".as_slice()),
        (11, b"META-INF/versions/11/p/A.class".as_slice()),
        (17, b"META-INF/versions/17/p/A.class".as_slice()),
    ] {
        let report = Engine::new()
            .select_multi_release(
                &snapshot,
                &view(&snapshot, release, MultiReleasePolicy::Enabled),
                &mut budget,
            )
            .unwrap();
        assert_eq!(selected(&report), expected);
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    }
    let bytes = zip(&[
        (b"META-INF/MANIFEST.MF", b"Multi-Release: true\n\n"),
        (b"p/A.class", &root),
        (b"META-INF/versions/11/p/A.class", &v11),
    ]);
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .unwrap();
    let report = Engine::new()
        .select_multi_release(
            &snapshot,
            &view(&snapshot, 17, MultiReleasePolicy::Enabled),
            &mut budget,
        )
        .unwrap();
    assert_eq!(selected(&report), b"META-INF/versions/11/p/A.class");
}

#[test]
fn missing_manifest_selects_base_and_keeps_versioned_inactive() {
    let root = class(52);
    let v11 = class(55);
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(
            ArtifactInput::bytes(zip(&[
                (b"p/A.class", &root),
                (b"META-INF/versions/11/p/A.class", &v11),
            ])),
            &mut budget,
        )
        .unwrap();
    let report = Engine::new()
        .select_multi_release(
            &snapshot,
            &view(&snapshot, 17, MultiReleasePolicy::Enabled),
            &mut budget,
        )
        .unwrap();
    assert_eq!(report.containers[0].manifest.state, ManifestState::Missing);
    assert_eq!(selected(&report), b"p/A.class");
    assert!(report.containers[0].entries.iter().any(|e| matches!(
        e.variant,
        MultiReleaseEntryVariant::Versioned { release: 11 }
    ) && matches!(
        e.decision,
        MultiReleaseSelectionDecision::Inactive {
            reason: MultiReleaseInactiveReason::ManifestMissing
        }
    )));
}

#[test]
fn unsupported_policies_preserve_candidates_without_selected_fact() {
    let root = class(52);
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(
            ArtifactInput::bytes(zip(&[(b"p/A.class", &root)])),
            &mut budget,
        )
        .unwrap();
    for policy in [
        MultiReleasePolicy::Custom { id: "x".into() },
        MultiReleasePolicy::Unknown,
    ] {
        let report = Engine::new()
            .select_multi_release(&snapshot, &view(&snapshot, 17, policy), &mut budget)
            .unwrap();
        assert!(matches!(
            report.execution,
            ExecutionReport::Failed {
                reason: TerminationReason::Unsupported { .. },
                ..
            }
        ));
        assert!(matches!(
            report.containers[0].selections[0].outcome,
            MultiReleaseSelectionOutcome::Unknown { .. }
        ));
    }
}

#[test]
fn typed_diagnostic_rejects_unknown_fields_codes_and_severity_mismatch() {
    let valid = serde_json::json!({"code":"multi_release_manifest_noncanonical_path","severity":"warning","message":"x","provenance":null});
    assert!(serde_json::from_value::<MultiReleaseDiagnostic>(valid.clone()).is_ok());
    let mut mismatch = valid.clone();
    mismatch["severity"] = serde_json::json!("error");
    assert!(serde_json::from_value::<MultiReleaseDiagnostic>(mismatch).is_err());
    let mut unknown = valid.clone();
    unknown["extra"] = serde_json::json!(1);
    assert!(serde_json::from_value::<MultiReleaseDiagnostic>(unknown).is_err());
    let mut code = valid;
    code["code"] = serde_json::json!("future_code");
    assert!(serde_json::from_value::<MultiReleaseDiagnostic>(code).is_err());
}
