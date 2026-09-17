//! P2 2.2 acceptance: the read records of the public reports.
//!
//! The 2.2 slice turns the single 2.1 lookup into a request-scoped closure and publishes what
//! that closure really read. What a test through the public API can and has to prove is:
//!
//! 1. a performed class lookup records the definition it read, once, in read order, with the
//!    demand that read it (`RequestedDefinition`), while a member symbol — which this slice
//!    does not resolve — records nothing at all,
//! 2. the record carries the definition that was really read: a standalone root that declares
//!    another name is read, charged and recorded before the position that decides the name,
//! 3. byte-equal definitions under two loaders stay two bindings, and each report records the
//!    one its own search order selected,
//! 4. `reads.len() <= usage.class_headers` holds for a decision, for a budget stop and for a
//!    cancellation, and a request never records more than it charged,
//! 5. a class request reads no code byte of a class that really has a body — the evidence is
//!    `code_bytes == 0` next to the P1 body path's non-zero charge on the same fixture, since
//!    `method_bodies` has no charge point before 3.x — and the reasons it publishes are exactly
//!    the demands a header closure may have,
//! 6. an unreadable candidate is charged but never recorded, every indistinguishable candidate
//!    of one position is recorded, and a read failure keeps the records of the reads that
//!    really happened before it.
//!
//! The closure's own expansion semantics — dependency depth, fan-out, cycles, missing
//! supertypes — need the crate-private walks that 2.3/2.5 call, so those are pinned by the
//! `providers` unit tests; this file covers what the public reports expose today.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

/// The committed historical fixture: one class whose `finallyPath(I)I` really has a body.
const HISTORICAL: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

const STORE: u16 = 0;

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        // Funded for the body-path control below; the closure under test never charges it.
        code_bytes: 1 << 20,
        output_bytes: 1 << 20,
        result_items: 10_000,
        class_headers: 100,
        nested_depth: 4,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

/// Smallest class file the reader accepts, with `this_class` set to `this_class`.
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

/// One stored ZIP with the given entries.
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

/// The physical definition the engine derives for one archive entry.
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
        providers: Vec::new(),
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

/// One `app` loader whose only root is the given snapshot.
fn single_loader(runtime: &ArtifactSnapshot) -> ResolutionEnvironment {
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        vec![snapshot_root(runtime)],
    );
    environment(runtime, caller.clone(), vec![caller])
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

fn class_headers(report: &ResolutionReport) -> u64 {
    usage_of(&report.execution).counted_usage(CountedBudgetDimension::ClassHeaders)
}

/// `reads.len() <= usage.class_headers`, checked against the report's own usage.
fn assert_reads_within_attempts(report: &ResolutionReport) {
    assert!(
        u64::try_from(report.reads.len()).expect("a read count fits u64") <= class_headers(report),
        "{} reads cannot exceed {} header read attempts: {:?}",
        report.reads.len(),
        class_headers(report),
        report.reads
    );
}

fn resolved_of(report: &ResolutionReport) -> &ResolvedMemberRef {
    report
        .resolved
        .as_ref()
        .expect("a resolved lookup publishes its definition")
}

fn search_range(start: u64, end: u64) -> CoverageRange {
    CoverageRange {
        label: "provider_search_position".to_string(),
        start,
        end,
    }
}

#[test]
fn a_class_lookup_records_the_header_it_read() {
    let bytes = class_bytes(b"p/S", 52);
    let snapshot = open(zip_of(&[(b"p/S.class", &bytes)]));
    let definition = archive_definition(&snapshot, b"p/S.class", &bytes);
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        vec![snapshot_root(&snapshot)],
    );
    let environment = environment(&snapshot, caller.clone(), vec![caller]);
    let request = class_request(environment, &loader("app"), b"p/S");
    let mut budget = Budget::new(limits());
    let report = resolve(std::slice::from_ref(&snapshot), &request, &mut budget);

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(resolved_of(&report).definition, definition);
    assert_eq!(
        report.reads,
        vec![HeaderRead {
            loader: loader("app"),
            definition: definition.clone(),
            reason: ReadReason::RequestedDefinition,
        }],
        "one read of the requested definition, in read order"
    );
    assert_eq!(
        class_headers(&report),
        1,
        "one read attempt, and the record does not add a second"
    );
    assert_reads_within_attempts(&report);
    // "No unrelated body was read" is *not* asserted here: this synthetic fixture has no
    // method at all, so a zero would be a property of the fixture. The real evidence is
    // `a_request_for_a_class_that_has_a_body_reads_no_code_byte` below, which uses a class
    // with a `Code` attribute and contrasts the closure against the P1 body path.
    assert!(
        report
            .reads
            .iter()
            .all(|read| read.reason == ReadReason::RequestedDefinition),
        "the reason set stays inside what a class request demands"
    );

    // The field is additive but serialized like every other one, so a consumer that reads the
    // report JSON sees the evidence and its snake_case reason.
    let json = serde_json::to_string(&report).expect("the report serializes");
    assert!(json.contains("\"reads\""));
    assert!(json.contains("\"reason\":\"requested_definition\""));
    assert_eq!(
        serde_json::from_str::<ResolutionReport>(&json).expect("the report round-trips"),
        report
    );
}

/// The closure reads no `Code` byte of a class that really has a body.
///
/// `usage.method_bodies` cannot carry this evidence before 3.x (nothing in the crate charges
/// it), so the evidence is `code_bytes == 0` on a class whose body the P1 path demonstrably
/// does charge — the control below is what gives the metric its discriminating power.
#[test]
fn a_request_for_a_class_that_has_a_body_reads_no_code_byte() {
    let snapshot = open(HISTORICAL.to_vec());
    assert_eq!(snapshot.kind(), ArtifactKind::StandaloneClass);

    // Sanity: this fixture really declares a method that really has a `Code` attribute, read
    // through the public header surface.
    let mut header_budget = Budget::new(limits());
    let header = Engine::new()
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut header_budget,
            InspectionMode::Strict,
        )
        .expect("the fixture header reads")
        .inspection
        .header;
    let body_method = header
        .methods
        .iter()
        .find(|method| method.name.raw().0 == b"finallyPath")
        .expect("the fixture declares `finallyPath`");
    assert!(
        body_method
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Code"),
        "the fixture's `finallyPath` really carries a `Code` attribute: {:?}",
        body_method
            .attributes
            .iter()
            .map(|attribute| attribute.name.raw().0.clone())
            .collect::<Vec<_>>()
    );

    // Control: the P1 body path on this same fixture charges code bytes, so a zero below is a
    // fact about the closure and not about a dimension that cannot move.
    let mut body_budget = Budget::new(limits());
    let body = Engine::new()
        .inspect_method_bytecode(
            &snapshot,
            ClassTarget::Root,
            MethodSelector {
                name: JvmBytes(b"finallyPath".to_vec()),
                descriptor: JvmBytes(b"(I)I".to_vec()),
            },
            &mut body_budget,
        )
        .expect("the fixture's body is readable");
    assert!(
        !body.inspection.instructions.is_empty(),
        "the control really decoded instructions"
    );
    let control_code_bytes = body_budget.usage().code_bytes;
    assert!(
        control_code_bytes > 0,
        "the body path charges code bytes on this fixture, otherwise the assertion below is \
         vacuous: {control_code_bytes}"
    );

    // The closure over the same class: one header read, and not one code byte.
    let environment = single_loader(&snapshot);
    let request = class_request(environment, &loader("app"), b"HistoricalControlFlow");
    let mut budget = Budget::new(limits());
    let report = resolve(std::slice::from_ref(&snapshot), &request, &mut budget);

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(class_headers(&report), 1);
    assert_eq!(
        report.reads,
        vec![HeaderRead {
            loader: loader("app"),
            definition: standalone_definition(&snapshot, HISTORICAL),
            reason: ReadReason::RequestedDefinition,
        }]
    );
    let usage = usage_of(&report.execution);
    assert_eq!(
        usage.counted_usage(CountedBudgetDimension::CodeBytes),
        0,
        "a header closure reads no code byte, though the class has a body: {usage:?}"
    );
    assert_eq!(budget.usage().code_bytes, 0);
    assert_eq!(
        usage.class_bytes,
        u64::try_from(HISTORICAL.len()).expect("fixture length fits u64"),
        "the class bytes themselves were read, so the header really was parsed"
    );
}

#[test]
fn a_damaged_candidate_is_charged_and_never_recorded() {
    // Position 0 is a standalone CLASS root that declares another name: it is read
    // successfully and recorded before the search continues. Position 1 holds the requested
    // name, and its bytes are not a class file at all.
    let other_bytes = class_bytes(b"other/O", 52);
    let other = open(other_bytes.clone());
    let damaged = open(zip_of(&[(b"p/D.class", b"not a class file")]));
    let roots = vec![snapshot_root(&other), snapshot_root(&damaged)];
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        roots.clone(),
    );
    let environment = environment(&other, caller.clone(), vec![caller]);
    let request = class_request(environment, &loader("app"), b"p/D");
    let mut budget = Budget::new(limits());
    let report = resolve(&[other.clone(), damaged], &request, &mut budget);

    assert!(
        report.state.is_none(),
        "a read failure is not a semantic decision: {:?}",
        report.state
    );
    assert!(report.resolved.is_none());
    assert_eq!(
        class_headers(&report),
        2,
        "the failed read is an attempt like any other, charged before the read"
    );
    assert_eq!(
        report.reads,
        vec![HeaderRead {
            loader: loader("app"),
            definition: standalone_definition(&other, &other_bytes),
            reason: ReadReason::RequestedDefinition,
        }],
        "only a read that produced a definition identity is recorded; the bytes that were not \
         a class file produce no record, and the successful read before them keeps its own"
    );
    assert_eq!(
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>(),
        vec!["classfile_decode"]
    );
    assert_reads_within_attempts(&report);
}

#[test]
fn every_indistinguishable_candidate_of_one_position_is_recorded() {
    let high = class_bytes(b"p/A", 52);
    let low = class_bytes(b"p/A", 51);
    let snapshot = open(zip_of(&[(b"p/A.class", &high), (b"p/A.class", &low)]));
    let environment = single_loader(&snapshot);
    let request = class_request(environment, &loader("app"), b"p/A");
    let mut budget = Budget::new(limits());
    let report = resolve(std::slice::from_ref(&snapshot), &request, &mut budget);

    assert_eq!(report.state, Some(ResolutionState::Ambiguous));
    assert_eq!(
        report.candidates.len(),
        2,
        "each origin is listed on its own"
    );
    assert_eq!(
        report.reads.len(),
        2,
        "an ambiguous position really read both candidates, so both are recorded: {:?}",
        report.reads
    );
    assert_eq!(
        report
            .reads
            .iter()
            .map(|read| (&read.definition, read.reason))
            .collect::<Vec<_>>(),
        report
            .candidates
            .iter()
            .map(|candidate| (&candidate.definition, ReadReason::RequestedDefinition))
            .collect::<Vec<_>>(),
        "the records name the same definitions the candidates name, in read order"
    );
    assert_eq!(class_headers(&report), 2);
    assert_reads_within_attempts(&report);
}

#[test]
fn a_read_is_recorded_for_the_definition_whose_bytes_were_read() {
    // The first position is a standalone CLASS root that declares another name: it is read and
    // charged before the position that decides the name, and it is recorded under the
    // definition it really read rather than hidden because it did not decide.
    let other_bytes = class_bytes(b"other/O", 52);
    let other = open(other_bytes.clone());
    let bytes = class_bytes(b"p/S", 52);
    let snapshot = open(zip_of(&[(b"p/S.class", &bytes)]));
    let roots = vec![snapshot_root(&other), snapshot_root(&snapshot)];
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        roots.clone(),
    );
    let environment = environment(&other, caller.clone(), vec![caller]);
    let request = class_request(environment, &loader("app"), b"p/S");
    let mut budget = Budget::new(limits());
    let report = resolve(&[other.clone(), snapshot.clone()], &request, &mut budget);

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        resolved_of(&report).definition,
        archive_definition(&snapshot, b"p/S.class", &bytes)
    );
    assert_eq!(
        class_headers(&report),
        2,
        "the root that declares another name was read once and did not match"
    );
    assert_eq!(
        report.reads,
        vec![
            HeaderRead {
                loader: loader("app"),
                definition: standalone_definition(&other, &other_bytes),
                reason: ReadReason::RequestedDefinition,
            },
            HeaderRead {
                loader: loader("app"),
                definition: archive_definition(&snapshot, b"p/S.class", &bytes),
                reason: ReadReason::RequestedDefinition,
            },
        ],
        "every header that was read is recorded, in read order, under the definition it read"
    );
    assert_reads_within_attempts(&report);
}

#[test]
fn a_member_symbol_reads_no_header() {
    let bytes = class_bytes(b"p/S", 52);
    let snapshot = open(zip_of(&[(b"p/S.class", &bytes)]));
    let environment = single_loader(&snapshot);
    let request = ResolutionRequest {
        environment,
        target: SymbolRef::Method {
            owner: JvmBytes(b"p/S".to_vec()),
            name: JvmBytes(b"m".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        },
        use_kind: ReferenceUse::InvokeVirtual,
        caller: CallerContext {
            loader: loader("app"),
            enclosing: None,
        },
        dispatch: None,
    };
    let mut budget = Budget::new(limits());
    let report = resolve(std::slice::from_ref(&snapshot), &request, &mut budget);

    assert_eq!(report.analysis, ResolutionAnalysis::NotPerformed);
    assert!(
        report.reads.is_empty(),
        "a capability that did not run cannot have read a header"
    );
    assert_eq!(class_headers(&report), 0);
    assert_reads_within_attempts(&report);
}

#[test]
fn byte_equal_definitions_under_two_loaders_stay_two_records() {
    let bytes = class_bytes(b"p/S", 52);
    let platform = open(zip_of(&[(b"p/S.class", &bytes)]));
    let app = open(zip_of(&[
        (b"p/S.class", &bytes),
        (b"other/O.class", &class_bytes(b"other/O", 52)),
    ]));
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
        let environment = environment(&app, caller.clone(), vec![caller, parent]);
        let request = class_request(environment, &loader("app"), b"p/S");
        let report = resolve(
            &[platform.clone(), app.clone()],
            &request,
            &mut Budget::new(limits()),
        );

        assert_eq!(report.state, Some(ResolutionState::Resolved));
        assert_eq!(resolved_of(&report).loader, expected_loader);
        assert_eq!(resolved_of(&report).definition, expected);
        assert_eq!(
            report.reads,
            vec![HeaderRead {
                loader: expected_loader,
                definition: expected,
                reason: ReadReason::RequestedDefinition,
            }],
            "the record names the loader that really provided the definition"
        );
        assert_reads_within_attempts(&report);
    }
}

#[test]
fn a_pre_cancelled_request_reports_cancelled_and_reads_nothing() {
    let bytes = class_bytes(b"p/S", 52);
    let snapshot = open(zip_of(&[(b"p/S.class", &bytes)]));
    let environment = single_loader(&snapshot);
    let request = class_request(environment, &loader("app"), b"p/S");
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let report = resolve(std::slice::from_ref(&snapshot), &request, &mut budget);

    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert!(
        report.state.is_none(),
        "a cancellation is not a semantic decision: {:?}",
        report.state
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert!(
        report.reads.is_empty(),
        "a refusal that happened before the read records nothing"
    );
    assert_eq!(class_headers(&report), 0);
    assert_reads_within_attempts(&report);
    assert_eq!(
        report.coverage.runtime_resolution.skipped,
        vec![search_range(0, 1)],
        "the position the stop never reached stays unfinished"
    );
}

#[test]
fn a_budget_stop_keeps_every_record_inside_the_charged_attempts() {
    // The standalone root is read first (one attempt, one record); the next attempt is
    // refused by the header budget, so the stop is a budget stop and the healthy definition
    // behind it is never used as a fallback.
    let other_bytes = class_bytes(b"other/O", 52);
    let other = open(other_bytes.clone());
    let bytes = class_bytes(b"p/S", 52);
    let snapshot = open(zip_of(&[(b"p/S.class", &bytes)]));
    let roots = vec![snapshot_root(&other), snapshot_root(&snapshot)];
    let caller = domain(
        &loader("app"),
        None,
        DelegationPolicy::ParentFirst,
        roots.clone(),
    );
    let environment = environment(&other, caller.clone(), vec![caller]);
    let request = class_request(environment, &loader("app"), b"p/S");
    let mut budget = Budget::new(Limits {
        class_headers: 1,
        ..limits()
    });
    let report = resolve(&[other.clone(), snapshot.clone()], &request, &mut budget);

    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert_eq!(report.state, Some(ResolutionState::BudgetExceeded));
    assert!(
        report.resolved.is_none(),
        "the definition behind the stop is not a fallback"
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
    assert_eq!(class_headers(&report), 1);
    assert_eq!(
        report.reads,
        vec![HeaderRead {
            loader: loader("app"),
            definition: standalone_definition(&other, &other_bytes),
            reason: ReadReason::RequestedDefinition,
        }],
        "the read that happened before the stop keeps its record"
    );
    assert_reads_within_attempts(&report);
    assert_eq!(
        report.coverage.runtime_resolution.skipped,
        vec![search_range(1, 2)],
        "the position the budget refused stays unfinished"
    );
}

#[test]
fn the_other_two_reports_also_publish_their_reads() {
    // Declaration references and method analysis read no header in this slice, so both report
    // an empty list; the field exists on every report, which is what makes it comparable.
    let bytes = class_bytes(b"p/S", 52);
    let snapshot = open(zip_of(&[(b"p/S.class", &bytes)]));
    let environment = single_loader(&snapshot);
    let declaration = ResolvedMemberRef {
        loader: loader("app"),
        definition: archive_definition(&snapshot, b"p/S.class", &bytes),
        member: SymbolRef::Field {
            owner: JvmBytes(b"p/S".to_vec()),
            name: JvmBytes(b"f".to_vec()),
            descriptor: JvmBytes(b"I".to_vec()),
        },
    };
    let query = DeclarationRefQuery {
        environment: environment.clone(),
        declaration,
        scope: PhysicalScope::SnapshotAll,
        consumers: ConsumerSchema::new(1, [ConsumerKind::Field]),
        max_items: 0,
    };
    let declaration_report = Engine::new()
        .declaration_references(
            std::slice::from_ref(&snapshot),
            &query,
            &mut Budget::new(limits()),
        )
        .expect("a legal query is answered, not raised");
    assert!(declaration_report.reads.is_empty());

    let analysis = MethodAnalysisRequest {
        environment,
        method: PhysicalMethodId {
            owner: archive_definition(&snapshot, b"p/S.class", &bytes),
            name: JvmBytes(b"m".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        },
        stages: vec![AnalysisStage::RawFacts],
    };
    let analysis_report = Engine::new()
        .analyze_method(
            std::slice::from_ref(&snapshot),
            &analysis,
            &mut Budget::new(limits()),
        )
        .expect("a legal request is answered, not raised");
    assert!(
        analysis_report.reads.is_empty(),
        "no header was demanded, so nothing is recorded"
    );
}
