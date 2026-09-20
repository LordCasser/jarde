//! P3 1.3c acceptance: one recovery request reads one member's body and presents it (A16).
//!
//! The property A16 states is about the *request*, not about the artifact: a request that names one
//! member must load that member's body and no other, however many bodies the class declares. The
//! entry it is checked through is the one the CLI calls, [`Engine::recover_method`], and both halves
//! that entry answers with — the method-analysis report of the run and the recovery report of that
//! same run — describe one request, so the run's own usage is where the counting is read
//! (`ClassHeaders`, `MethodBodies`, `CodeBytes` and the construction counts: P2 5.2's vocabulary, and
//! the reason this file is written the way that one is).
//!
//! The premise is not taken on faith: the fixture's own header is read through the reader's
//! `inspect_header` entry first and the `Code` shells it declares are asserted, so "several bodies
//! exist and one attempt was charged" is a statement about the same bytes. The fixture is the
//! committed ECJ 4.6.1 v52 class rather than an assembled one, because the point of the case is that
//! a real class with several bodies is presented one member at a time.
//!
//! What this file does **not** prove, said where it could be mistaken for it: no per-pass
//! construction counter exists on the P2 side, so "the recovery started no second analysis" is
//! argued from the usage dimensions of the one run (one header, one body, one read) and from the
//! entry's own shape (one call to the analysis entry, one presentation of its payload), not from a
//! counter that would say it directly.

use jarde::*;
use std::slice;

/// The committed historical fixture, compiled with an inline `finally` (class-file version 52).
const V52: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

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

/// Names of the members that declare a `Code` attribute, decided from the header shells of a header
/// the reader read through its own entry point.
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

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

/// One caller domain rooted at the fixture's own snapshot, and nothing else: the simplest
/// environment the library's validator accepts without a problem.
fn environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
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

/// `add(II)I` of the fixture, named the way the library derives the identity it has read.
fn add_request(
    snapshot: &ArtifactSnapshot,
    inspected: &EngineHeaderReport,
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment: environment(snapshot),
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: inspected.source.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: bytes(b"add"),
            descriptor: bytes(b"(II)I"),
        },
        stages: AnalysisStage::ALL.to_vec(),
    }
}

/// The usage snapshot of a finished run.
fn usage(report: &MethodAnalysisReport) -> UsageSnapshot {
    match &report.execution {
        ExecutionReport::Complete { usage } => usage.clone(),
        other => panic!("a legal request completes: {other:?}"),
    }
}

#[test]
fn one_recovery_request_reads_one_body_and_presents_that_member() {
    // The premise, through the reader's own header entry: this class declares several members with a
    // body. A request that loaded "the class's bodies" would charge more than one `MethodBodies`
    // attempt and read more than one header; the numbers below are therefore about the request.
    let mut premise_budget = Budget::new(limits());
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(V52.to_vec()), &mut premise_budget)
        .expect("the fixture opens as a standalone CLASS");
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut premise_budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    let declared = members_with_code(&inspected.inspection.header);
    assert!(
        declared.len() >= 3,
        "the fixture declares several bodies, so a request for one of them has others it must not \
         load: {declared:?}"
    );
    assert!(
        declared.iter().any(|name| name.as_slice() == b"add"),
        "the member this case presents is one of them: {declared:?}"
    );

    let request = add_request(&snapshot, &inspected);
    let mut budget = Budget::new(limits());
    let recovered = engine
        .recover_method(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("a legal request is answered, not raised");

    // A16: one header read, one body attempted, whatever the class declares.
    let run = usage(recovered.analysis());
    assert_eq!(
        run.class_headers, 1,
        "the run read the driver method's own class definition once: {declared:?}"
    );
    assert_eq!(
        run.method_bodies,
        1,
        "exactly one body was attempted, not one per member: {} declared one",
        declared.len()
    );
    assert!(run.code_bytes > 0, "and a body really was decoded");
    assert!(run.ir_items > 0 && run.ir_edges > 0, "the IR was built");
    assert_eq!(
        recovered
            .analysis()
            .reads
            .iter()
            .map(|read| read.reason)
            .collect::<Vec<_>>(),
        vec![ReadReason::DriverMethodBody],
        "the one read this request charged is the driver method's body"
    );

    // And the presentation is of that member: its name, its descriptor, its own body.
    let report = recovered.recovery();
    assert_eq!(report.method, "add(II)I");
    // The parameter slots are the payload's own declaration (P3 3.1): the run's one header read
    // located the member and stated its flags and descriptor, so `add(II)I`'s two argument slots are
    // named as the parameters they are — slot 0 is the receiver this member's flags say it has, and
    // the arguments are slots 1 and 2. Nothing here is invented: a body with no debug metadata gets
    // no source name, and the ordinals are the ones the signature states.
    assert!(
        report.text.contains("return arg1 + arg2;"),
        "{}",
        report.text
    );
    assert_eq!(report.representation, Representation::Java);
    assert_eq!(report.quality, Quality::Structured);
    assert_eq!(report.profile.java_release, 8);
    assert_eq!(
        report
            .rules
            .iter()
            .map(|rule| rule.citation())
            .collect::<Vec<_>>(),
        // `straight@1` wrote the body, and `declaration@1` wrote the envelope's declaration line: the
        // class's own name and flags travel with the member's declaration (the declaring-class
        // handoff), so the rule concludes a form for this member instead of refusing the fact as
        // missing. The body's own quality is untouched by it — `Structured` above is the same verdict
        // the body had before the class facts arrived.
        vec!["straight@1".to_string(), "declaration@1".to_string()],
        "the report says which rule produced the text"
    );
    assert!(matches!(report.outcome, RecoveryOutcome::Produced));

    // The presentation read the payload of *that* run: the analysis half of the same answer still
    // reports the one read and the one body the run charged.
    assert_eq!(run.class_headers, usage(recovered.analysis()).class_headers);
    assert_eq!(run.method_bodies, usage(recovered.analysis()).method_bodies);
    assert_eq!(recovered.analysis().body, MethodBodyState::Present);
}

#[test]
fn a_recovery_request_that_asks_for_no_ssa_stops_inside_a_successful_answer() {
    // The schedule is the request's own statement: a request that never asked for the SSA phase has
    // no SSA table, and the presentation states that as a stop in the answer rather than as a raised
    // error or as an empty body. (The CLI's own case for the same property is in
    // `crates/jarde-cli/tests/json_cli.rs`; this is the library half of it.)
    let mut premise_budget = Budget::new(limits());
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(V52.to_vec()), &mut premise_budget)
        .expect("the fixture opens as a standalone CLASS");
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut premise_budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    let mut request = add_request(&snapshot, &inspected);
    request.stages = vec![AnalysisStage::Frame];

    let mut budget = Budget::new(limits());
    let recovered = engine
        .recover_method(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("a legal request is answered, not raised");
    let report = recovered.recovery();
    assert!(
        matches!(
            report.outcome,
            RecoveryOutcome::Stopped(StopReason::IrTableMissing { table: "ssa" })
        ),
        "{:?}",
        report.outcome
    );
    assert_eq!(report.text, "", "a stop produces no artifact");
    assert!(report.regions.is_empty() && report.fallbacks.is_empty());
    assert_eq!(report.representation, Representation::Bytecode);
    assert_eq!(
        report
            .diagnostics
            .first()
            .map(|diagnostic| diagnostic.code.as_str()),
        Some("jre_ir_table_missing")
    );
}
