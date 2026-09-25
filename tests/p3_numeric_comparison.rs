//! `recover-numeric-comparison-conditions`: compose lcmp/fcmp*/dcmp* with their
//! same-block zero branch, retaining long boundaries, NaN polarity, sources and effects.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] = include_bytes!("fixtures/p3-numeric-comparison/v8/NumericComparisons.class");

const DIRECT: [(&[u8], &[u8], &str); 21] = [
    (b"long_eq", b"(JJ)I", "arg0 == arg2"),
    (b"long_ne", b"(JJ)I", "arg0 != arg2"),
    (b"long_lt", b"(JJ)I", "arg0 < arg2"),
    (b"long_le", b"(JJ)I", "arg0 <= arg2"),
    (b"long_gt", b"(JJ)I", "arg0 > arg2"),
    (b"long_ge", b"(JJ)I", "arg0 >= arg2"),
    (b"long_not_lt", b"(JJ)I", "arg0 >= arg2"),
    (b"float_eq", b"(FF)I", "arg0 == arg1"),
    (b"float_ne", b"(FF)I", "arg0 != arg1"),
    (b"float_lt", b"(FF)I", "arg0 < arg1"),
    (b"float_le", b"(FF)I", "arg0 <= arg1"),
    (b"float_gt", b"(FF)I", "arg0 > arg1"),
    (b"float_ge", b"(FF)I", "arg0 >= arg1"),
    (b"float_not_lt", b"(FF)I", "!(arg0 < arg1)"),
    (b"double_eq", b"(DD)I", "arg0 == arg2"),
    (b"double_ne", b"(DD)I", "arg0 != arg2"),
    (b"double_lt", b"(DD)I", "arg0 < arg2"),
    (b"double_le", b"(DD)I", "arg0 <= arg2"),
    (b"double_gt", b"(DD)I", "arg0 > arg2"),
    (b"double_ge", b"(DD)I", "arg0 >= arg2"),
    (b"double_not_lt", b"(DD)I", "!(arg0 < arg2)"),
];

const EFFECTS: [(&[u8], &[u8], &str); 3] = [
    (b"callOrder", b"(JJ)I", "if (left(arg0) < right(arg2))"),
    (
        b"callThrowLeft",
        b"(JJ)I",
        "if (throwingLeft(arg0) < right(arg2))",
    ),
    (
        b"callThrowRight",
        b"(JJ)I",
        "if (left(arg0) < throwingRight(arg2))",
    ),
];

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
        class_headers: 32,
        method_bodies: 64,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

fn environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    ResolutionEnvironment {
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
    }
}

struct Fixture {
    snapshot: ArtifactSnapshot,
    class_bytes: ClassBytesId,
}

fn fixture(engine: &Engine) -> Fixture {
    fixture_bytes(engine, FIXTURE)
}

fn fixture_bytes(engine: &Engine, bytes: &[u8]) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("the numeric-comparison fixture opens");
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the fixture header is readable");
    let declared: Vec<Vec<u8>> = inspected
        .inspection
        .header
        .methods
        .iter()
        .map(|member| member.name.raw().0.clone())
        .collect();
    for &(name, _, _) in DIRECT.iter().chain(EFFECTS.iter()) {
        assert!(
            declared.iter().any(|member| member.as_slice() == name),
            "the fixture declares `{}`: {declared:?}",
            String::from_utf8_lossy(name)
        );
    }
    Fixture {
        snapshot,
        class_bytes: inspected.source.class_bytes.clone(),
    }
}

const LONG_EQ_CODE: &[u8] = &[
    0x1e, 0x20, 0x94, 0x9a, 0x00, 0x06, 0x10, 0x07, 0xac, 0x10, 0x09, 0xac,
];
const LONG_EQ_CODE_ATTRIBUTE_LENGTH: u32 = 33;
const LONG_EQ_STACK_MAP_ATTRIBUTE_LENGTH: u32 = 3;
const LONG_EQ_STACK_MAP_ENTRIES: u16 = 1;

/// Make one verifier-valid refusal probe from the frozen `long_eq` Code attribute.  This is the
/// same narrow, complete-Code match as the evidence script: it changes no constant-pool entry or
/// other method, and it deliberately has no class-file parser hidden in the test.
fn long_eq_refusal_patch(
    bytes: &[u8],
    insertion: &[u8],
    insertion_offset: usize,
    frame: &[u8],
    stack_map_delta: u32,
    locals_delta: u16,
) -> Vec<u8> {
    assert!(
        !frame.is_empty(),
        "a refusal patch must carry a StackMap frame"
    );
    assert_eq!(
        stack_map_delta,
        (frame.len() - 1) as u32,
        "StackMapTable length delta must match the replacement frame"
    );
    let sites: Vec<usize> = bytes
        .windows(LONG_EQ_CODE.len())
        .enumerate()
        .filter_map(|(index, window)| (window == LONG_EQ_CODE).then_some(index))
        .collect();
    assert_eq!(sites.len(), 1, "the frozen long_eq Code occurs once");
    let start = sites[0];
    let code_length_at = start - 4;
    let attribute_length_at = start - 12;
    let max_locals_at = start - 6;
    let code_length = u32::from_be_bytes(bytes[code_length_at..start].try_into().unwrap());
    let attribute_length =
        u32::from_be_bytes(bytes[attribute_length_at..start - 8].try_into().unwrap());
    let max_locals = u16::from_be_bytes(bytes[max_locals_at..start - 4].try_into().unwrap());
    assert_eq!(code_length as usize, LONG_EQ_CODE.len());
    assert_eq!(attribute_length, LONG_EQ_CODE_ATTRIBUTE_LENGTH);
    assert_eq!(max_locals, 4);

    let original_frame_at = start + LONG_EQ_CODE.len() + 12;
    let original_stack_map_length_at = original_frame_at - 6;
    let original_entries_at = original_frame_at - 2;
    assert_eq!(bytes[original_frame_at], 9);
    assert_eq!(
        u32::from_be_bytes(
            bytes[original_stack_map_length_at..original_stack_map_length_at + 4]
                .try_into()
                .unwrap()
        ),
        LONG_EQ_STACK_MAP_ATTRIBUTE_LENGTH
    );
    assert_eq!(
        u16::from_be_bytes(
            bytes[original_entries_at..original_entries_at + 2]
                .try_into()
                .unwrap()
        ),
        LONG_EQ_STACK_MAP_ENTRIES
    );

    // The branch and its target move together when instructions are inserted into the code.  The
    // caller chooses the offset to exercise either a non-adjacent result reader or an overwritten
    // local that leaves the original operand value on the operand stack.
    assert!(insertion_offset <= LONG_EQ_CODE.len());
    let insertion_at = start + insertion_offset;
    let mut patched = Vec::with_capacity(bytes.len() + insertion.len() + frame.len() - 1);
    patched.extend_from_slice(&bytes[..insertion_at]);
    patched.extend_from_slice(insertion);
    patched.extend_from_slice(&bytes[insertion_at..]);
    patched[code_length_at..start]
        .copy_from_slice(&(code_length + insertion.len() as u32).to_be_bytes());
    patched[max_locals_at..start - 4].copy_from_slice(&(max_locals + locals_delta).to_be_bytes());
    patched[attribute_length_at..start - 8].copy_from_slice(
        &(attribute_length + insertion.len() as u32 + stack_map_delta).to_be_bytes(),
    );
    assert_eq!(
        u32::from_be_bytes(patched[code_length_at..start].try_into().unwrap()),
        code_length + insertion.len() as u32
    );
    assert_eq!(
        u32::from_be_bytes(patched[attribute_length_at..start - 8].try_into().unwrap()),
        LONG_EQ_CODE_ATTRIBUTE_LENGTH + insertion.len() as u32 + stack_map_delta
    );

    // Code tail: exception_table_length (2), attributes_count (2), StackMapTable name (2),
    // attribute_length (4), and number_of_entries (2) precede the sole frame.
    let frame_at = start + LONG_EQ_CODE.len() + insertion.len() + 12;
    assert_eq!(patched[frame_at], 9);
    assert_eq!(
        u16::from_be_bytes(patched[frame_at - 2..frame_at].try_into().unwrap()),
        LONG_EQ_STACK_MAP_ENTRIES
    );
    if frame.len() == 1 {
        assert_eq!(
            u32::from_be_bytes(patched[frame_at - 6..frame_at - 2].try_into().unwrap()),
            LONG_EQ_STACK_MAP_ATTRIBUTE_LENGTH
        );
        patched[frame_at] = frame[0];
    } else {
        let stack_map_length_at = frame_at - 6;
        let old_stack_map_length = u32::from_be_bytes(
            patched[stack_map_length_at..stack_map_length_at + 4]
                .try_into()
                .unwrap(),
        );
        assert_eq!(old_stack_map_length, LONG_EQ_STACK_MAP_ATTRIBUTE_LENGTH);
        patched[stack_map_length_at..stack_map_length_at + 4]
            .copy_from_slice(&(old_stack_map_length + stack_map_delta).to_be_bytes());
        patched.splice(frame_at..frame_at + 1, frame.iter().copied());
        assert_eq!(
            u32::from_be_bytes(patched[frame_at - 6..frame_at - 2].try_into().unwrap()),
            LONG_EQ_STACK_MAP_ATTRIBUTE_LENGTH + stack_map_delta
        );
    }
    patched
}

fn recover(engine: &Engine, fixture: &Fixture, name: &[u8], descriptor: &[u8]) -> RecoveryReport {
    let request = MethodAnalysisRequest {
        environment: environment(&fixture.snapshot),
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: fixture.snapshot.id().clone(),
                },
                class_bytes: fixture.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    let mut budget = Budget::new(limits());
    engine
        .recover_method_with_evidence(
            slice::from_ref(&fixture.snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("a legal recovery request is answered")
        .recovery()
        .clone()
}

fn body(report: &RecoveryReport) -> String {
    let lines: Vec<&str> = report
        .text
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if lines.is_empty() {
        return String::new();
    }
    let start = if lines.first() == Some(&"{") { 1 } else { 0 };
    let end = lines.len() - if lines.last() == Some(&"}") { 1 } else { 0 };
    lines[start..end].join("\n")
}

fn quoted_bcis(report: &RecoveryReport) -> Vec<u32> {
    report
        .text
        .lines()
        .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
        .flat_map(|bcis| {
            bcis.split_whitespace()
                .map(|bci| bci.parse::<u32>().expect("quoted BCI is numeric"))
        })
        .collect()
}

#[test]
fn direct_long_float_double_relations_recover_with_expected_nan_polarity() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    for &(name, descriptor, relation) in &DIRECT {
        let report = recover(&engine, &fixture, name, descriptor);
        assert_eq!(
            report.representation,
            Representation::Java,
            "{}",
            report.text
        );
        assert_eq!(report.quality, Quality::Structured, "{}", report.text);
        assert!(
            report.produced(),
            "{}: {:?}\n{}",
            String::from_utf8_lossy(name),
            report.stop(),
            report.text
        );
        let rendered = body(&report);
        assert!(
            rendered.contains(&format!("if ({relation})")),
            "{} should preserve `{relation}`:\n{}",
            String::from_utf8_lossy(name),
            report.text
        );
        assert!(!report.text.contains("@bytecode"), "{}", report.text);
    }
}

#[test]
fn accepted_comparison_keeps_compare_and_branch_origins() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    let report = recover(&engine, &fixture, b"long_lt", b"(JJ)I");
    assert_eq!(
        report.representation,
        Representation::Java,
        "{}",
        report.text
    );
    assert_eq!(report.quality, Quality::Structured, "{}", report.text);
    assert!(report.produced(), "{}", report.text);
    assert!(
        !report.source_map.of_bci(2).is_empty(),
        "compare BCI: {:?}",
        report.source_map.segments()
    );
    assert!(
        !report.source_map.of_bci(3).is_empty(),
        "branch BCI: {:?}",
        report.source_map.segments()
    );
    for bci in [2, 3] {
        let all = report.source_map.of_bci(bci);
        let classified = report.source_map.direct_of_bci(bci).len()
            + report.source_map.derived_of_bci(bci).len();
        assert_eq!(
            classified,
            all.len(),
            "BCI {bci} has an unclassified provenance"
        );
    }
}

#[test]
fn call_operands_keep_order_and_the_negative_boolean_merge_stays_quoted() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    for &(name, descriptor, condition) in &EFFECTS {
        let report = recover(&engine, &fixture, name, descriptor);
        assert!(report.produced(), "{}", report.text);
        let rendered = body(&report);
        assert!(
            rendered.contains(condition),
            "{} should retain its calls:\n{}",
            String::from_utf8_lossy(name),
            report.text
        );
        assert_eq!(
            rendered.matches("left(arg0)").count(),
            if name != b"callThrowLeft" { 1 } else { 0 }
        );
        assert_eq!(
            rendered.matches("right(arg2)").count(),
            if name == b"callOrder" || name == b"callThrowLeft" {
                1
            } else {
                0
            }
        );
    }
    let positive = recover(&engine, &fixture, b"sameBlock", b"(JJ)I");
    assert_eq!(
        positive.representation,
        Representation::Java,
        "{}",
        positive.text
    );
    assert_eq!(positive.quality, Quality::Structured, "{}", positive.text);
    assert!(positive.produced(), "{}", positive.text);
    assert!(
        body(&positive).contains("if (arg0 < arg2)"),
        "{}",
        positive.text
    );
    assert!(!positive.text.contains("@bytecode"), "{}", positive.text);

    let report = recover(&engine, &fixture, b"booleanMerge", b"(DD)Z");
    assert!(report.produced(), "{}", report.text);
    assert_eq!(
        report.representation,
        Representation::Mixed,
        "the converged boolean remains a mixed fallback:\n{}",
        report.text
    );
    assert_eq!(
        report.quality,
        Quality::Fallback,
        "the converged boolean remains a fallback:\n{}",
        report.text
    );
    assert!(
        report.text.contains("@bytecode 11"),
        "the converged return remains quoted:\n{}",
        report.text
    );
    for bci in [2, 3, 11] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "the comparison, branch, and converged return keep source BCI {bci}:\n{}",
            report.text
        );
    }
}

fn assert_long_eq_refusal(report: &RecoveryReport, compare_bci: u32, branch_bci: u32) {
    assert!(
        report.produced(),
        "a refusal remains an answer:\n{}",
        report.text
    );
    assert_eq!(
        report.representation,
        Representation::Mixed,
        "a non-adjacent compare is not a Java-only recovery:\n{}",
        report.text
    );
    assert_eq!(
        report.quality,
        Quality::Fallback,
        "a non-adjacent compare must remain a fallback:\n{}",
        report.text
    );
    let quoted = quoted_bcis(report);
    assert!(
        quoted.contains(&compare_bci),
        "the refusal quote must retain lcmp BCI {compare_bci}: {:?}\n{}",
        quoted,
        report.text
    );
    assert!(
        quoted.contains(&branch_bci),
        "the refusal quote must retain zero-branch BCI {branch_bci}: {:?}\n{}",
        quoted,
        report.text
    );
    assert!(
        !body(report).contains("if (arg0 == arg2)") && !body(report).contains("if (arg0 != arg2)"),
        "a non-adjacent compare result must not be rewritten as a direct condition:\n{}",
        report.text
    );
    assert!(
        !report.source_map.of_bci(compare_bci).is_empty(),
        "lcmp source was dropped:\n{}",
        report.text
    );
    assert!(
        !report.source_map.of_bci(branch_bci).is_empty(),
        "zero-branch source was dropped at BCI {branch_bci}:\n{}",
        report.text
    );
}

#[test]
fn long_compare_with_stack_duplicate_and_pop_keeps_refusal_sources() {
    // lcmp -> dup -> pop -> ifne inserts stack operations and removes the immediate lcmp/branch
    // edge. This is a separate non-adjacent boundary probe; whether the engine exposes more than
    // one direct SSA reader is established only by the later Cargo run. The class bytes are a
    // temporary in-memory probe; the verifier-valid patch is archived beside the evidence script,
    // while the committed fixture remains untouched.
    let patched = long_eq_refusal_patch(FIXTURE, &[0x59, 0x57], 3, &[0x0b], 0, 0);
    let engine = Engine::new();
    let report = recover(
        &engine,
        &fixture_bytes(&engine, &patched),
        b"long_eq",
        b"(JJ)I",
    );
    assert_long_eq_refusal(&report, 2, 5);
}

#[test]
fn long_compare_through_local_keeps_refusal_sources() {
    // lcmp -> istore/iload -> ifne is a local transfer rather than the single same-block reader
    // accepted by the change.  The appended StackMap frame makes this patch valid under
    // java -Xverify:all; its output is recorded in the refusal-boundaries evidence directory.
    let patched = long_eq_refusal_patch(
        FIXTURE,
        &[0x36, 0x04, 0x15, 0x04],
        3,
        &[0xfc, 0x00, 0x0d, 0x01],
        3,
        1,
    );
    let engine = Engine::new();
    let report = recover(
        &engine,
        &fixture_bytes(&engine, &patched),
        b"long_eq",
        b"(JJ)I",
    );
    assert_long_eq_refusal(&report, 2, 7);
}

#[test]
fn long_compare_with_overwritten_local_keeps_refusal_sources() {
    // Keep the original lload_0 value on the operand stack, then overwrite local slot 0 before
    // lload_2/lcmp/ifne.  The compare does not read the rewritten local, so it cannot be rendered
    // from the current local expression without changing the bytecode's value flow.
    let patched = long_eq_refusal_patch(FIXTURE, &[0x09, 0x3f], 1, &[0x0b], 0, 0);
    let engine = Engine::new();
    let report = recover(
        &engine,
        &fixture_bytes(&engine, &patched),
        b"long_eq",
        b"(JJ)I",
    );
    assert_long_eq_refusal(&report, 4, 5);
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-numeric-comparison-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create the JDK comparison directory");
        Self(path)
    }

    fn write(&self, name: &str, text: &str) {
        fs::write(self.0.join(name), text).expect("write a JDK comparison source");
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn declaration(name: &[u8], descriptor: &[u8]) -> &'static str {
    match (name, descriptor) {
        (b"long_eq", b"(JJ)I") => "public static int long_eq(long arg0, long arg2)",
        (b"long_ne", b"(JJ)I") => "public static int long_ne(long arg0, long arg2)",
        (b"long_lt", b"(JJ)I") => "public static int long_lt(long arg0, long arg2)",
        (b"long_le", b"(JJ)I") => "public static int long_le(long arg0, long arg2)",
        (b"long_gt", b"(JJ)I") => "public static int long_gt(long arg0, long arg2)",
        (b"long_ge", b"(JJ)I") => "public static int long_ge(long arg0, long arg2)",
        (b"long_not_lt", b"(JJ)I") => "public static int long_not_lt(long arg0, long arg2)",
        (b"float_eq", b"(FF)I") => "public static int float_eq(float arg0, float arg1)",
        (b"float_ne", b"(FF)I") => "public static int float_ne(float arg0, float arg1)",
        (b"float_lt", b"(FF)I") => "public static int float_lt(float arg0, float arg1)",
        (b"float_le", b"(FF)I") => "public static int float_le(float arg0, float arg1)",
        (b"float_gt", b"(FF)I") => "public static int float_gt(float arg0, float arg1)",
        (b"float_ge", b"(FF)I") => "public static int float_ge(float arg0, float arg1)",
        (b"float_not_lt", b"(FF)I") => "public static int float_not_lt(float arg0, float arg1)",
        (b"double_eq", b"(DD)I") => "public static int double_eq(double arg0, double arg2)",
        (b"double_ne", b"(DD)I") => "public static int double_ne(double arg0, double arg2)",
        (b"double_lt", b"(DD)I") => "public static int double_lt(double arg0, double arg2)",
        (b"double_le", b"(DD)I") => "public static int double_le(double arg0, double arg2)",
        (b"double_gt", b"(DD)I") => "public static int double_gt(double arg0, double arg2)",
        (b"double_ge", b"(DD)I") => "public static int double_ge(double arg0, double arg2)",
        (b"double_not_lt", b"(DD)I") => "public static int double_not_lt(double arg0, double arg2)",
        (b"callOrder", b"(JJ)I") => "public static int callOrder(long arg0, long arg2)",
        (b"callThrowLeft", b"(JJ)I") => "public static int callThrowLeft(long arg0, long arg2)",
        (b"callThrowRight", b"(JJ)I") => "public static int callThrowRight(long arg0, long arg2)",
        _ => panic!("declaration is missing for {:?} {:?}", name, descriptor),
    }
}

fn recovered_class(engine: &Engine, fixture: &Fixture) -> String {
    let mut source = String::from(
        "public final class RecoveredNumericComparisons {\n".to_owned()
            + "    static int calls;\n"
            + "    static void resetCalls() { calls = 0; }\n"
            + "    static int calls() { return calls; }\n"
            + "    static long left(long value) { calls = calls * 10 + 1; return value; }\n"
            + "    static long right(long value) { calls = calls * 10 + 2; return value; }\n"
            + "    static long throwingLeft(long value) { calls = calls * 10 + 1; throw new IllegalStateException(\"left\"); }\n"
            + "    static long throwingRight(long value) { calls = calls * 10 + 2; throw new IllegalStateException(\"right\"); }\n",
    );
    for &(name, descriptor, _) in DIRECT.iter().chain(EFFECTS.iter()) {
        let report = recover(engine, fixture, name, descriptor);
        assert!(report.produced(), "{}", report.text);
        assert!(!report.text.contains("@bytecode"), "{}", report.text);
        source.push_str("    ");
        source.push_str(declaration(name, descriptor));
        source.push_str(" {\n");
        for line in body(&report).lines() {
            source.push_str("        ");
            source.push_str(line);
            source.push('\n');
        }
        source.push_str("    }\n");
    }
    source.push_str("}\n");
    source
}

fn driver_source() -> &'static str {
    r#"
import java.lang.reflect.Method;

public final class NumericComparisonDriver {
    private static String run(boolean recovered) throws Exception {
        Class<?> type = recovered ? RecoveredNumericComparisons.class : NumericComparisons.class;
        String[] ops = {"eq", "ne", "lt", "le", "gt", "ge", "not_lt"};
        Object[][] values = {
            {Long.MIN_VALUE, -1L, 0L, 1L, Long.MAX_VALUE},
            {Float.NaN, Float.NEGATIVE_INFINITY, -1.0f, -0.0f, 0.0f, 1.0f, Float.POSITIVE_INFINITY,
                Float.MIN_VALUE, Float.MAX_VALUE},
            {Double.NaN, Double.NEGATIVE_INFINITY, -1.0d, -0.0d, 0.0d, 1.0d,
                Double.POSITIVE_INFINITY, Double.MIN_VALUE, Double.MAX_VALUE}
        };
        Class<?>[] types = {long.class, float.class, double.class};
        StringBuilder out = new StringBuilder();
        for (int t = 0; t < types.length; t++) for (String op : ops) {
            Method method = type.getMethod(types[t].getName() + "_" + op, types[t], types[t]);
            for (Object left : values[t]) for (Object right : values[t])
                out.append(method.invoke(null, left, right)).append('\n');
        }
        if (recovered) RecoveredNumericComparisons.resetCalls(); else NumericComparisons.resetCalls();
        out.append(recovered ? RecoveredNumericComparisons.callOrder(1, 2) : NumericComparisons.callOrder(1, 2));
        out.append(':').append(recovered ? RecoveredNumericComparisons.calls() : NumericComparisons.calls()).append('\n');
        if (recovered) RecoveredNumericComparisons.resetCalls(); else NumericComparisons.resetCalls();
        try {
            if (recovered) RecoveredNumericComparisons.callThrowLeft(1, 2);
            else NumericComparisons.callThrowLeft(1, 2);
            out.append("returned");
        } catch (IllegalStateException error) {
            out.append(error.getClass().getSimpleName()).append(':').append(error.getMessage());
        }
        out.append(':').append(recovered ? RecoveredNumericComparisons.calls() : NumericComparisons.calls()).append('\n');
        if (recovered) RecoveredNumericComparisons.resetCalls(); else NumericComparisons.resetCalls();
        try {
            if (recovered) RecoveredNumericComparisons.callThrowRight(1, 2);
            else NumericComparisons.callThrowRight(1, 2);
            out.append("returned");
        } catch (IllegalStateException error) {
            out.append(error.getClass().getSimpleName()).append(':').append(error.getMessage());
        }
        out.append(':').append(recovered ? RecoveredNumericComparisons.calls() : NumericComparisons.calls()).append('\n');
        return out.toString();
    }

    public static void main(String[] args) throws Exception {
        String original = run(false);
        String generated = run(true);
        if (!original.equals(generated)) throw new AssertionError(original + "---generated---\n" + generated);
        System.out.print(original);
    }
}
"#
}

#[test]
#[ignore = "runs javac/java with NaN, signed zero, infinities, calls and exceptions"]
fn jdk_execution_matches_original_for_comparisons_and_effects() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    let recovered = recovered_class(&engine, &fixture);
    let dir = TempDir::new();
    fs::write(dir.path().join("NumericComparisons.class"), FIXTURE).expect("write original class");
    fs::write(
        dir.path().join("RecoveredNumericComparisons.java"),
        recovered,
    )
    .expect("write recovered class");
    dir.write("NumericComparisonDriver.java", driver_source());
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-cp"])
        .arg(dir.path())
        .args(["-d"])
        .arg(dir.path())
        .arg(dir.path().join("RecoveredNumericComparisons.java"))
        .arg(dir.path().join("NumericComparisonDriver.java"))
        .output()
        .expect("run javac for numeric comparison recovery");
    assert!(
        compile.status.success(),
        "javac failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(dir.path())
        .arg("NumericComparisonDriver")
        .output()
        .expect("run the verified numeric comparison comparison");
    assert!(
        run.status.success(),
        "java -Xverify:all failed:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
}
