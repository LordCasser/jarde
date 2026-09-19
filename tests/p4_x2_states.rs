//! P4 2.2 acceptance: the X2 declaration/dispatch query states.
//!
//! What this file proves through the public API is the separation the design's decision 3 asks
//! for — evidence and assumptions a reviewer can check, instead of one confidence number:
//!
//! 1. **declaration resolution, possible dispatch and open-world are three typed planes.**
//!    `ResolutionReport.state` answers what the declaration search decided, `report.dispatch`
//!    lists the known candidates of an explicit range, and each candidate carries the open-world
//!    fact it stands under. A candidate is never a runtime target: `DispatchReport` has exactly
//!    three fields and `DispatchCandidate` two, which this file pins by destructuring both
//!    exhaustively (a field naming one runtime target could not be added without editing these
//!    assertions), and `DispatchReport::open_world` says whether the range can be stated
//!    completely;
//! 2. **a missing dependency is not a negation** (A11). A member whose hierarchy needs a class
//!    this snapshot's order does not provide is `UnresolvedDependency` with the class named in
//!    `unresolved_dependencies` — never `Missing`, which says no readable position declares the
//!    member. The same distinction holds inside a dispatch range, where the unread class is
//!    published by name and the candidate that stands under it carries
//!    `OpenWorldEvidence::MissingDependency`;
//! 3. **a default conflict is a link error of the declaration plane** (`IncompatibleClassChange`
//!    with `resolution_default_conflict`), and the two cases that are *not* conflicts — a
//!    subinterface's default overriding its parent's, and a class overriding both inherits — stay
//!    ordinary resolutions, so the conflict is not over-reported;
//! 4. **A11's three sentences**: a symbolic reference keeps its raw owner while the resolution
//!    names the declaring class, a wider range extends that to the candidates of a range, and a
//!    missing dependency never becomes the negative answer;
//! 5. **the states are read through the existing query surface**: `Engine::resolve_symbol`, the
//!    P2 entry the design fixed for X2. No `QueryRelation` is added — the P1 X1 scan keeps its own
//!    relation set, and `may_dispatch_to` stays the unsupported *scan* relation it already is,
//!    because a structural scan cannot enumerate a dispatch range;
//! 6. **2.2 is a per-view plane, independent of the 2.1 matrix.** A resolution request names
//!    exactly one `RuntimeView` and the report identifies it, so a multi-profile answer is one
//!    request per view (the matrix selects those views) — these states are not a second view
//!    selector, and nothing here re-derives a profile's physical choice.
//!
//! Fixtures are built from one class-file writer and stored ZIPs; no framework.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const STORE: u16 = 0;

// The member and class access flags these fixtures use (JVMS 4.1/4.6).
const PUBLIC: u16 = 0x0001;
const INTERFACE: u16 = 0x0200;
const ABSTRACT: u16 = 0x0400;

/// The `major_version` every fixture shares: Java 8, the release default methods appear in.
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
        code_bytes: 1 << 22,
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
    /// Whether the declaration carries a one-instruction `Code` attribute: a Java 8 interface
    /// default is a non-abstract interface method with a body.
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

    /// A class with no superclass: the hierarchy root every complete fixture needs.
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

    fn extends(mut self, name: &[u8]) -> Self {
        self.super_class = Some(name.to_vec());
        self
    }

    fn implements(mut self, name: &[u8]) -> Self {
        self.interfaces.push(name.to_vec());
        self
    }

    /// An abstract declaration: a member without a body.
    fn method(mut self, name: &[u8], descriptor: &[u8], access_flags: u16) -> Self {
        self.methods.push(Member {
            name: name.to_vec(),
            descriptor: descriptor.to_vec(),
            access_flags,
            body: false,
        });
        self
    }

    /// A member with a real body: a class's own implementation, and a Java 8 interface `default`
    /// when the flags leave it non-abstract.
    fn method_with_body(mut self, name: &[u8], descriptor: &[u8], access_flags: u16) -> Self {
        self.methods.push(Member {
            name: name.to_vec(),
            descriptor: descriptor.to_vec(),
            access_flags,
            body: true,
        });
        self
    }

    /// One public `()V` interface `default`: non-abstract, with a body.
    fn default_method(self, name: &[u8]) -> Self {
        self.method_with_body(name, b"()V", PUBLIC)
    }

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

fn open(classes: Vec<Class>) -> ArtifactSnapshot {
    let entries = classes
        .iter()
        .map(|class| (entry(&class.name()), class.build()))
        .collect::<Vec<_>>();
    Engine::new()
        .open(
            ArtifactInput::bytes(zip_of(&entries)),
            &mut Budget::new(limits()),
        )
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

fn snapshot_root(snapshot: &ArtifactSnapshot) -> LoadRoot {
    LoadRoot::Snapshot {
        snapshot: snapshot.id().clone(),
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

/// The one-loader world every case starts from: the caller searches the snapshot's own root.
fn single_loader(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
    let caller = domain(&loader("app"), None, vec![snapshot_root(snapshot)]);
    environment(snapshot, caller.clone(), vec![caller])
}

fn method_target(owner: &[u8], name: &[u8], descriptor: &[u8]) -> SymbolRef {
    SymbolRef::Method {
        owner: JvmBytes(owner.to_vec()),
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
    }
}

/// A member request, with an optional dispatch range over the whole snapshot.
fn member_request(
    environment: ResolutionEnvironment,
    target: SymbolRef,
    use_kind: ReferenceUse,
    dispatch: bool,
) -> ResolutionRequest {
    ResolutionRequest {
        environment,
        target,
        use_kind,
        caller: CallerContext {
            loader: loader("app"),
            // No use site: these cases are about the resolution and the range, not the access
            // rules.
            enclosing: None,
        },
        dispatch: dispatch.then(|| DispatchScope {
            scope: PhysicalScope::SnapshotAll,
            consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
        }),
    }
}

fn resolve(content: &[ArtifactSnapshot], request: &ResolutionRequest) -> ResolutionReport {
    let mut budget = Budget::new(limits());
    Engine::new()
        .resolve_symbol(content, request, &mut budget)
        .expect("a legal request is answered, not raised")
}

fn diagnostic_codes(report: &ResolutionReport) -> Vec<&str> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

/// An interface with one abstract method, a base class declaring it, and two subclasses — one
/// symbolic (no declaration of its own) and one that implements it.
///
/// `java/lang/Object` is the only root, so every hierarchy here is read completely.
fn hierarchy_world() -> Vec<Class> {
    vec![
        Class::root(b"java/lang/Object"),
        Class::interface(b"i/I").method(b"foo", b"()V", PUBLIC | ABSTRACT),
        Class::new(b"p/Base")
            .implements(b"i/I")
            .method_with_body(b"foo", b"()V", PUBLIC),
        // The symbolic subclass: the reference names `p/Sub`, and the declaration is `p/Base`'s.
        Class::new(b"p/Sub").extends(b"p/Base"),
        // A second subclass that overrides the declaration: one more known candidate.
        Class::new(b"p/Sub2")
            .extends(b"p/Base")
            .method_with_body(b"foo", b"()V", PUBLIC),
    ]
}

/// A11, sentence one and two: the symbolic owner stays the raw reference, and the resolution names
/// the declaring class — inside a range, once per class that really overrides the declaration.
#[test]
fn a_symbolic_owner_resolves_to_the_declaration_and_a_range_extends_it_to_its_candidates() {
    let snapshot = open(hierarchy_world());
    let request = member_request(
        single_loader(&snapshot),
        method_target(b"p/Sub", b"foo", b"()V"),
        ReferenceUse::InvokeVirtual,
        true,
    );
    let report = resolve(std::slice::from_ref(&snapshot), &request);

    // The raw reference is preserved byte for byte: the report never rewrites an owner.
    assert_eq!(report.target, request.target);
    assert_eq!(
        report.state,
        Some(ResolutionState::Resolved),
        "`p/Sub` inherits `p/Base.foo`: {:?}",
        diagnostic_codes(&report)
    );
    let resolved = report
        .resolved
        .as_ref()
        .expect("a resolved report names one");
    assert_eq!(
        resolved.member,
        method_target(b"p/Base", b"foo", b"()V"),
        "the selected declaration is the declaring class's own, not the reference's owner"
    );
    assert!(
        report.unresolved_dependencies.is_empty(),
        "every branch of this hierarchy was read: {:?}",
        report.unresolved_dependencies
    );

    // The schema of a range is exactly these three fields and of a candidate exactly two: a field
    // naming one runtime target could not be added without failing to compile here, so "one
    // candidate" can never be read as "the target".
    let dispatch = report.dispatch.as_ref().expect("the range was requested");
    let DispatchReport {
        scope,
        candidates,
        open_world,
    } = dispatch.clone();
    assert_eq!(scope, PhysicalScope::SnapshotAll);
    let owners = candidates
        .iter()
        .map(|candidate| {
            let DispatchCandidate { member, evidence } = candidate;
            assert_eq!(
                evidence, &None,
                "a complete range still states its candidates without an open-world fact"
            );
            let SymbolRef::Method { owner, .. } = &member.member else {
                panic!("a method declaration publishes a method candidate");
            };
            owner.clone()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        owners,
        vec![JvmBytes(b"p/Sub2".to_vec())],
        "only the subclass that declares its own `foo` is a candidate: the declaring class is \
         never its own override"
    );
    assert!(
        !open_world,
        "this range is complete: every class was enumerated and every hierarchy read"
    );
}

/// A11, sentence three — the load-bearing one: a class the hierarchy needs and this snapshot does
/// not provide is not the statement that the member does not exist.
#[test]
fn a_missing_dependency_is_stated_by_name_and_never_as_the_negative_answer() {
    // `p/Orphan extends p/Absent`, and `p/Absent` is not in the snapshot. The owner itself was
    // read, so the resolution knows where the search stopped.
    let snapshot = open(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Orphan").extends(b"p/Absent"),
    ]);
    let request = member_request(
        single_loader(&snapshot),
        method_target(b"p/Orphan", b"foo", b"()V"),
        ReferenceUse::InvokeVirtual,
        false,
    );
    let report = resolve(std::slice::from_ref(&snapshot), &request);

    assert_eq!(
        report.state,
        Some(ResolutionState::UnresolvedDependency),
        "the search never entered `p/Absent`, so it cannot answer for it: {:?}",
        diagnostic_codes(&report)
    );
    assert_ne!(
        report.state,
        Some(ResolutionState::Missing),
        "`Missing` would claim this order proves no class declares `foo`"
    );
    assert!(report.resolved.is_none());
    assert_eq!(
        report.unresolved_dependencies,
        vec![UnresolvedDependency {
            name: JvmBytes(b"p/Absent".to_vec()),
            loader: loader("app"),
            reason: ReadReason::ParentChain,
            declared_by: Some(JvmBytes(b"p/Orphan".to_vec())),
            gap: DependencyGap::Missing,
        }],
        "the missing dependency is published by name, with the loader that searched it, the \
         `super_class` edge that needed it and the class that declares that edge"
    );

    // The control: the same fixture with the supertype present decides `Missing`, because then
    // the search really did read every branch that could declare the member.
    let complete = open(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Absent"),
        Class::new(b"p/Orphan").extends(b"p/Absent"),
    ]);
    let request = member_request(
        single_loader(&complete),
        method_target(b"p/Orphan", b"foo", b"()V"),
        ReferenceUse::InvokeVirtual,
        false,
    );
    let report = resolve(std::slice::from_ref(&complete), &request);
    assert_eq!(report.state, Some(ResolutionState::Missing));
    assert!(
        report.unresolved_dependencies.is_empty(),
        "nothing was left unread, so the negation is the whole answer"
    );
}

/// The open-world scenario of the requirement: a virtual method whose range holds an unread
/// supertype above a candidate answers known candidates plus the open-world state, and no unique
/// target.
#[test]
fn an_open_world_range_publishes_its_candidates_and_the_class_it_could_not_read() {
    // `p/Sub2` overrides the declaration and implements `i/Gone`, which the snapshot does not
    // hold: the candidate is known, its own hierarchy is not, and the runtime may well hold a
    // subclass this snapshot cannot see.
    let snapshot = open(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method_with_body(b"foo", b"()V", PUBLIC),
        Class::new(b"p/Sub2")
            .extends(b"p/Base")
            .implements(b"i/Gone")
            .method_with_body(b"foo", b"()V", PUBLIC),
    ]);
    let request = member_request(
        single_loader(&snapshot),
        method_target(b"p/Base", b"foo", b"()V"),
        ReferenceUse::InvokeVirtual,
        true,
    );
    let report = resolve(std::slice::from_ref(&snapshot), &request);

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    let dispatch = report.dispatch.as_ref().expect("the range was requested");
    let DispatchReport {
        scope: _,
        candidates,
        open_world,
    } = dispatch.clone();
    assert!(
        open_world,
        "a candidate stands under an open-world fact, so the range cannot be stated completely"
    );
    assert_eq!(
        candidates.len(),
        1,
        "this snapshot proves one override: {:?}",
        candidates
    );
    let DispatchCandidate { member, evidence } = &candidates[0];
    assert_eq!(
        member.member,
        method_target(b"p/Sub2", b"foo", b"()V"),
        "the candidate is its own declaration, not the declaration that was asked for"
    );
    assert_eq!(
        evidence,
        &Some(OpenWorldEvidence::MissingDependency),
        "the candidate says which fact of the range it stands under"
    );
    assert_eq!(
        report.unresolved_dependencies,
        vec![UnresolvedDependency {
            name: JvmBytes(b"i/Gone".to_vec()),
            loader: loader("app"),
            reason: ReadReason::HierarchyClosure,
            declared_by: Some(JvmBytes(b"p/Sub2".to_vec())),
            gap: DependencyGap::Missing,
        }],
        "and the range names the class it could not read, under the `interfaces` edge that \
         needed it"
    );
}

/// A loader outside the caller's delegation chain is the other open-world fact: another loader may
/// hold another definition of the same name, so the range is not complete either.
#[test]
fn a_loader_outside_the_chain_keeps_the_range_open_world() {
    let snapshot = open(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method_with_body(b"foo", b"()V", PUBLIC),
        Class::new(b"p/Sub2")
            .extends(b"p/Base")
            .method_with_body(b"foo", b"()V", PUBLIC),
    ]);
    // `platform` declares no root this request reads from, and the caller does not delegate to it:
    // it is a loader this answer cannot speak for.
    let caller = domain(&loader("app"), None, vec![snapshot_root(&snapshot)]);
    let environment = environment(
        &snapshot,
        caller.clone(),
        vec![caller, domain(&loader("platform"), None, Vec::new())],
    );
    let request = member_request(
        environment,
        method_target(b"p/Base", b"foo", b"()V"),
        ReferenceUse::InvokeVirtual,
        true,
    );
    let report = resolve(std::slice::from_ref(&snapshot), &request);

    let dispatch = report.dispatch.as_ref().expect("the range was requested");
    assert!(
        dispatch.open_world,
        "a declared loader outside the chain is one more place a subclass may live"
    );
    assert_eq!(dispatch.candidates.len(), 1);
    assert_eq!(
        dispatch.candidates[0].evidence,
        Some(OpenWorldEvidence::UnknownLoader)
    );
    assert!(
        report.unresolved_dependencies.is_empty(),
        "every name this request searched resolved: the open-world fact is the loader, not a gap"
    );
}

/// A range position the caller's order resolves *outside* the requested range is unsearched range:
/// the plane is a prefix and says so, and the class is not reported as absent.
#[test]
fn a_range_position_resolved_outside_the_range_is_unsearched_not_excluded() {
    let listed = open(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method_with_body(b"foo", b"()V", PUBLIC),
        Class::new(b"p/Sub2")
            .extends(b"p/Base")
            .method_with_body(b"foo", b"()V", PUBLIC),
    ]);
    let shadow = open(vec![
        Class::new(b"p/Sub2")
            .extends(b"p/Base")
            .method_with_body(b"foo", b"()V", PUBLIC),
    ]);
    // The shadowing root is searched first, so `p/Sub2` resolves to a definition outside the
    // requested range and the range's own position for it stays undecided.
    let caller = domain(
        &loader("app"),
        None,
        vec![snapshot_root(&shadow), snapshot_root(&listed)],
    );
    let environment = environment(&listed, caller.clone(), vec![caller]);
    let request = member_request(
        environment,
        method_target(b"p/Base", b"foo", b"()V"),
        ReferenceUse::InvokeVirtual,
        true,
    );
    let report = resolve(&[shadow, listed], &request);

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    let dispatch = report.dispatch.as_ref().expect("the range was requested");
    assert!(
        dispatch.candidates.is_empty(),
        "the override lives in the requested range, but this request could not reach it: {:?}",
        dispatch.candidates
    );
    assert!(
        dispatch.open_world,
        "a position this request could not decide keeps the range open"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial,
        "an undecided position is unsearched range, so the plane is a prefix"
    );
    assert!(
        report.unresolved_dependencies.is_empty(),
        "the name itself did resolve, so no dependency of this request is unread for it"
    );
}

/// A Java 8 default conflict is a link error decided at resolution, and the report states it as
/// the declaration plane's `IncompatibleClassChange` — never as a silent pick.
#[test]
fn two_unrelated_defaults_are_incompatible_class_change() {
    let snapshot = open(vec![
        Class::root(b"java/lang/Object"),
        Class::interface(b"i/Def").default_method(b"m"),
        Class::interface(b"i/Def2").default_method(b"m"),
        // Neither interface extends the other and the class overrides neither: JVMS 5.4.3.3
        // leaves two maximally-specific defaults, which is the link error the invocation raises
        // as `IncompatibleClassChangeError`.
        Class::new(b"p/Both")
            .implements(b"i/Def")
            .implements(b"i/Def2"),
    ]);
    let request = member_request(
        single_loader(&snapshot),
        method_target(b"p/Both", b"m", b"()V"),
        ReferenceUse::InvokeVirtual,
        false,
    );
    let report = resolve(std::slice::from_ref(&snapshot), &request);

    assert_eq!(
        report.state,
        Some(ResolutionState::IncompatibleClassChange),
        "two unrelated defaults cannot be told apart: {:?}",
        diagnostic_codes(&report)
    );
    assert!(
        report.resolved.is_none(),
        "no declaration is picked silently"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_default_conflict"]
    );
    assert!(report.unresolved_dependencies.is_empty());
    assert!(
        report.dispatch.is_none(),
        "no declaration resolved, so no range claims candidates"
    );
}

/// The two cases that are *not* a default conflict: a subinterface's default overrides its
/// parent's, and a class's own declaration wins over both inherits — neither is a link error.
#[test]
fn a_default_that_overrides_another_is_not_a_conflict() {
    let snapshot = open(vec![
        Class::root(b"java/lang/Object"),
        Class::interface(b"i/Parent").default_method(b"m"),
        // The more derived declaration overrides the less derived one (JVMS 5.4.3.3), so the
        // maximally-specific set holds exactly one declaration.
        Class::interface(b"i/Child")
            .implements(b"i/Parent")
            .default_method(b"m"),
        // One inherit, one override: both resolve, and neither is over-reported as a conflict.
        Class::new(b"p/Inherits").implements(b"i/Child"),
        Class::new(b"p/Overrides")
            .implements(b"i/Parent")
            .implements(b"i/Child")
            .method_with_body(b"m", b"()V", PUBLIC),
    ]);

    for (owner, expected) in [
        (b"p/Inherits".as_slice(), b"i/Child".as_slice()),
        (b"p/Overrides".as_slice(), b"p/Overrides".as_slice()),
    ] {
        let request = member_request(
            single_loader(&snapshot),
            method_target(owner, b"m", b"()V"),
            ReferenceUse::InvokeVirtual,
            false,
        );
        let report = resolve(std::slice::from_ref(&snapshot), &request);

        assert_eq!(
            report.state,
            Some(ResolutionState::Resolved),
            "{owner:?}: one maximally-specific default is not a conflict: {:?}",
            diagnostic_codes(&report)
        );
        let resolved = report
            .resolved
            .as_ref()
            .expect("a resolved report names one");
        assert_eq!(
            resolved.member,
            method_target(expected, b"m", b"()V"),
            "{owner:?}: the most derived declaration is the one that resolves"
        );
        assert!(
            diagnostic_codes(&report)
                .iter()
                .all(|code| *code != "resolution_default_conflict"),
            "{owner:?}: reporting the conflict here would be a false alarm"
        );
    }
}

/// 2.2 is a per-view plane: the request names one `RuntimeView` and the report identifies it, so a
/// multi-profile answer is one request per view — the 2.1 matrix selects the views, and nothing
/// here re-derives a profile's physical choice.
#[test]
fn the_states_of_one_request_belong_to_the_one_view_it_names() {
    let snapshot = open(hierarchy_world());
    for release in [8_u16, 17_u16] {
        let mut environment = single_loader(&snapshot);
        environment.runtime.profile.java_release = release;
        let request = member_request(
            environment,
            method_target(b"p/Sub", b"foo", b"()V"),
            ReferenceUse::InvokeVirtual,
            true,
        );
        let report = resolve(std::slice::from_ref(&snapshot), &request);

        assert_eq!(
            report.environment_identity.runtime, request.environment.runtime,
            "the answer names the view it was decided under"
        );
        assert_eq!(
            report.state,
            Some(ResolutionState::Resolved),
            "release {release}: this fixture holds no multi-release variant, so the view's own \
             selection is what the answer is decided from"
        );
        assert_eq!(
            report.unresolved_dependencies,
            Vec::new(),
            "release {release}: the dependency plane is the view's own"
        );
    }
}
