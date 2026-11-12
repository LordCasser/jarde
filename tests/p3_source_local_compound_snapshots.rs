//! Public Engine tests for source-local snapshots in a pure int assignment.
//! Inputs are generated JVM programs. Production admission cannot depend on their names or BCIs.
use jarde::class_source::ClassSourceReport;
use jarde::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{fs, path::PathBuf, process::Command, slice};

struct Case {
    class: &'static str,
    method: &'static str,
    descriptor: &'static str,
    code: Vec<u8>,
    debug_locals: &'static [(u16, u16, &'static str)],
}
impl Case {
    fn new(class: &'static str, code: &[u8]) -> Self {
        Self {
            class,
            method: "compute",
            descriptor: "()I",
            code: code.to_vec(),
            debug_locals: &[],
        }
    }
}

fn base_code() -> Vec<u8> {
    // int i = 5; i += i++ + 1; return i;
    vec![
        0x08, 0x3b, 0x1a, 0x1a, 0x84, 0, 1, 0x04, 0x60, 0x60, 0x3b, 0x1a, 0xac,
    ]
}
// Retain the exact original failing test input: its unread iinc result prevents an LVT split.
fn unread_update_debug_case() -> Case {
    let mut case = Case::new("SplitSourceIdentity", &base_code());
    case.debug_locals = &[(2, 8, "before"), (11, 2, "after")];
    case
}
fn owned_split_case(with_debug: bool) -> Case {
    let mut case = Case::new(
        if with_debug {
            "OwnedSplitSourceIdentity"
        } else {
            "SourceIdentityMetadataControl"
        },
        // The added load consumes iinc's output before the final Store. Every local write now
        // has a read owner: initial Store and iinc belong to `before`, final Store to `after`.
        &[
            0x08, 0x3b, 0x1a, 0x1a, 0x84, 0, 1, 0x1a, 0x04, 0x60, 0x60, 0x60, 0x3b, 0x1a, 0xac,
        ],
    );
    if with_debug {
        case.debug_locals = &[(2, 10, "before"), (13, 2, "after")];
    }
    case
}

fn positives() -> Vec<Case> {
    let mut renamed = Case::new("RenamedSourceLocal", &base_code());
    renamed.method = "renamedCalculation";
    vec![
        Case::new("SourceLocalCompound", &base_code()),
        unread_update_debug_case(),
        renamed,
        // Changed constants, arithmetic, update direction and all subsequent offsets.
        Case::new(
            "ChangedSourceValues",
            &[
                0x10, 247, 0x3b, 0x1a, 0x1a, 0x84, 0, 253, 0x10, 7, 0x64, 0x68, 0x3b, 0x1a, 0xac,
            ],
        ),
        // Widened local encodings with the original semantics and a different source slot.
        Case::new(
            "WideSourceOffsets",
            &[
                0x10, 5, 0xc4, 0x36, 0, 3, 0xc4, 0x15, 0, 3, 0xc4, 0x15, 0, 3, 0xc4, 0x84, 0, 3, 0,
                1, 0x10, 1, 0x60, 0x60, 0xc4, 0x36, 0, 3, 0xc4, 0x15, 0, 3, 0xac,
            ],
        ),
        // Pure initializer arithmetic overflows in both JVM and Java; no guessed literal folding.
        Case::new(
            "OverflowSourceInitialValue",
            &[
                0x11, 0x7f, 0xff, 0x11, 0x7f, 0xff, 0x68, 0x07, 0x68, 0x3b, 0x1a, 0x1a, 0x84, 0,
                127, 0x11, 0x80, 0, 0x68, 0x60, 0x3b, 0x1a, 0xac,
            ],
        ),
        // The two snapshots denote different SSA local states, separated by the first update.
        Case::new(
            "TwoSourceUpdateEpochs",
            &[
                0x08, 0x3b, 0x1a, 0x84, 0, 1, 0x1a, 0x84, 0, 2, 0x04, 0x60, 0x60, 0x3b, 0x1a, 0xac,
            ],
        ),
    ]
}
fn predecessor_case() -> Case {
    let mut case = Case::new(
        "SourceWithPredecessor",
        &[
            0x1a, 0x99, 0, 5, 0x03, 0xac, 0x08, 0x3c, 0x1b, 0x1b, 0x84, 1, 1, 0x04, 0x60, 0x60,
            0x3c, 0x1b, 0xac,
        ],
    );
    case.descriptor = "(I)I";
    case
}
fn cross_block_case() -> Case {
    let mut case = Case::new(
        "SourceAcrossStackJoin",
        &[
            0x08, 0x3c, 0x1b, 0x1b, 0x1a, 0x99, 0, 6, 0xa7, 0, 3, 0x84, 1, 1, 0x04, 0x60, 0x60,
            0x3c, 0x1b, 0xac,
        ],
    );
    case.descriptor = "(I)I";
    case
}
fn negatives() -> Vec<Case> {
    vec![
        predecessor_case(),
        cross_block_case(),
        Case::new(
            "ThrowingSourceSibling",
            &[
                0x08, 0x3b, 0x1a, 0x1a, 0x84, 0, 1, 0x03, 0x6c, 0x60, 0x3b, 0x1a, 0xac,
            ],
        ),
        Case::new(
            "ConvertedSourceSibling",
            &[
                0x08, 0x3b, 0x1a, 0x1a, 0x84, 0, 1, 0x85, 0x88, 0x04, 0x60, 0x60, 0x3b, 0x1a, 0xac,
            ],
        ),
        Case::new(
            "UnprovedSourceInitializer",
            &[
                0x09, 0x0a, 0x94, 0x3b, 0x1a, 0x1a, 0x84, 0, 1, 0x04, 0x60, 0x60, 0x3b, 0x1a, 0xac,
            ],
        ),
        Case::new(
            "ReassignedSourceLocal",
            &[
                0x08, 0x3b, 0x1a, 0x1a, 0x84, 0, 1, 0x10, 9, 0x3b, 0x04, 0x60, 0x60, 0x3b, 0x1a,
                0xac,
            ],
        ),
        Case::new(
            "UnrelatedLocalDeclaration",
            &[
                0x06, 0x3c, 0x08, 0x3b, 0x1a, 0x1a, 0x84, 0, 1, 0x1b, 0x60, 0x60, 0x3b, 0x1a, 0xac,
            ],
        ),
    ]
}
fn depth_case(class: &'static str, negations: usize) -> Case {
    let mut code = vec![0x08, 0x3b, 0x1a, 0x1a, 0x84, 0, 1, 0x04, 0x60];
    code.extend(std::iter::repeat_n(0x74, negations));
    code.extend([0x60, 0x3b, 0x1a, 0xac]);
    Case::new(class, &code)
}
fn balanced_stable_case(class: &'static str, levels: usize) -> Case {
    fn balanced(code: &mut Vec<u8>, levels: usize) {
        if levels == 0 {
            code.push(0x1a);
        } else {
            balanced(code, levels - 1);
            balanced(code, levels - 1);
            code.push(0x60);
        }
    }
    let mut code = vec![0x08, 0x3b];
    balanced(&mut code, levels);
    code.extend([0x3b, 0x1a, 0xac]);
    Case::new(class, &code)
}
#[test]
fn stable_source_trees_with_32_and_256_leaves_have_no_snapshots() {
    for (name, levels) in [("StableSource32", 5), ("StableSource256", 8)] {
        let report = recover(&balanced_stable_case(name, levels));
        assert_clean(&report);
        assert!(!report.text.contains("int saved"), "{}", report.text);
    }
}
fn stable_case() -> Case {
    Case::new(
        "UnchangedSourceLocal",
        &[0x08, 0x3b, 0x1a, 0x1a, 0x04, 0x60, 0x60, 0x3b, 0x1a, 0xac],
    )
}
fn u16b(out: &mut Vec<u8>, n: u16) {
    out.extend(n.to_be_bytes());
}
fn u32b(out: &mut Vec<u8>, n: u32) {
    out.extend(n.to_be_bytes());
}
fn utf8(out: &mut Vec<u8>, s: &str) {
    out.push(1);
    u16b(out, s.len().try_into().unwrap());
    out.extend(s.as_bytes());
}
fn class_bytes(case: &Case) -> Vec<u8> {
    let mut out = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut out, 0);
    u16b(&mut out, 49);
    let debug = !case.debug_locals.is_empty();
    u16b(
        &mut out,
        if debug {
            10 + case.debug_locals.len() as u16
        } else {
            8
        },
    );
    utf8(&mut out, case.class);
    out.push(7);
    u16b(&mut out, 1);
    utf8(&mut out, "java/lang/Object");
    out.push(7);
    u16b(&mut out, 3);
    utf8(&mut out, case.method);
    utf8(&mut out, case.descriptor);
    utf8(&mut out, "Code");
    if debug {
        utf8(&mut out, "LocalVariableTable"); // 8
        utf8(&mut out, "I"); // 9
        for &(_, _, name) in case.debug_locals {
            utf8(&mut out, name);
        } // 10 onward
    }
    for n in [0x21, 2, 4, 0, 0, 1, 0x09, 5, 6, 1, 7] {
        u16b(&mut out, n);
    }
    let debug_size = if debug {
        8 + 10 * case.debug_locals.len() as u32
    } else {
        0
    };
    u32b(&mut out, 12 + case.code.len() as u32 + debug_size);
    u16b(&mut out, 16);
    u16b(&mut out, 4);
    u32b(&mut out, case.code.len() as u32);
    out.extend(&case.code);
    u16b(&mut out, 0); // exception table
    u16b(&mut out, u16::from(debug)); // Code attributes
    if debug {
        u16b(&mut out, 8);
        u32b(&mut out, 2 + 10 * case.debug_locals.len() as u32);
        u16b(&mut out, case.debug_locals.len() as u16);
        for (i, &(start, length, _)) in case.debug_locals.iter().enumerate() {
            assert!((start as usize + length as usize) <= case.code.len());
            for n in [start, length, 10 + i as u16, 9, 0] {
                u16b(&mut out, n);
            }
        }
    }
    u16b(&mut out, 0); // class attributes
    out
}
fn budget() -> Budget {
    task_budget(&[]).unwrap()
}
fn recover(case: &Case) -> ClassSourceReport {
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(class_bytes(case)), &mut budget())
        .unwrap();
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(case.class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    match engine
        .class_source(slice::from_ref(&snapshot), &request, &mut budget())
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("{}: {other:?}", case.class),
    }
}

fn assert_clean(report: &ClassSourceReport) {
    let text = &report.text;
    assert!(
        !text.contains("not recovered")
            && !text.contains("@bytecode")
            && !text.contains("jarde_refused_body"),
        "{text}"
    );
}
fn assert_saved(case: &Case, report: &ClassSourceReport) {
    assert_clean(report);
    let text = &report.text;
    assert_eq!(
        text.matches("int saved").count(),
        2,
        "{}: {text}",
        case.class
    );
    assert!(
        text.contains("saved0 + (saved1 + 1)")
            || case.class == "ChangedSourceValues"
            || case.class == "OverflowSourceInitialValue"
            || case.class == "SourceDepthAtLimit",
        "{text}"
    );
    assert!(
        text.find("int saved0 =").unwrap() < text.find("int saved1 =").unwrap(),
        "{text}"
    );
}
#[test]
fn renamed_revalued_and_reencoded_source_local_compounds_keep_both_snapshots() {
    for case in positives() {
        assert_saved(&case, &recover(&case));
    }
}
#[test]
fn original_typed_sequence_remains_exact() {
    let text = recover(&Case::new("OriginalSourceSequence", &base_code())).text;
    let statements: Vec<_> = text
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with(';'))
        .collect();
    assert_eq!(
        statements,
        [
            "int local0 = 5;",
            "int saved0 = local0;",
            "int saved1 = local0;",
            "local0 = local0 + 1;",
            "local0 = saved0 + (saved1 + 1);",
            "return local0;"
        ],
        "{text}"
    );
}
#[test]
fn unsupported_siblings_initializers_and_writes_do_not_publish_dead_snapshots() {
    for case in negatives() {
        let text = recover(&case).text;
        assert!(
            text.contains("not recovered")
                || text.contains("@bytecode")
                || text.contains("jarde_refused_body"),
            "{}: {text}",
            case.class
        );
        assert!(!text.contains("int saved"), "{}: {text}", case.class);
    }
}
#[test]
fn unread_update_debug_rows_keep_the_proved_whole_local_sequence() {
    // The focused builder test checks the actual SlotEvidence::Unnamed and the unread iinc
    // definition. This public Engine control preserves the original input, rather than calling
    // two debug rows a proven Split. It also participates in the unedited-class runtime oracle.
    let report = recover(&unread_update_debug_case());
    assert_clean(&report);
    let statements: Vec<_> = report
        .text
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with(';'))
        .collect();
    assert_eq!(
        statements,
        [
            "int local0 = 5;",
            "int saved0 = local0;",
            "int saved1 = local0;",
            "local0 = local0 + 1;",
            "local0 = saved0 + (saved1 + 1);",
            "return local0;",
        ],
        "{}",
        report.text
    );
}

#[test]
fn genuinely_split_source_local_lifetimes_do_not_gain_a_snapshot_proof() {
    // Same instructions without debug identities form a supported whole-local control.
    // The focused test proves that the two-row input actually publishes SlotEvidence::Split.
    let control = recover(&owned_split_case(false));
    assert_clean(&control);
    assert_eq!(
        control.text.matches("int saved").count(),
        2,
        "{}",
        control.text
    );
    assert!(
        control.text.contains("saved0 + (saved1 + (local0 + 1))"),
        "{}",
        control.text
    );
    let text = recover(&owned_split_case(true)).text;
    assert!(
        text.contains("not recovered")
            || text.contains("@bytecode")
            || text.contains("jarde_refused_body"),
        "{text}"
    );
    assert!(!text.contains("int saved"), "{text}");
}

#[test]
fn stable_source_loads_do_not_gain_snapshots() {
    let report = recover(&stable_case());
    assert_clean(&report);
    assert!(!report.text.contains("int saved"), "{}", report.text);
}
#[test]
fn source_local_depth_boundary_checks_the_complete_rhs() {
    let at = depth_case("SourceDepthAtLimit", 22);
    assert_saved(&at, &recover(&at));
    for (name, depth) in [("SourceDepthBeyondLimit", 23), ("SourceDepthSixtyFour", 64)] {
        let text = recover(&depth_case(name, depth)).text;
        assert!(
            text.contains("not recovered")
                || text.contains("@bytecode")
                || text.contains("jarde_refused_body"),
            "{text}"
        );
        assert!(!text.contains("int saved"), "{text}");
    }
}
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jarde-source-local-snapshot-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn run(program: &str, args: &[&str], dir: &std::path::Path) -> Vec<u8> {
    let output = Command::new(program)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{program} {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

#[test]
#[ignore = "needs javac and java on PATH; compiles full unedited Engine class-source output"]
fn full_unedited_source_local_classes_match_original_execution() {
    let scratch = Scratch::new();
    for case in positives().into_iter().chain([
        depth_case("SourceDepthAtLimit", 22),
        stable_case(),
        balanced_stable_case("StableSource32", 5),
        balanced_stable_case("StableSource256", 8),
        owned_split_case(false),
    ]) {
        let original = scratch.0.join(format!("{}-original", case.class));
        let generated = scratch.0.join(format!("{}-generated", case.class));
        fs::create_dir(&original).unwrap();
        fs::create_dir(&generated).unwrap();
        fs::write(
            original.join(format!("{}.class", case.class)),
            class_bytes(&case),
        )
        .unwrap();
        let report = recover(&case);
        assert_clean(&report);
        // Deliberately no comment stripping, body extraction, replacement or wrapper repair.
        fs::write(generated.join(format!("{}.java", case.class)), &report.text).unwrap();
        let harness = format!(
            "public class Replay {{ public static void main(String[] args) {{ System.out.println({}.{}()); }} }}",
            case.class, case.method
        );
        for dir in [&original, &generated] {
            fs::write(dir.join("Replay.java"), &harness).unwrap();
            if dir == &generated {
                run(
                    "javac",
                    &[
                        "--release",
                        "8",
                        &format!("{}.java", case.class),
                        "Replay.java",
                    ],
                    dir,
                );
            } else {
                run("javac", &["--release", "8", "Replay.java"], dir);
            }
        }
        assert_eq!(
            run("java", &["-Xverify:all", "-cp", ".", "Replay"], &original),
            run("java", &["-Xverify:all", "-cp", ".", "Replay"], &generated),
            "{}",
            case.class
        );
    }
}

// Method-level requests make resource exhaustion observable as an atomic absent artifact,
// rather than the class-source surface's intentionally embedded method refusal.
fn method_input(case: &Case) -> (ArtifactSnapshot, MethodAnalysisRequest) {
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(class_bytes(case)), &mut budget())
        .unwrap();
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget(),
            InspectionMode::Strict,
        )
        .unwrap();
    let environment = EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope: PhysicalScope::SnapshotAll,
        policy: EnvironmentPolicy::SingleClass,
        profile: RuntimeProfile {
            java_release: 8,
            multi_release: MultiReleasePolicy::Disabled,
            layout: LayoutMode::Generic,
        },
        loader: LoaderId("app".to_owned()),
    }
    .build(slice::from_ref(&snapshot))
    .unwrap();
    let request = MethodAnalysisRequest {
        environment,
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: inspected.source.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(case.method.as_bytes().to_vec()),
            descriptor: JvmBytes(case.descriptor.as_bytes().to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    (snapshot, request)
}
// Essential has no post-commit evidence work. For this ordinary method, the remaining
// facade assembly is uncharged, so measured N-1 and a late charge stay before source commit.
// Focused builder tests separately target every charge inside the new proof itself.
fn recover_method(
    snapshot: &ArtifactSnapshot,
    request: &MethodAnalysisRequest,
    budget: &mut Budget,
) -> RecoveryReport {
    Engine::new()
        .recover_method_with_evidence(
            slice::from_ref(snapshot),
            SourceTarget::JAVA_8,
            request,
            &RecoveryEvidenceRequest::essential(),
            budget,
        )
        .unwrap()
        .recovery()
        .clone()
}
fn assert_no_artifact(report: &RecoveryReport) {
    assert!(!report.produced(), "{report:?}");
    assert!(report.text.is_empty(), "{}", report.text);
    assert!(report.source_map.is_empty(), "{:?}", report.source_map);
}
#[test]
fn public_source_local_snapshot_source_commit_has_exact_resource_boundaries() {
    use jarde::budget::{BudgetDimension, CountedBudgetDimension};
    let case = depth_case("MeteredSourceLocalSnapshot", 22);
    let (snapshot, request) = method_input(&case);
    let mut measured = budget();
    let full = recover_method(&snapshot, &request, &mut measured);
    assert!(
        full.produced() && full.text.contains("int saved0 = local0;"),
        "{full:?}"
    );
    for (counted, stopped_dimension) in [
        (
            CountedBudgetDimension::AnalysisSteps,
            BudgetDimension::AnalysisSteps,
        ),
        (CountedBudgetDimension::IrItems, BudgetDimension::IrItems),
        (
            CountedBudgetDimension::OutputBytes,
            BudgetDimension::OutputBytes,
        ),
    ] {
        let needed = measured.usage().counted_usage(counted);
        assert!(needed > 0, "{counted:?} is charged");
        for limit in [needed, needed - 1] {
            let mut limits = task_limits(&[]).unwrap();
            match counted {
                CountedBudgetDimension::AnalysisSteps => limits.analysis_steps = limit,
                CountedBudgetDimension::IrItems => limits.ir_items = limit,
                CountedBudgetDimension::OutputBytes => limits.output_bytes = limit,
                _ => unreachable!(),
            }
            let mut bounded = Budget::new(limits);
            let report = recover_method(&snapshot, &request, &mut bounded);
            if limit == needed {
                assert!(report.produced(), "{counted:?} at N={needed}: {report:?}");
                assert_eq!(report.text, full.text);
                assert_eq!(bounded.usage().counted_usage(counted), needed);
            } else {
                assert_no_artifact(&report);
                assert!(
                    matches!(report.stop(), Some(StopReason::Budget { dimension, .. })
                    if *dimension == stopped_dimension),
                    "{counted:?} at N-1: {report:?}"
                );
            }
        }
    }
}

#[derive(Debug)]
struct CancelOnCharge {
    charges: AtomicU64,
    cancel_at: Option<u64>,
    token: jarde::budget::CancellationToken,
}
impl jarde_reader::ledger::LedgerObserver for CancelOnCharge {
    fn entry(
        &self,
        site: jarde_reader::ledger::LedgerSite,
        _: Option<jarde_reader::ledger::LedgerEntryTiming>,
    ) {
        if matches!(site, jarde_reader::ledger::LedgerSite::Charge(_)) {
            let charge = self.charges.fetch_add(1, Ordering::Relaxed) + 1;
            if self.cancel_at == Some(charge) {
                self.token.cancel();
            }
        }
    }
    fn refusal(
        &self,
        _: jarde_reader::ledger::LedgerSite,
        _: Option<jarde::budget::BudgetDimension>,
    ) {
    }
}
#[test]
fn public_source_local_snapshot_essential_late_cancellation_leaves_no_artifact() {
    use jarde_reader::ledger::{OperationLedger, UsageOwner};
    use std::sync::Arc;
    let case = depth_case("CancelledSourceLocalSnapshot", 22);
    let (snapshot, request) = method_input(&case);
    let token = jarde::budget::CancellationToken::new();
    let observer = Arc::new(CancelOnCharge {
        charges: AtomicU64::new(0),
        cancel_at: None,
        token: token.clone(),
    });
    let mut measured = Budget::with_cancellation_token(task_limits(&[]).unwrap(), token);
    measured.with_ledger(
        OperationLedger::new(&measured).with_observer(observer.clone()),
        UsageOwner::Methods,
    );
    let full = recover_method(&snapshot, &request, &mut measured);
    assert!(
        full.produced() && full.text.contains("int saved0 = local0;"),
        "{full:?}"
    );
    let charges = observer.charges.load(Ordering::Relaxed);
    assert!(
        charges > 10,
        "the public recovery has a meaningful late cancellation point"
    );
    // The penultimate charge has a subsequent budget operation, so cancellation cannot be
    // mistaken for pre-cancel or an unobserved signal after the final operation completed.
    // With essential evidence, this is a pre-commit stop. Public ledger sites do not name
    // proof helpers: helper-specific interruption is covered by the focused builder tests.
    let cancel_at = charges - 1;
    let token = jarde::budget::CancellationToken::new();
    let observer = Arc::new(CancelOnCharge {
        charges: AtomicU64::new(0),
        cancel_at: Some(cancel_at),
        token: token.clone(),
    });
    let mut cancelled = Budget::with_cancellation_token(task_limits(&[]).unwrap(), token);
    cancelled.with_ledger(
        OperationLedger::new(&cancelled).with_observer(observer.clone()),
        UsageOwner::Methods,
    );
    let report = recover_method(&snapshot, &request, &mut cancelled);
    assert!(observer.charges.load(Ordering::Relaxed) >= cancel_at);
    assert!(cancelled.usage().analysis_steps > 0);
    assert_no_artifact(&report);
    assert!(
        matches!(report.stop(), Some(StopReason::Cancelled { .. })),
        "{report:?}"
    );
}

#[test]
fn public_source_local_snapshot_all_evidence_stop_keeps_the_committed_text() {
    use jarde::budget::BudgetDimension;
    let case = depth_case("EvidenceStoppedSourceLocalSnapshot", 22);
    let (snapshot, request) = method_input(&case);
    let run = |budget: &mut Budget| {
        Engine::new()
            .recover_method_with_evidence(
                slice::from_ref(&snapshot),
                SourceTarget::JAVA_8,
                &request,
                &RecoveryEvidenceRequest::all(),
                budget,
            )
            .unwrap()
            .recovery()
            .clone()
    };
    let mut measured = budget();
    let full = run(&mut measured);
    assert!(
        full.produced() && full.text.contains("int saved0 = local0;"),
        "{full:?}"
    );
    assert!(matches!(full.execution, ExecutionReport::Complete { .. }));
    assert!(matches!(
        full.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete
    ));
    let mut limits = task_limits(&[]).unwrap();
    limits.ir_items = measured.usage().ir_items - 1;
    let partial = run(&mut Budget::new(limits));
    assert!(
        partial.produced(),
        "optional evidence cannot retract committed text: {partial:?}"
    );
    assert_eq!(partial.text, full.text);
    assert!(
        partial.stop().is_none(),
        "artifact outcome remains Produced"
    );
    assert!(
        matches!(
            partial.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::IrItems
                },
                ..
            }
        ),
        "{partial:?}"
    );
    assert!(partial.source_map.len() < full.source_map.len());
    assert!(
        matches!(
            partial.evidence.state(RecoveryEvidenceKind::SourceMap),
            EvidenceState::Partial { .. }
        ),
        "{partial:?}"
    );
    // The ordinary Engine method path has no reusable AST sidecar. Withdrawal of physical
    // and declared ASTs after this kind of stop is already checked by jarde-java's
    // s2b_declared_value_types_r1::constructor_witness_retention_obeys_whole_recovery_ir_budget.
}

#[test]
fn source_local_neighbors_have_real_canonical_and_ssa_boundaries() {
    use jarde_jvm::method_ir::{Definition, Slot};
    enum Boundary {
        Entry,
        Predecessor,
        CrossBlock,
    }
    for (case, boundary) in [
        (
            Case::new("SourceStructuralControl", &base_code()),
            Boundary::Entry,
        ),
        (predecessor_case(), Boundary::Predecessor),
        (cross_block_case(), Boundary::CrossBlock),
    ] {
        let (snapshot, request) = method_input(&case);
        let analyzed = jarde_jvm::engine::analyze_method_ir(
            slice::from_ref(&snapshot),
            &request,
            &mut budget(),
        )
        .unwrap();
        let canonical = analyzed.ir().canonical().unwrap();
        let ssa = analyzed.ir().ssa().unwrap();
        let slot = u16::from(case.descriptor == "(I)I");
        let load_opcode = if slot == 0 { 0x1a } else { 0x1b };
        let store_opcode = if slot == 0 { 0x3b } else { 0x3c };
        let (load_block, load) = ssa
            .blocks()
            .iter()
            .flat_map(|block| {
                block
                    .instructions()
                    .iter()
                    .map(move |instruction| (block, instruction))
            })
            .filter(|(_, instruction)| instruction.opcode() == load_opcode)
            .min_by_key(|(_, instruction)| instruction.bci())
            .unwrap();
        let (store_block, store) = ssa
            .blocks()
            .iter()
            .flat_map(|block| {
                block
                    .instructions()
                    .iter()
                    .map(move |instruction| (block, instruction))
            })
            .filter(|(_, instruction)| instruction.opcode() == store_opcode)
            .max_by_key(|(_, instruction)| instruction.bci())
            .unwrap();
        let [(Slot::Local(read_slot), old)] = load.reads() else {
            panic!("exact local load")
        };
        assert_eq!(*read_slot, slot);
        assert!(
            matches!(ssa.value(*old).def(), Definition::Instruction { block, bci }
            if block == load_block.block() && *bci < load.bci()),
            "source local is Store-defined, never an Entry parameter"
        );
        match boundary {
            Boundary::Entry => {
                assert_eq!(canonical.blocks().len(), 1);
                assert_eq!(load_block.block().bci(), 0);
                assert_eq!(store_block.block(), load_block.block());
                assert!(canonical.edges().is_empty());
                let last_use = ssa.value(*old).uses().last().unwrap();
                let increment = load_block
                    .instructions()
                    .iter()
                    .find(|instruction| Some(instruction.bci()) == last_use.bci())
                    .unwrap();
                assert_eq!(increment.opcode(), 0x84);
                assert_eq!(increment.reads(), &[(Slot::Local(slot), *old)]);
                assert!(load.bci() < increment.bci() && increment.bci() < store.bci());
                assert_eq!(
                    ssa.value(*old).uses().len(),
                    3,
                    "both old loads and iinc read the same original local identity"
                );
            }
            Boundary::Predecessor => {
                assert_ne!(load_block.block().bci(), 0);
                assert_eq!(store_block.block(), load_block.block());
                assert!(
                    canonical
                        .edges()
                        .iter()
                        .any(|edge| edge.to() == load_block.block())
                );
            }
            Boundary::CrossBlock => {
                assert_ne!(store_block.block(), load_block.block());
                assert!(
                    canonical
                        .edges()
                        .iter()
                        .filter(|edge| edge.to() == store_block.block())
                        .count()
                        >= 2
                );
                assert!(
                    store_block
                        .entry()
                        .iter()
                        .any(|(slot, _)| matches!(slot, Slot::Stack(_))),
                    "the join really carries an old operand stack value"
                );
            }
        }
    }
}
