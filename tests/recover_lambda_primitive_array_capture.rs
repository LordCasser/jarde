//! Acceptance tests of change `recover-lambda-primitive-array-capture` (DT-26): an
//! `invokedynamic` site that captures a **primitive-array** local the body created with `newarray`
//! (`int[] t = {0}; l.forEach(i -> t[0] += i);`) is no longer refused by the capture check.
//!
//! What changed is the *input* to the three-way check, never the check. The frame pass leaves
//! `newarray`'s result an unknown reference on purpose (the array of a primitive element type is
//! defined by the bootstrap loader, which a standalone read does not declare), and the capture type
//! was read from that frame entry alone — so the check compared the **absence of a statement**
//! (`Object`) against the site's and the implementation's own `int[]` and refused. The type now
//! comes from `crate::build::capture_value_type`: the frame's named reference when it has one, and
//! otherwise the array the **creating instruction's own `atype`** states, which is the same fact
//! `written_type` already reads to declare `int[] local1 = new int[]{0};`. The check itself
//! (`jre_lambda_sam_types`) is byte-for-byte unchanged, and a site whose three statements genuinely
//! disagree still refuses (the crate-level probes in
//! `crates/jarde-java/tests/p3_java_recovery.rs`).
//!
//! The frozen fixtures under `tests/fixtures/recover-lambda-primitive-array-capture/` are the dual
//! legs of the patrol probe: `v8/` compiled by **real javac 8** (Corretto 1.8.0_432) and `v23/` by
//! `javac --release 8` (23.0.1) of the same source. Both legs were compiled with javac's default
//! `-g:lines,source`, so **neither class carries a `LocalVariableTable`** — the measured fact that
//! decided the direction (the change's `instrumentation.md`).
//!
//! Frozen behaviors:
//! 1. the main anchor (`P02_lambda`: `sum` captures `int[]`, `map` captures `StringBuilder`) —
//!    both legs render byte-identical to the frozen post-change record, whose presentation source
//!    region carries **zero** quotes (the frozen pre-change record carries the refusal at
//!    `@bytecode 15 8 9`), and the rendered text recompiles under `javac --release 8` and prints
//!    what the original classes print (`6`, `ab`);
//! 2. zero regression — `map`, its `lambda$map$1$jarde` companion and `main` are byte-identical
//!    between the two frozen records of each leg;
//! 3. `multianewarray` (`P02_multianewarray`, `int[][]`) is out of this change's scope and renders
//!    byte-identical before and after: its capture site was never refused (the pool names `[[I`, so
//!    the frame stated the type), and the quotes it carries are the companion body's two-dimensional
//!    compound assignment — DT-26's existing expression domain, not the capture gate.

use jarde::class_source::ClassSourceReport;
use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

/// The two frozen legs: real javac 8, and `javac --release 8` on the toolchain's JDK.
const LEGS: [&str; 2] = ["v8", "v23"];

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-lambda-array-capture-{label}-{}-{nonce}-{}",
            std::process::id(),
            NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("a private compilation directory is created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn jar_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut zip = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in entries {
            let (mut entry, config) = zip
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(bytes).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        zip.finish().unwrap();
    }
    output.into_inner()
}

fn fixture(leg: &str, class: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/recover-lambda-primitive-array-capture")
        .join(leg)
        .join(format!("{class}.class"));
    std::fs::read(&path).unwrap_or_else(|error| panic!("fixture {} reads: {error}", path.display()))
}

/// One frozen record of this change's evidence directory: the pre-change rendering (the refusal
/// baseline and the zero-regression comparison) and the accepted post-change rendering.
fn evidence(directory: &str, name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("openspec/evidence/java-syntax-2026-10-05/recover-lambda-primitive-array-capture")
        .join(directory)
        .join(name);
    let text =
        std::fs::read(&path).unwrap_or_else(|error| panic!("evidence {name} reads: {error}"));
    String::from_utf8(text).unwrap_or_else(|error| panic!("evidence {name} is utf-8: {error}"))
}

/// The probe source the frozen classes were compiled from, as the patrol recorded it.
fn probe_source(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("openspec/evidence/java-syntax-2026-10-04/dual-javac-sweep/probes")
        .join(name);
    let text = std::fs::read(&path).unwrap_or_else(|error| panic!("probe {name} reads: {error}"));
    String::from_utf8(text).expect("the probe source is utf-8")
}

fn limits() -> Limits {
    jarde::facade::task_limits(&[]).expect("the task defaults are a bounded budget")
}

/// Renders one class of one single-entry jar through the library entry `class-source` uses, at
/// release 8 under the `PlainJar` policy — the posture the frozen records were taken in.
fn source_of(jar: &[u8], class: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(jar.to_vec()), &mut budget)
        .expect("the fixture jar opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    match engine.class_source(
        std::slice::from_ref(&snapshot),
        &request,
        &mut Budget::new(limits()),
    ) {
        Ok(OperationOutcome::Performed(report)) => report,
        other => panic!("one class answers one class-source request: {other:?}"),
    }
}

fn anchor_jar(leg: &str) -> Vec<u8> {
    jar_of(&[(b"P02_lambda.class", fixture(leg, "P02_lambda").as_slice())])
}

fn multianewarray_jar(leg: &str) -> Vec<u8> {
    jar_of(&[(
        b"P02_multianewarray.class",
        fixture(leg, "P02_multianewarray").as_slice(),
    )])
}

/// The `// @bytecode` quotes one text carries.
fn quotes(text: &str) -> usize {
    text.lines()
        .filter(|line| line.trim().starts_with("// @bytecode"))
        .count()
}

/// One member's text out of a frozen rendering. The assembled source separates members by a blank
/// line, and each recovered member states its own `// @method <name><descriptor>` envelope line, so
/// the member is found by that line: the walk back stops at the member's **declaration** (the
/// nearest preceding four-space line that is not a class-level `// jarde:` marker, which belongs to
/// the class and not to this member), and the walk forward stops at the closing `    }`.
fn member_of_rendering(text: &str, name: &str) -> String {
    let envelope = format!("// @method {name}(");
    let start = text
        .find(&envelope)
        .unwrap_or_else(|| panic!("`{name}` is not in the frozen rendering:\n{text}"));
    let before: Vec<&str> = text[..start].split('\n').collect();
    let declaration = before
        .iter()
        .rfind(|line| {
            line.starts_with("    ")
                && !line.starts_with("     ")
                && !line.trim().starts_with("//")
                && !line.trim().is_empty()
        })
        .unwrap_or_else(|| panic!("`{name}` has no declaration line:\n{text}"));
    let declaration_index = text[..start]
        .rfind(declaration)
        .expect("the declaration line is before the envelope");
    let after = &text[declaration_index..];
    let end = after
        .find("\n    }\n")
        .or_else(|| after.ends_with("\n    }").then(|| after.len() - 5))
        .expect("the member closes at its own brace");
    after[..end + "\n    }".len()].to_owned()
}

// 1. The main anchor: both legs render exactly the frozen post-change record, and it carries no
//    quote in the presentation source region.
#[test]
fn the_primitive_array_capture_recovers_in_both_javac_legs() {
    for leg in LEGS {
        let report = source_of(&anchor_jar(leg), "P02_lambda");
        assert_eq!(
            report.text,
            evidence("fixed", &format!("P02_lambda-{leg}.fixed.txt")),
            "the {leg} rendering must equal the frozen fixed-leg record"
        );
        assert_eq!(
            quotes(&report.text),
            0,
            "the recovered {leg} presentation carries a quote:\n{}",
            report.text
        );
        assert!(
            !report.text.contains("captured operand"),
            "the capture-type refusal is gone from the {leg} presentation:\n{}",
            report.text
        );
        // `sum` captures its array in the same inline shape `map` has always had for `StringBuilder`.
        let sum = member_of_rendering(&report.text, "sum");
        assert!(
            sum.contains("int[] local1 = new int[]{0};"),
            "`sum` still declares the array the creation made:\n{sum}"
        );
        assert!(
            sum.contains(
                "arg0.forEach((java.util.function.Consumer) ((java.lang.Object p0) -> \
                 P02_lambda.lambda$sum$0$jarde(local1, (java.lang.Integer) p0)));"
            ),
            "`sum` must capture the array at the site:\n{sum}"
        );
        // The companion the site now names takes the array it was captured with. Its envelope
        // states the physical name javac synthesized; the presentation renames it `$jarde`.
        let companion = member_of_rendering(&report.text, "lambda$sum$0");
        assert!(
            companion.contains("private static void lambda$sum$0$jarde(int[] arg0,")
                && companion.contains("arg0[0] += arg1.intValue();"),
            "the companion keeps the `int[]` parameter and its compound store:\n{companion}"
        );
        assert_eq!(
            quotes(&companion),
            0,
            "the companion body of this shape is written in full:\n{companion}"
        );
    }
    // No version branch: the two legs answer identically (the class bytes differ, the presentation
    // does not). This is the "本形双腿一致" claim of the change, stated as an assertion.
    assert_eq!(
        evidence("fixed", "P02_lambda-v8.fixed.txt"),
        evidence("fixed", "P02_lambda-v23.fixed.txt"),
        "the two legs must render the same text"
    );
}

// 1b. The falsifier: the same bytes, recorded before the change, refuse the site at the three BCIs.
// Without this the test above could be about a fixture that always worked.
#[test]
fn the_frozen_baseline_of_the_same_anchor_refused_the_capture() {
    for leg in LEGS {
        let baseline = evidence("baseline", &format!("P02_lambda-{leg}.baseline.txt"));
        assert_eq!(
            quotes(&baseline),
            1,
            "the pre-change record quotes exactly the capture site:\n{baseline}"
        );
        assert!(
            baseline.contains("// @bytecode 15 8 9"),
            "the pre-change record names the site and the capture's BCIs:\n{baseline}"
        );
        assert!(
            baseline.contains(
                "captured operand 0 is `Object` in the frame, `int[]` in the site descriptor and \
                 `int[]` in the implementation: the capture conversion and its creation-time \
                 effects are not proven"
            ),
            "the pre-change refusal was the frame's unstated array:\n{baseline}"
        );
        // The site kept its bytecode; only `map` was presented. The refusal is version-independent.
        let sum = member_of_rendering(&baseline, "sum");
        assert!(
            !sum.contains("->"),
            "the pre-change `sum` quoted its site instead of presenting a lambda:\n{sum}"
        );
        assert_eq!(
            evidence("baseline", "P02_lambda-v8.baseline.txt"),
            evidence("baseline", "P02_lambda-v23.baseline.txt"),
            "both legs refused identically before the change"
        );
    }
}

// 1c. The recovered text recompiles as Java 8 and prints what the original classes print, in both
//     legs: the recovery is behavior-preserving, not merely prettier.
#[test]
fn the_recovered_text_recompiles_and_prints_the_original_output() {
    for leg in LEGS {
        let rendered = source_of(&anchor_jar(leg), "P02_lambda");
        let temp = TestDirectory::new(&format!("recompile-{leg}"));
        std::fs::write(temp.path().join("P02_lambda.java"), &rendered.text)
            .expect("the rendered text is written");
        std::fs::write(temp.path().join("original.jar"), anchor_jar(leg))
            .expect("the original jar is written");
        let compiled = Command::new("javac")
            .args(["--release", "8", "-Xlint:-options", "P02_lambda.java"])
            .current_dir(temp.path())
            .output()
            .expect("javac runs");
        assert!(
            compiled.status.success(),
            "the recovered {leg} source compiles under --release 8:\n{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let run = |classpath: &str, label: &str| {
            let output = Command::new("java")
                .args(["-Xverify:all", "-cp", classpath])
                .arg("P02_lambda")
                .current_dir(temp.path())
                .output()
                .unwrap_or_else(|error| panic!("{label}: java runs: {error}"));
            assert!(output.status.success(), "{label}: the run succeeds");
            String::from_utf8(output.stdout).expect("the run prints text")
        };
        let original = run("original.jar", "original class");
        let recovered = run(".", "recompiled rendering");
        assert_eq!(
            original, "6\nab\n",
            "the frozen behavior baseline of the {leg} anchor"
        );
        assert_eq!(
            recovered, original,
            "the recompiled {leg} rendering behaves like the original classes"
        );
        assert_eq!(
            recovered,
            evidence("fixed", &format!("P02_lambda-{leg}.recompiled-run.stdout")),
            "the {leg} run equals the frozen run record"
        );
    }
}

// 2. Zero regression, member by member: the reference capture that always worked, its companion,
//    and `main` are byte-identical between the frozen records of each leg.
#[test]
fn the_reference_capture_and_the_rest_of_the_class_are_unchanged() {
    for leg in LEGS {
        let before = evidence("baseline", &format!("P02_lambda-{leg}.baseline.txt"));
        let after = evidence("fixed", &format!("P02_lambda-{leg}.fixed.txt"));
        for name in ["map", "lambda$map$1", "main", "<init>"] {
            assert_eq!(
                member_of_rendering(&before, name),
                member_of_rendering(&after, name),
                "`{name}` changed in the {leg} rendering"
            );
        }
        // The reference capture's inline shape is the pattern the array capture now matches.
        let map = member_of_rendering(&after, "map");
        assert!(
            map.contains("java.lang.StringBuilder local1 = new java.lang.StringBuilder();")
                && map.contains("lambda$map$1$jarde(local1, (java.lang.String) p0)"),
            "`map` keeps its recorded presentation:\n{map}"
        );
        assert!(
            !map.contains("@bytecode"),
            "`map` carries no quote either before or after:\n{map}"
        );
    }
}

// 3. The historical capture snapshots remain frozen; nested update recovery changes only the
//    helper body and preserves the already proved capture and other members in both legs.
#[test]
fn the_multianewarray_capture_preserves_its_members_while_the_nested_helper_recovers() {
    for leg in LEGS {
        let report = source_of(&multianewarray_jar(leg), "P02_multianewarray");
        let before = evidence(
            "baseline",
            &format!("P02_multianewarray-{leg}.baseline.txt"),
        );
        let after = evidence("fixed", &format!("P02_multianewarray-{leg}.fixed.txt"));
        assert_eq!(
            before, after,
            "the {leg} historical baseline/fixed snapshots remain unchanged"
        );
        for name in ["<init>", "sum", "main"] {
            assert_eq!(
                member_of_rendering(&report.text, name),
                member_of_rendering(&after, name),
                "the {leg} `{name}` region stays unchanged by nested helper recovery"
            );
        }
        // Its capture site was never refused: `multianewarray` names its class in the pool, so the
        // frame stated `[[I` and the three-way check had real evidence to compare.
        assert!(
            !report.text.contains("captured operand"),
            "the {leg} multianewarray capture was refused:\n{}",
            report.text
        );
        let sum = member_of_rendering(&report.text, "sum");
        assert!(
            sum.contains("int[][] local1 = new int[][]{new int[]{0}};")
                && sum.contains("lambda$sum$0$jarde(local1, (java.lang.Integer) p0)"),
            "the {leg} `int[][]` capture is presented inline and was before too:\n{sum}"
        );
        // Nested compound recovery changes only the helper: the prior fixed snapshot remains the
        // historical record, while its lambda body now owns the original row/index copies.
        let companion = member_of_rendering(&report.text, "lambda$sum$0");
        assert_eq!(
            quotes(&companion),
            0,
            "{leg} helper has no bytecode quote:\n{companion}"
        );
        assert!(
            companion.contains("arg0[0][0] += arg1.intValue();"),
            "{leg} helper preserves the nested int compound update:\n{companion}"
        );
        let helper = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"lambda$sum$0")
            .expect("the P02 companion helper is present");
        let helper_body = match &helper.outcome {
            ClassSourceOutcome::Recovered { report, .. } => report,
            other => panic!("{leg} helper has no complete recovery record: {other:?}"),
        };
        assert_eq!(
            helper_body.quality,
            Quality::Structured,
            "{leg}: {helper_body:?}"
        );
        assert_eq!(helper_body.representation, Representation::Java, "{leg}");
        for bci in [0, 1, 2, 3, 4, 5, 6, 7, 10, 11, 12] {
            assert!(
                !helper_body.source_map.of_bci(bci).is_empty(),
                "{leg} helper source map omitted physical BCI {bci}: {:?}",
                helper_body.source_map.segments()
            );
        }
        assert_eq!(
            quotes(&report.text),
            0,
            "the {leg} complete class has no bytecode quotes:\n{}",
            report.text
        );
    }
}

// The frozen classes are the patrol's own probe, compiled by the two documented javac legs — not a
// hand-written stand-in. `P02_lambda` is copied verbatim; `P02_multianewarray` is the change's own
// `int[][]` control, written beside it.
#[test]
fn the_frozen_anchor_is_the_recorded_patrol_probe() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/recover-lambda-primitive-array-capture")
        .join("P02_lambda.java");
    assert_eq!(
        std::fs::read_to_string(&source).expect("the fixture source reads"),
        probe_source("P02_lambda.java"),
        "both legs are compiled from the patrol's own `P02_lambda` probe"
    );
    // Both legs are Java 8 class files (major 52) — the posture every other fixture of this family
    // is read in, and what makes the `--release 8` leg comparable to the real javac 8 one.
    for leg in LEGS {
        for class in ["P02_lambda", "P02_multianewarray"] {
            let bytes = fixture(leg, class);
            assert_eq!(
                &bytes[0..8],
                &[0xca, 0xfe, 0xba, 0xbe, 0x00, 0x00, 0x00, 0x34],
                "{leg}/{class} is a major-52 (Java 8) class file"
            );
            // javac's default `-g:lines,source` emits no LocalVariableTable; that measured absence is
            // why the direction is the `atype` and not a debug-info name table. A `Code` attribute
            // here declares exactly one nested attribute count of `LineNumberTable`-and-signature
            // shape, so the simplest honest statement of the fact is the byte the classes carry.
            assert!(
                !bytes
                    .windows(19)
                    .any(|window| window == b"LocalVariableTable"),
                "{leg}/{class} unexpectedly carries a LocalVariableTable"
            );
        }
    }
}
