//! `add-artifact-navigation`: listing an artifact's classes and members, and handing back identities.
//!
//! The contract this file holds is the change's two deltas, and every archive here is built in memory
//! by this file (the repository's own `rawzip` dev-dependency) so a case can pin the exact
//! containment, order, corruption and budget it is about:
//!
//! * **two evidence levels, kept apart.** A class *candidate* is selected by the raw-name rule alone
//!   and reads **zero** headers; a class *declaration* is what one real header read established. A
//!   path is not a declaration, and no report here lets one stand for the other.
//! * **containment preserved.** The same class name — even the same bytes — at two physical origins
//!   stays two items with their own origins and ordinals. Nothing is merged and nothing is first-wins.
//! * **listing starts no analysis.** Only the class bytes and their member tables are read: no body is
//!   decoded, and the resolver/CFG/SSA/region/AST construction dimensions stay at zero.
//! * **friendly input, physical result.** Two spellings of one name bind the same physical identity,
//!   ambiguity returns every candidate with the identity that decides between them, and no match is an
//!   empty candidate list over the range that was scanned.
//!
//! The handoff cases use the committed ECJ 4.6.1 sample beside hand-built archives, because the point
//! there is that a *real* listed identity drives a real analysis request to the same definition.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::slice;

const STORE: u16 = 0;

/// The one class name the two-origin archives share.
const SHARED: &[u8] = b"p/S";

/// Limits that fund every dimension a listing spends, unless a case says otherwise.
fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 10_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 10_000,
        output_bytes: 1 << 20,
        class_headers: 100,
        method_bodies: 100,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut Budget::new(limits()))
        .expect("the fixture snapshot opens")
}

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

fn text(value: &[u8]) -> String {
    String::from_utf8_lossy(value).into_owned()
}

/// The physical origin one definition is reported under.
fn definition_provenance(definition: &PhysicalDefinitionId) -> Provenance {
    Provenance {
        location: Location::ClassOffset {
            definition: definition.clone(),
            offset: 0,
        },
    }
}

/// The diagnostic codes of one report's diagnostics, in order.
fn diagnostic_codes(diagnostics: &[Diagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.clone())
        .collect()
}

/// The four construction dimensions A17 is about, read through the counted enum so a renamed
/// dimension is a compile error instead of a silently unchecked field.
fn construction(usage: &UsageSnapshot) -> Vec<(CountedBudgetDimension, u64)> {
    [
        CountedBudgetDimension::IrItems,
        CountedBudgetDimension::IrEdges,
        CountedBudgetDimension::AnalysisSteps,
        CountedBudgetDimension::NormalizationClones,
    ]
    .into_iter()
    .map(|dimension| (dimension, usage.counted_usage(dimension)))
    .collect()
}

/// Every dimension a listing must leave untouched, named so a failure says which one moved.
fn assert_no_analysis(label: &str, usage: &UsageSnapshot) {
    let construction = construction(usage);
    assert!(
        construction.iter().all(|(_, count)| *count == 0),
        "{label}: a listing builds no IR item, no edge, no step and no clone: {construction:?}"
    );
    assert_eq!(
        (usage.method_bodies, usage.code_bytes),
        (0, 0),
        "{label}: a listing decodes no body: {usage:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------------------------

fn u16b(output: &mut Vec<u8>, value: u16) {
    output.extend_from_slice(&value.to_be_bytes());
}

fn u32b(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes());
}

/// One constant-pool `Utf8` entry, appended in order, returning its index.
fn utf8(pool: &mut Vec<Vec<u8>>, value: &[u8]) -> u16 {
    let mut entry = vec![1];
    u16b(
        &mut entry,
        u16::try_from(value.len()).expect("the fixture text fits u16"),
    );
    entry.extend_from_slice(value);
    pool.push(entry);
    u16::try_from(pool.len()).expect("the fixture pool fits u16")
}

/// One constant-pool `Class` entry pointing at an `Utf8` entry.
fn class_entry(pool: &mut Vec<Vec<u8>>, name_index: u16) -> u16 {
    let mut entry = vec![7];
    u16b(&mut entry, name_index);
    pool.push(entry);
    u16::try_from(pool.len()).expect("the fixture pool fits u16")
}

/// One field or method record of a hand-built class.
///
/// A field and a method are the same record shape (`flags`, `name`, `descriptor`, attributes), and
/// the one attribute this builder writes is `Code`: a real bounded shell around the given content, or
/// a deliberately lying length for the damaged-record case.
struct MemberSpec<'a> {
    name: &'a [u8],
    descriptor: &'a [u8],
    access_flags: u16,
    code: Option<&'a [u8]>,
    declared_code_length: Option<u32>,
}

impl<'a> MemberSpec<'a> {
    /// A concrete method: `ACC_PUBLIC` and one `return` inside a `Code` attribute.
    fn method(name: &'a [u8], descriptor: &'a [u8]) -> Self {
        Self {
            name,
            descriptor,
            access_flags: 0x0001,
            code: Some(b"\xb1"),
            declared_code_length: None,
        }
    }

    /// An `abstract` method: `ACC_PUBLIC | ACC_ABSTRACT`, no `Code`.
    fn abstract_method(name: &'a [u8], descriptor: &'a [u8]) -> Self {
        Self {
            name,
            descriptor,
            access_flags: 0x0401,
            code: None,
            declared_code_length: None,
        }
    }

    /// A `native` method: `ACC_PUBLIC | ACC_NATIVE`, no `Code`.
    fn native_method(name: &'a [u8], descriptor: &'a [u8]) -> Self {
        Self {
            name,
            descriptor,
            access_flags: 0x0101,
            code: None,
            declared_code_length: None,
        }
    }

    fn field(name: &'a [u8], descriptor: &'a [u8]) -> Self {
        Self {
            name,
            descriptor,
            access_flags: 0x0001,
            code: None,
            declared_code_length: None,
        }
    }

    /// The same method with a `Code` length that is not its content's own: the damaged record.
    fn damaged(mut self, declared_code_length: u32) -> Self {
        self.declared_code_length = Some(declared_code_length);
        self
    }
}

/// One hand-built class file: `this_class`, `java/lang/Object` as superclass, the given members.
fn class_file(
    this_class: &[u8],
    major: u16,
    fields: &[MemberSpec<'_>],
    methods: &[MemberSpec<'_>],
) -> Vec<u8> {
    let mut pool = Vec::new();
    let this_utf8 = utf8(&mut pool, this_class);
    let this_class_index = class_entry(&mut pool, this_utf8);
    let object_utf8 = utf8(&mut pool, b"java/lang/Object");
    let object_class_index = class_entry(&mut pool, object_utf8);
    let mut indices = Vec::new();
    for member in fields.iter().chain(methods.iter()) {
        indices.push((
            utf8(&mut pool, member.name),
            utf8(&mut pool, member.descriptor),
        ));
    }
    let code_name = utf8(&mut pool, b"Code");

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
    u16b(&mut bytes, 0);
    u16b(&mut bytes, major);
    u16b(
        &mut bytes,
        u16::try_from(pool.len() + 1).expect("the fixture pool fits u16"),
    );
    for entry in &pool {
        bytes.extend_from_slice(entry);
    }
    u16b(&mut bytes, 0x0021);
    u16b(&mut bytes, this_class_index);
    u16b(&mut bytes, object_class_index);
    u16b(&mut bytes, 0);
    u16b(
        &mut bytes,
        u16::try_from(fields.len()).expect("the fixture fits u16"),
    );
    let mut next = 0usize;
    for member in fields {
        let (name, descriptor) = indices[next];
        next += 1;
        write_member(&mut bytes, member, name, descriptor, code_name);
    }
    u16b(
        &mut bytes,
        u16::try_from(methods.len()).expect("the fixture fits u16"),
    );
    for member in methods {
        let (name, descriptor) = indices[next];
        next += 1;
        write_member(&mut bytes, member, name, descriptor, code_name);
    }
    u16b(&mut bytes, 0);
    bytes
}

fn write_member(
    bytes: &mut Vec<u8>,
    member: &MemberSpec<'_>,
    name: u16,
    descriptor: u16,
    code_name: u16,
) {
    u16b(bytes, member.access_flags);
    u16b(bytes, name);
    u16b(bytes, descriptor);
    match member.code {
        None => u16b(bytes, 0),
        Some(code) => {
            u16b(bytes, 1);
            u16b(bytes, code_name);
            u32b(
                bytes,
                member
                    .declared_code_length
                    .unwrap_or_else(|| u32::try_from(code.len()).expect("fixture code fits u32")),
            );
            bytes.extend_from_slice(code);
        }
    }
}

/// One stored ZIP with the given entries, in the given order.
fn zip_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .expect("the fixture entry starts");
            let mut writer = config.wrap(&mut entry);
            writer
                .write_all(data)
                .expect("the fixture entry is written");
            let (_, descriptor) = writer.finish().expect("the fixture entry closes");
            entry
                .finish(descriptor)
                .expect("the fixture entry finishes");
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

/// A WAR holding the same class name at two origins: an upper directory entry and a nested library.
///
/// `upper_shared` and `nested_shared` are the bytes each origin holds for the shared name, so a case
/// can give the two origins the same bytes (containment, not content, is what makes them two) or
/// different ones (so a read of the wrong definition is visible in the answer). Both containers also
/// hold a `p/Only` class of their own, which keeps the shared name's candidate count honest.
fn two_origin_war(upper_shared: &[u8], nested_shared: &[u8]) -> Vec<u8> {
    let only = class_file(b"p/Only", 52, &[], &[MemberSpec::method(b"only", b"()V")]);
    let library = zip_of(&[(b"p/S.class", nested_shared), (b"p/Only.class", &only)]);
    zip_of(&[
        (b"WEB-INF/classes/p/S.class", upper_shared),
        (b"WEB-INF/classes/p/Only.class", &only),
        (b"WEB-INF/lib/L.jar", &library),
        (b"META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\n"),
    ])
}

fn root_origin(snapshot: &ArtifactSnapshot) -> ContainerOrigin {
    ContainerOrigin {
        snapshot: snapshot.id().clone(),
        root_container: ContainerId("root".into()),
        steps: Vec::new(),
    }
}

fn tree_scope(snapshot: &ArtifactSnapshot) -> PhysicalScope {
    PhysicalScope::ArtifactTree {
        root_container: root_origin(snapshot).root_container,
    }
}

/// The one caller domain rooted at the given position, with no resolution provider: the simplest
/// environment a method request is accepted under.
fn environment(snapshot: &ArtifactSnapshot, root: LoadRoot) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![root],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    ResolutionEnvironment {
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
    }
}

/// The raw names of one listing's methods, in the read's own order.
fn method_names(listing: &MemberListing) -> Vec<String> {
    listing
        .methods()
        .map(|method| text(&method.identity.name.0))
        .collect()
}

// ---------------------------------------------------------------------------------------------
// 1.1 / 1.2 / A07 / A08 / A16: the listing levels, containment and the read counters
// ---------------------------------------------------------------------------------------------

/// The candidate listing is names only: its partition is total, and it reads zero headers.
#[test]
fn a_candidate_listing_reads_no_header_and_keeps_containment() {
    let engine = Engine::new();
    let shared = class_file(SHARED, 52, &[], &[MemberSpec::method(b"upper", b"()V")]);
    let snapshot = open(two_origin_war(&shared, &shared));
    let mut budget = Budget::new(limits());
    let listing = engine
        .list_class_candidates(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the root container lists");

    let candidates: Vec<&PhysicalEntryId> = listing
        .candidates()
        .map(|location| location.entry().expect("a container candidate is an entry"))
        .collect();
    let resources: Vec<String> = listing
        .resources()
        .map(|entry| text(&entry.raw_name.0))
        .collect();
    assert_eq!(
        candidates
            .iter()
            .map(|entry| text(&entry.raw_name.0))
            .collect::<Vec<_>>(),
        vec!["WEB-INF/classes/p/S.class", "WEB-INF/classes/p/Only.class"],
        "the snapshot scope is the root container: its own class-looking entries are the candidates"
    );
    assert_eq!(
        resources,
        vec!["WEB-INF/lib/L.jar", "META-INF/MANIFEST.MF"],
        "every entry the rule does not select is published as the ordinary resource it is"
    );
    assert_eq!(
        listing.items.len(),
        4,
        "the partition is total: every enumerated entry appears exactly once"
    );
    let usage = budget.usage();
    assert_eq!(
        (
            usage.class_headers,
            usage.class_bytes,
            usage.code_bytes,
            usage.method_bodies
        ),
        (0, 0, 0, 0),
        "the candidate listing read no class at all: {usage:?}"
    );
    assert_eq!(
        usage.result_items, 8,
        "the four entries the enumeration holds and the four items this listing publishes: {usage:?}"
    );
    assert_eq!(
        usage.archive_entries, 4,
        "the root container's four records were read once: {usage:?}"
    );
    assert!(matches!(
        listing.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(listing.diagnostics.is_empty());
    assert_no_analysis("class candidate listing", &usage);
}

/// One name and the same bytes at two origins: two items, never one.
#[test]
fn the_same_class_at_two_origins_stays_two_items() {
    let engine = Engine::new();
    let shared = class_file(SHARED, 52, &[], &[MemberSpec::method(b"upper", b"()V")]);
    let snapshot = open(two_origin_war(&shared, &shared));
    let scope = tree_scope(&snapshot);
    let mut budget = Budget::new(limits());
    let listing = engine
        .list_class_declarations(&snapshot, &scope, &mut budget)
        .expect("the tree lists");
    assert_eq!(
        listing.candidates, 4,
        "the tree walks both containers: two class entries in each of them"
    );
    let shared_items: Vec<&ClassDeclarationItem> = listing
        .items
        .iter()
        .filter(|item| item.declaration.this_class.raw().0 == SHARED)
        .collect();
    assert_eq!(
        shared_items.len(),
        2,
        "the same name at an upper directory entry and in the explicitly expanded nested library is two items: {:?}",
        listing.items
    );
    let upper = shared_items
        .iter()
        .find(|item| item.definition.entry().unwrap().origin.steps.is_empty())
        .expect("the application directory's own record");
    let nested = shared_items
        .iter()
        .find(|item| !item.definition.entry().unwrap().origin.steps.is_empty())
        .expect("the nested library's own record");
    let upper_entry = upper.definition.entry().unwrap();
    let nested_entry = nested.definition.entry().unwrap();
    assert_eq!(upper_entry.raw_name.0, b"WEB-INF/classes/p/S.class");
    assert_eq!(nested_entry.raw_name.0, b"p/S.class");
    assert_eq!(
        nested_entry.origin.steps[0].via_raw_name.0, b"WEB-INF/lib/L.jar",
        "the nested item keeps the entry that reaches it"
    );
    assert_ne!(
        (upper_entry.origin.current_container(), upper_entry.ordinal),
        (
            nested_entry.origin.current_container(),
            nested_entry.ordinal
        ),
        "each definition keeps its own container and its own ordinal"
    );
    assert_eq!(
        upper.definition.class_bytes, nested.definition.class_bytes,
        "the bytes really are the same, which is what makes this a containment claim and not a content one"
    );
    assert_eq!(
        (upper.binding.clone(), nested.binding.clone()),
        (
            ClassNameBinding::PathNameAgrees,
            ClassNameBinding::PathNameAgrees
        )
    );
    assert_eq!(listing.items[0].declaration.access_flags, 0x0021);
    assert_eq!(
        budget.usage().class_headers,
        4,
        "one header read attempt per candidate, and no more: {:?}",
        budget.usage()
    );
    assert!(matches!(
        listing.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(listing.diagnostics.is_empty());
    assert_no_analysis("confirmed class listing", &budget.usage());
}

/// A path that disagrees with the declared name keeps both names, and binds neither.
#[test]
fn a_path_that_disagrees_with_the_declared_name_keeps_both() {
    let engine = Engine::new();
    let wrong = class_file(b"p/Real", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let snapshot = open(zip_of(&[(b"p/Wrong.class", &wrong)]));
    let mut budget = Budget::new(limits());
    let listing = engine
        .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the container confirms its one candidate");
    assert_eq!(listing.items.len(), 1);
    let item = &listing.items[0];
    assert_eq!(
        item.declaration.this_class.raw().0,
        b"p/Real",
        "the declared name is the header's own fact"
    );
    assert_eq!(
        item.binding,
        ClassNameBinding::PathNameDiffers {
            path_name: bytes(b"p/Wrong")
        },
        "the path's own name is published beside it, neither rewriting the other"
    );
    assert_eq!(
        item.definition
            .entry()
            .expect("an archive entry")
            .raw_name
            .0,
        b"p/Wrong.class",
        "the physical raw name stays the entry's own"
    );
    assert_eq!(
        item.member_table, None,
        "the member table of this read was read to its end"
    );
    assert!(
        listing.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "navigation_path_name_mismatch" && diagnostic.provenance.is_some()
        }),
        "the disagreement is reported with the entry's physical origin: {:?}",
        listing.diagnostics
    );
    assert!(
        matches!(listing.execution, ExecutionReport::Complete { .. }),
        "a mismatch is a domain finding, not a failure of the scan: {:?}",
        listing.execution
    );
}

/// A candidate whose declaration cannot be read ends the scan with the prefix it did confirm.
#[test]
fn a_damaged_candidate_ends_the_confirmed_scan_with_a_reliable_prefix() {
    let engine = Engine::new();
    let readable = class_file(b"p/A", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let broken = b"\xca\xfe\xba\xbe\x00\x00\x00\x34".to_vec();
    let snapshot = open(zip_of(&[
        (b"p/A.class", &readable),
        (b"p/Broken.class", &broken),
        (b"p/C.class", &readable),
    ]));

    let mut candidates_budget = Budget::new(limits());
    let candidates = engine
        .list_class_candidates(
            &snapshot,
            &PhysicalScope::SnapshotAll,
            &mut candidates_budget,
        )
        .expect(
            "the candidate listing never reads a header, so the damaged bytes stay a candidate",
        );
    assert_eq!(candidates.candidates().count(), 3);
    assert!(
        candidates
            .candidates()
            .any(|location| location.entry().unwrap().raw_name.0 == b"p/Broken.class"),
        "the damaged entry is a candidate: its path matched the rule, which is all a candidate claims"
    );
    assert!(
        candidates.diagnostics.is_empty(),
        "a candidate listing has no diagnostic to state about bytes it never read"
    );
    assert_eq!(candidates_budget.usage().class_headers, 0);

    let mut budget = Budget::new(limits());
    let listing = engine
        .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("a failed candidate read is reported, not raised");
    assert_eq!(listing.candidates, 3);
    assert_eq!(
        listing.items.len(),
        1,
        "only the candidate whose declaration was really read enters the confirmed set: {:?}",
        listing.items
    );
    assert_eq!(listing.items[0].declaration.this_class.raw().0, b"p/A");
    assert_eq!(
        listing
            .unconfirmed
            .iter()
            .map(|location| text(&location.entry().unwrap().raw_name.0))
            .collect::<Vec<_>>(),
        vec!["p/Broken.class", "p/C.class"],
        "the unread candidates keep their physical identity and their order"
    );
    let failed = listing
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .expect("the damaged candidate is named");
    assert_eq!(failed.code, "classfile_decode");
    assert_eq!(
        failed
            .provenance
            .as_ref()
            .expect("the diagnostic carries the entry")
            .location,
        Location::Entry {
            id: listing.unconfirmed[0].entry().unwrap().clone(),
            span: ByteSpan::new(0, u64::try_from(broken.len()).unwrap()),
        },
        "the diagnostic is located at the physical entry whose read failed"
    );
    match &listing.execution {
        ExecutionReport::Failed { reason, .. } => assert_eq!(
            reason,
            &TerminationReason::Error {
                code: "classfile_decode".into()
            }
        ),
        other => panic!("a structure this read could not read is `Failed`: {other:?}"),
    }
    assert_eq!(
        listing.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    assert!(
        listing
            .coverage
            .artifact_structural
            .skipped
            .contains(&CoverageRange {
                label: "class_declarations".into(),
                start: 2,
                end: 3
            }),
        "the candidates the scan never reached are marked: {:?}",
        listing.coverage.artifact_structural
    );
    assert_eq!(
        budget.usage().class_headers,
        2,
        "the attempt that failed was charged, and the candidates after it were not read: {:?}",
        budget.usage()
    );
}

/// A budget stop publishes the confirmed prefix and names the dimension that stopped it.
#[test]
fn a_budget_stop_keeps_the_confirmed_prefix() {
    let engine = Engine::new();
    let readable = class_file(b"p/A", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let snapshot = open(zip_of(&[
        (b"p/A.class", &readable),
        (b"p/B.class", &readable),
        (b"p/C.class", &readable),
    ]));
    let mut budget = Budget::new(Limits {
        class_headers: 1,
        ..limits()
    });
    let listing = engine
        .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("an exhausted dimension is a stop, not an error");
    assert_eq!(listing.items.len(), 1);
    assert_eq!(listing.unconfirmed.len(), 2);
    assert_eq!(
        diagnostic_codes(&listing.diagnostics),
        vec!["budget_exceeded_class_headers"]
    );
    assert_eq!(
        listing.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ClassHeaders
            },
            usage: budget.usage(),
        },
        "the stop names the dimension that ran out and carries the request's whole usage"
    );
    assert_eq!(
        listing.coverage.artifact_structural.state,
        CoverageState::Partial
    );
}

/// A cancelled request publishes what it read and says that it was cancelled.
#[test]
fn a_cancelled_listing_publishes_the_prefix_it_read() {
    let engine = Engine::new();
    let readable = class_file(b"p/A", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let snapshot = open(zip_of(&[(b"p/A.class", &readable)]));
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), cancellation);
    let listing = engine
        .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("a cancelled request is reported, not raised");
    assert_eq!(
        (listing.items.len(), listing.candidates),
        (0, 0),
        "a cancelled scope scan never reached the directory, so it can name no candidate at all: the \n         execution and the diagnostic are what say the scope was not read, not a silent empty list"
    );
    assert_eq!(diagnostic_codes(&listing.diagnostics), vec!["cancelled"]);
    assert!(matches!(
        listing.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert_eq!(
        listing.coverage.artifact_structural.state,
        CoverageState::Partial
    );

    // The same request without a cancellation confirms the candidate, so the empty prefix above is
    // the cancellation and not an unreadable fixture.
    let mut budget = Budget::new(limits());
    let listing = engine
        .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the same scope confirms when it is not cancelled");
    assert_eq!(listing.items.len(), 1);
}

// ---------------------------------------------------------------------------------------------
// 2.1 / 2.2 / A13 / A16 / A17: member listing, its handoff and its isolation
// ---------------------------------------------------------------------------------------------

/// A listed method's own identity drives a real method request to the same physical definition.
#[test]
fn a_listed_method_is_directly_usable_as_a_request_target() {
    let sample = include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");
    let engine = Engine::new();
    let snapshot = open(sample.to_vec());
    let mut budget = Budget::new(limits());
    let confirmed = engine
        .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the standalone class confirms");
    assert_eq!(confirmed.items.len(), 1);
    let definition = confirmed.items[0].definition.clone();
    assert_eq!(
        confirmed.items[0].binding,
        ClassNameBinding::StandaloneRoot,
        "a standalone root has no path to compare with"
    );
    assert_eq!(confirmed.items[0].member_table, None);

    let mut budget = Budget::new(limits());
    let listing = engine
        .list_members(&snapshot, &definition, &mut budget)
        .expect("the definition the listing handed back is directly readable");
    assert_eq!(listing.definition, definition);
    assert_eq!(
        listing
            .declaration()
            .expect("the class-level item is published first")
            .declaration
            .this_class
            .raw()
            .0,
        b"HistoricalControlFlow"
    );
    assert_eq!(listing.stopped_at, None);
    assert!(
        budget.usage().result_items >= u64::try_from(listing.items.len()).unwrap(),
        "the listing's own items were charged, on top of the read's per-record and per-shell bills: {:?}",
        budget.usage()
    );
    assert_eq!(
        budget.usage().class_headers,
        1,
        "the member listing read one class header by identity: {:?}",
        budget.usage()
    );
    assert_no_analysis("member listing", &budget.usage());

    let method = listing
        .methods()
        .find(|method| method.identity.name.0 == b"finallyPath")
        .expect("the fixture declares `finallyPath`");
    assert_eq!(method.identity.descriptor.0, b"(I)I");
    assert_eq!(method.name.raw().0, method.identity.name.0);
    assert_eq!(method.descriptor.raw().0, method.identity.descriptor.0);
    assert!(matches!(
        method.body,
        MemberBodyEvidence::CodeAttribute { .. }
    ));

    // The handoff: the listed identity drives a real method request, with nothing about the member
    // retyped and no entry, ordinal or descriptor reassembled by the caller.
    let request = MethodAnalysisRequest {
        environment: environment(
            &snapshot,
            LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone(),
            },
        ),
        method: method.identity.clone(),
        stages: AnalysisStage::ALL.to_vec(),
    };
    let mut budget = Budget::new(limits());
    let report = engine
        .analyze_method(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("a legal method request is answered");
    assert_eq!(
        report.reads.len(),
        1,
        "the request read one class definition: {:?}",
        report.reads
    );
    assert_eq!(
        report.reads[0].definition, method.identity.owner,
        "the definition the request read is the one the listing handed back"
    );
    assert_eq!(
        report.reads[0].reason,
        ReadReason::DriverMethodBody,
        "and it read that class for the member the listing named"
    );
    assert_eq!(report.body, MethodBodyState::Present);
    assert_eq!(
        budget.usage().method_bodies,
        1,
        "exactly the requested member's body was attempted, which is what the listing never did"
    );
}

/// The same handoff inside an archive: the identity's own container origin is the read position.
#[test]
fn a_listed_member_in_an_archive_reads_the_same_physical_definition() {
    let engine = Engine::new();
    let class = class_file(
        b"p/S",
        52,
        &[MemberSpec::field(b"value", b"I")],
        &[MemberSpec::method(b"run", b"()V")],
    );
    let snapshot = open(zip_of(&[(b"p/S.class", &class)]));
    let mut budget = Budget::new(limits());
    let confirmed = engine
        .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the class confirms");
    let definition = confirmed.items[0].definition.clone();
    assert_eq!(confirmed.items[0].binding, ClassNameBinding::PathNameAgrees);

    let mut budget = Budget::new(limits());
    let listing = engine
        .list_members(&snapshot, &definition, &mut budget)
        .expect("the entry identity reads");
    let field = listing
        .fields()
        .next()
        .expect("the fixture declares a field");
    assert_eq!(field.index, 0);
    assert_eq!(field.name.raw().0, b"value");
    assert_eq!(
        field.identity,
        PhysicalMemberId {
            owner: definition.clone(),
            member: MemberKey::Field {
                name: bytes(b"value"),
                descriptor: bytes(b"I"),
            },
        }
    );
    let method = listing
        .methods()
        .next()
        .expect("the fixture declares a method");

    let request = MethodAnalysisRequest {
        environment: environment(
            &snapshot,
            LoadRoot::Container {
                origin: definition.entry().unwrap().origin.clone(),
                prefix: ArchiveNameBytes(Vec::new()),
            },
        ),
        method: method.identity.clone(),
        stages: AnalysisStage::ALL.to_vec(),
    };
    let mut budget = Budget::new(limits());
    let report = engine
        .analyze_method(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("the member's own identity is a legal request");
    assert_eq!(report.reads.len(), 1);
    assert_eq!(report.reads[0].definition, definition);
    assert_eq!(
        report.body,
        MethodBodyState::Present,
        "the run really read the body of the definition the listing named"
    );
    assert_eq!(budget.usage().method_bodies, 1);
}

/// Declarations without a body are published as declarations, never as an empty body.
#[test]
fn an_abstract_or_native_member_carries_no_fabricated_body() {
    let engine = Engine::new();
    let class = class_file(
        b"p/Shape",
        52,
        &[MemberSpec::field(b"size", b"I")],
        &[
            MemberSpec::method(b"concrete", b"()V"),
            MemberSpec::abstract_method(b"abstractly", b"(I)I"),
            MemberSpec::native_method(b"natively", b"()V"),
        ],
    );
    let snapshot = open(zip_of(&[(b"p/Shape.class", &class)]));
    let mut budget = Budget::new(limits());
    let confirmed = engine
        .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the class confirms");
    let definition = confirmed.items[0].definition.clone();
    let mut budget = Budget::new(limits());
    let listing = engine
        .list_members(&snapshot, &definition, &mut budget)
        .expect("the members list");
    assert_eq!(
        method_names(&listing),
        vec!["concrete", "abstractly", "natively"],
        "the method set is the member table's own order"
    );
    assert_eq!(listing.fields().count(), 1);
    assert_eq!(
        listing.declaration().unwrap().declaration.access_flags,
        0x0021
    );
    let bodies: Vec<(String, MemberBodyEvidence)> = listing
        .methods()
        .map(|method| (text(&method.identity.name.0), method.body.clone()))
        .collect();
    for (name, body) in &bodies {
        match (name.as_str(), body) {
            ("concrete", MemberBodyEvidence::CodeAttribute { .. }) => {}
            ("abstractly" | "natively", MemberBodyEvidence::NoCodeAttribute) => {}
            other => panic!("{name} states the wrong body evidence: {other:?}"),
        }
    }
    assert_eq!(
        listing
            .methods()
            .find(|method| method.identity.name.0 == b"abstractly")
            .unwrap()
            .access_flags,
        0x0401
    );
    assert_eq!(
        listing
            .methods()
            .find(|method| method.identity.name.0 == b"natively")
            .unwrap()
            .access_flags,
        0x0101
    );
    assert_no_analysis("member listing of a body-less member", &budget.usage());
    assert_eq!(
        (
            listing.coverage.runtime_resolution.state,
            listing.coverage.dynamic_analysis.state
        ),
        (CoverageState::NotRequested, CoverageState::NotRequested),
        "the analysis planes were not even requested"
    );
}

/// A damaged member record does not erase the class: the prefix stays, the stop is located.
#[test]
fn a_damaged_member_does_not_erase_the_class() {
    let engine = Engine::new();
    let damaged = class_file(
        b"p/Damaged",
        52,
        &[],
        &[
            MemberSpec::method(b"first", b"()V"),
            MemberSpec::method(b"second", b"()V").damaged(64),
            MemberSpec::method(b"third", b"()V"),
        ],
    );
    let snapshot = open(zip_of(&[(b"p/Damaged.class", &damaged)]));

    // The confirmed listing reads the declaration of the class whose member table is damaged: the
    // damaged record stops the *member* walk, not the class.
    let mut budget = Budget::new(limits());
    let confirmed = engine
        .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the class is confirmed from its own declaration");
    assert_eq!(confirmed.items.len(), 1);
    let item = &confirmed.items[0];
    assert_eq!(item.declaration.this_class.raw().0, b"p/Damaged");
    let stop = item
        .member_table
        .clone()
        .expect("the damaged record is recorded on the item");
    assert_eq!(stop.phase, MemberTablePhase::Methods);
    assert_eq!(stop.index, 1);
    assert_eq!(stop.code, "classfile_invalid_attribute_span");
    assert!(
        confirmed.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == stop.code
                && diagnostic.provenance == Some(definition_provenance(&item.definition))
        }),
        "the stop is reported with the class's physical origin: {:?}",
        confirmed.diagnostics
    );
    assert_eq!(
        confirmed.unconfirmed.len(),
        0,
        "the class was confirmed: nothing about it is left unread"
    );

    // The member listing on the handoff identity publishes the reliable prefix.
    let definition = item.definition.clone();
    let mut budget = Budget::new(limits());
    let listing = engine
        .list_members(&snapshot, &definition, &mut budget)
        .expect("the damaged member does not erase the class");
    assert_eq!(
        listing
            .declaration()
            .expect("the class declaration was read")
            .declaration
            .this_class
            .raw()
            .0,
        b"p/Damaged"
    );
    assert_eq!(
        method_names(&listing),
        vec!["first"],
        "the members before the damaged record stay usable"
    );
    assert_eq!(listing.stopped_at.clone(), Some(stop.clone()));
    assert!(
        listing.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == stop.code
                && diagnostic.provenance == Some(definition_provenance(&definition))
        }),
        "the stop carries the class's physical origin: {:?}",
        listing.diagnostics
    );
    assert!(matches!(listing.execution, ExecutionReport::Partial { .. }));
    assert!(
        listing
            .coverage
            .artifact_structural
            .skipped
            .contains(&CoverageRange {
                label: "class_methods".into(),
                start: 1,
                end: 3
            }),
        "the records the walk never reached are marked: {:?}",
        listing.coverage.artifact_structural
    );
    assert_eq!(
        (budget.usage().method_bodies, budget.usage().code_bytes),
        (0, 0),
        "the damaged record was read as a declaration and never as a body"
    );
}

/// An identity that does not name these bytes is refused, never read as "something near it".
#[test]
fn an_identity_that_does_not_match_its_bytes_is_refused() {
    let engine = Engine::new();
    let class = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let snapshot = open(zip_of(&[(b"p/S.class", &class)]));
    let mut budget = Budget::new(limits());
    let confirmed = engine
        .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the class confirms");
    let definition = confirmed.items[0].definition.clone();

    let mut wrong_bytes = definition.clone();
    wrong_bytes.class_bytes.digest = Digest("not-the-bytes".into());
    let mut budget = Budget::new(limits());
    assert!(matches!(
        engine.list_members(&snapshot, &wrong_bytes, &mut budget),
        Err(Error::InvalidInput { code, .. }) if code == "definition_class_bytes_mismatch"
    ));

    let mut missing = definition.clone();
    let PhysicalClassLocation::ArchiveEntry { entry } = &mut missing.location else {
        panic!("the fixture's definition is an archive entry");
    };
    entry.ordinal = 7;
    let mut budget = Budget::new(limits());
    assert!(matches!(
        engine.list_members(&snapshot, &missing, &mut budget),
        Err(Error::InvalidInput { code, .. }) if code == "definition_entry_not_found"
    ));

    let mut other_bytes = zip_of(&[(b"p/S.class", &class)]);
    other_bytes.extend_from_slice(b"\n");
    let other = open(other_bytes);
    assert_ne!(other.id(), snapshot.id());
    let mut budget = Budget::new(limits());
    assert!(matches!(
        engine.list_members(&other, &definition, &mut budget),
        Err(Error::InvalidInput { code, .. }) if code == "definition_snapshot_mismatch"
    ));
}

/// Every navigation level reads class structure and nothing else: no body, no analysis.
#[test]
fn navigation_reads_headers_only() {
    let engine = Engine::new();
    let class = class_file(
        b"p/S",
        52,
        &[MemberSpec::field(b"value", b"I")],
        &[MemberSpec::method(b"run", b"()V")],
    );
    let snapshot = open(zip_of(&[(b"p/S.class", &class)]));

    let mut budget = Budget::new(limits());
    let confirmed = engine
        .list_class_declarations(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the class confirms");
    let confirmed_usage = budget.usage();
    assert!(
        confirmed_usage.class_headers == 1 && confirmed_usage.class_bytes > 0,
        "the confirmed listing really read the class: {confirmed_usage:?}"
    );
    assert_no_analysis("confirmed finding", &confirmed_usage);
    assert_eq!(
        confirmed.coverage.runtime_resolution.state,
        CoverageState::NotRequested
    );
    let definition = confirmed.items[0].definition.clone();

    let mut budget = Budget::new(limits());
    let members = engine
        .list_members(&snapshot, &definition, &mut budget)
        .expect("the members list");
    assert_eq!(
        members.items.len(),
        3,
        "the class item, its field and its method"
    );
    assert_no_analysis("member listing", &budget.usage());

    let mut budget = Budget::new(limits());
    let found = engine
        .find_targets(
            &snapshot,
            &PhysicalScope::SnapshotAll,
            &NavigationQuery {
                class: ClassNameQuery::dotted("p.S"),
                member: Some(MemberQuery {
                    name: bytes(b"run"),
                    descriptor: None,
                    kind: MemberQueryKind::Methods,
                }),
            },
            &mut budget,
        )
        .expect("the member search is answered");
    assert_eq!(found.methods().count(), 1);
    let usage = budget.usage();
    assert_eq!(
        usage.class_headers, 1,
        "one header read attempt for the one candidate: {usage:?}"
    );
    assert_no_analysis("navigation lookup", &usage);
    assert!(
        found.coverage.runtime_resolution.state == CoverageState::NotRequested
            && found.coverage.dynamic_analysis.state == CoverageState::NotRequested
    );
}

// ---------------------------------------------------------------------------------------------
// 3.1 / 3.2 / A07 / A13 / A14: friendly names, ambiguity and stability
// ---------------------------------------------------------------------------------------------

/// Two spellings of one name bind the same physical identity, not the spelling.
#[test]
fn two_spellings_of_one_name_bind_the_same_definition() {
    let engine = Engine::new();
    let class = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let snapshot = open(zip_of(&[(b"p/S.class", &class)]));

    let ask = |class_query: ClassNameQuery| {
        let mut budget = Budget::new(limits());
        let report = engine
            .find_targets(
                &snapshot,
                &PhysicalScope::SnapshotAll,
                &NavigationQuery {
                    class: class_query,
                    member: None,
                },
                &mut budget,
            )
            .expect("a name search is answered");
        (report, budget.usage())
    };

    let (dotted, dotted_usage) = ask(ClassNameQuery::dotted("p.S"));
    let (internal, _) = ask(ClassNameQuery::internal("p/S"));
    assert_eq!(dotted.classes().count(), 1);
    assert_eq!(internal.classes().count(), 1);
    let dotted_class = dotted.classes().next().unwrap();
    let internal_class = internal.classes().next().unwrap();
    assert_eq!(
        dotted_class.definition, internal_class.definition,
        "the two spellings denote one physical definition"
    );
    assert_eq!(
        dotted_class.definition.entry().unwrap().ordinal,
        internal_class.definition.entry().unwrap().ordinal
    );
    assert!(
        dotted_class.definition.class_bytes.length > 0
            && dotted_class.definition.class_bytes.digest
                == internal_class.definition.class_bytes.digest,
        "the identity carries the class bytes' own digest and length, not the spelling"
    );
    assert_eq!(dotted.query.class.spelling(), "p.S");
    assert_eq!(internal.query.class.spelling(), "p/S");
    assert_eq!(dotted_usage.class_headers, 1);
    assert!(matches!(dotted.execution, ExecutionReport::Complete { .. }));
    assert!(dotted.diagnostics.is_empty());
}

/// Overloads come back as every candidate, and a descriptor makes the answer unique.
#[test]
fn overloads_are_answered_with_every_candidate_until_a_descriptor_is_given() {
    let engine = Engine::new();
    let class = class_file(
        b"p/S",
        52,
        &[MemberSpec::field(b"run", b"I")],
        &[
            MemberSpec::method(b"run", b"()V"),
            MemberSpec::method(b"run", b"(I)I"),
        ],
    );
    let snapshot = open(zip_of(&[(b"p/S.class", &class)]));
    let ask = |member: Option<MemberQuery>| {
        let mut budget = Budget::new(limits());
        engine
            .find_targets(
                &snapshot,
                &PhysicalScope::SnapshotAll,
                &NavigationQuery {
                    class: ClassNameQuery::dotted("p.S"),
                    member,
                },
                &mut budget,
            )
            .expect("a member search is answered")
    };

    let by_name = ask(Some(MemberQuery {
        name: bytes(b"run"),
        descriptor: None,
        kind: MemberQueryKind::Methods,
    }));
    let descriptors: Vec<String> = by_name
        .methods()
        .map(|method| text(&method.identity.descriptor.0))
        .collect();
    assert_eq!(
        descriptors,
        vec!["()V", "(I)I"],
        "both overloads come back with their own descriptors"
    );
    assert_eq!(by_name.methods().count(), 2);
    assert_eq!(
        by_name.fields().count(),
        0,
        "the kind filter keeps the field named `run` out of a method request"
    );

    let by_descriptor = ask(Some(MemberQuery {
        name: bytes(b"run"),
        descriptor: Some(bytes(b"(I)I")),
        kind: MemberQueryKind::Methods,
    }));
    assert_eq!(by_descriptor.methods().count(), 1);
    assert_eq!(
        by_descriptor
            .methods()
            .next()
            .unwrap()
            .identity
            .descriptor
            .0,
        b"(I)I"
    );

    let both = ask(Some(MemberQuery {
        name: bytes(b"run"),
        descriptor: None,
        kind: MemberQueryKind::Both,
    }));
    assert_eq!((both.methods().count(), both.fields().count()), (2, 1));
    let owner = both
        .methods()
        .next()
        .expect("the methods are published")
        .identity
        .owner
        .clone();
    assert!(both.fields().all(|field| field.identity.owner == owner));
}

/// No match is an empty candidate list over the scanned range — and a `Complete` answer.
#[test]
fn no_match_is_an_empty_answer_and_not_a_failure() {
    let engine = Engine::new();
    let class = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let snapshot = open(zip_of(&[(b"p/S.class", &class)]));
    let mut budget = Budget::new(limits());
    let report = engine
        .find_targets(
            &snapshot,
            &PhysicalScope::SnapshotAll,
            &NavigationQuery {
                class: ClassNameQuery::dotted("p.Missing"),
                member: None,
            },
            &mut budget,
        )
        .expect("a name the scope does not hold is still an answer");
    assert!(report.candidates.is_empty());
    assert!(report.diagnostics.is_empty());
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema,
        "the whole scope was searched, so `no match` is a complete result: {:?}",
        report.coverage.artifact_structural
    );
    assert!(
        report
            .coverage
            .artifact_structural
            .scanned
            .contains(&CoverageRange {
                label: "navigation_candidates".into(),
                start: 0,
                end: 0
            }),
        "the scanned range is stated even when it holds no candidate: {:?}",
        report.coverage.artifact_structural
    );
    assert_eq!(budget.usage().class_headers, 0);

    // A name that matches an entry's raw name but not its declaration binds nothing either.
    let wrong = class_file(b"p/Real", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let snapshot = open(zip_of(&[(b"p/Spelled.class", &wrong)]));
    let mut budget = Budget::new(limits());
    let report = engine
        .find_targets(
            &snapshot,
            &PhysicalScope::SnapshotAll,
            &NavigationQuery {
                class: ClassNameQuery::internal("p/Spelled"),
                member: None,
            },
            &mut budget,
        )
        .expect("the entry's own name is searched");
    assert!(
        report.candidates.is_empty(),
        "the entry's path is not a declaration: {:?}",
        report.candidates
    );
    assert_eq!(
        diagnostic_codes(&report.diagnostics),
        vec!["navigation_path_name_mismatch"]
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
}

/// Two definitions of one name: every candidate, with the identity that decides between them.
#[test]
fn duplicate_definitions_are_all_returned_and_a_chosen_identity_is_unique() {
    let engine = Engine::new();
    let upper = class_file(SHARED, 52, &[], &[MemberSpec::method(b"upper", b"()V")]);
    let nested = class_file(SHARED, 52, &[], &[MemberSpec::method(b"nested", b"()V")]);
    let snapshot = open(two_origin_war(&upper, &nested));

    let mut budget = Budget::new(limits());
    let report = engine
        .find_targets(
            &snapshot,
            &tree_scope(&snapshot),
            &NavigationQuery {
                class: ClassNameQuery::dotted("p.S"),
                member: None,
            },
            &mut budget,
        )
        .expect("both definitions are answered");
    let classes: Vec<&ClassDeclarationItem> = report.classes().collect();
    assert_eq!(
        classes.len(),
        2,
        "the same name at two origins is two candidates, never a silent first: {:?}",
        report.diagnostics
    );
    let application = classes
        .iter()
        .find(|class| class.definition.entry().unwrap().origin.steps.is_empty())
        .expect("the application directory's record");
    let library = classes
        .iter()
        .find(|class| !class.definition.entry().unwrap().origin.steps.is_empty())
        .expect("the nested library's record");
    assert_ne!(
        application.definition, library.definition,
        "the basis for choosing is the physical definition, not the scan order"
    );
    assert!(report.diagnostics.is_empty());
    assert_eq!(budget.usage().class_headers, 2);

    // Feeding the chosen definition back reads exactly that definition's class.
    let mut budget = Budget::new(limits());
    let listing = engine
        .list_members(&snapshot, &library.definition, &mut budget)
        .expect("the chosen identity reads");
    assert_eq!(listing.definition, library.definition);
    assert_eq!(method_names(&listing), vec!["nested"]);
    let mut budget = Budget::new(limits());
    let listing = engine
        .list_members(&snapshot, &application.definition, &mut budget)
        .expect("the other identity reads its own definition");
    assert_eq!(method_names(&listing), vec!["upper"]);
}

/// Changing what the artifact stores first changes neither the identity nor what it reads.
#[test]
fn a_chosen_identity_does_not_depend_on_traversal_order() {
    let engine = Engine::new();
    let upper = class_file(SHARED, 52, &[], &[MemberSpec::method(b"upper", b"()V")]);
    let nested = class_file(SHARED, 52, &[], &[MemberSpec::method(b"nested", b"()V")]);
    let library = zip_of(&[(b"p/S.class", &nested)]);

    // The same two positions in the opposite record order: a first-wins scan would elect the other
    // definition in the second archive.
    let first = open(two_origin_war(&upper, &nested));
    let second = open(zip_of(&[
        (b"WEB-INF/lib/L.jar", &library),
        (b"WEB-INF/classes/p/S.class", &upper),
        (b"WEB-INF/classes/p/Only.class", &upper),
    ]));

    let choose_library = |snapshot: &ArtifactSnapshot| {
        let mut budget = Budget::new(limits());
        let report = engine
            .find_targets(
                snapshot,
                &tree_scope(snapshot),
                &NavigationQuery {
                    class: ClassNameQuery::internal("p/S"),
                    member: None,
                },
                &mut budget,
            )
            .expect("the tree is searched");
        report
            .classes()
            .find(|class| !class.definition.entry().unwrap().origin.steps.is_empty())
            .expect("the nested library holds one of them")
            .definition
            .clone()
    };
    let first_choice = choose_library(&first);
    let second_choice = choose_library(&second);
    // The identity is compared in its container's own coordinates: two archives have two snapshot
    // ids and two record orders, and what must not move is which physical entry the identity names.
    let identity = |definition: &PhysicalDefinitionId| {
        let entry = definition.entry().expect("an archive entry");
        let step = entry.origin.steps.last().expect("a nested container");
        (
            step.via_raw_name.clone(),
            entry.ordinal,
            entry.raw_name.clone(),
            definition.class_bytes.clone(),
            definition.variant.clone(),
        )
    };
    assert_eq!(
        identity(&first_choice),
        identity(&second_choice),
        "the same physical identity is reachable whatever order the container stores"
    );

    for (snapshot, definition) in [(&first, &first_choice), (&second, &second_choice)] {
        let mut budget = Budget::new(limits());
        let listing = engine
            .list_members(snapshot, definition, &mut budget)
            .expect("the identity reads its own artifact");
        assert_eq!(
            method_names(&listing),
            vec!["nested"],
            "the identity, not the scan order, decides which class is read"
        );
    }

    // The identities are bound to their own snapshot: the second archive's identity is refused by
    // the first one, so an identity is not "an entry that looks the same in another artifact".
    let mut budget = Budget::new(limits());
    assert!(matches!(
        engine.list_members(&first, &second_choice, &mut budget),
        Err(Error::InvalidInput { code, .. }) if code == "definition_snapshot_mismatch"
    ));
}

// ---------------------------------------------------------------------------------------------
// Scope shapes and the listing's own refusals
// ---------------------------------------------------------------------------------------------

/// A standalone class lists its own root, and an artifact-tree scope is an input error for it.
#[test]
fn a_standalone_class_lists_its_own_root_and_refuses_a_tree_scope() {
    let sample = include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");
    let engine = Engine::new();
    let snapshot = open(sample.to_vec());
    let mut budget = Budget::new(limits());
    let candidates = engine
        .list_class_candidates(&snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the standalone scope lists");
    assert_eq!(candidates.items.len(), 1);
    assert_eq!(candidates.resources().count(), 0);
    assert_eq!(
        candidates.candidates().next().unwrap(),
        &PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone()
        }
    );
    assert_eq!(budget.usage().class_headers, 0);

    let mut budget = Budget::new(limits());
    assert!(matches!(
        engine.list_class_declarations(
            &snapshot,
            &PhysicalScope::ArtifactTree {
                root_container: ContainerId("root".into())
            },
            &mut budget,
        ),
        Err(Error::InvalidInput { code, .. }) if code == "navigation_not_zip"
    ));
}

/// A tree root that is not this snapshot's root is refused before anything is read.
#[test]
fn a_tree_root_that_is_not_this_snapshot_is_refused() {
    let engine = Engine::new();
    let class = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let snapshot = open(zip_of(&[(b"p/S.class", &class)]));
    let mut budget = Budget::new(limits());
    assert!(matches!(
        engine.list_class_candidates(
            &snapshot,
            &PhysicalScope::ArtifactTree {
                root_container: ContainerId("not-the-root".into())
            },
            &mut budget,
        ),
        Err(Error::InvalidInput { code, .. }) if code == "navigation_root_container_mismatch"
    ));
}

// ---------------------------------------------------------------------------------------------
// The reports are output documents
// ---------------------------------------------------------------------------------------------

/// One report, through its own serde shape and back: the shape an adapter serializes is the shape
/// this engine publishes, and no item kind is lost in the tag it is written under.
fn round_trip<T>(label: &str, report: &T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(report).expect("a report serializes");
    let back: T = serde_json::from_str(&json)
        .unwrap_or_else(|error| panic!("{label} does not read back as its own document: {error}"));
    assert_eq!(&back, report, "{label} changed in its own document shape");
}

/// Every navigation report round-trips, and every item variant keeps its tag.
#[test]
fn navigation_reports_round_trip_through_their_json_shape() {
    let engine = Engine::new();
    let class = class_file(
        b"p/S",
        52,
        &[MemberSpec::field(b"value", b"I")],
        &[
            MemberSpec::method(b"run", b"()V"),
            MemberSpec::abstract_method(b"compute", b"(I)I"),
        ],
    );
    let snapshot = open(zip_of(&[
        (b"p/S.class", &class),
        (b"META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\n"),
    ]));
    let scope = PhysicalScope::SnapshotAll;

    let mut budget = Budget::new(limits());
    let candidates = engine
        .list_class_candidates(&snapshot, &scope, &mut budget)
        .expect("the scope lists");
    round_trip("ClassCandidateListing", &candidates);
    assert!(
        serde_json::to_string(&candidates)
            .expect("serializes")
            .contains("\"kind\":\"class_candidate\""),
        "a candidate item states its kind"
    );
    round_trip(
        "ClassListingItem::Resource",
        &ClassListingItem::Resource {
            entry: candidates
                .items
                .iter()
                .find_map(|item| match item {
                    ClassListingItem::Resource { entry } => Some(entry.clone()),
                    ClassListingItem::ClassCandidate { .. } => None,
                })
                .expect("the fixture has a manifest entry"),
        },
    );

    let mut budget = Budget::new(limits());
    let confirmed = engine
        .list_class_declarations(&snapshot, &scope, &mut budget)
        .expect("the candidate confirms");
    round_trip("ClassDeclarationListing", &confirmed);
    let definition = confirmed.items[0].definition.clone();

    let mut budget = Budget::new(limits());
    let members = engine
        .list_members(&snapshot, &definition, &mut budget)
        .expect("the members list");
    round_trip("MemberListing", &members);
    for item in &members.items {
        round_trip("ClassContentItem", item);
    }
    assert!(
        serde_json::to_string(&members)
            .expect("serializes")
            .contains("\"kind\":\"no_code_attribute\""),
        "the body-less member states its own evidence"
    );

    let mut budget = Budget::new(limits());
    let found = engine
        .find_targets(
            &snapshot,
            &scope,
            &NavigationQuery {
                class: ClassNameQuery::dotted("p.S"),
                member: Some(MemberQuery {
                    name: bytes(b"run"),
                    descriptor: None,
                    kind: MemberQueryKind::Methods,
                }),
            },
            &mut budget,
        )
        .expect("the lookup answers");
    round_trip("NavigationReport", &found);
    round_trip("NavigationQuery", &found.query);
    round_trip("ClassNameQuery", &found.query.class);
    round_trip(
        "MemberQuery",
        found
            .query
            .member
            .as_ref()
            .expect("the query carried the member filter"),
    );
    assert!(
        serde_json::to_string(&found)
            .expect("serializes")
            .contains("\"spelling\":\"p.S\""),
        "the request is echoed with the spelling the caller used: {}",
        serde_json::to_string(&found).expect("serializes")
    );
}
