//! P1 demand-driven scan acceptance: a page bounds the *work*, not only the published list.
//!
//! The fixture is one artifact-tree scope with the shape the review asked for:
//!
//! * `p/Dense.class` declares `dense()V` — `DENSE_HITS` invocations of `p/Target.hit:()V` —
//!   followed by `LATER_MEMBERS` members that each invoke the same target once,
//! * `p/Filler.class` is a later entry with a member of its own,
//! * `lib/inner.jar` is a nested container holding `p/Inner.class`, so the walk has to
//!   materialize it only if it really reaches that entry.
//!
//! Everything below is asserted through the public entry point, and every archive and class
//! file is built in process from this file's own builders: no fixture file, no `/tmp`, no
//! compiler.
//!
//! What the tests pin, and in which terms:
//!
//! 1. a small page decodes the member the page filled in and *nothing* behind it — the
//!    counted evidence is the budget's `code_bytes` (instruction bytes of the one member),
//!    the byte dimensions of the read (the later entries' bytes are never read) and the
//!    store's own container counters (`nested_materializations` 0, `directory_parses` 1),
//!    never a duration;
//! 2. the range behind the page stays *unknown*: the coverage names the entry the
//!    invocation examined and claims no range behind it, and it never says "complete" or
//!    "empty" about work it did not do;
//! 3. the continuation splices back to the unpaged scan exactly: same items, same order,
//!    no repeat and no gap, and the page that reached the end is the one that reports
//!    `complete_within_schema`;
//! 4. the continuation resumes at the recorded *step*: the page that filled exactly at the
//!    last item of a member keeps a boundary naming the next member (its raw name and
//!    descriptor included), so the member it already published is not decoded again.
//!
//! The counter-example this file is written against is the old shape: a scan that collects
//! a unit's items before it applies the page size. Every number the small page pins moves
//! under it — the later members' code bytes, the entries' read bytes and the store's
//! nested-container counter — which is exactly why they are asserted here.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

/// Invocations of the target in the first (dense) member of `p/Dense`.
const DENSE_HITS: usize = 64;

/// Members after the dense one, all in the same class file.
const LATER_MEMBERS: usize = 24;

/// Small page used by every pagination assertion.
const PAGE: u64 = 4;

/// Code bytes the dense member costs: one three-byte `invokevirtual` per hit plus the
/// one-byte `return`.
const DENSE_CODE_BYTES: u64 = (DENSE_HITS as u64) * 3 + 1;

/// Code bytes one single-hit member costs.
const SINGLE_CODE_BYTES: u64 = 4;

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
        nested_depth: 4,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

/// A store that retains nothing: it answers no lookup, so it changes no path's work, and
/// its report is the count of the directories a request parsed and the nested containers
/// it materialized.
fn counting_store() -> FactsCache {
    FactsCache::new(FactsIdentity::current(), FactsCapacity::none())
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

    fn bytes(&self) -> Vec<u8> {
        self.entries.iter().flatten().copied().collect()
    }
}

/// One class file whose members are `public static` `()V` methods, one per `(name, hits)`,
/// each invoking `p/Target.hit:()V` `hits` times and returning.
fn class_bytes(members: &[(&[u8], usize)]) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_name = pool.utf8(b"p/Dense");
    let this_class = pool.class(this_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let super_class = pool.class(object_name);
    let void_descriptor = pool.utf8(b"()V");
    let code_attribute = pool.utf8(b"Code");
    let target_name = pool.utf8(b"p/Target");
    let target_owner = pool.class(target_name);
    let hit = pool.utf8(b"hit");
    let hit_and_type = pool.name_and_type(hit, void_descriptor);
    let target = pool.method_ref(target_owner, hit_and_type);
    let names = members
        .iter()
        .map(|(name, _)| pool.utf8(name))
        .collect::<Vec<_>>();

    let mut methods = Vec::new();
    u16b(
        &mut methods,
        u16::try_from(members.len()).expect("the fixture member count fits u16"),
    );
    for ((_, hits), name) in members.iter().zip(&names) {
        let mut code = Vec::with_capacity(hits * 3 + 1);
        for _ in 0..*hits {
            code.push(0xb6); // invokevirtual p/Target.hit:()V
            u16b(&mut code, target);
        }
        code.push(0xb1); // return

        let mut body = Vec::new();
        u16b(&mut body, 1); // max_stack
        u16b(&mut body, 0); // max_locals: a static member takes none
        u32b(
            &mut body,
            u32::try_from(code.len()).expect("the fixture code fits u32"),
        );
        body.extend_from_slice(&code);
        u16b(&mut body, 0); // exception table
        u16b(&mut body, 0); // code attributes

        u16b(&mut methods, 0x0009); // public static
        u16b(&mut methods, *name);
        u16b(&mut methods, void_descriptor);
        u16b(&mut methods, 1); // attributes
        u16b(&mut methods, code_attribute);
        u32b(
            &mut methods,
            u32::try_from(body.len()).expect("the fixture body fits u32"),
        );
        methods.extend_from_slice(&body);
    }

    let mut bytes = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0); // minor
    u16b(&mut bytes, 52); // major: Java 8
    u16b(
        &mut bytes,
        u16::try_from(pool.entries.len() + 1).expect("the fixture pool fits u16"),
    );
    bytes.extend_from_slice(&pool.bytes());
    u16b(&mut bytes, 0x0021); // public super
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, super_class);
    u16b(&mut bytes, 0); // interfaces
    u16b(&mut bytes, 0); // fields
    bytes.extend_from_slice(&methods);
    u16b(&mut bytes, 0); // class attributes
    bytes
}

/// The members of `p/Dense`: the dense one first, then the later ones.
fn dense_members() -> Vec<(Vec<u8>, usize)> {
    let mut members = vec![(b"dense".to_vec(), DENSE_HITS)];
    for index in 0..LATER_MEMBERS {
        members.push((format!("later{index}").into_bytes(), 1));
    }
    members
}

/// A STORED archive holding exactly the given entries, in the given order.
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

/// The scope under test: `p/Dense.class`, `p/Filler.class` and a nested container, in that
/// entry order.
fn artifact() -> Vec<u8> {
    let dense = {
        let members = dense_members();
        let borrowed = members
            .iter()
            .map(|(name, hits)| (name.as_slice(), *hits))
            .collect::<Vec<_>>();
        class_bytes(&borrowed)
    };
    let filler = class_bytes(&[(b"filler", 1)]);
    let inner = jar_bytes(&[(b"p/Inner.class", &class_bytes(&[(b"inner", 1)]))]);
    jar_bytes(&[
        (b"p/Dense.class", &dense),
        (b"p/Filler.class", &filler),
        (b"lib/inner.jar", &inner),
    ])
}

fn open(input: Vec<u8>) -> ArtifactSnapshot {
    let mut budget = Budget::new(limits());
    Engine::new()
        .open(ArtifactInput::bytes(input), &mut budget)
        .expect("the hand-written fixture must open")
}

/// One request over the whole artifact tree: the nested container is part of the scope.
fn request(snapshot: &ArtifactSnapshot, max_items: u64) -> QueryRequest {
    QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target: QueryTarget::Symbol {
            value: SymbolRef::Method {
                owner: JvmBytes(b"p/Target".to_vec()),
                name: JvmBytes(b"hit".to_vec()),
                descriptor: JvmBytes(b"()V".to_vec()),
            },
        },
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::ArtifactTree {
                root_container: ContainerId("root".into()),
            },
        },
        consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
        max_items,
        cursor: None,
    }
}

/// One run with its own budget and its own counting store.
fn run(
    snapshot: &ArtifactSnapshot,
    request: &QueryRequest,
) -> (QueryReport, UsageSnapshot, FactsReport) {
    let store = counting_store();
    let mut budget = Budget::new(limits()).with_facts_cache(store.clone());
    let report = Engine::new()
        .query(snapshot, request, &mut budget)
        .expect("the fixture query must return a report");
    (report, budget.usage(), store.report())
}

/// The BCI of one item, for order assertions.
fn bci(item: &XrefItem) -> u32 {
    item.evidence
        .bci
        .expect("an invocation item carries its BCI")
}

/// The raw name of the entry one item was read from.
fn entry_name(item: &XrefItem) -> Vec<u8> {
    let Location::Code { method, .. } = &item.source.location else {
        panic!("every item of this fixture comes from a code location");
    };
    method
        .owner
        .entry()
        .expect("every class of this fixture is an archive entry")
        .raw_name
        .0
        .clone()
}

/// The container origin of one item's class, for the nested-container assertions.
fn container_of(item: &XrefItem) -> ContainerId {
    let Location::Code { method, .. } = &item.source.location else {
        panic!("every item of this fixture comes from a code location");
    };
    method
        .owner
        .entry()
        .expect("every class of this fixture is an archive entry")
        .origin
        .current_container()
        .clone()
}

#[test]
fn a_small_page_stops_before_the_work_behind_it() {
    let snapshot = open(artifact());
    let (report, usage, store) = run(&snapshot, &request(&snapshot, PAGE));

    // The page publishes exactly its own limit, all of it from the dense member.
    assert_eq!(u64::try_from(report.items.len()).unwrap(), PAGE);
    assert_eq!(report.page.returned_items, PAGE);
    for item in &report.items {
        assert_eq!(entry_name(item), b"p/Dense.class");
    }
    assert_eq!(
        report.items.iter().map(bci).collect::<Vec<_>>(),
        vec![0, 3, 6, 9],
        "the page carries the first instructions of the dense member"
    );

    // The dense member was decoded — and no other member.
    assert_eq!(
        usage.code_bytes, DENSE_CODE_BYTES,
        "only the member the page filled in was decoded: a later member would add its own \
         four instruction bytes here"
    );

    // The later entries' bytes were never read: the dense class file is the only entry
    // materialized, and the nested container was neither materialized nor even opened.
    let dense_len = class_bytes_of(&dense_members()).len() as u64;
    assert_eq!(
        (usage.read_bytes, usage.entry_bytes),
        (dense_len, dense_len),
        "the only entry whose bytes were read is the dense class file: a stored read is \
         charged in its compressed and its logical byte dimension"
    );
    assert_eq!(
        store.nested_materializations, 0,
        "a small page never descends into the nested container behind it"
    );
    assert_eq!(
        store.directory_parses, 1,
        "the walk opened the root container's directory and nothing else"
    );

    // The range behind the page stays unknown: the invocation names the entry it examined
    // and claims nothing behind it — neither as examined nor as empty.
    assert!(report.page.has_more);
    assert!(report.page.cursor.is_some());
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial,
        "a page that stopped never claims the scope was covered"
    );
    let scanned = &report.coverage.dimensions.artifact_structural.scanned;
    assert_eq!(
        scanned
            .iter()
            .filter(|range| range.label == "container:root:xref_scan_entries")
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        vec![(0, 1)],
        "the walk examined the dense entry alone"
    );
    let full = full_report(&snapshot);
    let nested = container_of(full.items.last().expect("one item inside"));
    assert!(
        scanned.iter().all(|range| !range.label.contains(&nested.0)),
        "no range claims the nested container: it was never reached: {scanned:?}"
    );
    assert!(
        report
            .coverage
            .dimensions
            .artifact_structural
            .skipped
            .is_empty(),
        "nothing behind the page is named as skipped either: it is unknown"
    );
    assert_eq!(
        report.coverage.scanned_items, PAGE,
        "the invocation counted exactly the items it examined"
    );
    assert!(
        !matches!(report.execution, ExecutionReport::Failed { .. }),
        "a page limit is not a failure: {:?}",
        report.execution
    );
}

/// The unpaged run of the same identity, used as the reference the pages splice to.
fn full_report(snapshot: &ArtifactSnapshot) -> QueryReport {
    let (report, _, _) = run(snapshot, &request(snapshot, 0));
    report
}

/// The dense class file's own bytes, as the fixture builder produces them.
fn class_bytes_of(members: &[(Vec<u8>, usize)]) -> Vec<u8> {
    let borrowed = members
        .iter()
        .map(|(name, hits)| (name.as_slice(), *hits))
        .collect::<Vec<_>>();
    class_bytes(&borrowed)
}

#[test]
fn the_continuation_splices_back_to_the_unpaged_scan() {
    let snapshot = open(artifact());
    let full = full_report(&snapshot);
    assert_eq!(
        u64::try_from(full.items.len()).unwrap(),
        DENSE_HITS as u64 + LATER_MEMBERS as u64 + 2,
        "the unpaged scan finds the dense member, every later member, the filler and the \
         class inside the nested container"
    );
    assert!(matches!(full.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        full.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema,
        "the unpaged scan covered the whole scope"
    );

    let mut cursor = None;
    let mut collected: Vec<XrefItem> = Vec::new();
    let mut pages = 0_u32;
    let mut anchored = false;
    loop {
        pages += 1;
        assert!(pages <= 64, "pagination must terminate");
        let mut page_request = request(&snapshot, PAGE);
        page_request.cursor = cursor.clone();
        let (page, _, store) = run(&snapshot, &page_request);
        assert!(page.page.returned_items <= PAGE);
        assert_eq!(
            u64::try_from(page.items.len()).unwrap(),
            page.page.returned_items
        );
        let start = collected.len();
        let length = page.items.len();
        collected.extend(page.items.iter().cloned());
        assert_eq!(
            collected[start..],
            full.items[start..start + length],
            "page {pages} carries the next items of the unpaged scan, in order"
        );
        match page.page.cursor.clone() {
            Some(next) => {
                assert!(page.page.has_more);
                assert!(
                    !matches!(
                        page.coverage.dimensions.artifact_structural.state,
                        CoverageState::CompleteWithinSchema
                    ),
                    "a page that stopped does not claim the whole scope"
                );
                // The dense member's items come out 4 at a time, so some page ends exactly
                // at its last one: that page's boundary names the *next* member with its
                // own raw name and descriptor, which is the anchor a continuation checks.
                if let QueryPosition::Method {
                    index,
                    name,
                    descriptor,
                } = &next.boundary.position
                    && !anchored
                {
                    assert_eq!(*index, 1, "the member after the dense one");
                    assert_eq!(name.0, b"later0");
                    assert_eq!(descriptor.0, b"()V");
                    anchored = true;
                }
                cursor = Some(next);
            }
            None => {
                assert!(!page.page.has_more, "the last page ends the scan");
                assert_eq!(
                    page.coverage.dimensions.artifact_structural.state,
                    CoverageState::CompleteWithinSchema,
                    "the page that reached the end of the scope reports it"
                );
                assert!(
                    page.coverage
                        .dimensions
                        .artifact_structural
                        .skipped
                        .is_empty(),
                    "the walk that reached the end has nothing unexamined"
                );
                assert!(
                    store.nested_materializations > 0,
                    "the page that reached the nested container's entry materialized it, \
                     which is the work a small page never pays for"
                );
                break;
            }
        }
    }
    assert!(
        anchored,
        "one page must end exactly at the dense member's last item"
    );
    assert_eq!(
        collected, full.items,
        "the pages must not repeat or skip a single item"
    );
    // Whole pages cover the scope; the page after them only exists when the last one was
    // *full*: a page that filled exactly at a step's last item stops before the next step
    // and keeps a conservative continuation, while a page with room left runs to the end of
    // the scope and reports it.
    let total = DENSE_HITS as u64 + LATER_MEMBERS as u64 + 2;
    let full_last_page = u32::from(total.is_multiple_of(PAGE));
    assert_eq!(
        pages,
        u32::try_from(total.div_ceil(PAGE)).unwrap() + full_last_page,
        "one page per whole page of items, plus the page a full last page's continuation needs"
    );
}

#[test]
fn a_page_that_ends_at_a_member_boundary_does_not_decode_it_again() {
    let snapshot = open(artifact());
    // The first page's limit is exactly the dense member's item count, so it ends at that
    // member's last item and its boundary names the member that follows it.
    let (first, first_usage, _) = run(&snapshot, &request(&snapshot, DENSE_HITS as u64));
    assert_eq!(u64::try_from(first.items.len()).unwrap(), DENSE_HITS as u64);
    assert_eq!(
        first_usage.code_bytes, DENSE_CODE_BYTES,
        "the page decoded exactly the member it published from"
    );
    let cursor = first.page.cursor.clone().expect("a truncated page");
    assert_eq!(
        cursor.boundary.position,
        QueryPosition::Method {
            index: 1,
            name: JvmBytes(b"later0".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        },
        "the position names the next member by its own class-file identity"
    );

    // The continuation resumes *at that member*: the dense member it already published is
    // not decoded a second time, so its code bytes are not charged again.
    let mut continuation = request(&snapshot, DENSE_HITS as u64);
    continuation.cursor = Some(cursor);
    let (second, second_usage, store) = run(&snapshot, &continuation);
    assert_eq!(
        second_usage.code_bytes,
        (LATER_MEMBERS as u64 + 2) * SINGLE_CODE_BYTES,
        "the continuation decoded the later members, the filler and the nested entry's \
         class — and not the dense member again"
    );
    assert_eq!(
        u64::try_from(second.items.len()).unwrap(),
        LATER_MEMBERS as u64 + 2
    );
    assert!(
        !second.page.has_more && second.page.cursor.is_none(),
        "that page reached the end of the scope"
    );
    assert!(
        store.nested_materializations > 0,
        "reaching the nested container's entry materializes it, unlike the page that \
         stopped before it"
    );
}

#[test]
fn a_replayed_prefix_is_never_published_twice() {
    let snapshot = open(artifact());
    // Two pages of four items over the dense member: the second page resumes *inside* the
    // member, skips the four items the first page published and publishes the next four.
    let (first, _, _) = run(&snapshot, &request(&snapshot, PAGE));
    assert_eq!(
        first
            .page
            .cursor
            .as_ref()
            .map(|cursor| cursor.boundary.clone()),
        Some(QueryBoundary {
            container: first
                .page
                .cursor
                .as_ref()
                .unwrap()
                .boundary
                .container
                .clone(),
            ordinal: 0,
            position: QueryPosition::Code,
            item_index: PAGE,
        }),
        "a page cut inside the member that opened the code producer names that step"
    );
    let mut continuation = request(&snapshot, PAGE);
    continuation.cursor = first.page.cursor.clone();
    let (second, second_usage, _) = run(&snapshot, &continuation);
    assert_eq!(
        second.items.iter().map(bci).collect::<Vec<_>>(),
        vec![12, 15, 18, 21],
        "the continuation starts after the published prefix"
    );
    assert_eq!(
        second.coverage.scanned_items,
        PAGE * 2,
        "the replayed prefix is re-read once (scanned) but never published again"
    );
    assert_eq!(
        second_usage.code_bytes, DENSE_CODE_BYTES,
        "resuming inside a member re-decodes that one body — the reader decodes a body as \
         one unit — and nothing behind it"
    );
}
