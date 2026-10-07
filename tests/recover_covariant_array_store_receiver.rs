//! `recover-covariant-array-store-receiver`: the write side's receiver **widening**.
//!
//! The array-covariant-store patrol
//! (`openspec/evidence/java-syntax-2026-10-05/array-covariant-store-patrol/`) recorded the shape
//! this change closes:
//!
//! ```text
//! Object[] a = new String[2];
//! a[0] = Integer.valueOf(1);      // compiles against `Object[]`, throws `ArrayStoreException`
//! ```
//!
//! No `LocalVariableTable` states the source's own `Object[]`: the local's frame type is the array
//! the `anewarray` built, so the recovered text declares `java.lang.String[] local0 = new
//! java.lang.String[2];` and writes `local0[0] = java.lang.Integer.valueOf(1);` — text `javac`
//! refuses (`incompatible types: Integer cannot be converted to String`), while the bytecode's
//! `aastore` is legal and its runtime check is exact. The same holds for the `Number[] n = new
//! Integer[2]; n[0] = Double.valueOf(2.5);` form. jadx fails on the same shape.
//!
//! What this change adds is the **access's own** presentation: when a store's presented component
//! type is a reference the stored value's type is not proven to meet, the receiver is written
//! under `((java.lang.Object[]) receiver)[index] = value`. The rule is the closed one the array
//! initializer's element rule already states (the exact component type, an `Object` component, or a
//! `null` value) — which class is assignable to which is a subtype judgment this layer deliberately
//! does not make — so the widening changes nothing about evaluation (the receiver is evaluated
//! once), nothing about the runtime check (every reference array is a subtype of `Object[]`, so the
//! cast cannot fail, and the `aastore` still checks the value against the array's runtime
//! component), and nothing about the declaration (the local keeps the array its own facts state).
//!
//! The fixtures are the patrol's own `AS` plus three shapes this change adds, on **both** compiler
//! legs (javac 23.0.1 `--release 8` and real javac 8, the same sources both times):
//!
//! * `AS` — the patrol's `storeWrong`/`storeNumber` render the widened receiver and `storeRight`
//!   (same-type store) is byte-identical to the pre-change text;
//! * `SD` — the behavior driver: a same-type store read back, the two proven-compatible controls
//!   (`Object` component, `null` value), the three covariant stores caught as `ArrayStoreException`
//!   with their own type and throwing method, the primitive-array negative, and a same-type element
//!   receiver;
//! * `UB` — a receiver whose component no fact states (a merged local) keeps its presentation
//!   verbatim: nothing is guessed, so the widening does not apply; a `checkcast` receiver whose
//!   component the pool states is likewise untouched;
//! * `SC` — a store whose value type is a **subtype** of the component (`String` into
//!   `CharSequence[]`): no fact in this layer proves that compatibility, so this store widens too.
//!   The text still compiles and still answers what the class answers; the admission's boundary is
//!   pinned here rather than left implicit.
//!
//! The ignored replay (`cargo test -- --ignored`) strips the presentation the way the patrol's own
//! `verify-uncompilable-AS.java` was made — comment lines dropped — compiles the stripped text with
//! **both** compilers (`javac --release 8` and Corretto 1.8.0_432's own `javac`, whose path the
//! test asserts before it is used), runs every text under `-Xverify:all` beside the fixture's own
//! class files, and compares the two runs: `AS` answers `s`/`ASE1`/`ASE2` on both sides, `SD`'s
//! driver answers each store's exception type and throwing method identically, and `UB`'s stripped
//! text must stay uncompilable (the merged local's own debt, which this change does not touch).

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const STORE: u16 = 0;

/// The real javac 8 of this repository's fixture protocol (Corretto 1.8.0_432): the second leg's
/// compiler and JVM, named by path exactly as the fixture README states it.
const CORRETTO_HOME: &str = "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home";

// -------------------------------------------------------------------------------------------
// The fixtures: one source, two compiler legs.
// -------------------------------------------------------------------------------------------

/// One compiler leg: the label a failure names, whether it is the real JDK 8, and its class files.
struct Leg {
    label: &'static str,
    real_javac8: bool,
    files: &'static [(&'static str, &'static [u8])],
}

impl Leg {
    /// One fixture's container: its own class file (one class per fixture here).
    fn fixture(&self, class: &str) -> Vec<u8> {
        let own = format!("{class}.class");
        let (name, bytes) = self
            .files
            .iter()
            .find(|(name, _)| *name == own)
            .unwrap_or_else(|| panic!("the fixture `{class}` is committed"));
        zip_of(&[(name.as_bytes(), bytes)])
    }

    /// Compile one source with this leg's own compiler and flags: javac 23 targets Java 8 with
    /// `--release 8`, the real javac 8 needs no flag (its default target is 8) and is named by
    /// path, which the test asserts before it is used.
    fn javac(&self) -> Command {
        let mut command = if self.real_javac8 {
            assert!(
                Path::new(CORRETTO_HOME).join("bin/javac").exists(),
                "the real javac 8 of the fixture protocol is installed at {CORRETTO_HOME}"
            );
            Command::new(format!("{CORRETTO_HOME}/bin/javac"))
        } else {
            let mut command = Command::new("javac");
            command.args(["--release", "8", "-Xlint:-options"]);
            command
        };
        command.args(["-J-Duser.language=en", "-J-Duser.country=US"]);
        command
    }

    fn compile(&self, source: &Path, output: &Path) {
        let result = self
            .javac()
            .args(["-d"])
            .arg(output)
            .arg(source)
            .output()
            .expect("the leg's compiler is installed");
        assert!(
            result.status.success(),
            "{}: javac rejected the presented text:\n{}\n{}",
            self.label,
            String::from_utf8_lossy(&result.stderr),
            fs::read_to_string(source).expect("the text reads")
        );
    }

    /// The JVM of this leg: the installed one, or the real JDK 8 beside its compiler.
    fn java(&self) -> Command {
        if self.real_javac8 {
            Command::new(format!("{CORRETTO_HOME}/bin/java"))
        } else {
            Command::new("java")
        }
    }

    /// Run one class under `-Xverify:all` with this leg's own JVM and answer what it printed.
    fn run(&self, directory: &Path, class: &str) -> String {
        let result = self
            .java()
            .args(["-Xverify:all", "-cp"])
            .arg(directory)
            .arg(class)
            .output()
            .expect("the leg's JVM is installed");
        assert!(
            result.status.success(),
            "{}: `{class}` failed under -Xverify:all:\n{}",
            self.label,
            String::from_utf8_lossy(&result.stderr)
        );
        String::from_utf8_lossy(&result.stdout)
            .trim_end()
            .to_owned()
    }
}

/// javac 23.0.1, `javac --release 8 -Xlint:-options -d v8 *.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "AS.class",
        include_bytes!("fixtures/recover-covariant-array-store-receiver/v8/AS.class"),
    ),
    (
        "SD.class",
        include_bytes!("fixtures/recover-covariant-array-store-receiver/v8/SD.class"),
    ),
    (
        "UB.class",
        include_bytes!("fixtures/recover-covariant-array-store-receiver/v8/UB.class"),
    ),
    (
        "SC.class",
        include_bytes!("fixtures/recover-covariant-array-store-receiver/v8/SC.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -d v8-javac8 *.java`: no `--release`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "AS.class",
        include_bytes!("fixtures/recover-covariant-array-store-receiver/v8-javac8/AS.class"),
    ),
    (
        "SD.class",
        include_bytes!("fixtures/recover-covariant-array-store-receiver/v8-javac8/SD.class"),
    ),
    (
        "UB.class",
        include_bytes!("fixtures/recover-covariant-array-store-receiver/v8-javac8/UB.class"),
    ),
    (
        "SC.class",
        include_bytes!("fixtures/recover-covariant-array-store-receiver/v8-javac8/SC.class"),
    ),
];

const LEGS: &[Leg] = &[
    Leg {
        label: "javac 23.0.1 --release 8",
        real_javac8: false,
        files: V8_FILES,
    },
    Leg {
        label: "Corretto 1.8.0_432 (real javac 8)",
        real_javac8: true,
        files: V8_JAVAC8_FILES,
    },
];

// -------------------------------------------------------------------------------------------
// The presented texts, pinned whole (the p3 surface tests' own convention).
// -------------------------------------------------------------------------------------------

const AS_V8: &str = "// jarde: presentation of `AS` from the class file's own declaration and one recovery run per member.\n// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.\npublic class AS extends java.lang.Object {\n    public AS() {\n        // @method <init>()V\n        // @declaration a constructor of `AS`, member flags 0x0001\n        // recovered from bytecode; presentation is not claimed to compile\n        super();\n        return;\n    }\n\n    static java.lang.String storeWrong() {\n        // @method storeWrong()Ljava/lang/String;\n        // @declaration a static method of `AS`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.String[] local0 = new java.lang.String[2];\n        ((java.lang.Object[]) local0)[0] = java.lang.Integer.valueOf(1);\n        return \"unreachable\";\n    }\n\n    static java.lang.String storeRight() {\n        // @method storeRight()Ljava/lang/String;\n        // @declaration a static method of `AS`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.String[] local0 = new java.lang.String[2];\n        local0[0] = \"s\";\n        return (java.lang.String) local0[0];\n    }\n\n    static java.lang.String storeNumber() {\n        // @method storeNumber()Ljava/lang/String;\n        // @declaration a static method of `AS`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.Integer[] local0 = new java.lang.Integer[2];\n        ((java.lang.Object[]) local0)[0] = java.lang.Double.valueOf(0x1.4000000000000p1d);\n        return \"unreachable2\";\n    }\n\n    public static void main(java.lang.String[] arg0) {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `AS`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println((java.lang.String) storeRight());\n        try {\n            java.lang.System.out.println((java.lang.String) storeWrong());\n        } catch (java.lang.ArrayStoreException local1) {\n            java.lang.System.out.println(\"ASE1\");\n        }\n        try {\n            java.lang.System.out.println((java.lang.String) storeNumber());\n        } catch (java.lang.ArrayStoreException local1) {\n            java.lang.System.out.println(\"ASE2\");\n        }\n        return;\n    }\n}\n";

const SD_V8: &str = "// jarde: presentation of `SD` from the class file's own declaration and one recovery run per member.\n// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.\npublic class SD extends java.lang.Object {\n    public SD() {\n        // @method <init>()V\n        // @declaration a constructor of `SD`, member flags 0x0001\n        // recovered from bytecode; presentation is not claimed to compile\n        super();\n        return;\n    }\n\n    static java.lang.String readBack() {\n        // @method readBack()Ljava/lang/String;\n        // @declaration a static method of `SD`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.String[] local0 = new java.lang.String[2];\n        local0[0] = \"s\";\n        return (java.lang.String) local0[0];\n    }\n\n    static java.lang.String objectComponent() {\n        // @method objectComponent()Ljava/lang/String;\n        // @declaration a static method of `SD`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.Object[] local0 = new java.lang.Object[2];\n        local0[0] = \"o\";\n        return (java.lang.String) local0[0];\n    }\n\n    static java.lang.String nullStore() {\n        // @method nullStore()Ljava/lang/String;\n        // @declaration a static method of `SD`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.String[] local0 = new java.lang.String[2];\n        local0[0] = null;\n        return \"n\";\n    }\n\n    static java.lang.String catchWrong() {\n        // @method catchWrong()Ljava/lang/String;\n        // @declaration a static method of `SD`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.String[] local0 = new java.lang.String[2];\n        try {\n            ((java.lang.Object[]) local0)[0] = java.lang.Integer.valueOf(1);\n            return \"no-throw\";\n        } catch (java.lang.ArrayStoreException local1) {\n            return new java.lang.StringBuilder().append(\"wrong@\").append((java.lang.String) local1.getClass().getName()).append(\"/\").append((java.lang.String) local1.getStackTrace()[0].getMethodName()).toString();\n        }\n    }\n\n    static java.lang.String catchNumber() {\n        // @method catchNumber()Ljava/lang/String;\n        // @declaration a static method of `SD`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.Integer[] local0 = new java.lang.Integer[2];\n        try {\n            ((java.lang.Object[]) local0)[0] = java.lang.Double.valueOf(0x1.4000000000000p1d);\n            return \"no-throw2\";\n        } catch (java.lang.ArrayStoreException local1) {\n            return new java.lang.StringBuilder().append(\"number@\").append((java.lang.String) local1.getClass().getName()).append(\"/\").append((java.lang.String) local1.getStackTrace()[0].getMethodName()).toString();\n        }\n    }\n\n    static java.lang.String catchElement() {\n        // @method catchElement()Ljava/lang/String;\n        // @declaration a static method of `SD`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.String[][] local0 = new java.lang.String[1][1];\n        try {\n            ((java.lang.Object[]) local0[0])[0] = java.lang.Integer.valueOf(1);\n            return \"no-throw3\";\n        } catch (java.lang.ArrayStoreException local1) {\n            return new java.lang.StringBuilder().append(\"element@\").append((java.lang.String) local1.getClass().getName()).append(\"/\").append((java.lang.String) local1.getStackTrace()[0].getMethodName()).toString();\n        }\n    }\n\n    static java.lang.String primitiveArray() {\n        // @method primitiveArray()Ljava/lang/String;\n        // @declaration a static method of `SD`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        int[] local0 = new int[2];\n        local0[0] = 7;\n        return new java.lang.StringBuilder().append(\"p\").append(local0[0]).toString();\n    }\n\n    static java.lang.String elementReceiver() {\n        // @method elementReceiver()Ljava/lang/String;\n        // @declaration a static method of `SD`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.String[][] local0 = new java.lang.String[1][1];\n        local0[0][0] = \"r\";\n        return local0[0][0];\n    }\n\n    public static void main(java.lang.String[] arg0) {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `SD`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println((java.lang.String) readBack());\n        java.lang.System.out.println((java.lang.String) objectComponent());\n        java.lang.System.out.println((java.lang.String) nullStore());\n        java.lang.System.out.println((java.lang.String) catchWrong());\n        java.lang.System.out.println((java.lang.String) catchNumber());\n        java.lang.System.out.println((java.lang.String) catchElement());\n        java.lang.System.out.println((java.lang.String) primitiveArray());\n        java.lang.System.out.println((java.lang.String) elementReceiver());\n        return;\n    }\n}\n";

const UB_V8: &str = "// jarde: presentation of `UB` from the class file's own declaration and one recovery run per member.\n// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.\npublic class UB extends java.lang.Object {\n    public UB() {\n        // @method <init>()V\n        // @declaration a constructor of `UB`, member flags 0x0001\n        // recovered from bytecode; presentation is not claimed to compile\n        super();\n        return;\n    }\n\n    static java.lang.String merged(java.lang.String[] arg0, java.lang.Integer[] arg1, boolean arg2) {\n        // @method merged([Ljava/lang/String;[Ljava/lang/Integer;Z)Ljava/lang/String;\n        // @declaration a static method of `UB`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.String[] local3;\n        if (arg2) {\n            local3 = arg0;\n        } else {\n            local3 = arg1;\n        }\n        local3[0] = java.lang.Integer.valueOf(1);\n        return \"u\";\n    }\n\n    static java.lang.String throughObject(java.lang.Object arg0) {\n        // @method throughObject(Ljava/lang/Object;)Ljava/lang/String;\n        // @declaration a static method of `UB`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        ((java.lang.String[]) arg0)[0] = \"s\";\n        return \"v\";\n    }\n\n    public static void main(java.lang.String[] arg0) {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `UB`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println((java.lang.String) merged(new java.lang.String[1], new java.lang.Integer[1], true));\n        java.lang.System.out.println((java.lang.String) throughObject((java.lang.Object) new java.lang.String[1]));\n        return;\n    }\n}\n";

const UB_JAVAC8_V8: &str = "// jarde: presentation of `UB` from the class file's own declaration and one recovery run per member.\n// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.\npublic class UB extends java.lang.Object {\n    public UB() {\n        // @method <init>()V\n        // @declaration a constructor of `UB`, member flags 0x0001\n        // recovered from bytecode; presentation is not claimed to compile\n        super();\n        return;\n    }\n\n    static java.lang.String merged(java.lang.String[] arg0, java.lang.Integer[] arg1, boolean arg2) {\n        // @method merged([Ljava/lang/String;[Ljava/lang/Integer;Z)Ljava/lang/String;\n        // @declaration a static method of `UB`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.String[] local3;\n        if (arg2) {\n            local3 = arg0;\n        } else {\n            local3 = arg1;\n        }\n        local3[0] = java.lang.Integer.valueOf(1);\n        return \"u\";\n    }\n\n    static java.lang.String throughObject(java.lang.Object arg0) {\n        // @method throughObject(Ljava/lang/Object;)Ljava/lang/String;\n        // @declaration a static method of `UB`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        ((java.lang.String[]) (java.lang.String[]) arg0)[0] = \"s\";\n        return \"v\";\n    }\n\n    public static void main(java.lang.String[] arg0) {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `UB`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println((java.lang.String) merged(new java.lang.String[1], new java.lang.Integer[1], true));\n        java.lang.System.out.println((java.lang.String) throughObject((java.lang.Object) new java.lang.String[1]));\n        return;\n    }\n}\n";

const SC_V8: &str = "// jarde: presentation of `SC` from the class file's own declaration and one recovery run per member.\n// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.\npublic class SC extends java.lang.Object {\n    public SC() {\n        // @method <init>()V\n        // @declaration a constructor of `SC`, member flags 0x0001\n        // recovered from bytecode; presentation is not claimed to compile\n        super();\n        return;\n    }\n\n    static java.lang.String subtypeStore() {\n        // @method subtypeStore()Ljava/lang/String;\n        // @declaration a static method of `SC`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.CharSequence[] local0 = new java.lang.CharSequence[2];\n        ((java.lang.Object[]) local0)[0] = \"s\";\n        return local0[0].toString();\n    }\n\n    public static void main(java.lang.String[] arg0) {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `SC`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println((java.lang.String) subtypeStore());\n        return;\n    }\n}\n";

// -------------------------------------------------------------------------------------------
// The request path (the surface tests' shape): one snapshot, one class, the whole scope.
// -------------------------------------------------------------------------------------------

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a ZIP")
}

/// One class-source presentation over the snapshot's own root container: the policy `--policy
/// plain-jar` declares.
fn class_source_of(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
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
    match Engine::new()
        .class_source(slice::from_ref(snapshot), &request, &mut budget())
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one committed sample answers one definition, got {} candidate(s)",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one committed sample answers one definition, got an unfinished selection with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

/// One member's recovery report, for the planes the text alone cannot state.
fn recovered<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"));
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` is a recovered member: {other:?}"),
    }
}

/// The envelope's own claim, asserted before any text is compared: the presentation is this
/// layer's class view and says so, and every recovered body carries its own marker.
fn assert_the_self_header(report: &ClassSourceReport, class: &str) {
    assert!(
        report.text.starts_with(&format!(
            "// jarde: presentation of `{class}` from the class file's own"
        )),
        "the class view's own header leads:\n{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("presentation is not claimed to compile"),
        "the recovered body's own claim stays:\n{}",
        report.text
    );
}

/// The one ZIP the class-source request reads, built from the committed class files.
fn zip_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .expect("the fixture entry starts");
            let mut writer = config.wrap(&mut entry);
            writer
                .write_all(data)
                .expect("the fixture entry is written");
            let (_, descriptor) = writer.finish().expect("the fixture entry closes");
            entry
                .finish(descriptor)
                .expect("the fixture entry finishes");
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

/// One fixture's presentation, on one leg.
fn presentation(leg: &Leg, class: &str) -> ClassSourceReport {
    let snapshot = open(&leg.fixture(class));
    class_source_of(&snapshot, class)
}

// -------------------------------------------------------------------------------------------
// The anchors.
// -------------------------------------------------------------------------------------------

#[test]
fn the_patrols_two_covariant_stores_widen_their_access_receiver() {
    for leg in LEGS {
        let report = presentation(leg, "AS");
        assert_the_self_header(&report, "AS");
        assert_eq!(
            report.text, AS_V8,
            "{}: the patrol's class renders the widened receivers",
            leg.label
        );
        assert!(
            !report.text.contains("@bytecode"),
            "{}: no instruction of `AS` stays quoted:\n{}",
            leg.label,
            report.text
        );
        // The two covariant stores: the receiver widens to `Object[]`, the value keeps its own
        // spelling, and the statement is still anchored where the `aastore` runs (BCI 11).
        let anchors: std::collections::BTreeSet<u32> = recovered(&report, "storeWrong")
            .source_map
            .segments()
            .iter()
            .flat_map(|segment| segment.origin().bcis())
            .collect();
        for bci in [5, 6, 8, 11] {
            assert!(
                anchors.contains(&bci),
                "{}: BCI {bci} is an anchor of the widened store: {anchors:?}",
                leg.label
            );
        }
        // The same-type control is the pre-change text: nothing widened, nothing moved.
        assert!(
            report.text.contains(
                "        local0[0] = \"s\";\n        return (java.lang.String) local0[0];\n"
            ),
            "{}: the same-type store stays the text it always was:\n{}",
            leg.label,
            report.text
        );
    }
}

#[test]
fn the_store_driver_keeps_its_controls_and_its_negatives() {
    for leg in LEGS {
        let report = presentation(leg, "SD");
        assert_the_self_header(&report, "SD");
        assert_eq!(
            report.text, SD_V8,
            "{}: the driver's class renders the widened stores beside its controls",
            leg.label
        );
        assert!(
            !report.text.contains("@bytecode"),
            "{}: no instruction of `SD` stays quoted:\n{}",
            leg.label,
            report.text
        );
        // The proven-compatible controls: the exact component type, an `Object` component and a
        // `null` value are written exactly as before.
        for control in [
            "        local0[0] = \"s\";\n",
            "        local0[0] = \"o\";\n",
            "        local0[0] = null;\n",
            "        local0[0] = 7;\n",
            "        local0[0][0] = \"r\";\n",
        ] {
            assert!(
                report.text.contains(control),
                "{}: the control `{}` keeps its presentation:\n{}",
                leg.label,
                control.trim(),
                report.text
            );
        }
    }
}

#[test]
fn an_unproven_component_keeps_its_presentation() {
    for leg in LEGS {
        let report = presentation(leg, "UB");
        assert_the_self_header(&report, "UB");
        let expected = if leg.real_javac8 { UB_JAVAC8_V8 } else { UB_V8 };
        assert_eq!(
            report.text, expected,
            "{}: a receiver whose component no fact states keeps its text",
            leg.label
        );
        // The merged local's store states no component, so nothing is widened and the member keeps
        // the debt it had: the widening reads facts and does not guess one.
        assert!(
            report
                .text
                .contains("        local3[0] = java.lang.Integer.valueOf(1);\n"),
            "{}: the unproven receiver is not widened:\n{}",
            leg.label,
            report.text
        );
        // A `checkcast` receiver whose component the pool states is untouched: its store is the
        // same-type case and was never this change's surface.
        assert!(
            report.text.contains(")[0] = \"s\";\n"),
            "{}: a stated component store keeps its text:\n{}",
            leg.label,
            report.text
        );
    }
}

#[test]
fn a_subtype_value_is_not_proven_compatible_and_widens_too() {
    // The admission's boundary, pinned: `String` is assignable to `CharSequence`, and that
    // assignability is a subtype judgment this layer deliberately does not make — so a store into
    // a `CharSequence[]` widens exactly like the patrol's covariant shape. The presentation still
    // compiles and the class still answers what it answered (the ignored replay states both).
    for leg in LEGS {
        let report = presentation(leg, "SC");
        assert_the_self_header(&report, "SC");
        assert_eq!(
            report.text, SC_V8,
            "{}: an unproven-compatible store widens the same way",
            leg.label
        );
        assert!(
            report
                .text
                .contains("        ((java.lang.Object[]) local0)[0] = \"s\";\n"),
            "{}: the widened form is the admission's own:\n{}",
            leg.label,
            report.text
        );
    }
}

// -------------------------------------------------------------------------------------------
// The replay: the presented text, stripped, compiled and run (needs both JDKs).
// -------------------------------------------------------------------------------------------

static UNIQUE: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos();
        let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-covariant-store-{label}-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the replay directory");
        Self(path)
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

/// One presented class text with every `//` comment line dropped — the strip the patrol's own
/// `verify-uncompilable-AS.java` was made by.
fn stripped(text: &str) -> String {
    let mut body: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect();
    body.push("");
    body.join("\n")
}

/// The stripped presentation of one fixture: its own text, comments dropped.
fn replay_source(leg: &Leg, class: &str) -> String {
    stripped(&presentation(leg, class).text)
}

/// The stripped text compiled and run, and the fixture's own class run — the two answers a leg's
/// replay compares.
fn replay(leg: &Leg, class: &str) -> (String, String) {
    let temp = TempDir::new(class);
    let presented = temp.path().join("presented");
    let original = temp.path().join("original");
    fs::create_dir_all(&presented).expect("create the presented directory");
    fs::create_dir_all(&original).expect("create the original directory");
    let source = temp.path().join(format!("{class}.java"));
    fs::write(&source, replay_source(leg, class)).expect("write the presented text");
    leg.compile(&source, &presented);
    let (name, bytes) = leg
        .files
        .iter()
        .find(|(name, _)| *name == format!("{class}.class"))
        .expect("the fixture's own class file is committed");
    fs::write(original.join(name), bytes).expect("the fixture class is written");
    (leg.run(&presented, class), leg.run(&original, class))
}

#[test]
#[ignore = "needs both JDKs: it strips the class-source comment lines, compiles the text with \
            `javac --release 8` **and** the real javac 8 and runs both under `-Xverify:all` (see \
            the module doc)"]
fn the_patrols_class_answers_what_its_class_answers() {
    for leg in LEGS {
        let (presented, original) = replay(leg, "AS");
        assert_eq!(
            original, "s\nASE1\nASE2",
            "{}: the fixture's own classes answer the patrol's readings",
            leg.label
        );
        assert_eq!(
            presented, original,
            "{}: the stripped text answers exactly what the class answers, ASE1/ASE2 included",
            leg.label
        );
    }
}

#[test]
#[ignore = "needs both JDKs: it strips the class-source comment lines, compiles the text with \
            `javac --release 8` **and** the real javac 8 and runs both under `-Xverify:all` (see \
            the module doc)"]
fn the_store_driver_answers_what_its_class_answers() {
    // Each covariant store prints its exception's own type and the method that threw it, so an
    // equal answer is an equal **type** at an equal **point** — not merely "something threw".
    let expected = "s\no\nn\n\
                    wrong@java.lang.ArrayStoreException/catchWrong\n\
                    number@java.lang.ArrayStoreException/catchNumber\n\
                    element@java.lang.ArrayStoreException/catchElement\n\
                    p7\nr";
    for leg in LEGS {
        let (presented, original) = replay(leg, "SD");
        assert_eq!(
            original, expected,
            "{}: the driver's own classes throw where they always did",
            leg.label
        );
        assert_eq!(
            presented, original,
            "{}: the stripped text throws the same type at the same point",
            leg.label
        );
    }
}

#[test]
#[ignore = "needs both JDKs: it strips the class-source comment lines, compiles the text with \
            `javac --release 8` **and** the real javac 8 and runs both under `-Xverify:all` (see \
            the module doc)"]
fn the_subtype_store_answers_what_its_class_answers() {
    for leg in LEGS {
        let (presented, original) = replay(leg, "SC");
        assert_eq!(original, "s", "{}: the fixture's own class", leg.label);
        assert_eq!(
            presented, original,
            "{}: the widened (unproven-compatible) store still compiles and still answers",
            leg.label
        );
    }
}

#[test]
#[ignore = "needs both JDKs: the merged local's own rendering debt keeps the text uncompilable \
            (see the module doc)"]
fn an_unproven_component_text_stays_uncompilable() {
    for leg in LEGS {
        let temp = TempDir::new("UB-debt");
        let source = temp.path().join("UB.java");
        fs::write(&source, replay_source(leg, "UB")).expect("write the presented text");
        let result = leg
            .javac()
            .args(["-d"])
            .arg(temp.path())
            .arg(&source)
            .output()
            .expect("the leg's compiler is installed");
        // The reviewed state of the debt: a receiver whose component no fact states is presented
        // as it was, and the merged local's declaration is what the text gets wrong. A run that
        // compiles it is a debt this test's assertion wants to hear about.
        assert!(
            !result.status.success(),
            "{}: the unproven receiver's text must stay uncompilable while nothing states its \
             component:\n{}",
            leg.label,
            fs::read_to_string(&source).expect("the text reads")
        );
    }
}
