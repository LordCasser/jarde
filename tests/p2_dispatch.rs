//! P2 2.5 acceptance: the dispatch plane — known candidates inside one explicit range.
//!
//! What this file has to prove through the public API is:
//!
//! 1. the range is enumerated by name and resolved through the 2.2 closure, so every class of
//!    the range that overrides or implements the resolved declaration becomes one candidate,
//!    with the candidate's own declaring class published as the member owner — the member test
//!    is structural (kind, raw name and descriptor) and screens no member flag, so `private`,
//!    `static`, `abstract` and special names like `<init>` are all published alike;
//! 2. the plane is open-world by structure, not by wording: a candidate carries the open-world
//!    fact it stands under (`MissingDependency` for its own unread supertype, `UnknownLoader`
//!    for a declared loader outside the caller's chain, `RuntimeTransformation` for a
//!    participating domain's uncertainty, `OrderedRoot` only past the first root of its layer,
//!    and `ExternalSubclass` for the external/unprovided roots the validator rejects wholesale),
//!    and a candidate the plane can state completely carries no evidence at all — which is what
//!    lets a complete range answer `open_world = false`;
//! 3. a single known candidate is never published as a unique target: `DispatchReport` and
//!    `DispatchCandidate` have no such field, and the caller only has `candidates` +
//!    `open_world` + the evidence;
//! 4. the range is read as headers only: the evidence is `code_bytes == 0` next to the P1 body
//!    path's non-zero charge on the same class (`method_bodies` has no charge point before 3.x
//!    and proves nothing), with `class_headers > 0` and `DispatchScope` read reasons;
//! 5. a stop keeps what it already found: a refused `ClassHeaders` or `ResultItems` charge
//!    publishes the candidates published so far, keeps `open_world = true` and reports `Partial`
//!    — and the two stops are told apart by what they leave behind: a refused charge for an
//!    entry (a candidate, a 2.3 rule diagnostic, a closure branch warning) is a *publication*
//!    stop and declares no skipped range, while a truncated listing is a search stop that keeps
//!    the positions it declared and never examined;
//! 6. a range position the environment cannot resolve inside the range is undecided, not
//!    excluded: the candidate is absent *and* the resolution plane is partial;
//! 7. a request without a range, and a request whose declaration did not resolve, produce no
//!    dispatch at all — never an empty candidate list.
//!
//! Fixtures are built from one class-file writer and stored ZIPs; no framework.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const STORE: u16 = 0;

/// The member and class access flags these fixtures use (JVMS 4.1/4.6).
const PUBLIC: u16 = 0x0001;
const PRIVATE: u16 = 0x0002;
const STATIC: u16 = 0x0008;
const FINAL: u16 = 0x0010;
const INTERFACE: u16 = 0x0200;
const ABSTRACT: u16 = 0x0400;

/// The `major_version` every fixture shares: Java 8, the profile this change resolves under.
const MAJOR: u16 = 52;

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 22,
        archive_entries: 10_000,
        entry_bytes: 1 << 22,
        read_bytes: 1 << 22,
        class_bytes: 1 << 22,
        attribute_bytes: 1 << 22,
        output_bytes: 1 << 22,
        // Funded for the one body-path control; the dispatch plane never charges it.
        code_bytes: 1 << 20,
        result_items: 100_000,
        class_headers: 200,
        nested_depth: 4,
        dependency_depth: 8,
        analysis_steps: 100_000,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

/// One field or method declaration of a fixture class.
#[derive(Clone)]
struct Member {
    name: Vec<u8>,
    descriptor: Vec<u8>,
    access_flags: u16,
    /// Whether the declaration carries a one-instruction `Code` attribute.
    body: bool,
}

/// One class file, built from its own declaration list.
#[derive(Clone)]
struct Class {
    name: Vec<u8>,
    super_class: Option<Vec<u8>>,
    interfaces: Vec<Vec<u8>>,
    access_flags: u16,
    fields: Vec<Member>,
    methods: Vec<Member>,
}

impl Class {
    fn new(name: &[u8]) -> Self {
        Self {
            name: name.to_vec(),
            super_class: Some(b"java/lang/Object".to_vec()),
            interfaces: Vec::new(),
            access_flags: PUBLIC | 0x0020,
            fields: Vec::new(),
            methods: Vec::new(),
        }
    }

    /// A class with no superclass: the hierarchy root a gap-free fixture needs.
    fn root(name: &[u8]) -> Self {
        Self {
            super_class: None,
            ..Self::new(name)
        }
    }

    fn interface(name: &[u8]) -> Self {
        Self {
            access_flags: PUBLIC | INTERFACE | ABSTRACT,
            ..Self::new(name)
        }
    }

    /// A non-interface class that declares itself abstract, so its declarations may be abstract.
    fn abstract_class(name: &[u8]) -> Self {
        Self {
            access_flags: PUBLIC | ABSTRACT,
            ..Self::new(name)
        }
    }

    fn extends(mut self, name: &[u8]) -> Self {
        self.super_class = Some(name.to_vec());
        self
    }

    fn implements(mut self, name: &[u8]) -> Self {
        self.interfaces.push(name.to_vec());
        self
    }

    fn field(mut self, name: &[u8], descriptor: &[u8], access_flags: u16) -> Self {
        self.fields.push(Member {
            name: name.to_vec(),
            descriptor: descriptor.to_vec(),
            access_flags,
            body: false,
        });
        self
    }

    fn method(mut self, name: &[u8], descriptor: &[u8], access_flags: u16) -> Self {
        self.methods.push(Member {
            name: name.to_vec(),
            descriptor: descriptor.to_vec(),
            access_flags,
            body: false,
        });
        self
    }

    /// One method that really carries a body, so the body-path control can charge code bytes.
    fn method_with_body(mut self, name: &[u8], descriptor: &[u8], access_flags: u16) -> Self {
        self.methods.push(Member {
            name: name.to_vec(),
            descriptor: descriptor.to_vec(),
            access_flags,
            body: true,
        });
        self
    }

    /// The internal name this fixture declares as its own `this_class`.
    fn name(&self) -> Vec<u8> {
        self.name.clone()
    }

    fn build(&self) -> Vec<u8> {
        let mut text: Vec<Vec<u8>> = Vec::new();
        let name_index = utf8_index(&mut text, &self.name);
        let super_index = self
            .super_class
            .as_ref()
            .map(|name| utf8_index(&mut text, name));
        let interface_indexes = self
            .interfaces
            .iter()
            .map(|name| utf8_index(&mut text, name))
            .collect::<Vec<_>>();
        let bodies = self
            .fields
            .iter()
            .chain(self.methods.iter())
            .any(|member| member.body);
        let code_index = bodies.then(|| utf8_index(&mut text, b"Code"));
        let declared = |members: &[Member], text: &mut Vec<Vec<u8>>| {
            members
                .iter()
                .map(|member| {
                    (
                        member.access_flags,
                        utf8_index(text, &member.name),
                        utf8_index(text, &member.descriptor),
                    )
                })
                .collect::<Vec<_>>()
        };
        let fields = declared(&self.fields, &mut text);
        let methods = declared(&self.methods, &mut text);

        // The class entries come after every `CONSTANT_Utf8` entry, so they can reference them.
        let mut classes = vec![name_index];
        if let Some(super_index) = super_index {
            classes.push(super_index);
        }
        classes.extend(interface_indexes.iter().copied());
        let text_count = u16::try_from(text.len()).expect("fixture pool fits u16");
        let class_ref = |position: usize| -> u16 {
            text_count + 1 + u16::try_from(position).expect("fixture class index fits u16")
        };
        let this_class = class_ref(0);
        let super_class = super_index.map(|_| class_ref(1));
        let offset = usize::from(super_index.is_some()) + 1;
        let interface_refs = (0..interface_indexes.len())
            .map(|position| class_ref(offset + position))
            .collect::<Vec<_>>();

        let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // minor
        bytes.extend_from_slice(&MAJOR.to_be_bytes());
        bytes.extend_from_slice(
            &u16::try_from(1 + text.len() + classes.len())
                .expect("fixture pool count fits u16")
                .to_be_bytes(),
        );
        for value in &text {
            bytes.push(1_u8); // CONSTANT_Utf8
            bytes.extend_from_slice(
                &u16::try_from(value.len())
                    .expect("fixture name fits u16")
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(value);
        }
        for value in &classes {
            bytes.push(7_u8); // CONSTANT_Class
            bytes.extend_from_slice(&value.to_be_bytes());
        }
        bytes.extend_from_slice(&self.access_flags.to_be_bytes());
        bytes.extend_from_slice(&this_class.to_be_bytes());
        bytes.extend_from_slice(&super_class.unwrap_or(0).to_be_bytes());
        bytes.extend_from_slice(
            &u16::try_from(interface_refs.len())
                .expect("fixture interface count fits u16")
                .to_be_bytes(),
        );
        for reference in &interface_refs {
            bytes.extend_from_slice(&reference.to_be_bytes());
        }
        bytes.extend_from_slice(
            &u16::try_from(fields.len())
                .expect("fixture field count fits u16")
                .to_be_bytes(),
        );
        for (access, name, descriptor) in &fields {
            bytes.extend_from_slice(&access.to_be_bytes());
            bytes.extend_from_slice(&name.to_be_bytes());
            bytes.extend_from_slice(&descriptor.to_be_bytes());
            bytes.extend_from_slice(&0_u16.to_be_bytes());
        }
        bytes.extend_from_slice(
            &u16::try_from(self.methods.len())
                .expect("fixture method count fits u16")
                .to_be_bytes(),
        );
        for (index, (access, name, descriptor)) in methods.iter().enumerate() {
            bytes.extend_from_slice(&access.to_be_bytes());
            bytes.extend_from_slice(&name.to_be_bytes());
            bytes.extend_from_slice(&descriptor.to_be_bytes());
            if self.methods[index].body {
                bytes.extend_from_slice(&1_u16.to_be_bytes());
                bytes.extend_from_slice(
                    &code_index
                        .expect("a body declares the Code attribute name")
                        .to_be_bytes(),
                );
                // max_stack, max_locals, code_length, one `return`, no handlers, no attributes.
                let code: &[u8] = &[0, 0, 0, 1, 0, 0, 0, 1, 0xb1, 0, 0, 0, 0];
                bytes.extend_from_slice(
                    &u32::try_from(code.len())
                        .expect("fixture code fits u32")
                        .to_be_bytes(),
                );
                bytes.extend_from_slice(code);
            } else {
                bytes.extend_from_slice(&0_u16.to_be_bytes());
            }
        }
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // class attributes
        bytes
    }
}

/// One `CONSTANT_Utf8` index of a value, adding the entry when it is new.
fn utf8_index(text: &mut Vec<Vec<u8>>, value: &[u8]) -> u16 {
    if let Some(position) = text.iter().position(|known| known == value) {
        return u16::try_from(position).expect("fixture pool index fits u16") + 1;
    }
    text.push(value.to_vec());
    u16::try_from(text.len()).expect("fixture pool index fits u16")
}

fn zip_of(entries: &[(Vec<u8>, Vec<u8>)]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.clone()))
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

fn entry(class: &[u8]) -> Vec<u8> {
    let mut name = class.to_vec();
    name.extend_from_slice(b".class");
    name
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut Budget::new(limits()))
        .expect("the fixture snapshot opens")
}

fn loader(name: &str) -> LoaderId {
    LoaderId(name.to_string())
}

fn domain(loader: &LoaderId, parent: Option<LoaderId>, roots: Vec<LoadRoot>) -> LoadDomain {
    LoadDomain {
        loader: loader.clone(),
        parent_loader: parent,
        delegation: DelegationPolicy::ParentFirst,
        roots,
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    }
}

/// The load root one fixture's own content is: a standalone CLASS snapshot is one whole
/// definition, and a ZIP snapshot is searched in its root container with an empty prefix.
fn snapshot_root(snapshot: &ArtifactSnapshot) -> LoadRoot {
    match snapshot.kind() {
        ArtifactKind::StandaloneClass => LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        },
        ArtifactKind::Zip => LoadRoot::Container {
            origin: ContainerOrigin {
                snapshot: snapshot.id().clone(),
                root_container: ContainerId("root".into()),
                steps: Vec::new(),
            },
            prefix: ArchiveNameBytes(Vec::new()),
        },
    }
}

fn environment(
    snapshot: &ArtifactSnapshot,
    caller: LoadDomain,
    domains: Vec<LoadDomain>,
) -> ResolutionEnvironment {
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
            load_domain: caller,
        },
        domains,
        providers: Vec::new(),
    }
}

/// The classes every dispatch test resolves over, in range order.
///
/// `java/lang/Object` is present so a complete range has complete supertype closures; `i/I` and
/// `i/J` are abstract interface declarations, `i/K` declares a field, `p/H` implements `i/K` and
/// declares the same field, `p/C` implements `i/I` without declaring `m` (the negative
/// controls), `p/A`/`p/B`/`p/D` implement `i/I` and declare `m` themselves, and `notes.txt` is
/// not a class entry at all. The classes that are no candidate of `i/I.m` come before the
/// candidates on purpose: a budget stop then leaves a proper prefix of the candidates.
fn class_world() -> Vec<(Vec<u8>, Vec<u8>)> {
    let classes = vec![
        Class::root(b"java/lang/Object"),
        Class::interface(b"i/I").method(b"m", b"()V", PUBLIC | ABSTRACT),
        Class::interface(b"i/J").method(b"m", b"()V", PUBLIC | ABSTRACT),
        Class::interface(b"i/K").field(b"f", b"I", PUBLIC | STATIC | FINAL),
        Class::new(b"p/H")
            .implements(b"i/K")
            .field(b"f", b"I", PUBLIC),
        Class::new(b"p/C")
            .implements(b"i/I")
            .method(b"other", b"()V", PUBLIC),
        Class::new(b"p/A")
            .implements(b"i/I")
            .implements(b"i/J")
            .method_with_body(b"m", b"()V", PUBLIC),
        Class::new(b"p/B")
            .implements(b"i/I")
            .method(b"m", b"()V", PUBLIC),
        Class::new(b"p/D")
            .implements(b"i/I")
            .method(b"m", b"()V", PUBLIC),
    ];
    let mut entries = classes
        .iter()
        .map(|class| (entry(&class.name()), class.build()))
        .collect::<Vec<_>>();
    entries.push((b"notes.txt".to_vec(), b"not a class".to_vec()));
    entries
}

fn world_snapshot() -> ArtifactSnapshot {
    open(zip_of(&class_world()))
}

/// A snapshot that holds no class of the world: a second position of an ordered-root fixture.
fn other_snapshot() -> ArtifactSnapshot {
    open(zip_of(&[(
        entry(b"z/Z"),
        Class::new(b"z/Z").method(b"run", b"()V", PUBLIC).build(),
    )]))
}

/// A snapshot that holds one class *name* of the world, so an earlier position can shadow it.
fn shadow_snapshot() -> ArtifactSnapshot {
    open(zip_of(&[(
        entry(b"p/B"),
        Class::new(b"p/B")
            .implements(b"i/I")
            .method(b"m", b"()V", PUBLIC)
            .build(),
    )]))
}

/// One range whose last class reaches a second dependency layer.
///
/// `p/A` is already a candidate before `p/Deep` is walked, and `p/Deep` reaches
/// `java/lang/Object` only through `p/Base` — the layer `dependency_depth = 1` refuses.
fn deep_chain_world() -> Vec<(Vec<u8>, Vec<u8>)> {
    let classes = [
        Class::root(b"java/lang/Object"),
        Class::interface(b"i/I").method(b"m", b"()V", PUBLIC | ABSTRACT),
        Class::new(b"p/A")
            .implements(b"i/I")
            .method(b"m", b"()V", PUBLIC),
        Class::new(b"p/Base"),
        Class::new(b"p/Deep")
            .extends(b"p/Base")
            .implements(b"i/I")
            .method(b"m", b"()V", PUBLIC),
    ];
    classes
        .iter()
        .map(|class| (entry(&class.name()), class.build()))
        .collect()
}

/// One range twice, with a single class's `super_class` the only difference.
///
/// `p/CycA` is a candidate of `i/I.m` either way. With `cyclic`, `p/CycB` extends `p/CycA`, so
/// both classes' supertype closures are cycles the walk refuses; without it, `p/CycB` extends
/// `java/lang/Object` and the range is complete. The two fixtures have the same entry names, the
/// same class set and the same member declarations, so the only difference in the report is the
/// hierarchy the walks refused.
fn cycle_pair(cyclic: bool) -> Vec<(Vec<u8>, Vec<u8>)> {
    let second: &[u8] = if cyclic {
        b"p/CycA"
    } else {
        b"java/lang/Object"
    };
    let classes = [
        Class::root(b"java/lang/Object"),
        Class::interface(b"i/I").method(b"m", b"()V", PUBLIC | ABSTRACT),
        Class::new(b"p/CycA")
            .extends(b"p/CycB")
            .implements(b"i/I")
            .method(b"m", b"()V", PUBLIC),
        Class::new(b"p/CycB").extends(second),
    ];
    classes
        .iter()
        .map(|class| (entry(&class.name()), class.build()))
        .collect()
}

/// A tree fixture that holds one class name twice: `p/B` in the root container and in the
/// nested JAR, so an artifact-tree range lists the same name from two containers.
fn tree_with_a_shared_name() -> ArtifactSnapshot {
    let inner = zip_of(&[(
        entry(b"p/B"),
        Class::new(b"p/B")
            .implements(b"i/I")
            .method(b"m", b"()V", PUBLIC)
            .build(),
    )]);
    open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"i/I"),
            Class::interface(b"i/I")
                .method(b"m", b"()V", PUBLIC | ABSTRACT)
                .build(),
        ),
        (
            entry(b"p/B"),
            Class::new(b"p/B")
                .implements(b"i/I")
                .method(b"m", b"()V", PUBLIC)
                .build(),
        ),
        (b"lib/inner.jar".to_vec(), inner),
    ]))
}

/// The declared roots of one tree fixture, the root container first, and its tree scope.
fn tree_range(snapshot: &ArtifactSnapshot) -> (Vec<LoadRoot>, PhysicalScope) {
    let mut budget = Budget::new(limits());
    let tree = Engine::new()
        .enumerate_artifact_tree(snapshot, &mut budget)
        .expect("the fixture tree enumerates");
    let root = tree
        .containers
        .iter()
        .find(|container| container.origin.steps.is_empty())
        .expect("the fixture has a root container")
        .origin
        .clone();
    let nested = tree
        .containers
        .iter()
        .find(|container| !container.origin.steps.is_empty())
        .expect("the fixture has a nested container")
        .origin
        .clone();
    let scope = PhysicalScope::ArtifactTree {
        root_container: root.current_container().clone(),
    };
    (
        vec![
            LoadRoot::Container {
                origin: root,
                prefix: ArchiveNameBytes(Vec::new()),
            },
            LoadRoot::Container {
                origin: nested,
                prefix: ArchiveNameBytes(Vec::new()),
            },
        ],
        scope,
    )
}

/// One `provider_search_position` coverage range of the resolution plane.
fn search_range(start: u64, end: u64) -> CoverageRange {
    CoverageRange {
        label: "provider_search_position".to_string(),
        start,
        end,
    }
}

fn method(owner: &[u8], name: &[u8], descriptor: &[u8]) -> SymbolRef {
    SymbolRef::Method {
        owner: JvmBytes(owner.to_vec()),
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
    }
}

fn field(owner: &[u8], name: &[u8], descriptor: &[u8]) -> SymbolRef {
    SymbolRef::Field {
        owner: JvmBytes(owner.to_vec()),
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
    }
}

/// One dispatch request over one member declaration inside one physical range.
fn dispatch_request(
    environment: ResolutionEnvironment,
    caller: &LoaderId,
    target: SymbolRef,
    use_kind: ReferenceUse,
    scope: PhysicalScope,
) -> ResolutionRequest {
    ResolutionRequest {
        environment,
        target,
        use_kind,
        caller: CallerContext {
            loader: caller.clone(),
            // No use site: the plane under test is the range, not the access rules.
            enclosing: None,
        },
        dispatch: Some(DispatchScope {
            scope,
            consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
        }),
    }
}

fn resolve(content: &[ArtifactSnapshot], request: &ResolutionRequest) -> ResolutionReport {
    resolve_under(content, request, &mut Budget::new(limits()))
}

fn resolve_under(
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

fn code_bytes(report: &ResolutionReport) -> u64 {
    usage_of(&report.execution).counted_usage(CountedBudgetDimension::CodeBytes)
}

fn archive_entries(report: &ResolutionReport) -> u64 {
    usage_of(&report.execution).counted_usage(CountedBudgetDimension::ArchiveEntries)
}

fn result_items(report: &ResolutionReport) -> u64 {
    usage_of(&report.execution).counted_usage(CountedBudgetDimension::ResultItems)
}

fn diagnostic_codes(report: &ResolutionReport) -> Vec<String> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.clone())
        .collect()
}

fn dispatch_of(report: &ResolutionReport) -> &DispatchReport {
    report
        .dispatch
        .as_ref()
        .expect("the request asked for a range and its declaration resolved")
}

/// The member owner bytes of every candidate, in published order.
fn candidate_owners(report: &ResolutionReport) -> Vec<Vec<u8>> {
    dispatch_of(report)
        .candidates
        .iter()
        .map(|candidate| match &candidate.member.member {
            SymbolRef::Field { owner, .. } | SymbolRef::Method { owner, .. } => owner.0.clone(),
            SymbolRef::Class { owner } => owner.0.clone(),
        })
        .collect()
}

/// The evidence of every candidate, in published order.
fn candidate_evidence(report: &ResolutionReport) -> Vec<Option<OpenWorldEvidence>> {
    dispatch_of(report)
        .candidates
        .iter()
        .map(|candidate| candidate.evidence.clone())
        .collect()
}

/// The candidates' physical definitions, in published order.
fn candidate_definitions(report: &ResolutionReport) -> Vec<PhysicalDefinitionId> {
    dispatch_of(report)
        .candidates
        .iter()
        .map(|candidate| candidate.member.definition.clone())
        .collect()
}

/// The physical definitions one report read under the `DispatchScope` reason.
fn dispatch_reads(report: &ResolutionReport) -> Vec<PhysicalDefinitionId> {
    report
        .reads
        .iter()
        .filter(|read| read.reason == ReadReason::DispatchScope)
        .map(|read| read.definition.clone())
        .collect()
}

/// The single-loader environment: one `app` loader whose only root holds the world.
fn single_loader(world: &ArtifactSnapshot) -> ResolutionEnvironment {
    let app = domain(&loader("app"), None, vec![snapshot_root(world)]);
    environment(world, app.clone(), vec![app])
}

// ---------------------------------------------------------------------------
// Candidate discovery and evidence
// ---------------------------------------------------------------------------

#[test]
fn every_implementation_in_the_range_is_one_known_candidate() {
    let world = world_snapshot();
    let environment = single_loader(&world);
    let content = std::slice::from_ref(&world);
    let request = dispatch_request(
        environment,
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        PhysicalScope::SnapshotAll,
    );
    let report = resolve(content, &request);

    // The declaration resolves by the 2.3 interface rules (an abstract interface declaration
    // resolves and carries its own warning), and the range answers with the classes that declare
    // `m` and reach `i/I` above themselves.
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        report
            .resolved
            .as_ref()
            .map(|resolved| resolved.member.clone()),
        Some(method(b"i/I", b"m", b"()V"))
    );
    let dispatch = dispatch_of(&report);
    assert_eq!(dispatch.scope, PhysicalScope::SnapshotAll);
    assert_eq!(
        candidate_owners(&report),
        vec![b"p/A".to_vec(), b"p/B".to_vec(), b"p/D".to_vec()],
        "every class of the range that declares the member and implements the owner, in range \
         order; `p/C` declares no `m`, `p/H` declares a field, and `i/J` is no subtype of `i/I`"
    );
    assert_eq!(
        candidate_evidence(&report),
        vec![None, None, None],
        "a complete range with no declared uncertainty states no open-world fact"
    );
    assert!(
        !dispatch.open_world,
        "the range was enumerated completely, every class resolved inside it and every closure \
         was read: {dispatch:?}"
    );
    // Every candidate is the class's own declaration: the owner is the candidate's class, and
    // the name and descriptor are the resolved declaration's bytes.
    for candidate in &dispatch.candidates {
        match &candidate.member.member {
            SymbolRef::Method {
                name, descriptor, ..
            } => {
                assert_eq!(name.0, b"m".to_vec());
                assert_eq!(descriptor.0, b"()V".to_vec());
            }
            other => panic!("a method declaration was enumerated: {other:?}"),
        }
    }
    assert!(
        dispatch
            .candidates
            .iter()
            .all(|candidate| candidate.member.loader == loader("app")),
        "every candidate was read in the caller's own order"
    );
    // The billing of the plane: one header attempt per class of the range (`i/I` is answered
    // from the request memo, because the declaration resolution already read it) plus the
    // declaration's own owner read, all of them recorded exactly once.
    assert_eq!(class_headers(&report), 9, "{:?}", report.reads);
    assert_eq!(report.reads.len(), 9);
    assert_eq!(
        report
            .reads
            .iter()
            .filter(|read| read.reason == ReadReason::DispatchScope)
            .count(),
        8,
        "every class of the range is a `DispatchScope` read: {:?}",
        report.reads
    );
}

#[test]
fn a_runtime_transformation_moves_only_the_open_world_flag() {
    let world = world_snapshot();
    let plain = single_loader(&world);
    let content = std::slice::from_ref(&world);
    let target = method(b"i/I", b"m", b"()V");
    let complete = resolve(
        content,
        &dispatch_request(
            plain,
            &loader("app"),
            target.clone(),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );

    // The same fixture with one declared uncertainty: the candidate set is a fact about the
    // range and does not move, while the open-world plane does.
    let mut app = domain(&loader("app"), None, vec![snapshot_root(&world)]);
    app.runtime_transformation = RuntimeUncertainty::Possible;
    let uncertain = environment(&world, app.clone(), vec![app]);
    let report = resolve(
        content,
        &dispatch_request(
            uncertain,
            &loader("app"),
            target,
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );

    assert_eq!(candidate_owners(&report), candidate_owners(&complete));
    assert_eq!(
        candidate_definitions(&report),
        candidate_definitions(&complete),
        "the same physical definitions, not merely the same names"
    );
    assert_eq!(
        candidate_evidence(&report),
        vec![
            Some(OpenWorldEvidence::RuntimeTransformation),
            Some(OpenWorldEvidence::RuntimeTransformation),
            Some(OpenWorldEvidence::RuntimeTransformation),
        ]
    );
    assert!(dispatch_of(&report).open_world);
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema,
        "the range was still enumerated completely: `open_world` is its own plane"
    );
    assert_eq!(
        class_headers(&report),
        class_headers(&complete),
        "the range reads are the same reads"
    );
}

#[test]
fn an_ordered_root_is_evidence_only_past_the_first_position() {
    let world = world_snapshot();
    let other = other_snapshot();
    let content = vec![other.clone(), world.clone()];
    // The world sits at declared root index 1: the search passes over position 0 first.
    let app = domain(
        &loader("app"),
        None,
        vec![snapshot_root(&other), snapshot_root(&world)],
    );
    let environment = environment(&world, app.clone(), vec![app]);
    let report = resolve(
        &content,
        &dispatch_request(
            environment,
            &loader("app"),
            method(b"i/I", b"m", b"()V"),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );

    assert_eq!(
        candidate_owners(&report),
        vec![b"p/A".to_vec(), b"p/B".to_vec(), b"p/D".to_vec()]
    );
    assert_eq!(
        candidate_evidence(&report),
        vec![
            Some(OpenWorldEvidence::OrderedRoot { index: 1 }),
            Some(OpenWorldEvidence::OrderedRoot { index: 1 }),
            Some(OpenWorldEvidence::OrderedRoot { index: 1 }),
        ],
        "the candidates really came from the second declared root of their loader"
    );
    assert!(dispatch_of(&report).open_world);

    // The same range under a single root: the candidates are decided at declared root index 0,
    // which is no open-world evidence at all.
    let single = single_loader(&world);
    let report = resolve(
        std::slice::from_ref(&world),
        &dispatch_request(
            single,
            &loader("app"),
            method(b"i/I", b"m", b"()V"),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );
    assert_eq!(candidate_evidence(&report), vec![None, None, None]);
    assert!(!dispatch_of(&report).open_world);
}

#[test]
fn an_external_root_is_refused_before_the_range_is_enumerated() {
    // 1.1 rejects an environment that declares an unreadable root outright, so the declaration
    // never resolves and the dispatch plane never runs. That is why the `ExternalSubclass`
    // classification is not reachable through a performed request: the external root cannot
    // coexist with a resolution, and the requested range is named instead of enumerated.
    let world = world_snapshot();
    let app = domain(
        &loader("app"),
        None,
        vec![
            snapshot_root(&world),
            LoadRoot::External {
                id: "boot".to_string(),
            },
        ],
    );
    let environment = environment(&world, app.clone(), vec![app]);
    let request = dispatch_request(
        environment,
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        PhysicalScope::SnapshotAll,
    );
    let report = resolve(std::slice::from_ref(&world), &request);

    assert_eq!(report.analysis, ResolutionAnalysis::NotPerformed);
    assert_eq!(report.state, None);
    assert!(
        report.dispatch.is_none(),
        "a range whose declaration never resolved must not look like an empty one"
    );
    assert!(
        report
            .environment_problems
            .iter()
            .any(|problem| problem.code == EnvironmentProblemCode::UnreadableRoot),
        "the external root is a rejected declaration: {:?}",
        report.environment_problems
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec![
            "unreadable_root",
            "resolution_not_implemented",
            "resolution_dispatch_no_declaration"
        ],
        "no candidate and no evidence is published: the range was not enumerated"
    );
    assert_eq!(
        class_headers(&report),
        0,
        "a rejected environment reads nothing"
    );
    assert!(report.reads.is_empty());
}

#[test]
fn a_declared_loader_outside_the_chain_is_an_unknown_loader_fact() {
    let world = world_snapshot();
    let other = other_snapshot();
    let content = vec![world.clone(), other.clone()];
    // The caller's chain is just `app`; the second declared loader is never searched, so a class
    // of the range may also be loaded by it.
    let app = domain(&loader("app"), None, vec![snapshot_root(&world)]);
    let dangling = domain(&loader("platform"), None, vec![snapshot_root(&other)]);
    let environment = environment(&world, app.clone(), vec![app, dangling]);
    let report = resolve(
        &content,
        &dispatch_request(
            environment,
            &loader("app"),
            method(b"i/I", b"m", b"()V"),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );

    assert_eq!(
        candidate_owners(&report),
        vec![b"p/A".to_vec(), b"p/B".to_vec(), b"p/D".to_vec()]
    );
    assert_eq!(
        candidate_evidence(&report),
        vec![
            Some(OpenWorldEvidence::UnknownLoader),
            Some(OpenWorldEvidence::UnknownLoader),
            Some(OpenWorldEvidence::UnknownLoader),
        ]
    );
    assert!(dispatch_of(&report).open_world);
}

#[test]
fn a_missing_supertype_branch_is_the_candidates_own_dependency_fact() {
    let mut classes = class_world();
    // `p/E` implements `i/I` and declares `m`, but its superclass is not provided: the candidate
    // exists and its own closure states the missing dependency.
    let orphan = Class::new(b"p/E")
        .extends(b"p/Absent")
        .implements(b"i/I")
        .method(b"m", b"()V", PUBLIC);
    classes.insert(classes.len() - 1, (entry(b"p/E"), orphan.build()));
    let world = open(zip_of(&classes));
    let environment = single_loader(&world);
    let report = resolve(
        std::slice::from_ref(&world),
        &dispatch_request(
            environment,
            &loader("app"),
            method(b"i/I", b"m", b"()V"),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );

    assert_eq!(
        candidate_owners(&report),
        vec![
            b"p/A".to_vec(),
            b"p/B".to_vec(),
            b"p/D".to_vec(),
            b"p/E".to_vec()
        ]
    );
    assert_eq!(
        candidate_evidence(&report),
        vec![None, None, None, Some(OpenWorldEvidence::MissingDependency),],
        "only the candidate whose own supertype closure is unread carries the fact"
    );
    assert!(dispatch_of(&report).open_world);
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial,
        "an unread branch makes the plane partial: {:?}",
        report.coverage
    );
}

#[test]
fn a_field_declaration_enumerates_the_classes_that_declare_the_same_field() {
    let world = world_snapshot();
    let environment = single_loader(&world);
    let report = resolve(
        std::slice::from_ref(&world),
        &dispatch_request(
            environment,
            &loader("app"),
            field(b"i/K", b"f", b"I"),
            ReferenceUse::FieldRead,
            PhysicalScope::SnapshotAll,
        ),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(candidate_owners(&report), vec![b"p/H".to_vec()]);
    assert_eq!(candidate_evidence(&report), vec![None]);
    match &dispatch_of(&report).candidates[0].member.member {
        SymbolRef::Field {
            name, descriptor, ..
        } => {
            assert_eq!(name.0, b"f".to_vec());
            assert_eq!(descriptor.0, b"I".to_vec());
        }
        other => panic!("a field declaration was enumerated: {other:?}"),
    }
    assert!(!dispatch_of(&report).open_world);
}

// ---------------------------------------------------------------------------
// The open-world plane is not a uniqueness claim
// ---------------------------------------------------------------------------

#[test]
fn a_single_known_candidate_is_never_published_as_a_unique_target() {
    let world = world_snapshot();
    let plain = single_loader(&world);
    let report = resolve(
        std::slice::from_ref(&world),
        &dispatch_request(
            plain,
            &loader("app"),
            method(b"i/J", b"m", b"()V"),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );

    assert_eq!(candidate_owners(&report), vec![b"p/A".to_vec()]);
    assert_eq!(candidate_evidence(&report), vec![None]);
    assert!(
        !dispatch_of(&report).open_world,
        "one candidate from a complete range is still one known candidate"
    );

    // The structure carries no uniqueness: the dispatch report's keys and a candidate's keys are
    // exactly the designed ones, so no caller can read a unique target out of them.
    let json = serde_json::to_value(dispatch_of(&report)).expect("the report serializes");
    let mut report_keys = json
        .as_object()
        .expect("a dispatch report is an object")
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    report_keys.sort();
    assert_eq!(report_keys, vec!["candidates", "open_world", "scope"]);
    let mut candidate_keys = json["candidates"][0]
        .as_object()
        .expect("a candidate is an object")
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    candidate_keys.sort();
    assert_eq!(candidate_keys, vec!["evidence", "member"]);
    for forbidden in ["unique", "runtime_target", "single_target", "only_target"] {
        assert!(
            !json.to_string().contains(forbidden),
            "no field of the plane may name `{forbidden}`"
        );
    }

    // One candidate and one declared uncertainty: the plane still says the range is open, so a
    // caller cannot read "one candidate" as "the target".
    let mut app = domain(&loader("app"), None, vec![snapshot_root(&world)]);
    app.external_override = RuntimeUncertainty::Possible;
    let uncertain = environment(&world, app.clone(), vec![app]);
    let report = resolve(
        std::slice::from_ref(&world),
        &dispatch_request(
            uncertain,
            &loader("app"),
            method(b"i/J", b"m", b"()V"),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );
    assert_eq!(candidate_owners(&report), vec![b"p/A".to_vec()]);
    assert_eq!(
        candidate_evidence(&report),
        vec![Some(OpenWorldEvidence::RuntimeTransformation)],
        "the single candidate states its own fact instead of standing for a unique target"
    );
    assert!(dispatch_of(&report).open_world);
}

// ---------------------------------------------------------------------------
// Header-only reads and the stop semantics
// ---------------------------------------------------------------------------

#[test]
fn the_range_reads_headers_only_and_never_a_method_body() {
    let world = world_snapshot();
    let environment = single_loader(&world);
    let content = std::slice::from_ref(&world);
    let report = resolve(
        content,
        &dispatch_request(
            environment,
            &loader("app"),
            method(b"i/I", b"m", b"()V"),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );

    assert!(
        class_headers(&report) > 0,
        "the range really read headers: {}",
        class_headers(&report)
    );
    assert_eq!(
        code_bytes(&report),
        0,
        "the range read no instruction byte: {report:?}"
    );
    assert!(
        !dispatch_reads(&report).is_empty(),
        "the range's reads carry the `DispatchScope` reason: {:?}",
        report.reads
    );

    // Control: the same fixture's `p/A.m` really carries a body, and the P1 body path charges
    // code bytes for it, so the zero above is a fact about the dispatch plane and not about the
    // fixture. (`method_bodies` is not the evidence: it has no charge point before 3.x, so a
    // zero there would say nothing about what was read.)
    let mut listing_budget = Budget::new(limits());
    let listing = Engine::new()
        .enumerate(&world, &mut listing_budget)
        .expect("the fixture archive lists");
    let body_entry = listing
        .entries
        .iter()
        .find(|listed| listed.id.raw_name.0 == entry(b"p/A"))
        .expect("the fixture holds the class with a body");
    let mut body_budget = Budget::new(limits());
    let body = Engine::new()
        .inspect_method_bytecode(
            &world,
            ClassTarget::Entry(body_entry),
            MethodSelector {
                name: JvmBytes(b"m".to_vec()),
                descriptor: JvmBytes(b"()V".to_vec()),
            },
            &mut body_budget,
        )
        .expect("the fixture body is readable");
    assert!(!body.inspection.instructions.is_empty());
    assert!(
        body_budget.usage().code_bytes > 0,
        "the body path charges code bytes on this fixture, otherwise the assertion above is \
         vacuous: {}",
        body_budget.usage().code_bytes
    );
}

#[test]
fn a_class_header_stop_keeps_the_candidates_already_found() {
    let world = world_snapshot();
    let environment = single_loader(&world);
    let content = std::slice::from_ref(&world);
    let request = dispatch_request(
        environment,
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        PhysicalScope::SnapshotAll,
    );
    let complete = resolve(content, &request);
    let total = class_headers(&complete);
    assert_eq!(candidate_owners(&complete).len(), 3);

    // One header attempt less than the complete run: the last class of the range cannot be read,
    // so the plane keeps the candidates it already published and reports the stop.
    let mut budget = Budget::new(Limits {
        class_headers: total - 1,
        ..limits()
    });
    let report = resolve_under(content, &request, &mut budget);

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    let stopped = candidate_owners(&report);
    assert!(
        !stopped.is_empty() && stopped.len() < candidate_owners(&complete).len(),
        "the stop keeps the trustworthy prefix: {stopped:?}"
    );
    assert_eq!(
        stopped,
        candidate_owners(&complete)[..stopped.len()].to_vec(),
        "the prefix is the first candidates of the range order"
    );
    assert!(dispatch_of(&report).open_world);
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ClassHeaders
            },
            ..
        }
    ));
    assert!(
        diagnostic_codes(&report).contains(&"budget_exceeded_class_headers".to_string()),
        "{:?}",
        report.diagnostics
    );
    // The stop left positions the order declared and never examined, and the closure knows it:
    // the declaration's own search and the seven range lookups before the refusal examined one
    // position each, and the refused ninth search declared its own position without examining
    // it. Dropping that range would publish a plane that looks finished up to its last examined
    // position.
    let coverage = &report.coverage.runtime_resolution;
    assert_eq!(coverage.state, CoverageState::Partial);
    assert_eq!(coverage.scanned, vec![search_range(0, 8)], "{coverage:?}");
    assert_eq!(coverage.skipped, vec![search_range(8, 9)], "{coverage:?}");
}

#[test]
fn a_result_items_stop_publishes_no_skipped_range() {
    // The publication phase is not a search phase. Two declared roots make that observable:
    // every search of this fixture finds its class at the first position and declares both, so
    // the closure's own sums are nine examined positions against eighteen declared ones. A
    // refused charge for publishing one candidate ended no search — the searches that ran
    // reached their own conclusions and the order stops by rule at the first position that holds
    // the class — so reading the remaining positions as skipped would claim an unfinished
    // search this request never had.
    let world = world_snapshot();
    let other = other_snapshot();
    let content = vec![world.clone(), other.clone()];
    let app = domain(
        &loader("app"),
        None,
        vec![snapshot_root(&world), snapshot_root(&other)],
    );
    let request = dispatch_request(
        environment(&world, app.clone(), vec![app]),
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        PhysicalScope::SnapshotAll,
    );
    let complete = resolve(&content, &request);
    assert_eq!(
        complete.coverage.runtime_resolution.skipped,
        Vec::new(),
        "the complete run's searches stopped by rule, which is no skipped range"
    );

    let mut budget = Budget::new(Limits {
        result_items: result_items(&complete) - 1,
        ..limits()
    });
    let report = resolve_under(&content, &request, &mut budget);

    assert_eq!(
        candidate_owners(&report),
        candidate_owners(&complete)[..2].to_vec(),
        "the refused charge keeps the candidates already published"
    );
    assert_eq!(
        report.state,
        Some(ResolutionState::Resolved),
        "the plane's stop is published in `execution` and never rewrites the declaration's decision"
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert!(dispatch_of(&report).open_world);
    let coverage = &report.coverage.runtime_resolution;
    assert_eq!(coverage.state, CoverageState::Partial);
    assert!(!coverage.scanned.is_empty(), "{coverage:?}");
    assert_eq!(
        coverage.skipped,
        Vec::new(),
        "a stop of the publication phase declares no unfinished search: {coverage:?}"
    );
}

#[test]
fn a_dependency_depth_stop_keeps_the_prefix_and_the_positions_it_declared() {
    // `p/Deep` reaches `java/lang/Object` only through `p/Base`, so with `dependency_depth = 1`
    // its walk is refused one layer above its superclass — after `p/A` was published and after
    // every class of the range before it was decided. The stop is the plane's own: the
    // declaration stays `Resolved` and the interruption lives in `execution`, `open_world` and
    // the coverage planes.
    let world = open(zip_of(&deep_chain_world()));
    let other = other_snapshot();
    let content = vec![world.clone(), other.clone()];
    let app = domain(
        &loader("app"),
        None,
        vec![snapshot_root(&world), snapshot_root(&other)],
    );
    let request = dispatch_request(
        environment(&world, app.clone(), vec![app]),
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        PhysicalScope::SnapshotAll,
    );

    // Control: the same fixture without the depth limit walks `p/Deep` completely, publishes it
    // and reaches the end of its range.
    let complete = resolve(&content, &request);
    assert_eq!(
        candidate_owners(&complete),
        vec![b"p/A".to_vec(), b"p/Deep".to_vec()]
    );
    assert_eq!(
        complete.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema
    );

    let mut budget = Budget::new(Limits {
        dependency_depth: 1,
        ..limits()
    });
    let report = resolve_under(&content, &request, &mut budget);

    assert_eq!(
        report.state,
        Some(ResolutionState::Resolved),
        "the stop of the plane does not rewrite the declaration's own decision"
    );
    assert_eq!(
        candidate_owners(&report),
        vec![b"p/A".to_vec()],
        "the candidates found before the refused layer, and not the class whose walk stopped"
    );
    assert_eq!(
        candidate_evidence(&report),
        vec![None],
        "the prefix is decided at the first declared root and states no open-world fact of its \
         own: `open_world` is true because the plane stopped"
    );
    assert!(dispatch_of(&report).open_world);
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::DependencyDepth
            },
            ..
        }
    ));
    assert!(
        diagnostic_codes(&report).contains(&"budget_exceeded_dependency_depth".to_string()),
        "{:?}",
        report.diagnostics
    );
    // The closure's sums: five searches ran (`i/I` for the declaration, then `java/lang/Object`,
    // `p/A`, `p/Base` and `p/Deep` for the range — `i/I` itself is answered from the request
    // memo), each finding its class at the first of the two declared positions, so nine
    // positions were declared and five examined.
    let coverage = &report.coverage.runtime_resolution;
    assert_eq!(coverage.state, CoverageState::Partial);
    assert_eq!(coverage.scanned, vec![search_range(0, 5)], "{coverage:?}");
    assert_eq!(coverage.skipped, vec![search_range(5, 10)], "{coverage:?}");
}

#[test]
fn a_range_whose_listing_cannot_be_paid_for_is_a_truncated_range() {
    // `archive_entries` is charged while the containers are listed and read. The declaration
    // resolution of this fixture costs eleven units; the twelfth reaches the 2.5 plane, whose own
    // listing is refused at its first entry. No class of the range was listed, so no candidate
    // can be published — and the range the plane did reach is the empty prefix it could list.
    let world = world_snapshot();
    let environment = single_loader(&world);
    let request = dispatch_request(
        environment,
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        PhysicalScope::SnapshotAll,
    );
    let content = std::slice::from_ref(&world);

    let mut budget = Budget::new(Limits {
        archive_entries: 12,
        ..limits()
    });
    let report = resolve_under(content, &request, &mut budget);

    assert_eq!(
        report.state,
        Some(ResolutionState::Resolved),
        "the declaration resolved before the range's own listing was refused"
    );
    let dispatch = dispatch_of(&report);
    assert!(
        dispatch.candidates.is_empty(),
        "the truncated listing published no class: {:?}",
        dispatch.candidates
    );
    assert!(
        dispatch.open_world,
        "a range this plane could not list is an unknown part of the range"
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
    assert!(
        diagnostic_codes(&report).contains(&"budget_exceeded_archive_entries".to_string()),
        "{:?}",
        report.diagnostics
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );

    // The boundary this test rests on: one unit less and the declaration itself never resolved,
    // so the range was never asked for at all.
    let mut short = Budget::new(Limits {
        archive_entries: 11,
        ..limits()
    });
    let short = resolve_under(content, &request, &mut short);
    assert_eq!(short.state, Some(ResolutionState::BudgetExceeded));
    assert!(
        short.dispatch.is_none(),
        "a declaration that stopped publishes no range"
    );
}

#[test]
fn a_truncated_listing_keeps_the_positions_it_declared_but_never_examined() {
    // The truncation case above is single-root: every search of it found its class at the only
    // position of its order, so `examined == positions`, both classifications of that stop are
    // empty, and what the stop does to the skipped range cannot be observed there. Two declared
    // roots make it observable: the one search this request ran examined position 0 of two, and
    // the listing that was cut short never reached the classes behind it, so the position it
    // declared and never examined stays visible as unfinished range — this stop ends a search of
    // the plane, unlike the publication stop of a refused candidate charge. The boundary is the
    // same as above: the declaration's own lookup costs eleven `archive_entries` units and the
    // twelfth is the range's listing, refused at its first entry.
    let world = world_snapshot();
    let other = other_snapshot();
    let content = vec![world.clone(), other.clone()];
    let app = domain(
        &loader("app"),
        None,
        vec![snapshot_root(&world), snapshot_root(&other)],
    );
    let request = dispatch_request(
        environment(&world, app.clone(), vec![app]),
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        PhysicalScope::SnapshotAll,
    );

    // Control: a budget that can pay for the same range reaches the end of it, so nothing about
    // this fixture is incomplete on its own.
    let complete = resolve(&content, &request);
    assert!(matches!(
        complete.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(
        complete.coverage.runtime_resolution.skipped,
        Vec::new(),
        "a complete range declares no unexamined position"
    );

    let mut budget = Budget::new(Limits {
        archive_entries: 12,
        ..limits()
    });
    let report = resolve_under(&content, &request, &mut budget);

    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ArchiveEntries
            },
            ..
        }
    ));
    assert!(
        diagnostic_codes(&report).contains(&"budget_exceeded_archive_entries".to_string()),
        "{:?}",
        report.diagnostics
    );
    assert!(dispatch_of(&report).open_world);
    assert!(
        dispatch_of(&report).candidates.is_empty(),
        "the truncated listing never reached a class of the range, so it published none"
    );
    let coverage = &report.coverage.runtime_resolution;
    assert_eq!(coverage.state, CoverageState::Partial);
    assert_eq!(coverage.scanned, vec![search_range(0, 1)], "{coverage:?}");
    assert_eq!(
        coverage.skipped,
        vec![search_range(1, 2)],
        "the truncated listing leaves the position it declared and never examined visible: \
         {coverage:?}"
    );
}

#[test]
fn a_result_items_stop_keeps_the_candidates_already_published() {
    let world = world_snapshot();
    let environment = single_loader(&world);
    let content = std::slice::from_ref(&world);
    let request = dispatch_request(
        environment,
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        PhysicalScope::SnapshotAll,
    );
    let complete = resolve(content, &request);
    let total = result_items(&complete);

    // One published item less than the complete run: the range listing and the first candidates
    // are paid for, the last candidate is not.
    let mut budget = Budget::new(Limits {
        result_items: total - 1,
        ..limits()
    });
    let report = resolve_under(content, &request, &mut budget);

    let published = candidate_owners(&report);
    assert!(
        !published.is_empty() && published.len() < candidate_owners(&complete).len(),
        "the refused charge keeps the candidates already published: {published:?}"
    );
    assert_eq!(
        published,
        candidate_owners(&complete)[..published.len()].to_vec()
    );
    assert!(dispatch_of(&report).open_world);
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert!(
        diagnostic_codes(&report).contains(&"budget_exceeded_result_items".to_string()),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn a_range_position_that_resolves_outside_the_range_is_undecided() {
    let world = world_snapshot();
    let shadow = shadow_snapshot();
    let content = vec![shadow.clone(), world.clone()];
    let target = method(b"i/I", b"m", b"()V");

    // The shadowing position comes first: `p/B` of the *range* is not the definition this
    // environment loads, so that position of the range stays undecided instead of being
    // published as a candidate or silently dropped as excluded.
    let app = domain(
        &loader("app"),
        None,
        vec![snapshot_root(&shadow), snapshot_root(&world)],
    );
    let shadowing = environment(&world, app.clone(), vec![app]);
    let report = resolve(
        &content,
        &dispatch_request(
            shadowing,
            &loader("app"),
            target.clone(),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );
    assert_eq!(
        candidate_owners(&report),
        vec![b"p/A".to_vec(), b"p/D".to_vec()],
        "the name `p/B` resolves to a definition outside the requested range"
    );
    assert!(dispatch_of(&report).open_world);
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial,
        "an undecided position of the range is unsearched range"
    );

    // The same range with the world first: now the range's own `p/B` is the definition the
    // environment loads, so the candidate set is complete.
    let app = domain(
        &loader("app"),
        None,
        vec![snapshot_root(&world), snapshot_root(&shadow)],
    );
    let world_first = environment(&world, app.clone(), vec![app]);
    let report = resolve(
        &content,
        &dispatch_request(
            world_first,
            &loader("app"),
            target,
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );
    assert_eq!(
        candidate_owners(&report),
        vec![b"p/A".to_vec(), b"p/B".to_vec(), b"p/D".to_vec()]
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema
    );
}

#[test]
fn an_artifact_tree_range_covers_the_containers_its_positions_reach() {
    // The nested JAR holds an implementation of the same declaration, and the loader declares
    // both containers as its own positions.
    let inner = zip_of(&[(
        entry(b"p/B"),
        Class::new(b"p/B")
            .implements(b"i/I")
            .method(b"m", b"()V", PUBLIC)
            .build(),
    )]);
    let outer = open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"i/I"),
            Class::interface(b"i/I")
                .method(b"m", b"()V", PUBLIC | ABSTRACT)
                .build(),
        ),
        (
            entry(b"p/A"),
            Class::new(b"p/A")
                .implements(b"i/I")
                .method(b"m", b"()V", PUBLIC)
                .build(),
        ),
        (b"lib/inner.jar".to_vec(), inner),
    ]));
    let mut budget = Budget::new(limits());
    let tree = Engine::new()
        .enumerate_artifact_tree(&outer, &mut budget)
        .expect("the fixture tree enumerates");
    let root_container = tree
        .containers
        .iter()
        .find(|container| container.origin.steps.is_empty())
        .expect("the root container");
    let nested = tree
        .containers
        .iter()
        .find(|container| !container.origin.steps.is_empty())
        .expect("the nested container is a container of its own");
    let app = domain(
        &loader("app"),
        None,
        vec![
            LoadRoot::Container {
                origin: root_container.origin.clone(),
                prefix: ArchiveNameBytes(Vec::new()),
            },
            LoadRoot::Container {
                origin: nested.origin.clone(),
                prefix: ArchiveNameBytes(Vec::new()),
            },
        ],
    );
    let environment = environment(&outer, app.clone(), vec![app]);
    let content = std::slice::from_ref(&outer);
    let target = method(b"i/I", b"m", b"()V");

    // `SnapshotAll` is the snapshot's root container: the nested implementation is not part of
    // that flat range.
    let flat = resolve(
        content,
        &dispatch_request(
            environment.clone(),
            &loader("app"),
            target.clone(),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );
    assert_eq!(candidate_owners(&flat), vec![b"p/A".to_vec()]);

    // `ArtifactTree` is the container tree, so the nested implementation is a class of the range
    // and becomes a candidate as well.
    let tree_report = resolve(
        content,
        &dispatch_request(
            environment,
            &loader("app"),
            target,
            ReferenceUse::InvokeInterface,
            PhysicalScope::ArtifactTree {
                root_container: root_container.origin.current_container().clone(),
            },
        ),
    );
    assert_eq!(
        candidate_owners(&tree_report),
        vec![b"p/A".to_vec(), b"p/B".to_vec()]
    );
    assert_eq!(
        candidate_evidence(&tree_report),
        vec![None, Some(OpenWorldEvidence::OrderedRoot { index: 1 })],
        "`p/A` is decided at the first position of the two-position order (declared root index \
         0), while `p/B` is only found at the second one (declared root index 1)"
    );
}

#[test]
fn a_standalone_class_root_can_be_the_whole_range() {
    // A standalone CLASS root has no path-derived name, so the range reads its header once to
    // learn the name it declares for itself and then demands that name through the closure. The
    // class is at declared root index 1 of the loader, so the candidate says so.
    let world = world_snapshot();
    let standalone = open(
        Class::new(b"p/S")
            .implements(b"i/I")
            .method(b"m", b"()V", PUBLIC)
            .build(),
    );
    assert_eq!(standalone.kind(), ArtifactKind::StandaloneClass);
    let app = domain(
        &loader("app"),
        None,
        vec![
            snapshot_root(&world),
            LoadRoot::StandaloneClass {
                snapshot: standalone.id().clone(),
            },
        ],
    );
    let environment = environment(&standalone, app.clone(), vec![app]);
    let content = vec![world, standalone];
    let report = resolve(
        &content,
        &dispatch_request(
            environment,
            &loader("app"),
            method(b"i/I", b"m", b"()V"),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );

    assert_eq!(candidate_owners(&report), vec![b"p/S".to_vec()]);
    assert_eq!(
        candidate_evidence(&report),
        vec![Some(OpenWorldEvidence::OrderedRoot { index: 1 })]
    );
    assert!(dispatch_of(&report).open_world);
    assert!(class_headers(&report) > 0);
    assert_eq!(code_bytes(&report), 0);
}

#[test]
fn a_tree_range_over_a_class_snapshot_is_a_failed_range_not_an_empty_one() {
    // An artifact tree needs a ZIP snapshot. The declaration still resolves (the world holds
    // it), and the range itself is reported as failed instead of as an empty tree: the caller
    // sees the artifact layer's own code and no candidate.
    let world = world_snapshot();
    let standalone = open(
        Class::new(b"p/S")
            .implements(b"i/I")
            .method(b"m", b"()V", PUBLIC)
            .build(),
    );
    let app = domain(
        &loader("app"),
        None,
        vec![
            snapshot_root(&world),
            LoadRoot::StandaloneClass {
                snapshot: standalone.id().clone(),
            },
        ],
    );
    let environment = environment(&standalone, app.clone(), vec![app]);
    let report = resolve(
        &[world, standalone],
        &dispatch_request(
            environment,
            &loader("app"),
            method(b"i/I", b"m", b"()V"),
            ReferenceUse::InvokeInterface,
            PhysicalScope::ArtifactTree {
                root_container: ContainerId("root".to_string()),
            },
        ),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert!(matches!(
        report.execution,
        ExecutionReport::Failed {
            reason: TerminationReason::Error { ref code },
            ..
        } if code == "not_zip"
    ));
    let dispatch = dispatch_of(&report);
    assert!(dispatch.candidates.is_empty());
    assert!(
        dispatch.open_world,
        "a range that could not be enumerated is open"
    );
    assert!(diagnostic_codes(&report).contains(&"not_zip".to_string()));
}

// ---------------------------------------------------------------------------
// The plane never runs for a request that has nothing to dispatch on
// ---------------------------------------------------------------------------

#[test]
fn a_request_without_a_range_publishes_no_dispatch_at_all() {
    let world = world_snapshot();
    let environment = single_loader(&world);
    let request = ResolutionRequest {
        environment,
        target: method(b"i/I", b"m", b"()V"),
        use_kind: ReferenceUse::InvokeInterface,
        caller: CallerContext {
            loader: loader("app"),
            enclosing: None,
        },
        dispatch: None,
    };
    let report = resolve(std::slice::from_ref(&world), &request);

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert!(report.dispatch.is_none());
    let codes = diagnostic_codes(&report);
    assert!(
        !codes
            .iter()
            .any(|code| code.starts_with("resolution_dispatch")),
        "a request that asked for no range produces no dispatch diagnostics: {codes:?}"
    );
    assert!(
        codes.iter().all(|code| code != "dispatch_not_implemented"),
        "the capability code of the unimplemented slice is gone: {codes:?}"
    );
    assert_eq!(
        class_headers(&report),
        1,
        "the member path reads the declaration's owner and nothing else"
    );
    assert_eq!(report.reads.len(), 1);
    assert_eq!(
        report.reads[0].reason,
        ReadReason::MemberOwner,
        "the declaration's owner is the one read of a member request with no range"
    );
}

#[test]
fn a_pre_cancelled_request_never_starts_the_range() {
    let world = world_snapshot();
    let environment = single_loader(&world);
    let request = dispatch_request(
        environment,
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        PhysicalScope::SnapshotAll,
    );
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let report = resolve_under(std::slice::from_ref(&world), &request, &mut budget);

    assert!(matches!(
        report.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert_eq!(report.state, None);
    assert!(
        report.dispatch.is_none(),
        "a cancelled request claims no range"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["cancelled", "resolution_dispatch_no_declaration"]
    );
    assert_eq!(class_headers(&report), 0);
}

#[test]
fn a_range_over_a_foreign_tree_root_is_refused_as_an_input_mismatch() {
    let world = world_snapshot();
    let environment = single_loader(&world);
    let request = dispatch_request(
        environment,
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        PhysicalScope::ArtifactTree {
            root_container: ContainerId("somewhere-else".to_string()),
        },
    );
    let error = Engine::new()
        .resolve_symbol(
            std::slice::from_ref(&world),
            &request,
            &mut Budget::new(limits()),
        )
        .expect_err("a tree root that cannot describe this snapshot is refused");
    assert!(
        error.to_string().contains("snapshot root container"),
        "the refusal names the root: {error}"
    );
}

// ---------------------------------------------------------------------------
// The names the range lists and the positions they resolve to
// ---------------------------------------------------------------------------

#[test]
fn a_range_position_no_position_provides_is_undecided() {
    // The tree range lists `p/B`, which lives in the nested JAR — but the loader declares only
    // the root container, so no position of the order provides that name. The position is
    // undecided, not excluded: the plane keeps the candidates of the positions it did decide
    // (`p/A`), states the unknown part of the range in `open_world` and reports a partial
    // coverage, without claiming any open-world fact for the candidate itself.
    let world = open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"i/I"),
            Class::interface(b"i/I")
                .method(b"m", b"()V", PUBLIC | ABSTRACT)
                .build(),
        ),
        (
            entry(b"p/A"),
            Class::new(b"p/A")
                .implements(b"i/I")
                .method(b"m", b"()V", PUBLIC)
                .build(),
        ),
        (
            b"lib/inner.jar".to_vec(),
            zip_of(&[(
                entry(b"p/B"),
                Class::new(b"p/B")
                    .implements(b"i/I")
                    .method(b"m", b"()V", PUBLIC)
                    .build(),
            )]),
        ),
    ]));
    let (mut roots, scope) = tree_range(&world);
    assert_eq!(roots.len(), 2);
    roots.truncate(1);
    let app = domain(&loader("app"), None, roots);
    let request = dispatch_request(
        environment(&world, app.clone(), vec![app]),
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        scope,
    );
    let report = resolve(std::slice::from_ref(&world), &request);

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        candidate_owners(&report),
        vec![b"p/A".to_vec()],
        "the decided prefix of the range, and not the class no position provides"
    );
    assert_eq!(candidate_evidence(&report), vec![None]);
    assert!(
        dispatch_of(&report).open_world,
        "an undecided position of the range is an unknown part of it"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert!(
        matches!(report.execution, ExecutionReport::Complete { .. }),
        "an undecided position is no stop: the plane enumerated what it could list"
    );
}

#[test]
fn an_ambiguous_range_position_is_undecided_not_excluded() {
    // The range's own container holds `p/B.class` twice: 2.1 cannot tell the two definitions
    // apart, so that position is ambiguous — the class is neither a candidate nor excluded, the
    // range states it could not be read, and the duplicate raw name is reported where it was
    // found.
    let world = open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"i/I"),
            Class::interface(b"i/I")
                .method(b"m", b"()V", PUBLIC | ABSTRACT)
                .build(),
        ),
        (
            entry(b"p/A"),
            Class::new(b"p/A")
                .implements(b"i/I")
                .method(b"m", b"()V", PUBLIC)
                .build(),
        ),
        (
            entry(b"p/B"),
            Class::new(b"p/B")
                .implements(b"i/I")
                .method(b"m", b"()V", PUBLIC)
                .build(),
        ),
        // The same raw name again, as a different class file.
        (
            entry(b"p/B"),
            Class::new(b"p/B")
                .implements(b"i/I")
                .method(b"m", b"()V", PUBLIC)
                .field(b"f", b"I", PUBLIC)
                .build(),
        ),
    ]));
    let environment = single_loader(&world);
    let request = dispatch_request(
        environment,
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        PhysicalScope::SnapshotAll,
    );
    let report = resolve(std::slice::from_ref(&world), &request);

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(candidate_owners(&report), vec![b"p/A".to_vec()]);
    assert!(dispatch_of(&report).open_world);
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    let codes = diagnostic_codes(&report);
    assert!(
        codes.contains(&"duplicate_raw_name".to_string()),
        "the listing names the duplicate it had to read: {codes:?}"
    );
    assert!(
        !codes
            .iter()
            .any(|code| code.starts_with("resolution_hierarchy")),
        "an ambiguous position is not a hierarchy gap: {codes:?}"
    );
}

#[test]
fn the_range_enumerates_each_name_once() {
    // An artifact-tree range lists `p/B` from the root container *and* from the nested JAR. The
    // name is one class of the range: the environment's order decides which definition it
    // resolves to, and enumerating it twice would publish the same candidate twice and charge
    // one more `ResultItems` for it.
    let world = tree_with_a_shared_name();
    let (roots, scope) = tree_range(&world);
    let app = domain(&loader("app"), None, roots);
    let request = dispatch_request(
        environment(&world, app.clone(), vec![app]),
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        scope,
    );
    let content = std::slice::from_ref(&world);
    let report = resolve(content, &request);

    assert_eq!(
        candidate_owners(&report),
        vec![b"p/B".to_vec()],
        "one name of the range is one candidate"
    );
    let definitions = candidate_definitions(&report);
    assert_eq!(
        definitions.len(),
        1,
        "the same physical definition is never published twice: {definitions:?}"
    );
    assert_eq!(
        definitions[0]
            .location
            .entry()
            .expect("the root container's entry")
            .origin
            .steps
            .len(),
        0,
        "the first occurrence of the name in range order is the one the order resolves to"
    );
    // The bill of this whole request, pinned: the name is located at the two declared positions
    // the range walks — the tree's root container and its nested JAR — and each location charges
    // the records of the container it really read, plus the declaration's own warning and the one
    // published candidate. It was 34 while a location was a whole-tree listing that also charged
    // two containers and a nested-archive candidate per listing; those charges are gone because a
    // lookup no longer produces a listing nobody asked for, and the records the directories really
    // contain are charged on both paths. 22 is the *measured* sum of the directed access on this
    // fixture, a recorded pin rather than a number derived from this sentence. The second half of
    // the pin is the boundary: one unit less and the run stops at that very candidate, so a second
    // publication of the same name could not have been paid for by this number.
    let bill = result_items(&report);
    assert_eq!(bill, 22, "{:?}", report.execution);
    let mut budget = Budget::new(Limits {
        result_items: bill - 1,
        ..limits()
    });
    let short = resolve_under(content, &request, &mut budget);
    assert!(matches!(
        short.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert!(dispatch_of(&short).candidates.is_empty());
}

#[test]
fn a_declaration_of_another_kind_is_not_a_candidate() {
    // The candidate rule compares the member kind as well as the raw name and descriptor.
    // `p/FieldM` declares `m` with the declaration's own descriptor bytes — as a *field* — and
    // is no override of the method declaration, while `p/MethodM` declares the same name and
    // descriptor as a method and is one.
    let world = open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"i/I"),
            Class::interface(b"i/I")
                .method(b"m", b"()V", PUBLIC | ABSTRACT)
                .build(),
        ),
        (
            entry(b"p/FieldM"),
            Class::new(b"p/FieldM")
                .implements(b"i/I")
                .field(b"m", b"()V", PUBLIC)
                .build(),
        ),
        (
            entry(b"p/MethodM"),
            Class::new(b"p/MethodM")
                .implements(b"i/I")
                .method(b"m", b"()V", PUBLIC)
                .build(),
        ),
    ]));
    let environment = single_loader(&world);
    let report = resolve(
        std::slice::from_ref(&world),
        &dispatch_request(
            environment,
            &loader("app"),
            method(b"i/I", b"m", b"()V"),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(candidate_owners(&report), vec![b"p/MethodM".to_vec()]);
}

#[test]
fn the_candidate_rule_is_structural_and_screens_no_member_flag() {
    // design.md 2.5 验收「结构性候选规则」: the plane publishes the class's own declaration
    // whatever its flags say. `private`, `static` and `abstract` are 2.3's rules (and the access
    // rules are the caller's business), so screening any of them here would drop a candidate the
    // report is supposed to hand back for judgement — and a plane that screened them would make
    // this whole range look like a declaration nothing overrides.
    let world = open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"i/I"),
            Class::interface(b"i/I")
                .method(b"m", b"()V", PUBLIC | ABSTRACT)
                .build(),
        ),
        (
            entry(b"p/StaticM"),
            Class::new(b"p/StaticM")
                .implements(b"i/I")
                .method(b"m", b"()V", PUBLIC | STATIC)
                .build(),
        ),
        (
            entry(b"p/PrivateM"),
            Class::new(b"p/PrivateM")
                .implements(b"i/I")
                .method(b"m", b"()V", PRIVATE)
                .build(),
        ),
        (
            entry(b"p/AbstractM"),
            Class::abstract_class(b"p/AbstractM")
                .implements(b"i/I")
                .method(b"m", b"()V", PUBLIC | ABSTRACT)
                .build(),
        ),
        (
            entry(b"p/Base"),
            Class::new(b"p/Base")
                .method(b"<init>", b"()V", PUBLIC)
                .build(),
        ),
        (
            entry(b"p/Sub"),
            Class::new(b"p/Sub")
                .extends(b"p/Base")
                .method(b"<init>", b"()V", PUBLIC)
                .build(),
        ),
    ]));
    let content = std::slice::from_ref(&world);
    let report = resolve(
        content,
        &dispatch_request(
            single_loader(&world),
            &loader("app"),
            method(b"i/I", b"m", b"()V"),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        candidate_owners(&report),
        vec![
            b"p/StaticM".to_vec(),
            b"p/PrivateM".to_vec(),
            b"p/AbstractM".to_vec()
        ],
        "a static, a private and an abstract declaration are candidates alike: {:?}",
        report.diagnostics
    );
    assert_eq!(
        candidate_evidence(&report),
        vec![None, None, None],
        "each of them is decided at the first declared root of a complete range"
    );
    assert!(
        !dispatch_of(&report).open_world,
        "nothing of this range is unknown, so no candidate stands under an open-world fact"
    );

    // The name can be a special one: `<init>` is published by the same structural rule, which is
    // what a plane applying the 2.3 call-kind rules to the *range* would have dropped.
    let init = resolve(
        content,
        &dispatch_request(
            single_loader(&world),
            &loader("app"),
            method(b"p/Base", b"<init>", b"()V"),
            ReferenceUse::InvokeSpecial,
            PhysicalScope::SnapshotAll,
        ),
    );
    assert_eq!(init.state, Some(ResolutionState::Resolved));
    assert_eq!(
        candidate_owners(&init),
        vec![b"p/Sub".to_vec()],
        "`p/Sub`'s own `<init>` is a candidate of the declaration it overrides: {:?}",
        init.diagnostics
    );
}

// ---------------------------------------------------------------------------
// The closure's own diagnostics
// ---------------------------------------------------------------------------

#[test]
fn a_cyclic_hierarchy_in_the_range_leaves_the_plane_open() {
    // The walk refuses a supertype edge whose name already repeats on the path that reached it:
    // `p/CycB extends p/CycA extends p/CycB` is illegal, so that branch is not expanded. Both
    // classes of the range whose closure is cyclic are candidates anyway, and their own evidence
    // states no fact — the cycle is a gap of the walk, so it moves `open_world` and the coverage
    // plane instead, and names itself in a diagnostic that says which loader and which path it
    // refused.
    let cyclic = open(zip_of(&cycle_pair(true)));
    let request = dispatch_request(
        single_loader(&cyclic),
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        PhysicalScope::SnapshotAll,
    );
    let report = resolve(std::slice::from_ref(&cyclic), &request);

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(candidate_owners(&report), vec![b"p/CycA".to_vec()]);
    assert_eq!(
        candidate_evidence(&report),
        vec![None],
        "the candidate itself states no open-world fact"
    );
    assert!(
        dispatch_of(&report).open_world,
        "an unread branch is an unknown part of the range"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    let cycles = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "resolution_hierarchy_cycle")
        .collect::<Vec<_>>();
    assert_eq!(
        cycles.len(),
        2,
        "one refused edge per walked class: {:?}",
        diagnostic_codes(&report)
    );
    for cycle in cycles {
        assert_eq!(cycle.severity, DiagnosticSeverity::Warning);
        for expected in ["loader `app`", "p/CycA", "p/CycB"] {
            assert!(
                cycle.message.contains(expected),
                "the diagnostic locates the cycle ({expected}): {}",
                cycle.message
            );
        }
    }

    // Control: the same range with `p/CycB extends java/lang/Object`. The entries, the classes
    // and the candidates are the same, so the closure is the only difference.
    let plain = open(zip_of(&cycle_pair(false)));
    let plain_report = resolve(
        std::slice::from_ref(&plain),
        &dispatch_request(
            single_loader(&plain),
            &loader("app"),
            method(b"i/I", b"m", b"()V"),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );
    assert_eq!(candidate_owners(&plain_report), candidate_owners(&report));
    assert!(!dispatch_of(&plain_report).open_world);
    assert_eq!(
        plain_report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(
        !diagnostic_codes(&plain_report).contains(&"resolution_hierarchy_cycle".to_string()),
        "{:?}",
        plain_report.diagnostics
    );
}

#[test]
fn a_closure_diagnostic_is_charged_like_every_other_resolution_result() {
    // The cycle diagnostics really are report entries, and every entry of a performed request
    // costs one `ResultItems` before it is published. The two runs read the same four headers of
    // the same classes and list the same entries of the same fixture — the cycle is the only
    // difference — so the bills differ by exactly the number of diagnostics the cyclic run adds.
    // A report that published them for free would show the same bill for both.
    let cyclic = open(zip_of(&cycle_pair(true)));
    let cyclic_report = resolve(
        std::slice::from_ref(&cyclic),
        &dispatch_request(
            single_loader(&cyclic),
            &loader("app"),
            method(b"i/I", b"m", b"()V"),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );
    let plain = open(zip_of(&cycle_pair(false)));
    let plain_report = resolve(
        std::slice::from_ref(&plain),
        &dispatch_request(
            single_loader(&plain),
            &loader("app"),
            method(b"i/I", b"m", b"()V"),
            ReferenceUse::InvokeInterface,
            PhysicalScope::SnapshotAll,
        ),
    );

    // The fixture parity the delta rests on: the same work, the same candidates.
    assert_eq!(class_headers(&cyclic_report), class_headers(&plain_report));
    assert_eq!(cyclic_report.reads.len(), plain_report.reads.len());
    assert_eq!(
        archive_entries(&cyclic_report),
        archive_entries(&plain_report)
    );
    assert_eq!(
        candidate_owners(&cyclic_report),
        candidate_owners(&plain_report)
    );

    let added = cyclic_report.diagnostics.len() - plain_report.diagnostics.len();
    assert_eq!(
        added,
        2,
        "the cyclic run adds one diagnostic per refused edge: {:?} vs {:?}",
        diagnostic_codes(&cyclic_report),
        diagnostic_codes(&plain_report)
    );
    assert_eq!(
        result_items(&cyclic_report) - result_items(&plain_report),
        u64::try_from(added).expect("a small diagnostic count"),
        "each diagnostic that enters this report costs one `ResultItems`"
    );
}

#[test]
fn a_refused_closure_diagnostic_keeps_the_entries_before_it() {
    // The closure's branch warnings are entries like any other, so one of them can be the charge
    // a budget refuses. After the declaration's own warning and the candidate it published, this
    // range leaves two cycle diagnostics — and one unit short of the bill, the *last* of them is
    // the refused charge: everything before it stays in the report, the stop names its dimension,
    // and nothing the plane already published is undone.
    let cyclic = open(zip_of(&cycle_pair(true)));
    let request = dispatch_request(
        single_loader(&cyclic),
        &loader("app"),
        method(b"i/I", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        PhysicalScope::SnapshotAll,
    );
    let content = std::slice::from_ref(&cyclic);
    let complete = resolve(content, &request);
    assert_eq!(
        diagnostic_codes(&complete),
        vec![
            "resolution_method_is_abstract",
            "resolution_hierarchy_cycle",
            "resolution_hierarchy_cycle"
        ],
        "{:?}",
        complete.diagnostics
    );

    let mut budget = Budget::new(Limits {
        result_items: result_items(&complete) - 1,
        ..limits()
    });
    let report = resolve_under(content, &request, &mut budget);

    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert_eq!(
        diagnostic_codes(&report),
        vec![
            "resolution_method_is_abstract",
            "resolution_hierarchy_cycle",
            "budget_exceeded_result_items"
        ],
        "the second cycle diagnostic is the refused charge; the entries before it are kept"
    );
    assert_eq!(
        report.state,
        Some(ResolutionState::Resolved),
        "the refused entry does not rewrite the declaration's own decision"
    );
    assert_eq!(
        candidate_owners(&report),
        vec![b"p/CycA".to_vec()],
        "the candidate the range published before the refused charge is kept"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
}

#[test]
fn a_rule_diagnostic_costs_one_result_item_before_the_range_runs() {
    // 2.3's rule diagnostics are resolution results like the closure's own branch warnings: each
    // one costs one `ResultItems` before it enters the report. The fixture's declaration is an
    // abstract interface method, so the member resolution publishes exactly that warning — and
    // in a request with no range it is the *last* entry charged, which is what makes the charge
    // observable: one unit short, the warning is the entry the budget refuses.
    let world = open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"i/I"),
            Class::interface(b"i/I")
                .method(b"m", b"()V", PUBLIC | ABSTRACT)
                .build(),
        ),
    ]));
    let environment = single_loader(&world);
    let caller = CallerContext {
        loader: loader("app"),
        enclosing: None,
    };
    let without_range = ResolutionRequest {
        environment: environment.clone(),
        target: method(b"i/I", b"m", b"()V"),
        use_kind: ReferenceUse::InvokeInterface,
        caller: caller.clone(),
        dispatch: None,
    };
    let content = std::slice::from_ref(&world);
    let complete = resolve(content, &without_range);

    assert_eq!(
        diagnostic_codes(&complete),
        vec!["resolution_method_is_abstract"],
        "the declaration resolved with its own warning and nothing else"
    );
    assert!(matches!(
        complete.execution,
        ExecutionReport::Complete { .. }
    ));

    // The requested range costs more entries than this, so a budget of `bill - 1` is exhausted
    // at the warning itself: the charge is refused, the stop explains why, and the range — which
    // this request can no longer pay for — is not enumerated at all.
    let with_range = ResolutionRequest {
        environment,
        target: method(b"i/I", b"m", b"()V"),
        use_kind: ReferenceUse::InvokeInterface,
        caller,
        dispatch: Some(DispatchScope {
            scope: PhysicalScope::SnapshotAll,
            consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
        }),
    };
    let mut budget = Budget::new(Limits {
        result_items: result_items(&complete) - 1,
        ..limits()
    });
    let report = resolve_under(content, &with_range, &mut budget);

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        diagnostic_codes(&report),
        vec!["budget_exceeded_result_items"],
        "the refused entry is the warning, and the stop it leaves explains the end"
    );
    assert!(
        !matches!(report.execution, ExecutionReport::Complete { .. }),
        "the report names the refused charge: {:?}",
        report.execution
    );
    assert!(
        report.dispatch.is_none(),
        "a request whose publication stopped enumerates no range and claims no candidate"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
}

#[test]
fn every_rule_diagnostic_of_one_request_is_charged_and_published() {
    // One request can publish several 2.3 rule diagnostics, and each of them is an entry of the
    // report: charged one `ResultItems` before it is published, in the order the resolution
    // produced them. This declaration produces two — its supertype list holds a name no position
    // provides (`resolution_hierarchy_missing`) and the maximally-specific set it does hold is
    // abstract (`resolution_method_is_abstract`) — so a plane that charged only the first would
    // still look complete under a single-diagnostic fixture. The control below keeps the work
    // identical (same entries, same supertype list, same lookups) and silences only the
    // abstractness, so the two bills differ by exactly that one entry.
    let gapped = open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"i/I"),
            Class::interface(b"i/I")
                .implements(b"i/J")
                .implements(b"i/Absent")
                .build(),
        ),
        (
            entry(b"i/J"),
            Class::interface(b"i/J")
                .method(b"m", b"()V", PUBLIC | ABSTRACT)
                .build(),
        ),
    ]));
    let request = |world: &ArtifactSnapshot| ResolutionRequest {
        environment: single_loader(world),
        target: method(b"i/I", b"m", b"()V"),
        use_kind: ReferenceUse::InvokeInterface,
        caller: CallerContext {
            loader: loader("app"),
            enclosing: None,
        },
        dispatch: None,
    };
    let content = std::slice::from_ref(&gapped);
    let complete = resolve(content, &request(&gapped));
    assert_eq!(
        diagnostic_codes(&complete),
        vec![
            "resolution_hierarchy_missing",
            "resolution_method_is_abstract"
        ],
        "{:?}",
        complete.diagnostics
    );
    assert!(matches!(
        complete.execution,
        ExecutionReport::Complete { .. }
    ));
    let bill = result_items(&complete);

    // The control fixture: the same three entries, the same supertype list and therefore the
    // same lookups — only the abstractness of the one declaration the maximally-specific set
    // holds differs, and with it the warning. A report that charged only its first rule
    // diagnostic would show the same bill for both fixtures.
    let defaulted = open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"i/I"),
            Class::interface(b"i/I")
                .implements(b"i/J")
                .implements(b"i/Absent")
                .build(),
        ),
        (
            entry(b"i/J"),
            Class::interface(b"i/J")
                .method(b"m", b"()V", PUBLIC)
                .build(),
        ),
    ]));
    let one = resolve(std::slice::from_ref(&defaulted), &request(&defaulted));
    assert_eq!(
        diagnostic_codes(&one),
        vec!["resolution_hierarchy_missing"],
        "the same unread name, minus the abstract declaration: {:?}",
        one.diagnostics
    );
    assert_eq!(
        bill - result_items(&one),
        1,
        "each published rule diagnostic costs one `ResultItems`"
    );

    // One unit short of the bill, the *second* diagnostic is the entry the budget refuses: the
    // first one stays in the report, the stop names the dimension, and the declaration the
    // request resolved is still published.
    let mut budget = Budget::new(Limits {
        result_items: bill - 1,
        ..limits()
    });
    let short = resolve_under(content, &request(&gapped), &mut budget);
    assert!(matches!(
        short.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
    assert_eq!(
        diagnostic_codes(&short),
        vec![
            "resolution_hierarchy_missing",
            "budget_exceeded_result_items"
        ],
        "the first diagnostic is published, the second one is the refused charge"
    );
    assert_eq!(
        short.state,
        Some(ResolutionState::Resolved),
        "a refused entry does not rewrite the decision the resolution already reached"
    );
    assert_eq!(short.resolved, complete.resolved);
}

// ---------------------------------------------------------------------------
// 0.1: the subtype test is a node test, not an owner-name test
// ---------------------------------------------------------------------------

/// The physical definition of one class name as a loader that only roots that snapshot selects it.
///
/// The comparison a dispatch candidate needs is between *definitions*, so a test that wants to
/// name one has to read it from a world that holds exactly it: the same name under two loaders is
/// two classes, and only the physical pair tells them apart.
fn definition_in(snapshot: &ArtifactSnapshot, name: &[u8]) -> PhysicalDefinitionId {
    let environment = single_loader(snapshot);
    let request = ResolutionRequest {
        environment,
        target: SymbolRef::Class {
            owner: JvmBytes(name.to_vec()),
        },
        use_kind: ReferenceUse::ClassReference,
        caller: CallerContext {
            loader: loader("app"),
            enclosing: None,
        },
        dispatch: None,
    };
    resolve(std::slice::from_ref(snapshot), &request)
        .resolved
        .unwrap_or_else(|| panic!("the fixture holds `{}`", String::from_utf8_lossy(name)))
        .definition
}

/// The subtype test compares ancestor **nodes**, not the owner string of a supertype layer.
///
/// `child` is ChildFirst with its own root and `parent` as its parent loader. Both loaders define
/// `p/Base` (an interface here, so the whole chain is declared the way JVMS declares it) and
/// `p/Hook`; the parent's `p/Hook extends p/Base`, and the request's target `p/Owner.f` resolves
/// to the field `p/Base.f` of the **parent**. The range is the child's snapshot, and three of its
/// classes pin the plane:
///
/// * `p/Mid implements p/Base` reaches the *child's* `p/Base` and stops there. Its supertype
///   layer's owner string is `p/Base`, exactly like the declaration's, and its node is not: a
///   string comparison would publish `p/Mid` as an override of a declaration it does not inherit
///   from, while the node test keeps it out;
/// * `p/Y extends p/Alpha implements p/Iface` really inherits from the declaration: the `p/Alpha`
///   branch reaches the child's `p/Hook`, and the `p/Iface` branch — an interface only the parent
///   provides — reaches the parent's `p/Hook` and from there the parent's `p/Base`. It is
///   published, which is the positive control;
/// * `p/Y` is also where the walk's own identity has to be node-keyed. The same *name* `p/Hook`
///   occurs twice on its supertype closure — the child's definition on the `p/Alpha` branch, the
///   parent's on the `p/Iface` branch — and the declaration sits behind the *second* one. A walk
///   that remembered names instead of nodes would refuse to expand it as a repeat and lose the
///   declaration `p/Y` exists to reach.
///
/// Both sides are decided evidence, not silence: the reads below name the parent's `p/Hook` and
/// `p/Base`, so the plane excluded `p/Mid` and published `p/Y` after really looking at both
/// loaders' definitions.
#[test]
fn the_subtype_test_compares_ancestor_nodes_not_owner_names() {
    let child_snapshot = open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (entry(b"p/Base"), Class::interface(b"p/Base").build()),
        (
            entry(b"p/Mid"),
            Class::new(b"p/Mid")
                .implements(b"p/Base")
                .field(b"f", b"I", PUBLIC)
                .build(),
        ),
        (entry(b"p/Hook"), Class::interface(b"p/Hook").build()),
        (
            entry(b"p/Alpha"),
            Class::new(b"p/Alpha").implements(b"p/Hook").build(),
        ),
        (
            entry(b"p/Y"),
            Class::new(b"p/Y")
                .extends(b"p/Alpha")
                .implements(b"p/Iface")
                .field(b"f", b"I", PUBLIC)
                .build(),
        ),
    ]));
    let parent_snapshot = open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"p/Base"),
            Class::interface(b"p/Base")
                .field(b"f", b"I", PUBLIC | STATIC | FINAL)
                .build(),
        ),
        (
            entry(b"p/Hook"),
            Class::interface(b"p/Hook").extends(b"p/Base").build(),
        ),
        (
            entry(b"p/Iface"),
            Class::interface(b"p/Iface").extends(b"p/Hook").build(),
        ),
        (
            entry(b"p/Owner"),
            Class::new(b"p/Owner").implements(b"p/Base").build(),
        ),
    ]));

    let parent_loader = loader("parent");
    let mut child = domain(
        &loader("child"),
        Some(parent_loader.clone()),
        vec![snapshot_root(&child_snapshot)],
    );
    child.delegation = DelegationPolicy::ChildFirst;
    let parent = domain(&parent_loader, None, vec![snapshot_root(&parent_snapshot)]);
    let environment = environment(&child_snapshot, child.clone(), vec![child, parent]);
    let content = vec![child_snapshot.clone(), parent_snapshot.clone()];
    let report = resolve(
        &content,
        &dispatch_request(
            environment,
            &loader("child"),
            field(b"p/Owner", b"f", b"I"),
            ReferenceUse::FieldRead,
            PhysicalScope::SnapshotAll,
        ),
    );

    let child_base = definition_in(&child_snapshot, b"p/Base");
    let parent_base = definition_in(&parent_snapshot, b"p/Base");
    let child_hook = definition_in(&child_snapshot, b"p/Hook");
    let parent_hook = definition_in(&parent_snapshot, b"p/Hook");
    let iface = definition_in(&parent_snapshot, b"p/Iface");
    let child_y = definition_in(&child_snapshot, b"p/Y");
    assert_ne!(
        child_base, parent_base,
        "the fixture really holds two `p/Base` definitions"
    );
    assert_ne!(child_hook, parent_hook, "and two `p/Hook` definitions");

    // The declaration is the parent's interface field: `p/Owner` is provided by the parent, so the
    // names its header holds are resolved from the parent's own order (JVMS 5.4.3.1).
    assert_eq!(report.state, Some(ResolutionState::Resolved), "{report:?}");
    let resolved = report.resolved.as_ref().expect("the declaration resolves");
    assert_eq!(
        resolved.member,
        SymbolRef::Field {
            owner: JvmBytes(b"p/Base".to_vec()),
            name: JvmBytes(b"f".to_vec()),
            descriptor: JvmBytes(b"I".to_vec()),
        },
        "both `p/Base`s are named `p/Base`, so the owner string is the same either way"
    );
    assert_eq!(resolved.definition, parent_base);
    assert_eq!(resolved.loader, parent_loader);

    assert_eq!(
        candidate_owners(&report),
        vec![b"p/Y".to_vec()],
        "`p/Y` inherits from the declaration's class; `p/Mid` reaches only the child's same-named \
         interface, and a name is not an inheritance edge"
    );
    assert_eq!(candidate_definitions(&report), vec![child_y]);
    assert_eq!(
        candidate_evidence(&report),
        vec![None],
        "every position of both chains is readable and inside the caller's own chain"
    );
    assert!(!dispatch_of(&report).open_world);

    // The walk really looked at both loaders' definitions of the repeated name, and past the
    // second one to the declaration: a walk that keyed its expanded set on names stops at the
    // child's `p/Hook` and never publishes `p/Y`.
    let read_definitions = report
        .reads
        .iter()
        .map(|read| read.definition.clone())
        .collect::<Vec<_>>();
    for (definition, what) in [
        (&child_base, "the child's `p/Base`, `p/Mid`'s ancestor"),
        (
            &child_hook,
            "the child's `p/Hook`, `p/Alpha`'s superinterface",
        ),
        (
            &iface,
            "the parent's `p/Iface`, which only that loader provides",
        ),
        (
            &parent_hook,
            "the parent's `p/Hook`, the same name under its second loader",
        ),
        (
            &parent_base,
            "the parent's `p/Base`, the declaration's class",
        ),
    ] {
        assert!(
            read_definitions.contains(definition),
            "{what} was read: {read_definitions:?}"
        );
    }
}
