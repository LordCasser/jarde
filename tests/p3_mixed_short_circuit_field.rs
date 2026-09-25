//! Complete-class Java 8 replay for the two mixed short-circuit field values.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const CLASS: &[u8] = include_bytes!(
    "fixtures/p3-conditional-values/mixed-short-circuit-field/MixedBooleanField.class"
);
const SOURCE: &str =
    include_str!("fixtures/p3-conditional-values/mixed-short-circuit-field/MixedBooleanField.java");
const RUNNER: &str =
    include_str!("fixtures/p3-conditional-values/mixed-short-circuit-field/Runner.java");
const EXPECTED: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-25/mixed-short-circuit-field/original-run.txt"
);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("jarde-mixed-field-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).expect("scratch directory");
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn recovered_source() -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
        .expect("frozen Java 8 class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("MixedBooleanField"),
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
    let OperationOutcome::Performed(source) = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("class source request succeeds")
    else {
        panic!("one standalone class must bind uniquely");
    };
    source
}

#[test]
fn mixed_field_methods_have_one_structured_write_and_complete_sources() {
    let source = recovered_source();
    for name in [b"andOr".as_slice(), b"orAnd".as_slice()] {
        let method = source
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == name)
            .expect("frozen method exists");
        let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
            panic!("mixed method did not recover: {:?}", method.outcome);
        };
        assert_eq!(report.quality, jarde_jvm::ir::Quality::Structured);
        assert_eq!(report.representation, Representation::Java);
        assert!(report.fallbacks.is_empty(), "{}", report.text);
        assert_eq!(report.text.matches("MixedBooleanField.result =").count(), 1);
        assert!(!report.text.contains("@bytecode"), "{}", report.text);
        for bci in [0, 1, 4, 7, 10, 13, 16, 17, 20, 21, 24] {
            assert!(
                !report.source_map.of_bci(bci).is_empty(),
                "BCI {bci} is unmapped: {}",
                report.text
            );
        }
    }
}

#[test]
#[ignore = "requires a JDK on PATH for javac --release 8 and java -Xverify:all"]
fn mixed_field_complete_class_matches_all_sixteen_jvm_paths() {
    let source = recovered_source();
    let scratch = Scratch::new();
    let original = scratch.0.join("original");
    let recovered = scratch.0.join("recovered");
    fs::create_dir(&original).expect("original directory");
    fs::create_dir(&recovered).expect("recovered directory");
    fs::write(original.join("MixedBooleanField.java"), SOURCE).expect("original source");
    fs::write(original.join("Runner.java"), RUNNER).expect("original runner");
    fs::write(recovered.join("MixedBooleanField.java"), source.text).expect("recovered source");
    fs::write(recovered.join("Runner.java"), RUNNER).expect("recovered runner");

    for directory in [&original, &recovered] {
        let compile = Command::new("javac")
            .args([
                "--release",
                "8",
                "-g:none",
                "MixedBooleanField.java",
                "Runner.java",
            ])
            .current_dir(directory)
            .output()
            .expect("javac is installed");
        assert!(
            compile.status.success(),
            "{}",
            String::from_utf8_lossy(&compile.stderr)
        );
    }
    assert_eq!(
        fs::read(original.join("MixedBooleanField.class")).expect("rebuilt original"),
        CLASS,
        "the committed class must stay byte-identical to its Java 8 source"
    );
    for directory in [&original, &recovered] {
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp", ".", "Runner"])
            .current_dir(directory)
            .output()
            .expect("java is installed");
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8(run.stdout).expect("UTF-8 trace"),
            EXPECTED
        );
    }
    assert_eq!(EXPECTED.lines().count(), 16);
}
