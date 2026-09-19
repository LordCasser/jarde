//! `P4` 2.1: the runtime matrix — one physical scan, several profiles, and the A06/A07 evidence.
//!
//! Three properties are asserted here from the outside: the same snapshot answered for Java
//! 8/11/17 without any view standing for the others, every physical entry still listed whether or
//! not a profile selected it, and one enumeration serving the whole batch.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const ACTIVE_MANIFEST: &[u8] = b"Manifest-Version: 1.0\r\nMulti-Release: true\r\n\r\n";
const INACTIVE_MANIFEST: &[u8] = b"Manifest-Version: 1.0\r\n\r\n";

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
        class_headers: 10_000,
        method_bodies: 10_000,
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

/// Minimal, complete public class file with an explicit major version and internal name.
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

fn class_named(major: u16, name: &[u8]) -> Vec<u8> {
    class_with(major, 0x21, name)
}

/// The class the multi-release fixtures publish: the same `p/Join.class` the frozen P1 generator
/// (`mr_fixture` in `tests/p1_xref_golden.rs`, recorded in `tests/fixtures/README.md`) uses at the
/// same three releases.
fn class(major: u16) -> Vec<u8> {
    class_named(major, b"p/Join")
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

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    let mut budget = Budget::new(limits());
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .unwrap()
}

/// A jar whose root, `versions/11` and `versions/17` all publish `p/Join.class`.
///
/// This is the shape the frozen P1 language already describes — `tests/fixtures/README.md`
/// records `multi-release.json`'s `mr_fixture` as "base/v11/v17 `p/Join.class`" with Java
/// 8/11/17 selections — rebuilt here because that generator is test-local to
/// `tests/p1_xref_golden.rs` and the fixture files are reports, not archive bytes. The releases
/// (52/55/61) and the manifest bytes are the same.
fn divergent_jar() -> Vec<u8> {
    zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"p/Join.class", &class(52)),
        (b"META-INF/versions/11/p/Join.class", &class(55)),
        (b"META-INF/versions/17/p/Join.class", &class(61)),
    ])
}

fn profile(release: u16) -> RuntimeProfile {
    RuntimeProfile {
        java_release: release,
        multi_release: MultiReleasePolicy::Enabled,
        layout: LayoutMode::Generic,
    }
}

fn domain(loader: &str, roots: Vec<LoadRoot>, delegation: DelegationPolicy) -> LoadDomain {
    LoadDomain {
        loader: LoaderId(loader.into()),
        parent_loader: None,
        delegation,
        roots,
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    }
}

fn snapshot_root(snapshot: &ArtifactSnapshot) -> LoadRoot {
    LoadRoot::Snapshot {
        snapshot: snapshot.id().clone(),
    }
}

fn request(
    snapshot: &ArtifactSnapshot,
    profiles: Vec<RuntimeProfile>,
    domains: Vec<LoadDomain>,
) -> RuntimeMatrixRequest {
    RuntimeMatrixRequest {
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        profiles,
        domains,
        requester: LoaderId("app".into()),
    }
}

fn matrix(snapshot: &ArtifactSnapshot, request: &RuntimeMatrixRequest) -> RuntimeMatrix {
    let mut budget = Budget::new(limits());
    Engine::new()
        .runtime_matrix(snapshot, request, &mut budget)
        .unwrap()
}

fn definition_of<'a>(
    matrix: &'a RuntimeMatrix,
    index: usize,
    path: &[u8],
) -> &'a RuntimeMatrixDefinition {
    matrix.profiles[index]
        .definition(path)
        .unwrap_or_else(|| panic!("profile {index} has no definition for {:?}", path))
}

fn selected_entry(definition: &RuntimeMatrixDefinition) -> PhysicalEntryId {
    definition
        .selected()
        .expect("the fixture selects an entry")
        .entry
        .clone()
}

fn entry_named(definition: &RuntimeMatrixDefinition, name: &[u8]) -> PhysicalEntryId {
    definition
        .candidates()
        .into_iter()
        .find(|candidate| candidate.entry.raw_name.0 == name)
        .unwrap_or_else(|| panic!("no candidate named {:?}", name))
        .entry
        .clone()
}

/// A06: three profiles over one snapshot answer differently, and none of them stands for the rest.
#[test]
fn three_profiles_select_different_entries_and_keep_every_physical_entry() {
    let snapshot = open(divergent_jar());
    let request = request(
        &snapshot,
        vec![profile(8), profile(11), profile(17)],
        vec![domain(
            "app",
            vec![snapshot_root(&snapshot)],
            DelegationPolicy::ParentFirst,
        )],
    );
    let matrix = matrix(&snapshot, &request);

    // The one scan is the whole "physical 全量": manifest plus three class entries.
    assert_eq!(matrix.scan.physical_scans, 1);
    assert_eq!(matrix.physical_entries().len(), 4);
    for index in 0..3 {
        assert_eq!(matrix.profiles[index].containers.len(), 1);
        assert_eq!(matrix.profiles[index].containers[0].entries.len(), 4);
    }

    // Java 8 stays on the base entry and says why.
    let eight = definition_of(&matrix, 0, b"p/Join.class");
    assert_eq!(
        eight.origins[0].rule,
        RuntimeSelectionRule::BaseSelected {
            reason: BaseSelectionReason::TargetBelowNine { target: 8 }
        }
    );
    assert_eq!(
        selected_entry(eight),
        entry_named(eight, b"p/Join.class"),
        "java 8 selects the root entry"
    );

    // Java 11 and 17 select their own versioned entries.
    let eleven = definition_of(&matrix, 1, b"p/Join.class");
    assert_eq!(
        eleven.origins[0].rule,
        RuntimeSelectionRule::HighestReleaseAtOrBelowTarget {
            release: 11,
            target: 11
        }
    );
    assert_eq!(
        selected_entry(eleven),
        entry_named(eleven, b"META-INF/versions/11/p/Join.class")
    );
    let seventeen = definition_of(&matrix, 2, b"p/Join.class");
    assert_eq!(
        seventeen.origins[0].rule,
        RuntimeSelectionRule::HighestReleaseAtOrBelowTarget {
            release: 17,
            target: 17
        }
    );
    assert_eq!(
        selected_entry(seventeen),
        entry_named(seventeen, b"META-INF/versions/17/p/Join.class")
    );

    // Three different answers, all present at once: the matrix never collapses to the last one.
    let answers: Vec<PhysicalEntryId> = [eight, eleven, seventeen]
        .iter()
        .map(|definition| selected_entry(definition))
        .collect();
    assert_eq!(
        answers,
        vec![
            entry_named(eight, b"p/Join.class"),
            entry_named(eleven, b"META-INF/versions/11/p/Join.class"),
            entry_named(seventeen, b"META-INF/versions/17/p/Join.class"),
        ]
    );

    // Every unselected physical entry is still listed, with the decision that excluded it.
    for (definition, unselected) in [(eight, 2_usize), (eleven, 2), (seventeen, 2)] {
        assert_eq!(definition.candidates().len(), 3);
        assert_eq!(definition.unselected().len(), unselected);
    }
    for candidate in eight.unselected() {
        assert!(
            matches!(
                candidate.decision,
                MultiReleaseSelectionDecision::Inactive { .. }
            ),
            "java 8 neither selects nor silently drops {:?}: {:?}",
            candidate.entry.raw_name,
            candidate.decision
        );
    }
    for candidate in seventeen.unselected() {
        assert_ne!(candidate.entry, selected_entry(seventeen));
    }
    assert_eq!(
        entry_named(eight, b"META-INF/versions/17/p/Join.class")
            .raw_name
            .0,
        b"META-INF/versions/17/p/Join.class"
    );

    // Each profile is its own group: the three answers differ, so no compression applies, and
    // the groups partition the profiles.
    assert_eq!(matrix.groups.len(), 3);
    let mut covered: Vec<usize> = matrix
        .groups
        .iter()
        .flat_map(|group| group.members.clone())
        .collect();
    covered.sort_unstable();
    assert_eq!(covered, vec![0, 1, 2]);
    for index in 0..3 {
        let group = matrix.group_of(index);
        assert_eq!(group.members, vec![index]);
        assert!(matches!(group.reason, GroupReason::SingleStableInterval));
    }
    let eight_group = matrix.group_of(0);
    assert_eq!(
        eight_group.stable_interval,
        Some(StableInterval {
            lowest_release: 0,
            highest_release: Some(8)
        })
    );
    assert_eq!(
        matrix.group_of(1).stable_interval,
        Some(StableInterval {
            lowest_release: 11,
            highest_release: Some(16)
        })
    );
    assert_eq!(
        matrix.group_of(2).stable_interval,
        Some(StableInterval {
            lowest_release: 17,
            highest_release: None
        })
    );

    // The reuse is visible: each view keeps the manifest evidence and the per-entry decisions.
    assert_eq!(
        matrix.profiles[0].containers[0].manifest.state,
        ManifestState::Active
    );
    assert!(matches!(
        matrix.profiles[1].execution,
        ExecutionReport::Complete { .. }
    ));
}

/// The shared scan: one enumeration for three profiles, measured against per-profile runs.
#[test]
fn one_scan_serves_three_profiles_and_costs_a_third_of_the_scans() {
    let snapshot = open(divergent_jar());
    let profiles = vec![profile(8), profile(11), profile(17)];
    let request = request(
        &snapshot,
        profiles.clone(),
        vec![domain(
            "app",
            vec![snapshot_root(&snapshot)],
            DelegationPolicy::ParentFirst,
        )],
    );
    let matrix = matrix(&snapshot, &request);

    // The enumeration of this physical view, measured on its own.
    let one_scan = one_physical_scan(&snapshot, &profiles[0], &request);
    let shared = matrix.scan.shared_scan;
    assert_eq!(shared, one_scan);
    // Four entries: the manifest and the three class entries. The enumeration of a stored
    // in-memory archive reads its directory, not its data.
    assert_eq!(shared.archive_entries, 4);
    assert_eq!(shared.result_items, 4);
    assert_eq!(shared.read_bytes, 0);

    // Baseline: the same three profiles, each reading the physical view for itself.
    let baseline = standalone_total(&snapshot, &profiles, &request);

    // The matrix is exactly one enumeration plus the three selections: the measured totals of the
    // three standalone runs are the matrix's numbers with the scan counted three times.
    assert_eq!(
        scan_times_three_plus_profiles(&matrix),
        baseline,
        "measured matrix cost against three standalone runs"
    );
    assert_eq!(
        matrix.scan.rescan_projection().archive_entries,
        shared.archive_entries * 3
    );
    // The saving is the two enumerations that did not happen: 2 x 4 directory records here.
    assert_eq!(baseline.archive_entries, 42);
    assert_eq!(matrix.usage.archive_entries, 34);
    assert!(matrix.usage.archive_entries < baseline.archive_entries);
    // The per-profile part is the selection's own work — the compliance probe re-reading a
    // versioned entry — and it is the same work each standalone run did.
    let per_profile = per_profile_usage(&matrix);
    assert_eq!(per_profile.read_bytes, 220 * 3);
    assert_eq!(per_profile.class_bytes, 174 * 3);
}

/// The nested-container case: there the shared scan also stops re-reading expanded containers.
#[test]
fn a_tree_scope_shares_the_re_reads_of_nested_containers_too() {
    let snapshot = open(war_bytes());
    let profiles = vec![
        RuntimeProfile {
            java_release: 11,
            multi_release: MultiReleasePolicy::Enabled,
            layout: LayoutMode::War,
        },
        RuntimeProfile {
            java_release: 17,
            multi_release: MultiReleasePolicy::Enabled,
            layout: LayoutMode::War,
        },
        RuntimeProfile {
            java_release: 21,
            multi_release: MultiReleasePolicy::Enabled,
            layout: LayoutMode::War,
        },
    ];
    let request = RuntimeMatrixRequest {
        physical: tree_view(&snapshot),
        profiles: profiles.clone(),
        domains: vec![domain(
            "app",
            vec![snapshot_root(&snapshot)],
            DelegationPolicy::ParentFirst,
        )],
        requester: LoaderId("app".into()),
    };
    let matrix = matrix(&snapshot, &request);

    let shared = matrix.scan.shared_scan;
    // Expanding `WEB-INF/lib/library.jar` is part of the scan, and it reads bytes.
    assert!(shared.read_bytes > 0);
    assert_eq!(shared, one_physical_scan(&snapshot, &profiles[0], &request));

    let baseline = standalone_total(&snapshot, &profiles, &request);
    // Each standalone run is the scan plus the selection: its measured cost is one scan plus one
    // profile, so three of them are the matrix's rescan projection plus the three selections.
    let per_profile = per_profile_usage(&matrix);
    assert_eq!(scan_times_three_plus_profiles(&matrix), baseline);
    assert_eq!(
        matrix.usage.read_bytes,
        shared.read_bytes + per_profile.read_bytes
    );
    assert_eq!(
        matrix.usage.archive_entries,
        shared.archive_entries + per_profile.archive_entries
    );
    assert!(
        matrix.usage.read_bytes < baseline.read_bytes,
        "the matrix reads {} bytes where three runs read {}",
        matrix.usage.read_bytes,
        baseline.read_bytes
    );
    assert_eq!(baseline.read_bytes, 717);
    assert_eq!(matrix.usage.read_bytes, 331);
    assert_eq!(baseline.archive_entries, 27);
    assert_eq!(matrix.usage.archive_entries, 11);
}

/// One physical scan of one view, measured on its own.
fn one_physical_scan(
    snapshot: &ArtifactSnapshot,
    profile: &RuntimeProfile,
    request: &RuntimeMatrixRequest,
) -> PhaseUsage {
    let mut budget = Budget::new(limits());
    let before = budget.usage();
    let view = RuntimeView {
        physical: request.physical.clone(),
        profile: profile.clone(),
        load_domain: request.domains[0].clone(),
    };
    let evidence = multi_release::physical_evidence(snapshot, &view, &mut budget).unwrap();
    let usage = PhaseUsage::between(&before, &budget.usage());
    drop(evidence);
    usage
}

/// The cost of the same profiles when each reads the physical view for itself.
fn standalone_total(
    snapshot: &ArtifactSnapshot,
    profiles: &[RuntimeProfile],
    request: &RuntimeMatrixRequest,
) -> PhaseUsage {
    profiles
        .iter()
        .map(|profile| {
            let view = RuntimeView {
                physical: request.physical.clone(),
                profile: profile.clone(),
                load_domain: request.domains[0].clone(),
            };
            let mut budget = Budget::new(limits());
            let before = budget.usage();
            Engine::new()
                .select_multi_release(snapshot, &view, &mut budget)
                .unwrap();
            PhaseUsage::between(&before, &budget.usage())
        })
        .fold(PhaseUsage::default(), sum_usage)
}

fn per_profile_usage(matrix: &RuntimeMatrix) -> PhaseUsage {
    matrix
        .scan
        .per_profile
        .iter()
        .copied()
        .fold(PhaseUsage::default(), sum_usage)
}

/// The matrix's measured cost with its one scan counted once per profile, so it can be compared
/// against profiles that each scanned for themselves.
fn scan_times_three_plus_profiles(matrix: &RuntimeMatrix) -> PhaseUsage {
    sum_usage(matrix.scan.rescan_projection(), per_profile_usage(matrix))
}

fn sum_usage(left: PhaseUsage, right: PhaseUsage) -> PhaseUsage {
    PhaseUsage {
        input_bytes: left.input_bytes + right.input_bytes,
        archive_entries: left.archive_entries + right.archive_entries,
        entry_bytes: left.entry_bytes + right.entry_bytes,
        read_bytes: left.read_bytes + right.read_bytes,
        class_bytes: left.class_bytes + right.class_bytes,
        attribute_bytes: left.attribute_bytes + right.attribute_bytes,
        code_bytes: left.code_bytes + right.code_bytes,
        result_items: left.result_items + right.result_items,
        output_bytes: left.output_bytes + right.output_bytes,
        class_headers: left.class_headers + right.class_headers,
        method_bodies: left.method_bodies + right.method_bodies,
    }
}

/// Compression: profiles whose selection is the same **and** proven to stay the same merge.
#[test]
fn profiles_inside_one_stable_interval_merge_with_the_proof_they_share() {
    let snapshot = open(divergent_jar());
    let request = request(
        &snapshot,
        vec![profile(11), profile(18), profile(21), profile(25)],
        vec![domain(
            "app",
            vec![snapshot_root(&snapshot)],
            DelegationPolicy::ParentFirst,
        )],
    );
    let matrix = matrix(&snapshot, &request);

    // 18, 21 and 25 all reach `versions/17`, and none of them can be moved by a later release.
    let merged = matrix.group_of(1);
    assert_eq!(merged.members, vec![1, 2, 3]);
    assert_eq!(merged.reason, GroupReason::SharedStableInterval);
    assert_eq!(
        merged.stable_interval,
        Some(StableInterval {
            lowest_release: 17,
            highest_release: None
        })
    );
    for index in [1, 2, 3] {
        assert_eq!(
            selected_entry(definition_of(&matrix, index, b"p/Join.class")),
            entry_named(
                definition_of(&matrix, index, b"p/Join.class"),
                b"META-INF/versions/17/p/Join.class"
            )
        );
    }

    // Java 11 is alone: its interval stops where 17 begins.
    let eleven = matrix.group_of(0);
    assert_eq!(eleven.members, vec![0]);
    assert_eq!(
        eleven.stable_interval,
        Some(StableInterval {
            lowest_release: 11,
            highest_release: Some(16)
        })
    );
    assert_eq!(matrix.groups.len(), 2);
}

/// The refusal: two profiles that agree without a proof stay separate.
#[test]
fn agreement_without_a_proven_interval_is_never_merged() {
    let snapshot = open(divergent_jar());
    let custom = |release: u16| RuntimeProfile {
        java_release: release,
        multi_release: MultiReleasePolicy::Custom {
            id: "house-rules".into(),
        },
        layout: LayoutMode::Generic,
    };
    let request = request(
        &snapshot,
        vec![custom(11), custom(17)],
        vec![domain(
            "app",
            vec![snapshot_root(&snapshot)],
            DelegationPolicy::ParentFirst,
        )],
    );
    let matrix = matrix(&snapshot, &request);

    // Both profiles reach the same (unknown) answer, and neither is presented as a selection.
    for index in [0, 1] {
        let policies = &matrix.profiles[index].policies;
        assert!(!policies.multi_release.is_supported());
        let definition = definition_of(&matrix, index, b"p/Join.class");
        assert_eq!(
            definition.origins[0].rule,
            RuntimeSelectionRule::Unknown {
                reason: MultiReleaseUnknownReason::CustomPolicy
            }
        );
        assert!(definition.selected().is_none());
        // The unselected list is every entry: an unknown rule selects nothing.
        assert_eq!(definition.unselected().len(), definition.candidates().len());
        for candidate in definition.candidates() {
            assert!(matches!(
                candidate.decision,
                MultiReleaseSelectionDecision::Unknown { .. }
            ));
        }
    }

    // Same answer, no interval: two groups, each stating that it is a single unproven view.
    assert_eq!(matrix.groups.len(), 2);
    for index in [0, 1] {
        let group = matrix.group_of(index);
        assert_eq!(group.members, vec![index]);
        assert_eq!(group.stable_interval, None);
        assert!(matches!(group.reason, GroupReason::NotProvable { .. }));
    }
}

/// A06: a versioned entry the manifest condition can never reach is a diagnostic, not a silence.
#[test]
fn a_manifest_condition_that_hides_versioned_entries_is_diagnosed() {
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", INACTIVE_MANIFEST),
        (b"p/Join.class", &class(52)),
        (b"META-INF/versions/11/p/Join.class", &class(55)),
    ]));
    let request = request(
        &snapshot,
        vec![profile(11)],
        vec![domain(
            "app",
            vec![snapshot_root(&snapshot)],
            DelegationPolicy::ParentFirst,
        )],
    );
    let matrix = matrix(&snapshot, &request);

    let definition = definition_of(&matrix, 0, b"p/Join.class");
    let RuntimeSelectionRule::BaseSelected { reason } = &definition.origins[0].rule else {
        panic!(
            "the inactive manifest selects the base entry: {:?}",
            definition.origins[0].rule
        );
    };
    assert!(
        matches!(reason, BaseSelectionReason::ManifestNotActive { .. }),
        "the reason names the manifest condition: {reason:?}"
    );
    // The versioned entry is shipped and listed, it is simply unreachable.
    assert_eq!(definition.candidates().len(), 2);
    assert_eq!(
        definition
            .candidates()
            .iter()
            .filter(|candidate| matches!(
                candidate.variant,
                MultiReleaseEntryVariant::Versioned { release: 11 }
            ))
            .count(),
        1
    );

    let diagnostics: Vec<&RuntimeMatrixDiagnostic> = matrix
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code()
                == Some(RuntimeMatrixDiagnosticCode::RuntimeMatrixVersionedEntriesInactive)
        })
        .collect();
    assert_eq!(diagnostics.len(), 1, "{:?}", matrix.diagnostics);
    assert_eq!(diagnostics[0].severity(), DiagnosticSeverity::Warning);
    assert!(matches!(
        diagnostics[0],
        RuntimeMatrixDiagnostic::Matrix {
            profile: Some(0),
            ..
        }
    ));
}

/// A06: a selected entry the compliance probe proved nonconformant is an error, with its code.
#[test]
fn a_nonconformant_selection_is_an_error_and_keeps_the_predecessor_diagnostic() {
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"META-INF/versions/11/p/Join.class", &class(55)),
    ]));
    let request = request(
        &snapshot,
        vec![profile(11)],
        vec![domain(
            "app",
            vec![snapshot_root(&snapshot)],
            DelegationPolicy::ParentFirst,
        )],
    );
    let matrix = matrix(&snapshot, &request);

    let definition = definition_of(&matrix, 0, b"p/Join.class");
    let selected = definition
        .selected()
        .expect("the versioned entry is selected");
    assert_eq!(selected.compliance, MultiReleaseCompliance::NonConformant);

    assert!(
        matrix.diagnostics.iter().any(|diagnostic| {
            diagnostic.code()
                == Some(RuntimeMatrixDiagnosticCode::RuntimeMatrixSelectedEntryNonConformant)
                && diagnostic.severity() == DiagnosticSeverity::Error
        }),
        "{:?}",
        matrix.diagnostics
    );
    // The reused report's own code is forwarded with the profile it belongs to.
    assert!(
        matrix.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            RuntimeMatrixDiagnostic::Profile {
                profile: 0,
                diagnostic: MultiReleaseReportDiagnostic::Domain { diagnostic },
            } if diagnostic.code == MultiReleaseDiagnosticCode::MultiReleasePublicPredecessorMissing
        )),
        "{:?}",
        matrix.diagnostics
    );
}

/// A WAR whose class directory and one library both publish `p/Dup.class`.
///
/// `nested_fixture` (the bytes behind `p1-golden/nested.json`) names the class that sits inside
/// `WEB-INF/lib/*.jar` `p/Dup.class`; the same name is kept here, with a class directory added so
/// that the layout projection has a layer to strip and one entry that is on no layer at all.
fn war_bytes() -> Vec<u8> {
    let library = zip(&[(b"p/Dup.class", &class_named(55, b"p/Dup"))]);
    zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"WEB-INF/classes/p/Dup.class", &class_named(52, b"p/Dup")),
        (b"WEB-INF/lib/a.jar", &library),
        (b"p/Outside.class", &class_named(52, b"p/Outside")),
    ])
}

fn tree_view(snapshot: &ArtifactSnapshot) -> PhysicalView {
    PhysicalView {
        snapshot: snapshot.id().clone(),
        scope: PhysicalScope::ArtifactTree {
            root_container: ContainerId("root".into()),
        },
    }
}

/// A07: the same binary name in two roots of one WAR, with both origins and the declared order.
#[test]
fn a_name_in_two_war_roots_reports_both_origins_and_the_order_the_declaration_gives() {
    let snapshot = open(war_bytes());
    let tree = tree_view(&snapshot);
    let war = |roots: Vec<LoadRoot>, delegation: DelegationPolicy| LoadDomain {
        loader: LoaderId("app".into()),
        parent_loader: None,
        delegation,
        roots,
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let profiles = vec![RuntimeProfile {
        java_release: 17,
        multi_release: MultiReleasePolicy::Enabled,
        layout: LayoutMode::War,
    }];
    let build = |domains: Vec<LoadDomain>| {
        let request = RuntimeMatrixRequest {
            physical: tree.clone(),
            profiles: profiles.clone(),
            domains,
            requester: LoaderId("app".into()),
        };
        matrix(&snapshot, &request)
    };

    // One declared root covering the whole snapshot: both origins are reached, at the same
    // position, so the declaration does not separate them.
    let single = build(vec![war(
        vec![snapshot_root(&snapshot)],
        DelegationPolicy::ParentFirst,
    )]);
    let definition = definition_of(&single, 0, b"p/Dup.class");
    assert_eq!(
        definition.origins.len(),
        2,
        "both roots publish p/Dup.class"
    );
    let root_container = definition.origins[0].container.clone();
    let nested = definition.origins[1].container.clone();
    assert_ne!(root_container, nested);
    assert!(nested.steps.len() > root_container.steps.len());
    for origin in &definition.origins {
        assert_eq!(origin.candidates.len(), 1);
        assert_eq!(
            origin.candidates[0].decision,
            MultiReleaseSelectionDecision::Selected
        );
    }
    assert_ne!(
        definition.origins[0].candidates[0].entry.ordinal,
        definition.origins[1].candidates[0].entry.ordinal,
        "the two origins are different physical entries"
    );
    assert!(matches!(
        definition.loader,
        LoaderOrder::Ambiguous { position: 0, ref containers } if containers.len() == 2
    ));

    // The layout decides which entries of the WAR are even on the path.
    assert_eq!(
        definition.origins[0].candidates[0].on_path,
        Some(true),
        "WEB-INF/classes is on the war path"
    );
    assert_eq!(
        definition.origins[1].candidates[0].on_path,
        Some(true),
        "a WEB-INF/lib container is on the war path"
    );
    let outside = single.profiles[0].definition(b"p/Outside.class").unwrap();
    assert_eq!(outside.origins[0].candidates[0].on_path, Some(false));
    assert_eq!(single.profiles[0].layout.present.len(), 2);
    assert_eq!(single.profiles[0].layout.layers.len(), 2);
    assert_eq!(single.profiles[0].layout.containers.len(), 2);

    // Naming the nested container as a root makes the order explicit, and the order decides.
    let nested_root = LoadRoot::ArtifactTree {
        root: nested.clone(),
    };
    let ordered = build(vec![war(
        vec![nested_root.clone(), snapshot_root(&snapshot)],
        DelegationPolicy::ParentFirst,
    )]);
    let definition = definition_of(&ordered, 0, b"p/Dup.class");
    assert_eq!(
        definition.loader,
        LoaderOrder::Ordered {
            loader: LoaderId("app".into()),
            position: 0
        }
    );
    assert_eq!(
        definition.origins[1].candidates[0].root,
        RootAttribution::Covered {
            loader: LoaderId("app".into()),
            position: 0
        }
    );
    assert_eq!(
        definition.origins[0].candidates[0].root,
        RootAttribution::Covered {
            loader: LoaderId("app".into()),
            position: 1
        }
    );

    // Two loaders and a parent-first child: the parent is searched first, which puts both origins
    // back at the same position. A child-first child searches its own roots first.
    let parent = LoadDomain {
        loader: LoaderId("parent".into()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![snapshot_root(&snapshot)],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let child = |delegation| {
        let mut domain = war(vec![nested_root.clone()], delegation);
        domain.parent_loader = Some(LoaderId("parent".into()));
        domain
    };
    let parent_first = build(vec![child(DelegationPolicy::ParentFirst), parent.clone()]);
    assert!(matches!(
        definition_of(&parent_first, 0, b"p/Dup.class").loader,
        LoaderOrder::Ambiguous { position: 0, .. }
    ));
    let child_first = build(vec![child(DelegationPolicy::ChildFirst), parent.clone()]);
    let definition = definition_of(&child_first, 0, b"p/Dup.class");
    assert_eq!(
        definition.loader,
        LoaderOrder::Ordered {
            loader: LoaderId("app".into()),
            position: 0
        }
    );

    // A policy the engine cannot read produces no order claim at all, and no selection claim is
    // smuggled in with it.
    let custom = build(vec![war(
        vec![nested_root],
        DelegationPolicy::Custom {
            id: "shaded".into(),
        },
    )]);
    assert!(matches!(
        custom.domain_graph.ordering,
        OrderVerdict::Undetermined {
            reason: OrderUnknownReason::DelegationPolicyUnsupported,
            ..
        }
    ));
    assert!(custom.domain_graph.search.is_empty());
    let definition = definition_of(&custom, 0, b"p/Dup.class");
    assert!(matches!(
        definition.loader,
        LoaderOrder::Unknown {
            reason: OrderUnknownReason::DelegationPolicyUnsupported,
            ..
        }
    ));
    assert!(matches!(
        definition.origins[0].candidates[0].root,
        RootAttribution::Undetermined { .. }
    ));
    assert_eq!(definition.origins.len(), 2);
}

/// Module policy: what the engine can express today, stated as data instead of guessed.
#[test]
fn a_module_path_domain_reports_the_policy_verdict_and_claims_no_order() {
    let snapshot = open(divergent_jar());
    let mut domain = domain(
        "app",
        vec![snapshot_root(&snapshot)],
        DelegationPolicy::ParentFirst,
    );
    domain.module_mode = ModuleMode::ModulePath;
    let request = request(&snapshot, vec![profile(11)], vec![domain]);
    let matrix = matrix(&snapshot, &request);

    assert!(!matrix.domain_graph.chain[0].module.is_supported());
    assert_eq!(
        matrix.domain_graph.chain[0].module_mode,
        ModuleMode::ModulePath
    );
    assert!(matches!(
        matrix.domain_graph.ordering,
        OrderVerdict::Undetermined {
            reason: OrderUnknownReason::ModuleModeUnsupported,
            ..
        }
    ));
    let definition = definition_of(&matrix, 0, b"p/Join.class");
    assert!(matches!(definition.loader, LoaderOrder::Unknown { .. }));
    // The multi-release selection still ran: it is per container and does not depend on the loader.
    assert_eq!(
        selected_entry(definition),
        entry_named(definition, b"META-INF/versions/11/p/Join.class")
    );
}

/// The request's own bounds, which keep the report from reading as more than it is.
#[test]
fn the_request_rejects_what_it_cannot_answer_honestly() {
    let snapshot = open(divergent_jar());
    let root = snapshot_root(&snapshot);
    let build = |profiles: Vec<RuntimeProfile>, domains: Vec<LoadDomain>| {
        let request = RuntimeMatrixRequest {
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profiles,
            domains,
            requester: LoaderId("app".into()),
        };
        let mut budget = Budget::new(limits());
        Engine::new().runtime_matrix(&snapshot, &request, &mut budget)
    };

    let error = |result: Result<RuntimeMatrix>| match result {
        Err(Error::InvalidInput { code, .. }) => code,
        other => panic!("expected an invalid input, got {other:?}"),
    };
    let one = domain("app", vec![root.clone()], DelegationPolicy::ParentFirst);
    assert_eq!(
        error(build(Vec::new(), vec![one.clone()])),
        "runtime_matrix_empty_profiles"
    );
    assert_eq!(
        error(build(vec![profile(11), profile(11)], vec![one.clone()])),
        "runtime_matrix_duplicate_profile"
    );
    let many: Vec<RuntimeProfile> = (11..(11 + MAX_PROFILES as u16 + 1)).map(profile).collect();
    assert_eq!(many.len(), MAX_PROFILES + 1);
    assert_eq!(
        error(build(many, vec![one.clone()])),
        "runtime_matrix_profile_limit"
    );
    assert_eq!(
        error(build(vec![profile(11)], vec![one.clone(), one.clone()])),
        "runtime_matrix_duplicate_loader"
    );
    let mut other = domain("other", vec![root.clone()], DelegationPolicy::ParentFirst);
    other.parent_loader = Some(LoaderId("app".into()));
    let mut cyclic = domain("app", vec![root.clone()], DelegationPolicy::ParentFirst);
    cyclic.parent_loader = Some(LoaderId("other".into()));
    assert_eq!(
        error(build(vec![profile(11)], vec![cyclic, other])),
        "runtime_matrix_loader_cycle"
    );
    assert_eq!(
        error(build(
            vec![profile(11)],
            vec![domain(
                "elsewhere",
                vec![root],
                DelegationPolicy::ParentFirst
            )]
        )),
        "runtime_matrix_requester_missing"
    );
}

/// A parent outside the request is a stated gap, not an assumed order.
#[test]
fn an_unsupplied_parent_makes_the_order_unknown() {
    let snapshot = open(divergent_jar());
    let mut app = domain(
        "app",
        vec![snapshot_root(&snapshot)],
        DelegationPolicy::ParentFirst,
    );
    app.parent_loader = Some(LoaderId("outside".into()));
    let request = request(&snapshot, vec![profile(11)], vec![app]);
    let matrix = matrix(&snapshot, &request);

    assert!(matches!(
        matrix.domain_graph.ordering,
        OrderVerdict::Undetermined {
            reason: OrderUnknownReason::ParentDomainNotSupplied,
            ..
        }
    ));
    assert_eq!(matrix.domain_graph.chain.len(), 1);
    assert_eq!(
        matrix.domain_graph.chain[0].parent_loader,
        Some(LoaderId("outside".into()))
    );
    // The answer to the selection question is still reported, with no order attached to it.
    let definition = definition_of(&matrix, 0, b"p/Join.class");
    assert!(matches!(definition.loader, LoaderOrder::Unknown { .. }));
    assert_eq!(definition.candidates().len(), 3);
}

/// The matrix answers the physical plane, and it never claims a verification it did not perform.
///
/// `RuntimeMatrix.verification` is the header plane's own `VerificationStatus` — the same type
/// `HeaderInspection` carries — and the matrix writes `NotPerformed` into it where the report is
/// built. The assertion is negative on purpose: `Performed` would be the report claiming a plane
/// this entry never enters, because what the matrix does is choose a physical entry per profile and
/// answer the loader, layout and crate-graph questions. It reads no method body, builds no CFG and
/// hands nothing to a verifier, so the field is a claim the matrix is not allowed to make rather
/// than a value that happens to hold today.
#[test]
fn the_matrix_never_claims_verification() {
    let snapshot = open(divergent_jar());
    let request = request(
        &snapshot,
        vec![profile(8), profile(11), profile(17)],
        vec![domain(
            "app",
            vec![snapshot_root(&snapshot)],
            DelegationPolicy::ParentFirst,
        )],
    );
    let matrix = matrix(&snapshot, &request);

    assert_eq!(matrix.verification, VerificationStatus::NotPerformed);
    // It is not `NotPerformed` because the run gave up: three profiles were answered, and none of
    // them reported a problem.
    assert_eq!(matrix.profiles.len(), 3);
    assert!(
        matrix
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity() != DiagnosticSeverity::Error),
        "{:?}",
        matrix.diagnostics
    );
}

/// The same field on a run that does report an error, which is where a claim would be tempting.
///
/// A refused request never gets this far — `RuntimeMatrix` comes back as `Err`, so there is no
/// report to assert on — and the run that reaches a report while having something to complain about
/// is the nonconformant selection below: an `Error` diagnostic, and still no verification claimed.
/// An error in the physical answer is not "a verifier ran and disagreed".
#[test]
fn a_matrix_that_reports_an_error_never_claims_verification() {
    let snapshot = open(zip(&[
        (b"META-INF/MANIFEST.MF", ACTIVE_MANIFEST),
        (b"META-INF/versions/11/p/Join.class", &class(55)),
    ]));
    let request = request(
        &snapshot,
        vec![profile(11)],
        vec![domain(
            "app",
            vec![snapshot_root(&snapshot)],
            DelegationPolicy::ParentFirst,
        )],
    );
    let matrix = matrix(&snapshot, &request);

    assert!(
        matrix
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity() == DiagnosticSeverity::Error),
        "{:?}",
        matrix.diagnostics
    );
    assert_eq!(matrix.verification, VerificationStatus::NotPerformed);
}
