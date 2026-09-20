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
        ..Limits::default()
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
fn limits_with(result_items: u64, read_bytes: u64, class_bytes: u64) -> Limits {
    Limits {
        result_items,
        read_bytes,
        class_bytes,
        ..limits()
    }
}
/// Minimal, complete class file with explicit major version, access flags and `this_class`.
fn class_with(major: u16, access: u16, this_name: &[u8]) -> Vec<u8> {
    let mut v = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut v, 0);
    u16b(&mut v, major);
    u16b(&mut v, 5);
    utf8(&mut v, this_name);
    v.push(7);
    u16b(&mut v, 1);
    utf8(&mut v, b"java/lang/Object");
    v.push(7);
    u16b(&mut v, 3);
    u16b(&mut v, access);
    u16b(&mut v, 2);
    u16b(&mut v, 4);
    u16b(&mut v, 0);
    u16b(&mut v, 0);
    u16b(&mut v, 0);
    u16b(&mut v, 0);
    v
}
fn class(major: u16) -> Vec<u8> {
    class_with(major, 0x21, b"p/A")
}

const ACTIVE_MANIFEST: &[u8] = b"Manifest-Version: 1.0\r\nMulti-Release: true\r\n\r\n";

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

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    let mut budget = Budget::new(limits());
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .unwrap()
}

fn runtime_view(
    snapshot: &ArtifactSnapshot,
    release: u16,
    policy: MultiReleasePolicy,
    scope: PhysicalScope,
) -> RuntimeView {
    RuntimeView {
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope,
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
            roots: vec![LoadRoot::Container {
                origin: ContainerOrigin {
                    snapshot: snapshot.id().clone(),
                    root_container: ContainerId("root".into()),
                    steps: Vec::new(),
                },
                prefix: ArchiveNameBytes(Vec::new()),
            }],
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        },
    }
}

fn view_for(snapshot: &ArtifactSnapshot, release: u16, policy: MultiReleasePolicy) -> RuntimeView {
    runtime_view(snapshot, release, policy, PhysicalScope::SnapshotAll)
}

fn tree_view_for(
    snapshot: &ArtifactSnapshot,
    release: u16,
    policy: MultiReleasePolicy,
) -> RuntimeView {
    runtime_view(
        snapshot,
        release,
        policy,
        PhysicalScope::ArtifactTree {
            root_container: ContainerId("root".into()),
        },
    )
}

fn select_with(
    snapshot: &ArtifactSnapshot,
    runtime: &RuntimeView,
    budget: &mut Budget,
) -> MultiReleaseViewReport {
    Engine::new()
        .select_multi_release(snapshot, runtime, budget)
        .unwrap()
}

fn select(snapshot: &ArtifactSnapshot, runtime: &RuntimeView) -> MultiReleaseViewReport {
    let mut budget = Budget::new(limits());
    select_with(snapshot, runtime, &mut budget)
}

fn containers(report: &MultiReleaseViewReport) -> &[MultiReleaseContainerReport] {
    &report.containers
}

fn container(report: &MultiReleaseViewReport) -> &MultiReleaseContainerReport {
    &report.containers[0]
}

fn entries_of<'a>(
    container: &'a MultiReleaseContainerReport,
    name: &[u8],
) -> Vec<&'a MultiReleaseEntryEvidence> {
    container
        .entries
        .iter()
        .filter(|item| item.entry.raw_name.0 == name)
        .collect()
}

fn entry<'a>(report: &'a MultiReleaseViewReport, name: &[u8]) -> &'a MultiReleaseEntryEvidence {
    let found = entries_of(container(report), name);
    assert_eq!(
        found.len(),
        1,
        "expected exactly one entry evidence for {:?}",
        String::from_utf8_lossy(name)
    );
    found[0]
}

fn selection<'a>(report: &'a MultiReleaseViewReport, path: &[u8]) -> &'a MultiReleaseSelection {
    container(report)
        .selections
        .iter()
        .find(|item| item.logical_path.0 == path)
        .unwrap_or_else(|| {
            panic!(
                "no selection for {:?}: {:?}",
                String::from_utf8_lossy(path),
                container(report)
                    .selections
                    .iter()
                    .map(|item| String::from_utf8_lossy(&item.logical_path.0).into_owned())
                    .collect::<Vec<_>>()
            )
        })
}

fn decision(report: &MultiReleaseViewReport, name: &[u8]) -> MultiReleaseSelectionDecision {
    entry(report, name).decision.clone()
}

fn selected_name(report: &MultiReleaseViewReport) -> Vec<u8> {
    selected_name_of(report, b"p/A.class")
}

/// The raw entry name a logical path's group really selected.
fn selected_name_of(report: &MultiReleaseViewReport, logical: &[u8]) -> Vec<u8> {
    match &selection(report, logical).outcome {
        MultiReleaseSelectionOutcome::Selected { entry } => entry.raw_name.0.clone(),
        other => panic!(
            "expected a selected entry for {:?}, found {other:?}",
            String::from_utf8_lossy(logical)
        ),
    }
}

/// The real winning `PhysicalEntryId`s recorded on a shadowed candidate.
fn winners(report: &MultiReleaseViewReport, name: &[u8]) -> Vec<PhysicalEntryId> {
    match decision(report, name) {
        MultiReleaseSelectionDecision::Shadowed { winners } => winners,
        other => panic!(
            "expected {:?} to be shadowed, found {other:?}",
            String::from_utf8_lossy(name)
        ),
    }
}

/// The `MultiReleaseSelectionDecision` of every group member is cross-checked against the
/// container-level `Selected` outcome, which names the same entry.
fn assert_winner_is_selected(report: &MultiReleaseViewReport, logical: &[u8], winner: &[u8]) {
    let MultiReleaseSelectionOutcome::Selected { entry } = &selection(report, logical).outcome
    else {
        panic!("expected a selected outcome for {logical:?}");
    };
    assert_eq!(
        entry.raw_name.0,
        winner,
        "selection outcome and candidate decision disagree for {:?}",
        String::from_utf8_lossy(logical)
    );
    assert!(entries_of(container(report), winner).iter().any(|item| {
        item.decision == MultiReleaseSelectionDecision::Selected && item.entry == *entry
    }));
}

fn domain_codes(report: &MultiReleaseViewReport) -> Vec<MultiReleaseDiagnosticCode> {
    report
        .diagnostics
        .iter()
        .filter_map(|item| match item {
            MultiReleaseReportDiagnostic::Domain { diagnostic } => Some(diagnostic.code),
            MultiReleaseReportDiagnostic::Terminal { .. } => None,
        })
        .collect()
}

fn terminal_codes(report: &MultiReleaseViewReport) -> Vec<String> {
    report
        .diagnostics
        .iter()
        .filter_map(|item| match item {
            MultiReleaseReportDiagnostic::Terminal { diagnostic } => Some(diagnostic.code.clone()),
            MultiReleaseReportDiagnostic::Domain { .. } => None,
        })
        .collect()
}

fn diagnostic_ordinals(report: &MultiReleaseViewReport) -> Vec<u64> {
    report
        .diagnostics
        .iter()
        .filter_map(|item| match item {
            MultiReleaseReportDiagnostic::Domain { diagnostic } => {
                match diagnostic.provenance.as_ref().map(|value| &value.location) {
                    Some(Location::Entry { id, .. }) => Some(id.ordinal),
                    _ => None,
                }
            }
            MultiReleaseReportDiagnostic::Terminal { .. } => None,
        })
        .collect()
}

fn dimension(container: &MultiReleaseContainerReport) -> &CoverageDimension {
    &container.coverage.runtime_resolution
}

fn scanned_in(coverage: &CoverageDimension, metric: &str, ordinal: u64) -> bool {
    coverage.scanned.iter().any(|range| {
        range.label.ends_with(metric) && range.start == ordinal && range.end == ordinal + 1
    })
}

fn skipped_in(coverage: &CoverageDimension, metric: &str, ordinal: u64) -> bool {
    coverage.skipped.iter().any(|range| {
        range.label.ends_with(metric) && range.start == ordinal && range.end == ordinal + 1
    })
}

fn is_scanned(container: &MultiReleaseContainerReport, metric: &str, ordinal: u64) -> bool {
    scanned_in(dimension(container), metric, ordinal)
}

fn is_skipped(container: &MultiReleaseContainerReport, metric: &str, ordinal: u64) -> bool {
    skipped_in(dimension(container), metric, ordinal)
}

/// Every aggregate `skipped` label covering exactly one ordinal, so a test can name the
/// container whose ordinals were declared unprocessed.
fn aggregate_skipped_labels(report: &MultiReleaseViewReport, ordinal: u64) -> Vec<String> {
    report
        .coverage
        .runtime_resolution
        .skipped
        .iter()
        .filter(|range| range.start == ordinal && range.end == ordinal + 1)
        .map(|range| range.label.clone())
        .collect()
}

fn container_id(origin: &ContainerOrigin) -> String {
    origin.current_container().0.clone()
}

fn a06_fixture() -> Vec<u8> {
    zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
        (b"META-INF/versions/17/p/A.class", &class(61)),
    ])
}

#[test]
fn a06_selects_root_11_and_17_and_marks_the_other_variants() {
    let snapshot = open(a06_fixture());
    for (release, expected) in [
        (8_u16, b"p/A.class".as_slice()),
        (11, b"META-INF/versions/11/p/A.class"),
        (17, b"META-INF/versions/17/p/A.class"),
    ] {
        let report = select(
            &snapshot,
            &view_for(&snapshot, release, MultiReleasePolicy::Enabled),
        );
        assert_eq!(selected_name(&report), expected, "java {release}");
        assert_eq!(report.verification, VerificationStatus::NotPerformed);
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
        assert!(matches!(
            container(&report).execution,
            ExecutionReport::Complete { .. }
        ));
        let manifest = &container(&report).manifest;
        assert_eq!(manifest.state, ManifestState::Active);
        assert_eq!(manifest.entries.len(), 1);
        assert_eq!(
            manifest.attribute_name.as_ref().unwrap().0,
            b"Multi-Release"
        );
        assert_eq!(manifest.attribute_value.as_ref().unwrap().0, b"true");
        assert!(entry(&report, b"p/A.class").class_evidence.is_some());
        assert!(
            entry(&report, b"META-INF/versions/11/p/A.class")
                .class_evidence
                .is_some()
        );
        assert!(
            entry(&report, b"META-INF/versions/17/p/A.class")
                .class_evidence
                .is_some()
        );
        match release {
            8 => {
                assert!(matches!(
                    decision(&report, b"META-INF/versions/11/p/A.class"),
                    MultiReleaseSelectionDecision::Inactive {
                        reason: MultiReleaseInactiveReason::TargetBelowNine
                    }
                ));
                assert!(matches!(
                    decision(&report, b"META-INF/versions/17/p/A.class"),
                    MultiReleaseSelectionDecision::Inactive {
                        reason: MultiReleaseInactiveReason::TargetBelowNine
                    }
                ));
                assert_eq!(
                    decision(&report, b"p/A.class"),
                    MultiReleaseSelectionDecision::Selected
                );
            }
            11 => {
                // The shadowed base names the very v11 entry that this view selected.
                assert_eq!(
                    winners(&report, b"p/A.class"),
                    vec![
                        entry(&report, b"META-INF/versions/11/p/A.class")
                            .entry
                            .clone()
                    ]
                );
                assert_eq!(
                    decision(&report, b"META-INF/versions/11/p/A.class"),
                    MultiReleaseSelectionDecision::Selected
                );
                assert!(matches!(
                    decision(&report, b"META-INF/versions/17/p/A.class"),
                    MultiReleaseSelectionDecision::Inactive {
                        reason: MultiReleaseInactiveReason::ReleaseAboveTarget
                    }
                ));
            }
            _ => {
                let v17 = entry(&report, b"META-INF/versions/17/p/A.class")
                    .entry
                    .clone();
                assert_eq!(winners(&report, b"p/A.class"), vec![v17.clone()]);
                assert_eq!(
                    winners(&report, b"META-INF/versions/11/p/A.class"),
                    vec![v17.clone()]
                );
                assert_eq!(
                    decision(&report, b"META-INF/versions/17/p/A.class"),
                    MultiReleaseSelectionDecision::Selected
                );
            }
        }
        for ordinal in 0..4 {
            assert!(
                is_scanned(
                    container(&report),
                    "multi_release_selection_entries",
                    ordinal
                ),
                "ordinal {ordinal} not scanned at java {release}"
            );
        }
        assert!(dimension(container(&report)).skipped.is_empty());
        assert_eq!(
            dimension(container(&report)).state,
            CoverageState::CompleteWithinSchema
        );
    }
}

#[test]
fn a06_without_v17_falls_back_to_v11_and_without_manifest_to_base() {
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert_eq!(selected_name(&report), b"META-INF/versions/11/p/A.class");
    assert_winner_is_selected(&report, b"p/A.class", b"META-INF/versions/11/p/A.class");
    // The shadowed base names the real winning entry, not merely a shape.
    let shadowing = winners(&report, b"p/A.class");
    assert_eq!(shadowing.len(), 1);
    assert_eq!(shadowing[0].raw_name.0, b"META-INF/versions/11/p/A.class");
    assert_eq!(shadowing[0].ordinal, 2);
    assert!(shadowing[0].origin.steps.is_empty());
    assert_eq!(
        shadowing[0],
        entry(&report, b"META-INF/versions/11/p/A.class").entry
    );
    assert!(matches!(
        container(&report).execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(container(&report).entries.len(), 3);
    assert!(dimension(container(&report)).skipped.is_empty());

    let snapshot = open(zip(&[
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert_eq!(container(&report).manifest.state, ManifestState::Missing);
    assert!(container(&report).manifest.entries.is_empty());
    assert_eq!(selected_name(&report), b"p/A.class");
    assert!(matches!(
        decision(&report, b"META-INF/versions/11/p/A.class"),
        MultiReleaseSelectionDecision::Inactive {
            reason: MultiReleaseInactiveReason::ManifestMissing
        }
    ));
    assert!(matches!(
        container(&report).execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(container(&report).entries.len(), 2);
}

#[test]
fn disabled_policy_and_low_target_inactivate_versioned_entries() {
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Disabled),
    );
    assert_eq!(selected_name(&report), b"p/A.class");
    assert!(matches!(
        decision(&report, b"META-INF/versions/11/p/A.class"),
        MultiReleaseSelectionDecision::Inactive {
            reason: MultiReleaseInactiveReason::PolicyDisabled
        }
    ));
    // the physical Manifest evidence is still read under a Disabled policy
    assert_eq!(container(&report).manifest.state, ManifestState::Active);
    assert!(matches!(
        container(&report).execution,
        ExecutionReport::Complete { .. }
    ));
}

#[test]
fn missing_base_reports_the_closed_no_selection_reason() {
    let snapshot = open(zip(&[(b"META-INF/versions/11/p/A.class", &class(55))]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert!(matches!(
        selection(&report, b"p/A.class").outcome,
        MultiReleaseSelectionOutcome::NoSelection {
            reason: MultiReleaseNoSelectionReason::NoBaseWhileInactive
        }
    ));
    assert!(matches!(
        decision(&report, b"META-INF/versions/11/p/A.class"),
        MultiReleaseSelectionDecision::Inactive {
            reason: MultiReleaseInactiveReason::ManifestMissing
        }
    ));

    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"META-INF/versions/17/p/A.class", &class(61)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 11, MultiReleasePolicy::Enabled),
    );
    assert!(matches!(
        selection(&report, b"p/A.class").outcome,
        MultiReleaseSelectionOutcome::NoSelection {
            reason: MultiReleaseNoSelectionReason::NoApplicableRelease
        }
    ));
    assert!(matches!(
        decision(&report, b"META-INF/versions/17/p/A.class"),
        MultiReleaseSelectionDecision::Inactive {
            reason: MultiReleaseInactiveReason::ReleaseAboveTarget
        }
    ));
}

#[test]
fn manifest_evidence_states_and_path_diagnostics() {
    // Two case-equivalent candidates are ambiguous evidence, not a first-wins guess.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"META-INF/manifest.mf", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    let manifest = &container(&report).manifest;
    assert_eq!(manifest.state, ManifestState::Ambiguous);
    assert_eq!(manifest.entries.len(), 2);
    assert_eq!(manifest.entries[0].ordinal, 0);
    assert_eq!(manifest.entries[1].ordinal, 1);
    assert!(manifest.attribute_name.is_none());
    assert!(manifest.attribute_value.is_none());
    assert_eq!(
        domain_codes(&report),
        vec![
            MultiReleaseDiagnosticCode::MultiReleaseManifestNoncanonicalPath,
            MultiReleaseDiagnosticCode::MultiReleaseManifestDuplicate,
            MultiReleaseDiagnosticCode::MultiReleaseManifestDuplicate,
        ]
    );
    assert!(matches!(
        decision(&report, b"META-INF/versions/11/p/A.class"),
        MultiReleaseSelectionDecision::Unknown {
            reason: MultiReleaseUnknownReason::ManifestEvidenceAmbiguous
        }
    ));
    // Complete domain evidence, so the interruption-free work stays Complete.
    assert!(matches!(
        container(&report).execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(
        dimension(container(&report)).state,
        CoverageState::CompleteWithinSchema
    );

    // Malformed main sections: duplicate attribute, orphan continuation, unterminated
    // header, NUL byte, missing separator and a separator that is not exactly one space.
    for broken in [
        b"Multi-Release: true\r\nmulti-release: true\r\n\r\n".as_slice(),
        b" orphan continuation\r\nMulti-Release: true\r\n\r\n",
        b"Multi-Release: true\r\n",
        b"Multi-Release: tr\0ue\r\n\r\n",
        b"Multi-Release true\r\n\r\n",
        b"Multi-Release:true\r\n\r\n",
    ] {
        let snapshot = open(zip(&[
            (b"META-INF/MANIFEST.MF", broken),
            (b"p/A.class", &class(52)),
            (b"META-INF/versions/11/p/A.class", &class(55)),
        ]));
        let report = select(
            &snapshot,
            &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        );
        let manifest = &container(&report).manifest;
        assert_eq!(manifest.state, ManifestState::Malformed);
        assert!(manifest.attribute_name.is_none());
        assert_eq!(
            domain_codes(&report),
            vec![MultiReleaseDiagnosticCode::MultiReleaseManifestMalformed]
        );
        assert!(matches!(
            decision(&report, b"p/A.class"),
            MultiReleaseSelectionDecision::Unknown {
                reason: MultiReleaseUnknownReason::ManifestEvidenceMalformed
            }
        ));
    }

    // Only the expanded value `true` activates; the raw value keeps its bytes and case.
    for (value_bytes, state, raw) in [
        (
            b"Multi-Release: TRUE\r\n\r\n".as_slice(),
            ManifestState::Active,
            b"TRUE".as_slice(),
        ),
        (
            b"Multi-Release: true \r\n\r\n",
            ManifestState::Inactive,
            b"true ",
        ),
        (
            b"Multi-Release: false\r\n\r\n",
            ManifestState::Inactive,
            b"false",
        ),
        // `name: value` consumes exactly one space, so a second one belongs to the value.
        (
            b"Multi-Release:  true\r\n\r\n",
            ManifestState::Inactive,
            b" true",
        ),
        // CR-only line endings and a single-space continuation still expand to `true`.
        (b"Multi-Release: true\r\r", ManifestState::Active, b"true"),
        (
            b"Multi-Release: tr\r\n ue\r\n\r\n",
            ManifestState::Active,
            b"true",
        ),
        // LF-only line endings activate exactly like CRLF.
        (
            b"Manifest-Version: 1.0\nMulti-Release: true\n\n",
            ManifestState::Active,
            b"true",
        ),
    ] {
        let snapshot = open(zip(&[
            (b"META-INF/MANIFEST.MF", value_bytes),
            (b"p/A.class", &class(52)),
        ]));
        let report = select(
            &snapshot,
            &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        );
        let manifest = &container(&report).manifest;
        assert_eq!(manifest.state, state);
        assert_eq!(manifest.attribute_value.as_ref().unwrap().0, raw);
    }

    // A LF-only Manifest activates versioned selection just like the canonical CRLF form.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", b"Multi-Release: true\n\n"),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert_eq!(container(&report).manifest.state, ManifestState::Active);
    assert_eq!(selected_name(&report), b"META-INF/versions/11/p/A.class");
    assert!(matches!(
        container(&report).execution,
        ExecutionReport::Complete { .. }
    ));

    // A named section never activates MR processing; only the main section is inspected.
    let named_only = b"Manifest-Version: 1.0\r\n\r\nName: p/A.class\r\nMulti-Release: true\r\n\r\n";
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", named_only.as_slice()),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    let manifest = &container(&report).manifest;
    assert_eq!(manifest.state, ManifestState::Inactive);
    assert!(manifest.attribute_name.is_none());
    assert!(manifest.attribute_value.is_none());
    assert_eq!(selected_name(&report), b"p/A.class");
    assert!(matches!(
        decision(&report, b"META-INF/versions/11/p/A.class"),
        MultiReleaseSelectionDecision::Inactive {
            reason: MultiReleaseInactiveReason::ManifestInactive
        }
    ));
    assert!(domain_codes(&report).is_empty());

    // The same lines without a blank separator are still the main section and do activate.
    let main_section = b"Manifest-Version: 1.0\r\nName: p/A.class\r\nMulti-Release: true\r\n\r\n";
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", main_section.as_slice()),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert_eq!(container(&report).manifest.state, ManifestState::Active);
    assert_eq!(selected_name(&report), b"META-INF/versions/11/p/A.class");

    // A single non-canonical case variant still supplies activation evidence.
    let snapshot = open(zip(&[
        (b"meta-inf/manifest.mf", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert_eq!(container(&report).manifest.state, ManifestState::Active);
    assert_eq!(
        container(&report).manifest.entries[0].raw_name.0,
        b"meta-inf/manifest.mf"
    );
    assert_eq!(
        domain_codes(&report),
        vec![MultiReleaseDiagnosticCode::MultiReleaseManifestNoncanonicalPath]
    );
    let warning = report
        .diagnostics
        .iter()
        .find_map(|item| match item {
            MultiReleaseReportDiagnostic::Domain { diagnostic } => Some(diagnostic),
            MultiReleaseReportDiagnostic::Terminal { .. } => None,
        })
        .unwrap();
    assert_eq!(
        warning.code,
        MultiReleaseDiagnosticCode::MultiReleaseManifestNoncanonicalPath
    );
    assert_eq!(warning.severity, DiagnosticSeverity::Warning);
    assert_eq!(diagnostic_ordinals(&report), vec![0]);
    assert_eq!(selected_name(&report), b"META-INF/versions/11/p/A.class");
}

#[test]
fn invalid_version_paths_keep_physical_evidence_and_one_diagnostic_each() {
    let cases: &[(
        &[u8],
        MultiReleaseVersionPathIssue,
        MultiReleaseDiagnosticCode,
    )] = &[
        (
            b"META-INF/versions/9",
            MultiReleaseVersionPathIssue::EmptyLogicalPath,
            MultiReleaseDiagnosticCode::MultiReleaseVersionPathInvalid,
        ),
        (
            b"META-INF/versions//A.class",
            MultiReleaseVersionPathIssue::EmptyRelease,
            MultiReleaseDiagnosticCode::MultiReleaseVersionPathInvalid,
        ),
        (
            b"META-INF/versions/x/A.class",
            MultiReleaseVersionPathIssue::NonDecimalRelease,
            MultiReleaseDiagnosticCode::MultiReleaseVersionPathInvalid,
        ),
        (
            b"META-INF/versions/09/A.class",
            MultiReleaseVersionPathIssue::LeadingZeroRelease,
            MultiReleaseDiagnosticCode::MultiReleaseVersionPathInvalid,
        ),
        (
            b"META-INF/versions/99999999999999999999/A.class",
            MultiReleaseVersionPathIssue::ReleaseOverflow,
            MultiReleaseDiagnosticCode::MultiReleaseVersionOverflow,
        ),
        (
            b"META-INF/versions/8/A.class",
            MultiReleaseVersionPathIssue::ReleaseBelowNine,
            MultiReleaseDiagnosticCode::MultiReleaseVersionBelowNine,
        ),
    ];
    for (name, issue, code) in cases {
        let snapshot = open(zip(&[
            (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
            (b"p/A.class", &class(52)),
            (name, b"payload"),
        ]));
        let report = select(
            &snapshot,
            &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        );
        let item = entry(&report, name);
        assert_eq!(
            item.variant,
            MultiReleaseEntryVariant::InvalidVersioned { issue: *issue },
            "variant for {:?}",
            String::from_utf8_lossy(name)
        );
        assert!(matches!(
            item.decision,
            MultiReleaseSelectionDecision::NotApplicable {
                reason: MultiReleaseNotApplicableReason::InvalidVersionPath
            }
        ));
        assert_eq!(item.compliance, MultiReleaseCompliance::NonConformant);
        assert!(item.logical_path.is_none());
        assert!(item.class_evidence.is_none());
        assert_eq!(domain_codes(&report), vec![*code]);
        assert_eq!(diagnostic_ordinals(&report), vec![item.entry.ordinal]);
        // No selection is created for the invalid path, and the unrelated path stays proven.
        assert_eq!(selected_name(&report), b"p/A.class");
        assert_eq!(container(&report).selections.len(), 2);
    }

    // A versioned directory is a Directory entry, never a candidate.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"META-INF/versions/11/", b""),
        (b"p/A.class", &class(52)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    let directory = entry(&report, b"META-INF/versions/11/");
    assert_eq!(directory.variant, MultiReleaseEntryVariant::Base);
    assert!(matches!(
        directory.decision,
        MultiReleaseSelectionDecision::NotApplicable {
            reason: MultiReleaseNotApplicableReason::DirectoryEntry
        }
    ));
    assert_eq!(directory.compliance, MultiReleaseCompliance::NotApplicable);
    assert!(directory.logical_path.is_none());
    assert!(domain_codes(&report).is_empty());
    assert_eq!(container(&report).selections.len(), 2);

    // A legal version directory cannot select a META-INF resource.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"META-INF/versions/11/META-INF/x.mf", b"x"),
        (b"p/A.class", &class(52)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    let resource = entry(&report, b"META-INF/versions/11/META-INF/x.mf");
    assert!(matches!(
        resource.decision,
        MultiReleaseSelectionDecision::NotApplicable {
            reason: MultiReleaseNotApplicableReason::VersionedMetaInfResource
        }
    ));
    assert_eq!(resource.compliance, MultiReleaseCompliance::NonConformant);
    assert_eq!(
        domain_codes(&report),
        vec![MultiReleaseDiagnosticCode::MultiReleaseMetaInfResource]
    );
    assert_eq!(container(&report).selections.len(), 2);

    // A versioned META-INF *class* is not selectable either, and is never probed: the
    // non-conformance is already proven by its raw path.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (
            b"META-INF/versions/11/META-INF/X.class",
            &class_with(55, 0x21, b"META-INF/X"),
        ),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    let versioned_class = entry(&report, b"META-INF/versions/11/META-INF/X.class");
    assert!(matches!(
        versioned_class.decision,
        MultiReleaseSelectionDecision::NotApplicable {
            reason: MultiReleaseNotApplicableReason::VersionedMetaInfResource
        }
    ));
    assert_eq!(
        versioned_class.compliance,
        MultiReleaseCompliance::NonConformant
    );
    assert_eq!(
        versioned_class.logical_path.as_ref().unwrap().0,
        b"META-INF/X.class"
    );
    assert!(
        versioned_class.class_evidence.is_none(),
        "a versioned META-INF resource must not be probed"
    );
    assert_eq!(
        domain_codes(&report),
        vec![MultiReleaseDiagnosticCode::MultiReleaseMetaInfResource]
    );
    assert_eq!(selected_name(&report), b"p/A.class");
    assert_eq!(container(&report).selections.len(), 2);
    // A non-applicable entry has no bounded Header check to declare at all.
    assert!(
        dimension(container(&report))
            .scanned
            .iter()
            .chain(dimension(container(&report)).skipped.iter())
            .all(
                |range| !range.label.ends_with("multi_release_compliance_entries")
                    || range.start != versioned_class.entry.ordinal
            )
    );
}

#[test]
fn release_nine_is_a_legal_version_directory_and_selects_at_target_nine() {
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/9/p/A.class", &class(53)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    for (release, expected) in [
        (8_u16, b"p/A.class".as_slice()),
        (9, b"META-INF/versions/9/p/A.class"),
        (10, b"META-INF/versions/9/p/A.class"),
        (11, b"META-INF/versions/11/p/A.class"),
    ] {
        let report = select(
            &snapshot,
            &view_for(&snapshot, release, MultiReleasePolicy::Enabled),
        );
        assert_eq!(selected_name(&report), expected, "java {release}");
        assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    }
    let report = select(
        &snapshot,
        &view_for(&snapshot, 11, MultiReleasePolicy::Enabled),
    );
    let nine = entry(&report, b"META-INF/versions/9/p/A.class");
    assert_eq!(
        nine.variant,
        MultiReleaseEntryVariant::Versioned { release: 9 }
    );
    assert_eq!(nine.logical_path.as_ref().unwrap().0, b"p/A.class");
    assert_eq!(nine.class_evidence.as_ref().unwrap().major_version, 53);
    assert_eq!(
        nine.compliance,
        MultiReleaseCompliance::ConformantWithinChecks
    );
    assert_eq!(
        winners(&report, b"META-INF/versions/9/p/A.class"),
        vec![
            entry(&report, b"META-INF/versions/11/p/A.class")
                .entry
                .clone()
        ]
    );

    // Release 9 is a legal version path on its own, never an EmptyLogicalPath.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"META-INF/versions/9/A.class", &class(53)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 9, MultiReleasePolicy::Enabled),
    );
    let nine = entry(&report, b"META-INF/versions/9/A.class");
    assert_eq!(
        nine.variant,
        MultiReleaseEntryVariant::Versioned { release: 9 }
    );
    assert_eq!(nine.logical_path.as_ref().unwrap().0, b"A.class");
    assert_eq!(nine.decision, MultiReleaseSelectionDecision::Selected);
    assert_eq!(
        selected_name_of(&report, b"A.class"),
        b"META-INF/versions/9/A.class"
    );
    // The only finding is the missing public predecessor of `A.class`, not a path issue.
    assert_eq!(
        domain_codes(&report),
        vec![MultiReleaseDiagnosticCode::MultiReleasePublicPredecessorMissing]
    );
}

#[test]
fn duplicate_candidates_are_ambiguous_and_never_first_wins() {
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"META-INF/versions/11/p/A.class", &class(55)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
        (b"p/A.class", &class(52)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    match &selection(&report, b"p/A.class").outcome {
        MultiReleaseSelectionOutcome::Ambiguous { entries } => {
            assert_eq!(entries.len(), 2);
            assert_ne!(entries[0], entries[1]);
            assert_eq!(entries[0].ordinal, 1);
            assert_eq!(entries[1].ordinal, 2);
        }
        other => panic!("expected an ambiguous group, found {other:?}"),
    }
    assert_eq!(
        domain_codes(&report),
        vec![
            MultiReleaseDiagnosticCode::MultiReleaseDuplicateCandidate,
            MultiReleaseDiagnosticCode::MultiReleaseDuplicateCandidate,
        ]
    );
    assert_eq!(diagnostic_ordinals(&report), vec![1, 2]);
    for item in &container(&report).entries {
        if matches!(item.variant, MultiReleaseEntryVariant::Versioned { .. }) {
            assert_eq!(item.decision, MultiReleaseSelectionDecision::Ambiguous);
            assert_eq!(item.compliance, MultiReleaseCompliance::NonConformant);
        }
    }
    let base = entry(&report, b"p/A.class");
    // The shadowed base names both ambiguous winners of the duplicated v11 level.
    let ambiguous_winners: Vec<PhysicalEntryId> =
        entries_of(container(&report), b"META-INF/versions/11/p/A.class")
            .iter()
            .map(|item| item.entry.clone())
            .collect();
    assert_eq!(ambiguous_winners.len(), 2);
    assert_eq!(winners(&report, b"p/A.class"), ambiguous_winners);
    assert_eq!(base.compliance, MultiReleaseCompliance::NotApplicable);

    // A lower-level duplicate does not overturn a unique higher winner.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert_eq!(selected_name(&report), b"META-INF/versions/11/p/A.class");
    let bases = entries_of(container(&report), b"p/A.class");
    assert_eq!(bases.len(), 2);
    let unique_winner = entry(&report, b"META-INF/versions/11/p/A.class")
        .entry
        .clone();
    for base in bases {
        // Both duplicate bases are shadowed by the same real winning entry.
        assert_eq!(
            base.decision,
            MultiReleaseSelectionDecision::Shadowed {
                winners: vec![unique_winner.clone()]
            }
        );
        assert_eq!(base.compliance, MultiReleaseCompliance::NonConformant);
    }
    assert_eq!(
        entry(&report, b"META-INF/versions/11/p/A.class").compliance,
        MultiReleaseCompliance::Unknown {
            reason: MultiReleaseComplianceUnknownReason::PredecessorAmbiguous
        }
    );

    // A duplicated base level without a versioned candidate is an ambiguous selection.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"p/A.class", &class(52)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    match &selection(&report, b"p/A.class").outcome {
        MultiReleaseSelectionOutcome::Ambiguous { entries } => assert_eq!(entries.len(), 2),
        other => panic!("expected an ambiguous base, found {other:?}"),
    }
    // The duplicate-Base rule still proves a non-conformance on each base entry.
    for base in entries_of(container(&report), b"p/A.class") {
        assert_eq!(base.decision, MultiReleaseSelectionDecision::Ambiguous);
        assert_eq!(base.compliance, MultiReleaseCompliance::NonConformant);
    }
    assert_eq!(
        domain_codes(&report),
        vec![
            MultiReleaseDiagnosticCode::MultiReleaseDuplicateCandidate,
            MultiReleaseDiagnosticCode::MultiReleaseDuplicateCandidate,
        ]
    );
}

#[test]
fn class_compliance_diagnostics_do_not_rewrite_path_selection() {
    // A versioned class above release + 44 stays selected but is non-conformant.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(60)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert_eq!(selected_name(&report), b"META-INF/versions/11/p/A.class");
    let newer = entry(&report, b"META-INF/versions/11/p/A.class");
    assert_eq!(newer.compliance, MultiReleaseCompliance::NonConformant);
    assert_eq!(newer.class_evidence.as_ref().unwrap().major_version, 60);
    assert_eq!(
        domain_codes(&report),
        vec![MultiReleaseDiagnosticCode::MultiReleaseClassVersionTooNew]
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));

    // A malformed versioned class is a completed non-conformance, not an interruption.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (
            b"META-INF/versions/11/p/A.class",
            b"\xca\xfe\xba\xbe\x00\x00\x00\x37",
        ),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert_eq!(selected_name(&report), b"META-INF/versions/11/p/A.class");
    let broken = entry(&report, b"META-INF/versions/11/p/A.class");
    assert_eq!(broken.compliance, MultiReleaseCompliance::NonConformant);
    assert!(broken.class_evidence.is_none());
    assert_eq!(
        domain_codes(&report),
        vec![MultiReleaseDiagnosticCode::MultiReleaseClassMalformed]
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));

    // A public versioned class without any root predecessor.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert_eq!(selected_name(&report), b"META-INF/versions/11/p/A.class");
    assert_eq!(
        entry(&report, b"META-INF/versions/11/p/A.class").compliance,
        MultiReleaseCompliance::NonConformant
    );
    assert_eq!(
        domain_codes(&report),
        vec![MultiReleaseDiagnosticCode::MultiReleasePublicPredecessorMissing]
    );

    // A non-public root predecessor and a predecessor with another this_class.
    for (base, expected_base_ok) in [
        (class_with(52, 0x20, b"p/A"), true),
        (class_with(52, 0x21, b"q/B"), true),
    ] {
        assert!(expected_base_ok);
        let snapshot = open(zip(&[
            (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
            (b"p/A.class", &base),
            (b"META-INF/versions/11/p/A.class", &class(55)),
        ]));
        let report = select(
            &snapshot,
            &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        );
        assert_eq!(
            entry(&report, b"META-INF/versions/11/p/A.class").compliance,
            MultiReleaseCompliance::NonConformant
        );
        assert_eq!(
            domain_codes(&report),
            vec![MultiReleaseDiagnosticCode::MultiReleasePublicPredecessorMismatch]
        );
        assert_eq!(
            entry(&report, b"p/A.class").compliance,
            MultiReleaseCompliance::NotApplicable
        );
    }

    // A uniquely readable but malformed predecessor: the versioned candidate is the
    // non-conformant side, while the plain Base entry keeps its closed-table state and is
    // only reported through the diagnostic that its own bytes prove.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", b"\xca\xfe\xba\xbe\x00\x00\x00\x37"),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    let malformed_base = entry(&report, b"p/A.class");
    assert_eq!(
        malformed_base.compliance,
        MultiReleaseCompliance::NotApplicable
    );
    assert!(malformed_base.class_evidence.is_none());
    assert_eq!(
        entry(&report, b"META-INF/versions/11/p/A.class").compliance,
        MultiReleaseCompliance::NonConformant
    );
    assert_eq!(
        domain_codes(&report),
        vec![
            MultiReleaseDiagnosticCode::MultiReleaseClassMalformed,
            MultiReleaseDiagnosticCode::MultiReleasePublicPredecessorMismatch,
        ]
    );
    // The Base ordinal still counts as checked in the compliance dimension.
    assert!(is_scanned(
        container(&report),
        "multi_release_compliance_entries",
        malformed_base.entry.ordinal
    ));

    // A non-public versioned class needs no predecessor.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (
            b"META-INF/versions/11/p/A.class",
            &class_with(55, 0x20, b"p/A"),
        ),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert_eq!(
        entry(&report, b"META-INF/versions/11/p/A.class").compliance,
        MultiReleaseCompliance::ConformantWithinChecks
    );
    assert!(domain_codes(&report).is_empty());
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));

    // module-info evidence replaces missing/mismatched predecessor verdicts with Unknown.
    let versioned = class(55);
    for base in [None, Some(class_with(52, 0x20, b"p/A"))] {
        let mut fixture: Vec<(&[u8], &[u8])> = vec![
            (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
            (b"module-info.class", b"module"),
            (b"META-INF/versions/11/p/A.class", &versioned),
        ];
        if let Some(base) = &base {
            fixture.push((b"p/A.class", base));
        }
        let snapshot = open(zip(&fixture));
        let report = select(
            &snapshot,
            &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        );
        assert_eq!(
            entry(&report, b"META-INF/versions/11/p/A.class").compliance,
            MultiReleaseCompliance::Unknown {
                reason: MultiReleaseComplianceUnknownReason::ModuleExportsNotInspected
            }
        );
        assert!(domain_codes(&report).is_empty());
    }
}

#[test]
fn budget_exhaustion_is_partial_and_never_fakes_completion() {
    let snapshot = open(a06_fixture());

    // (a) An interrupted Manifest read leaves the Manifest evidence unknown.
    let mut budget = Budget::new(limits_with(10_000, 8, 1 << 24));
    let report = select_with(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        &mut budget,
    );
    assert_eq!(container(&report).manifest.state, ManifestState::Unknown);
    assert!(matches!(
        decision(&report, b"p/A.class"),
        MultiReleaseSelectionDecision::Unknown {
            reason: MultiReleaseUnknownReason::ManifestEvidenceUnreadable
        }
    ));
    assert_eq!(container(&report).entries.len(), 4);
    assert!(is_skipped(
        container(&report),
        "multi_release_selection_entries",
        0
    ));
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ReadBytes
            },
            ..
        }
    ));
    assert!(matches!(
        container(&report).execution,
        ExecutionReport::Partial { .. }
    ));
    assert_eq!(
        terminal_codes(&report),
        vec!["budget_exceeded_read_bytes".to_string()]
    );

    // (b) Exhausting the entry-evidence item budget keeps a reliable prefix.
    let mut budget = Budget::new(limits_with(8, 1 << 24, 1 << 24));
    let report = select_with(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        &mut budget,
    );
    let truncated = container(&report);
    assert_eq!(truncated.entries.len(), 2);
    assert_eq!(truncated.entries[0].entry.ordinal, 0);
    assert_eq!(truncated.entries[1].entry.ordinal, 1);
    assert!(truncated.selections.is_empty());
    assert!(matches!(
        truncated.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert_eq!(
        terminal_codes(&report),
        vec!["budget_exceeded_result_items".to_string()]
    );
    for ordinal in [2, 3] {
        assert!(is_skipped(
            truncated,
            "multi_release_selection_entries",
            ordinal
        ));
        assert!(is_skipped(
            truncated,
            "multi_release_compliance_entries",
            ordinal
        ));
    }
    match &report.physical {
        MultiReleasePhysicalEvidence::Snapshot { report: physical } => {
            assert_eq!(physical.entries.len(), 4);
        }
        other => panic!("expected snapshot evidence, found {other:?}"),
    }

    // (c) An interrupted compliance probe keeps the proven path selection.
    let mut budget = Budget::new(limits_with(10_000, 1 << 24, 4));
    let report = select_with(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        &mut budget,
    );
    assert_eq!(selected_name(&report), b"META-INF/versions/17/p/A.class");
    assert_eq!(container(&report).selections.len(), 2);
    for name in [
        b"META-INF/versions/11/p/A.class".as_slice(),
        b"META-INF/versions/17/p/A.class",
    ] {
        assert_eq!(
            entry(&report, name).compliance,
            MultiReleaseCompliance::Unknown {
                reason: MultiReleaseComplianceUnknownReason::ProbeInterrupted
            }
        );
        assert!(entry(&report, name).class_evidence.is_none());
        assert!(is_skipped(
            container(&report),
            "multi_release_compliance_entries",
            entry(&report, name).entry.ordinal
        ));
    }
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ClassBytes
            },
            ..
        }
    ));
}

#[test]
fn unsupported_policies_preserve_candidates_without_a_selected_fact() {
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    for (policy, code) in [
        (
            MultiReleasePolicy::Custom { id: "x".into() },
            "multi_release_custom_policy",
        ),
        (MultiReleasePolicy::Unknown, "multi_release_unknown_policy"),
    ] {
        let report = select(&snapshot, &view_for(&snapshot, 17, policy));
        match &report.execution {
            ExecutionReport::Failed {
                reason: TerminationReason::Unsupported { code: actual },
                ..
            } => assert_eq!(actual, code, "policy verdict code"),
            other => panic!("expected a Failed unsupported execution, found {other:?}"),
        }
        let container = container(&report);
        assert_eq!(container.entries.len(), 3);
        assert_eq!(container.manifest.state, ManifestState::Unknown);
        assert!(container.manifest.attribute_name.is_none());
        assert_eq!(container.manifest.entries.len(), 1);
        for item in &container.entries {
            let expected = match item.variant {
                MultiReleaseEntryVariant::Versioned { .. } => match code {
                    "multi_release_custom_policy" => MultiReleaseSelectionDecision::Unknown {
                        reason: MultiReleaseUnknownReason::CustomPolicy,
                    },
                    _ => MultiReleaseSelectionDecision::Unknown {
                        reason: MultiReleaseUnknownReason::UnknownPolicy,
                    },
                },
                _ => MultiReleaseSelectionDecision::Unknown {
                    reason: if code == "multi_release_custom_policy" {
                        MultiReleaseUnknownReason::CustomPolicy
                    } else {
                        MultiReleaseUnknownReason::UnknownPolicy
                    },
                },
            };
            assert_eq!(item.decision, expected);
            assert_eq!(
                item.compliance,
                MultiReleaseCompliance::Unknown {
                    reason: MultiReleaseComplianceUnknownReason::PolicyUnsupported
                }
            );
            assert!(item.class_evidence.is_none());
        }
        assert!(matches!(
            container.selections[0].outcome,
            MultiReleaseSelectionOutcome::Unknown { .. }
        ));
        // The physical provider really ran and its evidence stays complete.
        match &report.physical {
            MultiReleasePhysicalEvidence::Snapshot { report: physical } => {
                assert!(matches!(
                    physical.execution,
                    ExecutionReport::Complete { .. }
                ));
            }
            other => panic!("expected snapshot evidence, found {other:?}"),
        }
    }
}

#[test]
fn cancellation_stops_before_any_faked_selection() {
    let snapshot = open(a06_fixture());
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let report = select_with(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        &mut budget,
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert!(containers(&report).is_empty());
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    match &report.physical {
        MultiReleasePhysicalEvidence::Snapshot { report: physical } => {
            assert!(matches!(
                physical.execution,
                ExecutionReport::Cancelled { .. }
            ));
        }
        other => panic!("expected snapshot evidence, found {other:?}"),
    }
}

#[test]
fn selections_follow_the_group_minimum_ordinal_not_path_bytes() {
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"m/M.class", &class(52)),
        (b"a/A.class", &class(52)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    let paths: Vec<Vec<u8>> = container(&report)
        .selections
        .iter()
        .map(|item| item.logical_path.0.clone())
        .collect();
    assert_eq!(
        paths,
        vec![
            b"META-INF/MANIFEST.MF".to_vec(),
            b"m/M.class".to_vec(),
            b"a/A.class".to_vec(),
        ]
    );
    let ordinals: Vec<u64> = container(&report)
        .entries
        .iter()
        .map(|item| item.entry.ordinal)
        .collect();
    assert_eq!(ordinals, vec![0, 1, 2]);
    match &selection(&report, b"a/A.class").outcome {
        MultiReleaseSelectionOutcome::Selected { entry } => {
            assert_eq!(entry.raw_name.0, b"a/A.class");
        }
        other => panic!("expected a selected a/A.class, found {other:?}"),
    }
}

#[test]
fn physical_scope_controls_nested_multi_release_inspection() {
    let child = zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]);
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"lib/inner.jar", &child),
    ]));

    // ArtifactTree scope inspects root and established child separately.
    let report = select(
        &snapshot,
        &tree_view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert_eq!(containers(&report).len(), 2);
    let tree = match &report.physical {
        MultiReleasePhysicalEvidence::ArtifactTree { report } => report,
        other => panic!("expected artifact tree evidence, found {other:?}"),
    };
    assert_eq!(tree.containers.len(), 2);
    for (index, physical) in tree.containers.iter().enumerate() {
        assert_eq!(containers(&report)[index].origin, physical.origin);
        assert_eq!(
            containers(&report)[index].manifest.state,
            ManifestState::Active
        );
        assert_eq!(
            containers(&report)[index].entries.len(),
            physical.entries.len()
        );
    }
    // root: Manifest, p/A.class and lib/inner.jar; child: Manifest and p/A.class
    assert_eq!(containers(&report)[0].selections.len(), 3);
    assert_eq!(containers(&report)[1].selections.len(), 2);
    assert_eq!(selected_name(&report), b"p/A.class");
    // The child container has its own versioned winner.
    let child_selection = containers(&report)[1]
        .selections
        .iter()
        .find(|item| item.logical_path.0 == b"p/A.class")
        .unwrap();
    match &child_selection.outcome {
        MultiReleaseSelectionOutcome::Selected { entry } => {
            assert_eq!(entry.raw_name.0, b"META-INF/versions/11/p/A.class");
            assert_eq!(entry.origin.steps.len(), 1);
        }
        other => panic!("expected a selected child entry, found {other:?}"),
    }

    // SnapshotAll scope keeps the nested archive as an ordinary physical entry.
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert_eq!(containers(&report).len(), 1);
    let nested = entry(&report, b"lib/inner.jar");
    assert_eq!(nested.variant, MultiReleaseEntryVariant::Base);
    assert_eq!(nested.decision, MultiReleaseSelectionDecision::Selected);
    assert_eq!(nested.logical_path.as_ref().unwrap().0, b"lib/inner.jar");
    assert!(nested.entry.origin.steps.is_empty());
}

#[test]
fn invalid_view_inputs_are_rejected_with_stable_codes() {
    // A non-ZIP snapshot cannot be inspected for multi-release selection.
    let class_snapshot = open(class(52));
    let mut budget = Budget::new(limits());
    let error = Engine::new()
        .select_multi_release(
            &class_snapshot,
            &view_for(&class_snapshot, 17, MultiReleasePolicy::Enabled),
            &mut budget,
        )
        .unwrap_err();
    assert!(
        matches!(error, Error::InvalidInput { ref code, .. } if code == "multi_release_not_zip")
    );

    let snapshot = open(a06_fixture());
    // The view must name this very snapshot.
    let mut mismatched = view_for(&snapshot, 17, MultiReleasePolicy::Enabled);
    mismatched.physical.snapshot = SnapshotId("other".into());
    let mut budget = Budget::new(limits());
    let error = Engine::new()
        .select_multi_release(&snapshot, &mismatched, &mut budget)
        .unwrap_err();
    assert!(
        matches!(error, Error::InvalidInput { ref code, .. } if code == "multi_release_snapshot_mismatch")
    );

    // The tree scope must name the real root container.
    let wrong_root = runtime_view(
        &snapshot,
        17,
        MultiReleasePolicy::Enabled,
        PhysicalScope::ArtifactTree {
            root_container: ContainerId("nested".into()),
        },
    );
    let mut budget = Budget::new(limits());
    let error = Engine::new()
        .select_multi_release(&snapshot, &wrong_root, &mut budget)
        .unwrap_err();
    assert!(
        matches!(error, Error::InvalidInput { ref code, .. } if code == "multi_release_root_container_mismatch")
    );
}

#[test]
fn physical_evidence_is_the_single_provider_result_it_reports() {
    let snapshot = open(a06_fixture());
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    let mut budget = Budget::new(limits());
    let standalone = snapshot.enumerate(&mut budget).unwrap();
    assert_eq!(
        report.physical,
        MultiReleasePhysicalEvidence::Snapshot { report: standalone }
    );

    let child = zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]);
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"lib/inner.jar", &child),
    ]));
    let report = select(
        &snapshot,
        &tree_view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    let mut budget = Budget::new(limits());
    let standalone = snapshot.enumerate_artifact_tree(&mut budget).unwrap();
    assert_eq!(
        report.physical,
        MultiReleasePhysicalEvidence::ArtifactTree { report: standalone }
    );
}

#[test]
fn report_json_round_trips_and_mr_owned_types_are_closed() {
    let snapshot = open(zip(&[
        (b"meta-inf/manifest.mf", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert_eq!(
        domain_codes(&report),
        vec![MultiReleaseDiagnosticCode::MultiReleaseManifestNoncanonicalPath]
    );
    let value = serde_json::to_value(&report).unwrap();
    let back: MultiReleaseViewReport = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(back, report);

    let mut closed = value.clone();
    closed["containers"][0]["manifest"]["unexpected"] = serde_json::json!(1);
    assert!(serde_json::from_value::<MultiReleaseViewReport>(closed).is_err());
    let mut closed = value.clone();
    closed["containers"][0]["entries"][0]["unexpected"] = serde_json::json!(1);
    assert!(serde_json::from_value::<MultiReleaseViewReport>(closed).is_err());
    let mut closed = value.clone();
    closed["diagnostics"][0]["diagnostic"]["unexpected"] = serde_json::json!(1);
    assert!(serde_json::from_value::<MultiReleaseViewReport>(closed).is_err());
    // The MR-owned internally tagged enums are closed as well: an unknown field next to the
    // tag is refused instead of being silently dropped.
    let mut closed = value.clone();
    closed["physical"]["unexpected"] = serde_json::json!(1);
    assert!(
        serde_json::from_value::<MultiReleaseViewReport>(closed).is_err(),
        "physical evidence accepted an unknown field"
    );
    let mut closed = value.clone();
    closed["containers"][0]["selections"][0]["outcome"]["unexpected"] = serde_json::json!(1);
    assert!(
        serde_json::from_value::<MultiReleaseViewReport>(closed).is_err(),
        "selection outcome accepted an unknown field"
    );
    // Same for an enum variant that carries fields: the versioned entry of this fixture.
    let mut closed = value.clone();
    closed["containers"][0]["entries"][2]["variant"]["unexpected"] = serde_json::json!(1);
    assert_eq!(
        closed["containers"][0]["entries"][2]["variant"]["kind"],
        serde_json::json!("versioned")
    );
    assert!(
        serde_json::from_value::<MultiReleaseViewReport>(closed).is_err(),
        "entry variant accepted an unknown field"
    );
    // A unit variant carries no struct fields, so an internal tag has nothing to deny there;
    // that serde boundary is recorded here rather than silently assumed to be closed.
    let mut tolerated = value.clone();
    tolerated["containers"][0]["entries"][0]["variant"]["unexpected"] = serde_json::json!(1);
    assert_eq!(
        tolerated["containers"][0]["entries"][0]["variant"]["kind"],
        serde_json::json!("base")
    );
    assert!(
        serde_json::from_value::<MultiReleaseViewReport>(tolerated).is_ok(),
        "serde no longer ignores extra keys on unit variants: tighten this test"
    );
    let mut closed = value;
    closed["diagnostics"][0]["diagnostic"]["code"] = serde_json::json!("future_code");
    assert!(serde_json::from_value::<MultiReleaseViewReport>(closed).is_err());

    // A report whose diagnostics include a terminal entry round-trips too.
    let snapshot = open(a06_fixture());
    let mut budget = Budget::new(limits_with(10_000, 8, 1 << 24));
    let interrupted = select_with(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        &mut budget,
    );
    assert_eq!(
        terminal_codes(&interrupted),
        vec!["budget_exceeded_read_bytes".to_string()]
    );
    let value = serde_json::to_value(&interrupted).unwrap();
    let back: MultiReleaseViewReport = serde_json::from_value(value).unwrap();
    assert_eq!(back, interrupted);

    // Closed outcome/reason enums reject unknown variants and values.
    assert!(
        serde_json::from_value::<MultiReleaseSelectionDecision>(
            serde_json::json!({"kind": "future"})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<MultiReleaseSelectionOutcome>(
            serde_json::json!({"kind": "future"})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<MultiReleaseUnknownReason>(serde_json::json!("future")).is_err()
    );
}

#[test]
fn container_reservation_failure_declares_every_known_ordinal_skipped() {
    let snapshot = open(a06_fixture());
    // 4 physical entries consume 4 result items, so the 2-item container reservation fails.
    let mut budget = Budget::new(limits_with(5, 1 << 24, 1 << 24));
    let report = select_with(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        &mut budget,
    );
    assert!(containers(&report).is_empty());
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert_eq!(
        terminal_codes(&report),
        vec!["budget_exceeded_result_items".to_string()]
    );
    let coverage = &report.coverage.runtime_resolution;
    for ordinal in 0..4 {
        assert!(skipped_in(
            coverage,
            "multi_release_selection_entries",
            ordinal
        ));
    }
    for ordinal in [2, 3] {
        assert!(skipped_in(
            coverage,
            "multi_release_compliance_entries",
            ordinal
        ));
    }
    match &report.physical {
        MultiReleasePhysicalEvidence::Snapshot { report: physical } => {
            assert_eq!(physical.entries.len(), 4);
            assert!(matches!(
                physical.execution,
                ExecutionReport::Complete { .. }
            ));
        }
        other => panic!("expected snapshot evidence, found {other:?}"),
    }
}

#[test]
fn container_without_enumerated_ordinals_produces_no_manifest_claim() {
    // A root container that enumerated no ordinal at all gets no MR report.
    let snapshot = open(a06_fixture());
    let mut starved = limits();
    starved.archive_entries = 0;
    let mut budget = Budget::new(starved);
    let report = select_with(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        &mut budget,
    );
    assert!(containers(&report).is_empty());
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ArchiveEntries
            },
            ..
        }
    ));
    // The physical suffix is passed through with its label and bounds unchanged.
    assert_eq!(
        report.coverage.runtime_resolution.skipped,
        vec![CoverageRange {
            label: "central_directory_entries".into(),
            start: 0,
            end: 4,
        }]
    );

    // An established child that enumerated no ordinal is only physical evidence.
    let child = zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]);
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"lib/inner.jar", &child),
    ]));
    // Three root entries plus the three central-directory entries replayed while locating
    // lib/inner.jar exhaust the archive-entry budget before the child's first entry.
    let mut starved = limits();
    starved.archive_entries = 6;
    let mut budget = Budget::new(starved);
    let report = select_with(
        &snapshot,
        &tree_view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        &mut budget,
    );
    let tree = match &report.physical {
        MultiReleasePhysicalEvidence::ArtifactTree { report } => report,
        other => panic!("expected artifact tree evidence, found {other:?}"),
    };
    assert_eq!(tree.containers.len(), 2);
    assert!(tree.containers[1].entries.is_empty());
    assert!(matches!(
        tree.containers[1].execution,
        ExecutionReport::Partial { .. }
    ));
    // The child was established from the nested archive, so its failure stays visible there.
    assert!(tree.layout_nodes.iter().any(|node| matches!(
        node.source,
        LayoutNodeSource::Archive {
            child_container: Some(_),
            ..
        }
    )));
    // Only the root container, with its three enumerated ordinals, produces an MR report;
    // the shared archive-entry budget also interrupted the root Manifest read.
    assert_eq!(containers(&report).len(), 1);
    assert_eq!(containers(&report)[0].origin, tree.containers[0].origin);
    assert_eq!(container(&report).entries.len(), 3);
    assert_eq!(container(&report).manifest.state, ManifestState::Unknown);
    let skipped = &report.coverage.runtime_resolution.skipped;
    let child_skipped = tree.containers[1]
        .coverage
        .artifact_structural
        .skipped
        .clone();
    assert_eq!(child_skipped.len(), 1);
    assert_eq!(child_skipped[0].start, 0);
    assert_eq!(child_skipped[0].end, 3);
    assert!(
        child_skipped[0]
            .label
            .ends_with(":central_directory_entries")
    );
    // The unprocessed child's physical suffix is passed through verbatim.
    assert!(skipped.contains(&child_skipped[0]));
    assert!(skipped.contains(&CoverageRange {
        label: "container:root:multi_release_selection_entries".into(),
        start: 0,
        end: 1,
    }));
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ArchiveEntries
            },
            ..
        }
    ));
}

#[test]
fn empty_container_completes_without_entries_or_manifest() {
    let snapshot = open(zip(&[]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    assert_eq!(containers(&report).len(), 1);
    assert_eq!(container(&report).manifest.state, ManifestState::Missing);
    assert!(container(&report).entries.is_empty());
    assert!(container(&report).selections.is_empty());
    assert!(matches!(
        container(&report).execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(
        dimension(container(&report)).state,
        CoverageState::CompleteWithinSchema
    );
    assert!(dimension(container(&report)).scanned.is_empty());
    assert!(dimension(container(&report)).skipped.is_empty());
}

#[test]
fn selection_item_exhaustion_keeps_the_proven_prefix_and_stops() {
    let snapshot = open(a06_fixture());
    // 4 enumeration + 2 reservation + 4 entry evidence + 1 selection fit in 11 items.
    let mut budget = Budget::new(limits_with(11, 1 << 24, 1 << 24));
    let report = select_with(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        &mut budget,
    );
    let truncated = container(&report);
    // Every entry evidence was charged, but the p/A.class selection could not be.
    assert_eq!(truncated.entries.len(), 4);
    assert_eq!(truncated.selections.len(), 1);
    assert_eq!(
        truncated.selections[0].logical_path.0,
        b"META-INF/MANIFEST.MF"
    );
    assert!(matches!(
        truncated.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert_eq!(
        terminal_codes(&report),
        vec!["budget_exceeded_result_items".to_string()]
    );
    // The group whose selection item was refused is not claimed as scanned path evidence.
    assert!(is_scanned(truncated, "multi_release_selection_entries", 0));
    for ordinal in [1, 2, 3] {
        assert!(is_skipped(
            truncated,
            "multi_release_selection_entries",
            ordinal
        ));
    }
    // The probe never ran, so the applicable candidates are declared unprobed.
    for ordinal in [2, 3] {
        assert!(is_skipped(
            truncated,
            "multi_release_compliance_entries",
            ordinal
        ));
    }
}

#[test]
fn artifact_tree_keeps_reports_for_children_the_physical_aggregate_already_stopped() {
    // root: p/A.class, lib/a.jar (fully enumerable) and lib/b.jar, which runs out of the
    // archive-entry budget after its first entry. The physical aggregate therefore stops,
    // but the MR run must still inspect every container whose ordinals it saw.
    let child_a = zip(&[(b"p/A.class", &class(52))]);
    let child_b = zip(&[
        (b"q/B.class", &class(52)),
        (b"r/C.class", &class(52)),
        (b"s/D.class", &class(52)),
    ]);
    let snapshot = open(zip(&[
        (b"p/A.class", &class(52)),
        (b"lib/a.jar", &child_a),
        (b"lib/b.jar", &child_b),
    ]));
    let mut starved = limits();
    // 3 root entries + 2 + 3 replay charges + 1 + 1 enumerated entries: the 11th is refused.
    starved.archive_entries = 10;
    let mut budget = Budget::new(starved);
    let report = select_with(
        &snapshot,
        &tree_view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        &mut budget,
    );
    let tree = match &report.physical {
        MultiReleasePhysicalEvidence::ArtifactTree { report } => report,
        other => panic!("expected artifact tree evidence, found {other:?}"),
    };
    assert_eq!(tree.containers.len(), 3);
    assert!(matches!(
        tree.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ArchiveEntries
            },
            ..
        }
    ));
    assert_eq!(tree.containers[0].entries.len(), 3);
    assert_eq!(tree.containers[1].entries.len(), 1);
    assert_eq!(tree.containers[2].entries.len(), 1);
    assert!(matches!(
        tree.containers[2].execution,
        ExecutionReport::Partial { .. }
    ));

    // All three established containers still get their own MR report.
    assert_eq!(containers(&report).len(), 3);
    for (index, physical) in tree.containers.iter().enumerate() {
        assert_eq!(containers(&report)[index].origin, physical.origin);
    }
    assert!(matches!(
        containers(&report)[0].execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(containers(&report)[0].selections.len(), 3);
    // The fully enumerated child keeps its complete selection facts.
    let child_a_report = &containers(&report)[1];
    assert_eq!(child_a_report.origin, tree.containers[1].origin);
    assert!(matches!(
        child_a_report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(child_a_report.manifest.state, ManifestState::Missing);
    assert_eq!(child_a_report.entries.len(), 1);
    assert_eq!(child_a_report.selections.len(), 1);
    assert_eq!(
        child_a_report.selections[0].outcome,
        MultiReleaseSelectionOutcome::Selected {
            entry: tree.containers[1].entries[0].id.clone()
        }
    );
    assert!(!child_a_report.origin.steps.is_empty());

    // The partially enumerated child reports what it saw, without inventing a fallback.
    let child_b_report = &containers(&report)[2];
    assert_eq!(child_b_report.origin, tree.containers[2].origin);
    assert!(matches!(
        child_b_report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ArchiveEntries
            },
            ..
        }
    ));
    assert_eq!(child_b_report.entries.len(), 1);
    assert_eq!(
        child_b_report.entries[0].decision,
        MultiReleaseSelectionDecision::Unknown {
            reason: MultiReleaseUnknownReason::PhysicalEvidenceIncomplete
        }
    );
    assert_eq!(
        child_b_report.entries[0].compliance,
        MultiReleaseCompliance::Unknown {
            reason: MultiReleaseComplianceUnknownReason::PhysicalEvidenceIncomplete
        }
    );
    assert_eq!(
        child_b_report.selections,
        vec![MultiReleaseSelection {
            logical_path: ArchiveNameBytes(b"q/B.class".to_vec()),
            outcome: MultiReleaseSelectionOutcome::Unknown {
                reason: MultiReleaseUnknownReason::PhysicalEvidenceIncomplete
            },
        }]
    );
    // The ordinal it never saw stays skipped with its physical label and bounds.
    assert_eq!(
        child_b_report.coverage.artifact_structural.skipped,
        vec![CoverageRange {
            label: format!(
                "container:{}:central_directory_entries",
                container_id(&child_b_report.origin)
            ),
            start: 1,
            end: 3,
        }]
    );
    // The aggregate still merges the physical provider's own execution reason.
    assert_eq!(
        std::mem::discriminant(&report.execution),
        std::mem::discriminant(&tree.execution)
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ArchiveEntries
            },
            ..
        }
    ));
}

#[test]
fn items_exhausted_during_root_processing_declare_the_child_ordinals_skipped() {
    let child = zip(&[
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]);
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"lib/inner.jar", &child),
    ]));
    // 1 + 3 + 1 candidate + 1 + 2 enumeration items, then the root reservation (2), its three
    // entry evidence items and one of its selections: the second selection is refused.
    let mut starved = limits();
    starved.result_items = 14;
    let mut budget = Budget::new(starved);
    let report = select_with(
        &snapshot,
        &tree_view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        &mut budget,
    );
    let tree = match &report.physical {
        MultiReleasePhysicalEvidence::ArtifactTree { report } => report,
        other => panic!("expected artifact tree evidence, found {other:?}"),
    };
    assert_eq!(tree.containers.len(), 2);
    assert_eq!(tree.containers[1].entries.len(), 2);
    assert!(matches!(
        tree.containers[1].execution,
        ExecutionReport::Complete { .. }
    ));
    // The root was inspected and then stopped, so only the root has an MR report.
    assert_eq!(containers(&report).len(), 1);
    assert_eq!(containers(&report)[0].origin, tree.containers[0].origin);
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert_eq!(
        terminal_codes(&report),
        vec!["budget_exceeded_result_items".to_string()]
    );

    // Every ordinal the unprocessed child did enumerate is declared skipped in both MR
    // dimensions, on top of that container's own physical coverage.
    let child_id = container_id(&tree.containers[1].origin);
    for (ordinal, compliance) in [(0_u64, false), (1, true)] {
        let labels = aggregate_skipped_labels(&report, ordinal);
        assert!(
            labels.contains(&format!(
                "container:{child_id}:multi_release_selection_entries"
            )),
            "selection range missing for child ordinal {ordinal}: {labels:?}"
        );
        let has_compliance = labels.contains(&format!(
            "container:{child_id}:multi_release_compliance_entries"
        ));
        assert_eq!(
            has_compliance, compliance,
            "compliance range presence wrong for child ordinal {ordinal}: {labels:?}"
        );
    }
    assert!(
        report
            .coverage
            .runtime_resolution
            .scanned
            .iter()
            .all(|range| !range.label.contains(&child_id)),
        "the unprocessed child must not claim scanned coverage"
    );
}

#[test]
fn interrupted_predecessor_probe_keeps_base_not_applicable() {
    // The class-byte budget covers exactly the versioned Header probe, so the Base
    // predecessor check cannot finish even though the entry itself is readable.
    let versioned = class(55);
    assert_eq!(versioned.len(), class(52).len());
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &versioned),
    ]));
    let mut starved = limits();
    starved.class_bytes = versioned.len() as u64;
    let mut budget = Budget::new(starved);
    let report = select_with(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        &mut budget,
    );
    // The path rules are still fully proven.
    assert_eq!(selected_name(&report), b"META-INF/versions/11/p/A.class");
    assert_winner_is_selected(&report, b"p/A.class", b"META-INF/versions/11/p/A.class");
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ClassBytes
            },
            ..
        }
    ));
    // The versioned candidate's own Header probe ran, but its compliance verdict did not.
    let candidate = entry(&report, b"META-INF/versions/11/p/A.class");
    assert_eq!(
        candidate.compliance,
        MultiReleaseCompliance::Unknown {
            reason: MultiReleaseComplianceUnknownReason::ProbeInterrupted
        }
    );
    assert_eq!(candidate.class_evidence.as_ref().unwrap().major_version, 55);
    assert!(matches!(
        candidate.decision,
        MultiReleaseSelectionDecision::Selected
    ));
    assert!(is_skipped(
        container(&report),
        "multi_release_compliance_entries",
        candidate.entry.ordinal
    ));
    // The interruption is reported against the predecessor entry it happened on.
    assert_eq!(
        terminal_codes(&report),
        vec!["budget_exceeded_class_bytes".to_string()]
    );
    let terminal_ordinal = report
        .diagnostics
        .iter()
        .find_map(|item| match item {
            MultiReleaseReportDiagnostic::Terminal { diagnostic } => {
                match diagnostic.provenance.as_ref().map(|value| &value.location) {
                    Some(Location::Entry { id, .. }) => Some(id.ordinal),
                    _ => None,
                }
            }
            MultiReleaseReportDiagnostic::Domain { .. } => None,
        })
        .unwrap();
    assert_eq!(terminal_ordinal, entry(&report, b"p/A.class").entry.ordinal);
    // The Base entry is neither a violation nor a bounded compliance check target in the
    // closed table, yet the check that was attempted on it must still be visible.
    assert_eq!(
        entry(&report, b"p/A.class").compliance,
        MultiReleaseCompliance::NotApplicable
    );
    let base_ordinal = entry(&report, b"p/A.class").entry.ordinal;
    assert!(
        is_skipped(
            container(&report),
            "multi_release_compliance_entries",
            base_ordinal
        ),
        "an attempted predecessor probe must leave a skipped compliance range"
    );
    assert!(!is_scanned(
        container(&report),
        "multi_release_compliance_entries",
        base_ordinal
    ));
    assert!(domain_codes(&report).is_empty());
}

#[test]
fn item_exhaustion_refuses_later_domain_diagnostics_and_evidence() {
    let v11 = b"\xca\xfe\xba\xbe\x00\x00\x00\x37".to_vec();
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &v11),
        (b"META-INF/versions/17/p/A.class", &class(61)),
    ]));
    // 4 enumeration + 2 reservation + 4 entry evidence + 2 selections exhaust the budget, so
    // the class non-conformance the probe proves can no longer be reported as an item.
    let mut starved = limits();
    starved.result_items = 12;
    let mut budget = Budget::new(starved);
    let report = select_with(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        &mut budget,
    );
    let container = container(&report);
    assert_eq!(container.entries.len(), 4);
    assert_eq!(container.selections.len(), 2);
    assert!(domain_codes(&report).is_empty());
    assert_eq!(
        terminal_codes(&report),
        vec!["budget_exceeded_result_items".to_string()]
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    // The malformed class bytes still prove a non-conformance, but nothing was fabricated
    // for the candidate that was never probed.
    assert_eq!(
        entry(&report, b"META-INF/versions/11/p/A.class").compliance,
        MultiReleaseCompliance::NonConformant
    );
    assert!(
        entry(&report, b"META-INF/versions/11/p/A.class")
            .class_evidence
            .is_none()
    );
    assert_eq!(
        entry(&report, b"META-INF/versions/17/p/A.class").compliance,
        MultiReleaseCompliance::Unknown {
            reason: MultiReleaseComplianceUnknownReason::ProbeInterrupted
        }
    );
    assert!(is_skipped(
        container,
        "multi_release_compliance_entries",
        entry(&report, b"META-INF/versions/17/p/A.class")
            .entry
            .ordinal
    ));
    // Selection facts were fully proven before the probe ran.
    assert_eq!(selected_name(&report), b"META-INF/versions/17/p/A.class");
}

#[test]
fn truncated_physical_enumeration_after_the_manifest_keeps_its_decision_evidence() {
    // The Manifest ordinal is enumerated but the central directory stops right after the
    // first class entry, so the Manifest was never read while its decision evidence exists.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let mut starved = limits();
    starved.archive_entries = 2;
    let mut budget = Budget::new(starved);
    let report = select_with(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
        &mut budget,
    );
    let container = container(&report);
    assert_eq!(container.entries.len(), 2);
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ArchiveEntries
            },
            ..
        }
    ));
    assert!(matches!(
        container.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ArchiveEntries
            },
            ..
        }
    ));
    // The Manifest candidate is enumerated, so its path/Manifest decision evidence counts
    // as scanned even though the Manifest bytes could not be read.
    let manifest_ordinal = entry(&report, b"META-INF/MANIFEST.MF").entry.ordinal;
    assert_eq!(manifest_ordinal, 0);
    assert!(is_scanned(
        container,
        "multi_release_selection_entries",
        manifest_ordinal
    ));
    assert!(!is_skipped(
        container,
        "multi_release_selection_entries",
        manifest_ordinal
    ));
    let manifest = &container.manifest;
    assert_eq!(manifest.state, ManifestState::Unknown);
    assert_eq!(manifest.entries.len(), 1);
    assert_eq!(manifest.entries[0].ordinal, manifest_ordinal);
    assert!(manifest.attribute_name.is_none());
    assert!(manifest.attribute_value.is_none());
    // No logical group may claim a fallback while the suffix is unknown.
    for logical in [b"p/A.class".as_slice(), b"META-INF/MANIFEST.MF"] {
        assert_eq!(
            selection(&report, logical).outcome,
            MultiReleaseSelectionOutcome::Unknown {
                reason: MultiReleaseUnknownReason::PhysicalEvidenceIncomplete
            },
            "{logical:?}"
        );
    }
    // The ordinal that was never enumerated stays declared by the physical provider; the
    // enumerated ones keep their decision evidence.
    assert!(is_scanned(container, "multi_release_selection_entries", 1));
    assert_eq!(
        container.coverage.artifact_structural.skipped,
        vec![CoverageRange {
            label: "central_directory_entries".into(),
            start: 2,
            end: 3,
        }]
    );
}

#[test]
fn empty_manifest_is_malformed_and_never_activates_multi_release() {
    // An empty Manifest has no terminated main section. This implementation treats an
    // unterminated header as Malformed, which is stricter than the JDK parser's tolerance
    // for a missing trailing blank line: no attribute can be read, so no activation may be
    // asserted. The candidate stays physical evidence under `Unknown`.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", b""),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let report = select(
        &snapshot,
        &view_for(&snapshot, 17, MultiReleasePolicy::Enabled),
    );
    let manifest = &container(&report).manifest;
    assert_eq!(manifest.state, ManifestState::Malformed);
    assert_eq!(manifest.entries.len(), 1);
    assert!(manifest.attribute_name.is_none());
    assert!(manifest.attribute_value.is_none());
    assert_eq!(
        domain_codes(&report),
        vec![MultiReleaseDiagnosticCode::MultiReleaseManifestMalformed]
    );
    // A malformed main section is not activation evidence for any group, the base included.
    for logical in [b"p/A.class".as_slice(), b"META-INF/MANIFEST.MF"] {
        assert_eq!(
            selection(&report, logical).outcome,
            MultiReleaseSelectionOutcome::Unknown {
                reason: MultiReleaseUnknownReason::ManifestEvidenceMalformed
            },
            "{logical:?}"
        );
    }
    assert_eq!(
        decision(&report, b"META-INF/versions/11/p/A.class"),
        MultiReleaseSelectionDecision::Unknown {
            reason: MultiReleaseUnknownReason::ManifestEvidenceMalformed
        }
    );
    // A proven malformed Manifest is a complete domain result, not an interrupted one.
    assert!(matches!(
        container(&report).execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(
        dimension(container(&report)).state,
        CoverageState::CompleteWithinSchema
    );
}

#[test]
fn interrupted_item_budget_outranks_the_custom_policy_verdict() {
    // A Custom policy would report `Failed{multi_release_custom_policy}`, but the run first
    // runs out of result items while emitting its duplicate diagnostics: a blocking budget
    // error is the reason the probe never ran, so it must not be masked by the policy code.
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/A.class", &class(52)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
        (b"META-INF/versions/11/p/A.class", &class(55)),
    ]));
    let mut starved = limits();
    starved.result_items = 8;
    let mut budget = Budget::new(starved);
    let report = select_with(
        &snapshot,
        &view_for(
            &snapshot,
            17,
            MultiReleasePolicy::Custom {
                id: "custom".into(),
            },
        ),
        &mut budget,
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert!(
        !matches!(
            report.execution,
            ExecutionReport::Failed {
                reason: TerminationReason::Unsupported { .. },
                ..
            }
        ),
        "the policy verdict masked the blocking budget error: {:?}",
        report.execution
    );
    // The refused diagnostic is what stopped the run, and it is still reported as terminal.
    assert_eq!(
        terminal_codes(&report),
        vec!["budget_exceeded_result_items".to_string()]
    );
    assert_eq!(
        domain_codes(&report),
        vec![MultiReleaseDiagnosticCode::MultiReleaseDuplicateCandidate]
    );
    let container = container(&report);
    assert!(matches!(
        container.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    // The physical provider still returned every candidate it enumerated.
    match &report.physical {
        MultiReleasePhysicalEvidence::Snapshot { report: physical } => {
            assert_eq!(physical.entries.len(), 4);
            assert!(matches!(
                physical.execution,
                ExecutionReport::Complete { .. }
            ));
        }
        other => panic!("expected snapshot evidence, found {other:?}"),
    }
    // Nothing was fabricated for the ordinals the item budget could no longer cover.
    assert!(container.entries.is_empty());
    assert!(container.selections.is_empty());
    for ordinal in 0..4 {
        assert!(is_skipped(
            container,
            "multi_release_selection_entries",
            ordinal
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
