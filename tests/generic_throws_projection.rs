use jarde::*;
use std::{
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const CLASS_NAME: &str = "GenericThrowsBoundary";
const SOURCE: &str = r#"
public abstract class GenericThrowsBoundary<E extends Exception> {
    public abstract void invoke() throws E;
}
"#;
const CALLER: &str = r#"
public class GenericThrowsCaller {
    static class Impl extends GenericThrowsBoundary<RuntimeException> {
        public void invoke() { System.out.println("invoked"); }
    }
    static void narrowed(GenericThrowsBoundary<RuntimeException> value) {
        value.invoke();
    }
    public static void main(String[] args) throws Exception {
        narrowed(new Impl());
        System.out.println("throws=" + GenericThrowsBoundary.class
            .getMethod("invoke").getGenericExceptionTypes()[0].getTypeName());
    }
}
"#;
const METHOD_BODY_LOCAL_THROWS: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/body-method-local-generic-throws/fixture/MethodBodyThrows.java"
);
const METHOD_BODY_LOCAL_CALLER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/body-method-local-generic-throws/fixture/MethodBodyThrowsCaller.java"
);
const METHOD_BODY_LOCAL_REFLECT: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/body-method-local-generic-throws/fixture/MethodBodyThrowsReflect.java"
);

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "jarde-generic-throws-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn compile(dir: &Path, debug: &str, sources: &[(&str, &str)]) {
    let mut command = Command::new("javac");
    command
        .args(["--release", "8", "-Xlint:-options", debug, "-d"])
        .arg(dir);
    for (name, source) in sources {
        let path = dir.join(format!("{name}.java"));
        fs::write(&path, source).unwrap();
        command.arg(path);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn class_source(bytes: Vec<u8>) -> ClassSourceReport {
    class_source_named(bytes, CLASS_NAME)
}

fn class_source_named(bytes: Vec<u8>, class_name: &str) -> ClassSourceReport {
    match class_source_named_with_evidence(bytes, class_name, None, None) {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class source result: {other:?}"),
    }
}

fn class_source_named_with_evidence(
    bytes: Vec<u8>,
    class_name: &str,
    evidence: Option<RecoveryEvidenceRequest>,
    limits: Option<Limits>,
) -> OperationOutcome<ClassSourceReport> {
    let engine = Engine::new();
    let mut budget = Budget::new(limits.unwrap_or(task_limits(&[]).unwrap()));
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class_name),
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
    match evidence {
        Some(evidence) => {
            engine.class_source_with_evidence(&[snapshot], &request, &evidence, &mut budget)
        }
        None => engine.class_source(&[snapshot], &request, &mut budget),
    }
    .unwrap()
}

fn replace_utf8(bytes: &[u8], old: &[u8], new: &[u8]) -> Vec<u8> {
    let count = u16::from_be_bytes([bytes[8], bytes[9]]) as usize;
    let mut output = bytes[..10].to_vec();
    let mut offset = 10;
    let mut index = 1;
    while index < count {
        let tag = bytes[offset];
        let entry_start = offset;
        offset += 1;
        match tag {
            1 => {
                let length = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;
                let start = offset + 2;
                let end = start + length;
                if let Some(relative) = bytes[start..end]
                    .windows(old.len())
                    .position(|window| window == old)
                {
                    let mut value = bytes[start..end].to_vec();
                    if old.len() != new.len() {
                        assert_eq!(
                            &value, old,
                            "length-changing replacement must target a complete UTF8 constant"
                        );
                    }
                    value.splice(relative..relative + old.len(), new.iter().copied());
                    output.push(tag);
                    output.extend_from_slice(&(value.len() as u16).to_be_bytes());
                    output.extend_from_slice(&value);
                } else {
                    output.extend_from_slice(&bytes[entry_start..end]);
                }
                offset = end;
            }
            3 | 4 | 9 | 10 | 11 | 12 | 17 | 18 => {
                offset += 4;
                output.extend_from_slice(&bytes[entry_start..offset]);
            }
            5 | 6 => {
                offset += 8;
                output.extend_from_slice(&bytes[entry_start..offset]);
                index += 1;
            }
            7 | 8 | 16 | 19 | 20 => {
                offset += 2;
                output.extend_from_slice(&bytes[entry_start..offset]);
            }
            15 => {
                offset += 3;
                output.extend_from_slice(&bytes[entry_start..offset]);
            }
            other => panic!("unsupported constant-pool tag {other}"),
        }
        index += 1;
    }
    output.extend_from_slice(&bytes[offset..]);
    output
}

#[test]
fn class_throws_variable_survives_both_debug_variants_and_typed_callers() {
    for (label, debug) in [("g", "-g"), ("none", "-g:none")] {
        let original = temp_dir(&format!("original-{label}"));
        compile(&original, debug, &[(CLASS_NAME, SOURCE)]);
        let bytes = fs::read(original.join(format!("{CLASS_NAME}.class"))).unwrap();
        let report = class_source(bytes);
        assert!(
            report.text.contains("abstract void invoke() throws E;"),
            "{}",
            report.text
        );
        assert!(
            !report.text.contains("generic Signature projection refused"),
            "{}",
            report.text
        );

        let rebuilt = temp_dir(&format!("rebuilt-{label}"));
        let rebuilt_path = rebuilt.join(format!("{CLASS_NAME}.java"));
        fs::write(&rebuilt_path, &report.text).unwrap();
        fs::write(rebuilt.join("GenericThrowsCaller.java"), CALLER).unwrap();
        let output = Command::new("javac")
            .args(["--release", "8", "-Xlint:-options", "-d"])
            .arg(&rebuilt)
            .arg(&rebuilt_path)
            .arg(rebuilt.join("GenericThrowsCaller.java"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            report.text
        );
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(&rebuilt)
            .arg("GenericThrowsCaller")
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8(run.stdout).unwrap(),
            "invoked\nthrows=E\n"
        );
        fs::remove_dir_all(original).unwrap();
        fs::remove_dir_all(rebuilt).unwrap();
    }
}

#[test]
fn mixed_throws_positions_and_absent_suffix_keep_their_sources() {
    let source = r#"
import java.io.IOException;
public abstract class GenericThrowsMixed<E extends Exception> {
    public abstract void mixed() throws IOException, E;
    public abstract void physical() throws Exception;
}

"#;
    let dir = temp_dir("mixed-source");
    compile(&dir, "-g:none", &[("GenericThrowsMixed", source)]);
    let path = dir.join("GenericThrowsMixed.class");
    let bytes = fs::read(&path).unwrap();
    let report = class_source_named(bytes.clone(), "GenericThrowsMixed");
    assert!(
        report
            .text
            .contains("mixed() throws java.io.IOException, E;"),
        "{}",
        report.text
    );

    // Remove only the Signature throws suffix; retain both physical Exceptions entries.
    let without_suffix = replace_utf8(&bytes, b"()V^Ljava/io/IOException;^TE;", b"()V");
    let report = class_source_named(without_suffix, "GenericThrowsMixed");
    assert!(
        report
            .text
            .contains("mixed() throws java.io.IOException, java.lang.Exception;"),
        "{}",
        report.text
    );
    assert!(
        !report
            .text
            .contains("generic Signature projection refused for `mixed()V`"),
        "{}",
        report.text
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn method_local_generic_no_body_signatures_keep_throws_and_parameter_positions() {
    let source = r#"
import java.io.IOException;
public abstract class MethodLocalNoBody {
    public abstract <T extends Number> T echo(T value);
    public abstract <X extends Exception> void raise() throws X;
    public abstract <T extends Number> T checked(T value) throws IOException;
    public abstract <T extends Number> T first(@Mark T... values);
}

"#;
    let mark = r#"
import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.PARAMETER)
@interface Mark { }
"#;
    let caller = r#"
import java.io.IOException;
public class MethodLocalNoBodyCaller {
    static final class Impl extends MethodLocalNoBody {
        public <T extends Number> T echo(T value) { return value; }
        public <X extends Exception> void raise() throws X { }
        public <T extends Number> T checked(T value) throws IOException { return value; }
        public <T extends Number> T first(T... values) { return values[0]; }
    }
    static Integer narrowed(MethodLocalNoBody value) {
        value.<RuntimeException>raise();
        return value.first(value.echo(Integer.valueOf(7)));
    }
    public static void main(String[] args) throws Exception {
        MethodLocalNoBody value = new Impl();
        System.out.println("value=" + narrowed(value));
        System.out.println("echo=" + MethodLocalNoBody.class.getMethod("echo", Number.class)
            .getGenericReturnType().getTypeName());
        System.out.println("raise=" + MethodLocalNoBody.class.getMethod("raise")
            .getGenericExceptionTypes()[0].getTypeName());
        System.out.println("checked=" + MethodLocalNoBody.class.getMethod("checked", Number.class)
            .getExceptionTypes()[0].getName());
        System.out.println("first=" + MethodLocalNoBody.class.getMethod("first", Number[].class)
            .getParameterAnnotations()[0][0].annotationType().getSimpleName());
    }
}
"#;

    for (label, debug) in [("g", "-g"), ("none", "-g:none")] {
        let original = temp_dir(&format!("method-local-original-{label}"));
        compile(
            &original,
            debug,
            &[
                ("MethodLocalNoBody", source),
                ("Mark", mark),
                ("MethodLocalNoBodyCaller", caller),
            ],
        );
        let bytes = fs::read(original.join("MethodLocalNoBody.class")).unwrap();
        let essential = match class_source_named_with_evidence(
            bytes.clone(),
            "MethodLocalNoBody",
            Some(RecoveryEvidenceRequest::essential()),
            None,
        ) {
            OperationOutcome::Performed(report) => report,
            other => panic!("unexpected essential outcome: {other:?}"),
        };
        let all = match class_source_named_with_evidence(
            bytes,
            "MethodLocalNoBody",
            Some(RecoveryEvidenceRequest::all()),
            None,
        ) {
            OperationOutcome::Performed(report) => report,
            other => panic!("unexpected all-evidence outcome: {other:?}"),
        };
        assert_eq!(essential.text, all.text);
        for expected in [
            "abstract <T extends java.lang.Number> T echo(T arg1);",
            "abstract <X extends java.lang.Exception> void raise() throws X;",
            "abstract <T extends java.lang.Number> T checked(T arg1) throws java.io.IOException;",
            "abstract <T extends java.lang.Number> T first(@Mark T... arg1);",
        ] {
            assert!(
                all.text.contains(expected),
                "missing {expected}: {}",
                all.text
            );
        }
        assert!(
            !all.text.contains("generic Signature projection refused"),
            "{}",
            all.text
        );

        let rebuilt = temp_dir(&format!("method-local-rebuilt-{label}"));
        fs::write(rebuilt.join("MethodLocalNoBody.java"), &all.text).unwrap();
        fs::write(rebuilt.join("Mark.java"), mark).unwrap();
        fs::write(rebuilt.join("MethodLocalNoBodyCaller.java"), caller).unwrap();
        let javac = Command::new("javac")
            .args(["--release", "8", "-Xlint:-options", "-d"])
            .arg(&rebuilt)
            .arg(rebuilt.join("MethodLocalNoBody.java"))
            .arg(rebuilt.join("Mark.java"))
            .arg(rebuilt.join("MethodLocalNoBodyCaller.java"))
            .output()
            .unwrap();
        assert!(
            javac.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&javac.stderr),
            all.text
        );
        let executed = Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(&rebuilt)
            .arg("MethodLocalNoBodyCaller")
            .output()
            .unwrap();
        assert!(
            executed.status.success(),
            "{}",
            String::from_utf8_lossy(&executed.stderr)
        );
        assert_eq!(
            String::from_utf8(executed.stdout).unwrap(),
            "value=7\necho=T\nraise=X\nchecked=java.io.IOException\nfirst=Mark\n"
        );
        fs::remove_dir_all(original).unwrap();
        fs::remove_dir_all(rebuilt).unwrap();
    }
}
#[test]
fn no_body_method_local_projection_requires_a_safe_object_name_and_no_local_methodref() {
    let source = r#"
public abstract class MethodLocalBindingBoundary {
    public abstract <T extends Number> T target(T value);
    public abstract <X extends Exception> void wait(X value) throws X;
    public static Number call(MethodLocalBindingBoundary value) {
        return value.target(Integer.valueOf(7));
    }
}
"#;
    let dir = temp_dir("method-local-binding-boundary");
    compile(&dir, "-g:none", &[("MethodLocalBindingBoundary", source)]);
    let report = class_source_named(
        fs::read(dir.join("MethodLocalBindingBoundary.class")).unwrap(),
        "MethodLocalBindingBoundary",
    );
    assert!(
        report.text.contains("generic_call_binding_unproved"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("java.lang.Number target(java.lang.Number arg1)"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("method name may override an Object instance method"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("void wait(java.lang.Exception arg1) throws java.lang.Exception;"),
        "{}",
        report.text
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn method_local_no_body_projection_stops_atomically_on_budget_and_cancellation() {
    let source = r#"
public abstract class MethodLocalStopBoundary {
    public abstract <X extends Exception> void raise() throws X;
}
"#;
    let dir = temp_dir("method-local-stop-boundary");
    compile(&dir, "-g:none", &[("MethodLocalStopBoundary", source)]);
    let bytes = fs::read(dir.join("MethodLocalStopBoundary.class")).unwrap();
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut task_budget(&[]).unwrap())
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("MethodLocalStopBoundary"),
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
    let baseline = match engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected baseline: {other:?}"),
    };
    assert!(baseline.text.contains("<X extends java.lang.Exception>"));

    let mut limits = task_limits(&[]).unwrap();
    limits.analysis_steps = baseline.usage.analysis_steps.saturating_sub(1);
    let limited = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::new(limits),
        )
        .unwrap();
    match limited {
        OperationOutcome::Incomplete(_) => {}
        OperationOutcome::Performed(report) => assert!(
            !report.text.contains("<X extends java.lang.Exception>"),
            "budget stop published a partial method-local header: {}",
            report.text
        ),
        other => panic!("unexpected limited outcome: {other:?}"),
    }

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let cancelled = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::with_cancellation_token(task_limits(&[]).unwrap(), cancellation),
        )
        .unwrap();
    assert!(matches!(cancelled, OperationOutcome::Incomplete(_)));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn unsupported_bound_and_unbound_variable_refuse_locally() {
    let dir = temp_dir("negative-source");
    compile(&dir, "-g:none", &[(CLASS_NAME, SOURCE)]);
    let path = dir.join(format!("{CLASS_NAME}.class"));
    let bytes = fs::read(&path).unwrap();

    // Change only generic metadata: Throwable and Exception have equal-length names, while
    // the descriptor and physical Exceptions attribute remain byte-for-byte unchanged.
    let non_exception = replace_utf8(
        &bytes,
        b"E:Ljava/lang/Exception;",
        b"E:Ljava/lang/Throwable;",
    );
    fs::write(&path, &non_exception).unwrap();
    let caller = r#"
public class GenericThrowsVerifierCaller {
    public static void main(String[] args) {
        GenericThrowsBoundary<RuntimeException> value = new GenericThrowsBoundary<RuntimeException>() {
            public void invoke() { }
        };
        value.invoke();
    }
}
"#;
    let caller_path = dir.join("GenericThrowsVerifierCaller.java");
    fs::write(&caller_path, caller).unwrap();
    let compiled = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options", "-cp"])
        .arg(&dir)
        .arg("-d")
        .arg(&dir)
        .arg(&caller_path)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let verify = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&dir)
        .arg("GenericThrowsVerifierCaller")
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "{}",
        String::from_utf8_lossy(&verify.stderr)
    );
    let report = class_source(non_exception);
    assert!(
        report.text.contains("jvm_signature_erasure_mismatch"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("throws java.lang.Exception;"),
        "{}",
        report.text
    );

    // An undeclared throws variable is rejected by the reader's scope proof before spelling.
    let unbound = replace_utf8(&bytes, b"()V^TE;", b"()V^TX;");
    let report = class_source(unbound);
    assert!(
        report.text.contains("jvm_signature_scope_unproved"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("throws java.lang.Exception;"),
        "{}",
        report.text
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn empty_body_methods_project_and_evidence_choices_publish_the_same_header() {
    let body_source = r#"
public class GenericThrowsBody<E extends Exception> {
    public void body() throws E { }
}

"#;
    let dir = temp_dir("body-source");
    compile(&dir, "-g:none", &[("GenericThrowsBody", body_source)]);
    let body_report = class_source_named(
        fs::read(dir.join("GenericThrowsBody.class")).unwrap(),
        "GenericThrowsBody",
    );
    assert!(
        body_report.text.contains("body() throws E"),
        "{}",
        body_report.text
    );
    assert!(
        body_report
            .text
            .contains("same-run AST/Code/SSA empty-void proof")
    );
    fs::remove_dir_all(dir).unwrap();

    let dir = temp_dir("evidence-source");
    compile(&dir, "-g:none", &[(CLASS_NAME, SOURCE)]);
    let bytes = fs::read(dir.join(format!("{CLASS_NAME}.class"))).unwrap();
    let run = |evidence| match class_source_named_with_evidence(
        bytes.clone(),
        CLASS_NAME,
        Some(evidence),
        None,
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class source result: {other:?}"),
    };
    let essential = run(RecoveryEvidenceRequest::essential());
    let all = run(RecoveryEvidenceRequest::all());
    assert_eq!(essential.text, all.text);
    assert!(essential.text.contains("throws E;"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn method_local_body_throws_projection_compiles_and_runs_with_and_without_debug_tables() {
    for (label, debug) in [("debug", "-g"), ("no-debug", "-g:none")] {
        let original = temp_dir(&format!("method-body-local-{label}"));
        compile(
            &original,
            debug,
            &[
                ("MethodBodyThrows", METHOD_BODY_LOCAL_THROWS),
                ("MethodBodyThrowsCaller", METHOD_BODY_LOCAL_CALLER),
                ("MethodBodyThrowsReflect", METHOD_BODY_LOCAL_REFLECT),
            ],
        );
        let bytes = fs::read(original.join("methodbodythrows/MethodBodyThrows.class")).unwrap();
        let report = class_source_named(bytes.clone(), "methodbodythrows/MethodBodyThrows");
        let essential = match class_source_named_with_evidence(
            bytes.clone(),
            "methodbodythrows/MethodBodyThrows",
            Some(RecoveryEvidenceRequest::essential()),
            None,
        ) {
            OperationOutcome::Performed(report) => report,
            other => panic!("unexpected essential-evidence outcome: {other:?}"),
        };
        let all = match class_source_named_with_evidence(
            bytes.clone(),
            "methodbodythrows/MethodBodyThrows",
            Some(RecoveryEvidenceRequest::all()),
            None,
        ) {
            OperationOutcome::Performed(report) => report,
            other => panic!("unexpected all-evidence outcome: {other:?}"),
        };
        assert_eq!(report.text, essential.text);
        assert_eq!(report.text, all.text);
        assert!(
            report
                .text
                .contains("public <X extends java.lang.Exception> void run() throws X"),
            "{}",
            report.text
        );
        assert!(
            report.text.contains(
                "same-run AST/Code/SSA empty-void and method-local Signature scope/erasure proof"
            ),
            "{}",
            report.text
        );
        assert!(
            !report
                .text
                .contains("generic Signature projection refused for `run()V`"),
            "{}",
            report.text
        );

        let rebuilt = temp_dir(&format!("method-body-local-rebuilt-{label}"));
        let class_path = rebuilt.join("MethodBodyThrows.java");
        fs::write(&class_path, &report.text).unwrap();
        let caller_path = rebuilt.join("MethodBodyThrowsCaller.java");
        fs::write(&caller_path, METHOD_BODY_LOCAL_CALLER).unwrap();
        let reflect_path = rebuilt.join("MethodBodyThrowsReflect.java");
        fs::write(&reflect_path, METHOD_BODY_LOCAL_REFLECT).unwrap();
        let javac = Command::new("javac")
            .args(["--release", "8", "-Xlint:-options", "-d"])
            .arg(&rebuilt)
            .arg(&class_path)
            .arg(&caller_path)
            .arg(&reflect_path)
            .output()
            .unwrap();
        assert!(
            javac.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&javac.stderr),
            report.text
        );
        let caller = Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(&rebuilt)
            .arg("methodbodythrows.MethodBodyThrowsCaller")
            .output()
            .unwrap();
        assert!(
            caller.status.success(),
            "{}",
            String::from_utf8_lossy(&caller.stderr)
        );
        assert_eq!(String::from_utf8(caller.stdout).unwrap(), "called\n");
        let reflection = Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(&rebuilt)
            .arg("methodbodythrows.MethodBodyThrowsReflect")
            .output()
            .unwrap();
        assert!(
            reflection.status.success(),
            "{}",
            String::from_utf8_lossy(&reflection.stderr)
        );
        assert_eq!(
            String::from_utf8(reflection.stdout).unwrap(),
            "parameters=1\nthrows=X\n"
        );
    }
}

#[test]
fn method_local_body_throws_refuses_a_generic_class_signature() {
    let source = r#"
package methodbodythrows;
public class MethodBodyThrowsGeneric<T> {
    public <X extends Exception> void run() throws X { }
}
"#;
    let verifier = r#"
package methodbodythrows;
public class MethodBodyThrowsGenericVerifier {
    public static void main(String[] args) throws Exception {
        Class.forName("methodbodythrows.MethodBodyThrowsGeneric");
    }
}
"#;
    let dir = temp_dir("method-body-local-generic-class");
    compile(
        &dir,
        "-g:none",
        &[
            ("MethodBodyThrowsGeneric", source),
            ("MethodBodyThrowsGenericVerifier", verifier),
        ],
    );
    let class_path = dir.join("methodbodythrows/MethodBodyThrowsGeneric.class");
    let bytes = fs::read(&class_path).unwrap();
    let bytes = replace_utf8(
        &bytes,
        b"<T:Ljava/lang/Object;>Ljava/lang/Object;",
        b"Ljava/lang/Object;",
    );
    fs::write(&class_path, &bytes).unwrap();
    let verified = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&dir)
        .arg("methodbodythrows.MethodBodyThrowsGenericVerifier")
        .output()
        .unwrap();
    assert!(
        verified.status.success(),
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );
    let report = class_source_named(bytes, "methodbodythrows/MethodBodyThrowsGeneric");
    assert!(
        report
            .text
            .contains("generic Signature projection refused for `run()V`"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("void run() throws java.lang.Exception"),
        "{}",
        report.text
    );
    assert!(
        !report
            .text
            .contains("public <X extends java.lang.Exception> void run() throws X"),
        "{}",
        report.text
    );
}

#[test]
fn method_local_body_throws_with_a_handler_keeps_physical_declaration() {
    let source = r#"
public class MethodLocalHandler {
    public <X extends Exception> void run() throws X {
        try { throw new Exception("handled"); } catch (Exception ignored) { }
    }
}
"#;
    let verifier = r#"
public class MethodLocalHandlerVerifier {
    public static void main(String[] args) throws Exception {
        Class.forName("MethodLocalHandler");
    }
}
"#;
    let dir = temp_dir("method-local-handler");
    compile(
        &dir,
        "-g:none",
        &[
            ("MethodLocalHandler", source),
            ("MethodLocalHandlerVerifier", verifier),
        ],
    );
    let class_path = dir.join("MethodLocalHandler.class");
    let javap = Command::new("javap")
        .args(["-v"])
        .arg(&class_path)
        .output()
        .unwrap();
    assert!(
        javap.status.success(),
        "{}",
        String::from_utf8_lossy(&javap.stderr)
    );
    assert!(
        String::from_utf8_lossy(&javap.stdout).contains("Exception table:"),
        "{}",
        String::from_utf8_lossy(&javap.stdout)
    );
    let verified = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&dir)
        .arg("MethodLocalHandlerVerifier")
        .output()
        .unwrap();
    assert!(
        verified.status.success(),
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );

    let report = class_source_named(fs::read(class_path).unwrap(), "MethodLocalHandler");
    assert!(
        report
            .text
            .contains("generic Signature projection refused for `run()V`"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("public void run() throws java.lang.Exception"),
        "{}",
        report.text
    );
    assert!(
        !report
            .text
            .contains("public <X extends java.lang.Exception> void run() throws X"),
        "{}",
        report.text
    );
}

#[test]
fn method_local_body_throws_object_method_name_keeps_physical_declaration() {
    let verifier = r#"
package methodbodythrows;
public class MethodBodyThrowsVerifier {
    public static void main(String[] args) throws Exception {
        Class.forName("methodbodythrows.MethodBodyThrows");
    }
}
"#;
    let dir = temp_dir("method-body-local-object-name");
    compile(
        &dir,
        "-g:none",
        &[
            ("MethodBodyThrows", METHOD_BODY_LOCAL_THROWS),
            ("MethodBodyThrowsVerifier", verifier),
        ],
    );
    let class_path = dir.join("methodbodythrows/MethodBodyThrows.class");
    let original = fs::read(&class_path).unwrap();
    let renamed = replace_utf8(&original, b"run", b"finalize");
    fs::write(&class_path, &renamed).unwrap();
    let verified = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&dir)
        .arg("methodbodythrows.MethodBodyThrowsVerifier")
        .output()
        .unwrap();
    assert!(
        verified.status.success(),
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );

    let report = class_source_named(renamed, "methodbodythrows/MethodBodyThrows");
    assert!(
        report
            .text
            .contains("generic Signature projection refused for `finalize()V`"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("public void finalize() throws java.lang.Exception"),
        "{}",
        report.text
    );
    assert!(
        !report
            .text
            .contains("public <X extends java.lang.Exception> void finalize() throws X"),
        "{}",
        report.text
    );
    let rebuilt = temp_dir("method-body-local-object-name-rebuilt");
    let source_path = rebuilt.join("MethodBodyThrows.java");
    fs::write(&source_path, &report.text).unwrap();
    let javac = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options", "-d"])
        .arg(&rebuilt)
        .arg(&source_path)
        .output()
        .unwrap();
    assert!(
        javac.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&javac.stderr),
        report.text
    );
}

#[test]
fn method_local_body_throws_budget_stop_and_cancellation_never_publish_a_partial_header() {
    let dir = temp_dir("method-body-local-stop");
    compile(
        &dir,
        "-g:none",
        &[("MethodBodyThrows", METHOD_BODY_LOCAL_THROWS)],
    );
    let bytes = fs::read(dir.join("methodbodythrows/MethodBodyThrows.class")).unwrap();
    let baseline = class_source_named(bytes.clone(), "methodbodythrows/MethodBodyThrows");
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut task_budget(&[]).unwrap())
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("methodbodythrows/MethodBodyThrows"),
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
    let mut limits = task_limits(&[]).unwrap();
    limits.analysis_steps = baseline.usage.analysis_steps.saturating_sub(1);
    let stopped = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::new(limits),
        )
        .unwrap();
    match stopped {
        OperationOutcome::Incomplete(_) => {}
        OperationOutcome::Performed(report) => assert!(
            !report
                .text
                .contains("public <X extends java.lang.Exception> void run() throws X"),
            "{}",
            report.text
        ),
        other => panic!("unexpected budget-stopped result: {other:?}"),
    }

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let stopped = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::with_cancellation_token(task_limits(&[]).unwrap(), cancellation),
        )
        .unwrap();
    assert!(matches!(stopped, OperationOutcome::Incomplete(_)));
}

#[test]
fn budget_stop_and_cancellation_never_publish_a_partial_throws_header() {
    let dir = temp_dir("stopped-source");
    compile(&dir, "-g:none", &[(CLASS_NAME, SOURCE)]);
    let bytes = fs::read(dir.join(format!("{CLASS_NAME}.class"))).unwrap();
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut task_budget(&[]).unwrap())
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(CLASS_NAME),
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
    let baseline = match engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected baseline: {other:?}"),
    };
    let mut limits = task_limits(&[]).unwrap();
    limits.analysis_steps = baseline.usage.analysis_steps.saturating_sub(1);
    let stopped = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::new(limits),
        )
        .unwrap();
    match stopped {
        OperationOutcome::Incomplete(_) => {}
        OperationOutcome::Performed(report) => assert!(
            !report.text.contains("invoke() throws E;"),
            "{}",
            report.text
        ),
        other => panic!("unexpected stopped outcome: {other:?}"),
    }

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let stopped = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::with_cancellation_token(task_limits(&[]).unwrap(), cancellation),
        )
        .unwrap();
    assert!(matches!(stopped, OperationOutcome::Incomplete(_)));
    fs::remove_dir_all(dir).unwrap();
}
