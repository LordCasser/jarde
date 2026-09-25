//! `recover-unary-negation`: the four JVM numeric negations are one value operation, with Java's
//! unary numeric promotion and grouping preserved at every consumer.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] = include_bytes!("fixtures/p3-unary-negation/v8/UnaryNegation.class");

const DECLARED: [(&[u8], &[u8]); 19] = [
    (b"negInt", b"(I)I"),
    (b"negLong", b"(J)J"),
    (b"negFloat", b"(F)F"),
    (b"negDouble", b"(D)D"),
    (b"negByte", b"(B)I"),
    (b"negChar", b"(C)I"),
    (b"negShort", b"(S)I"),
    (b"nested", b"(I)I"),
    (b"negSum", b"(II)I"),
    (b"multiplyRight", b"(II)I"),
    (b"divideRight", b"(II)I"),
    (b"local", b"(I)I"),
    (b"consume", b"(I)I"),
    (b"callArgument", b"(I)I"),
    (b"throwing", b"(I)I"),
    (b"negatedCall", b"(I)I"),
    (b"negatedFail", b"(I)I"),
    (b"fail", b"(I)I"),
    (b"unsupportedOperand", b"()I"),
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
        method_bodies: 32,
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
    let mut budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(FIXTURE.to_vec()), &mut budget)
        .expect("the unary-negation fixture opens");
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
    for (name, _) in DECLARED {
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

fn recover(engine: &Engine, fixture: &Fixture, name: &[u8], descriptor: &[u8]) -> RecoveryReport {
    let mut budget = Budget::new(limits());
    recover_with_budget(engine, fixture, name, descriptor, &mut budget)
}

fn recover_with_budget(
    engine: &Engine,
    fixture: &Fixture,
    name: &[u8],
    descriptor: &[u8],
    budget: &mut Budget,
) -> RecoveryReport {
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
    engine
        .recover_method_with_evidence(
            slice::from_ref(&fixture.snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            budget,
        )
        .expect("a legal recovery request is answered")
        .recovery()
        .clone()
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-unary-negation-{}-{nonce}",
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

fn body(report: &RecoveryReport) -> String {
    report
        .text
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "{" && *line != "}")
        .collect::<Vec<_>>()
        .join("\n")
}

fn recovered_body(fixture: &Fixture, name: &[u8], descriptor: &[u8]) -> String {
    let report = recover(&Engine::new(), fixture, name, descriptor);
    assert!(report.produced(), "recovery stopped: {:?}", report.stop());
    body(&report)
}

#[test]
fn four_direct_numeric_negations_are_recovered() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    for (name, descriptor) in [
        (b"negInt".as_slice(), b"(I)I".as_slice()),
        (b"negLong".as_slice(), b"(J)J".as_slice()),
        (b"negFloat".as_slice(), b"(F)F".as_slice()),
        (b"negDouble".as_slice(), b"(D)D".as_slice()),
    ] {
        let report = recover(&engine, &fixture, name, descriptor);
        assert_eq!(
            report.representation,
            Representation::Java,
            "{}",
            report.text
        );
        assert_eq!(report.quality, Quality::Structured, "{}", report.text);
        assert_eq!(body(&report), "return -arg0;", "{}", report.text);
    }
}

#[test]
fn negation_and_its_consumer_keep_distinct_source_origins() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    let report = recover(&engine, &fixture, b"negInt", b"(I)I");
    assert!(
        !report.source_map.direct_of_bci(1).is_empty(),
        "the ineg at BCI 1 anchors the unary node: {:?}",
        report.source_map.segments()
    );
    assert!(
        !report.source_map.direct_of_bci(2).is_empty(),
        "the ireturn at BCI 2 anchors its return statement: {:?}",
        report.source_map.segments()
    );
}

#[test]
fn negation_emission_still_honors_the_output_budget() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    let produced = recover(&engine, &fixture, b"negInt", b"(I)I");
    let charged = match produced.execution {
        ExecutionReport::Complete { usage } => usage.output_bytes,
        other => panic!("the ample run completes: {other:?}"),
    };
    let artifact = u64::try_from(produced.text.len()).expect("the artifact length fits u64");
    assert!(
        charged > artifact,
        "the run charged envelope and artifact output"
    );
    let mut constrained = limits();
    constrained.output_bytes = charged - artifact;
    let mut budget = Budget::new(constrained);
    let report = recover_with_budget(&engine, &fixture, b"negInt", b"(I)I", &mut budget);
    assert!(
        matches!(
            report.outcome,
            RecoveryOutcome::Stopped(StopReason::Budget {
                dimension: CountedBudgetDimension::OutputBytes,
                ..
            })
        ),
        "a tight output budget remains observable: {:?}",
        report.outcome
    );
    assert!(
        report.text.is_empty(),
        "a stopped emission has no partial text"
    );
    assert!(report.source_map.segments().is_empty());
}

#[test]
fn narrow_parameters_are_unary_promoted_to_int() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    for (name, descriptor) in [
        (b"negByte".as_slice(), b"(B)I".as_slice()),
        (b"negChar".as_slice(), b"(C)I".as_slice()),
        (b"negShort".as_slice(), b"(S)I".as_slice()),
    ] {
        assert_eq!(
            recovered_body(&fixture, name, descriptor),
            "return -arg0;",
            "{}",
            String::from_utf8_lossy(name)
        );
    }
}

#[test]
fn negation_keeps_nested_and_binary_grouping() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    assert_eq!(
        recovered_body(&fixture, b"nested", b"(I)I"),
        "return -(-arg0);"
    );
    assert_eq!(
        recovered_body(&fixture, b"negSum", b"(II)I"),
        "return -(arg0 + arg1);"
    );
    assert_eq!(
        recovered_body(&fixture, b"multiplyRight", b"(II)I"),
        "return arg0 * -arg1;"
    );
    assert_eq!(
        recovered_body(&fixture, b"divideRight", b"(II)I"),
        "return arg0 / -arg1;"
    );
}

#[test]
fn negation_is_preserved_through_local_and_call_consumers() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    assert_eq!(
        recovered_body(&fixture, b"local", b"(I)I"),
        "int local1 = -arg0;\nreturn local1;"
    );
    assert_eq!(
        recovered_body(&fixture, b"callArgument", b"(I)I"),
        "return consume(-arg0);"
    );
    assert_eq!(
        recovered_body(&fixture, b"throwing", b"(I)I"),
        "return fail(-arg0);"
    );
    assert_eq!(
        recovered_body(&fixture, b"negatedCall", b"(I)I"),
        "return -consume(arg0);"
    );
    assert_eq!(
        recovered_body(&fixture, b"negatedFail", b"(I)I"),
        "return -fail(arg0);"
    );
}

#[test]
fn converted_negated_operand_keeps_conversion_and_bytecode_origins() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    let report = recover(&engine, &fixture, b"unsupportedOperand", b"()I");
    assert!(report.produced(), "recovery stopped: {:?}", report.stop());
    assert!(
        report.representation == Representation::Java,
        "{}",
        report.text
    );
    assert!(report.quality == Quality::Structured, "{}", report.text);
    assert_eq!(
        body(&report),
        "return -(byte) consume(7);",
        "the conversion, negation, and call remain in Java syntax:\n{}",
        report.text
    );
    for (bci, operation) in [
        (0, "bipush"),
        (2, "consume"),
        (5, "i2b"),
        (6, "ineg"),
        (7, "ireturn"),
    ] {
        assert!(
            !report.source_map.direct_of_bci(bci).is_empty(),
            "the {operation} at BCI {bci} has a direct source origin: {:?}",
            report.source_map.segments()
        );
    }
}

#[test]
#[ignore = "runs javac/java for the controlled fixture; run with --ignored"]
fn jdk_execution_matches_the_original_for_numeric_boundaries_and_effects() {
    let engine = Engine::new();
    let fixture = fixture(&engine);
    let methods = [
        ("negInt", "(I)I", "public static int negInt(int arg0)"),
        ("negLong", "(J)J", "public static long negLong(long arg0)"),
        (
            "negFloat",
            "(F)F",
            "public static float negFloat(float arg0)",
        ),
        (
            "negDouble",
            "(D)D",
            "public static double negDouble(double arg0)",
        ),
        ("negByte", "(B)I", "public static int negByte(byte arg0)"),
        ("negChar", "(C)I", "public static int negChar(char arg0)"),
        ("negShort", "(S)I", "public static int negShort(short arg0)"),
        ("nested", "(I)I", "public static int nested(int arg0)"),
        (
            "negSum",
            "(II)I",
            "public static int negSum(int arg0, int arg1)",
        ),
        (
            "multiplyRight",
            "(II)I",
            "public static int multiplyRight(int arg0, int arg1)",
        ),
        (
            "divideRight",
            "(II)I",
            "public static int divideRight(int arg0, int arg1)",
        ),
        ("local", "(I)I", "public static int local(int arg0)"),
        (
            "callArgument",
            "(I)I",
            "public static int callArgument(int arg0)",
        ),
        ("throwing", "(I)I", "public static int throwing(int arg0)"),
        (
            "negatedCall",
            "(I)I",
            "public static int negatedCall(int arg0)",
        ),
        (
            "negatedFail",
            "(I)I",
            "public static int negatedFail(int arg0)",
        ),
    ];
    let mut recovered = String::from("public final class RecoveredUnaryNegation {\n");
    recovered.push_str(
        "    static int calls;\n    static int consume(int value) { calls++; return value; }\n    static int fail(int value) { throw new IllegalStateException(Integer.toString(value)); }\n",
    );
    for (name, descriptor, declaration) in methods {
        let report = recover(&engine, &fixture, name.as_bytes(), descriptor.as_bytes());
        assert_eq!(
            report.representation,
            Representation::Java,
            "{}",
            report.text
        );
        recovered.push_str("    ");
        recovered.push_str(declaration);
        recovered.push_str(" {\n");
        for line in body(&report).lines() {
            recovered.push_str("        ");
            recovered.push_str(line);
            recovered.push('\n');
        }
        recovered.push_str("    }\n");
    }
    recovered.push_str("}\n");

    let driver = r#"
public final class UnaryNegationDriver {
    private static String f(float value) {
        if (Float.isNaN(value)) return "NaN";
        return Integer.toHexString(Float.floatToRawIntBits(value));
    }
    private static String d(double value) {
        if (Double.isNaN(value)) return "NaN";
        return Long.toHexString(Double.doubleToRawLongBits(value));
    }
    private static String thrownInt(int value, boolean recovered) {
        try {
            if (recovered) RecoveredUnaryNegation.throwing(value);
            else UnaryNegation.throwing(value);
            return "returned";
        } catch (IllegalStateException error) {
            return error.getClass().getSimpleName() + ":" + error.getMessage();
        }
    }
    private static String thrownNegated(int value, boolean recovered) {
        try {
            if (recovered) RecoveredUnaryNegation.negatedFail(value);
            else UnaryNegation.negatedFail(value);
            return "returned";
        } catch (IllegalStateException error) {
            return error.getClass().getSimpleName() + ":" + error.getMessage();
        }
    }
    private static String trace(boolean recovered) {
        StringBuilder result = new StringBuilder();
        result.append(recovered ? RecoveredUnaryNegation.negInt(Integer.MIN_VALUE) : UnaryNegation.negInt(Integer.MIN_VALUE)).append('\n');
        result.append(recovered ? RecoveredUnaryNegation.negLong(Long.MIN_VALUE) : UnaryNegation.negLong(Long.MIN_VALUE)).append('\n');
        float[] floats = {-0.0f, 0.0f, Float.POSITIVE_INFINITY, Float.NEGATIVE_INFINITY, Float.NaN};
        for (float value : floats) result.append(f(recovered ? RecoveredUnaryNegation.negFloat(value) : UnaryNegation.negFloat(value))).append('\n');
        double[] doubles = {-0.0d, 0.0d, Double.POSITIVE_INFINITY, Double.NEGATIVE_INFINITY, Double.NaN};
        for (double value : doubles) result.append(d(recovered ? RecoveredUnaryNegation.negDouble(value) : UnaryNegation.negDouble(value))).append('\n');
        result.append(recovered ? RecoveredUnaryNegation.negByte((byte)-128) : UnaryNegation.negByte((byte)-128)).append('\n');
        result.append(recovered ? RecoveredUnaryNegation.negChar((char)65) : UnaryNegation.negChar((char)65)).append('\n');
        result.append(recovered ? RecoveredUnaryNegation.negShort((short)-32768) : UnaryNegation.negShort((short)-32768)).append('\n');
        result.append(recovered ? RecoveredUnaryNegation.nested(7) : UnaryNegation.nested(7)).append('\n');
        result.append(recovered ? RecoveredUnaryNegation.negSum(3, 4) : UnaryNegation.negSum(3, 4)).append('\n');
        result.append(recovered ? RecoveredUnaryNegation.multiplyRight(6, 3) : UnaryNegation.multiplyRight(6, 3)).append('\n');
        result.append(recovered ? RecoveredUnaryNegation.divideRight(18, 3) : UnaryNegation.divideRight(18, 3)).append('\n');
        result.append(recovered ? RecoveredUnaryNegation.local(9) : UnaryNegation.local(9)).append('\n');
        if (recovered) RecoveredUnaryNegation.calls = 0; else UnaryNegation.calls = 0;
        result.append(recovered ? RecoveredUnaryNegation.callArgument(11) : UnaryNegation.callArgument(11)).append(':').append(recovered ? RecoveredUnaryNegation.calls : UnaryNegation.calls).append('\n');
        result.append(thrownInt(13, recovered)).append('\n');
        result.append(thrownNegated(17, recovered)).append('\n');
        if (recovered) RecoveredUnaryNegation.calls = 0; else UnaryNegation.calls = 0;
        result.append(recovered ? RecoveredUnaryNegation.negatedCall(19) : UnaryNegation.negatedCall(19)).append(':').append(recovered ? RecoveredUnaryNegation.calls : UnaryNegation.calls).append('\n');
        return result.toString();
    }
    public static void main(String[] args) {
        System.out.print(trace(false));
        System.out.print("--generated--\n");
        System.out.print(trace(true));
    }
}
"#;
    let dir = TempDir::new();
    fs::write(dir.path().join("UnaryNegation.class"), FIXTURE).expect("write original class");
    dir.write("RecoveredUnaryNegation.java", &recovered);
    dir.write("UnaryNegationDriver.java", driver);
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-cp"])
        .arg(dir.path())
        .args(["-d"])
        .arg(dir.path())
        .arg(dir.path().join("RecoveredUnaryNegation.java"))
        .arg(dir.path().join("UnaryNegationDriver.java"))
        .output()
        .expect("run javac for the controlled fixture");
    assert!(
        compile.status.success(),
        "javac failed: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-cp"])
        .arg(dir.path())
        .arg("UnaryNegationDriver")
        .output()
        .expect("run the controlled fixture comparison");
    assert!(
        run.status.success(),
        "java failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let output = String::from_utf8(run.stdout).expect("the comparison trace is UTF-8");
    let (original, generated) = output
        .split_once("--generated--\n")
        .expect("driver emits original and generated sections");
    assert_eq!(
        original, generated,
        "the recovered methods match the original trace"
    );
}
