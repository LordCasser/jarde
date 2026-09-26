//! P3 2.2 acceptance A12 and P3 3.2's anchors: presenting a synthetic accessor as a field access is
//! **derived**, the two original X1 edges survive it, and the anchors say which member body every
//! bytecode index is in.
//!
//! The property this file pins is about the artifact's own facts, not about the text:
//!
//! 1. `Engine::query` — X1 — still answers with **both** edges for the same fixture: the caller's
//!    call to `access$100` and that accessor's read of the field. They are asserted **each on its
//!    own** (their own source methods and BCIs), because "the total did not change" cannot tell a
//!    merged edge from two edges;
//! 2. the same two answers come out **field by field identical** whether or not a recovery run was
//!    performed over the same bytes in between — with `elapsed_millis` left out, since it is a
//!    measurement rather than a charge (P2 5.1/5.3's comparison, which this file repeats for the
//!    recovery side);
//! 3. the recovery run **of the same public entry** presents the call as the direct field expression
//!    the source had, having read the one member the call site named — and only that member (§3.2's
//!    on-demand read, with the usage numbers saying so);
//! 4. every anchor of that artifact names the member body its bytecode index is in, which is the
//!    only thing that tells two same-numbered indexes apart: the call site in this body and the
//!    field access inside the callee's, or two callees that each put their field access at BCI 1
//!    (`the_anchors_name_the_member_each_field_access_is_in_and_only_named_bodies_are_read`);
//! 5. a cloned bytecode index is one coordinate in one member: the fallback quote names it once —
//!    a clone is the same place reached twice, not a second place — and every quote names an
//!    anchor of that member
//!    (`a_bytecode_index_a_clone_repeats_still_names_the_member_it_is_in`).
//!
//! The fixtures are assembled here rather than committed, because no historical corpus in this
//! repository holds a compiler-generated accessor: `Test` declares the private fields, the static
//! synthetic accessors that read them, a member no call site names, and the members that call them;
//! the clone case assembles the `jsr`/`ret` body a version 45 class may spell.

use jarde::*;
use std::collections::BTreeSet;

const CALL_SITE: u32 = 3;
const FIELD_SITE: u32 = 1;

/// The counting tests need the process-global port to themselves: one target runs its tests in
/// parallel, and a count taken while another test is reading a class would be read as this
/// request's own. Every test here takes the gate (they are all short), so the one test that reads
/// the port can.
static GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn gate() -> std::sync::MutexGuard<'static, ()> {
    GATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 100,
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

/// The constant pool of the fixture, written entry by entry.
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

fn member_ref(pool: &mut Pool, class: u16, name: &str, descriptor: &str) -> u16 {
    let name = pool.utf8(name);
    let descriptor = pool.utf8(descriptor);
    let name_and_type = pool.name_and_type(name, descriptor);
    pool.method_ref(class, name_and_type)
}

/// One member of the fixture, with its body.
struct Member {
    flags: u16,
    name: u16,
    descriptor: u16,
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
}

/// `Test` with the two fields, the accessors a compiler generated for them, and the members that
/// call them:
///
/// ```text
/// f:I                        private, read by `access$100` alone
/// g:I                        private, read by `access$200` and `access$400`
/// h:I                        private, read by `access$300` alone
/// access$100(LTest;)I        public static synthetic: aload_0; getfield Test.f:I; ireturn
/// access$200(LTest;)I        public static synthetic: aload_0; getfield Test.g:I; ireturn
/// access$300(LTest;)I        public static synthetic: the same access at the same BCI, over `h`
/// access$400(LTest;)I        public static synthetic: the pure read of `g`, called by nobody
/// method()I                  public: iconst_0; istore_1; aload_0;
///                                    invokestatic Test.access$100(LTest;)I; ireturn
/// both()I                    public: aload_0; invokestatic Test.access$200(LTest;)I; aload_0;
///                                    invokestatic Test.access$300(LTest;)I; iadd; ireturn
/// ```
///
/// Two accessors whose field access sits at **one** bytecode index (1) in **two** member bodies, one
/// member body no call site names, and — in `both()` — a call site at BCI 1 whose callee's field
/// access is at BCI 1 too. Those are the coordinates a bare bytecode index cannot tell apart.
///
/// The edges the A12 case queries stay single-edged on purpose: `f` is read by one accessor alone,
/// and that accessor is called from `method()` alone, so "the two original edges" are two items.
fn accessor_class() -> Vec<u8> {
    accessor_class_with(false)
}

/// The same class, plus a member whose call site names **another class**'s member with the very same
/// name and descriptor this class also declares — the trap P3 3.2's binding has to refuse: a member
/// of a class that happens to share a name is not the member the call reaches, and reading it from
/// these bytes would present a call to one class as a field access of another.
fn foreign_call_class() -> Vec<u8> {
    accessor_class_with(true)
}

fn accessor_class_with(foreign_call: bool) -> Vec<u8> {
    let mut pool = Pool::default();
    pool.utf8("Code");
    let test_name = pool.utf8("Test");
    let test = pool.class(test_name);
    let object_name = pool.utf8("java/lang/Object");
    let object = pool.class(object_name);
    let field_descriptor = pool.utf8("I");
    let field_names = [pool.utf8("f"), pool.utf8("g"), pool.utf8("h")];
    let fields: Vec<u16> = field_names
        .iter()
        .map(|name| {
            let and_type = pool.name_and_type(*name, field_descriptor);
            pool.field_ref(test, and_type)
        })
        .collect();
    let accessor_name = pool.utf8("access$100");
    let second_accessor_name = pool.utf8("access$200");
    let third_accessor_name = pool.utf8("access$300");
    let fourth_accessor_name = pool.utf8("access$400");
    let accessor_descriptor = pool.utf8("(LTest;)I");
    let accessor = member_ref(&mut pool, test, "access$100", "(LTest;)I");
    let second_accessor = member_ref(&mut pool, test, "access$200", "(LTest;)I");
    let third_accessor = member_ref(&mut pool, test, "access$300", "(LTest;)I");
    let method_name = pool.utf8("method");
    let method_descriptor = pool.utf8("()I");
    let both_name = pool.utf8("both");
    let foreign_name = pool.utf8("foreign");
    let foreign_member = foreign_call.then(|| {
        let other_name = pool.utf8("Other");
        let other = pool.class(other_name);
        member_ref(&mut pool, other, "access$100", "(LTest;)I")
    });
    let read = |field: u16| -> Vec<u8> {
        vec![
            0x2a, // 0: aload_0
            0xb4, //
        ]
        .into_iter()
        .chain(field.to_be_bytes())
        .chain([0xac]) // 4: ireturn
        .collect()
    };
    let method_body = vec![
        0x03, // 0: iconst_0
        0x3c, // 1: istore_1
        0x2a, // 2: aload_0
        0xb8, //
    ]
    .into_iter()
    .chain(accessor.to_be_bytes())
    .chain([0xac]) // 6: ireturn
    .collect();
    let both_body = vec![
        0x2a, // 0: aload_0
        0xb8, //
    ]
    .into_iter()
    .chain(second_accessor.to_be_bytes())
    .chain([0x2a, 0xb8]) // 4: aload_0
    .chain(third_accessor.to_be_bytes())
    .chain([0x60, 0xac]) // 8: iadd, 9: ireturn
    .collect();
    let foreign_body = foreign_member.map(|reference| {
        vec![
            0x2a, // 0: aload_0
            0xb8, //
        ]
        .into_iter()
        .chain(reference.to_be_bytes())
        .chain([0xac]) // 4: ireturn
        .collect::<Vec<u8>>()
    });
    let accessor_member = |name: u16, code: Vec<u8>| Member {
        flags: 0x1008,
        name,
        descriptor: accessor_descriptor,
        max_stack: 1,
        max_locals: 1,
        code,
    };
    let mut members = vec![
        accessor_member(accessor_name, read(fields[0])),
        accessor_member(second_accessor_name, read(fields[1])),
        accessor_member(third_accessor_name, read(fields[2])),
        // Declared with a body and named by no call site: a run that read "the class's members"
        // would read this one, and the usage numbers below would say so.
        accessor_member(fourth_accessor_name, read(fields[1])),
        Member {
            flags: 0x0001,
            name: method_name,
            descriptor: method_descriptor,
            max_stack: 1,
            max_locals: 2,
            code: method_body,
        },
        Member {
            flags: 0x0001,
            name: both_name,
            descriptor: method_descriptor,
            max_stack: 2,
            max_locals: 1,
            code: both_body,
        },
    ];
    if let Some(code) = foreign_body {
        members.push(Member {
            flags: 0x0001,
            name: foreign_name,
            descriptor: method_descriptor,
            max_stack: 1,
            max_locals: 1,
            code,
        });
    }
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
    out.extend_from_slice(&test.to_be_bytes());
    out.extend_from_slice(&object.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes()); // interfaces
    out.extend_from_slice(&3_u16.to_be_bytes()); // three fields
    for name in field_names {
        out.extend_from_slice(&0x0002_u16.to_be_bytes());
        out.extend_from_slice(&name.to_be_bytes());
        out.extend_from_slice(&field_descriptor.to_be_bytes());
        out.extend_from_slice(&0_u16.to_be_bytes());
    }
    out.extend_from_slice(
        &u16::try_from(members.len())
            .expect("a fixture has few members")
            .to_be_bytes(),
    );
    for member in &members {
        out.extend_from_slice(&member.flags.to_be_bytes());
        out.extend_from_slice(&member.name.to_be_bytes());
        out.extend_from_slice(&member.descriptor.to_be_bytes());
        out.extend_from_slice(&1_u16.to_be_bytes()); // one attribute
        out.extend_from_slice(&1_u16.to_be_bytes()); // "Code"
        let mut attribute = Vec::new();
        attribute.extend_from_slice(&member.max_stack.to_be_bytes());
        attribute.extend_from_slice(&member.max_locals.to_be_bytes());
        attribute.extend_from_slice(
            &u32::try_from(member.code.len())
                .expect("a fixture body fits u32")
                .to_be_bytes(),
        );
        attribute.extend_from_slice(&member.code);
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

fn open(class: &[u8]) -> ArtifactSnapshot {
    let mut budget = Budget::new(limits());
    Engine::new()
        .open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS")
}

/// The consumers of one symbol, as the report writes them.
fn consumers(snapshot: &ArtifactSnapshot, target: QueryTarget, kind: ConsumerKind) -> QueryReport {
    let mut kinds = BTreeSet::new();
    kinds.insert(kind);
    let request = QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target,
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: ConsumerSchema { version: 1, kinds },
        max_items: 0,
        cursor: None,
    };
    let mut budget = Budget::new(limits());
    Engine::new()
        .query(snapshot, &request, &mut budget)
        .expect("the query returns a report")
}

fn method_target(name: &str, descriptor: &str) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Method {
            owner: JvmBytes(b"Test".to_vec()),
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
    }
}

fn field_target(name: &str, descriptor: &str) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Field {
            owner: JvmBytes(b"Test".to_vec()),
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
    }
}

/// One report as a wire document, with the one measurement field zeroed.
///
/// `elapsed_millis` is taken again on every call and is not a charge: two runs of the same request
/// cannot agree on it, and comparing it would compare the clock rather than the answer.
fn comparable(report: &QueryReport) -> serde_json::Value {
    let mut document = serde_json::to_value(report).expect("the report is serializable");
    strip_elapsed(&mut document);
    document
}

fn strip_elapsed(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            for (key, field) in fields.iter_mut() {
                if key == "elapsed_millis" {
                    *field = serde_json::Value::from(0);
                } else {
                    strip_elapsed(field);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items.iter_mut() {
                strip_elapsed(item);
            }
        }
        _ => {}
    }
}

/// The member and bytecode index one item's source names, when it is a code location.
fn code_source(item: &XrefItem) -> (&PhysicalMethodId, u32) {
    match &item.source.location {
        Location::Code { method, bci } => (method, *bci),
        other => panic!("the item's source is a code location, got {other:?}"),
    }
}

/// Every item whose source is a **code location**: the edges the bytecode really has.
///
/// A query for a symbol a body references answers with more than that — the raw constant-pool
/// candidates are items of the same answer, and they carry a class offset rather than a member and
/// a bytecode index. The two edges A12 is about are the ones a body's own instruction makes.
fn code_edges(report: &QueryReport) -> Vec<&XrefItem> {
    report
        .items
        .iter()
        .filter(|item| matches!(item.source.location, Location::Code { .. }))
        .collect()
}

/// One recovery run over one member of the fixture, through the entry point the CLI calls.
///
/// The whole answer comes back, together with what the call charged: this is the same public entry
/// the CLI adapter calls, and every part of it — the run's report, the presentation, the members it
/// read for the call sites the body named — describes this one request.
fn recover(
    class: &[u8],
    snapshot: &ArtifactSnapshot,
    name: &[u8],
    descriptor: &[u8],
) -> (RecoveredMethod, UsageSnapshot) {
    let request = request_for(snapshot, class, name, descriptor);
    let mut budget = Budget::new(limits());
    let recovered = Engine::new()
        .recover_method_with_evidence(
            std::slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("the recovery of a member of the fixture runs");
    (recovered, budget.usage())
}

/// One method-analysis request over one member of the fixture, as the CLI-shaped entries build it.
fn request_for(
    snapshot: &ArtifactSnapshot,
    class: &[u8],
    name: &[u8],
    descriptor: &[u8],
) -> MethodAnalysisRequest {
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
    MethodAnalysisRequest {
        environment,
        method: method_id(snapshot, class, name, descriptor),
        stages: AnalysisStage::ALL.to_vec(),
    }
}

/// The physical identity of one member of the fixture's class, as the reader states a definition.
fn method_id(
    snapshot: &ArtifactSnapshot,
    class: &[u8],
    name: &[u8],
    descriptor: &[u8],
) -> PhysicalMethodId {
    PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(class).to_hex().to_string()),
                length: u64::try_from(class.len()).expect("fixture fits u64"),
            },
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
    }
}

/// Every member of the fixture the reader's own header read states a `Code` attribute for.
fn bodies_declared(snapshot: &ArtifactSnapshot) -> Vec<Vec<u8>> {
    let mut budget = Budget::new(limits());
    let inspected = Engine::new()
        .inspect_header(
            snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    inspected
        .inspection
        .header
        .methods
        .iter()
        .filter(|member| {
            member
                .attributes
                .iter()
                .any(|shell| shell.name.raw().0.as_slice() == b"Code")
        })
        .map(|member| member.name.raw().0.clone())
        .collect()
}

/// One node's anchors, as the map's own read surface states them: the bytecode index and the member
/// body of the node's **own** anchor, and of the one it presents.
///
/// The member is a [`PhysicalMethodId`], which is what tells two anchors with one bytecode index
/// apart — and the reason this test never has to name the map's internal types: it asks the map the
/// two questions the map answers.
struct Anchors<'r> {
    own: (u32, Option<&'r PhysicalMethodId>),
    presented: Vec<(u32, Option<&'r PhysicalMethodId>)>,
}

fn anchors_of<'r>(report: &'r RecoveryReport, text: &str) -> Anchors<'r> {
    let node = report
        .source_map
        .segments()
        .iter()
        .find(|segment| segment.text(&report.text) == text)
        .unwrap_or_else(|| panic!("no node's text is `{text}`:\n{}", report.text));
    let own = node.origin().primary();
    Anchors {
        own: (own.bci(), own.method()),
        presented: node
            .origin()
            .derived()
            .iter()
            .map(|anchor| (anchor.bci(), anchor.method()))
            .collect(),
    }
}

#[test]
fn the_two_original_edges_survive_a_recovery_run_field_by_field() {
    let _gate = gate();
    let class = accessor_class();
    let callers = |snapshot: &ArtifactSnapshot| {
        consumers(
            snapshot,
            method_target("access$100", "(LTest;)I"),
            ConsumerKind::Invocation,
        )
    };
    let readers = |snapshot: &ArtifactSnapshot| {
        consumers(snapshot, field_target("f", "I"), ConsumerKind::Field)
    };

    // The run the property is about, and the same fixture without it.
    let plain = open(&class);
    let before_callers = callers(&plain);
    let before_readers = readers(&plain);
    let recovered_snapshot = open(&class);
    let (recovered, _usage) = recover(&class, &recovered_snapshot, b"method", b"()I");
    let after_callers = callers(&recovered_snapshot);
    let after_readers = readers(&recovered_snapshot);

    // X1 answers the same thing either way, field by field (`elapsed_millis` aside).
    assert_eq!(comparable(&before_callers), comparable(&after_callers));
    assert_eq!(comparable(&before_readers), comparable(&after_readers));

    // Each edge is there *on its own*: the caller's call to the accessor, and the accessor's read of
    // the field — named by their own source member and bytecode index, which a merged edge could not
    // satisfy.
    assert_eq!(
        code_edges(&before_callers).len(),
        1,
        "one consumer of the accessor: {:?}",
        before_callers.items
    );
    let (caller, bci) = code_source(code_edges(&before_callers)[0]);
    assert_eq!(&caller.name.0, b"method");
    assert_eq!(bci, CALL_SITE);
    assert_eq!(
        code_edges(&before_readers).len(),
        1,
        "one consumer of the field: {:?}",
        before_readers.items
    );
    let (accessor, bci) = code_source(code_edges(&before_readers)[0]);
    assert_eq!(&accessor.name.0, b"access$100");
    assert_eq!(bci, FIELD_SITE);
    assert!(
        code_edges(&before_callers)[0] != code_edges(&before_readers)[0],
        "the two edges are two items, not one"
    );

    // And the recovery run of *this* entry presents the call as the field access it forwards —
    // A12's presentation half — while X1 above answers exactly what it answered without the run.
    let report = recovered.recovery();
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report.text.contains("return this.f;"),
        "the direct field expression the source had:\n{}",
        report.text
    );
    assert_eq!(report.accessors.len(), 1);
    assert!(report.accessors[0].presented());
    assert!(
        report.fields.is_empty(),
        "the caller has an accessor invocation, not its callee's field instruction"
    );

    // The class's own members were read for the call site the body named — and only for it.
    let callees = recovered.callees().expect("the body named a call site");
    assert_eq!(
        callees
            .members()
            .iter()
            .map(|member| member.identity().name.0.clone())
            .collect::<Vec<_>>(),
        vec![b"access$100".to_vec()],
        "one member, the one the call site named: {:?}",
        callees.members()
    );
}

/// The same entry, on a body whose two call sites reach **two** members whose field access sits at
/// one bytecode index: the anchors say which member each field access is in, and the read says which
/// bodies were paid for.
#[test]
fn the_anchors_name_the_member_each_field_access_is_in_and_only_named_bodies_are_read() {
    let _gate = gate();
    let class = accessor_class();
    let snapshot = open(&class);

    // The premise, from the class's own header: it declares several bodies, so "one body per named
    // member" is a statement about the request and not about the class.
    let declared = bodies_declared(&snapshot);
    assert!(
        declared.len() >= 5 && declared.iter().any(|name| name.as_slice() == b"access$300"),
        "the fixture declares bodies no call site names: {declared:?}"
    );

    let (recovered, usage) = recover(&class, &snapshot, b"both", b"()I");
    let report = recovered.recovery();
    let text = &report.text;
    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(report.representation, Representation::Java, "{report:?}");

    // A12: the text is the direct field expressions the source had, not the calls a compiler made.
    assert!(text.contains("return this.g + this.h;"), "{text}");
    assert!(
        !text.contains("access$200(") && !text.contains("access$300("),
        "the hidden calls are presented as the accesses they forward:\n{text}"
    );

    // The two call sites, each with the member its call named and the definition that member was read
    // from — which is the definition the presented body itself was read from.
    let presented = method_id(&snapshot, &class, b"both", b"()I");
    assert_eq!(report.accessors.len(), 2, "{:?}", report.accessors);
    for (record, name) in report.accessors.iter().zip(["access$200", "access$300"]) {
        assert!(record.presented(), "{record:?}");
        assert_eq!(record.name, name);
        assert_eq!(
            record
                .callee
                .as_ref()
                .map(|identity| identity.name.0.clone()),
            Some(name.as_bytes().to_vec()),
            "the record states the member it read: {record:?}"
        );
        assert_eq!(
            record.callee.as_ref().map(|identity| &identity.owner),
            Some(&presented.owner),
            "and it is a member of the very definition the presented body was read from"
        );
    }

    // The anchors. Two nodes, each with the call site's own BCI as its own anchor and the field
    // access **inside the callee's body** as one it presents — where both callees put that access at
    // BCI 1, the same index the call site itself is at.
    let field = anchors_of(report, "this.g");
    let second = anchors_of(report, "this.h");
    assert_eq!(
        field.own,
        (FIELD_SITE, Some(&presented)),
        "the node's own anchor is this body's call site"
    );
    for (node, name) in [(&field, "access$200"), (&second, "access$300")] {
        assert_eq!(
            node.presented.len(),
            1,
            "the field access is the second original anchor"
        );
        assert_eq!(node.presented[0].0, FIELD_SITE, "at BCI 1");
        assert_eq!(
            node.presented[0]
                .1
                .map(|identity| (identity.name.0.clone(), identity.owner.clone())),
            Some((name.as_bytes().to_vec(), presented.owner.clone())),
            "and it says which member's body that BCI is in, in which definition"
        );
    }
    assert_ne!(
        field.presented, second.presented,
        "two callees whose field access sits at one BCI are two anchors"
    );

    // Direct and Derived did not move: at one bytecode index this body's own call site is the node's
    // own anchor, and a callee's field access is what it presents.
    assert_eq!(
        report.source_map.of_bci(FIELD_SITE).len(),
        2,
        "both field accesses are mentioned at BCI 1: {:?}",
        report.source_map.segments()
    );
    let direct = report.source_map.direct_of_bci(FIELD_SITE);
    assert_eq!(
        direct.len(),
        1,
        "the call site at BCI 1 is one node's own anchor"
    );
    assert_eq!(direct[0].text(&report.text), "this.g");
    let presented_here = report.source_map.derived_of_bci(FIELD_SITE);
    assert_eq!(
        presented_here.len(),
        1,
        "and the other callee's field access at BCI 1 is what one node presents there"
    );
    assert_eq!(presented_here[0].text(&report.text), "this.h");

    // The read: one class, and exactly the two members the call sites named — read from the
    // definition the presented body came from, with the reason the read states.
    let callees = recovered.callees().expect("the body named two call sites");
    assert_eq!(callees.class(), "Test");
    assert_eq!(
        callees
            .members()
            .iter()
            .map(|member| member.identity().name.0.clone())
            .collect::<Vec<_>>(),
        vec![b"access$200".to_vec(), b"access$300".to_vec()],
        "the members the class declares among the candidates, in candidate order"
    );
    assert!(callees.refusals().is_empty(), "{:?}", callees.refusals());
    assert_eq!(
        callees
            .reads()
            .iter()
            .map(|read| (read.reason, read.definition.clone()))
            .collect::<Vec<_>>(),
        vec![(
            ReadReason::CalleeMemberBody,
            method_id(&snapshot, &class, b"both", b"()I").owner
        )],
        "one read, of the presented body's own definition, for the named callees"
    );
    for member in callees.members() {
        assert!(member.body().is_some(), "{member:?}");
        assert_eq!(
            member.identity().owner,
            presented.owner,
            "every member states the definition it was read from"
        );
    }

    // The numbers: the presented body and the two members its call sites named — and not the third
    // synthetic member the class declares, which no call site named (A16).
    assert_eq!(
        usage.method_bodies, 3,
        "one body per named member, not one per declared member: {declared:?}"
    );
    // One class header for the whole request (D2 3.3): the run read the presented body's own
    // definition, and the members it declares were read from the *preparation* built over that
    // same read — the payload carries a body's tables, not its bytes, and the class is therefore
    // read once and prepared once.
    assert_eq!(
        usage.class_headers, 1,
        "the presented body's own definition, prepared once for the members it declares"
    );
    assert_eq!(
        recovered.analysis().reads.len(),
        1,
        "the analysis run's own reads are still its driver read: {:?}",
        recovered.analysis().reads
    );
}

/// `Test.sub(I)I` — a body whose one subroutine is entered **twice**, which is the shape P2's
/// normalization clones:
///
/// ```text
/// 0:  iconst_0      4:  istore_1     8:  istore_1     14: astore_2
/// 1:  istore_1      5:  iload_1      9:  iload_1      15: iinc 1, 1
/// 2:  iload_1       6:  jsr 14       10: jsr 14       18: ret 2
/// 3:  jsr 14        7:  (end)        11: istore_1
///                                    12: iload_1
///                                    13: ireturn
/// ```
///
/// The class file is version 45, the last dialect that may spell a subroutine with `jsr`/`ret`.
fn clone_class() -> Vec<u8> {
    let mut pool = Pool::default();
    pool.utf8("Code");
    let test_name = pool.utf8("Test");
    let test = pool.class(test_name);
    let object_name = pool.utf8("java/lang/Object");
    let object = pool.class(object_name);
    let subject_name = pool.utf8("sub");
    let subject_descriptor = pool.utf8("(I)I");
    let mut code = vec![
        0x03, // 0: iconst_0
        0x3c, // 1: istore_1
        0x1b, // 2: iload_1
        0xa8, 0x00, 0x0b, // 3: jsr 14
        0x3c, // 6: istore_1
        0x1b, // 7: iload_1
        0xa8, 0x00, 0x06, // 8: jsr 14
        0x3c, // 11: istore_1
        0x1b, // 12: iload_1
        0xac, // 13: ireturn
        0x4d, // 14: astore_2
        0x84, 0x01, 0x01, // 15: iinc 1, 1
        0xa9, 0x02, // 18: ret 2
    ];
    debug_assert_eq!(code.len(), 20);
    let mut out = Vec::new();
    out.extend_from_slice(&0xcafebabe_u32.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes());
    out.extend_from_slice(&45_u16.to_be_bytes());
    out.extend_from_slice(
        &u16::try_from(pool.entries.len() + 1)
            .expect("a fixture pool fits u16")
            .to_be_bytes(),
    );
    for entry in &pool.entries {
        out.extend_from_slice(entry);
    }
    out.extend_from_slice(&0x0021_u16.to_be_bytes());
    out.extend_from_slice(&test.to_be_bytes());
    out.extend_from_slice(&object.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes()); // interfaces
    out.extend_from_slice(&0_u16.to_be_bytes()); // fields
    out.extend_from_slice(&1_u16.to_be_bytes()); // one member
    out.extend_from_slice(&0x0008_u16.to_be_bytes()); // static
    out.extend_from_slice(&subject_name.to_be_bytes());
    out.extend_from_slice(&subject_descriptor.to_be_bytes());
    out.extend_from_slice(&1_u16.to_be_bytes()); // one attribute
    out.extend_from_slice(&1_u16.to_be_bytes()); // "Code"
    let mut attribute = Vec::new();
    attribute.extend_from_slice(&1_u16.to_be_bytes()); // max_stack
    attribute.extend_from_slice(&3_u16.to_be_bytes()); // max_locals
    attribute.extend_from_slice(
        &u32::try_from(code.len())
            .expect("a fixture body fits u32")
            .to_be_bytes(),
    );
    attribute.append(&mut code);
    attribute.extend_from_slice(&0_u16.to_be_bytes()); // no exception table
    attribute.extend_from_slice(&0_u16.to_be_bytes()); // no nested attributes
    out.extend_from_slice(
        &u32::try_from(attribute.len())
            .expect("a fixture attribute fits u32")
            .to_be_bytes(),
    );
    out.extend_from_slice(&attribute);
    out.extend_from_slice(&0_u16.to_be_bytes()); // no class attributes
    out
}

/// A call site that names **another class**'s member is not read from this class's bytes (P3 3.2).
///
/// The fixture declares a member with the very name and descriptor the call names, so an
/// implementation that accepted "same name and descriptor is the same member" would read *this*
/// class's body and present a call to `Other` as a field access of `Test`. It does not: the read
/// refuses the candidate with both names stated, and the call keeps the call it had.
#[test]
fn a_call_to_another_classs_same_named_member_is_not_read_from_this_class() {
    let _gate = gate();
    let class = foreign_call_class();
    let snapshot = open(&class);
    let (recovered, usage) = recover(&class, &snapshot, b"foreign", b"()I");
    let report = recovered.recovery();
    assert!(report.produced(), "{:?}", report.outcome);

    // The read happened — one header, of the presented body's own definition — and answered the
    // candidate as what it is: a member of a class these bytes are not.
    let callees = recovered.callees().expect("the body named a call site");
    assert_eq!(callees.class(), "Test");
    assert!(callees.members().is_empty(), "{:?}", callees.members());
    assert_eq!(callees.refusals().len(), 1, "{:?}", callees.refusals());
    let refusal = &callees.refusals()[0];
    assert_eq!(refusal.code(), "callee_not_this_class");
    assert_eq!(refusal.candidate().call_site(), 1);
    assert!(
        refusal.message().contains("Other.access$100")
            && refusal.message().contains("declares `Test`"),
        "{}",
        refusal.message()
    );
    assert_eq!(
        usage.method_bodies, 1,
        "no body of this class was read for it — the presented body is the one attempt"
    );
    // The refusal is decided from the prepared class (D2 3.3), which is the request's one read of
    // this definition: no second header read happens for a call site that names another class.
    assert_eq!(
        usage.class_headers, 1,
        "the read is a header read of the same definition"
    );

    // And the rule refuses the site on the owner it names: the call is written as the call it was.
    assert_eq!(report.accessors.len(), 1);
    let accessor = &report.accessors[0];
    assert!(!accessor.presented(), "{accessor:?}");
    assert_eq!(
        accessor.callee, None,
        "nothing was read, so nothing is stated"
    );
    assert_eq!(
        accessor.refusal.as_ref().map(|refusal| refusal.code),
        Some("jre_accessor_not_a_member")
    );
    assert!(
        report.text.contains("access$100(this)"),
        "the call to another class keeps the call it had:\n{}",
        report.text
    );
    assert!(
        !report.text.contains(".f"),
        "and this class's own member of that name is not presented as its callee:\n{}",
        report.text
    );
}

/// A bytecode index is a coordinate *in* a member body, so one original BCI may be named by more than
/// one node and still say which member it is in — the shape a canonical clone produces.
#[test]
fn a_bytecode_index_a_clone_repeats_still_names_the_member_it_is_in() {
    let _gate = gate();
    let class = clone_class();
    let snapshot = open(&class);
    let (recovered, usage) = recover(&class, &snapshot, b"sub", b"(I)I");
    let report = recovered.recovery();
    assert!(report.produced(), "{:?}", report.outcome);

    // The premise is this very run's own counter: the body's subroutine is entered twice, and the
    // canonical pass cloned it once per entrance — so the artifact below is one whose graph really
    // holds clones, not one this test asserts to be.
    assert!(
        usage.normalization_clones >= 1,
        "the `jsr` subroutine is entered twice and the run cloned it: {usage:?}"
    );

    // Every anchor of this artifact is an anchor **of that member**: the run's own declaration states
    // which member the body is, and no anchor invents a second one — the anchor keeps the original
    // bytecode index, never the clone's own identity.
    let owner = method_id(&snapshot, &class, b"sub", b"(I)I");
    let coordinates: Vec<(u32, Option<PhysicalMethodId>)> = report
        .source_map
        .segments()
        .iter()
        .flat_map(|segment| {
            std::iter::once(segment.origin().primary()).chain(segment.origin().derived().iter())
        })
        .map(|anchor| (anchor.bci(), anchor.method().cloned()))
        .collect();
    assert!(!coordinates.is_empty(), "{}", report.text);
    assert!(
        coordinates
            .iter()
            .all(|(_, method)| method.as_ref() == Some(&owner)),
        "one body, one member: {} anchors of {:?}\n{}",
        coordinates.len(),
        owner.name.0,
        report.text
    );

    // And the mapping is by **coordinate**, not by mention: the fallback quote names the body's
    // instruction starts once each — a quote is the set of coordinates this text was produced
    // from, and a clone is the same coordinate reached twice, not a second place — while the run
    // still holds the clones (`normalization_clones` above). Every bytecode the quotes name is
    // anchored, one anchor per quoted coordinate, and no anchor names a bytecode no quote
    // accounts for.
    let quoted = quoted_bcis(&report.text);
    let anchors: Vec<u32> = coordinates.iter().map(|(bci, _)| *bci).collect();
    assert_eq!(
        quoted,
        distinct_quoted(&quoted),
        "the fallback quote names each coordinate once, whatever the clones: {quoted:?}"
    );
    assert_eq!(
        distinct_quoted(&anchors),
        distinct_quoted(&quoted),
        "the anchors are exactly the coordinates the quotes name, one per place: {anchors:?} vs \
         {quoted:?}\n{}",
        report.text
    );
    assert_eq!(
        coordinates.len(),
        quoted.len(),
        "one anchor per quoted coordinate: {} anchors, {} quoted mentions",
        coordinates.len(),
        quoted.len()
    );
}

/// The distinct values of one list, in order.
fn distinct_quoted(values: &[u32]) -> Vec<u32> {
    let mut distinct = values.to_vec();
    distinct.sort_unstable();
    distinct.dedup();
    distinct
}

/// Every bytecode index the artifact's own quotes name, in the order each quote states them.
fn quoted_bcis(text: &str) -> Vec<u32> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
        .flat_map(|bcis| {
            bcis.split_whitespace()
                .map(|bci| bci.parse::<u32>().expect("a quoted BCI is a number"))
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// D2 3.3: one read, one preparation, for the presented body and the callees it named
// ---------------------------------------------------------------------------------------------

/// D2 3.3's count gate: the driver run and the same-class callee read share **one** materialization
/// of the presented body's class and **one** preparation over it.
///
/// The fixture is the one above: a body whose two call sites name two members of its own class, so
/// the request really has a same-class callee read to pay for. At the D0 revision that read was a
/// second class read of the same definition (`class_headers == 2`, and the class was materialized
/// twice); D2 hands the run's own read to the preparation, so the definition is read once — and the
/// preparation is the one figure that shows the callee read consumed it instead of reading again.
///
/// The port is test-support only, and this file is a plain target: the case is compiled only in the
/// build that has the port.
#[cfg(feature = "test-support")]
#[test]
fn the_driver_and_its_same_class_callees_share_one_read_and_one_preparation() {
    use jarde::d0_counts;

    let _gate = gate();
    let class = accessor_class_with(false);
    let snapshot = open(&class);
    let request = request_for(&snapshot, &class, b"both", b"()I");
    let mut budget = Budget::new(limits());
    let before = d0_counts::snapshot();
    let recovered = Engine::new()
        .recover_method_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("the recovery of the fixture's member runs");
    let counted = before.since(d0_counts::snapshot());
    let usage = budget.usage();
    println!(
        "recover_method (same-class callees): materializations={} preparations={} body_decodes={} \
         | class_headers={} method_bodies={}",
        counted.class_materializations,
        counted.class_preparations,
        counted.body_decodes,
        usage.class_headers,
        usage.method_bodies
    );

    // The callee evidence is really there: the members the call sites named were read, from the
    // definition the presented body was read from.
    let callees = recovered.callees().expect("the body named two call sites");
    assert_eq!(
        callees
            .members()
            .iter()
            .map(|member| member.identity().name.0.clone())
            .collect::<Vec<_>>(),
        vec![b"access$200".to_vec(), b"access$300".to_vec()],
        "the members the call sites named, in candidate order: {:?}",
        callees.members()
    );
    assert_eq!(
        counted.class_materializations, 1,
        "the presented body's own definition is materialized once, for the whole request"
    );
    assert_eq!(
        counted.class_preparations, 1,
        "and one preparation over that read serves the presented body and its callees"
    );
    assert_eq!(
        counted.body_decodes, 1,
        "the *driver's* own decode is the one this facade counts; the two callee bodies are decoded \
         by the callee read itself, and their number is what `method_bodies` states"
    );
    assert_eq!(
        usage.class_headers, 1,
        "no member's read charged a class header of its own: {usage:?}"
    );
    assert_eq!(
        usage.method_bodies, 3,
        "one body attempt per decoded body: {usage:?}"
    );
    assert_eq!(
        recovered
            .analysis()
            .reads
            .iter()
            .filter(|read| read.reason == ReadReason::DriverMethodBody)
            .count(),
        1,
        "the loader binding check still ran, once, over the request's own read: {:?}",
        recovered.analysis().reads
    );
}
