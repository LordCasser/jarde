//! Negative shift boundaries whose verifier type is still `int`, plus an old local that must keep
//! its pre-overwrite value. Descriptor edits preserve the class-file layout and are applied only to
//! byte arrays held by this test.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const SLICE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/original/ShiftSlice.class"
);
const LOCAL_SOURCE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/shift-negative-boundaries/ShiftLocalBoundary.java"
);

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a JVM class-file variant opens")
}

fn class_source(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceReport {
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
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("class-source answers the single class")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one class produces one class-source answer: {other:?}"),
    }
}

fn method<'a>(
    report: &'a ClassSourceReport,
    name: &[u8],
    descriptor: &[u8],
) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == name && method.item.descriptor.raw().0 == descriptor
        })
        .unwrap_or_else(|| panic!("missing method {:?}{:?}", name, descriptor))
}

/// The existing descriptor constants have equal encoded lengths, so these edits preserve every
/// method and instruction BCI. They are not written to the repository as class files.
fn replace_one_constant(bytes: &[u8], before: &[u8], after: &[u8]) -> Vec<u8> {
    assert_eq!(before.len(), after.len());
    let mut patched = bytes.to_vec();
    let matches = patched
        .windows(before.len())
        .enumerate()
        .filter_map(|(at, window)| (window == before).then_some(at))
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "one frozen descriptor constant is patched"
    );
    patched[matches[0]..matches[0] + after.len()].copy_from_slice(after);
    patched
}

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("jarde-shift-negative-{nonce}"));
        fs::create_dir(&path).expect("create an isolated compiler output directory");
        Self(path)
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn overwritten_argument_does_not_replace_the_saved_shift_operand() {
    let temp = TemporaryDirectory::new();
    let source = temp.0.join("ShiftLocalBoundary.java");
    fs::write(&source, LOCAL_SOURCE).expect("write the source-only javac input");
    let javac = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(&temp.0)
        .arg(&source)
        .output()
        .expect("the Java 8 compiler is available for this verifier-boundary test");
    assert!(
        javac.status.success(),
        "javac rejected the source fixture:\n{}",
        String::from_utf8_lossy(&javac.stderr)
    );

    let bytes = fs::read(temp.0.join("ShiftLocalBoundary.class"))
        .expect("javac emitted the local-rewrite input");
    let snapshot = open(&bytes);
    let report = class_source(&snapshot, "ShiftLocalBoundary");
    let shifted = method(&report, b"savedBeforeOverwrite", b"(II)I");
    let ClassSourceOutcome::Recovered {
        report: recovery, ..
    } = &shifted.outcome
    else {
        panic!("the pre-overwrite local is recoverable: {}", shifted.text);
    };
    assert_eq!(
        recovery.representation,
        Representation::Java,
        "{}",
        shifted.text
    );
    assert!(
        shifted.text.contains("return local2 << arg1;")
            || shifted.text.contains("return local1 << arg1;"),
        "the shift reads the saved value after arg0 is overwritten:\n{}",
        shifted.text
    );
    assert!(
        !recovery.source_map.of_bci(8).is_empty(),
        "the shift remains anchored at its original BCI 8: {:?}\n{}",
        recovery.source_map.segments(),
        shifted.text
    );
}

#[test]
fn a_boolean_return_consumer_keeps_the_shift_refusal_at_its_real_bci() {
    // intLeft's JVM code is unchanged: iload_0; iload_1; ishl; ireturn. Replacing the method
    // result descriptor I with Z is verifier-valid because both use the JVM int verification type,
    // but Java cannot return the int shift value as boolean.
    let bytes = replace_one_constant(SLICE, b"(II)I", b"(II)Z");
    let snapshot = open(&bytes);
    let report = class_source(&snapshot, "ShiftSlice");
    let shifted = method(&report, b"intLeft", b"(II)Z");
    let ClassSourceOutcome::Recovered {
        report: recovery, ..
    } = &shifted.outcome
    else {
        panic!(
            "the rejected consumer still has a recovery report: {}",
            shifted.text
        );
    };
    assert_eq!(
        recovery.representation,
        Representation::Mixed,
        "{}",
        shifted.text
    );
    assert_eq!(recovery.quality, Quality::Fallback, "{}", shifted.text);
    assert!(shifted.text.contains("@bytecode 3"), "{}", shifted.text);
    assert!(
        !recovery.source_map.of_bci(3).is_empty(),
        "the refused consumer ireturn remains anchored at its actual BCI 3: {:?}\n{}",
        recovery.source_map.segments(),
        shifted.text
    );
    assert!(
        !shifted.text.contains("return arg0 << arg1;"),
        "the int-to-boolean consumer is not presented as Java:\n{}",
        shifted.text
    );
}
