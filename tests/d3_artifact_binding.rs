//! D3' of change `add-demand-driven-core-results` (tasks 5.1–5.3): the **artifact binding** and the
//! evidence a later request may attach to a text it already holds.
//!
//! Every case here reads the library's own public surface: a report's artifact binding
//! (`RecoveryReport::artifact`), the entry points that produce it, the budget each request pays with
//! and the D0 counting port (`jarde::d0_counts`). The collision matrix is a matrix of *inputs* —
//! same name different content, same bytes through two origins, a duplicated entry, a duplicated
//! member record, a changed rule set and a changed text — and what each row asserts is that the run
//! answers with its own artifact or with a stated mismatch, never with another artifact's identity.
//!
//! The fixtures are assembled in this file (`assembled`, the same route `tests/engine.rs` takes) or
//! taken from the committed corpus (`tests/fixtures/p3-scope`), so every input is deterministic and
//! no case needs a file system.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::slice;
use std::sync::Mutex;

/// The same source compiled twice by `javac --release 8` (`tests/fixtures/p3-scope`): one class file
/// with no debug attributes and one with them. Same class name (`Scope`), same members, two
/// contents — the corpus sample the "one name, two artifacts" row needs.
const SCOPE: &[u8] = include_bytes!("fixtures/p3-scope/v8/Scope.class");
const SCOPE_DEBUG: &[u8] = include_bytes!("fixtures/p3-scope/v8-debug/Scope.class");

// ---------------------------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------------------------

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

fn new_budget() -> Budget {
    Budget::new(limits())
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    let mut budget = new_budget();
    ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("the assembled fixture is a readable class file")
}

/// The counting tests of this binary run one at a time: the port is one process-wide set of
/// counters, so two tests counting at once would each read the other's work.
static GATE: Mutex<()> = Mutex::new(());

fn gate() -> std::sync::MutexGuard<'static, ()> {
    GATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A stored-only archive, built with the repository's own `rawzip` dev-dependency.
fn zip_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    const STORE: u16 = 0;
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

/// One member record of an assembled class.
struct Member<'a> {
    access_flags: u16,
    name: &'a [u8],
    descriptor: &'a [u8],
    /// The instruction bytes of the member's `Code` entry, with its stack and local counts.
    code: &'a [u8],
    max_stack: u16,
    max_locals: u16,
}

/// A constant pool built as the fixture names it, one entry per distinct spelling.
#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    fn utf8(&mut self, value: &[u8]) -> u16 {
        let mut entry = vec![1];
        entry.extend_from_slice(
            &u16::try_from(value.len())
                .expect("fixture name fits u16")
                .to_be_bytes(),
        );
        entry.extend_from_slice(value);
        self.push(entry)
    }

    fn class(&mut self, name: u16) -> u16 {
        let mut entry = vec![7];
        entry.extend_from_slice(&name.to_be_bytes());
        self.push(entry)
    }

    fn push(&mut self, entry: Vec<u8>) -> u16 {
        if let Some(position) = self.entries.iter().position(|held| *held == entry) {
            return u16::try_from(position + 1).expect("fixture pool fits u16");
        }
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("fixture pool fits u16")
    }
}

/// One class file: version 52, `class` as `this_class`, `java/lang/Object` as its superclass, no
/// interfaces, no fields, no class attributes, and the member records given — duplicates included,
/// because a duplicate declaration is one of the inputs this file has to state.
fn assembled(class: &[u8], members: &[Member<'_>]) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_name = pool.utf8(class);
    let this = pool.class(this_name);
    let object = pool.utf8(b"java/lang/Object");
    let super_class = pool.class(object);
    let code_name = pool.utf8(b"Code");
    // Every member's own name and descriptor is interned **before** the pool is written: a record's
    // index has to address an entry the pool in front of it really holds.
    let spelled: Vec<(u16, u16)> = members
        .iter()
        .map(|member| (pool.utf8(member.name), pool.utf8(member.descriptor)))
        .collect();
    let mut out = Vec::new();
    out.extend_from_slice(&0xcafe_babe_u32.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes());
    out.extend_from_slice(&52_u16.to_be_bytes());
    out.extend_from_slice(
        &(u16::try_from(pool.entries.len()).expect("fixture pool fits u16") + 1).to_be_bytes(),
    );
    for entry in &pool.entries {
        out.extend_from_slice(entry);
    }
    out.extend_from_slice(&0x0021_u16.to_be_bytes());
    out.extend_from_slice(&this.to_be_bytes());
    out.extend_from_slice(&super_class.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes()); // interfaces
    out.extend_from_slice(&0_u16.to_be_bytes()); // fields
    out.extend_from_slice(
        &u16::try_from(members.len())
            .expect("fixture members fit u16")
            .to_be_bytes(),
    );
    for (member, (name, descriptor)) in members.iter().zip(&spelled) {
        out.extend_from_slice(&member.access_flags.to_be_bytes());
        out.extend_from_slice(&name.to_be_bytes());
        out.extend_from_slice(&descriptor.to_be_bytes());
        out.extend_from_slice(&1_u16.to_be_bytes()); // one attribute
        let mut content = Vec::new();
        content.extend_from_slice(&member.max_stack.to_be_bytes());
        content.extend_from_slice(&member.max_locals.to_be_bytes());
        content.extend_from_slice(
            &u32::try_from(member.code.len())
                .expect("fixture code fits u32")
                .to_be_bytes(),
        );
        content.extend_from_slice(member.code);
        content.extend_from_slice(&0_u16.to_be_bytes()); // exception table
        content.extend_from_slice(&0_u16.to_be_bytes()); // nested attributes
        out.extend_from_slice(&code_name.to_be_bytes());
        out.extend_from_slice(
            &u32::try_from(content.len())
                .expect("fixture attribute fits u32")
                .to_be_bytes(),
        );
        out.extend_from_slice(&content);
    }
    out.extend_from_slice(&0_u16.to_be_bytes()); // class attributes
    out
}

/// The body of a `static int value()` returning one small constant: `iconst_<n>; ireturn`.
fn constant_body(value: u8) -> Vec<u8> {
    assert!(value <= 5, "the fixture spells small constants only");
    vec![0x03 + value, 0xac]
}

/// The member every single-member fixture declares: `public static int value()`, returning `value`.
fn value_member(value: u8) -> Member<'static> {
    Member {
        access_flags: 0x0009,
        name: b"value",
        descriptor: b"()I",
        code: Box::leak(constant_body(value).into_boxed_slice()),
        max_stack: 1,
        max_locals: 0,
    }
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

/// The same request over a ZIP snapshot, whose classes are read through an explicit classpath root.
fn archive_environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
    let mut environment = environment(snapshot);
    let root = LoadRoot::Container {
        origin: ContainerOrigin {
            snapshot: snapshot.id().clone(),
            root_container: ContainerId("root".into()),
            steps: Vec::new(),
        },
        prefix: ArchiveNameBytes(Vec::new()),
    };
    environment.runtime.load_domain.roots = vec![root.clone()];
    environment.domains[0].roots = vec![root];
    environment
}

fn request(
    environment: ResolutionEnvironment,
    owner: &PhysicalDefinitionId,
    name: &[u8],
    descriptor: &[u8],
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment,
        method: PhysicalMethodId {
            owner: owner.clone(),
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    }
}

/// Every definition one snapshot holds, as the public listing entry states them.
fn definitions(engine: &Engine, snapshot: &ArtifactSnapshot) -> Vec<PhysicalDefinitionId> {
    let mut budget = new_budget();
    engine
        .list_class_declarations(snapshot, &PhysicalScope::SnapshotAll, &mut budget)
        .expect("the fixture's classes are listed")
        .items
        .into_iter()
        .map(|item| item.definition)
        .collect()
}

/// One recovery under one selection, with the budget the caller keeps.
fn recover(
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

/// The binding of one produced report, as the caller keeps it.
fn binding_of(recovered: &RecoveredMethod) -> ArtifactBinding {
    recovered
        .recovery()
        .artifact
        .binding()
        .expect("a run whose entry stated a subject publishes the binding of its artifact")
        .clone()
}

/// This run's verdict, as the mismatch it states.
fn mismatch_of(report: &RecoveryReport) -> ArtifactMismatch {
    match report.artifact.agreement() {
        ArtifactAgreement::Mismatched { mismatch } => mismatch.clone(),
        other => panic!("the verdict is a mismatch, found {other:?}"),
    }
}

/// The decisions and the gaps a selection may not move.
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

/// One run over one definition and one member under one selection, with the budget the caller keeps.
fn run_with(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    environment: ResolutionEnvironment,
    definition: &PhysicalDefinitionId,
    member: (&[u8], &[u8]),
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> RecoveredMethod {
    let request = request(environment, definition, member.0, member.1);
    recover(engine, snapshot, &request, evidence, budget)
}

// ---------------------------------------------------------------------------------------------
// 5.1 — the collision matrix
// ---------------------------------------------------------------------------------------------

/// One name, two contents: two definitions that spell the same class and the same member are still
/// two artifacts, and the first one's binding never explains the second one's text.
///
/// The two samples are the same source compiled twice (`tests/fixtures/p3-scope`): class `Scope`,
/// member `simple()I`, two different class files. Nothing about the spelling distinguishes them.
#[test]
fn two_definitions_of_one_name_are_two_artifacts() {
    let _gate = gate();
    let engine = Engine::new();
    let one = open(SCOPE);
    let two = open(SCOPE_DEBUG);
    let found_one = definitions(&engine, &one);
    let found_two = definitions(&engine, &two);
    assert_eq!(found_one.len(), 1, "the fixture declares one class");
    assert_eq!(found_two.len(), 1, "the fixture declares one class");
    assert_ne!(
        found_one[0].class_bytes, found_two[0].class_bytes,
        "the two samples are different bytes"
    );

    let mut first_budget = new_budget();
    let first = run_with(
        &engine,
        &one,
        environment(&one),
        &found_one[0],
        OTHER,
        &RecoveryEvidenceRequest::essential(),
        &mut first_budget,
    );
    let mut second_budget = new_budget();
    let second = run_with(
        &engine,
        &two,
        environment(&two),
        &found_two[0],
        OTHER,
        &RecoveryEvidenceRequest::essential(),
        &mut second_budget,
    );
    let first_binding = binding_of(&first);
    let second_binding = binding_of(&second);
    assert!(
        first.recovery().produced() && second.recovery().produced(),
        "both samples present the member: {:?} / {:?}",
        first.recovery().outcome,
        second.recovery().outcome
    );
    // The name a display would show is the same for both; the identity is not.
    assert_eq!(
        first_binding.method().name.0,
        second_binding.method().name.0,
        "both artifacts are of a member spelled the same way"
    );
    assert_eq!(
        first_binding.method().descriptor.0,
        second_binding.method().descriptor.0
    );
    assert_eq!(
        first_binding.method().owner.class_bytes.digest.0.len(),
        64,
        "the binding states the class-bytes digest the read established"
    );
    assert_ne!(
        first_binding, second_binding,
        "the binding states the content identity, not the spelling"
    );
    assert!(
        first_binding
            .mismatch(&second_binding)
            .expect("the two bindings differ")
            .disagrees_on(ArtifactDimension::Method),
        "and it differs on the physical identity"
    );

    // Explaining the second artifact with the first artifact's binding is a mismatch, never an
    // agreement: the text of the run that produced the first binding is not the text this run wrote.
    let mut budget = new_budget();
    let reused = run_with(
        &engine,
        &two,
        environment(&two),
        &found_two[0],
        OTHER,
        &RecoveryEvidenceRequest::all().with_expected_artifact(first_binding),
        &mut budget,
    );
    assert!(
        reused.recovery().produced(),
        "the run's own artifact stands whatever the verdict about the one the request named"
    );
    let mismatch = mismatch_of(reused.recovery());
    assert!(mismatch.disagrees_on(ArtifactDimension::Method));
    assert_eq!(mismatch.code(), ARTIFACT_MISMATCH_CODE);
    assert!(
        reused
            .recovery()
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == ARTIFACT_MISMATCH_CODE),
        "the verdict is a gap of the run: {:?}",
        reused.recovery().diagnostics
    );
    assert!(
        !reused.recovery().artifact.attaches(),
        "a mismatch attaches no evidence to the artifact the request named"
    );
    for kind in RecoveryEvidenceKind::SUPPORTED {
        assert_eq!(
            reused.recovery().evidence.state(kind),
            EvidenceState::NotPerformed,
            "nothing of the selection was materialized: {kind:?}"
        );
    }
    assert!(
        reused.recovery().text == second.recovery().text,
        "and the artifact this run committed is the one it always writes"
    );
}

/// One class stored twice: the same bytes reached through two origins are two definitions, and the
/// binding states the origin — the entry ordinal or the container included — and never only the
/// bytes.
///
/// The row is stated in two halves, because the two halves are two different facts:
///
/// * a **duplicated entry** (one archive holding the same raw name twice) is two candidates the
///   reader states and the loader refuses to bind — so no artifact is produced for either of them
///   and no binding can explain one with the other;
/// * the same class bytes reached through **two containers** are two definitions the loader binds,
///   two artifacts with an identical text, and two bindings that do not agree.
#[test]
fn one_class_stored_twice_is_two_artifacts() {
    let _gate = gate();
    let engine = Engine::new();

    // (1) A duplicated entry: the physical view keeps both candidates, and the run that would have to
    // elect one refuses to present a body at all.
    let archive = zip_of(&[(b"Scope.class", SCOPE), (b"Scope.class", SCOPE)]);
    let snapshot = open(&archive);
    let found = definitions(&engine, &snapshot);
    assert_eq!(
        found.len(),
        2,
        "a duplicated entry is two candidate definitions: {found:?}"
    );
    assert_eq!(
        found[0].class_bytes, found[1].class_bytes,
        "the two entries hold the same bytes"
    );
    assert_ne!(
        found[0].location, found[1].location,
        "and they are two origins"
    );
    let entries: Vec<u64> = found
        .iter()
        .map(|definition| {
            definition
                .location
                .entry()
                .expect("the archive's definitions are entries")
                .ordinal
        })
        .collect();
    assert_ne!(entries[0], entries[1], "the entry ordinals differ");
    for definition in &found {
        let mut budget = new_budget();
        let recovered = run_with(
            &engine,
            &snapshot,
            archive_environment(&snapshot),
            definition,
            OTHER,
            &RecoveryEvidenceRequest::essential(),
            &mut budget,
        );
        assert!(
            !recovered.recovery().produced(),
            "one of two indistinguishable candidates is not a body to present: {:?}",
            recovered.recovery().outcome
        );
        assert!(
            recovered.recovery().artifact.binding().is_none(),
            "and no artifact means no binding: {:?}",
            recovered.recovery().artifact
        );
    }

    // (2) The same class bytes through two containers: two definitions the loader really binds.
    let first_archive = zip_of(&[(b"Scope.class", SCOPE)]);
    let second_archive = zip_of(&[(b"Scope.class", SCOPE), (b"notes.txt", b"padding")]);
    let first_snapshot = open(&first_archive);
    let second_snapshot = open(&second_archive);
    let first_found = definitions(&engine, &first_snapshot);
    let second_found = definitions(&engine, &second_snapshot);
    assert_eq!(first_found.len(), 1);
    assert_eq!(second_found.len(), 1);
    assert_eq!(
        first_found[0].class_bytes, second_found[0].class_bytes,
        "the two containers hold the same class bytes"
    );
    assert_ne!(
        first_found[0].snapshot(),
        second_found[0].snapshot(),
        "and they are two origins"
    );

    let mut runs = Vec::new();
    for (snapshot, definition) in [
        (&first_snapshot, &first_found[0]),
        (&second_snapshot, &second_found[0]),
    ] {
        let mut budget = new_budget();
        let recovered = run_with(
            &engine,
            snapshot,
            archive_environment(snapshot),
            definition,
            OTHER,
            &RecoveryEvidenceRequest::essential(),
            &mut budget,
        );
        assert!(
            recovered.recovery().produced(),
            "each container's own definition is bound and presented: {:?} / {:?}",
            recovered.recovery().outcome,
            recovered.analysis().execution
        );
        runs.push(recovered);
    }
    let bindings: Vec<ArtifactBinding> = runs.iter().map(binding_of).collect();
    assert_eq!(
        runs[0].recovery().text,
        runs[1].recovery().text,
        "the same bytes and the same member are the same text"
    );
    assert_eq!(
        bindings[0].method().owner.class_bytes,
        bindings[1].method().owner.class_bytes,
        "the two artifacts are of the same class bytes"
    );
    let mismatch = bindings[0]
        .mismatch(&bindings[1])
        .expect("two origins are two artifacts");
    assert!(
        mismatch.disagrees_on(ArtifactDimension::Method),
        "the origin is part of the identity: {:?}",
        mismatch.dimensions()
    );
    assert!(
        !mismatch.disagrees_on(ArtifactDimension::Text),
        "an identical text is still not an identity: {:?}",
        mismatch.dimensions()
    );
    // The one artifact the caller holds does not explain the other one, even with an identical text.
    let mut budget = new_budget();
    let reused = run_with(
        &engine,
        &second_snapshot,
        archive_environment(&second_snapshot),
        &second_found[0],
        OTHER,
        &RecoveryEvidenceRequest::all().with_expected_artifact(bindings[0].clone()),
        &mut budget,
    );
    assert!(mismatch_of(reused.recovery()).disagrees_on(ArtifactDimension::Method));
    assert_eq!(
        reused.recovery().text,
        runs[1].recovery().text,
        "the artifact this run commits is its own, whatever the verdict"
    );
    for kind in RecoveryEvidenceKind::SUPPORTED {
        assert_eq!(
            reused.recovery().evidence.state(kind),
            EvidenceState::NotPerformed,
            "nothing of the selection was materialized: {kind:?}"
        );
    }
}

/// A duplicated member record: the class declares one name and descriptor twice, so there is no
/// *record* to write an artifact for — and the run never answers with the one a name would have
/// picked, nor with a binding for a body it did not present.
#[test]
fn a_duplicated_member_record_binds_nothing() {
    let _gate = gate();
    let engine = Engine::new();
    let class = assembled(b"Fixture", &[value_member(1), value_member(1)]);
    let snapshot = open(&class);
    let found = definitions(&engine, &snapshot);
    assert_eq!(found.len(), 1, "the fixture declares one class");
    let mut budget = new_budget();
    let request = request(
        environment(&snapshot),
        &found[0],
        VALUE_MEMBER.0,
        VALUE_MEMBER.1,
    );
    let outcome = engine.recover_method(slice::from_ref(&snapshot), &request, &mut budget);
    let recovered = match outcome {
        Ok(recovered) => recovered,
        Err(Error::InvalidInput { ref code, .. }) => {
            assert_eq!(
                code, "classfile_method_ambiguous",
                "two records of one spelling are two records, and this entry elects neither"
            );
            return;
        }
        Err(other) => panic!("the duplicated declaration is refused: {other:?}"),
    };
    // Whatever the run did with the body, it never claims an artifact for a record it could not
    // establish: no text and no binding.
    assert!(
        !recovered.recovery().produced(),
        "a duplicated record is not a body to present: {:?}",
        recovered.recovery().outcome
    );
    assert!(
        recovered.recovery().artifact.binding().is_none(),
        "no artifact, no binding: {:?}",
        recovered.recovery().artifact
    );
    let analysis = recovered.analysis();
    assert!(
        !matches!(analysis.execution, ExecutionReport::Complete { .. }),
        "the analysis states its own stop: {:?}",
        analysis.execution
    );
}

/// A changed rule set: the same body presented under another profile is a different artifact, even
/// when the text the two profiles write happens to be the same.
#[test]
fn a_changed_rule_set_is_a_mismatch() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(&assembled(b"Fixture", &[value_member(1)]));
    let found = definitions(&engine, &snapshot);
    let mut java_8_budget = new_budget();
    let java_8 = run_with(
        &engine,
        &snapshot,
        environment(&snapshot),
        &found[0],
        VALUE_MEMBER,
        &RecoveryEvidenceRequest::essential(),
        &mut java_8_budget,
    );
    let expected = binding_of(&java_8);

    // The same bytes, the same member, another profile: the rules a profile admits are part of what
    // the artifact is, so the binding differs even before the text does.
    let mut java_7 = environment(&snapshot);
    java_7.runtime.profile.java_release = 7;
    java_7.domains[0].module_mode = ModuleMode::ClassPath;
    let mut budget = new_budget();
    let run = run_with(
        &engine,
        &snapshot,
        java_7,
        &found[0],
        VALUE_MEMBER,
        &RecoveryEvidenceRequest::all().with_expected_artifact(expected.clone()),
        &mut budget,
    );
    let report = run.recovery();
    assert!(
        report.produced(),
        "the other profile still presents the body"
    );
    assert_eq!(
        report.text,
        java_8.recovery().text,
        "the text this body recovers is the same under both profiles"
    );
    let mismatch = mismatch_of(report);
    assert!(
        mismatch.disagrees_on(ArtifactDimension::Configuration),
        "the profile is part of the configuration the artifact was written under: {:?}",
        mismatch.dimensions()
    );
    assert!(
        !mismatch.disagrees_on(ArtifactDimension::Text),
        "the text did not change, and the verdict does not claim it did: {:?}",
        mismatch.dimensions()
    );
    assert_eq!(
        report.evidence.state(RecoveryEvidenceKind::RegionDetails),
        EvidenceState::NotPerformed
    );
}

/// A binding of another **schema** is refused as a mismatch rather than answered: the versioned
/// contract is not a suggestion, and an older or newer document is not read as agreement.
#[test]
fn a_binding_of_another_schema_is_a_mismatch() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(&assembled(b"Fixture", &[value_member(1)]));
    let found = definitions(&engine, &snapshot);
    let mut budget = new_budget();
    let first = run_with(
        &engine,
        &snapshot,
        environment(&snapshot),
        &found[0],
        VALUE_MEMBER,
        &RecoveryEvidenceRequest::essential(),
        &mut budget,
    );
    let mut document = serde_json::to_value(binding_of(&first)).expect("a binding serializes");
    document["schema"] = serde_json::json!(ARTIFACT_SCHEMA + 1);
    let other: ArtifactBinding =
        serde_json::from_value(document).expect("the value is a binding document");

    let mut budget = new_budget();
    let run = run_with(
        &engine,
        &snapshot,
        environment(&snapshot),
        &found[0],
        VALUE_MEMBER,
        &RecoveryEvidenceRequest::all().with_expected_artifact(other),
        &mut budget,
    );
    let mismatch = mismatch_of(run.recovery());
    assert!(mismatch.disagrees_on(ArtifactDimension::Schema));
    assert_eq!(
        run.recovery()
            .artifact
            .binding()
            .map(ArtifactBinding::schema),
        Some(ARTIFACT_SCHEMA),
        "the run states its own schema, never the one the request named"
    );
}

/// An expectation without a category to select explains nothing, and the request is refused in the
/// same shape vocabulary the driver range already uses.
#[test]
fn an_expectation_without_a_selection_is_refused() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(&assembled(b"Fixture", &[value_member(1)]));
    let found = definitions(&engine, &snapshot);
    let mut budget = new_budget();
    let first = run_with(
        &engine,
        &snapshot,
        environment(&snapshot),
        &found[0],
        VALUE_MEMBER,
        &RecoveryEvidenceRequest::essential(),
        &mut budget,
    );
    let mut budget = new_budget();
    let refused = run_with(
        &engine,
        &snapshot,
        environment(&snapshot),
        &found[0],
        VALUE_MEMBER,
        &RecoveryEvidenceRequest::essential().with_expected_artifact(binding_of(&first)),
        &mut budget,
    );
    match refused.recovery().stop() {
        Some(StopReason::EvidenceRefused { code, .. }) => {
            assert_eq!(*code, "jre_evidence_artifact_shape");
        }
        other => panic!("the shape is refused, found {other:?}"),
    }
}

// ---------------------------------------------------------------------------------------------
// 5.2 — the same pipeline rebuilds the artifact and only then attaches evidence
// ---------------------------------------------------------------------------------------------

/// One class file's member, recovered under one selection with a budget the caller keeps.
struct Run {
    recovered: RecoveredMethod,
    usage: UsageSnapshot,
    budget: Budget,
}

impl Run {
    fn report(&self) -> &RecoveryReport {
        self.recovered.recovery()
    }
}

/// The member every 5.2/5.3 case is about, and the second member the sequence changes to.
const DRIVER: (&[u8], &[u8]) = (b"scope", b"(Z)I");
const OTHER: (&[u8], &[u8]) = (b"simple", b"()I");
/// The member the assembled single-member fixtures declare.
const VALUE_MEMBER: (&[u8], &[u8]) = (b"value", b"()I");

fn scope_run(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    definition: &PhysicalDefinitionId,
    member: (&[u8], &[u8]),
    evidence: &RecoveryEvidenceRequest,
    limits: Limits,
) -> Run {
    let budget = Budget::new(limits);
    let mut budget = budget;
    let recovered = run_with(
        engine,
        snapshot,
        environment(snapshot),
        definition,
        member,
        evidence,
        &mut budget,
    );
    Run {
        recovered,
        usage: budget.usage(),
        budget,
    }
}

/// The artifact binding of one produced report, taken with the text it explains.
fn kept_text(run: &Run) -> String {
    run.report().text.clone()
}

/// A binding the caller keeps explains the very text it was published with, after every temporary of
/// the first request is gone and with a budget of its own.
#[test]
fn the_evidence_is_rebuilt_after_every_temporary_of_the_first_request_is_dropped() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definitions(&engine, &snapshot)[0].clone();

    let first = scope_run(
        &engine,
        &snapshot,
        &definition,
        DRIVER,
        &RecoveryEvidenceRequest::essential(),
        limits(),
    );
    let text = kept_text(&first);
    let kept_decisions = decisions(first.report());
    let method = first.recovered.facts().method().name().to_string();
    let binding = binding_of(&first.recovered);
    let first_usage = first.usage;
    // The report, the run's payload and its read all go away here: what survives is the value.
    drop(first.recovered);
    drop(first.budget);

    // A full selection over the same body **and the same budget shape**, with no expectation: the
    // work one request of this shape does. The request below must do all of it again.
    let before = d0_counts::snapshot();
    let first_all = scope_run(
        &engine,
        &snapshot,
        &definition,
        DRIVER,
        &RecoveryEvidenceRequest::all(),
        limits(),
    );
    let first_all_counted = before.since(d0_counts::snapshot());
    let first_all_usage = first_all.usage.clone();
    drop(first_all);

    let before = d0_counts::snapshot();
    let second = scope_run(
        &engine,
        &snapshot,
        &definition,
        DRIVER,
        &RecoveryEvidenceRequest::all().with_expected_artifact(binding.clone()),
        limits(),
    );
    let counted = before.since(d0_counts::snapshot());
    let report = second.report();
    assert!(
        matches!(report.artifact.agreement(), ArtifactAgreement::Agreed),
        "the rebuilt artifact is the one the caller named: {:?}",
        report.artifact.agreement()
    );
    assert_eq!(
        report.text, text,
        "the rebuilt text is byte for byte the one kept"
    );
    assert_eq!(
        decisions(report),
        kept_decisions,
        "and so are its decisions"
    );
    assert_eq!(second.recovered.facts().method().name(), method);
    assert_eq!(
        report.artifact.binding(),
        Some(&binding),
        "and the binding this run publishes states the same contract facts"
    );
    for kind in RecoveryEvidenceKind::SUPPORTED {
        assert_eq!(
            report.evidence.state(kind),
            EvidenceState::Complete,
            "the selection was answered: {kind:?}"
        );
    }
    assert!(
        !report.regions.is_empty() && !report.source_map.is_empty(),
        "the evidence really arrived: {} region(s), {} segment(s)",
        report.regions.len(),
        report.source_map.len()
    );
    // The rebuild is a rebuild: this request read the class, decoded the body and materialized its
    // own records — and it did exactly the work every first `all()` request of the same body does,
    // which is the count a delivery served from the first request's state would not produce.
    assert_eq!(counted.recovery_runs, 1);
    assert!(
        counted.class_materializations >= 1 && counted.body_decodes >= 1,
        "the second request read and decoded for itself: {counted:?}"
    );
    assert_eq!(
        counted, first_all_counted,
        "one request's evidence is not the next request's: two full selections do the same work"
    );
    assert!(
        second.usage.ir_items > first_usage.ir_items,
        "the evidence phase is charged to this request's own budget: {} against {}",
        second.usage.ir_items,
        first_usage.ir_items
    );
    assert_eq!(
        first_all_usage, second.usage,
        "and two requests of one shape state the same cost, whatever either one kept"
    );
    assert_eq!(
        kept_text(&second),
        text,
        "and the artifact is still the same text"
    );
    drop(second.recovered);
    drop(second.budget);
}

/// A text that is not the artifact the request named is a **mismatch**, stated dimension by
/// dimension — and never the "no evidence" a category that was not selected states.
#[test]
fn a_text_that_is_not_this_artifact_is_a_mismatch_and_never_a_missing_answer() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definitions(&engine, &snapshot)[0].clone();
    let first = scope_run(
        &engine,
        &snapshot,
        &definition,
        DRIVER,
        &RecoveryEvidenceRequest::essential(),
        limits(),
    );
    let binding = binding_of(&first.recovered);
    let text = kept_text(&first);
    drop(first);

    // The caller's own text: the same artifact, one byte longer than the one this run writes. A
    // binding that states only a length, only a name or only a bytecode index would answer this.
    let mut document = serde_json::to_value(&binding).expect("a binding serializes");
    let bytes = document["text_bytes"]
        .as_u64()
        .expect("the binding states how many bytes its text is");
    document["text_bytes"] = serde_json::json!(bytes + 1);
    let wrong: ArtifactBinding =
        serde_json::from_value(document).expect("the value is a binding document");

    let second = scope_run(
        &engine,
        &snapshot,
        &definition,
        DRIVER,
        &RecoveryEvidenceRequest::all().with_expected_artifact(wrong),
        limits(),
    );
    let report = second.report();
    let mismatch = mismatch_of(report);
    assert!(
        mismatch.disagrees_on(ArtifactDimension::Text),
        "the text is the dimension that changed: {:?}",
        mismatch.dimensions()
    );
    assert_eq!(
        mismatch.dimensions().len(),
        1,
        "and no other dimension did: {:?}",
        mismatch.dimensions()
    );
    assert_eq!(report.text, text, "this run's own artifact is delivered");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == ARTIFACT_MISMATCH_CODE),
        "the mismatch is a stated gap: {:?}",
        report.diagnostics
    );
    // A mismatch is not an absence: a category nobody selected says `NotRequested`, and a category
    // whose evidence was withheld by the verdict says `NotPerformed`.
    assert_eq!(
        report.evidence.state(RecoveryEvidenceKind::RegionDetails),
        EvidenceState::NotPerformed,
        "selected, and nothing of it was materialized"
    );
    let unrequested = scope_run(
        &engine,
        &snapshot,
        &definition,
        DRIVER,
        &RecoveryEvidenceRequest::essential(),
        limits(),
    );
    for kind in RecoveryEvidenceKind::SUPPORTED {
        assert_eq!(
            unrequested.report().evidence.state(kind),
            EvidenceState::NotRequested,
            "the two states are different statements: {kind:?}"
        );
    }
    assert!(
        report.regions.is_empty()
            && report.source_map.is_empty()
            && report.aliased_names.is_empty(),
        "nothing of the selection was attached to the caller's text"
    );
}

/// A budget that pays for the artifact and not for the evidence stops inside the phase: the artifact
/// stands, the categories state the prefix they reached, and the stop is the run's own.
#[test]
fn a_tight_budget_stops_the_rebuild_where_it_really_stops() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definitions(&engine, &snapshot)[0].clone();
    let first = scope_run(
        &engine,
        &snapshot,
        &definition,
        DRIVER,
        &RecoveryEvidenceRequest::essential(),
        limits(),
    );
    let binding = binding_of(&first.recovered);
    let text = kept_text(&first);
    let base = first.usage.ir_items;
    assert!(base > 0, "the artifact itself is work: {base}");
    drop(first);

    // One item more than the artifact costs: the phase may materialize exactly one record.
    let tight = Limits {
        ir_items: base + 1,
        ..limits()
    };
    let second = scope_run(
        &engine,
        &snapshot,
        &definition,
        DRIVER,
        &RecoveryEvidenceRequest::all().with_expected_artifact(binding),
        tight,
    );
    let report = second.report();
    assert!(
        report.produced(),
        "the artifact was committed before the phase"
    );
    assert_eq!(
        report.text, text,
        "and it is the text the caller's binding names"
    );
    assert!(
        report.artifact.attaches(),
        "the artifact agreed, so the phase really was entered"
    );
    // A produced artifact with a stopped evidence phase: the run's own artifact stands, the *phase*
    // is what stopped, and the two are stated apart (`outcome` against `execution`).
    assert!(
        matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::IrItems
                },
                ..
            }
        ),
        "the stop is the budget's own vocabulary: {:?}",
        report.execution
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_output_budget"),
        "and the stop is a stated gap: {:?}",
        report.diagnostics
    );
    assert_eq!(
        second.usage.ir_items,
        base + 1,
        "the phase stopped exactly at this request's own limit"
    );
    assert_eq!(
        report.evidence.state(RecoveryEvidenceKind::RegionDetails),
        EvidenceState::Partial { delivered: 1 },
        "the first category states the prefix it delivered"
    );
    for kind in [
        RecoveryEvidenceKind::RuleDetails,
        RecoveryEvidenceKind::NameDetails,
        RecoveryEvidenceKind::SourceMap,
    ] {
        assert_eq!(
            report.evidence.state(kind),
            EvidenceState::NotPerformed,
            "a category the phase never reached is not an empty delivery: {kind:?}"
        );
    }
    assert!(report.aliased_names.is_empty() && report.source_map.is_empty());
}

// ---------------------------------------------------------------------------------------------
// 5.3 — the sequence: essential, local evidence, all, another method, abandonment
// ---------------------------------------------------------------------------------------------

/// The one artifact of one method does not change as the evidence selection does, the binding holds
/// no payload of the run, and the only retention that outlives a request is the store the caller
/// declared.
#[test]
fn the_sequence_keeps_the_artifact_and_bounds_every_residency() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definitions(&engine, &snapshot)[0].clone();
    // A store with an explicit, small capacity: what the sequence may keep across its requests is
    // bounded by the declaration the caller made, and nothing else is kept.
    let capacity = FactsCapacity::new(3, 1 << 20);
    let store = FactsCache::current(capacity);
    let mut store_budget = new_budget().with_facts_cache(store.clone());

    // 1. Essential: the necessary results, and the binding the rest of the sequence explains.
    let essential = recover(
        &engine,
        &snapshot,
        &request(environment(&snapshot), &definition, DRIVER.0, DRIVER.1),
        &RecoveryEvidenceRequest::essential(),
        &mut store_budget,
    );
    let text = essential.recovery().text.clone();
    let kept_decisions = decisions(essential.recovery());
    let binding = binding_of(&essential);
    assert!(essential.recovery().produced());

    // 2. Local evidence: one driver BCI, the same artifact named as the one it explains.
    let local = RecoveryEvidenceRequest::essential()
        .with_kind(RecoveryEvidenceKind::RegionDetails)
        .with_driver_bci_range(BytecodeRange::new(0, 1))
        .with_expected_artifact(binding.clone());
    let mut local_budget = new_budget().with_facts_cache(store.clone());
    let local_run = recover(
        &engine,
        &snapshot,
        &request(environment(&snapshot), &definition, DRIVER.0, DRIVER.1),
        &local,
        &mut local_budget,
    );
    assert!(matches!(
        local_run.recovery().artifact.agreement(),
        ArtifactAgreement::Agreed
    ));
    assert_eq!(local_run.recovery().text, text);
    assert_eq!(decisions(local_run.recovery()), kept_decisions);
    assert_eq!(
        local_run.recovery().artifact.binding(),
        Some(&binding),
        "the binding does not move with the selection"
    );
    assert_eq!(
        local_run
            .recovery()
            .evidence
            .state(RecoveryEvidenceKind::RegionDetails),
        EvidenceState::Complete
    );
    assert!(
        local_run
            .recovery()
            .regions
            .iter()
            .all(|region| region.bci == 0 || region.blocks.contains(&0)),
        "the local selection states the regions it intersects: {:?}",
        local_run.recovery().regions
    );

    // 3. All: every category, the same artifact, the same text.
    let mut all_budget = new_budget().with_facts_cache(store.clone());
    let all = recover(
        &engine,
        &snapshot,
        &request(environment(&snapshot), &definition, DRIVER.0, DRIVER.1),
        &RecoveryEvidenceRequest::all().with_expected_artifact(binding.clone()),
        &mut all_budget,
    );
    assert!(matches!(
        all.recovery().artifact.agreement(),
        ArtifactAgreement::Agreed
    ));
    assert_eq!(all.recovery().text, text);
    assert_eq!(decisions(all.recovery()), kept_decisions);
    for kind in RecoveryEvidenceKind::SUPPORTED {
        assert_eq!(all.recovery().evidence.state(kind), EvidenceState::Complete);
    }
    assert_eq!(all.recovery().artifact.binding(), Some(&binding));

    // 4. Another method: the artifact the caller holds is not this one, and the sequence says so
    //    instead of attaching this method's evidence to the other method's text.
    let mut other_budget = new_budget().with_facts_cache(store.clone());
    let other = recover(
        &engine,
        &snapshot,
        &request(environment(&snapshot), &definition, OTHER.0, OTHER.1),
        &RecoveryEvidenceRequest::all().with_expected_artifact(binding.clone()),
        &mut other_budget,
    );
    let other_binding = mismatch_of(other.recovery());
    assert!(other_binding.disagrees_on(ArtifactDimension::Method));
    assert!(
        other.recovery().produced() && !other.recovery().text.is_empty(),
        "the other method's own artifact is delivered"
    );
    for kind in RecoveryEvidenceKind::SUPPORTED {
        assert_eq!(
            other.recovery().evidence.state(kind),
            EvidenceState::NotPerformed,
            "no evidence of the other method is attached to this text: {kind:?}"
        );
    }

    // 5. Abandonment: the caller asks for a listing and nothing else. No body is decoded, no
    //    presentation runs, and what the sequence kept is inside the declared store capacity.
    let before = d0_counts::snapshot();
    let listed = definitions(&engine, &snapshot);
    let counted = before.since(d0_counts::snapshot());
    assert_eq!(listed.len(), 1);
    assert!(
        counted.body_decodes == 0 && counted.recovery_runs == 0,
        "listing a scope decodes no body: {counted:?}"
    );
    let held = store.report();
    assert!(
        held.entries <= capacity.entries && held.retained_bytes <= capacity.retained_bytes,
        "the store stays inside the capacity it was opened with: {:?} against {:?}",
        held,
        capacity
    );
    println!(
        "sequence: text {} byte(s), store {} entries / {} byte(s) of {} allowed",
        text.len(),
        held.entries,
        held.retained_bytes,
        capacity.retained_bytes
    );
}

/// The member record a binding states is the position the read really established, and it is stated
/// per member: a presentation that prepares the class once states, for every member, the ordinal
/// that member's own record has in the class's table.
///
/// The class-source presentation is the entry where a member table is walked, so its members'
/// bindings are the ones a duplicate record could otherwise be conflated under one name.
#[test]
fn a_binding_states_the_member_record_the_read_established() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definitions(&engine, &snapshot)[0].clone();
    let request = ClassSourceRequest {
        class: ClassRef::Definition {
            definition: definition.clone(),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_string()),
        },
    };
    let mut budget = new_budget();
    let report = match engine
        .class_source(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("the class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("the fixture's definition is presented: {other:?}"),
    };
    let mut stated = Vec::new();
    for member in &report.methods {
        let ClassSourceOutcome::Recovered { report, .. } = &member.outcome else {
            continue;
        };
        let binding = report
            .artifact
            .binding()
            .expect("a member whose body a run presented has an artifact binding");
        assert_eq!(
            binding.member_ordinal(),
            Some(MethodOrdinal(
                u32::try_from(member.item.index).expect("the fixture's table fits u32")
            )),
            "the binding states the record this read located for `{}`",
            String::from_utf8_lossy(&member.item.name.raw().0)
        );
        assert_eq!(
            binding.method(),
            &member.item.identity,
            "and the physical identity this presentation runs the member under"
        );
        stated.push((
            binding.member_ordinal(),
            String::from_utf8_lossy(&binding.method().name.0).into_owned(),
        ));
    }
    assert!(
        stated.len() >= 2,
        "the fixture presents several bodies: {stated:?}"
    );
    let ordinals: Vec<Option<MethodOrdinal>> = stated.iter().map(|(ordinal, _)| *ordinal).collect();
    let mut distinct = ordinals.clone();
    distinct.sort();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        ordinals.len(),
        "two members are two records: {stated:?}"
    );
    assert!(
        stated.iter().any(|(_, name)| name == "scope"),
        "the member this file recovers by request is among them: {stated:?}"
    );
}

/// A run that committed no artifact compares nothing at all — a third answer, neither an agreement
/// nor a mismatch — and the categories it selected state that they were never performed rather than
/// arriving empty.
#[test]
fn a_run_without_an_artifact_compares_nothing() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(SCOPE);
    let definition = definitions(&engine, &snapshot)[0].clone();
    let first = scope_run(
        &engine,
        &snapshot,
        &definition,
        DRIVER,
        &RecoveryEvidenceRequest::essential(),
        limits(),
    );
    let binding = binding_of(&first.recovered);
    drop(first.recovered);
    drop(first.budget);

    // An output bound of zero refuses the write: the run stops before an artifact exists.
    let stopped = scope_run(
        &engine,
        &snapshot,
        &definition,
        DRIVER,
        &RecoveryEvidenceRequest::all().with_expected_artifact(binding),
        Limits {
            output_bytes: 0,
            ..limits()
        },
    );
    let report = stopped.report();
    assert!(!report.produced(), "{:?}", report.outcome);
    assert!(
        report.artifact.binding().is_none(),
        "no artifact, no binding: {:?}",
        report.artifact
    );
    let ArtifactAgreement::Unverifiable { reason } = report.artifact.agreement() else {
        panic!(
            "the verdict is neither an agreement nor a mismatch: {:?}",
            report.artifact.agreement()
        );
    };
    assert!(
        reason.contains("stopped"),
        "and it says why there was nothing to compare: {reason}"
    );
    assert!(report.text.is_empty() && report.source_map.is_empty());
    for kind in RecoveryEvidenceKind::SUPPORTED {
        assert_eq!(
            report.evidence.state(kind),
            EvidenceState::NotPerformed,
            "a selected category is `NotPerformed`, never an empty delivery: {kind:?}"
        );
    }
}

/// The binding is a value: no payload of the run can be inside it, and the report it travels in is
/// compared, cloned and serialized like the plan data it is.
///
/// The witnesses are three, and the runtime one is the rebuild rather than an `Arc` count: there is
/// no handle on the binding to count. Adding one does not merely fail an assertion — a binding that
/// held the run's payload stops the report's own derivations (`Eq`, `PartialEq`, `Serialize`) from
/// being satisfied, so the shape cannot be written down at all (verified by mutation: the compiler
/// refuses `Arc<MethodIr>` with `the trait bound MethodIr: Eq is not satisfied` and `Arc<MethodIr>:
/// Serialize is not satisfied`). What a run *can* be checked for is that holding the binding buys
/// nothing: the next request re-reads the class, re-decodes the body and materializes its own records
/// (`the_evidence_is_rebuilt_after_every_temporary_of_the_first_request_is_dropped`).
#[test]
fn the_binding_holds_no_payload_of_the_run() {
    let _gate = gate();
    /// The derivation the report and the binding both satisfy. `MethodIr`, the region tree and the
    /// built program satisfy none of these, so a binding that kept one of them would take the report
    /// out of this bound — the compiler is the witness, and the runtime witness is that a request
    /// that only holds a binding rebuilds everything it needs.
    fn assert_a_value<T: Clone + std::fmt::Debug + Eq + PartialEq + serde::Serialize>() {}
    assert_a_value::<RecoveryReport>();
    assert_a_value::<ArtifactBinding>();
    assert_a_value::<RecoveryArtifact>();
    assert_a_value::<ArtifactMismatch>();
    // Contract facts and a digest, not a table: the binding carries no text and no IR by value.
    let size = std::mem::size_of::<ArtifactBinding>();
    assert!(
        size < 1024,
        "the binding is a handful of identities and a digest, not a payload: {size} byte(s)"
    );
    let text = std::mem::size_of::<String>();
    assert!(size > text, "and it is not empty either: {size} byte(s)");
}
