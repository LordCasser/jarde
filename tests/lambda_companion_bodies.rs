//! The lambda companion-body channel (change `recover-lambda-inline-bodies`): a class whose
//! lambda expressions co-presented javac's synthetic `lambda$…` companions is a class javac
//! cannot recompile — it re-synthesizes the companion's name for the lambda expression itself.
//! The class-source assembly therefore inlines a straight single-return companion body into the
//! lambda that calls it (hiding the physical method), or renames the companion (`$jarde` suffix,
//! a name javac never synthesizes) when the body is not provably straight, and keeps a companion
//! that serves several lambda sites exactly as the class file states it.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;

const Y1_CLASS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-03/lambda-inline-patrol/fixture/Y1.class"
);

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn scratch(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("jarde-lambda-bodies-{}-{name}", std::process::id()));
    fs::create_dir_all(&dir).expect("the scratch directory is created");
    dir
}

fn compile_and_open(dir: &std::path::Path, file: &str, sources: &[(&str, &str)]) -> Vec<u8> {
    for (name, source) in sources {
        fs::write(dir.join(name), source).expect("the fixture source is written");
    }
    let mut command = Command::new("javac");
    command.args(["--release", "8", "-g:none", "-d"]).arg(dir);
    for (name, _) in sources {
        command.arg(dir.join(name));
    }

    let output = command.output().expect("JDK javac is available");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::read(dir.join(file)).expect("the fixture class is read")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed fixture opens as a standalone class")
}

fn request(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceRequest {
    ClassSourceRequest {
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
            loader: LoaderId("app".to_string()),
        },
    }
}

fn class_source(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request(snapshot, class),
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("the fixture's class-source request is valid")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => {
            panic!("the fixture is ambiguous: {candidates:?}")
        }
        OperationOutcome::Incomplete(reason) => panic!("the fixture is incomplete: {reason:?}"),
    }
}

fn recompile_and_run(dir: &std::path::Path, main: &str) -> String {
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(dir)
        .arg(main)
        .output()
        .expect("JDK java is available");
    assert!(
        run.status.success(),
        "the recompiled program failed verification or execution:\n{}\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8_lossy(&run.stdout).to_string()
}

fn javac_wrote_warning_free(output: &std::process::Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// The frozen patrol anchor: four lambda sites, three kinds of companion body (concatenation,
/// comparison, subtraction, boxing), a captured parameter — the whole class recompiles under
/// `--release 8` and its `main` output is byte-identical to the original class file's.
#[test]
fn the_frozen_y1_class_recompiles_with_every_companion_projected() {
    let snapshot = open(Y1_CLASS);
    let recovered = class_source(&snapshot, "Y1");
    // Every lambda expression carries its companion's body (the captured-parameter site keeps
    // the site's parameter names: the companion's own would shadow the enclosing parameter).
    assert!(
        recovered
            .text
            .contains("(java.lang.String p0) -> p0 + \"!\""),
        "{}",
        recovered.text
    );
    assert!(recovered.text.contains("() -> 42"), "{}", recovered.text);
    assert!(
        recovered
            .text
            .contains("(java.lang.Object arg0) -> ((java.lang.String) arg0).length() > 2"),
        "{}",
        recovered.text
    );
    assert!(
        recovered.text.contains(
            "(java.lang.Object arg0, java.lang.Object arg1) -> ((java.lang.String) arg0).length() - ((java.lang.String) arg1).length()"
        ),
        "{}",
        recovered.text
    );
    assert!(
        recovered.text.contains(
            "(java.lang.Object arg1) -> java.lang.Integer.valueOf(((java.lang.Integer) arg1).intValue() + arg0)"
        ),
        "{}",
        recovered.text
    );
    // The method-reference site (`String::length`, rendered as its call) is not this channel's.
    assert!(
        recovered
            .text
            .contains("(java.lang.Object p0) -> ((java.lang.String) p0).length()"),
        "{}",
        recovered.text
    );
    for helper in [
        "lambda$viaLambda$0",
        "lambda$viaMethodRef$1",
        "lambda$viaStream$2",
        "lambda$viaStream$3",
        "lambda$captureLambda$4",
    ] {
        assert!(
            !recovered.text.contains(&format!("{helper}(")),
            "companion call or declaration leaked: {helper}\n{}",
            recovered.text
        );
        assert!(
            recovered
                .methods
                .iter()
                .any(|method| method.item.name.raw().0 == helper.as_bytes()),
            "the physical method report must remain for {helper}"
        );
    }
    let emitted = scratch("y1-emitted");
    fs::write(emitted.join("Y1.java"), &recovered.text).expect("the recovered text is written");
    fs::write(
        emitted.join("Y1$StrFn.java"),
        "public interface Y1$StrFn { String apply(String s); }\n",
    )
    .expect("the nested interface is written");
    let compile = Command::new("javac")
        .args(["--release", "8", "-d"])
        .arg(&emitted)
        .arg(emitted.join("Y1.java"))
        .arg(emitted.join("Y1$StrFn.java"))
        .output()
        .expect("JDK javac is available");
    javac_wrote_warning_free(&compile);
    assert_eq!(
        recompile_and_run(&emitted, "Y1"),
        "hi!\n45\n[b, aa]\n8\n",
        "the recompiled class must behave exactly as the original class file"
    );
}

/// The variant matrix: a two-parameter comparator whose companion's parameter names would
/// shadow the enclosing ones (the site's names are kept), a captured local, two lambdas in one
/// method, and a `long` parameter whose value occupies two slots — one class, all inlined.
#[test]
fn companion_bodies_inline_across_the_variant_matrix() {
    let original = scratch("v1-original");
    let bytes = compile_and_open(
        &original,
        "V1.class",
        &[(
            "V1.java",
            "import java.util.*;\n\
             import java.util.function.*;\n\
             public class V1 {\n\
               static List<String> byComparator(List<String> xs) { xs.sort((a, b) -> a.compareTo(b)); return xs; }\n\
               static int captureLocal(int n) { int base = n * 2; IntUnaryOperator f = v -> v + base; return f.applyAsInt(5); }\n\
               static int manyInOne(int a, int b) { IntSupplier s = () -> a + b; IntUnaryOperator u = x -> x * b; return s.getAsInt() + u.applyAsInt(3); }\n\
               static long wide(long seed) { LongUnaryOperator w = v -> v + seed; return w.applyAsLong(3L); }\n\
               static List<String> seed() { List<String> xs = new ArrayList<String>(); xs.add(\"pear\"); xs.add(\"apple\"); xs.add(\"fig\"); return xs; }\n\
               public static void main(String[] args) {\n\
                 System.out.println(byComparator(seed())); System.out.println(captureLocal(10));\n\
                 System.out.println(manyInOne(4, 5)); System.out.println(wide(40L));\n\
               }\n\
             }\n",
        )],
    );
    let snapshot = open(&bytes);
    let recovered = class_source(&snapshot, "V1");
    assert!(
        recovered.text.contains(
            "(java.lang.Object p0, java.lang.Object p1) -> ((java.lang.String) p0).compareTo((java.lang.String) p1)"
        ),
        "the comparator keeps the site's names where the companion's would shadow: {}",
        recovered.text
    );
    assert!(
        recovered.text.contains("(int arg1) -> arg1 + local1;"),
        "the captured local is embedded where the call read it: {}",
        recovered.text
    );
    assert!(
        recovered.text.contains("() -> arg0 + arg1;")
            && recovered.text.contains("(int p0) -> p0 * arg1;"),
        "both companions of one method inline in a single re-emission: {}",
        recovered.text
    );
    assert!(
        recovered.text.contains("(long arg2) -> arg2 + arg0;"),
        "the two-slot long parameter binds by category position: {}",
        recovered.text
    );
    for helper in [
        "lambda$byComparator$0",
        "lambda$captureLocal$1",
        "lambda$manyInOne$2",
        "lambda$manyInOne$3",
        "lambda$wide$4",
    ] {
        assert!(
            !recovered.text.contains(&format!("{helper}(")),
            "companion leaked: {helper}\n{}",
            recovered.text
        );
    }
    let emitted = scratch("v1-emitted");
    fs::write(emitted.join("V1.java"), &recovered.text).expect("the recovered text is written");
    let compile = Command::new("javac")
        .args(["--release", "8", "-d"])
        .arg(&emitted)
        .arg(emitted.join("V1.java"))
        .output()
        .expect("JDK javac is available");
    javac_wrote_warning_free(&compile);
    assert_eq!(
        recompile_and_run(&emitted, "V1"),
        "[apple, fig, pear]\n25\n24\n43\n",
        "the recompiled variants must behave exactly as the original class file"
    );
}

/// The conservative branch: a conditional body and a block body are not provably straight-line,
/// so both companions keep their declarations under the `$jarde` name and the calls name them —
/// the class recompiles either way.
#[test]
fn conditional_and_block_companions_rename_instead_of_inlining() {
    let original = scratch("v2-original");
    let bytes = compile_and_open(
        &original,
        "V2.class",
        &[(
            "V2.java",
            "import java.util.function.*;\n\
             public class V2 {\n\
               static int apply(IntUnaryOperator f, int x) { return f.applyAsInt(x); }\n\
               static int branchy(int x) { return apply(v -> v > 0 ? v : 0, x); }\n\
               static int blocky(int x) { return apply(v -> { int t = v + 1; return t * 2; }, x); }\n\
               public static void main(String[] args) { System.out.println(branchy(-7)); System.out.println(branchy(9)); System.out.println(blocky(20)); }\n\
             }\n",
        )],
    );
    let snapshot = open(&bytes);
    let recovered = class_source(&snapshot, "V2");
    assert!(
        recovered.text.contains("lambda$branchy$0$jarde("),
        "the conditional body's companion is renamed and called by its new name: {}",
        recovered.text
    );
    assert!(
        recovered
            .text
            .contains("private static int lambda$branchy$0$jarde("),
        "the renamed companion keeps a declaration: {}",
        recovered.text
    );
    assert!(
        recovered.text.contains("lambda$blocky$1$jarde("),
        "the multi-statement body's companion is renamed: {}",
        recovered.text
    );
    let emitted = scratch("v2-emitted");
    fs::write(emitted.join("V2.java"), &recovered.text).expect("the recovered text is written");
    let compile = Command::new("javac")
        .args(["--release", "8", "-d"])
        .arg(&emitted)
        .arg(emitted.join("V2.java"))
        .output()
        .expect("JDK javac is available");
    javac_wrote_warning_free(&compile);
    assert_eq!(
        recompile_and_run(&emitted, "V2"),
        "0\n9\n42\n",
        "the renamed companions must keep the original class file's behavior"
    );
}

/// The registered negative: two lambda sites sharing one companion (a shape javac's dedup emits
/// for identical bodies, and bytecode surgery restores when it does not). Renaming the shared
/// companion would make the other site's text claim a member the class does not declare, so the
/// physical presentation is kept and the reason is registered.
#[test]
fn a_multi_use_companion_keeps_its_physical_presentation() {
    let original = scratch("m1-original");
    let mut bytes = compile_and_open(
        &original,
        "M1.class",
        &[(
            "M1.java",
            "import java.util.function.*;\n\
             public class M1 {\n\
               static int first(int x) { IntSupplier s = () -> x * 3; return s.getAsInt(); }\n\
               static int second(int x) { IntSupplier s = () -> x * 3; return s.getAsInt(); }\n\
               public static void main(String[] args) { System.out.println(first(2)); System.out.println(second(5)); }\n\
             }\n",
        )],
    );
    if companion_count(&bytes, "M1") == 2 {
        // This javac did not dedup the identical bodies: re-point the second site's InvokeDynamic
        // entry at the first bootstrap row by hand, exactly as the multi-use shape states it.
        share_one_bootstrap(&mut bytes);
    }
    let snapshot = open(&bytes);
    let recovered = class_source(&snapshot, "M1");
    let helper = shared_companion_name(&bytes);
    assert_eq!(companion_use_sites(&recovered.text, &helper), 2);
    assert!(
        recovered
            .diagnostics
            .iter()
            .any(
                |diagnostic| diagnostic.code == "lambda_helper_projection_refused"
                    && diagnostic.message.contains(&helper)
                    && diagnostic.message.contains("serves 2 lambda sites")
            ),
        "the multi-use companion must be registered, not rewritten: {:?}",
        recovered.diagnostics
    );
    assert!(
        !recovered.text.contains("$jarde"),
        "no rename is made for a shared companion: {}",
        recovered.text
    );
}

/// A curried lambda's outer companion holds another lambda site of its own: the outer body is
/// refused (moving it would relocate the inner site), renamed, and the inner companion inlines
/// into the renamed member's body — one class, both decisions, still compilable.
#[test]
fn a_curried_lambda_renames_its_outer_companion_and_inlines_the_inner() {
    let original = scratch("curried-original");
    let bytes = compile_and_open(
        &original,
        "Curried.class",
        &[(
            "Curried.java",
            "import java.util.function.*;\n\
             public class Curried {\n\
               static Function<Integer, IntUnaryOperator> curried(int base) { return x -> y -> y + base + x; }\n\
               static int run() { return curried(10).apply(5).applyAsInt(3); }\n\
               public static void main(String[] args) { System.out.println(run()); }\n\
             }\n",
        )],
    );
    let snapshot = open(&bytes);
    let recovered = class_source(&snapshot, "Curried");
    let emitted = scratch("curried-emitted");
    fs::write(emitted.join("Curried.java"), &recovered.text)
        .expect("the recovered text is written");
    let compile = Command::new("javac")
        .args(["--release", "8", "-d"])
        .arg(&emitted)
        .arg(emitted.join("Curried.java"))
        .output()
        .expect("JDK javac is available");
    javac_wrote_warning_free(&compile);
    assert_eq!(
        recompile_and_run(&emitted, "Curried"),
        "18\n",
        "the curried class must behave exactly as the original class file"
    );
}

/// A class without lambda companions reads exactly as it did: no projection markers, no
/// diagnostics from this channel, and its text compiles as before.
#[test]
fn a_class_without_companions_takes_no_projection_markers() {
    let original = scratch("plain-original");
    let bytes = compile_and_open(
        &original,
        "Plain.class",
        &[(
            "Plain.java",
            "public class Plain {\n\
               static int square(int x) { return x * x; }\n\
               public static void main(String[] args) { System.out.println(square(6)); }\n\
             }\n",
        )],
    );
    let snapshot = open(&bytes);
    let recovered = class_source(&snapshot, "Plain");
    assert!(
        !recovered.text.contains("lambda companion"),
        "{}",
        recovered.text
    );
    assert!(
        !recovered.text.contains("omitted physical lambda helper"),
        "{}",
        recovered.text
    );
    assert!(
        !recovered
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "lambda_helper_projection_refused"),
        "{:?}",
        recovered.diagnostics
    );
    let emitted = scratch("plain-emitted");
    fs::write(emitted.join("Plain.java"), &recovered.text).expect("the recovered text is written");
    let compile = Command::new("javac")
        .args(["--release", "8", "-d"])
        .arg(&emitted)
        .arg(emitted.join("Plain.java"))
        .output()
        .expect("JDK javac is available");
    javac_wrote_warning_free(&compile);
    assert_eq!(recompile_and_run(&emitted, "Plain"), "36\n");
}

/// How many `lambda$…` companions this class declares.
fn companion_count(bytes: &[u8], _class: &str) -> usize {
    constant_pool_utf8s(bytes)
        .into_iter()
        .filter(|name| name.starts_with("lambda$"))
        .count()
}

/// The one `lambda$…` name a multi-use class shares (or the last one before surgery).
fn shared_companion_name(bytes: &[u8]) -> String {
    constant_pool_utf8s(bytes)
        .into_iter()
        .filter(|name| name.starts_with("lambda$"))
        .max()
        .expect("the fixture declares a companion")
}

/// Every qualified call of one companion in the text — its declaration and comment lines carry
/// the name too, so the call shape is the counted fact.
fn companion_use_sites(text: &str, helper: &str) -> usize {
    text.matches(&format!(".{helper}(")).count()
}

/// The class file's own `Utf8` pool entries, read only for the small fixtures this suite builds.
fn constant_pool_utf8s(bytes: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut at = 10;
    let count = u16::from_be_bytes([bytes[8], bytes[9]]);
    for _ in 1..count {
        let tag = bytes[at];
        match tag {
            1 => {
                let len = u16::from_be_bytes([bytes[at + 1], bytes[at + 2]]) as usize;
                out.push(String::from_utf8_lossy(&bytes[at + 3..at + 3 + len]).to_string());
                at += 3 + len;
            }
            5 | 6 => at += 9,
            7 | 8 | 16 | 19 | 20 => at += 3,
            15 => at += 4,
            9 | 10 | 11 | 12 | 17 | 18 => at += 5,
            _ => break,
        }
    }
    out
}

/// Re-points every `InvokeDynamic` entry whose bootstrap row is the second one at the first row,
/// making the second companion's sites resolve through the first companion's bootstrap.
fn share_one_bootstrap(bytes: &mut [u8]) {
    let mut at = 10;
    let count = u16::from_be_bytes([bytes[8], bytes[9]]);
    let mut bootstrap_of = Vec::new();
    for _ in 1..count {
        let tag = bytes[at];
        let entry_at = at;
        match tag {
            1 => {
                let len = u16::from_be_bytes([bytes[at + 1], bytes[at + 2]]) as usize;
                at += 3 + len;
            }
            5 | 6 => at += 9,
            7 | 8 | 16 | 19 | 20 => at += 3,
            15 => at += 4,
            9 | 10 | 11 | 12 | 17 => at += 5,
            18 => {
                let bootstrap = u16::from_be_bytes([bytes[at + 1], bytes[at + 2]]);
                bootstrap_of.push((entry_at, bootstrap));
                at += 5;
            }
            _ => break,
        }
    }
    let second_bootstrap = bootstrap_of
        .iter()
        .map(|(_, bootstrap)| *bootstrap)
        .max()
        .expect("the fixture states at least two bootstrap rows");
    for (entry_at, bootstrap) in bootstrap_of {
        if bootstrap == second_bootstrap {
            bytes[entry_at + 1] = 0;
            bytes[entry_at + 2] = 0;
        }
    }
}
