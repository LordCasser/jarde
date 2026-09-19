//! P3 2.2 acceptance: the three verified shapes of this slice — a concatenation chain, a bridge
//! method's forward, and a synthetic accessor's call site — presented, refused, and read back.
//!
//! What this file has to prove through the public surface:
//!
//! 1. a `StringBuilder`/`StringBuffer` chain becomes one `+` expression whose operands are written
//!    **once**, in the order the chain's `append` calls read them, with a throwing argument in the
//!    middle of the expression staying there;
//! 2. a chain that is not that shape — one a branch cuts, one whose instance is stored in a local,
//!    one whose `append` overload the rule cannot prove `+` reproduces, one with a single `append` —
//!    is **refused** and quoted, with the record naming the link that failed;
//! 3. a member the class declares a bridge is presented as the forward it is, including the cast the
//!    rule proves to be the erasure of the forwarded value; a bridge that does something else, one
//!    the class did not declare, and one whose declaration the run does not hold are refused;
//! 4. a synthetic accessor's call site becomes the direct field access A12 asks for, in both
//!    directions (a read and a write), with the segment carrying the call site's BCI **and** the
//!    field access inside the accessor's body;
//! 5. a body that builds a concatenation out of an accessor's field is presented as both at once,
//!    and a body with no debug metadata is still named deterministically;
//! 6. an output budget that refuses the emission hands out no text, and a stopped run records no
//!    shape as presented.
//!
//! The only facts this file supplies are the member's identity, its declaration flags and its debug
//! names — what the payload does not carry — plus, for the accessor cases, the class's other members
//! as the same read decoded them. Everything about what an instruction *does* is decoded by
//! `jarde-java` out of the payload or out of that member's body; nothing here states that a body
//! reads a field, whatever the fixture's names suggest.

use jarde_java::{
    AccessorField, AccessorShape, ClassMembers, MemberBody, MethodFacts, Provenance, RecoveryFacts,
    RecoveryRequest, StopReason, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest, Quality, Representation};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::classfile::{class_facts, method_code_facts};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::LoaderId;
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

/// The two concatenation classes: the one javac builds from release 5 on, and the earlier spelling.
const STRING_BUILDER: &str = "java/lang/StringBuilder";
const STRING_BUFFER: &str = "java/lang/StringBuffer";

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

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

/// The analyzed payload of one method of one class file, kept alive for the request that reads it.
struct Payload {
    analysis: jarde_jvm::method_ir::MethodIrAnalysis,
}

fn analyze(class: &[u8], name: &[u8], descriptor: &[u8]) -> Payload {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: bytes(name),
        descriptor: bytes(descriptor),
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Snapshot {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let environment = ResolutionEnvironment {
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
    };
    let request = MethodAnalysisRequest {
        environment,
        method,
        stages: AnalysisStage::ALL.to_vec(),
    };
    let analysis = analyze_method_ir(&[snapshot], &request, &mut budget)
        .expect("the analysis of an assembled body runs");
    Payload { analysis }
}

/// The facts of one method, read from the real class file: identity, **declaration flags** and debug
/// names. The flags are what a bridge rule needs and the payload does not carry, and they are read
/// from the member the class really declares rather than stated by this file.
fn facts_of(
    class: &[u8],
    name: &[u8],
    parameters: u16,
    debug: Vec<Option<String>>,
) -> RecoveryFacts {
    let mut budget = Budget::new(limits());
    let header = class_facts(class, &mut budget).expect("the fixture is a class file");
    let member = header
        .methods
        .iter()
        .find(|member| member.name.raw().0 == name)
        .expect("the fixture declares the method");
    RecoveryFacts::new(
        MethodFacts::new(
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(&member.descriptor.raw().0),
            parameters,
        )
        .with_access_flags(member.access_flags),
    )
    .with_debug_locals(debug)
}

/// The same facts with **no** declaration flags: the run cannot tell a bridge from an ordinary
/// member, which is the state the entry point of this build is in (it states no flags either).
fn facts_without_flags(class: &[u8], name: &[u8], parameters: u16) -> RecoveryFacts {
    let mut budget = Budget::new(limits());
    let header = class_facts(class, &mut budget).expect("the fixture is a class file");
    let member = header
        .methods
        .iter()
        .find(|member| member.name.raw().0 == name)
        .expect("the fixture declares the method");
    RecoveryFacts::new(MethodFacts::new(
        String::from_utf8_lossy(name),
        String::from_utf8_lossy(&member.descriptor.raw().0),
        parameters,
    ))
}

/// Every member of the fixture's class, as the same read decoded them: the declaration plus the
/// decoded body. This is the evidence a synthetic accessor call site is decided from.
fn members_of(class: &[u8]) -> ClassMembers {
    let mut budget = Budget::new(limits());
    let header = class_facts(class, &mut budget).expect("the fixture is a class file");
    let owner = String::from_utf8_lossy(&header.this_class.raw().0).into_owned();
    let members = header
        .methods
        .iter()
        .filter_map(|member| {
            let code = method_code_facts(class, member, &mut budget).ok()?;
            Some(MemberBody::new(
                owner.clone(),
                String::from_utf8_lossy(&member.name.raw().0),
                String::from_utf8_lossy(&member.descriptor.raw().0),
                member.access_flags,
                code,
            ))
        })
        .collect();
    ClassMembers::new(owner, members)
}

fn recover_body(
    payload: &Payload,
    facts: &RecoveryFacts,
    members: Option<&ClassMembers>,
    budget: &mut Budget,
) -> jarde_java::RecoveryReport {
    let request = RecoveryRequest::new(payload.analysis.ir(), facts, jarde_java::pass::JAVA_8);
    let request = match members {
        Some(members) => request.with_members(members),
        None => request,
    };
    recover(&request, budget)
}

/// Presents one fixture body under the full budget.
fn present(
    class: &[u8],
    name: &[u8],
    descriptor: &[u8],
    parameters: u16,
    debug: Vec<Option<String>>,
) -> jarde_java::RecoveryReport {
    let payload = analyze(class, name, descriptor);
    let facts = facts_of(class, name, parameters, debug);
    let members = members_of(class);
    let mut budget = Budget::new(limits());
    recover_body(&payload, &facts, Some(&members), &mut budget)
}

/// The three pieces of the `return` expression one report wrote, split on the `+` operator.
fn pieces(report: &jarde_java::RecoveryReport) -> Vec<String> {
    let line = report
        .text
        .lines()
        .find(|line| line.contains("return "))
        .unwrap_or_else(|| panic!("the report has a return statement:\n{}", report.text));
    line.trim()
        .trim_start_matches("return ")
        .trim_end_matches(';')
        .split(" + ")
        .map(str::to_string)
        .collect()
}

// ---------------------------------------------------------------------------
// The fixture class builder
// ---------------------------------------------------------------------------

/// The constant pool of a fixture: entries appended in order, their 1-based indexes returned.
#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    fn push(&mut self, entry: Vec<u8>) -> u16 {
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("a fixture pool fits u16")
    }

    fn utf8(&mut self, text: &str) -> u16 {
        let mut entry = vec![1u8];
        entry.extend_from_slice(
            &u16::try_from(text.len())
                .expect("a fixture name fits u16")
                .to_be_bytes(),
        );
        entry.extend_from_slice(text.as_bytes());
        self.push(entry)
    }

    fn class(&mut self, name: u16) -> u16 {
        let mut entry = vec![7u8];
        entry.extend_from_slice(&name.to_be_bytes());
        self.push(entry)
    }

    fn string(&mut self, text: u16) -> u16 {
        let mut entry = vec![8u8];
        entry.extend_from_slice(&text.to_be_bytes());
        self.push(entry)
    }

    fn name_and_type(&mut self, name: u16, descriptor: u16) -> u16 {
        let mut entry = vec![12u8];
        entry.extend_from_slice(&name.to_be_bytes());
        entry.extend_from_slice(&descriptor.to_be_bytes());
        self.push(entry)
    }

    fn field_ref(&mut self, class: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![9u8];
        entry.extend_from_slice(&class.to_be_bytes());
        entry.extend_from_slice(&name_and_type.to_be_bytes());
        self.push(entry)
    }

    fn method_ref(&mut self, class: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![10u8];
        entry.extend_from_slice(&class.to_be_bytes());
        entry.extend_from_slice(&name_and_type.to_be_bytes());
        self.push(entry)
    }
}

/// One member reference of a fixture: its name and descriptor being pooled as a side effect.
fn member_ref(pool: &mut Pool, class: u16, name: &str, descriptor: &str) -> u16 {
    let name = pool.utf8(name);
    let descriptor = pool.utf8(descriptor);
    let name_and_type = pool.name_and_type(name, descriptor);
    pool.method_ref(class, name_and_type)
}

/// One field reference of a fixture.
fn field_ref(pool: &mut Pool, class: u16, name: &str, descriptor: &str) -> u16 {
    let name = pool.utf8(name);
    let descriptor = pool.utf8(descriptor);
    let name_and_type = pool.name_and_type(name, descriptor);
    pool.field_ref(class, name_and_type)
}

/// One field of a fixture class.
struct FieldDef {
    flags: u16,
    name: u16,
    descriptor: u16,
}

/// One member of a fixture class, with its body.
struct MemberDef {
    flags: u16,
    name: u16,
    descriptor: u16,
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
}

/// The fixture's own bytecode, written instruction by instruction.
#[derive(Default)]
struct Code(Vec<u8>);

impl Code {
    fn op(mut self, opcode: u8) -> Self {
        self.0.push(opcode);
        self
    }

    fn index(mut self, index: u16) -> Self {
        self.0.extend_from_slice(&index.to_be_bytes());
        self
    }

    fn byte(mut self, byte: u8) -> Self {
        self.0.push(byte);
        self
    }

    fn done(self) -> Vec<u8> {
        self.0
    }
}

fn class_bytes(
    pool: &Pool,
    this_class: u16,
    super_class: u16,
    fields: &[FieldDef],
    methods: &[MemberDef],
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes());
    out.extend_from_slice(&52_u16.to_be_bytes());
    out.extend_from_slice(
        &u16::try_from(pool.entries.len() + 1)
            .expect("a fixture pool fits u16")
            .to_be_bytes(),
    );
    for entry in &pool.entries {
        out.extend_from_slice(entry);
    }
    out.extend_from_slice(&0x0021_u16.to_be_bytes());
    out.extend_from_slice(&this_class.to_be_bytes());
    out.extend_from_slice(&super_class.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes());
    out.extend_from_slice(
        &u16::try_from(fields.len())
            .expect("a fixture has few fields")
            .to_be_bytes(),
    );
    for field in fields {
        out.extend_from_slice(&field.flags.to_be_bytes());
        out.extend_from_slice(&field.name.to_be_bytes());
        out.extend_from_slice(&field.descriptor.to_be_bytes());
        out.extend_from_slice(&0_u16.to_be_bytes());
    }
    out.extend_from_slice(
        &u16::try_from(methods.len())
            .expect("a fixture has few methods")
            .to_be_bytes(),
    );
    for method in methods {
        out.extend_from_slice(&method.flags.to_be_bytes());
        out.extend_from_slice(&method.name.to_be_bytes());
        out.extend_from_slice(&method.descriptor.to_be_bytes());
        out.extend_from_slice(&1_u16.to_be_bytes());
        let code_name = 1_u16; // overwritten below: the pool holds `Code` first for every fixture
        out.extend_from_slice(&code_name.to_be_bytes());
        let mut attribute = Vec::new();
        attribute.extend_from_slice(&method.max_stack.to_be_bytes());
        attribute.extend_from_slice(&method.max_locals.to_be_bytes());
        attribute.extend_from_slice(
            &u32::try_from(method.code.len())
                .expect("a fixture body fits u32")
                .to_be_bytes(),
        );
        attribute.extend_from_slice(&method.code);
        attribute.extend_from_slice(&0_u16.to_be_bytes());
        attribute.extend_from_slice(&0_u16.to_be_bytes());
        out.extend_from_slice(
            &u32::try_from(attribute.len())
                .expect("a fixture attribute fits u32")
                .to_be_bytes(),
        );
        out.extend_from_slice(&attribute);
    }
    out.extend_from_slice(&0_u16.to_be_bytes());
    out
}

/// The first pool entry every fixture writes: the name of the `Code` attribute.
fn code_attribute(pool: &mut Pool) -> u16 {
    pool.utf8("Code")
}

/// The constants every fixture needs: `Test` and `java/lang/Object`.
fn base_pool() -> (Pool, u16, u16, u16) {
    let mut pool = Pool::default();
    let code = code_attribute(&mut pool);
    let test_name = pool.utf8("Test");
    let test = pool.class(test_name);
    let object_name = pool.utf8("java/lang/Object");
    let object = pool.class(object_name);
    (pool, code, test, object)
}

/// The two concatenation classes' pool entries, as one fixture needs them.
struct Concat {
    class: u16,
    init: u16,
    append_string: u16,
}

fn concat_pool(pool: &mut Pool, internal: &str) -> Concat {
    let name = pool.utf8(internal);
    let class = pool.class(name);
    let init = member_ref(pool, class, "<init>", "()V");
    let append_string = member_ref(
        pool,
        class,
        "append",
        &format!("(Ljava/lang/String;)L{internal};"),
    );
    Concat {
        class,
        init,
        append_string,
    }
}

// ---------------------------------------------------------------------------
// A concatenation chain: the verified shape, and the four ways out of it
// ---------------------------------------------------------------------------

/// `Test.method()Ljava/lang/String;` = `"x" + 5 + f()`.
///
/// Three operands of three kinds — a `String` literal, an `int` the opcode encodes, and a **call
/// that can throw** — with the throwing one in the last position:
///
/// ```text
/// 0:  new java/lang/StringBuilder          12: bipush 5
/// 3:  dup                                  14: append(int)
/// 4:  invokespecial <init>()V              17: invokestatic f()Ljava/lang/String;
/// 7:  ldc "x"                              20: append(String)
/// 9:  append(String)                       23: toString()Ljava/lang/String;
///                                          26: areturn
/// ```
fn concat_order_class() -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let concat = concat_pool(&mut pool, STRING_BUILDER);
    let literal_name = pool.utf8("x");
    let literal = pool.string(literal_name);
    let append_int = member_ref(
        &mut pool,
        concat.class,
        "append",
        &format!("(I)L{STRING_BUILDER};"),
    );
    let to_string = member_ref(&mut pool, concat.class, "toString", "()Ljava/lang/String;");
    let f = member_ref(&mut pool, test, "f", "()Ljava/lang/String;");
    let method = pool.utf8("method");
    let method_descriptor = pool.utf8("()Ljava/lang/String;");
    let f_name = pool.utf8("f");
    let f_descriptor = pool.utf8("()Ljava/lang/String;");
    let f_literal_name = pool.utf8("F");
    let f_literal = pool.string(f_literal_name);
    let short = |index: u16| u8::try_from(index).expect("a fixture's `ldc` index fits a byte");
    let code = Code::default()
        .op(0xbb)
        .index(concat.class) // 0:  new
        .op(0x59) // 3:  dup
        .op(0xb7)
        .index(concat.init) // 4:  invokespecial <init>()V
        .op(0x12)
        .byte(short(literal)) // 7:  ldc "x"
        .op(0xb6)
        .index(concat.append_string) // 9:  append(String)
        .op(0x10)
        .byte(5) // 12: bipush 5
        .op(0xb6)
        .index(append_int) // 14: append(int)
        .op(0xb8)
        .index(f) // 17: invokestatic Test.f
        .op(0xb6)
        .index(concat.append_string) // 20: append(String)
        .op(0xb6)
        .index(to_string) // 23: toString
        .op(0xb0) // 26: areturn
        .done();
    let f_code = Code::default()
        .op(0x12)
        .byte(short(f_literal))
        .op(0xb0)
        .done();
    class_bytes(
        &pool,
        test,
        object,
        &[],
        &[
            MemberDef {
                flags: 0x0009,
                name: method,
                descriptor: method_descriptor,
                max_stack: 2,
                max_locals: 0,
                code,
            },
            MemberDef {
                flags: 0x0009,
                name: f_name,
                descriptor: f_descriptor,
                max_stack: 1,
                max_locals: 0,
                code: f_code,
            },
        ],
    )
}

/// `Test.method(I)Ljava/lang/String;` = `value + f() + "x"`, on a `StringBuffer`.
///
/// The pre-5 spelling of the same chain, with a local operand and the throwing call **between** the
/// other two: whatever the rule writes has to keep that call there.
fn concat_buffer_class() -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let concat = concat_pool(&mut pool, STRING_BUFFER);
    let append_int = member_ref(
        &mut pool,
        concat.class,
        "append",
        &format!("(I)L{STRING_BUFFER};"),
    );
    let to_string = member_ref(&mut pool, concat.class, "toString", "()Ljava/lang/String;");
    let f = member_ref(&mut pool, test, "f", "()Ljava/lang/String;");
    let literal_name = pool.utf8("x");
    let literal = pool.string(literal_name);
    let method = pool.utf8("method");
    let method_descriptor = pool.utf8("(I)Ljava/lang/String;");
    let f_name = pool.utf8("f");
    let f_descriptor = pool.utf8("()Ljava/lang/String;");
    let f_literal_name = pool.utf8("F");
    let f_literal = pool.string(f_literal_name);
    let short = |index: u16| u8::try_from(index).expect("a fixture's `ldc` index fits a byte");
    let code = Code::default()
        .op(0xbb)
        .index(concat.class) // 0:  new
        .op(0x59) // 3:  dup
        .op(0xb7)
        .index(concat.init) // 4:  invokespecial <init>()V
        .op(0x1a) // 7:  iload_0  (the parameter)
        .op(0xb6)
        .index(append_int) // 8:  append(int)
        .op(0xb8)
        .index(f) // 11: invokestatic Test.f
        .op(0xb6)
        .index(concat.append_string) // 14: append(String)
        .op(0x12)
        .byte(short(literal)) // 17: ldc "x"
        .op(0xb6)
        .index(concat.append_string) // 19: append(String)
        .op(0xb6)
        .index(to_string) // 22: toString
        .op(0xb0) // 25: areturn
        .done();
    let f_code = Code::default()
        .op(0x12)
        .byte(short(f_literal))
        .op(0xb0)
        .done();
    class_bytes(
        &pool,
        test,
        object,
        &[],
        &[
            MemberDef {
                flags: 0x0009,
                name: method,
                descriptor: method_descriptor,
                max_stack: 2,
                max_locals: 1,
                code,
            },
            MemberDef {
                flags: 0x0009,
                name: f_name,
                descriptor: f_descriptor,
                max_stack: 1,
                max_locals: 0,
                code: f_code,
            },
        ],
    )
}

/// `Test.method(I)Ljava/lang/String;` where a branch sits **inside** the chain: the two arms append
/// different things and the `toString` is in another block.
fn concat_branch_class() -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let concat = concat_pool(&mut pool, STRING_BUILDER);
    let to_string = member_ref(&mut pool, concat.class, "toString", "()Ljava/lang/String;");
    let a_name = pool.utf8("a");
    let a = pool.string(a_name);
    let b_name = pool.utf8("b");
    let b = pool.string(b_name);
    let method = pool.utf8("method");
    let method_descriptor = pool.utf8("(I)Ljava/lang/String;");
    let short = |index: u16| u8::try_from(index).expect("a fixture's `ldc` index fits a byte");
    let code = Code::default()
        .op(0xbb)
        .index(concat.class) // 0:  new
        .op(0x59) // 3:  dup
        .op(0xb7)
        .index(concat.init) // 4:  invokespecial <init>()V
        .op(0x1a) // 7:  iload_0
        .op(0x99)
        .index(8) // 8:  ifeq → 16
        .op(0x12)
        .byte(short(a)) // 11: ldc "a"
        .op(0xb6)
        .index(concat.append_string) // 13: append(String)
        .op(0x12)
        .byte(short(b)) // 17: ldc "b"
        .op(0xb6)
        .index(concat.append_string) // 19: append(String)
        .op(0xb6)
        .index(to_string) // 22: toString
        .op(0x4c) // 25: astore_1
        .op(0x2b) // 26: aload_1
        .op(0xb0) // 27: areturn
        .done();
    class_bytes(
        &pool,
        test,
        object,
        &[],
        &[MemberDef {
            flags: 0x0009,
            name: method,
            descriptor: method_descriptor,
            max_stack: 2,
            max_locals: 2,
            code,
        }],
    )
}

/// `Test.method(I)Ljava/lang/String;` where the instance is **stored in a local** — the shape a
/// source-written `StringBuilder sb = new StringBuilder(); …` leaves behind.
fn concat_stored_class() -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let concat = concat_pool(&mut pool, STRING_BUILDER);
    let to_string = member_ref(&mut pool, concat.class, "toString", "()Ljava/lang/String;");
    let a_name = pool.utf8("a");
    let a = pool.string(a_name);
    let b_name = pool.utf8("b");
    let b = pool.string(b_name);
    let method = pool.utf8("method");
    let method_descriptor = pool.utf8("(I)Ljava/lang/String;");
    let short = |index: u16| u8::try_from(index).expect("a fixture's `ldc` index fits a byte");
    let code = Code::default()
        .op(0xbb)
        .index(concat.class) // 0:  new
        .op(0x59) // 3:  dup
        .op(0xb7)
        .index(concat.init) // 4:  invokespecial <init>()V
        .op(0x4c) // 7:  astore_1  ← the instance is aliased by a local
        .op(0x2b) // 8:  aload_1
        .op(0x12)
        .byte(short(a)) // 9:  ldc "a"
        .op(0xb6)
        .index(concat.append_string) // 11: append(String)
        .op(0x12)
        .byte(short(b)) // 14: ldc "b"
        .op(0xb6)
        .index(concat.append_string) // 16: append(String)
        .op(0xb6)
        .index(to_string) // 19: toString
        .op(0xb0) // 22: areturn
        .done();
    class_bytes(
        &pool,
        test,
        object,
        &[],
        &[MemberDef {
            flags: 0x0009,
            name: method,
            descriptor: method_descriptor,
            max_stack: 2,
            max_locals: 2,
            code,
        }],
    )
}

/// `Test.method()Ljava/lang/String;` whose second `append` is the `CharSequence` overload: `+` on
/// that operand would convert through `toString`, which is not what the overload writes.
fn concat_char_sequence_class() -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let concat = concat_pool(&mut pool, STRING_BUILDER);
    let append_chars = member_ref(
        &mut pool,
        concat.class,
        "append",
        &format!("(Ljava/lang/CharSequence;)L{STRING_BUILDER};"),
    );
    let to_string = member_ref(&mut pool, concat.class, "toString", "()Ljava/lang/String;");
    let literal_name = pool.utf8("x");
    let literal = pool.string(literal_name);
    let method = pool.utf8("method");
    let method_descriptor = pool.utf8("()Ljava/lang/String;");
    let short = |index: u16| u8::try_from(index).expect("a fixture's `ldc` index fits a byte");
    let code = Code::default()
        .op(0xbb)
        .index(concat.class) // 0:  new
        .op(0x59) // 3:  dup
        .op(0xb7)
        .index(concat.init) // 4:  invokespecial <init>()V
        .op(0x01) // 7:  aconst_null
        .op(0xb6)
        .index(append_chars) // 8:  append(CharSequence)
        .op(0x12)
        .byte(short(literal)) // 11: ldc "x"
        .op(0xb6)
        .index(concat.append_string) // 13: append(String)
        .op(0xb6)
        .index(to_string) // 16: toString
        .op(0xb0) // 19: areturn
        .done();
    class_bytes(
        &pool,
        test,
        object,
        &[],
        &[MemberDef {
            flags: 0x0009,
            name: method,
            descriptor: method_descriptor,
            max_stack: 2,
            max_locals: 0,
            code,
        }],
    )
}

/// `Test.method()Ljava/lang/String;` whose chain appends exactly one value: there is no `+` to write.
fn concat_single_append_class() -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let concat = concat_pool(&mut pool, STRING_BUILDER);
    let to_string = member_ref(&mut pool, concat.class, "toString", "()Ljava/lang/String;");
    let literal_name = pool.utf8("x");
    let literal = pool.string(literal_name);
    let method = pool.utf8("method");
    let method_descriptor = pool.utf8("()Ljava/lang/String;");
    let short = |index: u16| u8::try_from(index).expect("a fixture's `ldc` index fits a byte");
    let code = Code::default()
        .op(0xbb)
        .index(concat.class) // 0:  new
        .op(0x59) // 3:  dup
        .op(0xb7)
        .index(concat.init) // 4:  invokespecial <init>()V
        .op(0x12)
        .byte(short(literal)) // 7:  ldc "x"
        .op(0xb6)
        .index(concat.append_string) // 9:  append(String)
        .op(0xb6)
        .index(to_string) // 12: toString
        .op(0xb0) // 15: areturn
        .done();
    class_bytes(
        &pool,
        test,
        object,
        &[],
        &[MemberDef {
            flags: 0x0009,
            name: method,
            descriptor: method_descriptor,
            max_stack: 2,
            max_locals: 0,
            code,
        }],
    )
}

#[test]
fn a_verified_chain_is_written_once_with_its_operands_in_the_order_it_read_them() {
    let class = concat_order_class();
    let report = present(&class, b"method", b"()Ljava/lang/String;", 0, Vec::new());
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert_eq!(report.quality, Quality::Structured);
    assert!(report.fallbacks.is_empty(), "{:?}", report.fallbacks);
    assert!(
        report.text.contains("return \"x\" + 5 + f();"),
        "{}",
        report.text
    );
    // One evaluation of the throwing argument, and it stays where the bytecode called it: an
    // `append` chain evaluates its operands once, left to right.
    assert_eq!(
        report.text.matches("f()").count(),
        1,
        "the callable operand is written exactly once:\n{}",
        report.text
    );
    assert_eq!(pieces(&report), vec!["\"x\"", "5", "f()"]);

    // The evidence: one chain, the class it builds, every `append` with its own BCI and the
    // parameter type its own pool reference states.
    assert_eq!(report.concats.len(), 1);
    let chain = &report.concats[0];
    assert!(chain.presented());
    assert_eq!(chain.head, 0);
    assert_eq!(chain.tail, Some(23));
    assert_eq!(chain.class, STRING_BUILDER);
    assert_eq!(
        chain
            .appends
            .iter()
            .map(|append| append.bci)
            .collect::<Vec<_>>(),
        vec![9, 14, 20]
    );
    assert_eq!(
        chain
            .appends
            .iter()
            .map(|append| append.parameter.as_str())
            .collect::<Vec<_>>(),
        vec!["java.lang.String", "int", "java.lang.String"]
    );
    assert_eq!(chain.rule().citation(), "concat@1");
    assert!(report.rules.contains(&chain.rule()));
    assert!(report.accessors.is_empty());
    assert!(report.bridges.is_empty());
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_concat_chains"),
        "{:?}",
        report.diagnostics
    );

    // Every original BCI of the chain reached the artifact, and the value the chain built is
    // anchored where its `toString` was: one node presents many instructions, not just one.
    let anchors: std::collections::BTreeSet<u32> = report
        .source_map
        .segments()
        .iter()
        .flat_map(|segment| segment.origin().bcis())
        .collect();
    for bci in [0, 3, 4, 7, 9, 12, 14, 17, 20, 23] {
        assert!(
            anchors.contains(&bci),
            "BCI {bci} is one of the chain's own anchors: {anchors:?}"
        );
    }
    assert!(
        report
            .text_of_bci(23)
            .iter()
            .any(|text| text.contains("f()")),
        "the `toString`'s own BCI reaches the expression: {:?}",
        report.text_of_bci(23)
    );
}

#[test]
fn the_same_shape_on_a_string_buffer_is_presented_with_its_call_in_the_middle() {
    let class = concat_buffer_class();
    let report = present(
        &class,
        b"method",
        b"(I)Ljava/lang/String;",
        1,
        vec![Some("value".into())],
    );
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert_eq!(report.concats.len(), 1);
    assert_eq!(report.concats[0].class, STRING_BUFFER);
    assert!(report.concats[0].presented());
    // The class is accepted by name, and the accepted names are exactly the two the design states:
    // the *same* shape on a class this rule does not know is not claimed (see the unit tests of
    // `concat`, which pin the list).
    assert_eq!(pieces(&report), vec!["value", "f()", "\"x\""]);
    assert_eq!(
        report.concats[0]
            .appends
            .iter()
            .map(|append| append.bci)
            .collect::<Vec<_>>(),
        vec![8, 14, 19]
    );
}

#[test]
fn a_chain_a_branch_cuts_is_refused_rather_than_written_as_one_expression() {
    let class = concat_branch_class();
    let report = present(&class, b"method", b"(I)Ljava/lang/String;", 1, Vec::new());
    assert_eq!(report.concats.len(), 1);
    let chain = &report.concats[0];
    assert!(!chain.presented());
    let refusal = chain.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_concat_split");
    assert!(
        refusal.message.contains("another block"),
        "{}",
        refusal.message
    );
    assert_eq!(refusal.rule.citation(), "concat@1");
    assert!(report.rules.contains(&refusal.rule));
    assert!(!report.text.contains("\"a\" + "), "{}", report.text);
    assert_eq!(report.representation, Representation::Mixed);
    assert_eq!(report.quality, Quality::Fallback);
}

#[test]
fn a_chain_whose_instance_is_stored_in_a_local_is_refused() {
    let class = concat_stored_class();
    let report = present(&class, b"method", b"(I)Ljava/lang/String;", 1, Vec::new());
    assert_eq!(report.concats.len(), 1);
    let chain = &report.concats[0];
    assert!(!chain.presented());
    let refusal = chain.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_concat_interleaved_effect");
    assert_eq!(
        refusal.requirement.as_deref(),
        Some("a test block whose every instruction is part of a value expression"),
        "the refusal names the requirement the rule declared"
    );
    assert!(
        refusal.message.contains("BCI 7"),
        "the store that aliases the instance is the instruction the refusal names: {}",
        refusal.message
    );
    assert!(!report.text.contains("\"a\" + "), "{}", report.text);
    assert!(report.text.contains("// @bytecode"), "{}", report.text);
}

#[test]
fn an_append_overload_that_plus_would_not_reproduce_is_refused() {
    let class = concat_char_sequence_class();
    let report = present(&class, b"method", b"()Ljava/lang/String;", 0, Vec::new());
    assert_eq!(report.concats.len(), 1);
    let chain = &report.concats[0];
    assert!(!chain.presented());
    let refusal = chain.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_concat_shape");
    assert!(
        refusal.message.contains("java.lang.CharSequence"),
        "the refusal names the overload it cannot prove: {}",
        refusal.message
    );
    assert!(report.text.contains("// @bytecode"), "{}", report.text);
}

#[test]
fn a_single_append_is_not_a_concatenation() {
    let class = concat_single_append_class();
    let report = present(&class, b"method", b"()Ljava/lang/String;", 0, Vec::new());
    assert_eq!(report.concats.len(), 1);
    let chain = &report.concats[0];
    assert!(!chain.presented());
    let refusal = chain.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_concat_shape");
    assert!(
        refusal.message.contains("at least two operands"),
        "{}",
        refusal.message
    );
}

// ---------------------------------------------------------------------------
// A bridge method: the forward it is, and the three bodies it may not be
// ---------------------------------------------------------------------------

/// The `real()Ljava/lang/String;` member every bridge fixture forwards to.
fn real_member(pool: &mut Pool, test: u16) -> (u16, u16, Vec<u8>, u16) {
    let name = pool.utf8("real");
    let descriptor = pool.utf8("()Ljava/lang/String;");
    let literal_name = pool.utf8("s");
    let literal = pool.string(literal_name);
    let short = |index: u16| u8::try_from(index).expect("a fixture's `ldc` index fits a byte");
    let code = Code::default()
        .op(0x12)
        .byte(short(literal))
        .op(0xb0)
        .done();
    let name_and_type = pool.name_and_type(name, descriptor);
    let forward = pool.method_ref(test, name_and_type);
    (name, descriptor, code, forward)
}

/// The flags a compiler sets on a bridge: `public bridge synthetic`.
const BRIDGE_FLAGS: u16 = 0x0001 | 0x0040 | 0x1000;

/// One bridge fixture: `Test.real()` plus the bridge member the caller describes.
///
/// `body` is the bridge's own bytecode, `flags` its declaration, and `name`/`descriptor` its
/// identity; the parameter cast fixture supplies its own forwarded member through `extra`.
/// The body one bridge fixture declares for its bridge member.
enum BridgeBody {
    /// `aload_0; invokevirtual real; areturn` — the forward on its own.
    Forward,
    /// `aload_0; invokevirtual real; checkcast java/lang/String; areturn` — the cast that is the
    /// erasure of the forwarded value, because the invocation declares it returns a `String`.
    ForwardWithErasedCast,
    /// `aload_0; invokevirtual real; astore_1; aload_1; areturn` — a forward and something else.
    ForwardAndStore,
    /// `aload_0; aload_1; checkcast java/lang/String; invokevirtual real2; areturn` — a cast on the
    /// **parameter**, which is a check that can fail.
    ParameterCast,
}

fn bridge_class(name: &str, descriptor: &str, flags: u16, body: BridgeBody) -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let (real_name, real_descriptor, real_code, real) = real_member(&mut pool, test);
    let real2 = member_ref(
        &mut pool,
        test,
        "real2",
        "(Ljava/lang/String;)Ljava/lang/String;",
    );
    let string_name = pool.utf8("java/lang/String");
    let string = pool.class(string_name);
    let bridge_name = pool.utf8(name);
    let bridge_descriptor = pool.utf8(descriptor);
    let code = match body {
        BridgeBody::Forward => Code::default()
            .op(0x2a) // 0: aload_0
            .op(0xb6)
            .index(real) // 1: invokevirtual Test.real
            .op(0xb0) // 4: areturn
            .done(),
        BridgeBody::ForwardWithErasedCast => Code::default()
            .op(0x2a) // 0: aload_0
            .op(0xb6)
            .index(real) // 1: invokevirtual Test.real
            .op(0xc0)
            .index(string) // 4: checkcast java/lang/String
            .op(0xb0) // 7: areturn
            .done(),
        BridgeBody::ForwardAndStore => Code::default()
            .op(0x2a) // 0: aload_0
            .op(0xb6)
            .index(real) // 1: invokevirtual Test.real
            .op(0x4c) // 4: astore_1
            .op(0x2b) // 5: aload_1
            .op(0xb0) // 6: areturn
            .done(),
        BridgeBody::ParameterCast => Code::default()
            .op(0x2a) // 0: aload_0
            .op(0x2b) // 1: aload_1
            .op(0xc0)
            .index(string) // 2: checkcast java/lang/String
            .op(0xb6)
            .index(real2) // 5: invokevirtual Test.real2
            .op(0xb0) // 8: areturn
            .done(),
    };
    class_bytes(
        &pool,
        test,
        object,
        &[],
        &[
            MemberDef {
                flags: 0x0001,
                name: real_name,
                descriptor: real_descriptor,
                max_stack: 1,
                max_locals: 1,
                code: real_code,
            },
            MemberDef {
                flags,
                name: bridge_name,
                descriptor: bridge_descriptor,
                max_stack: 2,
                max_locals: 2,
                code,
            },
        ],
    )
}

#[test]
fn a_declared_bridge_is_presented_as_the_forward_it_is_with_the_cast_it_erases() {
    let class = bridge_class(
        "c",
        "()Ljava/lang/Object;",
        BRIDGE_FLAGS,
        BridgeBody::ForwardWithErasedCast,
    );
    let report = present(
        &class,
        b"c",
        b"()Ljava/lang/Object;",
        1,
        vec![Some("self".into())],
    );
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert_eq!(report.quality, Quality::Structured);
    assert!(
        !report.text.contains("// @bytecode"),
        "the erased cast is not quoted:\n{}",
        report.text
    );
    assert!(
        report.text.contains("return self.real();"),
        "{}",
        report.text
    );
    assert_eq!(report.bridges.len(), 1);
    let bridge = &report.bridges[0];
    assert!(bridge.presented());
    assert_eq!(
        bridge.forwarded.as_deref(),
        Some("Test.real()Ljava/lang/String;")
    );
    assert_eq!(bridge.erased.as_deref(), Some("java/lang/String"));
    assert_eq!(bridge.rule().citation(), "bridge@1");
    assert!(report.rules.contains(&bridge.rule()));
    // The cast is dropped *and* kept: its own BCI reached the artifact as an anchor of the value
    // whose erasure it is.
    let anchors: std::collections::BTreeSet<u32> = report
        .source_map
        .segments()
        .iter()
        .flat_map(|segment| segment.origin().bcis())
        .collect();
    assert!(
        anchors.contains(&4),
        "the cast's own BCI is an anchor: {anchors:?}"
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_bridge"),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn a_bridge_without_a_cast_is_presented_without_any_erasure() {
    let class = bridge_class(
        "m",
        "()Ljava/lang/Object;",
        BRIDGE_FLAGS,
        BridgeBody::Forward,
    );
    let report = present(
        &class,
        b"m",
        b"()Ljava/lang/Object;",
        1,
        vec![Some("self".into())],
    );
    assert!(
        report.text.contains("return self.real();"),
        "{}",
        report.text
    );
    assert_eq!(report.bridges.len(), 1);
    assert!(report.bridges[0].presented());
    assert_eq!(report.bridges[0].erased, None);
}

#[test]
fn a_member_the_class_does_not_declare_a_bridge_keeps_its_cast_quoted() {
    // The same body as the positive case, and the class declares the member **without** the bridge
    // flag: the rule reads the declaration, not the shape, so the cast stays quoted.
    let class = bridge_class(
        "c",
        "()Ljava/lang/Object;",
        0x0001,
        BridgeBody::ForwardWithErasedCast,
    );
    let report = present(
        &class,
        b"c",
        b"()Ljava/lang/Object;",
        1,
        vec![Some("self".into())],
    );
    assert_eq!(report.bridges.len(), 1);
    let bridge = &report.bridges[0];
    assert!(!bridge.presented());
    let refusal = bridge.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_bridge_not_declared");
    assert!(
        refusal.message.contains("bridge flag"),
        "{}",
        refusal.message
    );
    assert!(report.text.contains("// @bytecode"), "{}", report.text);
    assert_eq!(report.representation, Representation::Mixed);
}

#[test]
fn a_bridge_that_does_something_besides_forward_is_refused_and_its_body_kept() {
    let class = bridge_class(
        "m",
        "()Ljava/lang/Object;",
        BRIDGE_FLAGS,
        BridgeBody::ForwardAndStore,
    );
    let report = present(
        &class,
        b"m",
        b"()Ljava/lang/Object;",
        1,
        vec![Some("self".into())],
    );
    assert_eq!(report.bridges.len(), 1);
    let bridge = &report.bridges[0];
    assert!(!bridge.presented());
    let refusal = bridge.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_bridge_shape");
    // The body is still presented the ordinary way: refusing the *bridge shape* is not a refusal of
    // the member, and the store it performs keeps its statement.
    assert_eq!(report.representation, Representation::Java);
    assert!(
        report.text.contains("self.real();") && report.text.contains("return local1;"),
        "{}",
        report.text
    );
}

#[test]
fn a_cast_a_bridge_applies_to_a_parameter_is_not_an_erasure() {
    let class = bridge_class(
        "p",
        "(Ljava/lang/Object;)Ljava/lang/String;",
        BRIDGE_FLAGS,
        BridgeBody::ParameterCast,
    );
    let report = present(
        &class,
        b"p",
        b"(Ljava/lang/Object;)Ljava/lang/String;",
        2,
        vec![Some("self".into()), Some("value".into())],
    );
    assert_eq!(report.bridges.len(), 1);
    let bridge = &report.bridges[0];
    assert!(!bridge.presented());
    let refusal = bridge.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_bridge_cast_not_erasure");
    assert!(refusal.message.contains("can fail"), "{}", refusal.message);
    // The cast is a check this layer cannot prove cannot fail, so the body is quoted where it is.
    assert!(report.text.contains("// @bytecode"), "{}", report.text);
    assert_eq!(report.representation, Representation::Mixed);
}

#[test]
fn a_run_that_states_no_access_flags_cannot_decide_a_bridge() {
    let class = bridge_class(
        "c",
        "()Ljava/lang/Object;",
        BRIDGE_FLAGS,
        BridgeBody::ForwardWithErasedCast,
    );
    let payload = analyze(&class, b"c", b"()Ljava/lang/Object;");
    let facts = facts_without_flags(&class, b"c", 1);
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, None, &mut budget);
    assert_eq!(report.bridges.len(), 1);
    let bridge = &report.bridges[0];
    assert!(!bridge.presented());
    let refusal = bridge.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_bridge_flags_missing");
    assert_eq!(
        refusal.requirement.as_deref(),
        Some("the `access_flags` attribute")
    );
    assert!(report.text.contains("// @bytecode"), "{}", report.text);
}

// ---------------------------------------------------------------------------
// A synthetic accessor: the field access the call site becomes (A12)
// ---------------------------------------------------------------------------

/// `Test` with one private field and the members a compiler generates for accesses to it, plus the
/// bodies that call them:
///
/// ```text
/// f:I                            the field every accessor reaches
/// access$100(LTest;)I            static synthetic: aload_0; getfield Test.f:I; ireturn
/// access$102(LTest;I)V           static synthetic: aload_0; iload_1; putfield Test.f:I; return
/// access$200(LTest;)I            static synthetic: the same read **and** an increment
/// access$300(LTest;)I            private static: the pure read, declared by a source
/// method()I                      iconst_0; istore_1; aload_0; invokestatic access$100; ireturn
/// write(I)V                      aload_0; iload_1; invokestatic access$102; return
/// other_200()I                   aload_0; invokestatic access$200; ireturn
/// other_300()I                   aload_0; invokestatic access$300; ireturn
/// combined()Ljava/lang/String;   new StringBuilder; dup; <init>; aload_0;
///                                invokestatic access$100; append(int); ldc "!";
///                                append(String); toString; areturn
/// ```
fn accessor_class() -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let field_name = pool.utf8("f");
    let field_descriptor = pool.utf8("I");
    let field = field_ref(&mut pool, test, "f", "I");
    let concat = concat_pool(&mut pool, STRING_BUILDER);
    let append_int = member_ref(
        &mut pool,
        concat.class,
        "append",
        &format!("(I)L{STRING_BUILDER};"),
    );
    let to_string = member_ref(&mut pool, concat.class, "toString", "()Ljava/lang/String;");
    let read = member_ref(&mut pool, test, "access$100", "(LTest;)I");
    let write = member_ref(&mut pool, test, "access$102", "(LTest;I)V");
    let extra = member_ref(&mut pool, test, "access$200", "(LTest;)I");
    let plain = member_ref(&mut pool, test, "access$300", "(LTest;)I");
    let literal_name = pool.utf8("!");
    let literal = pool.string(literal_name);
    let access100 = pool.utf8("access$100");
    let access102 = pool.utf8("access$102");
    let access200 = pool.utf8("access$200");
    let access300 = pool.utf8("access$300");
    let method_name = pool.utf8("method");
    let write_name = pool.utf8("write");
    let other_200_name = pool.utf8("other_200");
    let other_300_name = pool.utf8("other_300");
    let combined_name = pool.utf8("combined");
    let access_int = pool.utf8("(LTest;)I");
    let access_setter = pool.utf8("(LTest;I)V");
    let no_arguments = pool.utf8("()I");
    let one_int = pool.utf8("(I)V");
    let returns_string = pool.utf8("()Ljava/lang/String;");
    let short = |index: u16| u8::try_from(index).expect("a fixture's `ldc` index fits a byte");
    let read_body = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0xb4)
        .index(field) // 1: getfield Test.f:I
        .op(0xac) // 4: ireturn
        .done();
    let write_body = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0x1b) // 1: iload_1
        .op(0xb5)
        .index(field) // 2: putfield Test.f:I
        .op(0xb1) // 5: return
        .done();
    let extra_body = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0xb4)
        .index(field) // 1: getfield Test.f:I
        .op(0x04) // 4: iconst_1
        .op(0x60) // 5: iadd
        .op(0xac) // 6: ireturn
        .done();
    let method_body = Code::default()
        .op(0x03) // 0: iconst_0
        .op(0x3c) // 1: istore_1
        .op(0x2a) // 2: aload_0
        .op(0xb8)
        .index(read) // 3: invokestatic access$100
        .op(0xac) // 6: ireturn
        .done();
    let write_call_body = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0x1b) // 1: iload_1
        .op(0xb8)
        .index(write) // 2: invokestatic access$102
        .op(0xb1) // 5: return
        .done();
    let other_200_body = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0xb8)
        .index(extra) // 1: invokestatic access$200
        .op(0xac) // 4: ireturn
        .done();
    let other_300_body = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0xb8)
        .index(plain) // 1: invokestatic access$300
        .op(0xac) // 4: ireturn
        .done();
    let combined_body = Code::default()
        .op(0xbb)
        .index(concat.class) // 0:  new
        .op(0x59) // 3:  dup
        .op(0xb7)
        .index(concat.init) // 4:  invokespecial <init>
        .op(0x2a) // 7:  aload_0
        .op(0xb8)
        .index(read) // 8:  invokestatic access$100
        .op(0xb6)
        .index(append_int) // 11: append(int)
        .op(0x12)
        .byte(short(literal)) // 14: ldc "!"
        .op(0xb6)
        .index(concat.append_string) // 16: append(String)
        .op(0xb6)
        .index(to_string) // 19: toString
        .op(0xb0) // 22: areturn
        .done();
    class_bytes(
        &pool,
        test,
        object,
        &[FieldDef {
            flags: 0x0002,
            name: field_name,
            descriptor: field_descriptor,
        }],
        &[
            MemberDef {
                flags: 0x1008,
                name: access100,
                descriptor: access_int,
                max_stack: 1,
                max_locals: 1,
                code: read_body.clone(),
            },
            MemberDef {
                flags: 0x1008,
                name: access102,
                descriptor: access_setter,
                max_stack: 2,
                max_locals: 2,
                code: write_body,
            },
            MemberDef {
                flags: 0x1008,
                name: access200,
                descriptor: access_int,
                max_stack: 1,
                max_locals: 1,
                code: extra_body,
            },
            MemberDef {
                flags: 0x000a,
                name: access300,
                descriptor: access_int,
                max_stack: 1,
                max_locals: 1,
                code: read_body.clone(),
            },
            MemberDef {
                flags: 0x0001,
                name: method_name,
                descriptor: no_arguments,
                max_stack: 1,
                max_locals: 2,
                code: method_body,
            },
            MemberDef {
                flags: 0x0001,
                name: write_name,
                descriptor: one_int,
                max_stack: 2,
                max_locals: 2,
                code: write_call_body,
            },
            MemberDef {
                flags: 0x0001,
                name: other_200_name,
                descriptor: no_arguments,
                max_stack: 1,
                max_locals: 1,
                code: other_200_body,
            },
            MemberDef {
                flags: 0x0001,
                name: other_300_name,
                descriptor: no_arguments,
                max_stack: 1,
                max_locals: 1,
                code: other_300_body,
            },
            MemberDef {
                flags: 0x0001,
                name: combined_name,
                descriptor: returns_string,
                max_stack: 2,
                max_locals: 1,
                code: combined_body,
            },
        ],
    )
}

#[test]
fn a_synthetic_accessors_call_site_is_presented_as_the_field_access_it_forwards() {
    let class = accessor_class();
    let report = present(&class, b"method", b"()I", 1, vec![Some("self".into())]);
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert_eq!(report.quality, Quality::Structured);
    assert!(report.text.contains("int local1 = 0;"), "{}", report.text);
    assert!(report.text.contains("return self.f;"), "{}", report.text);

    // The evidence: the call site, the member the class declared (with its flags), the field the
    // member's own body reads, and which of the two bodies it is.
    assert_eq!(report.accessors.len(), 1);
    let accessor = &report.accessors[0];
    assert!(accessor.presented());
    assert_eq!(accessor.call_site, 3);
    assert_eq!(accessor.name, "access$100");
    assert_eq!(accessor.descriptor, "(LTest;)I");
    assert_eq!(accessor.access_flags, Some(0x1008));
    assert_eq!(
        accessor.field,
        Some(AccessorField {
            owner: "Test".into(),
            name: "f".into(),
            descriptor: "I".into(),
        })
    );
    assert_eq!(accessor.shape, Some(AccessorShape::FieldRead));
    assert_eq!(accessor.rule().citation(), "accessor@1");
    assert!(report.rules.contains(&accessor.rule()));
    assert!(report.concats.is_empty());

    // The A12 half that lives in the artifact: the derived presentation carries **both** original
    // BCIs — the call site's own (BCI 3 in this body) and the field access inside the accessor's
    // body (BCI 1 there) — and it marks the second one `Derived`.
    let field = report
        .source_map
        .segments()
        .iter()
        .find(|segment| segment.text(&report.text) == "self.f")
        .expect("the field access is a node of its own");
    assert_eq!(field.origin().primary().bci(), 3);
    assert_eq!(field.origin().primary().provenance(), Provenance::Direct);
    assert_eq!(field.origin().derived().len(), 1);
    assert_eq!(field.origin().derived()[0].bci(), 1);
    assert_eq!(
        field.origin().derived()[0].provenance(),
        Provenance::Derived
    );
}

#[test]
fn a_write_accessor_becomes_the_assignment_it_performs() {
    let class = accessor_class();
    let report = present(
        &class,
        b"write",
        b"(I)V",
        2,
        vec![Some("self".into()), Some("value".into())],
    );
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert!(report.text.contains("self.f = value;"), "{}", report.text);
    assert_eq!(report.accessors.len(), 1);
    let accessor = &report.accessors[0];
    assert!(accessor.presented());
    assert_eq!(accessor.shape, Some(AccessorShape::FieldWrite));
    assert_eq!(accessor.call_site, 2);
    assert_eq!(accessor.access_flags, Some(0x1008));
    assert_eq!(
        accessor.field.as_ref().map(|field| field.name.as_str()),
        Some("f")
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_accessor_sites"),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn an_accessor_whose_body_does_more_than_forward_is_refused() {
    let class = accessor_class();
    let report = present(&class, b"other_200", b"()I", 1, vec![Some("self".into())]);
    assert_eq!(report.accessors.len(), 1);
    let accessor = &report.accessors[0];
    assert!(!accessor.presented());
    let refusal = accessor.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_accessor_body");
    assert!(
        refusal.message.contains("access$200"),
        "{}",
        refusal.message
    );
    assert_eq!(refusal.rule.citation(), "accessor@1");
    // The call it had is the call it keeps: nothing about the member's body is presented.
    assert!(report.text.contains("access$200(self)"), "{}", report.text);
}

#[test]
fn a_member_the_class_did_not_declare_synthetic_is_not_an_accessor() {
    let class = accessor_class();
    let report = present(&class, b"other_300", b"()I", 1, vec![Some("self".into())]);
    assert_eq!(report.accessors.len(), 1);
    let accessor = &report.accessors[0];
    assert!(!accessor.presented());
    assert_eq!(accessor.access_flags, Some(0x000a));
    let refusal = accessor.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_accessor_declaration");
    assert!(
        refusal.message.contains("static and synthetic"),
        "{}",
        refusal.message
    );
    assert!(report.text.contains("access$300(self)"), "{}", report.text);
}

#[test]
fn a_run_with_no_member_table_states_the_table_it_is_missing() {
    let class = accessor_class();
    let payload = analyze(&class, b"method", b"()I");
    let facts = facts_of(&class, b"method", 1, vec![Some("self".into())]);
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, None, &mut budget);
    assert_eq!(report.accessors.len(), 1);
    let accessor = &report.accessors[0];
    assert!(!accessor.presented());
    assert_eq!(accessor.access_flags, None);
    let refusal = accessor.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_accessor_members_missing");
    assert_eq!(
        refusal.requirement.as_deref(),
        Some("the `class members` table of this run")
    );
    assert!(report.text.contains("access$100(self)"), "{}", report.text);
}

#[test]
fn a_body_that_builds_a_concatenation_out_of_an_accessor_is_presented_as_both() {
    let class = accessor_class();
    let report = present(
        &class,
        b"combined",
        b"()Ljava/lang/String;",
        1,
        vec![Some("self".into())],
    );
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert_eq!(report.quality, Quality::Structured);
    assert!(
        report.text.contains("return self.f + \"!\";"),
        "{}",
        report.text
    );
    assert_eq!(report.concats.len(), 1);
    assert!(report.concats[0].presented());
    assert_eq!(report.accessors.len(), 1);
    assert!(report.accessors[0].presented());
    assert_eq!(report.accessors[0].call_site, 8);
    assert!(
        !report.text.contains("access$"),
        "the accessor is not called in the text:\n{}",
        report.text
    );
    let rules: Vec<String> = report.rules.iter().map(|rule| rule.citation()).collect();
    assert!(rules.contains(&"concat@1".to_string()), "{rules:?}");
    assert!(rules.contains(&"accessor@1".to_string()), "{rules:?}");
}

#[test]
fn a_body_without_debug_metadata_is_still_presented_with_deterministic_names() {
    let class = accessor_class();
    let first = present(&class, b"combined", b"()Ljava/lang/String;", 1, Vec::new());
    let second = present(&class, b"combined", b"()Ljava/lang/String;", 1, Vec::new());
    assert_eq!(
        first, second,
        "the presentation is a function of the evidence"
    );
    assert!(!first.text.contains("self"), "{}", first.text);
    assert!(
        first.text.contains(".f + \"!\""),
        "the field access and the literal are still written:\n{}",
        first.text
    );
    assert!(first.accessors[0].presented());
    assert!(first.concats[0].presented());
}

// ---------------------------------------------------------------------------
// The independent oracle
// ---------------------------------------------------------------------------
//
// Two readings of the same fixture: this model's own decoder, stack machine and text parser, and the
// run's. The model shares the fixture's bytecode — the ground truth it checks — and nothing else.
//
// ORACLE MODEL BEGIN

/// The value the model states the fixture's accessor reads out of its field: the model's own ground
/// truth for the fixture, chosen here and not read from a presentation.
const FIELD: i64 = 5;
/// The name of the field the accessor the model states reaches.
const FIELD_NAME: &str = "f";

/// What the model states each called member of a fixture returns.
///
/// Both called members return the same text on purpose: then a text that calls them in the wrong
/// order is the *only* thing the calls' own comparison can catch, and nothing about the value hides
/// it.
fn called(name: &str) -> &'static str {
    match name.rsplit('.').next().expect("a member name") {
        "f" | "g" => "X",
        other => panic!("the model does not know the member `{other}`"),
    }
}

/// One cell of the model's stack machine.
#[derive(Clone, Debug, PartialEq)]
enum Cell {
    Int(i64),
    Text(String),
    Builder(String),
}

impl Cell {
    /// What the cell contributes to a string being built.
    fn written(&self) -> String {
        match self {
            Self::Int(value) => value.to_string(),
            Self::Text(text) | Self::Builder(text) => text.clone(),
        }
    }
}

/// What one side of the comparison observed.
#[derive(Debug, PartialEq)]
struct Observed {
    /// Every call the side makes, in the order it makes them.
    calls: Vec<String>,
    /// The value the side produced.
    returned: Option<String>,
    /// How many times the side spells an accessor call.
    accessor_calls: usize,
    /// Every field the side reads instead of one, by name.
    accessor_reads: Vec<String>,
}

/// The constant pool of one fixture, read by the model's own parser.
struct OraclePool {
    tags: Vec<u8>,
    first: Vec<u16>,
    second: Vec<u16>,
    text: Vec<String>,
}

/// Reads one fixture's pool. A tag the model does not read is a fixture it cannot check, and it says
/// so rather than guessing.
fn read_pool(class: &[u8]) -> OraclePool {
    let count = usize::from(u16::from_be_bytes([class[8], class[9]]));
    let mut at = 10;
    let mut pool = OraclePool {
        tags: vec![0],
        first: vec![0],
        second: vec![0],
        text: vec![String::new()],
    };
    while pool.tags.len() < count {
        let tag = class[at];
        at += 1;
        let mut text = String::new();
        let (first, second) = match tag {
            1 => {
                let length = usize::from(u16::from_be_bytes([class[at], class[at + 1]]));
                at += 2;
                text = String::from_utf8_lossy(&class[at..at + length]).into_owned();
                at += length;
                (0, 0)
            }
            3 => {
                let value =
                    i32::from_be_bytes([class[at], class[at + 1], class[at + 2], class[at + 3]]);
                at += 4;
                (u16::try_from(value).expect("a fixture's small integer"), 0)
            }
            7 | 8 | 16 => {
                let index = u16::from_be_bytes([class[at], class[at + 1]]);
                at += 2;
                (index, 0)
            }
            9 | 10 | 11 | 12 | 18 => {
                let owner = u16::from_be_bytes([class[at], class[at + 1]]);
                let name = u16::from_be_bytes([class[at + 2], class[at + 3]]);
                at += 4;
                (owner, name)
            }
            other => panic!("the model does not read the pool tag {other}"),
        };
        pool.tags.push(tag);
        pool.first.push(first);
        pool.second.push(second);
        pool.text.push(text);
    }
    pool
}

impl OraclePool {
    fn text(&self, index: u16) -> &str {
        &self.text[usize::from(index)]
    }

    fn class_name(&self, index: u16) -> &str {
        self.text(self.first[usize::from(index)])
    }

    /// The member a reference names, as `Owner.name`.
    fn member(&self, index: u16) -> String {
        let index = usize::from(index);
        let name_and_type = usize::from(self.second[index]);
        format!(
            "{}.{}",
            self.class_name(self.first[index]),
            self.text(self.first[name_and_type])
        )
    }

    /// The value one `ldc` pushes.
    fn constant(&self, index: u16) -> Cell {
        let index = usize::from(index);
        match self.tags[index] {
            8 => Cell::Text(self.text(self.first[index]).to_string()),
            3 => Cell::Int(i64::from(self.first[index] as i16)),
            other => panic!("the model does not push a constant of the tag {other}"),
        }
    }
}

/// Runs one fixture body: the model's own decoder and stack machine.
fn run_bytecode(code: &[u8], pool: &OraclePool, receiver: Option<&str>) -> Observed {
    let mut stack: Vec<Cell> = Vec::new();
    let locals: Vec<Cell> = receiver
        .map(|name| vec![Cell::Text(name.to_string())])
        .unwrap_or_default();
    let mut observed = Observed {
        calls: Vec::new(),
        returned: None,
        accessor_calls: 0,
        accessor_reads: Vec::new(),
    };
    let mut at = 0;
    while at < code.len() {
        let opcode = code[at];
        match opcode {
            0xbb => {
                stack.push(Cell::Builder(String::new()));
                at += 3;
            }
            0x59 => {
                let top = stack
                    .last()
                    .cloned()
                    .expect("`dup` needs something to copy");
                stack.push(top);
                at += 1;
            }
            // The constructor consumes the copy the allocation pushed; the instance is the same one.
            0xb7 => {
                stack.pop();
                at += 3;
            }
            0x12 => {
                let index = u16::from(code[at + 1]);
                stack.push(pool.constant(index));
                at += 2;
            }
            0x10 => {
                stack.push(Cell::Int(i64::from(code[at + 1] as i8)));
                at += 2;
            }
            0x2a | 0x1a => {
                stack.push(locals[0].clone());
                at += 1;
            }
            0xb6 | 0xb8 => {
                let index = u16::from_be_bytes([code[at + 1], code[at + 2]]);
                let member = pool.member(index);
                let name = member
                    .rsplit('.')
                    .next()
                    .expect("a member name")
                    .to_string();
                match name.as_str() {
                    "append" => {
                        let value = stack.pop().expect("`append` reads a value");
                        let mut receiver = stack.pop().expect("`append` reads its receiver");
                        match &mut receiver {
                            Cell::Builder(text) => text.push_str(&value.written()),
                            _ => panic!("`append` was called on a value that is not a builder"),
                        }
                        stack.push(receiver);
                    }
                    "toString" => {
                        let receiver = stack.pop().expect("`toString` reads its receiver");
                        stack.push(Cell::Text(receiver.written()));
                    }
                    _ if member.contains("access$") => {
                        stack.pop().expect("the accessor reads the instance");
                        observed.accessor_calls += 1;
                        stack.push(Cell::Int(FIELD));
                    }
                    _ => {
                        // The two sides are compared by the member's **own name**: how a static call
                        // is qualified in the text is a presentation decision this model does not
                        // check, and the question it does check is which call happens when.
                        observed.calls.push(name.clone());
                        stack.push(Cell::Text(called(&member).to_string()));
                    }
                }
                at += 3;
            }
            // The cast keeps the value it is given.
            0xc0 => at += 3,
            0xb0 | 0xac => {
                observed.returned = Some(stack.pop().expect("a return needs a value").written());
                at += 1;
            }
            other => panic!("the model does not decode the opcode {other:#04x}"),
        }
    }
    observed
}

/// Runs one produced artifact: the model's own parser and evaluator for the subset it knows.
fn run_text(artifact: &str) -> Observed {
    let expression = artifact
        .lines()
        .find(|line| line.contains("return "))
        .expect("the artifact has a return statement")
        .trim()
        .trim_start_matches("return ")
        .trim_end_matches(';');
    let mut observed = Observed {
        calls: Vec::new(),
        returned: None,
        accessor_calls: 0,
        accessor_reads: Vec::new(),
    };
    let mut pieces: Vec<String> = Vec::new();
    for term in expression.split(" + ") {
        let term = term.trim();
        if term.starts_with('"') && term.ends_with('"') {
            pieces.push(term.trim_matches('"').to_string());
            continue;
        }
        if !term.is_empty() && term.chars().all(|character| character.is_ascii_digit()) {
            pieces.push(term.to_string());
            continue;
        }
        if let Some(callee) = term.strip_suffix("()") {
            if callee.contains("access$") {
                observed.accessor_calls += 1;
                pieces.push(FIELD.to_string());
                continue;
            }
            observed.calls.push(callee.to_string());
            pieces.push(called(callee).to_string());
            continue;
        }
        let (_, field) = term
            .rsplit_once('.')
            .unwrap_or_else(|| panic!("the model does not read the term `{term}`"));
        observed.accessor_reads.push(field.to_string());
        pieces.push(FIELD.to_string());
    }
    observed.returned = Some(pieces.concat());
    observed
}

/// The comparison: the calls in order, the value, and the two spellings of an accessor.
fn compare(from_bytes: &Observed, from_text: &Observed) -> Result<(), String> {
    if from_bytes.calls != from_text.calls {
        return Err(format!(
            "the calls differ: the bytecode makes {:?} and the text {:?}",
            from_bytes.calls, from_text.calls
        ));
    }
    if from_bytes.returned != from_text.returned {
        return Err(format!(
            "the value differs: the bytecode produced {:?} and the text {:?}",
            from_bytes.returned, from_text.returned
        ));
    }
    if from_bytes.accessor_calls != from_text.accessor_reads.len() {
        return Err(format!(
            "the accessor is called {} time(s) in the bytecode and written as {} field read(s) in the text",
            from_bytes.accessor_calls,
            from_text.accessor_reads.len()
        ));
    }
    if from_text.accessor_calls != 0 {
        return Err(format!(
            "the text still calls the accessor {} time(s), and the presentation is a field access",
            from_text.accessor_calls
        ));
    }
    if let Some(field) = from_text
        .accessor_reads
        .iter()
        .find(|field| field.as_str() != FIELD_NAME)
    {
        return Err(format!(
            "the text reads the field `{field}`, and the accessor the model states reaches `{FIELD_NAME}`"
        ));
    }
    Ok(())
}
// ORACLE MODEL END

/// The model's concatenation fixture: `Test.method()Ljava/lang/String;` = `"x" + Test.f() + Test.g()`.
fn oracle_concat_class() -> (Vec<u8>, Vec<u8>) {
    let (mut pool, _code, test, object) = base_pool();
    let concat = concat_pool(&mut pool, STRING_BUILDER);
    let to_string = member_ref(&mut pool, concat.class, "toString", "()Ljava/lang/String;");
    let f = member_ref(&mut pool, test, "f", "()Ljava/lang/String;");
    let g = member_ref(&mut pool, test, "g", "()Ljava/lang/String;");
    let literal_name = pool.utf8("x");
    let literal = pool.string(literal_name);
    let method = pool.utf8("method");
    let method_descriptor = pool.utf8("()Ljava/lang/String;");
    let returns_string = pool.utf8("()Ljava/lang/String;");
    let f_name = pool.utf8("f");
    let g_name = pool.utf8("g");
    let result_name = pool.utf8("X");
    let result = pool.string(result_name);
    let short = |index: u16| u8::try_from(index).expect("a fixture's `ldc` index fits a byte");
    let code = Code::default()
        .op(0xbb)
        .index(concat.class) // 0:  new
        .op(0x59) // 3:  dup
        .op(0xb7)
        .index(concat.init) // 4:  invokespecial <init>
        .op(0x12)
        .byte(short(literal)) // 7:  ldc "x"
        .op(0xb6)
        .index(concat.append_string) // 9:  append(String)
        .op(0xb8)
        .index(f) // 12: invokestatic Test.f
        .op(0xb6)
        .index(concat.append_string) // 15: append(String)
        .op(0xb8)
        .index(g) // 18: invokestatic Test.g
        .op(0xb6)
        .index(concat.append_string) // 21: append(String)
        .op(0xb6)
        .index(to_string) // 24: toString
        .op(0xb0) // 27: areturn
        .done();
    let member = Code::default().op(0x12).byte(short(result)).op(0xb0).done();
    let class = class_bytes(
        &pool,
        test,
        object,
        &[],
        &[
            MemberDef {
                flags: 0x0009,
                name: method,
                descriptor: method_descriptor,
                max_stack: 2,
                max_locals: 0,
                code: code.clone(),
            },
            MemberDef {
                flags: 0x0009,
                name: f_name,
                descriptor: returns_string,
                max_stack: 1,
                max_locals: 0,
                code: member.clone(),
            },
            MemberDef {
                flags: 0x0009,
                name: g_name,
                descriptor: returns_string,
                max_stack: 1,
                max_locals: 0,
                code: member,
            },
        ],
    );
    (class, code)
}

/// The model's accessor fixture: `Test.combined()Ljava/lang/String;` reads the field through a
/// synthetic accessor and appends a literal.
fn oracle_accessor_class() -> (Vec<u8>, Vec<u8>) {
    let (mut pool, _code, test, object) = base_pool();
    let field_name = pool.utf8("f");
    let field_descriptor = pool.utf8("I");
    let field = field_ref(&mut pool, test, "f", "I");
    let concat = concat_pool(&mut pool, STRING_BUILDER);
    let append_int = member_ref(
        &mut pool,
        concat.class,
        "append",
        &format!("(I)L{STRING_BUILDER};"),
    );
    let to_string = member_ref(&mut pool, concat.class, "toString", "()Ljava/lang/String;");
    let access = member_ref(&mut pool, test, "access$100", "(LTest;)I");
    let literal_name = pool.utf8("!");
    let literal = pool.string(literal_name);
    let access_name = pool.utf8("access$100");
    let access_descriptor = pool.utf8("(LTest;)I");
    let combined_name = pool.utf8("combined");
    let combined_descriptor = pool.utf8("()Ljava/lang/String;");
    let short = |index: u16| u8::try_from(index).expect("a fixture's `ldc` index fits a byte");
    let accessor = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0xb4)
        .index(field) // 1: getfield Test.f:I
        .op(0xac) // 4: ireturn
        .done();
    let code = Code::default()
        .op(0xbb)
        .index(concat.class) // 0:  new
        .op(0x59) // 3:  dup
        .op(0xb7)
        .index(concat.init) // 4:  invokespecial <init>
        .op(0x2a) // 7:  aload_0
        .op(0xb8)
        .index(access) // 8:  invokestatic access$100
        .op(0xb6)
        .index(append_int) // 11: append(int)
        .op(0x12)
        .byte(short(literal)) // 14: ldc "!"
        .op(0xb6)
        .index(concat.append_string) // 16: append(String)
        .op(0xb6)
        .index(to_string) // 19: toString
        .op(0xb0) // 22: areturn
        .done();
    let class = class_bytes(
        &pool,
        test,
        object,
        &[FieldDef {
            flags: 0x0002,
            name: field_name,
            descriptor: field_descriptor,
        }],
        &[
            MemberDef {
                flags: 0x1008,
                name: access_name,
                descriptor: access_descriptor,
                max_stack: 1,
                max_locals: 1,
                code: accessor,
            },
            MemberDef {
                flags: 0x0001,
                name: combined_name,
                descriptor: combined_descriptor,
                max_stack: 2,
                max_locals: 1,
                code: code.clone(),
            },
        ],
    );
    (class, code)
}

#[test]
fn the_oracle_agrees_with_the_run_on_a_concatenation_and_on_a_field_access() {
    // The chain: the calls, in the fixture's own order, once each.
    let (class, code) = oracle_concat_class();
    let report = present(&class, b"method", b"()Ljava/lang/String;", 0, Vec::new());
    assert!(report.produced(), "{:?}", report.stop());
    let pool = read_pool(&class);
    let from_bytes = run_bytecode(&code, &pool, None);
    let from_text = run_text(&report.text);
    assert_eq!(
        from_bytes.calls,
        vec!["f".to_string(), "g".to_string()],
        "the model read the fixture's own two calls"
    );
    assert!(
        compare(&from_bytes, &from_text).is_ok(),
        "{from_bytes:?} against {from_text:?}\n{}",
        report.text
    );

    // The field access: the accessor is called once in the bytecode and written as one field read,
    // and the text spells no call for it at all.
    let (class, code) = oracle_accessor_class();
    let report = present(
        &class,
        b"combined",
        b"()Ljava/lang/String;",
        1,
        vec![Some("self".into())],
    );
    assert!(report.accessors[0].presented(), "{:?}", report.accessors);
    let pool = read_pool(&class);
    let from_bytes = run_bytecode(&code, &pool, Some("self"));
    let from_text = run_text(&report.text);
    assert_eq!(from_bytes.accessor_calls, 1);
    assert_eq!(from_text.accessor_reads, vec![FIELD_NAME.to_string()]);
    assert!(
        compare(&from_bytes, &from_text).is_ok(),
        "{from_bytes:?} against {from_text:?}\n{}",
        report.text
    );
}

#[test]
fn the_oracle_rejects_a_concatenation_whose_calls_are_written_in_another_order() {
    let (class, code) = oracle_concat_class();
    let report = present(&class, b"method", b"()Ljava/lang/String;", 0, Vec::new());
    let pool = read_pool(&class);
    let from_bytes = run_bytecode(&code, &pool, None);
    // The same artifact with the two calls swapped: the model must see that the bytecode called `f`
    // before `g` and the text calls `g` before `f`, however equal the values end up being.
    let swapped = report
        .text
        .replace("f()", "\u{0}")
        .replace("g()", "f()")
        .replace('\u{0}', "g()");
    assert!(swapped.contains("g() + f()"), "{swapped}");
    let from_text = run_text(&swapped);
    let rejected = compare(&from_bytes, &from_text);
    assert!(
        rejected.is_err(),
        "a reordered concatenation is not the same expression: {from_text:?}"
    );
}

#[test]
fn the_oracle_rejects_a_concatenation_that_calls_an_operand_twice() {
    let (class, code) = oracle_concat_class();
    let report = present(&class, b"method", b"()Ljava/lang/String;", 0, Vec::new());
    let pool = read_pool(&class);
    let from_bytes = run_bytecode(&code, &pool, None);
    let doubled = report.text.replace("f() + ", "f() + f() + ");
    assert!(doubled.contains("f() + f()"), "{doubled}");
    let from_text = run_text(&doubled);
    assert!(
        compare(&from_bytes, &from_text).is_err(),
        "an operand evaluated twice is not the same body: {from_text:?}"
    );
}

#[test]
fn the_models_section_is_independent_of_the_presentation() {
    const SOURCE: &str = include_str!("p3_patterns.rs");
    let section = SOURCE
        .split("ORACLE MODEL BEGIN")
        .nth(1)
        .expect("the model section is marked")
        .split("ORACLE MODEL END")
        .next()
        .expect("the model section has an end");
    // The model reads the fixture's bytes and the produced text, and nothing the presentation
    // defines: a reader of the model can check it without knowing what it is being checked against.
    for forbidden in [
        "jarde_java",
        "Recovery",
        "recover(",
        "Region",
        "ExprKind",
        "StmtKind",
        "text_of_bci",
        "concat::",
        "accessor::",
        "bridge::",
        "build::",
        "emit::",
        "Segment",
        "SourceMap",
    ] {
        assert!(
            !section.contains(forbidden),
            "the model section must not be written in terms of `{forbidden}`"
        );
    }
    // And it is not vacuous: the four things it has to have are in it.
    for present in [
        "fn run_bytecode",
        "fn run_text",
        "fn compare",
        "fn read_pool",
    ] {
        assert!(
            section.contains(present),
            "the model section holds `{present}`"
        );
    }
}

#[test]
fn a_budget_that_refuses_the_emission_of_a_new_shape_hands_out_nothing() {
    let class = accessor_class();
    let payload = analyze(&class, b"combined", b"()Ljava/lang/String;");
    let facts = facts_of(&class, b"combined", 1, vec![Some("self".into())]);
    let members = members_of(&class);
    // The envelope alone does not fit: the stop happens inside the emission, and a stopped run
    // presents no shape — no text, no segments, and no record that would claim one.
    let mut budget = Budget::new(Limits {
        output_bytes: 16,
        ..limits()
    });
    let report = recover_body(&payload, &facts, Some(&members), &mut budget);
    assert!(!report.produced(), "{:?}", report.outcome);
    assert!(matches!(report.stop(), Some(StopReason::Budget { .. })));
    assert!(report.text.is_empty(), "{}", report.text);
    assert_eq!(report.source_map.len(), 0);
    assert!(report.concats.is_empty());
    assert!(report.accessors.is_empty());
    assert!(report.bridges.is_empty());
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_output_budget"),
        "{:?}",
        report.diagnostics
    );
}
