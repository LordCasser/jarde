use jarde::{
    ArchiveNameBytes, ArtifactInput, Budget, BudgetDimension, ContainerOriginStep, CoverageState,
    Engine, ExecutionReport, LayoutNodeKind, LayoutNodeSource, Limits, NestedArchiveState,
    PhysicalScope, TerminationReason,
};
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const STORE: u16 = 0;
const DEFLATE: u16 = 8;

fn limits() -> Limits {
    Limits {
        input_bytes: 16 * 1024 * 1024,
        archive_entries: 10_000,
        entry_bytes: 16 * 1024 * 1024,
        read_bytes: 16 * 1024 * 1024,
        class_bytes: 16 * 1024 * 1024,
        attribute_bytes: 16 * 1024 * 1024,
        code_bytes: 16 * 1024 * 1024,
        result_items: 10_000,
        output_bytes: 16 * 1024 * 1024,
        nested_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn zip(entries: &[(&[u8], &[u8], u16)]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data, method) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(*method))
                .start()
                .unwrap();
            if *method == DEFLATE {
                let encoder =
                    flate2::write::DeflateEncoder::new(&mut entry, flate2::Compression::default());
                let mut writer = config.wrap(encoder);
                writer.write_all(data).unwrap();
                let (encoder, descriptor) = writer.finish().unwrap();
                encoder.finish().unwrap();
                entry.finish(descriptor).unwrap();
            } else {
                let mut writer = config.wrap(&mut entry);
                writer.write_all(data).unwrap();
                let (_, descriptor) = writer.finish().unwrap();
                entry.finish(descriptor).unwrap();
            }
        }
        archive.finish().unwrap();
    }
    output.into_inner()
}

fn open(bytes: Vec<u8>) -> (jarde::ArtifactSnapshot, Budget) {
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .unwrap();
    (snapshot, budget)
}

#[test]
fn boot_tree_is_explicit_and_nested_entry_is_replayable_without_intermediate_output() {
    let inner_class = b"inner-class";
    let inner = zip(&[(b"p/Inner.class", inner_class, STORE)]);
    let outer = zip(&[
        (b"BOOT-INF/classes/App.class", b"app", STORE),
        (b"BOOT-INF/lib/inner.jar", &inner, DEFLATE),
    ]);
    let (snapshot, mut budget) = open(outer);

    let ordinary = snapshot.enumerate(&mut budget).unwrap();
    assert_eq!(
        ordinary.entries[1].nested_archive,
        NestedArchiveState::CandidateNotScanned
    );

    let tree = Engine::new()
        .enumerate_artifact_tree(&snapshot, &mut budget)
        .unwrap();
    assert!(matches!(tree.execution, ExecutionReport::Complete { .. }));
    assert_eq!(tree.containers.len(), 2);
    assert!(
        tree.layout_nodes
            .iter()
            .any(|node| node.kind == LayoutNodeKind::BootClasses)
    );
    assert!(tree.layout_nodes.iter().any(|node| {
        node.kind == LayoutNodeKind::BootLibrary
            && matches!(
                node.source,
                LayoutNodeSource::Archive {
                    child_container: Some(_),
                    ..
                }
            )
    }));
    let nested = tree.containers[1]
        .entries
        .iter()
        .find(|entry| entry.id.raw_name.0 == b"p/Inner.class")
        .unwrap();
    assert_eq!(nested.id.origin.steps.len(), 1);

    let output_before = budget.usage().output_bytes;
    let materialized = snapshot.read_entry(nested, &mut budget).unwrap();
    assert_eq!(materialized.bytes, inner_class);
    assert_eq!(
        budget.usage().output_bytes - output_before,
        inner_class.len() as u64
    );
}

#[test]
fn duplicate_nested_entries_derive_distinct_child_and_inner_identities() {
    let inner = zip(&[(b"p/Same.class", b"same", STORE)]);
    let outer = zip(&[
        (b"lib/same.jar", &inner, STORE),
        (b"lib/same.jar", &inner, STORE),
    ]);
    let (snapshot, mut budget) = open(outer);
    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();
    assert_eq!(tree.containers.len(), 3);
    assert_ne!(
        tree.containers[1].origin.current_container(),
        tree.containers[2].origin.current_container()
    );
    assert_ne!(
        tree.containers[1].entries[0].id,
        tree.containers[2].entries[0].id
    );
}

#[test]
fn nested_depth_is_a_high_water_mark_and_preserves_siblings() {
    let deepest = zip(&[(b"Deep.class", b"deep", STORE)]);
    let middle = zip(&[
        (b"deeper.jar", &deepest, DEFLATE),
        (b"Sibling.class", b"ok", STORE),
    ]);
    let outer = zip(&[(b"middle.jar", &middle, DEFLATE)]);

    let (snapshot, _) = open(outer.clone());
    let mut shallow_limits = limits();
    shallow_limits.nested_depth = 1;
    let mut shallow = Budget::new(shallow_limits);
    let tree = snapshot.enumerate_artifact_tree(&mut shallow).unwrap();
    assert!(matches!(
        tree.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::NestedDepth
            },
            ..
        }
    ));
    assert_eq!(shallow.usage().nested_depth, 1);
    assert!(
        tree.containers[1]
            .entries
            .iter()
            .any(|entry| entry.id.raw_name.0 == b"Sibling.class")
    );

    let (snapshot, _) = open(outer);
    let mut deep_limits = limits();
    deep_limits.nested_depth = 2;
    let mut deep = Budget::new(deep_limits);
    let tree = snapshot.enumerate_artifact_tree(&mut deep).unwrap();
    assert!(matches!(tree.execution, ExecutionReport::Complete { .. }));
    assert_eq!(deep.usage().nested_depth, 2);
    assert_eq!(tree.containers.len(), 3);
}

#[test]
fn depth_limit_skips_each_candidate_at_its_parent_ordinal() {
    let child = zip(&[(b"Child.class", b"child", STORE)]);
    let outer = zip(&[
        (b"first.jar", &child, STORE),
        (b"second.jar", &child, STORE),
    ]);
    let (snapshot, _) = open(outer);
    let mut configured = limits();
    configured.nested_depth = 0;
    let mut budget = Budget::new(configured);

    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();

    assert!(matches!(
        tree.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::NestedDepth
            },
            ..
        }
    ));
    let label = format!(
        "container:{}:nested_archive_candidates",
        tree.containers[0].origin.current_container().0
    );
    assert!(
        tree.coverage
            .artifact_structural
            .scanned
            .iter()
            .all(|range| range.label != label)
    );
    for ordinal in 0..2 {
        assert!(
            tree.coverage
                .artifact_structural
                .skipped
                .iter()
                .any(|range| range.label == label
                    && range.start == ordinal
                    && range.end == ordinal + 1)
        );
    }
}

#[test]
fn malformed_and_unsupported_children_do_not_erase_valid_siblings() {
    let valid = zip(&[(b"Good.class", b"good", STORE)]);
    let malformed_outer = zip(&[
        (b"bad.jar", b"not a zip", STORE),
        (b"good.jar", &valid, STORE),
    ]);
    let (snapshot, mut budget) = open(malformed_outer);
    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();
    assert!(matches!(
        tree.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::Error { .. },
            ..
        }
    ));
    assert_eq!(tree.containers.len(), 2);
    assert!(tree.diagnostics.iter().any(|diagnostic| matches!(diagnostic.provenance.as_ref().map(|p| &p.location), Some(jarde::Location::Entry { id, .. }) if id.raw_name.0 == b"bad.jar")));
    let root_label = format!(
        "container:{}:nested_archive_candidates",
        tree.containers[0].origin.current_container().0
    );
    assert!(
        tree.coverage
            .artifact_structural
            .skipped
            .iter()
            .any(|range| range.label == root_label && range.start == 0 && range.end == 1)
    );
    assert!(
        tree.coverage
            .artifact_structural
            .scanned
            .iter()
            .any(|range| range.label == root_label && range.start == 1 && range.end == 2)
    );
    assert!(
        !tree
            .coverage
            .artifact_structural
            .scanned
            .iter()
            .any(|range| range.label == root_label && range.start == 0 && range.end == 1)
    );
    assert!(
        !tree
            .coverage
            .artifact_structural
            .skipped
            .iter()
            .any(|range| range.label == root_label && range.start == 1 && range.end == 2)
    );

    let unsupported_outer = zip(&[(b"bad.jar", &valid, 12), (b"good.jar", &valid, STORE)]);
    let (snapshot, mut budget) = open(unsupported_outer);
    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();
    assert!(matches!(
        tree.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::Unsupported { .. },
            ..
        }
    ));
    assert_eq!(tree.containers.len(), 2);
}

#[test]
fn budget_and_precancellation_return_non_complete_reliable_prefixes() {
    let inner = zip(&[(b"Inner.class", b"inner", STORE)]);
    let outer = zip(&[
        (b"inner.jar", &inner, DEFLATE),
        (b"Root.class", b"root", STORE),
    ]);
    let (snapshot, _) = open(outer);

    let mut entry_limits = limits();
    entry_limits.entry_bytes = 1;
    let mut entry_budget = Budget::new(entry_limits);
    let report = snapshot.enumerate_artifact_tree(&mut entry_budget).unwrap();
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::EntryBytes
            },
            ..
        }
    ));
    assert_eq!(report.containers.len(), 1);
    assert!(!report.containers[0].entries.is_empty());

    let mut archive_limits = limits();
    archive_limits.archive_entries = 1;
    let mut archive_budget = Budget::new(archive_limits);
    let report = snapshot
        .enumerate_artifact_tree(&mut archive_budget)
        .unwrap();
    assert!(!matches!(
        report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(report.containers.len(), 1);
    assert_eq!(report.containers[0].entries.len(), 1);
    let prefix_candidate_label = format!(
        "container:{}:nested_archive_candidates",
        report.containers[0].origin.current_container().0
    );
    assert!(
        report
            .coverage
            .artifact_structural
            .skipped
            .iter()
            .any(|range| range.label == prefix_candidate_label
                && range.start == 0
                && range.end == 1)
    );

    let mut cancelled = Budget::new(limits());
    cancelled.cancellation_token().cancel();
    let report = snapshot.enumerate_artifact_tree(&mut cancelled).unwrap();
    assert!(matches!(
        report.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert!(report.containers.is_empty());
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    let PhysicalScope::ArtifactTree { root_container } = &report.view.scope else {
        panic!("expected artifact-tree scope");
    };
    let root_label = format!("container:{}:central_directory_entries", root_container.0);
    assert!(
        report
            .coverage
            .artifact_structural
            .skipped
            .iter()
            .any(|range| range.label == root_label && range.start == 0 && range.end == 2)
    );

    let mut no_result_limits = limits();
    no_result_limits.result_items = 0;
    let mut no_results = Budget::new(no_result_limits);
    let report = snapshot.enumerate_artifact_tree(&mut no_results).unwrap();
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert!(report.containers.is_empty());
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    let PhysicalScope::ArtifactTree { root_container } = &report.view.scope else {
        panic!("expected artifact-tree scope");
    };
    let root_label = format!("container:{}:central_directory_entries", root_container.0);
    assert!(
        report
            .coverage
            .artifact_structural
            .skipped
            .iter()
            .any(|range| range.label == root_label && range.start == 0 && range.end == 2)
    );
    assert_eq!(no_results.usage().result_items, 0);
}

#[test]
fn prefix_result_item_exhaustion_skips_all_remaining_root_candidates() {
    let child = zip(&[(b"Child.class", b"child", STORE)]);
    let outer = zip(&[
        (b"BOOT-INF/classes/App.class", b"app", STORE),
        (b"first.jar", &child, STORE),
        (b"second.jar", &child, STORE),
    ]);
    let (snapshot, _) = open(outer);
    let mut configured = limits();
    configured.result_items = 4;
    let mut budget = Budget::new(configured);

    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();

    assert!(matches!(
        tree.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert_eq!(tree.containers.len(), 1);
    let label = format!(
        "container:{}:nested_archive_candidates",
        tree.containers[0].origin.current_container().0
    );
    for ordinal in 1..=2 {
        assert!(
            tree.coverage
                .artifact_structural
                .skipped
                .iter()
                .any(|range| range.label == label
                    && range.start == ordinal
                    && range.end == ordinal + 1)
        );
        assert!(
            tree.coverage
                .artifact_structural
                .scanned
                .iter()
                .all(|range| !(range.label == label
                    && range.start == ordinal
                    && range.end == ordinal + 1))
        );
    }
    assert_eq!(budget.usage().result_items, 4);
}

#[test]
fn result_item_exhaustion_before_second_child_report_keeps_pending_coverage_and_provenance() {
    let first = zip(&[(b"First.class", b"first", STORE)]);
    let second = zip(&[(b"Second.class", b"second", STORE)]);
    let outer = zip(&[
        (b"first.jar", &first, STORE),
        (b"second.jar", &second, STORE),
    ]);
    let (snapshot, _) = open(outer);
    let mut configured = limits();
    configured.result_items = 7;
    let mut budget = Budget::new(configured);

    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();

    assert!(matches!(
        tree.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert_eq!(tree.containers.len(), 2);
    assert!(tree.containers[0].origin.steps.is_empty());
    assert_eq!(
        tree.containers[1].origin.steps[0].via_raw_name.0,
        b"first.jar"
    );

    let second_origin = tree
        .layout_nodes
        .iter()
        .find_map(|node| match &node.source {
            LayoutNodeSource::Archive {
                entry,
                child_container: Some(origin),
            } if entry.raw_name.0 == b"second.jar" => Some(origin),
            _ => None,
        })
        .expect("root established the second child container");
    let second_label = format!(
        "container:{}:central_directory_entries",
        second_origin.current_container().0
    );
    assert!(
        tree.coverage
            .artifact_structural
            .skipped
            .iter()
            .any(|range| { range.label == second_label && range.start == 0 && range.end == 1 })
    );
    assert!(tree.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.provenance.as_ref().map(|provenance| &provenance.location),
        Some(jarde::Location::Entry { id, .. }) if id.raw_name.0 == b"second.jar"
    )));
    assert_eq!(budget.usage().result_items, 7);
    match &tree.execution {
        ExecutionReport::Partial { usage, .. } => assert_eq!(usage, &budget.usage()),
        other => panic!("expected partial execution, got {other:?}"),
    }
}

#[test]
fn forged_nested_origins_and_metadata_are_rejected() {
    let inner = zip(&[(b"Inner.class", b"inner", STORE)]);
    let outer = zip(&[(b"a.jar", &inner, STORE), (b"b.jar", &inner, STORE)]);
    let (snapshot, mut budget) = open(outer);
    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();
    let entry = tree.containers[1].entries[0].clone();

    let mut forged_child = entry.clone();
    forged_child.id.origin.steps[0].child_container.0.push('x');
    assert!(snapshot.read_entry(&forged_child, &mut budget).is_err());

    let mut forged_parent = entry.clone();
    forged_parent.id.origin.steps[0].via_ordinal = 1;
    assert!(snapshot.read_entry(&forged_parent, &mut budget).is_err());

    let mut forged_metadata = entry;
    forged_metadata.crc32 ^= 1;
    assert!(snapshot.read_entry(&forged_metadata, &mut budget).is_err());
}

#[test]
fn boot_and_war_library_names_require_exact_prefix_case_direct_path_and_lowercase_jar() {
    let child = zip(&[(b"C.class", b"c", STORE)]);
    let outer = zip(&[
        (b"BOOT-INF/lib/good.jar", &child, STORE),
        (b"BOOT-INF/lib/upper.JAR", &child, STORE),
        (b"boot-inf/lib/wrong.jar", &child, STORE),
        (b"BOOT-INF/lib/deep/nested.jar", &child, STORE),
        (b"WEB-INF/lib/good.jar", &child, STORE),
        (b"WEB-inf/lib/wrong.jar", &child, STORE),
        (b"WEB-INF/lib/deep/nested.jar", &child, STORE),
    ]);
    let (snapshot, mut budget) = open(outer);
    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();

    let kind_for = |name: &[u8]| {
        tree.layout_nodes
            .iter()
            .find_map(|node| match &node.source {
                LayoutNodeSource::Archive { entry, .. } if entry.raw_name.0 == name => {
                    Some(node.kind)
                }
                _ => None,
            })
            .unwrap()
    };
    assert_eq!(
        kind_for(b"BOOT-INF/lib/good.jar"),
        LayoutNodeKind::BootLibrary
    );
    assert_eq!(
        kind_for(b"WEB-INF/lib/good.jar"),
        LayoutNodeKind::WarLibrary
    );
    for name in [
        b"BOOT-INF/lib/upper.JAR".as_slice(),
        b"boot-inf/lib/wrong.jar",
        b"BOOT-INF/lib/deep/nested.jar",
        b"WEB-inf/lib/wrong.jar",
        b"WEB-INF/lib/deep/nested.jar",
    ] {
        assert_eq!(kind_for(name), LayoutNodeKind::NestedArchive);
    }
}

#[test]
fn child_enumeration_failure_keeps_failed_report_parent_provenance_and_valid_sibling() {
    let mut broken = zip(&[(b"Broken.class", b"broken", STORE)]);
    let local_name_offset = 30;
    broken[local_name_offset] ^= 1;
    rawzip::ZipArchive::from_slice(&broken).unwrap();
    let valid = zip(&[(b"Good.class", b"good", STORE)]);
    let outer = zip(&[
        (b"broken.jar", &broken, STORE),
        (b"good.jar", &valid, STORE),
    ]);
    let (snapshot, mut budget) = open(outer);
    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();

    assert!(matches!(tree.execution, ExecutionReport::Partial { .. }));
    assert_eq!(tree.containers.len(), 3);
    assert!(matches!(
        tree.containers[1].execution,
        ExecutionReport::Failed { .. }
    ));
    assert!(tree.containers[1].entries.is_empty());
    assert!(
        tree.containers[2]
            .entries
            .iter()
            .any(|entry| entry.id.raw_name.0 == b"Good.class")
    );
    assert!(tree.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic.provenance.as_ref().map(|p| &p.location),
        Some(jarde::Location::Entry { id, span })
            if id.raw_name.0 == b"broken.jar" && span == &tree.containers[0].entries[0].layout.compressed_data
    )));
}

#[test]
fn local_error_then_entry_bytes_budget_reports_the_actual_terminal_budget() {
    let valid = zip(&[(b"Good.class", b"good", STORE)]);
    let outer = zip(&[(b"bad.jar", b"bad", STORE), (b"good.jar", &valid, DEFLATE)]);
    let (snapshot, _) = open(outer);
    let mut configured = limits();
    configured.entry_bytes = 4;
    let mut budget = Budget::new(configured);
    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();
    assert!(matches!(
        tree.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::EntryBytes
            },
            ..
        }
    ));
}

#[test]
fn nested_replay_preserves_depth_entry_count_and_cancellation_errors_without_result_items() {
    let inner = zip(&[(b"Inner.class", b"inner", STORE)]);
    let outer = zip(&[(b"inner.jar", &inner, STORE)]);
    let (snapshot, mut tree_budget) = open(outer);
    let tree = snapshot.enumerate_artifact_tree(&mut tree_budget).unwrap();
    let nested = tree.containers[1].entries[0].clone();

    let mut depth_limits = limits();
    depth_limits.nested_depth = 0;
    let error = snapshot
        .read_entry(&nested, &mut Budget::new(depth_limits))
        .unwrap_err();
    assert!(matches!(
        error,
        jarde::Error::BudgetExceeded {
            dimension: BudgetDimension::NestedDepth,
            ..
        }
    ));

    let mut no_results = limits();
    no_results.result_items = 0;
    let mut budget = Budget::new(no_results);
    assert_eq!(
        snapshot.read_entry(&nested, &mut budget).unwrap().bytes,
        b"inner"
    );
    assert_eq!(budget.usage().result_items, 0);

    let mut no_entries = limits();
    no_entries.archive_entries = 0;
    let error = snapshot
        .read_entry(&nested, &mut Budget::new(no_entries))
        .unwrap_err();
    assert!(matches!(
        error,
        jarde::Error::BudgetExceeded {
            dimension: BudgetDimension::ArchiveEntries,
            ..
        }
    ));

    let mut cancelled = Budget::new(limits());
    cancelled.cancellation_token().cancel();
    assert!(matches!(
        snapshot.read_entry(&nested, &mut cancelled).unwrap_err(),
        jarde::Error::Cancelled { .. }
    ));
}

#[test]
fn war_and_ordinary_nested_layout_are_physical_evidence_only() {
    let child = zip(&[(b"C.class", b"c", STORE)]);
    let outer = zip(&[
        (b"WEB-INF/classes/App.class", b"app", STORE),
        (b"WEB-INF/lib/dependency.jar", &child, STORE),
        (b"other.war", &child, STORE),
    ]);
    let (snapshot, mut budget) = open(outer);
    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();
    assert!(
        tree.layout_nodes
            .iter()
            .any(|node| node.kind == LayoutNodeKind::WarClasses)
    );
    assert!(
        tree.layout_nodes
            .iter()
            .any(|node| node.kind == LayoutNodeKind::WarLibrary)
    );
    assert!(
        tree.layout_nodes
            .iter()
            .any(|node| node.kind == LayoutNodeKind::NestedArchive)
    );
    let json = serde_json::to_string(&tree).unwrap();
    assert!(!json.contains("runtime_view"));
    assert!(!json.contains("selected_definition"));

    let _: ArchiveNameBytes = ArchiveNameBytes(b"lossless".to_vec());
    let _: Option<ContainerOriginStep> = None;
}
