//! Acceptance tests of change `recover-parameterized-interface-headers`.
//!
//! The interface leg publishes the class `Signature`'s interface type arguments into the
//! `implements` clause (`implements java.lang.Comparable<IfaceImpl>`), which is what lets a
//! recompiled source regenerate the erased parameter bridge — so the bridge admission of
//! `recover-bridge-admission-gates` (whose precondition refuses a bridge under a raw generic
//! interface header) admits the projection instead of refusing it. The parent leg publishes the
//! pool-spelled parameterized superclass (`extends NB$Box<java.lang.String>`) of a `$`-named
//! parent, and the same seam then hides `cc4b6f11`'s parameter bridge.
//!
//! Every positive is a three-way run: the frozen original class under `java -Xverify:all`, the
//! rendered text recompiled with `javac --release 8`, and the two traces compared. Every
//! negative is one of the pinned terminal states: an interface the environment does not provide
//! keeps its physical spelling and its bridge visible; a definition that contradicts the class
//! `Signature` refuses the header; the parent cells whose criteria are unmet stay refused, and
//! the bare `$`-named parent keeps the raw header.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const STORE: u16 = 0;

const IFACE_IMPL: &[u8] =
    include_bytes!("fixtures/p3-interface-header-projection/v8/IfaceImpl.class");
const MULTI_IFACE: &[u8] =
    include_bytes!("fixtures/p3-interface-header-projection/v8/MultiIface.class");
const ERASED_CALL: &[u8] =
    include_bytes!("fixtures/p3-interface-header-projection/v8/ErasedCall.class");
const UNRESOLVED: &[u8] =
    include_bytes!("fixtures/p3-interface-header-projection/v8/Unresolved.class");
const ARITY: &[u8] = include_bytes!("fixtures/p3-interface-header-projection/v8/Arity.class");
const ARITY_API: &[u8] =
    include_bytes!("fixtures/p3-interface-header-projection/v8/ArityApi.class");
const TYPE_USE: &[u8] = include_bytes!("fixtures/p3-interface-header-projection/v8/TypeUse.class");
const MARK: &[u8] = include_bytes!("fixtures/p3-interface-header-projection/v8/Mark.class");

const NB: &[u8] = include_bytes!("fixtures/p3-nested-parent-projection/v8/NB.class");
const NB_ROOT: &[u8] = include_bytes!("fixtures/p3-nested-parent-projection/v8/NB$Root.class");
const NB_BOX: &[u8] = include_bytes!("fixtures/p3-nested-parent-projection/v8/NB$Box.class");
const NESTED_EXTENDS: &[u8] =
    include_bytes!("fixtures/p3-nested-parent-projection/v8/NestedExtends.class");
const BARE_BOX: &[u8] = include_bytes!("fixtures/p3-nested-parent-projection/v8/BareBox.class");
const NB_TWIN: &[u8] = include_bytes!("fixtures/p3-nested-parent-projection/v8/NB$Twin.class");
const ARITY_EXTENDS: &[u8] =
    include_bytes!("fixtures/p3-nested-parent-projection/v8/ArityExtends.class");
const MO: &[u8] = include_bytes!("fixtures/p3-nested-parent-projection/v8/MO.class");
const MO_MID: &[u8] = include_bytes!("fixtures/p3-nested-parent-projection/v8/MO$Mid.class");
const MULTISEG: &[u8] = include_bytes!("fixtures/p3-nested-parent-projection/v8/Multiseg.class");

/// The interface-reference call the acceptance asks for is made from a **source** runner (the
/// same shape `br_family_recovered_source_recompiles_and_runs_like_the_original` uses): the
/// fixture classes themselves carry no body that calls their own erased contract, because such a
/// body's recovered call spells the erased cast and is the recorded boundary `ErasedCall` pins.
const IFACE_RUNNER: &str = r#"
public class IfaceRunner {
    public static void main(String[] args) {
        java.lang.Comparable<IfaceImpl> contract = new IfaceImpl();
        System.out.println(contract.compareTo(new IfaceImpl()));
        System.out.println(new IfaceImpl().compareTo(new IfaceImpl()));
    }
}
"#;

const MULTI_RUNNER: &str = r#"
public class MultiRunner {
    public static void main(String[] args) {
        java.lang.Comparable<MultiIface> contract = new MultiIface();
        System.out.println(contract.compareTo(new MultiIface()));
        ((java.lang.Runnable) new MultiIface()).run();
        System.out.println("run");
    }
}
"#;

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

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget())
        .expect("the fixture snapshot opens")
}

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
        .class_source(std::slice::from_ref(snapshot), &request, &mut budget())
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source result: {other:?}"),
    }
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-parameterized-interface-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the Java comparison directory");
        Self(path)
    }

    fn child(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(&path).expect("create a Java comparison case directory");
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn compile_java_8(directory: &Path, source: &str, classpath: &Path) {
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-Xlint:-options", "-classpath"])
        .arg(classpath)
        .arg("-d")
        .arg(directory)
        .arg(directory.join(source))
        .output()
        .expect("JDK javac is available");
    assert!(
        compile.status.success(),
        "javac rejected {source}:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
}

fn run_java(directory: &Path, main: &str) -> String {
    let run = Command::new("java")
        .args(["-Xverify:all", "-classpath"])
        .arg(directory)
        .arg(main)
        .output()
        .expect("JDK java is available");
    assert!(
        run.status.success(),
        "{main} failed JVM verification or execution:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the trace is UTF-8")
}

/// The frozen class runs under `-Xverify:all` through `runner`, the rendered text is recompiled
/// with `javac --release 8` and run through the same runner, and the two traces must both equal
/// `expected`. The runner is source, so its interface-reference calls are javac's own — the
/// acceptance's `0` through an interface reference is that call.
fn assert_roundtrip_matches(
    original: &[u8],
    name: &str,
    rendered: &str,
    runner: &str,
    expected: &str,
) {
    let runner_name = runner
        .lines()
        .find_map(|line| line.strip_prefix("public class "))
        .and_then(|rest| rest.split_whitespace().next())
        .expect("the runner declares one public class");

    let scratch = Scratch::new(name);
    let original_dir = scratch.child("original");
    fs::write(original_dir.join(format!("{name}.class")), original).expect("write the original");
    fs::write(original_dir.join(format!("{runner_name}.java")), runner).expect("write the runner");
    compile_java_8(&original_dir, &format!("{runner_name}.java"), &original_dir);
    let original_trace = run_java(&original_dir, runner_name);
    assert_eq!(original_trace, expected);

    let recovered_dir = scratch.child("recovered");
    fs::write(recovered_dir.join(format!("{name}.java")), rendered)
        .expect("write the rendered source");
    compile_java_8(&recovered_dir, &format!("{name}.java"), &recovered_dir);
    fs::write(recovered_dir.join(format!("{runner_name}.java")), runner).expect("write the runner");
    compile_java_8(
        &recovered_dir,
        &format!("{runner_name}.java"),
        &recovered_dir,
    );
    let recovered_trace = run_java(&recovered_dir, runner_name);
    assert_eq!(
        recovered_trace, original_trace,
        "the recompiled source must dispatch like the original class"
    );
}

fn declaration_of(report: &ClassSourceReport) -> String {
    report
        .declaration
        .as_ref()
        .expect("a class report carries its declaration")
        .declaration
        .clone()
}

#[test]
fn a_parameterized_interface_entry_publishes_its_arguments_and_hides_the_bridge() {
    let jar = open(zip_of(&[(b"IfaceImpl.class", IFACE_IMPL)]));
    let report = class_source_of(&jar, "IfaceImpl");
    assert_eq!(
        declaration_of(&report),
        "public class IfaceImpl extends java.lang.Object implements java.lang.Comparable<IfaceImpl>"
    );
    assert_eq!(report.bridge_proofs.len(), 1);
    let proof = &report.bridge_proofs[0];
    assert!(proof.admitted, "{:?}", proof.refusal);
    assert!(proof.projected);
    assert_eq!(proof.refusal, None);
    assert!(
        !report.text.contains("compareTo(java.lang.Object"),
        "the header carries the contract's arguments, so the bridge hides"
    );
    assert!(report.text.contains("public int compareTo(IfaceImpl"));

    assert_roundtrip_matches(
        IFACE_IMPL,
        "IfaceImpl",
        &report.text,
        IFACE_RUNNER,
        "0\n0\n",
    );
}

#[test]
fn a_multi_interface_header_projects_the_parameterized_entry_only() {
    let jar = open(zip_of(&[(b"MultiIface.class", MULTI_IFACE)]));
    let report = class_source_of(&jar, "MultiIface");
    assert_eq!(
        declaration_of(&report),
        "public class MultiIface extends java.lang.Object implements java.lang.Comparable<MultiIface>, java.lang.Runnable"
    );
    assert_eq!(report.bridge_proofs.len(), 1);
    assert!(report.bridge_proofs[0].admitted);
    assert!(report.bridge_proofs[0].projected);
    assert!(!report.text.contains("compareTo(java.lang.Object"));

    assert_roundtrip_matches(
        MULTI_IFACE,
        "MultiIface",
        &report.text,
        MULTI_RUNNER,
        "1\nrun\n",
    );
}

#[test]
fn a_body_calling_its_own_erased_contract_records_the_argument_spelling_boundary() {
    // The recorded boundary (see the fixture's own note): the header projects and the bridge
    // hides, but the recovered body's call is the bytecode's erased one
    // (`compareTo((java.lang.Object) …)`), which only the visible bridge could answer — so this
    // body does not compile until the invocation-argument-typing domain re-types such call sites.
    // The test asserts the terminal state *and* the failure, so the boundary is loud rather than
    // silent; the class text carries the standard per-member marker.
    let jar = open(zip_of(&[(b"ErasedCall.class", ERASED_CALL)]));
    let report = class_source_of(&jar, "ErasedCall");
    assert_eq!(
        declaration_of(&report),
        "public class ErasedCall extends java.lang.Object implements java.lang.Comparable<ErasedCall>"
    );
    assert_eq!(report.bridge_proofs.len(), 1);
    assert!(report.bridge_proofs[0].admitted);
    assert!(report.bridge_proofs[0].projected);
    assert!(
        report
            .text
            .contains("compareTo((java.lang.Object) new ErasedCall())"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("// recovered from bytecode; presentation is not claimed to compile")
    );

    let scratch = Scratch::new("ErasedCall");
    let recovered_dir = scratch.child("recovered");
    fs::write(recovered_dir.join("ErasedCall.java"), &report.text)
        .expect("write the rendered source");
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-Xlint:-options", "-d"])
        .arg(&recovered_dir)
        .arg(recovered_dir.join("ErasedCall.java"))
        .output()
        .expect("JDK javac is available");
    assert!(
        !compile.status.success(),
        "the erased call is expected to stay uncompilable until its argument is re-typed"
    );
    let stderr = String::from_utf8_lossy(&compile.stderr);
    assert!(
        stderr.contains("compareTo"),
        "the failure is about the erased call spelling: {stderr}"
    );
}

#[test]
fn an_unresolvable_interface_keeps_the_physical_spelling_and_the_visible_bridge() {
    // The environment ships `Unresolved.class` only: the interface its `Signature` names has no
    // definition to prove the type arguments against, so the entry keeps the physical spelling
    // and the erased contract keeps its bridge visible (the raw header could not regenerate it).
    let jar = open(zip_of(&[(b"Unresolved.class", UNRESOLVED)]));
    let report = class_source_of(&jar, "Unresolved");
    assert_eq!(report.declaration.as_ref().unwrap().generic_refusal, None);
    assert_eq!(
        declaration_of(&report),
        "public class Unresolved extends java.lang.Object implements MissingApi"
    );
    assert_eq!(report.bridge_proofs.len(), 1);
    let proof = &report.bridge_proofs[0];
    assert!(!proof.admitted);
    assert!(!proof.projected);
    assert_eq!(
        proof.refusal.as_deref(),
        Some("a direct parent or interface needed for the erased method is unresolved")
    );
    assert!(
        report.text.contains("void put(java.lang.Object"),
        "the bridge stays visible"
    );
}

#[test]
fn an_interface_definition_that_contradicts_the_signature_refuses_the_header() {
    // The environment ships the `api2` definition of `ArityApi` (two type parameters) while the
    // class's own `Signature` states one argument: the definition resolves and contradicts the
    // claim, so the header is refused rather than spelled from a contradicted claim.
    let jar = open(zip_of(&[
        (b"Arity.class", ARITY),
        (b"ArityApi.class", ARITY_API),
    ]));
    let report = class_source_of(&jar, "Arity");
    assert_eq!(
        report
            .declaration
            .as_ref()
            .unwrap()
            .generic_refusal
            .as_deref(),
        Some(
            "unsupported (class_generic_source_unproved): an interface definition contradicts the class Signature's type arguments"
        )
    );
    assert_eq!(
        declaration_of(&report),
        "public class Arity extends java.lang.Object implements ArityApi",
        "the physical header stays the whole presentation"
    );
}

#[test]
fn a_type_use_annotated_implements_clause_keeps_the_raw_header_and_visible_bridge() {
    // The class-level type-use annotation keeps the generic header unpublished, exactly as it did
    // before an interface entry could enter the projection at all: the shared own-header gates
    // answer this shape with the raw header, not with a refusal — and the bridge admission then
    // keeps the erased contract's bridge visible, which is the invariant this cell pins.
    let jar = open(zip_of(&[
        (b"TypeUse.class", TYPE_USE),
        (b"Mark.class", MARK),
    ]));
    let report = class_source_of(&jar, "TypeUse");
    assert_eq!(report.declaration.as_ref().unwrap().generic_refusal, None);
    assert_eq!(
        declaration_of(&report),
        "public class TypeUse extends java.lang.Object implements java.lang.Comparable"
    );
    assert_eq!(report.bridge_proofs.len(), 1);
    let proof = &report.bridge_proofs[0];
    assert!(!proof.admitted);
    assert_eq!(
        proof.refusal.as_deref(),
        Some(
            "the erased contract comes from a generic interface the class header spells without its type arguments, so the source could not regenerate the bridge"
        )
    );
    assert!(report.text.contains("int compareTo(java.lang.Object"));
}

#[test]
fn the_parent_cells_whose_criteria_are_unmet_keep_their_refusals() {
    // `NestedExtends extends NB$Box<String>`: the `$` name alone is no longer a refusal, but
    // `NB$Box` has a non-`Object` superclass, so the one proved single-parameter parent
    // definition does not hold.
    let jar = open(zip_of(&[
        (b"NB.class", NB),
        (b"NB$Root.class", NB_ROOT),
        (b"NB$Box.class", NB_BOX),
        (b"NestedExtends.class", NESTED_EXTENDS),
    ]));
    let report = class_source_of(&jar, "NestedExtends");
    assert_eq!(
        report
            .declaration
            .as_ref()
            .unwrap()
            .generic_refusal
            .as_deref(),
        Some(
            "unsupported (class_generic_source_unproved): direct superclass does not resolve to one proved single-parameter parent definition"
        )
    );
    assert_eq!(
        declaration_of(&report),
        "public class NestedExtends extends NB$Box"
    );

    // `ArityExtends extends NB$Twin<String>`: the shipped definition declares two type
    // parameters, so the same refusal holds.
    let jar = open(zip_of(&[
        (b"NB$Twin.class", NB_TWIN),
        (b"ArityExtends.class", ARITY_EXTENDS),
    ]));
    let report = class_source_of(&jar, "ArityExtends");
    assert_eq!(
        report
            .declaration
            .as_ref()
            .unwrap()
            .generic_refusal
            .as_deref(),
        Some(
            "unsupported (class_generic_source_unproved): direct superclass does not resolve to one proved single-parameter parent definition"
        )
    );

    // `Multiseg extends MO<String>.Mid`: a multi-segment binary path whose middle segment carries
    // the type arguments; the one-segment direct-parent candidate never covers it, so the
    // existing refusal stands.
    let jar = open(zip_of(&[
        (b"MO.class", MO),
        (b"MO$Mid.class", MO_MID),
        (b"Multiseg.class", MULTISEG),
    ]));
    let report = class_source_of(&jar, "Multiseg");
    assert_eq!(
        report
            .declaration
            .as_ref()
            .unwrap()
            .generic_refusal
            .as_deref(),
        Some(
            "unsupported (class_generic_source_unproved): only a single direct Parent<String> superclass is supported for a class without type parameters"
        )
    );
    assert_eq!(
        declaration_of(&report),
        "public class Multiseg extends MO$Mid"
    );
}

#[test]
fn a_bare_dollar_named_parent_keeps_the_raw_header() {
    // The census's control cell: a `$`-named parent without type arguments has nothing to
    // project, so the raw physical header is the whole presentation.
    let jar = open(zip_of(&[
        (b"NB.class", NB),
        (b"NB$Root.class", NB_ROOT),
        (b"NB$Box.class", NB_BOX),
        (b"BareBox.class", BARE_BOX),
    ]));
    let report = class_source_of(&jar, "BareBox");
    assert_eq!(report.declaration.as_ref().unwrap().generic_refusal, None);
    assert_eq!(
        declaration_of(&report),
        "public class BareBox extends NB$Box"
    );
}
