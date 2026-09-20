//! P2 2.3 acceptance: member resolution, the invocation-kind rules and the access rules.
//!
//! The slice under test turns a field or method reference into the declaration it resolves to.
//! What this file has to prove through the public API is:
//!
//! 1. the three JVMS search paths are structural, not a name recursion: a field is looked for in
//!    the class, then its superinterfaces (before the superclass), then its superclass; a method
//!    of a class is looked for in the class and its superclass chain before the superinterfaces;
//!    a method of an interface is looked for in the interface and its superinterfaces;
//! 2. the selected member is the **declaration**: its owner is the declaring class's own internal
//!    name, and `report.target` keeps the raw reference (which is what makes a
//!    signature-polymorphic call site show both descriptors);
//! 3. a missing member, an ambiguous owner, a duplicate declaration and an unreadable supertype
//!    are different answers and none of them fabricates a declaration;
//! 4. the invocation-kind rules reject a static/instance mismatch, a class/interface mismatch, a
//!    constructor outside `invoke_special` and `invoke_special` on an abstract method with
//!    `IncompatibleClassChange`, and the Java 8 default conflict joins them;
//! 5. the access rules are applied to a known caller and never claimed when the caller's class is
//!    unknown — `Resolved` plus `resolution_access_not_checked` says what was *not* checked;
//! 6. `reads` names the class the reference names and the class of the use site as `MemberOwner`
//!    reads and every step of a hierarchy as a `HierarchyClosure` read, once per binding;
//! 7. a member resolution reads no code byte: the evidence is `code_bytes == 0` next to the P1
//!    body path's non-zero charge on the same class (`method_bodies` has no charge point before
//!    3.x and proves nothing).
//!
//! Fixtures are built from one general class-file writer (declaration lists, access flags, an
//! optional one-instruction body) and stored ZIPs; no framework and no new dependency.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const STORE: u16 = 0;

/// `ACC_PUBLIC`, `ACC_PRIVATE`, `ACC_PROTECTED`, `ACC_STATIC`, `ACC_FINAL`, `ACC_NATIVE`, the
/// class/interface flags and `ACC_ABSTRACT` (JVMS 4.1/4.6).
const PUBLIC: u16 = 0x0001;
const PRIVATE: u16 = 0x0002;
const PROTECTED: u16 = 0x0004;
const STATIC: u16 = 0x0008;
const FINAL: u16 = 0x0010;
const NATIVE: u16 = 0x0100;
const INTERFACE: u16 = 0x0200;
const ABSTRACT: u16 = 0x0400;

/// A `major_version` every fixture shares: Java 8, the profile this change resolves under.
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
        // Funded for the one body-path control; the member searches under test never charge it.
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

    fn abstract_class(name: &[u8]) -> Self {
        Self {
            access_flags: PUBLIC | 0x0020 | ABSTRACT,
            ..Self::new(name)
        }
    }

    fn super_class(mut self, name: &[u8]) -> Self {
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

    /// One method that really carries a body, so a later test can charge code bytes on it.
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
    if let Some(position) = text.iter().position(|candidate| candidate == value) {
        return u16::try_from(position + 1).expect("fixture pool index fits u16");
    }
    text.push(value.to_vec());
    u16::try_from(text.len()).expect("fixture pool index fits u16")
}

/// Every class of the fixture world as an `(internal name, class bytes)` pair.
///
/// One world holds the hierarchy the searches walk (`java/lang/Object` as its root), the access
/// matrix (members of every access flag in two packages), the interface shapes of the Java 8
/// default rules, the signature-polymorphic declaration, and the shapes that make a branch
/// unreadable (a missing supertype and a cyclic chain).
macro_rules! world {
    ($($class:expr),* $(,)?) => {{
        let mut classes: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
        $(
            let class = $class;
            classes.push((class.name(), class.build()));
        )*
        classes
    }};
}

fn world_classes() -> Vec<(Vec<u8>, Vec<u8>)> {
    world![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base")
            .field(b"pub_f", b"I", PUBLIC)
            .field(b"priv_f", b"I", PRIVATE)
            .field(b"pkg_f", b"I", 0)
            .field(b"prot_f", b"I", PROTECTED)
            .field(b"stat_f", b"I", PUBLIC | STATIC)
            .method(b"pub_m", b"()V", PUBLIC)
            .method(b"priv_m", b"()V", PRIVATE)
            .method(b"pkg_m", b"()V", 0)
            .method(b"prot_m", b"()V", PROTECTED)
            .method(b"stat_m", b"()V", PUBLIC | STATIC)
            .method(b"<init>", b"()V", PUBLIC),
        Class::new(b"p/Other").method(b"invoke", b"()V", PUBLIC),
        Class::new(b"p/Sub").super_class(b"q/Base"),
        Class::new(b"p/DeepSub").super_class(b"p/Sub"),
        Class::new(b"p/OrphanSub").super_class(b"p/Absent"),
        Class::new(b"p/Orphan").super_class(b"p/Absent"),
        Class::new(b"p/CycA").super_class(b"p/CycB"),
        Class::new(b"p/CycB").super_class(b"p/CycA"),
        Class::new(b"p/Dup")
            .field(b"f", b"I", PUBLIC)
            .field(b"f", b"I", PUBLIC),
        Class::new(b"p/Inherit").super_class(b"q/Base"),
        Class::new(b"p/SuperField").field(b"f", b"I", PUBLIC),
        Class::new(b"p/Mixed")
            .super_class(b"p/SuperField")
            .implements(b"i/IfaceField"),
        // A method search over one owner that has both kinds of edge: the superclass chain holds
        // no `m` and the superinterface does, so one report publishes both read reasons.
        Class::new(b"p/SuperOnly").method(b"other_m", b"()V", PUBLIC),
        Class::new(b"p/BothEdges")
            .super_class(b"p/SuperOnly")
            .implements(b"i/Def"),
        Class::new(b"p/Body").method_with_body(b"run", b"()V", PUBLIC),
        Class::new(b"q/Base")
            .field(b"pub_f", b"I", PUBLIC)
            .field(b"priv_f", b"I", PRIVATE)
            .field(b"pkg_f", b"I", 0)
            .field(b"prot_f", b"I", PROTECTED)
            .method(b"pub_m", b"()V", PUBLIC)
            .method(b"priv_m", b"()V", PRIVATE)
            .method(b"pkg_m", b"()V", 0)
            .method(b"prot_m", b"()V", PROTECTED),
        Class::abstract_class(b"p/AbstractBase").method(b"abs_m", b"()V", PUBLIC | ABSTRACT),
        Class::new(b"p/Impl").super_class(b"p/AbstractBase"),
        Class::new(b"p/LoadBase").field(b"pkg_f", b"I", 0),
        Class::new(b"p/LoadCaller"),
        Class::interface(b"i/Abs").method(b"m", b"()V", PUBLIC | ABSTRACT),
        Class::interface(b"i/Def").method(b"m", b"()V", PUBLIC),
        Class::interface(b"i/Def2").method(b"m", b"()V", PUBLIC),
        // `static`/`private` interface declarations are outside the maximally-specific set
        // (JVMS 5.4.3.3): `i/StaticM` provides no candidate, and it must not suppress the
        // `default` of `i/Def` either; `i/PrivM`/`i/PrivSub` are the private shapes.
        Class::interface(b"i/StaticM").method(b"m", b"()V", PUBLIC | STATIC),
        Class::interface(b"i/StaticAndDefaultOwner")
            .implements(b"i/StaticM")
            .implements(b"i/Def"),
        Class::new(b"p/StaticOnlyUser").implements(b"i/StaticM"),
        Class::interface(b"i/PrivM").method(b"m", b"()V", PRIVATE),
        Class::new(b"p/PrivateConflictUser")
            .implements(b"i/PrivM")
            .implements(b"i/Def"),
        Class::interface(b"i/PrivSub")
            .implements(b"i/Def")
            .method(b"m", b"()V", PRIVATE),
        Class::new(b"p/PrivateSubUser").implements(b"i/PrivSub"),
        // Two direct superinterfaces that declare the same member: the declaration order of
        // `interfaces` is what decides, so each of these pins the order the search takes.
        Class::interface(b"i/FieldA").field(b"dup_f", b"I", PUBLIC | STATIC | FINAL),
        Class::interface(b"i/FieldB").field(b"dup_f", b"I", PUBLIC | STATIC | FINAL),
        Class::new(b"p/TwoIfaces")
            .implements(b"i/FieldA")
            .implements(b"i/FieldB"),
        Class::interface(b"i/MethodA").method(b"m", b"()V", PUBLIC | ABSTRACT),
        Class::interface(b"i/MethodB").method(b"m", b"()V", PUBLIC | ABSTRACT),
        Class::new(b"p/TwoMethodIfaces")
            .implements(b"i/MethodA")
            .implements(b"i/MethodB"),
        Class::interface(b"i/Child")
            .implements(b"i/Def")
            .method(b"m", b"()V", PUBLIC),
        Class::interface(b"i/One").implements(b"i/Def"),
        Class::interface(b"i/ConflictOwner")
            .implements(b"i/Def")
            .implements(b"i/Def2"),
        Class::interface(b"i/AbstractOwner").implements(b"i/Abs"),
        Class::interface(b"i/Dupm")
            .method(b"m", b"()V", PUBLIC)
            .method(b"m", b"()V", PUBLIC),
        Class::interface(b"i/DupOwner").implements(b"i/Dupm"),
        Class::interface(b"i/OverrideOwner").implements(b"i/Child"),
        Class::interface(b"i/IfaceField").field(b"f", b"I", PUBLIC | STATIC | FINAL),
        Class::new(b"p/IfaceDefaultUser").implements(b"i/Def"),
        Class::new(b"p/Deep").super_class(b"p/IfaceDefaultUser"),
        Class::new(b"Top").method(b"pkg_m", b"()V", 0),
        Class::new(b"TopOther"),
        Class::new(b"p/IfaceConflictUser")
            .implements(b"i/Def")
            .implements(b"i/Def2"),
        Class::new(b"java/lang/invoke/MethodHandle")
            .method(
                b"invoke",
                b"([Ljava/lang/Object;)Ljava/lang/Object;",
                PUBLIC | NATIVE
            )
            .method(
                b"invokeExact",
                b"([Ljava/lang/Object;)Ljava/lang/Object;",
                PUBLIC | NATIVE
            ),
    ]
}

/// The fixture world as stored entries: the entry name of one class is its own `this_class`.
fn entries() -> Vec<(Vec<u8>, Vec<u8>)> {
    world_classes()
        .into_iter()
        .map(|(name, bytes)| (entry(&name), bytes))
        .collect()
}

/// A world whose one root holds exactly the given `(entry name, class bytes)` pairs.
///
/// A fixture needs its own world when one entry name has to appear twice — the shared world is
/// one ZIP and holds each class once — or when it must not carry the shared classes at all.
fn world_of(pairs: Vec<(Vec<u8>, Vec<u8>)>) -> World {
    let snapshot = open(zip_of(&pairs));
    let app = domain(&loader("app"), None, vec![snapshot_root(&snapshot)]);
    let environment = environment(&snapshot, app.clone(), vec![app]);
    World {
        content: vec![snapshot],
        environment,
        app: loader("app"),
    }
}

/// One stored ZIP with the given entries.
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

/// The physical definition the engine derives for one stored entry.
///
/// Built from the snapshot's own public enumeration and the entry's bytes, so the test does not
/// restate an identity the engine owns. It is how a test names an entry whose *path* no lookup
/// resolves — the malformed candidates whose header declares another name.
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
        .expect("the fixture stores the named entry");
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
/// definition, and a ZIP snapshot is searched in its root container with an empty prefix. The
/// fixture says which shape it is; no layout prefix is ever inferred from it.
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

/// The fixture world plus the environment one test resolves under.
struct World {
    content: Vec<ArtifactSnapshot>,
    environment: ResolutionEnvironment,
    app: LoaderId,
}

impl World {
    /// The single-loader world: one `app` loader whose only root holds every fixture class.
    fn single() -> Self {
        let snapshot = open(zip_of(&entries()));
        let app = domain(&loader("app"), None, vec![snapshot_root(&snapshot)]);
        let environment = environment(&snapshot, app.clone(), vec![app]);
        Self {
            content: vec![snapshot],
            environment,
            app: loader("app"),
        }
    }

    /// The same world under one loader whose parent holds only `p/LoadBase`, so one class name
    /// resolves to a definition of the other loader.
    fn two_loaders() -> Self {
        let app = open(zip_of(&entries()));
        let platform = open(zip_of(&[(
            entry(b"p/LoadBase"),
            Class::new(b"p/LoadBase").field(b"pkg_f", b"I", 0).build(),
        )]));
        let platform_loader = loader("platform");
        let parent = domain(&platform_loader, None, vec![snapshot_root(&platform)]);
        let caller = domain(
            &loader("app"),
            Some(platform_loader),
            vec![snapshot_root(&app)],
        );
        let environment = environment(&app, caller.clone(), vec![caller, parent]);
        Self {
            content: vec![app, platform],
            environment,
            app: loader("app"),
        }
    }

    fn resolve(
        &self,
        target: SymbolRef,
        use_kind: ReferenceUse,
        caller: CallerContext,
    ) -> ResolutionReport {
        self.resolve_with(target, use_kind, caller, limits())
    }

    fn resolve_with(
        &self,
        target: SymbolRef,
        use_kind: ReferenceUse,
        caller: CallerContext,
        limits: Limits,
    ) -> ResolutionReport {
        let mut budget = Budget::new(limits);
        self.resolve_under(target, use_kind, caller, &mut budget)
    }

    /// Resolves one reference under a budget the caller owns.
    ///
    /// The stop paths (a pre-cancelled token, a limit that is already exhausted) need a budget
    /// the test built itself, so the request construction lives here once.
    fn resolve_under(
        &self,
        target: SymbolRef,
        use_kind: ReferenceUse,
        caller: CallerContext,
        budget: &mut Budget,
    ) -> ResolutionReport {
        let request = ResolutionRequest {
            environment: self.environment.clone(),
            target,
            use_kind,
            caller,
            dispatch: None,
        };
        Engine::new()
            .resolve_symbol(&self.content, &request, budget)
            .expect("a legal request is answered, not raised")
    }

    /// The physical definition of one fixture class, read through the class-symbol lookup.
    fn definition(&self, class: &[u8]) -> PhysicalDefinitionId {
        let request = ResolutionRequest {
            environment: self.environment.clone(),
            target: SymbolRef::Class {
                owner: JvmBytes(class.to_vec()),
            },
            use_kind: ReferenceUse::ClassReference,
            caller: CallerContext {
                loader: self.app.clone(),
                enclosing: None,
            },
            dispatch: None,
        };
        Engine::new()
            .resolve_symbol(&self.content, &request, &mut Budget::new(limits()))
            .expect("the fixture class request is legal")
            .resolved
            .expect("the fixture class resolves")
            .definition
    }

    /// The caller identity of one fixture class: the use site's enclosing method is a method of
    /// that class, which is what names the caller's class.
    fn caller(&self, class: &[u8]) -> CallerContext {
        self.caller_in(class, self.app.clone())
    }

    fn caller_in(&self, class: &[u8], loader: LoaderId) -> CallerContext {
        CallerContext {
            loader,
            enclosing: Some(PhysicalMethodId {
                owner: self.definition(class),
                name: JvmBytes(b"bench".to_vec()),
                descriptor: JvmBytes(b"()V".to_vec()),
            }),
        }
    }

    /// A caller whose class is unknown: the request names no use site.
    fn abstract_caller(&self) -> CallerContext {
        CallerContext {
            loader: self.app.clone(),
            enclosing: None,
        }
    }
}

/// The name one built class file declares for itself (its own bytes are the fixture's source of
/// truth, so an entry name cannot silently disagree with it).
fn field(owner: &[u8], name: &[u8], descriptor: &[u8]) -> SymbolRef {
    SymbolRef::Field {
        owner: JvmBytes(owner.to_vec()),
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
    }
}

fn method(owner: &[u8], name: &[u8], descriptor: &[u8]) -> SymbolRef {
    SymbolRef::Method {
        owner: JvmBytes(owner.to_vec()),
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
    }
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

fn diagnostic_codes(report: &ResolutionReport) -> Vec<String> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.clone())
        .collect()
}

fn diagnostic_of<'a>(report: &'a ResolutionReport, code: &str) -> &'a Diagnostic {
    report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == code)
        .unwrap_or_else(|| panic!("the report carries `{code}`: {:?}", report.diagnostics))
}

fn resolved_of(report: &ResolutionReport) -> &ResolvedMemberRef {
    report
        .resolved
        .as_ref()
        .expect("a resolved member publishes its declaration")
}

/// The `(loader, reason)` pairs of one report's read records, in read order.
fn read_reasons(report: &ResolutionReport) -> Vec<(String, ReadReason)> {
    report
        .reads
        .iter()
        .map(|read| (read.loader.0.clone(), read.reason))
        .collect()
}

/// The 2.2 inequality every report has to keep: no more records than read attempts.
fn assert_reads_within_attempts(report: &ResolutionReport) {
    assert!(
        u64::try_from(report.reads.len()).expect("fixture read count fits u64")
            <= class_headers(report),
        "reads ({}) must not exceed the charged header attempts ({}): {:?}",
        report.reads.len(),
        class_headers(report),
        report.reads
    );
}

/// A header search reads no instruction and charges no body read.
///
/// The evidence that no Body was read is `code_bytes == 0`. `method_bodies` has no charge point
/// before 3.x, so its zero is asserted here as the dimension's own fact and proves nothing about
/// the search — the asserted inequality above is what says the records stay within the attempts.
fn assert_no_body_read(report: &ResolutionReport) {
    let usage = usage_of(&report.execution);
    assert_eq!(
        usage.counted_usage(CountedBudgetDimension::CodeBytes),
        0,
        "a header search reads no instruction: {usage:?}"
    );
    assert_eq!(
        usage.counted_usage(CountedBudgetDimension::MethodBodies),
        0,
        "no body read is charged (the dimension has no charge point before 3.x): {usage:?}"
    );
}

/// The owner of one member symbol.
fn owner_of(symbol: &SymbolRef) -> &[u8] {
    match symbol {
        SymbolRef::Class { owner }
        | SymbolRef::Field { owner, .. }
        | SymbolRef::Method { owner, .. } => &owner.0,
    }
}

/// The descriptor of a field or method symbol.
fn descriptor_of(symbol: &SymbolRef) -> &[u8] {
    match symbol {
        SymbolRef::Class { .. } => panic!("a class symbol has no descriptor"),
        SymbolRef::Field { descriptor, .. } | SymbolRef::Method { descriptor, .. } => &descriptor.0,
    }
}

#[test]
fn every_fixture_class_resolves_as_a_class_symbol() {
    let world = World::single();
    for (name, _) in world_classes() {
        let request = ResolutionRequest {
            environment: world.environment.clone(),
            target: SymbolRef::Class {
                owner: JvmBytes(name.clone()),
            },
            use_kind: ReferenceUse::ClassReference,
            caller: CallerContext {
                loader: world.app.clone(),
                enclosing: None,
            },
            dispatch: None,
        };
        let report = Engine::new()
            .resolve_symbol(&world.content, &request, &mut Budget::new(limits()))
            .expect("the fixture class request is legal");
        assert_eq!(
            report.state,
            Some(ResolutionState::Resolved),
            "fixture `{}` must be a class file the reader accepts: {report:?}",
            String::from_utf8_lossy(&name)
        );
    }
}

#[test]
fn the_owner_state_is_inherited_when_the_owner_does_not_resolve() {
    let world = World::single();
    let target = field(b"p/Absent", b"f", b"I");
    let report = world.resolve(
        target.clone(),
        ReferenceUse::FieldRead,
        world.caller(b"p/Base"),
    );

    assert_eq!(report.state, Some(ResolutionState::Missing));
    assert!(report.resolved.is_none());
    assert!(report.candidates.is_empty());
    assert_eq!(report.target, target);
    assert_eq!(
        class_headers(&report),
        0,
        "a name no position holds is listed but never read, and the member search has no owner"
    );
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn an_inconsistent_caller_definition_is_a_stop_and_never_a_wrong_check() {
    // The access rules read the caller's class from the definition the request names. A
    // definition that does not describe the provided bytes would measure the access against a
    // class the request never named, so the request stops with the inconsistency named instead.
    let world = World::single();
    let mut caller = world.caller(b"p/Other");
    let enclosing = caller
        .enclosing
        .as_mut()
        .expect("the fixture caller names its use site");
    enclosing.owner.class_bytes.length += 1;

    let report = world.resolve(
        method(b"p/Base", b"priv_m", b"()V"),
        ReferenceUse::InvokeVirtual,
        caller,
    );
    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert!(
        report.state.is_none(),
        "an inconsistent request is not a semantic decision: {:?}",
        report.state
    );
    assert_eq!(diagnostic_codes(&report), vec!["class_definition_mismatch"]);
    assert!(matches!(
        report.execution,
        ExecutionReport::Failed {
            reason: TerminationReason::Error { ref code },
            ..
        } if code == "class_definition_mismatch"
    ));
    assert!(report.resolved.is_none());

    // The definition naming content the request does not provide is the same kind of stop: the
    // caller's class cannot be read, and no access rule is decided on it.
    let mut caller = world.caller(b"p/Other");
    let enclosing = caller
        .enclosing
        .as_mut()
        .expect("the fixture caller names its use site");
    enclosing.owner.location = PhysicalClassLocation::StandaloneRoot {
        snapshot: SnapshotId("elsewhere".to_string()),
    };
    let report = world.resolve(
        method(b"p/Base", b"priv_m", b"()V"),
        ReferenceUse::InvokeVirtual,
        caller,
    );
    assert!(report.state.is_none());
    assert_eq!(diagnostic_codes(&report), vec!["content_not_provided"]);
    assert!(report.resolved.is_none());
}

#[test]
fn two_declarations_in_one_interface_are_ambiguous() {
    let world = World::single();
    let report = world.resolve(
        method(b"i/DupOwner", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        world.caller(b"p/Other"),
    );

    assert_eq!(report.state, Some(ResolutionState::Ambiguous));
    assert!(report.resolved.is_none());
    assert_eq!(
        report.candidates.len(),
        2,
        "the closure reached the interface that declares the member twice"
    );
    assert_eq!(
        report.candidates[0], report.candidates[1],
        "one interface has one coordinate in this schema, so the two declarations can only be \
         counted"
    );
}

#[test]
fn a_class_inherits_a_default_method_through_its_superclass_interfaces() {
    // JVMS 5.4.3.3 searches the superinterfaces of the class, and those include the interfaces
    // its superclasses implement — not only the ones the class declares itself.
    let world = World::single();
    let report = world.resolve(
        method(b"p/Deep", b"m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Other"),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(
        resolved_of(&report).member,
        method(b"i/Def", b"m", b"()V"),
        "the default comes from the interface the superclass implements"
    );
    assert_eq!(
        read_reasons(&report),
        vec![
            ("app".to_string(), ReadReason::MemberOwner),
            // `p/Deep extends p/IfaceDefaultUser extends java/lang/Object`: two `super_class`
            // steps, both parent-chain reads.
            ("app".to_string(), ReadReason::ParentChain),
            ("app".to_string(), ReadReason::ParentChain),
            // The interface the chain implements: an `interfaces` step of the chain.
            ("app".to_string(), ReadReason::HierarchyClosure),
        ],
        "the chain steps and the interface step of the chain keep their own reasons"
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn the_default_package_is_a_run_time_package_of_its_own() {
    let world = World::single();

    let same = world.resolve(
        method(b"Top", b"pkg_m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"TopOther"),
    );
    assert_eq!(same.state, Some(ResolutionState::Resolved));
    assert!(same.diagnostics.is_empty(), "{:?}", same.diagnostics);

    let other = world.resolve(
        method(b"Top", b"pkg_m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Other"),
    );
    assert_eq!(other.state, Some(ResolutionState::Inaccessible));
    assert_eq!(diagnostic_codes(&other), vec!["resolution_access_denied"]);
}

#[test]
fn a_member_of_the_owner_resolves_to_its_own_declaration() {
    let world = World::single();
    let target = field(b"p/Base", b"pub_f", b"I");
    let report = world.resolve(
        target.clone(),
        ReferenceUse::FieldRead,
        world.caller(b"p/Base"),
    );

    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(report.target, target, "the raw reference is kept");
    assert_eq!(report.use_kind, ReferenceUse::FieldRead);
    assert_eq!(report.environment_problems, Vec::new());
    assert!(
        report.diagnostics.is_empty(),
        "a public field of the caller's own class needs no rule: {:?}",
        report.diagnostics
    );
    assert!(report.candidates.is_empty());
    assert!(report.dispatch.is_none());
    assert_eq!(
        resolved_of(&report).definition,
        world.definition(b"p/Base"),
        "the definition is the one the declaring class was read from"
    );
    assert_eq!(resolved_of(&report).loader, loader("app"));
    assert_eq!(resolved_of(&report).member, target);
    assert_eq!(
        read_reasons(&report),
        vec![("app".to_string(), ReadReason::MemberOwner)],
        "the class the reference names is a member-owner read"
    );
    assert_eq!(class_headers(&report), 1);
    assert_eq!(
        code_bytes(&report),
        0,
        "a header search reads no code byte: {report:?}"
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema
    );
    assert_eq!(
        report.coverage.runtime_resolution.scanned,
        vec![CoverageRange {
            label: "provider_search_position".to_string(),
            start: 0,
            end: 1,
        }],
        "one class-name search examined one position"
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn an_inherited_member_resolves_to_the_declaring_class() {
    let world = World::single();
    let report = world.resolve(
        method(b"p/Inherit", b"pub_m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Inherit"),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(
        resolved_of(&report).member,
        method(b"q/Base", b"pub_m", b"()V"),
        "the member is the declaration: its owner is the declaring class, not the reference's"
    );
    assert_eq!(resolved_of(&report).definition, world.definition(b"q/Base"));
    assert_eq!(
        read_reasons(&report),
        vec![
            // The class the reference names, read by identity.
            ("app".to_string(), ReadReason::MemberOwner),
            // `p/Inherit extends q/Base`: the step is a `super_class` edge.
            ("app".to_string(), ReadReason::ParentChain),
        ],
        "the superclass step is a parent-chain read, not an interface read"
    );
    assert_eq!(class_headers(&report), 2);
    assert_eq!(code_bytes(&report), 0);
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn a_search_that_takes_both_kinds_of_edge_publishes_both_reasons() {
    // `p/BothEdges extends p/SuperOnly implements i/Def`: the method search walks the superclass
    // chain first (which holds no `m`), then the superinterface, so one report has to carry a
    // parent-chain read *and* an interface read. Without this fixture a change that labels every
    // hierarchy step with one reason would leave the other variant unproduced here.
    let world = World::single();
    let report = world.resolve(
        method(b"p/BothEdges", b"m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Other"),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(
        resolved_of(&report).member,
        method(b"i/Def", b"m", b"()V"),
        "the default comes from the interface the superclass chain did not provide"
    );
    assert_eq!(
        read_reasons(&report),
        vec![
            ("app".to_string(), ReadReason::MemberOwner),
            // `p/BothEdges extends p/SuperOnly extends java/lang/Object`: two `super_class`
            // steps, both parent-chain reads, and neither of them declares `m`.
            ("app".to_string(), ReadReason::ParentChain),
            ("app".to_string(), ReadReason::ParentChain),
            // Then the superinterface of the class chain: an `interfaces` step.
            ("app".to_string(), ReadReason::HierarchyClosure),
        ],
        "the same search publishes a parent-chain read for each superclass step and an interface \
         read for the superinterface step, in the order the search took them"
    );
    let reasons = read_reasons(&report);
    for expected in [ReadReason::ParentChain, ReadReason::HierarchyClosure] {
        assert!(
            reasons.iter().any(|(_, reason)| *reason == expected),
            "the reason sequence really produces {expected:?}: {reasons:?}"
        );
    }
    assert_eq!(class_headers(&report), 4);
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn an_interface_field_is_found_before_a_superclass_field() {
    let world = World::single();
    let report = world.resolve(
        field(b"p/Mixed", b"f", b"I"),
        ReferenceUse::FieldRead,
        world.caller(b"p/Mixed"),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        resolved_of(&report).member,
        field(b"i/IfaceField", b"f", b"I"),
        "JVMS 5.4.3.2 searches the direct superinterfaces before the superclass"
    );
    assert_eq!(class_headers(&report), 2);
    assert_eq!(
        read_reasons(&report),
        vec![
            ("app".to_string(), ReadReason::MemberOwner),
            // `p/Mixed implements i/IfaceField`: the field walk takes an `interfaces` edge.
            ("app".to_string(), ReadReason::HierarchyClosure),
        ],
        "the field step that found the declaration is an interface read"
    );
    assert!(
        report
            .reads
            .iter()
            .all(|read| read.definition != world.definition(b"p/SuperField")),
        "the search stopped at the interface field and never read the superclass: {:?}",
        report.reads
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn a_missing_member_keeps_the_reference_and_fabricates_nothing() {
    let world = World::single();
    let target = field(b"p/Base", b"absent_f", b"I");
    let report = world.resolve(
        target.clone(),
        ReferenceUse::FieldRead,
        world.caller(b"p/Base"),
    );

    assert_eq!(report.state, Some(ResolutionState::Missing));
    assert!(report.resolved.is_none());
    assert!(report.candidates.is_empty());
    assert_eq!(report.target, target, "the raw symbol and descriptor stay");
    assert_eq!(
        report.diagnostics,
        Vec::new(),
        "a name no class declares is a fact, not a warning"
    );
    assert_eq!(
        class_headers(&report),
        2,
        "the class itself and its superclass were searched"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema,
        "the whole hierarchy was readable, so `Missing` is complete"
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn a_duplicate_declaration_in_one_class_is_ambiguous() {
    let world = World::single();
    let report = world.resolve(
        field(b"p/Dup", b"f", b"I"),
        ReferenceUse::FieldRead,
        world.caller(b"p/Dup"),
    );

    assert_eq!(report.state, Some(ResolutionState::Ambiguous));
    assert!(report.resolved.is_none());
    assert_eq!(
        report.candidates.len(),
        2,
        "both declarations of the class are published"
    );
    assert_eq!(
        report.candidates[0], report.candidates[1],
        "the report schema has no within-class coordinate, so two indistinguishable declarations \
         of one class can only be counted, never told apart: {:?}",
        report.candidates
    );
    assert_eq!(class_headers(&report), 1);
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn a_missing_supertype_leaves_its_branch_unread() {
    let world = World::single();
    let report = world.resolve(
        field(b"p/Orphan", b"f", b"I"),
        ReferenceUse::FieldRead,
        world.caller(b"p/Orphan"),
    );

    assert_eq!(
        report.state,
        Some(ResolutionState::UnresolvedDependency),
        "`p/Absent` is not in this snapshot, so the search never read the branch that could \
         declare `f`: the negation `Missing` is not claimed (A11, 2.2)"
    );
    assert_eq!(
        report.unresolved_dependencies,
        vec![UnresolvedDependency {
            name: JvmBytes(b"p/Absent".to_vec()),
            loader: loader("app"),
            reason: ReadReason::ParentChain,
            declared_by: Some(JvmBytes(b"p/Orphan".to_vec())),
            gap: DependencyGap::Missing,
        }],
        "the unread class is named, with the `super_class` edge of `p/Orphan` that needed it"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_hierarchy_missing"]
    );
    let diagnostic = diagnostic_of(&report, "resolution_hierarchy_missing");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
    assert!(
        diagnostic.message.contains("p/Absent"),
        "the warning names the unreadable class: {}",
        diagnostic.message
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial,
        "an unread branch makes the resolution plane partial even though it decided"
    );
    assert_eq!(
        class_headers(&report),
        1,
        "a name no position holds is listed but never read"
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn a_supertype_that_cannot_be_told_apart_leaves_its_branch_unread() {
    // The superclass name of `p/AmbSub` has two indistinguishable definitions at one selection
    // position. The branch cannot be read — no ordinal or hash decides which one to search — so
    // the resolution says so and stays incomplete instead of picking one of them.
    let world = world_of(vec![
        (
            entry(b"p/AmbBase"),
            Class::new(b"p/AmbBase")
                .super_class(b"p/AmbTop")
                .field(b"f", b"I", PUBLIC)
                .build(),
        ),
        (
            entry(b"p/AmbBase"),
            Class::new(b"p/AmbBase")
                .super_class(b"p/AmbTop")
                .method(b"other", b"()V", PUBLIC)
                .build(),
        ),
        (
            entry(b"p/AmbSub"),
            Class::new(b"p/AmbSub").super_class(b"p/AmbBase").build(),
        ),
        (
            entry(b"p/AmbTop"),
            Class::new(b"p/AmbTop").field(b"g", b"I", PUBLIC).build(),
        ),
    ]);
    let report = world.resolve(
        field(b"p/AmbSub", b"f", b"I"),
        ReferenceUse::FieldRead,
        world.abstract_caller(),
    );

    assert_eq!(
        report.state,
        Some(ResolutionState::UnresolvedDependency),
        "the superclass position of `p/AmbSub` cannot be told apart, so the search never entered \
         it: no declaration is claimed and `Missing` is not the answer either (A11, 2.2)"
    );
    assert!(report.resolved.is_none());
    assert_eq!(
        report.unresolved_dependencies,
        vec![UnresolvedDependency {
            name: JvmBytes(b"p/AmbBase".to_vec()),
            loader: loader("app"),
            reason: ReadReason::ParentChain,
            declared_by: Some(JvmBytes(b"p/AmbSub".to_vec())),
            gap: DependencyGap::Ambiguous,
        }],
        "the gap names the position that could not be resolved and the class that needed it"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_hierarchy_ambiguous"]
    );
    let diagnostic = diagnostic_of(&report, "resolution_hierarchy_ambiguous");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
    assert!(
        diagnostic.message.contains("p/AmbBase")
            && diagnostic.message.contains("cannot be told apart"),
        "the warning names the position it could not decide: {}",
        diagnostic.message
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial,
        "the branch that could not be read leaves the resolution plane partial"
    );
    assert_eq!(
        class_headers(&report),
        3,
        "the owner is one attempt and each indistinguishable candidate is one of its own"
    );
    assert_eq!(
        read_reasons(&report),
        vec![
            ("app".to_string(), ReadReason::MemberOwner),
            // The ambiguous superclass is the field walk's `super_class` step, and both of its
            // candidates really were read before the branch was declared undecidable.
            ("app".to_string(), ReadReason::ParentChain),
            ("app".to_string(), ReadReason::ParentChain),
        ],
        "both candidates of the ambiguous position are recorded, under the edge that reached them"
    );
    assert!(
        report.reads.iter().all(|read| {
            read.definition
                .entry()
                .expect("every fixture definition is an archive entry")
                .raw_name
                .0
                != entry(b"p/AmbTop")
        }),
        "an undecidable branch is not searched through, so the supertypes of the class it could \
         not decide stay unread: {:?}",
        report.reads
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn a_cyclic_superclass_chain_stops_with_a_warning() {
    let world = World::single();

    // The field order follows the superinterfaces and then the superclass, so its own walk has
    // to refuse the repeating edge.
    let field_report = world.resolve(
        field(b"p/CycA", b"f", b"I"),
        ReferenceUse::FieldRead,
        world.caller(b"p/CycA"),
    );
    assert_eq!(
        field_report.state,
        Some(ResolutionState::UnresolvedDependency),
        "the superclass chain repeats, so the search never read the whole hierarchy: the \
         declaration is undecided, not absent (A11, 2.2)"
    );
    assert_eq!(
        field_report.unresolved_dependencies,
        vec![UnresolvedDependency {
            name: JvmBytes(b"p/CycA".to_vec()),
            loader: loader("app"),
            reason: ReadReason::ParentChain,
            declared_by: Some(JvmBytes(b"p/CycB".to_vec())),
            gap: DependencyGap::Cyclic,
        }],
        "the refused edge names the class that repeats on its own path and the class that \
         declares that edge"
    );
    assert_eq!(
        diagnostic_codes(&field_report),
        vec!["resolution_hierarchy_cycle"]
    );
    assert_eq!(
        class_headers(&field_report),
        2,
        "the chain stops at the repeat"
    );
    assert_eq!(
        read_reasons(&field_report),
        vec![
            ("app".to_string(), ReadReason::MemberOwner),
            // `p/CycA extends p/CycB`, and p/CycB is read before its own edge is refused.
            ("app".to_string(), ReadReason::ParentChain),
        ],
        "a field walk's superclass step is a parent-chain read"
    );
    assert_eq!(
        field_report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert_reads_within_attempts(&field_report);
    assert_no_body_read(&field_report);

    // The superclass chain of a method refuses the same edge before it is read.
    let method_report = world.resolve(
        method(b"p/CycA", b"m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/CycA"),
    );
    assert_eq!(
        method_report.state,
        Some(ResolutionState::UnresolvedDependency),
        "the method's own superclass walk refuses the same repeating edge, so it is undecided \
         the same way rather than negated"
    );
    assert_eq!(
        method_report.unresolved_dependencies, field_report.unresolved_dependencies,
        "both walks state the one refused edge"
    );
    assert_eq!(
        diagnostic_codes(&method_report),
        vec!["resolution_hierarchy_cycle"]
    );
    assert_eq!(class_headers(&method_report), 2);
    assert_eq!(
        read_reasons(&method_report),
        vec![
            ("app".to_string(), ReadReason::MemberOwner),
            ("app".to_string(), ReadReason::ParentChain),
        ],
        "the whole-superclass-chain walk reads the same edge kind as the field walk"
    );
    assert_reads_within_attempts(&method_report);
    assert_no_body_read(&method_report);
}

#[test]
fn a_field_reference_is_not_held_to_the_static_rules() {
    // The field rules compare the instruction (`GetStatic`/`PutStatic` against the instance
    // forms), which this vocabulary does not carry: a static field read and a package-private
    // field write both resolve, and neither is a kind mismatch.
    let world = World::single();
    let stat = world.resolve(
        field(b"p/Base", b"stat_f", b"I"),
        ReferenceUse::FieldRead,
        world.caller(b"p/Base"),
    );
    assert_eq!(stat.state, Some(ResolutionState::Resolved));
    assert!(stat.diagnostics.is_empty(), "{:?}", stat.diagnostics);

    let written = world.resolve(
        field(b"p/Base", b"pkg_f", b"I"),
        ReferenceUse::FieldWrite,
        world.caller(b"p/Other"),
    );
    assert_eq!(written.state, Some(ResolutionState::Resolved));
    assert!(written.diagnostics.is_empty(), "{:?}", written.diagnostics);
}

#[test]
fn a_static_method_resolves_for_a_static_reference() {
    let world = World::single();
    let report = world.resolve(
        method(b"p/Base", b"stat_m", b"()V"),
        ReferenceUse::InvokeStatic,
        world.caller(b"p/Base"),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(
        resolved_of(&report).member,
        method(b"p/Base", b"stat_m", b"()V")
    );
}

#[test]
fn an_instance_method_with_a_static_reference_is_an_incompatible_change() {
    let world = World::single();
    let report = world.resolve(
        method(b"p/Base", b"pub_m", b"()V"),
        ReferenceUse::InvokeStatic,
        world.caller(b"p/Base"),
    );

    assert_eq!(report.state, Some(ResolutionState::IncompatibleClassChange));
    assert!(report.resolved.is_none());
    assert_eq!(diagnostic_codes(&report), vec!["resolution_kind_mismatch"]);
    let diagnostic = diagnostic_of(&report, "resolution_kind_mismatch");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert!(
        diagnostic.message.contains("p/Base") && diagnostic.message.contains("pub_m"),
        "the rejection names the declaration it refused: {}",
        diagnostic.message
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn a_static_method_with_an_instance_reference_is_an_incompatible_change() {
    let world = World::single();
    for use_kind in [ReferenceUse::InvokeVirtual, ReferenceUse::InvokeInterface] {
        // `invoke_interface` meets the owner-kind rule first: `p/Base` is a class, so the
        // rejection is the interface/class mismatch and not the static rule.
        let report = world.resolve(
            method(b"p/Base", b"stat_m", b"()V"),
            use_kind,
            world.caller(b"p/Base"),
        );
        assert_eq!(
            report.state,
            Some(ResolutionState::IncompatibleClassChange),
            "{use_kind:?}"
        );
        assert_eq!(diagnostic_codes(&report), vec!["resolution_kind_mismatch"]);
    }
}

#[test]
fn a_constructor_resolves_through_invoke_special_only() {
    let world = World::single();
    let target = method(b"p/Base", b"<init>", b"()V");

    let resolved = world.resolve(
        target.clone(),
        ReferenceUse::InvokeSpecial,
        world.caller(b"p/Base"),
    );
    assert_eq!(resolved.state, Some(ResolutionState::Resolved));
    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    assert_eq!(resolved_of(&resolved).member, target);

    for use_kind in [
        ReferenceUse::InvokeVirtual,
        ReferenceUse::InvokeStatic,
        ReferenceUse::InvokeDynamic,
    ] {
        let refused = world.resolve(target.clone(), use_kind, world.caller(b"p/Base"));
        assert_eq!(
            refused.state,
            Some(ResolutionState::IncompatibleClassChange),
            "{use_kind:?} must not resolve a constructor"
        );
        assert_eq!(diagnostic_codes(&refused), vec!["resolution_kind_mismatch"]);
    }
}

#[test]
fn an_interface_reference_needs_an_interface_owner() {
    let world = World::single();

    // Control: the owner is an interface, so interface method resolution applies and the
    // declaration is found in the interface itself.
    let resolved = world.resolve(
        method(b"i/Def", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        world.caller(b"p/Other"),
    );
    assert_eq!(resolved.state, Some(ResolutionState::Resolved));
    assert!(
        resolved.diagnostics.is_empty(),
        "{:?}",
        resolved.diagnostics
    );
    assert_eq!(
        resolved_of(&resolved).member,
        method(b"i/Def", b"m", b"()V")
    );

    let refused = world.resolve(
        method(b"p/Base", b"pub_m", b"()V"),
        ReferenceUse::InvokeInterface,
        world.caller(b"p/Base"),
    );
    assert_eq!(
        refused.state,
        Some(ResolutionState::IncompatibleClassChange)
    );
    let diagnostic = diagnostic_of(&refused, "resolution_kind_mismatch");
    assert!(
        diagnostic.message.contains("p/Base") && diagnostic.message.contains("invoke_interface"),
        "the rejection names the owner kind the reference required: {}",
        diagnostic.message
    );
}

#[test]
fn an_interface_super_call_resolves_through_invoke_special() {
    // `I.super.m()`: the call site sits in the subinterface that overrides the method with its
    // own `default`, and the constant-pool entry names the **direct superinterface's** method —
    // here `i/Def.m`, because `i/Child` overrides it. Resolution follows the reference's owner,
    // so it must land on `i/Def.m` even though the caller's own class declares `m` too. An
    // interface owner is a legal owner for `invoke_special` on the same grounds as a class
    // owner: the rule set has no class-only requirement for this kind.
    let world = World::single();
    let super_method = method(b"i/Def", b"m", b"()V");
    let report = world.resolve(
        super_method.clone(),
        ReferenceUse::InvokeSpecial,
        world.caller(b"i/Child"),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert!(
        report.diagnostics.is_empty(),
        "a non-abstract, public interface method named by `invoke_special` satisfies every rule: \
         {:?}",
        report.diagnostics
    );
    assert_eq!(report.target, super_method, "the raw reference is kept");
    assert_eq!(
        resolved_of(&report).member,
        super_method,
        "the declaration the reference names is the superinterface's method, not the caller's \
         override"
    );
    assert_eq!(
        read_reasons(&report),
        vec![("app".to_string(), ReadReason::MemberOwner)],
        "the declaring interface is the owner itself, so the search never leaves it"
    );
    assert_eq!(
        class_headers(&report),
        1,
        "one owner read: the public member needs no rule that reads the caller's class"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);

    // The contrast that makes the assertion above about the reference's owner and not about the
    // search always preferring `i/Def`: naming the subinterface's own override resolves to it.
    let own_override = method(b"i/Child", b"m", b"()V");
    let own = world.resolve(
        own_override.clone(),
        ReferenceUse::InvokeSpecial,
        world.caller(b"i/Child"),
    );
    assert_eq!(own.state, Some(ResolutionState::Resolved));
    assert!(own.diagnostics.is_empty(), "{:?}", own.diagnostics);
    assert_eq!(resolved_of(&own).member, own_override);
    assert_eq!(class_headers(&own), 1);
    assert_reads_within_attempts(&own);
    assert_no_body_read(&own);
}

#[test]
fn a_class_reference_needs_a_class_owner() {
    let world = World::single();
    let refused = world.resolve(
        method(b"i/Def", b"m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Other"),
    );

    assert_eq!(
        refused.state,
        Some(ResolutionState::IncompatibleClassChange)
    );
    assert_eq!(diagnostic_codes(&refused), vec!["resolution_kind_mismatch"]);
    assert_eq!(
        class_headers(&refused),
        1,
        "the owner header was read before the rule could reject it"
    );
    assert_reads_within_attempts(&refused);
    assert_no_body_read(&refused);
}

#[test]
fn an_abstract_method_resolves_and_warns() {
    let world = World::single();
    let target = method(b"p/Impl", b"abs_m", b"()V");

    let resolved = world.resolve(
        target.clone(),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Impl"),
    );
    assert_eq!(resolved.state, Some(ResolutionState::Resolved));
    assert_eq!(
        diagnostic_codes(&resolved),
        vec!["resolution_method_is_abstract"]
    );
    assert_eq!(
        diagnostic_of(&resolved, "resolution_method_is_abstract").severity,
        DiagnosticSeverity::Warning
    );
    assert_eq!(
        resolved_of(&resolved).member,
        method(b"p/AbstractBase", b"abs_m", b"()V"),
        "resolution succeeds on the abstract declaration; the failure belongs to the invocation"
    );

    let refused = world.resolve(target, ReferenceUse::InvokeSpecial, world.caller(b"p/Impl"));
    assert_eq!(
        refused.state,
        Some(ResolutionState::IncompatibleClassChange)
    );
    assert_eq!(diagnostic_codes(&refused), vec!["resolution_kind_mismatch"]);
}

#[test]
fn a_single_default_method_resolves_through_the_interface_step() {
    let world = World::single();
    let report = world.resolve(
        method(b"i/One", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        world.caller(b"p/Other"),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(
        resolved_of(&report).member,
        method(b"i/Def", b"m", b"()V"),
        "the one non-abstract maximally-specific declaration is the default that resolves"
    );
    assert_eq!(
        read_reasons(&report),
        vec![
            ("app".to_string(), ReadReason::MemberOwner),
            ("app".to_string(), ReadReason::HierarchyClosure),
        ]
    );
    assert_eq!(class_headers(&report), 2);
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn two_default_methods_are_a_default_conflict() {
    let world = World::single();
    for (owner, use_kind) in [
        (b"i/ConflictOwner".as_slice(), ReferenceUse::InvokeInterface),
        (
            b"p/IfaceConflictUser".as_slice(),
            ReferenceUse::InvokeVirtual,
        ),
    ] {
        let report = world.resolve(method(owner, b"m", b"()V"), use_kind, world.caller(owner));

        assert_eq!(
            report.state,
            Some(ResolutionState::IncompatibleClassChange),
            "{owner:?}: two non-abstract defaults cannot be told apart"
        );
        assert!(report.resolved.is_none());
        assert_eq!(
            diagnostic_codes(&report),
            vec!["resolution_default_conflict"]
        );
        let diagnostic = diagnostic_of(&report, "resolution_default_conflict");
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        for expected in ["i/Def", "i/Def2"] {
            assert!(
                diagnostic.message.contains(expected),
                "the conflict names both declarations: {}",
                diagnostic.message
            );
        }
        assert!(
            report.candidates.is_empty(),
            "a conflict is not an indistinguishable position: {:?}",
            report.candidates
        );
        assert_reads_within_attempts(&report);
        assert_no_body_read(&report);
    }
}

#[test]
fn an_abstract_only_interface_resolves_with_a_warning() {
    let world = World::single();
    let report = world.resolve(
        method(b"i/AbstractOwner", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        world.caller(b"p/Other"),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_method_is_abstract"]
    );
    assert_eq!(
        resolved_of(&report).member,
        method(b"i/Abs", b"m", b"()V"),
        "an all-abstract maximally-specific set still resolves, deterministically"
    );
}

#[test]
fn a_subinterface_default_overrides_its_parent() {
    let world = World::single();
    let report = world.resolve(
        method(b"i/OverrideOwner", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        world.caller(b"p/Other"),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert!(
        report.diagnostics.is_empty(),
        "the parent default is overridden, not a conflict: {:?}",
        report.diagnostics
    );
    assert_eq!(
        resolved_of(&report).member,
        method(b"i/Child", b"m", b"()V"),
        "the maximally-specific declaration is the more derived one"
    );
    assert_eq!(
        class_headers(&report),
        3,
        "the owner, the overriding interface and the overridden one were all read"
    );
    assert!(
        report
            .reads
            .iter()
            .filter(|read| read.reason == ReadReason::HierarchyClosure)
            .count()
            == 2,
        "{:?}",
        report.reads
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn a_class_inherits_a_default_method_through_the_interface_step() {
    let world = World::single();
    let report = world.resolve(
        method(b"p/IfaceDefaultUser", b"m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Other"),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        resolved_of(&report).member,
        method(b"i/Def", b"m", b"()V"),
        "JVMS 5.4.3.3 reaches the superinterfaces only when the superclass chain holds nothing"
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn a_static_interface_declaration_is_ignored_by_the_superinterface_step() {
    // `i/StaticM` declares a `static` method and `i/Def` a `default` one; both are direct
    // superinterfaces of the owner. JVMS ignores the static declaration by resolution, so the
    // maximally-specific set is `i/Def.m` alone: resolving to the static method would be wrong,
    // and reporting a conflict between a static and a `default` declaration would be wrong too.
    let world = World::single();

    let through_interface = world.resolve(
        method(b"i/StaticAndDefaultOwner", b"m", b"()V"),
        ReferenceUse::InvokeInterface,
        world.caller(b"p/Other"),
    );
    assert_eq!(
        through_interface.state,
        Some(ResolutionState::Resolved),
        "a static superinterface method neither resolves nor causes a conflict: {:?}",
        through_interface.diagnostics
    );
    assert!(
        through_interface.diagnostics.is_empty(),
        "the one remaining candidate is a plain default: {:?}",
        through_interface.diagnostics
    );
    assert_eq!(
        resolved_of(&through_interface).member,
        method(b"i/Def", b"m", b"()V"),
        "the default of the other superinterface is the maximally-specific declaration"
    );

    // Through a class, where the same static declaration leaves the interface step empty: the
    // reference does not name the static method at all, so the lookup fails instead of
    // selecting it.
    let through_class = world.resolve(
        method(b"p/StaticOnlyUser", b"m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Other"),
    );
    assert_eq!(
        through_class.state,
        Some(ResolutionState::Missing),
        "an empty maximally-specific set is a lookup failure: {:?}",
        through_class.diagnostics
    );
    assert!(through_class.resolved.is_none());
    assert!(
        through_class.diagnostics.is_empty(),
        "a static declaration stays out of the resolution entirely: {:?}",
        through_class.diagnostics
    );
    assert_eq!(
        read_reasons(&through_class),
        vec![
            ("app".to_string(), ReadReason::MemberOwner),
            ("app".to_string(), ReadReason::ParentChain),
            ("app".to_string(), ReadReason::HierarchyClosure),
        ],
        "the interface really was searched; it just holds no candidate"
    );
    assert_reads_within_attempts(&through_class);
    assert_no_body_read(&through_class);
}

#[test]
fn a_private_interface_declaration_is_ignored_by_the_superinterface_step() {
    // Two shapes of the same rule (JVMS: private superinterface methods are ignored by
    // resolution): an incomparable private declaration next to a `default` must not conflict
    // with it, and a private declaration in a subinterface must not hide its parent's default.
    // The reader does not gate a member's flags on the class file version, so the private
    // interface declarations are the 53+ shape carried at this fixture's major version; the
    // resolution rules under test are version independent.
    let world = World::single();

    let incomparable = world.resolve(
        method(b"p/PrivateConflictUser", b"m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Other"),
    );
    assert_eq!(
        incomparable.state,
        Some(ResolutionState::Resolved),
        "a private declaration is not a second non-abstract candidate: {:?}",
        incomparable.diagnostics
    );
    assert!(
        incomparable.diagnostics.is_empty(),
        "{:?}",
        incomparable.diagnostics
    );
    assert_eq!(
        resolved_of(&incomparable).member,
        method(b"i/Def", b"m", b"()V")
    );

    let overridden = world.resolve(
        method(b"p/PrivateSubUser", b"m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Other"),
    );
    assert_eq!(
        overridden.state,
        Some(ResolutionState::Resolved),
        "{:?}",
        overridden.diagnostics
    );
    assert!(
        overridden.diagnostics.is_empty(),
        "{:?}",
        overridden.diagnostics
    );
    assert_eq!(
        resolved_of(&overridden).member,
        method(b"i/Def", b"m", b"()V"),
        "the private declaration of the subinterface hides nothing: the parent default is the \
         maximally-specific method"
    );
    assert_reads_within_attempts(&overridden);
    assert_no_body_read(&overridden);
}

#[test]
fn a_named_static_interface_declaration_still_resolves_and_still_breaks_the_kind_rule() {
    // The superinterface step filters `static`/`private` declarations, but the owner's own
    // declaration is what a reference *names*: `invokestatic` resolves it, and
    // `invokeinterface` naming the same static method is rejected by the invocation-kind rules.
    let world = World::single();
    let target = method(b"i/StaticM", b"m", b"()V");

    let named_static = world.resolve(
        target.clone(),
        ReferenceUse::InvokeStatic,
        world.caller(b"p/Other"),
    );
    assert_eq!(named_static.state, Some(ResolutionState::Resolved));
    assert!(
        named_static.diagnostics.is_empty(),
        "a static method named by `invokestatic` needs no rule: {:?}",
        named_static.diagnostics
    );
    assert_eq!(resolved_of(&named_static).member, target);
    assert_eq!(class_headers(&named_static), 1);

    let wrong_kind = world.resolve(
        target.clone(),
        ReferenceUse::InvokeInterface,
        world.caller(b"p/Other"),
    );
    assert_eq!(
        wrong_kind.state,
        Some(ResolutionState::IncompatibleClassChange),
        "the interface owner is a legal owner for `invokeinterface`; the static declaration is \
         what the kind rule rejects"
    );
    assert!(wrong_kind.resolved.is_none());
    assert_eq!(
        diagnostic_codes(&wrong_kind),
        vec!["resolution_kind_mismatch"]
    );
    assert!(
        diagnostic_of(&wrong_kind, "resolution_kind_mismatch")
            .message
            .contains("static"),
        "the rejection names the static declaration: {}",
        diagnostic_of(&wrong_kind, "resolution_kind_mismatch").message
    );
    assert_reads_within_attempts(&wrong_kind);
    assert_no_body_read(&wrong_kind);
}

#[test]
fn the_first_declared_direct_superinterface_wins() {
    let world = World::single();

    // A field: JVMS 5.4.3.2 searches the direct superinterfaces in declaration order, so the
    // first one that declares the field decides and the second is never read.
    let field_report = world.resolve(
        field(b"p/TwoIfaces", b"dup_f", b"I"),
        ReferenceUse::FieldRead,
        world.caller(b"p/TwoIfaces"),
    );
    assert_eq!(field_report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        resolved_of(&field_report).member,
        field(b"i/FieldA", b"dup_f", b"I"),
        "`interfaces` is an ordered list: `i/FieldA` is declared before `i/FieldB`"
    );
    assert_eq!(
        read_reasons(&field_report),
        vec![
            ("app".to_string(), ReadReason::MemberOwner),
            ("app".to_string(), ReadReason::HierarchyClosure),
        ],
        "the second superinterface is not read at all once the first one declares the field"
    );
    assert!(
        field_report
            .reads
            .iter()
            .all(|read| read.definition != world.definition(b"i/FieldB")),
        "{:?}",
        field_report.reads
    );
    assert_reads_within_attempts(&field_report);
    assert_no_body_read(&field_report);

    // A method whose candidates are all abstract: the JVMS leaves the choice arbitrary, and this
    // slice's deterministic pick is the first candidate in expansion order, which is the
    // declaration order of `interfaces`.
    let method_report = world.resolve(
        method(b"p/TwoMethodIfaces", b"m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/TwoMethodIfaces"),
    );
    assert_eq!(method_report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        resolved_of(&method_report).member,
        method(b"i/MethodA", b"m", b"()V"),
        "the abstract-only choice is the first declaration in expansion order, so it follows the \
         declaration order of the direct superinterfaces"
    );
    assert_eq!(
        diagnostic_codes(&method_report),
        vec!["resolution_method_is_abstract"]
    );
    assert_reads_within_attempts(&method_report);
    assert_no_body_read(&method_report);
}

#[test]
fn a_signature_polymorphic_method_resolves_by_name_only() {
    let world = World::single();
    let call_site = method(
        b"java/lang/invoke/MethodHandle",
        b"invokeExact",
        b"(Ljava/lang/String;)V",
    );
    let report = world.resolve(
        call_site.clone(),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Other"),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_signature_polymorphic"]
    );
    assert_eq!(
        diagnostic_of(&report, "resolution_signature_polymorphic").severity,
        DiagnosticSeverity::Warning
    );
    assert_eq!(
        report.target, call_site,
        "the call site's descriptor stays visible in the target"
    );
    let member = &resolved_of(&report).member;
    assert_eq!(owner_of(member), b"java/lang/invoke/MethodHandle");
    assert_eq!(
        descriptor_of(member),
        b"([Ljava/lang/Object;)Ljava/lang/Object;",
        "the resolved member is the declaration, with its own descriptor"
    );
    assert_ne!(
        descriptor_of(member),
        descriptor_of(&call_site),
        "both descriptors are visible and they differ by rule"
    );

    // The invocation-kind rules still apply to a signature-polymorphic declaration, and the
    // note about the two descriptors belongs to the resolution: a rejection carries the
    // rejection alone.
    let refused = world.resolve(
        call_site.clone(),
        ReferenceUse::InvokeStatic,
        world.caller(b"p/Other"),
    );
    assert_eq!(
        refused.state,
        Some(ResolutionState::IncompatibleClassChange)
    );
    assert_eq!(diagnostic_codes(&refused), vec!["resolution_kind_mismatch"]);

    // The branch belongs to `java/lang/invoke/MethodHandle`, not to the name: another class's
    // `invoke` is still matched by name and descriptor.
    let other = world.resolve(
        method(b"p/Other", b"invoke", b"(II)I"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Other"),
    );
    assert_eq!(other.state, Some(ResolutionState::Missing));
    assert!(other.diagnostics.is_empty(), "{:?}", other.diagnostics);
}

#[test]
fn the_access_matrix_follows_jvms_5_4_4() {
    let world = World::single();

    // private: accessible to the declaring class itself, denied to another class.
    let own = world.resolve(
        method(b"p/Base", b"priv_m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Base"),
    );
    assert_eq!(own.state, Some(ResolutionState::Resolved));
    assert!(own.diagnostics.is_empty(), "{:?}", own.diagnostics);
    assert_eq!(
        class_headers(&own),
        1,
        "the caller's class is the declaring class, so the caller read is the binding the owner \
         search already read: one binding, one attempt"
    );

    let denied = world.resolve(
        method(b"p/Base", b"priv_m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Other"),
    );
    assert_eq!(denied.state, Some(ResolutionState::Inaccessible));
    assert!(denied.resolved.is_none());
    assert_eq!(diagnostic_codes(&denied), vec!["resolution_access_denied"]);
    assert_eq!(
        diagnostic_of(&denied, "resolution_access_denied").severity,
        DiagnosticSeverity::Error
    );
    assert_eq!(
        read_reasons(&denied),
        vec![
            ("app".to_string(), ReadReason::MemberOwner),
            ("app".to_string(), ReadReason::MemberOwner),
        ],
        "the declaring class and the caller's class are both member-owner reads"
    );
    // Two attempts: the declaring class read as the member's owner, and the caller's class read
    // by identity for the access rules. The binding check that read performs resolves the name
    // the caller's header declares (`p/Other`) in `app`'s own order — one further search — but the
    // position that holds exactly that definition is answered from the facts the identity read
    // just produced, so one binding is not read (or charged) twice in one request. The read
    // *record* set is the two bindings as before; a definition the loader does not bind is
    // refused instead of being stamped with the loader that claims it.
    assert_eq!(class_headers(&denied), 2);
    assert_reads_within_attempts(&denied);
    assert_no_body_read(&denied);

    // Package private: same package passes, another package is denied.
    let same_package = world.resolve(
        field(b"p/Base", b"pkg_f", b"I"),
        ReferenceUse::FieldRead,
        world.caller(b"p/Other"),
    );
    assert_eq!(same_package.state, Some(ResolutionState::Resolved));
    assert!(same_package.diagnostics.is_empty());

    let other_package = world.resolve(
        field(b"q/Base", b"pkg_f", b"I"),
        ReferenceUse::FieldRead,
        world.caller(b"p/Other"),
    );
    assert_eq!(other_package.state, Some(ResolutionState::Inaccessible));
    assert_eq!(
        diagnostic_codes(&other_package),
        vec!["resolution_access_denied"]
    );

    // protected: a subtype in another package passes, an unrelated class in another package is
    // denied.
    let subtype = world.resolve(
        field(b"q/Base", b"prot_f", b"I"),
        ReferenceUse::FieldRead,
        world.caller(b"p/Sub"),
    );
    assert_eq!(subtype.state, Some(ResolutionState::Resolved));
    assert!(subtype.diagnostics.is_empty(), "{:?}", subtype.diagnostics);
    // Two attempts, two *records*: the subtype walk finds the declaring class in the memo instead
    // of reading it a second time, and the caller's class is the second read — the 0.1 binding
    // check resolves the caller's own name (`p/Sub`) in the caller's order against that read's
    // facts, so it costs no third attempt. `read_reasons` below is the deduped record set.
    assert_eq!(class_headers(&subtype), 2);
    assert_eq!(
        read_reasons(&subtype),
        vec![
            ("app".to_string(), ReadReason::MemberOwner),
            ("app".to_string(), ReadReason::MemberOwner),
        ],
        "the binding check is answered from the caller read it verifies, so it adds no second record"
    );
    assert_reads_within_attempts(&subtype);
    assert_no_body_read(&subtype);

    let unrelated = world.resolve(
        field(b"q/Base", b"prot_f", b"I"),
        ReferenceUse::FieldRead,
        world.caller(b"p/Other"),
    );
    assert_eq!(unrelated.state, Some(ResolutionState::Inaccessible));
    assert_eq!(
        diagnostic_codes(&unrelated),
        vec!["resolution_access_denied"]
    );

    // The subtype walk follows the chain, not only the caller's direct supertype.
    let deep = world.resolve(
        field(b"q/Base", b"prot_f", b"I"),
        ReferenceUse::FieldRead,
        world.caller(b"p/DeepSub"),
    );
    assert_eq!(deep.state, Some(ResolutionState::Resolved));
    assert!(deep.diagnostics.is_empty(), "{:?}", deep.diagnostics);
    assert_eq!(
        read_reasons(&deep),
        vec![
            // The declaring class the reference names.
            ("app".to_string(), ReadReason::MemberOwner),
            // The caller's class, which the access rules read by identity.
            ("app".to_string(), ReadReason::MemberOwner),
            // `p/DeepSub extends p/Sub extends q/Base`: the subtype walk's own steps are
            // hierarchy edges too, so a superclass step there is a parent-chain read.
            ("app".to_string(), ReadReason::ParentChain),
        ],
        "the caller's own hierarchy keeps the edge reasons as well"
    );

    // public: accessible across packages, and the caller's class is not read for it.
    let public = world.resolve(
        field(b"q/Base", b"pub_f", b"I"),
        ReferenceUse::FieldRead,
        world.caller(b"p/Other"),
    );
    assert_eq!(public.state, Some(ResolutionState::Resolved));
    assert!(public.diagnostics.is_empty());
    assert_eq!(
        read_reasons(&public),
        vec![("app".to_string(), ReadReason::MemberOwner)],
        "a public member is accessible to every caller, so its class is the only read"
    );
    assert_eq!(class_headers(&public), 1);
}

#[test]
fn an_unknown_caller_class_leaves_the_access_rules_unchecked() {
    let world = World::single();
    let report = world.resolve(
        method(b"p/Base", b"priv_m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.abstract_caller(),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_access_not_checked"],
        "the resolution says what it did not check instead of claiming a pass"
    );
    assert_eq!(
        diagnostic_of(&report, "resolution_access_not_checked").severity,
        DiagnosticSeverity::Warning
    );
    assert_eq!(
        resolved_of(&report).member,
        method(b"p/Base", b"priv_m", b"()V")
    );
    assert_eq!(
        class_headers(&report),
        1,
        "no caller class was read, because none is known"
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn an_unreadable_caller_hierarchy_leaves_the_access_rules_unchecked() {
    let world = World::single();
    let report = world.resolve(
        field(b"q/Base", b"prot_f", b"I"),
        ReferenceUse::FieldRead,
        world.caller(b"p/OrphanSub"),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        diagnostic_codes(&report),
        vec![
            "resolution_hierarchy_missing",
            "resolution_access_not_checked",
        ],
        "the subtype test could not be completed, so access is neither granted nor denied"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn the_same_package_in_another_loader_is_a_different_runtime_package() {
    // The caller is defined by the `app` loader and the declaring class by its parent: the
    // package *name* is the same and the run-time package is not, so no package-private rule
    // passes between them (JVMS 5.3, 5.4.4).
    let two = World::two_loaders();
    let report = two.resolve(
        field(b"p/LoadBase", b"pkg_f", b"I"),
        ReferenceUse::FieldRead,
        two.caller_in(b"p/LoadCaller", loader("app")),
    );

    assert_eq!(report.state, Some(ResolutionState::Inaccessible));
    assert_eq!(diagnostic_codes(&report), vec!["resolution_access_denied"]);
    assert_eq!(
        report.reads[0].loader,
        loader("platform"),
        "the declaration comes from the loader the search order selected: {:?}",
        report.reads
    );

    // Control: the same two names under one loader, where the package really is the same.
    let single = World::single();
    let allowed = single.resolve(
        field(b"p/LoadBase", b"pkg_f", b"I"),
        ReferenceUse::FieldRead,
        single.caller(b"p/LoadCaller"),
    );
    assert_eq!(allowed.state, Some(ResolutionState::Resolved));
    assert!(allowed.diagnostics.is_empty(), "{:?}", allowed.diagnostics);
}

#[test]
fn an_ambiguous_owner_class_publishes_its_candidates() {
    let snapshot = open(zip_of(&[
        (
            entry(b"p/Ambig"),
            Class::new(b"p/Ambig").method(b"m", b"()V", PUBLIC).build(),
        ),
        (
            entry(b"p/Ambig"),
            Class::new(b"p/Ambig").field(b"f", b"I", PUBLIC).build(),
        ),
    ]));
    let app = domain(&loader("app"), None, vec![snapshot_root(&snapshot)]);
    let environment = environment(&snapshot, app.clone(), vec![app]);
    let world = World {
        content: vec![snapshot],
        environment,
        app: loader("app"),
    };
    let target = field(b"p/Ambig", b"f", b"I");
    let report = world.resolve(
        target.clone(),
        ReferenceUse::FieldRead,
        world.abstract_caller(),
    );

    assert_eq!(report.state, Some(ResolutionState::Ambiguous));
    assert!(report.resolved.is_none());
    assert_eq!(report.candidates.len(), 2, "both positions are published");
    assert_ne!(
        report.candidates[0].definition, report.candidates[1].definition,
        "the two definitions keep their own origins"
    );
    for candidate in &report.candidates {
        assert_eq!(
            candidate.member, target,
            "an ambiguous owner publishes the raw reference, not an invented declaration"
        );
    }
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn a_member_search_stops_at_its_header_budget() {
    let world = World::single();
    let report = world.resolve_with(
        method(b"p/Inherit", b"pub_m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.abstract_caller(),
        Limits {
            class_headers: 1,
            ..limits()
        },
    );

    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert_eq!(report.state, Some(ResolutionState::BudgetExceeded));
    assert!(report.resolved.is_none());
    assert_eq!(
        diagnostic_codes(&report),
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
    assert_eq!(class_headers(&report), 1);
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert_eq!(
        report.coverage.runtime_resolution.scanned[0].end, 1,
        "the search that decided is the covered prefix"
    );
    assert_eq!(
        report
            .coverage
            .runtime_resolution
            .skipped
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        vec![(1, 2)],
        "the position the refused search never examined stays declared as unfinished"
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn a_member_search_stops_at_its_dependency_depth() {
    let world = World::single();
    let report = world.resolve_with(
        field(b"p/Mixed", b"absent_f", b"I"),
        ReferenceUse::FieldRead,
        world.abstract_caller(),
        Limits {
            dependency_depth: 1,
            ..limits()
        },
    );

    assert_eq!(report.state, Some(ResolutionState::BudgetExceeded));
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
        report
            .reads
            .iter()
            .all(|read| read.definition != world.definition(b"java/lang/Object")),
        "the depth stop happens before the layer is read: {:?}",
        report.reads
    );
}

#[test]
fn a_declaration_of_the_owner_itself_costs_no_dependency_depth() {
    // The member's own owner is step 0 of the search, not a step up a hierarchy, so a request
    // that declares no dependency depth at all still reads it.
    let world = World::single();
    let own = world.resolve_with(
        method(b"p/Base", b"pub_m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.abstract_caller(),
        Limits {
            dependency_depth: 0,
            ..limits()
        },
    );
    assert_eq!(
        own.state,
        Some(ResolutionState::Resolved),
        "the owner is read at depth 0, which the limit allows: {:?}",
        own.diagnostics
    );
    assert_eq!(
        resolved_of(&own).member,
        method(b"p/Base", b"pub_m", b"()V")
    );
    assert_eq!(usage_of(&own.execution).dependency_depth, 0);

    // The contrast that makes the first half a fact about the depth rather than about the
    // fixture being easy: the same zero limit stops the search one step up the chain.
    let inherited = world.resolve_with(
        method(b"p/Inherit", b"pub_m", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.abstract_caller(),
        Limits {
            dependency_depth: 0,
            ..limits()
        },
    );
    assert_eq!(inherited.state, Some(ResolutionState::BudgetExceeded));
    assert!(matches!(
        inherited.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::DependencyDepth
            },
            ..
        }
    ));
}

#[test]
fn an_invokedynamic_reference_is_not_held_to_the_invocation_kind_rules() {
    // `CONSTANT_InvokeDynamic` carries no method reference to constrain, so the owner is only
    // the search start: neither the static rule nor the owner-kind rule applies.
    let world = World::single();
    for (target, reason) in [
        (
            method(b"p/Base", b"stat_m", b"()V"),
            "a static class method needs no rule under `invoke_dynamic`",
        ),
        (
            method(b"i/StaticM", b"m", b"()V"),
            "an interface owner is a legal search start for `invoke_dynamic`",
        ),
    ] {
        let report = world.resolve(
            target.clone(),
            ReferenceUse::InvokeDynamic,
            world.abstract_caller(),
        );
        assert_eq!(
            report.state,
            Some(ResolutionState::Resolved),
            "{reason}: {:?}",
            report.diagnostics
        );
        assert!(
            report.diagnostics.is_empty(),
            "{reason}: {:?}",
            report.diagnostics
        );
        assert_eq!(resolved_of(&report).member, target, "{reason}");
        assert_reads_within_attempts(&report);
        assert_no_body_read(&report);
    }
}

#[test]
fn a_pre_cancelled_member_request_reports_cancelled_and_reads_nothing() {
    let world = World::single();
    let target = method(b"p/Base", b"pub_m", b"()V");
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let report = world.resolve_under(
        target.clone(),
        ReferenceUse::InvokeVirtual,
        world.abstract_caller(),
        &mut budget,
    );

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
    assert!(report.resolved.is_none());
    assert_eq!(report.target, target, "the raw reference is preserved");
    assert!(
        report.reads.is_empty(),
        "the refusal happened before the first read: {:?}",
        report.reads
    );
    assert_eq!(class_headers(&report), 0);
    assert_eq!(diagnostic_codes(&report), vec!["cancelled"]);
    // The stop happened before any effective order was derived, so there is no search position
    // to declare as covered or as skipped: the plane is partial without inventing a range.
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert!(report.coverage.runtime_resolution.scanned.is_empty());
    assert!(report.coverage.runtime_resolution.skipped.is_empty());
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

#[test]
fn a_member_resolution_reads_no_code_byte_of_a_class_that_has_a_body() {
    let world = World::single();
    let report = world.resolve(
        method(b"p/Body", b"run", b"()V"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Body"),
    );
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        code_bytes(&report),
        0,
        "the header search read the declaration and no instruction: {report:?}"
    );

    // Control: the fixture really carries a body, and the P1 body path charges code bytes for
    // it, so the zero above is a fact about the member search and not about the fixture.
    let mut listing_budget = Budget::new(limits());
    let listing = Engine::new()
        .enumerate(&world.content[0], &mut listing_budget)
        .expect("the fixture archive lists");
    let body_entry = listing
        .entries
        .iter()
        .find(|listed| listed.id.raw_name.0 == entry(b"p/Body"))
        .expect("the fixture holds the class with a body");
    let mut body_budget = Budget::new(limits());
    let body = Engine::new()
        .inspect_method_bytecode(
            &world.content[0],
            ClassTarget::Entry(body_entry),
            MethodSelector {
                name: JvmBytes(b"run".to_vec()),
                descriptor: JvmBytes(b"()V".to_vec()),
            },
            &mut body_budget,
        )
        .expect("the fixture body is readable");
    assert!(
        !body.inspection.instructions.is_empty(),
        "the control really decoded instructions"
    );
    assert!(
        body_budget.usage().code_bytes > 0,
        "the body path charges code bytes on this fixture, otherwise the assertion above is \
         vacuous: {}",
        body_budget.usage().code_bytes
    );
}

#[test]
fn an_array_owner_is_unsupported() {
    let world = World::single();
    let report = world.resolve(
        method(b"[I", b"clone", b"()Ljava/lang/Object;"),
        ReferenceUse::InvokeVirtual,
        world.caller(b"p/Other"),
    );

    assert_eq!(report.state, Some(ResolutionState::UnsupportedPolicy));
    assert!(report.resolved.is_none());
    assert_eq!(diagnostic_codes(&report), vec!["resolution_array_owner"]);
    assert_eq!(
        diagnostic_of(&report, "resolution_array_owner").severity,
        DiagnosticSeverity::Error
    );
    assert!(
        diagnostic_of(&report, "resolution_array_owner")
            .message
            .contains("[I"),
        "the diagnostic names the array owner"
    );
    assert_eq!(
        report.target,
        method(b"[I", b"clone", b"()Ljava/lang/Object;"),
        "the raw reference is kept"
    );
    assert!(
        report.reads.is_empty(),
        "an owner kind this slice does not resolve reads no class: {:?}",
        report.reads
    );
    assert_eq!(class_headers(&report), 0);
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial,
        "the resolution was requested and no range of it was searched"
    );
}

// ---------------------------------------------------------------------------
// 0.1: a successor symbol request is resolved from its own defining loader
// ---------------------------------------------------------------------------

/// The R1 counterexample of the 2026-09-18 review, kept as a permanent regression.
///
/// `child` is ChildFirst with its own root and `parent` as its parent loader; `parent` is
/// ParentFirst with its own root. The child holds one `p/Base`, the parent holds `p/Owner extends
/// p/Base` **and a different `p/Base`**, and both bases declare `public int f`. JVMS 5.4.3.1
/// resolves the names `p/Owner` holds from `p/Owner`'s own defining loader, so the field is
/// declared by the *parent's* `p/Base`: the child's same-named class must not answer for it, even
/// though the request itself starts in the child.
///
/// Before 0.1 every demand of the request searched from `runtime.load_domain.loader` and the walk
/// carried only names, so the public entry returned the **child's** base and reported `Resolved`,
/// `Complete` and no diagnostic. The assertions therefore name the physical definition, not the
/// owner string: an equal name is exactly what this bug produced.
#[test]
fn review_parent_defined_owner_resolves_its_base_from_the_parent_loader() {
    let child_snapshot = open(zip_of(&[(
        entry(b"p/Base"),
        // The child's own base: same name, same declared field, its own bytes.
        Class::new(b"p/Base")
            .field(b"f", b"I", PUBLIC)
            .field(b"child_only_marker", b"I", PUBLIC)
            .build(),
    )]));
    let parent_snapshot = open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"p/Owner"),
            Class::new(b"p/Owner").super_class(b"p/Base").build(),
        ),
        (
            entry(b"p/Base"),
            Class::new(b"p/Base").field(b"f", b"I", PUBLIC).build(),
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
    let world = World {
        content: vec![child_snapshot.clone(), parent_snapshot.clone()],
        environment,
        app: loader("child"),
    };

    // The two physical definitions this fixture can answer with, each read from the only loader
    // that actually holds it: comparing against these is what a name cannot fake.
    let parent_base = single_loader_world(&parent_snapshot).definition(b"p/Base");
    let child_base = single_loader_world(&child_snapshot).definition(b"p/Base");
    assert_ne!(
        parent_base, child_base,
        "the fixture really holds two different `p/Base` definitions"
    );

    let target = field(b"p/Owner", b"f", b"I");
    let report = world.resolve(target, ReferenceUse::FieldRead, world.abstract_caller());

    let resolved = resolved_of(&report);
    assert_eq!(report.state, Some(ResolutionState::Resolved), "{report:?}");
    assert_eq!(
        resolved.loader,
        loader("parent"),
        "the declaration belongs to the loader that defines `p/Owner`"
    );
    assert_eq!(
        resolved.definition, parent_base,
        "the resolution must select the parent's `p/Base`, the definition its own loader holds"
    );
    assert_ne!(
        resolved.definition, child_base,
        "the child's same-named class must never answer for a delegated class's superclass"
    );
    assert_eq!(
        resolved.definition.snapshot(),
        parent_snapshot.id(),
        "and it comes from the parent's snapshot, physically"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema,
        "the hierarchy was readable, so the answer really is complete"
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

/// One loader, one root: the world that names what a given snapshot's `p/Base` physically is.
fn single_loader_world(snapshot: &ArtifactSnapshot) -> World {
    let app = domain(&loader("only"), None, vec![snapshot_root(snapshot)]);
    World {
        content: vec![snapshot.clone()],
        environment: environment(snapshot, app.clone(), vec![app]),
        app: loader("only"),
    }
}

// ---------------------------------------------------------------------------
// 0.1: one name, two loaders — the request memo keeps the two searches apart
// ---------------------------------------------------------------------------

/// One request asks for `p/Base` in two different loaders, and each answer is its own.
///
/// `child` is ChildFirst with its own root and `parent` as its parent loader; `parent` is
/// ParentFirst with its own root. The child holds `p/Mid extends p/Base` and **its own** `p/Base`;
/// the parent holds `p/Hook extends p/Base` and its own `p/Base`, and only the parent's base
/// declares `f`. So the member search has to demand the *same name* twice in one request with two
/// different initiating loaders: once from `child` for `p/Mid`'s superclass (the child's base),
/// and once from `parent` after the walk reaches `p/Hook`, which only the parent provides.
///
/// The answer is the parent's definition of `p/Base`, and the assertions name it as a
/// **definition**: the member's owner string is `p/Base` in both loaders, so a name cannot
/// distinguish the two and only the physical pair can. A memo keyed by name alone would answer the
/// second demand with the first decision (the child's base), the walk would lose the declaration
/// and the request would report `Missing` — which is the bug this test exists to catch.
#[test]
fn one_request_searches_one_name_in_two_loaders_and_keeps_them_apart() {
    let child_snapshot = open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"p/Mid"),
            Class::new(b"p/Mid").super_class(b"p/Base").build(),
        ),
        // The child's own base: no `f`, and a superclass only the parent provides (which is the
        // second, different initiating loader of the same name).
        (
            entry(b"p/Base"),
            Class::new(b"p/Base").super_class(b"p/Hook").build(),
        ),
    ]));
    let parent_snapshot = open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"p/Hook"),
            Class::new(b"p/Hook").super_class(b"p/Base").build(),
        ),
        (
            entry(b"p/Base"),
            Class::new(b"p/Base").field(b"f", b"I", PUBLIC).build(),
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
    let world = World {
        content: vec![child_snapshot.clone(), parent_snapshot.clone()],
        environment,
        app: loader("child"),
    };

    let child_base = single_loader_world(&child_snapshot).definition(b"p/Base");
    let parent_base = single_loader_world(&parent_snapshot).definition(b"p/Base");
    let hook = single_loader_world(&parent_snapshot).definition(b"p/Hook");
    let mid = single_loader_world(&child_snapshot).definition(b"p/Mid");
    assert_ne!(
        child_base, parent_base,
        "the fixture really holds two different `p/Base` definitions"
    );

    let report = world.resolve(
        field(b"p/Mid", b"f", b"I"),
        ReferenceUse::FieldRead,
        world.abstract_caller(),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved), "{report:?}");
    let resolved = resolved_of(&report);
    assert_eq!(
        resolved.member,
        field(b"p/Base", b"f", b"I"),
        "both bases are named `p/Base`, so the declaration's owner string is the same either way"
    );
    assert_eq!(
        resolved.loader, parent_loader,
        "the declaration belongs to the loader that defines `p/Hook`, the class that declares `f`"
    );
    assert_eq!(
        resolved.definition, parent_base,
        "the walk's last step demanded `p/Base` from the parent and selected its definition"
    );
    assert_ne!(
        resolved.definition, child_base,
        "the child's same-named base, decided earlier in the same request, is a different class"
    );
    // The two searches of the one name, each in its own loader, in the order the walk made them:
    // the child's base for `p/Mid`'s superclass and the parent's base for `p/Hook`'s. A memo that
    // remembered the name instead of the `(initiating loader, name)` key could not publish both.
    assert_eq!(
        report.reads,
        vec![
            HeaderRead {
                loader: loader("child"),
                definition: mid,
                reason: ReadReason::MemberOwner,
            },
            HeaderRead {
                loader: loader("child"),
                definition: child_base.clone(),
                reason: ReadReason::ParentChain,
            },
            HeaderRead {
                loader: loader("parent"),
                definition: hook,
                reason: ReadReason::ParentChain,
            },
            HeaderRead {
                loader: loader("parent"),
                definition: parent_base.clone(),
                reason: ReadReason::ParentChain,
            },
        ]
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

// ---------------------------------------------------------------------------
// 0.1: the binding check refuses a definition its loader does not select
// ---------------------------------------------------------------------------

/// The `resolution_definition_unbound` producer: a caller definition the loader shadows.
///
/// `app` declares two ordered roots, and the first one holds a class named `p/Other` that is not
/// the one the fixture world holds. The request names the *world's* `p/Other` as the class
/// declaring its use site, so the definition is readable, describes the bytes at its own
/// coordinate, and still is not a class `app` has: the loader's own order selects the earlier
/// position instead. That is what the 0.1 binding check refuses, and it refuses it as a stop —
/// no access rule is decided on a class the loader would never load, and the physical read that
/// established the mismatch stays published.
#[test]
fn a_caller_definition_the_declared_loader_does_not_bind_is_a_stop() {
    let world_snapshot = open(zip_of(&entries()));
    let shadow_snapshot = open(zip_of(&[(
        entry(b"p/Other"),
        Class::new(b"p/Other")
            .field(b"marker", b"I", PUBLIC)
            .build(),
    )]));
    let app = domain(
        &loader("app"),
        None,
        vec![
            snapshot_root(&shadow_snapshot),
            snapshot_root(&world_snapshot),
        ],
    );
    let environment = environment(&world_snapshot, app.clone(), vec![app]);
    let world = World {
        content: vec![world_snapshot.clone(), shadow_snapshot.clone()],
        environment,
        app: loader("app"),
    };
    // The two definitions of that one name, each read from the loader that really holds it.
    let claimed = single_loader_world(&world_snapshot).definition(b"p/Other");
    let shadow = single_loader_world(&shadow_snapshot).definition(b"p/Other");
    let declaring = single_loader_world(&world_snapshot).definition(b"p/Base");
    assert_ne!(
        claimed, shadow,
        "the fixture holds two `p/Other` definitions"
    );

    let mut caller = world.abstract_caller();
    caller.enclosing = Some(PhysicalMethodId {
        owner: claimed.clone(),
        name: JvmBytes(b"bench".to_vec()),
        descriptor: JvmBytes(b"()V".to_vec()),
    });
    let report = world.resolve(
        method(b"p/Base", b"priv_m", b"()V"),
        ReferenceUse::InvokeVirtual,
        caller,
    );

    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert!(
        report.state.is_none(),
        "a definition the loader does not bind is a stop, not a semantic decision: {:?}",
        report.state
    );
    assert!(report.resolved.is_none());
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_definition_unbound"]
    );
    let diagnostic = diagnostic_of(&report, "resolution_definition_unbound");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert!(
        diagnostic.message.contains("`app`"),
        "the diagnostic names the loader that does not bind it: {}",
        diagnostic.message
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Failed {
            reason: TerminationReason::Error { ref code },
            ..
        } if code == "resolution_definition_unbound"
    ));
    // The physical facts are not withdrawn with the runtime semantics built on them: the read of
    // the claimed definition is still recorded next to the read that refuted it, and the claimed
    // definition really is the definition of the world's own class bytes (see `claimed` above,
    // which the world's single-loader view still resolves).
    assert_eq!(
        report.reads,
        vec![
            HeaderRead {
                loader: loader("app"),
                definition: declaring,
                reason: ReadReason::MemberOwner,
            },
            HeaderRead {
                loader: loader("app"),
                definition: claimed.clone(),
                reason: ReadReason::MemberOwner,
            },
            HeaderRead {
                loader: loader("app"),
                definition: shadow,
                reason: ReadReason::MemberOwner,
            },
        ]
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

// ---------------------------------------------------------------------------
// 0.1: another snapshot is a dependency, not a mismatch
// ---------------------------------------------------------------------------

/// A claimed definition of **another** snapshot passes the binding check when its loader selects
/// it: the contract compares the loader's decision, not two snapshot identities.
///
/// The caller is defined by `dep`, whose own root holds the `dep` snapshot; the request's physical
/// view names the `main` snapshot, which is the very snapshot the declaring class comes from and
/// the dependency the `dep` snapshot's caller subclasses. The caller's class is read by identity
/// (its name was never demanded before), so the check really runs and really has to accept a
/// definition outside the request's own physical snapshot — a check that had degenerated into
/// "the snapshots must be equal" would stop here instead.
#[test]
fn a_caller_definition_of_another_snapshot_passes_when_its_loader_selects_it() {
    let main_snapshot = open(zip_of(&[
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"p/Target"),
            Class::new(b"p/Target")
                .field(b"prot_f", b"I", PROTECTED)
                .build(),
        ),
    ]));
    let dep_snapshot = open(zip_of(&[(
        entry(b"p/Caller"),
        Class::new(b"p/Caller").super_class(b"p/Target").build(),
    )]));
    let main_loader = loader("main");
    let dep = {
        let mut dep = domain(
            &loader("dep"),
            Some(main_loader.clone()),
            vec![snapshot_root(&dep_snapshot)],
        );
        // The caller's own root is searched before the parent's, so the definition this request
        // claims is the one on its own side of the delegation.
        dep.delegation = DelegationPolicy::ChildFirst;
        dep
    };
    let main = domain(&main_loader, None, vec![snapshot_root(&main_snapshot)]);
    let environment = environment(&main_snapshot, dep.clone(), vec![dep.clone(), main]);
    let world = World {
        content: vec![main_snapshot.clone(), dep_snapshot.clone()],
        environment,
        app: loader("dep"),
    };
    let target = single_loader_world(&main_snapshot).definition(b"p/Target");
    let caller_definition = single_loader_world(&dep_snapshot).definition(b"p/Caller");
    let physical = world.environment.runtime.physical.snapshot.clone();
    assert_ne!(
        caller_definition.snapshot(),
        &physical,
        "the caller's definition is outside the request's physical snapshot"
    );

    let report = world.resolve(
        field(b"p/Target", b"prot_f", b"I"),
        ReferenceUse::FieldRead,
        world.caller_in(b"p/Caller", loader("dep")),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved), "{report:?}");
    assert!(
        report.diagnostics.is_empty(),
        "the protected member is inherited by the caller's class, and nothing else is reported: \
         {:?}",
        report.diagnostics
    );
    assert_eq!(
        resolved_of(&report).definition,
        target,
        "the member is declared by the `main` snapshot's class"
    );
    assert_eq!(
        resolved_of(&report).loader,
        main_loader,
        "and the loader that provides that definition is the one that declares it"
    );
    let caller_read = report
        .reads
        .iter()
        .find(|read| read.definition == caller_definition)
        .unwrap_or_else(|| panic!("the caller's class was read: {:?}", report.reads));
    assert_eq!(caller_read.loader, loader("dep"));
    assert_eq!(caller_read.reason, ReadReason::MemberOwner);
    assert_eq!(
        class_headers(&report),
        2,
        "the declaring class and the caller's class are the two bindings this request reads: the \
         binding check is answered from the caller's own read"
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

// ---------------------------------------------------------------------------
// 0.1 复核 F1: a memoized binding is reused only when it was checked under its own name
// ---------------------------------------------------------------------------

/// The refusal one report stopped under, as the `(code, message)` pair the two paths compare.
fn stop_of(diagnostics: &[Diagnostic]) -> Vec<(String, String)> {
    diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code.clone(), diagnostic.message.clone()))
        .collect()
}

/// The state of one stage of a method-analysis report.
fn stage(report: &MethodAnalysisReport, stage: AnalysisStage) -> StageState {
    report
        .stages
        .iter()
        .find(|result| result.stage == stage)
        .unwrap_or_else(|| panic!("{stage:?} is scheduled"))
        .state
        .clone()
}

/// The malformed artifact both paths refuse: an entry that declares another name.
///
/// `p/Fake.class` holds bytes whose `this_class` is `p/Real`, and the same root holds a
/// **different** `p/Real.class`. A member request for `p/Fake`'s private field asks for a name no
/// position can bind: the only candidate stored under `p/Fake.class` declares `p/Real`, so the
/// candidate is refused where it was read (`resolution_definition_name_mismatch`) instead of being
/// bound under the requested name — the check F1 found missing from the candidate election. The
/// driver path over the same physical definition reads it by identity, sees its own name `p/Real`
/// resolve to the *other* definition, and refuses the claim (`resolution_definition_unbound`).
/// The two refusals are different facts and both are locatable; what neither path may do is turn
/// either artifact into a binding, and the memo shortcut F1 closed cannot reappear: a resolution
/// is no longer `Found` under a name its header does not declare.
#[test]
fn an_entry_that_declares_another_name_is_refused_at_its_own_candidate() {
    let claimed_bytes = Class::new(b"p/Real")
        .field(b"priv_f", b"I", PRIVATE)
        .method_with_body(b"bench", b"()V", PUBLIC)
        .build();
    let other_bytes = Class::new(b"p/Real").field(b"other", b"I", PUBLIC).build();
    let world = world_of(vec![
        (entry(b"p/Fake"), claimed_bytes.clone()),
        (entry(b"p/Real"), other_bytes.clone()),
    ]);
    // The entry whose path disagrees with its header is not a name any lookup resolves any more, so
    // its physical identity comes from the snapshot's own listing rather than from a search.
    let claimed = archive_definition(&world.content[0], b"p/Fake.class", &claimed_bytes);
    let other = world.definition(b"p/Real");
    assert_ne!(
        claimed, other,
        "the fixture holds two definitions behind the one declared name `p/Real`"
    );
    let driver = PhysicalMethodId {
        owner: claimed.clone(),
        name: JvmBytes(b"bench".to_vec()),
        descriptor: JvmBytes(b"()V".to_vec()),
    };

    // The member path: the request names `p/Fake` as the owner, so the demand for that name reads
    // the one candidate stored under it and stops at the name its bytes declare.
    let report = world.resolve(
        field(b"p/Fake", b"priv_f", b"I"),
        ReferenceUse::FieldRead,
        world.caller_in(b"p/Real", loader("app")),
    );
    assert!(
        report.state.is_none() && report.resolved.is_none(),
        "a candidate that declares another name is a stop, not a binding: {report:?}"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_definition_name_mismatch"]
    );
    let refusal = diagnostic_of(&report, "resolution_definition_name_mismatch");
    assert_eq!(refusal.severity, DiagnosticSeverity::Error);
    assert!(
        refusal.message.contains("`p/Real`")
            && refusal.message.contains("`p/Fake`")
            && refusal.message.contains("\"p/Fake.class\""),
        "the refusal names the name the candidate declares, the name that was requested and the \
         physical candidate it read: {}",
        refusal.message
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Failed {
            reason: TerminationReason::Error { ref code },
            ..
        } if code == "resolution_definition_name_mismatch"
    ));
    // The read really happened and is recorded: a refused candidate keeps the physical facts it was
    // built on, and the search does not continue into a later position.
    let claimed_read = report
        .reads
        .iter()
        .find(|read| read.definition == claimed)
        .unwrap_or_else(|| panic!("the candidate was read: {:?}", report.reads));
    assert_eq!(claimed_read.loader, loader("app"));
    assert_eq!(claimed_read.reason, ReadReason::MemberOwner);
    assert_eq!(
        class_headers(&report),
        1,
        "the first candidate's own header decided the demand: no later position was searched"
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);

    // The driver path over the same physical definition, under the same environment and content.
    let request = MethodAnalysisRequest {
        environment: world.environment.clone(),
        method: driver,
        stages: vec![AnalysisStage::RawCfg],
    };
    let mut budget = Budget::new(limits());
    let analyzed = Engine::new()
        .analyze_method(&world.content, &request, &mut budget)
        .expect("a legal request is answered, not raised");
    assert_eq!(
        stage(&analyzed, AnalysisStage::RawFacts),
        StageState::Failed {
            code: "resolution_definition_unbound".to_string()
        },
        "the fresh path refuses the claim the definition's own name does not select"
    );
    assert_eq!(
        stage(&analyzed, AnalysisStage::RawCfg),
        StageState::NotPerformed
    );
    assert_eq!(analyzed.body, MethodBodyState::NotInspected);

    // The identity read happened and is published: the refused binding keeps the read record of the
    // definition it claims, and the refusal names both the claim and the definition the search
    // selected instead.
    let stop = stop_of(&analyzed.diagnostics);
    assert!(
        stop.iter()
            .any(|(code, message)| code == "resolution_definition_unbound"
                && message.contains("`p/Real`")),
        "the driver refusal is locatable: {stop:?}"
    );
}

// ---------------------------------------------------------------------------
// 0.1: the walk's own root — a delegated self-supertype, and a name that is another node
// ---------------------------------------------------------------------------

/// A root reached through delegation reports its own self-supertype as the illegal cycle it is.
///
/// `app` holds nothing named `p/Self`, so the caller's demand is answered by the parent loader's
/// position: the class the walk starts from is the parent's definition, and the names *its*
/// header holds are searched from the parent. That header declares `super_class = p/Self` — the
/// same name and the same node the class already is — which JVMS 4.7.7 forbids. The repeating
/// edge is demanded under a key the caller's search never used (`(parent, p/Self)`, not
/// `(app, p/Self)`), so the walk reaches its own root the hard way and still has to report it:
/// the branch is refused with `resolution_hierarchy_cycle` and counted unread, exactly as it is
/// when the root belongs to the request's own loader. Reporting has to win over "this node was
/// already expanded" — the root is in that set by definition.
#[test]
fn a_delegated_root_that_declares_itself_as_its_supertype_is_a_cycle() {
    let parent_snapshot = open(zip_of(&[(
        entry(b"p/Self"),
        Class::new(b"p/Self").super_class(b"p/Self").build(),
    )]));
    let app_snapshot = open(zip_of(&[(
        entry(b"p/Other"),
        Class::new(b"p/Other").build(),
    )]));
    let parent_loader = loader("parent");
    let parent = domain(&parent_loader, None, vec![snapshot_root(&parent_snapshot)]);
    let caller = domain(
        &loader("app"),
        Some(parent_loader.clone()),
        vec![snapshot_root(&app_snapshot)],
    );
    let environment = environment(&app_snapshot, caller.clone(), vec![caller, parent]);
    let world = World {
        content: vec![app_snapshot.clone(), parent_snapshot.clone()],
        environment,
        app: loader("app"),
    };
    let root = single_loader_world(&parent_snapshot).definition(b"p/Self");
    assert_ne!(
        root.snapshot(),
        &world.environment.runtime.physical.snapshot,
        "the root really is reached through the delegation and not from the request's snapshot"
    );

    let report = world.resolve(
        field(b"p/Self", b"f", b"I"),
        ReferenceUse::FieldRead,
        world.abstract_caller(),
    );

    assert_eq!(
        report.state,
        Some(ResolutionState::UnresolvedDependency),
        "the root's own repeating edge leaves the hierarchy unread, so the search states what it \
         could not read instead of negating it (A11, 2.2)"
    );
    assert!(report.resolved.is_none());
    assert_eq!(
        report.unresolved_dependencies,
        vec![UnresolvedDependency {
            name: JvmBytes(b"p/Self".to_vec()),
            loader: loader("parent"),
            reason: ReadReason::ParentChain,
            declared_by: Some(JvmBytes(b"p/Self".to_vec())),
            gap: DependencyGap::Cyclic,
        }],
        "the edge is the delegated root's own, so both the searched loader and the declaring \
         class are the parent's"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_hierarchy_cycle"]
    );
    let diagnostic = diagnostic_of(&report, "resolution_hierarchy_cycle");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
    assert!(
        diagnostic.message.contains("`parent`")
            && diagnostic.message.contains("is its own supertype")
            && diagnostic.message.contains("p/Self"),
        "the warning names the order that repeated and the node it repeated: {}",
        diagnostic.message
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial,
        "the refused branch is unread, so the closure is not complete"
    );
    assert_eq!(
        read_reasons(&report),
        vec![("parent".to_string(), ReadReason::MemberOwner)],
        "one binding, one read record, whichever key the search ran under"
    );
    assert_eq!(
        class_headers(&report),
        2,
        "the caller's key and the defining loader's key are two searches of the one binding"
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}

/// The root's name reached again through another loader is another node, and its declaration is
/// the one that resolves.
///
/// `app` is ChildFirst with its own root, so the caller's `p/Root` is **app's** definition, which
/// declares `super_class = p/Mid`; only the parent provides `p/Mid`, so the walk crosses into the
/// parent's order there, and `p/Mid`'s own supertype name — `p/Root` again — is searched from the
/// parent, where it is a **different** definition that declares `f`. A walk that deduplicated its
/// layers by name would refuse this branch as a repeat of its own root and report the member
/// missing; the node the search really reached is the parent's, and the declaration is published
/// from it. The owner string is the same `p/Root` either way, so the physical definition is the
/// only evidence that separates the two nodes.
#[test]
fn a_root_name_reached_again_under_another_loader_is_another_node() {
    let parent_snapshot = open(zip_of(&[
        (
            entry(b"p/Root"),
            Class::root(b"p/Root").field(b"f", b"I", PUBLIC).build(),
        ),
        (
            entry(b"p/Mid"),
            Class::new(b"p/Mid").super_class(b"p/Root").build(),
        ),
    ]));
    let app_snapshot = open(zip_of(&[(
        entry(b"p/Root"),
        Class::new(b"p/Root").super_class(b"p/Mid").build(),
    )]));
    let parent_loader = loader("parent");
    let parent = domain(&parent_loader, None, vec![snapshot_root(&parent_snapshot)]);
    let caller = {
        let mut caller = domain(
            &loader("app"),
            Some(parent_loader.clone()),
            vec![snapshot_root(&app_snapshot)],
        );
        // The caller's own order is searched before the parent's, so the walk starts from app's
        // `p/Root` — the node the branch below must not be confused with.
        caller.delegation = DelegationPolicy::ChildFirst;
        caller
    };
    let environment = environment(&app_snapshot, caller.clone(), vec![caller, parent]);
    let world = World {
        content: vec![app_snapshot.clone(), parent_snapshot.clone()],
        environment,
        app: loader("app"),
    };
    let app_root = single_loader_world(&app_snapshot).definition(b"p/Root");
    let parent_root = single_loader_world(&parent_snapshot).definition(b"p/Root");
    assert_ne!(
        app_root, parent_root,
        "the fixture holds two definitions of the one name `p/Root`"
    );

    let report = world.resolve(
        field(b"p/Root", b"f", b"I"),
        ReferenceUse::FieldRead,
        world.abstract_caller(),
    );

    assert_eq!(report.state, Some(ResolutionState::Resolved), "{report:?}");
    assert!(
        report.diagnostics.is_empty(),
        "nothing is unread, so nothing is reported: {:?}",
        report.diagnostics
    );
    let resolved = resolved_of(&report);
    assert_eq!(
        resolved.member,
        field(b"p/Root", b"f", b"I"),
        "both nodes spell the same owner, so the name alone cannot separate them"
    );
    assert_eq!(
        resolved.definition, parent_root,
        "the declaration belongs to the node the parent's order selected"
    );
    assert_eq!(resolved.loader, parent_loader);
    assert_ne!(
        resolved.definition, app_root,
        "the walk's own root is a different class and declares nothing"
    );
    assert_eq!(
        read_reasons(&report),
        vec![
            ("app".to_string(), ReadReason::MemberOwner),
            // `p/Root extends p/Mid`, and only the parent holds `p/Mid`.
            ("parent".to_string(), ReadReason::ParentChain),
            // `p/Mid extends p/Root` in the parent's own order: the same name, another node.
            ("parent".to_string(), ReadReason::ParentChain),
        ]
    );
    assert_reads_within_attempts(&report);
    assert_no_body_read(&report);
}
