//! `preserve-invocation-argument-types`: a call argument is written with the type proven at its
//! invocation site.  The committed Java 8 class deliberately selects overloads that differ only
//! by the argument type, so dropping that fact changes both the source and the result.
//!
//! The fixture has one permanent class.  The target overloads, the generic helper and the runner
//! are source-only inputs for the ignored JDK comparison; the runner is always created in a
//! temporary directory.  The method reference is named rather than synthetic so the fixture does
//! not depend on a generated lambda class name.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-invocation-arguments/v8/InvocationArgumentsProbe.class");
const PROBE_SOURCE: &str =
    include_str!("fixtures/p3-invocation-arguments/InvocationArgumentsProbe.java");
const TARGET_SOURCE: &str = include_str!("fixtures/p3-invocation-arguments/ArgumentTarget.java");
const GENERIC_SOURCE: &str = include_str!("fixtures/p3-invocation-arguments/GenericFactory.java");
const RUNNER_SOURCE: &str =
    include_str!("fixtures/p3-invocation-arguments/InvocationArgumentsRunner.java");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed invocation-arguments fixture opens as a standalone CLASS")
}

fn class_source_of(snapshot: &ArtifactSnapshot) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("InvocationArgumentsProbe"),
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
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
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

fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the fixture's method table"))
        .text
}

fn assert_presented<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    let text = text_of(report, name);
    assert!(
        !text.contains("@bytecode"),
        "`{name}` remains a fallback instead of a recovered body:\n{text}"
    );
    text
}

#[test]
fn fixture_has_one_real_class_and_all_argument_shapes() {
    assert!(PROBE_SOURCE.contains("(Object) GenericFactory.make()"));
    assert!(PROBE_SOURCE.contains("widening(char value)"));
    assert!(PROBE_SOURCE.contains("multiple(char value)"));
    assert!(PROBE_SOURCE.contains("(Object) (Runnable) InvocationArgumentsProbe::returnFive"));
    assert!(GENERIC_SOURCE.contains("public static <T> T make()"));
    assert!(PROBE_SOURCE.contains("(Runnable) InvocationArgumentsProbe::returnFive"));

    let report = class_source_of(&open(FIXTURE));
    assert_eq!(
        report.methods.len(),
        14,
        "constructor plus thirteen code methods"
    );
    for name in [
        "<init>",
        "objectString",
        "objectNull",
        "objectArray",
        "objectBoxed",
        "widening",
        "narrowByte",
        "narrowShort",
        "constructorObject",
        "multiple",
        "genericObject",
        "functionalRunnable",
        "functionalObject",
        "returnFive",
    ] {
        let method = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == name.as_bytes())
            .unwrap_or_else(|| panic!("missing `{name}` in the committed class"));
        assert!(
            matches!(method.item.body, MemberBodyEvidence::CodeAttribute { .. }),
            "`{name}` has a Code attribute"
        );
    }
}

#[test]
fn recovered_calls_keep_their_invocation_argument_types() {
    let report = class_source_of(&open(FIXTURE));

    let object_string = assert_presented(&report, "objectString");
    assert!(
        object_string.contains("ArgumentTarget.object((java.lang.Object) \"x\")"),
        "the Object overload is selected by the call descriptor:\n{object_string}"
    );

    let object_null = assert_presented(&report, "objectNull");
    assert!(
        object_null.contains("ArgumentTarget.object((java.lang.Object) null)"),
        "null retains the invocation descriptor's Object type:\n{object_null}"
    );

    let object_array = assert_presented(&report, "objectArray");
    assert!(
        object_array.contains("ArgumentTarget.array(")
            && object_array.contains("(java.lang.Object)")
            && object_array.contains("arg0"),
        "the array expression reaches the Object overload with its call type:\n{object_array}"
    );

    let object_boxed = assert_presented(&report, "objectBoxed");
    assert!(
        object_boxed
            .contains("ArgumentTarget.boxed((java.lang.Object) java.lang.Integer.valueOf(3))"),
        "boxing does not change the Object argument fact:\n{object_boxed}"
    );

    let widening = assert_presented(&report, "widening");
    assert!(
        widening.contains("ArgumentTarget.number(")
            && widening.contains("(int)")
            && widening.contains("arg0"),
        "the int invocation descriptor is retained for the char source value:\n{widening}"
    );

    let narrow_byte = assert_presented(&report, "narrowByte");
    assert!(
        narrow_byte.contains("ArgumentTarget.narrow((byte) 1)"),
        "the byte overload remains distinguishable from int:\n{narrow_byte}"
    );
    let narrow_short = assert_presented(&report, "narrowShort");
    assert!(
        narrow_short.contains("ArgumentTarget.narrow((short) 1)"),
        "the short overload remains distinguishable from int:\n{narrow_short}"
    );

    let constructor = assert_presented(&report, "constructorObject");
    assert!(
        constructor.contains("new ArgumentTarget((java.lang.Object) \"x\")"),
        "the constructor descriptor types its argument:\n{constructor}"
    );

    let multiple = assert_presented(&report, "multiple");
    for snippet in [
        "(java.lang.Object) \"a\"",
        "(java.lang.Object) \"b\"",
        "(int)",
    ] {
        assert!(
            multiple.contains(snippet),
            "the multi-argument call retains `{snippet}`:\n{multiple}"
        );
    }

    let generic = assert_presented(&report, "genericObject");
    assert!(
        generic.contains("ArgumentTarget.object((java.lang.Object) GenericFactory.make())"),
        "a generic poly invocation keeps the explicit Object call type:\n{generic}"
    );

    let functional = assert_presented(&report, "functionalRunnable");
    assert!(
        functional.contains(
            "ArgumentTarget.functional((java.lang.Runnable) InvocationArgumentsProbe::returnFive)"
        ),
        "the Runnable factory target is preserved at the overloaded call:\n{functional}"
    );
    let functional_object = assert_presented(&report, "functionalObject");
    assert!(
        functional_object.contains(
            "ArgumentTarget.objectFunctional((java.lang.Object) (java.lang.Runnable) InvocationArgumentsProbe::returnFive)",
        ),
        "an Object-consuming call keeps both the outer invocation type and inner functional target:\n{functional_object}"
    );
}

#[test]
fn call_casts_keep_their_call_site_as_derived_provenance() {
    let report = class_source_of(&open(FIXTURE));
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"objectString")
        .expect("objectString is present");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("objectString should have a recovered body")
    };
    assert!(
        !report.source_map.direct_of_bci(2).is_empty(),
        "the invokestatic remains a direct source-map anchor: {:?}",
        report.source_map.segments()
    );
    assert!(
        !report.source_map.derived_of_bci(2).is_empty(),
        "the call-site type constraint is a derived anchor of the argument cast: {:?}",
        report.source_map.segments()
    );
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-invocation-arguments-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the JDK comparison directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn javac(dir: &Path, files: &[&str]) {
    let output = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-g:none")
        .arg("-d")
        .arg(dir)
        .args(files)
        .current_dir(dir)
        .output()
        .expect("start javac");
    assert!(
        output.status.success(),
        "compiling generated recovery failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_runner(dir: &Path) -> String {
    let output = Command::new("java")
        .arg("-cp")
        .arg(dir)
        .arg("InvocationArgumentsRunner")
        .current_dir(dir)
        .output()
        .expect("execute the fixture runner");
    assert!(
        output.status.success(),
        "the fixture runner failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the fixture runner writes UTF-8")
}

fn write_sources(dir: &Path, probe: &str) {
    fs::write(dir.join("InvocationArgumentsProbe.java"), probe)
        .expect("write the generated probe source");
    fs::write(dir.join("ArgumentTarget.java"), TARGET_SOURCE)
        .expect("write the source-only overload target");
    fs::write(dir.join("GenericFactory.java"), GENERIC_SOURCE)
        .expect("write the source-only generic helper");
    fs::write(dir.join("InvocationArgumentsRunner.java"), RUNNER_SOURCE)
        .expect("write the temporary runner");
}

#[test]
#[ignore = "requires JDK: compile Engine::class_source output and execute a temporary runner"]
fn recovered_invocation_arguments_match_original_runtime() {
    let report = class_source_of(&open(FIXTURE));
    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered = scratch.path().join("recovered");
    fs::create_dir_all(&original).expect("create the original comparison directory");
    fs::create_dir_all(&recovered).expect("create the recovered comparison directory");

    write_sources(&original, PROBE_SOURCE);
    javac(
        &original,
        &[
            "ArgumentTarget.java",
            "GenericFactory.java",
            "InvocationArgumentsProbe.java",
            "InvocationArgumentsRunner.java",
        ],
    );
    let original_output = run_runner(&original);

    write_sources(&recovered, &report.text);
    javac(
        &recovered,
        &[
            "ArgumentTarget.java",
            "GenericFactory.java",
            "InvocationArgumentsProbe.java",
            "InvocationArgumentsRunner.java",
        ],
    );
    let recovered_output = run_runner(&recovered);

    assert_eq!(
        recovered_output, original_output,
        "genuine Engine-generated bodies preserve overload selection and runtime values"
    );
}
