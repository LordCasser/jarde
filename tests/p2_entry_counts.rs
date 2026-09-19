//! P2 5.2 acceptance: the real entry point reads the bytes it needs, and only those.
//!
//! The slice's counting vocabulary is the budget's own (1.2/3.1): `ClassHeaders` and
//! `MethodBodies` are read **attempts**, charged before the read, and `IrItems`, `IrEdges`,
//! `AnalysisSteps` and `NormalizationClones` are the construction counts of the P2 passes. What
//! this file has to prove through the public entry points is that those counts describe the
//! *request* and never the input:
//!
//! 1. one method analysis attempts exactly one body, however many bodies the class declares. The
//!    premise is not taken on faith: the fixture's own header is read through the reader's
//!    `inspect_header` entry and the `Code` shells it declares are asserted, so "three bodies
//!    exist and one attempt was charged" is a statement about the same bytes.
//! 2. the P1 physical query paths construct no P2 artifact: `class_headers` and `method_bodies`
//!    are zero (the P2 accounting never saw a read), every construction dimension is zero, and
//!    the resolution and dynamic coverage planes stay `NotRequested`. X1's evidence *is* the
//!    decoded instruction stream — it reads `Code` bytes through the reader's own path and
//!    always has — so `CodeBytes` is exactly what separates the two P1 paths: X0's raw pool
//!    probe decodes no instruction at all, X1's consumer scan decodes the instructions of the
//!    classes in scope. What must not appear on either path is a P2 construction.
//! 3. the same request published twice is the same report, field by field, with the wall clock
//!    removed — for a completed run, a `jsr`/`ret` run (where the cloning dimension is really
//!    charged), a stopped run whose report carries a diagnostic, and a rejected environment
//!    whose report carries two diagnostics in declaration order.
//!
//! Where the rest of 5.2's evidence lives: the A17 source guard and its `petgraph` cases are in
//! `tests/p2_contracts.rs` (`A17_IMPORT_TOKENS` and the `petgraph_import` sandbox case), and the
//! "the P1 output did not change" half of the A17 obligation is the field-by-field golden replay
//! of `tests/p1_xref_golden.rs` together with the P1 identity and billing case at the end of
//! `tests/p2_contracts.rs`.
//!
//! One gap is stated here instead of papered over: resolver, CFG, SSA, region and Java-AST
//! construction have no budget dimension of their own, so "the query started none of them" is
//! argued from the dimensions that do exist (all zero), from the planes a started P2 pass would
//! have to touch, and from the source-level guard that keeps the query crate from naming those
//! modules at all. A per-pass construction counter on the P2 side would be the direct evidence;
//! nothing in this file should be read as that counter.

use jarde::*;
use std::slice;

/// The committed historical fixture in two dialects: the same source compiled with an inline
/// `finally` (52) and with a `jsr`/`ret` subroutine (45).
const V52: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");
const V45: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class");

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
        class_headers: 10,
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

/// The same limits with one field replaced, for the stop case below.
fn limits_with(change: impl FnOnce(&mut Limits)) -> Limits {
    let mut limits = limits();
    change(&mut limits);
    limits
}

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

struct Fixture {
    snapshot: ArtifactSnapshot,
    definition: PhysicalDefinitionId,
}

/// Opens one fixture and derives the physical identity the way the engine does.
fn fixture(content: &[u8]) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(content.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(content).to_hex().to_string()),
            length: u64::try_from(content.len()).expect("fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    Fixture {
        snapshot,
        definition,
    }
}

/// One caller domain rooted at the fixture, and nothing else: the simplest environment the
/// validator accepts without a problem.
fn environment(fixture: &Fixture) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Snapshot {
            snapshot: fixture.snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: fixture.snapshot.id().clone(),
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

fn method(fixture: &Fixture, name: &[u8], descriptor: &[u8]) -> PhysicalMethodId {
    PhysicalMethodId {
        owner: fixture.definition.clone(),
        name: bytes(name),
        descriptor: bytes(descriptor),
    }
}

fn analysis_request(
    fixture: &Fixture,
    method: PhysicalMethodId,
    stages: Vec<AnalysisStage>,
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment: environment(fixture),
        method,
        stages,
    }
}

/// Runs one method-analysis request under `limits`, keeping the budget.
fn analyze(
    fixture: &Fixture,
    request: &MethodAnalysisRequest,
    limits: Limits,
) -> (MethodAnalysisReport, Budget) {
    let mut budget = Budget::new(limits);
    let report = Engine::new()
        .analyze_method(slice::from_ref(&fixture.snapshot), request, &mut budget)
        .expect("a legal request is answered, not raised");
    (report, budget)
}

/// State of one stage of a report.
fn stage(report: &MethodAnalysisReport, stage: AnalysisStage) -> StageState {
    report
        .stages
        .iter()
        .find(|result| result.stage == stage)
        .unwrap_or_else(|| panic!("{stage:?} is scheduled"))
        .state
        .clone()
}

/// Every scheduled stage of a report, in published order.
fn stage_states(report: &MethodAnalysisReport) -> Vec<StageState> {
    report
        .stages
        .iter()
        .map(|result| result.state.clone())
        .collect()
}

fn diagnostic_codes(report: &MethodAnalysisReport) -> Vec<&str> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

/// Names of the members that declare a `Code` attribute, decided from the header shells of a
/// header the reader read through its own entry point.
fn members_with_code(header: &ClassHeader) -> Vec<Vec<u8>> {
    header
        .methods
        .iter()
        .filter(|member| {
            member
                .attributes
                .iter()
                .any(|shell| shell.name.raw().0.as_slice() == b"Code")
        })
        .map(|member| member.name.raw().0.clone())
        .collect()
}

/// One usage snapshot with the wall clock removed: the comparison form of two reads of one budget.
///
/// `elapsed_millis` is a measurement, not a charge: [`Budget::usage`] takes it again on every
/// read, so the snapshot a report published and a later read of the same budget may legitimately
/// differ by a millisecond while every counted dimension is identical. Nothing else is dropped.
fn counted_usage(usage: &UsageSnapshot) -> UsageSnapshot {
    UsageSnapshot {
        elapsed_millis: 0,
        ..usage.clone()
    }
}

/// The report as JSON with the wall clock removed: the one field two runs of the same request
/// may legitimately differ in.
fn without_elapsed(report: &MethodAnalysisReport) -> serde_json::Value {
    fn strip(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.remove("elapsed_millis");
                for child in map.values_mut() {
                    strip(child);
                }
            }
            serde_json::Value::Array(items) => {
                for item in items {
                    strip(item);
                }
            }
            _ => {}
        }
    }
    let mut value = serde_json::to_value(report).expect("a report serializes");
    strip(&mut value);
    value
}

/// The counted BCI range a coverage plane scanned, as `(start, end)`.
fn bci_ranges(ranges: &[CoverageRange]) -> Vec<(u64, u64)> {
    ranges
        .iter()
        .filter(|range| range.label == "method_code_bci")
        .map(|range| (range.start, range.end))
        .collect()
}

// ---------------------------------------------------------------------------
// Synthetic classes
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Pool {
    bytes: Vec<u8>,
    count: u16,
}

impl Pool {
    fn push(&mut self, entry: &[u8]) -> u16 {
        self.bytes.extend_from_slice(entry);
        self.count += 1;
        self.count
    }

    fn utf8(&mut self, value: &[u8]) -> u16 {
        let mut entry = vec![1];
        entry.extend_from_slice(
            &u16::try_from(value.len())
                .expect("fixture name fits u16")
                .to_be_bytes(),
        );
        entry.extend_from_slice(value);
        self.push(&entry)
    }

    fn class(&mut self, name: u16) -> u16 {
        let mut entry = vec![7];
        entry.extend_from_slice(&name.to_be_bytes());
        self.push(&entry)
    }
}

/// One method of a synthetic class: access flags, raw name, raw descriptor, and the `Code`
/// bytes it declares (no `Code` attribute at all when absent).
type SyntheticMethod<'a> = (u16, &'a [u8], &'a [u8], Option<Vec<u8>>);

/// A structurally valid class file (`major`) with the given methods; a method without code has
/// no `Code` attribute at all.
fn class_bytes(this_class: &[u8], major: u16, methods: &[SyntheticMethod<'_>]) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_name = pool.utf8(this_class);
    let this_class_index = pool.class(this_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let code_name = pool.utf8(b"Code");
    let mut entries = Vec::new();
    for (access, name, descriptor, code) in methods {
        let name_index = pool.utf8(name);
        let descriptor_index = pool.utf8(descriptor);
        let mut method = Vec::new();
        method.extend_from_slice(&access.to_be_bytes());
        method.extend_from_slice(&name_index.to_be_bytes());
        method.extend_from_slice(&descriptor_index.to_be_bytes());
        match code {
            Some(code) => {
                let mut content = Vec::new();
                content.extend_from_slice(&1_u16.to_be_bytes()); // max_stack
                content.extend_from_slice(&1_u16.to_be_bytes()); // max_locals
                content.extend_from_slice(
                    &u32::try_from(code.len())
                        .expect("fixture code fits u32")
                        .to_be_bytes(),
                );
                content.extend_from_slice(code);
                content.extend_from_slice(&0_u16.to_be_bytes()); // exception table
                content.extend_from_slice(&0_u16.to_be_bytes()); // attributes
                method.extend_from_slice(&1_u16.to_be_bytes()); // attributes
                method.extend_from_slice(&code_name.to_be_bytes());
                method.extend_from_slice(
                    &u32::try_from(content.len())
                        .expect("fixture code content fits u32")
                        .to_be_bytes(),
                );
                method.extend_from_slice(&content);
            }
            None => method.extend_from_slice(&0_u16.to_be_bytes()),
        }
        entries.push(method);
    }

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    bytes.extend_from_slice(&0_u16.to_be_bytes());
    bytes.extend_from_slice(&major.to_be_bytes());
    bytes.extend_from_slice(&(pool.count + 1).to_be_bytes());
    bytes.extend_from_slice(&pool.bytes);
    bytes.extend_from_slice(&0x0021_u16.to_be_bytes()); // ACC_PUBLIC | ACC_SUPER
    bytes.extend_from_slice(&this_class_index.to_be_bytes());
    bytes.extend_from_slice(&object_class.to_be_bytes());
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // interfaces
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // fields
    bytes.extend_from_slice(
        &u16::try_from(entries.len())
            .expect("fixture method count fits u16")
            .to_be_bytes(),
    );
    for method in entries {
        bytes.extend_from_slice(&method);
    }
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // attributes
    bytes
}

/// The class of the first case: three members with a body and one that declares none.
///
/// The three bodies have different lengths on purpose — 1, 3 and 2 instruction bytes — so a run
/// that read "the class's bodies" instead of the driver method's body cannot pass by accident:
/// the charged `CodeBytes` names exactly which bodies were decoded.
///
/// `0xb1` is `return`, `0x00` is `nop`, `0x04`/`0xac` are `iconst_1`/`ireturn`.
fn multi_body_class() -> Vec<u8> {
    class_bytes(
        b"p/Multi",
        52,
        &[
            (0x0001, b"first", b"()V", Some(vec![0xb1])),
            (0x0001, b"second", b"()V", Some(vec![0x00, 0x00, 0xb1])),
            (0x0001, b"target", b"()I", Some(vec![0x04, 0xac])),
            (0x0401, b"declaredOnly", b"()V", None),
        ],
    )
}

// ---------------------------------------------------------------------------
// One method, one body
// ---------------------------------------------------------------------------

#[test]
fn one_method_analysis_attempts_one_body_however_many_the_class_declares() {
    // The premise, read through the reader's own header entry: this class declares four members
    // and three of them carry a `Code` attribute. A run that loaded "the class's bodies" would
    // charge three `MethodBodies` attempts and the sum of the three bodies' instruction bytes;
    // the numbers below are therefore about the request, not about the input.
    let fixture = fixture(&multi_body_class());
    let mut premise_budget = Budget::new(limits());
    let inspected = Engine::new()
        .inspect_header(
            &fixture.snapshot,
            ClassTarget::Root,
            &mut premise_budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    let header = &inspected.inspection.header;
    assert_eq!(header.methods.len(), 4, "the fixture declares four members");
    assert_eq!(
        members_with_code(header),
        vec![b"first".to_vec(), b"second".to_vec(), b"target".to_vec()],
        "three members really declare a body, so each request below has two unrelated bodies \
         it must not load"
    );
    assert_eq!(
        header.methods[3].access_flags & 0x0400,
        0x0400,
        "the fourth member declares itself abstract and carries no `Code` attribute"
    );

    // Every member with a body is analyzed by its own request, and each request pays for exactly
    // one body and exactly that body's instruction bytes — whichever member it names.
    for (name, descriptor, code_bytes) in [
        (&b"first"[..], &b"()V"[..], 1_u64),
        (&b"second"[..], &b"()V"[..], 3),
        (&b"target"[..], &b"()I"[..], 2),
    ] {
        let request = analysis_request(
            &fixture,
            method(&fixture, name, descriptor),
            vec![AnalysisStage::Ssa],
        );
        let (report, budget) = analyze(&fixture, &request, limits());
        let subject = String::from_utf8_lossy(name);

        assert_eq!(
            report.method,
            method(&fixture, name, descriptor),
            "the report answers about the member it was asked about"
        );
        assert_eq!(
            stage_states(&report),
            vec![StageState::Completed; 6],
            "`{subject}` is a complete body: the whole pipeline completed and no stage stopped"
        );
        assert_eq!(report.body, MethodBodyState::Present);
        assert_eq!(
            budget.usage().method_bodies,
            1,
            "`{subject}`: one body attempt for the driver method, not one per body the class \
             declares"
        );
        assert_eq!(
            budget.usage().class_headers,
            1,
            "`{subject}`: one header read — the driver method's own class"
        );
        assert_eq!(
            budget.usage().code_bytes,
            code_bytes,
            "`{subject}`: exactly this body's instruction bytes were decoded; the other two \
             bodies of the class were never read"
        );
        assert_eq!(
            report.reads.len(),
            1,
            "`{subject}`: a method-analysis request performs one read, the driver method's class"
        );
        assert!(
            report
                .reads
                .iter()
                .all(|read| read.reason == ReadReason::DriverMethodBody),
            "`{subject}`: the only read reason a method-analysis request publishes is the driver \
             method's own body, and no other reason appears: {:?}",
            report.reads
        );
        assert_eq!(
            report.reads[0].definition, fixture.definition,
            "`{subject}`: the read is the target class's own definition, never another class's"
        );
    }

    // A member whose declaration carries no `Code` attribute is not attempted even when the
    // request names it: the body plane states the declaration's own fact, and the three bodies
    // the class does hold are no reason to read a fourth one.
    let declared_only = analysis_request(
        &fixture,
        method(&fixture, b"declaredOnly", b"()V"),
        vec![AnalysisStage::RawFacts],
    );
    let (report, budget) = analyze(&fixture, &declared_only, limits());
    assert_eq!(
        report.body,
        MethodBodyState::DeclaredWithoutBody {
            no_body_kind: NoBodyKind::Abstract
        }
    );
    assert_eq!(
        stage(&report, AnalysisStage::RawFacts),
        StageState::NotPerformed,
        "no pass can run on a member that declares no body"
    );
    assert_eq!(budget.usage().method_bodies, 0);
    assert_eq!(budget.usage().code_bytes, 0);
    assert_eq!(
        report.reads.len(),
        1,
        "the header read happened: the declaration's own fact is what stops the run"
    );
}

// ---------------------------------------------------------------------------
// The P1 physical query paths construct no P2 artifact
// ---------------------------------------------------------------------------

/// The P2 construction dimensions a physical query must never charge, plus the two read
/// dimensions of the P2 accounting plane.
fn assert_no_p2_construction(usage: &UsageSnapshot, label: &str) {
    for dimension in [
        CountedBudgetDimension::IrItems,
        CountedBudgetDimension::IrEdges,
        CountedBudgetDimension::AnalysisSteps,
        CountedBudgetDimension::NormalizationClones,
    ] {
        assert_eq!(
            usage.counted_usage(dimension),
            0,
            "{label}: {dimension:?} counts a P2 construction, and a physical query starts no P2 \
             pass"
        );
    }
    assert_eq!(
        usage.class_headers, 0,
        "{label}: the P2 header accounting saw no read"
    );
    assert_eq!(
        usage.method_bodies, 0,
        "{label}: the P2 body accounting saw no attempt"
    );
}

/// The one query both P1-path cases send: the `java/lang/Object.<init>()V` invocation of the
/// fixture's own constructor.
///
/// The fixture really holds that use site — the `<init>` body starts with
/// `invokespecial java/lang/Object.<init>()V` at BCI 1 — so a run that publishes nothing is a
/// run that did not scan, not a scan that found nothing.
fn query_request(snapshot: &ArtifactSnapshot, relation: QueryRelation) -> QueryRequest {
    QueryRequest {
        relation,
        target: QueryTarget::Symbol {
            value: SymbolRef::Method {
                owner: bytes(b"java/lang/Object"),
                name: bytes(b"<init>"),
                descriptor: bytes(b"()V"),
            },
        },
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
        max_items: 0,
        cursor: None,
    }
}

fn run_query(fixture: &Fixture, relation: QueryRelation, limits: Limits) -> (QueryReport, Budget) {
    let request = query_request(&fixture.snapshot, relation);
    let mut budget = Budget::new(limits);
    let report = Engine::new()
        .query(&fixture.snapshot, &request, &mut budget)
        .expect("the fixture query runs");
    (report, budget)
}

#[test]
fn the_x1_consumer_scan_constructs_no_ir() {
    // X1 is P1's structural consumer scan: it reads the class and decodes the instruction
    // stream, which is P1's own evidence and always was. What 5.2 has to prove is that it starts
    // none of P2's construction and none of P2's read accounting — the resolver, the CFG, the
    // SSA, the region pass and the Java AST are all behind the crate boundary the A17 guard of
    // `tests/p2_contracts.rs` enforces, and this is the behavioural half of that obligation.
    let fixture = fixture(V52);
    let (report, budget) = run_query(&fixture, QueryRelation::MentionsSymbol, limits());

    // The scan really ran: the invocation is published with its own coordinates.
    assert_eq!(report.analysis, QueryAnalysis::Performed);
    assert_eq!(report.items.len(), 1, "the `<init>` body really invokes it");
    let item = &report.items[0];
    assert_eq!(item.derivation, XrefDerivation::StructuralConsumer);
    assert_eq!(item.consumer, Some(ConsumerKind::Invocation));
    assert_eq!(item.operation, XrefOperation::InvokeSpecial);
    assert_eq!(item.certainty, XrefCertainty::Exact);
    assert_eq!(item.evidence.bci, Some(1));

    let usage = budget.usage();
    assert!(
        usage.class_bytes > 0,
        "the class was really read; the zeroes below are not a query that did nothing"
    );
    assert_no_p2_construction(&usage, "X1");
    assert!(
        usage.code_bytes > 0,
        "X1's evidence is the decoded instruction stream, read through the reader's own path: \
         `method_bodies` is what P2 must not charge here, and it is zero"
    );
    // The planes a started P2 pass would have to touch: the query never asked for runtime
    // resolution and never ran a dynamic analysis, so both stay `NotRequested`.
    assert_eq!(
        report.coverage.dimensions.runtime_resolution.state,
        CoverageState::NotRequested
    );
    assert_eq!(
        report.coverage.dimensions.dynamic_analysis.state,
        CoverageState::NotRequested
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
}

#[test]
fn the_x0_pool_probe_constructs_no_ir_and_decodes_no_body() {
    // X0 answers from the constant pool alone. The stronger claim of the two P1 paths holds
    // here: not one instruction byte is decoded, so `code_bytes` is zero as well — the probe
    // reads the class's pool and stops, and the three `Code` attributes the class holds are
    // never even opened.
    let fixture = fixture(V52);
    let (x0, x0_budget) = run_query(&fixture, QueryRelation::ConstantPoolContains, limits());
    let (x1, x1_budget) = run_query(&fixture, QueryRelation::MentionsSymbol, limits());

    assert_eq!(x0.analysis, QueryAnalysis::Performed);
    assert_eq!(x0.items.len(), 1);
    assert!(
        x0.items.iter().all(
            |item| item.derivation == XrefDerivation::ConstantPoolCandidate
                && item.operation == XrefOperation::ConstantPoolEntry
                && item.consumer.is_none()
        ),
        "every X0 item is a raw pool candidate: {:?}",
        x0.items
    );

    let usage = x0_budget.usage();
    assert!(usage.class_bytes > 0, "the class was really read");
    assert_no_p2_construction(&usage, "X0");
    assert_eq!(
        usage.code_bytes, 0,
        "X0 determines the answer from the constant pool: it decodes no method body at all"
    );
    // The three `Code` attributes X1 opens are exactly the attribute bytes X0 never reads, so
    // the two paths differ by the bodies and by nothing else here.
    assert!(
        usage.attribute_bytes < x1_budget.usage().attribute_bytes,
        "X0 reads fewer attribute bytes than X1: {} < {}",
        usage.attribute_bytes,
        x1_budget.usage().attribute_bytes
    );
    assert_eq!(x1.relation, QueryRelation::MentionsSymbol);
    assert_eq!(x1.items.len(), 1);
}

// ---------------------------------------------------------------------------
// The same request twice
// ---------------------------------------------------------------------------

/// Runs one request twice on two fresh budgets and asserts the two reports and the two usage
/// snapshots agree field by field once the one measured field is removed.
///
/// The comparison is the serialized report — every plane, the order of `stages`, `reads`,
/// `diagnostics` and every coverage range included — with `elapsed_millis` stripped, exactly
/// the normalization the P1 golden replays use. The returned report is the first run's, so the
/// caller can assert the case's own content.
fn assert_repeats_field_by_field(
    fixture: &Fixture,
    request: &MethodAnalysisRequest,
    limits: Limits,
) -> (MethodAnalysisReport, UsageSnapshot) {
    let (first, first_budget) = analyze(fixture, request, limits.clone());
    let (second, second_budget) = analyze(fixture, request, limits);

    assert_eq!(
        without_elapsed(&first),
        without_elapsed(&second),
        "two runs of one request are the same report, including every published order"
    );
    for dimension in CountedBudgetDimension::ALL {
        assert_eq!(
            first_budget.usage().counted_usage(dimension),
            second_budget.usage().counted_usage(dimension),
            "{dimension:?} is charged deterministically"
        );
    }
    assert_eq!(
        counted_usage(&first_budget.usage()),
        counted_usage(&second_budget.usage())
    );
    assert_eq!(
        stage_states(&first),
        stage_states(&second),
        "the stage planes agree in order and in content"
    );
    (first, first_budget.usage())
}

#[test]
fn the_same_method_analysis_repeats_field_by_field() {
    // A completed run over the whole pipeline: every stage `Completed`, the BCI plane complete,
    // one read and no diagnostic.
    let v52 = fixture(V52);
    let complete = analysis_request(
        &v52,
        method(&v52, b"finallyPath", b"(I)I"),
        vec![AnalysisStage::Ssa],
    );
    let (report, usage) = assert_repeats_field_by_field(&v52, &complete, limits());
    assert_eq!(
        stage_states(&report),
        vec![StageState::Completed; 6],
        "the whole pipeline completed"
    );
    assert_eq!(
        bci_ranges(&report.coverage.artifact_structural.scanned),
        vec![(0, 15)]
    );
    assert!(diagnostic_codes(&report).is_empty());
    assert_eq!(usage.method_bodies, 1);
    assert_eq!(usage.code_bytes, 15);

    // A `jsr`/`ret` run: the dialect where call contexts and the clone normalization really do
    // work, so the run that has to repeat identically is not one that charged nothing.
    let v45 = fixture(V45);
    let subroutine = analysis_request(
        &v45,
        method(&v45, b"finallyPath", b"(I)I"),
        vec![AnalysisStage::Ssa],
    );
    let (report, usage) = assert_repeats_field_by_field(&v45, &subroutine, limits());
    assert_eq!(stage_states(&report), vec![StageState::Completed; 6]);
    assert!(
        usage.normalization_clones > 0,
        "the 45 dialect really clones: {}",
        usage.normalization_clones
    );

    // A run that stopped inside the body's decode: partial stages, a diagnostic and a coverage
    // plane that names what it skipped all have to repeat as published.
    let truncated = analysis_request(
        &v52,
        method(&v52, b"add", b"(II)I"),
        vec![AnalysisStage::Ssa],
    );
    let (report, usage) = assert_repeats_field_by_field(
        &v52,
        &truncated,
        limits_with(|limits| limits.code_bytes = 2),
    );
    assert_eq!(stage_states(&report), vec![StageState::Partial; 6]);
    assert_eq!(
        diagnostic_codes(&report),
        vec!["classfile_bytecode_budget_exceeded"]
    );
    assert!(!report.coverage.artifact_structural.skipped.is_empty());
    assert_eq!(usage.code_bytes, 2);

    // A rejected environment: the report never ran a pass, states its problems in declaration
    // order and publishes the two diagnostics in that same order.
    let mut rejected = environment(&v52);
    rejected.domains.clear();
    let request = MethodAnalysisRequest {
        environment: rejected,
        method: method(&v52, b"finallyPath", b"(I)I"),
        stages: vec![AnalysisStage::RawFacts],
    };
    let (report, usage) = assert_repeats_field_by_field(&v52, &request, limits());
    assert_eq!(
        diagnostic_codes(&report),
        vec!["duplicate_loader", "method_analysis_not_implemented"],
        "the problem of the environment comes first, then the capability that did not run"
    );
    assert_eq!(stage_states(&report), vec![StageState::NotPerformed]);
    assert!(report.reads.is_empty(), "no byte was read");
    assert_eq!(counted_usage(&usage), UsageSnapshot::default());
}

// ---------------------------------------------------------------------------
// Ordering, as the reports publish it
// ---------------------------------------------------------------------------

#[test]
fn every_published_sequence_is_in_its_own_order() {
    // 5.2's determinism clause is also 3.1's falsifiable point: the report's order-bearing
    // planes are the phase order of `stages` and the ascending scan of the coverage ranges, and
    // both are asserted here instead of being left implicit in the repeat comparison.
    let fixture = fixture(V52);
    let request = analysis_request(
        &fixture,
        method(&fixture, b"finallyPath", b"(I)I"),
        vec![AnalysisStage::Ssa],
    );
    let (report, _) = analyze(&fixture, &request, limits());

    assert_eq!(
        report
            .stages
            .iter()
            .map(|result| result.stage)
            .collect::<Vec<_>>(),
        AnalysisStage::ALL.to_vec(),
        "the stage results are the fixed pass-table order, not an iteration order"
    );
    for dimension in [
        &report.coverage.artifact_structural,
        &report.coverage.runtime_resolution,
        &report.coverage.dynamic_analysis,
    ] {
        assert!(
            dimension
                .scanned
                .windows(2)
                .all(|pair| pair[0].start <= pair[1].start),
            "coverage ranges are published in ascending order: {:?}",
            dimension.scanned
        );
        assert!(
            dimension
                .skipped
                .windows(2)
                .all(|pair| pair[0].start <= pair[1].start),
            "skipped ranges are published in ascending order: {:?}",
            dimension.skipped
        );
    }
}
