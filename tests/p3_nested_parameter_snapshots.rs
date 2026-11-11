//! Public Engine coverage for the bounded, single-entry parameter snapshot proof.
//! All fixtures are generated from JVM instructions, not patched compiler fingerprints.
use jarde::class_source::ClassSourceReport;
use jarde::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{fs, path::PathBuf, process::Command, slice};

struct Case {
    class: &'static str,
    method: &'static str,
    descriptor: &'static str,
    code: Vec<u8>,
    // Optional int slot-0 debug lifetimes: (start_pc, length, source name).
    debug_locals: &'static [(u16, u16, &'static str)],
}
impl Case {
    fn int(class: &'static str, code: &[u8]) -> Self {
        Self {
            class,
            method: "compute",
            descriptor: "(II)I",
            code: code.to_vec(),
            debug_locals: &[],
        }
    }
}

// A forward, single-successor/unique-predecessor goto is normalized into the
// entry superblock. It changes physical offsets, not this proof's block boundary.
fn fused_prefix_case() -> Case {
    Case::int(
        "FusedPrefixSnapshot",
        &[0xa7, 0, 3, 0x1a, 0x04, 0x60, 0x84, 0, 1, 0x1a, 0x60, 0xac],
    )
}
fn fused_stack_case() -> Case {
    Case::int(
        "FusedStackSnapshot",
        &[0x1a, 0x04, 0x60, 0xa7, 0, 3, 0x84, 0, 1, 0x1a, 0x60, 0xac],
    )
}
fn predecessor_case() -> Case {
    // The conditional entry has two successors: an early return and the snapshot
    // body. The latter has a genuine predecessor but cannot fuse into the entry.
    Case::int(
        "PredecessorBlock",
        &[
            0x1b, 0x99, 0, 5, 0x03, 0xac, 0x1a, 0x04, 0x60, 0x84, 0, 1, 0x1a, 0x60, 0xac,
        ],
    )
}
fn cross_block_stack_case() -> Case {
    // Both conditional edges carry the old sum into a two-predecessor join.
    // Unlike a solitary goto, neither edge can be fused away by canonical_cfg.
    Case::int(
        "CrossBlockStack",
        &[
            0x1a, 0x04, 0x60, 0x1b, 0x99, 0, 6, 0xa7, 0, 3, 0x84, 0, 1, 0x1a, 0x60, 0xac,
        ],
    )
}

fn positives() -> Vec<Case> {
    vec![
        fused_prefix_case(),
        fused_stack_case(),
        Case::int(
            "RenamedSnapshot",
            &[0x1a, 0x04, 0x60, 0x84, 0, 1, 0x1a, 0x60, 0xac],
        ),
        Case {
            class: "ChangedOffsets",
            method: "differentName",
            descriptor: "(II)I",
            debug_locals: &[],
            code: vec![0x15, 0, 0x10, 7, 0x64, 0x84, 0, 3, 0x15, 0, 0x68, 0xac],
        },
        Case::int(
            "WideOperands",
            &[
                0xc4, 0x15, 0, 0, 0x11, 0, 31, 0x60, 0xc4, 0x84, 0, 0, 0, 5, 0xc4, 0x15, 0, 0,
                0x64, 0xac,
            ],
        ),
        Case::int(
            "SiblingParameter",
            &[0x1a, 0x04, 0x60, 0x84, 0, 1, 0x1b, 0x05, 0x68, 0x60, 0xac],
        ),
        Case::int(
            "PureSiblingStore",
            &[
                0x1a, 0x04, 0x60, 0x84, 0, 1, 0x1b, 0x05, 0x68, 0x3c, 0x1b, 0x60, 0xac,
            ],
        ),
        Case::int(
            "NestedArithmetic",
            &[
                0x1a, 0x04, 0x60, 0x05, 0x68, 0x74, 0x84, 0, 1, 0x1a, 0x1b, 0x06, 0x64, 0x68, 0x64,
                0xac,
            ],
        ),
        Case {
            class: "LongSnapshot",
            method: "calculate",
            descriptor: "(JJ)J",
            debug_locals: &[],
            code: vec![0x1e, 0x0a, 0x61, 0x1e, 0x0a, 0x61, 0x3f, 0x1e, 0x65, 0xad],
        },
    ]
}

fn stable_case() -> Case {
    // Both parameter loads are indirect operands, but neither parameter changes.
    Case::int(
        "StableNestedLoads",
        &[0x1a, 0x04, 0x60, 0x1b, 0x05, 0x68, 0x60, 0xac],
    )
}
fn assert_no_snapshot(report: &ClassSourceReport) {
    let text = &report.text;
    assert!(
        !text.contains("saved0")
            && !text.contains("not recovered")
            && !text.contains("@bytecode")
            && !text.contains("jarde_refused_body"),
        "{text}"
    );
}
fn assert_stable(report: &ClassSourceReport) {
    assert_no_snapshot(report);
    assert!(
        report.text.contains("return arg0 + 1 + arg1 * 2;"),
        "{}",
        report.text
    );
}
fn balanced_stable_case() -> Case {
    fn balanced(code: &mut Vec<u8>, levels: usize) {
        if levels == 0 {
            code.push(0x1a);
        } else {
            balanced(code, levels - 1);
            balanced(code, levels - 1);
            code.push(0x60);
        }
    }
    let mut code = Vec::new();
    balanced(&mut code, 8);
    code.push(0xac);
    Case::int("BalancedStableLoads", &code)
}
#[test]
fn long_balanced_stable_parameter_tree_has_no_snapshots() {
    assert_no_snapshot(&recover(&balanced_stable_case()));
}
#[test]
fn stable_indirect_parameter_loads_recover_without_snapshots() {
    assert_stable(&recover(&stable_case()));
}

fn negatives() -> Vec<Case> {
    let base = [0x1a, 0x04, 0x60, 0x84, 0, 1, 0x1a, 0x60, 0xac];
    let mut branch = vec![0x1b, 0x99, 0, 7, 0x84, 1, 1, 0x00];
    branch.extend(base);
    let mut phi = vec![0x1b, 0x99, 0, 8, 0x04, 0x3b, 0xa7, 0, 5, 0x05, 0x3b];
    phi.extend(base);
    vec![
        Case::int(
            "DivisionSibling",
            &[0x1a, 0x04, 0x60, 0x84, 0, 1, 0x1b, 0x05, 0x6c, 0x60, 0xac],
        ),
        Case::int(
            "RemainderSibling",
            &[0x1a, 0x04, 0x60, 0x84, 0, 1, 0x1b, 0x05, 0x70, 0x60, 0xac],
        ),
        Case::int(
            "TwoStaleLoads",
            &[0x1a, 0x1a, 0x60, 0x84, 0, 1, 0x1a, 0x60, 0xac],
        ),
        Case::int(
            "SourceLocalSibling",
            &[0x06, 0x3d, 0x1a, 0x04, 0x60, 0x84, 0, 1, 0x1c, 0x60, 0xac],
        ),
        // lcmp produces an int but is outside the permitted Store RHS grammar.
        Case::int(
            "UnsupportedParameterStore",
            &[
                0x1a, 0x04, 0x60, 0x84, 0, 1, 0x09, 0x0a, 0x94, 0x3c, 0x1b, 0x60, 0xac,
            ],
        ),
        predecessor_case(),
        Case::int("BranchJoin", &branch),
        Case::int("PhiJoin", &phi),
        Case::int(
            "ConvertedSibling",
            &[0x1a, 0x04, 0x60, 0x84, 0, 1, 0x1b, 0x85, 0x88, 0x60, 0xac],
        ),
        cross_block_stack_case(),
        // The sibling is loaded before its parameter changes, too.
        Case::int(
            "StaleSibling",
            &[0x1a, 0x04, 0x60, 0x1b, 0x84, 1, 1, 0x84, 0, 1, 0x60, 0xac],
        ),
    ]
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
fn assert_recovered(case: &Case, report: &ClassSourceReport) {
    let text = &report.text;
    let kind = if case.descriptor == "(JJ)J" {
        "long"
    } else {
        "int"
    };
    let saved = format!("{kind} saved0 = arg0;");
    assert_eq!(text.matches(&saved).count(), 1, "{}: {text}", case.class);
    assert!(
        !text.contains("not recovered")
            && !text.contains("@bytecode")
            && !text.contains("jarde_refused_body"),
        "{}: {text}",
        case.class
    );
    assert!(
        text.find(&saved).unwrap() < text.find("arg0 = arg0 +").unwrap(),
        "{text}"
    );
}
#[test]
fn renamed_changed_values_and_widened_offsets_recover_typed_snapshots() {
    for case in positives() {
        assert_recovered(&case, &recover(&case));
    }
}
#[test]
fn impure_stale_local_and_control_flow_neighbors_remain_refused() {
    let mut failures = Vec::new();
    for case in negatives() {
        let report = recover(&case);
        let refused = report.text.contains("not recovered")
            || report.text.contains("@bytecode")
            || report.text.contains("jarde_refused_body");
        let no_saved = !report.text.contains("saved0 =");
        eprintln!("{}: refused={refused}, no_saved={no_saved}", case.class);
        // Preserve both original assertions, but report every failing neighbor together.
        if !refused {
            failures.push(format!(
                "{}: expected refusal:\n{}",
                case.class, report.text
            ));
        }
        if !no_saved {
            failures.push(format!(
                "{}: unexpected saved declaration:\n{}",
                case.class, report.text
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn int_shaped_non_int_header_parameters_do_not_gain_snapshots() {
    for (name, descriptor) in [
        ("BooleanHeader", "(ZI)I"),
        ("ByteHeader", "(BI)I"),
        ("ShortHeader", "(SI)I"),
        ("CharHeader", "(CI)I"),
    ] {
        let mut case = Case::int(name, &[0x1a, 0x04, 0x60, 0x84, 0, 1, 0x1a, 0x60, 0xac]);
        case.descriptor = descriptor;
        let text = recover(&case).text;
        assert!(
            text.contains("not recovered")
                || text.contains("@bytecode")
                || text.contains("jarde_refused_body"),
            "{name}: {text}"
        );
        assert!(!text.contains("saved0 ="), "{name}: {text}");
    }
}
#[test]
fn split_store_defined_lifetimes_do_not_prove_one_header_parameter_identity() {
    let mut case = Case::int(
        "SplitParameterIdentity",
        &[0x06, 0x3b, 0x1a, 0x04, 0x60, 0x07, 0x3b, 0x1a, 0x60, 0xac],
    );
    // Both loads are Store-defined: load 2 belongs to Store 1 and the first
    // lifetime; load 7 belongs to Store 6 and the second lifetime. Neither reads
    // an Entry value, so the actual reuse::Split proof can classify these rows.
    case.debug_locals = &[(2, 5, "before"), (7, 3, "after")];
    let text = recover(&case).text;
    assert!(
        text.contains("not recovered")
            || text.contains("@bytecode")
            || text.contains("jarde_refused_body"),
        "{text}"
    );
    assert!(!text.contains("saved0 ="), "{text}");
}

// The stale leaf sits under the initial add and the terminal add. Thus 22
// intervening negations give leaf depth 24; 23 give depth 25.
fn depth_case(class: &'static str, negations: usize) -> Case {
    let mut code = vec![0x1a, 0x04, 0x60];
    code.extend(std::iter::repeat_n(0x74, negations));
    code.extend([0x84, 0, 1, 0x1a, 0x60, 0xac]);
    Case::int(class, &code)
}
#[test]
fn depth_24_leaf_is_the_accepted_boundary() {
    let case = depth_case("DepthAtLimit", 22);
    assert_recovered(&case, &recover(&case));
}
#[test]
fn depth_25_leaf_exceeds_the_accepted_boundary() {
    let case = depth_case("DepthBeyondLimit", 23);
    let text = recover(&case).text;
    assert!(
        text.contains("not recovered")
            || text.contains("@bytecode")
            || text.contains("jarde_refused_body"),
        "{text}"
    );
    assert!(!text.contains("saved0 ="), "{text}");
}
#[test]
fn depth_64_refuses_on_the_default_test_thread_stack() {
    // No custom thread or enlarged stack: bounded rejection must happen before recursion grows.
    let case = depth_case("DepthSixtyFour", 64);
    let text = recover(&case).text;
    assert!(
        text.contains("not recovered")
            || text.contains("@bytecode")
            || text.contains("jarde_refused_body"),
        "{text}"
    );
    assert!(!text.contains("saved0 ="), "{text}");
}

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jarde-nested-snapshot-{}-{}",
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
#[ignore = "needs javac and java on PATH; compiles unedited Engine class-source output"]
fn complete_unedited_generated_classes_match_original_execution() {
    let scratch = Scratch::new();
    for case in positives().into_iter().chain([
        depth_case("DepthAtLimit", 22),
        stable_case(),
        balanced_stable_case(),
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
        if case.class == "StableNestedLoads" {
            assert_stable(&report);
        } else if case.class == "BalancedStableLoads" {
            assert_no_snapshot(&report);
        } else {
            assert_recovered(&case, &report);
        }
        // Deliberately no comment stripping, body extraction, replacement or wrapper repair.
        fs::write(generated.join(format!("{}.java", case.class)), &report.text).unwrap();
        let ty = if case.descriptor == "(JJ)J" {
            "long"
        } else {
            "int"
        };
        let values = if ty == "long" {
            "Long.MIN_VALUE, Long.MIN_VALUE + 1, -10000000001L, -7L, -1L, 0L, 1L, 7L, 10000000001L, Long.MAX_VALUE"
        } else {
            "Integer.MIN_VALUE, Integer.MIN_VALUE + 1, -100001, -7, -1, 0, 1, 7, 100001, Integer.MAX_VALUE"
        };
        let harness = format!(
            "public class Replay {{ public static void main(String[] args) {{ {ty}[] xs = {{{values}}}; for ({ty} a : xs) for ({ty} b : xs) System.out.println({}.{}(a, b)); }} }}",
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
        let expected = run("java", &["-Xverify:all", "-cp", ".", "Replay"], &original);
        let actual = run("java", &["-Xverify:all", "-cp", ".", "Replay"], &generated);
        assert_eq!(actual, expected, "{}: {}", case.class, report.text);
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
fn public_snapshot_source_commit_has_exact_resource_boundaries() {
    use jarde::budget::{BudgetDimension, CountedBudgetDimension};
    let case = depth_case("MeteredNestedSnapshot", 22);
    let (snapshot, request) = method_input(&case);
    let mut measured = budget();
    let full = recover_method(&snapshot, &request, &mut measured);
    assert!(
        full.produced() && full.text.contains("int saved0 = arg0;"),
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
fn public_snapshot_essential_late_cancellation_leaves_no_artifact() {
    use jarde_reader::ledger::{OperationLedger, UsageOwner};
    use std::sync::Arc;
    let case = depth_case("CancelledNestedSnapshot", 22);
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
        full.produced() && full.text.contains("int saved0 = arg0;"),
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
fn public_snapshot_all_evidence_stop_keeps_the_committed_text() {
    use jarde::budget::BudgetDimension;
    let case = depth_case("EvidenceStoppedNestedSnapshot", 22);
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
        full.produced() && full.text.contains("int saved0 = arg0;"),
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
fn control_flow_neighbors_have_the_claimed_canonical_boundaries() {
    use jarde_jvm::method_ir::Slot;
    enum Boundary {
        Fused,
        Predecessor,
        CrossBlock,
    }
    for (case, boundary) in [
        (fused_prefix_case(), Boundary::Fused),
        (fused_stack_case(), Boundary::Fused),
        (predecessor_case(), Boundary::Predecessor),
        (cross_block_stack_case(), Boundary::CrossBlock),
    ] {
        let (snapshot, request) = method_input(&case);
        // Inspect the actual immutable IR produced by the same analysis entry Engine
        // recovery calls; no graph or SSA table is manufactured from expected labels.
        let analyzed = jarde_jvm::engine::analyze_method_ir(
            slice::from_ref(&snapshot),
            &request,
            &mut budget(),
        )
        .unwrap();
        let canonical = analyzed.ir().canonical().unwrap();
        let ssa = analyzed.ir().ssa().unwrap();
        let (load_block, _) = ssa
            .blocks()
            .iter()
            .flat_map(|block| {
                block
                    .instructions()
                    .iter()
                    .map(move |instruction| (block, instruction))
            })
            .filter(|(_, instruction)| instruction.opcode() == 0x1a)
            .min_by_key(|(_, instruction)| instruction.bci())
            .unwrap();
        let (return_block, _) = ssa
            .blocks()
            .iter()
            .flat_map(|block| {
                block
                    .instructions()
                    .iter()
                    .map(move |instruction| (block, instruction))
            })
            .filter(|(_, instruction)| instruction.opcode() == 0xac)
            .max_by_key(|(_, instruction)| instruction.bci())
            .unwrap();
        match boundary {
            Boundary::Fused => {
                assert_eq!(canonical.blocks().len(), 1, "{}: {canonical:?}", case.class);
                let block = &canonical.blocks()[0];
                assert_eq!(block.id().bci(), 0);
                assert!(block.blocks().len() > 1, "the raw blocks really fused");
                assert_eq!(load_block.block(), block.id());
                assert_eq!(return_block.block(), block.id());
                assert!(!canonical.edges().iter().any(|edge| edge.to() == block.id()));
            }
            Boundary::Predecessor => {
                assert_ne!(load_block.block().bci(), 0);
                assert_eq!(return_block.block(), load_block.block());
                assert!(
                    canonical
                        .edges()
                        .iter()
                        .any(|edge| edge.to() == load_block.block())
                );
            }
            Boundary::CrossBlock => {
                assert_ne!(return_block.block(), load_block.block());
                assert!(
                    canonical
                        .edges()
                        .iter()
                        .filter(|edge| edge.to() == return_block.block())
                        .count()
                        >= 2
                );
                assert!(
                    return_block
                        .entry()
                        .iter()
                        .any(|(slot, _)| matches!(slot, Slot::Stack(_))),
                    "the join actually carries an operand stack value"
                );
            }
        }
    }
}
