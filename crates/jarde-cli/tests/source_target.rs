//! Source configuration is forwarded by each source-bearing CLI adapter.
use jarde::*;
use serde_json::{Value, json};
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    slice,
};
const BIN: &str = env!("CARGO_BIN_EXE_jarde-cli");
fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/p3-lambda-adaptation/v8/LambdaAdaptationProbe.class")
}
fn run(args: &[&str]) -> Output {
    Command::new(BIN).args(args).output().unwrap()
}
fn doc(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
fn strip_elapsed(v: &mut Value) {
    match v {
        Value::Object(m) => {
            m.remove("elapsed_millis");
            for x in m.values_mut() {
                strip_elapsed(x);
            }
        }
        Value::Array(xs) => {
            for x in xs {
                strip_elapsed(x);
            }
        }
        _ => {}
    }
}
fn same(mut a: Value, mut b: Value) {
    strip_elapsed(&mut a);
    strip_elapsed(&mut b);
    assert_eq!(a, b);
}
fn environment(snapshot: &ArtifactSnapshot) -> EnvironmentRequest {
    EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope: PhysicalScope::SnapshotAll,
        policy: EnvironmentPolicy::SingleClass,
        profile: RuntimeProfile {
            java_release: 8,
            multi_release: MultiReleasePolicy::Disabled,
            layout: LayoutMode::Generic,
        },
        loader: LoaderId("app".into()),
    }
}
#[test]
fn friendly_method_and_class_commands_match_library_for_both_targets() {
    let input = fixture();
    let path = input.to_str().unwrap();
    for release in [7, 8] {
        let target = SourceTarget::new(release).unwrap();
        let spelling = release.to_string();
        let mut budget = task_budget(&[]).unwrap();
        let engine = Engine::new();
        let snapshot = engine
            .open(ArtifactInput::Path(input.clone()), &mut budget)
            .unwrap();
        let request = ClassSourceRequest {
            source_target: target,
            class: ClassRef::Name {
                class: ClassNameQuery::internal("LambdaAdaptationProbe"),
            },
            environment: environment(&snapshot),
        };
        let outcome = engine
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut budget,
            )
            .unwrap();
        let cli = doc(&run(&[
            "class-source",
            "--input",
            path,
            "--class",
            "LambdaAdaptationProbe",
            "--policy",
            "single-class",
            "--release",
            "8",
            "--source-target",
            &spelling,
            "--evidence",
            "all",
            "--format",
            "json",
        ]));
        same(cli.clone(), serde_json::to_value(&outcome).unwrap());
        assert_eq!(cli["source_target"], json!({"java_release":release}));
        assert_eq!(cli["runtime_profile"]["java_release"], 8);
        let text = cli["text"].as_str().unwrap();
        assert_eq!(text.contains("->") || text.contains("::"), release == 8);
        let mut budget = task_budget(&[]).unwrap();
        let snapshot = engine
            .open(ArtifactInput::Path(input.clone()), &mut budget)
            .unwrap();
        let request = MethodOperationRequest {
            method: MethodRef::Name {
                class: ClassNameQuery::internal("LambdaAdaptationProbe"),
                name: JvmBytes(b"strings".to_vec()),
                descriptor: None,
            },
            environment: environment(&snapshot),
        };
        let outcome = engine
            .recover_target_with_evidence(
                slice::from_ref(&snapshot),
                target,
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut budget,
            )
            .unwrap();
        let cli = doc(&run(&[
            "recover",
            "--input",
            path,
            "--class-name",
            "LambdaAdaptationProbe",
            "--method-name",
            "strings",
            "--policy",
            "single-class",
            "--release",
            "8",
            "--source-target",
            &spelling,
            "--evidence",
            "all",
            "--format",
            "json",
        ]));
        same(cli.clone(), serde_json::to_value(outcome).unwrap());
        assert_eq!(
            cli["recovered"]["recovery"]["source_target"]["java_release"],
            release
        );
        assert_eq!(
            cli["recovered"]["recovery"]["runtime_profile"]["java_release"],
            8
        );
    }
}
#[test]
fn json_transport_declares_source_target_independently_of_runtime_and_keeps_pure_ir_usable() {
    let input = fixture();
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(ArtifactInput::Path(input.clone()), &mut budget)
        .unwrap();
    let request = MethodOperationRequest {
        method: MethodRef::Name {
            class: ClassNameQuery::internal("LambdaAdaptationProbe"),
            name: JvmBytes(b"strings".to_vec()),
            descriptor: None,
        },
        environment: environment(&snapshot),
    };
    let OperationOutcome::Performed(analyzed) = engine
        .analyze_target(slice::from_ref(&snapshot), &request, &mut budget)
        .unwrap()
    else {
        panic!("unique fixture")
    };
    for release in [7, 8] {
        let wire = json!({"input_path":input,"limits":task_limits(&[]).unwrap(),"operation":{"kind":"recover_method", "source_target":{"java_release":release}, "environment":request.environment.build(slice::from_ref(&snapshot)).unwrap(),"method":analyzed.method,"stages":MethodOperation::Recovery.stages()}});
        let mut child = Command::new(BIN)
            .args(["--request", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(wire.to_string().as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        let v = doc(&output);
        assert_eq!(
            v["result"]["report"]["source_target"]["java_release"],
            release
        );
        assert_eq!(v["result"]["report"]["runtime_profile"]["java_release"], 8);
        assert_eq!(
            v["result"]["analysis"]["method"],
            serde_json::to_value(&analyzed.method).unwrap()
        );
    }
}
#[test]
fn unregistered_targets_are_refused_before_input_io_and_pure_commands_reject_source_flags() {
    for command in ["class-source", "recover", "export"] {
        let mut args = vec![
            command,
            "--input",
            "/nonexistent/source-target-input",
            "--source-target",
            "9",
        ];
        if command == "class-source" {
            args.extend(["--class", "X"]);
        }
        if command == "recover" {
            args.extend(["--class-name", "X", "--method-name", "m"]);
        }
        if command == "export" {
            args.extend(["--output", "/nonexistent/source-target-output"]);
        }
        let out = run(&args);
        assert_eq!(out.status.code(), Some(2));
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("source_target_unregistered"),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    for command in ["class-view", "list-classes"] {
        let out = run(&[
            command,
            "--input",
            fixture().to_str().unwrap(),
            "--source-target",
            "7",
        ]);
        assert_eq!(out.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&out.stderr).contains("--source-target"));
    }
}
