//! P3 2.2 acceptance A12: presenting a synthetic accessor as a field access is **derived**, and the
//! two original X1 edges survive it.
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
//! 3. the recovery run itself does not present the call as a field access *here*: the library entry
//!    states no member table, so the site is refused with the table it is missing named
//!    (`jre_accessor_members_missing`) and the call it had is written. A12's presentation half is
//!    checked where the evidence is (`crates/jarde-java/tests/p3_patterns.rs`); what this file
//!    checks is that neither outcome touches X1.
//!
//! The fixture is assembled here rather than committed, because no historical corpus in this
//! repository holds a compiler-generated accessor: the class declares `f:I` (private), the static
//! synthetic `access$100(LTest;)I` that reads it, and `method()I` that calls the accessor.

use jarde::*;
use std::collections::BTreeSet;

const CALL_SITE: u32 = 3;
const FIELD_SITE: u32 = 1;

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

/// `Test` with the field, the accessor a compiler generated for it, and the member that calls it.
///
/// ```text
/// f:I                        private
/// access$100(LTest;)I        public static synthetic: aload_0; getfield Test.f:I; ireturn
/// method()I                  public: iconst_0; istore_1; aload_0;
///                                    invokestatic Test.access$100(LTest;)I; ireturn
/// ```
fn accessor_class() -> Vec<u8> {
    let mut pool = Pool::default();
    pool.utf8("Code");
    let test_name = pool.utf8("Test");
    let test = pool.class(test_name);
    let object_name = pool.utf8("java/lang/Object");
    let object = pool.class(object_name);
    let field_name = pool.utf8("f");
    let field_descriptor = pool.utf8("I");
    let field_and_type = pool.name_and_type(field_name, field_descriptor);
    let field = pool.field_ref(test, field_and_type);
    let accessor_name = pool.utf8("access$100");
    let accessor_descriptor = pool.utf8("(LTest;)I");
    let accessor = member_ref(&mut pool, test, "access$100", "(LTest;)I");
    let method_name = pool.utf8("method");
    let method_descriptor = pool.utf8("()I");
    let accessor_body = vec![
        0x2a, // 0: aload_0
        0xb4, //
    ]
    .into_iter()
    .chain(field.to_be_bytes())
    .chain([0xac]) // 4: ireturn
    .collect();
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
    let members = vec![
        Member {
            flags: 0x1008,
            name: accessor_name,
            descriptor: accessor_descriptor,
            max_stack: 1,
            max_locals: 1,
            code: accessor_body,
        },
        Member {
            flags: 0x0001,
            name: method_name,
            descriptor: method_descriptor,
            max_stack: 1,
            max_locals: 2,
            code: method_body,
        },
    ];
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
    out.extend_from_slice(&1_u16.to_be_bytes()); // one field
    out.extend_from_slice(&0x0002_u16.to_be_bytes());
    out.extend_from_slice(&field_name.to_be_bytes());
    out.extend_from_slice(&field_descriptor.to_be_bytes());
    out.extend_from_slice(&0_u16.to_be_bytes());
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

/// One recovery run over the fixture's `method`, through the entry point the CLI calls.
fn recover_the_caller(snapshot: &ArtifactSnapshot) -> RecoveryReport {
    let class = accessor_class();
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
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: ClassBytesId {
                    digest: Digest(blake3::hash(&class).to_hex().to_string()),
                    length: u64::try_from(class.len()).expect("fixture fits u64"),
                },
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(b"method".to_vec()),
            descriptor: JvmBytes(b"()I".to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    let mut budget = Budget::new(limits());
    let recovered = Engine::new()
        .recover_method(std::slice::from_ref(snapshot), &request, &mut budget)
        .expect("the recovery of the fixture's caller runs");
    recovered.recovery().clone()
}

#[test]
fn the_two_original_edges_survive_a_recovery_run_field_by_field() {
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
    let report = recover_the_caller(&recovered_snapshot);
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

    // The recovery run of this entry states no member table, so the call is *refused* and keeps the
    // call it had — and even that presentation does not touch X1 above.
    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(report.accessors.len(), 1);
    assert!(!report.accessors[0].presented());
    assert_eq!(
        report.accessors[0]
            .refusal
            .as_ref()
            .expect("the refusal is recorded")
            .code,
        "jre_accessor_members_missing"
    );
    assert!(
        report.text.contains("access$100("),
        "the call the library entry could not decide is still called:\n{}",
        report.text
    );
}
