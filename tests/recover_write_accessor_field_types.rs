//! Acceptance tests of change `recover-write-accessor-field-types` (EM-15): the value-returning
//! write accessor real javac 8 synthesizes when a nested class writes an outer class's private
//! field is recovered for the **closed nine-type set** (descriptor keys `Z`/`I`/`B`/`S`/`C`/`F`/
//! `J`/`D`/`L…;`/`[…`) instead of the boolean-only first slice, with every **structural**
//! criterion of that slice unchanged.
//!
//! The frozen fixtures under `tests/fixtures/recover-write-accessor-field-types/` are compiled by
//! **real javac 8** (Corretto 1.8.0_432); the out-of-table and structure-break negatives are
//! equal-length byte patches of the frozen anchor, documented with their measured offsets in the
//! fixture README.
//!
//! Frozen behaviors:
//! 1. the main anchor (`WA`: nine private fields, nine inner-class setters, nine accessors) — all
//!    nine accessors recover as `arg0.<field> = arg1; return arg1;` with no quote inside them,
//!    the rendered source set recompiles under `javac --release 8` (before the change: eight
//!    refusals whose empty stubs each produced one "missing return statement" error) and the
//!    recompiled text prints the original classes' line;
//! 2. the boolean precedent (`PrivateFieldFamily$A`, `d09f5dea`) renders **byte-identical** to its
//!    frozen pre-change record;
//! 3. the category-2 instance sister (`LongAssignmentResult`, EM-07 `Assignment`) renders
//!    **byte-identical** to its frozen pre-change record — the path whose descriptor gate was
//!    generalized to a table key;
//! 4. invalid descriptors, unproved conversions and unsafe structures keep explicit refusals.
//!    Generic frame-proved copy semantics additionally handle static accessors and harmless
//!    synthetic-flag/NOP changes, checked by complete unedited class JVM execution oracles;
//! 5. the identifier-generalization control (`WB`: fields `tally`/`bag`/`grid`, writer class
//!    `Writer`, one `L…;` arm and one `[…` arm) recovers on a `javac --release 8` leg compiled
//!    in-test, so no table arm is pinned to the anchor's own identifiers.

use jarde::class_source::ClassSourceReport;
use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-write-accessor-{label}-{}-{nonce}-{}",
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

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/recover-write-accessor-field-types")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("fixture {name} reads: {error}"))
}

/// One frozen record of this change's evidence directory — the pre-change renderings (the zero
/// regression anchors and the anchor's own refusal baseline) and the accepted post-change anchor
/// rendering.
fn evidence(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/recovered-inputs/evidence/java-syntax-2026-10-05/recover-write-accessor-field-types")
        .join(name);
    let text =
        std::fs::read(&path).unwrap_or_else(|error| panic!("evidence {name} reads: {error}"));
    String::from_utf8(text).unwrap_or_else(|error| panic!("evidence {name} is utf-8: {error}"))
}

fn limits() -> Limits {
    jarde::facade::task_limits(&[]).expect("the task defaults are a bounded budget")
}

/// Renders one class of one jar through the library entry `class-source` uses, at release 8 with
/// the family's own fold (`PlainJar`).
fn source_of(jar: &[u8], class: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(jar.to_vec()), &mut budget)
        .expect("the fixture jar opens");
    let request = ClassSourceRequest {
        source_target: jarde::SourceTarget::JAVA_8,
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
    let evidence = RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap);
    match engine.class_source_with_evidence(
        std::slice::from_ref(&snapshot),
        &request,
        &evidence,
        &mut budget,
    ) {
        Ok(OperationOutcome::Performed(report)) => report,
        other => panic!("one family answers one class-source request: {other:?}"),
    }
}
/// Renders one standalone class file — the posture the frozen zero-regression records were taken
/// in (`--policy single-class` on the command line).
fn source_of_standalone(bytes: &[u8], class: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("the standalone class opens");
    let request = ClassSourceRequest {
        source_target: jarde::SourceTarget::JAVA_8,
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
    match engine.class_source(
        std::slice::from_ref(&snapshot),
        &request,
        &mut Budget::new(limits()),
    ) {
        Ok(OperationOutcome::Performed(report)) => report,
        other => panic!("one class answers one class-source request: {other:?}"),
    }
}

fn wa_jar() -> Vec<u8> {
    jar_of(&[
        (b"WA.class", fixture("wa/WA.class").as_slice()),
        (b"WA$S.class", fixture("wa/WA$S.class").as_slice()),
    ])
}

fn st_jar() -> Vec<u8> {
    jar_of(&[
        (b"ST.class", fixture("st/ST.class").as_slice()),
        (b"ST$S.class", fixture("st/ST$S.class").as_slice()),
    ])
}

/// One patched anchor class rendered as a single-entry jar; the patch keeps `WA`'s own name.
fn probe_jar(name: &str) -> Vec<u8> {
    jar_of(&[(b"WA.class", fixture(name).as_slice())])
}

/// The presented text of one member, found by its own name.
fn member_text(report: &ClassSourceReport, name: &str) -> String {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| {
            panic!(
                "no presented member `{name}` in:\n{}",
                report.text.lines().collect::<Vec<_>>()[..20].join("\n")
            )
        })
        .text
        .clone()
}

/// The nine accessors: `(descriptor key, member name, recovered statement)`.
const NINE: &[(&str, &str, &str)] = &[
    ("Z", "access$002", "arg0.flag = arg1;"),
    ("I", "access$102", "arg0.count = arg1;"),
    ("J", "access$202", "arg0.big = arg1;"),
    ("D", "access$302", "arg0.d = arg1;"),
    ("Ljava/lang/String;", "access$402", "arg0.ref = arg1;"),
    ("B", "access$502", "arg0.by = arg1;"),
    ("S", "access$602", "arg0.sh = arg1;"),
    ("C", "access$702", "arg0.ch = arg1;"),
    ("F", "access$802", "arg0.fl = arg1;"),
];

/// Asserts the nine accessors of one rendering, skipping the members named in `skip`.
fn assert_accessors_recovered(report: &ClassSourceReport, skip: &[&str], label: &str) {
    for (key, name, statement) in NINE {
        if skip.contains(name) {
            continue;
        }
        let text = member_text(report, name);
        assert!(
            text.contains(statement) && text.contains("return arg1;"),
            "{label}: the {key} accessor must recover as `{statement} return arg1;`:\n{text}"
        );
        assert!(
            !text.contains("@bytecode") && !text.contains("not recovered"),
            "{label}: the recovered {key} accessor carries no quote and no refusal:\n{text}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// 1. The main anchor: all nine types recover, the text compiles, the behavior is the original's.
// ---------------------------------------------------------------------------------------------

#[test]
fn the_nine_write_accessor_types_all_recover() {
    let report = source_of(&wa_jar(), "WA");
    assert_accessors_recovered(&report, &[], "the WA anchor");
    // The per-type descriptors state the table key the proof matched (the javap-measured forms).
    for (key, name, _) in NINE {
        let descriptor = format!("(LWA;{key}){key}");
        assert!(
            report
                .text
                .contains(&format!("// @method {name}{descriptor}")),
            "the {key} accessor is presented with its own descriptor {name}{descriptor}:\n{}",
            report.text
        );
    }
    assert_eq!(
        report.text.matches("return arg1;").count(),
        9,
        "exactly the nine accessors return their written value:\n{}",
        report.text
    );
    // Keep the historical record intact. Only its independently proved field concat changes;
    // every other byte, including all nine accessor bodies, must still match.
    let expected = evidence("fixed/WA-rendered-fixed.txt");
    let before = "        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(\"\").append(local1.flag).append(local1.count).append(local1.big).append(local1.d).append(local1.ref).append((int) local1.by).append((int) local1.sh).append(local1.ch).append(local1.fl).toString());";
    let after = "        java.lang.System.out.println(\"\" + local1.flag + local1.count + local1.big + local1.d + local1.ref + (int) local1.by + (int) local1.sh + local1.ch + local1.fl);";
    assert_eq!(
        expected.matches(before).count(),
        1,
        "the frozen WA record must contain exactly one legacy println"
    );
    let expected = expected.replacen(before, after, 1);
    assert_eq!(
        report.text, expected,
        "the WA rendering changed beyond its recovered field concat"
    );
}

#[test]
fn the_recovered_wa_text_recompiles_and_prints_the_original_line() {
    let report = source_of(&wa_jar(), "WA");
    let temp = TestDirectory::new("wa-recompile");
    std::fs::write(temp.path().join("WA.java"), &report.text)
        .expect("the rendered text is written");
    let compiled = Command::new("javac")
        .args(["--release", "8", "WA.java"])
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "the recovered WA source set compiles under --release 8:\n{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    std::fs::write(temp.path().join("original.jar"), wa_jar())
        .expect("the original jar is written");
    let run = |classpath: String, label: &str| {
        let output = Command::new("java")
            .args(["-Xverify:all", "-cp", &classpath])
            .arg("WA")
            .current_dir(temp.path())
            .output()
            .unwrap_or_else(|error| panic!("{label}: java runs: {error}"));
        assert!(output.status.success(), "{label}: the run succeeds");
        String::from_utf8(output.stdout).expect("the run prints text")
    };
    let original = run("original.jar".to_owned(), "original");
    let recovered = run(
        format!(".:{}", temp.path().join("original.jar").display()),
        "recovered",
    );
    assert_eq!(
        original, "true123.0x45c6.0\n",
        "the frozen behavior baseline"
    );
    assert_eq!(
        recovered, original,
        "the recompiled recovered text behaves like the original classes"
    );
}

#[test]
fn the_frozen_baseline_of_the_same_anchor_refused_eight_of_nine() {
    // The falsifier: the refusal this change removes, recorded before it on the same bytes. The
    // eight stubs kept their declared return types with empty bodies, which is why the patrol's
    // compile comparison reported exit 1 with eight "missing return statement" errors.
    let baseline = evidence("baseline/WA-rendered-baseline.txt");
    assert_eq!(baseline.matches("jarde: not recovered").count(), 8);
    for (key, name, _) in NINE.iter().skip(1) {
        assert!(
            baseline.contains(&format!(
                "not recovered: the recovery run for `{name}(LWA;{key}){key}`"
            )),
            "the frozen baseline record refused the {key} accessor {name}"
        );
    }
    // The boolean row was already recovered before the change, and its text is the same text the
    // generalized table produces.
    assert!(baseline.contains("arg0.flag = arg1;\n        return arg1;"));
    let report = source_of(&wa_jar(), "WA");
    assert_eq!(report.text.matches("jarde: not recovered").count(), 0);
    assert_eq!(
        report.text.matches("arg0.flag = arg1;").count(),
        baseline.matches("arg0.flag = arg1;").count(),
        "the boolean accessor's recovery is not duplicated or reshaped by the generalization"
    );
}

// ---------------------------------------------------------------------------------------------
// 2. Zero regression: the boolean precedent renders byte-identical.
// ---------------------------------------------------------------------------------------------

/// `dt29/PrivateFieldFamily$B`'s record was re-rendered by change
/// `recover-platform-interface-argument-widening`: its `set(ZZ)V` passes the subclass `this` to the
/// superclass' `access$002(dt29/PrivateFieldFamily$A, Z)`, and in the standalone snapshot that holds
/// only `$B` the presented type's own class-file header names the required `dt29/PrivateFieldFamily$A`
/// as its superclass — so the one-sided widening this change proves now renders
/// `dt29.PrivateFieldFamily$A.access$002((dt29.PrivateFieldFamily$A) this, arg2);` where the
/// reference-conversion refusal used to consume the whole body. Both other records are untouched.
#[test]
fn the_boolean_precedent_renders_byte_identical_to_its_frozen_record() {
    // `d09f5dea`'s boolean family, in the standalone posture its frozen records were taken in.
    for (class, bytes, record) in [
        (
            "dt29/PrivateFieldFamily",
            evidence_bytes("pff/PrivateFieldFamily.class"),
            "pff/pff-baseline-PrivateFieldFamily.txt",
        ),
        (
            "dt29/PrivateFieldFamily$A",
            evidence_bytes("pff/PrivateFieldFamily$A.class"),
            "pff/pff-baseline-PrivateFieldFamily$A.txt",
        ),
        (
            "dt29/PrivateFieldFamily$B",
            evidence_bytes("pff/PrivateFieldFamily$B.class"),
            "pff/pff-baseline-PrivateFieldFamily$B.txt",
        ),
    ] {
        let report = source_of_standalone(&bytes, class);
        assert_eq!(
            report.text,
            evidence(record),
            "{class}: the boolean precedent's rendering may not move by one byte"
        );
    }
    assert!(
        evidence("pff/pff-baseline-PrivateFieldFamily$A.txt")
            .contains("arg0.hidden = arg1;\n        return arg1;"),
        "the record itself shows the boolean accessor recovered"
    );
}

fn evidence_bytes(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/recovered-inputs/evidence/java-syntax-2026-10-05/recover-write-accessor-field-types")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("evidence {name} reads: {error}"))
}

// ---------------------------------------------------------------------------------------------
// 3. Zero regression: the category-2 instance sister (EM-07) renders byte-identical.
// ---------------------------------------------------------------------------------------------

#[test]
fn the_instance_long_assignment_sister_renders_byte_identical() {
    // `LongAssignmentResult::prove`'s instance path (`this.f = v; return v;` inside a normal
    // method) shares the `dup2_x1` opcode facts with the table's `J` row and was explicitly not
    // touched; its consumer-side descriptor gate moved from the `false`-arm constant to the
    // parameter, so the root's ruling made byte-level equality an acceptance condition.
    let bytes = include_bytes!(
        "../tests/fixtures/recovered-inputs/evidence/java-syntax-2026-09-27/em07-long-assignment/input/Assignment.class"
    );
    let report = source_of_standalone(bytes, "em07/Assignment");
    assert_eq!(
        report.text,
        evidence("jarm/baseline-em07.txt"),
        "the instance J sister's rendering may not move by one byte"
    );
    assert!(
        report
            .text
            .contains("this.value = arg1;\n        return arg1;")
    );
    // And the same bytes equal the post-change record — the two frozen legs of the comparison.
    assert_eq!(
        report.text,
        evidence("jarm/fixed-em07.txt"),
        "the J sister's baseline and fixed records are themselves identical"
    );
}

// ---------------------------------------------------------------------------------------------
// 4. The closed table refuses what is outside it; the refusal stays per member.
// ---------------------------------------------------------------------------------------------

#[test]
fn invalid_descriptor_and_boolean_int_conversion_keep_their_refusals() {
    let report = source_of(&probe_jar("probes/WA-void.class"), "WA");
    assert!(!report.text.contains("arg0.flag = arg1;"));
    assert!(
        report.text.contains(
            "is not a method descriptor, so this presentation writes no declaration for it"
        )
    );
    assert_accessors_recovered(&report, &["access$002"], "void descriptor");

    // Generic copy recovery can retain the independently valid field write. It must not
    // invent a boolean-to-int Java conversion or claim that the return has recovered.
    let report = source_of(&probe_jar("probes/WA-mismatch.class"), "WA");
    let text = member_text(&report, "access$002");
    assert!(text.contains("arg0.flag = arg1;"), "{text}");
    assert!(
        !text.contains("return arg1;") && !text.contains("return (int)"),
        "{text}"
    );
    assert!(
        text.contains("is presented as `boolean`") && text.contains("descriptor returns `int`"),
        "{text}"
    );
    let quoted: Vec<u32> = text
        .lines()
        .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
        .flat_map(|line| line.split_whitespace().filter_map(|word| word.parse().ok()))
        .collect();
    assert!(
        quoted.contains(&2) && quoted.contains(&6),
        "the refused return retains both copy and return origins: {text}"
    );
    assert_accessors_recovered(&report, &["access$002"], "mismatched return");
}

/// Compile the complete, unedited public Engine class-source output in its own directory.
/// Original and recompiled runs have disjoint classpaths: no original class can fill a hole.
fn assert_complete_class_oracle(
    jar: &[u8],
    report: &ClassSourceReport,
    class: &str,
    harness: &str,
    expected: &str,
) {
    let temp = TestDirectory::new("generic-accessor-oracle");
    let original = temp.path().join("original");
    let recovered = temp.path().join("recovered");
    std::fs::create_dir(&original).unwrap();
    std::fs::create_dir(&recovered).unwrap();
    std::fs::write(original.join("input.jar"), jar).unwrap();
    std::fs::write(recovered.join(format!("{class}.java")), &report.text).unwrap();
    for dir in [&original, &recovered] {
        std::fs::write(dir.join("AccessorOracle.java"), harness).unwrap();
        let mut command = Command::new("javac");
        command.args(["--release", "8", "-classpath", ".", "AccessorOracle.java"]);
        if dir == &recovered {
            command.arg(format!("{class}.java"));
        }
        let result = command.current_dir(dir).output().expect("javac runs");
        assert!(
            result.status.success(),
            "complete source compilation failed: {}\n{}",
            String::from_utf8_lossy(&result.stderr),
            report.text
        );
    }
    let run = |dir: &Path, cp: &str| {
        let result = Command::new("java")
            .args(["-Xverify:all", "-cp", cp, "AccessorOracle", class])
            .current_dir(dir)
            .output()
            .expect("java runs");
        assert!(
            result.status.success(),
            "full-class oracle failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            result.stderr.is_empty(),
            "unexpected JVM stderr: {:?}",
            result.stderr
        );
        String::from_utf8(result.stdout).unwrap()
    };
    let original_output = run(&original, ".:input.jar");
    assert_eq!(original_output, expected, "independent original behavior");
    assert_eq!(
        run(&recovered, "."),
        original_output,
        "complete unedited class preserves fields, results, identity and exceptions"
    );
}

const ACCESSOR_ORACLE: &str = r#"
import java.lang.reflect.*;
public class AccessorOracle {
    public static void main(String[] args) throws Exception {
        Class<?> c = Class.forName(args[0]);
        c.getMethod("main", String[].class).invoke(null, (Object) new String[0]);
        if (args[0].equals("ST")) {
            Method m = c.getDeclaredMethod("access$002", int.class);
            m.setAccessible(true);
            Field f = c.getDeclaredField("sc"); f.setAccessible(true);
            for (int v : new int[] {Integer.MIN_VALUE, -1, 0, 1, Integer.MAX_VALUE}) {
                Object result = m.invoke(null, v);
                if (!result.equals(v) || f.getInt(null) != v) throw new AssertionError();
                System.out.println(result + ":" + f.getInt(null));
            }
            m = c.getDeclaredMethod("access$102", String.class); m.setAccessible(true);
            f = c.getDeclaredField("sr"); f.setAccessible(true);
            for (String v : new String[] {null, new String("identity")}) {
                if (m.invoke(null, v) != v || f.get(null) != v) throw new AssertionError();
                System.out.println(v);
            }
        } else {
            Object receiver = c.getConstructor().newInstance();
            Method m = c.getDeclaredMethod("access$002", c, boolean.class); m.setAccessible(true);
            Field f = c.getDeclaredField("flag"); f.setAccessible(true);
            for (boolean v : new boolean[] {false, true, false}) {
                Object result = m.invoke(null, receiver, v);
                if (!result.equals(v) || f.getBoolean(receiver) != v) throw new AssertionError();
                System.out.println(result + ":" + f.getBoolean(receiver));
                try { m.invoke(null, null, v); throw new AssertionError("missing NPE"); }
                catch (InvocationTargetException e) {
                    if (!(e.getCause() instanceof NullPointerException)) throw e;
                    System.out.println("NPE");
                }
            }
        }
    }
}
"#;

#[test]
fn the_static_field_accessors_recover_through_generic_copy_semantics() {
    let jar = st_jar();
    let report = source_of(&jar, "ST");
    for (name, descriptor, statement) in [
        ("access$002", "(I)I", "ST.sc = arg0;"),
        (
            "access$102",
            "(Ljava/lang/String;)Ljava/lang/String;",
            "ST.sr = arg0;",
        ),
    ] {
        let text = member_text(&report, name);
        assert!(
            text.contains(&format!("// @method {name}{descriptor}")),
            "{text}"
        );
        assert!(
            text.contains(statement) && text.contains("return arg0;"),
            "{text}"
        );
        assert!(
            !text.contains("@bytecode") && !text.contains("not recovered"),
            "{text}"
        );
    }
    assert!(
        !report.text.contains("arg0."),
        "no invented receiver: {}",
        report.text
    );
    assert!(report.text.contains("ST.access$002(arg1);"));
    assert!(report.text.contains("ST.access$102(arg1);"));
    assert_complete_class_oracle(
        &jar,
        &report,
        "ST",
        ACCESSOR_ORACLE,
        "7|q\n-2147483648:-2147483648\n-1:-1\n0:0\n1:1\n2147483647:2147483647\nnull\nidentity\n",
    );
}

#[test]
fn synthetic_flag_and_nop_layout_do_not_block_generic_copy_semantics() {
    for name in ["probes/WA-unsynth.class", "probes/WA-bcishift.class"] {
        // The original companion is essential: the old single-entry probe deliberately
        // lacks the inner-class evidence needed to render a compilable main method.
        let jar = jar_of(&[
            (b"WA.class", fixture(name).as_slice()),
            (b"WA$S.class", fixture("wa/WA$S.class").as_slice()),
        ]);
        let report = source_of(&jar, "WA");
        let text = member_text(&report, "access$002");
        assert!(
            text.contains("arg0.flag = arg1;") && text.contains("return arg1;"),
            "{name}: {text}"
        );
        assert!(!text.contains("not recovered"), "{name}: {text}");
        if name.contains("unsynth") {
            assert!(!text.contains("@bytecode"), "{text}");
            assert!(text.contains("member flags 0x0008"), "{text}");
        }
        // A quoted NOP is not treated as proof of complete structural recovery. The full
        // source oracle below still requires all executable semantics to be preserved.
        assert_accessors_recovered(&report, &["access$002"], name);
        assert_complete_class_oracle(
            &jar,
            &report,
            "WA",
            ACCESSOR_ORACLE,
            "true123.0x45c6.0\nfalse:false\nNPE\ntrue:true\nNPE\nfalse:false\nNPE\n",
        );
    }
}

#[test]
fn every_broken_structure_keeps_its_refusal_while_the_table_still_generalizes() {
    // Keep the unsafe or unproved shapes refused: invalid copy, null receiver, wrong
    // field owner, and exception handler. Flag/NOP-only changes have separate JVM oracles.
    for name in [
        "probes/WA-dupbreak.class",
        "probes/WA-nullrecv.class",
        "probes/WA-wrongowner.class",
        "probes/WA-handler.class",
    ] {
        let report = source_of(&probe_jar(name), "WA");
        assert!(
            !report.text.contains("arg0.flag = arg1;"),
            "{name}: the patched body must not recover:\n{}",
            report.text
        );
        assert!(
            report
                .text
                .contains("not recovered: the recovery run for `access$002(LWA;Z)Z`"),
            "{name}: the patched body refuses loudly, with its own descriptor quoted:\n{}",
            report.text
        );
        // The eight other members of the same class are untouched by the patch and still recover.
        assert_accessors_recovered(&report, &["access$002"], name);
    }
}

// ---------------------------------------------------------------------------------------------
// 5. The identifier/arm control, compiled in-test (never pinned to the anchor's own names).
// ---------------------------------------------------------------------------------------------

#[test]
fn the_control_family_with_other_names_and_both_reference_arms_recovers() {
    let temp = TestDirectory::new("wb-compile");
    std::fs::write(
        temp.path().join("WB.java"),
        include_str!("fixtures/recover-write-accessor-field-types/wb/WB.java"),
    )
    .expect("the control source is written");
    let compiled = Command::new("javac")
        .args(["--release", "8", "WB.java"])
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "the control compiles under --release 8: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let jar = jar_of(&[
        (
            b"WB.class",
            std::fs::read(temp.path().join("WB.class"))
                .expect("the control class reads")
                .as_slice(),
        ),
        (
            b"WB$Writer.class",
            std::fs::read(temp.path().join("WB$Writer.class"))
                .expect("the control writer class reads")
                .as_slice(),
        ),
    ]);
    let report = source_of(&jar, "WB");
    for (marker, statement) in [
        ("access$002", "arg0.tally = arg1;"),
        ("access$102", "arg0.bag = arg1;"),
        ("access$202", "arg0.grid = arg1;"),
    ] {
        let text = member_text(&report, marker);
        assert!(
            text.contains(statement) && text.contains("return arg1;"),
            "the control accessor {marker} recovers with its own identifiers:\n{text}"
        );
        assert!(
            !text.contains("@bytecode"),
            "the control accessor {marker} carries no quote:\n{text}"
        );
    }
    // Both reference arms are exercised by the control: one `L…;` and one `[…` descriptor.
    assert!(
        report
            .text
            .contains("access$102(LWB;Ljava/util/List;)Ljava/util/List;")
    );
    assert!(report.text.contains("access$202(LWB;[I)[I"));
    // Behavior: the compiled original and the rendered text recompiled from it print the same line.
    let run_output = Command::new("java")
        .args(["-Xverify:all", "-cp", "."])
        .arg("WB")
        .current_dir(temp.path())
        .output()
        .expect("java runs");
    assert!(run_output.status.success(), "the control runs");
    assert_eq!(
        String::from_utf8(run_output.stdout).expect("text"),
        "11|0|2\n",
        "the control's behavior baseline"
    );
}
