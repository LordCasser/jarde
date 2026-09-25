//! Java 8 regression for a postfix candidate whose handler starts inside the candidate chain.
//! The frozen class has a patched update() exception table; its only permanent binary is the
//! subject class under test.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-postfix-handler-boundary/v8/PostfixHandlerBoundary.class");
const EXPECTED_RUNTIME: &str = "escaped=IllegalArgumentException\ncalls=2\n";

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults to a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the verifier-valid Java 8 class opens as a standalone class")
}

fn class_source_of(snapshot: &ArtifactSnapshot) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("PostfixHandlerBoundary"),
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
        .expect("a valid class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one standalone class has {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one standalone class has an unfinished selection with {} candidates",
            candidates.candidates.len()
        ),
    }
}

fn update<'a>(report: &'a ClassSourceReport) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"update")
        .expect("update method exists")
}

fn temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after the epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "jarde-postfix-handler-boundary-{}-{nonce}",
        std::process::id()
    ))
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path = temp_dir();
        fs::create_dir(&path).expect("create temporary Java comparison directory");
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

fn execute_original() -> String {
    let scratch = Scratch::new();
    fs::write(scratch.path().join("PostfixHandlerBoundary.class"), FIXTURE)
        .expect("copy frozen Java 8 subject class");
    let output = Command::new("java")
        .args(["-Xverify:all", "-classpath"])
        .arg(scratch.path())
        .arg("PostfixHandlerBoundary")
        .output()
        .expect("java is available");
    assert!(
        output.status.success(),
        "verified original class execution failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the runner output is UTF-8")
}

#[test]
fn exception_handler_boundary_inside_postfix_chain_is_refused_with_real_bcis() {
    let runtime = execute_original();
    assert_eq!(
        runtime, EXPECTED_RUNTIME,
        "frozen JVM boundary behavior drifted"
    );

    let report = class_source_of(&open(FIXTURE));
    let member = update(&report);
    assert!(
        member.text.contains("@bytecode"),
        "the cross-handler candidate must remain quoted with its physical source:\n{}",
        member.text
    );
    assert!(
        !member.text.contains("return a()[i()]++"),
        "the candidate must not be folded into the handler body:\n{}",
        member.text
    );
    let ClassSourceOutcome::Recovered { report: body, .. } = &member.outcome else {
        panic!(
            "the refusal should retain a recovery report: {:?}",
            member.outcome
        );
    };
    assert_eq!(
        body.content,
        RecoveryContent::ExplanationOnly,
        "{}",
        member.text
    );
    assert!(
        body.fallbacks.contains(&"jre_guard_resource_init"),
        "the producer block should be refused by its actual guard: {:?}",
        body.fallbacks
    );
    assert!(
        body.fallbacks.contains(&"jre_region_uncovered_blocks"),
        "the exception handler block should be reported as uncovered: {:?}",
        body.fallbacks
    );
    for bci in [0, 3, 6, 7, 8, 9, 10, 11, 12, 13] {
        assert!(
            !body.source_map.of_bci(bci).is_empty(),
            "update dropped physical refusal-region BCI {bci}: {:?}",
            body.source_map.segments()
        );
    }
}
