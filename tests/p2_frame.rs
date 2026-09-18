//! P2 4.1 acceptance: the descriptor-driven frames through the public entry point.
//!
//! The frame table itself is a crate-private payload (invariant 11), so the value-level rules —
//! the local merge that answers `Top` instead of refusing, the category-2 binding, the
//! `dup`/`swap`/`pop` pairing, the descriptor-driven invocation shapes and D41 — are pinned by
//! the unit tests of `crates/jarde-jvm/src/frame.rs`, which decode real and synthetic bodies
//! directly. What this file proves through the public API is the wiring and the planes around
//! that payload:
//!
//! 1. the `frame` phase really runs over the canonical graph of a committed fixture and
//!    completes it — the body of that class carries no stack map or local-variable table, and
//!    the derivation happens anyway, from the descriptors and the data flow;
//! 2. it charges what its row declares: the frames are derived storage and a worklist walk, so a
//!    request that reaches the phase bills strictly more than the same request without it;
//! 3. a constructor completes too: its uninitialized `this` is converted by the
//!    `invokespecial <init>` of its superclass — the initialization conversion of 4.2 — and the
//!    phase behind it is still the one this build does not implement;
//! 4. a body that **uses** an uninitialized value where only an initialized reference is
//!    meaningful — a `new` consumed by `ifnull` — still stops under `ir_frame_deferred` with a
//!    `Partial` execution and keeps the raw and canonical facts: the conversion of 4.2 is the
//!    constructor call, that body performs no such call, and whether its bytes are legal at all
//!    is the verifier's question, which this build answers with `NotPerformed`;
//! 5. and in every case the product planes stay what they are: `verification` is `NotPerformed`
//!    and the semantic evidence is `Unproven`. Deriving frames is not verifying a method.

use jarde::*;
use std::slice;

/// The committed historical fixture: a class whose methods have real bodies, and whose `Code`
/// attributes carry no `StackMapTable` or local-variable table for this derivation to lean on.
const V52: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

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

struct Fixture {
    snapshot: ArtifactSnapshot,
    method: PhysicalMethodId,
}

/// Opens the fixture and derives the physical identity of one of its methods.
fn fixture(name: &[u8], descriptor: &[u8]) -> Fixture {
    fixture_of(V52, name, descriptor)
}

/// The same for any class file: the identity of the definition is the digest of the bytes the
/// request is answered from, so a class assembled beside a committed one is not a special case.
fn fixture_of(class: &[u8], name: &[u8], descriptor: &[u8]) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(class.to_vec()), &mut budget)
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
    Fixture { snapshot, method }
}

/// One caller domain rooted at the fixture, and nothing else.
fn environment(fixture: &Fixture) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Snapshot {
            snapshot: fixture.snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: fixture.snapshot.id().clone(),
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

fn analyze(
    fixture: &Fixture,
    stages: Vec<AnalysisStage>,
    limits: Limits,
) -> (MethodAnalysisReport, Budget) {
    let request = MethodAnalysisRequest {
        environment: environment(fixture),
        method: fixture.method.clone(),
        stages,
    };
    let mut budget = Budget::new(limits);
    let report = Engine::new()
        .analyze_method(slice::from_ref(&fixture.snapshot), &request, &mut budget)
        .expect("a legal request is answered, not raised");
    (report, budget)
}

/// The stage states of a report in scheduled order.
fn stage_states(report: &MethodAnalysisReport) -> Vec<StageState> {
    report
        .stages
        .iter()
        .map(|stage| stage.state.clone())
        .collect()
}

fn diagnostic_codes(report: &MethodAnalysisReport) -> Vec<&str> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

/// The code of the diagnostic this report carries, with its severity.
fn diagnostic(report: &MethodAnalysisReport, code: &str) -> DiagnosticSeverity {
    report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == code)
        .unwrap_or_else(|| panic!("the report carries `{code}`: {:#?}", report.diagnostics))
        .severity
}

/// The four product planes a P2 report always states, whatever the run proved.
fn assert_planes_stay_p1(report: &MethodAnalysisReport) {
    assert_eq!(report.representation, Representation::Bytecode);
    assert_eq!(report.syntax_status, SyntaxStatus::NotJava);
    assert_eq!(report.compile_status, CompileStatus::NotAttempted);
    assert_eq!(
        report.verification,
        VerificationStatus::NotPerformed,
        "deriving frames is not verifying the method"
    );
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::Unproven,
        "4.1 proves local invariants of its own states, and 4.3 is what may raise this"
    );
}

#[test]
fn the_frame_phase_completes_a_body_without_any_debug_table() {
    // `finallyPath(I)I` of the 52 fixture is an instance method with real control flow. Its
    // `Code` attribute carries no `StackMapTable` and no local-variable table, so what the frame
    // phase derives comes from the descriptors and the data flow alone — and the phase behind it
    // is still the one this build does not implement.
    let fixture = fixture(b"finallyPath", b"(I)I");
    let (report, budget) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());
    assert_eq!(
        stage_states(&report),
        vec![
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Failed {
                code: "ir_pass_not_implemented".to_string()
            },
        ],
        "the frame phase completed and `ssa` is the phase this build does not implement"
    );
    assert_eq!(diagnostic_codes(&report), vec!["ir_pass_not_implemented"]);
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(report.body, MethodBodyState::Present);
    assert_planes_stay_p1(&report);

    // The frames are derived storage and a worklist walk, so a request that reaches the phase
    // bills strictly more than the same request without it on both dimensions its row declares.
    let (_, without) = analyze(&fixture, vec![AnalysisStage::CanonicalCfg], limits());
    assert!(
        budget.usage().ir_items > without.usage().ir_items,
        "the frame slots are derived items: {} vs {}",
        budget.usage().ir_items,
        without.usage().ir_items
    );
    assert!(
        budget.usage().analysis_steps > without.usage().analysis_steps,
        "the fixpoint walks a worklist: {} vs {}",
        budget.usage().analysis_steps,
        without.usage().analysis_steps
    );
    assert_eq!(
        budget.usage().ir_edges,
        without.usage().ir_edges,
        "this pass builds no edge of its own"
    );
}

#[test]
fn a_constructor_completes_because_its_constructor_call_converts_the_this() {
    // A constructor's `this` is `uninitializedThis` until its own constructor call runs, and the
    // conversion of that call is part of the frame slice. The fixture's `<init>` performs an
    // `invokespecial java/lang/Object.<init>()V` — the superclass's constructor, one of the two
    // calls the class file's own `super_class` makes applicable — so the body the boundary used to
    // stop on is the body this run derives the frames of, and the phase behind it is still the one
    // this build does not implement.
    let fixture = fixture(b"<init>", b"()V");
    let (report, _) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());
    assert_eq!(
        stage_states(&report),
        vec![
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Failed {
                code: "ir_pass_not_implemented".to_string()
            },
        ],
        "the constructor's frames are derived, and `ssa` is the phase this build does not implement"
    );
    assert_eq!(diagnostic_codes(&report), vec!["ir_pass_not_implemented"]);
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(report.body, MethodBodyState::Present);
    assert_planes_stay_p1(&report);
}

#[test]
fn an_uninitialized_value_used_as_a_reference_stays_the_boundary_of_this_build() {
    // The counterpart of the constructor above, and the reason its completion is not "the pass
    // stopped refusing": the conversion is the one an applicable constructor call performs. A body
    // that consumes a `new` where only an initialized reference is meaningful performs no such
    // call, so it still stops — `Partial` under the frame slice's own code, no phase behind it,
    // and never a claim that the bytes contradict themselves, because whether they are legal is
    // the verifier's question and this build answers `NotPerformed`.
    let class = illegal_class(&[
        0xbb, 0x00, 0x02, // 0: new Test (constant pool 2)
        0xc6, 0x00, 0x03, // 3: ifnull +3 -> 6
        0xb1, // 6: return
    ]);
    let fixture = fixture_of(&class, b"method", b"()V");
    let (report, _) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());
    assert_eq!(
        stage_states(&report),
        vec![
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Partial,
            StageState::NotPerformed,
        ],
        "the frame phase stopped and the phase behind it never ran"
    );
    assert_eq!(diagnostic_codes(&report), vec!["ir_frame_deferred"]);
    assert_eq!(
        diagnostic(&report, "ir_frame_deferred"),
        DiagnosticSeverity::Warning,
        "a boundary of this build is a warning, not damage of the class"
    );
    assert!(
        matches!(
            &report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::Error { code },
                ..
            } if code == "ir_frame_deferred"
        ),
        "a stop at the boundary is a partial execution under its own code: {:?}",
        report.execution
    );
    assert_eq!(
        report.quality,
        Quality::Conservative,
        "the canonical CFG of the earlier phase is still the artifact this run produced"
    );
    assert_eq!(report.body, MethodBodyState::Present);
    assert_planes_stay_p1(&report);
}

#[test]
fn a_pre_initialization_putfield_of_the_own_name_is_accepted_without_any_declared_field() {
    // The restricted `putfield` of JVMS 4.10.1.9 is decided by the **name** its `Fieldref` gives for
    // the field's owner: the frame slice compares that name with the class file's own and holds no
    // field table, so a class that declares no field at all has the instruction accepted all the
    // same. `class_without_fields` is that shape at its sharpest — `fields_count` is zero and the
    // one field the pool names exists nowhere in the file — and the run derives the constructor's
    // frames instead of stopping at the boundary. Whether a body storing a field that no class
    // declares is legal is the verifier's question, which is why the planes below still state
    // `NotPerformed` and `Unproven`.
    //
    // 0  aload_0       the uninitialized `this`
    // 1  iconst_1      the value
    // 2  putfield #11  `Test.x:I`, a field no class in this file declares
    // 5  return
    let class = class_without_fields(&[0x2a, 0x04, 0xb5, 0x00, 0x0b, 0xb1]);
    let fixture = fixture_of(&class, b"<init>", b"()V");
    let (report, _) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());
    assert_eq!(
        stage_states(&report),
        vec![
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Failed {
                code: "ir_pass_not_implemented".to_string()
            },
        ],
        "the frames come out of a body whose `putfield` names its own class, declared field or not"
    );
    assert_eq!(
        diagnostic_codes(&report),
        vec!["ir_pass_not_implemented"],
        "the boundary code is not among them: the instruction is read, not refused"
    );
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(report.body, MethodBodyState::Present);
    assert_planes_stay_p1(&report);
}

/// A real class file with one `Test.method()V` whose body the caller writes, and the constant pool
/// entry `2` the `new` of those bodies names.
///
/// The committed fixture is a compiled class, so its bodies are bodies someone wrote as Java; a
/// body that *uses* an uninitialized value where only an initialized reference is meaningful has
/// to be assembled. It is assembled here and not through `jarde_reader`'s own test builder because
/// that builder lives behind another crate's `cfg(test)`/`test-support` surface, which this test
/// does not reach: what the request reads is a real CLASS either way.
fn illegal_class(code: &[u8]) -> Vec<u8> {
    let mut pool = Vec::new();
    utf8(&mut pool, b"Test"); // 1
    class(&mut pool, 1); // 2
    utf8(&mut pool, b"java/lang/Object"); // 3
    class(&mut pool, 3); // 4
    utf8(&mut pool, b"method"); // 5
    utf8(&mut pool, b"()V"); // 6
    utf8(&mut pool, b"Code"); // 7

    let mut content = code_attribute(code, 1, 1);
    let mut method = method(0x0009, 5, 6, &mut content);

    let mut bytes = header(8);
    bytes.extend_from_slice(&pool);
    class_tail(&mut bytes, 2, 4, &mut method);
    bytes
}

/// A real class file of one class `Test` whose `fields_count` is **zero**: the only field its
/// constant pool names is `Test.x:I`, and no class in the file declares it.
///
/// The class carries one non-static `<init>()V` whose body the caller writes. This is the shape the
/// frame slice's restricted `putfield` is decided on — the `Fieldref` names the class the method is
/// declared in, and nothing else about the field exists for the pass to read.
fn class_without_fields(code: &[u8]) -> Vec<u8> {
    let mut pool = Vec::new();
    utf8(&mut pool, b"Test"); // 1
    class(&mut pool, 1); // 2
    utf8(&mut pool, b"java/lang/Object"); // 3
    class(&mut pool, 3); // 4
    utf8(&mut pool, b"<init>"); // 5
    utf8(&mut pool, b"()V"); // 6
    utf8(&mut pool, b"Code"); // 7
    utf8(&mut pool, b"x"); // 8
    utf8(&mut pool, b"I"); // 9
    name_and_type(&mut pool, 8, 9); // 10
    field_ref(&mut pool, 2, 10); // 11

    let mut content = code_attribute(code, 2, 1);
    let mut method = method(0x0001, 5, 6, &mut content);

    let mut bytes = header(12);
    bytes.extend_from_slice(&pool);
    class_tail(&mut bytes, 2, 4, &mut method);
    bytes
}

/// A `u2` in the class-file byte order.
fn u16_be(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

/// One `CONSTANT_Utf8` entry.
fn utf8(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.push(1);
    bytes.extend_from_slice(
        &u16::try_from(value.len())
            .expect("a fixture name fits u16")
            .to_be_bytes(),
    );
    bytes.extend_from_slice(value);
}

/// One `CONSTANT_Class` entry over a `Utf8` name index.
fn class(bytes: &mut Vec<u8>, name: u16) {
    bytes.push(7);
    u16_be(bytes, name);
}

/// One `CONSTANT_NameAndType` entry.
fn name_and_type(bytes: &mut Vec<u8>, name: u16, descriptor: u16) {
    bytes.push(12);
    u16_be(bytes, name);
    u16_be(bytes, descriptor);
}

/// One `CONSTANT_Fieldref` entry over a `Class` and a `NameAndType` index.
fn field_ref(bytes: &mut Vec<u8>, owner: u16, name_and_type: u16) {
    bytes.push(9);
    u16_be(bytes, owner);
    u16_be(bytes, name_and_type);
}

/// The `Code` content of the one method: the caller's bytes and nothing else.
fn code_attribute(code: &[u8], max_stack: u16, max_locals: u16) -> Vec<u8> {
    let mut content = Vec::new();
    u16_be(&mut content, max_stack);
    u16_be(&mut content, max_locals);
    content.extend_from_slice(
        &u32::try_from(code.len())
            .expect("fixture code fits u32")
            .to_be_bytes(),
    );
    content.extend_from_slice(code);
    u16_be(&mut content, 0); // exception table
    u16_be(&mut content, 0); // Code attributes
    content
}

/// One method member whose single attribute is the `Code` at index `7`.
fn method(flags: u16, name: u16, descriptor: u16, content: &mut Vec<u8>) -> Vec<u8> {
    let mut method = Vec::new();
    u16_be(&mut method, flags);
    u16_be(&mut method, name);
    u16_be(&mut method, descriptor);
    u16_be(&mut method, 1); // one attribute
    u16_be(&mut method, 7); // `Code`
    method.extend_from_slice(
        &u32::try_from(content.len())
            .expect("fixture Code content fits u32")
            .to_be_bytes(),
    );
    method.append(content);
    method
}

/// The class-file header up to the constant pool, with `constant_pool_count` = `entries`.
fn header(entries: u16) -> Vec<u8> {
    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16_be(&mut bytes, 0); // minor
    u16_be(&mut bytes, 52); // major: the fixture era of this slice
    u16_be(&mut bytes, entries);
    bytes
}

/// Everything after the constant pool: this class, its superclass, and its one method — with no
/// interface and no field, which is what both fixtures of this file declare.
fn class_tail(bytes: &mut Vec<u8>, this_class: u16, super_class: u16, method: &mut Vec<u8>) {
    u16_be(bytes, 0x0021); // ACC_PUBLIC | ACC_SUPER
    u16_be(bytes, this_class);
    u16_be(bytes, super_class);
    u16_be(bytes, 0); // interfaces
    u16_be(bytes, 0); // fields
    u16_be(bytes, 1); // methods
    bytes.append(method);
    u16_be(bytes, 0); // class attributes
}
