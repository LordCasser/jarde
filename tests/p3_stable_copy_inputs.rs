//! Stable Load proofs must remain available to the unclaimed stack-copy renderer.
//! Every input goes through Engine. The optional replay compiles each entire, unedited
//! class-source artifact and compares it with verifier execution of the original class.
use jarde::class_source::ClassSourceReport;
use jarde::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{fs, path::PathBuf, process::Command, slice};

struct Case {
    class: &'static str,
    method: &'static str,
    descriptor: &'static str,
    code: Vec<u8>,
    copies: &'static [u32],
    saved: bool,
}
impl Case {
    fn new(
        class: &'static str,
        descriptor: &'static str,
        code: &[u8],
        copies: &'static [u32],
    ) -> Self {
        Self {
            class,
            method: "compute",
            descriptor,
            code: code.to_vec(),
            copies,
            saved: false,
        }
    }
}
fn array_case() -> Case {
    let mut case = Case::new(
        "StableArrayCopy",
        "([III)J",
        &[0x2a, 0x1b, 0x5c, 0x2e, 0x1c, 0x60, 0x5b, 0x4f, 0x85, 0xad],
        &[2, 6],
    );
    // The sum, unlike the stable array/index Loads, needs one evaluation before store and return.
    case.saved = true;
    case
}
fn positives() -> Vec<Case> {
    let mut changed = Case::new(
        "RenamedShiftedCopy",
        "(II)I",
        &[0x15, 0, 0x59, 0x15, 1, 0x10, 7, 0x68, 0x60, 0x60, 0xac],
        &[2],
    );
    changed.method = "renamed";
    let mut shifted_array = Case::new(
        "WideArrayCopy",
        "([III)J",
        &[
            0xc4, 0x19, 0, 0, 0xc4, 0x15, 0, 1, 0x5c, 0x2e, 0xc4, 0x15, 0, 2, 0x10, 7, 0x60, 0x60,
            0x5b, 0x4f, 0x85, 0xad,
        ],
        &[8, 18],
    );
    shifted_array.method = "changed";
    shifted_array.saved = true;
    vec![
        Case::new(
            "StableSingleCopy",
            "(II)I",
            &[0x1a, 0x59, 0x1b, 0x60, 0x60, 0xac],
            &[1],
        ),
        changed,
        Case::new(
            "StablePairCopy",
            "(II)I",
            &[0x1a, 0x1b, 0x5c, 0x60, 0x60, 0x60, 0xac],
            &[2],
        ),
        Case::new("StableLongCopy", "(J)J", &[0x1e, 0x5c, 0x61, 0xad], &[1]),
        Case::new("StableDoubleCopy", "(D)D", &[0x26, 0x5c, 0x63, 0xaf], &[1]),
        Case::new(
            "StableReferenceCopy",
            "(Ljava/lang/Object;)I",
            &[0x2a, 0x59, 0xa6, 0, 5, 0x04, 0xac, 0x03, 0xac],
            &[1],
        ),
        array_case(),
        shifted_array,
    ]
}
fn changed_slots() -> Vec<Case> {
    let mut cases = vec![
        Case::new(
            "ChangedIntCopy",
            "(II)I",
            &[0x1a, 0x59, 0x84, 0, 1, 0x60, 0x1a, 0x60, 0xac],
            &[1],
        ),
        Case::new(
            "ChangedLongCopy",
            "(J)J",
            &[0x1e, 0x5c, 0x0a, 0x3f, 0x61, 0x1e, 0x61, 0xad],
            &[1],
        ),
        Case::new(
            "ChangedReferenceCopy",
            "(Ljava/lang/Object;)I",
            &[0x2a, 0x59, 0x01, 0x4b, 0xa6, 0, 5, 0x04, 0xac, 0x03, 0xac],
            &[1],
        ),
    ];
    for case in &mut cases {
        case.saved = true;
    }
    cases
}
fn negatives() -> Vec<Case> {
    vec![
        // Both conditional successors carry the copied stack into the two-predecessor join.
        Case::new(
            "CrossBlockCopy",
            "(II)I",
            &[0x1a, 0x59, 0x1b, 0x99, 0, 6, 0xa7, 0, 3, 0x60, 0xac],
            &[1],
        ),
        // One copy escapes to an unsupported terminal. A closed reader proof must fail.
        Case::new(
            "DiscardedCopyConsumer",
            "(II)I",
            &[0x1a, 0x59, 0x57, 0xac],
            &[1],
        ),
        // A failed sibling consumer must not publish an intervening mutation on its own.
        Case::new(
            "PartialCopyConsumer",
            "(II)I",
            &[0x1a, 0x59, 0x84, 1, 1, 0x57, 0xac],
            &[1],
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
    // Version 49 requires no manufactured StackMapTable and is verifier-executable.
    let mut out = 0xcafebabe_u32.to_be_bytes().to_vec();
    for n in [0, 49, 8] {
        u16b(&mut out, n);
    }
    utf8(&mut out, case.class);
    out.push(7);
    u16b(&mut out, 1);
    utf8(&mut out, "java/lang/Object");
    out.push(7);
    u16b(&mut out, 3);
    utf8(&mut out, case.method);
    utf8(&mut out, case.descriptor);
    utf8(&mut out, "Code");
    for n in [0x21, 2, 4, 0, 0, 1, 0x09, 5, 6, 1, 7] {
        u16b(&mut out, n);
    }
    u32b(&mut out, 12 + case.code.len() as u32);
    for n in [16, 4] {
        u16b(&mut out, n);
    }
    u32b(&mut out, case.code.len() as u32);
    out.extend(&case.code);
    for n in [0, 0, 0] {
        u16b(&mut out, n);
    }
    out
}
fn budget() -> Budget {
    task_budget(&[]).unwrap()
}
fn recover_bytes(bytes: &[u8], class: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .unwrap();
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
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
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("{class}: {other:?}"),
    }
}
fn recover(case: &Case) -> ClassSourceReport {
    recover_bytes(&class_bytes(case), case.class)
}
fn no_refusal(text: &str) {
    assert!(
        !text.contains("not recovered")
            && !text.contains("@bytecode")
            && !text.contains("jarde_refused_body"),
        "{text}"
    );
}
fn assert_supported(case: &Case, report: &ClassSourceReport) {
    no_refusal(&report.text);
    assert_eq!(
        report.text.contains("saved0 ="),
        case.saved,
        "{}: {}",
        case.class,
        report.text
    );
    let method = report
        .methods
        .iter()
        .find(|m| m.item.name.raw().0 == case.method.as_bytes())
        .unwrap();
    let ClassSourceOutcome::Recovered {
        report: recovery, ..
    } = &method.outcome
    else {
        panic!("{}: {:?}", case.class, method.outcome)
    };
    assert_eq!(recovery.quality, Quality::Structured, "{}", report.text);
    for &bci in case.copies {
        assert!(
            !recovery.source_map.derived_of_bci(bci).is_empty(),
            "{}: copy BCI {bci} lost its derived ownership: {:?}",
            case.class,
            recovery.source_map
        );
    }
    if case.descriptor == "([III)J" {
        assert!(
            !recovery.text.contains(" += "),
            "non-int return must use the typed sequence: {}",
            recovery.text
        );
        assert_eq!(
            recovery.text.matches("[arg1]").count(),
            2,
            "{}",
            recovery.text
        );
    }
}
#[test]
fn stable_copy_loads_survive_renaming_values_widths_and_reference_categories() {
    for case in positives() {
        assert_supported(&case, &recover(&case));
    }
}
#[test]
fn changed_int_long_and_reference_slots_keep_their_saved_value() {
    for case in changed_slots() {
        assert_supported(&case, &recover(&case));
    }
}
#[test]
fn cross_block_and_extra_terminal_consumers_are_not_inline_certificates() {
    for case in negatives() {
        let report = recover(&case);
        assert!(
            report.text.contains("@bytecode"),
            "{}: {}",
            case.class,
            report.text
        );
        assert!(
            !report.text.contains("saved0 ="),
            "{}: {}",
            case.class,
            report.text
        );
        // A marked Mixed fragment may retain an independent statement. The contract is
        // that refusal cannot become a Structured, compilable implementation with a
        // different result; the complete-class rejection oracle below checks that too.
        assert!(
            !report
                .text
                .lines()
                .any(|line| line.trim().starts_with("return ")),
            "{}: a refused copy published a return: {}",
            case.class,
            report.text
        );
        let ClassSourceOutcome::Recovered {
            report: recovery, ..
        } = &report.methods[0].outcome
        else {
            panic!("{}: {:?}", case.class, report.methods[0].outcome)
        };
        assert_ne!(recovery.quality, Quality::Structured, "{}", report.text);
    }
}
#[test]
fn frame_int_does_not_turn_a_boolean_load_into_java_arithmetic() {
    let case = Case::new("BooleanCopyType", "(Z)I", &[0x1a, 0x59, 0x60, 0xac], &[1]);
    let report = recover(&case);
    assert!(report.text.contains("@bytecode"), "{}", report.text);
    assert!(
        !report.text.contains("return arg0 + arg0;"),
        "{}",
        report.text
    );
}
#[test]
fn boolean_arithmetic_is_refused_for_every_operator_and_operand_position() {
    for opcode in [0x60, 0x64, 0x68, 0x6c, 0x70] {
        for (class, descriptor, code, copies) in [
            (
                "BooleanCopyArithmetic",
                "(Z)I",
                vec![0x1a, 0x59, opcode, 0xac],
                vec![1],
            ),
            (
                "WideBooleanCopyArithmetic",
                "(Z)I",
                vec![0xc4, 0x15, 0, 0, 0x59, opcode, 0xac],
                vec![4],
            ),
            (
                "BooleanLeftArithmetic",
                "(ZI)I",
                vec![0x1a, 0x1b, opcode, 0xac],
                vec![],
            ),
            (
                "BooleanRightArithmetic",
                "(IZ)I",
                vec![0x1a, 0x1b, opcode, 0xac],
                vec![],
            ),
        ] {
            let case = Case::new(class, descriptor, &code, &[]);
            let report = recover(&case);
            assert!(
                report.text.contains("@bytecode"),
                "{opcode:#x}: {}",
                report.text
            );
            assert!(
                !report
                    .text
                    .lines()
                    .any(|line| line.trim().starts_with("return ")),
                "{opcode:#x}: {}",
                report.text
            );
            let recovery = match &report.methods[0].outcome {
                ClassSourceOutcome::Recovered { report, .. } => report,
                other => panic!("{other:?}"),
            };
            assert_ne!(
                recovery.quality,
                Quality::Structured,
                "{opcode:#x}: {}",
                report.text
            );
            for bci in copies {
                assert!(
                    !recovery.source_map.derived_of_bci(bci).is_empty()
                        || report.text.contains(&format!("@bytecode {bci}")),
                    "{opcode:#x}: lost copy {bci}: {}",
                    report.text
                );
            }
        }
    }
}

#[test]
fn invalid_category_two_copy_still_stops_before_java_publication() {
    let case = Case::new("InvalidCopyWidth", "(II)I", &[0x1a, 0x5c, 0x60, 0xac], &[1]);
    let (snapshot, request) = method_input(&case);
    let recovery = recover_method(&snapshot, &request, &mut budget());
    assert_no_artifact(&recovery);
}
const MIXED: &[(&str, &[u8])] = &[
    (
        "javac23-release8",
        include_bytes!("fixtures/recover-instance-field-assignment-chains/v8/MX.class"),
    ),
    (
        "javac8",
        include_bytes!("fixtures/recover-instance-field-assignment-chains/v8-javac8/MX.class"),
    ),
];
#[test]
fn mixed_static_instance_copies_keep_all_stores_in_bytecode_order() {
    for (label, bytes) in MIXED {
        let report = recover_bytes(bytes, "MX");
        no_refusal(&report.text);
        for (name, stores) in [
            (
                "mixStaticRight",
                ["MX.s = 5;", "this.b = 5;", "this.a = 5;"],
            ),
            ("mixStaticLeft", ["this.b = 6;", "this.a = 6;", "MX.s = 6;"]),
            (
                "mixStaticMid",
                ["this.b = 7;", "MX.s = saved0;", "this.a = saved0;"],
            ),
        ] {
            let text = &report
                .methods
                .iter()
                .find(|m| m.item.name.raw().0 == name.as_bytes())
                .unwrap()
                .text;
            if name == "mixStaticMid" {
                assert_eq!(
                    text.matches("int saved0 = 7;").count(),
                    1,
                    "{label}: {text}"
                );
                assert!(
                    text.find("this.b = 7;").unwrap() < text.find("int saved0 = 7;").unwrap(),
                    "{label}: {text}"
                );
                assert!(
                    text.find("int saved0 = 7;").unwrap() < text.find("MX.s = saved0;").unwrap(),
                    "{label}: {text}"
                );
            }
            let mut prior = None;
            for store in stores {
                assert_eq!(text.matches(store).count(), 1, "{label}/{name}: {text}");
                let at = text.find(store).unwrap();
                assert!(
                    prior.is_none_or(|prior| prior < at),
                    "{label}/{name}: {text}"
                );
                prior = Some(at);
            }
        }
    }
}
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jarde-stable-copy-{}-{}",
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
    let mut command = Command::new(program);
    if program == "javac" {
        command.arg("-Xlint:-options");
    }
    let output = command.args(args).current_dir(dir).output().unwrap();
    assert!(
        output.status.success(),
        "{program} {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{program} {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}
fn replay_body(case: &Case) -> String {
    let call = format!("{}.{}", case.class, case.method);
    match case.descriptor {
        "(II)I" => format!(
            "int[] xs = {{Integer.MIN_VALUE, -7, -1, 0, 1, 7, Integer.MAX_VALUE}}; for (int a : xs) for (int b : xs) System.out.println({call}(a, b));"
        ),
        "(J)J" => format!(
            "long[] xs = {{Long.MIN_VALUE, -10000000001L, -1, 0, 1, 10000000001L, Long.MAX_VALUE}}; for (long a : xs) System.out.println({call}(a));"
        ),
        "(D)D" => format!(
            "double[] xs = {{Double.NEGATIVE_INFINITY, -Double.MAX_VALUE, -1, -Double.MIN_VALUE, -0.0, 0.0, Double.MIN_VALUE, 1, Double.MAX_VALUE, Double.POSITIVE_INFINITY, Double.NaN}}; for (double a : xs) System.out.println(Double.doubleToLongBits({call}(a)));"
        ),
        "(Ljava/lang/Object;)I" => format!(
            "Object[] xs = {{null, new Object(), \"x\", new int[0]}}; for (Object a : xs) System.out.println({call}(a));"
        ),
        "([III)J" => format!(
            "int[] values = {{Integer.MIN_VALUE, -1, 0, 1, Integer.MAX_VALUE}}; for (int value : values) for (int delta : values) for (int index : new int[] {{-1, 0, 1}}) for (boolean missing : new boolean[] {{false, true}}) {{ int[] a = missing ? null : new int[] {{value}}; try {{ System.out.print({call}(a, index, delta)); }} catch (RuntimeException e) {{ System.out.print(e.getClass().getName()); }} System.out.println(\"/\" + (a == null ? \"null\" : Integer.toString(a[0]))); }}"
        ),
        other => panic!("no replay for {other}"),
    }
}
#[test]
#[ignore = "needs javac and java on PATH; compiles complete unedited Engine outputs"]
fn complete_unedited_stable_and_changed_copy_classes_match_jvm() {
    let scratch = Scratch::new();
    for case in positives().into_iter().chain(changed_slots()) {
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
        assert_supported(&case, &report);
        // No comment stripping, extraction, source replacement, or wrapper repair.
        fs::write(generated.join(format!("{}.java", case.class)), &report.text).unwrap();
        let harness = format!(
            "public class Replay {{ public static void main(String[] args) {{ {} }} }}",
            replay_body(&case)
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
        assert!(!expected.is_empty());
        let actual = run("java", &["-Xverify:all", "-cp", ".", "Replay"], &generated);
        assert_eq!(actual, expected, "{}: {}", case.class, report.text);
    }
    for (label, bytes) in MIXED {
        let original = scratch.0.join(format!("{label}-original"));
        let generated = scratch.0.join(format!("{label}-generated"));
        fs::create_dir(&original).unwrap();
        fs::create_dir(&generated).unwrap();
        fs::write(original.join("MX.class"), bytes).unwrap();
        let report = recover_bytes(bytes, "MX");
        no_refusal(&report.text);
        fs::write(generated.join("MX.java"), &report.text).unwrap();
        run("javac", &["--release", "8", "MX.java"], &generated);
        let expected = run("java", &["-Xverify:all", "-cp", ".", "MX"], &original);
        assert_eq!(expected, b"5/5/5\n6/6/6\n7/7/7\n");
        let actual = run("java", &["-Xverify:all", "-cp", ".", "MX"], &generated);
        assert_eq!(actual, expected, "{label}: {}", report.text);
    }
}

#[test]
#[ignore = "needs javac and java; verifies original negatives and rejects entire generated classes"]
fn refused_copy_fragments_cannot_compile_into_changed_behavior() {
    let scratch = Scratch::new();
    let mut cases = negatives();
    cases.push(Case::new(
        "BooleanCopyType",
        "(Z)I",
        &[0x1a, 0x59, 0x60, 0xac],
        &[1],
    ));
    for case in cases {
        let original = scratch.0.join(format!("{}-original", case.class));
        let generated = scratch.0.join(format!("{}-generated", case.class));
        fs::create_dir(&original).unwrap();
        fs::create_dir(&generated).unwrap();
        fs::write(
            original.join(format!("{}.class", case.class)),
            class_bytes(&case),
        )
        .unwrap();
        let body = if case.descriptor == "(Z)I" {
            format!(
                "for (boolean value : new boolean[] {{false, true}}) System.out.println({}.{}(value));",
                case.class, case.method
            )
        } else {
            replay_body(&case)
        };
        fs::write(
            original.join("Replay.java"),
            format!(
                "public class Replay {{ public static void main(String[] args) {{ {body} }} }}"
            ),
        )
        .unwrap();
        run("javac", &["--release", "8", "Replay.java"], &original);
        let expected = run("java", &["-Xverify:all", "-cp", ".", "Replay"], &original);
        assert!(!expected.is_empty());
        let report = recover(&case);
        assert!(report.text.contains("@bytecode"), "{}", report.text);
        fs::write(generated.join(format!("{}.java", case.class)), &report.text).unwrap();
        let result = Command::new("javac")
            .args([
                "-Xlint:-options",
                "--release",
                "8",
                &format!("{}.java", case.class),
            ])
            .current_dir(&generated)
            .output()
            .unwrap();
        assert!(
            !result.status.success(),
            "{}: refused fragment compiled: {}",
            case.class,
            report.text
        );
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("missing return statement"),
            "{}: {}",
            case.class,
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(!generated.join(format!("{}.class", case.class)).exists());
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
fn stable_copy_source_commit_has_exact_resource_boundaries() {
    use jarde::budget::{BudgetDimension, CountedBudgetDimension};
    let case = array_case();
    let (snapshot, request) = method_input(&case);
    let mut measured = budget();
    let full = recover_method(&snapshot, &request, &mut measured);
    assert!(
        full.produced() && !full.text.contains("@bytecode"),
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
fn stable_copy_late_cancellation_leaves_no_artifact() {
    use jarde_reader::ledger::{OperationLedger, UsageOwner};
    use std::sync::Arc;
    let case = array_case();
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
        full.produced() && !full.text.contains("@bytecode"),
        "{full:?}"
    );
    let charges = observer.charges.load(Ordering::Relaxed);
    assert!(
        charges > 10,
        "the public recovery has a meaningful late cancellation point"
    );
    // The penultimate charge has a subsequent budget operation, so cancellation cannot be
    // mistaken for pre-cancel or an unobserved signal after the final operation completed.
    // Essential evidence leaves no optional post-commit evidence work.
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

#[path = "support/copy_boundary_oracle.rs"]
mod boundary_effect_oracle;
#[test]
#[ignore = "strict observable-effect metamorphs; complete unedited Engine classes"]
fn observable_chain_effects_match_complete_original_classes() {
    let scratch = Scratch::new();
    for (class, source) in [
        (
            "ChainEffects",
            include_str!("fixtures/copy-boundary-effects/ChainEffects.java"),
        ),
        (
            "RenamedChainEffects",
            include_str!("fixtures/copy-boundary-effects/RenamedChainEffects.java"),
        ),
    ] {
        let dir = scratch.0.join(class);
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join(format!("{class}.java")), source).unwrap();
        let javac = std::env::var("JAVAC").unwrap_or_else(|_| "javac".into());
        let compiled = Command::new(javac)
            .current_dir(&dir)
            .args(["--release", "8"])
            .arg(format!("{class}.java"))
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let bytes = fs::read(dir.join(format!("{class}.class"))).unwrap();
        let report = recover_bytes(&bytes, class);
        let renamed = class == "RenamedChainEffects";
        let replay = EFFECT_REPLAY
            .replace("TARGET", class)
            .replace("FIRST", if renamed { "retained" } else { "saved" })
            .replace("SECOND", if renamed { "receivers" } else { "holderChain" })
            .replace("VALUE", if renamed { "-37" } else { "19" })
            .replace("CHAIN", if renamed { "41" } else { "23" })
            .replace("PREFIX", if renamed { "P" } else { "" });
        boundary_effect_oracle::require_equal(class, class, &bytes, &report.text, &replay);
    }
}

const EFFECT_REPLAY: &str = r#"public class Replay {
 static void check(boolean yes) { if (!yes) throw new AssertionError("oracle coverage/state"); }
 public static void main(String[] args) {
  int cases=0;
  for(int m=0;m<4;m++) for(int k=0;k<4;k++) {
   TARGET.reset(m); TARGET x=new TARGET(); String status="ok";
   try {
    if(k==0)x.FIRST();else if(k==1)x.SECOND();else if(k==2)x.expr();else x.cross();
   } catch(IllegalStateException e) {status=e.getClass().getName();}
     catch(IllegalArgumentException e) {status=e.getClass().getName();}
     catch(NullPointerException e) {status=e.getClass().getName();}
   String expected=m==1&&k!=1?"java.lang.IllegalStateException":m==2&&k==1?"java.lang.IllegalArgumentException":m==3&&k==1?"java.lang.NullPointerException":"ok";
   check(status.equals(expected));
   if(k==1) {
    check(TARGET.calls==2 && TARGET.trace.equals("H1H2"));
    check(x.a==0 && x.b==0 && x.c==0);
    check(TARGET.one.a==((m==2||m==3)?0:CHAIN));
    check(TARGET.two.b==(m==2?0:CHAIN));
   } else {
    boolean wrote=m!=1;
    check(TARGET.calls==1 && TARGET.trace.equals(k==0?"PREFIXF":"F"));
    check(x.a==(wrote?(k==0?VALUE:k==2?VALUE+1:0):0));
    check(x.b==(wrote&&(k==0||k==2)?VALUE:0));
    check(x.c==(wrote&&k==0?VALUE:0));
    check(TARGET.one.a==(wrote&&k==3?VALUE:0) && TARGET.two.b==(wrote&&k==3?VALUE:0));
   }
   System.out.println(status);
   System.out.println(TARGET.calls+"/"+TARGET.trace+"/"+x.a+"/"+x.b+"/"+x.c+"/"+TARGET.one.a+"/"+TARGET.two.b);
   cases++;
  }
  check(cases==16);
 }
}"#;
