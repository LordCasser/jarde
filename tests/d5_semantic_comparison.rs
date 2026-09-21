//! D5 task 7.1: the full-configuration comparison — one selection twice, and the three selections
//! against each other over the shipped P3 fixtures.
//!
//! # What 7.1 asks for, and what this file answers
//!
//! Task 7.1 asks for three things, and each has its own case below:
//!
//! 1. **same input, same selection, twice**: two runs of one request under one selection state the
//!    same report field for field, the wall clock excepted
//!    ([`a_repeated_request_states_the_same_report`]);
//! 2. **Essential / local / All state the same answer**: the text, the independent planes
//!    (`representation`, `quality`, `syntax_status`, `content`), the outcome, the fallbacks and
//!    every diagnostic — code *and* sentence — are identical under all three selections, and only
//!    the optional records differ ([`every_selection_presents_the_same_answer`]);
//! 3. **local evidence is the full evidence projected onto its range**: every record the local
//!    delivery holds is a record of the full one, in order (an ordered subsequence, category by
//!    category), the region records are exactly the full ones the range intersects, and every
//!    position the local segment table anchors quotes the same stretch of the same text
//!    ([`the_local_delivery_is_the_full_one_projected_onto_its_range`]).
//!
//! The fixtures are the committed P3 samples the change's D03/D04/D10 rows name: arithmetic
//! (`p3-nested-arithmetic`), receiver grouping (`p3-receiver-grouping`), boolean contexts
//! (`p3-boolean-contexts`), concatenation conversion (`p3-concat-conversion`), nested evaluation
//! effects (`p3-nested-eval`), origin — one source compiled with and without its debug tables
//! (`p3-scope`) — and a refusal (`p3-refused-cast`). **Every** member of the class is compared
//! (`<init>` and `<clinit>` included): a selection may not move any of them.
//!
//! # The controlled JDK half
//!
//! The last case re-runs a smaller table through `javac --release 8` and the JVM: for each listed
//! member the recovered text is compiled under the **essential** selection and under the **full**
//! one, the two compilations must reach the same verdict, their class files must be byte-identical,
//! and the program they form must print what the committed sample's own member prints. That is what
//! "the evidence selection may not change the body, and a new refusal may not delete an originally
//! correct sample" means where it can be checked: a selection that changed the text, dropped a rule's
//! output or added a refusal would answer a different program here. It is `#[ignore]`d because it
//! needs a JDK, exactly like the P3 3.3 comparison, so a machine without a compiler stays green:
//!
//! ```text
//! cargo test --test d5_semantic_comparison --all-features --locked -- --ignored --nocapture
//! ```
//!
//! **Boundaries this file states rather than hides**: the JDK half wraps the members whose
//! declaration this file can spell without reconstructing parameter names from a run's own facts
//! (`()`-descriptors only); the argument-taking members of every fixture are executed by
//! `tests/p3_execution_comparison.rs`, whose wrapper builder reads each parameter's spelling out of
//! the run itself. And **no wall-clock or resource claim is made anywhere in this file**: task 7.3
//! owns the timing protocol, and a comparison of text, planes and diagnostics is a *contract* and
//! *workload* statement.

use jarde::*;
use std::path::Path;
use std::process::Command;
use std::slice;

/// `ACC_STATIC` (JVMS table 4.6-A): the flag the wrapped declaration below reads.
const ACC_STATIC: u16 = 0x0008;
/// `ACC_PUBLIC`, for the same reason: a member no wrapper could call is not one this file wraps.
const ACC_PUBLIC: u16 = 0x0001;

// -------------------------------------------------------------------------------------------
// The fixtures: the committed P3 samples, one per family task 7.1 names.
// -------------------------------------------------------------------------------------------

struct Fixture {
    /// The family this sample stands for in the task's own list.
    family: &'static str,
    /// The class the sample declares, as the class file spells it.
    class: &'static str,
    /// The committed class file, read as bytes: the compiler is a generation-time input only.
    bytes: &'static [u8],
    /// What the JDK half must put on the classpath beside it: a sample whose refused members name
    /// another class of the same source states that class as its own file.
    classpath: &'static [(&'static str, &'static [u8])],
    /// Where the sample's provenance and its committed driver are recorded.
    source: &'static str,
}

const FIXTURES: &[Fixture] = &[
    Fixture {
        family: "arithmetic",
        class: "ModLike",
        bytes: include_bytes!("fixtures/p3-nested-arithmetic/v8/ModLike.class"),
        classpath: &[],
        source: "tests/fixtures/p3-nested-arithmetic/README.md",
    },
    Fixture {
        family: "receiver",
        class: "ReceiverGrouping",
        bytes: include_bytes!("fixtures/p3-receiver-grouping/v8/ReceiverGrouping.class"),
        classpath: &[],
        source: "tests/fixtures/p3-receiver-grouping/README.md",
    },
    Fixture {
        family: "boolean",
        class: "BooleanContexts",
        bytes: include_bytes!("fixtures/p3-boolean-contexts/v8/BooleanContexts.class"),
        classpath: &[],
        source: "tests/fixtures/p3-boolean-contexts/README.md",
    },
    Fixture {
        family: "concat",
        class: "ConcatConversion",
        bytes: include_bytes!("fixtures/p3-concat-conversion/v8/ConcatConversion.class"),
        classpath: &[],
        source: "tests/fixtures/p3-concat-conversion/README.md",
    },
    Fixture {
        family: "effect",
        class: "NestedEval",
        bytes: include_bytes!("fixtures/p3-nested-eval/v8/NestedEval.class"),
        classpath: &[],
        source: "tests/fixtures/p3-nested-eval/README.md",
    },
    Fixture {
        family: "origin (no debug tables)",
        class: "Scope",
        bytes: include_bytes!("fixtures/p3-scope/v8/Scope.class"),
        classpath: &[],
        source: "tests/fixtures/p3-scope/README.md",
    },
    Fixture {
        family: "origin (with debug tables)",
        class: "Scope",
        bytes: include_bytes!("fixtures/p3-scope/v8-debug/Scope.class"),
        classpath: &[],
        source: "tests/fixtures/p3-scope/README.md",
    },
    Fixture {
        family: "refusal",
        class: "RefusedCast",
        bytes: include_bytes!("fixtures/p3-refused-cast/v8/RefusedCast.class"),
        classpath: &[
            (
                "External.class",
                include_bytes!("fixtures/p3-refused-cast/v8/External.class"),
            ),
            (
                "Holder.class",
                include_bytes!("fixtures/p3-refused-cast/v8/Holder.class"),
            ),
        ],
        source: "tests/fixtures/p3-refused-cast/README.md",
    },
];

/// The fixtures are the committed P3 samples, one per family task 7.1 names.
///
/// Every entry names the record it came from, and that record is read here: a fixture whose
/// provenance pointer rots is a fixture nobody can check the classification below against.
#[test]
fn every_fixture_names_the_record_it_came_from() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for fixture in FIXTURES {
        let text = std::fs::read_to_string(root.join(fixture.source))
            .unwrap_or_else(|error| panic!("{}: {error}", fixture.source));
        assert!(
            text.contains(&format!("{}.java", fixture.class)),
            "{}: the record of `{}` names the source it was compiled from",
            fixture.source,
            fixture.class
        );
        assert!(
            !fixture.bytes.is_empty(),
            "{}: the committed class file is read",
            fixture.source
        );
    }
}

// -------------------------------------------------------------------------------------------
// The request shape every case here is made under.
// -------------------------------------------------------------------------------------------

/// Every dimension a request here uses, bounded generously: these cases measure what a selection
/// states, never how close a run came to a limit.
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

fn open(fixture: &Fixture) -> ArtifactSnapshot {
    let mut budget = Budget::new(limits());
    ArtifactSnapshot::open(ArtifactInput::bytes(fixture.bytes.to_vec()), &mut budget)
        .expect("the committed fixture is a readable class file")
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

/// One member of a class: the raw name and descriptor a request names it by, whether it declares a
/// `Code` attribute at all, and its own access flags.
#[derive(Clone)]
struct Member {
    name: Vec<u8>,
    descriptor: Vec<u8>,
    has_code: bool,
    access_flags: u16,
}

impl Member {
    fn label(&self) -> String {
        format!(
            "{}{}",
            String::from_utf8_lossy(&self.name),
            String::from_utf8_lossy(&self.descriptor)
        )
    }
}

/// One fixture, opened: its snapshot, the definition every request names, and the member records the
/// class's own header declares.
struct Sample {
    fixture: &'static Fixture,
    snapshot: ArtifactSnapshot,
    definition: PhysicalDefinitionId,
    members: Vec<Member>,
}

impl Sample {
    fn label(&self) -> String {
        format!("{} [{}]", self.fixture.class, self.fixture.family)
    }

    fn request(&self, member: &Member) -> MethodAnalysisRequest {
        MethodAnalysisRequest {
            environment: environment(&self.snapshot),
            method: PhysicalMethodId {
                owner: self.definition.clone(),
                name: JvmBytes(member.name.clone()),
                descriptor: JvmBytes(member.descriptor.clone()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        }
    }

    fn member(&self, name: &str) -> &Member {
        self.members
            .iter()
            .find(|member| member.name == name.as_bytes())
            .unwrap_or_else(|| panic!("{} declares `{name}`", self.fixture.class))
    }
}

fn sample_of(fixture: &'static Fixture) -> Sample {
    let snapshot = open(fixture);
    let mut budget = Budget::new(limits());
    let inspected = Engine::new()
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: inspected.source.class_bytes.clone(),
        variant: PhysicalVariant::Base,
    };
    let members: Vec<Member> = inspected
        .inspection
        .header
        .methods
        .iter()
        .map(|member| Member {
            name: member.name.raw().0.clone(),
            descriptor: member.descriptor.raw().0.clone(),
            // Whether the member declares a body at all: the same attribute shell this layer's other
            // readers read, so the applicability below is a fact about the bytes.
            has_code: member
                .attributes
                .iter()
                .any(|shell| shell.name.raw().0.as_slice() == b"Code"),
            access_flags: member.access_flags,
        })
        .collect();
    Sample {
        fixture,
        snapshot,
        definition,
        members,
    }
}

/// One recovery of one member under one selection, with its own budget.
fn recover(
    sample: &Sample,
    member: &Member,
    evidence: &RecoveryEvidenceRequest,
) -> RecoveredMethod {
    let mut budget = Budget::new(limits());
    Engine::new()
        .recover_method_with_evidence(
            slice::from_ref(&sample.snapshot),
            &sample.request(member),
            evidence,
            &mut budget,
        )
        .expect("a legal request is answered, not raised")
}

/// The driver range a local case here selects: the first decoded instruction's own span, read from
/// the bytes through the reader's own decoder rather than invented.
///
/// `None` for a member that declares no `Code`: a request for a position in a body that has none is
/// refused (D1's own case), so such a member has no local selection to compare and the cases below
/// leave it to the whole-method selections.
fn first_instruction_range(fixture: &Fixture, member: &Member) -> Option<BytecodeRange> {
    if !member.has_code {
        return None;
    }
    let snapshot = open(fixture);
    let mut budget = Budget::new(limits());
    let inspected = Engine::new()
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's header is readable");
    let header = inspected
        .inspection
        .header
        .methods
        .iter()
        .find(|candidate| {
            candidate.name.raw().0 == member.name
                && candidate.descriptor.raw().0 == member.descriptor
        })
        .expect("the member record this case came from");
    let code = method_code_facts(fixture.bytes, header, &mut budget)
        .expect("the member's own body decodes");
    let first = code.instructions.first()?;
    Some(BytecodeRange::new(first.bci, first.bci + first.width))
}

/// The three selections this file compares, each named so a failure states which one it was.
fn selections(sample: &Sample, member: &Member) -> Vec<(&'static str, RecoveryEvidenceRequest)> {
    let local = match first_instruction_range(sample.fixture, member) {
        Some(range) => RecoveryEvidenceRequest::all().with_driver_bci_range(range),
        // A member with no `Code` has no position to select: its local case *is* the whole-method
        // one, and the comparison below then holds it against the other two as it stands.
        None => RecoveryEvidenceRequest::all(),
    };
    vec![
        ("essential", RecoveryEvidenceRequest::essential()),
        ("local", local),
        ("all", RecoveryEvidenceRequest::all()),
    ]
}

/// Every member of one fixture that declares a body: the population every case below compares.
fn members_with_a_body(sample: &Sample) -> Vec<Member> {
    sample
        .members
        .iter()
        .filter(|member| member.has_code)
        .cloned()
        .collect()
}

// -------------------------------------------------------------------------------------------
// The answer a selection may not move.
// -------------------------------------------------------------------------------------------

/// Everything a selection may not move, as one string: the text, the independent planes, the
/// outcome, the fallbacks it kept, and every diagnostic — code **and** sentence, so an answer that
/// read its own figures off a table nobody selected is caught here.
fn answer(report: &RecoveryReport) -> String {
    let diagnostics = report
        .diagnostics
        .iter()
        .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "{}\n---\n{:?}/{:?}/{:?}/{:?}/{:?}/{:?}\n---\n{}",
        report.text,
        report.representation,
        report.quality,
        report.syntax_status,
        report.content,
        report.outcome,
        report.fallbacks,
        diagnostics,
    )
}

/// One report as a document without its wall-clock readings, so two runs of one request are compared
/// by what they *state*.
fn stated(recovered: &RecoveredMethod) -> String {
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

/// One category's owning records of one report, as the documents this comparison compares.
///
/// The mapping from category to payload is stated **again** here on purpose: read off a table the
/// library itself defined, a projection check would agree with the library's own idea of it.
fn records(report: &RecoveryReport, kind: RecoveryEvidenceKind) -> Vec<String> {
    match kind {
        RecoveryEvidenceKind::SourceMap => {
            report.source_map.segments().iter().map(document).collect()
        }
        RecoveryEvidenceKind::RegionDetails => report.regions.iter().map(document).collect(),
        RecoveryEvidenceKind::RuleDetails => {
            // The nine rule plans of one category, in the order the report states them: a record is
            // compared as the value it is, so this file never has to name the record's type and a
            // category the library grows stays compared.
            report
                .lambdas
                .iter()
                .map(document)
                .chain(report.concats.iter().map(document))
                .chain(report.accessors.iter().map(document))
                .chain(report.bridges.iter().map(document))
                .chain(report.news.iter().map(document))
                .chain(report.fields.iter().map(document))
                .chain(report.enum_switches.iter().map(document))
                .chain(report.init.iter().map(document))
                .chain(report.declaration.iter().map(document))
                .collect()
        }
        RecoveryEvidenceKind::NameDetails => report.aliased_names.iter().map(document).collect(),
        // The read details live beside the report, in the read the entry performed; this file holds
        // no entry that publishes one, so the category is empty on both sides. The projection is
        // therefore stated over the four categories this layer materializes, and that boundary is
        // this file's own, not the library's.
        RecoveryEvidenceKind::ReadDetails => Vec::new(),
    }
}

/// One owning record as the document the comparison above compares, so that a record is compared as
/// the value it is rather than by the fields this file happens to name.
fn document<T: serde::Serialize>(record: &T) -> String {
    serde_json::to_string(record).expect("an owning record is a document")
}

// -------------------------------------------------------------------------------------------
// 7.1 (1): one selection, twice.
// -------------------------------------------------------------------------------------------

/// The same input under the same selection states the same report, field for field.
///
/// This is D10's "同一输入同一选择重复运行逐字段一致" in the form the change states it: **only** the
/// wall-clock readings may differ, and the check is made on the serialized report so a field this
/// file never names is still compared.
#[test]
fn a_repeated_request_states_the_same_report() {
    let mut compared = 0usize;
    for fixture in FIXTURES {
        let sample = sample_of(fixture);
        for member in members_with_a_body(&sample) {
            for (name, selection) in selections(&sample, &member) {
                let first = stated(&recover(&sample, &member, &selection));
                let second = stated(&recover(&sample, &member, &selection));
                assert_eq!(
                    first,
                    second,
                    "{} `{}` under `{name}`: a repeated request states the same report",
                    sample.label(),
                    member.label()
                );
                compared += 1;
            }
        }
    }
    println!("determinism: {compared} (member, selection) pairs, each run twice, identical");
    assert!(
        compared >= 60,
        "the fixture set covers the families: only {compared} pairs"
    );
}

// -------------------------------------------------------------------------------------------
// 7.1 (2): three selections, one answer.
// -------------------------------------------------------------------------------------------

/// Essential, local evidence and All present the same artifact and state the same gaps.
#[test]
fn every_selection_presents_the_same_answer() {
    let mut checked = 0usize;
    let mut read_details_published = 0usize;
    for fixture in FIXTURES {
        let sample = sample_of(fixture);
        for member in members_with_a_body(&sample) {
            let selections = selections(&sample, &member);
            let runs: Vec<(&'static str, RecoveredMethod)> = selections
                .iter()
                .map(|(name, selection)| (*name, recover(&sample, &member, selection)))
                .collect();
            let reference = answer(runs[2].1.recovery());
            assert_eq!(
                answer(runs[0].1.recovery()),
                reference,
                "{} `{}`: the ordinary recovery states the same answer as the full one",
                sample.label(),
                member.label()
            );
            assert_eq!(
                answer(runs[1].1.recovery()),
                reference,
                "{} `{}`: a driver range selects evidence, never analysis",
                sample.label(),
                member.label()
            );
            // And the selections really differ in the way the change says they do: the ordinary one
            // materializes none of the optional records, and every category that *was* selected is
            // answered. A file whose three runs agreed in payload too would make the two assertions
            // above vacuous.
            for kind in RecoveryEvidenceKind::ALL {
                assert_eq!(
                    runs[0].1.recovery().evidence.state(kind),
                    EvidenceState::NotRequested,
                    "{} `{}`: {kind:?} is not requested by the ordinary recovery",
                    sample.label(),
                    member.label()
                );
                for (name, selection) in &selections {
                    if !selection.requests(kind) {
                        continue;
                    }
                    let state = runs[2].1.recovery().evidence.state(kind);
                    if kind == RecoveryEvidenceKind::ReadDetails {
                        // The one category this layer does not materialize itself: the entry that
                        // performed the read states it, and a run that published none of it states
                        // `NotPerformed`. Both are statements; `NotRequested` is not one of them.
                        assert!(
                            matches!(state, EvidenceState::Complete | EvidenceState::NotPerformed),
                            "{} `{}` under `{name}`: {kind:?} is answered with a statement, \
                             not with `NotRequested`: {state:?}",
                            sample.label(),
                            member.label()
                        );
                        read_details_published += usize::from(state == EvidenceState::Complete);
                    } else {
                        assert_eq!(
                            state,
                            EvidenceState::Complete,
                            "{} `{}` under `{name}`: {kind:?} was selected and answered",
                            sample.label(),
                            member.label()
                        );
                    }
                }
            }
            checked += 1;
        }
    }
    println!(
        "three-way comparison: {checked} member runs state one answer; \
         {read_details_published} of them publish read details"
    );
    assert!(
        checked >= 20,
        "the fixture set covers the families: {checked}"
    );
    assert!(
        read_details_published > 0,
        "the entry that performed the read really publishes the read details a selection asks for"
    );
}

// -------------------------------------------------------------------------------------------
// 7.1 (3): the local delivery is the full one projected onto its range.
// -------------------------------------------------------------------------------------------

/// Whether `records` is `full`'s own subsequence, in order: every record the smaller delivery holds
/// is one the larger one holds, and their order agrees.
fn is_subsequence(records: &[String], full: &[String]) -> bool {
    let mut at = 0usize;
    for record in records {
        match full[at..].iter().position(|candidate| candidate == record) {
            Some(step) => at += step + 1,
            None => return false,
        }
    }
    true
}

/// Every bytecode index a report's own segment table anchors.
fn anchored_bcis(report: &RecoveryReport) -> std::collections::BTreeSet<u32> {
    let mut anchors = std::collections::BTreeSet::new();
    for segment in report.source_map.segments() {
        anchors.extend(segment.origin().bcis());
    }
    anchors
}

/// The local delivery is the full one, filtered by the range the request stated.
#[test]
fn the_local_delivery_is_the_full_one_projected_onto_its_range() {
    let mut projected = 0usize;
    for fixture in FIXTURES {
        let sample = sample_of(fixture);
        for member in members_with_a_body(&sample) {
            let Some(range) = first_instruction_range(fixture, &member) else {
                continue;
            };
            let local = recover(
                &sample,
                &member,
                &RecoveryEvidenceRequest::all().with_driver_bci_range(range),
            );
            let full = recover(&sample, &member, &RecoveryEvidenceRequest::all());
            let local_report = local.recovery();
            let full_report = full.recovery();
            assert_eq!(
                local_report.text,
                full_report.text,
                "{} `{}`: the projection is over one artifact",
                sample.label(),
                member.label()
            );
            for kind in RecoveryEvidenceKind::ALL {
                let local_records = records(local_report, kind);
                let full_records = records(full_report, kind);
                assert!(
                    local_records.len() <= full_records.len(),
                    "{} `{}`: {kind:?} cannot grow by selecting less ({} against {})",
                    sample.label(),
                    member.label(),
                    local_records.len(),
                    full_records.len()
                );
                assert!(
                    is_subsequence(&local_records, &full_records),
                    "{} `{}`: every {kind:?} record of the local delivery is one of the full ones, \
                     in order",
                    sample.label(),
                    member.label()
                );
            }
            // The region records are the exact projection: the full ones the range intersects, in
            // the full one's order. This is the category D04's row is written about, and it is
            // checked here by the range's own rule rather than by a count.
            let expected: Vec<u32> = full_report
                .regions
                .iter()
                .filter(|region| range.intersects_any(region.blocks.iter().copied()))
                .map(|region| region.bci)
                .collect();
            assert_eq!(
                local_report
                    .regions
                    .iter()
                    .map(|region| region.bci)
                    .collect::<Vec<u32>>(),
                expected,
                "{} `{}`: the local regions are the full ones the range intersects",
                sample.label(),
                member.label()
            );
            // The segment table: every position the local map anchors is one the full map anchors,
            // and what each of them quotes is a *subset* of what the full map quotes for it — the
            // projection keeps a record whole and drops the records the range does not intersect,
            // so one position may be quoted by fewer segments and never by different ones.
            let local_anchors = anchored_bcis(local_report);
            let full_anchors = anchored_bcis(full_report);
            assert!(
                local_anchors.is_subset(&full_anchors),
                "{} `{}`: the local map anchors no position the full one does not",
                sample.label(),
                member.label()
            );
            for bci in &local_anchors {
                let quoted = |report: &RecoveryReport| {
                    report
                        .source_map
                        .text_of_bci(&report.text, *bci)
                        .into_iter()
                        .map(str::to_string)
                        .collect::<Vec<String>>()
                };
                let local_quotes = quoted(local_report);
                let full_quotes = quoted(full_report);
                assert!(
                    is_subsequence(&local_quotes, &full_quotes),
                    "{} `{}`: BCI {bci} quotes the same text in both deliveries, and the local map \
                     quotes only segments of the full one: {local_quotes:?} against {full_quotes:?}",
                    sample.label(),
                    member.label()
                );
            }
            assert_eq!(
                local_report.evidence.requested().driver_bci_range(),
                Some(range),
                "{} `{}`: the report echoes the effective selection",
                sample.label(),
                member.label()
            );
            projected += 1;
        }
    }
    println!("projection: {projected} member runs compared against the full delivery");
    assert!(
        projected >= 20,
        "the fixture set covers the families: {projected}"
    );
}

// -------------------------------------------------------------------------------------------
// 7.1 (4): the controlled JDK comparison, under the two ends of the selection.
// -------------------------------------------------------------------------------------------

/// One member the JDK half compiles and runs: the fixture it belongs to, the member's raw name and
/// descriptor, the class it must extend for its own class's names to resolve, and whether the P3 3.3
/// comparison classifies its body as one the run writes whole and the compiler accepts.
///
/// Every `compiles` entry is such a member (`tests/p3_execution_comparison.rs`, whose tables are the
/// classification this file re-states); the one entry that does not compile is the sample's quoted
/// refusal, which no selection may turn into a refusal of a *different* kind.
struct JdkMember {
    fixture: usize,
    name: &'static str,
    descriptor: &'static str,
    /// The class the wrapper extends, exactly as the P3 comparison states it, so that a body naming
    /// its own class's members resolves in the wrapper.
    extends: &'static str,
    /// Whether the recovered text compiles under a declaration this file can write.
    compiles: bool,
}

const JDK_MEMBERS: &[JdkMember] = &[
    JdkMember {
        fixture: 5,
        name: "simple",
        descriptor: "()I",
        extends: "",
        compiles: true,
    },
    JdkMember {
        fixture: 6,
        name: "simple",
        descriptor: "()I",
        extends: "",
        compiles: true,
    },
    JdkMember {
        fixture: 2,
        name: "flag",
        descriptor: "()Z",
        extends: "BooleanContexts",
        compiles: true,
    },
    JdkMember {
        fixture: 2,
        name: "answer",
        descriptor: "()I",
        extends: "BooleanContexts",
        compiles: true,
    },
    JdkMember {
        fixture: 3,
        name: "booleanLiteral",
        descriptor: "()Ljava/lang/String;",
        extends: "ConcatConversion",
        compiles: true,
    },
    JdkMember {
        fixture: 3,
        name: "nullPart",
        descriptor: "()Ljava/lang/String;",
        extends: "ConcatConversion",
        compiles: true,
    },
    JdkMember {
        fixture: 7,
        name: "leftRead",
        descriptor: "()I",
        extends: "RefusedCast",
        compiles: true,
    },
    JdkMember {
        fixture: 7,
        name: "chainCast",
        descriptor: "()Ljava/lang/String;",
        extends: "RefusedCast",
        compiles: false,
    },
];

/// The Java spelling of the result of one of the descriptors this file wraps.
///
/// Deliberately small: the table above states the descriptors, and a descriptor this mapping does not
/// hold is a table entry this file refuses to guess about.
fn return_type(descriptor: &str) -> String {
    let result = descriptor
        .strip_prefix('(')
        .and_then(|rest| rest.split_once(')'))
        .map(|(_, result)| result)
        .expect("a method descriptor states its result");
    match result {
        "I" => "int".to_string(),
        "J" => "long".to_string(),
        "Z" => "boolean".to_string(),
        "V" => "void".to_string(),
        "Ljava/lang/String;" => "java.lang.String".to_string(),
        other => panic!("this file wraps no member returning `{other}`"),
    }
}

/// The declaration this file writes for one wrapped member: its own flags, its descriptor's result
/// and its raw name. No parameter list: the table above holds `()`-members only.
fn declaration_of(member: &Member) -> String {
    let visibility = if member.access_flags & ACC_PUBLIC != 0 {
        "public "
    } else {
        ""
    };
    let static_ = if member.access_flags & ACC_STATIC != 0 {
        "static "
    } else {
        ""
    };
    format!(
        "{visibility}{static_}{} {}()",
        return_type(&String::from_utf8_lossy(&member.descriptor)),
        String::from_utf8_lossy(&member.name)
    )
}

/// One wrapped member's source: the recovered text under the declaration above.
fn wrapper_source(extends: &str, member: &Member, body: &str, scratch: &str) -> String {
    let inheritance = if extends.is_empty() {
        String::new()
    } else {
        format!(" extends {extends}")
    };
    format!(
        "public class {scratch}{inheritance} {{\n    {} {body}\n}}\n",
        declaration_of(member),
    )
}

/// The runner of one compiled case: the sample's own member and the recovered one, called the same
/// way in one JVM, each printing its own answer.
///
/// One recovered side is enough: the two selections' class files are compared byte for byte before
/// this runner exists, so "the other selection's program" is *this* program, not a second one.
fn runner_source(fixture: &Fixture, member: &Member, recovered: &str, scratch: &str) -> String {
    let name = String::from_utf8_lossy(&member.name);
    format!(
        "public class {scratch} {{\n    public static void main(String[] args) {{\n        \
         System.out.println({sample}.{name}());\n        \
         System.out.println({recovered}.{name}());\n    }}\n}}\n",
        sample = fixture.class,
    )
}

/// A directory for one case, removed with the guard so a failed case leaves nothing behind.
struct Scratch(std::path::PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-d5-semantic-{label}-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("the scratch directory is created");
        Self(path)
    }

    /// One compilation directory, holding the sample's own classes beside whatever this case writes
    /// into it: the wrapper of a sample whose bodies name their own class's members has to be
    /// compiled against that class, exactly as the P3 3.3 comparison compiles its wrappers.
    fn directory(&self, fixture: &Fixture, name: &str) -> std::path::PathBuf {
        let dir = self.0.join(name);
        std::fs::create_dir_all(&dir).expect("the compilation directory is created");
        std::fs::write(dir.join(format!("{}.class", fixture.class)), fixture.bytes)
            .expect("the sample's own class is written");
        for (file, bytes) in fixture.classpath {
            std::fs::write(dir.join(file), bytes).expect("the sample's sibling class is written");
        }
        dir
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `javac --release 8` over one file, answering its refusal as a string when it refuses.
fn javac(dir: &Path, files: &[&str]) -> std::result::Result<(), String> {
    let output = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-d")
        .arg(dir)
        .args(files)
        .current_dir(dir)
        .output()
        .map_err(|error| format!("javac could not be started: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

/// The recovered text compiled under one selection, into one directory of its own, as the class file
/// that compilation produced.
///
/// The two compilations of one row write the **same** class name into two directories for one
/// reason: javac states the source file's name and the class's name inside the class file, so a
/// byte-for-byte comparison of the two products is a statement that they *are* the same program, and
/// it stops being one the moment the two are compiled under different names.
fn compile_into(
    entry: &JdkMember,
    member: &Member,
    body: &str,
    dir: &Path,
    class: &str,
) -> std::result::Result<Vec<u8>, String> {
    std::fs::write(
        dir.join(format!("{class}.java")),
        wrapper_source(entry.extends, member, body, class),
    )
    .expect("the wrapper is written");
    javac(dir, &[&format!("{class}.java")])?;
    std::fs::read(dir.join(format!("{class}.class"))).map_err(|error| {
        format!("javac reported success but wrote no class file for {class}: {error}")
    })
}

/// The controlled JDK comparison: the essential selection and the full one compile the same program.
///
/// For each listed member the recovered text is compiled twice — once from the **essential** run and
/// once from the **full-evidence** run, both under the same class's name in two directories — and
/// the two compilations must reach the same verdict, produce **byte-identical** class files, and the
/// program they form must print what the committed sample's own member prints. The text being equal
/// is checked first (it is the assertion the JDK-free cases make); what this case adds is that the
/// *program* is the same one, through a compiler and a JVM this file does not control: a selection
/// that had changed the body, dropped a rule's output or added a refusal would answer differently
/// here, and a member the P3 3.3 comparison classifies as one the run writes whole would stop being
/// one.
#[test]
#[ignore = "needs a JDK on PATH: it compiles the recovered text with `javac --release 8` and runs \
            it with `java`. `cargo test` therefore stays green without a compiler; run it with \
            `-- --ignored` on a machine that has one"]
fn the_essential_and_the_full_selection_compile_the_same_program() {
    let samples: Vec<Sample> = FIXTURES.iter().map(sample_of).collect();
    let scratch = Scratch::new("compile");
    let mut rows: Vec<String> = Vec::new();
    for (index, entry) in JDK_MEMBERS.iter().enumerate() {
        let fixture = &FIXTURES[entry.fixture];
        let sample = &samples[entry.fixture];
        let member = sample.member(entry.name);
        assert_eq!(
            String::from_utf8_lossy(&member.descriptor),
            entry.descriptor,
            "{} `{}`: the table's descriptor is the class file's own",
            fixture.class,
            entry.name
        );
        assert_eq!(
            member.access_flags & (ACC_PUBLIC | ACC_STATIC),
            ACC_PUBLIC | ACC_STATIC,
            "{} `{}`: the table wraps a member a plain `main` can call",
            fixture.class,
            entry.name
        );
        let essential = recover(sample, member, &RecoveryEvidenceRequest::essential());
        let all = recover(sample, member, &RecoveryEvidenceRequest::all());
        assert_eq!(
            essential.recovery().text,
            all.recovery().text,
            "{} `{}`: the two selections present the same body",
            fixture.class,
            entry.name
        );
        let class = format!("D5{index}");
        let essential_dir = scratch.directory(fixture, "essential");
        let full_dir = scratch.directory(fixture, "full");
        let essential_verdict = compile_into(
            entry,
            member,
            &essential.recovery().text,
            &essential_dir,
            &class,
        );
        let all_verdict = compile_into(entry, member, &all.recovery().text, &full_dir, &class);
        assert_eq!(
            essential_verdict.is_ok(),
            all_verdict.is_ok(),
            "{} `{}`: the two selections reach the same compiler verdict (essential: {:?}, full: \
             {:?})",
            fixture.class,
            entry.name,
            essential_verdict.as_ref().err(),
            all_verdict.as_ref().err()
        );
        assert_eq!(
            essential_verdict.is_ok(),
            entry.compiles,
            "{} `{}`: the P3 3.3 comparison's own classification is what this table states \
             (essential verdict: {:?})",
            fixture.class,
            entry.name,
            essential_verdict.as_ref().err()
        );
        if !entry.compiles {
            // The quoted refusal: javac's own sentence is the artifact's, and it is the same one
            // under both selections — checked above — so this row records it and stops here.
            rows.push(format!(
                "{}/{}: both selections refuse, with javac's own sentence: {}",
                fixture.class,
                entry.name,
                refusal_line(&all_verdict.expect_err("a refused row states its refusal")),
            ));
            continue;
        }
        let essential_class = essential_verdict.expect("a row that compiles states its class file");
        let all_class = all_verdict.expect("the same verdict, checked above");
        assert_eq!(
            essential_class,
            all_class,
            "{} `{}`: the two selections compile the same program, byte for byte ({} against {} \
             bytes)",
            fixture.class,
            entry.name,
            essential_class.len(),
            all_class.len()
        );

        // The original side: the committed sample's own class, called beside the recovered text in
        // one JVM, so a member whose value depends on its class's static state is compared in the
        // run that holds that state. It is already in the essential directory, beside the wrapper.
        let dir = essential_dir;
        let runner = format!("D5{index}Runner");
        std::fs::write(
            dir.join(format!("{runner}.java")),
            runner_source(fixture, member, &class, &runner),
        )
        .expect("the runner is written");
        javac(&dir, &[&format!("{runner}.java")]).unwrap_or_else(|refusal| {
            panic!(
                "the runner of {} `{}` compiles:\n{refusal}",
                fixture.class, entry.name
            )
        });
        let output = Command::new("java")
            .arg("-cp")
            .arg(&dir)
            .arg(&runner)
            .current_dir(&dir)
            .output()
            .expect("java runs the compiled runner");
        assert!(
            output.status.success(),
            "{} `{}`: the compiled program runs: {}",
            fixture.class,
            entry.name,
            String::from_utf8_lossy(&output.stderr)
        );
        let printed: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_string)
            .collect();
        assert_eq!(
            printed.len(),
            2,
            "{} `{}`: the sample's own member and the recovered one each print one line: {printed:?}",
            fixture.class,
            entry.name
        );
        assert_eq!(
            printed[1], printed[0],
            "{} `{}`: the recovered text prints what the sample's own member prints",
            fixture.class, entry.name
        );
        rows.push(format!(
            "{}/{}: essential == full ({} bytes of class file), both print `{}`",
            fixture.class,
            entry.name,
            all_class.len(),
            printed[0],
        ));
    }
    for row in &rows {
        println!("{row}");
    }
    assert_eq!(rows.len(), JDK_MEMBERS.len());
}

/// The line of a compiler's refusal that names the source position, for the one line a row states.
///
/// javac writes its own options warnings first (the `--release 8` deprecation notice), and a row that
/// printed those would look identical for every refusal this file records; the diagnostic's own
/// location line is the one that states what the compiler refused.
fn refusal_line(message: &str) -> String {
    message
        .lines()
        .find(|line| line.contains(".java:"))
        .or_else(|| message.lines().next())
        .unwrap_or_default()
        .to_string()
}
