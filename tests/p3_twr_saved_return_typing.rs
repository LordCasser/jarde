//! Current saved-return contract: producer types remain truthful; null has no nominal
//! producer and may use its proved return consumer's reference type. The historical
//! Object-null contract is superseded by the CURRENT section of the main java8-recovery spec.
//! Whole, unedited public Engine class sources compile in isolated classpaths and
//! execute under full JVM verification. Frozen Wtyping covers all six families;
//! one source-built derivative adds independent failures, events and metamorphisms.

use jarde::*;
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;

const T2: &[u8] = include_bytes!(
    "../tests/fixtures/recovered-inputs/evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/fixture/T2.class"
);
const WTYPING: &[u8] = include_bytes!(
    "../tests/fixtures/recovered-inputs/evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/saved-return-typing/original/Wtyping.class"
);

/// The run of the original frozen `T2.main` — the text the recovered class prints too (the
/// compile-and-run leg records it beside the evidence directory's three-way column).
const T2_EXPECTED_RUN: &str = "done\nin\ndone\ndone\n";

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn class_text(bytes: &[u8], class: &str) -> String {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the frozen class opens");
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
    match Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut budget())
        .expect("the frozen class answers the class-source request")
    {
        OperationOutcome::Performed(report) => report.text,
        other => panic!("one frozen class produced a complete answer: {other:?}"),
    }
}

fn member_text(class: &str, bytes: &[u8], method: &str) -> String {
    let text = class_text(bytes, class);
    let start = text
        .find(&format!("{method}()"))
        .unwrap_or_else(|| panic!("{class}.{method} is presented"));
    text[start..]
        .split_once("\n    }")
        .map(|(body, _)| body.to_owned())
        .expect("the member's own text ends")
}

fn write_source(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, text).expect("the scratch source writes");
    path
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "jarde-saved-return-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&dir).expect("scratch directory creates");
        Self(dir)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_saved_return_declaration_spells_the_constants_own_type() {
    let text = member_text(
        "T2",
        T2,
        "public static java.lang.String voidBodyReturnInside",
    );
    assert!(
        text.contains(
            "try (T2 local0 = new T2()) {\n            touch(local0);\n            java.lang.String local1 = \"in\";\n            return local1;\n        }"
        ),
        "the saved return presents the constant's type, and the return compiles:\n{text}"
    );
    assert!(
        !text.contains("Object local1"),
        "no conservative spelling stays beside a named constant:\n{text}"
    );
}

#[test]
fn every_saved_value_family_respects_its_producer_or_proved_null_consumer() {
    let expectations = [
        (
            "public static java.lang.String stringLiteral",
            "try (Wtyping local0 = new Wtyping()) {\n            touch(local0);\n            java.lang.String local1 = \"in\";\n            return local1;\n        }",
        ),
        (
            "public static int intLiteral",
            "try (Wtyping local0 = new Wtyping()) {\n            touch(local0);\n            int local1 = 42;\n            return local1;\n        }",
        ),
        (
            "public static java.lang.StringBuilder constructorValue",
            "try (Wtyping local0 = new Wtyping()) {\n            touch(local0);\n            java.lang.StringBuilder local1 = new java.lang.StringBuilder(\"built\");\n            return local1;\n        }",
        ),
        (
            "public static java.lang.String callReturn",
            "try (Wtyping local0 = new Wtyping()) {\n            touch(local0);\n            java.lang.String local1 = java.lang.String.valueOf(7);\n            return local1;\n        }",
        ),
        (
            "public static java.lang.String nullValue",
            "try (Wtyping local0 = new Wtyping()) {\n            touch(local0);\n            java.lang.String local1 = null;\n            return local1;\n        }",
        ),
        (
            "public static java.lang.Class classLiteral",
            "try (Wtyping local0 = new Wtyping()) {\n            touch(local0);\n            java.lang.Class local1 = java.lang.String.class;\n            return local1;\n        }",
        ),
    ];
    for (method, expected) in expectations {
        let text = member_text("Wtyping", WTYPING, method);
        assert!(
            text.contains(expected),
            "{method} presents the saved value's own type:\n{text}"
        );
    }
}

#[test]
fn the_recovered_t2_compiles_as_presented_and_runs_the_original_paths() {
    let jarde_scratch = Scratch::new();
    let jarde_dir = &jarde_scratch.0;
    javac(&[write_source(jarde_dir, "T2.java", &class_text(T2, "T2"))]);

    let original_scratch = Scratch::new();
    let original_dir = &original_scratch.0;
    std::fs::write(original_dir.join("T2.class"), T2).expect("the frozen class writes");
    assert_eq!(
        java(original_dir, "T2", &[]),
        T2_EXPECTED_RUN,
        "the original class's own paths"
    );
    assert_eq!(
        java(jarde_dir, "T2", &[]),
        T2_EXPECTED_RUN,
        "the recovered class runs the same paths"
    );
}

fn javac(files: &[PathBuf]) {
    let dir = files[0].parent().expect("the scratch directory exists");
    let mut command =
        Command::new(std::env::var_os("JARDE_TEST_JAVAC").unwrap_or_else(|| "javac".into()));
    if std::env::var_os("JARDE_TEST_JAVAC8").is_some() {
        command.args(["-source", "8", "-target", "8"]);
    } else {
        command.args(["--release", "8"]);
    }
    let compile = command
        .args(["-g:none", "-Xlint:-options"])
        .arg("-cp")
        .arg(dir)
        .arg("-d")
        .arg(dir)
        .args(files)
        .output()
        .expect("the installed JDK provides javac");
    assert!(
        compile.status.success(),
        "javac accepted the recovered source:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
}

fn java(dir: &Path, class: &str, args: &[&str]) -> String {
    let run = Command::new(std::env::var_os("JARDE_TEST_JAVA").unwrap_or_else(|| "java".into()))
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg(class)
        .args(args)
        .output()
        .expect("the installed JDK provides java");
    assert!(
        run.status.success(),
        "`java -Xverify:all {class}` ran clean:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        run.stderr.is_empty(),
        "unexpected JVM stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the run's output is text")
}

const FAMILIES: &[(&str, &str, &str)] = &[
    ("stringLiteral", "String", "\"in\""),
    ("intLiteral", "int", "42"),
    (
        "constructorValue",
        "StringBuilder",
        "new StringBuilder(\"built\")",
    ),
    ("callReturn", "String", "String.valueOf(7)"),
    ("nullValue", "String", "null"),
    ("classLiteral", "Class", "String.class"),
];

// The oracle is compiled independently beside each input. The recovered directory
// never receives original target bytes, target source, or an original classpath.
const ORACLE: &str = r#"
public class SavedOracle {
    public static void main(String[] args) throws Exception {
        Class<?> type = Class.forName(args[0]);
        boolean frozen = args[1].equals("frozen");
        for (int path = 0; path < (frozen ? 2 : 6); path++) {
            if (frozen) {
                type.getField("boom").setBoolean(null, path != 0);
            } else {
                type.getField("bodyBoom").setBoolean(null, (path & 2) != 0);
                type.getField("closeBoom").setBoolean(null, (path & 1) != 0);
                type.getField("castWrong").setBoolean(null, path >= 4);
            }
            for (int i = 2; i < args.length; i++) {
                System.out.print(args[i] + "/" + path + ":");
                try {
                    Object value = type.getMethod(args[i]).invoke(null);
                    String text = value == null ? "null" :
                        value instanceof Class ? ((Class<?>) value).getName() + ".class" :
                        value.getClass() == Object.class ? "Object.class" : value.toString();
                    System.out.println("return=" + text);
                } catch (java.lang.reflect.InvocationTargetException wrapper) {
                    Throwable t = wrapper.getCause();
                    // CCE wording differs across JDKs; type, order and suppression stay exact.
                    String message = t instanceof ClassCastException ? "cast" : t.getMessage();
                    System.out.print(t.getClass().getName() + ":" + message);
                    System.out.print(";suppressed=" + t.getSuppressed().length);
                    for (Throwable suppressed : t.getSuppressed()) {
                        System.out.print(";" + suppressed.getClass().getName() + ":" + suppressed.getMessage());
                    }
                    System.out.println();
                }
            }
        }
    }
}
"#;

fn assert_oracle(
    original: &Path,
    recovered: &Path,
    class: &str,
    mode: &str,
    names: &[String],
    expected: &str,
) {
    for directory in [original, recovered] {
        javac(&[write_source(directory, "SavedOracle.java", ORACLE)]);
        let mut args = vec![class, mode];
        args.extend(names.iter().map(String::as_str));
        assert_eq!(
            java(directory, "SavedOracle", &args),
            expected,
            "{class} in {}",
            directory.display()
        );
    }
}

#[test]
fn whole_frozen_wtyping_compiles_unedited_and_runs_all_six_families() {
    let original = Scratch::new();
    let recovered = Scratch::new();
    std::fs::write(original.0.join("Wtyping.class"), WTYPING).unwrap();
    javac(&[write_source(
        &recovered.0,
        "Wtyping.java",
        &class_text(WTYPING, "Wtyping"),
    )]);
    let names: Vec<_> = FAMILIES
        .iter()
        .map(|(name, _, _)| (*name).to_owned())
        .collect();
    let results = ["in", "42", "built", "7", "null", "java.lang.String.class"];
    let mut expected = String::new();
    for (name, result) in names.iter().zip(results) {
        expected.push_str(&format!("{name}/0:return={result}\n"));
    }
    for name in &names {
        expected.push_str(&format!("{name}/1:java.lang.IllegalStateException:touch;suppressed=1;java.lang.IllegalStateException:close\n"));
    }
    assert_oracle(
        &original.0,
        &recovered.0,
        "Wtyping",
        "frozen",
        &names,
        &expected,
    );
}

fn null_position(bytes: &[u8], name: &str) -> u32 {
    let facts = inspect_method_bytecode(
        bytes,
        MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(b"()Ljava/lang/String;".to_vec()),
        },
        &mut budget(),
    )
    .expect("public reader inspects the saved-null method");
    assert!(facts.stopped_at.is_none());
    facts
        .instructions
        .iter()
        .find(|instruction| instruction.opcode == 0x01)
        .expect("aconst_null is physically present")
        .bci
}

#[test]
fn derivative_saved_returns_preserve_events_failures_and_structural_metamorphisms() {
    // Baseline, rename-only, value-only, offset-only, and combined. These are
    // same-template metamorphisms, not an independent held-out fixture set.
    let mut baseline_position = None;
    for (class, renamed, changed, shifted) in [
        ("SavedResource", false, false, false),
        ("RenamedResource", true, false, false),
        ("SavedResource", false, true, false),
        ("SavedResource", false, false, true),
        ("CombinedResource", true, true, true),
    ] {
        let original = Scratch::new();
        let recovered = Scratch::new();
        let mut source = format!(
            r#"
public class {class} implements AutoCloseable {{
    public static boolean bodyBoom;
    public static boolean closeBoom;
    public static boolean castWrong;
    public void close() {{
        System.out.print("close>");
        if (closeBoom) throw new IllegalStateException("close");
    }}
    public static void touch({class} resource) {{
        System.out.print("touch>");
        if (bodyBoom) throw new IllegalStateException("touch");
    }}
    public static void pad() {{ System.out.print("pad>"); }}
    public static Object objectText() {{
        System.out.print("cast>");
        if (castWrong) return new Object();
        return "cast";
    }}
"#
        );
        let mut families = FAMILIES.to_vec();
        families.extend([
            ("objectValue", "Object", "new Object()"),
            ("builderAsObject", "Object", "new StringBuilder(\"wide\")"),
            ("explicitCast", "String", "(String) objectText()"),
        ]);
        let names: Vec<_> = families
            .iter()
            .map(|(name, _, _)| {
                if renamed {
                    format!("renamed_{name}")
                } else {
                    (*name).to_owned()
                }
            })
            .collect();
        for ((_, ty, expression), name) in families.iter().zip(&names) {
            let expression = if changed {
                expression
                    .replace("\"in\"", "\"other\"")
                    .replace("42", "55")
                    .replace("built", "made")
                    .replace("valueOf(7)", "valueOf(9)")
            } else {
                (*expression).to_owned()
            };
            let pad = if shifted { "pad();" } else { "" };
            source.push_str(&format!("public static {ty} {name}() {{ try ({class} resource = new {class}()) {{ touch(resource); {pad} return {expression}; }} }}\n"));
        }
        source.push_str("}\n");
        javac(&[write_source(&original.0, &format!("{class}.java"), &source)]);
        let bytes = std::fs::read(original.0.join(format!("{class}.class"))).unwrap();
        let position = null_position(&bytes, &names[4]);
        let baseline = *baseline_position.get_or_insert(position);
        if shifted {
            assert!(
                position > baseline,
                "supported pad call must shift the saved computation"
            );
        } else {
            assert_eq!(position, baseline);
        }
        let text = class_text(&bytes, class);
        assert!(
            member_text(
                class,
                &bytes,
                &format!("public static java.lang.String {}", names[4])
            )
            .contains("java.lang.String local1 = null;"),
            "{text}"
        );
        javac(&[write_source(&recovered.0, &format!("{class}.java"), &text)]);
        let values = if changed {
            [
                "other",
                "55",
                "made",
                "9",
                "null",
                "java.lang.String.class",
                "Object.class",
                "wide",
                "cast",
            ]
        } else {
            [
                "in",
                "42",
                "built",
                "7",
                "null",
                "java.lang.String.class",
                "Object.class",
                "wide",
                "cast",
            ]
        };
        let mut expected = String::new();
        for path in 0..6 {
            for (name, value) in names.iter().zip(values) {
                let body_fails = path == 2 || path == 3;
                let pad = if shifted && !body_fails { "pad>" } else { "" };
                let cast = name.ends_with("explicitCast") && !body_fails;
                let cast_event = if cast { "cast>" } else { "" };
                expected.push_str(&format!("{name}/{path}:touch>{pad}{cast_event}close>"));
                if cast && path >= 4 {
                    expected.push_str(if path == 4 {
                        "java.lang.ClassCastException:cast;suppressed=0\n"
                    } else {
                        "java.lang.ClassCastException:cast;suppressed=1;java.lang.IllegalStateException:close\n"
                    });
                    continue;
                }
                match path {
                    0 | 4 => expected.push_str(&format!("return={value}\n")),
                    1 | 5 => expected.push_str("java.lang.IllegalStateException:close;suppressed=0\n"),
                    2 => expected.push_str("java.lang.IllegalStateException:touch;suppressed=0\n"),
                    3 => expected.push_str("java.lang.IllegalStateException:touch;suppressed=1;java.lang.IllegalStateException:close\n"),
                    _ => unreachable!(),
                }
            }
        }
        assert_oracle(
            &original.0,
            &recovered.0,
            class,
            "derivative",
            &names,
            &expected,
        );
    }
}
