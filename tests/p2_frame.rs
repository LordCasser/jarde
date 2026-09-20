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
//!    — deriving frames is not verifying a method — while `semantic_validation` is the run's own
//!    evidence and each test states it for the run it really performed: a request that reaches
//!    `ssa` and completes it reports the local invariants that phase checked, and the request
//!    that stops at the frame boundary reports `Unproven`;
//! 6. a handler's entry state is the merge of the states its throw sites **settle** on, not of
//!    the ones they were entered with the first time: the fixpoint re-takes an exception input
//!    whenever the source block runs again, even when that block's own exit state never changes
//!    (R9). The three assembled bodies of this section state that through the public entry as
//!    well — a single-site body whose handler went stale, the same cycle with the exception edge
//!    feeding a block that has already been processed, and a block with two throw sites whose
//!    handler entry is the merge of both;
//! 7. and it bills what it holds (R10): a block with a catch-all record really does keep one
//!    snapshot and one exception input per throw site, so its `IrItems` grows with the product of
//!    sites and local slots — while a block whose sites no handler record covers keeps none of
//!    them and bills the same for eight sites and for sixty-four. A request that runs out of items
//!    inside the frame phase or inside the one behind it stops with a diagnostic and publishes
//!    neither a table nor a name, exactly like a cancelled one.

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

/// The same limits with one field replaced, for the budget cases below.
fn limits_with(change: impl FnOnce(&mut Limits)) -> Limits {
    let mut limits = limits();
    change(&mut limits);
    limits
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
        roots: vec![LoadRoot::StandaloneClass {
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

/// The four product planes a P2 report always states, whatever the run proved: the P1 baseline
/// and the verifier status this build never raises.
///
/// `semantic_validation` is deliberately not one of them: since 4.3 it is the run's own evidence,
/// so each test below states it for the run that test really performed.
fn assert_planes_stay_p1(report: &MethodAnalysisReport) {
    assert_eq!(report.representation, Representation::Bytecode);
    assert_eq!(report.syntax_status, SyntaxStatus::NotJava);
    assert_eq!(report.compile_status, CompileStatus::NotAttempted);
    assert_eq!(
        report.verification,
        VerificationStatus::NotPerformed,
        "deriving frames is not verifying the method"
    );
}

#[test]
fn the_frame_phase_completes_a_body_without_any_debug_table() {
    // `finallyPath(I)I` of the 52 fixture is an instance method with real control flow. Its
    // `Code` attribute carries no `StackMapTable` and no local-variable table, so what the frame
    // phase derives comes from the descriptors and the data flow alone — and the phase behind it,
    // `ssa`, names exactly those frames and completes too.
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
            StageState::Completed,
        ],
        "the frame phase completed and `ssa` named the frames it published"
    );
    assert!(
        diagnostic_codes(&report).is_empty(),
        "a completed pipeline reports no diagnostic: {:?}",
        diagnostic_codes(&report)
    );
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(report.body, MethodBodyState::Present);
    assert_planes_stay_p1(&report);
    // The pipeline ran to its end, so the phase that checks the local invariants of the IR
    // completed: the frames this run published are internally consistent by the one plane that
    // states such evidence, and nothing here claims the bytes are legal.
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );

    // The frames are derived storage and a worklist walk, so a request that reaches the phase
    // bills strictly more than the same request without it on both dimensions its row declares;
    // the names 4.3 derives over them add the def-use edges its own row declares on top.
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
    assert!(
        budget.usage().ir_edges > without.usage().ir_edges,
        "4.1 builds no edge of its own and 4.3 bills one def-use edge per use: {} vs {}",
        budget.usage().ir_edges,
        without.usage().ir_edges
    );
}

#[test]
fn a_constructor_completes_because_its_constructor_call_converts_the_this() {
    // A constructor's `this` is `uninitializedThis` until its own constructor call runs, and the
    // conversion of that call is part of the frame slice. The fixture's `<init>` performs an
    // `invokespecial java/lang/Object.<init>()V` — the superclass's constructor, one of the two
    // calls the class file's own `super_class` makes applicable — so the body the boundary used to
    // stop on is the body this run derives the frames of, and 4.3 names the converted slots: the
    // alias conversion is a definition, so the token reaches its initialized state as a new value.
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
            StageState::Completed,
        ],
        "the constructor's frames are derived, and `ssa` named them"
    );
    assert!(
        diagnostic_codes(&report).is_empty(),
        "a completed pipeline reports no diagnostic: {:?}",
        diagnostic_codes(&report)
    );
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(report.body, MethodBodyState::Present);
    assert_planes_stay_p1(&report);
    // `ssa` completed over the converted `this`, so the local invariants it checks passed over
    // the IR this run published — one definition per value, and the alias the constructor call
    // converted is one of them.
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
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
    // The run stopped at the frame boundary: the phase behind it never ran, so this report
    // carries no semantic evidence at all — `Unproven` is what a stop inside this build states,
    // and it is not a claim that the body is damaged.
    assert_eq!(report.semantic_validation, SemanticValidation::Unproven);
}

#[test]
fn a_pre_initialization_putfield_of_the_own_name_is_accepted_without_any_declared_field() {
    // The restricted `putfield` of JVMS 4.10.1.9 is decided by the **name** its `Fieldref` gives for
    // the field's owner: the frame slice compares that name with the class file's own and holds no
    // field table, so a class that declares no field at all has the instruction accepted all the
    // same. `class_without_fields` is that shape at its sharpest — `fields_count` is zero and the
    // one field the pool names exists nowhere in the file — and the run derives the constructor's
    // frames instead of stopping at the boundary. Whether a body storing a field that no class
    // declares is legal is the verifier's question, and this build answers it with
    // `NotPerformed` — a different plane from the one the run's own completed `ssa` phase raises
    // over the IR it published.
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
            StageState::Completed,
        ],
        "the frames come out of a body whose `putfield` names its own class, declared field or not"
    );
    assert!(
        diagnostic_codes(&report).is_empty(),
        "no boundary code is among them: the instruction is read, not refused, and 4.3 names \
         the frames it published"
    );
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(report.body, MethodBodyState::Present);
    assert_planes_stay_p1(&report);
    // `ssa` completed over that body, so the local invariants of the published IR hold — the
    // values of this method are defined once and used where they were named, whatever the
    // verifier would say about the field the `putfield` names.
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
}

/// One record of a `Code` attribute's `exception_table` (JVMS 4.7.3).
///
/// Most fixtures below use `catch_type = 0` only — the catch-all record, which names no class and
/// therefore needs no constant-pool entry — so the class builder of this file holds the pool both
/// written fixtures already share. The one fixture whose records name a class states its own pool
/// ([`two_catch_types_one_handler_class`]), because the class it names has to be an entry of it.
#[derive(Clone, Copy)]
struct ExceptionRecord {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}

/// A real class file of one class `Test` whose single method declares `flags`, `name` and
/// `descriptor` and carries the caller's body, at class-file version `major`.
///
/// The pool is the one `illegal_class` writes — `Test`, its superclass, the member's name and
/// descriptor, and `Code` — so a body assembled here differs from that one only in what the caller
/// writes. `major` is a parameter because the version is a fact the reader keeps: a 49 class file
/// is the dialect the R9 counterexample below comes from, and it is not the 52 one the committed
/// fixtures are compiled at.
#[allow(clippy::too_many_arguments)]
fn class_of(
    major: u16,
    flags: u16,
    name: &[u8],
    descriptor: &[u8],
    code: &[u8],
    max_stack: u16,
    max_locals: u16,
    handlers: &[ExceptionRecord],
) -> Vec<u8> {
    assert!(
        handlers.iter().all(|record| record.catch_type == 0),
        "the pool of this builder holds no exception class entry: only catch-all records"
    );
    let mut pool = Vec::new();
    utf8(&mut pool, b"Test"); // 1
    class(&mut pool, 1); // 2
    utf8(&mut pool, b"java/lang/Object"); // 3
    class(&mut pool, 3); // 4
    utf8(&mut pool, name); // 5
    utf8(&mut pool, descriptor); // 6
    utf8(&mut pool, b"Code"); // 7

    let mut content = code_attribute_with(code, max_stack, max_locals, handlers);
    let mut method = method(flags, 5, 6, &mut content);

    let mut bytes = header_at(8, major);
    bytes.extend_from_slice(&pool);
    class_tail(&mut bytes, 2, 4, &mut method);
    bytes
}

/// The descriptor of the three exception fixtures below: one argument the handler bodies can return
/// either directly or through a slot, so the class file states a reference the frames can carry.
const EXCEPTION_METHOD: &[u8] = b"(Ljava/lang/Object;)Ljava/lang/Object;";

/// The class file of the R10 shape: `sites` throwing instructions in **one** block, `locals` local
/// slots, and — when `handler` — a catch-all record covering every one of those sites.
///
/// `04 04 6c 57` is `iconst_1; iconst_1; idiv; pop`: the `idiv` divides one by zero, so each
/// repetition holds one canonical throw site, and the block holds `sites` of them. The record's
/// protected range is `[0, 4 * sites)` and it catches everything, so the handler of the second
/// shape is entered with one input per site. `max_locals` is the caller's: it is what makes the
/// product of sites and locals the thing the budget has to bound.
fn throw_sites_class(sites: usize, locals: u16, handler: bool) -> Vec<u8> {
    /// One `iconst_1; iconst_1; idiv; pop`.
    const REPEAT: &[u8] = &[0x04, 0x04, 0x6c, 0x57];
    let mut code = Vec::new();
    for _ in 0..sites {
        code.extend_from_slice(REPEAT);
    }
    let end = u16::try_from(code.len()).expect("a fixture body fits u16");
    let handlers = if handler {
        // `return`, then the handler at the next byte: `pop; return`, which discards the caught
        // reference. The record's range stops at the `return` of the body, so the handler is only
        // ever entered through the exception edge.
        code.push(0xb1);
        let handler_pc = u16::try_from(code.len()).expect("a fixture body fits u16");
        code.extend_from_slice(&[0x57, 0xb1]);
        vec![ExceptionRecord {
            start_pc: 0,
            end_pc: end,
            handler_pc,
            catch_type: 0,
        }]
    } else {
        code.push(0xb1); // return
        Vec::new()
    };
    class_of(
        49, 0x0009, // ACC_PUBLIC | ACC_STATIC
        b"method", b"()V", &code, 2, locals, &handlers,
    )
}

/// The `IrItems` one request for `stages` over the R10 shape spends, and the request's report.
fn items_of(
    class: &[u8],
    stages: Vec<AnalysisStage>,
    limits: Limits,
) -> (u64, MethodAnalysisReport) {
    let fixture = fixture_of(class, b"method", b"()V");
    let (report, budget) = analyze(&fixture, stages, limits);
    (budget.usage().ir_items, report)
}

/// The class file of the R9 counterexample.
fn r9_class() -> Vec<u8> {
    /// The 17 bytes of the body the test documents.
    const CODE: &[u8] = &[
        0x01, 0x4c, // 0: aconst_null; 1: astore_1
        0x04, 0x03, 0x6c, 0x57, // 2: iconst_1; 3: iconst_0; 4: idiv; 5: pop
        0x2a, 0x4c, 0x03, 0x99, 0xff, 0xf9, // 6: aload_0; 7: astore_1; 8: iconst_0; 9: ifeq 2
        0x2b, 0xb0, // 12: aload_1; 13: areturn
        0x57, 0x2b, 0xb0, // 14: pop; 15: aload_1; 16: areturn
    ];
    // The branch really lands on BCI 2: `9 + (-7)`.
    assert_eq!(i32::from(i16::from_be_bytes([CODE[10], CODE[11]])), -7);
    class_of(
        49,
        0x0009, // ACC_PUBLIC | ACC_STATIC
        b"method",
        EXCEPTION_METHOD,
        CODE,
        2,
        2,
        &[ExceptionRecord {
            start_pc: 2,
            end_pc: 12,
            handler_pc: 14,
            catch_type: 0,
        }],
    )
}

/// The class file of the exception-back-edge contrast.
fn exception_back_edge_class() -> Vec<u8> {
    const CODE: &[u8] = &[
        0x01, 0x4c, // 0: aconst_null; 1: astore_1
        0x04, 0x03, 0x6c, 0x57, // 2: iconst_1; 3: iconst_0; 4: idiv; 5: pop
        0x2a, 0x4c, 0x2a, 0xc7, 0xff,
        0xf9, // 6: aload_0; 7: astore_1; 8: aload_0; 9: ifnonnull 2
        0x2b, 0xb0, // 12: aload_1; 13: areturn
        0x57, 0xa7, 0xff, 0xf3, // 14: pop; 15: goto 2
    ];
    // `ifnonnull` at 9 lands on BCI 2 and the `goto` at 15 does too.
    assert_eq!(i32::from(i16::from_be_bytes([CODE[10], CODE[11]])), -7);
    assert_eq!(i32::from(i16::from_be_bytes([CODE[16], CODE[17]])), -13);
    class_of(
        49,
        0x0009,
        b"method",
        EXCEPTION_METHOD,
        CODE,
        2,
        2,
        &[ExceptionRecord {
            start_pc: 2,
            end_pc: 12,
            handler_pc: 14,
            catch_type: 0,
        }],
    )
}

/// The class file of the two-throw-sites contrast.
fn two_throw_sites_class() -> Vec<u8> {
    const CODE: &[u8] = &[
        0x01, 0x4c, // 0: aconst_null; 1: astore_1
        0x04, 0x03, 0x6c, 0x57, // 2: iconst_1; 3: iconst_0; 4: idiv; 5: pop
        0x2a, 0x4c, 0x2a, 0x4d, // 6: aload_0; 7: astore_1; 8: aload_0; 9: astore_2
        0x04, 0x03, 0x6c, 0x57, // 10: iconst_1; 11: iconst_0; 12: idiv; 13: pop
        0x2a, 0xc7, 0xff, 0xf3, // 14: aload_0; 15: ifnonnull 2
        0x2b, 0xb0, // 18: aload_1; 19: areturn
        0x57, 0x2b, 0xb0, // 20: pop; 21: aload_1; 22: areturn
    ];
    // `ifnonnull` at 15 lands on BCI 2.
    assert_eq!(i32::from(i16::from_be_bytes([CODE[16], CODE[17]])), -13);
    class_of(
        49,
        0x0009,
        b"method",
        EXCEPTION_METHOD,
        CODE,
        2,
        3,
        &[ExceptionRecord {
            start_pc: 2,
            end_pc: 18,
            handler_pc: 20,
            catch_type: 0,
        }],
    )
}

/// The R9 counterexample, through the public entry point: an exception input follows the state its
/// throw site **settles** on, even when the block's own exit state never changes.
///
/// `Test.method(Ljava/lang/Object;)Ljava/lang/Object;` — static, `max_stack` 2, `max_locals` 2, a
/// 49 class file with no `StackMapTable` and no local-variable table:
///
/// ```text
///  0: aconst_null; 1: astore_1
///  2: iconst_1; 3: iconst_0; 4: idiv; 5: pop
///  6: aload_0; 7: astore_1; 8: iconst_0; 9: ifeq 2
/// 12: aload_1; 13: areturn
/// 14: pop; 15: aload_1; 16: areturn
/// exception_table: [start_pc=2, end_pc=12, handler_pc=14, catch_type=0]
/// ```
///
/// Slot 1 is `null` when BCI 2 is entered the first time and the argument reference after the back
/// edge settles; the block's exit is the argument reference on **both** visits, because BCI 7
/// overwrites the slot either way. A skip keyed on the exit alone therefore keeps the exception
/// input BCI 4 hands its handler at `null` while the handler's own input counts as one
/// definition of the reference, and `ssa` refuses the contradiction with `ir_ssa_inconsistent`.
/// The bytes are a body an OpenJDK runs and returns `null` for; what the fix has to restore is the
/// handler's entry state, not a check.
#[test]
fn an_exception_input_follows_the_state_its_throw_site_settles_on() {
    let class = r9_class();
    let fixture = fixture_of(&class, b"method", EXCEPTION_METHOD);
    // A request for the frames alone names the same body and completes — before the fix as well,
    // which is the honest boundary of this case: the stale state is a state the frame phase
    // *derives*, and nothing in that phase reads it back. What refuses it is the phase behind,
    // which is where the assertion with teeth is; the frame request is here so that the two are
    // stated about the same bytes.
    let (frames_only, _) = analyze(&fixture, vec![AnalysisStage::Frame], limits());
    assert_eq!(stage_states(&frames_only), vec![StageState::Completed; 5]);
    assert!(diagnostic_codes(&frames_only).is_empty());

    let (report, _) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());

    assert_eq!(
        stage_states(&report),
        vec![StageState::Completed; 6],
        "every phase names the IR of this body, the handler's entry state included: {:?}",
        report.diagnostics
    );
    assert!(diagnostic_codes(&report).is_empty());
    assert_planes_stay_p1(&report);
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
}

/// The first contrast of the case above: an exception edge whose handler feeds a block that has
/// already been processed — a cycle the exception path runs back into the loop's own head.
///
/// `Test.method(Ljava/lang/Object;)Ljava/lang/Object;`, static, `max_stack` 2, `max_locals` 2, the
/// same 49 class file:
///
/// ```text
///  0: aconst_null; 1: astore_1
///  2: iconst_1; 3: iconst_0; 4: idiv; 5: pop
///  6: aload_0; 7: astore_1; 8: aload_0; 9: ifnonnull 2
/// 12: aload_1; 13: areturn
/// 14: pop; 15: goto 2
/// exception_table: [start_pc=2, end_pc=12, handler_pc=14, catch_type=0]
/// ```
///
/// The handler is *inside* the cycle: BCI 2 is entered from the entry block, from its own branch
/// and from BCI 15, and the exception edge BCI 4 → BCI 14 is the only input BCI 14 has. Its entry
/// state therefore has to move when the loop head settles, and the block it feeds back into has
/// already been processed when that happens — the shape the fix must still converge on, and the
/// one that catches a fix that re-propagates exception inputs without the merges being idempotent.
/// The body is the one above with a handler that resumes the loop, and slot 1 settles on the
/// argument reference for the same reason.
#[test]
fn an_exception_input_into_an_already_processed_block_settles_with_it() {
    let class = exception_back_edge_class();
    let fixture = fixture_of(&class, b"method", EXCEPTION_METHOD);
    let (report, budget) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());

    assert_eq!(
        stage_states(&report),
        vec![StageState::Completed; 6],
        "the cycle through the handler converges: {:?}",
        report.diagnostics
    );
    assert!(diagnostic_codes(&report).is_empty());
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
    assert!(
        budget.usage().analysis_steps < limits().analysis_steps,
        "the cycle through the handler converges instead of spending the budget: {} steps",
        budget.usage().analysis_steps
    );
}

/// The second contrast: **two throw sites of one block**, both feeding the same handler, with the
/// loop's exit stable across its visits.
///
/// ```text
///  0: aconst_null; 1: astore_1
///  2: iconst_1; 3: iconst_0; 4: idiv; 5: pop              the first site, slot 1 null
///  6: aload_0; 7: astore_1; 8: aload_0; 9: astore_2       slot 1 and slot 2, the argument
/// 10: iconst_1; 11: iconst_0; 12: idiv; 13: pop           the second site, both slots the argument
/// 14: aload_0; 15: ifnonnull 2
/// 18: aload_1; 19: areturn
/// 20: pop; 21: aload_1; 22: areturn
/// exception_table: [start_pc=2, end_pc=18, handler_pc=20, catch_type=0]
/// ```
///
/// One canonical exception edge aggregates both sites, and the handler's entry state is the merge
/// of the two states they hand it — not the block's exit, which is the same on both visits, and
/// not one of the two sites. The fix re-takes that merge on every visit of the source block, so
/// this is the shape that says the re-taken inputs stay the *per-site* ones: a fix that handed the
/// handler the block's exit, or only the last site's state, would name a slot no site states.
#[test]
fn two_throw_sites_of_one_block_still_merge_their_own_states() {
    let class = two_throw_sites_class();
    let fixture = fixture_of(&class, b"method", EXCEPTION_METHOD);
    let (report, budget) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());

    assert_eq!(
        stage_states(&report),
        vec![StageState::Completed; 6],
        "both sites' states reach the handler and the walk terminates: {:?}",
        report.diagnostics
    );
    assert!(diagnostic_codes(&report).is_empty());
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
    // The re-taken inputs must not turn the walk into a loop of its own: a run that re-merged a
    // handler on every visit without the merge being idempotent would spend the step budget here.
    assert!(
        budget.usage().analysis_steps < limits().analysis_steps,
        "the walk converges instead of spending the budget: {} steps",
        budget.usage().analysis_steps
    );
}

/// The fuzz corpus seed of **two records naming one handler for one site**: the input
/// `fuzz/corpus/method_analysis/exception-overlap-mixed.class`, taken byte for byte from where the
/// fuzzer found it (189 bytes, the size the fuzz workspace's own corpus test pins) instead of
/// expressed again here as a body, so the regression answers the input that reported the defect.
///
/// ```text
///  0: iload_0
///  1: ifeq 9
///  4: aconst_null
///  5: athrow                 the body's one throw site, in the block that starts at BCI 4
///  6: nop; 7: nop; 8: nop    unreachable: BCI 9 is a branch target
///  9: iconst_1
/// 10: ireturn
/// 11: pop; iconst_2; ireturn the handler records 0 and 2 both name
/// 14: pop; iconst_3; ireturn the handler record 1 names
/// exception_table:
///   record 0: start_pc=0, end_pc=6, handler_pc=11, catch_type=java/lang/Throwable
///   record 1: start_pc=4, end_pc=9, handler_pc=14, catch_type=0 (catch-all)
///   record 2: start_pc=0, end_pc=9, handler_pc=11, catch_type=0 (catch-all)
/// ```
const OVERLAP_MIXED: &[u8] =
    include_bytes!("../fuzz/corpus/method_analysis/exception-overlap-mixed.class");

/// Two records of one exception table naming **one handler for one source block**: two edges
/// between the same pair of blocks, and the handler's entry state the merge of both records'
/// states.
///
/// Records 0 and 2 both cover the `athrow` at BCI 5 and both name handler BCI 11, so the block at
/// BCI 4 leaves through two exception edges into one handler; record 0 catches `java/lang/Throwable`
/// and record 2 catches everything. The two contributions therefore *differ* — the named reference
/// and the unknown one — and their merge is the unknown reference, which is the class the handler
/// is entered with. The run this body used to answer with was a contradiction of its own artifacts,
/// `ir_ssa_inconsistent`, because the table of logical inputs was keyed by the block the inputs come
/// from: the second edge's records replaced the first's, so the names over the handler saw one input
/// defining the named reference while the frames stated the unknown one. The assertion with teeth
/// is the first: every phase this build implements completes and nothing is reported, because the
/// body is bytes a JVM links and runs.
#[test]
fn two_records_naming_one_handler_still_hand_it_both_inputs() {
    let fixture = fixture_of(OVERLAP_MIXED, b"guarded", b"(I)I");
    let (report, _) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());
    assert_eq!(
        stage_states(&report),
        vec![StageState::Completed; 6],
        "both records' states reach the handler and every phase completes: {:?}",
        report.diagnostics
    );
    assert!(
        diagnostic_codes(&report).is_empty(),
        "neither the frames nor the names over them stopped: {:?}",
        report.diagnostics
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(report.body, MethodBodyState::Present);
    assert_planes_stay_p1(&report);
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
}

/// The class file of the **distinguishable** two-records-one-handler shape: one throw site, two
/// records naming one handler, and the two records catch two *different* classes.
///
/// `Test.method()V`, static, `max_stack` 2, `max_locals` 1, at class-file version 49 (a body of
/// this shape needs no `StackMapTable`, and the frame slice never reads one):
///
/// ```text
///  0: iconst_1        block A, the entry
///  1: iconst_0
///  2: idiv            the body's one throw site, covered by both records
///  3: pop
///  4: return
///  5: pop             the handler both records name
///  6: return
/// exception_table:
///   record 0: start_pc=0, end_pc=3, handler_pc=5, catch_type=#9  (java/lang/RuntimeException)
///   record 1: start_pc=0, end_pc=3, handler_pc=5, catch_type=#11 (java/lang/Throwable)
/// ```
///
/// The two records hand the handler two **named** classes this layer keeps apart — it holds no
/// class hierarchy, so it does not see that one is a subtype of the other — and the merge of two
/// different named references is the conservative unknown one. That is what makes the fold visible
/// from the bytecode plan alone: the handler's entry state is the unknown reference while *either*
/// record's own class is a name, so a source-keyed list of inputs that kept one record for the two
/// edges leaves a state and an input stating two different classes, whichever of the two records it
/// kept. The review's seven probes were the other arrangement — a named catch and a catch-all over
/// one site — and that one can pass before and after the key change, because the catch-all's
/// unknown reference is what the merge states anyway; this case exists so the regression cannot be
/// answered by a shape like that.
///
/// The order of the two records is part of the shape: the canonical graph sorts the exception edges
/// of one source by the record's own ordinal, so the record written last is the one a source-keyed
/// list keeps.
fn two_catch_types_one_handler_class() -> Vec<u8> {
    /// The body above: one `idiv`, the fall-through `return`, and the handler both records name.
    const CODE: &[u8] = &[
        0x04, 0x03, 0x6c, 0x57, // 0: iconst_1; 1: iconst_0; 2: idiv; 3: pop
        0xb1, // 4: return
        0x57, 0xb1, // 5: pop; 6: return (the handler both records name)
    ];
    let mut pool = Vec::new();
    utf8(&mut pool, b"Test"); // 1
    class(&mut pool, 1); // 2
    utf8(&mut pool, b"java/lang/Object"); // 3
    class(&mut pool, 3); // 4
    utf8(&mut pool, b"method"); // 5
    utf8(&mut pool, b"()V"); // 6
    utf8(&mut pool, b"Code"); // 7
    utf8(&mut pool, b"java/lang/RuntimeException"); // 8
    class(&mut pool, 8); // 9
    utf8(&mut pool, b"java/lang/Throwable"); // 10
    class(&mut pool, 10); // 11

    let mut content = code_attribute_with(
        CODE,
        2,
        1,
        &[
            ExceptionRecord {
                start_pc: 0,
                end_pc: 3,
                handler_pc: 5,
                catch_type: 9,
            },
            ExceptionRecord {
                start_pc: 0,
                end_pc: 3,
                handler_pc: 5,
                catch_type: 11,
            },
        ],
    );
    let mut method = method(0x0009, 5, 6, &mut content);

    let mut bytes = header_at(12, 49);
    bytes.extend_from_slice(&pool);
    class_tail(&mut bytes, 2, 4, &mut method);
    bytes
}

/// The same defect as the seed above, in the arrangement that can be **told apart** from a correct
/// run: two records of one exception table naming one handler for one source block, catching two
/// different classes, neither of them the catch-all.
///
/// The seed's records are a named catch and a catch-all, and the review's probes were shaped the
/// same way — a probe whose surviving record is the catch-all states the unknown reference the
/// merge states anyway, so it passed before and after the key change and proved nothing. Here both
/// records name a class, so the handler is entered with the unknown reference of their conservative
/// merge while the input a fold keeps defines a name: the two readings of one fact state two
/// different classes, whatever the fold keeps, and the run reports a legal body as contradicting
/// itself — `ir_ssa_inconsistent` when the record list was keyed by the source block and nothing
/// counted it, `ir_frame_inconsistent` now that the frame pass counts its own records against the
/// contributions it merged.
///
/// Both halves are asserted: the bytecode plan states the shape this case is about — two records,
/// one handler, two different catch types — and the analysis over the same bytes completes every
/// phase this build implements and reports nothing.
#[test]
fn the_two_records_of_one_handler_are_read_in_the_order_that_shows_a_fold() {
    let class = two_catch_types_one_handler_class();
    // The shape itself, read through 1.2's own inspection rather than assumed from the bytes
    // written above: two records, one handler, different catch types. A fixture that lost that
    // shape would stop exercising the defect and must fail here instead of passing quietly.
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(class.clone()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    let inspected = Engine::new()
        .inspect_method_bytecode(
            &snapshot,
            ClassTarget::Root,
            MethodSelector {
                name: bytes(b"method"),
                descriptor: bytes(b"()V"),
            },
            &mut budget,
        )
        .expect("the 1.2 inspection reads the body of this class");
    let handlers = &inspected.inspection.exception_handlers;
    assert_eq!(handlers.len(), 2, "two records");
    assert!(
        handlers
            .iter()
            .all(|record| (record.start_bci, record.end_bci, record.handler_bci) == (0, 3, 5)),
        "both cover the same range and name the same handler: {handlers:?}"
    );
    assert_eq!(
        handlers
            .iter()
            .map(|record| record.catch_type_index)
            .collect::<Vec<_>>(),
        vec![Some(9), Some(11)],
        "the two records name two different classes: the handler's merge is the unknown reference \
         and neither record's own class is it"
    );

    let fixture = fixture_of(&class, b"method", b"()V");
    let (report, _) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());
    assert_eq!(
        stage_states(&report),
        vec![StageState::Completed; 6],
        "both records' states reach the handler and every phase completes: {:?}",
        report.diagnostics
    );
    assert!(
        diagnostic_codes(&report).is_empty(),
        "neither the frames nor the names over them stopped: {:?}",
        report.diagnostics
    );
    assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(report.body, MethodBodyState::Present);
    assert_planes_stay_p1(&report);
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
}

/// One body of two records — one range that starts **inside** a block, one that starts on a block
/// — each covering one throwing instruction, and one handler that consumes the reference it is
/// entered with.
///
/// ```text
///  0: iconst_1        block A, the entry
///  1: iconst_0
///  2: idiv            throws: record 0 covers it, and record 0's range starts at BCI 1
///  3: pop
///  4: goto 7          block A ends
///  7: iconst_1        block B, the `goto` target
///  8: iconst_0
///  9: idiv            throws: record 1 covers it, and record 1's range starts with this block
/// 10: pop
/// 11: return
/// 12: pop; return     the handler of record 0
/// 14: pop; return     the handler of record 1
/// ```
///
/// The raw exception edges come from the **throw sites**: a site's feasible handlers are the
/// records whose range covers the site's own BCI (3.3), and no block start has to lie in the range
/// for that. `first` is the range record 0 declares — `(1, 4)` is the mid-block shape, `(0, 4)`
/// starts on the entry block, which is the shape a range test can see.
fn mid_block_record_class(first: (u16, u16)) -> Vec<u8> {
    /// The body above: `idiv` at BCI 2 and at BCI 9, and one handler per site.
    const CODE: &[u8] = &[
        0x04, 0x03, 0x6c, 0x57, // 0: iconst_1; 1: iconst_0; 2: idiv; 3: pop
        0xa7, 0x00, 0x03, // 4: goto 7
        0x04, 0x03, 0x6c, 0x57, // 7: iconst_1; 8: iconst_0; 9: idiv; 10: pop
        0xb1, // 11: return
        0x57, 0xb1, // 12: pop; return (record 0's handler)
        0x57, 0xb1, // 14: pop; return (record 1's handler)
    ];
    class_of(
        49,
        0x0009, // ACC_PUBLIC | ACC_STATIC
        b"method",
        b"()V",
        CODE,
        2,
        1,
        &[
            ExceptionRecord {
                start_pc: first.0,
                end_pc: first.1,
                handler_pc: 12,
                catch_type: 0,
            },
            ExceptionRecord {
                start_pc: 7,
                end_pc: 10,
                handler_pc: 14,
                catch_type: 0,
            },
        ],
    )
}

/// The report of one run over the body above, through the public entry, up to the names over the
/// frames: the whole pipeline this build implements.
fn mid_block_record_report(first: (u16, u16)) -> MethodAnalysisReport {
    let class = mid_block_record_class(first);
    let fixture = fixture_of(&class, b"method", b"()V");
    let (report, _) = analyze(&fixture, vec![AnalysisStage::Ssa], limits());
    report
}

/// A record's BCI range is a fact about the **sites** it covers, not about the block starts it
/// happens to contain: the canonical graph's handler rows and its exception edges are one fact
/// read once, so a range that starts in the middle of a block still gives its handler the input
/// its throw site hands it.
///
/// The body below is a method a JVM verifier accepts and runs — the `idiv` divides one by zero, the
/// handler it enters discards the caught reference, and both blocks return — so the mid-block shape
/// is legal bytes. What the run used to report about it was a contradiction of those bytes,
/// `ir_frame_inconsistent`, because the row lookup was keyed on the block's *start* while the edge
/// was built from the site's own BCI. The assertion with teeth is the first one: every phase this
/// build implements completes and no diagnostic is raised.
#[test]
fn a_range_starting_inside_a_block_still_hands_its_handler_an_input() {
    let report = mid_block_record_report((1, 4));
    assert_eq!(
        stage_states(&report),
        vec![StageState::Completed; 6],
        "the body is inside every bound it declares and contradicts nothing: {:?}",
        report.diagnostics
    );
    assert!(
        diagnostic_codes(&report).is_empty(),
        "neither the frame phase nor the names over it stopped: {:?}",
        report.diagnostics
    );
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(report.body, MethodBodyState::Present);
    assert_planes_stay_p1(&report);
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
}

/// The contrast of the case above, unchanged by the fix: the same body with record 0's range
/// starting **on** the entry block — the shape a range test has always seen — answers the same
/// way. Read the two runs together: it is the range that moved, not the record's effect.
#[test]
fn a_range_containing_the_block_start_is_still_analyzed() {
    let report = mid_block_record_report((0, 4));
    assert_eq!(
        stage_states(&report),
        vec![StageState::Completed; 6],
        "the range that contains the block start is still analyzed: {:?}",
        report.diagnostics
    );
    assert!(diagnostic_codes(&report).is_empty());
    assert_eq!(report.quality, Quality::Conservative);
    assert_planes_stay_p1(&report);
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
}

/// The R10 shape, first half: a throw site **no handler record covers** is not retained, so the
/// frame pass' bill for this shape is the same however many sites the block holds.
///
/// The body is `T` copies of `iconst_1; iconst_1; idiv; pop` and one `return`, with `max_locals`
/// 10000 and **no** exception table. Every `idiv` is a canonical throw site, so a pass that
/// snapshotted every site would hold `T * max_locals` slots at once while the exit state never
/// changes — the product R10 measured as growing with `T` while the bill did not. Nothing reads
/// those snapshots here: the exception inputs of a block are taken per covered site, and no record
/// covers any of them.
#[test]
fn an_uncovered_throw_site_is_not_retained() {
    const LOCALS: u16 = 10_000;
    let generous = limits_with(|limits| limits.ir_items = 1 << 24);
    let small = throw_sites_class(8, LOCALS, false);
    let large = throw_sites_class(64, LOCALS, false);
    let (frame_small, report_small) =
        items_of(&small, vec![AnalysisStage::Frame], generous.clone());
    let (canonical_small, _) =
        items_of(&small, vec![AnalysisStage::CanonicalCfg], generous.clone());
    let (frame_large, report_large) =
        items_of(&large, vec![AnalysisStage::Frame], generous.clone());
    let (canonical_large, _) = items_of(&large, vec![AnalysisStage::CanonicalCfg], generous);
    let delta_small = frame_small - canonical_small;
    let delta_large = frame_large - canonical_large;

    assert_eq!(
        stage_states(&report_small),
        vec![StageState::Completed; 5],
        "the frames of the 8-site body are derived: {:?}",
        report_small.diagnostics
    );
    assert_eq!(
        stage_states(&report_large),
        vec![StageState::Completed; 5],
        "and so are the frames of the 64-site one: {:?}",
        report_large.diagnostics
    );
    // One block, entered once: the entry state, the working copy that transfer derives its exit
    // in, the exit state the run keeps, one published block record and the table. Nothing per
    // throw site, and above all nothing per (site, local) pair.
    assert_eq!(delta_small, 3 * u64::from(LOCALS) + 2);
    assert_eq!(
        delta_large, delta_small,
        "the bill is the same for 8 sites and for 64: the snapshots no handler reads are not \
         retained, so there is no product for the budget to bound"
    );
    // The probe R10 was measured with: a request allowed the canonical price plus the 10001 items
    // that shape used to cost finishes no more. Both blocks of that bill — the working copy the
    // transfer derives its exit in and the exit state the run keeps — are charged now, so the stop
    // lands inside the frame phase. This is *not* the bounded product: there is no product left to
    // bound here, which is what the two deltas above say.
    let fixture = fixture_of(&small, b"method", b"()V");
    let (report, _) = analyze(
        &fixture,
        vec![AnalysisStage::Ssa],
        limits_with(|limits| limits.ir_items = canonical_small + 10_001),
    );
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
        "the old allowance is not enough for the state that block really holds"
    );
    assert_eq!(diagnostic_codes(&report), vec!["budget_exceeded_ir_items"]);
}

/// The R10 shape, second half: when a handler **does** cover the sites, the block really does hold
/// one snapshot and one exception input per site, and the bill grows with that product.
#[test]
fn a_covered_throw_site_product_is_billed_with_its_size() {
    const LOCALS: u16 = 10_000;
    const SMALL: usize = 8;
    const LARGE: usize = 64;
    let generous = limits_with(|limits| limits.ir_items = 1 << 24);
    let small = throw_sites_class(SMALL, LOCALS, true);
    let large = throw_sites_class(LARGE, LOCALS, true);
    let (frame_small, report_small) =
        items_of(&small, vec![AnalysisStage::Frame], generous.clone());
    let (canonical_small, _) =
        items_of(&small, vec![AnalysisStage::CanonicalCfg], generous.clone());
    let (frame_large, report_large) =
        items_of(&large, vec![AnalysisStage::Frame], generous.clone());
    let (canonical_large, _) = items_of(&large, vec![AnalysisStage::CanonicalCfg], generous);
    let delta_small = frame_small - canonical_small;
    let delta_large = frame_large - canonical_large;

    assert_eq!(
        stage_states(&report_small),
        vec![StageState::Completed; 5],
        "the body the catch-all handler is entered from is analyzed: {:?}",
        report_small.diagnostics
    );
    assert_eq!(stage_states(&report_large), vec![StageState::Completed; 5]);
    // What one covered site costs, by the storage it really makes this block hold: the snapshot the
    // transfer keeps (one item per local slot), the frame the exception edge carries — that many
    // slots plus the one caught reference — one logical input record, and the state that input is
    // merged into, which has the handler's shape and is charged on every input but the first.
    let locals = u64::from(LOCALS);
    let snapshot = locals;
    let input = locals + 1;
    let record = 1;
    let merge = locals + 1;
    // Everything that does not grow with the sites: the entry state, the two working copies the two
    // blocks derive their exits in, the two exit states the run keeps, the two published block
    // records and the table.
    let fixed = 50_004;
    assert_eq!(
        delta_small,
        fixed
            + u64::try_from(SMALL).expect("a small fixture") * (snapshot + input + record)
            + u64::try_from(SMALL - 1).expect("a small fixture") * merge
    );
    // The assertion with the teeth: 56 more covered sites cost 56 more of exactly those four
    // things. A bill that drops any one of them — the snapshot, the input, its record or the merge
    // — cannot produce this number.
    assert_eq!(
        delta_large - delta_small,
        u64::try_from(LARGE - SMALL).expect("a small fixture")
            * (snapshot + input + record + merge),
        "56 more sites of {LOCALS} locals are billed as 56 more products"
    );
}

/// The R10 shape, a stop: a request whose item budget runs out in the middle of the transfer
/// publishes **no** frame table, and the phases before it keep everything they published.
#[test]
fn an_exhausted_item_budget_publishes_no_frame_table() {
    const LOCALS: u16 = 10_000;
    let class = throw_sites_class(8, LOCALS, true);
    let generous = limits_with(|limits| limits.ir_items = 1 << 24);
    let (canonical, _) = items_of(&class, vec![AnalysisStage::CanonicalCfg], generous.clone());
    // The entry state (10000) and the working copy (10000) are charged, and then four of the eight
    // snapshots (40000): the limit is one item past that, so the stop lands in the middle of the
    // transfer, with eight snapshots' worth of state still to build.
    let limit = canonical + 60_001;
    let fixture = fixture_of(&class, b"method", b"()V");
    let (report, budget) = analyze(
        &fixture,
        vec![AnalysisStage::Ssa],
        limits_with(|limits| limits.ir_items = limit),
    );

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
        "the frame phase stopped, so the phase behind it never ran over a half-built table"
    );
    assert_eq!(diagnostic_codes(&report), vec!["budget_exceeded_ir_items"]);
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::IrItems
            },
            ..
        }
    ));
    // The artifact the earlier phases published is still the run's artifact, and the stop is not
    // evidence about the body: no frame table, no names, and nothing the run proved.
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(report.semantic_validation, SemanticValidation::Unproven);
    // The charges that fit are the entry state, the working copy, and four of the eight snapshots;
    // the fifth does not fit, which is what the budget reports, and the usage stays where the
    // accepted charges left it rather than jumping to the limit.
    assert_eq!(budget.usage().ir_items, canonical + 60_000);
    assert!(limit - budget.usage().ir_items < u64::from(LOCALS));
}

/// The same shape, cancelled: a cancellation is not a bound and must not publish a table either.
#[test]
fn a_cancelled_request_publishes_no_frame_table() {
    let class = throw_sites_class(8, 10_000, true);
    let fixture = fixture_of(&class, b"method", b"()V");
    let token = CancellationToken::new();
    token.cancel();
    let mut budget =
        Budget::with_cancellation_token(limits_with(|limits| limits.ir_items = 1 << 24), token);
    let request = MethodAnalysisRequest {
        environment: environment(&fixture),
        method: fixture.method.clone(),
        stages: vec![AnalysisStage::Ssa],
    };
    let report = Engine::new()
        .analyze_method(slice::from_ref(&fixture.snapshot), &request, &mut budget)
        .expect("a cancelled request is answered, not raised");

    assert!(matches!(
        report.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert_eq!(report.quality, Quality::Fallback);
    assert_eq!(
        stage_states(&report)
            .iter()
            .filter(|state| **state == StageState::Completed)
            .count(),
        0,
        "a cancelled run publishes nothing"
    );
    assert_eq!(report.semantic_validation, SemanticValidation::Unproven);
    assert_eq!(
        budget.usage().ir_items,
        0,
        "a cancelled run retained no slot of any frame state"
    );
}

/// The same shape, stopped inside the phase that names the frames: the frames stay the last valid
/// phase, and the names are not published half-built either.
#[test]
fn an_exhausted_item_budget_inside_the_names_phase_publishes_no_names() {
    const LOCALS: u16 = 10_000;
    let class = throw_sites_class(8, LOCALS, true);
    let generous = limits_with(|limits| limits.ir_items = 1 << 24);
    let (frames, _) = items_of(&class, vec![AnalysisStage::Frame], generous.clone());
    let (names, _) = items_of(&class, vec![AnalysisStage::Ssa], generous.clone());
    assert!(
        names > frames,
        "the names phase bills the frames it reads plus its own items: {names} vs {frames}"
    );
    let limit = frames + (names - frames) / 2;
    let fixture = fixture_of(&class, b"method", b"()V");
    let (report, budget) = analyze(
        &fixture,
        vec![AnalysisStage::Ssa],
        limits_with(|limits| limits.ir_items = limit),
    );

    assert_eq!(
        stage_states(&report),
        vec![
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Completed,
            StageState::Partial,
        ],
        "the frames stay the last valid phase when the names stop"
    );
    assert_eq!(diagnostic_codes(&report), vec!["budget_exceeded_ir_items"]);
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::IrItems
            },
            ..
        }
    ));
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::Unproven,
        "a stop inside the names phase is not a completed check of the local invariants"
    );
    assert!(
        budget.usage().ir_items < limit + u64::from(LOCALS),
        "the run stopped where the next charge would not fit: {} of {limit}",
        budget.usage().ir_items
    );
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
    code_attribute_with(code, max_stack, max_locals, &[])
}

/// The same `Code` content with a caller-written `exception_table`.
fn code_attribute_with(
    code: &[u8],
    max_stack: u16,
    max_locals: u16,
    handlers: &[ExceptionRecord],
) -> Vec<u8> {
    let mut content = Vec::new();
    u16_be(&mut content, max_stack);
    u16_be(&mut content, max_locals);
    content.extend_from_slice(
        &u32::try_from(code.len())
            .expect("fixture code fits u32")
            .to_be_bytes(),
    );
    content.extend_from_slice(code);
    u16_be(
        &mut content,
        u16::try_from(handlers.len()).expect("fixture handler count fits u16"),
    );
    for record in handlers {
        u16_be(&mut content, record.start_pc);
        u16_be(&mut content, record.end_pc);
        u16_be(&mut content, record.handler_pc);
        u16_be(&mut content, record.catch_type);
    }
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
    header_at(entries, 52)
}

/// The same header at the class-file version the caller names.
fn header_at(entries: u16, major: u16) -> Vec<u8> {
    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16_be(&mut bytes, 0); // minor
    u16_be(&mut bytes, major);
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
