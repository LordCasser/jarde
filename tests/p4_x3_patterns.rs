//! P4 2.3 acceptance: the bounded X3 reflection and `ServiceLoader` patterns.
//!
//! What this file proves through the public API is the separation the design's decision 3 asks for
//! — evidence and assumptions a reviewer can check, instead of one confidence number:
//!
//! 1. **a constant target is an inference with five elements.** `Class.forName` of a string the
//!    method's own bytes hold is `pattern_inferred_target`, and its [`PatternInference`] carries
//!    exactly the overload's rule, the constant input, the bounded propagation scope, the loader
//!    assumption and the rule version — destructured exhaustively here, so a conclusion cannot be
//!    published without one of them, and the serde spelling is pinned to the requirement's own
//!    label;
//! 2. **a dynamic input is `Unknown`, never a guess.** A parameter, a merge of two branches and a
//!    constant of *another* class are three different definitions, and each site names the one it
//!    came from instead of a made-up target;
//! 3. **the overload decides the conclusion.** `Class.forName(String)` and
//!    `Class.forName(String, boolean, ClassLoader)` read the *same* constant and state different
//!    loader assumptions — one looks the name up in the caller's own order, the other names a
//!    loader this snapshot does not identify. `ServiceLoader.load(Class)` uses the thread context
//!    loader, and a member lookup states a name and no member;
//! 4. **the propagation is bounded by one body and shows its path.** A constant stored in a local
//!    and loaded again is proved through its SSA value, and the path names the source, the load it
//!    was carried through and the call site — all of them inside that one method;
//! 5. **nothing is executed, loaded or decoded.** A name the snapshot does not hold is still
//!    inferred (and published with the snapshot's *separate* answer beside it, as a missing
//!    dependency when no position provides it); a member name the target class does not declare is
//!    still the name the site asks for; the `dynamic_analysis` plane stays `not_requested`; and
//!    the same inferences come out of a snapshot that does not contain the target class at all;
//! 6. **a budget stop is a stop.** With one analysis step the scan reports the exhausted dimension,
//!    marks its answer as a prefix and publishes no inferred site at all — a stop never becomes a
//!    wrong answer.
//!
//! Fixtures are built from one class-file writer that emits real bytecode; no framework, and no
//! dependency beyond the ZIP writer and serde the other P4 suites already use.

use jarde::reflection::{
    ConstantSourceKind, DynamicOrigin, LoaderAssumptionKind, PatternInference, PatternInput,
    PatternTargetState, PatternUnknown, ReflectedMemberKind, ReflectedTarget,
    ReflectionPatternReport, ReflectionPatternRequest, ReflectionSite, ReflectionSiteState,
    RuleVersion, patterns,
};
use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const STORE: u16 = 0;
const PUBLIC: u16 = 0x0001;
const STATIC: u16 = 0x0008;
const FINAL: u16 = 0x0010;
const MAJOR: u16 = 52;

/// The one-argument `Class.forName` overload's descriptor.
const FOR_NAME: &str = "(Ljava/lang/String;)Ljava/lang/Class;";
/// The three-argument `Class.forName` overload's descriptor: another rule, another loader.
const FOR_NAME_LOADER: &str = "(Ljava/lang/String;ZLjava/lang/ClassLoader;)Ljava/lang/Class;";
/// `Class.getDeclaredMethod`, whose owner is the *receiver*.
const GET_DECLARED_METHOD: &str =
    "(Ljava/lang/String;[Ljava/lang/Class;)Ljava/lang/reflect/Method;";
/// `Lookup.findStatic`, whose owner and name are two arguments.
const FIND_STATIC: &str = "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/MethodHandle;";
/// `ServiceLoader.load(Class)`, whose argument is a class literal.
const SERVICE_LOAD: &str = "(Ljava/lang/Class;)Ljava/util/ServiceLoader;";
/// `Class.getDeclaredMethods()`, registered as unsupported.
const GET_DECLARED_METHODS: &str = "()[Ljava/lang/reflect/Method;";
/// `Class.getSuperclass()`, an overload this registry does not hold.
const GET_SUPERCLASS: &str = "()Ljava/lang/Class;";

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
        method_bodies: 64,
        ir_items: 1_000_000,
        ir_edges: 1_000_000,
        normalization_clones: 10_000,
        nested_depth: 4,
        dependency_depth: 8,
        analysis_steps: 100_000,
        elapsed_millis: u64::MAX,
    }
}

/// One constant pool, built by interning: an entry is added once and its 1-based index returned.
#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    fn intern(&mut self, tag: u8, payload: &[u8]) -> u16 {
        for (position, entry) in self.entries.iter().enumerate() {
            if entry.first() == Some(&tag) && entry.get(1..) == Some(payload) {
                return u16::try_from(position + 1).expect("the fixture pool fits u16");
            }
        }
        let mut entry = vec![tag];
        entry.extend_from_slice(payload);
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("the fixture pool fits u16")
    }

    fn utf8(&mut self, value: &[u8]) -> u16 {
        let mut payload = u16::try_from(value.len())
            .expect("a fixture name fits u16")
            .to_be_bytes()
            .to_vec();
        payload.extend_from_slice(value);
        self.intern(1, &payload)
    }

    fn class(&mut self, name: &[u8]) -> u16 {
        let index = self.utf8(name);
        self.intern(7, &index.to_be_bytes())
    }

    fn string(&mut self, value: &[u8]) -> u16 {
        let index = self.utf8(value);
        self.intern(8, &index.to_be_bytes())
    }

    fn name_and_type(&mut self, name: &[u8], descriptor: &[u8]) -> u16 {
        let name = self.utf8(name);
        let descriptor = self.utf8(descriptor);
        let mut payload = name.to_be_bytes().to_vec();
        payload.extend_from_slice(&descriptor.to_be_bytes());
        self.intern(12, &payload)
    }

    fn member_ref(&mut self, tag: u8, owner: &[u8], name: &[u8], descriptor: &[u8]) -> u16 {
        let owner = self.class(owner);
        let name_and_type = self.name_and_type(name, descriptor);
        let mut payload = owner.to_be_bytes().to_vec();
        payload.extend_from_slice(&name_and_type.to_be_bytes());
        self.intern(tag, &payload)
    }

    fn method_ref(&mut self, owner: &[u8], name: &[u8], descriptor: &[u8]) -> u16 {
        self.member_ref(10, owner, name, descriptor)
    }

    fn field_ref(&mut self, owner: &[u8], name: &[u8], descriptor: &[u8]) -> u16 {
        self.member_ref(9, owner, name, descriptor)
    }
}

/// One fixture class file: its pool, its fields and its methods, with real `Code` attributes.
struct ClassFile {
    name: Vec<u8>,
    pool: Pool,
    this_class: u16,
    super_class: u16,
    access_flags: u16,
    code_name: u16,
    constant_value_name: u16,
    fields: Vec<(u16, u16, u16, Option<u16>)>,
    methods: Vec<(u16, u16, u16, u16, u16, Vec<u8>)>,
}

impl ClassFile {
    fn new(name: &[u8]) -> Self {
        let mut pool = Pool::default();
        let this_class = pool.class(name);
        let super_class = pool.class(b"java/lang/Object");
        let code_name = pool.utf8(b"Code");
        let constant_value_name = pool.utf8(b"ConstantValue");
        Self {
            name: name.to_vec(),
            pool,
            this_class,
            super_class,
            access_flags: PUBLIC | 0x0020,
            code_name,
            constant_value_name,
            fields: Vec::new(),
            methods: Vec::new(),
        }
    }

    fn name(&self) -> Vec<u8> {
        self.name.clone()
    }

    fn field(&mut self, name: &[u8], descriptor: &[u8], flags: u16, constant: Option<u16>) {
        let name = self.pool.utf8(name);
        let descriptor = self.pool.utf8(descriptor);
        self.fields.push((flags, name, descriptor, constant));
    }

    fn method(
        &mut self,
        name: &[u8],
        descriptor: &[u8],
        flags: u16,
        max_stack: u16,
        max_locals: u16,
        code: Vec<u8>,
    ) {
        let name = self.pool.utf8(name);
        let descriptor = self.pool.utf8(descriptor);
        self.methods
            .push((flags, name, descriptor, max_stack, max_locals, code));
    }

    fn build(&self) -> Vec<u8> {
        let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // minor
        bytes.extend_from_slice(&MAJOR.to_be_bytes());
        bytes.extend_from_slice(
            &u16::try_from(self.pool.entries.len() + 1)
                .expect("the fixture pool count fits u16")
                .to_be_bytes(),
        );
        for entry in &self.pool.entries {
            bytes.extend_from_slice(entry);
        }
        bytes.extend_from_slice(&self.access_flags.to_be_bytes());
        bytes.extend_from_slice(&self.this_class.to_be_bytes());
        bytes.extend_from_slice(&self.super_class.to_be_bytes());
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // interfaces
        bytes.extend_from_slice(
            &u16::try_from(self.fields.len())
                .expect("the fixture field count fits u16")
                .to_be_bytes(),
        );
        for (flags, name, descriptor, constant) in &self.fields {
            bytes.extend_from_slice(&flags.to_be_bytes());
            bytes.extend_from_slice(&name.to_be_bytes());
            bytes.extend_from_slice(&descriptor.to_be_bytes());
            match constant {
                None => bytes.extend_from_slice(&0_u16.to_be_bytes()),
                Some(index) => {
                    bytes.extend_from_slice(&1_u16.to_be_bytes());
                    bytes.extend_from_slice(&self.constant_value_name.to_be_bytes());
                    bytes.extend_from_slice(&2_u32.to_be_bytes());
                    bytes.extend_from_slice(&index.to_be_bytes());
                }
            }
        }
        bytes.extend_from_slice(
            &u16::try_from(self.methods.len())
                .expect("the fixture method count fits u16")
                .to_be_bytes(),
        );
        for (flags, name, descriptor, max_stack, max_locals, code) in &self.methods {
            bytes.extend_from_slice(&flags.to_be_bytes());
            bytes.extend_from_slice(&name.to_be_bytes());
            bytes.extend_from_slice(&descriptor.to_be_bytes());
            bytes.extend_from_slice(&1_u16.to_be_bytes());
            bytes.extend_from_slice(&self.code_name.to_be_bytes());
            bytes.extend_from_slice(
                &u32::try_from(12 + code.len())
                    .expect("the fixture code attribute fits u32")
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(&max_stack.to_be_bytes());
            bytes.extend_from_slice(&max_locals.to_be_bytes());
            bytes.extend_from_slice(
                &u32::try_from(code.len())
                    .expect("the fixture code fits u32")
                    .to_be_bytes(),
            );
            bytes.extend_from_slice(code);
            bytes.extend_from_slice(&0_u16.to_be_bytes()); // exception table
            bytes.extend_from_slice(&0_u16.to_be_bytes()); // code attributes
        }
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // class attributes
        bytes
    }
}

fn emit_ldc(code: &mut Vec<u8>, index: u16) {
    if index <= 0xff {
        code.push(0x12);
        code.push(u8::try_from(index).expect("a one-byte `ldc` index fits"));
    } else {
        code.push(0x13);
        code.extend_from_slice(&index.to_be_bytes());
    }
}

fn emit_invoke(code: &mut Vec<u8>, opcode: u8, index: u16) {
    code.push(opcode);
    code.extend_from_slice(&index.to_be_bytes());
}

fn emit_pop(code: &mut Vec<u8>) {
    code.push(0x57);
}

fn emit_returns(code: &mut Vec<u8>) {
    code.push(0xb1);
}

/// One branch whose target is patched once the whole body is written: the operand's position and
/// the bytecode index it has to reach.
fn patch_branch(code: &mut [u8], operand_at: usize, target: usize) {
    let delta = i32::try_from(target).expect("a fixture BCI fits")
        - i32::try_from(operand_at - 1).expect("a fixture BCI fits");
    let delta = i16::try_from(delta).expect("a fixture branch fits i16");
    let bytes = delta.to_be_bytes();
    code[operand_at] = bytes[0];
    code[operand_at + 1] = bytes[1];
}

/// The class the sites name, with one field that carries no constant of its own.
fn target_class() -> ClassFile {
    let mut class = ClassFile::new(b"p/Target");
    class.field(b"MARKER", b"I", PUBLIC | STATIC | FINAL, None);
    class
}

/// A class whose `static final String` carries a `ConstantValue`: another class's constant.
fn constants_class() -> ClassFile {
    let mut class = ClassFile::new(b"p/Constants");
    let value = class.pool.string(b"p/Target");
    class.field(
        b"NAME",
        b"Ljava/lang/String;",
        PUBLIC | STATIC | FINAL,
        Some(value),
    );
    class
}

/// The service interface a `ServiceLoader.load` site names.
fn service_class() -> ClassFile {
    ClassFile::new(b"p/Service")
}

/// The class under scan: one method per case, each with the bytecode the case needs.
///
/// The returned marks are the bytecode indices the propagation assertions read: the `ldc` that
/// proves the stored constant, the `aload` that loads it again, and that method's call site.
fn site_class() -> (ClassFile, [u32; 3]) {
    let mut class = ClassFile::new(b"p/Site");
    let own = class.pool.string(b"p/Target");
    class.field(
        b"OWN",
        b"Ljava/lang/String;",
        PUBLIC | STATIC | FINAL,
        Some(own),
    );
    class.field(
        b"MT",
        b"Ljava/lang/invoke/MethodType;",
        PUBLIC | STATIC,
        None,
    );

    let for_name = class
        .pool
        .method_ref(b"java/lang/Class", b"forName", FOR_NAME.as_bytes());
    let get_declared_method = class.pool.method_ref(
        b"java/lang/Class",
        b"getDeclaredMethod",
        GET_DECLARED_METHOD.as_bytes(),
    );
    let get_declared_methods = class.pool.method_ref(
        b"java/lang/Class",
        b"getDeclaredMethods",
        GET_DECLARED_METHODS.as_bytes(),
    );
    let find_static = class.pool.method_ref(
        b"java/lang/invoke/MethodHandles$Lookup",
        b"findStatic",
        FIND_STATIC.as_bytes(),
    );
    let lookup = class.pool.method_ref(
        b"java/lang/invoke/MethodHandles",
        b"lookup",
        b"()Ljava/lang/invoke/MethodHandles$Lookup;",
    );
    let service_load =
        class
            .pool
            .method_ref(b"java/util/ServiceLoader", b"load", SERVICE_LOAD.as_bytes());
    let target_class_literal = class.pool.class(b"p/Target");
    let class_literal = class.pool.class(b"java/lang/Class");
    let service_literal = class.pool.class(b"p/Service");
    let own_field = class
        .pool
        .field_ref(b"p/Site", b"OWN", b"Ljava/lang/String;");
    let other_field = class
        .pool
        .field_ref(b"p/Constants", b"NAME", b"Ljava/lang/String;");
    let mt_field = class
        .pool
        .field_ref(b"p/Site", b"MT", b"Ljava/lang/invoke/MethodType;");
    let name = class.pool.string(b"foo");
    let target = class.pool.string(b"p/Target");
    let other_name = class.pool.string(b"p/OtherName");

    // Four sites of one overload: the present target, an absent class, a wildcard byte and a
    // string that is not a binary name at all.
    let mut code = Vec::new();
    for value in [
        b"p/Target".as_slice(),
        b"p/Missing".as_slice(),
        b"p/*".as_slice(),
        b"p/.Hidden".as_slice(),
    ] {
        let index = class.pool.string(value);
        emit_ldc(&mut code, index);
        emit_invoke(&mut code, 0xb8, for_name);
        emit_pop(&mut code);
    }
    emit_returns(&mut code);
    class.method(b"constantTarget", b"()V", PUBLIC | STATIC, 1, 0, code);

    // A constant stored in a local and loaded again: the bounded propagation through one slot.
    let mut code = Vec::new();
    emit_ldc(&mut code, target);
    let stored_source = u32::try_from(code.len() - 2).expect("a fixture BCI fits");
    code.push(0x4b); // astore_0
    let stored_load = u32::try_from(code.len()).expect("a fixture BCI fits");
    code.push(0x2a); // aload_0
    let stored_call = u32::try_from(code.len()).expect("a fixture BCI fits");
    emit_invoke(&mut code, 0xb8, for_name);
    emit_pop(&mut code);
    emit_returns(&mut code);
    class.method(b"storedConstant", b"()V", PUBLIC | STATIC, 1, 1, code);

    // The class's own `static final String`: a `ConstantValue` this plane proves, with the
    // assumption a static field carries.
    let mut code = Vec::new();
    emit_invoke(&mut code, 0xb2, own_field);
    emit_invoke(&mut code, 0xb8, for_name);
    emit_pop(&mut code);
    emit_returns(&mut code);
    class.method(b"ownConstantField", b"()V", PUBLIC | STATIC, 1, 0, code);

    // Another class's constant, which this slice does not prove: proving it would read that class.
    let mut code = Vec::new();
    emit_invoke(&mut code, 0xb2, other_field);
    emit_invoke(&mut code, 0xb8, for_name);
    emit_pop(&mut code);
    emit_returns(&mut code);
    class.method(
        b"otherClassConstantField",
        b"()V",
        PUBLIC | STATIC,
        1,
        0,
        code,
    );

    // A parameter: entry state, no instruction defines it.
    let mut code = Vec::new();
    code.push(0x2a); // aload_0
    emit_invoke(&mut code, 0xb8, for_name);
    emit_pop(&mut code);
    emit_returns(&mut code);
    class.method(
        b"dynamicParameter",
        b"(Ljava/lang/String;)V",
        PUBLIC | STATIC,
        1,
        1,
        code,
    );

    // Two branches meeting at the call site: the value on the stack is a merge, not a constant.
    let mut code = vec![
        0x1a, // iload_0
        0x99, 0, 0, // ifeq, patched to the else block below
    ];
    let ifeq_operand = code.len() - 2;
    emit_ldc(&mut code, target);
    code.push(0xa7); // goto
    code.push(0);
    code.push(0);
    let goto_operand = code.len() - 2;
    let else_bci = code.len();
    emit_ldc(&mut code, other_name);
    let end_bci = code.len();
    patch_branch(&mut code, ifeq_operand, else_bci);
    patch_branch(&mut code, goto_operand, end_bci);
    emit_invoke(&mut code, 0xb8, for_name);
    emit_pop(&mut code);
    emit_returns(&mut code);
    class.method(b"dynamicMerge", b"(Z)V", PUBLIC | STATIC, 1, 1, code);

    // The same constant through the loader-naming overload: another rule, another assumption. A
    // second site passes a name *no* class of the snapshot provides, which is what shows the
    // difference: this overload searches no order, so it leaves no dependency behind.
    let mut code = Vec::new();
    let for_name_loader =
        class
            .pool
            .method_ref(b"java/lang/Class", b"forName", FOR_NAME_LOADER.as_bytes());
    let missing = class.pool.string(b"p/Missing");
    for value in [target, missing] {
        emit_ldc(&mut code, value);
        code.push(0x04); // iconst_1
        code.push(0x01); // aconst_null
        emit_invoke(&mut code, 0xb8, for_name_loader);
        emit_pop(&mut code);
    }
    emit_returns(&mut code);
    class.method(b"explicitLoader", b"()V", PUBLIC | STATIC, 3, 0, code);

    // A member lookup: the receiver and the name are two constants, the parameter types are not
    // read. `p/Target` declares no `foo`, which is exactly what a name-level answer states.
    let mut code = Vec::new();
    emit_ldc(&mut code, target_class_literal);
    emit_ldc(&mut code, name);
    code.push(0x03); // iconst_0
    emit_invoke(&mut code, 0xbd, class_literal); // anewarray java/lang/Class
    emit_invoke(&mut code, 0xb6, get_declared_method);
    emit_pop(&mut code);
    emit_returns(&mut code);
    class.method(b"memberLookup", b"()V", PUBLIC | STATIC, 4, 0, code);

    // A method-handle lookup: the owner is the first argument and the name the second.
    let mut code = Vec::new();
    emit_invoke(&mut code, 0xb8, lookup);
    emit_ldc(&mut code, target_class_literal);
    emit_ldc(&mut code, name);
    emit_invoke(&mut code, 0xb2, mt_field);
    emit_invoke(&mut code, 0xb6, find_static);
    emit_pop(&mut code);
    emit_returns(&mut code);
    class.method(b"lookupFindStatic", b"()V", PUBLIC | STATIC, 4, 0, code);

    // A registered overload this slice derives nothing from.
    let mut code = Vec::new();
    emit_ldc(&mut code, target_class_literal);
    emit_invoke(&mut code, 0xb6, get_declared_methods);
    emit_pop(&mut code);
    emit_returns(&mut code);
    class.method(
        b"unsupportedEnumeration",
        b"()V",
        PUBLIC | STATIC,
        1,
        0,
        code,
    );

    // An overload this registry does not hold: no rule, so no site.
    let mut code = Vec::new();
    let get_superclass = class.pool.method_ref(
        b"java/lang/Class",
        b"getSuperclass",
        GET_SUPERCLASS.as_bytes(),
    );
    emit_ldc(&mut code, target_class_literal);
    emit_invoke(&mut code, 0xb6, get_superclass);
    emit_pop(&mut code);
    emit_returns(&mut code);
    class.method(b"unregisteredApi", b"()V", PUBLIC | STATIC, 1, 0, code);

    // A service interface named by a class literal, under the thread context loader.
    let mut code = Vec::new();
    emit_ldc(&mut code, service_literal);
    emit_invoke(&mut code, 0xb8, service_load);
    emit_pop(&mut code);
    emit_returns(&mut code);
    class.method(b"serviceLoader", b"()V", PUBLIC | STATIC, 1, 0, code);

    (class, [stored_source, stored_load, stored_call])
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

/// One snapshot holding the scan's own class plus the classes a case needs.
///
/// `target` and `missing` decide whether `p/Target` and `p/Missing` are part of the snapshot at
/// all, which is how the same inference is shown to hold with and without the target.
fn open_world(target: bool, missing: bool) -> ArtifactSnapshot {
    open_world_with(target, missing, true)
}

fn open_world_with(target: bool, missing: bool, service: bool) -> ArtifactSnapshot {
    let mut classes = vec![constants_class()];
    if service {
        classes.push(service_class());
    }
    if target {
        classes.push(target_class());
    }
    if missing {
        classes.push(ClassFile::new(b"p/Missing"));
    }
    classes.push(site_class().0);
    let entries = classes
        .iter()
        .map(|class| {
            let mut name = class.name();
            name.extend_from_slice(b".class");
            (name, class.build())
        })
        .collect::<Vec<_>>();
    Engine::new()
        .open(
            ArtifactInput::bytes(zip_of(&entries)),
            &mut Budget::new(limits()),
        )
        .expect("the fixture snapshot opens")
}

fn loader() -> LoaderId {
    LoaderId("app".to_string())
}

fn environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: loader(),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Container {
            origin: ContainerOrigin {
                snapshot: snapshot.id().clone(),
                root_container: ContainerId("root".into()),
                steps: Vec::new(),
            },
            prefix: ArchiveNameBytes(Vec::new()),
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

fn scan(snapshot: &ArtifactSnapshot, limits: Limits) -> ReflectionPatternReport {
    scan_with(snapshot, limits, 0)
}

fn scan_with(
    snapshot: &ArtifactSnapshot,
    limits: Limits,
    max_items: u64,
) -> ReflectionPatternReport {
    let mut budget = Budget::new(limits);
    Engine::new()
        .reflection_patterns(
            std::slice::from_ref(snapshot),
            &ReflectionPatternRequest {
                environment: environment(snapshot),
                scope: PhysicalScope::SnapshotAll,
                max_items,
            },
            &mut budget,
        )
        .expect("a legal request is answered, not raised")
}

/// The site of one method whose overload carries this name.
fn site_of<'a>(
    report: &'a ReflectionPatternReport,
    method: &str,
    overload: &str,
) -> &'a ReflectionSite {
    report
        .sites
        .iter()
        .find(|site| {
            site.method.name.0 == method.as_bytes() && site.overload.name.0 == overload.as_bytes()
        })
        .unwrap_or_else(|| {
            panic!(
                "the `{overload}` site of `{method}` is published: {:#?}",
                report.sites
            )
        })
}

fn inference_of<'a>(
    report: &'a ReflectionPatternReport,
    method: &str,
    overload: &str,
) -> &'a PatternInference {
    match &site_of(report, method, overload).state {
        ReflectionSiteState::PatternInferredTarget(inference) => inference,
        other => panic!("`{method}` is inferred, not {other:#?}"),
    }
}

fn unknown_of<'a>(
    report: &'a ReflectionPatternReport,
    method: &str,
    overload: &str,
) -> &'a PatternUnknown {
    match &site_of(report, method, overload).state {
        ReflectionSiteState::Unknown { reason } => reason,
        other => panic!("`{method}` is Unknown, not {other:#?}"),
    }
}

/// The dependency records a proven name produced.
fn pattern_dependencies<'a>(
    report: &'a ReflectionPatternReport,
    name: &[u8],
) -> Vec<&'a UnresolvedDependency> {
    report
        .unresolved_dependencies
        .iter()
        .filter(|dependency| {
            dependency.name.0 == name && dependency.reason == ReadReason::PatternTarget
        })
        .collect()
}

/// The one inference of one method whose constant is `value`.
fn inference_with_constant<'a>(
    report: &'a ReflectionPatternReport,
    method: &str,
    value: &[u8],
) -> &'a PatternInference {
    report
        .sites
        .iter()
        .filter(|site| site.method.name.0 == method.as_bytes())
        .find_map(|site| match &site.state {
            ReflectionSiteState::PatternInferredTarget(inference)
                if inference.constant_input.0 == value =>
            {
                Some(inference.as_ref())
            }
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "the `{}` site of `{method}` is inferred",
                String::from_utf8_lossy(value)
            )
        })
}

/// The registry crosses the facade, and it holds the four families the requirement names.
#[test]
fn the_declared_patterns_cross_the_facade() {
    assert!(
        patterns().len() >= 13,
        "the registry holds one record per declared overload: {} found",
        patterns().len()
    );
    for (owner, name, descriptor) in [
        (
            &b"java/lang/Class"[..],
            &b"forName"[..],
            FOR_NAME.as_bytes(),
        ),
        (
            &b"java/lang/Class"[..],
            &b"getDeclaredMethod"[..],
            GET_DECLARED_METHOD.as_bytes(),
        ),
        (
            &b"java/lang/invoke/MethodHandles$Lookup"[..],
            &b"findStatic"[..],
            FIND_STATIC.as_bytes(),
        ),
        (
            &b"java/util/ServiceLoader"[..],
            &b"load"[..],
            SERVICE_LOAD.as_bytes(),
        ),
    ] {
        let rule = jarde::reflection::pattern_for(owner, name, descriptor)
            .unwrap_or_else(|| panic!("`{}` is registered", String::from_utf8_lossy(name)));
        assert!(
            !rule.not_claimed.is_empty(),
            "`{}` states what it does not claim",
            rule.id
        );
    }
}

/// The core scenario: a constant target is `pattern_inferred_target` with all five elements, and
/// the JSON spelling is the requirement's own label.
#[test]
fn a_constant_target_is_inferred_with_all_five_elements() {
    let snapshot = open_world(true, false);
    let report = scan(&snapshot, limits());

    assert_eq!(report.analysis, ResolutionAnalysis::Performed);
    assert!(
        matches!(report.execution, ExecutionReport::Complete { .. }),
        "nothing stopped the scan: {:#?}",
        report.execution
    );
    assert!(!report.has_more);

    let site = site_of(&report, "constantTarget", "forName");
    // The schema of one site is exactly these six fields: an X1 edge or derivation cannot be added
    // without editing this destructuring, which is what keeps an inference distinguishable from a
    // raw fact of the artifact.
    let ReflectionSite {
        definition: _,
        method,
        bci,
        overload,
        rule,
        state,
    } = site.clone();
    assert_eq!(method.name.0, b"constantTarget");
    assert_eq!(rule, "class-for-name");
    assert_eq!(overload.descriptor.0, FOR_NAME.as_bytes());
    assert_eq!(site.overload.name.0, b"forName");
    assert_eq!(site.overload.owner.0, b"java/lang/Class");

    let ReflectionSiteState::PatternInferredTarget(inference) = &state else {
        panic!("a proven constant is an inference: {state:#?}");
    };
    // The five elements, destructured exhaustively: a conclusion cannot be published without one.
    let PatternInference {
        rule,
        rule_version,
        constant_input,
        target,
        propagation,
        loader: assumed_loader,
        resolution,
    } = inference.as_ref();
    assert_eq!(*rule, "class-for-name");
    assert_eq!(*rule_version, RuleVersion::new("class-for-name", "1"));
    assert_eq!(*constant_input, JvmBytes(b"p/Target".to_vec()));
    assert_eq!(
        *target,
        ReflectedTarget::Type {
            name: JvmBytes(b"p/Target".to_vec())
        }
    );
    assert_eq!(propagation.source.kind, ConstantSourceKind::LdcString);
    assert_eq!(
        propagation.source.assumption, None,
        "a literal of this method's own bytes carries no assumption"
    );
    assert_eq!(propagation.budget_dimension, "analysis_steps");
    assert_eq!(
        propagation.transfers, 0,
        "the call site reads the source's own value"
    );
    assert_eq!(
        propagation.path,
        vec![propagation.source.bci, bci],
        "the path is the source and the call site, in the order the value travelled"
    );
    assert_eq!(
        assumed_loader.kind,
        LoaderAssumptionKind::CallerDefiningLoader
    );
    assert_eq!(assumed_loader.loader, Some(loader()));
    assert!(
        !assumed_loader.statement.is_empty(),
        "the assumption is stated, not implied"
    );
    assert!(
        matches!(
            resolution,
            PatternTargetState::Resolved { loader: resolved, .. } if *resolved == loader()
        ),
        "the snapshot's own answer is published beside the inference: {resolution:#?}"
    );

    let document = serde_json::to_value(&state).expect("the state is serializable");
    assert_eq!(
        document["state"], "pattern_inferred_target",
        "the requirement's own label is the wire spelling"
    );

    // Nothing was executed or observed: the dynamic plane was never requested.
    assert_eq!(
        report.coverage.dynamic_analysis.state,
        CoverageState::NotRequested
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
}

/// A constant that travels through a local slot is proved by the body's own value flow, and the
/// evidence names every read it travelled through.
#[test]
fn a_stored_constant_propagates_within_one_body() {
    let (_, [source_bci, load_bci, call_bci]) = site_class();
    let snapshot = open_world(true, false);
    let report = scan(&snapshot, limits());

    let site = site_of(&report, "storedConstant", "forName");
    assert_eq!(site.bci, call_bci);
    let inference = inference_of(&report, "storedConstant", "forName");
    assert_eq!(inference.constant_input, JvmBytes(b"p/Target".to_vec()));
    assert_eq!(inference.propagation.source.bci, source_bci);
    assert_eq!(
        inference.propagation.source.kind,
        ConstantSourceKind::LdcString
    );
    assert_eq!(
        inference.propagation.path,
        vec![source_bci, load_bci - 1, load_bci, call_bci],
        "the store that popped the source's value and the load that read the slot back are the \
         path the value travelled, and both are in this body"
    );
    assert_eq!(
        inference.propagation.transfers, 2,
        "one store and one load carried the value from the source to the call site"
    );
    assert!(
        inference
            .propagation
            .path
            .iter()
            .all(|step| *step <= call_bci),
        "every step of the proof is inside this one body"
    );

    // The class's own `static final String`: the same conclusion from a `ConstantValue`, with the
    // assumption a static field carries.
    let inference = inference_of(&report, "ownConstantField", "forName");
    assert_eq!(inference.constant_input, JvmBytes(b"p/Target".to_vec()));
    assert_eq!(
        inference.propagation.source.kind,
        ConstantSourceKind::StaticFieldConstantValue {
            owner: JvmBytes(b"p/Site".to_vec()),
            name: JvmBytes(b"OWN".to_vec()),
            descriptor: JvmBytes(b"Ljava/lang/String;".to_vec()),
        }
    );
    let assumption = inference
        .propagation
        .source
        .assumption
        .expect("a static field's value is process state, and the result says so");
    assert!(
        assumption.contains("static field"),
        "the assumption names what it is about: {assumption}"
    );
}

/// Three dynamic sources, three checked `Unknown` answers, and never a guessed target.
#[test]
fn a_dynamic_input_is_unknown_and_never_a_guess() {
    let snapshot = open_world(true, false);
    let report = scan(&snapshot, limits());

    let other_call = site_of(&report, "otherClassConstantField", "forName").bci;
    for (method, expected) in [
        ("dynamicParameter", DynamicOrigin::Entry),
        ("dynamicMerge", DynamicOrigin::Merge),
        (
            "otherClassConstantField",
            DynamicOrigin::Instruction {
                bci: other_call - 3,
                opcode: 0xb2,
            },
        ),
    ] {
        match unknown_of(&report, method, "forName") {
            PatternUnknown::DynamicInput { input, origin } => {
                assert_eq!(*input, PatternInput::TargetName, "{method}");
                assert_eq!(
                    *origin, expected,
                    "{method}: the definition that produced the value is named"
                );
            }
            other => panic!("{method} is an unknown input, not {other:#?}"),
        }
        assert_eq!(
            site_of(&report, method, "forName").overload.descriptor.0,
            FOR_NAME.as_bytes(),
            "{method}: the overload is still published"
        );
    }

    assert!(
        !report.sites.iter().any(|site| {
            matches!(site.state, ReflectionSiteState::PatternInferredTarget(_))
                && matches!(
                    site.method.name.0.as_slice(),
                    b"dynamicParameter" | b"dynamicMerge" | b"otherClassConstantField"
                )
        }),
        "no dynamic input became an inferred target"
    );
    assert_eq!(
        report.sites.len(),
        report.inferred_sites().count() + report.unknown_sites().count()
    );
    assert_eq!(
        u64::try_from(report.sites.len()).expect("a fixture count fits"),
        report.returned_items
    );
}

/// The overload decides the conclusion, an unsupported overload is a stated boundary, and an
/// overload the registry does not hold is not a subject at all.
#[test]
fn the_overload_decides_the_conclusion() {
    let snapshot = open_world(true, false);
    let report = scan(&snapshot, limits());

    let one = inference_of(&report, "constantTarget", "forName");
    assert_eq!(one.loader.kind, LoaderAssumptionKind::CallerDefiningLoader);
    assert_eq!(one.loader.loader, Some(loader()));
    assert!(matches!(
        one.resolution,
        PatternTargetState::Resolved { .. }
    ));

    // The same constant through the loader-naming overload: the name is inferred, the order is not
    // searched, and the result says why.
    let three = inference_of(&report, "explicitLoader", "forName");
    assert_eq!(three.constant_input, JvmBytes(b"p/Target".to_vec()));
    assert_eq!(
        three.loader.kind,
        LoaderAssumptionKind::ExplicitLoaderArgument
    );
    assert_eq!(
        three.loader.loader, None,
        "the loader object is not an identity this snapshot holds"
    );
    match &three.resolution {
        PatternTargetState::NotDemanded { reason } => {
            assert!(
                reason.contains("loader"),
                "the reason names the loader: {reason}"
            );
        }
        other => panic!("no order is searched for an explicit loader: {other:#?}"),
    }
    // The two overloads pass the *same* unprovided name, and exactly one dependency exists for it:
    // the caller-loader overload searched the order and found nothing, the loader-naming overload
    // searched nothing at all.
    let unnamed = inference_with_constant(&report, "explicitLoader", b"p/Missing");
    assert!(matches!(
        unnamed.resolution,
        PatternTargetState::NotDemanded { .. }
    ));
    assert_eq!(
        pattern_dependencies(&report, b"p/Missing").len(),
        1,
        "one demand, one dependency: {:?}",
        pattern_dependencies(&report, b"p/Missing")
    );

    // A registered overload this slice derives nothing from.
    match unknown_of(&report, "unsupportedEnumeration", "getDeclaredMethods") {
        PatternUnknown::PatternNotSupported { rule, reason } => {
            assert_eq!(*rule, "class-get-declared-methods");
            assert!(!reason.is_empty());
        }
        other => panic!("an unsupported overload states its reason: {other:#?}"),
    }

    // An overload the registry does not hold: no rule, so no site and no claim.
    assert!(
        !report
            .sites
            .iter()
            .any(|site| site.overload.name.0 == b"getSuperclass"),
        "an unregistered overload is not an X3 subject"
    );
}

/// The member and service patterns: what the site asks for, under which loader assumption, and
/// without reading the target at all.
#[test]
fn the_member_and_service_patterns_name_what_the_site_asks_for() {
    let snapshot = open_world(true, false);
    let report = scan(&snapshot, limits());

    // `p/Target` declares no `foo` at all, and the site still names it: this plane gives a
    // name-level answer and never decodes the target.
    let member = inference_of(&report, "memberLookup", "getDeclaredMethod");
    assert_eq!(
        member.target,
        ReflectedTarget::Member {
            owner: JvmBytes(b"p/Target".to_vec()),
            name: JvmBytes(b"foo".to_vec()),
            member: ReflectedMemberKind::Method,
        }
    );
    assert_eq!(member.constant_input, JvmBytes(b"foo".to_vec()));
    assert_eq!(
        member.propagation.source.kind,
        ConstantSourceKind::LdcString
    );
    assert!(
        matches!(
            &member.resolution,
            PatternTargetState::NotDemanded { reason } if reason.contains("parameter types")
        ),
        "the member is named and not identified, and the reason says which input is missing: {:#?}",
        member.resolution
    );

    // The method-handle overload reads its owner and its name from two *arguments*.
    let handle = inference_of(&report, "lookupFindStatic", "findStatic");
    assert_eq!(
        handle.target,
        ReflectedTarget::Member {
            owner: JvmBytes(b"p/Target".to_vec()),
            name: JvmBytes(b"foo".to_vec()),
            member: ReflectedMemberKind::Method,
        }
    );
    assert_eq!(handle.constant_input, JvmBytes(b"foo".to_vec()));

    // A service interface named by a class literal, under the thread context loader.
    let service = inference_of(&report, "serviceLoader", "load");
    assert_eq!(
        service.target,
        ReflectedTarget::ServiceInterface {
            name: JvmBytes(b"p/Service".to_vec())
        }
    );
    assert_eq!(service.constant_input, JvmBytes(b"p/Service".to_vec()));
    assert_eq!(
        service.propagation.source.kind,
        ConstantSourceKind::LdcClass
    );
    assert_eq!(
        service.loader.kind,
        LoaderAssumptionKind::ThreadContextLoader
    );
    assert_eq!(service.loader.loader, None);
    assert!(matches!(
        service.resolution,
        PatternTargetState::NotDemanded { .. }
    ));

    // Nothing was executed or loaded: the same inferences come out of a snapshot that does not
    // contain the target class at all.
    let without_target = open_world(false, false);
    let without = scan(&without_target, limits());
    for (method, overload) in [
        ("memberLookup", "getDeclaredMethod"),
        ("lookupFindStatic", "findStatic"),
        ("serviceLoader", "load"),
    ] {
        assert_eq!(
            inference_of(&without, method, overload).target,
            inference_of(&report, method, overload).target,
            "{method}: the inference does not depend on the target class being in the snapshot"
        );
        assert_eq!(
            inference_of(&without, method, overload).constant_input,
            inference_of(&report, method, overload).constant_input,
            "{method}: the constant is the class file's own"
        );
    }
}

/// A name the snapshot does not hold is still an inference, and the snapshot's own answer is
/// published beside it; a string that is not a binary name is not searched for at all.
#[test]
fn a_target_outside_the_snapshot_is_inferred_and_named() {
    let snapshot = open_world(true, false);
    let report = scan(&snapshot, limits());

    let constants = report
        .sites
        .iter()
        .filter(|site| site.method.name.0 == b"constantTarget")
        .map(|site| match &site.state {
            ReflectionSiteState::PatternInferredTarget(inference) => {
                inference.constant_input.clone()
            }
            other => panic!("every site of this method is an inference: {other:#?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        constants,
        vec![
            JvmBytes(b"p/Target".to_vec()),
            JvmBytes(b"p/Missing".to_vec()),
            JvmBytes(b"p/*".to_vec()),
            JvmBytes(b"p/.Hidden".to_vec()),
        ],
        "the sites are published in BCI order with the constant each one passes"
    );

    assert!(matches!(
        inference_with_constant(&report, "constantTarget", b"p/Target").resolution,
        PatternTargetState::Resolved { .. }
    ));
    assert!(
        matches!(
            inference_with_constant(&report, "constantTarget", b"p/Missing").resolution,
            PatternTargetState::NotInSnapshot { loader: ref found } if *found == loader()
        ),
        "the snapshot's own answer is separate from the inference: {:#?}",
        inference_with_constant(&report, "constantTarget", b"p/Missing").resolution
    );
    let dependencies = pattern_dependencies(&report, b"p/Missing");
    assert_eq!(
        dependencies.len(),
        1,
        "the unprovided name is published once by name: {:?}",
        report.unresolved_dependencies
    );
    let dependency = dependencies[0];
    assert_eq!(dependency.loader, loader());
    assert_eq!(dependency.gap, DependencyGap::Missing);
    assert_eq!(
        dependency.declared_by,
        Some(JvmBytes(b"p/Site".to_vec())),
        "the class that names it is the class the call site is in"
    );

    // A wildcard byte is an identifier byte (JVMS 4.2.1), so it is a name no order provides.
    assert!(matches!(
        inference_with_constant(&report, "constantTarget", b"p/*").resolution,
        PatternTargetState::NotInSnapshot { .. }
    ));
    assert_eq!(pattern_dependencies(&report, b"p/*").len(), 1);
    // A `.` is not an identifier byte, so the string is not a binary name and no order is searched.
    match &inference_with_constant(&report, "constantTarget", b"p/.Hidden").resolution {
        PatternTargetState::NotDemanded { reason } => {
            assert!(reason.contains("JVMS 4.2.1"), "the rule is named: {reason}");
        }
        other => panic!("a non-name is not searched for: {other:#?}"),
    }
    assert!(
        pattern_dependencies(&report, b"p/.Hidden").is_empty(),
        "nothing was demanded, so no dependency is claimed"
    );

    // The control: with the class present, the same site resolves and no dependency is left.
    let complete = open_world(true, true);
    let control = scan(&complete, limits());
    assert!(matches!(
        inference_with_constant(&control, "constantTarget", b"p/Missing").resolution,
        PatternTargetState::Resolved { .. }
    ));
    assert!(
        pattern_dependencies(&control, b"p/Missing").is_empty(),
        "a name the order provides is not a missing dependency"
    );
}

/// A refused charge ends the scan with the prefix it published: the exhausted dimension is named
/// per site, the answer is a prefix, and no site was guessed.
#[test]
fn a_budget_stop_is_reported_and_never_guessed() {
    let snapshot = open_world(true, false);
    let mut tight = limits();
    tight.analysis_steps = 1;
    let report = scan(&snapshot, tight);

    assert!(
        report.has_more,
        "a body whose values were never named makes the answer a prefix"
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
    assert_eq!(
        report.inferred_sites().count(),
        0,
        "a stop never becomes an inferred target: {:#?}",
        report.sites
    );
    assert!(
        !report.unknown_sites().collect::<Vec<_>>().is_empty(),
        "the call sites the decode found are published, as Unknown with the run's own stop"
    );
    for site in report.unknown_sites() {
        match &site.state {
            ReflectionSiteState::Unknown {
                reason: PatternUnknown::AnalysisStopped { code },
            } => assert_eq!(
                code, "budget_exceeded_analysis_steps",
                "the site names the dimension that stopped the run it was read from"
            ),
            other => panic!("a stopped body states it per site: {other:#?}"),
        }
    }
    assert!(
        report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("budget_exceeded_analysis_steps")),
        "the stopped bodies are explained once, too: {:#?}",
        report.diagnostics
    );
}

/// The interface a `ServiceLoader.load` site names is not looked up at all: its absence from the
/// snapshot is not even a dependency, because the loader the rule states is not an order this
/// request can walk.
#[test]
fn a_service_interface_outside_the_snapshot_is_still_an_inference() {
    let snapshot = open_world_with(false, false, false);
    let report = scan(&snapshot, limits());

    let service = inference_of(&report, "serviceLoader", "load");
    assert_eq!(
        service.target,
        ReflectedTarget::ServiceInterface {
            name: JvmBytes(b"p/Service".to_vec())
        },
        "the interface is named by the class literal the site holds, present or not"
    );
    match &service.resolution {
        PatternTargetState::NotDemanded { reason } => assert!(
            reason.contains("thread"),
            "the reason names the loader assumption: {reason}"
        ),
        other => panic!("no order is searched under a thread context loader: {other:#?}"),
    }
    assert!(
        pattern_dependencies(&report, b"p/Service").is_empty(),
        "nothing was demanded for the interface, so no dependency is claimed for it"
    );
    assert!(
        report.inferred_sites().count() > 0,
        "the scan really ran on this snapshot"
    );
}

/// The caller's item limit bounds the answer instead of being ignored.
#[test]
fn an_item_limit_makes_the_answer_a_prefix() {
    let snapshot = open_world(true, false);
    let report = scan_with(&snapshot, limits(), 1);

    assert_eq!(report.sites.len(), 1, "one item was asked for");
    assert_eq!(report.returned_items, 1);
    assert!(
        report.has_more,
        "a prefix is not the whole range, and says so"
    );
    assert_eq!(
        report.coverage.artifact_structural.state,
        CoverageState::Partial
    );
}
