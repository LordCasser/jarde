//! `add-task-oriented-operations`: one target, one bounded budget, one explicit environment.
//!
//! The contract this file holds is the change's four deltas, and every archive here is built in
//! memory by this file (the repository's own `rawzip` dev-dependency) or read from the committed
//! P3 samples, so a case can pin the exact containment, ambiguity, read count and stop it is about:
//!
//! * **one target selection.** A friendly name and an existing physical identity converge on one
//!   definition; an ambiguous name returns every candidate with its own identity and executes
//!   nothing; an identity of another snapshot is an input error and is never replaced by a
//!   same-named definition of the snapshot at hand (1.1, A07/A18).
//! * **operations pick their stages.** A task operation schedules a fixed table, publishes the set
//!   it used, and the same list passed explicitly to `Engine::analyze_method` reproduces the same
//!   schedule; the explicit entry point keeps its own validation and stop semantics (1.2, A16).
//! * **bounded default budgets, overridable by name.** Every counted dimension and the clock may be
//!   stated by name — a whole-package bulk caller states its own numbers instead of inheriting a
//!   view's — while the two high-water depths stay out of the set, and the effective `Limits` and
//!   the `UsageSnapshot` are published; a tight override really stops the work with the terminating
//!   dimension, an unknown dimension or a zero limit is an input error (1.3, A14).
//! * **three explicit environment policies.** Single class, plain JAR and explicit classpath build
//!   the declarations a caller would write by hand, and a Manifest `Class-Path`, a WAR/Boot layout
//!   or a nested library never generates a root; a layout policy is an explicit unsupported answer
//!   (2.1/2.2, A07/A08/A14).
//! * **one class read, on-demand bodies.** The class view charges one class header, one member walk
//!   and one body per requested method — never a header per method — and keeps `abstract`/`native`
//!   members and damaged member records isolated (3.1, A13/A16).
//! * **open stays light and repeats stay identical**, including with the optional facts cache
//!   attached (3.2, A15/A16/A17/A18).
//! * **references group by owner and keep the three derivation classes apart** (3.3, A01/A03/A11/A14).
//! * **recovery presents content first, then quality, then any stop**, read from the report's own
//!   fields and never from its text (3.4, A13/A14).

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::slice;

const STORE: u16 = 0;

/// The default task budget, as the library states it: bounded defaults, no override.
fn limits() -> Limits {
    task_limits(&[]).expect("the task defaults are a bounded budget")
}

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget())
        .expect("the fixture snapshot opens")
}

/// One snapshot opened under an explicit budget, so a case can assert what opening itself charged.
fn open_with(bytes: Vec<u8>, limits: Limits) -> (ArtifactSnapshot, UsageSnapshot) {
    let mut budget = Budget::new(limits);
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .expect("the fixture snapshot opens");
    (snapshot, budget.usage())
}

fn text(value: &[u8]) -> String {
    String::from_utf8_lossy(value).into_owned()
}

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

/// The one bound report of an operation, with a failure that names the candidates it got instead.
fn performed<T>(outcome: OperationOutcome<T>) -> T {
    match outcome {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "expected one bound target, got {} candidate(s) and no execution",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "expected one bound target, got an unfinished selection with {} candidate(s) and no \
             execution",
            candidates.candidates.len()
        ),
    }
}

/// The candidates of an ambiguous operation, with a failure when it executed instead.
fn ambiguous<T>(outcome: OperationOutcome<T>) -> TargetCandidates {
    match outcome {
        OperationOutcome::Ambiguous(candidates) => *candidates,
        OperationOutcome::Incomplete(candidates) => panic!(
            "expected an ambiguous name; the search did not finish with {} candidate(s) instead",
            candidates.candidates.len()
        ),
        OperationOutcome::Performed(_) => {
            panic!("expected candidates; the operation executed against a silently chosen target")
        }
    }
}

/// The confirmed prefix of an unfinished selection, with a failure when the operation bound one
/// identity anyway or refused the request.
fn incomplete<T>(outcome: OperationOutcome<T>) -> TargetCandidates {
    match outcome {
        OperationOutcome::Incomplete(candidates) => *candidates,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "expected an unfinished selection; the search bound {} candidate(s) as ambiguous \
             instead",
            candidates.candidates.len()
        ),
        OperationOutcome::Performed(_) => panic!(
            "expected an unfinished selection; the operation executed against a target an \
             unfinished search never bound"
        ),
    }
}

/// The usage one execution plane carries, at the moment it was assembled.
fn execution_usage(execution: &ExecutionReport) -> &UsageSnapshot {
    match execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage,
    }
}

fn codes(diagnostics: &[Diagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.clone())
        .collect()
}

fn code_of(error: &Error) -> String {
    match error {
        Error::InvalidInput { code, .. } | Error::Unsupported { code, .. } => code.clone(),
        Error::BudgetExceeded { dimension, .. } => format!("{dimension:?}"),
        Error::Cancelled { .. } => "cancelled".to_string(),
        Error::Io { operation, .. } => operation.clone(),
    }
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

/// No analysis plane moved: the four construction dimensions are zero. A class view decodes bodies,
/// so this is the check that says it started no pipeline; a selection additionally reads no body.
fn assert_no_ir(label: &str, usage: &UsageSnapshot) {
    let construction = construction(usage);
    assert!(
        construction.iter().all(|(_, count)| *count == 0),
        "{label}: no IR item, no edge, no step and no clone is built: {construction:?}"
    );
}

/// A selection reads no body and runs no analysis: the four construction dimensions, `method_bodies`
/// and `code_bytes` are untouched.
fn assert_no_analysis(label: &str, usage: &UsageSnapshot) {
    assert_no_ir(label, usage);
    assert_eq!(
        (usage.method_bodies, usage.code_bytes),
        (0, 0),
        "{label}: selecting a target decodes no body: {usage:?}"
    );
}

/// One JSON document with every `elapsed_millis` zeroed, so two runs of the same operation compare
/// on their facts and not on the wall clock.
fn facts_json<T: serde::Serialize>(value: &T) -> serde_json::Value {
    fn strip(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, entry) in map.iter_mut() {
                    if key == "elapsed_millis" {
                        *entry = serde_json::Value::from(0);
                    } else {
                        strip(entry);
                    }
                }
            }
            serde_json::Value::Array(items) => items.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let mut value = serde_json::to_value(value).expect("the report serializes");
    strip(&mut value);
    value
}

// ---------------------------------------------------------------------------------------------
// Fixtures: hand-built classes and archives, in the repository's own second fixture kind
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

/// One constant-pool `Class` entry pointing at a `Utf8` entry.
fn class_entry(pool: &mut Vec<Vec<u8>>, name_index: u16) -> u16 {
    let mut entry = vec![7];
    u16b(&mut entry, name_index);
    pool.push(entry);
    u16::try_from(pool.len()).expect("the fixture pool fits u16")
}

/// One constant-pool `Methodref` entry pointing at a `Class` entry and a `NameAndType`.
fn method_ref(pool: &mut Vec<Vec<u8>>, class_index: u16, name: u16, descriptor: u16) -> u16 {
    let mut name_and_type = vec![12];
    u16b(&mut name_and_type, name);
    u16b(&mut name_and_type, descriptor);
    pool.push(name_and_type);
    let name_and_type_index = u16::try_from(pool.len()).expect("the fixture pool fits u16");
    let mut entry = vec![10];
    u16b(&mut entry, class_index);
    u16b(&mut entry, name_and_type_index);
    pool.push(entry);
    u16::try_from(pool.len()).expect("the fixture pool fits u16")
}

/// One field or method record of a hand-built class.
struct MemberSpec<'a> {
    name: &'a [u8],
    descriptor: &'a [u8],
    access_flags: u16,
    code: Option<&'a [u8]>,
    declared_code_length: Option<u32>,
}

impl<'a> MemberSpec<'a> {
    /// A concrete method: `ACC_PUBLIC` and the given body bytes inside a real `Code` shell.
    fn code(name: &'a [u8], descriptor: &'a [u8], code: &'a [u8]) -> Self {
        Self {
            name,
            descriptor,
            access_flags: 0x0001,
            code: Some(code),
            declared_code_length: None,
        }
    }

    /// The default concrete method: `ACC_PUBLIC` and one `return`.
    fn method(name: &'a [u8], descriptor: &'a [u8]) -> Self {
        Self::code(name, descriptor, b"\xb1")
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
            // A complete `Code` attribute: max_stack, max_locals, the code array, an empty
            // exception table and no nested attribute, so every body here is one a real decode
            // reaches the end of. The damaged record states a length that is not this content's
            // own, which is what stops the member walk at that member.
            let content_length = 2 + 2 + 4 + code.len() + 2 + 2;
            u16b(bytes, 1);
            u16b(bytes, code_name);
            u32b(
                bytes,
                member.declared_code_length.unwrap_or_else(|| {
                    u32::try_from(content_length).expect("fixture code fits u32")
                }),
            );
            u16b(bytes, 2);
            u16b(bytes, 2);
            u32b(
                bytes,
                u32::try_from(code.len()).expect("fixture code fits u32"),
            );
            bytes.extend_from_slice(code);
            u16b(bytes, 0);
            u16b(bytes, 0);
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

/// The one root container a fresh snapshot establishes.
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

/// An archive holding the same class name at two origins: an upper directory entry and a nested
/// library, each declaring a different method, so a read of the wrong definition is visible.
fn two_origin_war(upper: &[u8], nested: &[u8]) -> Vec<u8> {
    let library = zip_of(&[(b"p/S.class", nested)]);
    zip_of(&[
        (b"WEB-INF/classes/p/S.class", upper),
        (b"WEB-INF/lib/L.jar", &library),
        (
            b"META-INF/MANIFEST.MF",
            b"Manifest-Version: 1.0\nMain-Class: p/S\n",
        ),
    ])
}

/// The candidate of one origin: the upper container's own entry, or the nested library's.
fn candidate_at_origin(candidates: &[ClassContentItem], nested: bool) -> PhysicalDefinitionId {
    candidates
        .iter()
        .find_map(|item| match item {
            ClassContentItem::ClassDeclaration(class) => {
                let origin = class.definition.entry().map(|entry| entry.origin.clone());
                match origin {
                    Some(origin) if origin.steps.is_empty() != nested => {
                        Some(class.definition.clone())
                    }
                    _ => None,
                }
            }
            ClassContentItem::Field(_) | ClassContentItem::Method(_) => None,
        })
        .unwrap_or_else(|| panic!("one candidate lives in a nested={nested} origin"))
}

fn method_names(report: &ClassViewReport) -> Vec<String> {
    report
        .methods()
        .map(|method| text(&method.identity.name.0))
        .collect()
}

/// The constants a hand-built call fixture needs, re-exported so the assertions read plainly.
const ACC_PUBLIC: u16 = 0x0001;

/// The environment declaration a request builds, compared field by field with a hand-written one.
fn hand_written_environment(
    snapshot: &ArtifactSnapshot,
    scope: PhysicalScope,
    roots: Vec<LoadRoot>,
) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots,
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope,
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

fn environment_request(
    snapshot: &ArtifactSnapshot,
    policy: EnvironmentPolicy,
) -> EnvironmentRequest {
    EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope: tree_scope(snapshot),
        policy,
        profile: RuntimeProfile {
            java_release: 8,
            multi_release: MultiReleasePolicy::Disabled,
            layout: LayoutMode::Generic,
        },
        loader: LoaderId("app".to_string()),
    }
}

// ---------------------------------------------------------------------------------------------
// 1.1 / A07 / A18: one target selection
// ---------------------------------------------------------------------------------------------

/// A friendly name and the identity it bound select the same definition, and the identity — never
/// the spelling — is what the report publishes.
#[test]
fn a_friendly_name_binds_the_identity_the_direct_request_uses() {
    let engine = Engine::new();
    let class = class_file(
        b"p/S",
        52,
        &[MemberSpec::field(b"value", b"I")],
        &[
            MemberSpec::method(b"run", b"()V"),
            MemberSpec::method(b"run", b"(I)V"),
        ],
    );
    let snapshot = open(zip_of(&[(b"p/S.class", &class)]));
    let scope = tree_scope(&snapshot);

    let by_name = performed(
        engine
            .class_view(
                &snapshot,
                &scope,
                &ClassViewRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::dotted("p.S"),
                    },
                    bodies: Vec::new(),
                },
                &mut budget(),
            )
            .expect("the named class view runs"),
    );
    let by_identity = performed(
        engine
            .class_view(
                &snapshot,
                &scope,
                &ClassViewRequest {
                    class: ClassRef::Definition {
                        definition: by_name.class.clone(),
                    },
                    bodies: Vec::new(),
                },
                &mut budget(),
            )
            .expect("the identity-addressed view runs"),
    );

    assert_eq!(by_name.class, by_identity.class);
    assert_eq!(by_name.items, by_identity.items);
    assert_eq!(method_names(&by_name), vec!["run", "run"]);
    assert_eq!(
        by_name.declaration().expect("the class item").binding,
        ClassNameBinding::PathNameAgrees
    );
    // The spelling is echoed for the query, and the identity is physical: a definition's own
    // location, bytes and variant, never the dotted name.
    assert_eq!(
        by_name.class.entry().expect("an archive entry").raw_name.0,
        b"p/S.class".to_vec()
    );
    assert!(by_name.class.class_bytes.length > 0);
}

/// Same name at two origins: the class view answers with both candidates and decodes nothing.
#[test]
fn an_ambiguous_class_name_returns_candidates_and_runs_nothing() {
    let engine = Engine::new();
    let upper = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"upper", b"()V")]);
    let nested = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"nested", b"()V")]);
    let snapshot = open(two_origin_war(&upper, &nested));
    let scope = tree_scope(&snapshot);
    let mut selection_budget = budget();

    let outcome = engine
        .class_view(
            &snapshot,
            &scope,
            &ClassViewRequest {
                class: ClassRef::Name {
                    class: ClassNameQuery::internal("p/S"),
                },
                bodies: Vec::new(),
            },
            &mut selection_budget,
        )
        .expect("the ambiguous search is an answer, not an error");
    let candidates = ambiguous(outcome);
    assert_eq!(candidates.candidates.len(), 2);
    assert_eq!(candidates.query.class.spelling(), "p/S");
    assert_eq!(
        candidates.limits,
        limits(),
        "an ambiguous answer publishes the effective configuration it ran under"
    );
    assert_no_analysis("an ambiguous class selection", &selection_budget.usage());
    // Both candidates are declarations of the requested name at their own physical origins.
    let upper_definition = candidate_at_origin(&candidates.candidates, false);
    let nested_definition = candidate_at_origin(&candidates.candidates, true);
    assert_ne!(upper_definition, nested_definition);
    assert_ne!(upper_definition.class_bytes, nested_definition.class_bytes);

    // Handing one candidate's own identity back binds exactly that definition: the members read
    // are the ones that origin declares, so "first wins" would be visible here. A fresh budget
    // makes the read count the identity-addressed request's own: one class header, no body.
    let mut chosen_budget = budget();
    let chosen = performed(
        engine
            .class_view(
                &snapshot,
                &scope,
                &ClassViewRequest {
                    class: ClassRef::Definition {
                        definition: nested_definition.clone(),
                    },
                    bodies: Vec::new(),
                },
                &mut chosen_budget,
            )
            .expect("the chosen identity reads that definition"),
    );
    assert_eq!(chosen.class, nested_definition);
    assert_eq!(method_names(&chosen), vec!["nested"]);
    assert_eq!(chosen.usage.class_headers, 1);
    assert_eq!(chosen.usage.method_bodies, 0);
    assert_no_analysis("the identity-addressed view", &chosen_budget.usage());
}

/// Same member name at several declared descriptors: the method operation answers with every
/// candidate and runs no analysis.
#[test]
fn an_ambiguous_member_name_returns_candidates_and_runs_nothing() {
    let engine = Engine::new();
    let class = class_file(
        b"p/S",
        52,
        &[],
        &[
            MemberSpec::method(b"run", b"()V"),
            MemberSpec::method(b"run", b"(I)V"),
        ],
    );
    let snapshot = open(zip_of(&[(b"p/S.class", &class)]));
    let request = MethodOperationRequest {
        method: MethodRef::Name {
            class: ClassNameQuery::dotted("p.S"),
            name: bytes(b"run"),
            descriptor: None,
        },
        environment: environment_request(&snapshot, EnvironmentPolicy::PlainJar),
    };
    let mut budget = budget();
    let outcome = engine
        .analyze_target(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("an ambiguous member name is an answer, not an error");
    let candidates = ambiguous(outcome);
    assert_eq!(candidates.candidates.len(), 2);
    let descriptors: Vec<String> = candidates
        .candidates
        .iter()
        .map(|item| match item {
            ClassContentItem::Method(method) => text(&method.identity.descriptor.0),
            _ => panic!("a method filter publishes method records"),
        })
        .collect();
    assert_eq!(descriptors, vec!["()V", "(I)V"]);
    assert_eq!(candidates.limits, limits());
    assert_no_analysis("an ambiguous member selection", &budget.usage());

    // With the identity the candidates carried, the same request binds exactly one target.
    let chosen = match &candidates.candidates[1] {
        ClassContentItem::Method(method) => method.identity.clone(),
        _ => unreachable!(),
    };
    let bound = performed(
        engine
            .analyze_target(
                slice::from_ref(&snapshot),
                &MethodOperationRequest {
                    method: MethodRef::Method {
                        method: chosen.clone(),
                    },
                    environment: request.environment.clone(),
                },
                &mut budget,
            )
            .expect("the identity-addressed analysis runs"),
    );
    assert_eq!(bound.method, chosen);
    assert_eq!(bound.analysis.stages.len(), 6);
}

/// An identity of another artifact is an input error — never a same-named definition of the
/// snapshot at hand.
#[test]
fn an_identity_of_another_snapshot_is_an_input_error() {
    let engine = Engine::new();
    let first = open(class_file(
        b"p/S",
        52,
        &[],
        &[MemberSpec::method(b"first", b"()V")],
    ));
    let second = open(class_file(
        b"p/S",
        52,
        &[],
        &[MemberSpec::method(b"second", b"()V")],
    ));
    assert_ne!(first.id(), second.id());

    let first_view = performed(
        engine
            .class_view(
                &first,
                &PhysicalScope::SnapshotAll,
                &ClassViewRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("p/S"),
                    },
                    bodies: Vec::new(),
                },
                &mut budget(),
            )
            .expect("the first snapshot holds the class"),
    );

    // The class view refuses the foreign definition instead of reading `second`'s same-named one.
    let error = engine
        .class_view(
            &second,
            &PhysicalScope::SnapshotAll,
            &ClassViewRequest {
                class: ClassRef::Definition {
                    definition: first_view.class.clone(),
                },
                bodies: Vec::new(),
            },
            &mut budget(),
        )
        .expect_err("a definition of another snapshot is an input error");
    assert_eq!(code_of(&error), "operation_target_snapshot_mismatch");

    // The method entry refuses the same way, before any environment work.
    let error = engine
        .analyze_target(
            slice::from_ref(&second),
            &MethodOperationRequest {
                method: MethodRef::Method {
                    method: PhysicalMethodId {
                        owner: first_view.class.clone(),
                        name: bytes(b"run"),
                        descriptor: bytes(b"()V"),
                    },
                },
                environment: environment_request(&second, EnvironmentPolicy::SingleClass),
            },
            &mut budget(),
        )
        .expect_err("a method identity of another snapshot is an input error");
    assert_eq!(code_of(&error), "operation_target_snapshot_mismatch");
}

/// A name the scope does not hold is an input error, not an empty report and not a silent miss.
#[test]
fn a_name_that_matches_nothing_is_an_input_error() {
    let engine = Engine::new();
    let class = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let snapshot = open(zip_of(&[(b"p/S.class", &class)]));
    let scope = tree_scope(&snapshot);
    let error = engine
        .class_view(
            &snapshot,
            &scope,
            &ClassViewRequest {
                class: ClassRef::Name {
                    class: ClassNameQuery::dotted("p/Absent"),
                },
                bodies: Vec::new(),
            },
            &mut budget(),
        )
        .expect_err("a class this scope does not hold is an input error");
    assert_eq!(code_of(&error), "operation_target_not_found");

    let error = engine
        .analyze_target(
            slice::from_ref(&snapshot),
            &MethodOperationRequest {
                method: MethodRef::Name {
                    class: ClassNameQuery::dotted("p.S"),
                    name: bytes(b"absent"),
                    descriptor: None,
                },
                environment: environment_request(&snapshot, EnvironmentPolicy::PlainJar),
            },
            &mut budget(),
        )
        .expect_err("a member this class does not declare is an input error");
    assert_eq!(code_of(&error), "operation_target_not_found");
}

/// A definition whose bytes do not match its identity is refused by the read, not read anyway.
#[test]
fn an_identity_that_does_not_match_its_bytes_is_refused() {
    let engine = Engine::new();
    let class = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let snapshot = open(zip_of(&[(b"p/S.class", &class)]));
    let scope = tree_scope(&snapshot);
    let view = performed(
        engine
            .class_view(
                &snapshot,
                &scope,
                &ClassViewRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("p/S"),
                    },
                    bodies: Vec::new(),
                },
                &mut budget(),
            )
            .expect("the named class view runs"),
    );
    let mut tampered = view.class.clone();
    tampered.class_bytes.digest = Digest("0".repeat(64));
    let error = engine
        .class_view(
            &snapshot,
            &scope,
            &ClassViewRequest {
                class: ClassRef::Definition {
                    definition: tampered,
                },
                bodies: Vec::new(),
            },
            &mut budget(),
        )
        .expect_err("the read verifies the identity it was given");
    assert_eq!(code_of(&error), "definition_class_bytes_mismatch");
}

// ---------------------------------------------------------------------------------------------
// 1.2 / A16: the operation picks its stages and publishes them
// ---------------------------------------------------------------------------------------------

/// The committed P3 sample the recovery cases below run over: `nestedPlain` is a body written
/// whole, `nestedLocal` keeps a statement beside a quote.
const NESTED_EVAL: &[u8] = include_bytes!("fixtures/p3-nested-eval/v8/NestedEval.class");

fn nested_eval() -> ArtifactSnapshot {
    open(NESTED_EVAL.to_vec())
}

/// The method identity one member of an opened class has, read through the class view's own listing.
fn member_identity(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    class: &str,
    name: &[u8],
    descriptor: Option<&[u8]>,
) -> PhysicalMethodId {
    let report = performed(
        engine
            .class_view(
                snapshot,
                &PhysicalScope::SnapshotAll,
                &ClassViewRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal(class),
                    },
                    bodies: Vec::new(),
                },
                &mut budget(),
            )
            .expect("the fixture's own class view runs"),
    );
    report
        .methods()
        .find(|method| {
            method.identity.name.0 == name
                && descriptor.is_none_or(|descriptor| method.identity.descriptor.0 == descriptor)
        })
        .map(|method| method.identity.clone())
        .unwrap_or_else(|| panic!("the fixture declares `{}`", text(name)))
}

/// The task operation publishes the stage list it scheduled, and that same list passed explicitly
/// reproduces the same schedule and the same stage results.
#[test]
fn the_operation_publishes_its_stages_and_explicit_stages_reproduce_the_schedule() {
    let engine = Engine::new();
    let snapshot = nested_eval();
    let method = member_identity(&engine, &snapshot, "NestedEval", b"nestedPlain", None);
    let request = MethodOperationRequest {
        method: MethodRef::Method {
            method: method.clone(),
        },
        environment: environment_request(&snapshot, EnvironmentPolicy::SingleClass),
    };
    let report = performed(
        engine
            .recover_target(slice::from_ref(&snapshot), &request, &mut budget())
            .expect("a legal task request is answered, not raised"),
    );
    assert_eq!(report.operation, MethodOperation::Recovery);
    assert_eq!(report.method, method);
    assert_eq!(report.stages, AnalysisStage::ALL.to_vec());
    assert_eq!(report.limits, limits());
    assert!(report.usage.method_bodies > 0);

    // The same list through the low-level entry reproduces the schedule, stage result by stage
    // result — the operation is not a second pipeline.
    let environment = request
        .environment
        .build(slice::from_ref(&snapshot))
        .expect("the single-class policy builds");
    let explicit = engine
        .analyze_method(
            slice::from_ref(&snapshot),
            &MethodAnalysisRequest {
                environment: environment.clone(),
                method: method.clone(),
                stages: report.stages.clone(),
            },
            &mut budget(),
        )
        .expect("a legal explicit request is answered, not raised");
    assert_eq!(explicit.stages, report.recovered.analysis().stages);
    assert_eq!(
        explicit.requested_stages,
        report.recovered.analysis().requested_stages
    );
    assert_eq!(explicit.stages.len(), 6);

    // The analysis operation publishes the same table and the same schedule.
    let analyzed = performed(
        engine
            .analyze_target(slice::from_ref(&snapshot), &request, &mut budget())
            .expect("a legal task request is answered, not raised"),
    );
    assert_eq!(analyzed.operation, MethodOperation::Analysis);
    assert_eq!(analyzed.stages, report.stages);
    assert_eq!(analyzed.analysis.stages, explicit.stages);
}

/// The explicit stage list stays the low-level control: a shorter list schedules a shorter prefix,
/// and the operation's own table is still published beside it.
#[test]
fn an_explicit_short_stage_list_keeps_its_own_schedule() {
    let engine = Engine::new();
    let snapshot = nested_eval();
    let method = member_identity(&engine, &snapshot, "NestedEval", b"nestedPlain", None);
    let environment = environment_request(&snapshot, EnvironmentPolicy::SingleClass)
        .build(slice::from_ref(&snapshot))
        .expect("the single-class policy builds");

    let raw_only = engine
        .analyze_method(
            slice::from_ref(&snapshot),
            &MethodAnalysisRequest {
                environment: environment.clone(),
                method: method.clone(),
                stages: vec![AnalysisStage::RawFacts],
            },
            &mut budget(),
        )
        .expect("one requested stage is a legal request");
    assert_eq!(raw_only.requested_stages, vec![AnalysisStage::RawFacts]);
    assert_eq!(raw_only.stages.len(), 1);
    assert_eq!(raw_only.stages[0].state, StageState::Completed);
    assert_eq!(raw_only.quality, Quality::Fallback);

    // The deepest stage is what expands the prerequisites, which is the schedule the operation
    // publishes and no request can shorten behind its back.
    let deepest = engine
        .analyze_method(
            slice::from_ref(&snapshot),
            &MethodAnalysisRequest {
                environment,
                method,
                stages: vec![AnalysisStage::Ssa],
            },
            &mut budget(),
        )
        .expect("the deepest stage is a legal request");
    assert_eq!(deepest.requested_stages, vec![AnalysisStage::Ssa]);
    assert_eq!(deepest.stages.len(), 6);
    assert_eq!(
        deepest.stages,
        AnalysisStage::ALL
            .into_iter()
            .map(|stage| StageResult {
                stage,
                state: StageState::Completed,
            })
            .collect::<Vec<_>>()
    );
    // The explicitly requested prefix is the same prefix of that schedule: nothing about the
    // operation's table changed what an explicit request means.
    assert_eq!(raw_only.stages, deepest.stages[..1].to_vec());
    // An empty stage list keeps its own input-error contract.
    let error = engine
        .analyze_method(
            slice::from_ref(&snapshot),
            &MethodAnalysisRequest {
                environment: environment_request(&snapshot, EnvironmentPolicy::SingleClass)
                    .build(slice::from_ref(&snapshot))
                    .expect("the single-class policy builds"),
                method: member_identity(&engine, &snapshot, "NestedEval", b"nestedPlain", None),
                stages: Vec::new(),
            },
            &mut budget(),
        )
        .expect_err("an empty stage list is an input error");
    assert_eq!(code_of(&error), "analysis_no_stages");
}

// ---------------------------------------------------------------------------------------------
// 1.3 / A14: bounded defaults, a few overrides, the effective configuration published
// ---------------------------------------------------------------------------------------------

/// Every dimension of the default task budget is bounded, and one override replaces exactly one.
#[test]
fn the_default_budget_is_bounded_and_overrides_replace_one_dimension() {
    let defaults = limits();
    for dimension in CountedBudgetDimension::ALL {
        assert!(
            defaults.counted_limit(dimension) > 0,
            "the default budget funds {dimension:?}: {defaults:?}"
        );
    }
    assert!(defaults.nested_depth > 0 && defaults.dependency_depth > 0);
    assert!(
        defaults.elapsed_millis > 0 && defaults.elapsed_millis < u64::MAX,
        "the wall clock is bounded too: {defaults:?}"
    );

    // Every counted dimension may be stated by name, and the clock with them: the set is the
    // reader's own counted set in its own order plus `elapsed_millis` last. The two high-water
    // depths are deliberately outside it — raising a depth decides which containers and
    // dependencies a request may walk at all, which its scope and its roots declare, not how much
    // work it may do.
    assert_eq!(
        OVERRIDABLE_BUDGET_DIMENSIONS.len(),
        CountedBudgetDimension::ALL.len() + 1
    );
    for (index, dimension) in CountedBudgetDimension::ALL.iter().enumerate() {
        assert_eq!(
            OVERRIDABLE_BUDGET_DIMENSIONS[index],
            budget_dimension_code((*dimension).into()),
            "the list holds every counted dimension, in the reader's own order"
        );
    }
    assert_eq!(
        OVERRIDABLE_BUDGET_DIMENSIONS[CountedBudgetDimension::ALL.len()],
        budget_dimension_code(BudgetDimension::ElapsedMillis),
        "the clock is the one non-counted dimension a request may state"
    );

    let overrides = [
        BudgetOverride::new("class_headers", 3).expect("a named dimension"),
        BudgetOverride::new("output_bytes", 1 << 12).expect("a named dimension"),
    ];
    let run_overrides = overrides;
    let effective = task_limits(&overrides).expect("the overrides are legal");
    assert_eq!(effective.class_headers, 3);
    assert_eq!(effective.output_bytes, 1 << 12);
    for dimension in CountedBudgetDimension::ALL {
        if matches!(
            dimension,
            CountedBudgetDimension::ClassHeaders | CountedBudgetDimension::OutputBytes
        ) {
            continue;
        }
        assert_eq!(
            effective.counted_limit(dimension),
            defaults.counted_limit(dimension),
            "an override leaves {dimension:?} at its bounded default"
        );
    }
    assert_eq!(effective.elapsed_millis, defaults.elapsed_millis);

    // The operation publishes the complete effective configuration it ran under.
    let engine = Engine::new();
    let class = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let snapshot = open(zip_of(&[(b"p/S.class", &class)]));
    let mut budget = task_budget(&run_overrides).expect("a legal task budget");
    let report = performed(
        engine
            .class_view(
                &snapshot,
                &tree_scope(&snapshot),
                &ClassViewRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("p/S"),
                    },
                    bodies: Vec::new(),
                },
                &mut budget,
            )
            .expect("a legal task request is answered"),
    );
    assert_eq!(report.limits, effective);
    assert_eq!(report.limits.class_headers, 3);
    assert_eq!(report.usage.class_headers, 1);
    assert_eq!(budget.limits(), &effective);
}

/// An unknown dimension or a zero limit is an input error — never a silent default.
#[test]
fn an_unknown_or_zero_override_is_an_input_error() {
    // `input_bytes` is a counted dimension like every other one, so naming it is legal: what the fix
    // keeps is that a name *outside* the countable set is refused rather than ignored.
    assert_eq!(
        BudgetOverride::new("input_bytes", 10)
            .expect("a counted dimension a task request may state")
            .dimension_code(),
        "input_bytes"
    );
    assert_eq!(
        code_of(&BudgetOverride::new("nonsense", 10).expect_err("not a dimension at all")),
        "budget_override_dimension_unknown"
    );
    // Neither depth is a count: a request that wants to walk deeper says so in its scope and in its
    // roots, and a limit named like a budget is refused rather than silently widening the walk.
    for depth in ["nested_depth", "dependency_depth"] {
        assert_eq!(
            code_of(&BudgetOverride::new(depth, 8).expect_err("a depth is not a count")),
            "budget_override_dimension_unknown",
            "`{depth}` is not a budget a task request tunes"
        );
    }
    assert_eq!(
        code_of(&BudgetOverride::new("output_bytes", 0).expect_err("zero funds nothing")),
        "budget_override_invalid"
    );
    // The effective configuration refuses a hand-built zero override the same way, so no path
    // reaches a run with a dimension silently left at its default.
    let zero = BudgetOverride::ClassHeaders { limit: 0 };
    assert_eq!(
        code_of(&task_limits(&[zero]).expect_err("zero is refused where the budget is built")),
        "budget_override_invalid"
    );
    assert!(task_budget(&[BudgetOverride::new("result_items", 5).expect("named")]).is_ok());
}

/// Every counted dimension and the clock can be stated by name, and each one replaces exactly the
/// field it names — the shape a whole-package bulk caller builds its own default set out of.
#[test]
fn every_counted_dimension_and_the_clock_can_be_overridden_by_name() {
    let defaults = limits();
    for dimension in CountedBudgetDimension::ALL {
        let name = budget_dimension_code(dimension.into());
        let over = BudgetOverride::new(name, 7)
            .unwrap_or_else(|error| panic!("`{name}` is a dimension a request may state: {error}"));
        assert_eq!(over.dimension_code(), name, "`{name}` names itself back");
        assert_eq!(over.limit(), 7);
        let effective = task_limits(&[over]).expect("one legal override");
        assert_eq!(
            effective.counted_limit(dimension),
            7,
            "`{name}` replaces its own field"
        );
        for other in CountedBudgetDimension::ALL {
            if other == dimension {
                continue;
            }
            assert_eq!(
                effective.counted_limit(other),
                defaults.counted_limit(other),
                "`{name}` leaves {other:?} at its bounded default"
            );
        }
        assert_eq!(effective.elapsed_millis, defaults.elapsed_millis);
        assert_eq!(effective.nested_depth, defaults.nested_depth);
        assert_eq!(effective.dependency_depth, defaults.dependency_depth);
    }

    // The clock is the one non-counted dimension in the set: it replaces its own field and no
    // counted one.
    let effective = task_limits(&[BudgetOverride::new("elapsed_millis", 7).expect("the clock")])
        .expect("one legal override");
    assert_eq!(effective.elapsed_millis, 7);
    for dimension in CountedBudgetDimension::ALL {
        assert_eq!(
            effective.counted_limit(dimension),
            defaults.counted_limit(dimension),
            "the clock leaves {dimension:?} at its bounded default"
        );
    }

    // And zero is refused for every name of the set, through the very constructor the CLI's
    // `--budget` parameter uses: no name of the set is a way to reach a degenerate budget.
    for name in OVERRIDABLE_BUDGET_DIMENSIONS {
        assert_eq!(
            code_of(
                &BudgetOverride::new(name, 0)
                    .expect_err("a dimension that funds nothing is refused")
            ),
            "budget_override_invalid",
            "`{name}=0`"
        );
    }
}

/// A tight override really truncates the work and the report states the dimension that stopped it.
#[test]
fn a_tight_override_stops_with_the_terminating_dimension() {
    let engine = Engine::new();
    let upper = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"upper", b"()V")]);
    let nested = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"nested", b"()V")]);
    let snapshot = open(two_origin_war(&upper, &nested));
    let scope = tree_scope(&snapshot);

    // One header attempt over a scope with two candidates of the name: the first binds and the
    // second charge is refused, so the search did not finish. One confirmed candidate is not a
    // unique target and the zero-or-one rule does not apply: the request answers with that
    // candidate, the search's own stop and the usage it really cost, and runs nothing.
    let tight = task_limits(&[BudgetOverride::new("class_headers", 1).expect("named")])
        .expect("a legal override");
    let mut tight_budget = Budget::new(tight.clone());
    let stopped = incomplete(
        engine
            .class_view(
                &snapshot,
                &scope,
                &ClassViewRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("p/S"),
                    },
                    bodies: Vec::new(),
                },
                &mut tight_budget,
            )
            .expect("a stopped request is a report, not an error"),
    );
    assert_eq!(stopped.limits, tight);
    assert_eq!(
        stopped.candidates.len(),
        1,
        "the confirmed prefix travels with the stop: {:?}",
        stopped.candidates
    );
    assert_eq!(
        stopped.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ClassHeaders,
            },
            usage: execution_usage(&stopped.execution).clone(),
        }
    );
    assert_eq!(
        execution_usage(&stopped.execution).class_headers,
        1,
        "the stop carries the usage the search really cost: {:?}",
        stopped.execution
    );
    assert!(codes(&stopped.diagnostics).contains(&"budget_exceeded_class_headers".to_string()));
    assert_no_analysis("an unfinished class selection", &tight_budget.usage());

    // The confirmed candidate is usable: feeding its own physical identity back reads exactly that
    // definition, which is how a caller continues from an unfinished search.
    let ClassContentItem::ClassDeclaration(confirmed) = &stopped.candidates[0] else {
        panic!("a class search publishes class declarations")
    };
    let chosen = performed(
        engine
            .class_view(
                &snapshot,
                &scope,
                &ClassViewRequest {
                    class: ClassRef::Definition {
                        definition: confirmed.definition.clone(),
                    },
                    bodies: Vec::new(),
                },
                &mut budget(),
            )
            .expect("the confirmed identity reads"),
    );
    assert_eq!(method_names(&chosen), vec!["upper"]);

    // One body attempt with two bodies asked for: the first is decoded, the second attempt is
    // refused, and the second body is not invented.
    let class = class_file(
        b"p/Bodies",
        52,
        &[],
        &[
            MemberSpec::method(b"one", b"()V"),
            MemberSpec::method(b"two", b"()V"),
        ],
    );
    let body_snapshot = open(zip_of(&[(b"p/Bodies.class", &class)]));
    let bodies_view = performed(
        engine
            .class_view(
                &body_snapshot,
                &tree_scope(&body_snapshot),
                &ClassViewRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("p/Bodies"),
                    },
                    bodies: Vec::new(),
                },
                &mut budget(),
            )
            .expect("the class view of the body fixture runs"),
    );
    let one = bodies_view
        .methods()
        .next()
        .expect("the first member")
        .identity
        .clone();
    let two = bodies_view
        .methods()
        .nth(1)
        .expect("the second member")
        .identity
        .clone();
    let tight = task_limits(&[BudgetOverride::new("method_bodies", 1).expect("named")])
        .expect("a legal override");
    let mut tight_budget = Budget::new(tight.clone());
    let report = performed(
        engine
            .class_view(
                &body_snapshot,
                &tree_scope(&body_snapshot),
                &ClassViewRequest {
                    class: ClassRef::Definition {
                        definition: bodies_view.class.clone(),
                    },
                    bodies: vec![
                        BodyRef::Method { method: one },
                        BodyRef::Method { method: two },
                    ],
                },
                &mut tight_budget,
            )
            .expect("a stopped request is a report, not an error"),
    );
    assert_eq!(report.usage.method_bodies, 1);
    assert_eq!(
        report.bodies.len(),
        1,
        "the refusal ends the request: the bodies after it are never started"
    );
    assert_eq!(
        report.bodies[0]
            .method()
            .map(|method| method.name.0.clone()),
        Some(b"one".to_vec()),
        "the one body that was started is the first requested one"
    );
    assert!(
        matches!(report.bodies[0], ClassViewBody::Read { .. }),
        "the first body is the read one: {:?}",
        report.bodies[0]
    );
    assert_eq!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::MethodBodies,
            },
            usage: execution_usage(&report.execution).clone(),
        }
    );
    assert!(codes(&report.diagnostics).contains(&"budget_exceeded_method_bodies".to_string()));

    // A tight output budget stops the analysis of a standalone class before its body is read, and
    // the report says so instead of claiming a completed run.
    let standalone = nested_eval();
    let tight = task_limits(&[BudgetOverride::new("output_bytes", 8).expect("named")])
        .expect("a legal override");
    let mut budget = Budget::new(tight.clone());
    let analyzed = performed(
        engine
            .analyze_target(
                slice::from_ref(&standalone),
                &MethodOperationRequest {
                    method: MethodRef::Method {
                        method: member_identity(
                            &engine,
                            &standalone,
                            "NestedEval",
                            b"nestedPlain",
                            None,
                        ),
                    },
                    environment: environment_request(&standalone, EnvironmentPolicy::SingleClass),
                },
                &mut budget,
            )
            .expect("a stopped request is a report, not an error"),
    );
    assert_eq!(analyzed.limits, tight);
    assert_eq!(
        analyzed.analysis.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::OutputBytes,
            },
            usage: execution_usage(&analyzed.analysis.execution).clone(),
        }
    );
    assert_eq!(analyzed.analysis.body, MethodBodyState::NotInspected);
}

// ---------------------------------------------------------------------------------------------
// 2.1 / A07 / A08: the three environment policies
// ---------------------------------------------------------------------------------------------

/// The single-class policy builds the declaration a caller would write by hand, and the engine's
/// own validator accepts it.
#[test]
fn the_single_class_policy_declares_its_own_root() {
    let class = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let snapshot = open(class);
    let request = environment_request(&snapshot, EnvironmentPolicy::SingleClass);
    let environment = request
        .build(slice::from_ref(&snapshot))
        .expect("a standalone class is what the policy names");
    assert_eq!(
        environment,
        hand_written_environment(
            &snapshot,
            tree_scope(&snapshot),
            vec![LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone()
            }],
        )
    );
    let (problems, identity) = validate_environment(slice::from_ref(&snapshot), &environment);
    assert!(
        problems.is_empty(),
        "the built declaration validates: {problems:?}"
    );
    assert_eq!(identity.domain_loaders, vec![LoaderId("app".to_string())]);
    assert!(identity.providers.is_empty());

    // The policy refuses a snapshot of another kind instead of inventing a container identity for
    // it: a standalone root is not fabricated for a ZIP.
    let jar = open(zip_of(&[(
        b"p/S.class",
        &class_file(b"p/S", 52, &[], &[MemberSpec::method(b"run", b"()V")]),
    )]));
    let error = environment_request(&jar, EnvironmentPolicy::SingleClass)
        .build(slice::from_ref(&jar))
        .expect_err("a ZIP is not a standalone class");
    assert_eq!(code_of(&error), "environment_policy_snapshot_kind_mismatch");
}

/// The plain-JAR policy roots the snapshot's own root container and activates no nested library.
#[test]
fn the_plain_jar_policy_activates_no_nested_library() {
    let engine = Engine::new();
    let class = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let nested = class_file(b"p/Nested", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let library = zip_of(&[(b"p/Nested.class", &nested)]);
    let jar = zip_of(&[(b"p/S.class", &class), (b"lib/L.jar", &library)]);
    let snapshot = open(jar);
    let environment = environment_request(&snapshot, EnvironmentPolicy::PlainJar)
        .build(slice::from_ref(&snapshot))
        .expect("a ZIP is what the policy names");

    assert_eq!(
        environment.domains[0].roots,
        vec![LoadRoot::Container {
            origin: root_origin(&snapshot),
            prefix: ArchiveNameBytes(Vec::new()),
        }]
    );
    assert_eq!(
        environment.domains[0].delegation,
        DelegationPolicy::ParentFirst
    );
    assert_eq!(environment.domains[0].module_mode, ModuleMode::ClassPath);
    assert!(environment.providers.is_empty());
    let (problems, _) = validate_environment(slice::from_ref(&snapshot), &environment);
    assert!(
        problems.is_empty(),
        "the built declaration validates: {problems:?}"
    );

    // The nested library really holds `p/Nested` — the tree walk reaches it — and the policy still
    // does not activate it: no root names that container, and a lookup does not find the class.
    let tree = engine
        .enumerate_artifact_tree(&snapshot, &mut budget())
        .expect("the artifact tree walks");
    let nested_entries: Vec<String> = tree
        .containers
        .iter()
        .filter(|container| !container.origin.steps.is_empty())
        .flat_map(|container| container.entries.iter())
        .map(|entry| text(&entry.id.raw_name.0))
        .collect();
    assert!(
        nested_entries.iter().any(|name| name == "p/Nested.class"),
        "the nested library is there: {nested_entries:?}"
    );

    let resolution = engine
        .resolve_symbol(
            slice::from_ref(&snapshot),
            &ResolutionRequest {
                environment: environment.clone(),
                target: SymbolRef::Class {
                    owner: bytes(b"p/Nested"),
                },
                use_kind: ReferenceUse::ClassReference,
                caller: CallerContext {
                    loader: LoaderId("app".to_string()),
                    enclosing: None,
                },
                dispatch: None,
            },
            &mut budget(),
        )
        .expect("a legal resolution request is answered");
    assert_eq!(resolution.analysis, ResolutionAnalysis::Performed);
    assert_eq!(resolution.state, Some(ResolutionState::Missing));
    assert!(resolution.resolved.is_none());
    assert!(
        resolution.reads.is_empty(),
        "nothing was read for a name no root position holds"
    );
    // The root position was searched as a whole, so the negation is `Missing` and not a partial
    // answer: the nested library is not part of the searched order at all.
    assert_ne!(
        resolution.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
}

/// An explicit classpath decides by declaration order, and several same-named definitions at
/// different roots are not an ambiguity.
#[test]
fn the_explicit_classpath_order_decides() {
    let engine = Engine::new();
    let first = open(class_file(
        b"p/S",
        52,
        &[],
        &[MemberSpec::method(b"first", b"()V")],
    ));
    let second = open(class_file(
        b"p/S",
        52,
        &[],
        &[MemberSpec::method(b"second", b"()V")],
    ));
    let content = [first.clone(), second.clone()];
    let root = |snapshot: &ArtifactSnapshot| LoadRoot::StandaloneClass {
        snapshot: snapshot.id().clone(),
    };
    let policy = |roots: Vec<LoadRoot>| {
        let mut request =
            environment_request(&first, EnvironmentPolicy::ExplicitClasspath { roots });
        request.snapshot = first.id().clone();
        request
    };
    let resolve = |environment: &ResolutionEnvironment| {
        engine
            .resolve_symbol(
                &content,
                &ResolutionRequest {
                    environment: environment.clone(),
                    target: SymbolRef::Class {
                        owner: bytes(b"p/S"),
                    },
                    use_kind: ReferenceUse::ClassReference,
                    caller: CallerContext {
                        loader: LoaderId("app".to_string()),
                        enclosing: None,
                    },
                    dispatch: None,
                },
                &mut budget(),
            )
            .expect("a legal resolution request is answered")
    };

    let ordered = policy(vec![root(&first), root(&second)])
        .build(&content)
        .expect("the classpath builds");
    let report = resolve(&ordered);
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    let resolved = report.resolved.expect("the first root decided");
    assert_eq!(resolved.definition, definition_of_snapshot(&first));
    assert_ne!(report.state, Some(ResolutionState::Ambiguous));

    let swapped = policy(vec![root(&second), root(&first)])
        .build(&content)
        .expect("the classpath builds");
    let report = resolve(&swapped);
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        report
            .resolved
            .expect("the first declared root decided")
            .definition,
        definition_of_snapshot(&second)
    );
    assert_ne!(
        definition_of_snapshot(&first),
        definition_of_snapshot(&second)
    );
}

/// The standalone definition of one snapshot, as its own root read states it.
fn definition_of_snapshot(snapshot: &ArtifactSnapshot) -> PhysicalDefinitionId {
    performed(
        Engine::new()
            .class_view(
                snapshot,
                &PhysicalScope::SnapshotAll,
                &ClassViewRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal(String::from("p/S")),
                    },
                    bodies: Vec::new(),
                },
                &mut budget(),
            )
            .expect("the fixture declares p/S"),
    )
    .class
}

// ---------------------------------------------------------------------------------------------
// 3.1 / A13 / A16: the class view reads one header, one listing and the requested bodies
// ---------------------------------------------------------------------------------------------

fn bodies_class() -> Vec<u8> {
    class_file(
        b"p/Bodies",
        52,
        &[MemberSpec::field(b"value", b"I")],
        &[
            MemberSpec::method(b"one", b"()V"),
            MemberSpec::method(b"two", b"()V"),
            MemberSpec::abstract_method(b"abstractOne", b"()V"),
            MemberSpec::native_method(b"nativeOne", b"()V"),
        ],
    )
}

/// The class view charges one header, one member walk and one body per requested concrete method —
/// and reads no other class of the archive.
#[test]
fn the_class_view_reads_one_header_one_listing_and_only_the_requested_bodies() {
    let engine = Engine::new();
    let other = class_file(b"p/T", 52, &[], &[MemberSpec::method(b"other", b"()V")]);
    let snapshot = open(zip_of(&[
        (b"p/Bodies.class", &bodies_class()),
        (b"p/T.class", &other),
    ]));
    let scope = tree_scope(&snapshot);
    let mut budget = budget();
    let report = performed(
        engine
            .class_view(
                &snapshot,
                &scope,
                &ClassViewRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::dotted("p.Bodies"),
                    },
                    bodies: vec![
                        BodyRef::Name {
                            name: bytes(b"one"),
                            descriptor: None,
                        },
                        BodyRef::Name {
                            name: bytes(b"abstractOne"),
                            descriptor: None,
                        },
                        BodyRef::Name {
                            name: bytes(b"nativeOne"),
                            descriptor: None,
                        },
                    ],
                },
                &mut budget,
            )
            .expect("the class view runs"),
    );

    // One header read for the whole view, and one body attempt: the abstract and native members
    // are declarations without a `Code`, so neither is attempted.
    assert_eq!(report.usage.class_headers, 1, "{:?}", report.usage);
    assert_eq!(report.usage.method_bodies, 1, "{:?}", report.usage);
    assert!(report.usage.code_bytes > 0);
    assert_no_ir("the class view", &budget.usage());

    // The same view without the body. What a body adds is its own counters *and* the one
    // preparation of the class it is decoded against (D2 3.2): the view's class read is the same
    // read whatever it asks for — the header attempt, the materialized bytes and the member walk do
    // not move — and `class_bytes` grows by exactly one parse of that same read, the preparation
    // that serves every requested body instead of a per-body `Class::new`.
    let mut without_budget = task_budget(&[]).expect("the task defaults are bounded");
    let without = performed(
        engine
            .class_view(
                &snapshot,
                &scope,
                &ClassViewRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::dotted("p.Bodies"),
                    },
                    bodies: Vec::new(),
                },
                &mut without_budget,
            )
            .expect("the same class view without bodies runs"),
    );
    assert_eq!(report.usage.class_headers, without.usage.class_headers);
    assert_eq!(
        report.usage.class_bytes,
        without.usage.class_bytes + report.class.class_bytes.length,
        "the body the view decoded was located and decoded through one preparation of the class it \
         read (D2 3.2), so exactly one parse of that class is added: {:?} vs {:?}",
        report.usage,
        without.usage
    );
    assert_eq!(report.usage.read_bytes, without.usage.read_bytes);
    assert!(
        report.usage.result_items > without.usage.result_items,
        "the body result is an item of its own: {:?} vs {:?}",
        report.usage,
        without.usage
    );
    assert_eq!(without.usage.method_bodies, 0);
    assert_eq!(without.usage.code_bytes, 0);

    // The declaration, the field and every method of that one read are published.
    assert_eq!(
        report
            .declaration()
            .expect("the class item")
            .declaration
            .this_class
            .raw()
            .0,
        b"p/Bodies"
    );
    assert_eq!(report.fields().count(), 1);
    assert_eq!(
        method_names(&report),
        vec!["one", "two", "abstractOne", "nativeOne"]
    );

    // The requested body is decoded from the same bytes, with the reader's own two phases.
    let ClassViewBody::Read {
        method,
        stages,
        instructions,
        coverage,
        execution,
        ..
    } = &report.bodies[0]
    else {
        panic!(
            "the requested concrete body is read: {:?}",
            report.bodies[0]
        );
    };
    assert_eq!(method.name.0, b"one");
    assert_eq!(
        stages,
        &vec![
            BodyStageResult {
                phase: BytecodeStopPhase::ExceptionHandlers,
                state: BodyStageState::Completed,
            },
            BodyStageResult {
                phase: BytecodeStopPhase::Instructions,
                state: BodyStageState::Completed,
            },
        ]
    );
    assert_eq!(instructions.len(), 1);
    assert_eq!(instructions[0].opcode, 0xb1);
    assert_eq!(instructions[0].bci, 0);
    assert_eq!(
        coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(matches!(execution, ExecutionReport::Complete { .. }));

    // `abstract` and `native` are stated as the declarations they are: no empty body is invented
    // and no attempt is charged for a body that is not there.
    assert_eq!(
        report.bodies[1],
        ClassViewBody::NotDeclared {
            method: report.bodies[1]
                .method()
                .expect("a no-body declaration is bound to its member")
                .clone(),
            no_body_kind: Some(NoBodyKind::Abstract),
        }
    );
    assert_eq!(
        report.bodies[2],
        ClassViewBody::NotDeclared {
            method: report.bodies[2]
                .method()
                .expect("a no-body declaration is bound to its member")
                .clone(),
            no_body_kind: Some(NoBodyKind::Native),
        }
    );
    assert_eq!(report.execution, {
        let mut complete = report.execution.clone();
        match &mut complete {
            ExecutionReport::Complete { usage } => {
                *usage = execution_usage(&report.execution).clone()
            }
            _ => panic!("the view completed: {complete:?}"),
        }
        complete
    });
}

/// One member name with two declared descriptors: the view answers with both candidates and reads
/// no body at all.
#[test]
fn an_ambiguous_body_name_returns_candidates_and_reads_no_body() {
    let engine = Engine::new();
    let class = class_file(
        b"p/Overloads",
        52,
        &[],
        &[
            MemberSpec::method(b"run", b"()V"),
            MemberSpec::method(b"run", b"(I)V"),
        ],
    );
    let snapshot = open(zip_of(&[(b"p/Overloads.class", &class)]));
    let mut budget = budget();
    let outcome = engine
        .class_view(
            &snapshot,
            &tree_scope(&snapshot),
            &ClassViewRequest {
                class: ClassRef::Name {
                    class: ClassNameQuery::internal("p/Overloads"),
                },
                bodies: vec![BodyRef::Name {
                    name: bytes(b"run"),
                    descriptor: None,
                }],
            },
            &mut budget,
        )
        .expect("an ambiguous body name is an answer");
    let candidates = ambiguous(outcome);
    assert_eq!(candidates.candidates.len(), 2);
    assert_eq!(budget.usage().method_bodies, 0);
    assert_eq!(candidates.limits, limits());
    assert_no_analysis("an ambiguous body selection", &budget.usage());
}

/// A damaged member record stops the member walk there; the class, the members before it and the
/// bodies of those members stay published, and a member beyond the stop is a refusal of its own.
#[test]
fn a_damaged_member_record_isolates_to_its_own_member() {
    let engine = Engine::new();
    let class = class_file(
        b"p/Damaged",
        52,
        &[],
        &[
            MemberSpec::method(b"before", b"()V"),
            MemberSpec::method(b"broken", b"()V").damaged(4096),
            MemberSpec::method(b"after", b"()V"),
        ],
    );
    let snapshot = open(zip_of(&[(b"p/Damaged.class", &class)]));
    let mut budget = budget();
    let report = performed(
        engine
            .class_view(
                &snapshot,
                &tree_scope(&snapshot),
                &ClassViewRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("p/Damaged"),
                    },
                    bodies: vec![
                        BodyRef::Name {
                            name: bytes(b"before"),
                            descriptor: None,
                        },
                        BodyRef::Name {
                            name: bytes(b"after"),
                            descriptor: None,
                        },
                    ],
                },
                &mut budget,
            )
            .expect("the class is confirmed even when a member record is damaged"),
    );

    // The class is not erased, and the members the walk really read are the reliable prefix.
    assert!(report.declaration().is_some());
    assert_eq!(method_names(&report), vec!["before"]);
    assert!(
        report
            .declaration()
            .expect("the class item")
            .member_table
            .is_some()
    );
    // The member the request named beyond the stop is refused, not read and not claimed missing:
    // the refusal carries the caller's own reference and no identity, because the walk never
    // reached the record that would state one.
    let ClassViewBody::Refused {
        reference,
        method,
        execution,
        diagnostics,
    } = &report.bodies[1]
    else {
        panic!(
            "the member beyond the stop is refused: {:?}",
            report.bodies[1]
        );
    };
    assert_eq!(
        reference,
        &BodyRef::Name {
            name: bytes(b"after"),
            descriptor: None,
        },
        "the refusal carries the request's own reference, not an invented identity"
    );
    assert_eq!(
        method, &None,
        "no identity is invented for a member the walk did not reach"
    );
    assert!(matches!(execution, ExecutionReport::Partial { .. }));
    assert!(!diagnostics.is_empty());
    // The member before the stop is bound, and its read is refused by the reader with its own code
    // — the class's member records are damaged, and a body decode parses the class they live in.
    // Nothing here erases the class or the members before the stop (A13).
    assert_eq!(
        report.bodies[0]
            .method()
            .map(|method| method.name.0.clone()),
        Some(bytes(b"before").0)
    );
    assert!(matches!(
        report.bodies[0],
        ClassViewBody::Refused {
            method: Some(_),
            ..
        }
    ));
    assert_eq!(
        report.usage.method_bodies, 1,
        "one body attempt was charged"
    );
    assert!(!matches!(
        report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(!report.diagnostics.is_empty());
}

/// A body identity of another definition, or a name the class does not declare, is an input error.
#[test]
fn a_body_reference_outside_this_class_is_an_input_error() {
    let engine = Engine::new();
    let class = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let other = class_file(b"p/T", 52, &[], &[MemberSpec::method(b"other", b"()V")]);
    let snapshot = open(zip_of(&[(b"p/S.class", &class), (b"p/T.class", &other)]));
    let scope = tree_scope(&snapshot);
    let foreign = performed(
        engine
            .class_view(
                &snapshot,
                &scope,
                &ClassViewRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("p/T"),
                    },
                    bodies: Vec::new(),
                },
                &mut budget(),
            )
            .expect("the other class views"),
    );
    let foreign_method = foreign
        .methods()
        .next()
        .expect("p/T declares one")
        .identity
        .clone();

    let error = engine
        .class_view(
            &snapshot,
            &scope,
            &ClassViewRequest {
                class: ClassRef::Name {
                    class: ClassNameQuery::internal("p/S"),
                },
                bodies: vec![BodyRef::Method {
                    method: foreign_method,
                }],
            },
            &mut budget(),
        )
        .expect_err("a member identity of another definition is an input error");
    assert_eq!(code_of(&error), "class_view_body_foreign_owner");

    let error = engine
        .class_view(
            &snapshot,
            &scope,
            &ClassViewRequest {
                class: ClassRef::Name {
                    class: ClassNameQuery::internal("p/S"),
                },
                bodies: vec![BodyRef::Name {
                    name: bytes(b"absent"),
                    descriptor: None,
                }],
            },
            &mut budget(),
        )
        .expect_err("a member the class does not declare is an input error");
    assert_eq!(code_of(&error), "class_view_body_not_found");
}

// ---------------------------------------------------------------------------------------------
// preserve-task-operation-stops / A07 / A13 / A14 / A16: an unfinished search decides nothing
// ---------------------------------------------------------------------------------------------

/// A class selection is decided by the search's own completeness, not by how many candidates it
/// happens to have confirmed: one candidate beside a same-named entry that does not read is not a
/// unique target, and zero candidates before such an entry is not a missing one.
///
/// The three orderings are the independent review's own fixture: the committed `NestedEval.class`
/// beside an `x/NestedEval.class` entry whose bytes are not a class, in the valid-only, damaged-
/// last and damaged-first orders.
#[test]
fn an_unfinished_name_search_is_neither_a_unique_nor_a_missing_target() {
    let engine = Engine::new();
    let broken: &[u8] = b"broken";
    let valid = open(zip_of(&[(b"NestedEval.class", NESTED_EVAL)]));
    let damaged_last = open(zip_of(&[
        (b"NestedEval.class", NESTED_EVAL),
        (b"x/NestedEval.class", broken),
    ]));
    let damaged_first = open(zip_of(&[
        (b"NestedEval.class", broken),
        (b"x/NestedEval.class", NESTED_EVAL),
    ]));
    let method_request = |snapshot: &ArtifactSnapshot| MethodOperationRequest {
        method: MethodRef::Name {
            class: ClassNameQuery::internal("NestedEval"),
            name: bytes(b"nestedPlain"),
            descriptor: Some(bytes(b"(I)I")),
        },
        environment: environment_request(snapshot, EnvironmentPolicy::PlainJar),
    };
    let class_request = || ClassViewRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("NestedEval"),
        },
        bodies: Vec::new(),
    };

    // The positive control: the whole scope was searched, one definition declares the name, and the
    // operation really runs over it.
    let mut valid_budget = budget();
    let recovered = performed(
        engine
            .recover_target(
                slice::from_ref(&valid),
                &method_request(&valid),
                &mut valid_budget,
            )
            .expect("a legal request is answered"),
    );
    assert_eq!(recovered.method.name.0, b"nestedPlain");
    assert!(matches!(
        recovered.presentation.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(valid_budget.usage().method_bodies, 1);

    // One candidate confirmed and then a same-named candidate that does not read: the confirmed
    // method is published as a candidate — not elected — and nothing is analysed or decoded.
    let mut after_budget = budget();
    let after = incomplete(
        engine
            .recover_target(
                slice::from_ref(&damaged_last),
                &method_request(&damaged_last),
                &mut after_budget,
            )
            .expect("a stopped search is a report, not an error"),
    );
    assert_eq!(after.query.class.spelling(), "NestedEval");
    assert_eq!(
        after.candidates.len(),
        1,
        "the confirmed prefix travels with the stop: {:?}",
        after.candidates
    );
    let ClassContentItem::Method(confirmed) = &after.candidates[0] else {
        panic!("a method search publishes method records")
    };
    assert_eq!(confirmed.identity.name.0, b"nestedPlain");
    assert_eq!(after.limits, limits());
    assert!(matches!(after.execution, ExecutionReport::Failed { .. }));
    assert!(
        codes(&after.diagnostics).contains(&"classfile_decode".to_string()),
        "the damaged sibling is not hidden: {:?}",
        after.diagnostics
    );
    assert_eq!(
        execution_usage(&after.execution).class_headers,
        2,
        "both candidates were examined: {:?}",
        after.execution
    );
    assert_no_analysis("an unfinished selection", execution_usage(&after.execution));
    assert_eq!(
        execution_usage(&after.execution).method_bodies,
        after_budget.usage().method_bodies,
        "the stop carries the request's real usage"
    );
    // The unfinished outcome is as deterministic as every other report: two runs of the same request
    // over the same immutable snapshot differ in the wall clock alone.
    let mut again_budget = budget();
    let again = incomplete(
        engine
            .recover_target(
                slice::from_ref(&damaged_last),
                &method_request(&damaged_last),
                &mut again_budget,
            )
            .expect("a stopped search is a report, not an error"),
    );
    assert_eq!(facts_json(&after), facts_json(&again));

    // The damaged candidate first: zero confirmed candidates, and still no claim that the name is
    // missing. The range the search really examined is stated.
    let mut before_budget = budget();
    let before = incomplete(
        engine
            .recover_target(
                slice::from_ref(&damaged_first),
                &method_request(&damaged_first),
                &mut before_budget,
            )
            .expect("a stopped search is a report, not an error"),
    );
    assert!(
        before.candidates.is_empty(),
        "nothing was confirmed before the stop: {:?}",
        before.candidates
    );
    assert!(matches!(before.execution, ExecutionReport::Failed { .. }));
    assert!(codes(&before.diagnostics).contains(&"classfile_decode".to_string()));
    assert_eq!(execution_usage(&before.execution).class_headers, 1);
    assert_scan_range(&before.coverage, 0, 1, 1, 2);
    assert_no_analysis(
        "a stop before any candidate",
        execution_usage(&before.execution),
    );

    // The class view answers the same way on the same archives: a named class whose search stopped
    // is an unfinished selection with the class item it confirmed and no member or body work.
    let mut view_budget = budget();
    let view = incomplete(
        engine
            .class_view(
                &damaged_last,
                &PhysicalScope::SnapshotAll,
                &class_request(),
                &mut view_budget,
            )
            .expect("a stopped search is a report, not an error"),
    );
    assert_eq!(view.candidates.len(), 1);
    assert_eq!(view_budget.usage().class_headers, 2);
    assert_eq!(view_budget.usage().method_bodies, 0);
    assert_no_ir("an unfinished class selection", &view_budget.usage());

    let mut view_budget = budget();
    let view = incomplete(
        engine
            .class_view(
                &damaged_first,
                &PhysicalScope::SnapshotAll,
                &class_request(),
                &mut view_budget,
            )
            .expect("a stopped search is a report, not an error"),
    );
    assert!(view.candidates.is_empty());
    assert_eq!(view_budget.usage().class_headers, 1);

    // A complete miss is still a complete answer, not an unfinished selection: the whole scope was
    // searched and no definition declares the name.
    let error = engine
        .recover_target(
            slice::from_ref(&valid),
            &MethodOperationRequest {
                method: MethodRef::Name {
                    class: ClassNameQuery::internal("NestedEval"),
                    name: bytes(b"absent"),
                    descriptor: None,
                },
                environment: environment_request(&valid, EnvironmentPolicy::PlainJar),
            },
            &mut budget(),
        )
        .expect_err("a name the whole scope searched is an input error");
    assert_eq!(code_of(&error), "operation_target_not_found");

    // A name that really has two definitions is still ambiguous rather than incomplete: the search
    // finished and bound both.
    let upper = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"upper", b"()V")]);
    let nested = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"nested", b"()V")]);
    let war = open(two_origin_war(&upper, &nested));
    let candidates = ambiguous(
        engine
            .class_view(
                &war,
                &tree_scope(&war),
                &ClassViewRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("p/S"),
                    },
                    bodies: Vec::new(),
                },
                &mut budget(),
            )
            .expect("the tree is searched"),
    );
    assert_eq!(candidates.candidates.len(), 2);
    assert!(matches!(
        candidates.execution,
        ExecutionReport::Complete { .. }
    ));

    // An explicit physical identity never searches: the very archive whose name search stopped
    // answers a definition-addressed view in full.
    let listing = engine
        .list_class_declarations(&damaged_last, &PhysicalScope::SnapshotAll, &mut budget())
        .expect("the confirmed listing keeps the entry it read");
    let declaration = &listing.items[0];
    let chosen = performed(
        engine
            .class_view(
                &damaged_last,
                &PhysicalScope::SnapshotAll,
                &ClassViewRequest {
                    class: ClassRef::Definition {
                        definition: declaration.definition.clone(),
                    },
                    bodies: Vec::new(),
                },
                &mut budget(),
            )
            .expect("an identity is read, not searched"),
    );
    assert_eq!(
        chosen
            .declaration()
            .expect("the identity's own class item")
            .declaration
            .this_class
            .raw()
            .0,
        b"NestedEval"
    );
}

/// The searched and skipped ranges of the candidate search, in its own coordinates.
fn assert_scan_range(
    coverage: &Coverage,
    scanned: u64,
    scanned_end: u64,
    skip: u64,
    skip_end: u64,
) {
    let structural = &coverage.artifact_structural;
    assert!(
        structural.scanned.contains(&CoverageRange {
            label: "navigation_candidates".into(),
            start: scanned,
            end: scanned_end,
        }) && structural.skipped.contains(&CoverageRange {
            label: "navigation_candidates".into(),
            start: skip,
            end: skip_end,
        }),
        "the search states the range it examined and the one it never reached: {structural:?}"
    );
    assert_eq!(structural.state, CoverageState::Partial);
}

/// A selection can be unfinished with zero candidates or with one — under an exhausted dimension, a
/// cancellation and a truncated member table — and none of those is a unique target or a missing
/// one. A member table that stopped leaves a member answer a prefix; a class answer rests on the
/// header, which the same read confirmed.
#[test]
fn a_stopped_selection_publishes_zero_or_more_candidates_and_runs_nothing() {
    let engine = Engine::new();

    // An exhausted dimension with nothing confirmed: the first candidate's path states the name
    // while its declaration says another, and the second candidate's header charge is refused.
    let other = class_file(b"p/Other", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let wanted = class_file(b"p/S", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let snapshot = open(zip_of(&[(b"p/S.class", &other), (b"x/p/S.class", &wanted)]));
    let request = ClassViewRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("p/S"),
        },
        bodies: Vec::new(),
    };
    let tight = task_limits(&[BudgetOverride::new("class_headers", 1).expect("named")])
        .expect("a legal override");
    let mut tight_budget = Budget::new(tight);
    let stopped = incomplete(
        engine
            .class_view(
                &snapshot,
                &PhysicalScope::SnapshotAll,
                &request,
                &mut tight_budget,
            )
            .expect("a stopped search is a report, not an error"),
    );
    assert!(
        stopped.candidates.is_empty(),
        "no binding was confirmed: {:?}",
        stopped.candidates
    );
    assert_eq!(
        stopped.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ClassHeaders,
            },
            usage: execution_usage(&stopped.execution).clone(),
        }
    );
    assert_eq!(execution_usage(&stopped.execution).class_headers, 1);
    assert_eq!(execution_usage(&stopped.execution).method_bodies, 0);
    assert!(codes(&stopped.diagnostics).contains(&"budget_exceeded_class_headers".to_string()));
    assert_scan_range(&stopped.coverage, 0, 1, 1, 2);
    assert_no_analysis(
        "an exhausted selection",
        execution_usage(&stopped.execution),
    );

    // The same request under a cancellation: the stop is the cancellation, and no candidate, header
    // or body was read.
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(limits(), token);
    let stopped = incomplete(
        engine
            .class_view(
                &snapshot,
                &PhysicalScope::SnapshotAll,
                &request,
                &mut cancelled,
            )
            .expect("a cancelled request is a report, not an error"),
    );
    assert!(stopped.candidates.is_empty());
    assert!(matches!(
        stopped.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert_eq!(execution_usage(&stopped.execution).class_headers, 0);
    assert_eq!(execution_usage(&stopped.execution).method_bodies, 0);
    assert_no_analysis("a cancelled selection", execution_usage(&stopped.execution));

    // A member selection against a member table that stopped: one match in the prefix is a prefix,
    // not a unique target, and no match in it is not a missing member.
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
    let damaged_snapshot = open(zip_of(&[(b"p/Damaged.class", &damaged)]));
    let environment = environment_request(&damaged_snapshot, EnvironmentPolicy::PlainJar);
    let ask = |name: &[u8], budget: &mut Budget| {
        engine
            .recover_target(
                slice::from_ref(&damaged_snapshot),
                &MethodOperationRequest {
                    method: MethodRef::Name {
                        class: ClassNameQuery::internal("p/Damaged"),
                        name: bytes(name),
                        descriptor: None,
                    },
                    environment: environment.clone(),
                },
                budget,
            )
            .expect("a stopped member search is a report, not an error")
    };

    let mut damaged_budget = budget();
    let one = incomplete(ask(b"first", &mut damaged_budget));
    assert_eq!(
        one.candidates.len(),
        1,
        "the confirmed prefix is published, never elected: {:?}",
        one.candidates
    );
    assert!(
        matches!(
            one.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::Error { ref code },
                ..
            } if code == "classfile_invalid_attribute_span"
        ),
        "{:?}",
        one.execution
    );
    assert!(
        codes(&one.diagnostics).contains(&"classfile_invalid_attribute_span".to_string()),
        "the member-table stop is published: {:?}",
        one.diagnostics
    );
    assert_no_analysis(
        "a member selection over a truncated table",
        execution_usage(&one.execution),
    );

    let mut damaged_budget = budget();
    let none = incomplete(ask(b"third", &mut damaged_budget));
    assert!(
        none.candidates.is_empty(),
        "a member beyond the stop is unread, never missing: {:?}",
        none.candidates
    );
    assert!(matches!(none.execution, ExecutionReport::Partial { .. }));
    assert_eq!(execution_usage(&none.execution).method_bodies, 0);

    // The same class addressed by name *for its declaration* is a different question: the header
    // confirmed it, so the view performs with the member prefix and the stop it published.
    let mut view_budget = budget();
    let view = performed(
        engine
            .class_view(
                &damaged_snapshot,
                &tree_scope(&damaged_snapshot),
                &ClassViewRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("p/Damaged"),
                    },
                    bodies: Vec::new(),
                },
                &mut view_budget,
            )
            .expect("the class is confirmed by its own header"),
    );
    assert_eq!(method_names(&view), vec!["first"]);
    assert!(
        view.declaration()
            .expect("the class item")
            .member_table
            .is_some(),
        "the member stop travels with the declaration"
    );
    assert!(!matches!(view.execution, ExecutionReport::Complete { .. }));
}

/// The independent review's damaged-body case: one illegal opcode at BCI 0 and one healthy body in
/// the same view. The library's own top-level execution says what its bodies said, the healthy body
/// keeps its own complete result, and the member table the same read really read stays complete.
#[test]
fn a_view_with_one_stopped_body_is_not_complete_and_keeps_the_other_body() {
    let engine = Engine::new();
    // The review's mutation: `nestedPlain(I)I`'s first opcode, `0x1a` (`iload_0`), becomes the
    // illegal `0xff` — a body that stops at BCI 0 and a class whose member table is intact.
    let mut damaged = NESTED_EVAL.to_vec();
    assert_eq!(
        damaged[311], 0x1a,
        "the fixture's own opcode is the assumption"
    );
    damaged[311] = 0xff;
    let snapshot = open(damaged);
    let request = ClassViewRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("NestedEval"),
        },
        bodies: vec![
            BodyRef::Name {
                name: bytes(b"nestedPlain"),
                descriptor: Some(bytes(b"(I)I")),
            },
            BodyRef::Name {
                name: bytes(b"nestedLocal"),
                descriptor: Some(bytes(b"(I)I")),
            },
        ],
    };
    let mut view_budget = budget();
    let report = performed(
        engine
            .class_view(
                &snapshot,
                &PhysicalScope::SnapshotAll,
                &request,
                &mut view_budget,
            )
            .expect("the view runs"),
    );

    // The top-level execution merges the body's own stop, so a library caller reads the depth of
    // the answer from the report rather than from the bodies.
    assert_eq!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::Error {
                code: "classfile_instruction_decode".to_string(),
            },
            usage: execution_usage(&report.execution).clone(),
        }
    );

    // The stopped body keeps its own result, its two phases and its own coverage.
    let ClassViewBody::Read {
        stopped_at,
        stages,
        coverage,
        execution,
        ..
    } = &report.bodies[0]
    else {
        panic!("the damaged body is a read: {:?}", report.bodies[0])
    };
    assert_eq!(
        stopped_at,
        &Some(BytecodeStop::Instructions {
            bci: 0,
            class_offset: 311,
            code: "classfile_instruction_decode".to_string(),
        })
    );
    assert_eq!(
        stages,
        &vec![
            BodyStageResult {
                phase: BytecodeStopPhase::ExceptionHandlers,
                state: BodyStageState::Completed,
            },
            BodyStageResult {
                phase: BytecodeStopPhase::Instructions,
                state: BodyStageState::Stopped {
                    code: "classfile_instruction_decode".to_string(),
                },
            },
        ]
    );
    assert_eq!(coverage.artifact_structural.state, CoverageState::Partial);
    assert!(matches!(execution, ExecutionReport::Partial { .. }));

    // The healthy body is untouched: its own complete result and its own complete coverage.
    let ClassViewBody::Read {
        stopped_at,
        coverage,
        execution,
        instructions,
        ..
    } = &report.bodies[1]
    else {
        panic!("the healthy body is a read: {:?}", report.bodies[1])
    };
    assert_eq!(stopped_at, &None);
    assert!(matches!(execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(!instructions.is_empty());

    // The class and member planes keep their own evidence: the member table really was read to its
    // declared end, so a stopped body does not rewrite it to unscanned.
    let structural = &report.coverage.artifact_structural;
    assert_eq!(structural.state, CoverageState::CompleteWithinSchema);
    assert!(
        structural.scanned.contains(&CoverageRange {
            label: "class_methods".into(),
            start: 0,
            end: 5,
        }),
        "{structural:?}"
    );
    assert!(structural.skipped.is_empty(), "{structural:?}");
    assert_eq!(
        report.declaration().expect("the class item").member_table,
        None
    );
    assert_eq!(report.usage.method_bodies, 2);
    assert_no_ir("a view with one stopped body", &view_budget.usage());

    // Two runs of the same request over the same immutable snapshot state the same facts: only the
    // wall-clock field of the report's own planes may differ.
    let mut again_budget = budget();
    let again = performed(
        engine
            .class_view(
                &snapshot,
                &PhysicalScope::SnapshotAll,
                &request,
                &mut again_budget,
            )
            .expect("the view runs twice"),
    );
    assert_eq!(facts_json(&report), facts_json(&again));
    let mut first_usage = view_budget.usage();
    let mut second_usage = again_budget.usage();
    first_usage.elapsed_millis = 0;
    second_usage.elapsed_millis = 0;
    assert_eq!(
        first_usage, second_usage,
        "only the wall-clock field of the usage planes may differ between two runs"
    );
}

// ---------------------------------------------------------------------------------------------
// 3.2 / A15 / A16 / A17 / A18: opening stays light and repeats stay identical
// ---------------------------------------------------------------------------------------------

/// Opening a snapshot reads no class and builds nothing: the artifact's own bytes are all it reads.
#[test]
fn opening_a_snapshot_reads_no_class_and_builds_nothing() {
    let class = bodies_class();
    let jar = zip_of(&[(b"p/Bodies.class", &class)]);
    let (snapshot, usage) = open_with(jar.clone(), limits());
    assert_eq!(usage.class_headers, 0, "{usage:?}");
    assert_eq!(usage.method_bodies, 0, "{usage:?}");
    assert_eq!(usage.code_bytes, 0, "{usage:?}");
    assert_eq!(usage.output_bytes, 0, "{usage:?}");
    assert_eq!(usage.archive_entries, 0, "opening does not even enumerate");
    assert!(
        usage.input_bytes > 0,
        "the artifact's own bytes are the read"
    );
    assert_no_analysis("opening a snapshot", &usage);

    // The identity is a function of the bytes, and the snapshot keeps them: opening the same
    // bytes again is the same immutable artifact.
    let (again, _) = open_with(jar, limits());
    assert_eq!(snapshot.id(), again.id());
    assert_eq!(snapshot.len(), again.len());
    assert_eq!(snapshot.kind(), ArtifactKind::Zip);
}

/// Two runs of one operation over one immutable snapshot state the same facts, identities, order,
/// coverage and execution — with the optional facts cache attached or not (A15).
#[test]
fn repeated_operations_over_one_snapshot_are_identical() {
    let engine = Engine::new();
    let snapshot = nested_eval();
    let method = member_identity(&engine, &snapshot, "NestedEval", b"nestedLocal", None);
    let request = MethodOperationRequest {
        method: MethodRef::Method {
            method: method.clone(),
        },
        environment: environment_request(&snapshot, EnvironmentPolicy::SingleClass),
    };

    let run = |cache: Option<FactsCache>| {
        let mut budget = match cache {
            Some(cache) => task_budget(&[])
                .expect("the task defaults are bounded")
                .with_facts_cache(cache),
            None => task_budget(&[]).expect("the task defaults are bounded"),
        };
        let report = performed(
            engine
                .recover_target(slice::from_ref(&snapshot), &request, &mut budget)
                .expect("a legal task request is answered"),
        );
        (
            facts_json(&report.presentation),
            facts_json(report.recovered.recovery()),
            facts_json(report.recovered.analysis()),
            report.method.clone(),
        )
    };

    let first = run(None);
    let second = run(None);
    assert_eq!(first, second, "one snapshot, one operation, one answer");
    assert_eq!(first.3, method, "the identity is the one the request named");

    // The same request with a shared facts cache enabled: the facts and identities are unchanged
    // (A15's reuse-on/reuse-off agreement), and the cache really was consulted.
    let cache = FactsCache::current(FactsCapacity::new(64, 1 << 20));
    let cached = run(Some(cache.clone()));
    assert_eq!(first.0, cached.0);
    assert_eq!(first.1, cached.1);
    assert_eq!(first.2, cached.2);
    assert!(
        cache.report().consultations > 0,
        "the enabled path really consulted the store: {:?}",
        cache.report()
    );
}

// ---------------------------------------------------------------------------------------------
// 3.4 / A13 / A14: the recovery presentation leads with delivered content
// ---------------------------------------------------------------------------------------------

/// The committed refusal sample: `fieldCast` is an artifact of quoted refusals alone.
const REFUSED_CAST: &[u8] = include_bytes!("fixtures/p3-refused-cast/v8/RefusedCast.class");

/// One task recovery of one member of a committed sample.
fn recover_fixture(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    class: &str,
    name: &[u8],
    budget: &mut Budget,
) -> MethodRecoveryReport {
    let method = member_identity(engine, snapshot, class, name, None);
    performed(
        engine
            .recover_target(
                slice::from_ref(snapshot),
                &MethodOperationRequest {
                    method: MethodRef::Method { method },
                    environment: environment_request(snapshot, EnvironmentPolicy::SingleClass),
                },
                budget,
            )
            .expect("a legal task request is answered"),
    )
}

/// A statement-bearing artifact presents `contains_statements` first, then its quality — and the
/// classification is not read as completeness, compilability or equivalence.
#[test]
fn a_statement_bearing_result_leads_with_content() {
    let engine = Engine::new();
    let snapshot = nested_eval();
    let report = recover_fixture(
        &engine,
        &snapshot,
        "NestedEval",
        b"nestedPlain",
        &mut budget(),
    );
    let presentation = &report.presentation;
    assert_eq!(presentation.content, RecoveryContent::ContainsStatements);
    assert_eq!(
        presentation.content,
        report.recovered.recovery().content,
        "the presentation reads the report's own field"
    );
    assert_eq!(presentation.quality, report.recovered.recovery().quality);
    assert_eq!(
        presentation.execution,
        report.recovered.recovery().execution
    );
    assert_eq!(presentation.stop, None, "an artifact was delivered");
    assert_eq!(
        presentation.parts(),
        vec![
            RecoveryPresentationPart::Content {
                content: RecoveryContent::ContainsStatements,
            },
            RecoveryPresentationPart::Quality {
                quality: report.recovered.recovery().quality,
            },
        ],
        "content first, then quality, and no stop part after a delivered artifact"
    );
    // The other planes keep their own meaning: nothing here says the text compiles or was verified.
    assert_eq!(
        report.recovered.recovery().compile_status,
        CompileStatus::NotAttempted
    );
    assert_eq!(
        report.recovered.recovery().verification,
        VerificationStatus::NotPerformed
    );
}

/// An explanation-only artifact presents `explanation_only` first; a stopped run presents
/// `not_produced` first and keeps the stop contract (empty text and source map, a stated stop).
#[test]
fn an_explanation_only_result_and_a_stop_are_presented_in_order() {
    let engine = Engine::new();
    let snapshot = open(REFUSED_CAST.to_vec());
    let report = recover_fixture(
        &engine,
        &snapshot,
        "RefusedCast",
        b"fieldCast",
        &mut budget(),
    );
    let presentation = &report.presentation;
    assert_eq!(presentation.content, RecoveryContent::ExplanationOnly);
    assert_eq!(presentation.stop, None);
    assert!(
        !report.recovered.recovery().text.is_empty(),
        "the explanation itself is delivered"
    );
    assert_eq!(
        presentation.parts(),
        vec![
            RecoveryPresentationPart::Content {
                content: RecoveryContent::ExplanationOnly,
            },
            RecoveryPresentationPart::Quality {
                quality: report.recovered.recovery().quality,
            },
        ]
    );

    // A tight output budget stops the run before an artifact is delivered: the stop reason is the
    // third part, and the delivery planes state the stop instead of an empty recovery.
    let tight = task_limits(&[BudgetOverride::new("output_bytes", 8).expect("named")])
        .expect("a legal override");
    let mut budget = Budget::new(tight);
    let report = recover_fixture(&engine, &snapshot, "RefusedCast", b"fieldCast", &mut budget);
    let presentation = &report.presentation;
    assert_eq!(presentation.content, RecoveryContent::NotProduced);
    assert_eq!(
        presentation.stop,
        report.recovered.recovery().outcome.stop().cloned()
    );
    assert!(presentation.stop.is_some(), "the stop reason is published");
    assert!(report.recovered.recovery().text.is_empty());
    assert!(report.recovered.recovery().source_map.segments().is_empty());
    assert!(!matches!(
        report.recovered.recovery().execution,
        ExecutionReport::Complete { .. }
    ));
    let parts = presentation.parts();
    assert_eq!(parts.len(), 3);
    assert!(matches!(parts[0], RecoveryPresentationPart::Content { .. }));
    assert!(matches!(parts[1], RecoveryPresentationPart::Quality { .. }));
    assert!(matches!(parts[2], RecoveryPresentationPart::Stop { .. }));
}

/// The presentation is a reading of the report's own fields: a text that spells `return` — or any
/// other wording — cannot move what it presents, and the delivered statement case keeps its
/// classification whatever its text says.
#[test]
fn the_presentation_does_not_re_read_the_text() {
    let engine = Engine::new();
    let snapshot = open(REFUSED_CAST.to_vec());
    let report = recover_fixture(
        &engine,
        &snapshot,
        "RefusedCast",
        b"fieldCast",
        &mut budget(),
    );
    let recovery = report.recovered.recovery();
    assert_eq!(recovery.content, RecoveryContent::ExplanationOnly);
    assert!(
        !recovery.text.contains("return"),
        "the committed explanation of this sample does not spell `return`"
    );
    let before = RecoveryPresentation::of(recovery);

    // The same report with an explanation that spells `return`: the wording is not a second
    // classification source, so content, quality, the stop and the ordered parts are unchanged.
    let mut spelled = recovery.clone();
    spelled.text = spelled.text.replace(
        "the instruction at BCI 3",
        "the return instruction at BCI 3",
    );
    assert!(spelled.text.contains("return"));
    let after = RecoveryPresentation::of(&spelled);
    assert_eq!(after, before);
    assert_eq!(after.content, RecoveryContent::ExplanationOnly);
    assert_eq!(
        after.parts(),
        before.parts(),
        "the presentation order does not depend on the text either"
    );

    // The other direction from the same sample: a delivered statement makes the content bearer,
    // and its own text spelling `return` changes nothing about that either.
    let statements = recover_fixture(
        &engine,
        &snapshot,
        "RefusedCast",
        b"leftRead",
        &mut budget(),
    );
    let recovery = statements.recovered.recovery();
    assert!(recovery.text.contains("return"));
    assert_eq!(recovery.content, RecoveryContent::ContainsStatements);
    assert_eq!(
        RecoveryPresentation::of(recovery).content,
        RecoveryContent::ContainsStatements
    );

    // And two artifacts whose literals differ but whose structure is the same are presented the
    // same way: the classification is structural, not lexical.
    let nested = nested_eval();
    let plain = recover_fixture(
        &engine,
        &nested,
        "NestedEval",
        b"nestedPlain",
        &mut budget(),
    );
    let local = recover_fixture(
        &engine,
        &nested,
        "NestedEval",
        b"nestedLocal",
        &mut budget(),
    );
    assert_eq!(
        plain.presentation.content,
        RecoveryContent::ContainsStatements
    );
    assert_eq!(
        local.presentation.content,
        RecoveryContent::ContainsStatements
    );
    assert_ne!(
        plain.recovered.recovery().text,
        local.recovered.recovery().text,
        "the two artifacts really differ in text"
    );
    assert_eq!(
        plain.presentation.parts().len(),
        local.presentation.parts().len(),
        "and the presentation order is the same shape"
    );
}

// ---------------------------------------------------------------------------------------------
// 3.3 / A01 / A03 / A11 / A14: references grouped by owning method, derivation classes apart
// ---------------------------------------------------------------------------------------------

/// One class file with a chosen superclass, no members.
fn bare_class_file(this_class: &[u8], super_class: &[u8]) -> Vec<u8> {
    let mut pool = Vec::new();
    let this_utf8 = utf8(&mut pool, this_class);
    let this_index = class_entry(&mut pool, this_utf8);
    let super_utf8 = utf8(&mut pool, super_class);
    let super_index = class_entry(&mut pool, super_utf8);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 52);
    u16b(
        &mut bytes,
        u16::try_from(pool.len() + 1).expect("the fixture pool fits u16"),
    );
    for entry in &pool {
        bytes.extend_from_slice(entry);
    }
    u16b(&mut bytes, 0x0021);
    u16b(&mut bytes, this_index);
    u16b(&mut bytes, super_index);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 0);
    bytes
}

/// One class whose `run()V` allocates `p/T`, calls `p/Sub.foo()V` and holds an unused
/// `p/Unused.foo()V` `Methodref`: the three positions a reference result is organised around.
fn caller_class_file() -> Vec<u8> {
    let mut pool = Vec::new();
    let this_utf8 = utf8(&mut pool, b"p/Caller");
    let this_index = class_entry(&mut pool, this_utf8);
    let object_utf8 = utf8(&mut pool, b"java/lang/Object");
    let object_index = class_entry(&mut pool, object_utf8);
    let run_name = utf8(&mut pool, b"run");
    let run_descriptor = utf8(&mut pool, b"()V");
    let code_name = utf8(&mut pool, b"Code");
    let target_utf8 = utf8(&mut pool, b"p/T");
    let target_index = class_entry(&mut pool, target_utf8);
    let sub_utf8 = utf8(&mut pool, b"p/Sub");
    let sub_index = class_entry(&mut pool, sub_utf8);
    let foo_name = utf8(&mut pool, b"foo");
    let foo_descriptor = utf8(&mut pool, b"()V");
    let call_reference = method_ref(&mut pool, sub_index, foo_name, foo_descriptor);
    let unused_utf8 = utf8(&mut pool, b"p/Unused");
    let unused_index = class_entry(&mut pool, unused_utf8);
    method_ref(&mut pool, unused_index, foo_name, foo_descriptor);

    let high = |index: u16| u8::try_from(index >> 8).expect("a pool index fits a byte");
    let low = |index: u16| u8::try_from(index & 0xff).expect("a pool index fits a byte");
    // `new p/T` (0, 1, 2), `pop` (3), `aload_0` (4), `invokevirtual p/Sub.foo()V` (5, 6, 7),
    // `return` (8): the use site is at BCI 5 and the allocation at 0, so a grouping that mixed
    // them up would be visible.
    let code = vec![
        0xbb,
        high(target_index),
        low(target_index),
        0x57,
        0x2a,
        0xb6,
        high(call_reference),
        low(call_reference),
        0xb1,
    ];
    let content_length = 2 + 2 + 4 + code.len() + 2 + 2;

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 52);
    u16b(
        &mut bytes,
        u16::try_from(pool.len() + 1).expect("the fixture pool fits u16"),
    );
    for entry in &pool {
        bytes.extend_from_slice(entry);
    }
    u16b(&mut bytes, 0x0021);
    u16b(&mut bytes, this_index);
    u16b(&mut bytes, object_index);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 1);
    u16b(&mut bytes, ACC_PUBLIC);
    u16b(&mut bytes, run_name);
    u16b(&mut bytes, run_descriptor);
    u16b(&mut bytes, 1);
    u16b(&mut bytes, code_name);
    u32b(
        &mut bytes,
        u32::try_from(content_length).expect("the fixture code fits u32"),
    );
    u16b(&mut bytes, 2);
    u16b(&mut bytes, 2);
    u32b(
        &mut bytes,
        u32::try_from(code.len()).expect("the fixture code fits u32"),
    );
    bytes.extend_from_slice(&code);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 0);
    bytes
}

/// The full archive: the declaration `p/Base`, the subclass, the allocated and extended `p/T`, the
/// caller and the manifest entry that names `p/T`.
fn reference_archive() -> Vec<u8> {
    zip_of(&[
        (
            b"p/Base.class",
            &class_file(b"p/Base", 52, &[], &[MemberSpec::method(b"foo", b"()V")]),
        ),
        (b"p/Sub.class", &bare_class_file(b"p/Sub", b"p/Base")),
        (b"p/T.class", &bare_class_file(b"p/T", b"java/lang/Object")),
        (b"p/Other.class", &bare_class_file(b"p/Other", b"p/T")),
        (b"p/Caller.class", &caller_class_file()),
        (
            b"META-INF/MANIFEST.MF",
            b"Manifest-Version: 1.0\nMain-Class: p/T\n",
        ),
    ])
}

/// The archive without `p/Base`: the owner `p/Sub` is there, so the candidate is found, and its
/// hierarchy is not, so the candidate stays undecided instead of becoming a negative.
fn caller_without_base_archive() -> Vec<u8> {
    zip_of(&[
        (b"p/Sub.class", &bare_class_file(b"p/Sub", b"p/Base")),
        (b"p/Caller.class", &caller_class_file()),
    ])
}

fn consumer_schema() -> ConsumerSchema {
    ConsumerSchema::new(
        1,
        [
            ConsumerKind::Invocation,
            ConsumerKind::Field,
            ConsumerKind::Type,
            ConsumerKind::Constant,
            ConsumerKind::Exception,
            ConsumerKind::Signature,
            ConsumerKind::Annotation,
            ConsumerKind::InnerNest,
            ConsumerKind::Module,
            ConsumerKind::Bootstrap,
            ConsumerKind::Resource,
        ],
    )
}

fn mentions(snapshot: &ArtifactSnapshot, symbol: SymbolRef) -> QueryRequest {
    QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target: QueryTarget::Symbol { value: symbol },
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: consumer_schema(),
        max_items: 0,
        cursor: None,
    }
}

/// The declaring class one resolved member names.
fn resolved_owner(declaration: &ResolvedMemberRef) -> Vec<u8> {
    match &declaration.member {
        SymbolRef::Class { owner }
        | SymbolRef::Field { owner, .. }
        | SymbolRef::Method { owner, .. } => owner.0.clone(),
    }
}

fn method_symbol(owner: &[u8], name: &[u8], descriptor: &[u8]) -> SymbolRef {
    SymbolRef::Method {
        owner: bytes(owner),
        name: bytes(name),
        descriptor: bytes(descriptor),
    }
}

/// One report's items, reorganised: the body hit is grouped under its owning method, the class-level
/// and resource hits keep their own positions, and the scan's planes are the scan's own.
#[test]
fn one_report_groups_body_class_level_and_resource_hits_by_owner() {
    let engine = Engine::new();
    let snapshot = open(reference_archive());
    let report = engine
        .query(
            &snapshot,
            &mentions(
                &snapshot,
                SymbolRef::Class {
                    owner: bytes(b"p/T"),
                },
            ),
            &mut budget(),
        )
        .expect("a legal query is answered");
    assert_eq!(report.analysis, QueryAnalysis::Performed);
    assert!(!report.items.is_empty());
    let published_items = report.items.len();
    let source = report.clone();
    let grouping = ReferenceGrouping::from_query(report);

    // The scan's own planes are echoed field by field, and the item list is the only thing moved.
    let ReferenceSource::Query {
        physical,
        relation,
        consumers,
        analysis,
        page,
        coverage,
        execution,
        diagnostics,
    } = &grouping.source
    else {
        panic!("a query report is a query source");
    };
    assert_eq!(physical, &source.physical);
    assert_eq!(relation, &source.relation);
    assert_eq!(consumers, &source.consumers);
    assert_eq!(analysis, &source.analysis);
    assert_eq!(page, &source.page);
    assert_eq!(coverage, &source.coverage);
    assert_eq!(execution, &source.execution);
    assert_eq!(diagnostics, &source.diagnostics);
    assert_eq!(grouping.finding_count(), published_items);

    // The use site is grouped under the method that owns it, with its own BCI.
    let group = grouping
        .methods
        .iter()
        .find(|group| group.method.name.0 == b"run")
        .expect("the body hit is grouped under its owning method");
    assert_eq!(
        group.method.owner.entry().expect("an entry").raw_name.0,
        b"p/Caller.class"
    );
    let site = group
        .findings
        .iter()
        .find(|finding| finding.class() == ReferenceFindingClass::StructuralReference)
        .expect("the allocation is a structural use site");
    assert_eq!(
        site.bci(),
        Some(0),
        "the use site keeps the BCI of the `new`, which is the position this target occurs at"
    );
    assert!(matches!(
        site,
        ReferenceFinding::StructuralReference { item }
            if item.consumer == Some(ConsumerKind::Type) && item.operation == XrefOperation::New
    ));
    // The class-level hit (the class another class extends) keeps its own position and is assigned
    // to no method.
    assert!(grouping.class_level.iter().any(|finding| matches!(
        finding,
        ReferenceFinding::StructuralReference { item }
            if item.consumer == Some(ConsumerKind::Type)
                && item.operation == XrefOperation::SuperClass
    )));
    // The resource hit (the manifest entry) keeps its position too.
    assert!(grouping.resources.iter().any(|finding| matches!(
        finding,
        ReferenceFinding::StructuralReference { item }
            if item.consumer == Some(ConsumerKind::Resource)
                && item.operation == XrefOperation::ManifestMainClass
    )));
    assert!(
        grouping
            .methods
            .iter()
            .all(|group| group.findings.iter().all(|finding| finding.bci().is_some())),
        "a method group holds body hits only"
    );
}

/// An unused `Methodref` is a constant-pool candidate: never a call, never a structural use site,
/// and the grouping keeps it out of every method group.
#[test]
fn a_constant_pool_candidate_stays_a_candidate() {
    let engine = Engine::new();
    let snapshot = open(reference_archive());
    let request = QueryRequest {
        relation: QueryRelation::ConstantPoolContains,
        target: QueryTarget::Symbol {
            value: method_symbol(b"p/Unused", b"foo", b"()V"),
        },
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: consumer_schema(),
        max_items: 0,
        cursor: None,
    };
    let report = engine
        .query(&snapshot, &request, &mut budget())
        .expect("a legal query is answered");
    assert!(report.items.iter().all(|item| {
        item.derivation == XrefDerivation::ConstantPoolCandidate
            && item.consumer.is_none()
            && item.evidence.bci.is_none()
    }));
    let grouping = ReferenceGrouping::from_query(report);
    assert!(grouping.methods.is_empty(), "a CP candidate owns no method");
    assert!(grouping.resources.is_empty());
    assert_eq!(grouping.class_level.len(), 1);
    assert_eq!(
        grouping.class_level[0].class(),
        ReferenceFindingClass::ConstantPoolCandidate
    );
    assert_eq!(grouping.class_level[0].bci(), None);
}

/// A `Sub` call is a structural use site, not a resolved declaration, until an explicit environment
/// resolves it to the `Base` declaration — and then it is a different finding, not a replacement.
#[test]
fn a_sub_call_becomes_a_resolved_declaration_only_under_an_environment() {
    let engine = Engine::new();
    let snapshot = open(reference_archive());
    let symbol = method_symbol(b"p/Sub", b"foo", b"()V");

    // Without a declaration environment: the raw symbol stays the CP owner, with no `resolves_to`.
    let report = engine
        .query(
            &snapshot,
            &mentions(&snapshot, symbol.clone()),
            &mut budget(),
        )
        .expect("a legal query is answered");
    let structural: Vec<&XrefItem> = report
        .items
        .iter()
        .filter(|item| item.consumer == Some(ConsumerKind::Invocation))
        .collect();
    assert_eq!(structural.len(), 1);
    assert_eq!(structural[0].derivation, XrefDerivation::StructuralConsumer);
    assert_eq!(
        structural[0].resolution,
        QueryResolution::NotRequested,
        "a structural scan claims no declaration resolution"
    );
    assert_eq!(
        structural[0].target,
        XrefTarget::Symbol {
            value: symbol.clone()
        }
    );
    let grouping = ReferenceGrouping::from_query(report);
    assert_eq!(grouping.methods.len(), 1);
    assert_eq!(grouping.methods[0].method.name.0, b"run");
    assert_eq!(
        grouping.methods[0].findings[0].bci(),
        Some(5),
        "the use site keeps the BCI of the invokevirtual, not the allocation before it"
    );

    // Under an explicit environment that provides the declaration, the same use site resolves to
    // `p/Base.foo()V` and is published as its own finding class.
    let environment = environment_request(&snapshot, EnvironmentPolicy::PlainJar)
        .build(slice::from_ref(&snapshot))
        .expect("a plain jar policy builds");
    let resolved = engine
        .resolve_symbol(
            slice::from_ref(&snapshot),
            &ResolutionRequest {
                environment: environment.clone(),
                target: symbol.clone(),
                use_kind: ReferenceUse::InvokeVirtual,
                caller: CallerContext {
                    loader: LoaderId("app".to_string()),
                    enclosing: None,
                },
                dispatch: None,
            },
            &mut budget(),
        )
        .expect("a legal resolution request is answered");
    assert_eq!(resolved.state, Some(ResolutionState::Resolved));
    let declaration = resolved.resolved.expect("p/Sub.foo resolves to p/Base.foo");
    assert_eq!(resolved_owner(&declaration), b"p/Base".to_vec());
    let report = engine
        .declaration_references(
            slice::from_ref(&snapshot),
            &DeclarationRefQuery {
                environment,
                declaration,
                scope: PhysicalScope::SnapshotAll,
                consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
                max_items: 0,
            },
            &mut budget(),
        )
        .expect("a legal declaration query is answered");
    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert_eq!(report.items.len(), 1);
    let unresolved_before = report.unresolved_candidates;
    let grouping = ReferenceGrouping::from_declaration(report);
    assert_eq!(grouping.methods.len(), 1);
    assert_eq!(grouping.methods[0].method.name.0, b"run");
    let finding = &grouping.methods[0].findings[0];
    assert_eq!(finding.class(), ReferenceFindingClass::ResolvedDeclaration);
    assert_eq!(finding.bci(), Some(5));
    assert!(matches!(
        finding,
        ReferenceFinding::ResolvedDeclaration { item }
            if item.state == ResolutionState::Resolved
                && item
                    .resolved
                    .as_ref()
                    .is_some_and(|resolved| resolved_owner(resolved) == b"p/Base".to_vec())
    ));
    assert_eq!(
        match &grouping.source {
            ReferenceSource::Declaration {
                unresolved_candidates,
                ..
            } => *unresolved_candidates,
            ReferenceSource::Query { .. } => panic!("a declaration report is a declaration source"),
        },
        unresolved_before
    );
}

/// A scan that could not decide its candidate publishes the count and the diagnostics, and the
/// grouping neither completes them nor invents a group (A14).
#[test]
fn grouping_does_not_complete_an_unresolved_candidate() {
    let engine = Engine::new();
    let snapshot = open(caller_without_base_archive());
    let environment = environment_request(&snapshot, EnvironmentPolicy::PlainJar)
        .build(slice::from_ref(&snapshot))
        .expect("a plain jar policy builds");
    let report = engine
        .declaration_references(
            slice::from_ref(&snapshot),
            &DeclarationRefQuery {
                environment,
                declaration: ResolvedMemberRef {
                    loader: LoaderId("app".to_string()),
                    definition: PhysicalDefinitionId {
                        location: PhysicalClassLocation::ArchiveEntry {
                            entry: PhysicalEntryId {
                                origin: root_origin(&snapshot),
                                ordinal: 0,
                                raw_name: ArchiveNameBytes(b"p/Base.class".to_vec()),
                            },
                        },
                        class_bytes: ClassBytesId {
                            digest: Digest("0".repeat(64)),
                            length: 0,
                        },
                        variant: PhysicalVariant::Base,
                    },
                    member: method_symbol(b"p/Base", b"foo", b"()V"),
                },
                scope: PhysicalScope::SnapshotAll,
                consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
                max_items: 0,
            },
            &mut budget(),
        )
        .expect("a declaration query over an incomplete environment is answered");
    assert_eq!(
        report.items.len(),
        0,
        "the candidate resolves to no declaration here"
    );
    assert_eq!(report.unresolved_candidates, 1);
    // The scan ran to the end of its range, so this is not a truncated answer: it is a decided
    // "undecided" — the count and the diagnostic are the whole statement.
    assert!(!report.has_more);
    let diagnostics = report.diagnostics.len();
    let grouping = ReferenceGrouping::from_declaration(report);
    assert!(grouping.methods.is_empty());
    assert_eq!(grouping.finding_count(), 0);
    let ReferenceSource::Declaration {
        unresolved_candidates,
        has_more,
        diagnostics: kept,
        ..
    } = &grouping.source
    else {
        panic!("a declaration report is a declaration source");
    };
    assert_eq!(*unresolved_candidates, 1);
    assert!(!*has_more);
    assert_eq!(kept.len(), diagnostics);
}

/// A policy the existing validator rejects leaves the run in its honest unavailable state and
/// keeps the original symbol: the policy layer rewrites nothing.
#[test]
fn a_policy_the_validator_rejects_keeps_the_unavailable_state() {
    let engine = Engine::new();
    let snapshot = nested_eval();
    let method = member_identity(&engine, &snapshot, "NestedEval", b"nestedPlain", None);
    // A classpath root naming a snapshot the request did not provide: the declaration is
    // buildable and the validator owns the finding.
    let missing = EnvironmentPolicy::ExplicitClasspath {
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: SnapshotId("not-provided-by-this-request".to_string()),
        }],
    };
    let mut request = environment_request(&snapshot, missing);
    request.snapshot = snapshot.id().clone();
    let report = performed(
        engine
            .analyze_target(
                slice::from_ref(&snapshot),
                &MethodOperationRequest {
                    method: MethodRef::Method {
                        method: method.clone(),
                    },
                    environment: request,
                },
                &mut budget(),
            )
            .expect("a rejected environment is a report, not an error"),
    );
    assert_eq!(report.method, method, "the original symbol is kept");
    assert_eq!(report.analysis.environment_problems.len(), 1);
    assert_eq!(
        report.analysis.environment_problems[0].code,
        EnvironmentProblemCode::ContentNotProvided
    );
    assert!(codes(&report.analysis.diagnostics).contains(&"content_not_provided".to_string()));
    assert_eq!(report.analysis.body, MethodBodyState::NotInspected);
    assert_eq!(
        report.analysis.semantic_validation,
        SemanticValidation::Unproven
    );
    assert!(
        report
            .analysis
            .stages
            .iter()
            .all(|stage| stage.state == StageState::NotPerformed),
        "a rejected environment starts no stage: {:?}",
        report.analysis.stages
    );
    assert_eq!(report.usage.method_bodies, 0);
    assert_eq!(report.usage.class_headers, 0);
}

/// A layout-detection archive: a Manifest `Class-Path`, a nested library, a WAR layout and a Boot
/// layout all present at once, so one snapshot can answer every non-inference negative.
fn layout_archive() -> Vec<u8> {
    let nested_class = class_file(b"p/Nested", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let war_class = class_file(b"p/W", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let boot_class = class_file(b"p/B", 52, &[], &[MemberSpec::method(b"run", b"()V")]);
    let nested = zip_of(&[(b"p/Nested.class", &nested_class)]);
    let boot_library = zip_of(&[(b"p/B.class", &boot_class)]);
    let manifest = b"Manifest-Version: 1.0\nClass-Path: lib/one.jar\nMain-Class: p/Main\n";
    zip_of(&[
        (b"META-INF/MANIFEST.MF", manifest),
        (b"lib/one.jar", &nested),
        (b"WEB-INF/classes/p/W.class", &war_class),
        (b"WEB-INF/lib/w.jar", &boot_library),
        (b"BOOT-INF/classes/p/B.class", &boot_class),
        (b"BOOT-INF/lib/b.jar", &boot_library),
    ])
}

/// A Manifest `Class-Path`, a WAR/Boot layout and a nested library are evidence about paths, not a
/// load policy: no policy generates a root from them, and the unprovided layout policy says so.
#[test]
fn a_layout_manifest_or_nested_library_is_not_a_load_policy() {
    let engine = Engine::new();
    let snapshot = open(layout_archive());
    let environment = environment_request(&snapshot, EnvironmentPolicy::PlainJar)
        .build(slice::from_ref(&snapshot))
        .expect("a plain jar policy builds");
    // Exactly one root: the snapshot's own root container at its own root. No `WEB-INF/classes/`,
    // no `BOOT-INF/classes/`, no `lib/` and no nested container ever becomes a position.
    assert_eq!(environment.domains[0].roots.len(), 1);
    assert_eq!(
        environment.domains[0].roots,
        vec![LoadRoot::Container {
            origin: root_origin(&snapshot),
            prefix: ArchiveNameBytes(Vec::new()),
        }]
    );
    let (problems, _) = validate_environment(slice::from_ref(&snapshot), &environment);
    assert!(
        problems.is_empty(),
        "the declaration validates: {problems:?}"
    );

    // The classes those layouts hold are really there (the tree walk reaches them) ...
    let tree = engine
        .enumerate_artifact_tree(&snapshot, &mut budget())
        .expect("the artifact tree walks");
    let names: Vec<String> = tree
        .containers
        .iter()
        .flat_map(|container| container.entries.iter())
        .map(|entry| text(&entry.id.raw_name.0))
        .collect();
    for expected in [
        "WEB-INF/classes/p/W.class",
        "BOOT-INF/classes/p/B.class",
        "lib/one.jar",
        "BOOT-INF/lib/b.jar",
    ] {
        assert!(
            names.iter().any(|name| name == expected),
            "the fixture really holds {expected}: {names:?}"
        );
    }

    // ... and a plain-jar lookup does not find them: no prefix and no nested library was inferred,
    // and the answer is the decided searching of the one declared root.
    for absent in [b"p/W".as_slice(), b"p/B", b"p/Nested", b"p/Main"] {
        let report = engine
            .resolve_symbol(
                slice::from_ref(&snapshot),
                &ResolutionRequest {
                    environment: environment.clone(),
                    target: SymbolRef::Class {
                        owner: bytes(absent),
                    },
                    use_kind: ReferenceUse::ClassReference,
                    caller: CallerContext {
                        loader: LoaderId("app".to_string()),
                        enclosing: None,
                    },
                    dispatch: None,
                },
                &mut budget(),
            )
            .expect("a legal resolution request is answered");
        assert_eq!(
            report.state,
            Some(ResolutionState::Missing),
            "`{}` is not on the declared order",
            text(absent)
        );
        assert!(report.resolved.is_none());
    }

    // The classes a layout does hold are reachable when the caller declares the root: the explicit
    // classpath is the one path that adds a position, and it adds only what was declared.
    let explicit = environment_request(
        &snapshot,
        EnvironmentPolicy::ExplicitClasspath {
            roots: vec![LoadRoot::Container {
                origin: root_origin(&snapshot),
                prefix: ArchiveNameBytes(b"WEB-INF/classes/".to_vec()),
            }],
        },
    )
    .build(slice::from_ref(&snapshot))
    .expect("an explicit classpath builds");
    let report = engine
        .resolve_symbol(
            slice::from_ref(&snapshot),
            &ResolutionRequest {
                environment: explicit,
                target: SymbolRef::Class {
                    owner: bytes(b"p/W"),
                },
                use_kind: ReferenceUse::ClassReference,
                caller: CallerContext {
                    loader: LoaderId("app".to_string()),
                    enclosing: None,
                },
                dispatch: None,
            },
            &mut budget(),
        )
        .expect("a legal resolution request is answered");
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert!(report.resolved.is_some());

    // A layout policy this stage does not provide answers with an explicit unsupported error — for
    // every mode, including the ones whose layout this fixture really shows — and never with roots.
    for mode in [LayoutMode::War, LayoutMode::SpringBoot, LayoutMode::Generic] {
        let error = environment_request(&snapshot, EnvironmentPolicy::Layout { mode })
            .build(slice::from_ref(&snapshot))
            .expect_err("no layout policy is provided by this stage");
        assert_eq!(code_of(&error), "environment_policy_layout_not_provided");
        assert!(matches!(error, Error::Unsupported { .. }));
    }
}
