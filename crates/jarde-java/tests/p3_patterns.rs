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
    AccessorField, AccessorShape, ClassMembers, ConstructorTarget, DeclarationForm, DeclaringClass,
    MemberBody, MethodFacts, Provenance, RecoveryFacts, RecoveryRequest, StopReason, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest, Quality, Representation, SyntaxStatus};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::classfile::{class_facts, method_code_facts};
use jarde_reader::model::{
    ClassBytesId, Digest, ExecutionReport, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
    PhysicalMethodId, PhysicalVariant,
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
        roots: vec![LoadRoot::StandaloneClass {
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
        .with_access_flags(member.access_flags)
        .with_declaring_class(DeclaringClass::new(
            String::from_utf8_lossy(&header.this_class.raw().0).into_owned(),
            header.access_flags,
        )),
    )
    .with_debug_locals(stated_names(debug))
}

/// The old facts seam, kept for tests whose subject is the absence of the declaring-class fact.
fn facts_without_declaring_class(
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
    .with_debug_locals(stated_names(debug))
}

/// The debug records of a test that states **one name per slot**: each name covers its slot with no
/// range stated, so no slot is ever split for them (P3 3.4).
fn stated_names(debug: Vec<Option<String>>) -> Vec<jarde_java::DebugLocal> {
    debug
        .into_iter()
        .enumerate()
        .filter_map(|(slot, name)| {
            name.map(|name| {
                jarde_java::DebugLocal::named(u16::try_from(slot).unwrap_or(u16::MAX), name)
            })
        })
        .collect()
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

/// The class-file definition one fixture's bytes are, as this file builds it in every helper.
fn definition_of(class: &[u8]) -> PhysicalDefinitionId {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
    }
}

/// One member's physical identity in the fixture's own definition.
fn member_identity(class: &[u8], name: &[u8], descriptor: &[u8]) -> PhysicalMethodId {
    PhysicalMethodId {
        owner: definition_of(class),
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
    }
}

/// Every member of the fixture's class, as the same read decoded them: the declaration plus the
/// decoded body. This is the evidence a synthetic accessor call site is decided from.
///
/// Each member's identity is built from the **same** class-file definition the presented payload was
/// analyzed under (P3 3.2): the fixture's own bytes, by digest and length, in the one snapshot this
/// helper opens for itself. So a member of this table is a member of the definition the presented
/// body was read from — which is what makes the derived anchor's BCI and constant pool a coordinate
/// in one class file rather than in "some class called `Test`".
fn members_of(class: &[u8]) -> ClassMembers {
    let mut budget = Budget::new(limits());
    let header = class_facts(class, &mut budget).expect("the fixture is a class file");
    let owner = String::from_utf8_lossy(&header.this_class.raw().0).into_owned();
    let definition = definition_of(class);
    let members = header
        .methods
        .iter()
        .map(|member| {
            let identity = PhysicalMethodId {
                owner: definition.clone(),
                name: member.name.raw().clone(),
                descriptor: member.descriptor.raw().clone(),
            };
            // Which members have a body is the class's own shell list, not an error from the reader:
            // a member the class declares without a `Code` attribute is a member of the table with
            // no body (P3 3.2).
            let has_code = member
                .attributes
                .iter()
                .any(|shell| shell.name.raw().0.as_slice() == b"Code");
            if !has_code {
                return MemberBody::without_body(owner.clone(), identity, member.access_flags);
            }
            let code = method_code_facts(class, member, &mut budget)
                .expect("a member the class declares a `Code` attribute for decodes");
            MemberBody::new(owner.clone(), identity, member.access_flags, code)
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
    let request = RecoveryRequest::new(payload.analysis.ir(), facts, jarde_java::pass::JAVA_8)
        .with_evidence(jarde_java::RecoveryEvidenceRequest::all());
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

/// Presents one specifically identified overload, for fixtures whose bridge and source method share
/// a name but differ in their return descriptors.
fn present_exact(
    class: &[u8],
    name: &[u8],
    descriptor: &[u8],
    parameters: u16,
    debug: Vec<Option<String>>,
) -> jarde_java::RecoveryReport {
    let payload = analyze(class, name, descriptor);
    let facts = facts_of_exact(class, name, descriptor, parameters, debug);
    let members = members_of(class);
    let mut budget = Budget::new(limits());
    recover_body(&payload, &facts, Some(&members), &mut budget)
}

fn facts_of_exact(
    class: &[u8],
    name: &[u8],
    descriptor: &[u8],
    parameters: u16,
    debug: Vec<Option<String>>,
) -> RecoveryFacts {
    let mut fact_budget = Budget::new(limits());
    let header = class_facts(class, &mut fact_budget).expect("the fixture is a class file");
    let member = header
        .methods
        .iter()
        .find(|member| member.name.raw().0 == name && member.descriptor.raw().0 == descriptor)
        .expect("the class declares the exact method identity");
    RecoveryFacts::new(
        MethodFacts::new(
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(descriptor),
            parameters,
        )
        .with_access_flags(member.access_flags)
        .with_declaring_class(DeclaringClass::new(
            String::from_utf8_lossy(&header.this_class.raw().0).into_owned(),
            header.access_flags,
        )),
    )
    .with_debug_locals(stated_names(debug))
}

fn recover_class_source_exact(
    class: &[u8],
    name: &[u8],
    descriptor: &[u8],
    parameters: u16,
    evidence: jarde_java::RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> jarde_java::report::ClassSourceRecovery {
    let payload = analyze(class, name, descriptor);
    let facts = facts_of_exact(
        class,
        name,
        descriptor,
        parameters,
        vec![Some("self".into())],
    );
    let members = members_of(class);
    let request = RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8)
        .with_evidence(evidence)
        .with_members(&members);
    jarde_java::report::recover_for_class_source(&request, budget, false, false)
}

/// Presents one body with an explicit IR-item limit, returning the budget so a test can assert
/// where a bounded quote walk stopped.
fn present_with_ir_limit(
    class: &[u8],
    name: &[u8],
    descriptor: &[u8],
    parameters: u16,
    debug: Vec<Option<String>>,
    ir_items: u64,
) -> (jarde_java::RecoveryReport, Budget) {
    let payload = analyze(class, name, descriptor);
    let facts = facts_of(class, name, parameters, debug);
    let members = members_of(class);
    let mut budget = Budget::new(Limits {
        ir_items,
        ..limits()
    });
    let report = recover_body(&payload, &facts, Some(&members), &mut budget);
    (report, budget)
}

/// Presents one fixture body while deliberately omitting its declaring-class fact.
fn present_without_declaring_class(
    class: &[u8],
    name: &[u8],
    descriptor: &[u8],
    parameters: u16,
    debug: Vec<Option<String>>,
) -> jarde_java::RecoveryReport {
    let payload = analyze(class, name, descriptor);
    let facts = facts_without_declaring_class(class, name, parameters, debug);
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

/// The class flags every fixture of the earlier slices writes: a plain `public` class.
const CLASS_FLAGS: u16 = 0x0021;

fn class_bytes(
    pool: &Pool,
    this_class: u16,
    super_class: u16,
    fields: &[FieldDef],
    methods: &[MemberDef],
) -> Vec<u8> {
    class_bytes_with(pool, CLASS_FLAGS, this_class, super_class, fields, methods)
}

/// The same class builder with the class's **own** flags stated: a fixture that is an interface (P3
/// 2.3's `declaration@1` reads `ACC_INTERFACE`) says so here, and nothing else about it changes.
fn class_bytes_with(
    pool: &Pool,
    class_flags: u16,
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
    out.extend_from_slice(&class_flags.to_be_bytes());
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
    //
    // **Changed by the concatenation-conversion fix (was: `["value", "f()", "\"x\""]`).** The
    // chain's first part is an `int`, so the first `+` is not a string concatenation until the empty
    // string starts it: `value + f() + "x"` adds nothing numerically *here* (the second operand is
    // already a `String`), but it is the same shape as `value + other + "x"` on two numeric parts,
    // where it adds numbers — and one rule for "the text starts in a string context" is what keeps
    // the two apart. The value is unchanged: `append(int)` is `String.valueOf(int)` exactly as
    // `"" + value` is. The empty string is written by the chain node, not as a part of its own: the
    // parts below are the chain's own `append`s, one each.
    assert_eq!(pieces(&report), vec!["\"\"", "value", "f()", "\"x\""]);
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
    // **Changed by P3 2.3 (was: the text contains `// @bytecode`).** The chain is still refused, and
    // the bytes it was built from are still presented: `new@1` writes the allocation and the calls
    // are written as the calls they are. What the refusal costs is the `+` spelling, not the
    // artifact — so the fact this test has to state is that no `+` exists and that both `append`s
    // appear exactly once, in the bytecode's order.
    assert!(!report.text.contains(" + "), "{}", report.text);
    assert!(
        report.text.contains("new java.lang.StringBuilder()"),
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("append(").count(),
        2,
        "each `append` is written once:\n{}",
        report.text
    );
    assert_eq!(report.representation, Representation::Java);
    assert_eq!(report.quality, Quality::Structured);
    assert_eq!(report.news.len(), 1);
    assert!(report.news[0].presented(), "{:?}", report.news);
    assert_eq!(report.news[0].class, STRING_BUILDER);
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
    // **Changed by P3 2.3 (was: the text contains `// @bytecode`).** The overload's conversion is
    // still not written away — no `+` appears anywhere — and the chain's own instructions are
    // presented as the calls they are, each once.
    assert!(!report.text.contains(" + "), "{}", report.text);
    assert_eq!(
        report.text.matches("append(").count(),
        2,
        "each `append` is written once:\n{}",
        report.text
    );
    assert_eq!(report.representation, Representation::Java);
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

fn assert_bridge_target(
    record: &jarde_java::BridgeRecord,
    owner: &str,
    name: &str,
    descriptor: &str,
    kind: jarde_java::facts::InvokeKind,
    interface_reference: bool,
) {
    let target = record
        .target
        .as_ref()
        .expect("the forward target is retained");
    assert_eq!(target.owner(), owner);
    assert_eq!(target.name(), name);
    assert_eq!(target.descriptor(), descriptor);
    assert_eq!(target.kind(), kind);
    assert_eq!(target.is_interface_reference(), interface_reference);
}

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
        report.text.contains("return this.real();"),
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
    assert_bridge_target(
        bridge,
        "Test",
        "real",
        "()Ljava/lang/String;",
        jarde_java::facts::InvokeKind::Virtual,
        false,
    );
    assert_eq!(bridge.call_bci, Some(1));
    assert_eq!(bridge.cast_bci, Some(4));
    assert!(bridge.pure_forward);
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
        report.text.contains("return this.real();"),
        "{}",
        report.text
    );
    assert_eq!(report.bridges.len(), 1);
    let bridge = &report.bridges[0];
    assert!(bridge.presented());
    assert!(bridge.pure_forward);
    assert_bridge_target(
        bridge,
        "Test",
        "real",
        "()Ljava/lang/String;",
        jarde_java::facts::InvokeKind::Virtual,
        false,
    );
    assert_eq!(bridge.call_bci, Some(1));
    assert_eq!(bridge.cast_bci, None);
    assert_eq!(bridge.erased, None);
}

fn read_bridge_fixture(relative: &str) -> Vec<u8> {
    std::fs::read(format!(
        "{}/../../tests/fixtures/p3-bridge-projection/{relative}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("the permanent bridge fixture is present")
}

fn bridge_fixture_flags(class: &[u8], name: &[u8], descriptor: &[u8]) -> u16 {
    let mut budget = Budget::new(limits());
    let header = class_facts(class, &mut budget).expect("the fixture is a class file");
    header
        .methods
        .iter()
        .find(|method| method.name.raw().0 == name && method.descriptor.raw().0 == descriptor)
        .expect("the physical bridge declaration is present")
        .access_flags
}

#[test]
fn bridge_projection_fixtures_retain_same_pass_targets_or_the_shape_refusal() {
    let descriptor = b"()Ljava/lang/Object;";
    let flags = 0x0001 | 0x0040 | 0x1000;

    let positive = read_bridge_fixture("positive/v8/BridgeProbe.class");
    assert_eq!(
        bridge_fixture_flags(&positive, b"get", descriptor) & flags,
        flags
    );
    let report = present_exact(&positive, b"get", descriptor, 1, vec![Some("self".into())]);
    assert_eq!(report.method, "get()Ljava/lang/Object;");
    let bridge = report
        .bridges
        .first()
        .expect("bridge@1 records the physical bridge");
    assert!(bridge.pure_forward && bridge.presented());
    assert_eq!(bridge.call_bci, Some(1));
    assert_eq!(bridge.cast_bci, None);
    assert_bridge_target(
        bridge,
        "BridgeProbe",
        "get",
        "()Ljava/lang/String;",
        jarde_java::facts::InvokeKind::Virtual,
        false,
    );

    let negative = read_bridge_fixture("negative/v8/FakeBridge.class");
    assert_eq!(
        bridge_fixture_flags(&negative, b"get", descriptor) & flags,
        flags
    );
    let report = present_exact(&negative, b"get", descriptor, 1, vec![Some("self".into())]);
    assert_eq!(report.method, "get()Ljava/lang/Object;");
    let bridge = report
        .bridges
        .first()
        .expect("the effectful physical bridge is reported");
    assert!(!bridge.pure_forward && !bridge.presented());
    assert_eq!(bridge.target, None);
    assert_eq!(
        bridge.refusal.as_ref().map(|refusal| refusal.code),
        Some("jre_bridge_shape")
    );
    assert!(bridge.refusal.as_ref().unwrap().message.contains("slot 0"));

    let orphan = read_bridge_fixture("orphan/v8/OrphanBridge.class");
    assert_eq!(
        bridge_fixture_flags(&orphan, b"get", descriptor) & flags,
        flags
    );
    let report = present_exact(&orphan, b"get", descriptor, 1, vec![Some("self".into())]);
    assert_eq!(report.method, "get()Ljava/lang/Object;");
    let bridge = report
        .bridges
        .first()
        .expect("the isolated physical bridge is reported");
    assert!(bridge.pure_forward && bridge.presented());
    assert_eq!(bridge.call_bci, Some(1));
    assert_eq!(bridge.cast_bci, None);
    assert_bridge_target(
        bridge,
        "OrphanBridge",
        "get",
        "()Ljava/lang/String;",
        jarde_java::facts::InvokeKind::Virtual,
        false,
    );
}

#[test]
fn class_source_bridge_sidecar_is_same_run_and_independent_of_rule_details() {
    let descriptor = b"()Ljava/lang/Object;";
    let flags = 0x0001 | 0x0040 | 0x1000;
    let positive = read_bridge_fixture("positive/v8/BridgeProbe.class");

    let mut essential_budget = Budget::new(limits());
    let essential = recover_class_source_exact(
        &positive,
        b"get",
        descriptor,
        1,
        jarde_java::RecoveryEvidenceRequest::essential(),
        &mut essential_budget,
    );
    assert!(
        essential.report.produced(),
        "{:?}",
        essential.report.outcome
    );
    assert!(essential.report.bridges.is_empty());
    let candidate = essential
        .bridge
        .as_ref()
        .expect("the sidecar is returned without RuleDetails");
    assert_eq!(
        candidate.member.as_ref(),
        Some(&member_identity(&positive, b"get", descriptor))
    );
    assert_eq!(
        candidate.access_flags.map(|value| value & flags),
        Some(flags)
    );
    assert!(candidate.pure_forward && candidate.presented);
    assert_eq!(candidate.call_bci, Some(1));
    assert_eq!(candidate.cast_bci, None);
    assert_eq!(candidate.refusal, None);
    let target = candidate
        .target
        .as_ref()
        .expect("same-run target is retained");
    assert_eq!(target.owner(), "BridgeProbe");
    assert_eq!(target.name(), "get");
    assert_eq!(target.descriptor(), "()Ljava/lang/String;");
    assert_eq!(target.kind(), jarde_java::facts::InvokeKind::Virtual);
    assert!(!target.is_interface_reference());

    let mut detailed_budget = Budget::new(limits());
    let detailed = recover_class_source_exact(
        &positive,
        b"get",
        descriptor,
        1,
        jarde_java::RecoveryEvidenceRequest::all(),
        &mut detailed_budget,
    );
    assert_eq!(detailed.bridge.as_ref(), Some(candidate));
    assert_eq!(detailed.report.bridges.len(), 1);

    let negative = read_bridge_fixture("negative/v8/FakeBridge.class");
    let mut negative_budget = Budget::new(limits());
    let rejected = recover_class_source_exact(
        &negative,
        b"get",
        descriptor,
        1,
        jarde_java::RecoveryEvidenceRequest::essential(),
        &mut negative_budget,
    );
    let candidate = rejected
        .bridge
        .expect("the refusal verdict is handed to the class source");
    assert!(!candidate.pure_forward && !candidate.presented);
    assert_eq!(candidate.target, None);
    assert_eq!(
        candidate.refusal.as_ref().map(|item| item.code),
        Some("jre_bridge_shape")
    );

    let orphan = read_bridge_fixture("orphan/v8/OrphanBridge.class");
    let mut orphan_budget = Budget::new(limits());
    let isolated = recover_class_source_exact(
        &orphan,
        b"get",
        descriptor,
        1,
        jarde_java::RecoveryEvidenceRequest::essential(),
        &mut orphan_budget,
    );
    let candidate = isolated
        .bridge
        .expect("the pure isolated bridge is retained");
    assert!(candidate.pure_forward && candidate.presented);
    assert_eq!(
        candidate.target.as_ref().map(|item| item.owner()),
        Some("OrphanBridge")
    );

    let token = jarde_reader::budget::CancellationToken::new();
    token.cancel();
    let mut cancelled_budget = Budget::with_cancellation_token(limits(), token);
    let cancelled = recover_class_source_exact(
        &positive,
        b"get",
        descriptor,
        1,
        jarde_java::RecoveryEvidenceRequest::essential(),
        &mut cancelled_budget,
    );
    assert!(
        !cancelled.report.produced(),
        "{:?}",
        cancelled.report.outcome
    );
    assert!(cancelled.bridge.is_none());
}

#[test]
fn a_member_the_class_does_not_declare_a_bridge_keeps_its_cast_quoted() {
    // The same body as the positive case, and the class declares the member **without** the bridge
    // flag: the rule reads the declaration, not the shape, so the cast is not erased by bridge@1;
    // ordinary cast recovery still writes the runtime check.
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
    assert!(bridge.pure_forward);
    assert_eq!(bridge.call_bci, Some(1));
    assert_eq!(bridge.cast_bci, None);
    assert_bridge_target(
        bridge,
        "Test",
        "real",
        "()Ljava/lang/String;",
        jarde_java::facts::InvokeKind::Virtual,
        false,
    );
    let refusal = bridge.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_bridge_not_declared");
    assert!(
        refusal.message.contains("bridge flag"),
        "{}",
        refusal.message
    );
    assert!(
        report
            .text
            .contains("return (java.lang.String) this.real();"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(report.representation, Representation::Java);
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
    assert!(!bridge.pure_forward);
    assert_eq!(bridge.target, None);
    assert_eq!(bridge.call_bci, None);
    assert_eq!(bridge.cast_bci, None);
    let refusal = bridge.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_bridge_shape");
    // The body is still presented the ordinary way: refusing the *bridge shape* is not a refusal of
    // the member, and the store it performs keeps its statement.
    assert_eq!(report.representation, Representation::Java);
    assert!(
        report.text.contains("this.real();") && report.text.contains("return local1;"),
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
    assert!(!bridge.pure_forward);
    assert_eq!(bridge.target, None);
    assert_eq!(bridge.call_bci, None);
    assert_eq!(bridge.cast_bci, None);
    let refusal = bridge.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_bridge_cast_not_erasure");
    assert!(refusal.message.contains("can fail"), "{}", refusal.message);
    // The bridge verdict remains a refusal, while ordinary cast recovery keeps the parameter check
    // in the call argument and writes the body as Java.
    assert!(
        report
            .text
            .contains("return this.real2((java.lang.String) value);"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(report.representation, Representation::Java);
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
    assert!(
        report
            .text
            .contains("return (java.lang.String) arg0.real();"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(report.representation, Representation::Java);
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
    let second_read = member_ref(&mut pool, test, "access$400", "(LTest;)I");
    let literal_name = pool.utf8("!");
    let literal = pool.string(literal_name);
    let access100 = pool.utf8("access$100");
    let access102 = pool.utf8("access$102");
    let access200 = pool.utf8("access$200");
    let access300 = pool.utf8("access$300");
    let access400 = pool.utf8("access$400");
    let method_name = pool.utf8("method");
    let write_name = pool.utf8("write");
    let other_200_name = pool.utf8("other_200");
    let other_300_name = pool.utf8("other_300");
    let both_fields_name = pool.utf8("both_fields");
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
    let both_fields_body = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0xb8)
        .index(read) // 1: invokestatic access$100
        .op(0x2a) // 4: aload_0
        .op(0xb8)
        .index(second_read) // 5: invokestatic access$400
        .op(0x60) // 8: iadd
        .op(0xac) // 9: ireturn
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
                // A second synthetic read of the same field: its `getfield` sits at BCI 1, exactly
                // where `access$100`'s does, in a member body of its own (P3 3.2).
                flags: 0x1008,
                name: access400,
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
                name: both_fields_name,
                descriptor: no_arguments,
                max_stack: 2,
                max_locals: 1,
                code: both_fields_body,
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

/// A verifier-valid narrow write accessor: the JVM descriptor admits the int-shaped local in the
/// value slot, while the accessor's field proof states the B/C/S position precisely.
fn narrow_write_accessor_class() -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let field_name = pool.utf8("f");
    let field_descriptor = pool.utf8("B");
    let field = field_ref(&mut pool, test, "f", "B");
    let accessor_name = pool.utf8("access$102");
    let accessor_descriptor = pool.utf8("(LTest;B)V");
    let accessor = member_ref(&mut pool, test, "access$102", "(LTest;B)V");
    let write_name = pool.utf8("write");
    let write_descriptor = pool.utf8("(I)V");
    let accessor_body = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0x1b) // 1: iload_1
        .op(0xb5)
        .index(field) // 2: putfield Test.f:B
        .op(0xb1) // 5: return
        .done();
    let write_body = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0x1b) // 1: iload_1
        .op(0xb8)
        .index(accessor) // 2: invokestatic access$102(LTest;B)V
        .op(0xb1) // 5: return
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
                name: accessor_name,
                descriptor: accessor_descriptor,
                max_stack: 2,
                max_locals: 2,
                code: accessor_body,
            },
            MemberDef {
                flags: 0x0001,
                name: write_name,
                descriptor: write_descriptor,
                max_stack: 2,
                max_locals: 2,
                code: write_body,
            },
        ],
    )
}

/// A verifier-valid `Z` write accessor whose argument remains an int in both descriptors.
fn boolean_write_accessor_class(extra_write: bool) -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let field_name = pool.utf8("f");
    let field = field_ref(&mut pool, test, "f", "Z");
    let extra_field = if extra_write {
        let name = pool.utf8("g");
        let field = field_ref(&mut pool, test, "g", "Z");
        Some((name, field))
    } else {
        None
    };
    let accessor_name = pool.utf8("access$102");
    let accessor_descriptor = pool.utf8("(LTest;Z)V");
    let accessor = member_ref(&mut pool, test, "access$102", "(LTest;Z)V");
    let write_name = pool.utf8("write");
    let write_descriptor = pool.utf8("(I)V");
    let mut accessor_body = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0x1b) // 1: iload_1
        .op(0xb5)
        .index(field); // 2: putfield Test.f:Z
    if let Some((_, extra_field)) = extra_field {
        accessor_body = accessor_body
            .op(0x2a) // 5: aload_0
            .op(0x04) // 6: iconst_1
            .op(0xb5)
            .index(extra_field); // 7: a second write invalidates the accessor shape
    }
    let accessor_body = accessor_body.op(0xb1).done();
    let write_body = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0x1b) // 1: iload_1
        .op(0xb8)
        .index(accessor) // 2: invokestatic access$102(LTest;Z)V; caller stack value is int-shaped
        .op(0xb1) // 5: return
        .done();
    let mut fields = vec![FieldDef {
        flags: 0x0002,
        name: field_name,
        descriptor: pool.utf8("Z"),
    }];
    if let Some((name, _)) = extra_field {
        fields.push(FieldDef {
            flags: 0x0002,
            name,
            descriptor: pool.utf8("Z"),
        });
    }
    class_bytes(
        &pool,
        test,
        object,
        &fields,
        &[
            MemberDef {
                flags: 0x1008,
                name: accessor_name,
                descriptor: accessor_descriptor,
                max_stack: 2,
                max_locals: 2,
                code: accessor_body,
            },
            MemberDef {
                flags: 0x0001,
                name: write_name,
                descriptor: write_descriptor,
                max_stack: 2,
                max_locals: 2,
                code: write_body,
            },
        ],
    )
}

#[test]
fn a_verified_narrow_write_accessor_keeps_the_field_cast() {
    let class = narrow_write_accessor_class();
    let report = present(
        &class,
        b"write",
        b"(I)V",
        2,
        vec![Some("self".into()), Some("value".into())],
    );
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert!(
        report.text.contains("this.f = (byte) value;"),
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("% 2"),
        "a B field keeps its own conversion"
    );
    let accessor = report
        .accessors
        .iter()
        .find(|accessor| accessor.name == "access$102")
        .expect("the write accessor is recorded");
    assert!(accessor.presented());
    assert_eq!(accessor.shape, Some(AccessorShape::FieldWrite));
    assert_eq!(
        accessor
            .field
            .as_ref()
            .map(|field| field.descriptor.as_str()),
        Some("B")
    );
}

#[test]
fn a_verified_boolean_write_accessor_keeps_call_and_callee_put_origins() {
    let class = boolean_write_accessor_class(false);
    let report = present(
        &class,
        b"write",
        b"(I)V",
        2,
        vec![Some("self".into()), Some("value".into())],
    );
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert!(
        report.text.contains("this.f = value % 2 != 0;"),
        "{}",
        report.text
    );
    assert_eq!(report.accessors.len(), 1);
    let accessor = &report.accessors[0];
    assert!(accessor.presented());
    assert_eq!(accessor.shape, Some(AccessorShape::FieldWrite));
    assert_eq!(accessor.call_site, 2);
    assert_eq!(
        accessor
            .field
            .as_ref()
            .map(|field| field.descriptor.as_str()),
        Some("Z")
    );

    let assignment = report
        .source_map
        .segments()
        .iter()
        .find(|segment| {
            segment
                .text(&report.text)
                .contains("this.f = value % 2 != 0;")
        })
        .expect("the assignment is anchored to its call site");
    assert_eq!(assignment.origin().primary().bci(), 2);
    assert_eq!(
        assignment.origin().primary().method().unwrap().name.0,
        b"write"
    );
    assert_eq!(assignment.origin().derived().len(), 1);
    assert_eq!(assignment.origin().derived()[0].bci(), 2);
    assert_eq!(
        assignment.origin().derived()[0].method().unwrap().name.0,
        b"access$102"
    );

    let callee = present(
        &class,
        b"access$102",
        b"(LTest;Z)V",
        2,
        vec![Some("receiver".into()), Some("value".into())],
    );
    assert!(callee.produced(), "{:?}", callee.stop());
    assert!(
        callee.text.contains("receiver.f = value;"),
        "{}",
        callee.text
    );
    assert!(
        !callee.text.contains("% 2"),
        "the callee's Z parameter is already boolean"
    );
    assert!(
        !callee.source_map.direct_of_bci(2).is_empty(),
        "{}",
        callee.text
    );
}

#[test]
fn an_unverified_boolean_write_accessor_stays_a_call() {
    let class = boolean_write_accessor_class(true);
    let report = present(
        &class,
        b"write",
        b"(I)V",
        2,
        vec![Some("self".into()), Some("value".into())],
    );
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.accessors.len(), 1);
    let accessor = &report.accessors[0];
    assert!(!accessor.presented());
    assert_eq!(accessor.shape, None);
    assert_eq!(accessor.refusal.as_ref().unwrap().code, "jre_accessor_body");
    assert!(report.text.contains("@bytecode 2"), "{}", report.text);
    assert!(
        report.text.contains("no proven conversion to `boolean`"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("% 2"), "{}", report.text);
}

#[test]
fn a_synthetic_accessors_call_site_is_presented_as_the_field_access_it_forwards() {
    let class = accessor_class();
    let report = present(&class, b"method", b"()I", 1, vec![Some("self".into())]);
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert_eq!(report.quality, Quality::Structured);
    assert!(report.text.contains("int local1 = 0;"), "{}", report.text);
    assert!(report.text.contains("return this.f;"), "{}", report.text);

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
        .find(|segment| segment.text(&report.text) == "this.f")
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
    assert!(report.text.contains("this.f = value;"), "{}", report.text);
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
    assert!(report.text.contains("access$200(this)"), "{}", report.text);
}

#[test]
fn a_member_the_class_declares_without_a_body_is_refused_as_itself() {
    // The one difference a member table with an *absent* body has to keep: "the class declares no
    // such member" and "the class declares this member and there is no body to read" are two
    // different statements, and the second one is the one this table makes.
    let class = accessor_class();
    let identity = member_identity(&class, b"access$100", b"(LTest;)I");
    let members = ClassMembers::new(
        "Test",
        vec![MemberBody::without_body("Test", identity, 0x1108)],
    );
    let payload = analyze(&class, b"method", b"()I");
    let facts = facts_of(&class, b"method", 1, vec![Some("self".into())]);
    let mut budget = Budget::new(limits());
    let report = recover_body(&payload, &facts, Some(&members), &mut budget);

    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.accessors.len(), 1);
    let accessor = &report.accessors[0];
    assert!(!accessor.presented());
    assert_eq!(accessor.access_flags, Some(0x1108));
    let refusal = accessor.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_accessor_body");
    assert!(
        refusal.message.contains("no body") && refusal.message.contains("access$100"),
        "the absence of the body is stated as itself: {}",
        refusal.message
    );
    assert!(
        !refusal.message.contains("declares no"),
        "and never as a member the class does not declare: {}",
        refusal.message
    );
    assert!(report.text.contains("access$100(this)"), "{}", report.text);
}

#[test]
fn the_anchor_of_a_presented_field_access_states_the_member_its_bci_is_in() {
    // Two call sites, two different callees whose field access sits at **one** BCI (1) each: the
    // bytecode index alone cannot tell the two derived anchors apart, and the member they name is
    // what does — and that member is a member of the same class-file definition the presented body
    // was read from.
    let class = accessor_class();
    let report = present(&class, b"both_fields", b"()I", 1, vec![Some("self".into())]);
    assert!(report.produced(), "{:?}", report.stop());
    assert!(
        report.text.contains("return this.f + this.f;"),
        "{}",
        report.text
    );
    assert_eq!(report.accessors.len(), 2);
    assert!(
        report.accessors.iter().all(|accessor| accessor.presented()),
        "{:?}",
        report.accessors
    );

    let anchors: Vec<jarde_java::Origin> = [1_u32, 5]
        .into_iter()
        .map(|call_site| {
            let node = report
                .source_map
                .direct_of_bci(call_site)
                .into_iter()
                .next()
                .unwrap_or_else(|| panic!("the call site at BCI {call_site} anchors a node"));
            assert_eq!(node.text(&report.text), "this.f", "{}", report.text);
            let derived = node.origin().derived();
            assert_eq!(derived.len(), 1);
            assert_eq!(
                derived[0].bci(),
                1,
                "the field access inside the callee's body"
            );
            derived[0].clone()
        })
        .collect();
    assert_ne!(anchors[0], anchors[1], "one BCI, two member bodies");
    assert_eq!(
        anchors[0].bci(),
        anchors[1].bci(),
        "and the same bytecode index in both"
    );
    assert_eq!(
        anchors[0].method().map(|method| method.name.0.clone()),
        Some(b"access$100".to_vec())
    );
    assert_eq!(
        anchors[1].method().map(|method| method.name.0.clone()),
        Some(b"access$400".to_vec())
    );
    assert_eq!(
        anchors[0].method().map(|method| method.owner.clone()),
        anchors[1].method().map(|method| method.owner.clone()),
        "both members are declared in one definition, which is what binds their constant pools"
    );
    // And the member of the body being presented is a fact of its own anchors.
    let own = report.source_map.direct_of_bci(1);
    assert_eq!(
        own[0]
            .origin()
            .primary()
            .method()
            .map(|method| method.name.0.clone()),
        Some(b"both_fields".to_vec()),
        "the run's own declaration names the presented member"
    );
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
    assert!(report.text.contains("access$300(this)"), "{}", report.text);
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
    assert!(report.text.contains("access$100(this)"), "{}", report.text);
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
    // **Changed by the concatenation-conversion fix (was: `return self.f + "!";` — the receiver was
    // spelled by the debug name of the fixture's slot 0 then, and is `this` since it is spelled by
    // its identity).** `this.f` is an `int`, so the chain's first `+` is not a string concatenation:
    // the empty string starts it, and the field read is converted where `append(int)` converted it.
    // The value is the same (`"" + this.f` is `String.valueOf(this.f)`), and the accessor is still
    // presented as the field read.
    assert!(
        report.text.contains("return \"\" + this.f + \"!\";"),
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

/// The same report with the one observed field a determinism comparison must exclude neutralized.
///
/// `recovery-validation`'s `Determinism excludes only observed elapsed time` scenario: a controlled
/// repeat of the same input, profile and limits compares equal except for the observed
/// `elapsed_millis`, and a 0/1 ms reading of the clock is not a difference in the presentation.
/// Nothing else is normalized — every result, origin, diagnostic, rule, order and counted field is
/// still compared exactly as it stands.
fn without_observed_elapsed(report: &jarde_java::RecoveryReport) -> jarde_java::RecoveryReport {
    let mut report = report.clone();
    // Every variant of the execution plane carries the usage of the request, so this is the field
    // wherever it appears: a stop's report carries one too.
    let usage = match &mut report.execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage,
    };
    usage.elapsed_millis = 0;
    report
}

#[test]
fn a_body_without_debug_metadata_is_still_presented_with_deterministic_names() {
    let class = accessor_class();
    let first = present(&class, b"combined", b"()Ljava/lang/String;", 1, Vec::new());
    let second = present(&class, b"combined", b"()Ljava/lang/String;", 1, Vec::new());
    assert_eq!(
        without_observed_elapsed(&first),
        without_observed_elapsed(&second),
        "the presentation is a function of the evidence, up to the one observed field the \
         determinism requirement excludes"
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
    /// An instance a `new` allocated: the class it is an instance of, and the text it has
    /// accumulated (a chain that appends into one keeps it here). The class is part of the cell
    /// because a construction's class and its argument values are what P3 2.3's comparison checks.
    Instance {
        class: String,
        text: String,
    },
}

impl Cell {
    /// What the cell contributes to a string being built.
    fn written(&self) -> String {
        match self {
            Self::Int(value) => value.to_string(),
            Self::Text(text) | Self::Instance { text, .. } => text.clone(),
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
    /// Every effect the side performs, in the order it performs it: a construction with the class and
    /// the argument values it is given, an ordinary call, a field write with the value written, and
    /// the constructor prologue. P3 2.3's patterns are about **order** — where a field initializer
    /// sits relative to the constructor call, which write comes first, how often a call happens — so
    /// the model keeps an ordered log of them and [`compare_effects`] requires the two logs to be
    /// equal. It is a separate comparison from [`compare`] because a fixture whose chain the run
    /// presents as one `+` expression has no construction in its text at all, and that is a shape
    /// [`compare`] already checks.
    effects: Vec<String>,
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

    /// The member's own name, without its owner: what a field write is compared by.
    fn field_name(&self, index: u16) -> String {
        let index = usize::from(index);
        let name_and_type = usize::from(self.second[index]);
        self.text(self.first[name_and_type]).to_string()
    }

    /// How many parameter values the descriptor of one member reference takes.
    ///
    /// The model reads the descriptor's own shape — `L…;` and one `[` per array dimension are one
    /// parameter each — so the argument count it checks is the class file's, not a fixture's comment.
    fn argument_count(&self, index: u16) -> usize {
        let index = usize::from(index);
        let name_and_type = usize::from(self.second[index]);
        let descriptor = self.text(self.second[name_and_type]);
        let mut characters = descriptor
            .strip_prefix('(')
            .expect("a method descriptor opens its parameters")
            .chars()
            .peekable();
        let mut count = 0usize;
        while let Some(character) = characters.next() {
            match character {
                ')' => break,
                '[' => {}
                'L' => {
                    for inner in characters.by_ref() {
                        if inner == ';' {
                            break;
                        }
                    }
                    count += 1;
                }
                _ => count += 1,
            }
        }
        count
    }
}

/// Runs one fixture body: the model's own decoder and stack machine.
///
/// `parameters` are the entry locals of the body, slot 0 first: a fixture that reads a parameter
/// states it here, and the model reads nothing about names from anywhere else.
fn run_bytecode(code: &[u8], pool: &OraclePool, parameters: &[&str]) -> Observed {
    let mut stack: Vec<Cell> = Vec::new();
    let mut locals: Vec<Cell> = vec![Cell::Text(String::new()); 4];
    for (slot, name) in parameters.iter().enumerate() {
        locals[slot] = Cell::Text((*name).to_string());
    }
    let mut observed = Observed {
        calls: Vec::new(),
        returned: None,
        accessor_calls: 0,
        accessor_reads: Vec::new(),
        effects: Vec::new(),
    };
    let mut at = 0;
    while at < code.len() {
        let opcode = code[at];
        match opcode {
            0xbb => {
                let index = u16::from_be_bytes([code[at + 1], code[at + 2]]);
                stack.push(Cell::Instance {
                    class: pool.class_name(index).to_string(),
                    text: String::new(),
                });
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
            // A constructor completes the instance it was given, or starts the body that declares it:
            // the model says which, with the argument values the constructor was handed, so that the
            // text side can be required to spell the same thing in the same position.
            0xb7 => {
                let index = u16::from_be_bytes([code[at + 1], code[at + 2]]);
                let member = pool.member(index);
                let mut arguments: Vec<String> = Vec::new();
                for _ in 0..pool.argument_count(index) {
                    arguments.push(stack.pop().expect("an argument").written());
                }
                arguments.reverse();
                let receiver = stack.pop().expect("a constructor reads its receiver");
                match receiver {
                    // The class is written the way Java source spells a type: the text side reads the
                    // artifact, which spells `p.Outer$1`, and the model's own key has to be the same
                    // name or the comparison would be about the pool's spelling rather than about the
                    // class.
                    Cell::Instance { class, .. } => observed.effects.push(format!(
                        "new {}({})",
                        class.replace('/', "."),
                        arguments.join(", ")
                    )),
                    Cell::Text(_) => observed.effects.push("prologue".to_string()),
                    other => panic!("the model does not construct {other:?} from `{member}`"),
                }
                at += 3;
            }
            // A field write: the member's own name and the value written, in the order the body does
            // it. The owner is not part of the model's key: the text names the receiver where the
            // bytecode names the owner, and which member that is is what the fixture's own
            // assertions check.
            0xb3 => {
                let index = u16::from_be_bytes([code[at + 1], code[at + 2]]);
                let value = stack.pop().expect("a field write reads a value");
                observed.effects.push(format!(
                    "write {} = {}",
                    pool.field_name(index),
                    value.written()
                ));
                at += 3;
            }
            0xb5 => {
                let index = u16::from_be_bytes([code[at + 1], code[at + 2]]);
                let value = stack.pop().expect("a field write reads a value");
                stack.pop().expect("an instance write reads its receiver");
                observed.effects.push(format!(
                    "write {} = {}",
                    pool.field_name(index),
                    value.written()
                ));
                at += 3;
            }
            0x12 => {
                let index = u16::from(code[at + 1]);
                stack.push(pool.constant(index));
                at += 2;
            }
            0x02..=0x08 => {
                stack.push(Cell::Int(i64::from(opcode) - 3));
                at += 1;
            }
            0x10 => {
                stack.push(Cell::Int(i64::from(code[at + 1] as i8)));
                at += 2;
            }
            0x1a | 0x2a => {
                stack.push(locals[0].clone());
                at += 1;
            }
            0x1b | 0x2b => {
                stack.push(locals[1].clone());
                at += 1;
            }
            0x4c..=0x4e => {
                let slot = usize::from(opcode - 0x4b);
                locals[slot] = stack.pop().expect("a store reads a value");
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
                            Cell::Instance { text, .. } => text.push_str(&value.written()),
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
                        observed.effects.push(format!("call {name}"));
                        stack.push(Cell::Text(called(&member).to_string()));
                    }
                }
                at += 3;
            }
            // The cast keeps the value it is given.
            0xc0 => at += 3,
            // A bare `return` leaves no value; `ireturn`/`areturn` leave the one on top. Which of the
            // two a text's `return` is has to agree, which is why the model keeps the difference
            // instead of collapsing it.
            0xb1 => at += 1,
            0xb0 | 0xac => {
                let value = stack.pop().expect("a return needs a value");
                observed.returned = Some(match &value {
                    Cell::Instance { class, .. } => format!("instance:{}", class.replace('/', ".")),
                    other => other.written(),
                });
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
        effects: Vec::new(),
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
/// One `new Class(arguments)` term, split into the class and the arguments as written.
fn construction(term: &str) -> Option<(String, String)> {
    let rest = term.strip_prefix("new ")?;
    let (class, arguments) = rest
        .split_once('(')
        .expect("a construction spells an argument list");
    let arguments = arguments
        .strip_suffix(')')
        .expect("a construction's argument list closes");
    Some((class.to_string(), arguments.to_string()))
}

/// Reads one produced artifact's **statements**: the constructions, the constructor call, the field
/// writes and the calls, each in the order the artifact writes them.
///
/// This is the text side of P3 2.3's comparison, and it is a different reader from [`run_text`] on
/// purpose: a shape whose artifact is a list of statements has no single `return` expression to
/// evaluate, and what those shapes raise is not a value but an order — which is what
/// [`compare_effects`] checks.
fn run_text_effects(artifact: &str) -> Observed {
    let mut observed = Observed {
        calls: Vec::new(),
        returned: None,
        accessor_calls: 0,
        accessor_reads: Vec::new(),
        effects: Vec::new(),
    };
    for line in artifact.lines() {
        let statement = line.trim();
        if statement.is_empty()
            || statement.starts_with("//")
            || statement == "{"
            || statement == "}"
        {
            continue;
        }
        let statement = statement.trim_end_matches(';');
        if statement == "super()" || statement == "this()" {
            observed.effects.push("prologue".to_string());
            continue;
        }
        if statement == "return" {
            continue;
        }
        if let Some(value) = statement.strip_prefix("return ") {
            if let Some((class, arguments)) = construction(value) {
                observed.effects.push(format!("new {class}({arguments})"));
                observed.returned = Some(format!("instance:{class}"));
                continue;
            }
            panic!("the model does not read the returned term `{value}`");
        }
        if let Some((target, value)) = statement.split_once(" = ") {
            if let Some((class, arguments)) = construction(value) {
                observed.effects.push(format!("new {class}({arguments})"));
                continue;
            }
            let member = target.rsplit('.').next().expect("a member name");
            observed.effects.push(format!("write {member} = {value}"));
            continue;
        }
        panic!("the model does not read the statement `{statement}`");
    }
    observed
}

/// The comparison for the shapes whose meaning is an order: the same calls, the same value, and the
/// same effect sequence — every construction with the class and the argument values it was given,
/// every field write with the value it wrote, and the constructor prologue, in the same positions.
fn compare_effects(from_bytes: &Observed, from_text: &Observed) -> Result<(), String> {
    compare(from_bytes, from_text)?;
    if from_bytes.effects != from_text.effects {
        return Err(format!(
            "the effects differ: the bytecode does {:?} and the text {:?}",
            from_bytes.effects, from_text.effects
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
    let from_bytes = run_bytecode(&code, &pool, &[]);
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
    let from_bytes = run_bytecode(&code, &pool, &["self"]);
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
    let from_bytes = run_bytecode(&code, &pool, &[]);
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
    let from_bytes = run_bytecode(&code, &pool, &[]);
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

// ---------------------------------------------------------------------------
// P3 2.3 章§0：一个被拒绝的读者会不会吞掉产生它的那次调用（探针）
// ---------------------------------------------------------------------------

/// `Test.value()Ljava/lang/Object;` called, cast by a `checkcast` **no rule claims**, and stored.
///
/// The call's value reaches exactly one instruction — that cast — and the cast is quoted as
/// bytecode. Whether the invocation survives in the artifact at all is the question the probe asks.
fn refused_cast_consumer_class() -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let string_name = pool.utf8("java/lang/String");
    let string = pool.class(string_name);
    let value = member_ref(&mut pool, test, "value", "()Ljava/lang/Object;");
    let method = pool.utf8("method");
    let method_descriptor = pool.utf8("()V");
    let value_name = pool.utf8("value");
    let value_descriptor = pool.utf8("()Ljava/lang/Object;");
    let code = Code::default()
        .op(0xb8)
        .index(value) // 0: invokestatic Test.value()Ljava/lang/Object;
        .op(0xc0)
        .index(string) // 3: checkcast java/lang/String
        .op(0x4c) // 6: astore_1
        .op(0xb1) // 7: return
        .done();
    let value_code = Code::default().op(0x01).op(0xb0).done();
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
                max_stack: 1,
                max_locals: 2,
                code,
            },
            MemberDef {
                flags: 0x0009,
                name: value_name,
                descriptor: value_descriptor,
                max_stack: 1,
                max_locals: 0,
                code: value_code,
            },
        ],
    )
}

/// A call whose **argument** is the value of a cast no rule claims, stored in a local: the store is
/// a reader of the call's value, so the call writes no statement of its own — and the store cannot
/// write the call, because the argument it would read has no text either.
fn refused_argument_class() -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let string_name = pool.utf8("java/lang/String");
    let string = pool.class(string_name);
    let take = member_ref(&mut pool, test, "take", "(Ljava/lang/String;)I");
    let method = pool.utf8("method");
    let method_descriptor = pool.utf8("(Ljava/lang/Object;)V");
    let take_name = pool.utf8("take");
    let take_descriptor = pool.utf8("(Ljava/lang/String;)I");
    let code = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0xc0)
        .index(string) // 1: checkcast java/lang/String
        .op(0xb8)
        .index(take) // 4: invokestatic Test.take(Ljava/lang/String;)I
        .op(0x3c) // 7: istore_1
        .op(0xb1) // 8: return
        .done();
    let take_code = Code::default().op(0x03).op(0xac).done();
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
                max_stack: 1,
                max_locals: 2,
                code,
            },
            MemberDef {
                flags: 0x0009,
                name: take_name,
                descriptor: take_descriptor,
                max_stack: 1,
                max_locals: 1,
                code: take_code,
            },
        ],
    )
}

/// Two descriptor-aware invocation boundaries share this tiny class: a static caller passes an
/// `Object` to an interface parameter, while an instance caller passes its declared receiver to
/// an `Object` parameter. The `take` body is deliberately inert; only the call-site descriptor is
/// under test.
fn reference_invocation_class(instance: bool) -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let runnable_name = pool.utf8("java/lang/Runnable");
    let _runnable = pool.class(runnable_name);
    let take_descriptor = if instance {
        "(Ljava/lang/Object;)I"
    } else {
        "(Ljava/lang/Runnable;)I"
    };
    let take = member_ref(&mut pool, test, "take", take_descriptor);
    let caller_name = pool.utf8("caller");
    let caller_descriptor = if instance {
        pool.utf8("()I")
    } else {
        pool.utf8("(Ljava/lang/Object;)I")
    };
    let take_name = pool.utf8("take");
    let take_descriptor = pool.utf8(take_descriptor);
    let caller = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0xb8)
        .index(take) // 1: invokestatic Test.take
        .op(0xac) // 4: ireturn
        .done();
    let take_body = Code::default()
        .op(0x10)
        .byte(7) // 0: bipush 7
        .op(0xac) // 2: ireturn
        .done();
    class_bytes(
        &pool,
        test,
        object,
        &[],
        &[
            MemberDef {
                flags: if instance { 0x0001 } else { 0x0009 },
                name: caller_name,
                descriptor: caller_descriptor,
                max_stack: 1,
                max_locals: 1,
                code: caller,
            },
            MemberDef {
                flags: 0x0009,
                name: take_name,
                descriptor: take_descriptor,
                max_stack: 1,
                max_locals: 1,
                code: take_body,
            },
        ],
    )
}

/// A refused two-argument call reads one deferred `RuntimeException` value twice through `dup`.
/// The verifier accepts the concrete-to-supertype boundary, while this source recovery path keeps
/// the unknown reference relation conservative. Both call operands therefore reach the same SSA
/// value during quote collection.
fn shared_deferred_argument_class() -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let value = member_ref(&mut pool, test, "value", "()Ljava/lang/RuntimeException;");
    let take = member_ref(
        &mut pool,
        test,
        "take",
        "(Ljava/lang/Exception;Ljava/lang/Exception;)I",
    );
    let caller_name = pool.utf8("caller");
    let caller_descriptor = pool.utf8("()I");
    let value_name = pool.utf8("value");
    let value_descriptor = pool.utf8("()Ljava/lang/RuntimeException;");
    let take_name = pool.utf8("take");
    let take_descriptor = pool.utf8("(Ljava/lang/Exception;Ljava/lang/Exception;)I");
    let caller = Code::default()
        .op(0xb8)
        .index(value) // 0: invokestatic Test.value()Ljava/lang/RuntimeException;
        .op(0x59) // 3: dup; both call arguments are one SSA value
        .op(0xb8)
        .index(take) // 4: invokestatic Test.take(Exception,Exception)
        .op(0xac) // 7: ireturn
        .done();
    let value_body = Code::default()
        .op(0x01) // 0: aconst_null
        .op(0xb0) // 1: areturn
        .done();
    let take_body = Code::default()
        .op(0x10)
        .byte(7) // 0: bipush 7
        .op(0xac) // 2: ireturn
        .done();
    class_bytes(
        &pool,
        test,
        object,
        &[],
        &[
            MemberDef {
                flags: 0x0009,
                name: caller_name,
                descriptor: caller_descriptor,
                max_stack: 2,
                max_locals: 0,
                code: caller,
            },
            MemberDef {
                flags: 0x0009,
                name: value_name,
                descriptor: value_descriptor,
                max_stack: 1,
                max_locals: 0,
                code: value_body,
            },
            MemberDef {
                flags: 0x0009,
                name: take_name,
                descriptor: take_descriptor,
                max_stack: 2,
                max_locals: 2,
                code: take_body,
            },
        ],
    )
}

#[test]
fn an_invocation_is_written_once_with_its_recovered_cast() {
    // The invocation's value reaches a local through an ordinary checkcast. Both instructions are
    // now recovered as one declaration, so the producer still appears exactly once and the cast's
    // runtime check remains in the expression.
    let class = refused_cast_consumer_class();
    let report = present(&class, b"method", b"()V", 0, Vec::new());
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(
        report.text.matches("value()").count(),
        1,
        "the invocation is written exactly once:\n{}",
        report.text
    );
    assert!(
        report.text.contains("local1 = (java.lang.String) value();"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(
        unaccounted_instructions(&report, &[0, 3, 6, 7]),
        Vec::<u32>::new(),
        "{}",
        report.text
    );
    assert!(
        !report.text_of_bci(0).is_empty(),
        "the invocation's own BCI reaches a segment: {:?}",
        report.text_of_bci(0)
    );
    assert_eq!(report.representation, Representation::Java);
}

#[test]
fn an_invocation_argument_keeps_its_recovered_cast() {
    // An ordinary cast used as a call argument remains in the local assignment and the invocation
    // is written once. The unsupported-reader quote control lives in p3_reference_cast's nestedCall.
    let class = refused_argument_class();
    let report = present(&class, b"method", b"(Ljava/lang/Object;)V", 0, Vec::new());
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.text.matches("take(").count(), 1, "{}", report.text);
    assert!(
        report
            .text
            .contains("local1 = take((java.lang.String) local0);"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(
        unaccounted_instructions(&report, &[1, 4, 7, 8]),
        Vec::<u32>::new(),
        "{}",
        report.text
    );
}

#[test]
fn a_static_object_to_interface_call_is_quoted_without_an_invented_cast() {
    let class = reference_invocation_class(false);
    let report = present(
        &class,
        b"caller",
        b"(Ljava/lang/Object;)I",
        1,
        vec![Some("value".into())],
    );
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Mixed);
    assert_eq!(report.quality, Quality::Fallback);
    assert_eq!(
        report
            .evidence
            .state(jarde_java::RecoveryEvidenceKind::SourceMap),
        jarde_java::EvidenceState::Complete
    );
    let quoted = quoted_bcis(&report);
    assert!(
        quoted.contains(&1),
        "the invocation BCI is quoted: {quoted:?}"
    );
    assert!(quoted.contains(&4), "the return BCI is quoted: {quoted:?}");
    assert!(
        !report.text.contains("take((java.lang.Runnable)"),
        "the refusal cannot invent an interface cast:\n{}",
        report.text
    );

    let caller = member_identity(&class, b"caller", b"(Ljava/lang/Object;)I");
    let quote = report
        .source_map
        .segments()
        .iter()
        .find(|segment| segment.text(&report.text).contains("@bytecode"))
        .expect("the refused call is mapped to its quoted source");
    assert_eq!(quote.origin().primary().bci(), 4);
    assert_eq!(quote.origin().primary().method(), Some(&caller));
    assert!(
        quote
            .origin()
            .derived()
            .iter()
            .any(|origin| origin.bci() == 1 && origin.method() == Some(&caller))
    );
}

#[test]
fn a_shared_deferred_argument_dag_is_quoted_once_and_budgeted() {
    let class = shared_deferred_argument_class();
    let (report, full_budget) =
        present_with_ir_limit(&class, b"caller", b"()I", 0, vec![], 1 << 20);
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Mixed);
    assert_eq!(report.quality, Quality::Fallback);
    let quoted = quoted_bcis(&report);
    assert_eq!(
        quoted,
        [3, 7, 4],
        "the shared quote keeps its first-visit order"
    );
    assert_eq!(quoted.iter().filter(|bci| **bci == 4).count(), 1);
    assert!(
        !report.text.contains("take((java.lang.Exception)"),
        "{}",
        report.text
    );

    let used = full_budget
        .usage()
        .counted_usage(jarde_reader::budget::CountedBudgetDimension::IrItems);
    assert!(used > 1, "the shared quote walk charged IR work");
    let low_limit = 1;
    let (limited, limited_budget) =
        present_with_ir_limit(&class, b"caller", b"()I", 0, vec![], low_limit);
    assert!(
        limited.stop().is_some(),
        "the lower bound stops the same walk (full used={used}, limit={low_limit}, used={})",
        limited_budget
            .usage()
            .counted_usage(jarde_reader::budget::CountedBudgetDimension::IrItems)
    );
    assert!(
        limited_budget
            .usage()
            .counted_usage(jarde_reader::budget::CountedBudgetDimension::IrItems)
            <= low_limit
    );
}

#[test]
fn an_instance_call_upcasts_this_to_object_from_the_declaring_class_fact() {
    let class = reference_invocation_class(true);
    let report = present(&class, b"caller", b"()I", 1, vec![Some("self".into())]);
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert_eq!(report.quality, Quality::Structured);
    assert!(
        report
            .text
            .contains("return take((java.lang.Object) this);"),
        "{}",
        report.text
    );
    assert_eq!(
        report
            .evidence
            .state(jarde_java::RecoveryEvidenceKind::SourceMap),
        jarde_java::EvidenceState::Complete
    );
    let caller = member_identity(&class, b"caller", b"()I");
    let calls = report.source_map.of_bci(1);
    assert!(!calls.is_empty(), "the call BCI is mapped: {}", report.text);
    assert!(
        calls
            .iter()
            .all(|segment| segment.origin().primary().method() == Some(&caller))
    );
    let returns = report.source_map.direct_of_bci(4);
    assert!(
        !returns.is_empty(),
        "the return BCI is mapped: {}",
        report.text
    );
    assert!(
        returns
            .iter()
            .all(|segment| segment.origin().primary().method() == Some(&caller))
    );
    let casts = report.source_map.derived_of_bci(1);
    assert!(
        !casts.is_empty(),
        "the cast keeps the call as derived source: {}",
        report.text
    );
    assert!(
        casts
            .iter()
            .all(|segment| segment.origin().primary().method() == Some(&caller))
    );
}

// ---------------------------------------------------------------------------
// P3 2.3: the class-level facts a caller states, and the harness that reads them
// ---------------------------------------------------------------------------

/// The facts of one method **with the class that declares it**: its member flags, its debug names,
/// and the class's own name and flags as the same header read states them.
///
/// This is what `declaration@1`, `field@1` and `init@1` read (P3 2.3). The facts are read out of the
/// real class file rather than written into this file, so a fixture whose flags say "interface" is
/// one because the bytes say so.
fn facts_of_in(
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
        .with_access_flags(member.access_flags)
        .with_declaring_class(DeclaringClass::new(
            String::from_utf8_lossy(&header.this_class.raw().0).into_owned(),
            header.access_flags,
        )),
    )
    .with_debug_locals(stated_names(debug))
}

/// Presents one fixture body with the class that declares it stated.
fn present_in(
    class: &[u8],
    name: &[u8],
    descriptor: &[u8],
    parameters: u16,
    debug: Vec<Option<String>>,
) -> jarde_java::RecoveryReport {
    let payload = analyze(class, name, descriptor);
    let facts = facts_of_in(class, name, parameters, debug);
    let members = members_of(class);
    let mut budget = Budget::new(limits());
    recover_body(&payload, &facts, Some(&members), &mut budget)
}

/// Every BCI the artifact names in a quoted bytecode line, in the order it names them.
fn quoted_bcis(report: &jarde_java::RecoveryReport) -> Vec<u32> {
    report
        .text
        .lines()
        .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
        .flat_map(|list| list.split_whitespace())
        .map(|bci| bci.parse().expect("a BCI in a quote"))
        .collect()
}

/// Every instruction of a body the artifact does **not** account for: one that no segment reaches and
/// no quoted bytecode line names.
///
/// This is the invariant P3 2.3 §0 restored. Before it, an invocation whose only reader was an
/// instruction this run quotes wrote no statement of its own (because "something reads it") and the
/// reader wrote nothing (because it is quoted), so the invocation was in neither the text nor the
/// quote: its effect had silently left the artifact.
fn unaccounted_instructions(report: &jarde_java::RecoveryReport, bcis: &[u32]) -> Vec<u32> {
    let quoted = quoted_bcis(report);
    bcis.iter()
        .copied()
        .filter(|bci| !quoted.contains(bci) && report.text_of_bci(*bci).is_empty())
        .collect()
}

/// The line of an artifact that contains one piece of text, or a panic naming the artifact.
fn line_with(report: &jarde_java::RecoveryReport, needle: &str) -> usize {
    report
        .text
        .lines()
        .position(|line| line.contains(needle))
        .unwrap_or_else(|| panic!("no line contains `{needle}`:\n{}", report.text))
}

// ---------------------------------------------------------------------------
// A: an anonymous class's use site, and the nesting this slice does not claim
// ---------------------------------------------------------------------------

/// `p/Outer.method()Lp/Outer$1;` — an instance method that creates the class the compiler minted for
/// an anonymous class, handing the synthetic constructor the enclosing instance the source captured.
/// The body's bytes come back with the class, for the oracle's own decoder.
fn anonymous_use_class() -> (Vec<u8>, Vec<u8>) {
    let mut pool = Pool::default();
    let _code = code_attribute(&mut pool);
    let outer_name = pool.utf8("p/Outer");
    let outer = pool.class(outer_name);
    let anonymous_name = pool.utf8("p/Outer$1");
    let anonymous = pool.class(anonymous_name);
    let object_name = pool.utf8("java/lang/Object");
    let object = pool.class(object_name);
    let init = member_ref(&mut pool, anonymous, "<init>", "(Lp/Outer;)V");
    let method = pool.utf8("method");
    let method_descriptor = pool.utf8("()Lp/Outer$1;");
    let code = Code::default()
        .op(0xbb)
        .index(anonymous) // 0: new p/Outer$1
        .op(0x59) // 3: dup
        .op(0x2a) // 4: aload_0  (the enclosing instance, pushed after the copy)
        .op(0xb7)
        .index(init) // 5: invokespecial p/Outer$1.<init>(Lp/Outer;)V
        .op(0xb0) // 8: areturn
        .done();
    let class = class_bytes(
        &pool,
        outer,
        object,
        &[],
        &[MemberDef {
            flags: 0x0001,
            name: method,
            descriptor: method_descriptor,
            max_stack: 3,
            max_locals: 1,
            code: code.clone(),
        }],
    );
    (class, code)
}

/// The same use site with a **copy** of the instance stored twice: the value left after the
/// constructor is copied again, and the only instructions that read it are ones this build quotes —
/// so the construction has no statement to be written in.
fn anonymous_use_twice_class() -> Vec<u8> {
    let mut pool = Pool::default();
    let _code = code_attribute(&mut pool);
    let outer_name = pool.utf8("p/Outer");
    let outer = pool.class(outer_name);
    let anonymous_name = pool.utf8("p/Outer$1");
    let anonymous = pool.class(anonymous_name);
    let object_name = pool.utf8("java/lang/Object");
    let object = pool.class(object_name);
    let init = member_ref(&mut pool, anonymous, "<init>", "(Lp/Outer;)V");
    let method = pool.utf8("method");
    let method_descriptor = pool.utf8("()Lp/Outer$1;");
    let code = Code::default()
        .op(0xbb)
        .index(anonymous) // 0: new p/Outer$1
        .op(0x59) // 3: dup
        .op(0x2a) // 4: aload_0
        .op(0xb7)
        .index(init) // 5: invokespecial p/Outer$1.<init>(Lp/Outer;)V
        .op(0x59) // 8: dup  ← a second copy, which this build quotes
        .op(0x4c) // 9: astore_1
        .op(0x4d) // 10: astore_2
        .op(0x2b) // 11: aload_1
        .op(0xb0) // 12: areturn
        .done();
    class_bytes(
        &pool,
        outer,
        object,
        &[],
        &[MemberDef {
            flags: 0x0001,
            name: method,
            descriptor: method_descriptor,
            max_stack: 3,
            max_locals: 3,
            code,
        }],
    )
}

#[test]
fn an_anonymous_classs_use_is_presented_with_the_enclosing_instance_it_really_read() {
    let (class, _code) = anonymous_use_class();
    let report = present_in(
        &class,
        b"method",
        b"()Lp/Outer$1;",
        1,
        vec![Some("self".into())],
    );
    assert!(report.produced(), "{:?}", report.stop());
    // The construction is one expression: the class the pool names (`p/Outer$1`, the name the
    // compiler minted) and the value the site really read as its argument.
    assert!(
        report.text.contains("return new p.Outer$1(this);"),
        "{}",
        report.text
    );
    assert_eq!(report.news.len(), 1);
    let site = &report.news[0];
    assert!(site.presented(), "{:?}", site);
    assert_eq!(site.class, "p/Outer$1");
    assert_eq!(site.head, 0);
    assert_eq!(site.dup, Some(3));
    assert_eq!(site.constructor, Some(5));
    assert_eq!(site.arguments, vec![4], "the argument's own BCI is kept");
    assert_eq!(site.rule().citation(), "new@1");
    // The nesting relation is *not* claimed: nothing in the artifact says the class is anonymous,
    // that it belongs to `p/Outer`, or that the argument is an enclosing instance.
    assert!(!report.text.contains("Outer.this"), "{}", report.text);
    assert_eq!(report.representation, Representation::Java);
    assert_eq!(report.quality, Quality::Structured);
}

#[test]
fn a_construction_only_quoted_instructions_read_is_refused_and_quoted_whole() {
    let class = anonymous_use_twice_class();
    let report = present_in(
        &class,
        b"method",
        b"()Lp/Outer$1;",
        1,
        vec![Some("self".into())],
    );
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.news.len(), 1);
    let site = &report.news[0];
    assert!(!site.presented(), "{:?}", site);
    let refusal = site.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_new_shape");
    assert!(
        refusal.message.contains("no place in the body"),
        "the refusal states why the construction cannot be written: {}",
        refusal.message
    );
    assert!(
        refusal.message.contains("8"),
        "and names the reader that is quoted instead: {}",
        refusal.message
    );
    assert!(
        !report.text.contains("new p.Outer$1(self)"),
        "{}",
        report.text
    );
    // Every instruction of the refused site is still accounted for: quoted, with its own BCI.
    assert_eq!(
        unaccounted_instructions(&report, &[0, 3, 5, 8, 9, 10, 12]),
        Vec::<u32>::new(),
        "{}",
        report.text
    );
    assert_eq!(report.representation, Representation::Mixed);
}

// ---------------------------------------------------------------------------
// B: the dispatch table a compiler's `switch` over an enum reads
// ---------------------------------------------------------------------------

/// `p/Outer.method(…)I` — `switch (order)` on an enum, as a compiler lowers it: a static `int[]`
/// table read at the constant's ordinal, then a `tableswitch` on the element.
///
/// The switch's own layout is computed here rather than written out, because the padding of a
/// `tableswitch` depends on the BCI it sits at — and the two variants of this fixture put it at two
/// different addresses. The BCIs of the instructions that produce statements come back with the
/// class, so a test can require every one of them to be accounted for.
fn enum_switch_class(index_is_a_call: bool) -> (Vec<u8>, Vec<u8>, Vec<u32>) {
    let mut pool = Pool::default();
    let _code = code_attribute(&mut pool);
    let outer_name = pool.utf8("p/Outer");
    let outer = pool.class(outer_name);
    let object_name = pool.utf8("java/lang/Object");
    let object = pool.class(object_name);
    let holder_name = pool.utf8("p/Outer$1");
    let holder = pool.class(holder_name);
    let order_name = pool.utf8("p/Order");
    let order = pool.class(order_name);
    let table = field_ref(&mut pool, holder, "$SwitchMap$p$Order", "[I");
    let ordinal = member_ref(&mut pool, order, "ordinal", "()I");
    let method = pool.utf8("method");
    let method_descriptor = if index_is_a_call {
        pool.utf8("(Lp/Order;)I")
    } else {
        pool.utf8("(I)I")
    };
    let mut code = Code::default()
        .op(0xb2)
        .index(table) // 0: getstatic p/Outer$1.$SwitchMap$p$Order [I
        .done();
    if index_is_a_call {
        code.extend_from_slice(&[0x2a]); // 3: aload_0
        code.extend_from_slice(&[0xb6, (ordinal >> 8) as u8, ordinal as u8]); // 4: invokevirtual ordinal()I
    } else {
        code.extend_from_slice(&[0x1a]); // 3: iload_0 — a local, not a call
    }
    let iaload = u32::try_from(code.len()).expect("a fixture body fits u32");
    code.push(0x2e);
    let switch_at = u32::try_from(code.len()).expect("a fixture body fits u32");
    // The operands of a `tableswitch` start at the next four-byte boundary after its opcode.
    let padding = (4 - ((switch_at as usize + 1) % 4)) % 4;
    let operands_at = switch_at as usize + 1 + padding;
    let arms_at = operands_at + 4 + 4 + 4 + 2 * 4;
    let (arm_one, arm_two, default) = (arms_at as u32, arms_at as u32 + 2, arms_at as u32 + 4);
    code.push(0xaa);
    code.extend(std::iter::repeat_n(0_u8, padding));
    for value in [
        i32::try_from(default - switch_at).expect("an offset fits"),
        1,
        2,
        i32::try_from(arm_one - switch_at).expect("an offset fits"),
        i32::try_from(arm_two - switch_at).expect("an offset fits"),
    ] {
        code.extend_from_slice(&value.to_be_bytes());
    }
    // Each arm is one `return` of its own constant, and the default is the third.
    let returns = arms_at as u32;
    code.extend_from_slice(&[0x04, 0xac]); // arm one: iconst_1; ireturn
    code.extend_from_slice(&[0x05, 0xac]); // arm two: iconst_2; ireturn
    code.extend_from_slice(&[0x03, 0xac]); // default: iconst_0; ireturn
    let statements = vec![0, iaload, switch_at, returns, returns + 2, returns + 4];
    let class = class_bytes(
        &pool,
        outer,
        object,
        &[],
        &[MemberDef {
            // A **static** method: slot 0 is the enum value the switch reads, not `this`.
            flags: 0x0008,
            name: method,
            descriptor: method_descriptor,
            max_stack: 2,
            max_locals: 1,
            code: code.clone(),
        }],
    );
    (class, code, statements)
}

#[test]
fn an_enum_switch_is_presented_as_the_table_read_the_bytecode_performs() {
    let (class, _code, _statements) = enum_switch_class(true);
    let report = present_in(
        &class,
        b"method",
        b"(Lp/Order;)I",
        1,
        vec![Some("order".into())],
    );
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.enum_switches.len(), 1);
    let read = &report.enum_switches[0];
    assert!(read.presented(), "{:?}", read);
    let table = read.table.as_ref().expect("the table is recorded");
    assert_eq!(table.owner, "p/Outer$1");
    assert_eq!(table.name, "$SwitchMap$p$Order");
    assert_eq!(table.descriptor, "[I");
    let index = read.index.as_ref().expect("the index call is recorded");
    assert_eq!(index.owner, "p/Order");
    assert_eq!(index.name, "ordinal");
    assert_eq!(index.descriptor, "()I");
    assert_eq!(read.rule().citation(), "enumswitch@1");
    // The table read is written where the switch's selector is written, and it is read once.
    assert!(
        report
            .text
            .contains("switch (p.Outer$1.$SwitchMap$p$Order[order.ordinal()])"),
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("ordinal()").count(),
        1,
        "{}",
        report.text
    );
    assert!(report.text.contains("case 1:"), "{}", report.text);
    assert!(report.text.contains("default:"), "{}", report.text);
    // The constant mapping is *not* claimed: the enum class's own constants were never read, so no
    // `case p.Order.…` label can be written, and the run says so.
    assert!(!report.text.contains("p.Order."), "{}", report.text);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_enumswitch"
                && diagnostic.message.contains("enum class's own declaration")),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn class_source_enum_candidate_keeps_same_run_table_switch_and_receiver_identity() {
    let (class, _code, statements) = enum_switch_class(true);
    let mut essential_budget = Budget::new(limits());
    let essential = recover_class_source_exact(
        &class,
        b"method",
        b"(Lp/Order;)I",
        1,
        jarde_java::RecoveryEvidenceRequest::essential(),
        &mut essential_budget,
    );
    assert!(
        essential.report.produced(),
        "{:?}",
        essential.report.outcome
    );
    assert_eq!(essential.enum_switches.len(), 1);
    let candidate = &essential.enum_switches[0];
    let member = candidate.member.as_ref().expect("same-run member identity");
    assert_eq!(member.name.0, b"method");
    assert_eq!(member.descriptor.0, b"(Lp/Order;)I");
    assert_eq!(candidate.table.owner, "p/Outer$1");
    assert_eq!(candidate.table.name, "$SwitchMap$p$Order");
    assert_eq!(candidate.table.descriptor, "[I");
    assert_eq!(candidate.index.owner, "p/Order");
    assert_eq!(candidate.index.name, "ordinal");
    assert_eq!(candidate.index.descriptor, "()I");
    assert_eq!(candidate.read_bci, statements[1]);
    assert_eq!(candidate.switch_bci, statements[2]);
    assert_eq!(candidate.keys, [1, 2]);
    assert_eq!(
        candidate.selector_receiver_type.as_deref(),
        Some(&b"Lp/Order;"[..])
    );

    let mut all_budget = Budget::new(limits());
    let all = recover_class_source_exact(
        &class,
        b"method",
        b"(Lp/Order;)I",
        1,
        jarde_java::RecoveryEvidenceRequest::all(),
        &mut all_budget,
    );
    assert!(all.report.produced(), "{:?}", all.report.outcome);
    assert_eq!(all.enum_switches, essential.enum_switches);
}

#[test]
fn a_table_read_indexed_by_a_local_is_refused_and_the_subscript_is_written() {
    let (class, _code, statements) = enum_switch_class(false);
    let report = present_in(&class, b"method", b"(I)I", 1, Vec::new());
    assert_eq!(report.enum_switches.len(), 1);
    let read = &report.enum_switches[0];
    assert!(!read.presented(), "{:?}", read);
    let refusal = read.refusal.as_ref().expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_enumswitch_shape");
    assert!(
        refusal.message.contains("which is not a call"),
        "{}",
        refusal.message
    );
    // P3 2b.1 supersedes what this test used to pin. `enumswitch@1` still refuses the read — the
    // index is a local and not a call, and the refusal is still recorded above — but a read no rule
    // claimed is no longer a reason to quote the block around it: the subscript is written, and the
    // switch is the statement its own decode states (its keys, its arms and its default), which is
    // measured here. The enum class's constants are still not claimed, so the arms keep the numbers
    // the payload's keys are.
    assert!(
        report
            .text
            .contains("switch (p.Outer$1.$SwitchMap$p$Order[arg0])"),
        "{}",
        report.text
    );
    assert!(report.text.contains("case 1:"), "{}", report.text);
    assert!(report.text.contains("case 2:"), "{}", report.text);
    assert!(report.text.contains("default:"), "{}", report.text);
    assert!(!report.text.contains("p.Order."), "{}", report.text);
    // The region names **every** instruction it covers — the selector's, the arms' and the
    // returns' — not just the `tableswitch`: P3 2.3 §0's other half.
    assert_eq!(
        unaccounted_instructions(&report, &statements),
        Vec::<u32>::new(),
        "{}",
        report.text
    );
    assert_eq!(
        report.representation,
        Representation::Java,
        "{}",
        report.text
    );
    assert_eq!(report.quality, Quality::Structured, "{}", report.text);
}

// ---------------------------------------------------------------------------
// C: what the class file declares the member to be
// ---------------------------------------------------------------------------

/// One interface (or class) whose single method is declared with the flags given.
fn one_method_class(class_flags: u16, member_flags: u16) -> Vec<u8> {
    let mut pool = Pool::default();
    let _code = code_attribute(&mut pool);
    let shape_name = pool.utf8("p/Shape");
    let shape = pool.class(shape_name);
    let object_name = pool.utf8("java/lang/Object");
    let object = pool.class(object_name);
    let run = pool.utf8("run");
    let descriptor = pool.utf8("()V");
    class_bytes_with(
        &pool,
        class_flags,
        shape,
        object,
        &[],
        &[MemberDef {
            flags: member_flags,
            name: run,
            descriptor,
            max_stack: 0,
            max_locals: 1,
            code: Code::default().op(0xb1).done(),
        }],
    )
}

#[test]
fn an_interfaces_non_abstract_method_is_stated_as_a_default_method() {
    // `ACC_INTERFACE | ACC_ABSTRACT` is what JVMS 4.1 requires of an interface's own flags; the
    // member's `public` without `abstract` is what makes it a `default` method.
    let class = one_method_class(0x0601, 0x0001);
    let report = present_in(&class, b"run", b"()V", 1, vec![Some("self".into())]);
    assert!(report.produced(), "{:?}", report.stop());
    let declaration = report
        .declaration
        .as_ref()
        .expect("the declaration is read");
    assert_eq!(declaration.form, Some(DeclarationForm::DefaultMethod));
    assert_eq!(declaration.interface, Some(true));
    assert_eq!(declaration.declaring_class.as_deref(), Some("p/Shape"));
    assert!(declaration.presented());
    assert!(
        report.text.contains(
            "// @declaration an interface's default method of `p.Shape`, member flags 0x0001"
        ),
        "{}",
        report.text
    );
    assert!(
        report.rules.contains(&declaration.rule()),
        "{:?}",
        report.rules
    );
}

#[test]
fn the_same_member_in_a_class_is_an_ordinary_method_and_a_static_one_is_stated_as_such() {
    // The same member flags in a class — which is exactly why the class's own fact has to be stated
    // instead of guessed from the member: a `public` method that is not abstract is a `default`
    // method in an interface and an ordinary method in a class.
    let class = one_method_class(CLASS_FLAGS, 0x0001);
    let report = present_in(&class, b"run", b"()V", 1, vec![Some("self".into())]);
    let declaration = report
        .declaration
        .as_ref()
        .expect("the declaration is read");
    assert_eq!(declaration.form, Some(DeclarationForm::InstanceMethod));
    assert_eq!(declaration.interface, Some(false));
    assert!(
        report
            .text
            .contains("// @declaration an instance method of `p.Shape`"),
        "{}",
        report.text
    );

    // And an interface's `static` method is the other Java 8 addition.
    let class = one_method_class(0x0601, 0x0009);
    let report = present_in(&class, b"run", b"()V", 0, Vec::new());
    let declaration = report
        .declaration
        .as_ref()
        .expect("the declaration is read");
    assert_eq!(
        declaration.form,
        Some(DeclarationForm::StaticInterfaceMethod)
    );
    assert!(
        report.text.contains("an interface's static method"),
        "{}",
        report.text
    );
}

#[test]
fn a_run_that_was_not_told_the_declaring_class_states_no_declaration() {
    // The facts this harness states name no declaring class, so this is the shape a caller that
    // states none has: the refusal names the fact it was missing, the envelope carries no
    // declaration line, and the rule is not listed as one that produced this artifact. (The root
    // facade entry is no longer this shape — the declaring-class handoff made `MethodIr` carry the
    // class's raw name and flags with the member's declaration, and `Engine::recover_method` fills
    // them from there — which is why this case drives the recovery layer directly, with facts that
    // state the member but not the class.)
    let class = one_method_class(0x0601, 0x0001);
    let report =
        present_without_declaring_class(&class, b"run", b"()V", 1, vec![Some("self".into())]);
    let declaration = report.declaration.as_ref().expect("the record is written");
    assert!(!declaration.presented());
    assert_eq!(declaration.form, None);
    let refusal = declaration
        .refusal
        .as_ref()
        .expect("the refusal is recorded");
    assert_eq!(refusal.code, "jre_declaration_class_not_in_run");
    assert_eq!(
        refusal.requirement.as_deref(),
        Some("the `declaring_class` attribute")
    );
    assert!(!report.text.contains("@declaration"), "{}", report.text);
    assert!(
        !report.rules.contains(&declaration.rule()),
        "{:?}",
        report.rules
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_declaration_class_not_in_run"),
        "{:?}",
        report.diagnostics
    );
}

// ---------------------------------------------------------------------------
// D: a constructor's field initializers, the static initializer, and the prologue
// ---------------------------------------------------------------------------

/// `Test.<init>()V` with a field initializer and a static one, exactly as a compiler writes them:
/// the constructor call first, then the writes in source order. The body's bytes come back with the
/// class so that the oracle's own decoder runs the very instructions the run was handed.
fn constructor_class() -> (Vec<u8>, Vec<u8>) {
    let (mut pool, _code, test, object) = base_pool();
    let f_name = pool.utf8("f");
    let f_descriptor = pool.utf8("I");
    let g_name = pool.utf8("g");
    let g_descriptor = pool.utf8("I");
    let f = field_ref(&mut pool, test, "f", "I");
    let g = field_ref(&mut pool, test, "g", "I");
    let super_init = member_ref(&mut pool, object, "<init>", "()V");
    let method = pool.utf8("<init>");
    let descriptor = pool.utf8("()V");
    let code = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0xb7)
        .index(super_init) // 1: invokespecial java/lang/Object.<init>()V
        .op(0x2a) // 4: aload_0
        .op(0x08) // 5: iconst_5
        .op(0xb5)
        .index(f) // 6: putfield Test.f:I
        .op(0x10)
        .byte(7) // 9: bipush 7
        .op(0xb3)
        .index(g) // 11: putstatic Test.g:I
        .op(0xb1) // 14: return
        .done();
    let class = class_bytes(
        &pool,
        test,
        object,
        &[
            FieldDef {
                flags: 0x0002,
                name: f_name,
                descriptor: f_descriptor,
            },
            FieldDef {
                flags: 0x0008,
                name: g_name,
                descriptor: g_descriptor,
            },
        ],
        &[MemberDef {
            flags: 0x0001,
            name: method,
            descriptor,
            max_stack: 1,
            max_locals: 1,
            code: code.clone(),
        }],
    );
    (class, code)
}

/// `Test.<clinit>()V`: the class initializer a compiler fills with the static initializers.
fn static_initializer_class() -> Vec<u8> {
    let (mut pool, _code, test, object) = base_pool();
    let g_name = pool.utf8("g");
    let g_descriptor = pool.utf8("I");
    let g = field_ref(&mut pool, test, "g", "I");
    let method = pool.utf8("<clinit>");
    let descriptor = pool.utf8("()V");
    let code = Code::default()
        .op(0x10)
        .byte(7) // 0: bipush 7
        .op(0xb3)
        .index(g) // 2: putstatic Test.g:I
        .op(0xb1) // 5: return
        .done();
    class_bytes(
        &pool,
        test,
        object,
        &[FieldDef {
            flags: 0x0008,
            name: g_name,
            descriptor: g_descriptor,
        }],
        &[MemberDef {
            flags: 0x0008,
            name: method,
            descriptor,
            max_stack: 1,
            max_locals: 0,
            code,
        }],
    )
}

/// An inner class's constructor: the synthetic reference to the enclosing instance is written
/// **before** the constructor call, which JVMS 4.10.1.9 allows and which is the one write whose
/// receiver is not a typed value.
fn inner_constructor_class() -> Vec<u8> {
    let mut pool = Pool::default();
    let _code = code_attribute(&mut pool);
    let inner_name = pool.utf8("p/Outer$1");
    let inner = pool.class(inner_name);
    let outer_name = pool.utf8("p/Outer");
    let outer = pool.class(outer_name);
    let field = field_ref(&mut pool, inner, "this$0", "Lp/Outer;");
    let super_init = member_ref(&mut pool, outer, "<init>", "()V");
    let method = pool.utf8("<init>");
    let descriptor = pool.utf8("(Lp/Outer;)V");
    let code = Code::default()
        .op(0x2a) // 0: aload_0
        .op(0x2b) // 1: aload_1  (the enclosing instance)
        .op(0xb5)
        .index(field) // 2: putfield p/Outer$1.this$0:Lp/Outer;  ← before the constructor call
        .op(0x2a) // 5: aload_0
        .op(0xb7)
        .index(super_init) // 6: invokespecial p/Outer.<init>()V
        .op(0xb1) // 9: return
        .done();
    let field_name = pool.utf8("this$0");
    let field_descriptor = pool.utf8("Lp/Outer;");
    class_bytes_with(
        &pool,
        CLASS_FLAGS,
        inner,
        outer,
        &[FieldDef {
            flags: 0x0002,
            name: field_name,
            descriptor: field_descriptor,
        }],
        &[MemberDef {
            flags: 0x0001,
            name: method,
            descriptor,
            max_stack: 2,
            max_locals: 2,
            code,
        }],
    )
}

#[test]
fn a_debug_table_that_names_slot_zero_this_is_not_a_keyword_to_alias() {
    // The same constructor body with the name a `-g` build's `LocalVariableTable` really states for
    // slot 0: `this`. The slot is the **receiver** (JVMS 4.10.1.9), so it is written as the keyword —
    // not as the alias `this_` the keyword rule would make of that debug name, and not as the ordinal
    // `arg0` a body with no table gets.
    let (class, _code) = constructor_class();
    let report = present_in(&class, b"<init>", b"()V", 1, vec![Some("this".into())]);
    assert!(report.produced(), "{:?}", report.stop());
    assert!(report.text.contains("this.f = 5;"), "{}", report.text);
    assert!(!report.text.contains("this_"), "{}", report.text);
    assert!(!report.text.contains("arg0"), "{}", report.text);
    // The evidence the table states is not hidden, so nothing is aliased and nothing is reported as
    // an alias of a name this layer could not write.
    assert!(
        report.aliased_names.is_empty(),
        "{:?}",
        report.aliased_names
    );
    assert_eq!(report.syntax_status, SyntaxStatus::Unchecked);
    assert!(
        !report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_name_aliased"),
        "{:?}",
        report.diagnostics
    );
    // The static field write beside it keeps its own spelling: this reaches slot 0 of an instance
    // member and nothing else.
    assert!(report.text.contains("Test.g = 7;"), "{}", report.text);
}

#[test]
fn a_constructors_field_initializers_are_written_after_its_constructor_call_and_in_order() {
    let (class, _code) = constructor_class();
    let report = present_in(&class, b"<init>", b"()V", 1, vec![Some("self".into())]);
    assert!(report.produced(), "{:?}", report.stop());
    let prologue = report.init.as_ref().expect("the prologue is read");
    assert!(prologue.presented(), "{:?}", prologue);
    assert_eq!(prologue.target, Some(ConstructorTarget::Super));
    assert_eq!(prologue.class.as_deref(), Some("java/lang/Object"));
    assert_eq!(prologue.declared.as_deref(), Some("Test"));
    assert_eq!(prologue.bci, Some(1));
    assert_eq!(prologue.rule().citation(), "init@1");

    // The instance initializer and the static one are the writes the body really performs.
    assert_eq!(report.fields.len(), 2, "{:?}", report.fields);
    assert!(report.fields.iter().all(|record| record.presented()));
    assert_eq!(
        report
            .fields
            .iter()
            .map(|record| (record.access, record.is_static, record.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("write", false, "f"), ("write", true, "g")]
    );

    // **The order is the invariant**: the constructor call first, then the instance initializer,
    // then the static one — never reordered, never moved out of the constructor.
    let super_line = line_with(&report, "super();");
    let field_line = line_with(&report, "this.f = 5;");
    let static_line = line_with(&report, "Test.g = 7;");
    let return_line = line_with(&report, "return;");
    assert!(
        super_line < field_line && field_line < static_line && static_line < return_line,
        "the writes keep the order the bytecode has:\n{}",
        report.text
    );
    assert_eq!(report.text.matches("super()").count(), 1, "{}", report.text);
    // The declaration of an instance initializer is read from its name, without the class's flags.
    assert_eq!(
        report
            .declaration
            .as_ref()
            .and_then(|declaration| declaration.form),
        Some(DeclarationForm::Constructor)
    );
}

#[test]
fn a_static_initializer_is_written_as_the_writes_it_performs_and_declared_as_one() {
    let class = static_initializer_class();
    let report = present_in(&class, b"<clinit>", b"()V", 0, Vec::new());
    assert!(report.produced(), "{:?}", report.stop());
    assert!(report.text.contains("Test.g = 7;"), "{}", report.text);
    assert!(!report.text.contains("return;"), "{}", report.text);
    let members = members_of(&class);
    let clinit = members
        .members()
        .iter()
        .find(|member| member.name() == "<clinit>" && member.descriptor() == "()V")
        .expect("the fixture declares the static initializer");
    let closing = report.source_map.direct_of_bci(5);
    assert_eq!(
        closing.len(),
        1,
        "the tail return maps once: {}",
        report.text
    );
    assert_eq!(closing[0].text(&report.text), "}\n");
    assert_eq!(closing[0].origin().primary().bci(), 5);
    assert_eq!(
        closing[0].origin().primary().method(),
        Some(clinit.identity()),
        "the closing brace keeps the <clinit> member identity"
    );
    assert_eq!(report.fields.len(), 1);
    assert!(report.fields[0].presented());
    assert_eq!(report.fields[0].name, "g");
    assert!(report.fields[0].is_static);
    assert_eq!(
        report
            .declaration
            .as_ref()
            .and_then(|declaration| declaration.form),
        Some(DeclarationForm::StaticInitializer)
    );
    assert!(
        report
            .text
            .contains("// @declaration a static initializer of `Test`"),
        "{}",
        report.text
    );
    // There is no constructor prologue to present: the body is not an instance initializer.
    assert!(report.init.is_none(), "{:?}", report.init);

    let payload = analyze(&class, b"<clinit>", b"()V");
    let facts = facts_of_in(&class, b"<clinit>", 0, Vec::new());
    let mut budget = Budget::new(limits());
    let essential = recover(
        &RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8)
            .with_members(&members),
        &mut budget,
    );
    assert!(essential.produced(), "{:?}", essential.stop());
    assert_eq!(essential.text, report.text);
    assert!(essential.source_map.is_empty());
    assert_eq!(
        essential
            .evidence
            .state(jarde_java::RecoveryEvidenceKind::SourceMap),
        jarde_java::EvidenceState::NotRequested
    );
}

#[test]
fn a_static_initializer_source_map_stop_keeps_the_artifact_and_unpaid_tail_unmapped() {
    let class = static_initializer_class();
    let payload = analyze(&class, b"<clinit>", b"()V");
    let facts = facts_of_in(&class, b"<clinit>", 0, Vec::new());
    let members = members_of(&class);
    let selection = jarde_java::RecoveryEvidenceRequest::essential()
        .with_kind(jarde_java::RecoveryEvidenceKind::SourceMap);
    let (full, used) = {
        let mut budget = Budget::new(limits());
        let full = recover(
            &RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8)
                .with_members(&members)
                .with_evidence(selection.clone()),
            &mut budget,
        );
        assert!(full.produced(), "{:?}", full.stop());
        assert!(full.source_map.len() >= 2, "{}", full.text);
        assert_eq!(
            full.evidence
                .state(jarde_java::RecoveryEvidenceKind::SourceMap),
            jarde_java::EvidenceState::Complete
        );
        let used = budget
            .usage()
            .counted_usage(jarde_reader::budget::CountedBudgetDimension::IrItems);
        (full, used)
    };
    let mut budget = Budget::new(Limits {
        ir_items: used.saturating_sub(1),
        ..limits()
    });
    let partial = recover(
        &RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8)
            .with_members(&members)
            .with_evidence(selection),
        &mut budget,
    );
    assert!(partial.produced(), "{:?}", partial.stop());
    assert_eq!(
        partial.text, full.text,
        "evidence cannot mutate the artifact"
    );
    assert_eq!(partial.content, full.content);
    assert!(partial.source_map.len() < full.source_map.len());
    assert!(partial.source_map.direct_of_bci(5).is_empty());
    assert!(matches!(
        partial
            .evidence
            .state(jarde_java::RecoveryEvidenceKind::SourceMap),
        jarde_java::EvidenceState::Partial { .. }
    ));
}

#[test]
fn a_write_made_before_the_constructor_call_stays_where_the_bytecode_made_it() {
    let class = inner_constructor_class();
    let report = present_in(
        &class,
        b"<init>",
        b"(Lp/Outer;)V",
        2,
        vec![Some("self".into()), Some("outer".into())],
    );
    assert!(report.produced(), "{:?}", report.stop());
    // The `UninitializedThis` write is presented under JVMS 4.10.1.9's own rule — the `Fieldref`
    // names the class being constructed — and it stays **before** the constructor call, which is
    // where the bytes put it (JLS 12.5 runs the instance initializers after `super(…)`, and this one
    // is the compiler's own pre-call write).
    let write_line = line_with(&report, "this.this$0 = outer;");
    let super_line = line_with(&report, "super();");
    assert!(
        write_line < super_line,
        "the write is not moved past the constructor call:\n{}",
        report.text
    );
    assert_eq!(report.fields.len(), 1);
    assert!(report.fields[0].presented(), "{:?}", report.fields);
    assert_eq!(report.fields[0].name, "this$0");
    assert_eq!(report.fields[0].access, "write");
    assert_eq!(
        report.init.as_ref().and_then(|prologue| prologue.target),
        Some(ConstructorTarget::Super)
    );
    assert_eq!(
        report
            .init
            .as_ref()
            .and_then(|prologue| prologue.class.as_deref()),
        Some("p/Outer")
    );
}

#[test]
fn a_run_that_was_not_told_the_class_refuses_the_pre_call_write_and_the_prologue() {
    // The same body with no class-level fact: neither the write on the uninitialized `this` nor the
    // prologue can be decided, and both are *stated* refusals with the missing fact named — not a
    // dropped assignment, not a guessed `super`/`this`.
    let class = inner_constructor_class();
    let report = present_without_declaring_class(
        &class,
        b"<init>",
        b"(Lp/Outer;)V",
        2,
        vec![Some("self".into()), Some("outer".into())],
    );
    let write = &report.fields[0];
    assert!(!write.presented(), "{:?}", write);
    assert_eq!(
        write.refusal.as_ref().map(|refusal| refusal.code),
        Some("jre_field_declaring_class_missing")
    );
    let prologue = report.init.as_ref().expect("the refusal is recorded");
    assert!(!prologue.presented());
    assert_eq!(
        prologue.refusal.as_ref().map(|refusal| refusal.code),
        Some("jre_init_class_not_in_run")
    );
    assert!(!report.text.contains("super()"), "{}", report.text);
    assert!(!report.text.contains("this()"), "{}", report.text);
    assert!(!report.text.contains("this$0 = outer"), "{}", report.text);
    // Nothing vanishes: every instruction of the body is written or quoted.
    assert_eq!(
        unaccounted_instructions(&report, &[2, 6, 9]),
        Vec::<u32>::new(),
        "{}",
        report.text
    );
}

// ---------------------------------------------------------------------------
// The oracle on the two 2.3 shapes whose meaning is an order
// ---------------------------------------------------------------------------

#[test]
fn the_oracle_agrees_with_the_run_on_a_constructor_and_on_a_construction() {
    // The constructor: the model runs the fixture's own bytes and reads the artifact's statements,
    // and requires the two to perform the same effects in the same order — the constructor call, the
    // instance initializer, the static one.
    let (class, code) = constructor_class();
    let report = present_in(&class, b"<init>", b"()V", 1, vec![Some("self".into())]);
    assert!(report.produced(), "{:?}", report.stop());
    let pool = read_pool(&class);
    // The model's entry locals are the names the artifact writes, slot 0 first: the receiver of this
    // instance method is `this` (JVMS 4.10.1.9), whatever the fixture's debug table calls it.
    let from_bytes = run_bytecode(&code, &pool, &["this"]);
    let from_text = run_text_effects(&report.text);
    assert_eq!(
        from_bytes.effects,
        vec![
            "prologue".to_string(),
            "write f = 5".to_string(),
            "write g = 7".to_string()
        ],
        "the model read the fixture's own order"
    );
    assert!(
        compare_effects(&from_bytes, &from_text).is_ok(),
        "{from_bytes:?} against {from_text:?}\n{}",
        report.text
    );

    // The construction: one instance of the class the pool itself names, handed the value the site
    // really read.
    let (class, code) = anonymous_use_class();
    let report = present_in(
        &class,
        b"method",
        b"()Lp/Outer$1;",
        1,
        vec![Some("self".into())],
    );
    let from_bytes = run_bytecode(&code, &read_pool(&class), &["this"]);
    let from_text = run_text_effects(&report.text);
    assert_eq!(from_bytes.effects, vec!["new p.Outer$1(this)".to_string()]);
    assert!(
        compare_effects(&from_bytes, &from_text).is_ok(),
        "{from_bytes:?} against {from_text:?}\n{}",
        report.text
    );
}

#[test]
fn the_oracle_rejects_a_constructor_whose_field_writes_are_in_another_order() {
    let (class, code) = constructor_class();
    let report = present_in(&class, b"<init>", b"()V", 1, vec![Some("self".into())]);
    let from_bytes = run_bytecode(&code, &read_pool(&class), &["this"]);
    // The same artifact with the two writes swapped: the bytecode wrote `f` before `g`, and the model
    // is required to see the swap however equal the two numbers happen to look.
    let swapped = report
        .text
        .replace("this.f = 5;", "\u{0}")
        .replace("Test.g = 7;", "this.f = 5;")
        .replace('\u{0}', "Test.g = 7;");
    assert!(swapped.contains("Test.g = 7;"), "{swapped}");
    let from_text = run_text_effects(&swapped);
    let rejected = compare_effects(&from_bytes, &from_text);
    assert!(
        rejected.is_err(),
        "two initializers written in another order are not the same body: {from_text:?}"
    );
}

#[test]
fn the_oracle_rejects_a_constructor_call_written_after_its_field_initializers() {
    let (class, code) = constructor_class();
    let report = present_in(&class, b"<init>", b"()V", 1, vec![Some("self".into())]);
    let from_bytes = run_bytecode(&code, &read_pool(&class), &["this"]);
    // JLS 12.5: the instance initializers run **after** the constructor call. An artifact that writes
    // them before it is a different program — the initializers would run on an object whose
    // superclass constructor has not run — and the model has to see that.
    let moved = report
        .text
        .replace("    super();\n", "")
        .replace("    return;\n", "    super();\n    return;\n");
    assert!(moved.contains("super();"), "{moved}");
    assert!(
        moved.find("super();").expect("the call") > moved.find("this.f = 5;").expect("the write"),
        "{moved}"
    );
    let from_text = run_text_effects(&moved);
    let rejected = compare_effects(&from_bytes, &from_text);
    assert!(
        rejected.is_err(),
        "a constructor call written after the initializers is not the same body: {from_text:?}"
    );
}
