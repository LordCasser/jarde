//! D1 of change `add-demand-driven-core-results` (tasks 2.1–2.4): the recovery **evidence
//! selection** as the entry points state it.
//!
//! What this file has to prove through the public surface:
//!
//! 1. **the same input, two selections**: the ordinary recovery and the full-evidence recovery
//!    present the same text, the same planes and the same gaps, and differ only in which optional
//!    records exist — with the ordinary one materializing none of them;
//! 2. **the status list is four statements**: `NotRequested`, `Complete` (a legal empty result
//!    included), `Partial` (the delivered prefix of a phase that stopped) and `NotPerformed` (a
//!    selected category no run reached) are told apart, and the list agrees with the payload;
//! 3. **the driver range selects evidence**: a range restricts the positional records to the ones it
//!    intersects, and it never selects a callee's own bytecode index;
//! 4. **a selection this entry cannot answer is refused**: a reversed range, a range past the decoded
//!    instructions, a boundary the body does not have, a range without a category, a body with no
//!    `Code` and a category this entry does not materialize are each answered with their own code —
//!    never silently widened to a full-evidence delivery;
//! 5. **closing the detail keeps the answer**: with no rule records, a refused region is still
//!    quoted statement by statement, its gap is still stated, and the run's stop is still a stop;
//! 6. **the default is one statement**: `Engine::recover_method` and
//!    `Engine::recover_method_with_evidence(…, essential, …)` answer the same report.
//!
//! Everything below reads the library's own types: no counter, no report field and no test-only
//! port is named for the *selection* itself. The construction counts of the unselected categories
//! are taken inside `jarde-java`, where the records are built (`crates/jarde-java/src/demand_counts.rs`
//! and its `evidence` gate): a record built and dropped before publication leaves nothing here to
//! see, which is exactly why the payload alone cannot prove it.

use jarde::*;
use std::slice;

/// A class with eight members, every one of them declaring a body: the same fixture the change's
/// D0/D2 gates count on.
const SCOPE: &[u8] = include_bytes!("fixtures/p3-scope/v8/Scope.class");

/// A body this slice refuses in the middle: `getstatic External.value; checkcast String; areturn`
/// is quoted rather than presented, so its refusal and its quality are the case's own.
const REFUSED: &[u8] = include_bytes!("fixtures/p3-refused-cast/v8/RefusedCast.class");

/// A class with a member that declares no `Code` at all: the case where a request for a *position in
/// a body* has no body to read.
const SHAPE: &[u8] = include_bytes!("fixtures/p3-declaration/v8/Shape.class");

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 24,
        archive_entries: 1024,
        entry_bytes: 1 << 24,
        read_bytes: 1 << 24,
        class_bytes: 1 << 24,
        attribute_bytes: 1 << 24,
        code_bytes: 1 << 24,
        result_items: 4096,
        output_bytes: 1 << 24,
        class_headers: 64,
        method_bodies: 64,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 4,
        dependency_depth: 8,
        elapsed_millis: 60_000,
    }
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    let mut budget = Budget::new(limits());
    ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("the committed fixture is a readable class file")
}

/// The one definition the snapshot holds, read through the public listing entry.
fn definition_of(engine: &Engine, snapshot: &ArtifactSnapshot) -> PhysicalDefinitionId {
    let mut budget = Budget::new(limits());
    engine
        .list_class_declarations(snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the fixture's one class is listed")
        .items
        .first()
        .expect("the standalone snapshot holds one class")
        .definition
        .clone()
}

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

fn request(
    snapshot: &ArtifactSnapshot,
    definition: &PhysicalDefinitionId,
    name: &[u8],
    descriptor: &[u8],
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment: environment(snapshot),
        method: PhysicalMethodId {
            owner: definition.clone(),
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    }
}

/// One recovery under one selection and one budget, with the budget the caller keeps.
fn recover_with(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    request: &MethodAnalysisRequest,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> RecoveredMethod {
    engine
        .recover_method_with_evidence(slice::from_ref(snapshot), request, evidence, budget)
        .expect("a legal request is answered, not raised")
}

/// The decisions and the gaps a selection may not move: what the artifact is, what it holds, which
/// fallbacks it kept, and every diagnostic this run states — codes **and** sentences, so a summary
/// that read its own figures off a table nobody selected would be caught here.
fn decisions(report: &RecoveryReport) -> String {
    format!(
        "{:?}/{:?}/{:?}/{:?}/{:?}/{:?}",
        report.representation,
        report.quality,
        report.syntax_status,
        report.content,
        report.fallbacks,
        report
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
            .collect::<Vec<_>>(),
    )
}

/// Whether the status list says what the report's own payload holds.
///
/// The mapping from category to payload is stated **again** here on purpose: a status list checked
/// against the library's own idea of the mapping would agree with itself, and the contract this case
/// exists for is "the list says what a caller receives".
fn evidence_agrees(report: &RecoveryReport) -> bool {
    let held = |kind: RecoveryEvidenceKind| match kind {
        RecoveryEvidenceKind::SourceMap => report.source_map.len(),
        RecoveryEvidenceKind::RegionDetails => report.regions.len(),
        RecoveryEvidenceKind::RuleDetails => {
            report.lambdas.len()
                + report.concats.len()
                + report.accessors.len()
                + report.bridges.len()
                + report.news.len()
                + report.fields.len()
                + report.enum_switches.len()
                + usize::from(report.init.is_some())
                + usize::from(report.declaration.is_some())
        }
        RecoveryEvidenceKind::NameDetails => report.aliased_names.len(),
        RecoveryEvidenceKind::ReadDetails => 0,
    };
    RecoveryEvidenceKind::ALL.into_iter().all(|kind| {
        let held = held(kind) as u64;
        match report.evidence.state(kind) {
            EvidenceState::NotRequested | EvidenceState::NotPerformed => held == 0,
            EvidenceState::Complete => true,
            EvidenceState::Partial { delivered } => held == delivered,
        }
    })
}

/// Every optional owning record of one report, as a number: what the selection decides the presence
/// of, and nothing else.
fn optional_records(report: &RecoveryReport) -> usize {
    report.regions.len()
        + report.lambdas.len()
        + report.concats.len()
        + report.accessors.len()
        + report.bridges.len()
        + report.news.len()
        + report.fields.len()
        + report.enum_switches.len()
        + usize::from(report.init.is_some())
        + usize::from(report.declaration.is_some())
        + report.source_map.len()
        + report.aliased_names.len()
        + report.rules.len()
}

/// The ordinary recovery and the full-evidence one present the same body, and only the second
/// materializes the optional records.
#[test]
fn the_same_request_under_either_selection_presents_the_same_body() {
    let engine = Engine::new();
    let scope = open(SCOPE);
    let definition = definition_of(&engine, &scope);
    let names: [(Vec<u8>, Vec<u8>); 3] = [
        (b"receiver".to_vec(), b"(J)J".to_vec()),
        (b"simple".to_vec(), b"()I".to_vec()),
        (b"_new".to_vec(), b"(J)V".to_vec()),
    ];
    let mut seen = 0usize;
    for (name, descriptor) in &names {
        let request = request(&scope, &definition, name, descriptor);
        let mut plain_budget = Budget::new(limits());
        let plain = recover_with(
            &engine,
            &scope,
            &request,
            &RecoveryEvidenceRequest::essential(),
            &mut plain_budget,
        );
        let mut full_budget = Budget::new(limits());
        let full = recover_with(
            &engine,
            &scope,
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut full_budget,
        );
        // A member the analysis cannot present is answered with a stop in both selections; the two
        // runs still have to agree about it.
        assert_eq!(
            full.recovery().produced(),
            plain.recovery().produced(),
            "{}: the selection does not decide whether a body is presented",
            String::from_utf8_lossy(name)
        );
        assert_eq!(full.recovery().text, plain.recovery().text);
        assert_eq!(decisions(full.recovery()), decisions(plain.recovery()));
        assert_eq!(full.facts().method().name(), plain.facts().method().name());
        for kind in RecoveryEvidenceKind::SUPPORTED {
            assert_eq!(
                plain.recovery().evidence.state(kind),
                EvidenceState::NotRequested,
                "the ordinary recovery asks for nothing optional"
            );
            if plain.recovery().produced() {
                assert_eq!(
                    full.recovery().evidence.state(kind),
                    EvidenceState::Complete,
                    "and the full one delivers every category it asked for"
                );
            } else {
                assert_eq!(
                    full.recovery().evidence.state(kind),
                    EvidenceState::NotPerformed,
                    "a run that never reached the evidence phase states so"
                );
            }
        }
        assert_eq!(
            optional_records(plain.recovery()),
            0,
            "the ordinary report holds no optional record at all"
        );
        assert!(
            evidence_agrees(plain.recovery()),
            "and the status list says exactly that"
        );
        assert!(
            evidence_agrees(full.recovery()),
            "the full report's list agrees with its own payload"
        );
        if optional_records(full.recovery()) > 0 {
            seen += 1;
        }
    }
    assert!(
        seen > 0,
        "at least one of the presented members really has optional evidence to materialize"
    );
}

/// The default is one statement: the entry the ordinary caller reaches and the entry that states the
/// essential selection answer the same report.
///
/// This is what "an ordinary recovery defaults to Essential" means where it can be checked: not a
/// convention in a document, but the same answer from both spellings — so an implementation that
/// changed the default to the full selection would answer differently here.
#[test]
fn the_ordinary_entry_and_the_stated_essential_selection_answer_the_same_report() {
    let engine = Engine::new();
    let scope = open(SCOPE);
    let definition = definition_of(&engine, &scope);
    let request = request(&scope, &definition, b"simple", b"()I");
    let mut default_budget = Budget::new(limits());
    let by_default = engine
        .recover_method(slice::from_ref(&scope), &request, &mut default_budget)
        .expect("a legal request is answered");
    let mut stated_budget = Budget::new(limits());
    let stated = recover_with(
        &engine,
        &scope,
        &request,
        &RecoveryEvidenceRequest::essential(),
        &mut stated_budget,
    );
    assert_eq!(
        serde_json_value(&by_default),
        serde_json_value(&stated),
        "the default entry is the essential selection, field by field"
    );
}

/// One report as a document without its wall-clock readings, so two runs of the same request are
/// compared by what they *state*.
fn serde_json_value(recovered: &RecoveredMethod) -> String {
    let document = serde_json::to_string(recovered).expect("a recovered method serializes");
    strip_elapsed(&document)
}

/// Every `"elapsed_millis": N` reading, as the text without its numbers.
fn strip_elapsed(document: &str) -> String {
    let mut stripped = String::with_capacity(document.len());
    let mut rest = document;
    while let Some(at) = rest.find("\"elapsed_millis\":") {
        stripped.push_str(&rest[..at]);
        stripped.push_str("\"elapsed_millis\":0");
        let after = &rest[at + "\"elapsed_millis\":".len()..];
        let end = after
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(after.len());
        rest = &after[end..];
    }
    stripped.push_str(rest);
    stripped
}

/// A driver range selects the records it intersects — and only those: the region records of a ranged
/// request are exactly the full ones the range intersects, in the same order.
#[test]
fn the_driver_range_selects_the_records_it_intersects() {
    let engine = Engine::new();
    let scope = open(SCOPE);
    let definition = definition_of(&engine, &scope);
    let request = request(&scope, &definition, b"receiver", b"(J)J");
    let mut full_budget = Budget::new(limits());
    let full = recover_with(
        &engine,
        &scope,
        &request,
        &RecoveryEvidenceRequest::all(),
        &mut full_budget,
    );
    let report = full.recovery();
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        !report.regions.is_empty(),
        "the fixture's member has regions to select"
    );
    // The one-block range of the first region's own start: a record is kept when one of the
    // positions it states is inside the range.
    let first = report.regions[0].bci;
    let range = BytecodeRange::new(first, first + 1);
    let selection = RecoveryEvidenceRequest::all().with_driver_bci_range(range);
    let mut ranged_budget = Budget::new(limits());
    let ranged = recover_with(&engine, &scope, &request, &selection, &mut ranged_budget);
    let ranged_report = ranged.recovery();
    assert_eq!(
        ranged_report.text, report.text,
        "a range selects evidence, never analysis"
    );
    let expected: Vec<u32> = report
        .regions
        .iter()
        .filter(|region| range.intersects_any(region.blocks.iter().copied()))
        .map(|region| region.bci)
        .collect();
    assert!(
        !expected.is_empty(),
        "the range really selects a region of this body"
    );
    assert_eq!(
        ranged_report
            .regions
            .iter()
            .map(|region| region.bci)
            .collect::<Vec<u32>>(),
        expected,
        "the local result is the full evidence projected onto the range"
    );
    assert!(
        ranged_report.source_map.len() <= report.source_map.len(),
        "and the segment table cannot grow by selecting less"
    );
    assert_eq!(
        ranged_report.evidence.requested().driver_bci_range(),
        Some(range),
        "the report echoes the effective selection"
    );
}

/// A legal empty range is a selection, not an error and not a stop: the category is delivered, and
/// what it delivers is nothing.
#[test]
fn a_legal_empty_range_is_a_complete_empty_selection() {
    let engine = Engine::new();
    let scope = open(SCOPE);
    let definition = definition_of(&engine, &scope);
    let request = request(&scope, &definition, b"simple", b"()I");
    let selection = RecoveryEvidenceRequest::essential()
        .with_kind(RecoveryEvidenceKind::RegionDetails)
        .with_kind(RecoveryEvidenceKind::SourceMap)
        .with_driver_bci_range(BytecodeRange::new(0, 0));
    let mut budget = Budget::new(limits());
    let recovered = recover_with(&engine, &scope, &request, &selection, &mut budget);
    let report = recovered.recovery();
    assert!(report.produced(), "{:?}", report.outcome);
    for kind in [
        RecoveryEvidenceKind::RegionDetails,
        RecoveryEvidenceKind::SourceMap,
    ] {
        assert_eq!(
            report.evidence.state(kind),
            EvidenceState::Complete,
            "{kind:?}"
        );
    }
    assert_eq!(
        report.evidence.state(RecoveryEvidenceKind::RuleDetails),
        EvidenceState::NotRequested
    );
    assert!(
        report.regions.is_empty(),
        "an empty range selects no region"
    );
    assert!(
        report.source_map.is_empty(),
        "and no segment: the table it delivers is empty, not the whole method's"
    );
    assert!(evidence_agrees(report));
}

/// Every selection this entry cannot answer is refused with its own code, and none of them is
/// widened into a full-evidence delivery or answered with an empty one.
#[test]
fn a_selection_this_entry_cannot_answer_is_refused_with_its_own_code() {
    let engine = Engine::new();
    let scope = open(SCOPE);
    let definition = definition_of(&engine, &scope);
    // A member whose body has an instruction wider than one byte, so this case can name a BCI
    // *inside* an instruction — a boundary the body really does not have. The instruction facts are
    // the fixture's own decode, read through the public inspection entry.
    let mut inspection_budget = Budget::new(limits());
    let inspected = engine
        .inspect_header(
            &scope,
            ClassTarget::Root,
            &mut inspection_budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    let target = inspected
        .inspection
        .header
        .methods
        .iter()
        .find_map(|member| {
            let bytecode = engine
                .inspect_method_bytecode(
                    &scope,
                    ClassTarget::Root,
                    MethodSelector {
                        name: member.name.raw().clone(),
                        descriptor: member.descriptor.raw().clone(),
                    },
                    &mut inspection_budget,
                )
                .ok()?;
            bytecode
                .inspection
                .instructions
                .iter()
                .find(|instruction| instruction.width >= 2)
                .map(|instruction| {
                    (
                        member.name.raw().clone(),
                        member.descriptor.raw().clone(),
                        instruction.bci + 1,
                    )
                })
        })
        .expect("the fixture has a member with an instruction wider than one byte");
    let (name, descriptor, interior) = target;
    let request = request(&scope, &definition, &name.0, &descriptor.0);
    let cases: [(&str, RecoveryEvidenceRequest); 5] = [
        (
            "shape",
            RecoveryEvidenceRequest::all().with_driver_bci_range(BytecodeRange::new(4, 1)),
        ),
        (
            "shape",
            RecoveryEvidenceRequest::essential().with_driver_bci_range(BytecodeRange::new(0, 1)),
        ),
        (
            "range",
            RecoveryEvidenceRequest::all().with_driver_bci_range(BytecodeRange::new(0, 100_000)),
        ),
        (
            "range",
            RecoveryEvidenceRequest::all()
                .with_driver_bci_range(BytecodeRange::new(interior, interior + 1)),
        ),
        (
            "kind",
            RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::ReadDetails),
        ),
    ];
    for (expected, selection) in &cases {
        let mut budget = Budget::new(limits());
        let recovered = recover_with(&engine, &scope, &request, selection, &mut budget);
        let report = recovered.recovery();
        let RecoveryOutcome::Stopped(StopReason::EvidenceRefused { code, .. }) = &report.outcome
        else {
            panic!(
                "{selection:?} is refused as a selection: {:?}",
                report.outcome
            );
        };
        assert!(
            code.contains(expected),
            "{selection:?} is refused with its own code, not {code}"
        );
        assert_eq!(report.text, "", "a refused selection presents nothing");
        assert_eq!(report.content, RecoveryContent::NotProduced);
        for kind in selection.kinds() {
            assert_eq!(
                report.evidence.state(kind),
                EvidenceState::NotPerformed,
                "a category the refusal never reached is not an empty result"
            );
        }
        assert!(evidence_agrees(report));
        assert_eq!(
            report.evidence.requested().driver_bci_range(),
            selection.driver_bci_range(),
            "the refusal names the selection it refused"
        );
    }
}

/// A member with no body has no `Code`, and a request that asks for a position in it is refused in
/// the vocabulary the run already states for that: the table is missing, and no range is invented.
#[test]
fn a_range_over_a_body_with_no_code_is_refused_as_a_missing_table() {
    let engine = Engine::new();
    let scope = open(SHAPE);
    let definition = definition_of(&engine, &scope);
    let mut inspection_budget = Budget::new(limits());
    let inspected = engine
        .inspect_header(
            &scope,
            ClassTarget::Root,
            &mut inspection_budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    let declared_without_body = inspected
        .inspection
        .header
        .methods
        .iter()
        .find(|member| {
            !member
                .attributes
                .iter()
                .any(|shell| shell.name.raw().0.as_slice() == b"Code")
        })
        .expect("the fixture declares a member with no `Code`");
    let request = request(
        &scope,
        &definition,
        &declared_without_body.name.raw().0,
        &declared_without_body.descriptor.raw().0,
    );
    // The member is in this snapshot and the request names it, so the *selection* is the only thing
    // that could be refused here — and it is not: the honest answer is that the body the range would
    // be read against does not exist.
    let selection = RecoveryEvidenceRequest::all().with_driver_bci_range(BytecodeRange::new(0, 1));
    let mut budget = Budget::new(limits());
    let recovered = recover_with(&engine, &scope, &request, &selection, &mut budget);
    let report = recovered.recovery();
    assert!(!report.produced(), "{:?}", report.outcome);
    assert!(
        matches!(
            &report.outcome,
            RecoveryOutcome::Stopped(StopReason::IrTableMissing { .. })
        ),
        "a body with no decode is refused as the table it is missing: {:?}",
        report.outcome
    );
    assert_eq!(report.text, "");
    for kind in selection.kinds() {
        assert_eq!(
            report.evidence.state(kind),
            EvidenceState::NotPerformed,
            "a selected category no run reached is not an empty result"
        );
    }
    assert!(evidence_agrees(report));

    // Without the range the same member is the same stop: the selection did not change the answer,
    // it was simply not applied to a body that does not exist.
    let mut plain_budget = Budget::new(limits());
    let plain = engine
        .recover_method(slice::from_ref(&scope), &request, &mut plain_budget)
        .expect("a legal request is answered");
    assert_eq!(
        format!("{:?}", plain.recovery().outcome),
        format!("{:?}", report.outcome)
    );
}

/// A selected category the phase stopped inside states the prefix it delivered, keeps the artifact
/// the run had already committed, and says the run stopped.
#[test]
fn a_selected_category_that_stopped_states_its_prefix() {
    let engine = Engine::new();
    let scope = open(SCOPE);
    let definition = definition_of(&engine, &scope);
    // The member with the most regions: the evidence phase materializes the region records first, so
    // a member with more than one of them is the one a bound can stop inside.
    let full = engine
        .class_source_with_evidence(
            slice::from_ref(&scope),
            &ClassSourceRequest {
                class: ClassRef::Definition {
                    definition: definition.clone(),
                },
                environment: EnvironmentRequest {
                    snapshot: scope.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                    policy: EnvironmentPolicy::SingleClass,
                    profile: RuntimeProfile {
                        java_release: 8,
                        multi_release: MultiReleasePolicy::Disabled,
                        layout: LayoutMode::Generic,
                    },
                    loader: LoaderId("app".to_owned()),
                },
            },
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits()),
        )
        .expect("the class is presented");
    let OperationOutcome::Performed(full) = full else {
        panic!("the identity binds one definition");
    };
    let mut candidates = Vec::new();
    for method in &full.methods {
        if let ClassSourceOutcome::Recovered { report, .. } = &method.outcome
            && report.regions.len() >= 2
        {
            candidates.push((
                method.item.identity.name.0.clone(),
                method.item.identity.descriptor.0.clone(),
                report.regions.len(),
                report.text.len(),
            ));
        }
    }
    let Some((name, descriptor, regions, text_bytes)) = candidates.into_iter().next() else {
        panic!("the fixture has a member with more than one region to stop inside");
    };
    let request = request(&scope, &definition, &name, &descriptor);
    // A bound that funds the artifact and every region record but one: measured from the run's own
    // usage, so the prefix is one record rather than a guess.
    let mut measuring = Budget::new(limits());
    let complete = recover_with(
        &engine,
        &scope,
        &request,
        &RecoveryEvidenceRequest::all(),
        &mut measuring,
    );
    assert!(
        complete.recovery().produced(),
        "{:?}",
        complete.recovery().outcome
    );
    let used = measuring.usage();
    let mut tight = Budget::new(Limits {
        ir_items: used.ir_items.saturating_sub(1),
        ..limits()
    });
    let stopped = recover_with(
        &engine,
        &scope,
        &request,
        &RecoveryEvidenceRequest::all(),
        &mut tight,
    );
    let report = stopped.recovery();
    assert!(
        report.produced(),
        "the artifact was committed before the phase ran: {:?}",
        report.outcome
    );
    assert_eq!(
        report.text.len(),
        text_bytes,
        "and the phase does not touch one byte of it"
    );
    let state = report.evidence.state(RecoveryEvidenceKind::RegionDetails);
    let EvidenceState::Partial { delivered } = state else {
        panic!("the phase stopped inside the region records: {state:?}");
    };
    assert!(delivered >= 1, "{state:?}");
    assert_eq!(
        u64::try_from(report.regions.len()).expect("a small count"),
        delivered
    );
    assert!(
        regions as u64 > delivered,
        "the prefix is shorter than the full delivery ({regions} regions)"
    );
    assert_eq!(
        report.evidence.state(RecoveryEvidenceKind::NameDetails),
        EvidenceState::NotPerformed,
        "a category the stopped phase never reached is not an empty result"
    );
    assert!(!matches!(
        report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(evidence_agrees(report));
}

/// Closing the rule records closes the *records*, not the answer: a refused region is still quoted
/// statement by statement in the text, the gap the run states is still stated, an artifact that is
/// explanation alone is still an explanation, and the run that stopped is still a stop.
///
/// The fixture's members are all presented here, and the cases are *found* in their reports rather
/// than assumed: which member quotes a refusal, which one is explanation only and which one stops is
/// a fact about these bytes.
#[test]
fn closing_the_rule_records_keeps_the_refusals_and_the_explanation() {
    let engine = Engine::new();
    let scope = open(REFUSED);
    let definition = definition_of(&engine, &scope);
    let mut inspection_budget = Budget::new(limits());
    let inspected = engine
        .inspect_header(
            &scope,
            ClassTarget::Root,
            &mut inspection_budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    let mut quoted = 0usize;
    let mut explanation_only = 0usize;
    let mut stopped = 0usize;
    for member in &inspected.inspection.header.methods {
        let request = request(
            &scope,
            &definition,
            &member.name.raw().0,
            &member.descriptor.raw().0,
        );
        let mut budget = Budget::new(limits());
        let plain = engine
            .recover_method(slice::from_ref(&scope), &request, &mut budget)
            .expect("a legal request is answered");
        let report = plain.recovery();
        assert!(
            report.rules.is_empty(),
            "the rule index is rule details, and the ordinary request asked for none: {:?}",
            report.rules
        );
        assert_eq!(
            report.evidence.state(RecoveryEvidenceKind::RuleDetails),
            EvidenceState::NotRequested
        );
        assert_eq!(optional_records(report), 0);
        assert!(evidence_agrees(report));
        match &report.outcome {
            RecoveryOutcome::Produced => {
                if report.text.contains("// @bytecode") {
                    quoted += 1;
                    // The gap the refusal states is a diagnostic of the same run and not a comment
                    // this test parses: the code is read off the report's own diagnostics.
                    let codes: Vec<&str> = report
                        .diagnostics
                        .iter()
                        .map(|diagnostic| diagnostic.code.as_str())
                        .collect();
                    assert!(
                        codes.iter().any(|code| code.starts_with("jre_")),
                        "a quoted refusal states its reason: {codes:?}"
                    );
                }
                if report.content == RecoveryContent::ExplanationOnly {
                    explanation_only += 1;
                    assert!(
                        !report.text.is_empty(),
                        "an explanation-only artifact is still an artifact"
                    );
                }
            }
            RecoveryOutcome::Stopped(_) => {
                stopped += 1;
                assert_eq!(report.text, "");
                for kind in RecoveryEvidenceKind::SUPPORTED {
                    assert_eq!(report.evidence.state(kind), EvidenceState::NotRequested);
                }
            }
        }

        // The same member under the full selection: the same text, the same planes, the same gaps.
        let mut full_budget = Budget::new(limits());
        let full = recover_with(
            &engine,
            &scope,
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut full_budget,
        );
        assert_eq!(full.recovery().text, report.text);
        assert_eq!(decisions(full.recovery()), decisions(report));
    }
    assert!(
        quoted > 0,
        "the fixture has a member whose refusal is quoted statement by statement"
    );
    assert!(
        stopped + explanation_only > 0,
        "and a member that is not a plain presentation: {stopped} stopped, {explanation_only} \
         explanation only"
    );
}
