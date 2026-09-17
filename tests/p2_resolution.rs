//! P2 2.1 acceptance: class-name lookup, effective search order and content identity.
//!
//! The slice under test turns a class symbol and an explicit `ResolutionEnvironment` into one
//! selected physical definition. What this file has to prove through the public API is:
//!
//! 1. `ParentFirst` and `ChildFirst` really search the declared chain in the declared order,
//!    and inside one loader the first declared root that holds the name wins,
//! 2. two candidates at one position are `Ambiguous` and keep their own origins — equal bytes
//!    do not merge two origins, and a byte-equal duplicate is not ordered by ordinal either,
//! 3. a rejected environment (missing or cyclic parent, unsupported module mode or delegation,
//!    unreadable or unprovided root, a caller identity that names another loader) never
//!    produces a definition of a class symbol,
//! 4. a standalone CLASS root declares its own name and is matched through `this_class`, while
//!    an artifact-tree root searches its own container only,
//! 5. a damaged candidate fails at its own origin and the search does not continue to a later
//!    root — both when the bytes cannot be decoded and when the read layer itself rejects them —
//!    and a stopped listing or a budget stop is never read as "the name is missing",
//! 6. the result planes follow invariant 3: `state = Some(..)` exactly when the run decided
//!    something (the three lookup decisions plus the budget stop), and a damaged candidate or a
//!    cancellation reports `state = None` with `execution` carrying the stop, so
//!    `Inaccessible`/`IncompatibleClassChange` stay reserved for the access and link rules,
//! 7. every lookup charges one `ClassHeaders` per header read **attempt** — before the read, so
//!    an attempt that fails still counts, and a refused attempt does not — and nothing else.
//!
//! Fixtures are built from three helpers: a minimal class file with a chosen `this_class` and
//! version, a stored ZIP (nested where a root container is tested), and a stored ZIP with one
//! entry's data damaged. Each assertion names the
//! selected position through the definition the report publishes — the snapshot the root was
//! declared with, the entry identity inside it and the digest of the class bytes — which
//! together identify the loader, the root and the definition. The crate-private `root_index` and
//! the effective sequence are pinned in the provider unit tests, which can read them directly.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const STORE: u16 = 0;

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        output_bytes: 1 << 20,
        result_items: 10_000,
        class_headers: 100,
        nested_depth: 4,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

/// Smallest class file the reader accepts, with `this_class` set to `this_class`.
///
/// `major` is the only knob: two class files for the same name with different majors are two
/// distinct byte sequences, which is how the "same name, different bytes" fixtures are built
/// without appending bytes the reader would reject as trailing input.
fn class_bytes(this_class: &[u8], major: u16) -> Vec<u8> {
    let object = b"java/lang/Object";
    let mut pool = Vec::new();
    pool.push(1_u8); // CONSTANT_Utf8 this_class
    pool.extend_from_slice(
        &u16::try_from(this_class.len())
            .expect("name fits u16")
            .to_be_bytes(),
    );
    pool.extend_from_slice(this_class);
    pool.extend_from_slice(&[7, 0, 1]); // CONSTANT_Class #1
    pool.push(1_u8); // CONSTANT_Utf8 "java/lang/Object"
    pool.extend_from_slice(
        &u16::try_from(object.len())
            .expect("name fits u16")
            .to_be_bytes(),
    );
    pool.extend_from_slice(object);
    pool.extend_from_slice(&[7, 0, 3]); // CONSTANT_Class #3

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // minor
    bytes.extend_from_slice(&major.to_be_bytes());
    bytes.extend_from_slice(&5_u16.to_be_bytes()); // constant_pool_count
    bytes.extend_from_slice(&pool);
    bytes.extend_from_slice(&0x0021_u16.to_be_bytes()); // ACC_PUBLIC | ACC_SUPER
    bytes.extend_from_slice(&2_u16.to_be_bytes()); // this_class
    bytes.extend_from_slice(&4_u16.to_be_bytes()); // super_class
    for _ in 0..4 {
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // interfaces, fields, methods, attributes
    }
    bytes
}

/// One stored ZIP with the given entries, duplicates included.
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
                .expect("the fixture entry is writable");
            let (_, descriptor) = writer.finish().expect("the fixture entry closes");
            entry
                .finish(descriptor)
                .expect("the fixture entry finishes");
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut Budget::new(limits()))
        .expect("the fixture snapshot opens")
}

/// A ZIP whose stored entry `entry_name` carries damaged data, plus the archive offset the
/// damage was applied at.
///
/// The damage is one flipped byte inside the entry's **data** area, located through the public
/// enumeration: the central directory stays readable, so the archive still lists the entry and
/// still resolves the candidate, and the failure appears only when the entry is read (the CRC
/// of a stored entry no longer matches its bytes). That is the *read-layer* damage this fixture
/// exists for — damage that is only visible after decoding is a different failure with a
/// different code and its own fixture.
fn zip_with_damaged_entry(entries: &[(&[u8], &[u8])], entry_name: &[u8]) -> (Vec<u8>, u64) {
    let bytes = zip_of(entries);
    let template = open(bytes.clone());
    let mut budget = Budget::new(limits());
    let report = template
        .enumerate(&mut budget)
        .expect("the template archive lists");
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.id.raw_name.0 == entry_name)
        .expect("the template holds the entry to damage");
    let offset = entry.layout.compressed_data.start;
    let index = usize::try_from(offset).expect("the fixture offset fits usize");
    assert!(
        usize::try_from(entry.layout.compressed_data.length).expect("fixture length fits usize")
            > 0,
        "the entry to damage has data to damage"
    );
    let mut damaged = bytes;
    damaged[index] ^= 0xff;
    (damaged, offset)
}

/// The physical definition the engine derives for one physical entry.
fn definition_of_entry(entry: &PhysicalEntry, content: &[u8]) -> PhysicalDefinitionId {
    PhysicalDefinitionId {
        location: PhysicalClassLocation::ArchiveEntry {
            entry: entry.id.clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(content).to_hex().to_string()),
            length: u64::try_from(content.len()).expect("fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
    }
}

/// The physical definition the engine derives for one archive entry.
///
/// Built from the public enumeration, so the test does not restate an identity the engine owns:
/// the entry the enumeration reports, its content digest and its length.
fn archive_definition(
    snapshot: &ArtifactSnapshot,
    entry_name: &[u8],
    content: &[u8],
) -> PhysicalDefinitionId {
    let mut budget = Budget::new(limits());
    let report = snapshot.enumerate(&mut budget).expect("the fixture lists");
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.id.raw_name.0 == entry_name)
        .expect("the fixture holds the named entry");
    definition_of_entry(entry, content)
}

/// The physical definition of a standalone CLASS root.
fn standalone_definition(snapshot: &ArtifactSnapshot, content: &[u8]) -> PhysicalDefinitionId {
    PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(content).to_hex().to_string()),
            length: u64::try_from(content.len()).expect("fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
    }
}

fn loader(name: &str) -> LoaderId {
    LoaderId(name.to_string())
}

fn domain(
    loader: &LoaderId,
    parent: Option<LoaderId>,
    delegation: DelegationPolicy,
    roots: Vec<LoadRoot>,
) -> LoadDomain {
    LoadDomain {
        loader: loader.clone(),
        parent_loader: parent,
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

fn environment(
    runtime_snapshot: &ArtifactSnapshot,
    caller: LoadDomain,
    domains: Vec<LoadDomain>,
    providers: Vec<HeaderProvider>,
) -> ResolutionEnvironment {
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: runtime_snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: caller,
        },
        domains,
        providers,
    }
}

fn class_request(
    environment: ResolutionEnvironment,
    caller: &LoaderId,
    class_name: &[u8],
) -> ResolutionRequest {
    ResolutionRequest {
        environment,
        target: SymbolRef::Class {
            owner: JvmBytes(class_name.to_vec()),
        },
        use_kind: ReferenceUse::ClassReference,
        caller: CallerContext {
            loader: caller.clone(),
            enclosing: None,
        },
        dispatch: None,
    }
}

fn resolve(
    content: &[ArtifactSnapshot],
    request: &ResolutionRequest,
    budget: &mut Budget,
) -> ResolutionReport {
    Engine::new()
        .resolve_symbol(content, request, budget)
        .expect("a legal request is answered, not raised")
}

fn usage_of(execution: &ExecutionReport) -> &UsageSnapshot {
    match execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage,
    }
}

/// Header read attempts of one report.
fn class_headers(report: &ResolutionReport) -> u64 {
    usage_of(&report.execution).counted_usage(CountedBudgetDimension::ClassHeaders)
}

fn diagnostic_codes(diagnostics: &[Diagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.clone())
        .collect()
}

fn problem_codes(problems: &[EnvironmentProblem]) -> Vec<&'static str> {
    problems
        .iter()
        .map(|problem| problem.code.as_str())
        .collect()
}

fn resolved_of(report: &ResolutionReport) -> &ResolvedMemberRef {
    report
        .resolved
        .as_ref()
        .expect("a resolved lookup publishes its definition")
}

/// A report that performed nothing may not carry a unique resolution or a covered range.
fn assert_not_performed(report: &ResolutionReport) {
    assert_eq!(report.analysis, ResolutionAnalysis::NotPerformed);
    assert!(
        report.state.is_none(),
        "state must be None: {:?}",
        report.state
    );
    assert!(report.resolved.is_none());
    assert!(report.candidates.is_empty());
    assert!(report.dispatch.is_none());
    assert_eq!(report.coverage, Coverage::not_requested());
    assert_eq!(class_headers(report), 0);
}

fn counted_usage_is_zero(usage: &UsageSnapshot) -> bool {
    CountedBudgetDimension::ALL
        .iter()
        .all(|dimension| usage.counted_usage(*dimension) == 0)
}

fn unsupported_code(execution: &ExecutionReport) -> Option<&str> {
    match execution {
        ExecutionReport::Failed {
            reason: TerminationReason::Unsupported { code },
            ..
        } => Some(code.as_str()),
        _ => None,
    }
}

fn search_range(start: u64, end: u64) -> CoverageRange {
    CoverageRange {
        label: "provider_search_position".to_string(),
        start,
        end,
    }
}

/// `platform` (no parent) and `app` (parent `platform`) both hold `p/S.class`, with different
/// bytes, so the two declarations delegate to different definitions.
struct OrderedRoots {
    content: Vec<ArtifactSnapshot>,
    platform_definition: PhysicalDefinitionId,
    app_definition: PhysicalDefinitionId,
}

fn ordered_roots(delegation: DelegationPolicy) -> (OrderedRoots, ResolutionEnvironment) {
    let platform_bytes = class_bytes(b"p/S", 52);
    let app_bytes = class_bytes(b"p/S", 51);
    let platform = open(zip_of(&[(b"p/S.class", &platform_bytes)]));
    let app = open(zip_of(&[(b"p/S.class", &app_bytes)]));
    let platform_definition = archive_definition(&platform, b"p/S.class", &platform_bytes);
    let app_definition = archive_definition(&app, b"p/S.class", &app_bytes);
    let platform_domain = domain(
        &loader("platform"),
        None,
        delegation.clone(),
        vec![snapshot_root(&platform)],
    );
    let app_domain = domain(
        &loader("app"),
        Some(loader("platform")),
        delegation,
        vec![snapshot_root(&app)],
    );
    let environment = environment(
        &platform,
        app_domain.clone(),
        vec![app_domain, platform_domain],
        Vec::new(),
    );
    (
        OrderedRoots {
            content: vec![platform, app],
            platform_definition,
            app_definition,
        },
        environment,
    )
}

#[test]
fn parent_first_selects_the_ancestor_and_child_first_the_caller() {
    // ParentFirst searches the top ancestor first, so the platform's definition wins even
    // though the caller's own root holds the same name.
    let (fixture, environment) = ordered_roots(DelegationPolicy::ParentFirst);
    let request = class_request(environment, &loader("app"), b"p/S");
    let mut budget = Budget::new(limits());
    let report = resolve(&fixture.content, &request, &mut budget);

    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(report.target, request.target);
    assert_eq!(report.use_kind, ReferenceUse::ClassReference);
    assert_eq!(report.environment_problems, Vec::new());
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(resolved_of(&report).loader, loader("platform"));
    assert_eq!(resolved_of(&report).definition, fixture.platform_definition);
    assert_eq!(
        resolved_of(&report).member,
        request.target,
        "a class resolution republishes the raw symbol it was asked for"
    );
    assert!(report.candidates.is_empty());
    assert_eq!(class_headers(&report), 1);
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema
    );
    assert_eq!(
        report.coverage.runtime_resolution.scanned,
        vec![search_range(0, 1)],
        "the deciding position is the prefix the decision covers"
    );
    assert!(
        report.coverage.runtime_resolution.skipped.is_empty(),
        "the order stops at the first match by rule, which is not skipped coverage"
    );

    // ChildFirst searches the caller's own roots first, so the same environment selects the
    // other definition.
    let (fixture, environment) = ordered_roots(DelegationPolicy::ChildFirst);
    let request = class_request(environment, &loader("app"), b"p/S");
    let report = resolve(&fixture.content, &request, &mut Budget::new(limits()));

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(resolved_of(&report).loader, loader("app"));
    assert_eq!(resolved_of(&report).definition, fixture.app_definition);
    assert_eq!(class_headers(&report), 1);
    assert_ne!(
        fixture.platform_definition, fixture.app_definition,
        "one name, two origins, two identities"
    );
}

#[test]
fn the_first_declared_root_that_holds_the_name_wins() {
    let first_bytes = class_bytes(b"p/S", 52);
    let second_bytes = class_bytes(b"p/S", 51);
    let unrelated = open(zip_of(&[(b"other/C.class", &class_bytes(b"other/C", 52))]));
    let first = open(zip_of(&[(b"p/S.class", &first_bytes)]));
    let second = open(zip_of(&[(b"p/S.class", &second_bytes)]));
    let first_definition = archive_definition(&first, b"p/S.class", &first_bytes);
    let roots = vec![
        snapshot_root(&unrelated),
        snapshot_root(&first),
        snapshot_root(&second),
    ];
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        roots.clone(),
    );
    // A provider names content of a participating domain: it adds no position of its own.
    let provider = HeaderProvider {
        id: ProviderId("app-headers".to_string()),
        roots: vec![roots[1].clone()],
    };
    let environment = environment(&first, caller.clone(), vec![caller], vec![provider.clone()]);
    let request = class_request(environment, &loader("app"), b"p/S");
    let mut budget = Budget::new(limits());
    let report = resolve(&[unrelated, first, second], &request, &mut budget);

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(resolved_of(&report).loader, loader("app"));
    assert_eq!(
        resolved_of(&report).definition,
        first_definition,
        "the first declared root that holds the name decides the lookup"
    );
    assert_eq!(
        class_headers(&report),
        1,
        "a root without a candidate is listed but never read"
    );
    assert_eq!(report.environment_identity.providers, vec![provider.id]);
    assert_eq!(
        report.coverage.runtime_resolution.scanned,
        vec![search_range(0, 2)]
    );
}

#[test]
fn several_candidates_at_one_position_are_ambiguous() {
    let high = class_bytes(b"p/A", 52);
    let low = class_bytes(b"p/A", 51);
    let snapshot = open(zip_of(&[(b"p/A.class", &high), (b"p/A.class", &low)]));
    let roots = vec![snapshot_root(&snapshot)];
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        roots.clone(),
    );
    let environment = environment(&snapshot, caller.clone(), vec![caller], Vec::new());
    let request = class_request(environment, &loader("app"), b"p/A");
    let mut budget = Budget::new(limits());
    let report = resolve(&[snapshot], &request, &mut budget);

    assert_eq!(report.state, Some(ResolutionState::Ambiguous));
    assert!(
        report.resolved.is_none(),
        "no candidate may be presented as the definition"
    );
    assert_eq!(
        report.candidates.len(),
        2,
        "each origin is listed on its own"
    );
    assert_eq!(report.candidates[0].loader, loader("app"));
    assert_eq!(
        report.candidates[0].definition.class_bytes.digest,
        Digest(blake3::hash(&high).to_hex().to_string())
    );
    assert_eq!(
        report.candidates[1].definition.class_bytes.digest,
        Digest(blake3::hash(&low).to_hex().to_string())
    );
    assert_ne!(
        report.candidates[0].definition,
        report.candidates[1].definition
    );
    assert_eq!(
        report.candidates[0]
            .definition
            .entry()
            .expect("an archive candidate has an entry")
            .ordinal,
        0
    );
    assert_eq!(
        report.candidates[1]
            .definition
            .entry()
            .expect("an archive candidate has an entry")
            .ordinal,
        1
    );
    assert_eq!(
        class_headers(&report),
        2,
        "every candidate of an indistinguishable position is a read attempt"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema
    );
}

#[test]
fn a_byte_equal_duplicate_is_ambiguous_not_ordinal() {
    let bytes = class_bytes(b"p/A", 52);
    let snapshot = open(zip_of(&[(b"p/A.class", &bytes), (b"p/A.class", &bytes)]));
    let roots = vec![snapshot_root(&snapshot)];
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        roots.clone(),
    );
    let environment = environment(&snapshot, caller.clone(), vec![caller], Vec::new());
    let request = class_request(environment, &loader("app"), b"p/A");
    let mut budget = Budget::new(limits());
    let report = resolve(&[snapshot], &request, &mut budget);

    assert_eq!(report.state, Some(ResolutionState::Ambiguous));
    assert_eq!(report.candidates.len(), 2);
    assert_eq!(
        report.candidates[0].definition.class_bytes, report.candidates[1].definition.class_bytes,
        "the fixture really is byte-equal"
    );
    assert_ne!(
        report.candidates[0].definition, report.candidates[1].definition,
        "equal bytes at two origins do not merge into one definition"
    );
    assert_eq!(class_headers(&report), 2);
}

#[test]
fn the_same_bytes_at_two_origins_are_not_merged() {
    let bytes = class_bytes(b"p/S", 52);
    // Two different archives (so two origins) that hold byte-equal class content: the second
    // one carries an unrelated entry, which is what makes the archive itself a different
    // snapshot.
    let platform = open(zip_of(&[(b"p/S.class", &bytes)]));
    let app = open(zip_of(&[
        (b"p/S.class", &bytes),
        (b"other/O.class", &class_bytes(b"other/O", 52)),
    ]));
    assert_ne!(
        platform.id(),
        app.id(),
        "the fixture really has two origins"
    );
    let platform_definition = archive_definition(&platform, b"p/S.class", &bytes);
    let app_definition = archive_definition(&app, b"p/S.class", &bytes);
    assert_eq!(
        platform_definition.class_bytes, app_definition.class_bytes,
        "the fixture really is byte-equal"
    );
    assert_ne!(
        platform_definition, app_definition,
        "the origin belongs to the definition identity"
    );

    for (delegation, expected_loader, expected) in [
        (
            DelegationPolicy::ParentFirst,
            loader("platform"),
            platform_definition.clone(),
        ),
        (
            DelegationPolicy::ChildFirst,
            loader("app"),
            app_definition.clone(),
        ),
    ] {
        let caller = domain(
            &loader("app"),
            Some(loader("platform")),
            delegation.clone(),
            vec![snapshot_root(&app)],
        );
        let parent = domain(
            &loader("platform"),
            None,
            delegation,
            vec![snapshot_root(&platform)],
        );
        let environment = environment(&app, caller.clone(), vec![caller, parent], Vec::new());
        let request = class_request(environment, &loader("app"), b"p/S");
        let report = resolve(
            &[platform.clone(), app.clone()],
            &request,
            &mut Budget::new(limits()),
        );

        assert_eq!(report.state, Some(ResolutionState::Resolved));
        assert_eq!(resolved_of(&report).loader, expected_loader);
        assert_eq!(
            resolved_of(&report).definition,
            expected,
            "the declared order decides between byte-equal origins"
        );
        assert_eq!(class_headers(&report), 1);
    }
}

#[test]
fn a_standalone_class_root_is_matched_through_its_own_name() {
    let standalone_bytes = class_bytes(b"p/S", 52);
    let standalone = open(standalone_bytes.clone());
    assert_eq!(standalone.kind(), ArtifactKind::StandaloneClass);
    let archive_bytes = class_bytes(b"p/S", 51);
    let archive = open(zip_of(&[(b"p/S.class", &archive_bytes)]));

    // The standalone root declares another name: it holds no candidate for this lookup and the
    // search continues with the next position.
    let other = open(class_bytes(b"other/C", 52));
    assert_eq!(other.kind(), ArtifactKind::StandaloneClass);
    let roots = vec![snapshot_root(&other), snapshot_root(&archive)];
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        roots.clone(),
    );
    let first_environment = environment(&other, caller.clone(), vec![caller], Vec::new());
    let request = class_request(first_environment, &loader("app"), b"p/S");
    let mut budget = Budget::new(limits());
    let report = resolve(&[other, archive.clone()], &request, &mut budget);

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        resolved_of(&report).definition,
        archive_definition(&archive, b"p/S.class", &archive_bytes)
    );
    assert_eq!(
        class_headers(&report),
        2,
        "the standalone root that declares another name was read once and did not match"
    );

    // The standalone root that declares this name is the definition, and it has no entry.
    let roots = vec![snapshot_root(&standalone)];
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        roots.clone(),
    );
    let second_environment = environment(&standalone, caller.clone(), vec![caller], Vec::new());
    let request = class_request(second_environment, &loader("app"), b"p/S");
    let mut budget = Budget::new(limits());
    let report = resolve(std::slice::from_ref(&standalone), &request, &mut budget);

    let definition = &resolved_of(&report).definition;
    assert_eq!(
        *definition,
        standalone_definition(&standalone, &standalone_bytes)
    );
    assert!(
        matches!(
            definition.location,
            PhysicalClassLocation::StandaloneRoot { .. }
        ),
        "a standalone CLASS root is not an archive entry"
    );
    assert!(definition.entry().is_none());
    assert_eq!(class_headers(&report), 1);
}

#[test]
fn a_damaged_candidate_fails_at_its_origin_without_falling_back() {
    let damaged = open(zip_of(&[(b"p/D.class", b"not a class file")]));
    let damaged_id = damaged.id().0.clone();
    let healthy_bytes = class_bytes(b"p/D", 52);
    let healthy = open(zip_of(&[(b"p/D.class", &healthy_bytes)]));
    let roots = vec![snapshot_root(&damaged), snapshot_root(&healthy)];
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        roots.clone(),
    );
    let damaged_environment = environment(&damaged, caller.clone(), vec![caller], Vec::new());
    let request = class_request(damaged_environment, &loader("app"), b"p/D");
    let mut budget = Budget::new(limits());
    let report = resolve(&[damaged, healthy], &request, &mut budget);

    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert!(
        report.state.is_none(),
        "a read failure is not a semantic decision: `state` stays None and `Inaccessible`/\
         `IncompatibleClassChange` stay reserved for the access and link rules (2.3/2.5), got {:?}",
        report.state
    );
    assert!(
        report.resolved.is_none(),
        "the later root must not be used as a fallback"
    );
    assert!(report.candidates.is_empty());
    assert_eq!(
        class_headers(&report),
        1,
        "the damaged candidate is the one read attempt"
    );
    assert_eq!(
        diagnostic_codes(&report.diagnostics),
        vec!["classfile_decode"]
    );
    let diagnostic = &report.diagnostics[0];
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    for expected in ["root 0", "loader `app`", "p/D.class", damaged_id.as_str()] {
        assert!(
            diagnostic.message.contains(expected),
            "the diagnostic names the origin ({expected}): {}",
            diagnostic.message
        );
    }
    assert!(matches!(
        report.execution,
        ExecutionReport::Failed {
            reason: TerminationReason::Error { ref code },
            ..
        } if code == "classfile_decode"
    ));
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert_eq!(
        report.coverage.runtime_resolution.scanned,
        Vec::new(),
        "a position that refused the search is not an examined position"
    );
    assert_eq!(
        report.coverage.runtime_resolution.skipped,
        vec![search_range(0, 2)],
        "the positions the stop never reached stay declared as unfinished"
    );
}

#[test]
fn a_failed_read_attempt_is_charged_and_has_no_fallback() {
    // The entry is damaged *below* the decoder: the archive lists it and the lookup selects it,
    // and the failure comes from the read layer (`entry_integrity`, the stored entry's CRC no
    // longer matching its bytes). Two properties have no other anchor on this source:
    //
    // * a read **attempt** is charged a `ClassHeaders` even though it failed — the charge is
    //   taken before the read, so the counter records work that was started, not work that
    //   succeeded;
    // * the failed read does not fall back to a later root, which is the same rule the decode
    //   failure is held to.
    let name: &[u8] = b"p/R.class";
    let healthy_bytes = class_bytes(b"p/R", 51);
    let healthy = open(zip_of(&[(name, &healthy_bytes)]));
    let (damaged_bytes, offset) = zip_with_damaged_entry(&[(name, &class_bytes(b"p/R", 52))], name);

    // Fixture sanity: the damage sits in the entry's data area, so the archive still lists the
    // entry and only *reading* it fails. Without this the test could be passing on an archive
    // that fails earlier, for another reason.
    let probe = open(damaged_bytes.clone());
    assert_eq!(probe.kind(), ArtifactKind::Zip);
    let mut sanity = Budget::new(limits());
    let listing = probe
        .enumerate(&mut sanity)
        .expect("the damaged archive lists");
    let listed = listing
        .entries
        .iter()
        .find(|entry| entry.id.raw_name.0 == name)
        .expect("the damaged archive still declares the entry");
    assert_eq!(listed.layout.compressed_data.start, offset);
    assert!(
        matches!(
            probe.read_entry(listed, &mut sanity),
            Err(Error::InvalidInput { ref code, .. }) if code == "entry_integrity"
        ),
        "the fixture's damage is a read-layer integrity failure"
    );

    let damaged = probe;
    let damaged_id = damaged.id().0.clone();
    let roots = vec![snapshot_root(&damaged), snapshot_root(&healthy)];
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        roots.clone(),
    );
    let damaged_environment = environment(&damaged, caller.clone(), vec![caller], Vec::new());
    let request = class_request(damaged_environment, &loader("app"), b"p/R");
    let content = [damaged, healthy];
    let mut budget = Budget::new(limits());
    let report = resolve(&content, &request, &mut budget);

    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert!(
        report.state.is_none(),
        "a read failure decides nothing, so `state` stays None: {:?}",
        report.state
    );
    assert_ne!(
        report.state,
        Some(ResolutionState::Resolved),
        "the healthy definition in the later root must not be used as a fallback"
    );
    assert!(report.resolved.is_none());
    assert!(report.candidates.is_empty());
    assert!(
        matches!(
            report.execution,
            ExecutionReport::Failed {
                reason: TerminationReason::Error { ref code },
                ..
            } if code == "entry_integrity"
        ),
        "the read layer's own failure is reported: {:?}",
        report.execution
    );
    assert_eq!(
        diagnostic_codes(&report.diagnostics),
        vec!["entry_integrity"]
    );
    let diagnostic = &report.diagnostics[0];
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    for expected in ["root 0", "loader `app`", "p/R.class", damaged_id.as_str()] {
        assert!(
            diagnostic.message.contains(expected),
            "the diagnostic names the origin ({expected}): {}",
            diagnostic.message
        );
    }

    // The anchor of "charged before the read": the attempt failed, and it is still one attempt.
    assert_eq!(
        class_headers(&report),
        1,
        "a failed read attempt is one header read attempt; charging it only after a successful \
         read would report zero work for a read that really was started"
    );
    assert_eq!(budget.usage().class_headers, 1);
    // The attempt really did reach the read layer: its own dimensions were charged before the
    // integrity check rejected the bytes.
    let usage = usage_of(&report.execution);
    assert!(
        usage.archive_entries >= 1 && usage.read_bytes > 0,
        "the read layer charged its own dimensions before failing: {usage:?}"
    );
    assert_eq!(
        usage,
        &budget.usage(),
        "the report's usage is the request budget's usage"
    );

    // The position that failed is a stop, not an examined position: nothing is covered and the
    // whole declared order stays unfinished.
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert_eq!(
        report.coverage.runtime_resolution.scanned,
        Vec::new(),
        "a position that refused the search is not an examined position"
    );
    assert_eq!(
        report.coverage.runtime_resolution.skipped,
        vec![search_range(0, 2)],
        "the positions the stop never reached stay declared as unfinished"
    );
}

#[test]
fn a_class_header_budget_stop_stays_a_budget_stop() {
    let bytes = class_bytes(b"p/S", 52);
    let snapshot = open(zip_of(&[(b"p/S.class", &bytes)]));
    let roots = vec![snapshot_root(&snapshot)];
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        roots.clone(),
    );
    let environment = environment(&snapshot, caller.clone(), vec![caller], Vec::new());
    let request = class_request(environment, &loader("app"), b"p/S");
    let mut budget = Budget::new(Limits {
        class_headers: 0,
        ..limits()
    });
    let report = resolve(&[snapshot], &request, &mut budget);

    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert_eq!(report.state, Some(ResolutionState::BudgetExceeded));
    assert!(report.resolved.is_none());
    assert_eq!(
        diagnostic_codes(&report.diagnostics),
        vec!["budget_exceeded_class_headers"]
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ClassHeaders
            },
            ..
        }
    ));
    assert_eq!(
        class_headers(&report),
        0,
        "a refused read attempt is not a header that was read"
    );
    assert_eq!(budget.usage().class_headers, 0);
    // The refusal happens *before* the read: the candidate the position selected was never
    // materialized, so the read layer's own dimensions stayed untouched (the listing that found
    // the candidate did run, which is why the entry count is not zero as well). Charging after a
    // successful read instead would spend these bytes and only then report the stop.
    let usage = usage_of(&report.execution);
    assert_eq!(
        (usage.read_bytes, usage.entry_bytes, usage.class_bytes),
        (0, 0, 0),
        "a refused attempt read no byte: {usage:?}"
    );
    assert!(usage.archive_entries >= 1, "the listing ran: {usage:?}");
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert_eq!(
        report.coverage.runtime_resolution.skipped,
        vec![search_range(0, 1)],
        "the position the budget stopped stays unfinished"
    );
}

#[test]
fn a_cancelled_lookup_is_cancelled_and_names_no_state() {
    let bytes = class_bytes(b"p/S", 52);
    let snapshot = open(zip_of(&[(b"p/S.class", &bytes)]));
    let roots = vec![snapshot_root(&snapshot)];
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        roots.clone(),
    );
    let environment = environment(&snapshot, caller.clone(), vec![caller], Vec::new());
    let request = class_request(environment, &loader("app"), b"p/S");
    let mut budget = Budget::new(limits());
    let token = budget.cancellation_token();
    token.cancel();
    let report = resolve(&[snapshot], &request, &mut budget);

    assert_eq!(
        report.analysis,
        ResolutionAnalysis::Performed,
        "the lookup ran and stopped at the cancellation: the capability did run, it just never \
         reached a decision (invariant 3 keeps the two planes independent)"
    );
    assert!(
        report.state.is_none(),
        "a cancellation is not a semantic decision, so `state` stays None: {:?}",
        report.state
    );
    assert!(report.resolved.is_none());
    assert_eq!(diagnostic_codes(&report.diagnostics), vec!["cancelled"]);
    assert!(matches!(
        report.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert_eq!(class_headers(&report), 0);
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert_eq!(
        report.coverage.runtime_resolution.skipped,
        vec![search_range(0, 1)]
    );
}

/// One exhaustive classification of [`ResolutionState`].
///
/// The compile-time anchor this file uses for enums without an `ALL` list: a state added to the
/// public set has to be classified here, so the assertions below cannot quietly stop covering
/// the whole set.
fn resolution_state_code(state: ResolutionState) -> &'static str {
    match state {
        ResolutionState::Resolved => "resolved",
        ResolutionState::Missing => "missing",
        ResolutionState::Ambiguous => "ambiguous",
        ResolutionState::Inaccessible => "inaccessible",
        ResolutionState::IncompatibleClassChange => "incompatible_class_change",
        ResolutionState::UnsupportedPolicy => "unsupported_policy",
        ResolutionState::BudgetExceeded => "budget_exceeded",
    }
}

#[test]
fn reading_failures_and_cancellations_never_occupy_a_semantic_state() {
    // The lookup can decide three things, and a budget stop is the fourth: those are the only
    // states this entry publishes, and `state = Some(..)` is reserved for exactly those runs
    // that reached a decision. A damaged candidate and a cancellation stop the run without
    // deciding anything, so they report `state = None` and let `execution` carry the stop —
    // `Inaccessible`/`IncompatibleClassChange` stay reserved for the access and link rules of
    // 2.3/2.5 and never stand for a read failure.
    let name_bytes = class_bytes(b"p/S", 52);
    let found = open(zip_of(&[(b"p/S.class", &name_bytes)]));
    let missing = open(zip_of(&[(b"other/C.class", &class_bytes(b"other/C", 52))]));
    let ambiguous = open(zip_of(&[
        (b"p/S.class", &name_bytes),
        (b"p/S.class", &class_bytes(b"p/S", 51)),
    ]));
    let damaged = open(zip_of(&[(b"p/D.class", b"not a class file")]));

    // One snapshot, one class symbol, the caller's own domain as the environment — the only
    // knob each case turns is the budget and the cancellation token.
    let run = |snapshot: &ArtifactSnapshot, name: &[u8], limits: Limits, cancel: bool| {
        let roots = vec![snapshot_root(snapshot)];
        let caller = domain(
            &loader("app"),
            None,
            DelegationPolicy::ParentFirst,
            roots.clone(),
        );
        let environment = environment(snapshot, caller.clone(), vec![caller], Vec::new());
        let request = class_request(environment, &loader("app"), name);
        let mut budget = Budget::new(limits);
        if cancel {
            budget.cancellation_token().cancel();
        }
        resolve(std::slice::from_ref(snapshot), &request, &mut budget)
    };

    let observed = vec![
        ("found", run(&found, b"p/S", limits(), false)),
        ("missing", run(&missing, b"p/S", limits(), false)),
        ("ambiguous", run(&ambiguous, b"p/S", limits(), false)),
        ("damaged candidate", run(&damaged, b"p/D", limits(), false)),
        ("cancelled", run(&found, b"p/S", limits(), true)),
        (
            "header budget stop",
            run(
                &found,
                b"p/S",
                Limits {
                    class_headers: 0,
                    ..limits()
                },
                false,
            ),
        ),
    ];
    let state_of = |label: &str| {
        observed
            .iter()
            .find(|(name, _)| *name == label)
            .expect("the label is part of the traversal")
            .1
            .state
    };

    // The four decisions, one fixture each.
    assert_eq!(state_of("found"), Some(ResolutionState::Resolved));
    assert_eq!(state_of("missing"), Some(ResolutionState::Missing));
    assert_eq!(state_of("ambiguous"), Some(ResolutionState::Ambiguous));
    assert_eq!(
        state_of("header budget stop"),
        Some(ResolutionState::BudgetExceeded),
        "a budget stop is a decision: the declared bounds are what decided the answer"
    );
    // The two runs that stopped before deciding.
    assert_eq!(state_of("damaged candidate"), None);
    assert_eq!(state_of("cancelled"), None);

    let decided = ["resolved", "missing", "ambiguous", "budget_exceeded"];
    let mut undecided = Vec::new();
    for (label, report) in &observed {
        assert!(
            !matches!(
                report.state,
                Some(
                    ResolutionState::Inaccessible
                        | ResolutionState::IncompatibleClassChange
                        | ResolutionState::UnsupportedPolicy
                )
            ),
            "{label} published `{}`: read failures, cancellations and unimplemented capabilities \
             never borrow a semantic state",
            report.state.map(resolution_state_code).unwrap_or("none")
        );
        match report.state {
            Some(state) => {
                assert!(
                    decided.contains(&resolution_state_code(state)),
                    "{label} published `{}` without being one of this entry's decisions",
                    resolution_state_code(state)
                );
                assert_eq!(
                    report.analysis,
                    ResolutionAnalysis::Performed,
                    "{label} reached a decision, so the capability ran"
                );
            }
            None => {
                undecided.push(*label);
                assert_eq!(
                    report.analysis,
                    ResolutionAnalysis::Performed,
                    "{label} entered the lookup, so the capability ran even though it decided \
                     nothing"
                );
                assert!(
                    !matches!(report.execution, ExecutionReport::Complete { .. }),
                    "{label} stopped without a decision, so its execution cannot be complete"
                );
                assert!(
                    report.resolved.is_none() && report.candidates.is_empty(),
                    "{label} decided nothing, so it publishes no definition and no candidate"
                );
            }
        }
    }
    assert_eq!(
        undecided,
        vec!["damaged candidate", "cancelled"],
        "exactly the two stops without a decision report `state = None`"
    );
}

#[test]
fn a_listing_that_stops_early_never_reads_as_missing() {
    let name_bytes = class_bytes(b"p/S", 52);
    let snapshot = open(zip_of(&[
        (b"p/S.class", &name_bytes),
        (b"other/C.class", &class_bytes(b"other/C", 52)),
    ]));
    let roots = vec![snapshot_root(&snapshot)];
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        roots.clone(),
    );
    let environment = environment(&snapshot, caller.clone(), vec![caller], Vec::new());
    let request = class_request(environment, &loader("app"), b"p/S");
    let mut budget = Budget::new(Limits {
        archive_entries: 1,
        ..limits()
    });
    let report = resolve(&[snapshot], &request, &mut budget);

    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert_eq!(report.state, Some(ResolutionState::BudgetExceeded));
    assert!(
        report.resolved.is_none(),
        "a name inside the listed prefix is still not decided: the candidate set is unknown"
    );
    assert_eq!(
        diagnostic_codes(&report.diagnostics),
        vec!["budget_exceeded_archive_entries"]
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
    assert_eq!(class_headers(&report), 0, "no candidate was read");
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
}

#[test]
fn a_rejected_environment_never_selects_a_definition() {
    let snapshot = open(zip_of(&[(b"p/S.class", &class_bytes(b"p/S", 52))]));
    let root = vec![snapshot_root(&snapshot)];
    let content = [snapshot.clone()];

    // A parent no domain binds.
    let caller = domain(
        &loader("app"),
        Some(loader("platform")),
        DelegationPolicy::ParentFirst,
        root.clone(),
    );
    assert_rejected(
        environment(&snapshot, caller.clone(), vec![caller], Vec::new()),
        &loader("app"),
        &content,
        &["missing_parent"],
    );

    // A parent cycle: the validator reports every domain whose chain re-enters itself.
    let caller = domain(
        &loader("app"),
        Some(loader("platform")),
        DelegationPolicy::ParentFirst,
        root.clone(),
    );
    let parent = domain(
        &loader("platform"),
        Some(loader("app")),
        DelegationPolicy::ParentFirst,
        Vec::new(),
    );
    assert_rejected(
        environment(&snapshot, caller.clone(), vec![caller, parent], Vec::new()),
        &loader("app"),
        &content,
        &["parent_cycle", "parent_cycle"],
    );

    // A module mode this slice cannot execute.
    let mut caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        root.clone(),
    );
    caller.module_mode = ModuleMode::ModulePath;
    assert_rejected(
        environment(&snapshot, caller.clone(), vec![caller], Vec::new()),
        &loader("app"),
        &content,
        &["unsupported_policy"],
    );

    // A delegation policy this slice cannot execute.
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::Custom {
            id: "osgi".to_string(),
        },
        root.clone(),
    );
    assert_rejected(
        environment(&snapshot, caller.clone(), vec![caller], Vec::new()),
        &loader("app"),
        &content,
        &["unsupported_policy"],
    );

    let caller = domain(&loader("app"), None, DelegationPolicy::Unknown, root);
    assert_rejected(
        environment(&snapshot, caller.clone(), vec![caller], Vec::new()),
        &loader("app"),
        &content,
        &["unsupported_policy"],
    );

    // A request whose caller identity names another loader than its own caller domain: the
    // same fixture resolves nothing, and the problem names the caller that has to change.
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        vec![snapshot_root(&snapshot)],
    );
    assert_rejected(
        environment(&snapshot, caller.clone(), vec![caller], Vec::new()),
        &loader("platform"),
        &content,
        &["caller_loader_mismatch"],
    );
}

#[test]
fn unreadable_and_unprovided_roots_are_problems_not_positions() {
    let snapshot = open(zip_of(&[(b"p/S.class", &class_bytes(b"p/S", 52))]));
    let content = [snapshot.clone()];

    // An external root is declared but not readable.
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        vec![
            snapshot_root(&snapshot),
            LoadRoot::External {
                id: "boot-classpath".to_string(),
            },
        ],
    );
    assert_rejected(
        environment(&snapshot, caller.clone(), vec![caller], Vec::new()),
        &loader("app"),
        &content,
        &["unreadable_root"],
    );

    // A root whose content the request does not provide is not searched either.
    let unprovided = open(zip_of(&[(b"q/T.class", &class_bytes(b"q/T", 52))]));
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        vec![snapshot_root(&snapshot), snapshot_root(&unprovided)],
    );
    assert_rejected(
        environment(&snapshot, caller.clone(), vec![caller], Vec::new()),
        &loader("app"),
        &content,
        &["content_not_provided"],
    );
}

#[test]
fn a_dispatch_request_over_a_class_symbol_names_the_missing_member_declaration() {
    // 2.5 enumerates the known overrides of a *resolved member declaration*. A class symbol is a
    // type, not a member declaration, so the lookup still runs and still resolves the class, and
    // the requested range is answered with no dispatch at all plus the diagnostic that says why
    // — never with an empty candidate list that would read as "nothing overrides it".
    let (fixture, environment) = ordered_roots(DelegationPolicy::ParentFirst);
    let mut request = class_request(environment, &loader("app"), b"p/S");
    request.dispatch = Some(DispatchScope {
        scope: PhysicalScope::SnapshotAll,
        consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
    });
    let report = resolve(&fixture.content, &request, &mut Budget::new(limits()));

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert!(
        report.dispatch.is_none(),
        "a range no member declaration resolved must not look like an empty one"
    );
    assert_eq!(
        diagnostic_codes(&report.diagnostics),
        vec!["resolution_dispatch_no_declaration"]
    );
    assert_eq!(
        report.diagnostics[0].severity,
        DiagnosticSeverity::Warning,
        "the declaration lookup in the same request did run, so the uncovered range is a \
         warning rather than a rejected request"
    );
    assert_eq!(class_headers(&report), 1);
}

#[test]
fn an_artifact_tree_root_searches_its_own_container_only() {
    let outer_bytes = class_bytes(b"p/S", 52);
    let inner_bytes = class_bytes(b"p/S", 51);
    let inner = zip_of(&[(b"p/S.class", &inner_bytes)]);
    let outer = open(zip_of(&[
        (b"BOOT-INF/classes/p/S.class", &outer_bytes),
        (b"BOOT-INF/lib/inner.jar", &inner),
    ]));

    let mut budget = Budget::new(limits());
    let tree = Engine::new()
        .enumerate_artifact_tree(&outer, &mut budget)
        .expect("the fixture tree enumerates");
    assert_eq!(tree.containers.len(), 2);
    let root_container = tree
        .containers
        .iter()
        .find(|container| container.origin.steps.is_empty())
        .expect("the root container");
    let nested = tree
        .containers
        .iter()
        .find(|container| !container.origin.steps.is_empty())
        .expect("the nested jar is a container of its own");
    let nested_entry = nested
        .entries
        .iter()
        .find(|entry| entry.id.raw_name.0 == b"p/S.class")
        .expect("the nested jar holds the entry");
    let nested_definition = definition_of_entry(nested_entry, &inner_bytes);

    // The root container holds no entry with this name: a nested container is a position only
    // when the environment declares it as its own root.
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        vec![LoadRoot::ArtifactTree {
            root: root_container.origin.clone(),
        }],
    );
    let root_environment = environment(&outer, caller.clone(), vec![caller], Vec::new());
    let request = class_request(root_environment, &loader("app"), b"p/S");
    let report = resolve(
        std::slice::from_ref(&outer),
        &request,
        &mut Budget::new(limits()),
    );
    assert_eq!(report.state, Some(ResolutionState::Missing));
    assert_eq!(class_headers(&report), 0);

    // The nested container is the position that holds the entry.
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        vec![LoadRoot::ArtifactTree {
            root: nested.origin.clone(),
        }],
    );
    let nested_environment = environment(&outer, caller.clone(), vec![caller], Vec::new());
    let request = class_request(nested_environment, &loader("app"), b"p/S");
    let report = resolve(
        std::slice::from_ref(&outer),
        &request,
        &mut Budget::new(limits()),
    );
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(resolved_of(&report).definition, nested_definition);
    assert_eq!(
        resolved_of(&report)
            .definition
            .entry()
            .expect("the nested entry is an archive entry")
            .origin
            .steps
            .len(),
        1,
        "the definition keeps the origin chain of the nested container"
    );
    assert_eq!(class_headers(&report), 1);
}

/// A rejected environment starts no lookup: the class symbol keeps the honest unavailable state
/// and the problems name the declarations that have to change.
///
/// The caller loader is a parameter because the request carries the caller identity: it is part
/// of the request the validator reads, next to the environment.
fn assert_rejected(
    environment: ResolutionEnvironment,
    caller: &LoaderId,
    content: &[ArtifactSnapshot],
    expected: &[&str],
) {
    let request = class_request(environment, caller, b"p/S");
    let mut budget = Budget::new(limits());
    let report = resolve(content, &request, &mut budget);

    assert_not_performed(&report);
    assert_eq!(report.target, request.target);
    assert_eq!(problem_codes(&report.environment_problems), expected);
    let mut expected_diagnostics = expected
        .iter()
        .map(|code| (*code).to_string())
        .collect::<Vec<_>>();
    expected_diagnostics.push("resolution_not_implemented".to_string());
    assert_eq!(
        diagnostic_codes(&report.diagnostics),
        expected_diagnostics,
        "every environment problem is reported under its own code, then the capability code"
    );
    for diagnostic in report.diagnostics.iter().take(expected.len()) {
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    }
    assert_eq!(
        unsupported_code(&report.execution),
        Some("resolution_not_implemented")
    );
    assert!(counted_usage_is_zero(&budget.usage()));
}
