//! P1 positions: every consumer stops at the step the boundary names, and a continuation
//! resumes there.
//!
//! `QueryPosition` is the step of one unit's scan the next item would come from — the resource
//! consumer's step, the code producer's member stream, the metadata consumer's step, the
//! bootstrap consumer's step — and this file pins that the three consumer-owned steps behave
//! like the code producer's: one step can answer several items, a page that fills inside it
//! names *that* step with the number of items it already published, and the continuation
//! re-runs exactly that step and skips that prefix.
//!
//! The scope is an artifact tree with a nested container, so the pages also cross a container
//! boundary: the unit stream reaches the entries of a container one at a time and descends only
//! while the scan keeps pulling, which is the shape a resource request needs — a resource fact
//! is a fact about an entry that is not a class, so its position is the entry itself and carries
//! no constant-pool anchor at all.
//!
//! Everything is asserted through the public entry point over hand-built fixtures: no fixture
//! file, no `/tmp`, no compiler.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

/// Attributes the manifest answers for the target class of the resource assertions.
const MANIFEST_HITS: usize = 5;

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

/// Constant-pool builder: entries keep the 1-based index of the order they were pushed in.
#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    fn push(&mut self, entry: Vec<u8>) -> u16 {
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("the fixture pool fits u16")
    }

    fn utf8(&mut self, text: &[u8]) -> u16 {
        let mut entry = vec![1];
        u16b(
            &mut entry,
            u16::try_from(text.len()).expect("the fixture text fits u16"),
        );
        entry.extend_from_slice(text);
        self.push(entry)
    }

    fn class(&mut self, name: u16) -> u16 {
        let mut entry = vec![7];
        u16b(&mut entry, name);
        self.push(entry)
    }

    fn method_type(&mut self, descriptor: u16) -> u16 {
        let mut entry = vec![16];
        u16b(&mut entry, descriptor);
        self.push(entry)
    }

    fn method_handle(&mut self, kind: u8, reference: u16) -> u16 {
        let mut entry = vec![15, kind];
        u16b(&mut entry, reference);
        self.push(entry)
    }

    fn name_and_type(&mut self, name: u16, descriptor: u16) -> u16 {
        let mut entry = vec![12];
        u16b(&mut entry, name);
        u16b(&mut entry, descriptor);
        self.push(entry)
    }

    fn method_ref(&mut self, class: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![10];
        u16b(&mut entry, class);
        u16b(&mut entry, name_and_type);
        self.push(entry)
    }

    fn invoke_dynamic(&mut self, bootstrap: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![18];
        u16b(&mut entry, bootstrap);
        u16b(&mut entry, name_and_type);
        self.push(entry)
    }

    fn bytes(&self) -> Vec<u8> {
        self.entries.iter().flatten().copied().collect()
    }
}

/// `Code` attribute body with no exception table and no nested attribute.
fn code_body(instructions: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    u16b(&mut body, 2); // max_stack
    u16b(&mut body, 0); // max_locals
    u32b(
        &mut body,
        u32::try_from(instructions.len()).expect("fixture code fits u32"),
    );
    body.extend_from_slice(instructions);
    u16b(&mut body, 0); // exception table
    u16b(&mut body, 0); // attributes
    body
}

/// One class file with the categories this file's positions are about.
///
/// The class implements two interfaces, allocates one class and carries one `invokedynamic`
/// site over a bootstrap table whose entry has three static arguments — so the metadata step
/// answers two items, the code step answers the `new`, and the bootstrap step answers the
/// bootstrap method and its arguments.
struct Fixture {
    bytes: Vec<u8>,
}

impl Fixture {
    /// A class whose super class and interface are the target class, whose body allocates it,
    /// and whose bootstrap table reaches it from a `Class` and a `MethodType` argument.
    ///
    /// Every consumer of the fixture therefore has a fact about the *same* target, which is what
    /// lets one request read the code, the metadata and the bootstrap steps of one unit:
    /// the code step answers the `new`, the metadata step answers the hierarchy, and the
    /// bootstrap step answers the arguments a `CONSTANT_Class` and a `CONSTANT_MethodType` name.
    fn site() -> Self {
        let mut pool = Pool::default();
        let this_name = pool.utf8(b"p/Site");
        let this_class = pool.class(this_name);
        let agent_name = pool.utf8(b"com/example/Agent");
        let agent_class = pool.class(agent_name);
        let void_descriptor = pool.utf8(b"()V");
        let code_attribute = pool.utf8(b"Code");
        let run = pool.utf8(b"run");
        let s = pool.utf8(b"callSite");
        let site_descriptor = pool.utf8(b"()Lp/Site;");
        let site_nat = pool.name_and_type(s, site_descriptor);
        let site = pool.invoke_dynamic(0, site_nat);
        let agent_descriptor = pool.utf8(b"(Lcom/example/Agent;)V");
        let agent_type = pool.method_type(agent_descriptor);
        let target_name = pool.utf8(b"p/Impl");
        let target_class = pool.class(target_name);
        let target_method = pool.utf8(b"make");
        let target_nat = pool.name_and_type(target_method, void_descriptor);
        let target_ref = pool.method_ref(target_class, target_nat);
        let handle = pool.method_handle(6, target_ref);

        // `new com/example/Agent; pop; invokedynamic callSite; pop; return`.
        let mut code = vec![0xbb];
        u16b(&mut code, agent_class);
        code.push(0x57); // pop
        code.push(0xba);
        u16b(&mut code, site);
        code.push(0);
        code.push(0);
        code.push(0x57); // pop
        code.push(0xb1); // return

        let mut methods = Vec::new();
        u16b(&mut methods, 1);
        u16b(&mut methods, 0x0009);
        u16b(&mut methods, run);
        u16b(&mut methods, void_descriptor);
        u16b(&mut methods, 1);
        u16b(&mut methods, code_attribute);
        let body = code_body(&code);
        u32b(
            &mut methods,
            u32::try_from(body.len()).expect("fixture body fits u32"),
        );
        methods.extend_from_slice(&body);

        let bootstrap_name = pool.utf8(b"BootstrapMethods");
        let mut bootstrap = Vec::new();
        u16b(&mut bootstrap, 1); // entries
        u16b(&mut bootstrap, handle);
        u16b(&mut bootstrap, 2); // static arguments
        u16b(&mut bootstrap, agent_class);
        u16b(&mut bootstrap, agent_type);

        let mut bytes = 0xcafe_babe_u32.to_be_bytes().to_vec();
        u16b(&mut bytes, 0); // minor
        u16b(&mut bytes, 52); // major: Java 8
        u16b(
            &mut bytes,
            u16::try_from(pool.entries.len() + 1).expect("the fixture pool fits u16"),
        );
        bytes.extend_from_slice(&pool.bytes());
        u16b(&mut bytes, 0x0031); // public final super
        u16b(&mut bytes, this_class);
        u16b(&mut bytes, agent_class); // super class
        u16b(&mut bytes, 1); // interfaces
        u16b(&mut bytes, agent_class);
        u16b(&mut bytes, 0); // fields
        bytes.extend_from_slice(&methods);
        u16b(&mut bytes, 1); // attributes
        u16b(&mut bytes, bootstrap_name);
        u32b(
            &mut bytes,
            u32::try_from(bootstrap.len()).expect("fixture attribute fits u32"),
        );
        bytes.extend_from_slice(&bootstrap);
        Self { bytes }
    }
}

/// A STORED archive holding the given entries in order.
fn jar_bytes(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
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

/// The manifest of the resource assertions: one `Premain-Class` line per hit, then a named
/// section that no attribute of it may be read as a main-section attribute.
fn manifest() -> Vec<u8> {
    let mut manifest = b"Manifest-Version: 1.0\r\n".to_vec();
    for _ in 0..MANIFEST_HITS {
        manifest.extend_from_slice(b"Premain-Class: com/example/Agent\r\n");
    }
    manifest.extend_from_slice(b"\r\nName: should/not/appear\r\n");
    manifest
}

/// The artifact tree: the manifest, the site class, and a nested container with another site
/// class, so a continuation crosses the container boundary the entry walk owns.
fn artifact() -> Vec<u8> {
    let site = Fixture::site();
    let inner = jar_bytes(&[(b"p/Deep.class", &site.bytes)]);
    let manifest = manifest();
    jar_bytes(&[
        (b"META-INF/MANIFEST.MF", &manifest),
        (b"p/Site.class", &site.bytes),
        (b"lib/inner.jar", &inner),
    ])
}

fn open(input: Vec<u8>) -> ArtifactSnapshot {
    let mut budget = Budget::new(limits());
    Engine::new()
        .open(ArtifactInput::bytes(input), &mut budget)
        .expect("the hand-written fixture must open")
}

/// One request over the whole artifact tree, with the caller's own consumer schema.
fn request(
    snapshot: &ArtifactSnapshot,
    consumers: &[ConsumerKind],
    max_items: u64,
) -> QueryRequest {
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
        consumers: ConsumerSchema::new(1, consumers.iter().copied()),
        max_items,
        cursor: None,
    }
}

fn run(snapshot: &ArtifactSnapshot, request: &QueryRequest) -> QueryReport {
    let mut budget = Budget::new(limits());
    Engine::new()
        .query(snapshot, request, &mut budget)
        .expect("the fixture query must return a report")
}

/// The items of one consumer category, in report order.
fn of_kind(report: &QueryReport, kind: ConsumerKind) -> Vec<&XrefItem> {
    report
        .items
        .iter()
        .filter(|item| item.consumer == Some(kind))
        .collect()
}

/// The boundary of a page that must carry one.
fn boundary(report: &QueryReport) -> QueryBoundary {
    report
        .page
        .cursor
        .as_ref()
        .expect("a truncated page hands out a cursor")
        .boundary
        .clone()
}

/// A page with one item, then a continuation with the same identity, until the scan ends.
fn pages(snapshot: &ArtifactSnapshot, request: &QueryRequest) -> Vec<QueryReport> {
    let mut pages = Vec::new();
    let mut cursor = None;
    loop {
        let mut page_request = request.clone();
        page_request.cursor = cursor.clone();
        let page = run(snapshot, &page_request);
        cursor = page.page.cursor.clone();
        pages.push(page);
        if cursor.is_none() {
            return pages;
        }
        assert!(pages.len() < 32, "pagination must terminate");
    }
}

#[test]
fn a_page_inside_the_metadata_step_names_it_and_replays_only_its_prefix() {
    let snapshot = open(artifact());
    // The `new` instruction is the code step's item; the two implemented interfaces are the
    // metadata step's. A page of two items therefore ends inside the metadata step.
    let full = run(&snapshot, &request(&snapshot, &[ConsumerKind::Type], 0));
    let operations = full
        .items
        .iter()
        .map(|item| item.operation)
        .collect::<Vec<_>>();
    assert_eq!(
        operations
            .iter()
            .filter(|operation| **operation == XrefOperation::New)
            .count(),
        2,
        "the two class candidates each allocate the target: {operations:?}"
    );
    assert_eq!(
        operations
            .iter()
            .filter(|operation| **operation == XrefOperation::Interface)
            .count(),
        2,
        "each class file implements the target class: {operations:?}"
    );
    assert_eq!(
        operations
            .iter()
            .filter(|operation| **operation == XrefOperation::SuperClass)
            .count(),
        2,
        "each class file's super class is the target class: {operations:?}"
    );

    let page_size = 2;
    let mut cursor = None;
    let mut collected = Vec::new();
    let mut saw_metadata_boundary = false;
    let mut pages = 0;
    loop {
        pages += 1;
        assert!(pages <= 32, "pagination must terminate");
        let mut page_request = request(&snapshot, &[ConsumerKind::Type], page_size);
        page_request.cursor = cursor.clone();
        let page = run(&snapshot, &page_request);
        collected.extend(page.items.iter().cloned());
        let next = page.page.cursor.clone();
        if let Some(next) = &next
            && next.boundary.position == QueryPosition::Metadata
        {
            saw_metadata_boundary = true;
            assert!(
                next.boundary.item_index > 0,
                "the boundary counts the metadata items this page published: {:?}",
                next.boundary
            );
        }
        cursor = next;
        if cursor.is_none() {
            break;
        }
    }
    assert!(
        saw_metadata_boundary,
        "one page must fill exactly inside a metadata step"
    );
    assert_eq!(
        collected, full.items,
        "a continuation resumes in the metadata step and neither repeats nor skips"
    );
}

#[test]
fn a_page_inside_the_bootstrap_step_names_it_and_replays_only_its_prefix() {
    let snapshot = open(artifact());
    let consumers = [ConsumerKind::Bootstrap, ConsumerKind::Type];
    let full = run(&snapshot, &request(&snapshot, &consumers, 0));
    let kinds = of_kind(&full, ConsumerKind::Bootstrap);
    assert_eq!(
        kinds.len(),
        2,
        "one bootstrap argument of each class names the target: {:?}",
        full.items
    );
    assert!(
        !of_kind(&full, ConsumerKind::Type).is_empty(),
        "the `MethodType` argument's descriptor names the target as a type fact: {:?}",
        full.items
    );

    let mut cursor = None;
    let mut collected = Vec::new();
    let mut saw_bootstrap_boundary = false;
    loop {
        let mut page_request = request(&snapshot, &consumers, 1);
        page_request.cursor = cursor.clone();
        let page = run(&snapshot, &page_request);
        collected.extend(page.items.iter().cloned());
        let next = page.page.cursor.clone();
        if let Some(next) = &next {
            // A boundary that names the bootstrap step with a non-zero count is a page cut
            // *inside* it; one with a zero count names the step the next item would come from
            // after the metadata step ended exactly at the limit.
            saw_bootstrap_boundary |=
                next.boundary.position == QueryPosition::Bootstrap && next.boundary.item_index > 0;
        }
        cursor = next;
        if cursor.is_none() {
            break;
        }
    }
    assert!(
        saw_bootstrap_boundary,
        "a one-item page must fill inside the bootstrap step"
    );
    assert_eq!(
        collected, full.items,
        "a continuation resumes in the bootstrap step and neither repeats nor skips"
    );
}

#[test]
fn a_page_inside_the_resource_step_names_it_over_a_tree_scope() {
    let snapshot = open(artifact());
    let consumers = [ConsumerKind::Resource];
    let full = run(&snapshot, &request(&snapshot, &consumers, 0));
    assert_eq!(
        full.items.len(),
        MANIFEST_HITS,
        "only the manifest main section answers this target"
    );
    for item in &full.items {
        assert_eq!(
            item.evidence.constant_pool_index, None,
            "a resource fact has no constant-pool anchor: its position is the entry"
        );
    }

    let pages = pages(&snapshot, &request(&snapshot, &consumers, 2));
    let collected = pages
        .iter()
        .flat_map(|page| page.items.iter().cloned())
        .collect::<Vec<_>>();
    assert_eq!(collected, full.items, "the pages tile the unpaged scan");
    let first = boundary(&pages[0]);
    assert_eq!(
        first.position,
        QueryPosition::Resource,
        "the page filled inside the resource step of the manifest unit"
    );
    assert_eq!(first.item_index, 2, "and published these items from it");
    assert_eq!(first.ordinal, 0, "the manifest is the first entry");
    // The continuation re-runs that one step from the entry's own bytes and re-reads its items:
    // the resource step has no constant-pool or member anchor to verify a position against, and
    // does not need one — the unit's container and ordinal are the anchor, and the step is
    // re-derived from the entry's bytes exactly as the first page derived it.
    assert_eq!(
        pages[1].coverage.scanned_items, 4,
        "the replayed prefix of the resumed step is re-read (scanned), never republished"
    );
    assert_eq!(
        pages[1].items.len(),
        2,
        "and the continuation publishes the items after that prefix"
    );
}

#[test]
fn mixed_consumers_share_one_cursor_and_one_item_order() {
    let snapshot = open(artifact());
    let consumers = [
        ConsumerKind::Resource,
        ConsumerKind::Type,
        ConsumerKind::Bootstrap,
    ];
    let full = run(&snapshot, &request(&snapshot, &consumers, 0));
    assert!(matches!(full.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        full.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );

    // One unit stream, one order: the manifest's resource items first, then the site class's
    // code, metadata and bootstrap items, then the nested container's own class.
    let entries = full
        .items
        .iter()
        .map(|item| match &item.source.location {
            Location::Resource { entry, .. } => entry.raw_name.0.clone(),
            Location::Code { method, .. } => method
                .owner
                .entry()
                .expect("every class of this fixture is an archive entry")
                .raw_name
                .0
                .clone(),
            // The metadata consumer's facts are read at a class offset rather than at an
            // instruction, and they belong to the unit's own class either way.
            Location::ClassOffset { definition, .. } => definition
                .entry()
                .expect("every class of this fixture is an archive entry")
                .raw_name
                .0
                .clone(),
            other => panic!("expected a resource, code or class-offset location, got {other:?}"),
        })
        .collect::<Vec<_>>();
    let mut runs: Vec<(Vec<u8>, usize)> = Vec::new();
    for name in &entries {
        match runs.last_mut() {
            Some((last, count)) if last == name => *count += 1,
            _ => runs.push((name.clone(), 1)),
        }
    }
    assert_eq!(
        runs.iter()
            .map(|(name, _)| String::from_utf8_lossy(name).into_owned())
            .collect::<Vec<_>>(),
        vec!["META-INF/MANIFEST.MF", "p/Site.class", "p/Deep.class"],
        "each entry's items come out together, in the scope's own entry order"
    );
    assert_eq!(of_kind(&full, ConsumerKind::Resource).len(), MANIFEST_HITS);

    // Paging across the container boundary reproduces the same document: the cursor binds the
    // container, so a continuation that reaches the nested container names it, not the root.
    let collected = pages(&snapshot, &request(&snapshot, &consumers, 2))
        .into_iter()
        .flat_map(|page| page.items)
        .collect::<Vec<_>>();
    assert_eq!(collected, full.items, "the pages tile the unpaged scan");

    let mut cursor = None;
    let mut crossed = false;
    let mut pages_seen = 0;
    loop {
        pages_seen += 1;
        assert!(pages_seen <= 48, "pagination must terminate");
        let mut page_request = request(&snapshot, &consumers, 2);
        page_request.cursor = cursor.clone();
        let page = run(&snapshot, &page_request);
        let next = page.page.cursor.clone();
        if let Some(next) = &next
            && next.boundary.container.steps.len() == 1
        {
            crossed = true;
            assert_eq!(
                next.boundary.container.steps[0].via_raw_name.0, b"lib/inner.jar",
                "the boundary names the nested container it resumed in"
            );
        }
        cursor = next;
        if cursor.is_none() {
            break;
        }
    }
    assert!(
        crossed,
        "one page must be cut inside the nested container's class"
    );
}

#[test]
fn unimplemented_categories_are_stated_without_changing_the_scanned_facts() {
    let snapshot = open(artifact());
    let resource_only = run(&snapshot, &request(&snapshot, &[ConsumerKind::Resource], 0));
    let with_unsupported = run(
        &snapshot,
        &request(
            &snapshot,
            &[
                ConsumerKind::Resource,
                ConsumerKind::Verification,
                ConsumerKind::Debug,
            ],
            0,
        ),
    );
    assert_eq!(
        with_unsupported.coverage.unsupported_categories,
        vec![ConsumerKind::Verification, ConsumerKind::Debug]
    );
    assert!(resource_only.coverage.unsupported_categories.is_empty());
    assert_eq!(with_unsupported.items, resource_only.items);
    assert_eq!(
        with_unsupported.coverage.scanned_items,
        resource_only.coverage.scanned_items
    );
    assert_eq!(
        with_unsupported
            .coverage
            .dimensions
            .artifact_structural
            .scanned,
        resource_only
            .coverage
            .dimensions
            .artifact_structural
            .scanned,
        "a category this engine cannot scan adds no range to the ones it did scan"
    );
    assert_eq!(
        with_unsupported
            .coverage
            .dimensions
            .artifact_structural
            .state,
        CoverageState::Partial,
        "a request that names a category this engine cannot scan never claims to have covered \
         the artifact structurally"
    );
    assert_eq!(
        resource_only.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(matches!(
        with_unsupported.execution,
        ExecutionReport::Complete { .. }
    ));
}
