//! A conditional return and fallthrough share the same physical cleanup copies.
//! Complete-source compilation and unchanged-driver execution are separate gates.
use jarde::*;
use std::slice;

const LEGS: &[(&str, &[u8])] = &[
    (
        "javac23 --release8",
        include_bytes!(
            "fixtures/recover-conditional-return-finally-copies/v8/LooplessFinally.class"
        ),
    ),
    (
        "javac8",
        include_bytes!(
            "fixtures/recover-conditional-return-finally-copies/v8-javac8/LooplessFinally.class"
        ),
    ),
];

fn source(bytes: &[u8]) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        source_target: jarde::SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal("LooplessFinally"),
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
    match engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one frozen physical class must be selected: {other:?}"),
    }
}

fn body(source: &ClassSourceReport) -> (&ClassSourceMethod, &RecoveryReport) {
    let method = source
        .methods
        .iter()
        .find(|m| m.item.name.raw().0 == b"loopless")
        .unwrap();
    assert_eq!(method.item.descriptor.raw().0, b"([I)I");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!(
            "the physical method must retain its report: {:?}",
            method.outcome
        )
    };
    (method, report)
}

#[test]
fn conditional_return_and_fallthrough_keep_one_structured_finally() {
    for (leg, bytes) in LEGS {
        let source = source(bytes);
        let (_, body) = body(&source);
        assert!(!body.text.contains("@bytecode"), "{leg}: {}", body.text);
        assert!(body.text.contains("try {"), "{leg}: {}", body.text);
        assert!(body.text.contains("finally {"), "{leg}: {}", body.text);
        assert_eq!(
            body.text.matches("println(\"f\")").count(),
            1,
            "{leg}: {}",
            body.text
        );
        assert!(body.text.contains("return -1;"), "{leg}: {}", body.text);
        let returned = body
            .text
            .find("return ")
            .expect("the conditional return is retained");
        let cleanup = body.text.find("finally {").unwrap();
        assert!(returned < cleanup, "{leg}: {}", body.text);
        assert_eq!(body.content, RecoveryContent::ContainsStatements);
        assert_eq!(source.methods.len(), 2, "all physical methods remain");
    }
}

#[test]
fn conditional_finally_keeps_all_three_copy_and_value_origins() {
    for (leg, bytes) in LEGS {
        let source = source(bytes);
        let (method, body) = body(&source);
        assert!(
            body.fields
                .iter()
                .any(|f| f.bci == 12 && f.access == "read" && f.presented),
            "{leg}: {}",
            body.text
        );
        for bci in [
            0, 1, 2, 5, 6, 7, 8, 9, 12, 15, 16, 19, 21, 24, 25, 26, 29, 31, 34, 37, 38, 41, 43, 46,
            47, 48, 49,
        ] {
            assert!(
                !body.source_map.direct_of_bci(bci).is_empty()
                    || !body.source_map.derived_of_bci(bci).is_empty(),
                "{leg}: missing physical {bci}: {:?}",
                body.source_map.segments()
            );
        }
        assert!(
            body.source_map
                .segments()
                .iter()
                .flat_map(|s| std::iter::once(s.origin().primary()).chain(s.origin().derived()))
                .all(|origin| origin.method() == Some(&method.item.identity))
        );
    }
}

const DRIVER: &str = include_str!(
    "fixtures/recover-conditional-return-finally-copies/LooplessFinallyContractProbe.java"
);
const EXPECTED: &str =
    include_str!("fixtures/recover-conditional-return-finally-copies/contract.stdout");

struct Scratch(std::path::PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        // Failed artifacts can be exported explicitly; routine scratch is always removed.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn command(command: &mut std::process::Command) -> std::process::Output {
    let output = command.output().expect("installed JDK command starts");
    assert!(
        output.status.success(),
        "{command:?}: status={:?}\nstdout={}\nstderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
#[ignore = "needs installed javac --release 8 and java; compiles whole untouched class"]
fn whole_loopless_finally_preserves_cleanup_completion_contract() {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jarde-cleanup-contract-{}-{stamp}",
        std::process::id()
    )));
    std::fs::create_dir(&scratch.0).unwrap();
    let export = std::env::var_os("JARDE_CLEANUP_CONTRACT_EVIDENCE").map(std::path::PathBuf::from);
    if let Some(path) = &export {
        // Never overwrite a prior evidence run.
        std::fs::create_dir(path)
            .expect("evidence destination must not already exist; parent must exist");
    }
    for (index, (leg, bytes)) in LEGS.iter().enumerate() {
        let report = source(bytes);
        assert_eq!(
            report.methods.len(),
            2,
            "{leg}: retain every physical method"
        );
        assert!(!report.text.contains("@bytecode"), "{leg}: {}", report.text);
        for (side, recovered) in [("original", false), ("recovered", true)] {
            let dir = export
                .as_ref()
                .unwrap_or(&scratch.0)
                .join(format!("leg{index}-{side}"));
            std::fs::create_dir(&dir).unwrap();
            let empty = dir.join("empty-sourcepath");
            std::fs::create_dir(&empty).unwrap();
            let driver = dir.join("LooplessFinallyContractProbe.java");
            std::fs::write(&driver, DRIVER).unwrap();
            let mut javac = std::process::Command::new("javac");
            javac
                .args(["--release", "8", "-g:none", "-proc:none"])
                .arg("-classpath")
                .arg(&dir)
                .arg("-sourcepath")
                .arg(&empty)
                .arg("-d")
                .arg(&dir);
            if recovered {
                let target = dir.join("LooplessFinally.java");
                std::fs::write(&target, &report.text).unwrap();
                javac.arg(target);
            } else {
                std::fs::write(dir.join("LooplessFinally.class"), bytes).unwrap();
            }
            javac.arg(&driver);
            // Preserve input and complete generated text before invoking javac, including on
            // failure when the caller chose an evidence directory.
            std::fs::write(dir.join("input.class"), bytes).unwrap();
            std::fs::write(dir.join("Engine.java.txt"), &report.text).unwrap();
            let compile = command(&mut javac);
            let run = command(
                std::process::Command::new("java")
                    .arg("-Xverify:all")
                    .arg("-cp")
                    .arg(&dir)
                    .arg("LooplessFinallyContractProbe"),
            );
            if let Some(path) = &export {
                let evidence = path.join(format!("leg{index}-{side}"));
                std::fs::write(evidence.join("input.class"), bytes).unwrap();
                std::fs::write(evidence.join("Engine.java"), &report.text).unwrap();
                std::fs::write(evidence.join("Probe.java"), DRIVER).unwrap();
                std::fs::write(evidence.join("compile.stdout"), &compile.stdout).unwrap();
                std::fs::write(evidence.join("compile.stderr"), &compile.stderr).unwrap();
                std::fs::write(evidence.join("run.stdout"), &run.stdout).unwrap();
                std::fs::write(evidence.join("run.stderr"), &run.stderr).unwrap();
            }
            assert!(run.stderr.is_empty(), "{leg}/{side}: unexpected stderr");
            assert_eq!(run.stdout, EXPECTED.as_bytes(), "{leg}/{side}");
        }
    }
}
