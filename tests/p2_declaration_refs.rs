//! P2 2.4 acceptance: declaration-reference queries over an explicit environment.
//!
//! The query answers *which use sites reference this declaration*. What this file has to prove
//! through the public API is:
//!
//! 1. the candidates come from the structure consumers themselves, so a call site whose
//!    constant-pool owner is `p/Sub` is found for the `p/Base.foo` declaration it resolves to —
//!    while P1's `mentions_symbol` keeps the two apart, which is the contrast acceptance A11
//!    draws;
//! 2. a constant-pool entry no instruction consumes is not a reference at all, and a candidate
//!    that resolves to a *different* declaration is decided and left out of `items` without
//!    being called undecided;
//! 3. a candidate no search could decide — a missing owner, a use site whose member kind the
//!    evidence does not carry, a budget that stops the resolution — is counted as unresolved,
//!    keeps its physical use site in a diagnostic, and leaves the resolution coverage partial;
//! 4. a scan that stopped before the end of its range keeps the reliable prefix and says so
//!    (`has_more`), while `max_items` truncates the published items without turning the
//!    execution partial and without any P1 cursor coming into play;
//! 5. the same class names under two loaders resolve under the requested environment and are
//!    never merged;
//! 6. every item keeps the P1 evidence shape (consumer, operation, the physical origin and the
//!    raw symbol the use site spells) and the resolution reads class headers only: the query's
//!    `code_bytes` is exactly what scanning those same bytes for use sites costs, and a query
//!    whose consumers read no bytecode charges no code byte at all.
//!
//! Fixtures are built from one small class-file writer (a constant-pool interner plus one
//! instruction list per method) and stored ZIPs; no framework and no new dependency.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

/// `ACC_PUBLIC`, `ACC_STATIC`, `ACC_NATIVE`, `ACC_INTERFACE`, `ACC_ABSTRACT` and `ACC_SUPER`
/// (JVMS 4.1/4.6).
const PUBLIC: u16 = 0x0001;
const STATIC: u16 = 0x0008;
const INTERFACE: u16 = 0x0200;
const ABSTRACT: u16 = 0x0400;
const NATIVE: u16 = 0x0100;
const SUPER: u16 = 0x0020;

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
        code_bytes: 1 << 22,
        result_items: 100_000,
        output_bytes: 1 << 22,
        class_headers: 1_000,
        nested_depth: 4,
        dependency_depth: 8,
        analysis_steps: 100_000,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

fn u16b(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn u32b(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

// ---------------------------------------------------------------------------
// Class-file writer
// ---------------------------------------------------------------------------

/// Constant-pool builder: entries are appended in order, identical entries are reused, and
/// every index is the 1-based index the class file really declares.
#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    /// Appends one entry, or returns the index of an identical earlier one.
    fn intern(&mut self, entry: Vec<u8>) -> u16 {
        if let Some(index) = self.entries.iter().position(|found| *found == entry) {
            return u16::try_from(index + 1).expect("fixture pool fits u16");
        }
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("fixture pool fits u16")
    }

    fn utf8(&mut self, text: &[u8]) -> u16 {
        let mut entry = vec![1];
        u16b(
            &mut entry,
            u16::try_from(text.len()).expect("fixture text fits u16"),
        );
        entry.extend_from_slice(text);
        self.intern(entry)
    }

    fn class(&mut self, name: u16) -> u16 {
        let mut entry = vec![7];
        u16b(&mut entry, name);
        self.intern(entry)
    }

    fn name_and_type(&mut self, name: u16, descriptor: u16) -> u16 {
        let mut entry = vec![12];
        u16b(&mut entry, name);
        u16b(&mut entry, descriptor);
        self.intern(entry)
    }

    /// `Fieldref` (9), `Methodref` (10) or `InterfaceMethodref` (11).
    fn member(&mut self, tag: u8, class: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![tag];
        u16b(&mut entry, class);
        u16b(&mut entry, name_and_type);
        self.intern(entry)
    }

    /// `CONSTANT_MethodHandle` (15): `kind` is the JVMS 4.4.8 reference kind.
    fn method_handle(&mut self, kind: u8, reference: u16) -> u16 {
        let mut entry = vec![15, kind];
        u16b(&mut entry, reference);
        self.intern(entry)
    }

    /// `CONSTANT_InvokeDynamic` (18): the bootstrap entry index and the site's name and type.
    fn invoke_dynamic(&mut self, bootstrap: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![18];
        u16b(&mut entry, bootstrap);
        u16b(&mut entry, name_and_type);
        self.intern(entry)
    }

    fn declared(&self) -> u16 {
        u16::try_from(self.entries.len() + 1).expect("fixture pool fits u16")
    }

    fn bytes(&self) -> Vec<u8> {
        self.entries.iter().flatten().copied().collect()
    }
}

/// One instruction of a fixture body.
///
/// The writer interns the constant-pool entries the instruction reads and encodes its own
/// operand, so a test states the owner, name and descriptor a class file really spells — the
/// whole point of these fixtures, because the owner of a reference may be a subclass of the
/// class that declares the member.
#[derive(Clone, Copy)]
enum Insn {
    /// One `invoke*` instruction reading a `Methodref` of the named member.
    Invoke {
        opcode: u8,
        owner: &'static [u8],
        name: &'static [u8],
        descriptor: &'static [u8],
    },
    /// One `get*`/`put*` instruction reading a `Fieldref` of the named member.
    Field {
        opcode: u8,
        owner: &'static [u8],
        name: &'static [u8],
        descriptor: &'static [u8],
    },
    /// One `ldc_w` of a `CONSTANT_MethodHandle` naming a member.
    Handle {
        kind: u8,
        owner: &'static [u8],
        name: &'static [u8],
        descriptor: &'static [u8],
    },
    /// One `invokedynamic` instruction and the `BootstrapMethods` entry it names.
    ///
    /// The bootstrap method is a `MethodHandle` of the named member, and every argument is a
    /// `MethodHandle` argument naming a member of its own — the shape the bootstrap consumer
    /// reports as `BootstrapArgument` facts.
    InvokeDynamic {
        bootstrap: (&'static [u8], &'static [u8], &'static [u8]),
        arguments: &'static [(&'static [u8], &'static [u8], &'static [u8])],
        name: &'static [u8],
        descriptor: &'static [u8],
    },
    /// `return`.
    Return,
}

fn invoke_virtual(owner: &'static [u8], name: &'static [u8], descriptor: &'static [u8]) -> Insn {
    Insn::Invoke {
        opcode: 0xb6,
        owner,
        name,
        descriptor,
    }
}

fn get_static(owner: &'static [u8], name: &'static [u8], descriptor: &'static [u8]) -> Insn {
    Insn::Field {
        opcode: 0xb2,
        owner,
        name,
        descriptor,
    }
}

fn get_field(owner: &'static [u8], name: &'static [u8], descriptor: &'static [u8]) -> Insn {
    Insn::Field {
        opcode: 0xb4,
        owner,
        name,
        descriptor,
    }
}

/// One field or method declaration of a fixture class.
struct Member {
    name: &'static [u8],
    descriptor: &'static [u8],
    access: u16,
    /// Instructions of the member's `Code` attribute, when it has one.
    body: Option<Vec<Insn>>,
}

/// One member reference of a fixture class: its entry tag and the symbol it names.
type Reference = (u8, &'static [u8], &'static [u8], &'static [u8]);

/// One class-level attribute of a fixture class.
struct ClassAttribute {
    name: &'static [u8],
    /// Content bytes the fixture already encoded (its pool entries are interned by the
    /// fixture, not by the writer).
    content: Vec<u8>,
}

/// One class file, built from its own declaration list and instruction bodies.
struct Class {
    name: &'static [u8],
    super_class: Option<&'static [u8]>,
    interfaces: Vec<&'static [u8]>,
    access: u16,
    fields: Vec<Member>,
    methods: Vec<Member>,
    attributes: Vec<ClassAttribute>,
    /// Member references the class file holds but no instruction consumes: the `Methodref`
    /// of A01's contrast.
    unused_references: Vec<Reference>,
}

impl Class {
    fn new(name: &'static [u8]) -> Self {
        Self {
            name,
            super_class: Some(b"java/lang/Object"),
            interfaces: Vec::new(),
            access: PUBLIC | SUPER,
            fields: Vec::new(),
            methods: Vec::new(),
            attributes: Vec::new(),
            unused_references: Vec::new(),
        }
    }

    /// An interface: `ACC_INTERFACE` with `ACC_ABSTRACT`, as JVMS 4.1 requires.
    fn interface_class(name: &'static [u8]) -> Self {
        Self {
            access: PUBLIC | INTERFACE | ABSTRACT,
            ..Self::new(name)
        }
    }

    fn implements(mut self, name: &'static [u8]) -> Self {
        self.interfaces.push(name);
        self
    }

    fn root(name: &'static [u8]) -> Self {
        Self {
            super_class: None,
            ..Self::new(name)
        }
    }

    fn super_class(mut self, name: &'static [u8]) -> Self {
        self.super_class = Some(name);
        self
    }

    fn field(mut self, name: &'static [u8], descriptor: &'static [u8], access: u16) -> Self {
        self.fields.push(Member {
            name,
            descriptor,
            access,
            body: None,
        });
        self
    }

    fn method(mut self, name: &'static [u8], descriptor: &'static [u8], access: u16) -> Self {
        self.methods.push(Member {
            name,
            descriptor,
            access,
            body: None,
        });
        self
    }

    fn method_with_body(
        mut self,
        name: &'static [u8],
        descriptor: &'static [u8],
        access: u16,
        body: Vec<Insn>,
    ) -> Self {
        self.methods.push(Member {
            name,
            descriptor,
            access,
            body: Some(body),
        });
        self
    }

    /// A class-level `EnclosingMethod` attribute naming `owner.name:descriptor`.
    fn enclosing_method(
        mut self,
        pool: &mut Pool,
        owner: &'static [u8],
        name: &'static [u8],
        descriptor: &'static [u8],
    ) -> Self {
        let owner_name = pool.utf8(owner);
        let class_index = pool.class(owner_name);
        let method_name = pool.utf8(name);
        let method_descriptor = pool.utf8(descriptor);
        let method_index = pool.name_and_type(method_name, method_descriptor);
        let mut content = Vec::new();
        u16b(&mut content, class_index);
        u16b(&mut content, method_index);
        self.attributes.push(ClassAttribute {
            name: b"EnclosingMethod",
            content,
        });
        self
    }

    fn unused_method_reference(
        mut self,
        owner: &'static [u8],
        name: &'static [u8],
        descriptor: &'static [u8],
    ) -> Self {
        self.unused_references.push((10, owner, name, descriptor));
        self
    }

    /// The internal name this fixture declares as its own `this_class`.
    fn name(&self) -> &'static [u8] {
        self.name
    }

    fn build(&self) -> Vec<u8> {
        self.build_with(&mut Pool::default())
    }

    /// Builds the class file, interning every entry into `pool`.
    ///
    /// The pool is passed in so a fixture can intern the entries a class-level attribute names
    /// before the class file itself is written, which is how the local-class fixture records an
    /// enclosing method.
    fn build_with(&self, pool: &mut Pool) -> Vec<u8> {
        let this_name = pool.utf8(self.name);
        let this_class = pool.class(this_name);
        let super_class = self.super_class.map(|name| {
            let super_name = pool.utf8(name);
            pool.class(super_name)
        });
        // Interned before the pool is written, like every other entry this class names.
        let interface_refs = self
            .interfaces
            .iter()
            .map(|interface| {
                let interface_name = pool.utf8(interface);
                pool.class(interface_name)
            })
            .collect::<Vec<_>>();
        let code_name = self
            .methods
            .iter()
            .any(|method| method.body.is_some())
            .then(|| pool.utf8(b"Code"));
        let field_names = self
            .fields
            .iter()
            .map(|member| {
                (
                    member.access,
                    pool.utf8(member.name),
                    pool.utf8(member.descriptor),
                )
            })
            .collect::<Vec<_>>();
        let method_names = self
            .methods
            .iter()
            .map(|member| {
                (
                    member.access,
                    pool.utf8(member.name),
                    pool.utf8(member.descriptor),
                )
            })
            .collect::<Vec<_>>();
        let attribute_names = self
            .attributes
            .iter()
            .map(|attribute| pool.utf8(attribute.name))
            .collect::<Vec<_>>();
        let bodies = {
            let mut bootstrap_entries = Vec::new();
            let bodies = self
                .methods
                .iter()
                .map(|method| {
                    method
                        .body
                        .as_ref()
                        .map(|body| encode(pool, body, &mut bootstrap_entries))
                })
                .collect::<Vec<_>>();
            let name = (!bootstrap_entries.is_empty()).then(|| pool.utf8(BOOTSTRAP_METHODS));
            (name, bootstrap_entries, bodies)
        };
        let (bootstrap_name, bootstrap_entries, bodies) = bodies;
        for (tag, owner, name, descriptor) in &self.unused_references {
            member_reference(pool, *tag, owner, name, descriptor);
        }

        let mut file = 0xcafebabe_u32.to_be_bytes().to_vec();
        u16b(&mut file, 0); // minor
        u16b(&mut file, MAJOR);
        u16b(&mut file, pool.declared());
        file.extend_from_slice(&pool.bytes());
        u16b(&mut file, self.access);
        u16b(&mut file, this_class);
        u16b(&mut file, super_class.unwrap_or(0));
        u16b(
            &mut file,
            u16::try_from(interface_refs.len()).expect("interfaces fit u16"),
        );
        for index in &interface_refs {
            u16b(&mut file, *index);
        }
        u16b(
            &mut file,
            u16::try_from(field_names.len()).expect("fields fit u16"),
        );
        for (access, name, descriptor) in &field_names {
            u16b(&mut file, *access);
            u16b(&mut file, *name);
            u16b(&mut file, *descriptor);
            u16b(&mut file, 0); // attributes
        }
        u16b(
            &mut file,
            u16::try_from(method_names.len()).expect("methods fit u16"),
        );
        for ((access, name, descriptor), body) in method_names.iter().zip(&bodies) {
            u16b(&mut file, *access);
            u16b(&mut file, *name);
            u16b(&mut file, *descriptor);
            match body {
                Some(code) => {
                    u16b(&mut file, 1);
                    u16b(
                        &mut file,
                        code_name.expect("a body declares the Code attribute name"),
                    );
                    let content = code_attribute(code);
                    u32b(
                        &mut file,
                        u32::try_from(content.len()).expect("code body fits u32"),
                    );
                    file.extend_from_slice(&content);
                }
                None => u16b(&mut file, 0),
            }
        }
        u16b(
            &mut file,
            u16::try_from(attribute_names.len() + usize::from(bootstrap_name.is_some()))
                .expect("attributes fit u16"),
        );
        for (name, attribute) in attribute_names.iter().zip(&self.attributes) {
            u16b(&mut file, *name);
            u32b(
                &mut file,
                u32::try_from(attribute.content.len()).expect("attribute fits u32"),
            );
            file.extend_from_slice(&attribute.content);
        }
        if let Some(name) = bootstrap_name {
            let content = bootstrap_attribute(&bootstrap_entries);
            u16b(&mut file, name);
            u32b(
                &mut file,
                u32::try_from(content.len()).expect("bootstrap body fits u32"),
            );
            file.extend_from_slice(&content);
        }
        file
    }
}

/// Attribute name of the deferred-graph table one `invokedynamic` instruction names.
const BOOTSTRAP_METHODS: &[u8] = b"BootstrapMethods";

/// One `BootstrapMethods` entry: the bootstrap handle and the argument entries it holds.
struct BootstrapEntry {
    handle: u16,
    arguments: Vec<u16>,
}

/// The `BootstrapMethods` attribute content of one class, in instruction order.
fn bootstrap_attribute(entries: &[BootstrapEntry]) -> Vec<u8> {
    let mut content = Vec::new();
    u16b(
        &mut content,
        u16::try_from(entries.len()).expect("fixture bootstrap entries fit u16"),
    );
    for entry in entries {
        u16b(&mut content, entry.handle);
        u16b(
            &mut content,
            u16::try_from(entry.arguments.len()).expect("arguments fit u16"),
        );
        for argument in &entry.arguments {
            u16b(&mut content, *argument);
        }
    }
    content
}

/// One `CONSTANT_MethodHandle` entry naming a member, with the JVMS 4.4.8 reference kind.
fn method_handle_of(pool: &mut Pool, kind: u8, member: (&[u8], &[u8], &[u8])) -> u16 {
    let reference = member_reference(pool, 10, member.0, member.1, member.2);
    pool.method_handle(kind, reference)
}

/// Encodes one instruction list, interning the reference entries its operands read.
///
/// Every `invokedynamic` instruction appends its own `BootstrapMethods` entry to `entries`, so
/// the index the instruction names is the entry position the attribute will hold.
fn encode(pool: &mut Pool, body: &[Insn], entries: &mut Vec<BootstrapEntry>) -> Vec<u8> {
    let mut code = Vec::new();
    for insn in body {
        match *insn {
            Insn::Invoke {
                opcode,
                owner,
                name,
                descriptor,
            } => {
                let reference = member_reference(pool, 10, owner, name, descriptor);
                code.push(opcode);
                u16b(&mut code, reference);
            }
            Insn::Field {
                opcode,
                owner,
                name,
                descriptor,
            } => {
                let reference = member_reference(pool, 9, owner, name, descriptor);
                code.push(opcode);
                u16b(&mut code, reference);
            }
            Insn::Handle {
                kind,
                owner,
                name,
                descriptor,
            } => {
                let handle = method_handle_of(pool, kind, (owner, name, descriptor));
                code.push(0x13); // ldc_w
                u16b(&mut code, handle);
            }
            Insn::InvokeDynamic {
                bootstrap,
                arguments,
                name,
                descriptor,
            } => {
                let handle = method_handle_of(pool, 6, bootstrap); // REF_invokeStatic
                let argument_handles = arguments
                    .iter()
                    .map(|argument| method_handle_of(pool, 5, *argument)) // REF_invokeVirtual
                    .collect::<Vec<_>>();
                let index = u16::try_from(entries.len()).expect("bootstrap entries fit u16");
                entries.push(BootstrapEntry {
                    handle,
                    arguments: argument_handles,
                });
                let site_name = pool.utf8(name);
                let site_descriptor = pool.utf8(descriptor);
                let name_and_type = pool.name_and_type(site_name, site_descriptor);
                let site = pool.invoke_dynamic(index, name_and_type);
                code.push(0xba); // invokedynamic
                u16b(&mut code, site);
                code.push(0);
                code.push(0);
            }
            Insn::Return => code.push(0xb1),
        }
    }
    code
}

/// One member reference entry of the named member, interning its parts first.
fn member_reference(pool: &mut Pool, tag: u8, owner: &[u8], name: &[u8], descriptor: &[u8]) -> u16 {
    let owner_name = pool.utf8(owner);
    let class_index = pool.class(owner_name);
    let member_name = pool.utf8(name);
    let member_descriptor = pool.utf8(descriptor);
    let name_and_type = pool.name_and_type(member_name, member_descriptor);
    pool.member(tag, class_index, name_and_type)
}

/// One `Code` attribute body: the instructions, no exception table, no attributes.
fn code_attribute(code: &[u8]) -> Vec<u8> {
    let mut content = Vec::new();
    u16b(&mut content, 2); // max_stack
    u16b(&mut content, 2); // max_locals
    u32b(
        &mut content,
        u32::try_from(code.len()).expect("fixture code fits u32"),
    );
    content.extend_from_slice(code);
    u16b(&mut content, 0); // exception table
    u16b(&mut content, 0); // attributes
    content
}

// ---------------------------------------------------------------------------
// World: content, environment and the query under test
// ---------------------------------------------------------------------------

fn zip_of(classes: &[(Vec<u8>, Vec<u8>)]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in classes {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.clone()))
                .compression_method(CompressionMethod::new(0))
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

fn open(classes: &[(Vec<u8>, Vec<u8>)]) -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(zip_of(classes)),
            &mut Budget::new(limits()),
        )
        .expect("the fixture snapshot opens")
}

/// Every class of a fixture as an `(entry name, class bytes)` pair.
fn pairs(classes: &[Class]) -> Vec<(Vec<u8>, Vec<u8>)> {
    classes
        .iter()
        .map(|class| (entry(class.name()), class.build()))
        .collect()
}

fn loader(name: &str) -> LoaderId {
    LoaderId(name.to_string())
}

fn domain(loader: &LoaderId, roots: Vec<LoadRoot>) -> LoadDomain {
    domain_with(loader, roots, DelegationPolicy::ParentFirst)
}

/// One loader domain, with the delegation policy the caller declares.
fn domain_with(
    loader: &LoaderId,
    roots: Vec<LoadRoot>,
    delegation: DelegationPolicy,
) -> LoadDomain {
    LoadDomain {
        loader: loader.clone(),
        parent_loader: None,
        delegation,
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

/// One fixture world: the classes, the snapshot that holds them, its environment, and the
/// content the query is answered from.
#[derive(Clone)]
struct World {
    classes: Vec<(Vec<u8>, Vec<u8>)>,
    snapshot: ArtifactSnapshot,
    environment: ResolutionEnvironment,
    content: Vec<ArtifactSnapshot>,
}

impl World {
    /// A world whose single `app` loader roots one snapshot holding every given class.
    fn single(classes: Vec<Class>) -> Self {
        Self::from_pairs(pairs(&classes))
    }

    /// The same world from already-built `(entry name, class bytes)` pairs, which is how a
    /// fixture places bytes a class builder would not produce (a damaged class candidate).
    fn from_pairs(classes: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        let snapshot = open(&classes);
        let app = domain(&loader("app"), vec![snapshot_root(&snapshot)]);
        let environment = environment(&snapshot, app.clone(), vec![app]);
        Self {
            classes,
            content: vec![snapshot.clone()],
            snapshot,
            environment,
        }
    }

    /// The physical definition of one class of this world's own snapshot.
    fn definition(&self, class: &[u8]) -> PhysicalDefinitionId {
        let content = &self
            .classes
            .iter()
            .find(|(name, _)| *name == entry(class))
            .unwrap_or_else(|| panic!("the fixture world holds {}", class.escape_ascii()))
            .1;
        archive_definition(&self.snapshot, &entry(class), content)
    }

    fn declaration(&self, owner: &[u8], name: &[u8], descriptor: &[u8]) -> ResolvedMemberRef {
        ResolvedMemberRef {
            loader: self.environment.runtime.load_domain.loader.clone(),
            definition: self.definition(owner),
            member: SymbolRef::Method {
                owner: bytes(owner),
                name: bytes(name),
                descriptor: bytes(descriptor),
            },
        }
    }

    fn field_declaration(&self, owner: &[u8], name: &[u8], descriptor: &[u8]) -> ResolvedMemberRef {
        ResolvedMemberRef {
            loader: self.environment.runtime.load_domain.loader.clone(),
            definition: self.definition(owner),
            member: SymbolRef::Field {
                owner: bytes(owner),
                name: bytes(name),
                descriptor: bytes(descriptor),
            },
        }
    }

    fn query(
        &self,
        declaration: ResolvedMemberRef,
        kinds: &[ConsumerKind],
        max_items: u64,
    ) -> DeclarationRefReport {
        self.query_with(declaration, kinds, max_items, limits()).0
    }

    /// Runs one query under a budget the test owns.
    fn query_with(
        &self,
        declaration: ResolvedMemberRef,
        kinds: &[ConsumerKind],
        max_items: u64,
        limits: Limits,
    ) -> (DeclarationRefReport, Budget) {
        self.query_under_with(
            &self.environment.clone(),
            declaration,
            kinds,
            max_items,
            limits,
        )
    }

    /// Runs one query under an environment the test owns, for the worlds whose snapshot more
    /// than one loader roots.
    fn query_under(
        &self,
        environment: &ResolutionEnvironment,
        declaration: ResolvedMemberRef,
        kinds: &[ConsumerKind],
    ) -> DeclarationRefReport {
        self.query_under_with(environment, declaration, kinds, 0, limits())
            .0
    }

    /// One query under a caller-owned environment and budget.
    fn query_under_with(
        &self,
        environment: &ResolutionEnvironment,
        declaration: ResolvedMemberRef,
        kinds: &[ConsumerKind],
        max_items: u64,
        limits: Limits,
    ) -> (DeclarationRefReport, Budget) {
        let query = DeclarationRefQuery {
            environment: environment.clone(),
            declaration,
            scope: PhysicalScope::SnapshotAll,
            consumers: ConsumerSchema::new(1, kinds.to_vec()),
            max_items,
        };
        let mut budget = Budget::new(limits);
        let report = Engine::new()
            .declaration_references(&self.content, &query, &mut budget)
            .expect("a legal query is answered, not raised");
        (report, budget)
    }

    /// One request-level query with a scope the test chooses, so the scope check is exercised
    /// through the entry point that owns it.
    fn query_scope(
        &self,
        declaration: ResolvedMemberRef,
        kinds: &[ConsumerKind],
        scope: PhysicalScope,
    ) -> std::result::Result<DeclarationRefReport, Error> {
        let query = DeclarationRefQuery {
            environment: self.environment.clone(),
            declaration,
            scope,
            consumers: ConsumerSchema::new(1, kinds.to_vec()),
            max_items: 0,
        };
        Engine::new().declaration_references(&self.content, &query, &mut Budget::new(limits()))
    }

    /// One member resolution under this world's environment, the way a caller would ask it.
    fn resolve(&self, target: SymbolRef, use_kind: ReferenceUse) -> ResolutionReport {
        let request = ResolutionRequest {
            environment: self.environment.clone(),
            target,
            use_kind,
            caller: CallerContext {
                loader: self.environment.runtime.load_domain.loader.clone(),
                enclosing: None,
            },
            dispatch: None,
        };
        let mut budget = Budget::new(limits());
        Engine::new()
            .resolve_symbol(&self.content, &request, &mut budget)
            .expect("a legal request is answered, not raised")
    }
}

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
        .unwrap_or_else(|| panic!("the fixture holds {}", entry_name.escape_ascii()));
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

fn usage_of(execution: &ExecutionReport) -> &UsageSnapshot {
    match execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage,
    }
}

/// The `(loader, definition, reason)` triples of one report's read records, in read order.
fn read_reasons(
    report: &DeclarationRefReport,
) -> Vec<(LoaderId, PhysicalDefinitionId, ReadReason)> {
    report
        .reads
        .iter()
        .map(|read| (read.loader.clone(), read.definition.clone(), read.reason))
        .collect()
}

fn diagnostic_codes(report: &DeclarationRefReport) -> Vec<&str> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

/// The physical use site one diagnostic or item points at.
fn location_of(provenance: &Provenance) -> &Location {
    &provenance.location
}

/// The descriptor of one member symbol, or `None` for a class symbol.
fn descriptor_of(symbol: &SymbolRef) -> Option<&JvmBytes> {
    match symbol {
        SymbolRef::Field { descriptor, .. } | SymbolRef::Method { descriptor, .. } => {
            Some(descriptor)
        }
        SymbolRef::Class { .. } => None,
    }
}

fn method_point(origin: &OriginSet) -> (&PhysicalMethodId, u32) {
    match origin.members.as_slice() {
        [OriginMember::MethodPoint { method, bci }] => (method, *bci),
        other => panic!("expected one method point, got {other:?}"),
    }
}

/// The unresolved-candidate diagnostics of one report.
fn undecided_diagnostics(report: &DeclarationRefReport) -> Vec<&Diagnostic> {
    report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "resolution_candidate_unresolved")
        .collect()
}

/// Asserts that the report's unresolved count and the diagnostics that carry their use sites are
/// the same set: the contract publishes them together, so a counted candidate whose origin was
/// dropped — or a diagnostic that was never counted — fails here.
fn assert_every_unresolved_candidate_keeps_its_use_site(report: &DeclarationRefReport) {
    assert_eq!(
        report.unresolved_candidates,
        u64::try_from(undecided_diagnostics(report).len()).expect("a fixture count fits u64"),
        "every counted undecided candidate keeps its use site in a diagnostic: {:?}",
        diagnostic_codes(report)
    );
    for diagnostic in undecided_diagnostics(report) {
        assert!(
            diagnostic.provenance.is_some(),
            "an undecided candidate without a use site cannot be located: {diagnostic:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// The fixture worlds
// ---------------------------------------------------------------------------

/// The world every clean positive case is measured against.
///
/// `p/Caller` calls `foo` on `p/Sub`, which declares nothing: the reference's owner is the
/// subclass while the declaration is `p/Base.foo` — the shape acceptance A11 is about. The
/// unused `Methodref` is the same name and descriptor as the declaration, consumed by nothing.
fn fixture_world() -> World {
    World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base")
            .method(b"foo", b"()V", PUBLIC)
            .method(b"bar", b"()V", PUBLIC)
            .field(b"value", b"I", PUBLIC | STATIC)
            .field(b"inst", b"I", PUBLIC),
        Class::new(b"p/Sub").super_class(b"p/Base"),
        // The A11 shape: the constant-pool owner of the reference is the subclass.
        Class::new(b"p/Caller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![
                invoke_virtual(b"p/Sub", b"foo", b"()V"),
                invoke_virtual(b"p/Sub", b"bar", b"()V"),
                Insn::Return,
            ],
        ),
        // Field accesses through the subclass: one static, one instance.
        Class::new(b"p/FieldCaller").method_with_body(
            b"read",
            b"()V",
            PUBLIC,
            vec![
                get_static(b"p/Sub", b"value", b"I"),
                get_field(b"p/Sub", b"inst", b"I"),
                Insn::Return,
            ],
        ),
        Class::new(b"p/UnusedCaller")
            .method_with_body(b"run", b"()V", PUBLIC, vec![Insn::Return])
            .unused_method_reference(b"p/Unused", b"foo", b"()V"),
    ])
}

/// A world whose only candidate names an owner the platform does not provide.
fn absent_owner_world() -> World {
    World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC),
        Class::new(b"p/Sub").super_class(b"p/Base"),
        Class::new(b"p/AbsentCaller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/Absent", b"foo", b"()V"), Insn::Return],
        ),
    ])
}

/// A world with exactly one candidate, which resolves to the declaration: `p/Caller` calls
/// `p/Sub.foo`, and `p/Sub` inherits `foo` from `p/Base`.
fn single_candidate_world() -> World {
    World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC),
        Class::new(b"p/Sub").super_class(b"p/Base"),
        Class::new(b"p/Caller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/Sub", b"foo", b"()V"), Insn::Return],
        ),
    ])
}

/// A world whose only candidate is a method handle loaded by `ldc_w`.
fn method_handle_world() -> World {
    World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC),
        Class::new(b"p/Sub").super_class(b"p/Base"),
        Class::new(b"p/HandleCaller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![
                Insn::Handle {
                    kind: 6, // REF_invokeStatic
                    owner: b"p/Sub",
                    name: b"foo",
                    descriptor: b"()V",
                },
                Insn::Return,
            ],
        ),
    ])
}

/// A world with two callers whose references resolve through two different subclasses, so the
/// second resolution cannot be answered from the first one's memo.
fn two_subclass_world() -> World {
    World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC),
        Class::new(b"p/Sub").super_class(b"p/Base"),
        Class::new(b"p/OtherSub").super_class(b"p/Base"),
        Class::new(b"p/FirstCaller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/Sub", b"foo", b"()V"), Insn::Return],
        ),
        Class::new(b"p/SecondCaller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/OtherSub", b"foo", b"()V"), Insn::Return],
        ),
    ])
}

/// The owner of the two signature-polymorphic methods (JVMS 2.9).
const METHOD_HANDLE: &[u8] = b"java/lang/invoke/MethodHandle";
/// Their declared descriptor: the call site picks its own, so this one is the declaration's.
const METHOD_HANDLE_DESCRIPTOR: &[u8] = b"([Ljava/lang/Object;)Ljava/lang/Object;";
/// The descriptor one call site of the fixture picks for itself.
const SIG_POLY_SITE_DESCRIPTOR: &[u8] = b"(Lp/A;)Lp/B;";

/// A world whose declaration is signature-polymorphic (JVMS 2.9).
///
/// `java/lang/invoke/MethodHandle.invoke` and `invokeExact` are declared with the platform's own
/// descriptor, and the call site names the method with a descriptor of its own — which is what a
/// signature-polymorphic call is. The third caller is the discriminating control: it names
/// `invoke` on another owner with exactly the declaration's name *and* descriptor, so a shape
/// that compared only the name (or the name and descriptor, whatever the owner) would call it a
/// candidate.
fn signature_polymorphic_world() -> World {
    World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(METHOD_HANDLE)
            .method(b"invoke", METHOD_HANDLE_DESCRIPTOR, PUBLIC | NATIVE)
            .method(b"invokeExact", METHOD_HANDLE_DESCRIPTOR, PUBLIC | NATIVE),
        Class::new(b"p/Caller").method_with_body(
            b"call",
            b"()V",
            PUBLIC,
            vec![
                invoke_virtual(METHOD_HANDLE, b"invoke", SIG_POLY_SITE_DESCRIPTOR),
                Insn::Return,
            ],
        ),
        Class::new(b"p/ExactCaller").method_with_body(
            b"call",
            b"()V",
            PUBLIC,
            vec![
                invoke_virtual(METHOD_HANDLE, b"invokeExact", b"()V"),
                Insn::Return,
            ],
        ),
        Class::new(b"p/Other"),
        Class::new(b"p/OtherCaller").method_with_body(
            b"call",
            b"()V",
            PUBLIC,
            vec![
                invoke_virtual(b"p/Other", b"invoke", METHOD_HANDLE_DESCRIPTOR),
                Insn::Return,
            ],
        ),
    ])
}

/// A world whose two call sites name the *same* reference, so the second resolution is answered
/// entirely from the first one's request memo.
///
/// That is what makes the cost of publishing the report — one `ResultItems` per entry — the
/// only difference between the two candidates, and therefore makes a budget that funds exactly
/// one publication meaningful.
fn two_identical_callers_world() -> World {
    World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC),
        Class::new(b"p/Sub").super_class(b"p/Base"),
        Class::new(b"p/FirstCaller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/Sub", b"foo", b"()V"), Insn::Return],
        ),
        Class::new(b"p/SecondCaller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/Sub", b"foo", b"()V"), Insn::Return],
        ),
    ])
}

/// The same one-candidate world, with the declaration made static.
///
/// The call site stays `invokevirtual`, so the resolution holds the declaration to the
/// invocation kind, refuses it with `IncompatibleClassChange` and leaves the candidate
/// undecided — while the same class headers are read as in [`single_candidate_world`], which
/// makes the two worlds a controlled pair: only the entries they publish differ.
fn kind_mismatch_world() -> World {
    World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC | STATIC),
        Class::new(b"p/Sub").super_class(b"p/Base"),
        Class::new(b"p/Caller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/Sub", b"foo", b"()V"), Insn::Return],
        ),
    ])
}

/// A world in which the candidate's owner class cannot be told apart.
///
/// The snapshot holds two entries with the same raw name `p/Amb.class` (a legitimate class
/// file each), so the 2.1 lookup meets two indistinguishable definitions at one selection
/// position. The call site names `p/Amb.foo`, and the definition the query asks about is
/// irrelevant: the search cannot leave the owner position at all.
fn ambiguous_owner_world() -> World {
    World::from_pairs(vec![
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"p/Base"),
            Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC).build(),
        ),
        (
            entry(b"p/Amb"),
            Class::new(b"p/Amb").method(b"foo", b"()V", PUBLIC).build(),
        ),
        // The same raw name again: a different class file, indistinguishable by name.
        (
            entry(b"p/Amb"),
            Class::new(b"p/Amb").field(b"f", b"I", PUBLIC).build(),
        ),
        (
            entry(b"p/Caller"),
            Class::new(b"p/Caller")
                .method_with_body(
                    b"run",
                    b"()V",
                    PUBLIC,
                    vec![invoke_virtual(b"p/Amb", b"foo", b"()V"), Insn::Return],
                )
                .build(),
        ),
    ])
}

/// A world whose candidate is a Java 8 default-method conflict.
///
/// Two interfaces each declare a non-abstract `m()`, and the caller's class implements both,
/// so the maximally-specific set holds two defaults and no single one may be selected
/// (JVMS 5.4.3.3; the slice reports the conflict at resolution on purpose).
fn default_conflict_world() -> World {
    World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base"),
        Class::interface_class(b"i/A").method(b"m", b"()V", PUBLIC),
        Class::interface_class(b"i/B").method(b"m", b"()V", PUBLIC),
        Class::new(b"p/User").implements(b"i/A").implements(b"i/B"),
        Class::new(b"p/Caller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/User", b"m", b"()V"), Insn::Return],
        ),
    ])
}

/// A world whose bootstrap argument names a member of `MethodHandle`.
///
/// The `invokedynamic` site's bootstrap method is `p/Bsm.bootstrap`, and its argument is a
/// `MethodHandle` naming `java/lang/invoke/MethodHandle.invoke` — a member fact the bootstrap
/// consumer reports as a `BootstrapArgument` node, with no instruction-level member kind of
/// its own.
fn bootstrap_argument_world() -> World {
    World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(METHOD_HANDLE)
            .method(b"invoke", METHOD_HANDLE_DESCRIPTOR, PUBLIC | NATIVE)
            .method(b"invokeExact", METHOD_HANDLE_DESCRIPTOR, PUBLIC | NATIVE),
        Class::new(b"p/Bsm").method(
            b"bootstrap",
            b"(Ljava/lang/invoke/MethodHandles$Lookup;)V",
            PUBLIC | STATIC,
        ),
        Class::new(b"p/Caller").method_with_body(
            b"call",
            b"()V",
            PUBLIC,
            vec![
                Insn::InvokeDynamic {
                    bootstrap: (
                        b"p/Bsm",
                        b"bootstrap",
                        b"(Ljava/lang/invoke/MethodHandles$Lookup;)V",
                    ),
                    arguments: &[(METHOD_HANDLE, b"invoke", METHOD_HANDLE_DESCRIPTOR)],
                    name: b"site",
                    descriptor: b"()V",
                },
                Insn::Return,
            ],
        ),
    ])
}

/// A world whose only instruction-level fact is a `ldc` of a handle to the *sibling* name of
/// a signature-polymorphic declaration.
///
/// The query under test asks about `MethodHandle.invoke`, and this world holds no `invoke`
/// fact of that shape at all: a shape that compared the owner and forgot the name would call
/// this `invokeExact` handle a candidate of the `invoke` query.
fn signature_polymorphic_sibling_world() -> World {
    World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(METHOD_HANDLE)
            .method(b"invoke", METHOD_HANDLE_DESCRIPTOR, PUBLIC | NATIVE)
            .method(b"invokeExact", METHOD_HANDLE_DESCRIPTOR, PUBLIC | NATIVE),
        Class::new(b"p/HandleCaller").method_with_body(
            b"call",
            b"()V",
            PUBLIC,
            vec![
                Insn::Handle {
                    kind: 5, // REF_invokeVirtual
                    owner: METHOD_HANDLE,
                    name: b"invokeExact",
                    descriptor: METHOD_HANDLE_DESCRIPTOR,
                },
                Insn::Return,
            ],
        ),
    ])
}

// ---------------------------------------------------------------------------
// The candidate is the use site, and P1 keeps the raw symbol
// ---------------------------------------------------------------------------

/// One `mentions_symbol` query over the world's snapshot, returning its item count.
fn p1_items(world: &World, target: SymbolRef, kinds: &[ConsumerKind]) -> usize {
    p1_query(world, QueryRelation::MentionsSymbol, target, kinds)
        .items
        .len()
}

/// One P1 query over the world's snapshot, so a test can contrast P1's own answer with the
/// declaration-reference answer for the same bytes.
fn p1_query(
    world: &World,
    relation: QueryRelation,
    target: SymbolRef,
    kinds: &[ConsumerKind],
) -> QueryReport {
    let request = QueryRequest {
        relation,
        target: QueryTarget::Symbol { value: target },
        physical: PhysicalView {
            snapshot: world.snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: ConsumerSchema::new(1, kinds.to_vec()),
        max_items: 0,
        cursor: None,
    };
    let mut budget = Budget::new(limits());
    Engine::new()
        .query(&world.snapshot, &request, &mut budget)
        .expect("the P1 query is answered")
}

#[test]
fn a_sub_call_site_resolves_to_the_base_declaration() {
    let world = fixture_world();
    let declaration = world.declaration(b"p/Base", b"foo", b"()V");

    // A11's contrast: P1 matches the raw symbol, so the `p/Sub.foo` call site is not an
    // answer for a `p/Base.foo` query at all — and the same query for `p/Sub.foo` is.
    assert_eq!(
        p1_items(
            &world,
            SymbolRef::Method {
                owner: bytes(b"p/Base"),
                name: bytes(b"foo"),
                descriptor: bytes(b"()V"),
            },
            &[ConsumerKind::Invocation],
        ),
        0,
        "P1 does not expand the declaration's owner, so no pool entry answers it"
    );
    assert_eq!(
        p1_items(
            &world,
            SymbolRef::Method {
                owner: bytes(b"p/Sub"),
                name: bytes(b"foo"),
                descriptor: bytes(b"()V"),
            },
            &[ConsumerKind::Invocation],
        ),
        1,
        "P1 reports the use site under the symbol it really spells"
    );

    let (report, budget) = world.query_with(
        declaration.clone(),
        &[ConsumerKind::Invocation],
        0,
        limits(),
    );
    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert_eq!(
        report.items.len(),
        1,
        "one use site references the declaration: {:?}",
        report.items
    );
    let item = &report.items[0];
    assert_eq!(
        item.referenced,
        SymbolRef::Method {
            owner: bytes(b"p/Sub"),
            name: bytes(b"foo"),
            descriptor: bytes(b"()V"),
        },
        "the item keeps the raw symbol of the use site, owner included"
    );
    assert_eq!(item.consumer, ConsumerKind::Invocation);
    assert_eq!(item.operation, XrefOperation::InvokeVirtual);
    assert_eq!(item.state, ResolutionState::Resolved);
    assert_eq!(
        item.resolved.as_ref(),
        Some(&declaration),
        "the resolution names the declaration the use site really references"
    );
    let (method, bci) = method_point(&item.origin);
    assert_eq!(method.name, bytes(b"run"));
    assert_eq!(method.descriptor, bytes(b"()V"));
    assert_eq!(
        bci, 0,
        "the origin is the instruction that names the subclass"
    );
    assert_eq!(
        method.owner,
        world.definition(b"p/Caller"),
        "the physical use site is the caller's own definition"
    );

    assert_eq!(report.unresolved_candidates, 0);
    assert!(!report.has_more);
    assert_eq!(report.returned_items, 1);
    assert_eq!(report.unsupported_categories, Vec::new());
    assert!(report.environment_problems.is_empty());
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema,
        "every candidate reached a decision about a declaration"
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(diagnostic_codes(&report), Vec::<&str>::new());

    // The resolution reads class headers — the subclass itself, then the class chain that
    // holds the declaration — and every read is one attempt.
    assert_eq!(
        read_reasons(&report),
        vec![
            (
                loader("app"),
                world.definition(b"p/Sub"),
                ReadReason::MemberOwner
            ),
            (
                loader("app"),
                world.definition(b"p/Base"),
                ReadReason::ParentChain
            ),
        ]
    );
    let usage = usage_of(&report.execution);
    assert_eq!(usage.class_headers, 2);
    assert_eq!(
        budget.usage().class_headers,
        2,
        "the report and the live budget agree on the reads"
    );
    assert_eq!(
        usage.method_bodies, 0,
        "no body read is charged (the dimension has no charge point before 3.x either)"
    );

    // The code bytes this query charged are exactly the ones scanning these bytes for use
    // sites costs: the same consumers through the P1 entry point charge the same number, so the
    // resolution added no code byte charge of its own and read no body.
    let mut p1_budget = Budget::new(limits());
    let p1_report = Engine::new()
        .query(
            &world.snapshot,
            &QueryRequest {
                relation: QueryRelation::MentionsSymbol,
                target: QueryTarget::Symbol {
                    value: SymbolRef::Method {
                        owner: bytes(b"p/Sub"),
                        name: bytes(b"foo"),
                        descriptor: bytes(b"()V"),
                    },
                },
                physical: PhysicalView {
                    snapshot: world.snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
                max_items: 0,
                cursor: None,
            },
            &mut p1_budget,
        )
        .expect("the control query is answered");
    assert!(usage.code_bytes > 0, "the structural scan did read bodies");
    assert_eq!(
        usage.code_bytes,
        usage_of(&p1_report.execution).code_bytes,
        "the declaration query reads no bytecode beyond the scan it reuses"
    );
}

#[test]
fn an_unconsumed_pool_entry_is_not_a_candidate() {
    // The snapshot holds a `Methodref p/Unused.foo:()V` no instruction consumes. Enumerating
    // the pool would answer the query's shape; the *consumers* answer the query, and they found
    // no use site at all.
    let world = World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC),
        Class::new(b"p/UnusedCaller")
            .method_with_body(b"run", b"()V", PUBLIC, vec![Insn::Return])
            .unused_method_reference(b"p/Unused", b"foo", b"()V"),
    ]);
    let declaration = world.declaration(b"p/Base", b"foo", b"()V");
    let report = world.query(declaration, &[ConsumerKind::Invocation], 0);

    assert!(report.items.is_empty());
    assert_eq!(
        report.unresolved_candidates, 0,
        "an unused entry is not a candidate at all, so it is not an undecided one either"
    );
    assert!(
        report.reads.is_empty(),
        "no candidate was found, so no resolution ran and no header was demanded"
    );
    assert_eq!(usage_of(&report.execution).class_headers, 0);
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema,
        "the scan covered its range and had nothing undecided"
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert!(!report.has_more);

    // The control that makes the empty answer a fact about consumers rather than about the
    // fixture: the raw constant-pool probe really does find that `Methodref`, so the entry is
    // in the class file this query scanned — no instruction consumes it.
    let pool = p1_query(
        &world,
        QueryRelation::ConstantPoolContains,
        SymbolRef::Method {
            owner: bytes(b"p/Unused"),
            name: bytes(b"foo"),
            descriptor: bytes(b"()V"),
        },
        &[ConsumerKind::Invocation],
    );
    assert_eq!(pool.items.len(), 1, "the entry exists: {:?}", pool.items);
    assert_eq!(
        pool.items[0].derivation,
        XrefDerivation::ConstantPoolCandidate,
        "the probe reports the raw entry and claims no consumer"
    );
    assert_eq!(pool.items[0].consumer, None);
    assert_eq!(pool.items[0].operation, XrefOperation::ConstantPoolEntry);
    assert_eq!(pool.items[0].evidence.bci, None, "no instruction reads it");
    assert!(
        pool.items[0].evidence.constant_pool_index.is_some(),
        "the probe names the entry it found"
    );
}

#[test]
fn a_candidate_that_resolves_to_another_declaration_is_not_an_item() {
    // Two hierarchy roots declare the same member shape, and the only candidate references the
    // one the query did *not* ask about: the candidate is decided, and decided to be a
    // reference to another declaration. It must neither appear as an item nor be reported as
    // undecided.
    let world = World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC),
        Class::new(b"p/OtherBase").method(b"foo", b"()V", PUBLIC),
        Class::new(b"p/Sub").super_class(b"p/Base"),
        Class::new(b"p/OtherSub").super_class(b"p/OtherBase"),
        Class::new(b"p/Caller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/OtherSub", b"foo", b"()V"), Insn::Return],
        ),
    ]);
    let report = world.query(
        world.declaration(b"p/Base", b"foo", b"()V"),
        &[ConsumerKind::Invocation],
        0,
    );

    assert!(report.items.is_empty());
    assert_eq!(report.unresolved_candidates, 0);
    assert!(
        !report.reads.is_empty(),
        "the candidate really was resolved: the reads prove the search ran"
    );
    assert_eq!(
        read_reasons(&report),
        vec![
            (
                loader("app"),
                world.definition(b"p/OtherSub"),
                ReadReason::MemberOwner
            ),
            (
                loader("app"),
                world.definition(b"p/OtherBase"),
                ReadReason::ParentChain
            ),
        ],
        "the search followed the candidate's own owner, not the queried declaration"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
}

// ---------------------------------------------------------------------------
// Undecided candidates stay undecided
// ---------------------------------------------------------------------------

#[test]
fn a_missing_owner_is_unresolved_and_keeps_its_use_site() {
    let world = absent_owner_world();
    let report = world.query(
        world.declaration(b"p/Base", b"foo", b"()V"),
        &[ConsumerKind::Invocation],
        0,
    );

    assert!(report.items.is_empty());
    assert_eq!(
        report.unresolved_candidates, 1,
        "the reference to the unprovided `p/Absent` is not an excluded candidate"
    );
    assert_every_unresolved_candidate_keeps_its_use_site(&report);
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial,
        "an undecided candidate leaves the resolution plane incomplete"
    );
    assert!(
        matches!(report.execution, ExecutionReport::Complete { .. }),
        "a missing dependency is a decision, not a stop: {:?}",
        report.execution
    );
    assert!(!report.has_more);

    let diagnostics = undecided_diagnostics(&report);
    assert_eq!(
        diagnostics.len(),
        1,
        "the unresolved candidate keeps its use site in a diagnostic: {:?}",
        diagnostic_codes(&report)
    );
    assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Warning);
    match location_of(
        diagnostics[0]
            .provenance
            .as_ref()
            .expect("the diagnostic names its use site"),
    ) {
        Location::Code { method, bci } => {
            assert_eq!(method.owner, world.definition(b"p/AbsentCaller"));
            assert_eq!(method.name, bytes(b"run"));
            assert_eq!(*bci, 0, "the origin is the instruction that was scanned");
        }
        other => panic!("expected the instruction's own location, got {other:?}"),
    }
}

#[test]
fn a_method_handle_load_stays_undecided() {
    // An `ldc_w` of a `CONSTANT_MethodHandle` names a member, but the handle's reference kind
    // lives in the constant pool and not in the item's evidence, so the query does not invent
    // an invocation kind for it: the candidate is kept undecided.
    let world = method_handle_world();
    let report = world.query(
        world.declaration(b"p/Base", b"foo", b"()V"),
        &[ConsumerKind::Constant],
        0,
    );

    assert!(report.items.is_empty());
    assert_eq!(report.unresolved_candidates, 1);
    assert_every_unresolved_candidate_keeps_its_use_site(&report);
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_candidate_unresolved"],
        "the undecided candidate keeps its use site and says why"
    );
    match location_of(
        report.diagnostics[0]
            .provenance
            .as_ref()
            .expect("the diagnostic names its use site"),
    ) {
        Location::Code { method, bci } => {
            assert_eq!(method.owner, world.definition(b"p/HandleCaller"));
            assert_eq!(*bci, 0);
        }
        other => panic!("expected the instruction's own location, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Signature-polymorphic declarations
// ---------------------------------------------------------------------------

#[test]
fn a_signature_polymorphic_call_site_is_a_candidate_at_its_own_descriptor() {
    let world = signature_polymorphic_world();
    let declaration = world.declaration(METHOD_HANDLE, b"invoke", METHOD_HANDLE_DESCRIPTOR);
    let site = SymbolRef::Method {
        owner: bytes(METHOD_HANDLE),
        name: bytes(b"invoke"),
        descriptor: bytes(SIG_POLY_SITE_DESCRIPTOR),
    };

    // The review's probe, as a control inside the test: the same site really does resolve to
    // this declaration, so a query that answers "no candidate at all" is a false negative.
    let resolution = world.resolve(site.clone(), ReferenceUse::InvokeVirtual);
    assert_eq!(resolution.state, Some(ResolutionState::Resolved));
    let resolved = resolution
        .resolved
        .as_ref()
        .expect("a resolved member query publishes its declaration");
    assert_eq!(
        resolved.member, declaration.member,
        "the resolution selected the declaration, whose descriptor is the declaration's own"
    );
    assert_ne!(
        descriptor_of(&resolved.member).expect("the resolved member is a method"),
        &bytes(SIG_POLY_SITE_DESCRIPTOR),
        "the site's descriptor and the declaration's are different, which is the whole point"
    );

    // P1 answers under the raw bytes the site spells, so it cannot answer the declaration's
    // own symbol — the contrast that makes this a candidate question and not a match question.
    assert_eq!(
        p1_items(
            &world,
            declaration.member.clone(),
            &[ConsumerKind::Invocation]
        ),
        0
    );
    assert_eq!(
        p1_items(&world, site.clone(), &[ConsumerKind::Invocation]),
        1
    );

    let report = world.query(declaration.clone(), &[ConsumerKind::Invocation], 0);
    assert_eq!(report.items.len(), 1, "one candidate: {:?}", report.items);
    let item = &report.items[0];
    assert_eq!(
        item.referenced, site,
        "the item keeps the symbol the call site spells, descriptor included"
    );
    assert_eq!(item.resolved.as_ref(), Some(&declaration));
    assert_eq!(item.state, ResolutionState::Resolved);
    assert_eq!(item.consumer, ConsumerKind::Invocation);
    assert_eq!(item.operation, XrefOperation::InvokeVirtual);
    let (method, bci) = method_point(&item.origin);
    assert_eq!(method.owner, world.definition(b"p/Caller"));
    assert_eq!(method.name, bytes(b"call"));
    assert_eq!(bci, 0);

    assert_eq!(
        report.unresolved_candidates, 0,
        "a site on another owner with the declaration's own name and descriptor is not even a \
         candidate of a signature-polymorphic declaration"
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "resolution_signature_polymorphic"),
        "the resolution names the approximation it applied: {:?}",
        diagnostic_codes(&report)
    );
    assert!(
        !report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "resolution_candidate_unresolved"),
        "nothing was left undecided: {:?}",
        diagnostic_codes(&report)
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert!(!report.has_more);
    assert!(report.environment_problems.is_empty());
}

#[test]
fn a_signature_polymorphic_invoke_exact_call_site_is_a_candidate_too() {
    // The second of the two methods JVMS 2.9 defines: its call site picks a descriptor of its
    // own as well, and the declaration is a different member of the same owner.
    let world = signature_polymorphic_world();
    let declaration = world.declaration(METHOD_HANDLE, b"invokeExact", METHOD_HANDLE_DESCRIPTOR);
    let report = world.query(declaration.clone(), &[ConsumerKind::Invocation], 0);

    assert_eq!(report.items.len(), 1, "{:?}", report.items);
    assert_eq!(
        report.items[0].referenced,
        SymbolRef::Method {
            owner: bytes(METHOD_HANDLE),
            name: bytes(b"invokeExact"),
            descriptor: bytes(b"()V"),
        },
        "the item keeps the call site's own descriptor"
    );
    assert_eq!(report.items[0].resolved.as_ref(), Some(&declaration));
    let (method, bci) = method_point(&report.items[0].origin);
    assert_eq!(method.owner, world.definition(b"p/ExactCaller"));
    assert_eq!(bci, 0);
    assert_eq!(report.unresolved_candidates, 0);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "resolution_signature_polymorphic")
    );
}

#[test]
fn a_sibling_name_on_the_same_owner_is_not_a_candidate() {
    // The sibling-name control for the shape's name dimension: the only fact in this world is a
    // `ldc` of a handle to `MethodHandle.invokeExact`, and the query asks about
    // `MethodHandle.invoke`. The fact is a member reference on the same owner, so a shape that
    // compared the owner and forgot the name would call it a candidate of the `invoke` query and
    // leave it undecided; the correct answer is that it is not a candidate at all.
    let world = signature_polymorphic_sibling_world();
    let report = world.query(
        world.declaration(METHOD_HANDLE, b"invoke", METHOD_HANDLE_DESCRIPTOR),
        &[ConsumerKind::Constant],
        0,
    );

    assert!(report.items.is_empty());
    assert_eq!(
        report.unresolved_candidates,
        0,
        "an `invokeExact` handle is not a candidate of the `invoke` declaration: {:?}",
        diagnostic_codes(&report)
    );
    assert_eq!(
        diagnostic_codes(&report),
        Vec::<&str>::new(),
        "the sibling fact is not even looked at"
    );
    assert!(
        report.reads.is_empty(),
        "no candidate was found, so no resolution ran"
    );
    assert!(
        matches!(report.execution, ExecutionReport::Complete { .. }),
        "the scan covered its range: {:?}",
        report.execution
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema,
        "nothing was left undecided"
    );

    // The other side of the same control: asked about `invokeExact`, the very same fact *is* a
    // candidate and stays undecided with its use site.
    let sibling = world.query(
        world.declaration(METHOD_HANDLE, b"invokeExact", METHOD_HANDLE_DESCRIPTOR),
        &[ConsumerKind::Constant],
        0,
    );
    assert!(sibling.items.is_empty());
    assert_eq!(sibling.unresolved_candidates, 1);
    assert_every_unresolved_candidate_keeps_its_use_site(&sibling);
    assert_eq!(
        diagnostic_codes(&sibling),
        vec!["resolution_candidate_unresolved"]
    );
    assert_eq!(
        location_of(
            undecided_diagnostics(&sibling)[0]
                .provenance
                .as_ref()
                .expect("the diagnostic names its use site")
        ),
        &Location::Code {
            method: PhysicalMethodId {
                owner: world.definition(b"p/HandleCaller"),
                name: bytes(b"call"),
                descriptor: bytes(b"()V"),
            },
            bci: 0,
        }
    );
}

#[test]
fn an_ordinary_declaration_matches_by_name_and_descriptor_not_by_name() {
    // The other half of the shape choice: `p/Base.foo` is not signature-polymorphic, so the
    // candidate shape compares the descriptor — a site that shares only the name is not a
    // candidate, even though a site on another owner that shares both is.
    let world = World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC),
        Class::new(b"p/Sub").super_class(b"p/Base"),
        Class::new(b"p/Other"),
        Class::new(b"p/Caller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![
                invoke_virtual(b"p/Sub", b"foo", b"()V"),
                invoke_virtual(b"p/Other", b"foo", b"(I)V"),
                Insn::Return,
            ],
        ),
    ]);
    let declaration = world.declaration(b"p/Base", b"foo", b"()V");
    let report = world.query(declaration.clone(), &[ConsumerKind::Invocation], 0);

    assert_eq!(report.items.len(), 1, "{:?}", report.items);
    assert_eq!(report.items[0].resolved.as_ref(), Some(&declaration));
    let (method, bci) = method_point(&report.items[0].origin);
    assert_eq!(method.owner, world.definition(b"p/Caller"));
    assert_eq!(
        bci, 0,
        "the candidate is the `()V` call, not the `(I)V` one"
    );
    assert_eq!(
        report.unresolved_candidates, 0,
        "the `(I)V` site shares the name but not the descriptor, so it is not a candidate"
    );
}

#[test]
fn an_ambiguous_owner_is_unresolved_and_keeps_its_use_site() {
    // The candidate's owner class cannot be told apart at its selection position (2.1 finds two
    // indistinguishable definitions for one raw entry name), so the search never reaches a
    // declaration: the candidate stays undecided with its use site instead of being called
    // excluded or resolved.
    let world = ambiguous_owner_world();
    let report = world.query(
        world.declaration(b"p/Base", b"foo", b"()V"),
        &[ConsumerKind::Invocation],
        0,
    );

    assert!(report.items.is_empty());
    assert_eq!(report.unresolved_candidates, 1);
    assert_every_unresolved_candidate_keeps_its_use_site(&report);
    assert_eq!(
        diagnostic_codes(&report),
        vec!["duplicate_raw_name", "resolution_candidate_unresolved"],
        "the scan reports the duplicate raw name it had to read, and the ambiguous position is \
         an undecided candidate, not a resolved one"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        location_of(
            undecided_diagnostics(&report)[0]
                .provenance
                .as_ref()
                .expect("the diagnostic names its use site")
        ),
        &Location::Code {
            method: PhysicalMethodId {
                owner: world.definition(b"p/Caller"),
                name: bytes(b"run"),
                descriptor: bytes(b"()V"),
            },
            bci: 0,
        }
    );
    // Both indistinguishable definitions were read before the position was declared ambiguous,
    // so the reads show the search really went there.
    assert_eq!(
        report
            .reads
            .iter()
            .filter(|read| read.reason == ReadReason::MemberOwner)
            .count(),
        2,
        "both candidates of the ambiguous position are reads of the member's owner: {:?}",
        read_reasons(&report)
    );
}

#[test]
fn a_default_conflict_is_unresolved_and_keeps_its_use_site() {
    // Two interfaces each declare a non-abstract `m()` and the caller's class implements both,
    // so the maximally-specific set holds two defaults: 2.3 reports the conflict rather than
    // picking one, and the candidate stays undecided.
    let world = default_conflict_world();
    let report = world.query(
        world.declaration(b"i/A", b"m", b"()V"),
        &[ConsumerKind::Invocation],
        0,
    );

    assert!(report.items.is_empty());
    assert_eq!(report.unresolved_candidates, 1);
    assert_every_unresolved_candidate_keeps_its_use_site(&report);
    assert_eq!(
        diagnostic_codes(&report),
        vec![
            "resolution_default_conflict",
            "resolution_candidate_unresolved"
        ],
        "the conflict is named, and the candidate it left open is counted"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        location_of(
            report.diagnostics[1]
                .provenance
                .as_ref()
                .expect("the diagnostic names its use site")
        ),
        &Location::Code {
            method: PhysicalMethodId {
                owner: world.definition(b"p/Caller"),
                name: bytes(b"run"),
                descriptor: bytes(b"()V"),
            },
            bci: 0,
        }
    );
}

#[test]
fn a_bootstrap_member_argument_stays_undecided() {
    // The bootstrap table of the caller's `invokedynamic` site holds an argument that is a
    // `MethodHandle` naming `MethodHandle.invoke`, which is exactly the declaration under test:
    // the bootstrap consumer reports it as a member fact, and — like the `ldc` handle — that
    // fact carries no instruction-level member kind, so the candidate stays undecided with its
    // own use site instead of being resolved under an invented kind.
    let world = bootstrap_argument_world();
    let declaration = world.declaration(METHOD_HANDLE, b"invoke", METHOD_HANDLE_DESCRIPTOR);
    let report = world.query(declaration, &[ConsumerKind::Bootstrap], 0);

    assert!(
        report.items.is_empty(),
        "a bootstrap argument is a member fact without a member kind: {:?}",
        report.items
    );
    assert_eq!(report.unresolved_candidates, 1);
    assert_every_unresolved_candidate_keeps_its_use_site(&report);
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_candidate_unresolved"],
        "the same decision point the `ldc` and metadata facts go through"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        location_of(
            report.diagnostics[0]
                .provenance
                .as_ref()
                .expect("the diagnostic names its use site")
        ),
        &Location::Code {
            method: PhysicalMethodId {
                owner: world.definition(b"p/Caller"),
                name: bytes(b"call"),
                descriptor: bytes(b"()V"),
            },
            bci: 0,
        },
        "the use site is the instruction whose bootstrap table holds the argument"
    );
    assert!(
        usage_of(&report.execution).class_headers == 0,
        "an undecided candidate starts no member search"
    );
}

// ---------------------------------------------------------------------------
// Field references and metadata facts
// ---------------------------------------------------------------------------

#[test]
fn field_accesses_resolve_through_the_declaring_class() {
    let world = fixture_world();

    let declaration = world.field_declaration(b"p/Base", b"value", b"I");
    let static_report = world.query(declaration.clone(), &[ConsumerKind::Field], 0);
    assert_eq!(static_report.items.len(), 1);
    assert_eq!(static_report.items[0].operation, XrefOperation::GetStatic);
    assert_eq!(static_report.items[0].consumer, ConsumerKind::Field);
    assert_eq!(
        static_report.items[0].referenced,
        SymbolRef::Field {
            owner: bytes(b"p/Sub"),
            name: bytes(b"value"),
            descriptor: bytes(b"I"),
        }
    );
    assert_eq!(static_report.items[0].resolved.as_ref(), Some(&declaration));
    let (method, bci) = method_point(&static_report.items[0].origin);
    assert_eq!(method.owner, world.definition(b"p/FieldCaller"));
    assert_eq!(method.name, bytes(b"read"));
    assert_eq!(bci, 0, "the static read is the first instruction");
    assert_eq!(static_report.unresolved_candidates, 0);

    let instance_report = world.query(
        world.field_declaration(b"p/Base", b"inst", b"I"),
        &[ConsumerKind::Field],
        0,
    );
    assert_eq!(instance_report.items.len(), 1);
    assert_eq!(instance_report.items[0].operation, XrefOperation::GetField);
    assert_eq!(
        instance_report.items[0].resolved.as_ref(),
        Some(&world.field_declaration(b"p/Base", b"inst", b"I"))
    );
    let (_, bci) = method_point(&instance_report.items[0].origin);
    assert_eq!(bci, 3, "the instance read is the second instruction");
}

#[test]
fn a_static_instruction_on_an_instance_field_is_reported_without_the_kind_rule() {
    // The design's boundary: the static/instance rule for a *field* reference belongs to the
    // instruction kind, which the member rules do not carry, so `GetField` on a static
    // declaration resolves here exactly like `GetStatic` would. The query reports the
    // resolution it performed instead of inventing a rule the rules table does not have.
    let world = World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").field(b"value", b"I", PUBLIC | STATIC),
        Class::new(b"p/Sub").super_class(b"p/Base"),
        Class::new(b"p/MixedCaller").method_with_body(
            b"read",
            b"()V",
            PUBLIC,
            vec![get_field(b"p/Sub", b"value", b"I"), Insn::Return],
        ),
    ]);
    let declaration = world.field_declaration(b"p/Base", b"value", b"I");
    let report = world.query(declaration.clone(), &[ConsumerKind::Field], 0);

    assert_eq!(report.items.len(), 1);
    assert_eq!(report.items[0].operation, XrefOperation::GetField);
    assert_eq!(report.items[0].resolved.as_ref(), Some(&declaration));
    assert_eq!(report.unresolved_candidates, 0);
}

#[test]
fn a_metadata_fact_whos_member_kind_is_unstated_stays_undecided() {
    // A local class records its enclosing method in the class file: the fact names a member
    // with the enclosing class as its owner, so it is a candidate use site like any other, and
    // the consumers that produce it read no instruction stream at all. What the fact does *not*
    // state is an invocation kind — and the member rules read one to hold a declaration to the
    // reference — so the query keeps the candidate undecided with its origin instead of
    // resolving it under a kind it invented.
    let mut pool = Pool::default();
    let local = Class::new(b"p/Local").enclosing_method(&mut pool, b"p/Sub", b"foo", b"()V");
    let local_bytes = local.build_with(&mut pool);
    let classes = vec![
        (
            entry(b"java/lang/Object"),
            Class::root(b"java/lang/Object").build(),
        ),
        (
            entry(b"p/Base"),
            Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC).build(),
        ),
        (
            entry(b"p/Sub"),
            Class::new(b"p/Sub").super_class(b"p/Base").build(),
        ),
        (entry(b"p/Local"), local_bytes),
    ];
    let world = World::from_pairs(classes);

    let report = world.query(
        world.declaration(b"p/Base", b"foo", b"()V"),
        &[ConsumerKind::InnerNest],
        0,
    );

    assert!(report.items.is_empty());
    assert_eq!(report.unresolved_candidates, 1);
    assert_every_unresolved_candidate_keeps_its_use_site(&report);
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_candidate_unresolved"],
        "the candidate is reported where it was found"
    );
    match location_of(
        report.diagnostics[0]
            .provenance
            .as_ref()
            .expect("the diagnostic names its use site"),
    ) {
        Location::ClassOffset { definition, .. } => {
            assert_eq!(
                *definition,
                world.definition(b"p/Local"),
                "the physical use site is the local class that records the fact"
            );
        }
        other => panic!("expected the class-file offset of the fact, got {other:?}"),
    }
    assert_eq!(
        usage_of(&report.execution).code_bytes,
        0,
        "a metadata-only query reads no instruction stream at all"
    );
    assert_eq!(
        usage_of(&report.execution).class_headers,
        0,
        "an undecided candidate demands no header: no member search was started"
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert!(!report.has_more);
}

#[test]
fn a_class_symbol_declaration_keeps_the_unavailable_state() {
    // The query answers member declarations: its candidate rule compares a member shape, and a
    // class symbol names a type rather than a member. A caller that resolved a class and asks
    // for its references therefore gets the honest unavailable state — no scan, no read and no
    // candidate — instead of a shape the query cannot express.
    let world = fixture_world();
    let declaration = ResolvedMemberRef {
        loader: loader("app"),
        definition: world.definition(b"p/Base"),
        member: SymbolRef::Class {
            owner: bytes(b"p/Base"),
        },
    };
    let (report, budget) = world.query_with(
        declaration.clone(),
        &[ConsumerKind::Invocation],
        0,
        limits(),
    );

    assert_eq!(report.analysis, ResolutionAnalysis::NotPerformed);
    assert_eq!(report.declaration, declaration);
    assert!(report.items.is_empty());
    assert_eq!(report.unresolved_candidates, 0);
    assert!(!report.has_more);
    assert_eq!(report.returned_items, 0);
    assert!(report.reads.is_empty());
    assert_eq!(report.coverage, Coverage::not_requested());
    assert!(matches!(
        report.execution,
        ExecutionReport::Failed {
            reason: TerminationReason::Unsupported { .. },
            ..
        }
    ));
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_not_implemented"],
        "the report says what it did not do instead of claiming an empty answer"
    );
    assert_eq!(budget.usage().class_headers, 0, "nothing was read");
    assert_eq!(budget.usage().result_items, 0);
}

// ---------------------------------------------------------------------------
// Budgets and truncation
// ---------------------------------------------------------------------------
#[test]
fn a_refused_header_read_that_stops_the_resolution_keeps_the_reliable_prefix() {
    // Two header reads resolve the first candidate; the second one's owner search is refused,
    // so the prefix stays published and the candidate that could not be resolved is counted as
    // undecided — never as excluded.
    let world = two_subclass_world();
    let declaration = world.declaration(b"p/Base", b"foo", b"()V");
    let (report, _) = world.query_with(
        declaration.clone(),
        &[ConsumerKind::Invocation],
        0,
        Limits {
            class_headers: 2,
            ..limits()
        },
    );

    assert_eq!(
        report.items.len(),
        1,
        "the reliable prefix is kept: {:?}",
        report.items
    );
    assert_eq!(report.items[0].resolved.as_ref(), Some(&declaration));
    assert_eq!(report.unresolved_candidates, 1);
    assert_every_unresolved_candidate_keeps_its_use_site(&report);
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ClassHeaders
            },
            ..
        }
    ));
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert!(
        report
            .coverage
            .runtime_resolution
            .skipped
            .iter()
            .any(|range| range.label == "provider_search_position"),
        "the refused position is published as unreached: {:?}",
        report.coverage.runtime_resolution.skipped
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec![
            "budget_exceeded_class_headers",
            "resolution_candidate_unresolved"
        ],
        "the stop is reported and the candidate it hit keeps its use site"
    );
    assert_eq!(
        undecided_diagnostics(&report)[0]
            .provenance
            .as_ref()
            .expect("the candidate's use site")
            .location
            .clone(),
        Location::Code {
            method: PhysicalMethodId {
                owner: world.definition(b"p/SecondCaller"),
                name: bytes(b"run"),
                descriptor: bytes(b"()V"),
            },
            bci: 0,
        }
    );
}

#[test]
fn a_result_item_budget_that_cannot_fund_the_scan_stops_without_claiming_an_answer() {
    // Listing a ZIP charges one `ResultItems` per entry, and so does decoding one instruction
    // and publishing one candidate: this budget funds the listing and nothing else, so the scan
    // stops at its first work charge. Nothing is published and nothing is undecided — and the
    // report says exactly that instead of presenting the empty list as its whole answer.
    let world = two_subclass_world();
    let listing = {
        let mut budget = Budget::new(limits());
        u64::try_from(
            world
                .snapshot
                .enumerate(&mut budget)
                .expect("the fixture lists")
                .entries
                .len(),
        )
        .expect("the fixture entry count fits u64")
    };
    let declaration = world.declaration(b"p/Base", b"foo", b"()V");
    let (report, budget) = world.query_with(
        declaration,
        &[ConsumerKind::Invocation],
        0,
        Limits {
            result_items: listing,
            ..limits()
        },
    );

    assert!(report.items.is_empty());
    assert_eq!(report.returned_items, 0);
    assert_eq!(
        report.unresolved_candidates, 0,
        "a scan that stopped before its first candidate decided no candidate at all"
    );
    assert!(report.reads.is_empty());
    assert!(
        report.has_more,
        "the scan stopped before the end of its range, so the empty list is not the answer"
    );
    assert_eq!(
        budget.usage().result_items,
        listing,
        "the refused charge consumed nothing beyond the listing"
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
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["budget_exceeded_result_items"],
        "the stop names the dimension that refused the work"
    );
    assert!(
        matches!(
            location_of(
                report.diagnostics[0]
                    .provenance
                    .as_ref()
                    .expect("the stop names the entry it was reading")
            ),
            Location::Entry { .. }
        ),
        "no candidate was published, so the stop can only name the entry it was reading"
    );
}

#[test]
fn publishing_the_report_stops_at_the_budget_and_keeps_its_prefix() {
    // Publishing a report entry costs one `ResultItems`, exactly like a P1 item. A budget that
    // funds everything except the second entry therefore keeps the first one, reports `Partial`
    // on the resolution plane, and says there is more — while the artifact plane stays complete,
    // because the scan itself really did cover its range.
    let world = two_identical_callers_world();
    let declaration = world.declaration(b"p/Base", b"foo", b"()V");
    let (full_report, full_budget) = world.query_with(
        declaration.clone(),
        &[ConsumerKind::Invocation],
        0,
        limits(),
    );
    assert_eq!(full_report.items.len(), 2, "{:?}", full_report.items);
    let full = full_budget.usage().result_items;

    let (report, budget) = world.query_with(
        declaration.clone(),
        &[ConsumerKind::Invocation],
        0,
        Limits {
            result_items: full - 1,
            ..limits()
        },
    );

    assert_eq!(
        report.items.len(),
        1,
        "the reliable prefix is kept: {:?}",
        report.items
    );
    assert_eq!(report.returned_items, 1);
    assert_eq!(report.items[0].resolved.as_ref(), Some(&declaration));
    let (method, bci) = method_point(&report.items[0].origin);
    assert_eq!(
        method.owner,
        world.definition(b"p/FirstCaller"),
        "the published item keeps its complete physical use site"
    );
    assert_eq!(method.name, bytes(b"run"));
    assert_eq!(bci, 0);
    assert!(
        report.has_more,
        "the report did not publish everything it found"
    );
    assert_eq!(
        report.unresolved_candidates, 0,
        "the candidate the report could not publish was decided, so it is not undecided"
    );
    assert_eq!(
        budget.usage().result_items,
        full - 1,
        "one unit less than the whole run: the second publication is the one that was refused"
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
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial,
        "the report published a prefix, so the resolution plane is incomplete"
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema,
        "the scan itself covered its whole range"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["budget_exceeded_result_items"],
        "the stop names the dimension that refused the publication"
    );
    assert_eq!(
        location_of(
            report.diagnostics[0]
                .provenance
                .as_ref()
                .expect("the stop names the entry it could not publish")
        ),
        &Location::Code {
            method: PhysicalMethodId {
                owner: world.definition(b"p/SecondCaller"),
                name: bytes(b"run"),
                descriptor: bytes(b"()V"),
            },
            bci: 0,
        }
    );
}

#[test]
fn a_stop_that_hits_the_same_candidate_twice_is_reported_once() {
    // One unit less than the resolution of the first candidate needs: the resolution stops on
    // its last header charge and the publication of the very same candidate is refused too.
    // Both stops are the same fact at the same use site, so the report publishes it once.
    let world = two_identical_callers_world();
    let declaration = world.declaration(b"p/Base", b"foo", b"()V");
    let (full_report, full_budget) = world.query_with(
        declaration.clone(),
        &[ConsumerKind::Invocation],
        0,
        limits(),
    );
    assert_eq!(full_report.items.len(), 2);
    let full = full_budget.usage().result_items;

    let (report, budget) = world.query_with(
        declaration,
        &[ConsumerKind::Invocation],
        0,
        Limits {
            result_items: full - 3,
            ..limits()
        },
    );

    assert!(report.items.is_empty());
    assert_eq!(report.unresolved_candidates, 0);
    assert!(report.has_more);
    assert_eq!(budget.usage().result_items, full - 3);
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
        vec!["budget_exceeded_result_items"],
        "the resolution stop and the refused publication are the same fact at the same use site"
    );
    assert_eq!(
        location_of(
            report.diagnostics[0]
                .provenance
                .as_ref()
                .expect("the stop names the candidate it stopped on")
        ),
        &Location::Code {
            method: PhysicalMethodId {
                owner: world.definition(b"p/FirstCaller"),
                name: bytes(b"run"),
                descriptor: bytes(b"()V"),
            },
            bci: 0,
        }
    );
}

#[test]
fn a_published_diagnostic_costs_a_result_item() {
    // Two worlds that do the same work and read the same headers, and differ only in what they
    // publish: one candidate resolves (one item, no diagnostic), the other is refused by the
    // invocation-kind rule (no item, two diagnostics). One unit of `ResultItems` is charged per
    // published entry — item or diagnostic — so the ICCE world costs exactly one unit more, and
    // the shared work costs the same in both.
    let ok_world = single_candidate_world();
    let icce_world = kind_mismatch_world();
    let declaration = |world: &World| world.declaration(b"p/Base", b"foo", b"()V");
    let (ok, ok_budget) = ok_world.query_with(
        declaration(&ok_world),
        &[ConsumerKind::Invocation],
        0,
        limits(),
    );
    let (icce, icce_budget) = icce_world.query_with(
        declaration(&icce_world),
        &[ConsumerKind::Invocation],
        0,
        limits(),
    );

    assert_eq!(ok.items.len(), 1);
    assert_eq!(diagnostic_codes(&ok), Vec::<&str>::new());
    assert_eq!(icce.items.len(), 0);
    assert_eq!(
        diagnostic_codes(&icce),
        vec![
            "resolution_kind_mismatch",
            "resolution_candidate_unresolved"
        ],
        "the refusal and the undecided candidate are two entries"
    );

    // The same work: listing, decoded instructions, scanned candidates and class headers.
    assert_eq!(
        ok_budget.usage().archive_entries,
        icce_budget.usage().archive_entries
    );
    assert_eq!(ok_budget.usage().code_bytes, icce_budget.usage().code_bytes);
    assert_eq!(ok_budget.usage().read_bytes, icce_budget.usage().read_bytes);
    assert_eq!(
        ok_budget.usage().class_headers,
        icce_budget.usage().class_headers
    );

    // One unit per published entry, on top of the identical shared work.
    let published = |report: &DeclarationRefReport| {
        u64::try_from(report.items.len() + report.diagnostics.len())
            .expect("a fixture count fits u64")
    };
    let ok_work = ok_budget.usage().result_items - published(&ok);
    let icce_work = icce_budget.usage().result_items - published(&icce);
    assert_eq!(
        icce_work, ok_work,
        "the two worlds do the same work, so what is left after their published entries must be \
         the same number of units"
    );
    assert_eq!(
        icce_budget.usage().result_items - ok_budget.usage().result_items,
        published(&icce) - published(&ok),
        "the worlds differ by exactly the entries they publish: {} vs {}",
        published(&icce),
        published(&ok)
    );
    assert_eq!(
        ok_budget.usage().result_items,
        ok_work + 1,
        "the OK world published one item"
    );
    assert_eq!(
        icce_budget.usage().result_items,
        icce_work + 2,
        "the ICCE world published two diagnostics, each charged one unit"
    );
}

#[test]
fn a_refused_diagnostic_charge_stops_the_report_at_the_prefix() {
    // The same ICCE world under a budget that funds the shared work plus exactly one published
    // diagnostic: the rule diagnostic enters the report, the undecided diagnostic of the same
    // candidate is refused, and the report says so. The candidate is then neither an item nor a
    // count — the use site survives in the stop diagnostic — and one unit less publishes
    // nothing at all.
    let world = kind_mismatch_world();
    let declaration = world.declaration(b"p/Base", b"foo", b"()V");
    let (full, full_budget) = world.query_with(
        declaration.clone(),
        &[ConsumerKind::Invocation],
        0,
        limits(),
    );
    assert_eq!(full.unresolved_candidates, 1);
    let work = full_budget.usage().result_items - 2;

    let (report, budget) = world.query_with(
        declaration.clone(),
        &[ConsumerKind::Invocation],
        0,
        Limits {
            result_items: work + 1,
            ..limits()
        },
    );

    assert!(report.items.is_empty());
    assert_eq!(
        report.unresolved_candidates, 0,
        "the candidate the report could not count keeps no count"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_kind_mismatch", "budget_exceeded_result_items"],
        "the rule diagnostic is published, the undecided one is refused, and the stop explains \
         itself"
    );
    assert!(report.has_more);
    assert_eq!(
        budget.usage().result_items,
        work + 1,
        "the refused charge consumed nothing"
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
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema,
        "the scan itself covered its whole range"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial,
        "the assembly stopped, so the resolution plane is incomplete"
    );
    assert_eq!(
        location_of(
            report
                .diagnostics
                .last()
                .and_then(|diagnostic| diagnostic.provenance.as_ref())
                .expect("the stop names the candidate it stopped on")
        ),
        &Location::Code {
            method: PhysicalMethodId {
                owner: world.definition(b"p/Caller"),
                name: bytes(b"run"),
                descriptor: bytes(b"()V"),
            },
            bci: 0,
        }
    );

    // The boundary one unit below: the first diagnostic itself is refused, so the report
    // publishes no diagnostic of the resolution at all and still explains the stop.
    let (report, budget) = world.query_with(
        declaration,
        &[ConsumerKind::Invocation],
        0,
        Limits {
            result_items: work,
            ..limits()
        },
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["budget_exceeded_result_items"]
    );
    assert_eq!(report.unresolved_candidates, 0);
    assert!(report.has_more);
    assert_eq!(budget.usage().result_items, work);
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ResultItems
            },
            ..
        }
    ));
}

#[test]
fn max_items_truncates_the_items_without_turning_the_execution_partial() {
    let world = two_subclass_world();
    let declaration = world.declaration(b"p/Base", b"foo", b"()V");
    let report = world.query(declaration.clone(), &[ConsumerKind::Invocation], 1);

    assert_eq!(report.items.len(), 1);
    assert_eq!(report.returned_items, 1);
    assert!(report.has_more);
    assert_eq!(report.unresolved_candidates, 0);
    assert!(
        matches!(report.execution, ExecutionReport::Complete { .. }),
        "a page limit is not an interruption: {:?}",
        report.execution
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial,
        "the scan really stopped before the end of its range"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial,
        "a truncated scan may hold candidates no resolution has seen, so this plane is partial too"
    );
}

// ---------------------------------------------------------------------------
// Two loaders, one name
// ---------------------------------------------------------------------------

/// Two worlds whose loaders root different snapshots holding the same class names.
///
/// Each loader declares exactly one root, so the effective order of one environment contains
/// only its own snapshot: a query asked under `app` resolves `p/Base` to the app definition
/// even though a class of the same name lives in the platform snapshot.
fn two_loader_worlds() -> (World, World) {
    let app_classes = vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC),
        Class::new(b"p/Sub").super_class(b"p/Base"),
        Class::new(b"p/Caller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/Sub", b"foo", b"()V"), Insn::Return],
        ),
    ];
    // The platform's own `p/Base` declares one more method, so the two definitions differ in
    // their bytes as well as in their position.
    let platform_classes = vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base")
            .method(b"foo", b"()V", PUBLIC)
            .method(b"extra", b"()V", PUBLIC),
        Class::new(b"p/Sub").super_class(b"p/Base"),
        Class::new(b"p/Caller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/Sub", b"foo", b"()V"), Insn::Return],
        ),
    ];
    let app_classes = pairs(&app_classes);
    let platform_classes = pairs(&platform_classes);
    let app_snapshot = open(&app_classes);
    let platform_snapshot = open(&platform_classes);
    let app = domain(&loader("app"), vec![snapshot_root(&app_snapshot)]);
    let platform = domain(&loader("platform"), vec![snapshot_root(&platform_snapshot)]);
    let app_environment = environment(&app_snapshot, app.clone(), vec![app]);
    let platform_environment = environment(&platform_snapshot, platform.clone(), vec![platform]);
    let content = vec![app_snapshot.clone(), platform_snapshot.clone()];
    (
        World {
            classes: app_classes,
            snapshot: app_snapshot,
            environment: app_environment,
            content: content.clone(),
        },
        World {
            classes: platform_classes,
            snapshot: platform_snapshot,
            environment: platform_environment,
            content,
        },
    )
}

#[test]
fn the_same_names_under_two_loaders_resolve_under_the_requested_environment() {
    let (app, platform) = two_loader_worlds();
    let app_declaration = app.declaration(b"p/Base", b"foo", b"()V");
    let platform_declaration = platform.declaration(b"p/Base", b"foo", b"()V");
    assert_ne!(
        app_declaration.definition, platform_declaration.definition,
        "the two snapshots really hold different definitions"
    );

    let app_report = app.query(app_declaration.clone(), &[ConsumerKind::Invocation], 0);
    assert_eq!(app_report.items.len(), 1);
    assert_eq!(
        app_report.items[0].resolved.as_ref(),
        Some(&app_declaration),
        "the app query resolves under the app loader"
    );
    assert_eq!(
        app_report
            .items
            .iter()
            .map(|item| item
                .resolved
                .as_ref()
                .map(|resolved| resolved.loader.clone()))
            .collect::<Vec<_>>(),
        vec![Some(loader("app"))]
    );

    let platform_report =
        platform.query(platform_declaration.clone(), &[ConsumerKind::Invocation], 0);
    assert_eq!(platform_report.items.len(), 1);
    assert_eq!(
        platform_report.items[0].resolved.as_ref(),
        Some(&platform_declaration),
        "the platform query resolves under the platform loader"
    );
    assert_eq!(
        platform_report
            .items
            .iter()
            .map(|item| item
                .resolved
                .as_ref()
                .map(|resolved| resolved.loader.clone()))
            .collect::<Vec<_>>(),
        vec![Some(loader("platform"))]
    );

    // The same query under the other environment is decided, and decided against: the app
    // resolution of `p/Base` is not merged with the platform definition the query asked about.
    let crossed = app.query(platform_declaration, &[ConsumerKind::Invocation], 0);
    assert!(crossed.items.is_empty());
    assert_eq!(
        crossed.unresolved_candidates, 0,
        "the candidate resolved to another definition; that is a decision"
    );
    assert!(
        !crossed.reads.is_empty(),
        "the crossed query still resolved the candidate it found"
    );
}

// ---------------------------------------------------------------------------
// One definition, two loaders: the loader axis of the comparison
// ---------------------------------------------------------------------------

/// One snapshot rooted by two loaders with different delegation policies.
///
/// Both domains see the *same* physical definitions, so the only thing that can tell two
/// resolutions apart is the loader a declaration was read under — which is why this world
/// makes the loader component of the comparison falsifiable, unlike the two-snapshot world
/// above.
struct TwoLoaders {
    world: World,
    app: ResolutionEnvironment,
    other: ResolutionEnvironment,
}

impl TwoLoaders {
    fn new() -> Self {
        let classes = pairs(&[
            Class::root(b"java/lang/Object"),
            Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC),
            Class::new(b"p/Sub").super_class(b"p/Base"),
            Class::new(b"p/Caller").method_with_body(
                b"run",
                b"()V",
                PUBLIC,
                vec![invoke_virtual(b"p/Sub", b"foo", b"()V"), Insn::Return],
            ),
        ]);
        let snapshot = open(&classes);
        let app = domain_with(
            &loader("app"),
            vec![snapshot_root(&snapshot)],
            DelegationPolicy::ParentFirst,
        );
        let other = domain_with(
            &loader("other"),
            vec![snapshot_root(&snapshot)],
            DelegationPolicy::ChildFirst,
        );
        let runtime = [app.clone(), other.clone()];
        let app_environment = environment(&snapshot, app, runtime.to_vec());
        let other_environment = environment(&snapshot, other, runtime.to_vec());
        Self {
            world: World::from_pairs(classes),
            app: app_environment,
            other: other_environment,
        }
    }
}

#[test]
fn the_loader_of_a_declaration_is_part_of_its_identity() {
    let two = TwoLoaders::new();
    let definition = two.world.definition(b"p/Base");
    let member = SymbolRef::Method {
        owner: bytes(b"p/Base"),
        name: bytes(b"foo"),
        descriptor: bytes(b"()V"),
    };
    let app_declaration = ResolvedMemberRef {
        loader: loader("app"),
        definition: definition.clone(),
        member: member.clone(),
    };
    let other_declaration = ResolvedMemberRef {
        loader: loader("other"),
        definition: definition.clone(),
        member: member.clone(),
    };
    assert_eq!(
        app_declaration.definition, other_declaration.definition,
        "both loaders root the same snapshot, so the definitions are the same one"
    );

    // The caller's own domain is the search start, so the same candidate resolves under the
    // loader of the environment it was asked under.
    let app_report = two.world.query_under(
        &two.app,
        app_declaration.clone(),
        &[ConsumerKind::Invocation],
    );
    assert_eq!(app_report.items.len(), 1, "{:?}", app_report.items);
    assert_eq!(
        app_report.items[0].resolved.as_ref(),
        Some(&app_declaration),
        "the app environment resolves the declaration under the app loader"
    );

    let other_report = two.world.query_under(
        &two.other,
        other_declaration.clone(),
        &[ConsumerKind::Invocation],
    );
    assert_eq!(other_report.items.len(), 1, "{:?}", other_report.items);
    assert_eq!(
        other_report.items[0].resolved.as_ref(),
        Some(&other_declaration),
        "the other environment resolves the same definition under its own loader"
    );
    assert_eq!(
        other_report.items[0]
            .resolved
            .as_ref()
            .map(|resolved| resolved.loader.clone()),
        Some(loader("other"))
    );

    // Crossed, the candidate resolves to another loader's declaration of the same definition:
    // decided, not published, and not undecided.
    let crossed = two.world.query_under(
        &two.app,
        other_declaration.clone(),
        &[ConsumerKind::Invocation],
    );
    assert!(crossed.items.is_empty());
    assert_eq!(
        crossed.unresolved_candidates, 0,
        "the candidate resolved to a declaration of another loader, which is a decision"
    );
    assert!(
        !crossed.reads.is_empty(),
        "the crossed query still resolved the candidate it found"
    );
    assert_eq!(
        crossed.coverage.runtime_resolution.state,
        CoverageState::CompleteWithinSchema
    );
}

// ---------------------------------------------------------------------------
// Every undecided source stays undecided
// ---------------------------------------------------------------------------

#[test]
fn a_duplicate_declaration_is_unresolved() {
    // The owner declares `foo:()V` twice, so no single declaration can be selected: the
    // candidate is undecided, keeps its use site and leaves the resolution plane partial.
    let world = World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Dup")
            .method(b"foo", b"()V", PUBLIC)
            .method(b"foo", b"()V", PUBLIC),
        Class::new(b"p/Sub").super_class(b"p/Dup"),
        Class::new(b"p/Caller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/Sub", b"foo", b"()V"), Insn::Return],
        ),
    ]);
    let report = world.query(
        world.declaration(b"p/Dup", b"foo", b"()V"),
        &[ConsumerKind::Invocation],
        0,
    );

    assert!(report.items.is_empty());
    assert_eq!(report.unresolved_candidates, 1);
    assert_every_unresolved_candidate_keeps_its_use_site(&report);
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_candidate_unresolved"],
        "the undecided candidate is reported at its use site"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
}

#[test]
fn a_kind_mismatch_is_unresolved() {
    // The declaration is static and the site calls it virtually: 2.3 answers
    // `IncompatibleClassChange`, which is not the queried declaration, so the candidate is
    // undecided and both diagnostics are published.
    let world = World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC | STATIC),
        Class::new(b"p/Sub").super_class(b"p/Base"),
        Class::new(b"p/Caller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/Sub", b"foo", b"()V"), Insn::Return],
        ),
    ]);
    let report = world.query(
        world.declaration(b"p/Base", b"foo", b"()V"),
        &[ConsumerKind::Invocation],
        0,
    );

    assert!(report.items.is_empty());
    assert_eq!(report.unresolved_candidates, 1);
    assert_every_unresolved_candidate_keeps_its_use_site(&report);
    assert_eq!(
        diagnostic_codes(&report),
        vec![
            "resolution_kind_mismatch",
            "resolution_candidate_unresolved"
        ],
        "the rule that refused the reference and the fact that the candidate stayed undecided"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
}

#[test]
fn an_array_owner_is_unresolved() {
    // The only candidate names an array type as its owner, which this engine does not resolve:
    // 2.3 answers `UnsupportedPolicy` and the candidate stays undecided.
    let world = World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base"),
        Class::new(b"p/Caller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![
                invoke_virtual(b"[I", b"clone", b"()Ljava/lang/Object;"),
                Insn::Return,
            ],
        ),
    ]);
    let report = world.query(
        world.declaration(b"p/Base", b"clone", b"()Ljava/lang/Object;"),
        &[ConsumerKind::Invocation],
        0,
    );

    assert!(report.items.is_empty());
    assert_eq!(report.unresolved_candidates, 1);
    assert_every_unresolved_candidate_keeps_its_use_site(&report);
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_array_owner", "resolution_candidate_unresolved"]
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
}

#[test]
fn a_cancelled_request_scans_nothing_and_never_claims_an_answer() {
    // The request is cancelled before it starts: the listing stops before the first entry, so
    // the scan has no candidate at all — nothing is undecided, nothing is published, and the
    // report says what happened instead of presenting an empty answer as the whole truth.
    let world = single_candidate_world();
    let query = DeclarationRefQuery {
        environment: world.environment.clone(),
        declaration: world.declaration(b"p/Base", b"foo", b"()V"),
        scope: PhysicalScope::SnapshotAll,
        consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
        max_items: 0,
    };
    let mut budget = Budget::new(limits());
    let token = budget.cancellation_token();
    token.cancel();
    let report = Engine::new()
        .declaration_references(&world.content, &query, &mut budget)
        .expect("a legal query is answered, not raised");

    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert!(report.items.is_empty());
    assert_eq!(report.unresolved_candidates, 0);
    assert!(
        report.has_more,
        "the run stopped before the end of its range"
    );
    assert!(report.reads.is_empty());
    assert!(matches!(
        report.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert_eq!(diagnostic_codes(&report), vec!["cancelled"]);
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
}

#[test]
fn a_damaged_class_candidate_is_a_scan_level_stop() {
    // One entry is named like a class but does not hold one. That is a damaged candidate, not a
    // candidate the query failed to decide: the scan stops there, keeps the candidate it had
    // already published, reports the failure and counts nothing as undecided.
    let mut classes = pairs(&[
        Class::root(b"java/lang/Object"),
        Class::new(b"p/Base").method(b"foo", b"()V", PUBLIC),
        Class::new(b"p/Sub").super_class(b"p/Base"),
        Class::new(b"p/Caller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/Sub", b"foo", b"()V"), Insn::Return],
        ),
    ]);
    classes.push((entry(b"p/Damaged"), b"not a class file".to_vec()));
    let world = World::from_pairs(classes);
    let declaration = world.declaration(b"p/Base", b"foo", b"()V");
    let report = world.query(declaration.clone(), &[ConsumerKind::Invocation], 0);

    assert_eq!(
        report.items.len(),
        1,
        "the candidate published before the damaged entry stays: {:?}",
        report.items
    );
    assert_eq!(report.items[0].resolved.as_ref(), Some(&declaration));
    assert_eq!(
        report.unresolved_candidates, 0,
        "a damaged candidate is a scan-level stop, not an undecided candidate"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["query_class_candidate_malformed"],
        "the failure names the entry it found"
    );
    assert!(report.has_more);
    assert!(matches!(
        report.execution,
        ExecutionReport::Failed {
            reason: TerminationReason::Error { ref code },
            ..
        } if code == "query_class_candidate_malformed"
    ));
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial,
        "a scan that stopped may hold candidates no resolution has seen"
    );
}

#[test]
fn a_rejected_environment_never_reaches_the_candidates() {
    // An environment problem is decided before any candidate is looked at — an external root is
    // declared but unreadable — so the query is not performed at all: it publishes the
    // environment problems, reads nothing and counts no candidate. An environment problem found
    // *during* a search is a different thing (it stops the candidate that hit it, see
    // `a_missing_owner_is_unresolved_and_keeps_its_use_site`); a rejected environment never
    // becomes a candidate-level undecided.
    let world = single_candidate_world();
    let external = LoadRoot::External {
        id: "host-jdk".to_string(),
    };
    let app = domain(&loader("app"), vec![external]);
    let environment = environment(&world.snapshot, app.clone(), vec![app]);
    let declaration = world.declaration(b"p/Base", b"foo", b"()V");

    let report = world.query_under(&environment, declaration, &[ConsumerKind::Invocation]);

    assert_eq!(report.analysis, ResolutionAnalysis::NotPerformed);
    assert_eq!(
        report
            .environment_problems
            .iter()
            .map(|problem| problem.code.as_str())
            .collect::<Vec<_>>(),
        vec!["unreadable_root"]
    );
    assert!(report.items.is_empty());
    assert_eq!(report.unresolved_candidates, 0);
    assert!(!report.has_more);
    assert_eq!(report.returned_items, 0);
    assert!(report.reads.is_empty());
    assert_eq!(report.coverage, Coverage::not_requested());
    assert_eq!(
        diagnostic_codes(&report),
        vec!["unreadable_root", "resolution_not_implemented"],
        "the environment problem and the capability state it leaves"
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Failed {
            reason: TerminationReason::Unsupported { .. },
            ..
        }
    ));
}

#[test]
fn a_search_diagnostic_enters_the_report_and_is_charged() {
    // The searched hierarchy is cyclic (`p/CycA extends p/CycB extends p/CycA`), which the member
    // search refuses to walk further and reports itself. That diagnostic is a resolution result:
    // it enters the report, the candidate whose search met the cycle stays undecided, and both
    // entries cost one `ResultItems`. One unit short of the run, the *second* entry is the one
    // that is refused — the diagnostic that explains why the search stopped is still published.
    let world = World::single(vec![
        Class::root(b"java/lang/Object"),
        Class::new(b"p/CycA").super_class(b"p/CycB"),
        Class::new(b"p/CycB").super_class(b"p/CycA"),
        Class::new(b"p/Caller").method_with_body(
            b"run",
            b"()V",
            PUBLIC,
            vec![invoke_virtual(b"p/CycA", b"foo", b"()V"), Insn::Return],
        ),
    ]);
    let declaration = world.declaration(b"p/CycA", b"foo", b"()V");
    let (report, full_budget) = world.query_with(
        declaration.clone(),
        &[ConsumerKind::Invocation],
        0,
        limits(),
    );

    assert!(report.items.is_empty());
    assert_eq!(report.unresolved_candidates, 1);
    assert_every_unresolved_candidate_keeps_its_use_site(&report);
    assert_eq!(
        diagnostic_codes(&report),
        vec![
            "resolution_hierarchy_cycle",
            "resolution_candidate_unresolved"
        ],
        "the cycle the search refused is reported before the candidate it left open"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );

    let (report, short_budget) = world.query_with(
        declaration,
        &[ConsumerKind::Invocation],
        0,
        Limits {
            result_items: full_budget.usage().result_items - 1,
            ..limits()
        },
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["resolution_hierarchy_cycle", "budget_exceeded_result_items"],
        "the search's own diagnostic is published, and the undecided entry is the one the budget \
         refused"
    );
    assert_eq!(
        report.unresolved_candidates, 0,
        "the candidate whose entry was refused keeps no count"
    );
    assert!(report.has_more);
    assert_eq!(
        short_budget.usage().result_items,
        full_budget.usage().result_items - 1
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema,
        "the scan itself covered its whole range"
    );
}

// ---------------------------------------------------------------------------
// The scope a query declares
// ---------------------------------------------------------------------------

#[test]
fn an_artifact_tree_scope_validates_its_root_container() {
    // The only root a snapshot establishes is its own root container. A scope that names
    // another one cannot describe this snapshot, so it is refused instead of being accepted and
    // silently ignored — and the real root really does scan.
    let world = single_candidate_world();
    let declaration = world.declaration(b"p/Base", b"foo", b"()V");

    let error = world
        .query_scope(
            declaration.clone(),
            &[ConsumerKind::Invocation],
            PhysicalScope::ArtifactTree {
                root_container: ContainerId("bogus-container".to_string()),
            },
        )
        .expect_err("an unknown tree root is a request mismatch");
    assert_eq!(
        error_code(&error),
        Some("query_artifact_tree_root_mismatch")
    );

    // The comparison is byte equality on the raw container name, like the P1 query's: a root
    // that differs only in case is a different root, not the one this snapshot established.
    let error = world
        .query_scope(
            declaration.clone(),
            &[ConsumerKind::Invocation],
            PhysicalScope::ArtifactTree {
                root_container: ContainerId("Root".to_string()),
            },
        )
        .expect_err("the root name is compared as written, not case-insensitively");
    assert_eq!(
        error_code(&error),
        Some("query_artifact_tree_root_mismatch")
    );

    let report = world
        .query_scope(
            declaration.clone(),
            &[ConsumerKind::Invocation],
            PhysicalScope::ArtifactTree {
                root_container: ContainerId("root".to_string()),
            },
        )
        .expect("the snapshot's own root is a legal scope");
    assert_eq!(report.items.len(), 1, "{:?}", report.items);
    assert_eq!(report.items[0].resolved.as_ref(), Some(&declaration));
    assert_eq!(
        report.scope,
        PhysicalScope::ArtifactTree {
            root_container: ContainerId("root".to_string()),
        }
    );
}

/// The stable code of one raised error, when it carries one.
fn error_code(error: &Error) -> Option<&str> {
    match error {
        Error::InvalidInput { code, .. } | Error::Unsupported { code, .. } => Some(code.as_str()),
        Error::Cancelled { .. } | Error::BudgetExceeded { .. } | Error::Io { .. } => None,
    }
}

#[test]
fn a_resolution_stop_outranks_an_assembly_stop() {
    // Both stops at once, with the resolution stop first: the second candidate's owner search is
    // refused by `AnalysisSteps`, and one unit short of the full run the diagnostic that would
    // report the candidate as undecided is refused too. The report names the stop that really
    // limited the resolution, and states the truncation through `has_more` and the coverage
    // planes rather than through the execution.
    let world = two_subclass_world();
    let declaration = world.declaration(b"p/Base", b"foo", b"()V");
    let two_steps = || Limits {
        analysis_steps: 2,
        ..limits()
    };
    let (full, full_budget) = world.query_with(
        declaration.clone(),
        &[ConsumerKind::Invocation],
        0,
        two_steps(),
    );
    assert_eq!(
        full.items.len(),
        1,
        "one candidate resolves under two steps"
    );
    assert_eq!(full.unresolved_candidates, 1);

    let (report, budget) = world.query_with(
        declaration,
        &[ConsumerKind::Invocation],
        0,
        Limits {
            result_items: full_budget.usage().result_items - 1,
            ..two_steps()
        },
    );

    assert_eq!(report.items.len(), 1, "the reliable prefix is kept");
    assert_eq!(
        report.unresolved_candidates, 0,
        "the candidate whose diagnostic was refused keeps no count"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec![
            "budget_exceeded_analysis_steps",
            "budget_exceeded_result_items"
        ],
        "the resolution stop is explained, and so is the publication the same run could not pay          for"
    );
    assert!(report.has_more);
    assert_eq!(
        budget.usage().result_items,
        full_budget.usage().result_items - 1
    );
    assert!(
        matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps
                },
                ..
            }
        ),
        "the stop that came first is the one the execution reports: {:?}",
        report.execution
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema,
        "the scan itself covered its whole range"
    );
    assert_eq!(
        report.coverage.runtime_resolution.state,
        CoverageState::Partial
    );
}
