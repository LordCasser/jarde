//! P3 acceptance for the `recover-preceded-statement-catches` slice: a named `catch` whose
//! protected range follows a statement that is **no store** presents as the ordinary `try`/`catch`
//! its own exception table names, and `jre_concat_split` names the `toString` the candidate chain's
//! own builder value reaches — not the first one the block order lists.
//!
//! The fixtures are the slice's frozen evidence
//! (`openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/`): the M1/M2/M3/M5 family
//! and the N1/N2b negatives as they were frozen, plus the `fixture/near/` verifier-valid neighbors
//! the change's tasks constructed (a `putfield`/`putstatic` prefix, a `getstatic` consumer
//! statement, a second store-prefix degradation, a late split chain behind an earlier `toString`,
//! and a two-resource `try`-with-resources). The original `FinallyOnce.class` is the debt this
//! slice closes.
//!
//! What this file has to prove through the public surface:
//!
//! 1. `helper(); try { … } catch (E e) { … }` — and the same shape behind a field write or a
//!    `getstatic`-consuming call — recovers completely, the prefix statement staying where the
//!    bytecode ran it;
//! 2. a store prefix (`r = open(); try … catch (E e)`) keeps degrading under the resource rules —
//!    the refusal stays, and the shape is never spelled as a user `catch`;
//! 3. a concatenation whose builder a branch cuts is refused with `jre_concat_split` naming the
//!    `toString` the chain's own value reaches, while an earlier same-owner `toString` in the
//!    method and a same-block chain stay out of the refusal's way;
//! 4. a multi-resource `try`-with-resources header keeps its certificate;
//! 5. the M1/M2 presentations are unchanged verbatim, and a stopped run publishes no partial text.

use jarde_java::{
    ClassMembers, DeclaringClass, MemberBody, MethodFacts, RecoveryFacts, RecoveryRequest,
    StopReason, recover,
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

/// The slice's frozen family, as the evidence directory holds it.
const M1: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/M1.class"
);
const M2: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/M2.class"
);
const M3: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/M3.class"
);
const M5: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/M5.class"
);
const N1: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/N1.class"
);
const N2B: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/N2b.class"
);
const P1_FIELD_WRITE: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/near/P1FieldWrite.class"
);
const P2_GETSTATIC_READ: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/near/P2GetstaticRead.class"
);
const P3_STORE_PREFIX: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/near/P3StorePrefix.class"
);
const P4_LATE_SPLIT: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/near/P4LateSplit.class"
);
const P5_TWO_RESOURCES: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/near/P5TwoResources.class"
);
/// The registered debt this slice closes: the whole class, not one method of it.
const FINALLY_ONCE: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-27/cf16-finally/original/FinallyOnce.class"
);

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
    let analysis = analyze_method_ir(
        &[snapshot],
        &MethodAnalysisRequest {
            environment,
            method,
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("the fixture method analyzes");
    Payload { analysis }
}

fn facts_of(class: &[u8], name: &[u8], parameters: u16) -> RecoveryFacts {
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
}

fn members_of(class: &[u8]) -> ClassMembers {
    let mut budget = Budget::new(limits());
    let header = class_facts(class, &mut budget).expect("the fixture is a class file");
    let owner = String::from_utf8_lossy(&header.this_class.raw().0).into_owned();
    let definition = {
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
    };
    let members = header
        .methods
        .iter()
        .map(|member| {
            let identity = PhysicalMethodId {
                owner: definition.clone(),
                name: member.name.raw().clone(),
                descriptor: member.descriptor.raw().clone(),
            };
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

/// Presents one fixture body under the full budget.
fn present(
    class: &[u8],
    name: &[u8],
    descriptor: &[u8],
    parameters: u16,
) -> jarde_java::RecoveryReport {
    let payload = analyze(class, name, descriptor);
    let facts = facts_of(class, name, parameters);
    let members = members_of(class);
    let mut budget = Budget::new(limits());
    let request = RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8)
        .with_members(&members)
        .with_evidence(jarde_java::RecoveryEvidenceRequest::all());
    recover(&request, &mut budget)
}

/// A `void` call in front of the protected range is no resource's initialisation: the row is the
/// `catch` its own table names, and the whole `main` presents — `helper()` first, then the one
/// `try`/`catch`, nothing refused.
#[test]
fn a_void_call_statement_before_a_named_catch_presents_the_catch() {
    let report = present(M5, b"main", b"([Ljava/lang/String;)V", 1);
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert_eq!(report.quality, Quality::Structured);
    assert!(
        !report.text.contains("@bytecode"),
        "every statement presented: {}",
        report.text
    );
    // The prefix statement stays where the bytecode ran it, before the statement's own `try`.
    let helper = report.text.find("helper();").expect("the prefix statement");
    let try_at = report.text.find("try {").expect("the try");
    let catch_at = report
        .text
        .find("catch (java.lang.IllegalStateException")
        .expect("the named clause");
    assert!(helper < try_at && try_at < catch_at, "{}", report.text);
    assert!(report.text.contains("t();"), "{}", report.text);
    assert!(
        report.text.contains("u(local1);") || report.text.contains("u(error);"),
        "{}",
        report.text
    );
    assert!(
        report
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "jre_guard_resource_init"),
        "the guard resource proof is out of the way: {:?}",
        report.diagnostics
    );
}

/// The same answer for a statement that consumes a `getstatic`: `System.out.println("before");`
/// ends in a call, not in a store, so the row behind it is a `catch`, not a resource header.
#[test]
fn a_getstatic_statement_before_a_named_catch_presents_the_catch() {
    let report = present(P2_GETSTATIC_READ, b"run", b"()Ljava/lang/String;", 0);
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert!(
        !report.text.contains("@bytecode"),
        "every statement presented: {}",
        report.text
    );
    let before = report
        .text
        .find("java.lang.System.out.println(\"before\");")
        .expect("the prefix statement");
    let try_at = report.text.find("try {").expect("the try");
    assert!(before < try_at, "{}", report.text);
    assert!(
        report
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "jre_guard_resource_init"),
        "{:?}",
        report.diagnostics
    );
}

/// A field write in front of the range was this family's other non-store statement: it stays
/// presented, and the write runs once, before the `try`.
#[test]
fn field_write_statements_before_a_named_catch_stay_presented() {
    // An instance method's slot count names `this` as well: the header's own slot 0.
    let putfield = present(
        P1_FIELD_WRITE,
        b"putfieldPrefix",
        b"()Ljava/lang/String;",
        1,
    );
    assert!(putfield.produced(), "{:?}", putfield.stop());
    assert_eq!(putfield.representation, Representation::Java);
    assert!(!putfield.text.contains("@bytecode"), "{}", putfield.text);
    let write = putfield.text.find("instance = 3;").expect("the putfield");
    let try_at = putfield.text.find("try {").expect("the try");
    assert!(write < try_at, "{}", putfield.text);

    let putstatic = present(P1_FIELD_WRITE, b"putstaticPrefix", b"()I", 0);
    assert!(putstatic.produced(), "{:?}", putstatic.stop());
    assert_eq!(putstatic.representation, Representation::Java);
    assert!(!putstatic.text.contains("@bytecode"), "{}", putstatic.text);
    let write = putstatic.text.find("marker = 7;").expect("the putstatic");
    let try_at = putstatic.text.find("try {").expect("the try");
    assert!(write < try_at, "{}", putstatic.text);
}

/// The registered debt: the original `FinallyOnce.main` — a void call, two entry-block chains and a
/// third chain inside the handler — presents whole, with every chain presented and no fallback.
#[test]
fn the_original_finally_once_main_recovers_with_its_three_chains() {
    let report = present(FINALLY_ONCE, b"main", b"([Ljava/lang/String;)V", 1);
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert!(
        !report.text.contains("@bytecode"),
        "every statement presented: {}",
        report.text
    );
    // The named row's range opens after the second `println` — the void statement this slice's
    // candidate gate answers *no* for — and the whole main presents around it.
    let prefix = report
        .text
        .rfind("println(handled(true)")
        .expect("the statement before the range");
    let try_at = report.text.find("try {").expect("the try");
    assert!(prefix < try_at, "{}", report.text);
    let presented = report
        .concats
        .iter()
        .filter(|chain| chain.presented())
        .count();
    assert_eq!(
        presented, 3,
        "all three chains present: {:?}",
        report.concats
    );
    assert!(report.concats.iter().all(|chain| chain.refusal.is_none()));
}

/// The M3 main is the whole shape in one method: two entry-block chains, the void call, and the
/// handler-block chain behind the named catch.
#[test]
fn the_whole_m3_main_recovers() {
    let report = present(M3, b"main", b"([Ljava/lang/String;)V", 1);
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    let presented = report
        .concats
        .iter()
        .filter(|chain| chain.presented())
        .count();
    assert_eq!(presented, 3, "{:?}", report.concats);
    let prefix = report
        .text
        .rfind("println(handled(true)")
        .expect("the statement before the range");
    let try_at = report.text.find("try {").expect("the try");
    assert!(prefix < try_at, "{}", report.text);
}

/// M1 and M2 were accepted before this slice; their presentations are pinned verbatim so the
/// candidate-gate and split-attribution changes cannot move a character of them.
#[test]
fn m1_and_m2_presentations_are_unchanged_verbatim() {
    let handled = present(M1, b"handled", b"(Z)Ljava/lang/String;", 1);
    assert!(handled.produced(), "{:?}", handled.stop());
    assert_eq!(handled.text, M1_HANDLED_VERBATIM);

    let main = present(M1, b"main", b"([Ljava/lang/String;)V", 1);
    assert!(main.produced(), "{:?}", main.stop());
    assert_eq!(main.text, M1_MAIN_VERBATIM);

    let escaping = present(M2, b"escaping", b"()V", 0);
    assert!(escaping.produced(), "{:?}", escaping.stop());
    assert_eq!(escaping.text, M2_ESCAPING_VERBATIM);

    let main = present(M2, b"main", b"([Ljava/lang/String;)V", 1);
    assert!(main.produced(), "{:?}", main.stop());
    assert_eq!(main.text, M2_MAIN_VERBATIM);
}

/// A store prefix keeps the resource rules' own answer: the row is examined as a header, the proof
/// degrades, and the shape is never spelled as a user `catch` — for the frozen N1 and for the
/// second store-prefix neighbor.
#[test]
fn a_store_prefix_still_degrades_to_the_resource_refusal() {
    for (name, class) in [("N1", N1), ("P3StorePrefix", P3_STORE_PREFIX)] {
        let report = present(class, b"main", b"([Ljava/lang/String;)V", 1);
        assert!(
            !report.text.contains("try {"),
            "{name} must not present a user catch: {}",
            report.text
        );
        assert!(
            report.text.contains("@bytecode"),
            "{name} keeps its fallback: {}",
            report.text
        );
        assert_eq!(report.quality, Quality::Fallback, "{name}");
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "jre_guard_handler"),
            "{name} keeps the resource degradation refusal: {:?}",
            report.diagnostics
        );
    }
}

/// The attribution: a true split chain that starts *after* an earlier same-owner `toString` names
/// **its own** consumer — BCI 68, the `toString` its builder reaches — and never the earlier one at
/// BCI 18. The earlier chain itself presents.
#[test]
fn a_split_refusal_names_the_to_string_the_chains_own_builder_reaches() {
    let report = present(P4_LATE_SPLIT, b"g", b"(Z)Ljava/lang/String;", 1);
    assert!(report.produced(), "{:?}", report.stop());
    let early = report
        .concats
        .iter()
        .find(|chain| chain.head == 0)
        .expect("the early chain is a candidate");
    assert!(
        early.presented(),
        "the early chain presents: {:?}",
        report.concats
    );
    let late = report
        .concats
        .iter()
        .find(|chain| chain.head == 33)
        .expect("the late chain is a candidate");
    let refusal = late.refusal.as_ref().expect("the late chain is refused");
    assert_eq!(refusal.code, "jre_concat_split");
    assert!(
        refusal.message.contains("BCI 68"),
        "the refusal names the chain's own consumer: {}",
        refusal.message
    );
    assert!(
        !refusal.message.contains("BCI 18"),
        "the earlier toString stays out of the attribution: {}",
        refusal.message
    );
    assert!(
        refusal.message.contains("another block"),
        "{}",
        refusal.message
    );
}

/// The frozen negative keeps its own refusal at its own `toString`: the hand-written cross-block
/// builder is refused with `jre_concat_split` naming BCI 32.
#[test]
fn a_cross_block_builder_keeps_its_split_refusal_at_its_own_to_string() {
    let report = present(N2B, b"g", b"(Z)Ljava/lang/String;", 1);
    assert!(report.produced(), "{:?}", report.stop());
    let chain = &report.concats[0];
    assert_eq!(chain.head, 0);
    assert!(!chain.presented());
    let refusal = chain.refusal.as_ref().expect("the chain is refused");
    assert_eq!(refusal.code, "jre_concat_split");
    assert!(
        refusal.message.contains("BCI 32"),
        "the refusal names the real consumption point: {}",
        refusal.message
    );
    assert!(
        refusal.message.contains("another block"),
        "{}",
        refusal.message
    );
}

/// A provable two-resource header keeps its certificate: the candidate gate's new answer is about
/// statements that are no store, and a resource's own store is always the range's neighbour.
#[test]
fn a_multi_resource_twr_header_keeps_its_certificate() {
    let report = present(P5_TWO_RESOURCES, b"two", b"()Ljava/lang/String;", 0);
    assert!(report.produced(), "{:?}", report.stop());
    assert_eq!(report.representation, Representation::Java);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    let try_at = report.text.find("try (").expect("the resources header");
    let first = report
        .text
        .find("java.io.ByteArrayOutputStream local0 = open()")
        .expect("resource a");
    let second = report
        .text
        .find("java.io.ByteArrayOutputStream local1 = open()")
        .expect("resource b");
    assert!(try_at < first && first < second, "{}", report.text);
}

/// A stopped run publishes nothing: no partial text under a refused budget, none under a cancelled
/// token, for a method whose recovery this slice changes.
#[test]
fn m3_recovery_stops_atomically_under_budget_and_cancellation() {
    let payload = analyze(M3, b"main", b"([Ljava/lang/String;)V");
    let facts = facts_of(M3, b"main", 1);
    let members = members_of(M3);
    let mut limited_budget = Budget::new(Limits {
        ir_items: 1,
        ..limits()
    });
    let request = RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8)
        .with_members(&members);
    let limited = recover(&request, &mut limited_budget);
    assert!(!limited.produced(), "{:?}", limited.outcome);
    assert!(limited.text.is_empty(), "{}", limited.text);
    assert!(limited.source_map.is_empty());

    let token = jarde_reader::budget::CancellationToken::new();
    token.cancel();
    let mut cancelled_budget = Budget::with_cancellation_token(limits(), token);
    let request = RecoveryRequest::new(payload.analysis.ir(), &facts, jarde_java::pass::JAVA_8)
        .with_members(&members);
    let cancelled = recover(&request, &mut cancelled_budget);
    assert!(!cancelled.produced(), "{:?}", cancelled.outcome);
    assert!(matches!(
        cancelled.stop(),
        Some(StopReason::Cancelled { .. })
    ));
    assert!(cancelled.text.is_empty());
    assert!(cancelled.source_map.is_empty());
}

/// The M1 `handled` presentation, pinned verbatim as this slice's baseline leaves it.
const M1_HANDLED_VERBATIM: &str = r#"// @method handled(Z)Ljava/lang/String;
// @declaration a static method of `M1`, member flags 0x0009
// recovered from bytecode; presentation is not claimed to compile
{
    M1.cleanupCount = 0;
    try {
        if (arg0) {
            throw new java.lang.IllegalArgumentException("arg");
        } else {
            M1.cleanupCount = M1.cleanupCount + 1;
            return "normal";
        }
    } catch (java.lang.IllegalArgumentException local1) {
        M1.cleanupCount = M1.cleanupCount + 1;
        return "caught:" + local1.getMessage();
    } finally {
        M1.cleanupCount = M1.cleanupCount + 1;
    }
}
"#;

/// The M1 `main` presentation, pinned verbatim as this slice's baseline leaves it.
const M1_MAIN_VERBATIM: &str = r#"// @method main([Ljava/lang/String;)V
// @declaration a static method of `M1`, member flags 0x0009
// recovered from bytecode; presentation is not claimed to compile
{
    java.lang.System.out.println(handled(false) + ":" + count());
    java.lang.System.out.println(handled(true) + ":" + count());
    return;
}
"#;

/// The M2 `escaping` presentation, pinned verbatim as this slice's baseline leaves it.
const M2_ESCAPING_VERBATIM: &str = r#"// @method escaping()V
// @declaration a static method of `M2`, member flags 0x0009
// recovered from bytecode; presentation is not claimed to compile
{
    M2.cleanupCount = 0;
    try {
        throw new java.lang.IllegalStateException("state");
    } catch (java.lang.Throwable local0) {
        M2.cleanupCount = M2.cleanupCount + 1;
        throw local0;
    }
}
"#;

/// The M2 `main` presentation, pinned verbatim as this slice's baseline leaves it.
const M2_MAIN_VERBATIM: &str = r#"// @method main([Ljava/lang/String;)V
// @declaration a static method of `M2`, member flags 0x0009
// recovered from bytecode; presentation is not claimed to compile
{
    try {
        escaping();
        java.lang.System.out.println("missing throw");
    } catch (java.lang.IllegalStateException local1) {
        java.lang.System.out.println(local1.getMessage() + ":" + count());
    }
    return;
}
"#;
