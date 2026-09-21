//! The incremental physical traversal cursor (bulk task 3.1, reader half).
//!
//! What these tests pin:
//!
//! * the cursor yields exactly the class candidates `enumerate_artifact_tree` presents, in the same
//!   physical order — containers in entry order, a child container at its own entry, entries in
//!   declaration order — with the same depths;
//! * it charges one `ArchiveEntries` per entry it examines and nothing for the ancestors it already
//!   holds, so discovery does not re-walk a container once per path that reaches it;
//! * a container it cannot read leaves that subtree unknown with a located diagnostic instead of
//!   shortening the denominator, while the root's own failure is an error;
//! * a cancellation is observed at the next entry and the walk does not continue after it;
//! * a standalone `CLASS` snapshot yields its one root, and an artifact-tree scope on it stays an
//!   input error.

use flate2::write::DeflateEncoder;
use jarde::{
    ArtifactInput, ArtifactSnapshot, Budget, CancellationToken, CoverageState, Error,
    ExecutionReport, LayoutNodeSource, Limits, PhysicalClassLocation, PhysicalScope,
    TerminationReason, UsageSnapshot,
};
use jarde_reader::scope_cursor::ScopeClass;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const STORE: u16 = 0;
const DEFLATE: u16 = 8;

/// The class-file fixture the archives hold: a real class, so a candidate that is yielded can be
/// read as a class if a caller wants to.
const CLASS_FIXTURE: &[u8] = include_bytes!("fixtures/p3-scope/v8/Scope.class");

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
        ..Limits::default()
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
                let encoder = DeflateEncoder::new(&mut entry, flate2::Compression::default());
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

fn open(bytes: Vec<u8>, limits: Limits) -> (ArtifactSnapshot, Budget) {
    let mut budget = Budget::new(limits);
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes), &mut budget).unwrap();
    (snapshot, budget)
}

/// The whole-tree scope of a fresh ZIP snapshot, as `PhysicalScope` spells it.
fn tree_scope() -> PhysicalScope {
    PhysicalScope::ArtifactTree {
        root_container: jarde::ContainerId("root".into()),
    }
}

/// The class candidates one scope holds, in the order the artifact-tree walk presents them.
///
/// This is the reference the cursor is compared against, and it is derived from the tree report
/// itself rather than from the fixture: the report's containers are in visit order, and the layout
/// nodes state which entry a child container hangs from, so descending at that entry reproduces the
/// tree walk's own physical order.
fn tree_order(snapshot: &ArtifactSnapshot, budget: &mut Budget) -> Vec<ScopeClass> {
    let report = snapshot.enumerate_artifact_tree(budget).unwrap();
    let mut out = Vec::new();
    let mut containers = report.containers.iter();
    if let Some(root) = containers.next() {
        visit(root, &report, &mut out);
    }
    out
}

fn visit(
    container: &jarde::ContainerReport,
    report: &jarde::ArtifactTreeReport,
    out: &mut Vec<ScopeClass>,
) {
    for entry in &container.entries {
        let child = report
            .layout_nodes
            .iter()
            .find_map(|node| match &node.source {
                LayoutNodeSource::Archive {
                    entry: evidence,
                    child_container,
                } if evidence == &entry.id => child_container.as_ref(),
                _ => None,
            });
        if let Some(child_container) = child
            && let Some(child) = report
                .containers
                .iter()
                .find(|candidate| candidate.origin == *child_container)
        {
            visit(child, report, out);
            continue;
        }
        if entry.id.raw_name.0.ends_with(b".class") {
            out.push(ScopeClass {
                location: PhysicalClassLocation::ArchiveEntry {
                    entry: entry.id.clone(),
                },
                entry: Some(entry.id.clone()),
                depth: container.depth,
            });
        }
    }
}

fn drain(
    cursor: &mut jarde_reader::scope_cursor::ScopeCursor,
    budget: &mut Budget,
) -> Vec<ScopeClass> {
    let mut out = Vec::new();
    while let Some(item) = cursor.next_class(budget).unwrap() {
        out.push(item);
    }
    out
}

/// The refusal code of one structured error.
fn error_code(error: Error) -> String {
    match error {
        Error::InvalidInput { code, .. } | Error::Unsupported { code, .. } => code,
        other => panic!("expected a structured refusal, got {other}"),
    }
}

fn refusal_code<T>(result: jarde::Result<T>) -> String {
    match result {
        Ok(_) => panic!("expected a refusal, got a value"),
        Err(error) => error_code(error),
    }
}

fn charges(usage: &UsageSnapshot) -> (u64, u64) {
    (usage.archive_entries, usage.result_items)
}

fn nested_fixture() -> Vec<u8> {
    let inner = zip(&[
        (b"p/Inner.class", CLASS_FIXTURE, STORE),
        (b"p/readme.txt", b"not a class", STORE),
    ]);
    zip(&[
        (b"BOOT-INF/classes/App.class", CLASS_FIXTURE, STORE),
        (b"BOOT-INF/lib/inner.jar", &inner, DEFLATE),
        (b"top/Top.class", CLASS_FIXTURE, DEFLATE),
        (b"notes.txt", b"a resource", STORE),
    ])
}

#[test]
fn the_cursor_yields_the_tree_walks_candidates_in_the_same_order() {
    let (snapshot, mut budget) = open(nested_fixture(), limits());
    let expected = tree_order(&snapshot, &mut budget);
    assert_eq!(
        expected
            .iter()
            .map(|item| (item.entry.as_ref().unwrap().raw_name.0.clone(), item.depth))
            .collect::<Vec<_>>(),
        vec![
            (b"BOOT-INF/classes/App.class".to_vec(), 0),
            (b"p/Inner.class".to_vec(), 1),
            (b"top/Top.class".to_vec(), 0),
        ],
        "the tree walk really descends at the nested entry and comes back"
    );

    let before = budget.usage();
    let mut cursor = snapshot.scope_cursor(&tree_scope()).unwrap();
    let walked = drain(&mut cursor, &mut budget);
    assert_eq!(walked, expected);
    assert_eq!(cursor.coverage_state(), CoverageState::CompleteWithinSchema);
    assert!(cursor.diagnostics().is_empty());
    assert_eq!(cursor.snapshot(), snapshot.id());
    // One charge per examined entry — the root's four entries and the child's two — and not one
    // more: descending reuses the parent the walk already holds instead of re-reading it.
    assert_eq!(
        charges(&budget.usage()),
        ((before.archive_entries + 6), before.result_items + 6,),
        "the cursor examined exactly the two containers' entries"
    );
    // Descending into the nested library applied the nested-depth rule at that container's depth,
    // exactly as the tree walk does.
    assert_eq!(budget.usage().nested_depth, 1);
    // A stopped cursor keeps answering nothing.
    assert_eq!(cursor.next_class(&mut budget).unwrap(), None);
}

#[test]
fn a_whole_snapshot_scope_is_the_root_containers_own_entries() {
    let archive = nested_fixture();
    let (snapshot, mut budget) = open(archive, limits());
    let mut cursor = snapshot.scope_cursor(&PhysicalScope::SnapshotAll).unwrap();
    let walked = drain(&mut cursor, &mut budget);
    assert_eq!(
        walked
            .iter()
            .map(|item| (item.entry.as_ref().unwrap().raw_name.0.clone(), item.depth))
            .collect::<Vec<_>>(),
        vec![
            (b"BOOT-INF/classes/App.class".to_vec(), 0),
            (b"top/Top.class".to_vec(), 0),
        ],
        "the whole-snapshot scope is the root container, so the nested library is not descended"
    );
    assert_eq!(cursor.coverage_state(), CoverageState::CompleteWithinSchema);
}

#[test]
fn a_standalone_snapshot_yields_its_root_and_refuses_a_tree_scope() {
    let (snapshot, mut budget) = open(CLASS_FIXTURE.to_vec(), limits());
    assert_eq!(
        refusal_code(snapshot.scope_cursor(&tree_scope())),
        "navigation_not_zip"
    );

    let mut cursor = snapshot.scope_cursor(&PhysicalScope::SnapshotAll).unwrap();
    let walked = drain(&mut cursor, &mut budget);
    assert_eq!(
        walked,
        vec![ScopeClass {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone()
            },
            entry: None,
            depth: 0,
        }]
    );
    assert_eq!(cursor.coverage_state(), CoverageState::CompleteWithinSchema);
    assert_eq!(budget.usage().archive_entries, 0);
}

#[test]
fn an_artifact_tree_scope_of_another_root_container_is_refused() {
    let (snapshot, _budget) = open(nested_fixture(), limits());
    assert_eq!(
        refusal_code(snapshot.scope_cursor(&PhysicalScope::ArtifactTree {
            root_container: jarde::ContainerId("not-the-root".into()),
        })),
        "navigation_root_container_mismatch"
    );
}

#[test]
fn a_root_container_that_cannot_be_read_is_an_error() {
    let mut archive = zip(&[(b"p/A.class", CLASS_FIXTURE, STORE)]);
    // Break the entry's local-header signature: the archive still opens — its end-of-directory
    // record and its central directory are intact — but this container's directory cannot be walked
    // to its end, which is exactly the case the scope has no prefix for.
    let local = archive
        .windows(4)
        .position(|window| window == b"PK\x03\x04")
        .expect("a fresh archive starts with a local header");
    archive[local + 2] = b'X';

    let (snapshot, mut budget) = open(archive, limits());
    let mut cursor = snapshot.scope_cursor(&tree_scope()).unwrap();
    let error = cursor
        .next_class(&mut budget)
        .expect_err("the root container's own failure is an error, not an empty scope");
    assert_eq!(error_code(error), "local_entry");
    assert_eq!(cursor.coverage_state(), CoverageState::Partial);
    assert_eq!(cursor.next_class(&mut budget).unwrap(), None);
}

#[test]
fn a_cancelled_request_stops_the_cursor_at_the_next_entry() {
    let token = CancellationToken::new();
    let (snapshot, mut budget) = open(nested_fixture(), limits());
    let mut cursor = snapshot.scope_cursor(&tree_scope()).unwrap();

    let first = cursor
        .next_class(&mut budget)
        .unwrap()
        .expect("the root container declares a class first");
    assert_eq!(
        first.entry.as_ref().unwrap().raw_name.0,
        b"BOOT-INF/classes/App.class"
    );

    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(limits(), token.clone());
    assert!(matches!(
        cursor.next_class(&mut cancelled),
        Err(Error::Cancelled { .. })
    ));
    // The walk does not continue after a stop, and the cursor reports it left the scope unfinished.
    assert_eq!(cursor.next_class(&mut cancelled).unwrap(), None);
    assert_eq!(cursor.coverage_state(), CoverageState::Partial);
}

#[test]
fn a_container_the_walk_cannot_read_leaves_its_subtree_unknown() {
    let broken = zip(&[
        (b"BOOT-INF/classes/Before.class", CLASS_FIXTURE, STORE),
        (
            b"BOOT-INF/lib/broken.jar",
            b"this is not a zip archive",
            STORE,
        ),
        (b"BOOT-INF/classes/After.class", CLASS_FIXTURE, STORE),
    ]);
    let (snapshot, mut budget) = open(broken, limits());

    // The tree walk reports the same unreadable child as a partially read tree with a located
    // diagnostic, which is the shape a caller must not mistake for a complete denominator.
    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();
    assert!(matches!(
        tree.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::Error { .. },
            ..
        }
    ));

    let mut cursor = snapshot.scope_cursor(&tree_scope()).unwrap();
    let walked = drain(&mut cursor, &mut budget);
    assert_eq!(
        walked
            .iter()
            .map(|item| item.entry.as_ref().unwrap().raw_name.0.clone())
            .collect::<Vec<_>>(),
        vec![
            b"BOOT-INF/classes/Before.class".to_vec(),
            b"BOOT-INF/classes/After.class".to_vec(),
        ],
        "the entries after the unreadable child keep being walked"
    );
    assert_eq!(cursor.coverage_state(), CoverageState::Partial);
    let diagnostic = cursor
        .diagnostics()
        .first()
        .expect("the unreadable subtree is published with its position");
    assert_eq!(diagnostic.code, "zip_open");
    let provenance = diagnostic
        .provenance
        .as_ref()
        .expect("a subtree failure names the entry it hangs from");
    match &provenance.location {
        jarde::Location::Entry { id, .. } => {
            assert_eq!(id.raw_name.0, b"BOOT-INF/lib/broken.jar");
            assert_eq!(id.ordinal, 1);
        }
        other => panic!("expected the entry's own location, got {other:?}"),
    }
}

#[test]
fn a_nested_depth_limit_is_reported_the_way_the_tree_walk_reports_it() {
    let depth_limited = Limits {
        nested_depth: 0,
        ..limits()
    };
    let (snapshot, mut budget) = open(nested_fixture(), depth_limited);

    let tree = snapshot.enumerate_artifact_tree(&mut budget).unwrap();
    assert!(matches!(
        tree.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: jarde::BudgetDimension::NestedDepth
            },
            ..
        }
    ));
    assert_eq!(tree.containers.len(), 1, "the child was never scanned");
    assert_eq!(budget.usage().nested_depth, 0);

    let mut cursor = snapshot.scope_cursor(&tree_scope()).unwrap();
    let walked = drain(&mut cursor, &mut budget);
    assert_eq!(
        walked
            .iter()
            .map(|item| item.entry.as_ref().unwrap().raw_name.0.clone())
            .collect::<Vec<_>>(),
        vec![
            b"BOOT-INF/classes/App.class".to_vec(),
            b"top/Top.class".to_vec(),
        ]
    );
    assert_eq!(cursor.coverage_state(), CoverageState::Partial);
    assert_eq!(
        cursor.diagnostics().first().map(|d| d.code.clone()),
        Some("budget_exceeded_nested_depth".to_owned())
    );
    assert_eq!(
        budget.usage().nested_depth,
        0,
        "a refused descent does not raise the accepted-depth high water"
    );
}

#[test]
fn a_walk_that_has_not_reached_the_end_reports_a_prefix_not_a_denominator() {
    // The completeness plane is the reader's own statement about the scope, and a progressive
    // consumer reads it *before* knowing whether more is coming: a cursor that has yielded one
    // candidate of a scope it has not exhausted cannot state that the scope is covered, because the
    // denominator it would be stating is not known yet. Only reaching the end of the scope (with
    // nothing left unknown) makes it complete.
    let (snapshot, mut budget) = open(nested_fixture(), limits());
    let mut cursor = snapshot.scope_cursor(&tree_scope()).unwrap();
    assert_eq!(
        cursor.coverage_state(),
        CoverageState::Partial,
        "a cursor that has not walked anything yet states no coverage"
    );
    let first = cursor
        .next_class(&mut budget)
        .unwrap()
        .expect("the fixture declares a class first");
    assert_eq!(
        first.entry.as_ref().unwrap().raw_name.0,
        b"BOOT-INF/classes/App.class"
    );
    assert_eq!(
        cursor.coverage_state(),
        CoverageState::Partial,
        "one candidate of a scope the walk has not finished is a prefix, not a denominator"
    );
    assert!(
        cursor.diagnostics().is_empty(),
        "and nothing failed: this is what 'not finished yet' looks like"
    );

    // The same cursor, walked to its end: now the plane may state a complete scope.
    let rest = drain(&mut cursor, &mut budget);
    assert_eq!(rest.len(), 2, "the two candidates after the first");
    assert_eq!(
        cursor.coverage_state(),
        CoverageState::CompleteWithinSchema,
        "the walk reached the end of an undamaged scope"
    );
}

#[test]
fn a_standalone_snapshot_reports_its_scope_complete_only_behind_its_one_root() {
    // The standalone scope's one item is not the whole scope by itself: the walk still has to find
    // that nothing follows it, which is what the next pull states.
    let (snapshot, mut budget) = open(CLASS_FIXTURE.to_vec(), limits());
    let mut cursor = snapshot.scope_cursor(&PhysicalScope::SnapshotAll).unwrap();
    assert_eq!(cursor.coverage_state(), CoverageState::Partial);
    let root = cursor
        .next_class(&mut budget)
        .unwrap()
        .expect("the standalone root is the scope's one candidate");
    assert!(root.entry.is_none(), "the standalone candidate is the root");
    assert_eq!(
        cursor.coverage_state(),
        CoverageState::Partial,
        "the walk has not been to the end of the scope yet"
    );
    assert_eq!(cursor.next_class(&mut budget).unwrap(), None);
    assert_eq!(
        cursor.coverage_state(),
        CoverageState::CompleteWithinSchema,
        "and now it has"
    );
}

#[test]
fn a_cancelled_request_yields_nothing_at_all_not_even_the_first_candidate() {
    // Cancellation is observed before **every** item, the standalone root and the first entry of a
    // container included: a request that was already cancelled neither examines an entry nor hands a
    // candidate on, so a consumer can never receive an item from a request whose budget says stop.
    let token = CancellationToken::new();
    token.cancel();

    let (zip_snapshot, _budget) = open(nested_fixture(), limits());
    let mut zip_cursor = zip_snapshot.scope_cursor(&tree_scope()).unwrap();
    let mut cancelled = Budget::with_cancellation_token(limits(), token.clone());
    assert!(
        matches!(
            zip_cursor.next_class(&mut cancelled),
            Err(Error::Cancelled { .. })
        ),
        "the tree walk observes the cancellation before its first entry"
    );
    assert_eq!(
        zip_cursor.coverage_state(),
        CoverageState::Partial,
        "and a stopped walk never claims a complete scope"
    );
    assert_eq!(zip_cursor.next_class(&mut cancelled).unwrap(), None);

    let (standalone, _budget) = open(CLASS_FIXTURE.to_vec(), limits());
    let mut standalone_cursor = standalone
        .scope_cursor(&PhysicalScope::SnapshotAll)
        .unwrap();
    assert!(
        matches!(
            standalone_cursor.next_class(&mut cancelled),
            Err(Error::Cancelled { .. })
        ),
        "the standalone root is yielded only by a request that may still work"
    );
    assert_eq!(standalone_cursor.next_class(&mut cancelled).unwrap(), None);
    assert_eq!(standalone_cursor.coverage_state(), CoverageState::Partial);
}

#[test]
fn a_cancellation_after_a_candidate_keeps_the_prefix_it_confirmed() {
    // The other half: a walk that was cancelled after it handed something over keeps what it
    // handed over, states that the scope is not covered, and yields nothing more.
    let token = CancellationToken::new();
    let (snapshot, mut budget) = open(nested_fixture(), limits());
    let mut cursor = snapshot.scope_cursor(&tree_scope()).unwrap();
    let prefix = [cursor
        .next_class(&mut budget)
        .unwrap()
        .expect("the fixture declares a class first")];
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(limits(), token.clone());
    assert!(matches!(
        cursor.next_class(&mut cancelled),
        Err(Error::Cancelled { .. })
    ));
    assert_eq!(
        budget.usage().archive_entries,
        4,
        "the prefix really was charged before the stop: the root's four entries"
    );
    assert_eq!(cursor.next_class(&mut cancelled).unwrap(), None);
    assert_eq!(cursor.coverage_state(), CoverageState::Partial);
    assert_eq!(
        prefix
            .iter()
            .map(|item| item.entry.as_ref().unwrap().raw_name.0.clone())
            .collect::<Vec<_>>(),
        vec![b"BOOT-INF/classes/App.class".to_vec()],
        "the candidate the walk already handed over stands"
    );
}
