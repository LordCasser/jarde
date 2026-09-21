//! P1 page planes: what each page really states about coverage, execution and diagnostics.
//!
//! Three different endings can leave a page short and a cursor behind, and a caller has to be able
//! to tell them apart without guessing:
//!
//! * a **full page** stopped because the caller's own item limit was reached — `execution` stays
//!   `Complete`, no terminal diagnostic is published, and only the *coverage* says the scope is
//!   not covered yet;
//! * a **cancelled** page stopped because the request was cancelled — `execution` is `Cancelled`
//!   and the terminal diagnostic names it;
//! * a **budget stop** names the dimension it exhausted, in the execution and in the diagnostic.
//!
//! What a page may *not* do is state more than it examined: a container the walk never reached
//! contributes no range and no denominator (unknown, not empty), a class candidate whose bytes are
//! damaged is reported when the page really reads it and not before, and a cursor is refused
//! unless it describes the binding that issued it — so no page can inherit a prefix it did not
//! examine from the cursor it was handed.
//!
//! Everything is asserted through the public entry point over hand-built fixtures: no fixture
//! file, no `/tmp`, no compiler.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

/// Items the dense manifest answers, so a page can fill inside the first entry of the scope.
const MANIFEST_HITS: usize = 6;

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 1_000,
        method_bodies: 1_000,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn u16b(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn u32b(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

/// A minimal, complete class file whose one member allocates `com/example/Agent`.
fn class_file() -> Vec<u8> {
    let mut pool: Vec<Vec<u8>> = Vec::new();
    let utf8 = |pool: &mut Vec<Vec<u8>>, text: &[u8]| {
        let mut entry = vec![1];
        u16b(
            &mut entry,
            u16::try_from(text.len()).expect("the fixture text fits u16"),
        );
        entry.extend_from_slice(text);
        pool.push(entry);
        u16::try_from(pool.len()).expect("the fixture pool fits u16")
    };
    let class = |pool: &mut Vec<Vec<u8>>, name: u16| {
        let mut entry = vec![7];
        u16b(&mut entry, name);
        pool.push(entry);
        u16::try_from(pool.len()).expect("the fixture pool fits u16")
    };
    let this_name = utf8(&mut pool, b"p/Seed");
    let this_class = class(&mut pool, this_name);
    let object_name = utf8(&mut pool, b"java/lang/Object");
    let super_class = class(&mut pool, object_name);
    let agent_name = utf8(&mut pool, b"com/example/Agent");
    let agent_class = class(&mut pool, agent_name);
    let void_descriptor = utf8(&mut pool, b"()V");
    let code_attribute = utf8(&mut pool, b"Code");
    let run = utf8(&mut pool, b"run");

    let mut code = vec![0xbb]; // new com/example/Agent
    u16b(&mut code, agent_class);
    code.push(0x57); // pop
    code.push(0xb1); // return
    let mut body = Vec::new();
    u16b(&mut body, 1); // max_stack
    u16b(&mut body, 0); // max_locals
    u32b(
        &mut body,
        u32::try_from(code.len()).expect("the fixture code fits u32"),
    );
    body.extend_from_slice(&code);
    u16b(&mut body, 0); // exception table
    u16b(&mut body, 0); // code attributes

    let mut methods = Vec::new();
    u16b(&mut methods, 1); // methods
    u16b(&mut methods, 0x0009); // public static
    u16b(&mut methods, run);
    u16b(&mut methods, void_descriptor);
    u16b(&mut methods, 1); // attributes
    u16b(&mut methods, code_attribute);
    u32b(
        &mut methods,
        u32::try_from(body.len()).expect("the fixture body fits u32"),
    );
    methods.extend_from_slice(&body);

    let mut bytes = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0); // minor
    u16b(&mut bytes, 52); // major: Java 8
    u16b(
        &mut bytes,
        u16::try_from(pool.len() + 1).expect("the fixture pool fits u16"),
    );
    for entry in &pool {
        bytes.extend_from_slice(entry);
    }
    u16b(&mut bytes, 0x0021); // public super
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, super_class);
    u16b(&mut bytes, 0); // interfaces
    u16b(&mut bytes, 0); // fields
    bytes.extend_from_slice(&methods);
    u16b(&mut bytes, 0); // class attributes
    bytes
}

/// One `Premain-Class` main-section line per hit, then a named section.
fn manifest() -> Vec<u8> {
    let mut manifest = b"Manifest-Version: 1.0\r\n".to_vec();
    for _ in 0..MANIFEST_HITS {
        manifest.extend_from_slice(b"Premain-Class: com/example/Agent\r\n");
    }
    manifest.extend_from_slice(b"\r\n");
    manifest
}

/// A STORED archive holding the given entries in order.
fn jar_bytes(entries: &[(&[u8], Vec<u8>)]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .expect("the fixture entry starts");
            let mut writer = config.wrap(&mut entry);
            writer.write_all(data).expect("the fixture payload writes");
            let (_, descriptor) = writer.finish().expect("the fixture payload finishes");
            entry
                .finish(descriptor)
                .expect("the fixture entry finishes");
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

/// The fixture: the dense manifest, a readable class, a *damaged* class candidate, and behind it
/// a nested container holding one more class — in that entry order.
fn artifact() -> Vec<u8> {
    let class = class_file();
    let inner = jar_bytes(&[(b"p/Deep.class", class.clone())]);
    jar_bytes(&[
        (b"META-INF/MANIFEST.MF", manifest()),
        (b"p/Readable.class", class.clone()),
        (b"p/Broken.class", vec![0xca, 0xfe, 0xba]),
        (b"lib/inner.jar", inner),
    ])
}

/// The same tree without the damaged candidate, so a traversal can reach the nested container.
fn clean_artifact() -> Vec<u8> {
    let class = class_file();
    let inner = jar_bytes(&[(b"p/Deep.class", class.clone())]);
    jar_bytes(&[
        (b"META-INF/MANIFEST.MF", manifest()),
        (b"p/Readable.class", class),
        (b"lib/inner.jar", inner),
    ])
}

fn open(input: Vec<u8>) -> ArtifactSnapshot {
    let mut budget = Budget::new(limits());
    Engine::new()
        .open(ArtifactInput::bytes(input), &mut budget)
        .expect("the hand-written fixture must open")
}

fn request(snapshot: &ArtifactSnapshot, max_items: u64) -> QueryRequest {
    QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target: QueryTarget::Symbol {
            value: SymbolRef::Class {
                owner: JvmBytes(b"com/example/Agent".to_vec()),
            },
        },
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::ArtifactTree {
                root_container: ContainerId("root".into()),
            },
        },
        consumers: ConsumerSchema::new(1, [ConsumerKind::Resource, ConsumerKind::Type]),
        max_items,
        cursor: None,
    }
}

fn run(snapshot: &ArtifactSnapshot, request: &QueryRequest) -> QueryReport {
    let mut budget = Budget::new(limits());
    Engine::new()
        .query(snapshot, request, &mut budget)
        .expect("a legal query is answered, not raised")
}

/// Every range label one structural dimension side states, for the "nothing unknown is claimed"
/// assertions.
fn labels(ranges: &[CoverageRange]) -> Vec<String> {
    ranges.iter().map(|range| range.label.clone()).collect()
}

/// `(start, end)` of every `container:root:xref_scan_entries` range of one side.
fn root_ranges(ranges: &[CoverageRange]) -> Vec<(u64, u64)> {
    ranges
        .iter()
        .filter(|range| range.label == "container:root:xref_scan_entries")
        .map(|range| (range.start, range.end))
        .collect()
}

#[test]
fn a_full_page_a_cancellation_and_a_budget_stop_are_three_states() {
    let snapshot = open(clean_artifact());
    let page_size = 2;

    let full_page = run(&snapshot, &request(&snapshot, page_size));
    assert_eq!(full_page.items.len(), page_size as usize);
    assert_eq!(
        full_page.page.returned_items,
        u64::try_from(full_page.items.len()).unwrap()
    );
    assert!(full_page.page.has_more && full_page.page.cursor.is_some());
    assert!(
        matches!(full_page.execution, ExecutionReport::Complete { .. }),
        "a page limit is not a degraded execution: {:?}",
        full_page.execution
    );
    assert!(
        full_page.diagnostics.is_empty(),
        "a page limit publishes no terminal diagnostic: {:?}",
        full_page.diagnostics
    );
    assert_eq!(
        full_page.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial,
        "the page states the range it did not cover"
    );

    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let cancelled = Engine::new()
        .query(&snapshot, &request(&snapshot, 0), &mut budget)
        .expect("a cancelled query is answered, not raised");
    assert!(matches!(
        cancelled.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert!(cancelled.items.is_empty());
    assert!(
        cancelled.page.has_more,
        "a cancelled scan never reached the end"
    );
    assert!(
        cancelled
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "cancelled")
    );
    let cancelled_structural = &cancelled.coverage.dimensions.artifact_structural;
    assert_eq!(cancelled_structural.state, CoverageState::Partial);
    assert_eq!(
        labels(&cancelled_structural.scanned),
        Vec::<String>::new(),
        "a cancelled scan examined nothing, so it scanned no range"
    );
    assert_eq!(
        cancelled_structural
            .skipped
            .iter()
            .map(|range| (range.label.clone(), range.start, range.end))
            .collect::<Vec<_>>(),
        vec![
            ("central_directory_entries".to_string(), 0, 3),
            ("container:root:xref_scan_entries".to_string(), 0, 3)
        ],
        "the scope's own directory declared how many entries it holds, and none of them was \
         examined: the range is stated as skipped work instead of vanishing"
    );

    let mut budget = Budget::new(Limits {
        result_items: 3,
        ..limits()
    });
    let budgeted = Engine::new()
        .query(&snapshot, &request(&snapshot, 0), &mut budget)
        .expect("a budget stop is answered, not raised");
    match &budgeted.execution {
        ExecutionReport::Partial { reason, usage } => {
            assert_eq!(
                reason,
                &TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::ResultItems
                }
            );
            assert_eq!(usage.result_items, 3);
        }
        other => panic!("an exhausted dimension is a partial execution, got {other:?}"),
    }
    assert!(
        budgeted.page.has_more && budgeted.page.cursor.is_none(),
        "a stop before the first unit published no item, so there is no position to resume from: \
         the page says the scope is not covered and hands out no cursor"
    );
    let budgeted_structural = &budgeted.coverage.dimensions.artifact_structural;
    assert_eq!(
        budgeted_structural
            .scanned
            .iter()
            .filter(|range| range.label == "central_directory_entries")
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        vec![(0, 3)],
        "the directory the walk validated is stated even though the stop followed it: {:?}",
        budgeted_structural.scanned
    );
    assert_eq!(
        root_ranges(&budgeted_structural.skipped),
        vec![(1, 3)],
        "and the entries no unit reached are named as the range this invocation did not examine"
    );
    assert_eq!(
        budgeted.items.len(),
        0,
        "a charge refused before the first item publishes no item: the prefix is empty, not wrong"
    );
    assert!(
        budgeted
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "budget_exceeded_result_items"),
        "{:?}",
        budgeted.diagnostics
    );
    assert_eq!(
        budgeted.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );

    // The three endings are told apart by the planes a caller reads, not by their item counts
    // alone: a full page is `Complete` with no terminal diagnostic, a cancellation is
    // `Cancelled`, and a budget stop names its own dimension.
    let full_diagnostics = labels_codes(&full_page);
    assert!(
        !full_diagnostics.contains(&"cancelled".to_string())
            && !full_diagnostics.contains(&"budget_exceeded_result_items".to_string())
    );
    assert_ne!(labels_codes(&cancelled), full_diagnostics);
    assert_ne!(labels_codes(&budgeted), full_diagnostics);
    assert_ne!(
        labels_codes(&cancelled),
        labels_codes(&budgeted),
        "a cancellation and an exhausted dimension are not the same terminal fact"
    );
}

/// The diagnostic codes one report publishes, sorted so the comparison is about the set.
fn labels_codes(report: &QueryReport) -> Vec<String> {
    let mut codes = report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.clone())
        .collect::<Vec<_>>();
    codes.sort();
    codes
}

#[test]
fn a_damaged_class_candidate_is_reported_when_the_page_reads_it() {
    let snapshot = open(artifact());
    let page_size = 2;

    // The first page stops inside the manifest and never reads the damaged candidate behind it.
    let first = run(&snapshot, &request(&snapshot, page_size));
    assert_eq!(first.items.len(), page_size as usize);
    assert!(
        first
            .items
            .iter()
            .all(|item| item.consumer == Some(ConsumerKind::Resource)),
        "the page is the manifest's own items"
    );
    assert!(
        !first
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.starts_with("query_class_candidate")),
        "a suffix this page never reached is not diagnosed yet: {:?}",
        first.diagnostics
    );
    assert!(matches!(first.execution, ExecutionReport::Complete { .. }));
    assert!(first.page.has_more);
    assert_eq!(
        root_ranges(&first.coverage.dimensions.artifact_structural.scanned),
        vec![(0, 1)]
    );

    // The page that reaches it reports it, with the entry it belongs to and the bytes it saw.
    let mut continuation = request(&snapshot, 0);
    continuation.cursor = first.page.cursor.clone();
    let reached = run(&snapshot, &continuation);
    assert!(matches!(
        reached.execution,
        ExecutionReport::Failed {
            reason: TerminationReason::Error { ref code },
            ..
        } if code == "query_class_candidate_malformed"
    ));
    let diagnostic = reached
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "query_class_candidate_malformed")
        .expect("the damaged candidate is reported");
    match diagnostic
        .provenance
        .as_ref()
        .map(|provenance| &provenance.location)
    {
        Some(Location::Entry { id, span }) => {
            assert_eq!(id.raw_name.0, b"p/Broken.class");
            assert_eq!(id.ordinal, 2);
            assert_eq!(*span, ByteSpan::new(0, 3));
        }
        other => panic!("expected the damaged entry as the diagnostic origin, got {other:?}"),
    }
    assert_eq!(
        reached.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
    assert!(
        reached.page.has_more,
        "the fail-stop cannot be read as the end of the range"
    );

    // Nothing behind the damage is claimed, and the manifestation of the damage is one range:
    // the entry the scan examined before it read the damaged candidate.
    let structural = &reached.coverage.dimensions.artifact_structural;
    assert_eq!(
        root_ranges(&structural.scanned),
        vec![(0, 3)],
        "the page examined the manifest, the readable class and the damaged candidate itself"
    );
    assert_eq!(
        structural
            .skipped
            .iter()
            .map(|range| (range.label.clone(), range.start, range.end))
            .collect::<Vec<_>>(),
        vec![("container:root:xref_scan_entries".to_string(), 3, 4)],
        "the entry behind the fail-stop stays unknown — named as skipped work of the container \
         whose denominator the walk established, and not as an entry it read"
    );
    assert!(
        structural
            .scanned
            .iter()
            .all(|range| !range.label.starts_with("container:") || range.label.contains("root")),
        "no range claims a container this invocation never opened: {:?}",
        structural.scanned
    );
}

#[test]
fn a_container_the_page_never_reaches_is_unknown_and_not_empty() {
    let snapshot = open(clean_artifact());
    let page_size = 2;

    let first = run(&snapshot, &request(&snapshot, page_size));
    let structural = &first.coverage.dimensions.artifact_structural;
    assert_eq!(
        labels(&structural.scanned),
        vec![
            "central_directory_entries".to_string(),
            "container:root:xref_scan_entries".to_string()
        ],
        "the page states the container it validated and the entry it examined, and nothing else"
    );
    assert_eq!(
        root_ranges(&structural.skipped),
        vec![(1, 3)],
        "the entries of the container it validated that no unit reached are named as skipped"
    );
    assert_eq!(
        structural
            .scanned
            .iter()
            .filter(|range| range.label == "central_directory_entries")
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        vec![(0, 3)],
        "the denominator of that container is the one its own directory declared"
    );
    assert!(
        structural
            .scanned
            .iter()
            .chain(&structural.skipped)
            .all(|range| !range.label.contains("container:") || range.label.contains(":root:")),
        "no range names or claims the nested container: it was never reached"
    );

    // The traversal that really reaches it states it, so the small page's silence is a statement
    // about its own work and not about the artifact.
    let full = run(&snapshot, &request(&snapshot, 0));
    assert!(
        full.coverage
            .dimensions
            .artifact_structural
            .scanned
            .iter()
            .any(|range| range.label.starts_with("container:") && !range.label.contains(":root:")),
        "the whole traversal states the nested container it opened: {:?}",
        full.coverage.dimensions.artifact_structural.scanned
    );
}

#[test]
fn a_cursor_is_refused_unless_it_describes_the_binding_that_issued_it() {
    let snapshot = open(artifact());
    let first = run(&snapshot, &request(&snapshot, 2));
    let cursor = first.page.cursor.clone().expect("a truncated page");

    // A cursor from another binding — same snapshot, another consumer schema — is refused before
    // any unit is read, so no prefix can be replayed under the wrong identity.
    let mut other_identity = request(&snapshot, 2);
    other_identity.consumers = ConsumerSchema::new(1, [ConsumerKind::Resource]);
    other_identity.cursor = Some(cursor.clone());
    let error = Engine::new()
        .query(&snapshot, &other_identity, &mut Budget::new(limits()))
        .unwrap_err();
    assert_eq!(
        cursor_code(&error),
        "query_cursor_mismatch",
        "a cursor only describes the prefix of the binding that issued it"
    );

    // A cursor whose boundary was edited keeps the digest of the boundary it was issued for, so
    // the binding check refuses it instead of scanning from a position nobody established. Both
    // halves of the boundary are bound: the unit ordinal and the *step* the next item would come
    // from — a forged step would silently skip the unit's remaining items.
    for forged in [
        {
            let mut forged = cursor.clone();
            forged.boundary.ordinal += 3;
            forged
        },
        {
            // The same unit, the same item count, another step: without the position in the binding
            // this continuation would resume past the unit's remaining items.
            let mut forged = cursor.clone();
            forged.boundary.position = QueryPosition::UnitComplete;
            forged
        },
    ] {
        let mut forged_request = request(&snapshot, 2);
        forged_request.cursor = Some(forged.clone());
        let error = Engine::new()
            .query(&snapshot, &forged_request, &mut Budget::new(limits()))
            .unwrap_err();
        assert_eq!(
            cursor_code(&error),
            "query_cursor_mismatch",
            "a boundary of {:?} must be refused",
            forged.boundary
        );
    }

    // The same cursor with a page size it was not issued under is *not* refused: the page size and
    // the budget are execution knobs rather than identity, so a continuation may change them.
    let mut continued = request(&snapshot, 1);
    continued.cursor = Some(cursor);
    let page = run(&snapshot, &continued);
    assert_eq!(page.items.len(), 1);
    assert_eq!(
        page.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial,
        "the page that resumed states the ranges it examined itself"
    );
    assert_eq!(
        root_ranges(&page.coverage.dimensions.artifact_structural.scanned),
        vec![(0, 1)],
        "the resumed page examined the unit it resumed in, and nothing else"
    );
}

/// The stable code of one query error.
fn cursor_code(error: &Error) -> String {
    match error {
        Error::InvalidInput { code, .. } => code.clone(),
        other => panic!("expected an input refusal, got {other:?}"),
    }
}

#[test]
fn a_root_directory_that_does_not_parse_is_refused_as_a_whole() {
    // A container whose central directory contradicts its own declared count is not a container
    // product: no record of it was verified, so the request is answered with the refusal and the
    // range that directory declared, never with the records that happened to parse before the
    // damage. This is the reader's directed-access contract, and the query states it instead of
    // publishing a prefix nobody established.
    let mut bytes = jar_bytes(&[
        (b"META-INF/MANIFEST.MF", manifest()),
        (b"p/Readable.class", class_file()),
    ]);
    let eocd = bytes.len() - 22;
    assert_eq!(
        u32::from_le_bytes(bytes[eocd..eocd + 4].try_into().unwrap()),
        0x0605_4b50,
        "the end-of-central-directory record is where this fixture says it is"
    );
    bytes[eocd + 8..eocd + 10].copy_from_slice(&3_u16.to_le_bytes());
    bytes[eocd + 10..eocd + 12].copy_from_slice(&3_u16.to_le_bytes());
    let snapshot = open(bytes.clone());

    let mut request = request(&snapshot, 0);
    request.physical.scope = PhysicalScope::SnapshotAll;
    let report = run(&snapshot, &request);
    assert!(report.items.is_empty());
    assert!(
        matches!(
            report.execution,
            ExecutionReport::Failed {
                reason: TerminationReason::Error { ref code },
                ..
            } if code == "entry_count_mismatch"
        ),
        "{:?}",
        report.execution
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "entry_count_mismatch"),
        "{:?}",
        report.diagnostics
    );
    assert!(report.page.has_more && report.page.cursor.is_none());
    let structural = &report.coverage.dimensions.artifact_structural;
    assert_eq!(structural.state, CoverageState::Partial);
    assert_eq!(
        labels(&structural.scanned),
        Vec::<String>::new(),
        "no record of the container was verified, so no range may claim one"
    );
    assert_eq!(
        structural
            .skipped
            .iter()
            .map(|range| (range.label.clone(), range.start, range.end))
            .collect::<Vec<_>>(),
        vec![
            ("central_directory_entries".to_string(), 0, 3),
            ("container:root:xref_scan_entries".to_string(), 0, 3)
        ],
        "the declared range is stated as work this invocation did not do: {:?}",
        structural.skipped
    );
}
