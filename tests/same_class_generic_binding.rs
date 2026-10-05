//! Acceptance tests of change `recover-same-class-generic-bindings`: a same-class
//! `Methodref`/`Fieldref` in the selected class's own pool no longer refuses a generic
//! projection by existing. The class-level assembly builds a bounded use-site inventory over
//! its own decoded bodies, proves every actual site still binds the physical member, and only
//! then publishes — or keeps the existing refusal, whole, with the physical identity intact.
//!
//! The anchors:
//!
//! 1. every positive family recompiles `--release 8`, runs under `-Xverify:all`, prints what the
//!    original family prints, and answers the same generic reflection the original answers;
//! 2. an unconsumed pool entry (a verifier-valid mutation) blocks nothing, while a same-arity
//!    sibling overload, a receiver-consumed field read and an inventory that never closed keep
//!    the exact refusal family texts;
//! 3. the Z1 patrol fixture replays item by item: `add`/`index` project with the binding proved,
//!    and the members whose bodies this slice cannot re-type keep explicit refusals.

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
            "jarde-same-class-binding-{label}-{}-{nonce}-{}",
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

/// One family compiled the way the patrol's fixtures were: `--release 8` with `-g:none`, one
/// jar entry per produced class file, in the writer's own order.
fn compile_family(label: &str, source: &str) -> (Vec<u8>, Vec<(String, Vec<u8>)>) {
    let temp = TestDirectory::new(label);
    let file = source
        .split_once("class ")
        .and_then(|(_, rest)| {
            let name: String = rest
                .chars()
                .take_while(|character| {
                    character.is_alphanumeric() || *character == '_' || *character == '$'
                })
                .collect();
            (!name.is_empty()).then_some(name)
        })
        .map(|name| format!("{name}.java"))
        .expect("the source states a class");
    std::fs::write(temp.path().join(&file), source).expect("the family source is written");
    let compiled = Command::new("javac")
        .args(["--release", "8", "-g:none"])
        .arg(&file)
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let mut owned = Vec::new();
    let mut directories = vec![temp.path().to_owned()];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(&directory).expect("the directory reads") {
            let entry = entry.expect("the directory entry reads");
            if entry.file_type().unwrap().is_dir() {
                directories.push(entry.path());
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("class") {
                continue;
            }
            let name = path
                .strip_prefix(temp.path())
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let bytes = std::fs::read(&path).expect("the class file reads");
            owned.push((name, bytes));
        }
    }
    let entries = owned
        .iter()
        .map(|(name, bytes)| (name.as_bytes(), bytes.as_slice()))
        .collect::<Vec<_>>();
    (jar_of(&entries), owned)
}

fn source_of(jar: &[u8], class: &str) -> ClassSourceReport {
    source_with(
        jar,
        class,
        jarde::facade::task_limits(&[]).expect("bounded"),
    )
}

fn source_with(jar: &[u8], class: &str, limits: Limits) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = Budget::new(limits);
    let snapshot = engine
        .open(ArtifactInput::bytes(jar.to_vec()), &mut budget)
        .expect("the family jar opens");
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

/// The presentation of one standalone `.class` file, the way the patrol replays a frozen fixture.
fn standalone_source(bytes: &[u8], class: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let mut open_budget = Budget::new(jarde::facade::task_limits(&[]).expect("bounded"));
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut open_budget)
        .expect("the class file opens");
    let request = ClassSourceRequest {
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
        &mut Budget::new(jarde::facade::task_limits(&[]).expect("bounded")),
    ) {
        Ok(OperationOutcome::Performed(report)) => report,
        other => panic!("one fixture answers one class-source request: {other:?}"),
    }
}

/// The unit a runtime leg compiles: this run's assembled source without the members it refused.
///
/// A refused member's body presents as its quotes alone, and a `void` body that wrote no statement
/// carries `jarde_refused_body();` — a symbol this project reserves and never declares — so the
/// assembled source of a class with a refused member is not compilable, by design: change
/// `preserve-postfix-fallback-soundness` replaced the partial text that compiled while dropping
/// the statement it could not prove with a refusal that states itself. The anchors of this file are
/// the members the run *proved*, so their runtime legs compile those, and each caller states the
/// refusal it leaves out.
fn proved_unit(report: &ClassSourceReport) -> String {
    let mut unit = report.text.clone();
    for method in &report.methods {
        if matches!(
            &method.outcome,
            ClassSourceOutcome::Recovered { report: run, .. }
                if run.content == RecoveryContent::ExplanationOnly
        ) {
            assert!(
                unit.contains(&method.text),
                "the refused member's own text is a substring of the assembled source:\n{}",
                report.text
            );
            unit = unit.replace(&method.text, "");
        }
    }
    unit
}

/// Compiles one recovered unit beside its own family jar, then runs a runner under
/// `-Xverify:all` — once against the recompiled unit and once against the original family — and
/// returns both outputs: behavior and generic reflection must agree path by path.
fn reflect_and_run(label: &str, unit: &str, class_name: &str, runner: &str, jar: &[u8]) {
    let temp = TestDirectory::new(label);
    std::fs::write(temp.path().join("family.jar"), jar).expect("the family jar is written");
    std::fs::write(temp.path().join(format!("{class_name}.java")), unit)
        .expect("the unit text is written");
    std::fs::write(temp.path().join("Runner.java"), runner).expect("the runner is written");
    let compiled = Command::new("javac")
        .args(["--release", "8", "-cp", "family.jar"])
        .arg(format!("{class_name}.java"))
        .arg("Runner.java")
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "{label}: the recovered unit recompiles under --release 8:\n{}\n{}",
        String::from_utf8_lossy(&compiled.stderr),
        unit
    );
    let run_against = |classpath: &str| {
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp", classpath])
            .arg("Runner")
            .current_dir(temp.path())
            .output()
            .expect("java runs");
        assert!(
            run.status.success(),
            "{label}: the runner verifies and runs ({classpath}):\n{}",
            String::from_utf8_lossy(&run.stderr)
        );
        String::from_utf8(run.stdout).expect("the run prints text")
    };
    let rebuilt = run_against(".:family.jar");
    // The same runner against the original family only: the original classes answer first.
    let original_dir = temp.path().join("original");
    std::fs::create_dir_all(&original_dir).expect("the original directory is created");
    std::fs::copy(
        temp.path().join("family.jar"),
        original_dir.join("family.jar"),
    )
    .expect("the original jar is copied");
    std::fs::copy(
        temp.path().join("Runner.class"),
        original_dir.join("Runner.class"),
    )
    .expect("the runner class is copied");
    let original = {
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp", ".:family.jar"])
            .arg("Runner")
            .current_dir(&original_dir)
            .output()
            .expect("java runs");
        assert!(
            run.status.success(),
            "{label}: the runner verifies and runs against the original family:\n{}",
            String::from_utf8_lossy(&run.stderr)
        );
        String::from_utf8(run.stdout).expect("the run prints text")
    };
    assert_eq!(
        rebuilt, original,
        "{label}: the recovered unit answers exactly what the original family answers"
    );
}

/// Appends dangling same-class member references to a compiled class's constant pool: one
/// `Methodref` and one `Fieldref` naming members nothing in the class consumes. The mutation is
/// append-only, so every existing pool index stays valid; the JVM accepts extra pool entries,
/// and the caller verifies that before trusting the bytes.
fn append_dangling_pool_references(bytes: &[u8], class_internal: &str) -> Vec<u8> {
    let mut pool_end = 10_usize;
    let count = u16::from_be_bytes([bytes[8], bytes[9]]);
    let mut index = 1_u16;
    while index < count {
        let tag = bytes[pool_end];
        let width = match tag {
            1 => {
                let length =
                    u16::from_be_bytes([bytes[pool_end + 1], bytes[pool_end + 2]]) as usize;
                3 + length
            }
            3 | 4 => 5,
            5 | 6 => 9,
            7 | 8 | 16 | 19 | 20 => 3,
            9 | 10 | 11 | 12 | 17 | 18 => 5,
            15 => 4,
            other => panic!("unknown constant pool tag {other} at {pool_end}"),
        };
        pool_end += width;
        index += if tag == 5 || tag == 6 { 2 } else { 1 };
    }
    let mut appended: Vec<u8> = Vec::new();
    // One slot index per appended entry — the count and every back-reference below agree on it.
    let mut slots = 0_u16;
    let push = |entry: &[u8], appended: &mut Vec<u8>, slots: &mut u16| -> u16 {
        let assigned = count + *slots;
        appended.extend_from_slice(entry);
        *slots += 1;
        assigned
    };
    let push_utf8 = |text: &str, appended: &mut Vec<u8>, slots: &mut u16| -> u16 {
        let assigned = count + *slots;
        appended.extend_from_slice(&[1]);
        appended.extend_from_slice(
            &u16::try_from(text.len())
                .expect("the Utf8 text fits its own length")
                .to_be_bytes(),
        );
        appended.extend_from_slice(text.as_bytes());
        *slots += 1;
        assigned
    };
    let mut entry = Vec::new();
    let class_index = {
        let class_utf8 = push_utf8(class_internal, &mut appended, &mut slots);
        entry.extend_from_slice(&[7]);
        entry.extend_from_slice(&class_utf8.to_be_bytes());
        push(&entry, &mut appended, &mut slots)
    };
    let method_nat = {
        let method_name = push_utf8("pick", &mut appended, &mut slots);
        let method_descriptor = push_utf8(
            "(Ljava/lang/Comparable;)Ljava/lang/Comparable;",
            &mut appended,
            &mut slots,
        );
        entry.clear();
        entry.extend_from_slice(&[12]);
        entry.extend_from_slice(&method_name.to_be_bytes());
        entry.extend_from_slice(&method_descriptor.to_be_bytes());
        push(&entry, &mut appended, &mut slots)
    };
    {
        entry.clear();
        entry.extend_from_slice(&[10]);
        entry.extend_from_slice(&class_index.to_be_bytes());
        entry.extend_from_slice(&method_nat.to_be_bytes());
        push(&entry, &mut appended, &mut slots);
    }
    let field_nat = {
        let field_name = push_utf8("unused", &mut appended, &mut slots);
        let field_descriptor = push_utf8("Ljava/util/List;", &mut appended, &mut slots);
        entry.clear();
        entry.extend_from_slice(&[12]);
        entry.extend_from_slice(&field_name.to_be_bytes());
        entry.extend_from_slice(&field_descriptor.to_be_bytes());
        push(&entry, &mut appended, &mut slots)
    };
    {
        entry.clear();
        entry.extend_from_slice(&[9]);
        entry.extend_from_slice(&class_index.to_be_bytes());
        entry.extend_from_slice(&field_nat.to_be_bytes());
        push(&entry, &mut appended, &mut slots);
    }
    let total = count + slots;
    let mut result = bytes[..pool_end].to_vec();
    result[8..10].copy_from_slice(&total.to_be_bytes());
    result.extend_from_slice(&appended);
    result.extend_from_slice(&bytes[pool_end..]);
    result
}

const METHOD_SOURCE: &str = "public class SCGA<T extends java.lang.Comparable<T>> {\n    public void note(T value) {\n        java.util.Collections.singletonList(value);\n        return;\n    }\n    public static void main(java.lang.String[] args) {\n        SCGA z = new SCGA<java.lang.String>();\n        java.lang.Comparable word = \"b\";\n        z.note(word);\n        System.out.println(\"note:\" + word.equals(\"b\"));\n    }\n}\n";

const FIELD_SOURCE: &str = "public class SCGB<T extends java.lang.Comparable<T>> {\n    private java.util.Map<java.lang.String, java.util.List<T>> index = new java.util.HashMap<java.lang.String, java.util.List<T>>();\n    public static void main(java.lang.String[] args) {\n        SCGB<java.lang.String> z = new SCGB<java.lang.String>();\n        if (z.index != null) { System.out.println(\"index:true\"); } else { System.out.println(\"index:false\"); }\n        java.lang.Object read = z.index;\n        if (read instanceof java.util.Map) { System.out.println(\"read:true\"); } else { System.out.println(\"read:false\"); }\n    }\n}\n";

const UNUSED_SOURCE: &str = "public class SCGC<T extends java.lang.Comparable<T>> {\n    public T pick(T value) { return value; }\n    private java.util.List<T> unused;\n    public static void main(java.lang.String[] args) {\n        System.out.println(\"unused-pool-entry\");\n    }\n}\n";

const SIBLING_SOURCE: &str = "public class SCGD<T extends java.lang.Comparable<T>> {\n    public T choose(T value) { return value; }\n    public java.lang.Object choose(java.lang.Object value) { return value; }\n    public static java.lang.Comparable seed() { return \"b\"; }\n    public static void main(java.lang.String[] args) {\n        SCGD z = new SCGD<java.lang.String>();\n        java.lang.Object picked = z.choose(seed());\n        System.out.println(picked);\n    }\n}\n";

const SHADOW_SOURCE: &str = "public class SCGE<T extends java.lang.Comparable<T>> extends SCGEBase {\n    private java.util.List<T> values = new java.util.ArrayList<T>();\n    public static void main(java.lang.String[] args) {\n        SCGE<java.lang.String> z = new SCGE<java.lang.String>();\n        if (z.values != null) { System.out.println(\"values:true\"); } else { System.out.println(\"values:false\"); }\n    }\n}\nclass SCGEBase { protected java.util.List values = new java.util.ArrayList(); }\n";

const RECEIVER_SOURCE: &str = "public class SCGF<T extends java.lang.Comparable<T>> {\n    private java.util.List<T> kept = new java.util.ArrayList<T>();\n    public void add(T value) { kept.add(value); }\n    public static void main(java.lang.String[] args) {\n        SCGF z = new SCGF<java.lang.String>();\n        java.lang.Comparable word = \"b\";\n        z.add(word);\n        System.out.println(\"kept:added\");\n    }\n}\n";

#[test]
fn same_class_method_call_proves_binding_and_reflects_like_the_original() {
    let (jar, _) = compile_family("scg-method", METHOD_SOURCE);
    let report = source_of(&jar, "SCGA");
    let note = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"note")
        .expect("the physical note method remains");
    assert_eq!(note.item.descriptor.raw().0, b"(Ljava/lang/Comparable;)V");
    assert!(
        note.declaration
            .as_deref()
            .is_some_and(|declaration| declaration.contains("public void note(T arg1)")),
        "{}",
        report.text
    );
    assert!(
        note.text.contains("generic Signature `(TT;)V` projected")
            && note.text.contains("same-class call binding proved"),
        "{}",
        note.text
    );
    // `main` sits on another slice's boundary: a `java.lang.String` argument for the erased
    // `java.lang.Comparable` parameter, a platform conversion this layer holds no evidence for.
    // `preserve-postfix-fallback-soundness` makes that refusal loud where the body is `void` —
    // the whole body presents as its quotes alone, and the reserved, undeclared symbol stands
    // where a statement would — instead of publishing the statements beside the refused call. The
    // binding this test accepts is `note`'s, so the runtime leg compiles the members the run
    // proved, and the runner spells the call `main` made.
    let main = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"main")
        .expect("the physical main remains");
    assert!(
        main.text.contains("jarde_refused_body();")
            && main.text.contains("no safe reference conversion evidence"),
        "{}",
        main.text
    );
    // The recovered family recompiles, verifies, runs and reflects exactly like the original:
    // the binding proof admitted a call the original bytecode still owns.
    reflect_and_run(
        "scg-method",
        &proved_unit(&report),
        "SCGA",
        "public class Runner { public static void main(String[] a) throws Exception {\n\
         System.out.println(Class.forName(\"SCGA\").getMethod(\"note\", java.lang.Comparable.class).getGenericParameterTypes()[0]);\n\
         SCGA z = new SCGA<java.lang.String>();\n\
         java.lang.Comparable word = \"b\";\n\
         z.note(word);\n\
         System.out.println(\"note:\" + word.equals(\"b\"));\n} }",
        &jar,
    );
}

#[test]
fn same_class_field_read_write_proves_binding_and_reflects_like_the_original() {
    let (jar, _) = compile_family("scg-field", FIELD_SOURCE);
    let report = source_of(&jar, "SCGB");
    let index = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"index")
        .expect("the physical field remains");
    assert!(
        index
            .declaration
            .as_deref()
            .is_some_and(|declaration| declaration
                .contains("private java.util.Map<java.lang.String, java.util.List<T>> index")),
        "{}",
        report.text
    );
    assert!(
        index
            .markers
            .iter()
            .any(|marker| marker
                .contains("projected after descriptor erasure and same-class uses at")),
        "{}",
        report.text
    );
    reflect_and_run(
        "scg-field",
        &report.text,
        "SCGB",
        "public class Runner { public static void main(String[] a) throws Exception {\n\
         System.out.println(Class.forName(\"SCGB\").getDeclaredField(\"index\").getGenericType());\n\
         SCGB.main(a);\n} }",
        &jar,
    );
}

#[test]
fn unconsumed_same_class_pool_entries_do_not_block_projection() {
    let (jar, family) = compile_family("scg-unused", UNUSED_SOURCE);
    let original = family
        .iter()
        .find(|(name, _)| name == "SCGC.class")
        .expect("the compiled class exists")
        .1
        .clone();
    let mutated = append_dangling_pool_references(&original, "SCGC");
    // The mutation is verifier-valid before anything trusts it.
    let temp = TestDirectory::new("scg-unused-verify");
    std::fs::write(temp.path().join("SCGC.class"), &mutated).expect("the mutated class is written");
    let verify = Command::new("java")
        .args(["-Xverify:all", "-cp", "."])
        .arg("SCGC")
        .current_dir(temp.path())
        .output()
        .expect("java runs");
    assert!(
        verify.status.success(),
        "the mutated class verifies:\n{}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert_eq!(
        String::from_utf8(verify.stdout).unwrap(),
        "unused-pool-entry\n"
    );
    let report = standalone_source(&mutated, "SCGC");
    let pick = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"pick")
        .expect("the physical pick method remains");
    assert!(
        pick.declaration
            .as_deref()
            .is_some_and(|declaration| declaration.contains("T pick(T arg1)")),
        "an unconsumed same-class Methodref blocked the projection:\n{}",
        report.text
    );
    let unused = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"unused")
        .expect("the physical field remains");
    assert!(
        unused
            .declaration
            .as_deref()
            .is_some_and(|declaration| declaration.contains("java.util.List<T> unused")),
        "an unconsumed same-class Fieldref blocked the projection:\n{}",
        report.text
    );
    // The same presentations the unmutated class answers, with the entry absent, still project:
    // the mutation only adds the gate condition the entry's existence used to be.
    let unmutated = standalone_source(&original, "SCGC");
    assert!(
        unmutated
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"pick")
            .and_then(|method| method.declaration.as_deref())
            .is_some_and(|declaration| declaration.contains("T pick(T arg1)"))
    );
    assert!(
        unmutated
            .fields
            .iter()
            .find(|field| field.item.name.raw().0 == b"unused")
            .and_then(|field| field.declaration.as_deref())
            .is_some_and(|declaration| declaration.contains("java.util.List<T> unused"))
    );
    let _ = jar;
}

#[test]
fn adjacent_same_arity_overload_keeps_the_binding_refusal() {
    let (jar, _) = compile_family("scg-sibling", SIBLING_SOURCE);
    let report = source_of(&jar, "SCGD");
    let generic = report
        .methods
        .iter()
        .find(|method| {
            method.item.descriptor.raw().0 == b"(Ljava/lang/Comparable;)Ljava/lang/Comparable;"
        })
        .expect("the physical generic choose remains");
    assert!(
        generic.text.contains("generic_call_binding_unproved"),
        "a same-arity sibling must keep the refusal:\n{}",
        generic.text
    );
    assert!(
        generic
            .declaration
            .as_deref()
            .is_some_and(|declaration| declaration
                .contains("java.lang.Comparable choose(java.lang.Comparable arg1)")),
        "{}",
        report.text
    );
    // The refused class still recompiles and behaves like the original: the refusal keeps every
    // physical binding the bytes state.
    reflect_and_run(
        "scg-sibling",
        &report.text,
        "SCGD",
        "public class Runner { public static void main(String[] a) throws Exception {\n\
         System.out.println(Class.forName(\"SCGD\").getMethods().length > 0);\n\
         SCGD.main(a);\n} }",
        &jar,
    );
}

#[test]
fn field_shadowing_projection_keeps_the_own_field_binding() {
    let (jar, _) = compile_family("scg-shadow", SHADOW_SOURCE);
    let report = source_of(&jar, "SCGE");
    let shadow = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"values")
        .expect("the physical shadow field remains");
    assert!(
        shadow
            .declaration
            .as_deref()
            .is_some_and(|declaration| declaration.contains("java.util.List<T> values")),
        "{}",
        report.text
    );
    // The base class's own `values` declaration stays out of this presentation: the projection
    // claims only the selected class's field, whose own declaration is what shadows.
    assert_eq!(
        report
            .fields
            .iter()
            .filter(|field| field.item.name.raw().0 == b"values")
            .count(),
        1
    );
    reflect_and_run(
        "scg-shadow",
        &report.text,
        "SCGE",
        "public class Runner { public static void main(String[] a) throws Exception {\n\
         System.out.println(Class.forName(\"SCGE\").getDeclaredField(\"values\").getGenericType());\n\
         SCGE.main(a);\n} }",
        &jar,
    );
}

#[test]
fn receiver_consumed_field_read_keeps_the_field_refusal() {
    let (jar, _) = compile_family("scg-receiver", RECEIVER_SOURCE);
    let report = source_of(&jar, "SCGF");
    let kept = report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"kept")
        .expect("the physical field remains");
    assert!(
        kept.markers
            .iter()
            .any(|marker| marker.contains("field_generic_body_unproved")),
        "a receiver-consumed read must keep the refusal:\n{}",
        report.text
    );
    assert!(
        kept.declaration
            .as_deref()
            .is_some_and(|declaration| declaration.contains("java.util.List kept")),
        "{}",
        report.text
    );
    // This fixture's `main` sits on the same platform-conversion boundary as the method-call
    // family's, and `preserve-postfix-fallback-soundness` refuses a `void` body whole where such a
    // refusal stands at its top level. The field refusal this test keeps is `kept`'s, so the
    // runtime leg compiles the members the run proved, with the runner spelling the call `main`
    // made.
    let main = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"main")
        .expect("the physical main remains");
    assert!(main.text.contains("jarde_refused_body();"), "{}", main.text);
    reflect_and_run(
        "scg-receiver",
        &proved_unit(&report),
        "SCGF",
        "public class Runner { public static void main(String[] a) throws Exception {\n\
         SCGF z = new SCGF<java.lang.String>();\n\
         java.lang.Comparable word = \"b\";\n\
         z.add(word);\n\
         System.out.println(\"kept:added\");\n} }",
        &jar,
    );
}

#[test]
fn z1_family_replays_item_by_item_after_the_binding_proof() {
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/openspec/evidence/java-syntax-2026-10-03/nested-generic-header-patrol/fixture"
    );
    let z1 = std::fs::read(format!("{fixture}/Z1.class")).expect("the Z1 fixture reads");
    let report = standalone_source(&z1, "Z1");
    let method = |name: &[u8]| {
        report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == name)
            .unwrap_or_else(|| panic!("the physical {} remains", String::from_utf8_lossy(name)))
    };
    // The two gates this change proves: `add`'s header and `index`'s field type publish.
    assert!(
        method(b"add")
            .declaration
            .as_deref()
            .is_some_and(|declaration| declaration.contains("public void add(T arg1)")),
        "{}",
        report.text
    );
    assert!(
        method(b"add")
            .text
            .contains("same-class call binding proved")
            && method(b"add").text.contains("(TT;)V"),
        "{}",
        report.text
    );
    assert!(
        report
            .fields
            .iter()
            .any(|field| field.item.name.raw().0 == b"index"
                && field
                    .declaration
                    .as_deref()
                    .is_some_and(|declaration| declaration
                        .contains("java.util.Map<java.lang.String, java.util.List<T>> index")))
    );
    // The members whose bodies this slice cannot re-type keep explicit refusals: the field read
    // that selects members on the value, and the bodies with no same-run return proof.
    assert!(report.fields.iter().any(|field| {
        field.item.name.raw().0 == b"items"
            && field
                .markers
                .iter()
                .any(|marker| marker.contains("field_generic_body_unproved"))
    }));
    assert!(
        method(b"first")
            .text
            .contains("generic Signature projection refused"),
        "{}",
        report.text
    );
    assert!(
        method(b"map")
            .text
            .contains("generic Signature projection refused"),
        "{}",
        report.text
    );
    assert!(
        method(b"named")
            .text
            .contains("generic Signature projection refused"),
        "{}",
        report.text
    );
    // The separated child keeps exactly the presentation the headers slice published.
    let box_bytes =
        std::fs::read(format!("{fixture}/Z1$Box.class")).expect("the Box fixture reads");
    let box_report = standalone_source(&box_bytes, "Z1$Box");
    assert!(
        box_report
            .text
            .contains("class Z1$Box<U> extends java.lang.Object {")
    );
    assert!(box_report.text.contains("U value;"));
}

#[test]
fn budget_and_cancellation_never_publish_a_partial_binding_projection() {
    let (jar, _) = compile_family("scg-stop", METHOD_SOURCE);
    let baseline = source_of(&jar, "SCGA");
    assert!(
        baseline.text.contains("public void note(T arg1)"),
        "{}",
        baseline.text
    );
    let mut limits = jarde::facade::task_limits(&[]).expect("bounded");
    limits.analysis_steps = baseline.usage.analysis_steps.saturating_sub(1);
    let engine = Engine::new();
    let mut open_budget = Budget::new(jarde::facade::task_limits(&[]).expect("bounded"));
    let snapshot = engine
        .open(ArtifactInput::bytes(jar.clone()), &mut open_budget)
        .expect("the family jar opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("SCGA"),
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
    let limited = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::new(limits),
        )
        .expect("the engine answers");
    match limited {
        OperationOutcome::Incomplete(_) => {}
        OperationOutcome::Performed(report) => {
            // A stop can land after the commit published `note` whole — but never mid-member: a
            // projected header always carries its proof marker and its recovered body, and a
            // header the budget could not pay for is absent entirely.
            if report.text.contains("public void note(T arg1)") {
                assert!(
                    report.text.contains("same-class call binding proved")
                        && report.text.contains("java.util.Collections.singletonList"),
                    "a budget stop published a partial generic header:\n{}",
                    report.text
                );
            }
        }
        other => panic!("unexpected limited outcome: {other:?}"),
    }
    let cancellation = jarde::CancellationToken::new();
    cancellation.cancel();
    let cancelled = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::with_cancellation_token(
                jarde::facade::task_limits(&[]).expect("bounded"),
                cancellation,
            ),
        )
        .expect("the engine answers");
    assert!(matches!(cancelled, OperationOutcome::Incomplete(_)));
}

#[test]
fn essential_and_all_evidence_admit_the_same_binding_projections() {
    let (jar, _) = compile_family("scg-evidence", METHOD_SOURCE);
    let engine = Engine::new();
    let mut open_budget = Budget::new(jarde::facade::task_limits(&[]).expect("bounded"));
    let snapshot = engine
        .open(ArtifactInput::bytes(jar.clone()), &mut open_budget)
        .expect("the family jar opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("SCGA"),
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
    let essential = engine
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::essential(),
            &mut Budget::new(jarde::facade::task_limits(&[]).expect("bounded")),
        )
        .expect("essential answers");
    let all = engine
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(jarde::facade::task_limits(&[]).expect("bounded")),
        )
        .expect("all answers");
    let (OperationOutcome::Performed(essential), OperationOutcome::Performed(all)) =
        (essential, all)
    else {
        panic!("both selections perform");
    };
    assert_eq!(essential.text, all.text);
    assert!(essential.text.contains("public void note(T arg1)"));
}
